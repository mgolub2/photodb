mod cli;
mod db;
mod hash;
mod image;
mod photo;
#[allow(dead_code)]
mod raw;

use clap::Parser;
use cli::{Cli, Mode};
use photo::Photo;
use std::{error::Error, fs, path::PathBuf};

use glob::{glob_with, MatchOptions};
use rayon::prelude::*;
use rusqlite::*;

use crate::image::{get_file_info, is_image_file, write_to_path};

fn import_directory(
    path_to_import: &PathBuf,
    import_path: &PathBuf,
    move_file: bool,
    insert: bool,
    database: &PathBuf,
) -> Result<u64, Box<dyn Error>> {
    if !path_to_import.is_dir() {
        println!("{} is not a directory", path_to_import.display());
        return Err("Not a directory".into());
    }
    let options: MatchOptions = Default::default();

    let img_files: Vec<_> = glob_with(
        path_to_import
            .join("**/*")
            .as_os_str()
            .to_str()
            .expect("join"),
        options,
    )?
    .filter_map(|x| x.ok())
    .filter_map(|path| is_image_file(&path).then_some(path))
    .collect();

    println!("Importing {} files", img_files.len());
    let imported_count = img_files
        .par_iter()
        .map(|path| {
            let buf = match fs::read(&path) {
                Ok(buf) => buf,
                Err(e) => {
                    println!("Error reading file {}: {}", path.display(), e);
                    return 0;
                }
            };
            let photo = get_file_info(&buf, &path, import_path);
            if let Some(metadata) = photo {
                let mut conn = Connection::open(database).unwrap();
                let mut do_insert = insert;
                if !db::is_imported(metadata.hash, &mut conn) {
                    let moved = match move_file {
                        true => match write_to_path(buf.clone().as_mut(), &metadata.db_path) {
                            Ok(_) => true,
                            Err(e) => {
                                println!("Error moving image: {}", e);
                                do_insert = false;
                                false
                            }
                        },
                        false => true,
                    };
                    let inserted = match do_insert {
                        true => match db::insert_file_to_db(&metadata, &mut conn) {
                            Ok(_) => true,
                            Err(e) => {
                                println!("Error inserting image: {}", e);
                                false
                            }
                        },
                        false => true,
                    };
                    println!("{} -> {}", path.display(), metadata.db_path.display());
                    return (inserted && moved) as u64;
                } else {
                    println!(
                        "Image already imported: {} -> {:#x}",
                        path.display(),
                        metadata.hash
                    );
                    return 0;
                }
            } else {
                println!("Unable to hash file: {}", path.display());
                return 0;
            }
        })
        .count();
    Ok(imported_count as u64)
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
    photos
        .par_iter()
        .for_each(|photo| match photo.db_path.exists() {
            true => {
                let hash = match fs::read(&photo.db_path) {
                    Ok(buf) => hash::read_hash_image(&buf),
                    Err(e) => {
                        println!("Error reading file: {}", e);
                        0
                    }
                };
                if hash != photo.hash {
                    println!(
                        "Hash mismatch on {} : {:#x} file != {:#x} db",
                        &photo.db_path.display(),
                        hash,
                        photo.hash
                    );
                } else {
                    println!("verified: {} -> {:#x}", photo.db_path.display(), hash);
                }
            }
            false => {
                println!("File not found: {}", photo.db_path.display());
            }
        });
    println!("Done verifying {} photos", photos.len());
}

fn main() {
    let args = Cli::parse();

    if args.create {
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
                Ok(v) => println!("Imported {} files", v),
                Err(e) => println!("Error importing files: {}", e),
            }
        }
        Mode::Verify => {
            verify_db(&args.database);
        }
    }
    0;
}
