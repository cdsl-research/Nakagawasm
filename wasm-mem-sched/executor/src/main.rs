use crate::config::{Config, ModuleKind};
use clap::Parser;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    process::{ExitStatus, Stdio},
    sync::Arc,
};
use tokio::{
    process::Command,
    signal::ctrl_c,
    sync::{mpsc, oneshot, Mutex},
    task::JoinHandle,
};
use tracing::info;
use uuid::Uuid;

mod config;

#[derive(Debug, Clone)]
pub struct Module {
    pub kind: ModuleKind,
    pub path: PathBuf,
    pub digest: String,
}

impl Module {
    pub async fn new(kind: ModuleKind, path: &Path) -> io::Result<Self> {
        Ok(Self {
            kind,
            path: path.to_owned(),
            digest: Self::digestize(path).await?,
        })
    }

    async fn digestize(path: &Path) -> io::Result<String> {
        let data = tokio::fs::read(path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        Ok(format!("{:0x}", hasher.finalize()))
    }
}

#[derive(Debug, Clone)]
pub struct InstanceSpec {
    pub module: Arc<Module>,
    pub status: InstanceStatus,
    pub uid: Uuid,
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
    pub spec: InstanceSpec,
    pub handler: JoinHandle<anyhow::Result<()>>,
    pub stop_cmd_sender: Option<oneshot::Sender<()>>,
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
                InstanceStatus::Running(child.id().map(|pid| Pid(pid)).ok_or_else(|| {
                    anyhow::anyhow!("The instance {} is already finished.", spec.uid)
                })?),
            ))
            .await?;

        let (tx, rx) = oneshot::channel();

        let handler = tokio::spawn(async move {
            tokio::select! {
                status = child.wait() => {
                    status_sender.send((spec.uid, InstanceStatus::Quit(status?))).await?;
                    Ok(())
                },
                _ = rx => {
                    child.kill().await?;
                    status_sender.send((spec.uid, InstanceStatus::Quit(child.wait().await?))).await?;
                    Ok(())
                }
            }
        });

        Ok(Self {
            spec,
            handler,
            stop_cmd_sender: Some(tx),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pid(pub u32);

#[derive(Debug, Clone)]
pub enum InstanceStatus {
    Starting,
    Running(Pid),
    Quit(ExitStatus),
}

#[derive(Debug, Parser)]
#[clap(about, version, author)]
pub struct Cli {
    #[clap(short, long, default_value_t = String::from("run.toml"))]
    pub config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let conf = Config::from_path(Path::new(&cli.config)).await?;

    let instance_collector = Arc::new(Mutex::new(HashMap::<Uuid, Instance>::new()));
    let (tx, mut rx) = mpsc::channel::<(Uuid, InstanceStatus)>(100);

    let mgr_handle = {
        let instance_collector = Arc::clone(&instance_collector);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    status = rx.recv() => {
                        let (uid, status) = status.unwrap();
                        info!("status({:?}) update recieved {:?}", status,uid);
                        instance_collector.lock().await.get_mut(&uid).unwrap().spec.status = status;
                    },
                    _ = ctrl_c() => {
                        info!("Got ctrl_c. instances: {:#?}", instance_collector.lock().await);
                        break;
                    }
                }
            }
        })
    };

    for c in conf.entries.iter() {
        let module = Arc::new(Module::new(c.kind, &c.path).await?);
        for _ in 0..c.count {
            let spec = InstanceSpec::new(module.clone(), Uuid::new_v4());
            let tx = tx.clone();
            let instance = Instance::spawn(spec, tx).await?;

            instance_collector
                .lock()
                .await
                .insert(instance.spec.uid, instance);
        }
    }

    mgr_handle.await?;

    Ok(())
}
