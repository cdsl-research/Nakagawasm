use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;
use tokio::sync::mpsc::Sender;

use crate::domain::InstanceMemoryMetrics;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceMemoryMetricsData {
    pub timestamp: PrimitiveDateTime,
    pub worker_id: String,
    pub instance_id: String,
    pub memory_usage: u32,
}

impl From<InstanceMemoryMetrics> for InstanceMemoryMetricsData {
    fn from(m: InstanceMemoryMetrics) -> Self {
        InstanceMemoryMetricsData {
            timestamp: m.timestamp,
            worker_id: m.worker_id.to_string(),
            instance_id: m.instance_id.to_string(),
            memory_usage: m.memory_usage,
        }
    }
}

// #[async_trait]
// pub trait MemoryRepository {
//     type M: Serialize + Send + Sync + 'static;
//     async fn store(&self, metrics: impl Into<Self::M>) -> anyhow::Result<()>;
// }

#[derive(Debug)]
pub struct CsvInstanceMemoryRepository {
    sender: Sender<InstanceMemoryMetricsData>,
}

impl CsvInstanceMemoryRepository {
    pub fn new(sender: Sender<InstanceMemoryMetricsData>) -> Self {
        Self { sender }
    }
}

impl CsvInstanceMemoryRepository {
    pub async fn store(&self, metrics: impl Into<InstanceMemoryMetricsData>) -> anyhow::Result<()> {
        self.sender.send(metrics.into()).await?;
        Ok(())
    }
}
