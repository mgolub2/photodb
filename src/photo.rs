use std::path::PathBuf;

pub struct Photo {
    pub hash: i128,
    pub model: String,
    pub year: i32,
    pub month: u32,
    pub db_path: PathBuf,
    pub og_path: PathBuf,
}
