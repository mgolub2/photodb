extern crate photodb;
use clap::Parser;
use photodb::cli;
use photodb::cli::Mode;
use photodb::db;
use photodb::photo;

use glob::{glob_with, MatchOptions};
use photo::Photo;
use photodb::raw::RawImage;
use rayon::prelude::*;
use rusqlite::*;
use std::{error::Error, fs, path::PathBuf};

use crate::photo::{get_file_info, is_image_file, write_to_path};

fn import_directory(
    path_to_import: &PathBuf, import_path: &PathBuf, move_file: bool, insert: bool,
    database: &PathBuf,
) -> Result<(u64, u64, u64, u64), Box<dyn Error>> {
    if !path_to_import.is_dir() {
        println!("{} is not a directory", path_to_import.display());
        return Err("Not a directory".into());
    }
    let options: MatchOptions = Default::default();

    let img_files: Vec<_> =
        glob_with(path_to_import.join("**/*").as_os_str().to_str().expect("join"), options)?
            .filter_map(|x| x.ok())
            .filter_map(|path| is_image_file(&path).then_some(path))
            .collect();

    let total_files = img_files.len();
    println!("Importing {} files", total_files);
    let imported_count: (u64, u64, u64) = img_files
        .par_iter()
        .map(|path| {
            let buf = match fs::read(&path) {
                Ok(buf) => buf,
                Err(e) => {
                    println!("Error: reading file {} -> {}", path.display(), e);
                    return (0, 0, 1);
                }
            };
            match get_file_info(&buf, &path, import_path) {
                Ok(metadata) => {
                    let mut conn = Connection::open(database).unwrap();
                    let mut do_move = move_file;
                    if !db::is_imported(metadata.hash, &mut conn) {
                        let inserted = match insert {
                            true => match db::insert_file_to_db(&metadata, &mut conn) {
                                Ok(_) => true,
                                Err(e) => {
                                    println!("Error: inserting image {} -> {}", path.display(), e);
                                    do_move = false;
                                    false
                                }
                            },
                            false => true,
                        };
                        let moved = match do_move {
                            true => match write_to_path(buf.clone().as_mut(), &metadata.db_path) {
                                Ok(_) => true,
                                Err(e) => {
                                    println!("Error: moving image {} -> {}", path.display(), e);
                                    false
                                }
                            },
                            false => inserted,
                        };
                        println!("{} -> {}", path.display(), metadata.db_path.display());
                        return ((inserted && moved) as u64, 0, !(inserted && moved) as u64);
                    } else {
                        println!(
                            "Image already imported: {} -> {:#x}",
                            path.display(),
                            metadata.hash
                        );
                        return (0, 1, 0);
                    }
                }
                Err(e) => {
                    println!("Error: unable to hash file: {} -> {}", path.display(), e);
                    return (0, 0, 1);
                }
            }
        })
        .reduce(|| (0, 0, 0), |(a, b, c), (d, e, f)| (a + d, b + e, c + f));
    Ok((total_files as u64, imported_count.0, imported_count.1, imported_count.2))
}

fn verify_db(database: &PathBuf) {
    let conn = Connection::open(database).unwrap();
    let mut stmt = conn.prepare("SELECT * FROM photodb").unwrap();
    let rows = stmt
        .query_map([], |row| {
            Ok(Photo {
                hash: row.get(0)?,
                og_path: PathBuf::from(row.get::<_, String>(1)?),
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
                Ok(buf) => match RawImage::new(&buf) {
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

    if args.create {
        // create the path if it doesn't exist
        if !args.database.exists() {
            fs::create_dir_all(&args.database.parent().unwrap()).unwrap();
        }
        let mut conn = Connection::open(&args.database).unwrap();
        db::create_table(&mut conn);
    }
    match &args.mode {
        Mode::Import { path } => {
            match import_directory(
                &path.clone().expect("path"),
                &args.import_path,
                args.move_files,
                args.insert,
                &args.database,
            ) {
                Ok(v) => println!(
                    "Imported {}/{} files. {} already imported. {} errors.",
                    v.1, v.0, v.2, v.3
                ),
                Err(e) => println!("Error importing files: {}", e),
            }
        }
        Mode::Verify => {
            verify_db(&args.database);
        }
    }
    0;
}
