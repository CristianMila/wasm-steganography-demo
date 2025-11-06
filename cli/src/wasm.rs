use wasmtime::component::*;
use wasmtime_wasi::{WasiCtx, WasiView, WasiCtxView};

pub(crate) struct MyState {
    pub(crate) ctx: WasiCtx,
    pub(crate) table: ResourceTable
}

impl WasiView for MyState {
    fn ctx(&'_ mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table
        }
    }
}

