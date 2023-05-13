use std::{fs, path::PathBuf};

use clap::{Parser, ValueEnum};
use glob::{glob_with, MatchOptions};
use photodb::{build_config_path, raw_photo::Photo};
use rayon::prelude::*;
use rusqlite::Connection;

/// Verify the contents of a photodb database
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// The database root to move files into
    #[clap(long, default_value = "photodb")]
    pub db_root: PathBuf,
    /// The path to the file or directory to read
    path: PathBuf,
    /// Mode to run in. Hash or File
    #[arg(value_enum)]
    mode: Mode,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    /// Run hashes and check the database
    Hash,
    /// Check for untracked files in the database root
    File,
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
    let args = Cli::parse();
    let db_path = &build_config_path(&args.db_root);
    if args.mode == Mode::Hash {
        verify_db(db_path);
    } else if args.mode == Mode::File {
        verify_files(db_path);
    }
}

fn verify_files(db_path: &PathBuf) {
    let options: MatchOptions = Default::default();
    let files: Vec<PathBuf> =
        glob_with(db_path.join("**/*").as_os_str().to_str().expect("join"), options)
            .unwrap()
            .filter_map(|x| x.ok())
            .collect();

    files
        .par_iter()
        .map(|file| {
            let conn = Connection::open(db_path).unwrap();
            let mut binding =
                conn.prepare("SELECT * FROM photodb WHERE imported_path LIKE ?1").unwrap();
            binding.query_row([format!("{}", db_path.to_str().unwrap())], |_| Ok({})).map_err(
                |_| {
                    println!("Error: file not found in database {}", file.display());
                },
            )
        })
        .collect::<Vec<_>>();
}
