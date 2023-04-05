use std::error::Error;

use crate::raw;
use xxhash_rust::xxh3::Xxh3;
const SEED: u64 = 0xdeadbeef;

pub fn read_hash_image(buf: &Vec<u8>) -> Result<i128, Box<dyn Error>> {
    let image = raw::RawImage::new(buf);
    let mut xxh: Xxh3 = Xxh3::with_seed(SEED);
    match image {
        Ok(image) => {
            for u16 in image.raw_data.iter() {
                xxh.update(&u16.to_le_bytes());
            }
            return Ok(xxh.digest128() as i128);
        }
        Err(e) => return Err(e),
    }
}
