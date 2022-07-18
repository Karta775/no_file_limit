
use clap::Parser;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::error::Error;
use serde::Serialize;
use serde::Deserialize;

const MEBIBYTE_SIZE: usize = usize::pow(2, 20);

/// Defeat awful file size limits
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The file you want to (de/re)construct
    #[clap(value_parser, required=true)]
    filepath: String,

    /// Deconstruct a file
    #[clap(short, long, value_parser)]
    deconstruct: bool,

    /// The chunk size in megabytes
    #[clap(short, long, value_parser, default_value="8")]
    chunk_size: usize,

    /// Reconstruct a file
    #[clap(short, long, value_parser)]
    reconstruct: bool,
}

#[derive(Serialize)]
#[derive(Deserialize)]
struct Metadata {
    filename: String,
    filesize: usize,
    chunk_size: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let path = Path::new(&args.filepath);
    let bytes = std::fs::read(path).expect("Couldn't read the file");
    let filename = String::from(path.file_name().unwrap().to_str().unwrap());
    let filesize = bytes.len();
    let chunk_size = MEBIBYTE_SIZE * args.chunk_size;

    // TODO: Turn this block into a function or something
    if args.deconstruct {
        // Validation
        if filesize < chunk_size {
            println!("Your input file is already smaller than the selected chunk size");
            return Ok(());
        }

        // Calculate the number of chunks that will be created
        let mut num_of_chunks = filesize / chunk_size;
        num_of_chunks += if filesize % chunk_size != 0 { 1 } else { 0 };

        // Cut the file into chunks
        for i in 0..num_of_chunks {
            let chunk_name = format!("{}.{:02}", filename, i + 1);
            let chunk_start = i * chunk_size;
            let chunk_end = usize::min(chunk_start + chunk_size, filesize);
            let mut file = File::create(chunk_name)
                .expect("Couldn't create a chunk file");
            file.write_all(&bytes[chunk_start..chunk_end])
                .expect("Couldn't write to a chunk file");
        }

        // Generate the metadata file
        let metadata_filename = format!("{}.nfl", filename);
        let metadata = Metadata {
            filename,
            filesize,
            chunk_size
        };
        let toml = toml::to_string(&metadata).unwrap();
        let mut metadata_file = File::create(metadata_filename)
            .expect("Couldn't create the metadata file");
        metadata_file.write_all(toml.as_bytes())
            .expect("Couldn't write to the metadata file");
    }

    return Ok(());
}