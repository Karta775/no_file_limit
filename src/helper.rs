use std::{fs, io};
use std::fs::{File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use indicatif::{ProgressBar, ProgressStyle};
use crate::metadata::*;

pub struct Slicer;
pub struct Glue;

fn chunk_filename(base_name: &str, index: usize) -> String {
    format!("{}.{:02}", &base_name, index + 1)
}

fn get_dir(path_str: &str) -> PathBuf {
    let mut dir = PathBuf::from(&path_str);
    dir.pop();
    dir
}

fn all_chunks_exist(path_str: &str) -> bool {
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

pub fn discard_metadata_file(base_name: &str) {
    let metadata_filename = format!("{}.{}", &base_name, METADATA_FILE_EXTENSION);
    match fs::remove_file(&metadata_filename) {
        Ok(_) => {}
        Err(_) => { println!("Couldn't remove {}", &metadata_filename) }
    }
}

fn num_of_chunks(filesize: usize, chunk_size: usize) -> usize {
    let mut num_of_chunks = filesize / chunk_size;
    num_of_chunks += if filesize % chunk_size != 0 { 1 } else { 0 };
    num_of_chunks
}

fn copy_chunk(metadata: &Metadata, reader: &mut BufReader<File>, writer: &mut BufWriter<File>,
              pb: &ProgressBar, index: usize) -> usize {
    let mut buffer: Vec<u8> = vec![0; 8192];
    let mut bytes_written = 0;
    let mut pb_throttle = 0;

    while bytes_written < usize::min((index + 1) * metadata.chunk_size, metadata.filesize) {
        reader.read(&mut buffer).unwrap();
        let bytes_w = writer.write(&buffer).unwrap();
        bytes_written += bytes_w;

        pb_throttle += bytes_w;
        if pb_throttle > MEBIBYTE_SIZE {
            pb.set_position(bytes_written as u64);
            pb_throttle = 0;
        }
    }
    bytes_written
}


impl Slicer {
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
        for i in 0..num_of_chunks {
            let chunk_name = format!("{}.{:02}", metadata.filename, i + 1);
            let file = File::create(chunk_name)?;
            let mut writer = BufWriter::with_capacity(8192, file);

            copy_chunk(&metadata, &mut reader, &mut writer, &pb, i);
            writer.flush()?;
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
        let file = File::open(&path_str)?;
        let metadata = {
            let filename = String::from(file_path.file_name().unwrap().to_str().unwrap());
            let filesize = fs::metadata(&path_str).unwrap().len() as usize;
            let chunk_size = MEBIBYTE_SIZE * chunk_size;
            let mut num_of_chunks = filesize / chunk_size;
            num_of_chunks += if filesize % chunk_size != 0 { 1 } else { 0 };
            generate_metadata(filename, filesize, chunk_size, num_of_chunks)
        };

        if !Self::valid(&metadata) {
            std::process::exit(1);
        }

        Self::create_chunks(file, &metadata)?;
        Self::create_metadata_file(&metadata)?;

        Ok(())
    }
}

impl Glue {
    fn discard_chunks(metadata: &Metadata) {
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

        let mut bytes_written = 0;
        for i in 0..metadata.num_of_chunks {
            let chunk = File::open(&chunk_filename(&metadata.filename, i))?;
            let mut reader = BufReader::with_capacity(8192, chunk);
            bytes_written = copy_chunk(&metadata, &mut reader, &mut writer, &pb, i);
        }
        writer.flush()?;
        pb.finish();

        if bytes_written != metadata.filesize {
            println!("The output file is not the same size as the input file, aborting...");
            fs::remove_file(&metadata.filename)?;
        } else {
            if !no_cleanup {
                println!(" 🧽 Cleaning up...");
                Self::discard_chunks(&metadata);
                discard_metadata_file(&metadata.filename);
            }
        }

        return Ok(());
    }
}
