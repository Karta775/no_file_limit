use std::thread;
use std::time::Duration;
use rand::{Rng, thread_rng};
use clap::Parser;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

const MEBIBYTE_SIZE: usize = usize::pow(2, 20);

/// Defeat awful file size limits
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The name of the file you want to (de/re)construct
    #[clap(value_parser, required=true)]
    filepath: String,

    /// Deconstruct a file into n-MB chunks
    #[clap(short, long, value_parser)]
    deconstruct: bool,

    /// Reconstruct a file from n-MB chunks
    #[clap(short, long, value_parser)]
    reconstruct: bool,
}

fn main() {
    let mut rng = thread_rng();
    let args = Args::parse();
    let chunk_size = MEBIBYTE_SIZE * 8;

    let path = Path::new(&args.filepath);
    let bytes = std::fs::read(path).expect("Couldn't read the file");
    let filename = path.display();
    let filesize = bytes.len();

    if args.deconstruct {
        // Calculate the number of chunks that will be created
        let mut num_of_chunks = filesize / chunk_size;
        num_of_chunks += if filesize % chunk_size != 0 { 1 } else { 0 };

        // Cut the file into chunks
        for i in 0..num_of_chunks {
            let chunk_name = format!("{}.{:01}", filename, i + 1);
            let chunk_start = i * chunk_size;
            let chunk_end = usize::min(chunk_start + chunk_size, filesize);
            let mut file = File::create(chunk_name)
                .expect("Couldn't create a file");
            file.write_all(&bytes[chunk_start..chunk_end])
                .expect("Couldn't write to the file");
        }
    }
}