use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Simple photo database management tool. Pixel content based depduplication via xxhash and libraw.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub(crate) struct Cli {
    /// Mode to run
    #[clap(subcommand)]
    pub(crate) mode: Mode,
    /// The database root to move files into
    #[clap(long, default_value = "photodb")]
    pub(crate) import_path: PathBuf,
    /// Move the files to the database root
    #[clap(short, long, default_value_t = false)]
    pub(crate) move_files: bool,
    /// Import the files into the database, checking for duplicates
    #[clap(short, long, default_value_t = false)]
    pub(crate) insert: bool,
    /// The name of the database to use
    #[clap(short, long, default_value = ".photodb/photo.db")]
    pub(crate) database: PathBuf,
    /// Create the database
    #[clap(short, long, default_value_t = false)]
    pub(crate) create: bool,
}

#[derive(Subcommand)]
pub(crate) enum Mode {
    /// Import files into the database
    Import {
        /// The path to the file or directory to read
        path: Option<PathBuf>,
    },
    /// Verify the raw image file hashes
    Verify,
}
