use crate::{hash, photo::Photo};
use chrono::{DateTime, Datelike, Utc};
use dateparser::parse;
use std::{
    error::Error,
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

const EXIF_DATE_KEYS: [&str; 3] =
    ["Exif.Photo.DateTimeOriginal", "Exif.Photo.DateTimeDigitized", "Exif.Image.DateTime"];

pub fn get_date(exif: &rexiv2::Metadata) -> Option<DateTime<Utc>> {
    for key in EXIF_DATE_KEYS.iter() {
        exif.get_tag_string(*key).ok().and_then(|date| {
            return match parse(&date) {
                Ok(date) => Some(date),
                Err(e) => {
                    println!("Warning: error parsing date {} -> {}", date, e);
                    None
                }
            };
        });
    }
    None
}

pub fn get_file_info(
    buf: &Vec<u8>, path: &PathBuf, import_path: &PathBuf,
) -> Result<Photo, Box<dyn Error>> {
    let hash = match hash::read_hash_image(&buf) {
        Ok(hash) => hash,
        Err(e) => {
            return Err(e);
        }
    };

    //let mut bufreader = Cursor::new(buf);
    //let exifreader = exif::Reader::new();
    let exif = match rexiv2::Metadata::new_from_buffer(buf) {
        Ok(exif) => Some(exif),
        Err(e) => {
            println!("Warning: error reading exif data {} -> {}", path.display(), e);
            None
        }
    };

    let model: String = exif
        .as_ref()
        .and_then(|ex| ex.get_tag_string("Model").ok())
        .and_then(|model| Some(model.replace("\"", "").replace(",", "").trim().to_string()))
        .unwrap_or("unknown".to_string());

    let date_tuple = exif
        .as_ref()
        .and_then(|exif| get_date(&exif))
        .and_then(|d| Some((d.year(), d.month())))
        .unwrap_or((0, 0));

    let import_path_full = import_path
        .join(date_tuple.0.to_string())
        .join(date_tuple.1.to_string())
        .join(model.to_string())
        .join(path.file_name().unwrap().to_str().unwrap());

    return Ok(Photo {
        hash: hash,
        model: model,
        year: date_tuple.0,
        month: date_tuple.1,
        db_path: import_path_full,
        og_path: path.to_path_buf(),
    });
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
