use chrono::Utc;
use cookie::{Cookie, CookieJar};
use hyper::http::HeaderValue;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use proxy::{Proxy, ProxyData};
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{convert::Infallible, net::SocketAddr};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{info, debug};
use wasmtime_wasi::WasiCtxBuilder;
use wit_bindgen_wasmtime::wasmtime::{Config, Engine, Instance, Linker, Module, Store};

wit_bindgen_wasmtime::import!("../wits/proxy.wit");

struct Context {
    pub wasi: wasmtime_wasi::WasiCtx,
    pub proxy: proxy::ProxyData,
}

struct Wasm {
    store: Store<Context>,
    instance: Instance,
    pub proxy: Proxy<Context>,
}

impl Wasm {
    pub fn new() -> anyhow::Result<Self> {
        let mut config = Config::default();
        config
            .cache_config_load_default()?
            .wasm_backtrace_details(wit_bindgen_wasmtime::wasmtime::WasmBacktraceDetails::Disable)
            .wasm_multi_memory(true)
            .wasm_bulk_memory(true)
            .wasm_reference_types(true);
        let engine = Engine::new(&config).unwrap();

        let module = Module::from_file(&engine, "target/wasm32-wasi/release/guest.wasm").unwrap();

        let mut linker = Linker::new(&engine);
        let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build();
        let mut store = Store::new(
            &engine,
            Context {
                wasi: wasi_ctx,
                proxy: ProxyData::default(),
            },
        );

        wasmtime_wasi::add_to_linker(&mut linker, |cx: &mut Context| &mut cx.wasi).unwrap();
        Proxy::add_to_linker(&mut linker, |cx: &mut Context| &mut cx.proxy).unwrap();

        let instance = linker.instantiate(&mut store, &module).unwrap();
        let proxy = Proxy::new(&mut store, &instance, |cx| &mut cx.proxy).unwrap();

        Ok(Self {
            store,
            instance,
            proxy,
        })
    }

    pub fn onhttp(&mut self, path: &str, auth: &str, method: &str) -> anyhow::Result<String> {
        let result = self
            .proxy
            .onhttp(&mut self.store, path, auth, method)
            .unwrap();
        Ok(result)
    }

    pub fn ontick(&mut self) -> anyhow::Result<()> {
        todo!()
    }

    /// Returns the byte length of the "memory" of this wasm instance.
    pub fn memory_size(&mut self) -> anyhow::Result<usize> {
        let size = self
            .instance
            .get_memory(&mut self.store, "memory")
            .unwrap()
            .data_size(&mut self.store);

        Ok(size)
    }
}

fn make_response(_req: Request<Body>, res: String) -> Result<Response<Body>, anyhow::Error> {
    let mut response = Response::new(Body::from(res.to_string()));
    response.headers_mut().append(
        "Set-Cookie",
        HeaderValue::from_bytes(format!("Authorization={}", res).as_bytes()).unwrap(),
    );
    Ok(response)
}

async fn handle(
    client_ip: IpAddr,
    req: Request<Body>,
    wasm: Arc<Mutex<Wasm>>,
) -> Result<Response<Body>, anyhow::Error> {
    info!(req=?req);
    // info!(path=?req.uri().path(), method=?req.method());
    if req.uri().path().starts_with("/target/first") {
        // will forward requests to port 13901
        match hyper_reverse_proxy::call(client_ip, "http://127.0.0.1:13901", req).await {
            Ok(response) => Ok(response),
            Err(_error) => Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap()),
        }
    } else if req.uri().path().starts_with("/target/second") {
        // will forward requests to port 13902
        match hyper_reverse_proxy::call(client_ip, "http://127.0.0.1:13902", req).await {
            Ok(response) => Ok(response),
            Err(_error) => Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap()),
        }
    } else {
        let mut jar = CookieJar::new();

        let auth = {
            if let Some(cookie) = req.headers().get("Cookie") {
                let cookies = std::str::from_utf8(cookie.as_bytes())?;
                for cookie_str in cookies.split(';').map(|s| s.trim()) {
                    if let Ok(cookie) = Cookie::parse(cookie_str) {
                        jar.add_original(cookie.into_owned());
                    }
                }
                jar.get("Authorization").map_or("", |c| c.value())
            } else {
                ""
            }
        };

        let result = wasm
            .lock()
            .unwrap()
            .onhttp(req.uri().path(), auth, req.method().as_str())?;
        make_response(req, result)
    }
}

#[tokio::main]
async fn main() {
    let timestamp = Utc::now().to_rfc3339();
    let mut memory_log = File::create(format!("memory_{}.log", timestamp))
        .await
        .unwrap();
    let event_log = File::create(format!("event_{}.log", timestamp))
        .await
        .unwrap();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(Mutex::new(event_log.into_std().await))
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    let wasm = Wasm::new().unwrap();
    let wasm = Arc::new(Mutex::new(wasm));

    let wasm_ = wasm.clone();
    tokio::spawn(async move {
        loop {
            // memory_log.w
            tokio::time::sleep(Duration::new(10, 0)).await;
            let memsize = wasm_.lock().unwrap().memory_size().unwrap();
            memory_log
                .write_all(format!("{},{}\n", Utc::now().to_rfc3339(), memsize).as_bytes())
                .await
                .unwrap();
        }
    });

    let bind_addr = "127.0.0.1:8000";
    let addr: SocketAddr = bind_addr.parse().expect("Could not parse ip:port.");

    let make_svc = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr().ip();
        let wasm = wasm.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                handle(remote_addr, req, wasm.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    debug!("Running server on {:?}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
