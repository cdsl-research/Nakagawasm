use std::{io, path::{PathBuf, Path}, process::Stdio};

use tokio::process::{Child, Command};

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
    exe: E,
    child: Child,
}
