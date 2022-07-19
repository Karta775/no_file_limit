use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
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
        let bytes = std::fs::read(&path_str).expect("Couldn't read the file");
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

    fn create_chunks(&self) {
        // Calculate the number of chunks that will be created
        let num_of_chunks = num_of_chunks(self.filesize, self.chunk_size);

        // Cut the file into chunks
        for i in 0..num_of_chunks {
            let chunk_name = format!("{}.{:02}", self.filename, i + 1);
            let chunk_start = i * self.chunk_size;
            let chunk_end = usize::min(chunk_start + self.chunk_size, self.filesize);
            let mut file = File::create(chunk_name)
                .expect("Couldn't create a chunk file");
            file.write_all(&self.bytes[chunk_start..chunk_end])
                .expect("Couldn't write to a chunk file");
        }
    }

    fn create_metadata(&self) {
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
        let mut metadata_file = File::create(metadata_filename)
            .expect("Couldn't create the metadata file");
        metadata_file.write_all(toml.as_bytes())
            .expect("Couldn't write to the metadata file");

    }

    pub fn deconstruct(&self) -> Result<(), Box<dyn Error>> {
        if !self.valid() {
            return Ok(()); // TODO: These Ok(()) returns seem weird, find something better
        }
        self.create_chunks();
        self.create_metadata();

        return Ok(());
    }
}

impl Glue {
    pub fn new(path_str: &str) -> Self {
        let metadata_path = PathBuf::from(path_str);
        let bytes = std::fs::read(&metadata_path).expect("Couldn't read the file");
        let metadata: Metadata = toml::from_slice(&bytes).expect("Couldn't parse the metadata");

        Glue {
            metadata_path,
            metadata
        }
    }

    pub fn reconstruct(&self) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(&self.metadata.filename)
            .expect("Couldn't create a chunk file");
        let mut file_bytes: Vec<u8> = Vec::new();

        for i in 0..self.metadata.num_of_chunks {
            let chunk_filename = format!("{}.{:02}", self.metadata.filename, i + 1);
            let chunk_bytes = std::fs::read(&chunk_filename).expect("Couldn't read the file");
            file_bytes.extend(chunk_bytes);
        }
        file.write_all(&file_bytes)
            .expect("Couldn't write to the file");

        if file_bytes.len() != self.metadata.filesize {
            println!("The output file is not the same size as the input file, something went wrong");
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