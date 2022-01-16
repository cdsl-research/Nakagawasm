use crate::config::{Config, ModuleKind};
use clap::Parser;
use std::{
    io::{self, Error},
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
};
use tokio::{process::{Child, Command}, sync::{RwLock, mpsc}};
use uuid::Uuid;

mod config;

#[derive(Debug)]
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
        let cmd = Command::new("sha256sum")
            .arg(path)
            .stdout(Stdio::piped())
            .spawn()?;

        let mut digest = cmd.wait_with_output().await?.stdout;
        // sha256's hex string is always 64 of length
        digest.truncate(64);
        digest.shrink_to_fit();

        // sha256sum's stdout is always ascii str
        Ok(String::from_utf8(digest).unwrap())
    }
}

#[derive(Debug, Clone)]
pub struct InstanceSpec {
    pub module: Arc<Module>,
    pub uid: Uuid,
}

impl InstanceSpec {
    pub fn new(module: Arc<Module>, uid: Uuid) -> Self {
        Self { module, uid }
    }
}

#[derive(Debug)]
pub struct Instance {
    pub spec: InstanceSpec,
    pub child: Child,
}

impl Instance {
    pub fn new(spec: InstanceSpec) -> io::Result<Self> {
        let path = spec.module.path.as_path();
        let child = match spec.module.kind {
            ModuleKind::Wasm32Wasi => Command::new("wasmtime")
                .arg(path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?,
            ModuleKind::Native => Command::new(path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?,
        };

        Ok(Self { spec, child })
    }
}

#[derive(Debug)]
pub enum Status {
    ExitSuccess(String),
    ExecFailed(Error),
}

#[derive(Debug, Parser)]
#[clap(about, version, author)]
pub struct Cli {
    #[clap(short, long, default_value_t = String::from("run.toml"))]
    pub config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let conf = Config::from_path(Path::new(&cli.config)).await?;

    eprintln!("{:?}", conf);

    let instane_collector = Arc::new(RwLock::new(Vec::<InstanceSpec>::new()));

    let (tx, mut rx) = mpsc::channel(100);

    for c in conf.entries.iter() {
        let module = Arc::new(Module::new(c.kind, &c.path).await?);
        for _ in 0..c.count {
            let module = module.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                let spec = InstanceSpec::new(module, Uuid::new_v4());
                let instance = match Instance::new(spec) {
                    Ok(instance) => instance,
                    Err(e) => {
                        tx.send(Status::ExecFailed(e)).await.unwrap();
                        return;
                    },
                };
                let output = instance.child.wait_with_output().await;
            });
        }
    }

    // pod manager spawn
    //  pod spawn
    //    instance spawn
    // metrics collector spawn
    // estimater spawn

    Ok(())
}
