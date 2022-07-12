use std::{process::Output, fmt};

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

impl fmt::Display for InstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_instance_id_to_string() {
        let id = InstanceId::generate();
        assert_eq!(id.to_string().len(), 26);
    }
}
