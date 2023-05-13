extern crate photodb;
use clap::Parser;
use photodb::photodb_error::PhotoDBError;
use photodb::raw_photo::exit;
use photodb::{build_config_path, db, util};

use glob::{glob_with, MatchOptions};
use photodb::raw_photo::Photo;
use rayon::prelude::*;
use rusqlite::*;
use std::{fs, path::PathBuf};

use crate::util::is_image_file;

/// Simple photo database management tool. Pixel content based depduplication via xxhash and libraw.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// The database root to move files into
    #[clap(long, default_value = "photodb")]
    pub db_root: PathBuf,
    /// Move the files to the database root
    #[clap(short, long, default_value_t = false)]
    pub move_files: bool,
    /// Import the files into the database, checking for duplicates
    #[clap(short, long, default_value_t = false)]
    pub insert: bool,
    /// Create the database
    #[clap(short, long, default_value_t = false)]
    pub create: bool,
    /// The path to the file or directory to read
    path: PathBuf,
}

fn import_directory(
    path_to_import: &PathBuf, import_path: &PathBuf, move_file: bool, insert: bool,
    database: &PathBuf,
) {
    if !path_to_import.is_dir() {
        println!("{} is not a directory", path_to_import.display());
        unsafe { exit(1) };
    }

    let img_files = get_img_file_list(path_to_import);
    let total_files = img_files.len();
    println!("Importing {} files", total_files);
    let photo_vec = get_photos_from_img_file_list(&img_files, import_path);
    let hashed = &photo_vec.len();
    println!("Hashed {}/{} files", hashed, total_files);
    let copy_list: Vec<Photo> = photo_vec
        .into_iter()
        .filter(|photo| !db::is_imported(photo.hash, database))
        .filter_map(|photo| {
            if insert {
                db::insert_file_to_db(&photo, database)
                    .map_err(|e| {
                        println!(
                            "{}",
                            PhotoDBError::new(
                                format!("inserting file: {}", e).as_str(),
                                &photo.og_path
                            )
                        );
                    })
                    .ok()
                    .and_then(|_| {
                        println!("inserted file: {} -> {}", photo.og_path.display(), photo.hash);
                        Some(photo)
                    })
            } else {
                println!(
                    "mock inserted file: {} -> {}",
                    photo.og_path.display(),
                    photo.db_path.display()
                );
                Some(photo)
            }
        })
        .collect();
    println!("{}/{} files to copy", copy_list.len(), total_files);
    let copied: u64;
    if move_file {
        copied = copy_list
            .par_iter()
            .map(|photo| {
                //check if photo.db_path exists, create it if it does not
                if !photo.db_path.parent().unwrap().exists() {
                    fs::create_dir_all(photo.db_path.parent().unwrap())
                        .map_err(|e| {
                            println!(
                                "{}",
                                PhotoDBError::new(
                                    format!("creating directory: {}", e).as_str(),
                                    &photo.og_path
                                )
                            );
                            return 0;
                        })
                        .ok();
                }
                fs::copy(&photo.og_path, &photo.db_path)
                    .map_err(|e| {
                        println!(
                            "{}",
                            PhotoDBError::new(
                                format!("copying file: {}", e).as_str(),
                                &photo.og_path
                            )
                        );
                        return 0;
                    })
                    .ok();
                println!(
                    "copied file: {} -> {}",
                    &photo.og_path.display(),
                    &photo.db_path.display()
                );
                1
            })
            .collect::<Vec<u64>>()
            .par_iter()
            .sum();
        println!("Copied {}/{} files", copied, copy_list.len());
    } else {
        copied = copy_list
            .par_iter()
            .map(|photo| {
                println!(
                    "mock copied file: {} -> {}",
                    &photo.og_path.display(),
                    &photo.db_path.display()
                );
                1
            })
            .collect::<Vec<u64>>()
            .par_iter()
            .sum();
    }
    println!("Copied {}/{} files", copied, copy_list.len());
}

fn get_img_file_list(path_to_import: &PathBuf) -> Vec<PathBuf> {
    let options: MatchOptions = Default::default();
    let img_files: Vec<_> =
        glob_with(path_to_import.join("**/*").as_os_str().to_str().expect("join"), options)
            .unwrap()
            .filter_map(|x| x.ok())
            .filter_map(|path| is_image_file(&path).then_some(path))
            .collect();
    img_files
}

fn get_photos_from_img_file_list(img_files: &Vec<PathBuf>, import_path: &PathBuf) -> Vec<Photo> {
    let photo_vec: Vec<Photo> = img_files
        .par_iter()
        .filter_map(|path| {
            fs::read(path)
                .map_err(|e| {
                    println!(
                        "{}",
                        PhotoDBError::new(format!("reading file: {}", e).as_str(), &path)
                    );
                })
                .as_ref()
                .ok()
                .and_then(|buf| Photo::new(buf, path, import_path).ok())
            //Some(Photo::new(&buf, &path, import_path))
        })
        .collect();
    photo_vec
}

fn main() {
    let args = Cli::parse();
    let db_path = build_config_path(&args.db_root);
    if args.create {
        fs::create_dir_all(db_path.parent().unwrap()).unwrap();
        let mut conn = Connection::open(&db_path).unwrap();
        db::create_table(&mut conn);
    }
    import_directory(&args.path, &args.db_root, args.move_files, args.insert, &db_path)
}
