use crate::config::{Config, ModuleKind};
use clap::Parser;
use futures::future::join_all;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    process::{ExitStatus, Stdio},
    sync::Arc,
};
use tokio::{
    fs,
    process::Command,
    signal::ctrl_c,
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tracing::{error, info};
use uuid::Uuid;

mod config;

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

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
    pub handler: Option<JoinHandle<anyhow::Result<()>>>,
    pub stop_cmd_sender: Option<oneshot::Sender<()>>,
}

impl Instance {
    #[tracing::instrument(name = "Instance::spawn")]
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
            stop_cmd_sender: Some(tx),
        })
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

#[derive(Debug, Clone)]
pub enum InstanceStatus {
    Starting,
    Running(Pid),
    Quit(ExitStatus),
}

#[derive(Debug)]
pub struct MemoryCollector {
    pub handler: Option<JoinHandle<anyhow::Result<()>>>,
}

impl MemoryCollector {
    pub fn new() -> Self {
        Self { handler: None }
    }

    pub async fn spawn(&mut self) -> anyhow::Result<()> {
        self.handler = Some(tokio::spawn(async move { Ok(()) }));
        Ok(())
    }
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

    let mut mem_collector = MemoryCollector::new();
    mem_collector.spawn().await?;

    let mgr_handler: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
        let mut instance_collector = HashMap::<Uuid, Instance>::new();
        let (status_sender, mut status_reciever) = mpsc::channel::<(Uuid, InstanceStatus)>(100);

        for c in conf.entries.iter() {
            let module = Arc::new(Module::new(c.kind, &c.path).await?);
            for _ in 0..c.count {
                let spec = InstanceSpec::new(Arc::clone(&module), Uuid::new_v4());
                let tx = status_sender.clone();
                let instance = Instance::spawn(spec, tx).await?;

                instance_collector.insert(instance.spec.uid, instance);
            }
        }

        loop {
            tokio::select! {
                status = status_reciever.recv() => {
                    let (uid, status) = status.unwrap();
                    info!("status({:?}) update recieved {:?}", status,uid);
                    instance_collector.get_mut(&uid).unwrap().spec.status = status;
                },
                _ = ctrl_c() => {
                    info!("Got ctrl+c.");
                    for (uid, instance) in instance_collector.iter_mut() {
                        let sender = instance.stop_cmd_sender.take();
                        if let Some(sender) = sender {
                            if let Err(e) = sender.send(()).map_err(|_| {
                                anyhow::anyhow!("the stop message will never be received. (UID: {})", uid)
                            }){
                                error!("{:?}", e);
                            };
                        }
                    }
                    join_all(instance_collector.iter_mut().filter_map(|(_uid, i)| i.handler.take())).await
                        .iter()
                        .for_each(|r| match r {
                            Ok(Ok(_)) => {}
                            Ok(Err(e)) => {
                                error!("{:?}", e);
                            }
                            Err(e) => {
                                error!("{:?}", e);
                            }
                        });
                    break;
                },
            }
        }
        Ok(())
    });

    mgr_handler.await??;
    if let Some(handler) = mem_collector.handler {
        handler.abort();
        match handler.await {
            Ok(_) => info!("A memory collector was shutdown."),
            Err(e) => error!("{e}"),
        }
    }

    Ok(())
}
