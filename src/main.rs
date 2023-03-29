use chrono::{Datelike, NaiveDate};
use std::{
    ffi::OsStr,
    fs::{self},
    io::{Cursor, Write},
    path::{self, PathBuf},
};

use exif::{In, Tag};
use glob::glob;
use libraw::Processor;
use rusqlite::*;
use xxhash_rust::xxh3::Xxh3;

const SEED: u64 = 0xdeadbeef;

#[cfg(not(debug_assertions))]
const DEBUG: bool = false;
#[cfg(debug_assertions)]
const DEBUG: bool = true;

use clap::{Parser, Subcommand};

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Mode to run
    #[clap(subcommand)]
    mode: Mode,
    /// The database root to move files into
    #[clap(long, default_value = "photodb")]
    import_path: PathBuf,
    /// Move the files to the database root
    #[clap(short, long, default_value_t = false)]
    move_files: bool,
    /// Import the files into the database, checking for duplicates
    #[clap(short, long, default_value_t = false)]
    insert: bool,
    /// The name of the database to use
    #[clap(short, long, default_value = ":memory:")]
    database: PathBuf,
    //// Create the database
    #[clap(short, long, default_value_t = false)]
    create: bool,
}

#[derive(Subcommand)]
enum Mode {
    /// Import files into the database
    Import {
        /// The path to the file or directory to read
        path: Option<PathBuf>,
    },
    //// Verify the raw file using the database
    Verify,
}

struct Photo {
    hash: i128,
    model: String,
    year: i32,
    month: u32,
    db_path: PathBuf,
    og_path: PathBuf,
}

fn read_hash_image(buf: &Vec<u8>) -> i128 {
    let processor = Processor::new();
    match processor.decode(&buf) {
        Ok(decoded) => {
            let mut xxh: Xxh3 = Xxh3::with_seed(SEED);
            for u16 in decoded.iter() {
                let u8_bytes = u16.to_be_bytes();
                let u8_arr = [u8_bytes[0], u8_bytes[1]];
                xxh.update(&u8_arr);
            }
            return xxh.digest128() as i128;
        }
        Err(e) => {
            println!("\tError: {}", e);
            return 0;
        }
    }
}

fn get_file_info(buf: &Vec<u8>, path: &PathBuf, import_path: &PathBuf) -> Option<Photo> {
    let hash = read_hash_image(&buf);
    if hash == 0 {
        return None;
    }

    let mut bufreader = Cursor::new(buf);
    let exifreader = exif::Reader::new();
    let exif = exifreader
        .read_from_container(&mut bufreader)
        .expect("Reading exif");
    let model = exif
        .get_field(Tag::Model, In::PRIMARY)
        .expect("model")
        .display_value()
        .to_string()
        .replace("\"", "")
        .replace(",", "")
        .trim()
        .to_string();
    let date = exif
        .get_field(Tag::DateTimeOriginal, In::PRIMARY)
        .expect("date")
        .display_value()
        .to_string();
    let year_month_day = NaiveDate::parse_from_str(&date, "%Y-%m-%d %H:%M:%S").expect("date");
    let model_date = (year_month_day.year(), year_month_day.month());

    if DEBUG {
        println!("\thash: {}", hash);
        println!("\tmodel: {}", model);
        println!("\tdate: {} {}", model_date.0, model_date.1);
    }

    let import_path_full = import_path
        .join(model_date.0.to_string())
        .join(model_date.1.to_string())
        .join(model.to_string())
        .join(path.file_name().unwrap().to_str().unwrap());

    return Some(Photo {
        hash: hash,
        model: model,
        year: model_date.0,
        month: model_date.1,
        db_path: import_path_full,
        og_path: path.to_path_buf(),
    });
}

fn is_image_file(path: &path::Path) -> bool {
    if path.is_file() {
        return match path
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "3fr" | "arw" | "cr2" | "fff" | "mef" | "mos" | "iiq" | "nef" | "tif" | "tiff"
            | "raf" | "rw2" | "dng" => true,
            _ => false,
        };
    } else {
        return false;
    }
}

// fn is_image_file(path: &path::Path) -> bool {
//     if !path.is_file() {
//       return false;
//     }

//     let matches_conditions = |s| { matches!(s, "3fr" | "arw" | "cr2" | "fff" | "mef" | "mos" |"iiq" | "nef" | "tif" | "tiff" | "raf" | "rw2" | "dng") };

//     path.extension().and_then(OsStr::to_str).map(str::to_lowercase).map(|s| s.as_str(.cl()).map(matches_conditions).unwrap_or(false)
//   }

