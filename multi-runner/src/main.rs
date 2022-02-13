use futures::future::join_all;
use std::{
    process::{Output, Stdio},
    time::Duration,
};
use tokio::{process::Command, time::sleep};
use tracing::{error, info};

const N: usize = 50;
const EXEC_SECS: u64 = 600;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    info!("started {} tasks on {} secs", N, EXEC_SECS);

    let tasks = (0..N).map(|_| task());
    join_all(tasks).await.iter().for_each(|x| {
        if let Err(e) = x {
            error!("{:?}", e);
        }
    });
}

#[tracing::instrument]
async fn task() -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new("manager")
        .kill_on_drop(true)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("process spawn failed");

    sleep(Duration::from_secs(EXEC_SECS)).await;

    let _ = child.kill().await;
    let Output {
        status,
        stdout,
        stderr,
    } = child.wait_with_output().await?;
    info!(
        "status: {} stdout: {}, stderr: {}",
        status,
        String::from_utf8(stdout)?,
        String::from_utf8(stderr)?
    );

    Ok(())
}
