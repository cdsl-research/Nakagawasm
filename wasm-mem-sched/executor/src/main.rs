use crate::config::{Config, ModuleKind};
use clap::Parser;
use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
};
use tokio::{
    process::Command,
    signal::ctrl_c,
    sync::{
        mpsc::{self, Sender},
        Mutex,
    },
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
            digest: Self::sha256(path).await?,
        })
    }

    async fn sha256(path: &Path) -> io::Result<String> {
        let mut digest = Command::new("sha256sum")
            .arg(path)
            .stdout(Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?
            .stdout;

        // sha256's hex string is always 64 of length
        digest.truncate(64);
        digest.shrink_to_fit();

        Ok(String::from_utf8(digest).expect("sha256sum's stdout is always ascii str"))
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
}

impl Instance {
    #[tracing::instrument]
    pub async fn spawn(
        spec: InstanceSpec,
        status_sender: Sender<(Uuid, InstanceStatus)>,
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
                InstanceStatus::Running(child.id().map(|pid| Pid(pid)).expect("")),
            ))
            .await?;

        let handler = tokio::spawn(async move {
            child.wait().await?;
            status_sender.send((spec.uid, InstanceStatus::Quit)).await?;
            Ok(())
        });

        Ok(Self { spec, handler })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pid(pub u32);

#[derive(Debug, Clone)]
pub enum InstanceStatus {
    Starting,
    Running(Pid),
    Quit,
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

    {
        let instance_collector = Arc::clone(&instance_collector);
        let _mgr_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    status = rx.recv() => {
                        let (uid, status) = status.unwrap();
                        info!("status({:?}) update recieved {:?}", status,uid);
                        instance_collector.lock().await.get_mut(&uid).unwrap().spec.status = status;
                    },
                };
            }
        });
    }

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

    ctrl_c().await?;

    Ok(())
}
