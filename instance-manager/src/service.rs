use std::process::Stdio;

use tokio::process::Command;

use crate::domain::{Instance, InstanceId, InstanceManifest, Worker, WorkerId, WorkerManifest};

pub async fn instance_create_service(man: &InstanceManifest) -> Instance {
    let id = InstanceId::generate();

    let child = Command::new("wasmedge")
        .args(&man.args)
        .env("PORT", man.port.to_string().as_str())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();

    Instance::new(id, child)
}

pub async fn worker_create_service(man: &WorkerManifest) -> Worker {
    let id = WorkerId::generate();
    let instance = instance_create_service(&man.instance_manifest).await;
    let instance_handler = instance.spawn();

    Worker::new(id, instance_handler)
}
