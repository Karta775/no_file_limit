# no_file_limit 📂
A tool to work around annoying file limits on platforms like Discord and WhatsApp.

## How it works 🧩
For the sender, the program will cut files into variably-sized chunks (8MB for Discord, 100MB for WhatsApp), and produce a metadata file for the receiver. All the receiver has to do is open the metadata file, and it will stitch together the original file from the chunks.

## Example 🚀
An example file with random data has been provided in `example/`
```shell
$ cargo build
$ cd example/
$ ls
random.data
$ cargo run -- -d random.data
$ ls
random.data
random.data.01
random.data.02
random.data.03
random.data.nfl
```

## Work in progress ⚠️
The program is not ready to use yet, but work is currently being done currently on the deconstructing part (cutting files into chunks).

## TODO 🛠
- [ ] Make the deconstructor
  - [x] Cut files into chunks
  - [x] Generate a metadata file 
  - [ ] Neatly package it into functions or a struct 🧑‍💻
- [ ] Make the reconstructor
  - [ ] Read a metadata file
  - [ ] Stitch files back together
- [ ] Make a decent CLI/TUI experience
- [ ] Make a nice GUI