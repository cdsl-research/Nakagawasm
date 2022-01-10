use crate::handler::Executable;
use std::io;
use tokio::process::Child;
use uuid::Uuid;

// #[derive(Debug)]
pub struct Pod {
    pub exe: Box<dyn Executable>,
    pub child: Child,
    pub uuid: Uuid,
}

impl Pod {
    pub fn new(exe: Box<dyn Executable>) -> io::Result<Self> {
        let child = exe.exec(&[])?;
        let uuid = Uuid::new_v4();

        Ok(Self { exe, child, uuid })
    }
}
