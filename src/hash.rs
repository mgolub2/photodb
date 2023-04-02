use crate::raw;
use xxhash_rust::xxh3::Xxh3;
const SEED: u64 = 0xdeadbeef;

pub(crate) fn read_hash_image(buf: &Vec<u8>) -> i128 {
    let image = raw::RawImage::new(buf);
    let mut xxh: Xxh3 = Xxh3::with_seed(SEED);
    match image {
        Some(image) => {
            //let mut count = 0;
            for u16 in image.raw_data.iter() {
                xxh.update(&u16.to_le_bytes());
                //count += 2;
            }
            //println!("{} bytes", count);
            return xxh.digest128() as i128;
        }
        None => return 0,
    }
}
