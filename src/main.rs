use std::thread;
use std::time::Duration;
use indicatif::ProgressBar;
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
    let filesize = bytes.len();

    if args.deconstruct {
        println!("This operation will cut into {} pieces.", filesize / chunk_size);
        println!("The leftover chunk will be {} bytes", filesize % chunk_size);
        let mut num_of_chunks = filesize / chunk_size;
        num_of_chunks += if filesize % chunk_size != 0 { 1 } else { 0 };
        println!("We're making {} chunks", num_of_chunks);
        let mut file = File::create("foo.txt").expect("Couldn't create a file");
        file.write_all(&bytes).expect("Couldn't write to the file");
    }

    println!("The chunk size is {}", chunk_size);
    println!("Size of file is {} bytes", bytes.len());

    let pb = ProgressBar::new(1000);
    for _ in 0..1000 {
        pb.inc(1);
        thread::sleep(Duration::from_millis(rng.gen_range(0..=3)));
    }

    pb.finish_with_message("done");
}