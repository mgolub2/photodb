use clap::Parser;
use glob::glob;
use photodb::photo::is_image_file;
use rayon::prelude::*;
use rexiv2::Metadata;
use std::path::PathBuf;

/// Simple photo database management tool. Pixel content based depduplication via xxhash and libraw.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct ExCLI {
    /// The database root to move files into
    pub img_or_img_dir: PathBuf,
    /// enable date exif tags only mode
    #[clap(short, long, default_value_t = false)]
    pub date_only: bool,
}

fn print_exif(path: &PathBuf) {
    let exif = Metadata::new_from_path(path).expect("read exif");
    println!("{}:", path.display());
    exif.get_exif_tags().ok().and_then(|tags| {
        Some(tags.iter().for_each(|f| {
            let val = exif.get_tag_string(f).expect("get tag");
            if val.len() > 100 {
                println!("\t{} :: <long value skipped>", f);
            } else {
                println!("\t{} :: {}", f, val);
            }
        }))
    });
}

fn print_dates(path: &PathBuf) {
    let exif = Metadata::new_from_path(path).expect("read exif");
    exif.get_exif_tags().ok().and_then(|tags| -> Option<Vec<()>> {
        Some(
            tags.iter()
                .filter_map(|t| {
                    (t.contains("Date")).then(|| {
                        let val = exif.get_tag_string(t).expect("get tag");
                        println!("{}", val);
                    })
                })
                .collect(),
        )
    });
}

fn main() {
    let args = ExCLI::parse();
    let func = if args.date_only { print_dates } else { print_exif };
    match args.img_or_img_dir {
        path if path.is_dir() => scan_dir(&path, func),
        path if path.is_file() => func(&path),
        _ => println!("Not a file or directory"),
    }
}

fn scan_dir(image_directory: &PathBuf, func: fn(&PathBuf)) {
    let img_files: Vec<PathBuf> =
        glob(&image_directory.join("**/*").as_os_str().to_str().expect("join"))
            .ok()
            .and_then(|paths| {
                Some(
                    paths
                        .filter_map(|p| {
                            p.ok().and_then(|p| if is_image_file(&p) { Some(p) } else { None })
                        })
                        .collect(),
                )
            })
            .unwrap_or_default();
    img_files.par_iter().for_each(|f| {
        func(f);
    });
}
