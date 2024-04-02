use chrono::{DateTime, NaiveDateTime, Utc};
use rusqlite::Connection;
use std::{
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

const EXIF_DATE_KEYS: [&str; 3] =
    ["Exif.Photo.DateTimeOriginal", "Exif.Photo.DateTimeDigitized", "Exif.Image.DateTime"];

// A-A-ATA:A:A
// A-A-ATA:A:AZ
// A:A:A
// A:A:A A:A
// A:A:A A:A:A
const EXIF_DATE_F_STR: [&str; 7] = [
    "%Y-%m-%d %H:%M:%S",
    "%Y:%m:%d %H:%M:%S",
    "%Y:%m:%dT%H:%M:%S",
    "%Y-%m-%dT%H:%M:%S",
    "%Y-%m-%dT%H:%M:%SZ",
    "%Y:%m:%d %H:%M",
    "$Y-%m-%d",
];

pub fn get_date(exif: &rexiv2::Metadata) -> Option<DateTime<Utc>> {
    let parse_from_str = NaiveDateTime::parse_from_str;
    for key in EXIF_DATE_KEYS.iter() {
        match exif.get_tag_string(*key).ok().and_then(|date| {
            EXIF_DATE_F_STR
                .iter()
                .find_map(|f| parse_from_str(date.as_str(), *f).ok())
                .and_then(|date| Some(DateTime::from_naive_utc_and_offset(date, Utc)))
        }) {
            Some(date) => return Some(date),
            None => continue,
        }
    }
    None
}

pub fn is_image_file(path: &Path) -> bool {
    Some(path.is_file() && !path.starts_with("."))
        .and_then(|_| path.extension().and_then(OsStr::to_str))
        .and_then(|f| match f.to_lowercase().as_str() {
            "3fr" | "arw" | "cr2" | "fff" | "mef" | "mos" | "iiq" | "nef" | "raf" | "rw2"
            | "dng" => Some(true),
            _ => Some(false),
        })
        .unwrap_or(false)
}

pub fn write_to_path(buf: &mut Vec<u8>, path: &PathBuf) -> Result<(), std::io::Error> {
    //write buf to path
    match fs::create_dir_all(path.parent().unwrap()) {
        Ok(_) => {
            let mut file = fs::File::create(path)?;
            return file.write_all(buf);
        }
        Err(e) => {
            println!("Error creating directory {}: {}", path.parent().unwrap().display(), e);
            return Err(e);
        }
    }
}

pub fn build_final_path(
    db_root: &PathBuf, model: &String, year: &i32, month: &u32, og_path: &PathBuf,
) -> PathBuf {
    db_root
        .join(year.to_string())
        .join(month.to_string())
        .join(model.to_string())
        .join(og_path.file_name().unwrap())
}

pub fn get_db_con(db_path: &PathBuf) -> Connection {
    let con: Connection = Connection::open(db_path).expect("conn failed");
    return con;
}
