use std::collections::HashSet;

use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use dotenvy::dotenv;
use std::env;

use crate::models::Photo;

pub fn get_connection_pool() -> Pool<ConnectionManager<SqliteConnection>> {
    dotenv().ok();
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<SqliteConnection>::new(url);
    // Refer to the `r2d2` documentation for more methods to use
    // when building a connection pool
    Pool::builder().test_on_check_out(true).build(manager).expect("Could not build connection pool")
}

pub fn is_imported(hash: i64, pool: &Pool<ConnectionManager<SqliteConnection>>) -> bool {
    use crate::schema::photos;
    let mut conn = pool.get().unwrap();
    let results = photos::table
        .filter(photos::hash.eq(hash))
        .limit(1)
        .load::<Photo>(&mut *conn)
        .expect("Error loading photos");
    !results.is_empty()
}

pub fn insert_file_to_db(
    photo: &Photo, pool: &Pool<ConnectionManager<SqliteConnection>>,
) -> Result<usize, diesel::result::Error> {
    use crate::schema::photos::dsl::*;
    let mut conn = pool.get().unwrap();
    diesel::insert_into(photos).values(photo).execute(&mut *conn)
}

pub fn get_photos(pool: &Pool<ConnectionManager<SqliteConnection>>) -> HashSet<Photo> {
    use crate::schema::photos::dsl::*;
    let mut conn = pool.get().unwrap();
    photos.load::<Photo>(&mut *conn).expect("Error loading photos").into_iter().collect()
}
