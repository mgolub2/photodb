use std::{fs, path};

use libraw::Processor;
use xxhash_rust::xxh3::Xxh3;
const SEED: u64 = 0xdeadbeef;

use clap::Parser;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    /// The path to the file to read
    path: std::path::PathBuf,
}

fn read_hash_image(path: &path::Path) -> u128 {
    let buf = fs::read(path).expect("read in");
    let processor = Processor::new();
    let decoded = processor.decode(&buf).expect("decoding successful");
    let mut xxh: Xxh3 = Xxh3::with_seed(SEED);
    for u16 in decoded.iter() {
        let u8_bytes = u16.to_be_bytes();
        let u8_arr = [u8_bytes[0], u8_bytes[1]];
        xxh.update(&u8_arr);
    }
    return xxh.digest128();
}


fn main() {
    let args = Cli::parse();
    let hash = read_hash_image(args.path.as_path());
    println!("hash: {}", hash);
}
