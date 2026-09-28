use wasmtime::component::{Component, HasSelf, Linker, ResourceTable, bindgen};
use wasmtime::{Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpCtxView, WasiHttpView};

bindgen!({
    world: "test",
    path: "../wit",
});

struct HostState {
    wasi: WasiCtx,
    http: WasiHttpCtx,
    table: ResourceTable,
}

impl WasiView for HostState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl WasiHttpView for HostState {
    fn http(&mut self) -> WasiHttpCtxView<'_> {
        WasiHttpCtxView {
            ctx: &mut self.http,
            table: &mut self.table,
            hooks: Default::default(),
        }
    }
}

impl zenmo::test::host_service::Host for HostState {
    fn hello_world(&mut self, data: String) -> String {
        format!("Hello from host, {data}!")
    }
}

fn main() -> wasmtime::Result<()> {
    let component_path = std::env::args()
        .nth(1)
        .expect("usage: host <path-to-plugin.wasm>");

    let engine = Engine::default();
    let component = Component::from_file(&engine, component_path)?;

    let mut linker: Linker<HostState> = Linker::new(&engine);

    // Supply the custom host-service import.
    Test::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| state)?;

    // Supply any WASI 0.2 imports needed by the component.
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
    wasmtime_wasi_http::p2::add_only_http_to_linker_sync(&mut linker)?;

    let state = HostState {
        wasi: WasiCtxBuilder::new()
            .inherit_network()
            .allow_tcp(true)
            .allow_ip_name_lookup(true)
            .inherit_stderr()
            .inherit_stdout()
            .build(),
        http: WasiHttpCtx::new(),
        table: ResourceTable::new(),
    };
    let mut store = Store::new(&engine, state);

    let plugin = Test::instantiate(&mut store, &component, &linker)?;
    let greeting = plugin
        .zenmo_test_plugin_service()
        .call_hello_world(&mut store, &"yes")?;

    println!("{greeting}");

    let result = plugin
        .zenmo_test_plugin_service()
        .call_request(&mut store)?;
    println!("{result}");

    Ok(())
}
