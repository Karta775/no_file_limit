pub mod deconstructor;
pub mod metadata;

use deconstructor::Deconstructor;
use clap::Parser;
use std::path::PathBuf;
use std::error::Error;

/// Defeat awful file size limits
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The file you want to (de/re)construct
    #[clap(value_parser, required=true)]
    filepath: String,

    /// The chunk size in megabytes
    #[clap(short, long, value_parser, default_value="8")]
    chunk_size: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let path = PathBuf::from(&args.filepath);
    let extension = path.extension().unwrap().to_str().unwrap().to_owned();

    // TODO: Turn this block into a function or something
    if extension == metadata::METADATA_FILE_EXTENSION {
        println!("I'm supposed to reconstruct the file :)");
    } else if extension != metadata::METADATA_FILE_EXTENSION {
        let deconstructor = Deconstructor::new(&args.filepath, args.chunk_size);
        deconstructor.deconstruct().expect("Failed to deconstruct the file");
    }

    return Ok(());
}