use clap::Parser;
use glob::{glob_with, MatchOptions};
use photodb::image::is_image_file;
use std::path::PathBuf;

/// Simple photo database management tool. Pixel content based depduplication via xxhash and libraw.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct ExCLI {
    /// The database root to move files into
    pub img_or_img_dir: PathBuf,
}

fn print_exif(path: &PathBuf) {
    let file = std::fs::File::open(path).expect("read file");
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader).expect("read exif");
    println!("{}:", path.display());
    for f in exif.fields() {
        let value = f.display_value().with_unit(&exif);
        if value.to_string().len() > 100 { continue; }
        println!("\t{}:{} :: {}", f.ifd_num, f.tag, value);
    }
}

fn main() {
    let args = ExCLI::parse();
    match args.img_or_img_dir {
        path if path.is_dir() => scan_dir(&path),
        path if path.is_file() => print_exif(&path),
        _ => println!("Not a file or directory"),
    }
}

fn scan_dir(image_directory: &PathBuf) {
    let options: MatchOptions = Default::default();
    let img_files: Option<PathBuf> =
        glob_with(&image_directory.join("**/*").as_os_str().to_str().expect("join"), options)
            .ok()
            .and_then(|paths| {
                Some(
                    paths
                        .filter_map(|x| x.ok())
                        .filter_map(|path| is_image_file(&path).then_some(path))
                        .collect(),
                )
            });
    for f in img_files.iter() {
        print_exif(f);
    }
}
