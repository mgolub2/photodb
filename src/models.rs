use std::hash::Hash;

use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::photos)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Photo {
    pub hash: i64,
    pub original_path: String,
    pub current_path: String,
    pub exif_date: i32,
    pub exif_json: String,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::duplicates)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Duplicate {
    pub id: i32,
    pub hash: i64,
    pub original_path: String,
    pub deleted: bool,
}

impl Eq for Photo {}

impl PartialEq for Photo {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Hash for Photo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_i64(self.hash);
    }
}
