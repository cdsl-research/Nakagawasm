use crate::handler::Executable;
use chrono::{DateTime, Local};
use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::RwLock;
use uuid::Uuid;
pub struct PodManager {
    pods: Vec<Pod>,
}

// #[derive(Debug)]
pub struct Pod {
    pub exe: Box<dyn Executable>,
    pub child: Child,
    pub uuid: Uuid,
    pub created_at: DateTime<Local>,
}

impl Pod {
    pub fn new(exe: Box<dyn Executable>) -> io::Result<Self> {
        let child = exe.exec(&[])?;
        let uuid = Uuid::new_v4();
        let created_at = chrono::Local::now();

        Ok(Self {
            exe,
            child,
            uuid,
            created_at,
        })
    }
}
