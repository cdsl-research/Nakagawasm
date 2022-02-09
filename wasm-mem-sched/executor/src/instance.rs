use crate::{config::ModuleKind, register::InstanceRegister, Module};
use futures::future::join_all;
use std::{collections::HashMap, io, process::Stdio};
use tokio::{
    fs,
    process::{Child, Command},
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tracing::{error, info};
use uuid::Uuid;

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

/// Instanceを実行するためのマニュフェスト的なもの
#[derive(Debug, Clone)]
pub struct InstanceSpec {
    pub uid: Uuid,
    pub module: Module,
    pub args: Vec<String>,
}

impl InstanceSpec {
    pub fn new(module: Module, args: Vec<String>) -> Self {
        Self {
            module,
            uid: Uuid::new_v4(),
            args,
        }
    }

    /// selfにしたがって，サブプロセスの実行を開始する
    pub fn spawn(&self) -> io::Result<Child> {
        let path = self.module.path.as_path();

        match self.module.kind {
            ModuleKind::Wasm32Wasi => Command::new("wasmtime")
                .kill_on_drop(true)
                .arg(path)
                .args(&self.args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn(),
            ModuleKind::Native => Command::new(path)
                .args(&self.args)
                .kill_on_drop(true)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn(),
        }
    }
}

/// `Instance` is the managed subprocess by this application.
#[derive(Debug)]
pub struct Instance {
    spec: InstanceSpec,
    handler: Option<JoinHandle<anyhow::Result<()>>>,
    stop_op_sender: Option<oneshot::Sender<()>>,
}

impl Instance {
    #[tracing::instrument]
    pub async fn spawn(
        spec: InstanceSpec,
        status_sender: mpsc::Sender<(Uuid, InstanceStatus)>,
    ) -> anyhow::Result<Self> {
        // ここでredisに登録

        let mut child = spec.spawn()?;

        // ここでredisのstatusをアップデート
        status_sender
            .send((spec.uid, InstanceStatus::Running))
            .await?;

        let (tx, stop_op_reciever) = oneshot::channel();

        let handler = tokio::spawn(async move {
            tokio::select! {
                status = child.wait() => {
                    status?;
                    status_sender.send((spec.uid, InstanceStatus::Quit)).await?;
                    Ok(())
                },
                _ = stop_op_reciever => {
                    info!("got stop message ({})", spec.uid);
                    child.kill().await?;
                    child.wait().await?;
                    status_sender.send((spec.uid, InstanceStatus::Quit)).await?;
                    Ok(())
                }
            }
        });

        Ok(Self {
            spec,
            handler: Some(handler),
            stop_op_sender: Some(tx),
        })
    }

    pub async fn shutdown(mut self) -> anyhow::Result<()> {
        if let Some(sender) = self.stop_op_sender.take() {
            sender.send(()).map_err(|_| {
                anyhow::anyhow!(
                    "the stop message will never be received. (UID: {})",
                    self.spec.uid
                )
            })?;
        };

        if let Some(handler) = self.handler.take() {
            handler.await??;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pid(pub u32);

impl Pid {
    pub async fn get_uss(&self) -> anyhow::Result<u64> {
        let s = fs::read_to_string(format!("/proc/{}/smaps", self.0)).await?;
        Ok(regex!(r"Private_((Clean)|(Dirty)):\s*(\d+)\skB")
            .captures_iter(&s)
            .map(|cap| cap.get(4).unwrap())
            .map(|m| m.as_str().parse::<u64>().unwrap())
            .sum())
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    strum::EnumString,
    strum::Display,
    strum::EnumIter,
    strum::IntoStaticStr,
)]
pub enum InstanceStatus {
    #[strum(serialize = "starting")]
    Starting,
    #[strum(serialize = "running")]
    Running,
    #[strum(serialize = "quit")]
    Quit,
}

#[derive(Debug)]
pub struct InstanceManager {
    instances: HashMap<Uuid, Instance>,
    register: Box<dyn InstanceRegister>,
    cmd_reciever: mpsc::Receiver<InstanceOps>,
}

pub enum InstanceOps {
    Start(InstanceSpec),
    Stop(Uuid),
}

impl InstanceManager {
    pub fn new(
        register: Box<dyn InstanceRegister>,
        cmd_reciever: mpsc::Receiver<InstanceOps>,
    ) -> Self {
        Self {
            cmd_reciever,
            instances: HashMap::new(),
            register,
        }
    }

    pub fn spawn(mut self) -> JoinHandle<anyhow::Result<()>> {
        let (status_sender, mut status_reciever) = mpsc::channel::<(Uuid, InstanceStatus)>(100);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    cmd = self.cmd_reciever.recv() => {
                        let cmd = cmd.unwrap();
                        match cmd {
                            InstanceOps::Start(spec) => {
                                self.start_instance(spec, status_sender.clone()).await?;
                            }
                            InstanceOps::Stop(uid) => {
                                self.stop_instance(uid).await?;
                            }
                        }
                    },
                    status = status_reciever.recv() => {
                        let (uid, status) = status.unwrap();
                        info!("status({:?}) update recieved {:?}", status,uid);
                        if status == InstanceStatus::Quit {
                            self.register.expire(&uid).await?;
                        }
                        self.register.update(&uid, status).await?;
                    },
                    _ = tokio::signal::ctrl_c() => {
                        self.shutdown_all().await?;
                        break;
                    }
                }
            }

            Ok(())
        })
    }

    async fn start_instance(
        &mut self,
        spec: InstanceSpec,
        status_sender: mpsc::Sender<(Uuid, InstanceStatus)>,
    ) -> anyhow::Result<()> {
        self.register.register(&spec).await?;
        let uid = spec.uid;
        let instance = Instance::spawn(spec, status_sender).await?;
        let _ = self.instances.insert(uid, instance);
        Ok(())
    }

    async fn stop_instance(&mut self, uid: Uuid) -> anyhow::Result<()> {
        if let Some(instance) = self.instances.remove(&uid) {
            instance.shutdown().await?;
        }

        Ok(())
    }

    #[tracing::instrument(name = "mgr::shutdown_all")]
    async fn shutdown_all(mut self) -> anyhow::Result<()> {
        let shutdown_tasks_iter = self
            .instances
            .drain()
            .map(|(_uid, instance)| instance.shutdown());

        join_all(shutdown_tasks_iter).await.iter().for_each(|r| {
            if let Err(e) = r {
                error!("{}", e);
            }
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[tokio::test]
    async fn test_instance_cycle() {
        let module = Module::new(ModuleKind::Native, Path::new("/usr/bin/sleep"))
            .await
            .unwrap();
        let spec = InstanceSpec::new(module, vec!["infinity".into()]);
        let (status_sender, mut status_reciever) = mpsc::channel(100);

        let instance = Instance::spawn(spec.clone(), status_sender).await.unwrap();

        let (uid, status) = status_reciever.recv().await.unwrap();
        assert_eq!(uid, spec.uid);
        assert!(matches!(status, InstanceStatus::Running));

        instance.shutdown().await.unwrap();
        let (uid, status) = status_reciever.recv().await.unwrap();
        assert_eq!(uid, spec.uid);
        assert!(matches!(status, InstanceStatus::Quit));
    }
}
