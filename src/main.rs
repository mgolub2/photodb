#![feature(thread_id_value)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub struct RawImage {
    //libraw_data: *mut libraw_data_t,
    raw_data: Vec<u16>,
}

impl RawImage {
    pub fn new(buf: &[u8]) -> Option<Self> {
        let libraw_data = unsafe { libraw_init(0) };
        match unsafe { libraw_open_buffer(libraw_data, buf.as_ptr() as *const _, buf.len()) } {
            LibRaw_errors_LIBRAW_SUCCESS => match unsafe { libraw_unpack(libraw_data) } {
                LibRaw_errors_LIBRAW_SUCCESS => {
                    let raw_image = unsafe { (*libraw_data).rawdata.raw_image };
                    let raw_image_size = unsafe {
                        (*libraw_data).sizes.iwidth as usize * (*libraw_data).sizes.iheight as usize
                    };
                    let raw_image_slice =
                        unsafe { slice::from_raw_parts(raw_image, raw_image_size) };
                    let raw_data = unsafe { raw_image_slice.align_to::<u8>().2.to_vec() };
                    unsafe { libraw_close(libraw_data) };
                    return Some(Self { raw_data });
                }
                _ => None,
            },
            _ => None,
        }
    }
}

use chrono::{Datelike, NaiveDate, ParseError};

use core::slice;
use std::{
    error::Error,
    ffi::OsStr,
    fs::{self},
    io::{Cursor, Write},
    path::{self, PathBuf},
    thread,
};

use exif::{In, Tag};
use glob::{glob_with, MatchOptions};
//use libraw::Processor;
use rayon::prelude::*;
use rusqlite::*;
use xxhash_rust::xxh3::Xxh3;

const SEED: u64 = 0xdeadbeef;

#[cfg(not(debug_assertions))]
const DEBUG: bool = false;
#[cfg(debug_assertions)]
const DEBUG: bool = true;

use clap::{Parser, Subcommand};

// #[macro_export]
// macro_rules! print_log {
//     ($log:ident, $($arg:tt)*) => {
//         let line = format!($($arg)*);
//         println!("{}",line);
//         $log.write(format!("{} >>> ", rstime::now()).as_bytes()).expect("msg_write_log");
//         $log.write(line.as_bytes()).expect("msg_write_log");
//         $log.write(b"\n").expect("msg_write_log");
//         $log.flush().expect("msg_flush_log");
//     };
// }

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
    /// Create the database
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
    /// Verify the raw image file hashes
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
    let image = RawImage::new(buf);
    let mut xxh: Xxh3 = Xxh3::with_seed(SEED);
    match image {
        Some(image) => {
            for u16 in image.raw_data.iter() {
                xxh.update(&u16.to_le_bytes());
            }
            return xxh.digest128() as i128;
        }
        None => return 0,
    }
}

fn get_date(exif: &exif::Exif) -> Result<NaiveDate, ParseError> {
    let exif_date_keys = [Tag::DateTimeOriginal, Tag::DateTime];
    //let format_strs = ["%Y-%m-%d %H:%M:%S", ];
    for key in exif_date_keys.iter() {
        if let Some(date) = exif.get_field(*key, In::PRIMARY) {
            if DEBUG {
                println!("Found date: {}", date.display_value().to_string());
            }
            return NaiveDate::parse_from_str(
                &date.display_value().to_string(),
                "%Y-%m-%d %H:%M:%S",
            );
        }
    }
    panic!("No date found!");
}

