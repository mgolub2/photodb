use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Simple photo database management tool. Pixel content based depduplication via xxhash and libraw.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Mode to run
    #[clap(subcommand)]
    pub mode: Mode,
    /// The database root to move files into
    #[clap(long, default_value = "photodb")]
    pub import_path: PathBuf,
    /// Move the files to the database root
    #[clap(short, long, default_value_t = false)]
    pub move_files: bool,
    /// Import the files into the database, checking for duplicates
    #[clap(short, long, default_value_t = false)]
    pub insert: bool,
    /// The name of the database to use
    #[clap(short, long, default_value = ".photodb/photo.db")]
    pub database: PathBuf,
    /// Create the database
    #[clap(short, long, default_value_t = false)]
    pub create: bool,
}

#[derive(Subcommand)]
pub enum Mode {
    /// Import files into the database
    Import {
        /// The path to the file or directory to read
        path: Option<PathBuf>,
    },
    /// Verify the raw image file hashes
    Verify,
}
