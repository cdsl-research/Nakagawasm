use std::path::{Path, PathBuf};

use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::fs;

mod cli;
mod handler;
mod pod;

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

pub struct Module {
    pub kind: ModuleKind,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();

    let conf: Config = {
        let path = Path::new(&cli.config);
        let conf = fs::read_to_string(path).await?;
        toml::from_str(&conf)?
    };

    eprintln!("{:?}", conf);

    Ok(())
}
