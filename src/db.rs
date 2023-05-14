use std::{collections::HashSet, path::PathBuf};

use rusqlite::{named_params, Connection, Result};

use crate::raw_photo::Photo;

pub const DB_PATH: &str = "photodb.sqlite";
pub const CONFIG_ROOT: &str = ".photodb";

pub fn build_config_path(db_root: &PathBuf) -> PathBuf {
    db_root.join(CONFIG_ROOT).join(DB_PATH)
}

pub fn create_table(con: &mut Connection) {
    let query = "
    CREATE TABLE photodb (hash BLOB UNIQUE, original_path TEXT, imported_path TEXT UNIQUE, year INTEGER, month INTEGER, model TEXT);
";

    match con.execute(query, ()) {
        Ok(_) => println!("Created database table for photodb."),
        Err(e) => println!("Error: creating table: {}", e),
    }
}

pub fn is_imported(hash: i128, database: &PathBuf) -> bool {
    let con: Connection = Connection::open(database).expect("conn failed");
    let mut stmt = con.prepare("SELECT * FROM photodb WHERE hash = :hash").expect("conn failed");
    let mut rows = stmt.query(named_params! { ":hash": hash }).expect("rows failed");
    let row = rows.next().expect("query failed");
    return match row {
        Some(_) => true,
        None => false,
    };
}

pub fn insert_file_to_db(metadata: &Photo, database: &PathBuf) -> Result<()> {
    let con: Connection = Connection::open(database).expect("conn failed");
    insert_file_to_db_con(metadata, &con)
}

pub fn insert_file_to_db_con(metadata: &Photo, con: &Connection) -> Result<()> {
    let mut stmt = con.prepare(
            "INSERT INTO photodb (hash, original_path, imported_path, year, month, model) VALUES (:hash, :og_path, :db_path, :year, :month, :model)").unwrap();
    stmt.execute(named_params! {
        ":hash": metadata.hash,
        ":og_path" : metadata.og_path.to_str().unwrap(),
        ":db_path" : metadata.db_path.to_str().unwrap(),
        ":year" : metadata.year,
        ":month" : metadata.month,
        ":model" : metadata.model,
    })?;
    Ok(())
}

pub fn get_photos(con1: &Connection) -> HashSet<Photo> {
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
