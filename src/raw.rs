#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use core::slice;

pub struct RawImage {
    pub raw_data: Vec<u16>,
}

impl RawImage {
    pub fn new(buf: &[u8]) -> Option<Self> {
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
                    unsafe { libraw_close(libraw_data) };
                    return Some(Self { raw_data });
                }
                _ => None,
            },
            _ => None,
        }
    }
}
