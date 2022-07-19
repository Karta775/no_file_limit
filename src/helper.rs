use std::{fs, io};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use serde::Serialize;
use serde::Deserialize;

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
    pub bytes: Vec<u8>,
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
        let file_path = PathBuf::from(path_str);
        let bytes = fs::read(&path_str).expect("Couldn't read the file");
        let filename = String::from(file_path.file_name().unwrap().to_str().unwrap());
        let filesize = bytes.len();
        let chunk_size = MEBIBYTE_SIZE * chunk_size;

        Slicer {
            file_path,
            bytes,
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

    fn create_chunks(&self) -> Result<(), io::Error> {
        // Calculate the number of chunks that will be created
        let num_of_chunks = num_of_chunks(self.filesize, self.chunk_size);

        // Cut the file into chunks
        for i in 0..num_of_chunks {
            let chunk_name = format!("{}.{:02}", self.filename, i + 1);
            let chunk_start = i * self.chunk_size;
            let chunk_end = usize::min(chunk_start + self.chunk_size, self.filesize);
            println!("Creating {}", chunk_name);
            let mut file = File::create(chunk_name)?;
            file.write_all(&self.bytes[chunk_start..chunk_end])?;
        }

        Ok(())
    }

    fn create_metadata(&self) -> Result<(), io::Error> {
        // Generate the metadata file
        let num_of_chunks = num_of_chunks(self.filesize, self.chunk_size);
        let metadata_filename = format!("{}.{}", self.filename, METADATA_FILE_EXTENSION);
        println!("Creating {}", metadata_filename);
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

    pub fn deconstruct(&self) -> Result<(), io::Error> {
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
        let bytes = fs::read(&metadata_path).expect("Couldn't read the file");
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
            println!("Merging {}", &self.chunk_filename(i));
            let chunk_bytes = fs::read(&self.chunk_filename(i))?;
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