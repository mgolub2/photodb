use crate::raw::RawImage;
use chrono::{DateTime, Datelike, NaiveDateTime, Utc};
use std::{
    error::Error,
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub struct Photo {
    pub hash: i128,
    pub model: String,
    pub year: i32,
    pub month: u32,
    pub db_path: PathBuf,
    pub og_path: PathBuf,
}

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
                .and_then(|date| Some(DateTime::from_utc(date, Utc)))
        }) {
            Some(date) => return Some(date),
            None => continue,
        }
    }
    None
}

pub fn get_file_info(
    buf: &Vec<u8>, path: &PathBuf, import_path: &PathBuf,
) -> Result<Photo, Box<dyn Error>> {
    let raw_image_data = match RawImage::new(&buf) {
        Ok(raw_image_data) => raw_image_data,
        Err(e) => {
            return Err(e);
        }
    };
    let exif = match rexiv2::Metadata::new_from_buffer(buf) {
        Ok(exif) => Some(exif),
        Err(e) => {
            println!("Warning: error reading exif data {} -> {}", path.display(), e);
            None
        }
    };

    let model: String = exif
        .as_ref()
        .and_then(|ex| ex.get_tag_string("Exif.Image.Model").ok())
        .unwrap_or(raw_image_data.make)
        .replace("\"", "")
        .replace(",", "")
        .trim()
        .to_string();

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
        hash: raw_image_data.hash,
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
