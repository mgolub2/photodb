# PhotoDB

A simple rust program for managing raw images.

## Features

* Uses libraw to get the actual pixel content of each raw.
* Pixel content is used as the hash of the image, so metadata changes to E.I the exif have no effect.
* Organizes files into Year / Month / Camera model folders
* Uses sqlite to store the hashes of imported files
* Can verify those hashes have not changed

## Usage
```plaintext
Search for a pattern in a file and display the lines that contain it

Usage: photodb [OPTIONS] <COMMAND>

Commands:
  import  Import files into the database
  verify  Verify the raw image file hashes
  help    Print this message or the help of the given subcommand(s)

Options:
      --import-path <IMPORT_PATH>  The database root to move files into [default: photodb]
  -m, --move-files                 Move the files to the database root
  -i, --insert                     Import the files into the database, checking for duplicates
  -d, --database <DATABASE>        The name of the database to use [default: :memory:]
  -c, --create                     Create the database
  -h, --help                       Print help
  -V, --version                    Print version
```

## Build and Install
```shell
git clone https://github.com/mgolub2/photodb.git
cd photodb
cargo build --release
mv target/release/photodb /usr/local/bin/photodb
```