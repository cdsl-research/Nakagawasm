wit_bindgen_wasmtime::import!("../wits/proxy.wit");
use proxy::{Proxy, ProxyData};
use wit_bindgen_wasmtime::wasmtime::{Config, Engine, Instance, Linker, Memory, Module, Store};

struct Context {
    pub wasi: wasmtime_wasi::WasiCtx,
    pub proxy: proxy::ProxyData,
}

struct Executor {}

impl Executor {
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
        let mut store = Store::new(
            &engine,
            Context {
                wasi: wasmtime_wasi::WasiCtxBuilder::new().inherit_stdio().build(),
                proxy: ProxyData::default(),
            },
        );

        wasmtime_wasi::add_to_linker(&mut linker, |cx: &mut Context| &mut cx.wasi).unwrap();
        Proxy::add_to_linker(&mut linker, |cx: &mut Context| &mut cx.proxy).unwrap();

        let instance = linker.instantiate(&mut store, &module).unwrap();
        let proxy = Proxy::new(&mut store, &instance, |cx| &mut cx.proxy).unwrap();

        let result = proxy.onhttp(&mut store, "/login", "", "POST").unwrap();

        println!("{}", result);

        Ok(Self {})
    }
}

fn main() {
    let _executor = Executor::new().unwrap();
}
