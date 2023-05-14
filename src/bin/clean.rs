use std::path::PathBuf;

use clap::Parser;
use photodb::{db::build_config_path, raw_photo::Photo, util::get_db_con};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct CleanCLI {
    /// The database root to use for cleaning the target folder
    db1: PathBuf,
    /// The target folder to clean
    target: PathBuf,
    /// Actually delete the files from the filesystem. If this is not set, the files will only be printed to stdout.
    #[clap(long, short, default_value = "false")]
    delete: bool,
}

fn main() {
    let args = CleanCLI::parse();
    println!("Cleaning {} from {}", args.target.display(), args.db1.display());
    let con1 = get_db_con(&build_config_path(&args.db1));
    //Filter the database for original paths matching the target folder:
    let mut binding = con1.prepare("SELECT * FROM photodb WHERE original_path LIKE ?1").unwrap();
    let matches = binding
        .query_map([format!("{}%", args.target.to_str().unwrap())], |row| {
            Ok(Photo {
                hash: row.get(0)?,
                og_path: PathBuf::from(row.get::<_, String>(1)?),
                db_root: PathBuf::new(),
                db_path: PathBuf::from(row.get::<_, String>(2)?),
                year: row.get(3)?,
                month: row.get(4)?,
                model: row.get(5)?,
            })
        })
        .unwrap()
        .collect::<Result<Vec<Photo>, _>>()
        .unwrap();
    println!("Found {} matches", matches.len());
    //Delete the original paths from the filesystem:
    for path in matches {
        assert!(path.db_path.exists());
        print!("Deleting {}...\t", path.og_path.display());
        if args.delete {
            std::fs::remove_file(path.og_path).expect("failed to delete file");
            println!("done");
        } else {
            println!("fake done");
        }
    }
}
