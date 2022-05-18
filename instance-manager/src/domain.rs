use std::marker::PhantomData;
use tokio::task::{JoinError, JoinHandle};

pub use instance::{InstanceId, Instance, InstanceManifest};
pub use metrics::MemoryUsage;
pub use worker::{Worker, WorkerId, WorkerManifest};

mod instance;
mod metrics;
mod worker;

#[derive(Debug)]
pub struct Handler<T, R>
where
    T: Send + Sync,
    R: Send + Sync,
{
    handle: JoinHandle<R>,
    _marker: PhantomData<T>,
}

impl<T, R> Handler<T, R>
where
    T: Send + Sync,
    R: Send + Sync,
{
    pub fn new(handle: JoinHandle<R>) -> Self {
        Self {
            handle,
            _marker: PhantomData,
        }
    }

    pub async fn wait(self) -> Result<R, JoinError> {
        self.handle.await
    }

    pub fn stop(&self) {
        self.handle.abort();
    }
}
