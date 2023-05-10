extern crate photodb;
use clap::Parser;
use photodb::cli::Mode;
use photodb::photodb_error::PhotoDBError;
use photodb::{build_config_path, db, util};
use photodb::{cli, raw_photo::exit};

use glob::{glob_with, MatchOptions};
use photodb::raw_photo::Photo;
use rayon::prelude::*;
use rusqlite::*;
use std::{fs, path::PathBuf};

use crate::util::is_image_file;

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

fn verify_db(database: &PathBuf) {
    let conn = Connection::open(database).unwrap();
    let mut stmt = conn.prepare("SELECT * FROM photodb").unwrap();
    let rows = stmt
        .query_map([], |row| {
            Ok(Photo {
                hash: row.get(0)?,
                og_path: PathBuf::from(row.get::<_, String>(1)?),
                db_root: PathBuf::new(),
                db_path: PathBuf::from(row.get::<_, String>(2)?),
                year: row.get(3)?,
                month: row.get(4)?,
                model: row.get(5)?,
            })
        })
        .unwrap();
    let photos = rows.collect::<Result<Vec<_>, _>>().unwrap();
    photos.par_iter().for_each(|photo| match photo.db_path.exists() {
        true => {
            let hash = match fs::read(&photo.db_path) {
                Ok(buf) => match Photo::new(&buf, &photo.og_path, &photo.db_root) {
                    Ok(raw_image) => raw_image.hash,
                    Err(e) => {
                        println!("Error: calculating hash {} -> {}", &photo.og_path.display(), e);
                        0
                    }
                },
                Err(e) => {
                    println!("Error: reading file {} -> {}", &photo.og_path.display(), e);
                    0
                }
            };
            if hash != photo.hash {
                println!(
                    "Error: hash mismatch on {} -> {:#x} file != {:#x} db",
                    &photo.db_path.display(),
                    hash,
                    photo.hash
                );
            } else {
                println!("Verified: {} -> {:#x}", photo.db_path.display(), hash);
            }
        }
        false => {
            println!("Error: file not found {} -> ???", photo.db_path.display());
        }
    });
    println!("Done verifying {} photos", photos.len());
}

fn main() {
    let args = cli::Cli::parse();
    let db_path = build_config_path(&args.db_root);
    if args.create {
        fs::create_dir_all(db_path.parent().unwrap()).unwrap();
        let mut conn = Connection::open(&db_path).unwrap();
        db::create_table(&mut conn);
    }
    match &args.mode {
        Mode::Import { path } => import_directory(
            &path.clone().expect("path"),
            &args.db_root,
            args.move_files,
            args.insert,
            &db_path,
        ),
        Mode::Verify => {
            verify_db(&db_path);
        }
    }
    0;
}