fn get_file_info(buf: &Vec<u8>, path: &PathBuf, import_path: &PathBuf) -> Option<Photo> {
    if DEBUG {
        println!(
            "Getting file info for: {} (thread#{})",
            path.display(),
            thread::current().id().as_u64()
        );
    }
    let hash = read_hash_image(&buf);
    if hash == 0 {
        return None;
    }

    let mut bufreader = Cursor::new(buf);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader);

    let model: String = match &exif {
        Ok(ex) => match ex.get_field(Tag::Model, In::PRIMARY) {
            Some(model) => model
                .display_value()
                .to_string()
                .replace("\"", "")
                .replace(",", "")
                .trim()
                .to_string(),
            None => "unknown".to_string(),
        },
        Err(_) => {
            if DEBUG {
                println!("No exif data found for: {}", path.display());
            }
            "unknown".to_string()
        }
    };
    let date_tuple: (i32, u32) = match &exif {
        Ok(exif) => {
            let date = get_date(&exif);
            match date {
                Ok(date) => (date.year(), date.month()),
                Err(_) => (0, 0),
            }
        }
        Err(_) => (0, 0),
    };

    if DEBUG {
        println!("\thash: {}", hash);
        println!("\tmodel: {}", model);
        println!("\tdate: {} {}", date_tuple.0, date_tuple.1);
    }

    let import_path_full = import_path
        .join(date_tuple.0.to_string())
        .join(date_tuple.1.to_string())
        .join(model.to_string())
        .join(path.file_name().unwrap().to_str().unwrap());

    return Some(Photo {
        hash: hash,
        model: model,
        year: date_tuple.0,
        month: date_tuple.1,
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
            "3fr" | "arw" | "cr2" | "fff" | "mef" | "mos" | "iiq" | "nef" | "raf" | "rw2"
            | "dng" => true,
            _ => false,
        };
    } else {
        return false;
    }
}

fn write_to_path(buf: &mut Vec<u8>, path: &PathBuf) -> Result<(), std::io::Error> {
    //write buf to path
    match fs::create_dir_all(path.parent().unwrap()) {
        Ok(_) => {
            let mut file = fs::File::create(path)?;
            return file.write_all(buf);
        }
        Err(e) => {
            println!(
                "Error creating directory {}: {}",
                path.parent().unwrap().display(),
                e
            );
            return Err(e);
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
    database: &PathBuf,
    //log: &mut BufWriter<fs::File>,
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
            let thread_num = thread::current().id().as_u64();
            if let Some(metadata) = photo {
                let mut conn = Connection::open(database).unwrap();
                if DEBUG {
                    println!("Checking {}... \t(thread#{})", path.display(), thread_num);
                }
                let mut do_insert = insert;
                if !is_imported(metadata.hash, &mut conn) {
                    if move_file {
                        match write_to_path(buf.clone().as_mut(), &metadata.db_path) {
                            Ok(_) => {
                                if DEBUG {
                                    println!(
                                        "Moved image: {} \t(thread#{})",
                                        metadata.db_path.display(),
                                        thread_num
                                    )
                                }
                            }
                            Err(e) => {
                                println!("Error moving image: {} \t(thread#{})", e, thread_num);
                                do_insert = false;
                            }
                        }
                    }
                    if do_insert {
                        match insert_file_to_db(&metadata, &mut conn) {
                            Ok(_) => {
                                if DEBUG {
                                    println!(
                                        "Inserted image: {} \t(thread#{})",
                                        metadata.db_path.display(),
                                        thread_num
                                    )
                                }
                            }
                            Err(e) => {
                                println!("Error inserting image: {} \t(thread#{})", e, thread_num);
                                return 0;
                            }
                        }
                    }
                    println!("{} -> {}", path.display(), metadata.db_path.display());
                    return 1;
                } else {
                    println!(
                        "Image already imported: {} \t(thread#{})",
                        path.display(),
                        thread_num
                    );
                    return 0;
                }
            } else {
                println!(
                    "Unable to hash file: {} \t(thread#{})\n",
                    path.display(),
                    thread_num
                );
                return 0;
            }
        })
        .count();
    //log.flush().expect("flush log");
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
                    Ok(buf) => read_hash_image(&buf),
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
                    println!(
                        "verified: {} -> {:#x} \t(thread#{})",
                        photo.db_path.display(),
                        hash,
                        thread::current().id().as_u64()
                    );
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
        create_table(&mut conn);
    }
    match &args.mode {
        Mode::Import { path } => {
            // let log_file = fs::File::create(
            //     &path
            //         .clone()
            //         .expect("path")
            //         .join(format!("photodb_import_{}.log", rstime::now())),
            // )
            // .expect("create log file");
            //let mut log = BufWriter::new(log_file);
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
