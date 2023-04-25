use std::{path::PathBuf, error::Error, fmt};

#[derive(Debug, Clone)]
pub struct PhotoDBError {
    details: String,
    path: PathBuf,
}

impl PhotoDBError {
    pub fn new(msg: &str, path: &PathBuf) -> PhotoDBError {
        PhotoDBError{details: msg.to_string(), path: path.to_path_buf()}
    }
}

impl fmt::Display for PhotoDBError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {} -> {}", self.path.display(), self.details)
    }
}

impl Error for PhotoDBError {
    fn description(&self) -> &str {
        &self.details
    }
}