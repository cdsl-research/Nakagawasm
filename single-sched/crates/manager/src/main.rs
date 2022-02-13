use cmd::cmd_client::CmdClient;
use std::{
    io,
    path::{Path, PathBuf},
    // process::Stdio,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::{Child, Command},
    sync::mpsc,
    task::JoinHandle,
    time::{sleep, sleep_until, Instant},
};
pub mod cmd {
    tonic::include_proto!("cmd");
}
mod config;

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

fn spawn_executor(config: &config::Config) -> io::Result<Child> {
    Command::new(&config.executor.path)
        .arg(&config.wasi.path)
        // .stdout(Stdio::piped())
        // .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
}
pub struct Pid(pub u32);

impl Pid {
    pub async fn uss(&self) -> anyhow::Result<u64> {
        let s = tokio::fs::read_to_string(format!("/proc/{}/smaps", self.0)).await?;
        Ok(regex!(r"Private_((Clean)|(Dirty)):\s*(\d+)\skB")
            .captures_iter(&s)
            .map(|cap| cap.get(4).unwrap())
            .map(|m| m.as_str().parse::<u64>().unwrap())
            .sum())
    }
}

async fn make_result_dir(outdir: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
    let mut path = PathBuf::from(outdir.as_ref());
    let now = chrono::Local::now();
    path.push(now.to_rfc3339());
    tokio::fs::create_dir_all(path.as_path()).await?;
    Ok(path)
}

async fn _spawn_logger(child: &mut Child, write_dir: &Path) -> JoinHandle<anyhow::Result<()>> {
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let mut write_dir = PathBuf::from(write_dir);

    let handler: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
        let span = tracing::info_span!("log_writer");
        let _enter = span.enter();

        tracing::info!("spawn log writer");

        write_dir.push("log.log");
        let mut file = tokio::fs::File::create(write_dir.as_path()).await?;

        let mut buf1 = String::new();
        let mut buf2 = String::new();
        loop {
            tokio::select! {
               res = stdout.read_to_string(&mut buf1) => {
                    tracing::info!("{:?}", res);
                    file.write_all(buf1.as_bytes()).await?;
                    buf1.clear();
                },
                res = stderr.read_to_string(&mut buf2) => {
                    tracing::info!("{:?}", res);
                    file.write_all(buf2.as_bytes()).await?;
                    buf2.clear();
               },
            }
        }
    });

    handler
}

async fn spawn_uss_sender(pid: Pid, sender: mpsc::Sender<u64>) -> JoinHandle<anyhow::Result<()>> {
    tokio::spawn(async move {
        tracing::info!("spawn uss sender");
        loop {
            let instant = Instant::now();
            let tick = sleep_until(instant + Duration::from_secs(10));
            let uss = pid.uss().await?;
            sender.send(uss).await?;
            tick.await;
        }
    })
}

async fn spawn_sched(
    mut child: Child,
    th: Option<u64>,
    write_dir: &Path,
) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    tracing::info!("called spwn_sched");

    let pid = Pid(child.id().expect("child is exited"));
    let (tx, mut rx) = mpsc::channel(100);

    let mut path = PathBuf::from(write_dir);
    path.push("mem.csv");
    let mut file = tokio::fs::File::create(path.as_path()).await?;

    let _handle: JoinHandle<Result<(), anyhow::Error>> = spawn_uss_sender(pid, tx).await;

    let mut client = loop {
        match CmdClient::connect("http://[::1]:50051").await {
            Ok(c) => break c,
            Err(e) => {
                tracing::info!("{:?}", e);
                tracing::info!("3 secs sleep");
                sleep(Duration::from_secs(3)).await;
            }
        }
    };

    let task: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
        tracing::info!("spawn sched");

        loop {
            tokio::select! {
                recv = rx.recv() => {
                    let uss = recv.unwrap();
                    let now = chrono::Local::now().to_rfc3339();
                    file.write_all(format!("{},{}\n", now, uss).as_bytes()).await?;
                    if let Some(th) = th {
                        if uss > th {
                            let restarted = client.restart(()).await?;
                            tracing::debug!("restarted: {:?}", restarted);
                        }
                    }
                },
                _ = child.wait() => {
                    break;
                },
            }
        }
        Ok(())
    });

    Ok(task)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_ansi(false).init();

    let s = tokio::fs::read_to_string("config.toml").await?;
    let config: config::Config = toml::from_str(&s)?;

    tracing::info!("{:?}", config);

    let child = spawn_executor(&config)?;

    let write_dir = make_result_dir(&config.outdir).await?;

    // let log_writer = spawn_logger(&mut child, write_dir.as_path()).await;
    let sched = spawn_sched(child, config.threshold, write_dir.as_path()).await?;

    tokio::signal::ctrl_c().await?;

    tracing::info!("got ctrl_c");

    // log_writer.abort();
    sched.abort();

    let _ = sched.await;
    // let _ = log_writer.await;

    Ok(())
}
