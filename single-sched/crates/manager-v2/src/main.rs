use std::{
    io::Result,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use crate::config::Config;
use tokio::{
    fs::File,
    io::AsyncWriteExt,
    process::{Child, Command},
    task::JoinHandle,
    time::{sleep_until, Instant},
};
use tonic::async_trait;
use tracing::info;

mod config;

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

async fn make_result_dir(outdir: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
    let mut path = PathBuf::from(outdir.as_ref());
    let now = chrono::Local::now();
    path.push(now.to_rfc3339());
    tokio::fs::create_dir_all(path.as_path()).await?;
    Ok(path)
}

#[async_trait]
trait Uss {
    async fn uss(&self) -> anyhow::Result<u32>;
}

#[async_trait]
impl Uss for Child {
    async fn uss(&self) -> anyhow::Result<u32> {
        let pid = self.id().ok_or_else(|| anyhow::anyhow!(""))?;
        let s = tokio::fs::read_to_string(format!("/proc/{}/smaps", pid)).await?;
        Ok(regex!(r"Private_((Clean)|(Dirty)):\s*(\d+)\skB")
            .captures_iter(&s)
            .map(|cap| cap.get(4).unwrap())
            .map(|m| m.as_str().parse::<u32>().unwrap())
            .sum())
    }
}

#[derive(Debug)]
struct Target {
    child: Option<Child>,
    config: Config,
}

impl Target {
    pub fn new(config: Config) -> Self {
        Self {
            child: None,
            config,
        }
    }

    #[tracing::instrument]
    pub async fn start(&mut self) -> anyhow::Result<()> {
        match &self.child {
            Some(_) => {
                info!("child is already running.");
            }
            None => self.child = Some(self.spawn_child()?),
        };
        Ok(())
    }

    #[tracing::instrument]
    pub async fn stop(&mut self) -> anyhow::Result<()> {
        if let Some(mut child) = self.child.take() {
            child.start_kill()?;
            let output = child.wait_with_output().await?;
            info!("child stopped: {:?}", output);
        };
        Ok(())
    }

    pub async fn restart(&mut self) -> anyhow::Result<()> {
        self.stop().await?;
        self.start().await?;
        Ok(())
    }

    fn spawn_child(&self) -> Result<Child> {
        Command::new("wasmtime")
            .arg("run")
            .arg(&self.config.wasi.path)
            .arg("--mapdir")
            .args(
                self.config
                    .wasi
                    .mapdirs
                    .iter()
                    .map(|mapdir| format!("{}::{}", mapdir.guest, mapdir.host)),
            )
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
    }
}

#[async_trait]
impl Uss for Target {
    async fn uss(&self) -> anyhow::Result<u32> {
        match &self.child {
            Some(child) => child.uss().await,
            None => Ok(0),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let config = Config::from_toml_file("config.toml").await?;

    let path = {
        let mut path = make_result_dir(&config.outdir).await?;
        path.push("mem.csv");
        path
    };
    let mut file = File::create(path).await?;

    let mut target = Target::new(config);

    let handler: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
        target.start().await?;

        loop {
            let start = Instant::now();

            tokio::select! {
                _ = sleep_until(start + Duration::from_secs(10)) => {
                    let uss = target.uss().await?;
                    let now = chrono::Local::now().to_rfc3339();
                    file.write_all(format!("{},{}\n", now, uss).as_bytes())
                        .await?;

                    if let Some(th) = target.config.threshold {
                        if uss > th {
                            target.restart().await?;
                        }
                    }
                },
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("received ctrl-c event");
                    target.stop().await?;
                    break;
                }
            }
        }
        Ok(())
    });

    handler.await??;

    Ok(())
}
