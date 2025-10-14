use clap::{Parser, Subcommand};
use clio::{InputPath, OutputPath};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Cli {
    /// Path to the WASM Component. Defaults to "./wasm_steganography.wasm"
    #[arg(long)]
    pub wasm: Option<InputPath>,
    #[command(subcommand)]
    pub command: Command
}

#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    /// Embeds a secret into an image
    Encode {
        /// Secret to be embeded into the image
        #[arg(short, long)]
        secret: String,
        /// Path to the image. Must be a 24bit depth BMP file.
        #[arg(short, long, value_parser)]
        input_file: InputPath,

        /// File path for the new encoded image.
        #[arg(short, long, value_parser, default_value="-")]
        output_file: OutputPath
    },
    /// Gets the secret out of a previously encoded image
    Decode {
        /// Path to the encoded image.
        #[arg(short, long, value_parser)]
        input_file: clio::InputPath,
    }
}
