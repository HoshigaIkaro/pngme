use std::path::PathBuf;

use clap::{ColorChoice, Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(next_line_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Encodes PNG file
    Encode {
        file_path: PathBuf,
        chunk_type: String,
        message: String,
        output_file: Option<String>,
    },
    /// Decods PNG file
    Decode {
        file_path: PathBuf,
        chunk_type: String,
    },
    /// Removes chunk type from file
    Remove {
        file_path: PathBuf,
        chunk_type: String,
    },
    /// Prints PNG header and chunks
    Print { file_path: PathBuf },
}
