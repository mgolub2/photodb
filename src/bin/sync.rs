use std::{fs, io, path::PathBuf};

use clap::Parser;
use photodb::{build_config_path, db, raw_photo::Photo};
use rusqlite::Connection;
use std::collections::HashSet;

/// Syncs two photodb databases. This is useful if you have two databases that you want to merge into one, or for backups.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct SyncCLI {
    /// The first database root to sync
    pub db1: PathBuf,
    /// The second database root to sync
    pub db2: PathBuf,
    /// Run a actual check on the filenames to see if they exist in the other database. This is a slow operation.
    #[clap(long, short, default_value = "false")]
    pub check: bool,
    /// Perform the sync operation. This will copy the missing files from the first database to the second database.
    #[clap(long, short, default_value = "false")]
    pub do_sync: bool,
}

fn get_db_con(db_path: &PathBuf) -> Connection {
    let con: Connection = Connection::open(db_path).expect("conn failed");
    return con;
}

pub fn h2_missing_h1(h1: HashSet<Photo>, h2: HashSet<Photo>) -> HashSet<Photo> {
    h1.into_iter().filter(|photo1| !h2.contains(photo1)).collect()
}

fn copy_file_with_directory_creation(src: &PathBuf, dst: &PathBuf) -> io::Result<()> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dst)?;
    Ok(())
}

fn main() {
    let args = SyncCLI::parse();
    println!("Syncing {} and {}", args.db1.display(), args.db2.display());
    let con1 = get_db_con(&build_config_path(&args.db1));
    let con2 = get_db_con(&build_config_path(&args.db2));
    let photos1 = get_photos(&con1);
    let photos2 = get_photos(&con2);
    println!("Found {} photos in {}", photos1.len(), args.db1.display());
    println!("Found {} photos in {}", photos2.len(), args.db2.display());
    println!("Finding missing photos...");
    let missing1 = h2_missing_h1(photos1, photos2.clone());
    //let missing2 = h2_missing_h1(photos2, photos1);
    println!(
        "Found {} missing photos in {} from {}.",
        missing1.len(),
        args.db2.display(),
        args.db1.display()
    );
    //print the list of missing photos:
    for photo in missing1 {
        println!("\t{}", photo.db_path.display());
        //test if the file exists in the second database:
        let filename = args
            .db2
            .join(photo.year.to_string())
            .join(photo.month.to_string())
            .join(photo.db_path.file_name().unwrap());
        if args.check {
            if filename.exists() {
                println!(
                    "\t\t{} exists in second database.",
                    photo.db_path.file_name().unwrap().to_str().unwrap()
                );
            } else {
                println!(
                    "\t\t{} does not exist in second database.",
                    photo.db_path.file_name().unwrap().to_str().unwrap()
                );
            }
        }
        if args.do_sync {
            println!("\tsyncing {} to {}", photo.db_path.display(), filename.display());
            let new_photo = Photo {
                hash: photo.hash,
                og_path: photo.og_path,
                db_root: args.db2.clone(),
                db_path: filename,
                year: photo.year,
                month: photo.month,
                model: photo.model,
            };
            db::insert_file_to_db_con(&new_photo, &con2)
                .and_then(|_| {
                    Ok({
                        //copy the file
                        copy_file_with_directory_creation(&photo.db_path, &new_photo.db_path)
                            .expect("copy failed");
                    })
                })
                .expect("insert failed");
        } else {
            println!("\tmock syncing {} to {}", photo.db_path.display(), filename.display());
        }
    }
    //println!("\tFound {} missing photos in {}", missing2.len(), args.db1.display());
}

fn get_photos(con1: &Connection) -> HashSet<Photo> {
    let mut stmt = con1.prepare("SELECT * FROM photodb").unwrap();
    stmt.query_map([], |row| {
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
    .collect::<Result<HashSet<_>, _>>()
    .unwrap()
}
