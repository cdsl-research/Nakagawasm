use std::path::PathBuf;
use tokio::task::JoinHandle;
use wasi_experimental_http_wasmtime::HttpCtx;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

#[derive(Debug)]
pub struct WasiRuntime {
    path: PathBuf,
    stop_handler: Option<StopHandler>,
}

#[derive(Debug)]
pub struct StopHandler {
    ir: InterruptHandle,
    handle: JoinHandle<()>,
}

impl StopHandler {
    pub fn new(ir: InterruptHandle, handle: JoinHandle<()>) -> Self {
        Self { ir, handle }
    }

    pub async fn stop(self) -> anyhow::Result<()> {
        self.ir.interrupt();
        self.handle.abort();
        self.handle.await?;
        Ok(())
    }
}

impl WasiRuntime {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            stop_handler: None,
        }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        if self.stop_handler.is_none() {
            return Ok(());
        }

        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
        let http = HttpCtx::new(None, Some(32))?;
        http.add_to_linker(&mut linker)?;

        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_args()?
            .build();
        let mut store = Store::new(&engine, wasi);

        let module = Module::from_file(&engine, self.path.as_path())?;
        let func = linker
            .module(&mut store, "", &module)?
            .get_default(&mut store, "")?
            .typed::<(), (), _>(&store)?;

        let ir_hdl = store.interrupt_handle()?;

        let handle = tokio::task::spawn_blocking(move || {
            let span = tracing::info_span!("wasmtime_module_run");
            let _enter = span.enter();

            match func.call(&mut store, ()) {
                Ok(_) => {}
                Err(e) => {
                    panic!("{}", e);
                }
            }
        });

        self.stop_handler = Some(StopHandler::new(ir_hdl, handle));

        Ok(())
    }

    pub async fn stop(&mut self) -> anyhow::Result<()> {
        if let Some(stop_handler) = self.stop_handler.take() {
            stop_handler.stop().await?;
        }

        Ok(())
    }
}
