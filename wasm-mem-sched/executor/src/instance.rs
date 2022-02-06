use crate::{config::ModuleKind, Module};
use futures::future::join_all;
use std::{
    collections::HashMap,
    process::{ExitStatus, Stdio},
    sync::Arc,
};
use tokio::{
    fs,
    process::Command,
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

#[derive(Debug, Clone)]
pub struct InstanceSpec {
    module: Arc<Module>,
    status: InstanceStatus,
    uid: Uuid,
}

impl InstanceSpec {
    pub fn new(module: Arc<Module>, uid: Uuid) -> Self {
        Self {
            module,
            status: InstanceStatus::Starting,
            uid,
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
        let path = spec.module.path.as_path();
        let mut child = match spec.module.kind {
            ModuleKind::Wasm32Wasi => Command::new("wasmtime")
                .kill_on_drop(true)
                .arg(path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?,
            ModuleKind::Native => Command::new(path)
                .kill_on_drop(true)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?,
        };

        status_sender
            .send((
                spec.uid,
                InstanceStatus::Running(child.id().map(Pid).ok_or_else(|| {
                    anyhow::anyhow!("The instance {} is already finished.", spec.uid)
                })?),
            ))
            .await?;

        let (tx, stop_op_reciever) = oneshot::channel();

        let handler = tokio::spawn(async move {
            tokio::select! {
                status = child.wait() => {
                    status_sender.send((spec.uid, InstanceStatus::Quit(status?))).await?;
                    Ok(())
                },
                _ = stop_op_reciever => {
                    info!("got stop message ({})", spec.uid);
                    child.kill().await?;
                    status_sender.send((spec.uid, InstanceStatus::Quit(child.wait().await?))).await?;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceStatus {
    Starting,
    Running(Pid),
    Quit(ExitStatus),
}

pub struct InstanceManager {
    instances: HashMap<Uuid, Instance>,
    status_reciever: mpsc::Receiver<(Uuid, InstanceStatus)>,
}

impl InstanceManager {
    pub fn new(status_reciever: mpsc::Receiver<(Uuid, InstanceStatus)>) -> Self {
        Self {
            instances: HashMap::new(),
            status_reciever,
        }
    }

    pub fn spawn(mut self) -> JoinHandle<anyhow::Result<()>> {
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    status = self.status_reciever.recv() => {
                        let (uid, status) = status.unwrap();
                        info!("status({:?}) update recieved {:?}", status,uid);
                        self.instances.get_mut(&uid).unwrap().spec.status = status;
                    },
                    _ = tokio::signal::ctrl_c() => {
                        self.shutdown().await?;
                        break;
                    }
                }
            }

            Ok(())
        })
    }

    async fn shutdown(mut self) -> anyhow::Result<()> {
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
        let module = Arc::new(
            Module::new(ModuleKind::Native, Path::new("/usr/bin/sleep"))
                .await
                .unwrap(),
        );

        let (status_sender, mut status_reciever) = mpsc::channel(100);

        let spec = InstanceSpec::new(module, Uuid::new_v4());
        let instance = Instance::spawn(spec.clone(), status_sender).await.unwrap();

        let (uid, status) = status_reciever.recv().await.unwrap();
        assert_eq!(uid, spec.uid);
        assert!(matches!(status, InstanceStatus::Running(_)));

        instance.shutdown().await.unwrap();
        let (uid, status) = status_reciever.recv().await.unwrap();
        assert_eq!(uid, spec.uid);
        assert!(matches!(status, InstanceStatus::Quit(_)));
    }
}
