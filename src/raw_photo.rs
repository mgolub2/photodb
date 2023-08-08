#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use chrono::Datelike;
use core::slice;
use rexiv2::Metadata;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use xxhash_rust::xxh3::Xxh3;
use libraw_rs_vendor::{libraw_data_t, libraw_init, libraw_close, libraw_unpack, libraw_open_buffer, LibRaw_errors_LIBRAW_SUCCESS};

use crate::photodb_error::PhotoDBError;
use crate::util::{build_final_path, get_date};
const SEED: u64 = 0xdeadbeef;

#[derive(Clone, Debug)]
pub struct Photo {
    pub hash: i128,
    pub year: i32,
    pub month: u32,
    pub model: String,
    pub db_root: PathBuf,
    pub db_path: PathBuf,
    pub og_path: PathBuf,
}

impl PartialEq for Photo {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for Photo {}

impl Hash for Photo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl Photo {
    pub fn new(buf: &[u8], og_path: &PathBuf, db_root: &PathBuf) -> Result<Self, PhotoDBError> {
        let libraw_data = unsafe { libraw_init(0) };
        let raw_data = Self::read_raw_data(libraw_data, buf, og_path)?;
        let hash = Self::get_hash(&raw_data);
        let model = Self::get_model(libraw_data);
        let exif = Self::get_exif(buf, og_path);
        let exif_model = Self::get_exif_model(&exif);
        let date_tuple = Self::get_date_tuple(&exif); //.unwrap()) } else {(0, 0)};
        let final_model = if exif_model.is_empty() { model } else { exif_model };
        let import_path_full =
            build_final_path(db_root, &final_model, &date_tuple.0, &date_tuple.1, &og_path);
        unsafe { libraw_close(libraw_data) };
        Ok(Self {
            hash: hash,
            year: date_tuple.0,
            month: date_tuple.1,
            model: final_model,
            db_root: db_root.to_path_buf(),
            db_path: import_path_full.to_path_buf(),
            og_path: og_path.to_path_buf(),
        })
    }

    fn get_exif_model(exif: &Result<Metadata, PhotoDBError>) -> String {
        match exif {
            Ok(exif) => {
                let model = exif.get_tag_string("Exif.Image.Model");
                match model {
                    Ok(model) => model.replace("\"", "").replace(",", "").trim().to_string(),
                    Err(_) => "".to_string(),
                }
            }
            Err(_) => "".to_string(),
        }
    }

    fn get_date_tuple(exifrs: &Result<Metadata, PhotoDBError>) -> (i32, u32) {
        match exifrs {
            Ok(exif) => get_date(&exif).and_then(|d| Some((d.year(), d.month()))).unwrap_or((0, 0)),
            Err(_) => (0, 0),
        }
    }

    fn get_model(libraw_data: *mut libraw_data_t) -> String {
        unsafe {
            let make = (*libraw_data).idata.make.as_ptr() as *const u8;
            let make_len = (*libraw_data).idata.make.len();
            let make_slice = slice::from_raw_parts(make, make_len);
            String::from_utf8_lossy(make_slice).to_string()
        }
    }

    fn get_hash(raw_data: &Vec<u16>) -> i128 {
        let mut xxh: Xxh3 = Xxh3::with_seed(SEED);
        for u16 in raw_data.iter() {
            xxh.update(&u16.to_le_bytes());
        }
        xxh.digest128() as i128
    }

    fn read_raw_data(
        libraw_data: *mut libraw_data_t, buf: &[u8], og_path: &PathBuf,
    ) -> Result<Vec<u16>, PhotoDBError> {
        match unsafe { libraw_open_buffer(libraw_data, buf.as_ptr() as *const _, buf.len()) } {
            LibRaw_errors_LIBRAW_SUCCESS => match unsafe { libraw_unpack(libraw_data) } {
                LibRaw_errors_LIBRAW_SUCCESS => {
                    let raw_image = unsafe { (*libraw_data).rawdata.raw_alloc };
                    let raw_image_size = unsafe {
                        (*libraw_data).sizes.raw_height as usize
                            * (*libraw_data).sizes.raw_width as usize
                    };
                    let raw_image_slice =
                        unsafe { slice::from_raw_parts(raw_image as *mut u16, raw_image_size) };
                    let mut raw_data = Vec::with_capacity(raw_image_size);
                    raw_data.extend_from_slice(raw_image_slice);
                    Ok(raw_data)
                }
                _ => Err(PhotoDBError::new("libraw_unpack failed", og_path)),
            },
            _ => Err(PhotoDBError::new("libraw_open_buffer failed", og_path)),
        }
    }

    fn get_exif(buf: &[u8], og_path: &PathBuf) -> Result<rexiv2::Metadata, PhotoDBError> {
        match rexiv2::Metadata::new_from_buffer(&buf) {
            Ok(exif) => Ok(exif),
            Err(e) => {
                Err(PhotoDBError::new(format!("unable to read exif data: {}", e).as_str(), og_path))
            }
        }
    }
}
