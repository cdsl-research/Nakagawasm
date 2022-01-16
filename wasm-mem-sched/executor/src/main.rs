use std::{
    io,
    path::{Path, PathBuf},
    process::Stdio,
};

use clap::Parser;
use tokio::{fs, process::Command};

mod config;
mod handler;
mod pod;

#[derive(Debug)]
pub struct Module {
    pub kind: config::ModuleKind,
    pub path: PathBuf,
    pub digest: String,
}

impl Module {
    pub async fn new(kind: config::ModuleKind, path: &Path) -> io::Result<Self> {
        Ok(Self {
            kind,
            path: path.to_owned(),
            digest: Self::sha256(path).await?,
        })
    }

    async fn sha256(path: &Path) -> io::Result<String> {
        let cmd = Command::new("sha256sum")
            .arg(path)
            .stdout(Stdio::piped())
            .spawn()?;

        let mut digest = cmd.wait_with_output().await?.stdout;
        // The length of sha256's hex string is 64
        digest.truncate(64);
        digest.shrink_to_fit();

        // sha256sum's stdout is always ascii str
        Ok(String::from_utf8(digest).unwrap())
    }
}

#[derive(Debug, Parser)]
#[clap(about, version, author)]
pub struct Cli {
    #[clap(short, long, default_value_t = String::from("run.toml"))]
    pub config: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let conf: config::Config = {
        let path = Path::new(&cli.config);
        let conf = fs::read_to_string(path).await?;
        toml::from_str(&conf)?
    };

    eprintln!("{:?}", conf);

    for c in conf.entries.iter() {
        let module = Module::new(c.kind, &c.path).await?;
        eprintln!("{:?}, cap={}", module, module.digest.capacity());
    }

    // pod manager spawn
    //  pod spawn
    //    instance spawn
    // metrics collector spawn
    // estimater spawn

    Ok(())
}
