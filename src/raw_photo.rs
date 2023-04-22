#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use chrono::Datelike;
use core::slice;
use std::{error::Error, path::PathBuf};
use xxhash_rust::xxh3::Xxh3;

use crate::util::get_date;
const SEED: u64 = 0xdeadbeef;

pub struct Photo {
    pub hash: i128,
    pub year: i32,
    pub month: u32,
    pub model: String,
    pub db_root: PathBuf,
    pub db_path: PathBuf,
    pub og_path: PathBuf,
}

impl Photo {
    pub fn new(buf: &[u8], og_path: &PathBuf, db_root: &PathBuf) -> Result<Self, Box<dyn Error>> {
        let libraw_data = unsafe { libraw_init(0) };
        let raw_data = Self::read_raw_data(libraw_data, buf)?;
        let hash = Self::get_hash(&raw_data);
        let make = Self::get_model(libraw_data);
        unsafe { libraw_close(libraw_data) };
        Ok(Self {
            hash: hash,
            year: 0,
            month: 0,
            model: make,
            db_root: db_root.to_path_buf(),
            db_path: PathBuf::new(),
            og_path: og_path.to_path_buf(),
        })
    }

    pub fn populate_exif_info(&mut self, buf: &[u8]) {
        let exif = match rexiv2::Metadata::new_from_buffer(&buf) {
            Ok(exif) => Some(exif),
            Err(e) => {
                println!("Warning: error reading exif data {} -> {}", self.og_path.display(), e);
                None
            }
        };

        let exif_model: String = exif
            .as_ref()
            .and_then(|ex| ex.get_tag_string("Exif.Image.Model").ok())
            .unwrap_or("".to_string())
            .replace("\"", "")
            .replace(",", "")
            .trim()
            .to_string();

        let date_tuple = exif
            .as_ref()
            .and_then(|exif| get_date(&exif))
            .and_then(|d| Some((d.year(), d.month())))
            .unwrap_or((0, 0));

        let import_path_full = self
            .db_root
            .join(date_tuple.0.to_string())
            .join(date_tuple.1.to_string())
            .join(exif_model.to_string())
            .join(self.og_path.file_name().unwrap().to_str().unwrap());
        self.month = date_tuple.1;
        self.year = date_tuple.0;
        self.db_path = import_path_full;
        if !exif_model.is_empty() {
            self.model = exif_model
        };
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
        libraw_data: *mut libraw_data_t, buf: &[u8],
    ) -> Result<Vec<u16>, Box<dyn Error>> {
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
                _ => Err("libraw_unpack failed".into()),
            },
            _ => Err("libraw_open_buffer failed".into()),
        }
    }
}
