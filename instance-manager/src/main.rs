use std::{fs::File, io::BufWriter};

use domain::{InstanceManifest, WorkerManifest};
use driver::CsvExportDriver;
use repository::CsvInstanceMemoryRepository;
use tokio::{signal::ctrl_c, sync::mpsc};
use tracing::Level;

mod domain;
mod driver;
mod repository;
mod service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let (im_send, im_recv) = mpsc::channel(16);
    let im_writer = BufWriter::new(File::create("instance_memory.csv")?);
    let im_exporter = CsvExportDriver::new(im_writer, im_recv).spawn();


    let repo = CsvInstanceMemoryRepository::new(im_send);

    let hm_writer = BufWriter::new(File::create("host_memory.csv")?);
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
