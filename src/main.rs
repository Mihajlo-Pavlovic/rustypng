use clap::Parser;

use crate::args::Commands;

pub mod args;
mod chunk;
mod chunk_type;
pub mod png;
// mod commands;

/// Generic PNGme error
pub type Error = Box<dyn std::error::Error>;

/// Generic PNGme result
pub type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let cli = args::Cli::parse();
    match &cli.command {
        Commands::Encode(encode) => println!(
            "{:?}, {:?}, {:?}, {:?}",
            encode.file_path, encode.chunk_type, encode.message, encode.output_file
        ),
        Commands::Decode(decode) => println!("{:?}, {:?}", decode.file_path, decode.chunk_type),
        Commands::Remove(remove) => println!("{:?}, {:?}", remove.file_path, remove.chunk_type),
        Commands::Print(print) => println!("{:?}", print.file_path),
    }
    return Ok(());
}
