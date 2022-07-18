# no_file_limit 📂
A tool to work around annoying file limits on platforms like Discord and WhatsApp.

## How it works
For the sender, the program will cut files into variably-sized chunks (8MB for Discord, 100MB for WhatsApp), and produce a metadata file for the receiver. All the receiver has to do is open the metadata file, and it will stitch together the original file from the chunks.

## Work in progress ⚠️
The program is not ready to use yet, but work is currently being done currently on the deconstructing part (cutting files into chunks).

## TODO 
- [ ] Make the deconstructor 🛠
  - [x] Cut files into chunks 🛠
  - [ ] Generate a metadata file
- [ ] Make the reconstructor
  - [ ] Read a metadata file
  - [ ] Stitch files back together
- [ ] Make a decent CLI/TUI experience
- [ ] Make a nice GUI