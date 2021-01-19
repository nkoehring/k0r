use actix_web::{web, Error as AWError};
use failure::Error;
use futures::{Future, TryFutureExt};
use r2d2_sqlite::SqliteConnectionManager;

use super::short_code::{random_uuid, ShortCode};

pub type Pool = r2d2::Pool<SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<SqliteConnectionManager>;

pub enum Queries {
    NeedsInit,
    InitDB,
    CreateUser(i64, bool),       // rate_limit, is_admin
    GetURL(String),              // short_code
    StoreNewURL(String, String), // api_key, url
}

fn check_database_schema(conn: Connection) -> Result<String, Error> {
    match conn.query_row(
        "SELECT COUNT(name) FROM sqlite_master WHERE type='table' AND name IN ('Users', 'URLs')",
        &[0],
        |row| row.get(0),
    ) {
        Ok(2) => Ok(String::from("")),
        _ => Err(Error::from(rusqlite::Error::QueryReturnedNoRows)),
    }
}

/// Initializes a new SQlite database with the default schema.
fn init_database(conn: Connection) -> Result<String, Error> {
    match conn.execute_batch(
        "
        BEGIN;
        CREATE TABLE IF NOT EXISTS Users(
          UserID   INTEGER PRIMARY KEY,
          APIKey    TEXT UNIQUE NOT NULL,
          RateLimit INTEGER DEFAULT 0,
          Admin     SMALLINT DEFAULT 0
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_api_key ON Users(APIKey);
        CREATE TABLE IF NOT EXISTS URLs(
          ID      INTEGER PRIMARY KEY,
          URL     TEXT NOT NULL,
          Visits  INTEGER DEFAULT 0,
          UserID INTEGER NOT NULL,
          FOREIGN KEY(UserID) REFERENCES Users(UserID)
        );
        COMMIT;",
    ) {
        Ok(_) => Ok(String::from("")), // db::query expects Result<String, Error>
        Err(err) => Err(Error::from(err)),
    }
}

/// Creates a user entry with random API key and returns the API key.
fn create_user(conn: Connection, rate_limit: i64, is_admin: bool) -> Result<String, Error> {
    let new_key = random_uuid();
    let is_admin = if is_admin { "1" } else { "0" };
    let _ = conn.execute(
        "INSERT INTO Users VALUES(NULL, ?1, ?2, ?3)",
        &[&new_key, &rate_limit.to_string(), &is_admin.to_string()],
    )?;
    Ok(new_key)
}

/// Looks up an URL by translating the short_code to its ID
fn get_url(conn: Connection, short_code: &str) -> Result<String, Error> {
    let row_id = ShortCode::from_code(short_code)?.n;
    conn.query_row(
        "SELECT URL FROM URLs WHERE ID = ?",
        &[row_id as i64],
        |row| row.get(0),
    )
    .map_err(Error::from)
}

/// Stores a new URL if api_key is assigned to a valid user
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

/// translates Queries to function calls and returns the result
pub fn query(pool: &Pool, query: Queries) -> impl Future<Output = Result<String, AWError>> {
    let pool = pool.clone();
    web::block(move || match query {
        Queries::NeedsInit => check_database_schema(pool.get()?),
        Queries::InitDB => init_database(pool.get()?),
        Queries::CreateUser(rate_limit, is_admin) => create_user(pool.get()?, rate_limit, is_admin),
        Queries::GetURL(short_code) => get_url(pool.get()?, &short_code),
        Queries::StoreNewURL(api_key, url) => store_url(pool.get()?, &api_key, &url),
    })
    .map_err(AWError::from)
}
