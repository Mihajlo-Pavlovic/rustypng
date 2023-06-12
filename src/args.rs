use clap::{command, Args, Parser, Subcommand};

#[derive(Parser)]
#[command()]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
#[derive(Subcommand)]
pub enum Commands {
    Encode(EncodeArgs),
    Decode(DecodeArgs),
    Remove(RemoveArgs),
    Print(PrintArgs),
}

#[derive(Args)]
pub struct EncodeArgs {
    pub file_path: String,
    pub chunk_type: String,
    pub message: String,
    pub output_file: Option<String>,
}

#[derive(Args)]
pub struct DecodeArgs {
    pub file_path: String,
    pub chunk_type: String,
}

#[derive(Args)]
pub struct RemoveArgs {
    pub file_path: String,
    pub chunk_type: String,
}

#[derive(Args)]
pub struct PrintArgs {
    pub file_path: String,
}
