use std::{fs, io, path::PathBuf};

use clap::Parser;
use photodb::{
    build_config_path,
    db::{self, get_photos},
    raw_photo::Photo,
    util::{build_final_path, get_db_con},
};
use rayon::prelude::*;
use std::collections::HashSet;

const NUM_THREADS: usize = 4;

/// Syncs two photodb databases. This is useful if you have two databases that you want to merge into one, or for backups.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct SyncCLI {
    /// The first database root to sync
    pub db1: PathBuf,
    /// The second database root to sync
    pub db2: PathBuf,
    /// Perform the sync operation. This will copy the missing files from the first database to the second database.
    #[clap(long, short, default_value = "false")]
    pub do_sync: bool,
}

pub fn h2_missing_h1(h1: HashSet<Photo>, h2: HashSet<Photo>) -> HashSet<Photo> {
    h1.into_iter().filter(|photo1| !h2.contains(photo1)).collect()
}

fn copy_file_with_directory_creation(src: &PathBuf, dst: &PathBuf) -> io::Result<PathBuf> {
    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    assert!(!dst.exists());
    fs::copy(src, dst)?;
    Ok(dst.to_path_buf())
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
    println!(
        "Found {} missing photos in {} from {}.",
        missing1.len(),
        args.db2.display(),
        args.db1.display()
    );
    //print the list of missing photos:
    let insert_list: Vec<Photo> = missing1
        .par_iter()
        .filter_map(|photo| {
            //test if the file exists in the second database:
            let filename = build_final_path(
                &args.db2,
                &photo.model,
                &photo.year,
                &photo.month,
                &photo.og_path,
            );
            if filename.exists() {
                println!("\t{} exists in second database.", photo.db_path.display(),);
                None
            } else {
                Some(photo.clone())
            }
        })
        .collect();

    let move_list: Vec<Photo> = insert_list
        .iter()
        .filter_map(|photo| {
            let filename = build_final_path(
                &args.db2,
                &photo.model,
                &photo.year,
                &photo.month,
                &photo.og_path,
            );
            if args.do_sync {
                println!("\tsyncing {} to {}", photo.db_path.display(), filename.display());
            } else {
                println!("\tmock syncing {} to {}", photo.db_path.display(), filename.display());
            }
            let new_photo = Photo {
                hash: photo.hash,
                og_path: photo.db_path.clone(),
                db_root: args.db2.clone(),
                db_path: filename,
                year: photo.year,
                month: photo.month,
                model: photo.model.clone(),
            };
            if args.do_sync {
                db::insert_file_to_db_con(&new_photo, &con2)
                    .map_err(|e| {
                        println!(
                            "Failed to insert {} into database: {}",
                            new_photo.db_path.display(),
                            e
                        );
                    })
                    .ok()?;
            }
            Some(new_photo)
        })
        .collect();
    if args.do_sync {
        rayon::ThreadPoolBuilder::new().num_threads(NUM_THREADS).build_global().unwrap();
        let moved: Vec<()> = move_list
            .par_iter()
            .filter_map(|photo| {
                copy_file_with_directory_creation(&photo.og_path, &photo.db_path)
                    .map_err(|e| {
                        println!(
                            "Failed to copy {} to {}: {}",
                            photo.og_path.display(),
                            photo.db_path.display(),
                            e
                        );
                    })
                    .map(|dst_path| {
                        println!("Copied {} to {}", photo.og_path.display(), dst_path.display())
                    })
                    .ok()
            })
            .collect();
        println!("Synced {}/{} photos.", moved.len(), missing1.len());
    } else {
        println!("Would have synced {}/{} photos.", move_list.len(), missing1.len());
    }
}
