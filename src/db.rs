use actix_web::{web, Error as AWError};
use failure::Error;
use futures::{Future, TryFutureExt};
use r2d2_sqlite::SqliteConnectionManager;

use super::short_code::ShortCode;

pub type Pool = r2d2::Pool<SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<SqliteConnectionManager>;

pub enum Queries {
    GetURL(String),
    StoreNewURL(String, String),
}

fn get_url(conn: Connection, short_code: &str) -> Result<String, Error> {
    let row_id = ShortCode::from_code(short_code)?.n;
    conn.query_row(
        "SELECT URL FROM URLs WHERE ID = ?",
        &[row_id as i64],
        |row| row.get(0),
    )
    .map_err(Error::from)
}

fn store_url(conn: Connection, api_key: &str, url: &str) -> Result<String, Error> {
    let user_id: i64 = conn.query_row(
        "SELECT UserID from Users WHERE APIKey = ?",
        &[api_key],
        |row| row.get(0),
    )?;
    let _ = conn.execute(
        "INSERT INTO URLs VALUES(NULL, ?, 0, ?)",
        &[url, &(user_id.to_string())],
    )?;
    // TODO: In case a plain [0-9a-z] string will be included into
    // IGNORED_SHORT_CODES, this function should work around such IDs as well.
    let short_code = ShortCode::new(conn.last_insert_rowid() as usize).code;
    Ok(short_code)
}

pub fn query(pool: &Pool, query: Queries) -> impl Future<Output = Result<String, AWError>> {
    let pool = pool.clone();
    web::block(move || match query {
        Queries::GetURL(short_code) => get_url(pool.get()?, &short_code),
        Queries::StoreNewURL(api_key, url) => store_url(pool.get()?, &api_key, &url),
    })
    .map_err(AWError::from)
}
