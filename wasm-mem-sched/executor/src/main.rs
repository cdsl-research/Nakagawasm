use std::path::{Path, PathBuf};

use clap::Parser;
use tokio::fs;

mod cli;
mod config;
mod handler;
mod pod;
pub struct Module {
    pub kind: config::ModuleKind,
    pub path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();

    let conf: config::Config = {
        let path = Path::new(&cli.config);
        let conf = fs::read_to_string(path).await?;
        toml::from_str(&conf)?
    };

    eprintln!("{:?}", conf);

    Ok(())
}
