use std::{fs, io};
use crate::metadata::*;
use std::fs::{File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use indicatif::{ProgressBar, ProgressStyle};

fn chunk_filename(base_name: &str, index: usize) -> String {
    format!("{}.{:02}", &base_name, index + 1)
}

pub fn get_dir(path_str: &str) -> PathBuf {
    let mut dir = PathBuf::from(&path_str);
    dir.pop();
    dir
}

pub fn all_chunks_exist(path_str: &str) -> bool {
    let metadata = read_metadata(&path_str);
    let mut dir = get_dir(&path_str).into_os_string().into_string().unwrap();
    if dir == "" { dir = String::from(".") };

    for i in 0..metadata.num_of_chunks {
        let chunk_filename = &chunk_filename(&metadata.filename, i);
        let chunk_exists = Path::new(&format!("{}/{}", &dir, &chunk_filename)).exists();
        if !chunk_exists {
            return false;
        }
    }
    true
}

pub fn discard_chunks(metadata: &Metadata) {
    for i in 0..metadata.num_of_chunks {
        let chunk_filename = chunk_filename(&metadata.filename, i);
        match fs::remove_file(&chunk_filename) {
            Ok(_) => {}
            Err(_) => { println!("Couldn't remove {}", &chunk_filename) }
        }
    }
}

pub fn reconstruct(path_str: &str, no_cleanup: bool) -> Result<(), io::Error> {
    let metadata = read_metadata(&path_str);

    // Check that all chunks are present
    if !all_chunks_exist(&path_str) {
        println!("You don't have enough chunks, there should be {} 😔", metadata.num_of_chunks);
        std::process::exit(1);
    }

    // Create a progress bar
    let pb = ProgressBar::new(metadata.filesize as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template(" 🧩 Merging chunks {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}")
        .progress_chars("#>-"));

    let file = File::create(&metadata.filename)?;
    let mut writer = BufWriter::with_capacity(8192, file);

    let mut bytes: Vec<u8> = vec![0; 8192];
    let mut pb_throttle = 0;
    let mut bytes_written = 0;
    for i in 0..metadata.num_of_chunks {
        let chunk = File::open(&chunk_filename(&metadata.filename, i))?;
        let mut reader = BufReader::with_capacity(8192, chunk);

        // TODO: Consolidate duplicate code
        while bytes_written < usize::min((i + 1) * metadata.chunk_size, metadata.filesize) {
            reader.read_exact(&mut bytes).unwrap();
            let bytes_w = writer.write(&bytes).unwrap();
            bytes_written += bytes_w;

            pb_throttle += bytes_w;
            if pb_throttle > MEBIBYTE_SIZE {
                pb.set_position(bytes_written as u64);
                pb_throttle = 0;
            }
        }
    }
    writer.flush()?;
    pb.finish();

    if bytes_written != metadata.filesize {
        println!("The output file is not the same size as the input file, aborting...");
        fs::remove_file(&metadata.filename)?;
    } else {
        if !no_cleanup {
            println!(" 🧽 Cleaning up...");
            discard_chunks(&metadata);
            discard_metadata(&metadata.filename);
        }
    }

    return Ok(());
}