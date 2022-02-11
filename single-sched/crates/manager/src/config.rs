use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub executor: Executor,
    pub wasi: Wasi,
    pub threshold: Option<u64>,
    pub outdir: String,
}

#[derive(Deserialize)]
pub struct Executor {
    pub path: String
}

#[derive(Deserialize)]
pub struct Wasi {
    pub path: String
}
