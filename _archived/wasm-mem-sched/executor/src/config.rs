use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

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
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum ModuleKind {
    #[serde(rename = "wasm32-wasi")]
    Wasm32Wasi,
    #[serde(rename = "native")]
    Native,
}

impl Config {
    pub async fn from_path(path: &Path) -> anyhow::Result<Self> {
        let text = fs::read_to_string(&path).await?;
        let conf = toml::from_str::<Config>(&text)?;
        Ok(conf)
    }
}
