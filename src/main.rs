use std::{
    fs::File,
    io::{Read, Seek, Write},
    path::Path,
    str::FromStr,
};

use args::Commands;
use clap::Parser;

mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let cli = args::Cli::parse();
    match cli.commands {
        Commands::Encode {
            file_path,
            chunk_type,
            message,
            output_file,
        } => {
            let bytes = get_file_bytes(&file_path)?;
            let mut png = png::Png::try_from(&bytes[..])?;
            let chunk = chunk::Chunk::new(
                chunk_type::ChunkType::from_str(&chunk_type)?,
                message.as_bytes().to_vec(),
            );
            png.append_chunk(chunk);
            let mut output_file = match output_file {
                Some(output_file_path) => File::create(output_file_path)?,
                None => File::create(file_path)?,
            };
            output_file.write(&png.as_bytes())?;
        }
        Commands::Decode {
            file_path,
            chunk_type,
        } => {
            let bytes = get_file_bytes(file_path)?;
            let png = png::Png::try_from(&bytes[..])?;
            let chunk = png.chunk_by_type(&chunk_type).ok_or("Chunk not found")?;
            let data_string = chunk.data_as_string()?;
            // println!("The chunk's data is:");
            println!("{data_string}");
        }
        Commands::Remove {
            file_path,
            chunk_type,
        } => {
            let bytes = get_file_bytes(&file_path)?;
            let mut png = png::Png::try_from(&bytes[..])?;
            png.remove_chunk(&chunk_type)?;
            let mut file = File::create(file_path)?;
            file.write(&png.as_bytes())?;
        }
        Commands::Print { file_path } => {
            let bytes = get_file_bytes(&file_path)?;
            let png = png::Png::try_from(&bytes[..])?;
            println!("{png}");
        }
    }
    Ok(())
}

fn get_file_bytes(file_path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}
