use std::path::PathBuf;

use db::DB_PATH;

pub mod cli;
pub mod db;
pub mod photodb_error;
pub mod raw_photo;
pub mod util;

pub const CONFIG_ROOT: &str = ".photodb";

pub fn build_config_path(db_root: &PathBuf) -> PathBuf {
    db_root.join(CONFIG_ROOT).join(DB_PATH)
}
