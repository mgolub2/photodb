use std::{
    ffi::OsStr,
    fs,
    io::{Cursor, Write},
    path::{Path, PathBuf},
};

use chrono::{Datelike, NaiveDate, ParseError};
use exif::{In, Tag};

use crate::{hash, photo::Photo};

pub(crate) fn get_date(exif: &exif::Exif) -> Result<NaiveDate, ParseError> {
    let exif_date_keys = [Tag::DateTimeOriginal, Tag::DateTime];
    //let format_strs = ["%Y-%m-%d %H:%M:%S", ];
    for key in exif_date_keys.iter() {
        if let Some(date) = exif.get_field(*key, In::PRIMARY) {
            return NaiveDate::parse_from_str(
                &date.display_value().to_string(),
                "%Y-%m-%d %H:%M:%S",
            );
        }
    }
    return NaiveDate::parse_from_str("fail", "%Y-%m-%d %H:%M:%S");
}

pub(crate) fn get_file_info(buf: &Vec<u8>, path: &PathBuf, import_path: &PathBuf) -> Option<Photo> {
    let hash = hash::read_hash_image(&buf);
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
        Err(_) => "unknown".to_string(),
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

pub(crate) fn is_image_file(path: &Path) -> bool {
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

pub(crate) fn write_to_path(buf: &mut Vec<u8>, path: &PathBuf) -> Result<(), std::io::Error> {
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
