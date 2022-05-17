use ulid::Ulid;

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

pub trait Instance: Eq {
    fn id(&self) -> InstanceId;
    fn create() -> Self;
}

#[derive(Debug)]
pub struct WasmEdgeInstance {
    id: InstanceId,
}

impl WasmEdgeInstance {
    pub fn new(id: InstanceId) -> Self {
        Self { id }
    }
}

impl Eq for WasmEdgeInstance {}

impl PartialEq for WasmEdgeInstance {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Instance for WasmEdgeInstance {
    fn id(&self) -> InstanceId {
        self.id
    }

    fn create() -> Self {
        Self {
            id: InstanceId::generate(),
        }
    }
}