fn write_to_path(buf: &mut Vec<u8>, path: &PathBuf) {
    //write buf to path
    match fs::create_dir_all(path.parent().unwrap()) {
        Ok(_) => match fs::File::create(path) {
            Ok(mut file) => {
                file.write_all(buf).expect("write file");
            }
            Err(e) => {
                println!("Error writing file: {}", e);
            }
        },
        Err(e) => {
            println!(
                "Error creating directory {}: {}",
                path.parent().unwrap().display(),
                e
            );
        }
    }
}

fn import_file(
    path: &PathBuf,
    import_path: &PathBuf,
    move_file: bool,
    insert: bool,
    con: &mut Connection,
) {
    let buf = fs::read(path).expect("read in");
    match get_file_info(&buf, path, import_path) {
        Some(metadata) => {
            if !is_imported(metadata.hash, con) {
                if move_file {
                    write_to_path(buf.clone().as_mut(), &metadata.db_path);
                }
                if insert {
                    match insert_file_to_db(&metadata, con) {
                        Ok(_) => {
                            if DEBUG {
                                println!("Inserted file: {}", metadata.db_path.display())
                            }
                        }
                        Err(e) => println!("Error inserting file: {}", e),
                    }
                }
                println!("{} -> {}", path.display(), metadata.db_path.display());
            }
        }
        None => {
            println!("Unable to hash file: {}", path.display());
        }
    }
}

fn create_table(con: &mut Connection) {
    let query = "
        CREATE TABLE photodb (hash BLOB UNIQUE, original_path TEXT, imported_path TEXT, year INTEGER, month INTEGER, model TEXT);
    ";

    match con.execute(query, ()) {
        Ok(_) => println!("Created database table for photodb."),
        Err(e) => println!("Error creating table: {}", e),
    }
}

fn is_imported(hash: i128, con: &mut Connection) -> bool {
    let mut stmt = con
        .prepare("SELECT * FROM photodb WHERE hash = :hash")
        .expect("conn failed");
    let mut rows = stmt
        .query(named_params! { ":hash": hash })
        .expect("rows failed");
    let row = rows.next().expect("query failed");
    if DEBUG {
        println!("row: {:?}", row);
    }
    return match row {
        Some(_) => true,
        None => false,
    };
}

fn insert_file_to_db(metadata: &Photo, conn: &mut Connection) -> Result<()> {
    let mut stmt = conn.prepare(
         "INSERT INTO photodb (hash, original_path, imported_path, year, month, model) VALUES (:hash, :og_path, :imprt_path, :year, :month, :model)").unwrap();
    stmt.execute(named_params! {
        ":hash": metadata.hash,
        ":og_path" : metadata.og_path.to_str().unwrap(),
        ":imprt_path" : metadata.db_path.to_str().unwrap(),
        ":year" : metadata.year,
        ":month" : metadata.month,
        ":model" : metadata.model,
    })?;
    Ok(())
}

fn import_directory(
    path_to_import: &PathBuf,
    import_path: &PathBuf,
    move_file: bool,
    insert: bool,
    con: &mut Connection,
) {
    for entry in glob(
        path_to_import
            .join("**/*")
            .into_os_string()
            .to_str()
            .expect("join"),
    )
    .expect("bad pattern")
    {
        if let Ok(path) = entry {
            if DEBUG {
                println!("path: {}", path.display());
            }
            if is_image_file(&path) {
                import_file(&path, &import_path, move_file, insert, con);
            }
        } else {
            println!("Error reading path {}: {}", &path_to_import.display(), entry.unwrap_err());
        }
    }
}


fn verify_db(
    con: &mut Connection,
) {
    let mut stmt = con.prepare("SELECT * FROM photodb").unwrap();
    let rows = stmt.query_map([], |row| {
        Ok(Photo {
            hash: row.get(0)?,
            og_path: PathBuf::from(row.get::<_, String>(1)?),
            db_path: PathBuf::from(row.get::<_, String>(2)?),
            year: row.get(3)?,
            month: row.get(4)?,
            model: row.get(5)?,
        })
    }).unwrap();

    for row in rows {
        let photo = row.unwrap();
        match photo.db_path.exists() {
            true => {
                let buf = fs::read(&photo.db_path).expect("read in");
                let hash = read_hash_image(&buf);
                if hash != photo.hash {
                    println!("Hash mismatch on {} : {} file != {} db", &photo.db_path.display(), hash, photo.hash);
                } else {
                    println!("Verified: {} -> {}", &photo.db_path.display(), hash);
                }
            }
            false => {
                println!("File not found: {}", photo.db_path.display());
            }
        } 

    }
}


fn main() {
    let args = Cli::parse();
    let mut conn = Connection::open(args.database).unwrap();
    if args.create {
        create_table(&mut conn);
    }
    match &args.mode {
        Mode::Import { path } => {
            import_directory(
                &path.clone().expect("path"),
                &args.import_path,
                args.move_files,
                args.insert,
                &mut conn,
            );
        }
        Mode::Verify => {
            verify_db(&mut conn);
        }
    }
}
