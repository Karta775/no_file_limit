use std::{fs, io};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use indicatif::{ProgressBar, ProgressStyle};
use crate::metadata::*;

fn num_of_chunks(filesize: usize, chunk_size: usize) -> usize {
    let mut num_of_chunks = filesize / chunk_size;
    num_of_chunks += if filesize % chunk_size != 0 { 1 } else { 0 };
    num_of_chunks
}

fn valid(metadata: &Metadata) -> bool  {
    let mut status = true;

    if metadata.filesize <= metadata.chunk_size {
        println!("Your input file is already less than or equal to the selected chunk size");
        status = false;
    }
    status
}

fn create_chunks(file: File, metadata: &Metadata) -> Result<(), io::Error> {
    // Calculate the number of chunks that will be created
    let num_of_chunks = num_of_chunks(metadata.filesize, metadata.chunk_size);

    // Create a progress bar
    let pb = ProgressBar::new(metadata.filesize as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template(" ✂️  Creating chunks {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}")
        .progress_chars("#>-"));

    // Cut the file into chunks

    let mut reader = BufReader::with_capacity(8192, file);
    let mut bytes_written = 0;
    for i in 0..num_of_chunks {
        let chunk_name = format!("{}.{:02}", metadata.filename, i + 1);
        let file = File::create(chunk_name)?;
        let mut writer = BufWriter::with_capacity(8192, file);

        let mut bytes: Vec<u8> = vec![0; 8192];
        let mut pb_throttle = 0;
        while bytes_written < usize::min((i + 1) * metadata.chunk_size, metadata.filesize) {
            reader.read_exact(&mut bytes).unwrap();
            let bytes_w = writer.write(&bytes).unwrap();
            bytes_written += bytes_w;

            // The progress bar needs to be updated infrequently to avoid slowdown
            pb_throttle += bytes_w;
            if pb_throttle > MEBIBYTE_SIZE {
                pb.set_position(bytes_written as u64);
                pb_throttle = 0;
            }
        }
        writer.flush().unwrap();
    }
    pb.finish();

    Ok(())
}

fn create_metadata_file(metadata: &Metadata) -> Result<(), io::Error> {
    let metadata_filename = format!("{}.{}", metadata.filename, METADATA_FILE_EXTENSION);
    let toml = toml::to_string(&metadata).unwrap();
    let mut metadata_file = File::create(metadata_filename)?;
    metadata_file.write_all(toml.as_bytes())?;
    Ok(())
}

pub fn deconstruct(path_str: &str, chunk_size: usize) -> Result<(), io::Error> {
    let file_path = PathBuf::from(&path_str);
    let file = File::open(&path_str).unwrap();
    let metadata = {
        let filename = String::from(file_path.file_name().unwrap().to_str().unwrap());
        let filesize = fs::metadata(&path_str).unwrap().len() as usize;
        let chunk_size = MEBIBYTE_SIZE * chunk_size;
        let mut num_of_chunks = filesize / chunk_size;
        num_of_chunks += if filesize % chunk_size != 0 { 1 } else { 0 };
        generate_metadata(filename, filesize, chunk_size, num_of_chunks)
    };

    if !valid(&metadata) {
        std::process::exit(1);
    }

    create_chunks(file, &metadata)?;
    create_metadata_file(&metadata)?;

    Ok(())
}
