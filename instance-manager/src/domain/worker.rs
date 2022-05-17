use ulid::Ulid;

#[derive(Debug)]
pub struct WorkerFactory {}

impl WorkerFactory {}

#[derive(Debug)]
pub struct Worker {
    pub id: WorkerId,
    // instance_handler
    // metrics_collect_handler
}

impl Worker {}

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
