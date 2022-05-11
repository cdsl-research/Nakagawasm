use futures::future::join_all;
use std::time::Duration;
use tokio::{process::Command, time::sleep};
use tracing::{error, info};

const N: usize = 50;
const EXEC_SECS: u64 = 600;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    info!("started {} tasks on {} secs", N, EXEC_SECS);

    let tasks = (0..N).map(|n| task(n));
    join_all(tasks).await.iter().for_each(|x| {
        if let Err(e) = x {
            error!("{:?}", e);
        }
    });
}

#[tracing::instrument]
async fn task(n: usize) -> anyhow::Result<()> {
    info!("started task{}", n);

    let mut child = Command::new("./manager-v2").kill_on_drop(true).spawn()?;

    sleep(Duration::from_secs(EXEC_SECS)).await;

    child.start_kill()?;
    child.wait().await?;

    Ok(())
}
