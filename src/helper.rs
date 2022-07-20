use std::{fs, io};
use std::fs::{File, read};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use serde::Serialize;
use serde::Deserialize;
use indicatif::{ProgressBar, ProgressStyle};

const MEBIBYTE_SIZE: usize = usize::pow(2, 20);
pub const METADATA_FILE_EXTENSION: &str = "nfl";

#[derive(Serialize)]
#[derive(Deserialize)]
pub struct Metadata {
    pub filename: String,
    pub filesize: usize,
    pub chunk_size: usize,
    pub num_of_chunks: usize,
}

pub struct Slicer {
    pub file_path: PathBuf,
    pub reader: BufReader<File>,
    pub filename: String,
    pub filesize: usize,
    pub chunk_size: usize,
}

pub struct Glue {
    pub metadata_path: PathBuf,
    pub metadata: Metadata,
}

fn num_of_chunks(filesize: usize, chunk_size: usize) -> usize {
    let mut num_of_chunks = filesize / chunk_size;
    num_of_chunks += if filesize % chunk_size != 0 { 1 } else { 0 };
    num_of_chunks
}

impl Slicer {
    pub fn new(path_str: &str, chunk_size: usize) -> Self {
        let file_path = PathBuf::from(&path_str);
        let file = File::open(&path_str).unwrap();
        let reader = BufReader::with_capacity(8192, file);
        let filename = String::from(file_path.file_name().unwrap().to_str().unwrap());
        let filesize = fs::metadata(&path_str).unwrap().len() as usize;
        let chunk_size = MEBIBYTE_SIZE * chunk_size;

        Slicer {
            file_path,
            reader,
            filename,
            filesize,
            chunk_size,
        }
    }

    fn valid(&self) -> bool  {
        let mut status = true;

        if self.filesize <= self.chunk_size {
            println!("Your input file is already less than or equal to the selected chunk size");
            status = false;
        }
        status
    }

    fn create_chunks(&mut self) -> Result<(), io::Error> {
        // Calculate the number of chunks that will be created
        let num_of_chunks = num_of_chunks(self.filesize, self.chunk_size);

        // Create a progress bar
        let pb = ProgressBar::new(self.filesize as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template(" ✂️  Creating chunks {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}")
            .progress_chars("#>-"));

        // Cut the file into chunks
        let mut bytes_written = 0;
        for i in 0..num_of_chunks {
            let chunk_name = format!("{}.{:02}", self.filename, i + 1);
            let file = File::create(chunk_name)?;
            let mut writer = BufWriter::with_capacity(8192, file);

            let mut bytes: Vec<u8> = vec![0; 8192];
            let mut pb_throttle = 0;
            while bytes_written < usize::min((i + 1) * self.chunk_size, self.filesize) {
                self.reader.read_exact(&mut bytes).unwrap();
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

    fn create_metadata(&self) -> Result<(), io::Error> {
        // Generate the metadata file
        let num_of_chunks = num_of_chunks(self.filesize, self.chunk_size);
        let metadata_filename = format!("{}.{}", self.filename, METADATA_FILE_EXTENSION);
        let metadata = Metadata {
            filename: self.filename.to_string(),
            filesize: self.filesize,
            chunk_size: self.chunk_size,
            num_of_chunks,
        };
        let toml = toml::to_string(&metadata).unwrap();
        let mut metadata_file = File::create(metadata_filename)?;
        metadata_file.write_all(toml.as_bytes())?;

        Ok(())
    }

    pub fn deconstruct(&mut self) -> Result<(), io::Error> {
        if !self.valid() {
            return Ok(()); // TODO: These Ok(()) returns seem weird, find something better
        }
        self.create_chunks()?;
        self.create_metadata()?;
        println!("Success!");

        Ok(())
    }
}

impl Glue {
    pub fn new(path_str: &str) -> Self {
        let metadata_path = PathBuf::from(path_str);
        let bytes = read(&metadata_path).expect("Couldn't read the file");
        let metadata: Metadata = toml::from_slice(&bytes).expect("Couldn't parse the metadata");

        Glue {
            metadata_path,
            metadata
        }
    }

    fn chunk_filename(&self, index: usize) -> String {
        format!("{}.{:02}", self.metadata.filename, index + 1)
    }

    pub fn all_chunks_exist(&self) -> (bool, usize) {
        let mut count = 0;
        for i in 0..self.metadata.num_of_chunks {
            let chunk_exists = Path::new(&self.chunk_filename(i)).exists();
            count += if chunk_exists { 1 } else { 0 };
        }
        (count == self.metadata.num_of_chunks, count)
    }

    pub fn discard_chunks(&self) {
        for i in 0..self.metadata.num_of_chunks {
            let chunk_filename = self.chunk_filename(i);
            match fs::remove_file(&chunk_filename) {
                Ok(_) => { println!("Removed {}", &chunk_filename) }
                Err(_) => { println!("Couldn't remove {}", &chunk_filename) }
            }
        }
    }

    pub fn discard_metadata(&self) {
        let metadata_filename = format!("{}.{}", &self.metadata.filename, METADATA_FILE_EXTENSION);
        match fs::remove_file(&metadata_filename) {
            Ok(_) => { println!("Removed {}", &metadata_filename) }
            Err(_) => { println!("Couldn't remove {}", &metadata_filename) }
        }
    }

    pub fn reconstruct(&self) -> Result<(), io::Error> {
        let (all_chunks_exist, count) = self.all_chunks_exist();
        if !all_chunks_exist {
            println!("Only {}/{} chunks were found :(", count, self.metadata.num_of_chunks);
            std::process::exit(1);
        }

        let mut file = File::create(&self.metadata.filename)?;
        let mut file_bytes: Vec<u8> = Vec::new();
        for i in 0..self.metadata.num_of_chunks {
            let chunk_bytes = read(&self.chunk_filename(i))?;
            file_bytes.extend(chunk_bytes);
        }
        file.write_all(&file_bytes)?;

        if file_bytes.len() != self.metadata.filesize {
            println!("The output file is not the same size as the input file, aborting...");
            fs::remove_file(&self.metadata.filename)?;
        } else {
            println!("Success! Cleaning up...");
            self.discard_chunks();
            self.discard_metadata();
        }

        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    #[should_panic(expected = "No such file or directory")]
    fn test_deconstruct_bad_file() {
        let slicer = Slicer::new("not.a.real.file", 8);
    }
}