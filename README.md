# no_file_limit 📂
A tool to work around annoying file limits on platforms like Discord and WhatsApp.

## How it works 🧩
For the sender, the program will cut files into variably-sized chunks (8MB for Discord, 100MB for WhatsApp), and produce a metadata file for the receiver. All the receiver has to do is open the metadata file, and it will stitch together the original file from the chunks.

## Example 🚀
![A demo GIF of the program deconstructing an reconstructing a file](demo.gif)

## Usage 💡
```
USAGE:
    nfl [OPTIONS] <FILEPATH>

ARGS:
    <FILEPATH>    The file you want to (de/re)construct

OPTIONS:
    -c, --chunk-size <CHUNK_SIZE>    The chunk size in megabytes
    -h, --help                       Print help information
    -n, --no-cleanup                 Don't clean up the chunks and metadata
    -V, --version                    Print version information
```

## TODO 🛠
- [x] Make the slicer
  - [x] Cut files into chunks
  - [x] Generate a metadata file 
  - [x] ~~Neatly~~ refactor it into functions or a struct
- [x] Make the reconstructor
  - [x] Read a metadata file
  - [x] Stitch files back together
- [x] Make a decent CLI/TUI experience
  - [x] Use a crate like indicatif to show progress
  - [x] Offer an interactive mode if no flags are set
- [ ] Fix the poor error handling 🧑‍💻📖
- [ ] Rewrite the nasty bits of code 🧑‍💻📖
- [ ] Make a nice GUI