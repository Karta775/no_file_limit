
mod slicer;
mod glue;
mod metadata;

use metadata::*;
use clap::Parser;
use std::path::PathBuf;
use std::io;
use dialoguer::{theme::ColorfulTheme, Select, Input};

const SELECTION_DISCORD: &str = "Discord";
const SELECTION_WHATSAPP: &str = "Whatsapp";
const SELECTION_FS_FAT: &str = "FAT filesystem";
const SELECTION_CUSTOM: &str = "Custom chunk size";

/// Defeat awful file size limits
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The file you want to (de/re)construct
    #[clap(value_parser, required=true)]
    filepath: String,

    /// The chunk size in megabytes
    #[clap(short, long, value_parser)]
    chunk_size: Option<usize>,

    /// Don't clean up the chunks and metadata
    #[clap(short, long)]
    no_cleanup: bool,
}

// TODO: Add a disable_emoji flag

fn main() -> Result<(), io::Error>{
    let args = Args::parse();
    let path = PathBuf::from(&args.filepath);
    // TODO: Error handling - file not found
    let extension = match path.extension() {
        Some(x) => x.to_str().unwrap(),
        None => "",
    };

    // If the metadata file is found then reconstruct, otherwise deconstruct
    if extension == METADATA_FILE_EXTENSION {
        glue::reconstruct(&args.filepath, args.no_cleanup)?;
    } else if extension != METADATA_FILE_EXTENSION {
        // Set the chunk size in MiB from commandline arguments or interactive mode
        let chunk_size = match args.chunk_size {
            Some(size) => size,
            None => select_chunk_size()
        };
        slicer::deconstruct(&args.filepath, chunk_size)?;
    }

    return Ok(())
}

fn select_chunk_size() -> usize {
    let selections = &[
        SELECTION_DISCORD,
        SELECTION_WHATSAPP,
        SELECTION_FS_FAT,
        SELECTION_CUSTOM,
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Where are you sending the file?")
        .default(0)
        .items(&selections[..])
        .interact()
        .unwrap();

    let chunk_size: usize = match selections[selection] {
        SELECTION_DISCORD => 8,
        SELECTION_WHATSAPP => 100,
        SELECTION_FS_FAT => 4192 - 1,
        SELECTION_CUSTOM => { // TODO: Error handling - bad input
            let num: String = Input::new()
                .with_prompt("Enter the size in megabytes")
                .interact_text().expect("Something went wrong");
            num.parse().unwrap()
        }
        _ => 0,
    };

    chunk_size
}