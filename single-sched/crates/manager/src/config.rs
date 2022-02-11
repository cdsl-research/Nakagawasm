use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub executor: Executor,
    pub wasi: Wasi,
    pub threshold: Option<u64>,
    pub outdir: String,
}

#[derive(Debug, Deserialize)]
pub struct Executor {
    pub path: String
}

#[derive(Debug, Deserialize)]
pub struct Wasi {
    pub path: String
}
