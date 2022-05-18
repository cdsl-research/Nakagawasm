use std::process::Output;

use tokio::signal::ctrl_c;
use ulid::Ulid;

use super::{Handler, Instance, InstanceManifest};

#[derive(Debug)]
pub struct Worker {
    pub id: WorkerId,
    pub instance_handler: Handler<Instance, anyhow::Result<Output>>,
    // pub metrics_collect_handler: Handler<>
}

impl Worker {
    pub fn new(id: WorkerId, instance_handler: Handler<Instance, anyhow::Result<Output>>) -> Self {
        Self {
            id,
            instance_handler,
        }
    }

    pub fn spawn(self) -> Handler<Self, anyhow::Result<Output>> {
        let handle = tokio::spawn(async move {
            tracing::debug!("Worker {:?} spawn!", self.id);
            ctrl_c().await.ok();

            self.instance_handler.stop();
            let result = self.instance_handler.wait().await??;
            Ok(result)
        });

        Handler::new(handle)
    }
}

pub struct WorkerManifest {
    pub instance_manifest: InstanceManifest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkerId(Ulid);

impl WorkerId {
    pub fn new(ulid: Ulid) -> Self {
        Self(ulid)
    }

    pub fn generate() -> Self {
        Self::new(Ulid::new())
    }
}
