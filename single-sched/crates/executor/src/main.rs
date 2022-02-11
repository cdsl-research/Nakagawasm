use std::{path::PathBuf, sync::{Arc}};
use cmd::{
    cmd_server::{Cmd as CmdExt, CmdServer},
    RestartResponse,
};
use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

mod cmd {
    tonic::include_proto!("cmd");
}
mod wasi;

#[derive(Debug)]
pub struct Cmd {
    runtime: Arc<Mutex<wasi::WasiRuntime>>,
}

#[tonic::async_trait]
impl CmdExt for Cmd {
    #[tracing::instrument(name = "restart")]
    async fn restart(&self, _request: Request<()>) -> Result<Response<RestartResponse>, Status> {
        info!("Recieved restart request.");
        let restarted = chrono::Local::now().to_rfc3339();
        let mut r = self.runtime.lock().await;
        r.stop().await.unwrap();
        r.start().await.unwrap();
        // self.runtime.get_mut().stop().await.unwrap();
        // self.runtime.get_mut().start().await.unwrap();
        Ok(Response::new(RestartResponse { restarted }))
    }
}

impl Cmd {
    pub fn new(runtime: wasi::WasiRuntime) -> Self {
        Self {
            runtime: Arc::new(Mutex::new(runtime)),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_ansi(false).init();

    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        panic!("Argument is missing")
    }

    let runtime = wasi::WasiRuntime::new(PathBuf::from(&args[1]));

    let cmd = Cmd::new(runtime);

    let addr = "[::1]:50051".parse()?;

    tracing::info!("start service");

    let server =     Server::builder()
    .add_service(CmdServer::new(cmd))
    .serve(addr);

    tokio::select!{
        server = server => {
            tracing::info!("stopped service, {:?}", server?);
        },
        _ = tokio::signal::ctrl_c() => {
            info!("got ctrl_c");
        }
    }

    Ok(())
}
