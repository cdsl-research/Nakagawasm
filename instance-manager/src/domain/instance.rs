use std::process::Output;

use tokio::{process::Child, signal::ctrl_c};
use ulid::Ulid;

use super::Handler;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceId(Ulid);

impl InstanceId {
    pub fn new(id: Ulid) -> Self {
        Self(id)
    }

    pub fn generate() -> Self {
        Self::new(Ulid::new())
    }
}

#[derive(Debug)]
pub struct Instance {
    pub id: InstanceId,
    child: Child,
}

impl Instance {
    pub fn new(id: InstanceId, child: Child) -> Self {
        Self { id, child }
    }

    pub fn spawn(mut self) -> Handler<Self, anyhow::Result<Output>> {
        let handle = tokio::spawn(async move {
            tracing::debug!("Instance {:?} spawn!", self.id);

            ctrl_c().await.ok();
            self.child.start_kill()?;
            let output = self.child.wait_with_output().await?;
            Ok(output)
        });

        Handler::new(handle)
    }
}

#[derive(Debug)]
pub struct InstanceManifest {
    pub args: Vec<String>,
    pub port: u16,
}
