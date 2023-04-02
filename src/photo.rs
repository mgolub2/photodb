use std::path::PathBuf;

pub(crate) struct Photo {
    pub(crate) hash: i128,
    pub(crate) model: String,
    pub(crate) year: i32,
    pub(crate) month: u32,
    pub(crate) db_path: PathBuf,
    pub(crate) og_path: PathBuf,
}
