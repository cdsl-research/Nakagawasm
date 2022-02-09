use crate::config::{Config, ModuleKind};
use clap::Parser;
use instance::{InstanceManager, InstanceStatus};
use register::RedisRegister;
use sha2::{Digest, Sha256};
use std::{
    io,
    path::{Path, PathBuf},
};
use tokio::{sync::mpsc, task::JoinHandle};
use uuid::Uuid;

mod config;
mod instance;
mod register;

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

#[derive(Debug, Parser)]
#[clap(about, version, author)]
pub struct Cli {
    #[clap(short, long, default_value_t = String::from("run.toml"))]
    pub config: String,
}

pub struct MetricsCollector {}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {}
    }

    pub fn spawn(self) -> JoinHandle<anyhow::Result<()>> {
        tokio::spawn(async move { Ok(()) })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let conf = Config::from_path(Path::new(&cli.config)).await?;

    let (cmd_sender, cmd_reciever) = mpsc::channel(100);

    let instance_manager = InstanceManager::new(
        Box::new(RedisRegister::new("redis://0.0.0.0:6379")?),
        cmd_reciever,
    );
    let mgr_handler = instance_manager.spawn();

    mgr_handler.await??;
    Ok(())
}
