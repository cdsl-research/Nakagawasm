use std::{
    io,
    path::{Path, PathBuf},
    process::Stdio,
};

use tokio::process::{Child, Command};
use uuid::Uuid;

pub trait Executable {
    fn exec(&self, args: &[String]) -> io::Result<Child>;
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct WasmModule(PathBuf);

impl WasmModule {
    pub fn new(path: &Path) -> Self {
        Self(path.into())
    }
}

impl Executable for WasmModule {
    fn exec(&self, args: &[String]) -> io::Result<Child> {
        Command::new("wasmtime")
            .arg(&self.0)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct NativeBinary(PathBuf);

impl NativeBinary {
    pub fn new(path: &Path) -> Self {
        Self(path.into())
    }
}

impl Executable for NativeBinary {
    fn exec(&self, args: &[String]) -> io::Result<Child> {
        Command::new(&self.0)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

#[derive(Debug)]
pub struct Pod<E: Executable> {
    pub exe: E,
    pub child: Child,
    pub uuid: Uuid,
}

impl<E: Executable> Pod<E> {
    pub fn new(exe: E) -> io::Result<Self> {
        let child = exe.exec(&[])?;
        let uuid = Uuid::new_v4();

        Ok(Self { exe, child, uuid })
    }
}
