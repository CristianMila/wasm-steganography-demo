mod wasm;
mod options;

use anyhow::Context;
use wasmtime::component::*;
use options::*;
use clap::Parser;
use clio::{InputPath, OutputPath};
use wasmtime::{component::Component, Engine, Store};
use wasmtime_wasi::WasiCtx;
use std::fs;

use crate::wasm::MyState;

const WASM_BYTES: &[u8] = include_bytes!(env!("WASM_STEGANOGRAPHY_FILE_PATH"));

wasmtime::component::bindgen!("steganography" in "../wasm/wit/world.wit");

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let args = Cli::parse();
    let engine = Engine::default();
    let component = Component::from_binary(&engine, WASM_BYTES).with_context(|| "Failed to open the wasm component.")?;
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
    let steg = Steganography::new(&mut store, &instance)?;

    match args.command {
        Command::Encode { secret, input_file, output_file } => {
            let file_type = input_file.path().extension().and_then(|s| s.to_str()).expect("unknown file extension").to_lowercase();
            let image_bytes = fs::read(InputPath::path(&input_file).path()).with_context(|| format!("Failed reading file: {}", &output_file.path()))?;
            let encoded_image = match file_type.as_str() {
                "bmp" => steg.call_encode_secret_into_bmp(&mut store, &secret.to_owned(), &image_bytes).expect("Failed call to wasm method."),
                "jpg" | "jpeg" => steg.call_encode_secret_into_jpeg(&mut store, &secret.to_owned(), &image_bytes).expect("Failed call to wasm method."),
                _ => panic!("unsupported file type: {}", file_type),
            };
            fs::write(OutputPath::path(&output_file).path(), &encoded_image).with_context(|| format!("Failed writing file: {}", &output_file.path()))?;
        },
        Command::Decode { input_file } => {
            let file_type = input_file.path().extension().and_then(|s| s.to_str()).expect("unknown file extension").to_lowercase();
            let image_bytes = fs::read(InputPath::path(&input_file).path()).with_context(|| format!("Failed reading file: {}", &input_file.path()))?;
            let secret_decoded = match file_type.as_str() {
                "bmp" => steg.call_decode_secret_from_bmp(&mut store, &image_bytes)?,
                "jpg" | "jpeg" => steg.call_decode_secret_from_jpeg(&mut store, &image_bytes)?,
                _ => panic!("unsupported file type: {}", file_type),
            };

            println!("{}", secret_decoded);
        }
    }

    Ok(())
}
