use anyhow::{Context, Result};
use wasmtime::component::*;
use wasmtime::{Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiView, WasiCtxView};

const WIT_ENCODE_SECRET_INTO_BMP: &str = "encode-secret-into-bmp";
const WIT_DECODE_SECRET_FROM_BMP: &str = "decode-secret-from-bmp";

pub(crate) struct MyState {
    ctx: WasiCtx,
    table: ResourceTable
}

impl WasiView for MyState {
    fn ctx(&'_ mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table
        }
    }
}

pub(crate) struct WasmComponent {
    pub instance: Instance,
    pub store: Store<MyState>,
}

impl WasmComponent {
    pub fn from_bytes(wasm_bytes: &[u8]) -> Result<Self> {
        let engine = Engine::default();
        let component = Component::from_binary(&engine, wasm_bytes).with_context(|| "Failed to open the wasm component.")?;
        let mut builder = WasiCtx::builder();
        let mut store = Store::new(&engine, MyState {
            ctx: builder.build(),
            table: ResourceTable::new() 
        });
        let mut linker = Linker::<MyState>::new(&engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker).context("Failed adding the imported log function to the wasm linker")?;

        let _ = linker.root().func_wrap("log", |_store, s: (String,)| {
            println!("{}", s.0);
            Ok(())
        });
        let instance = linker.instantiate(&mut store, &component).context("Failed to instantiate the linker")?;

        Ok(WasmComponent {
            instance,
            store,
            // export_index
        })
    }

    pub fn encode_secret_into_bmp(&mut self, secret: &str, image_bytes: Vec<u8>) -> Result<Vec<u8>> {
        let func = self.instance.get_export(&mut self.store, None, WIT_ENCODE_SECRET_INTO_BMP).context("couldnt find the function")?;
        let typed_func: TypedFunc<(String, Vec<u8>), (Vec<u8>,)> = self.instance.get_typed_func(&mut self.store, func.1).expect("filed to get the function");
        let image = typed_func.call(&mut self.store, (secret.to_owned(), image_bytes.to_vec())).expect("something happened picking the wasm function result");
        
        Ok(image.0)
    }

    pub fn decode_secret_from_bmp(&mut self, image_bytes: Vec<u8>) -> Result<String> {
        let func = self.instance.get_export(&mut self.store, None, WIT_DECODE_SECRET_FROM_BMP).expect("couldnt find the function");
        let typed_func_decode: TypedFunc<(Vec<u8>,), (String,)> = self.instance.get_typed_func(&mut self.store, func.1).expect("filed to get the function");
        let secret = typed_func_decode.call(&mut self.store, (image_bytes,)).expect("something happened decoding the secret");

        Ok(secret.0)
    }
}

