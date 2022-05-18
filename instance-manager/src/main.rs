use domain::{InstanceManifest, WorkerManifest};
use tokio::signal::ctrl_c;
use tracing::Level;

mod domain;
mod repository;
mod service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt().with_max_level(Level::DEBUG).init();

    let instance_manifest = InstanceManifest {
        args: [
            "--dir",
            ".:../server-contents-setup/static",
            "--enable-all",
            "../wasmedge-app/target/wasm32-wasi/release/wasmedge-app.wasm",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>(),
        port: 1234,
    };

    let worker_man = WorkerManifest { instance_manifest };
    let worker = service::worker_create_service(&worker_man).await;
    let handler = worker.spawn();

    ctrl_c().await.ok();

    handler.stop();
    handler.wait().await??;

    Ok(())
}
