use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use crate::metadata;

const MEBIBYTE_SIZE: usize = usize::pow(2, 20);

pub struct Deconstructor {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
    pub filename: String,
    pub filesize: usize,
    pub chunk_size: usize,
}

impl Deconstructor {
    pub fn new(path_str: &str, chunk_size: usize) -> Self {
        let path = PathBuf::from(path_str);
        let bytes = std::fs::read(&path).expect("Couldn't read the file");
        let filename = String::from(path.file_name().unwrap().to_str().unwrap());
        let filesize = bytes.len();
        let chunk_size = MEBIBYTE_SIZE * chunk_size;

        Deconstructor {
            path,
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
        let mut num_of_chunks = self.filesize / self.chunk_size;
        num_of_chunks += if self.filesize % self.chunk_size != 0 { 1 } else { 0 };

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
        let metadata_filename = format!("{}.{}", self.filename, metadata::METADATA_FILE_EXTENSION);
        let metadata = metadata::Metadata {
            filename: self.filename.to_string(),
            filesize: self.filesize,
            chunk_size: self.chunk_size,
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