#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use core::slice;
use std::error::Error;

pub struct RawImage {
    pub raw_data: Vec<u16>,
    pub year: i32,
    pub month: u32,
    pub make: String,
}

impl RawImage {
    pub fn new(buf: &[u8]) -> Result<Self, Box<dyn Error>> {
        let libraw_data = unsafe { libraw_init(0) };
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
                    let make = unsafe {
                        let make = (*libraw_data).idata.make.as_ptr() as *const u8;
                        let make_len = (*libraw_data).idata.make.len();
                        let make_slice = slice::from_raw_parts(make, make_len);
                        String::from_utf8_lossy(make_slice).to_string()
                    };
                    unsafe { libraw_close(libraw_data) };
                    return Ok(Self { raw_data, year: 0, month: 0, make });
                }
                _ => Err("libraw_unpack failed".into()),
            },
            _ => Err("libraw_open_buffer failed".into()),
        }
    }
}
