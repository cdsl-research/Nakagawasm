use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "module")]
    pub entries: Vec<ConfigEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub kind: ModuleKind,
    pub path: PathBuf,
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ModuleKind {
    #[serde(rename = "wasm32-wasi")]
    Wasm32Wasi,
    #[serde(rename = "native")]
    Native,
}
