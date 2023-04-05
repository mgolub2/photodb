use std::{
    error::Error,
    ffi::OsStr,
    fs,
    io::{Cursor, Write},
    path::{Path, PathBuf},
};

use chrono::{Datelike, NaiveDate};
use exif::{In, Tag};

use crate::{hash, photo::Photo};

pub(crate) fn get_date(exif: &exif::Exif) -> Option<NaiveDate> {
    let exif_date_keys = [Tag::DateTimeOriginal, Tag::DateTime];
    //let format_strs = ["%Y-%m-%d %H:%M:%S", ];
    for key in exif_date_keys.iter() {
        if let Some(date) = exif.get_field(*key, In::PRIMARY) {
            return match NaiveDate::parse_from_str(
                &date.display_value().to_string(),
                "%Y-%m-%d %H:%M:%S",
            ) {
                Ok(date) => Some(date),
                Err(e) => {
                    println!(
                        "Warning: error parsing date {} -> {}",
                        date.display_value(),
                        e
                    );
                    None
                }
            };
        }
    }
    None
}

pub(crate) fn get_file_info(
    buf: &Vec<u8>,
    path: &PathBuf,
    import_path: &PathBuf,
) -> Result<Photo, Box<dyn Error>> {
    let hash = match hash::read_hash_image(&buf) {
        Ok(hash) => hash,
        Err(e) => {
            return Err(e);
        }
    };

    let mut bufreader = Cursor::new(buf);
    let exifreader = exif::Reader::new();
    let exif = match exifreader.read_from_container(&mut bufreader) {
        Ok(exif) => Some(exif),
        Err(e) => {
            println!(
                "Warning: error reading exif data {} -> {}",
                path.display(),
                e
            );
            None
        }
    };

    let model: String = exif
        .as_ref()
        .and_then(|ex| ex.get_field(Tag::Model, In::PRIMARY).cloned())
        .and_then(|model| {
            Some(
                model
                    .display_value()
                    .to_string()
                    .replace("\"", "")
                    .replace(",", "")
                    .trim()
                    .to_string(),
            )
        })
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
