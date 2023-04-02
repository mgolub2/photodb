use rusqlite::{named_params, Connection, Result};

use crate::photo::Photo;

pub(crate) fn create_table(con: &mut Connection) {
    let query = "
    CREATE TABLE photodb (hash BLOB UNIQUE, original_path TEXT, imported_path TEXT, year INTEGER, month INTEGER, model TEXT);
";

    match con.execute(query, ()) {
        Ok(_) => println!("Created database table for photodb."),
        Err(e) => println!("Error creating table: {}", e),
    }
}

pub(crate) fn is_imported(hash: i128, con: &mut Connection) -> bool {
    let mut stmt = con
        .prepare("SELECT * FROM photodb WHERE hash = :hash")
        .expect("conn failed");
    let mut rows = stmt
        .query(named_params! { ":hash": hash })
        .expect("rows failed");
    let row = rows.next().expect("query failed");
    return match row {
        Some(_) => true,
        None => false,
    };
}

pub(crate) fn insert_file_to_db(metadata: &Photo, conn: &mut Connection) -> Result<()> {
    let mut stmt = conn.prepare(
            "INSERT INTO photodb (hash, original_path, imported_path, year, month, model) VALUES (:hash, :og_path, :imprt_path, :year, :month, :model)").unwrap();
    stmt.execute(named_params! {
        ":hash": metadata.hash,
        ":og_path" : metadata.og_path.to_str().unwrap(),
        ":imprt_path" : metadata.db_path.to_str().unwrap(),
        ":year" : metadata.year,
        ":month" : metadata.month,
        ":model" : metadata.model,
    })?;
    Ok(())
}
