mod wasm;
mod options;

use anyhow::Context;
use wasm::WasmComponent;
use options::*;
use clap::Parser;
use clio::{InputPath, OutputPath};
use std::fs;

const WASM_BYTES: &[u8] = include_bytes!(env!("WASM_STEGANOGRAPHY_FILE_PATH"));

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let wasm_bytes = match args.wasm {
        Some(path) => std::fs::read(path.path().path()).with_context(|| format!("Failed reading the wasm component at: {}", path))?,
        None => WASM_BYTES.to_vec()
    };
    
    match args.command {
        Command::Encode { secret, input_file, output_file } => {
            let mut component = WasmComponent::from_bytes(&wasm_bytes)?;
            let image_bytes = fs::read(InputPath::path(&input_file).path()).with_context(|| format!("Failed reading file: {}", &output_file.path()))?;
            let encoded_image = component.encode_secret_into_bmp(&secret, image_bytes)?;
            fs::write(OutputPath::path(&output_file).path(), &encoded_image).with_context(|| format!("Failed writing file: {}", &output_file.path()))?;
        },
        Command::Decode { input_file } => {
            let mut component = WasmComponent::from_bytes(&wasm_bytes)?;
            let image_bytes = fs::read(InputPath::path(&input_file).path()).with_context(|| format!("Failed reading file: {}", &input_file.path()))?;
            let secret_decoded = component.decode_secret_from_bmp(image_bytes)?;

            println!("{}", secret_decoded);
        }
    }

    Ok(())
}
