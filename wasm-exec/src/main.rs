use std::{process::Stdio, str::from_utf8};
use tokio::{
    fs::File,
    io::AsyncWriteExt,
    process::{Child, Command},
    time::{sleep_until, Duration, Instant},
};
use tracing::info;
use tracing_subscriber;

async fn get_child_rss(child: &Child) -> anyhow::Result<u64> {
    let ps = Command::new("ps")
        .arg("-p")
        .arg(child.id().expect("cannot get pid").to_string())
        .arg("o")
        .arg("rss")
        .arg("--no-headers")
        .stdout(Stdio::piped())
        .spawn()?;

    let rss = ps.wait_with_output().await?;
    let rss = from_utf8(&rss.stdout)?.trim().parse::<u64>()?;

    Ok(rss)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // let mut child = Command::new("wasmtime")
    //     .arg("../wasm-test-app/target/wasm32-wasi/debug/c.wasm")
    //     .stdout(Stdio::piped())
    //     .stderr(Stdio::piped())
    //     .spawn()
    //     .expect("failed to spawn");

    let mut child = Command::new("../wasm-test-app/target/debug/c")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn child process");

    info!("child process spawned!");

    let mut file = File::create("cnative.txt").await?;

    for _ in 0..200 {
        let instant = Instant::now();
        let tick = sleep_until(instant + Duration::from_millis(1000));
        let rss = get_child_rss(&child).await?;
        info!(rss);
        file.write(format!("{},{}\n", chrono::Local::now().to_string(), rss).as_bytes())
            .await?;
        tick.await;
    }

    child.start_kill()?;
    let out = child.wait_with_output().await?;
    info!("{:?}", out);

    Ok(())
}
