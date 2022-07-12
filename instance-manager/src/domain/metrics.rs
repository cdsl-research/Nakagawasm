use std::time::Duration;

use async_trait::async_trait;
use tokio::{
    process::Child,
    task::JoinHandle,
    time::{sleep_until, Instant},
};

use crate::repository::CsvInstanceMemoryRepository;

use super::{HostMachine, InstanceId, WorkerId};

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

#[async_trait]
pub trait MemoryUsage {
    async fn memory_usage(&self) -> anyhow::Result<u32>;
}

#[async_trait]
impl MemoryUsage for Child {
    async fn memory_usage(&self) -> anyhow::Result<u32> {
        let pid = self
            .id()
            .ok_or_else(|| anyhow::anyhow!("the child has been polled to completion"))?;
        let s = tokio::fs::read_to_string(format!("/proc/{}/smaps", pid)).await?;
        Ok(regex!(r"Private_((Clean)|(Dirty)):\s*(\d+)\skB")
            .captures_iter(&s)
            .map(|cap| cap.get(4).unwrap())
            .map(|m| m.as_str().parse::<u32>().unwrap())
            .sum())
    }
}

#[async_trait]
impl MemoryUsage for HostMachine {
    async fn memory_usage(&self) -> anyhow::Result<u32> {
        let s = tokio::fs::read_to_string("/proc/meminfo").await.unwrap();
        let mem_free: u32 = regex!(r"MemFree:\s*(\d+)\s*kB")
            .captures_iter(&s)
            .map(|cap| cap.get(1).unwrap().as_str().parse::<u32>().unwrap())
            .sum();
        let mem_total: u32 = regex!(r"MemTotal:\s*(\d+)\s*kB")
            .captures_iter(&s)
            .map(|cap| cap.get(1).unwrap().as_str().parse::<u32>().unwrap())
            .sum();
        Ok(mem_total - mem_free)
    }
}

#[derive(Debug, Clone)]
pub struct InstanceMemoryMetrics {
    pub timestamp: time::PrimitiveDateTime,
    pub worker_id: WorkerId,
    pub instance_id: InstanceId,
    pub memory_usage: u32,
}

#[derive(Debug)]
pub struct CsvMemoryMetricsCollector {
    repo: CsvInstanceMemoryRepository,
    worker_id: WorkerId,
    instance_id: InstanceId,
}

impl CsvMemoryMetricsCollector {
    pub fn new(
        repo: CsvInstanceMemoryRepository,
        worker_id: WorkerId,
        instance_id: InstanceId,
    ) -> Self {
        Self { repo, worker_id, instance_id }
    }

    // TODO: pidを取得する方法・（いずれは更新する必要もあるかも）
    pub fn spawn(mut self) -> JoinHandle<anyhow::Result<()>> {
        tokio::spawn(async {
            loop {
                let instant = Instant::now();

                sleep_until(instant + Duration::from_secs(10)).await;
            }
        })
    }
}
