use std::fs::read;
use std::path::PathBuf;
use serde::Serialize;
use serde::Deserialize;

pub const MEBIBYTE_SIZE: usize = usize::pow(2, 20);
pub const METADATA_FILE_EXTENSION: &str = "nfl";

#[derive(Serialize)]
#[derive(Deserialize)]
pub struct Metadata {
    pub filename: String,
    pub filesize: usize,
    pub chunk_size: usize,
    pub num_of_chunks: usize,
}

pub fn generate_metadata(filename: String, filesize: usize, chunk_size: usize, num_of_chunks: usize) -> Metadata {
    Metadata {
        filename,
        filesize,
        chunk_size,
        num_of_chunks,
    }
}

pub fn read_metadata(path_str: &str) -> Metadata {
    let metadata_path = PathBuf::from(&path_str);
    let bytes = read(&metadata_path).expect("Couldn't read the file");
    toml::from_slice(&bytes).expect("Couldn't parse the metadata")
}