use actix_web::{web, Error as AWError};
use failure::Error;
use failure_derive::Fail;
use futures::{Future, TryFutureExt};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::NO_PARAMS;

use super::short_code::{random_uuid, ShortCode};

type Result<T = DBValue, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Fail)]
pub enum DBError {
    #[fail(display = "The loaded database has not been initialized.")]
    InvalidSchema,

    #[fail(display = "Database error: {} ({})", msg, src)]
    SqliteError { msg: String, src: rusqlite::Error },
}

#[derive(Debug)]
pub enum DBValue {
    String(String),
    Number(i64),
    // Bool(bool),
    None,
}

pub type Pool = r2d2::Pool<SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<SqliteConnectionManager>;

#[derive(serde::Deserialize)]
pub struct UrlPostData {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub key: String,
}

pub enum Queries {
    NeedsInit,
    CountUsers,
    InitDB,
    CreateUser(i64, bool),    // rate_limit, is_admin
    GetURL(String),           // short_code
    StoreNewURL(UrlPostData), // api_key, url, title?, description?
}

fn get_database_schema(conn: &Connection) -> Result<String> {
    let mut stmt = conn.prepare(
        "
        SELECT
          m.name as table_name,
          p.name as column_name
        FROM
          sqlite_master AS m
        JOIN
          pragma_table_info(m.name) AS p
        ORDER BY
          m.name,
          p.name;
    ",
    )?;
    let mut rows = stmt.query(NO_PARAMS)?;

    let mut tuples = Vec::new();
    while let Some(row) = rows.next()? {
        let table: String = row.get(0)?;
        let column: String = row.get(1)?;
        tuples.push(format!("{}|{}", table, column));
    }

    let schema = tuples.join("\n");
    Ok(schema)
}

fn check_database_schema(conn: Connection) -> Result {
    // TODO: is that really a good way to check the schema?
    let expected_schema = String::from(
        "URLs|created_at
URLs|description
URLs|title
URLs|url
URLs|user_id
URLs|visits
Users|api_key
Users|is_admin
Users|rate_limit
Users|rowid",
    );
    let schema = get_database_schema(&conn)?;

    if schema == expected_schema {
        debug!("Schema validated!");
        Ok(DBValue::None)
    } else {
        debug!("Schema not valid!");
        Err(Error::from(DBError::InvalidSchema))
    }
}

/// Initializes a new SQlite database with the default schema.
fn init_database(conn: Connection) -> Result {
    conn.execute_batch(
        "
        BEGIN;
        CREATE TABLE IF NOT EXISTS Users(
          rowid     INTEGER NOT NULL,
          api_key    TEXT UNIQUE NOT NULL,
          rate_limit INTEGER DEFAULT 0,
          is_admin     SMALLINT DEFAULT 0,
          PRIMARY KEY(rowid)
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_api_key ON Users(api_key);
        CREATE TABLE IF NOT EXISTS URLs(
          url         TEXT NOT NULL,
          visits      INTEGER DEFAULT 0,
          title       TEXT,
          description TEXT,
          created_at   DATETIME,
          user_id      INTEGER NOT NULL,
          FOREIGN KEY(user_id) REFERENCES Users(rowid)
        );
        COMMIT;",
    )
    .map(|_| DBValue::None)
    .map_err(|src| {
        let msg = "Failed to init DB!".to_owned();
        Error::from(DBError::SqliteError { msg, src })
    })
}

fn count_users(conn: Connection) -> Result {
    conn.query_row("SELECT COUNT(rowid) FROM USERS", NO_PARAMS, |row| {
        row.get(0)
    })
    .map(|v| DBValue::Number(v))
    .map_err(|src| {
        let msg = "Could not check users.".to_owned();
        Error::from(DBError::SqliteError { msg, src })
    })
}

/// Creates a user entry with random API key and returns the API key.
fn create_user(conn: Connection, rate_limit: i64, is_admin: bool) -> Result {
    let new_key = random_uuid();
    let is_admin = if is_admin { "1" } else { "0" };
    conn.execute(
        "INSERT INTO Users VALUES(NULL, ?1, ?2, ?3)",
        &[&new_key, &rate_limit.to_string(), &is_admin.to_string()],
    )
    .map(|_| DBValue::String(new_key))
    .map_err(|src| {
        let msg = "Could not create user.".to_owned();
        Error::from(DBError::SqliteError { msg, src })
    })
}

/// Looks up an URL by translating the short_code to its ID
fn get_url(conn: Connection, short_code: &str) -> Result {
    let row_id = ShortCode::from_code(short_code)?.n;

    conn.query_row(
        "SELECT url FROM URLs WHERE rowid = ?",
        &[row_id as i64],
        |row| row.get(0),
    )
    .map(|url| DBValue::String(url))
    .map_err(|src| {
        let msg = "Could not retrieve URL".to_owned();
        Error::from(DBError::SqliteError { msg, src })
    })
}

/// Stores a new URL if api_key is assigned to a valid user
fn store_url(conn: Connection, data: &UrlPostData) -> Result {
    let user_id: i64 = conn.query_row(
        "SELECT rowid FROM Users WHERE api_key = ?",
        &[&data.key],
        |row| row.get(0),
    )?;
    let _ = conn.execute_named(
        "INSERT INTO URLs VALUES(:url, 0, :title, :description, DATETIME('now'), :user_id)",
        &[
            (":url", &data.url),
            (":title", data.title.as_ref().unwrap_or(&String::from(""))),
            (
                ":description",
                data.description.as_ref().unwrap_or(&String::from("")),
            ),
            (":user_id", &(user_id.to_string())),
        ],
    )?;
    // TODO: In case a plain [0-9a-z] string will be included into
    // IGNORED_SHORT_CODES, this function should work around such IDs as well.
    let short_code = ShortCode::new(conn.last_insert_rowid() as usize).code;
    Ok(DBValue::String(short_code))
}

/// translates Queries to function calls and returns the result
pub fn query(
    pool: &Pool,
    query: Queries,
) -> impl Future<Output = std::result::Result<DBValue, AWError>> {
    let pool = pool.clone();
    web::block(move || match query {
        Queries::NeedsInit => check_database_schema(pool.get()?),
        Queries::CountUsers => count_users(pool.get()?),
        Queries::InitDB => init_database(pool.get()?),
        Queries::CreateUser(rate_limit, is_admin) => create_user(pool.get()?, rate_limit, is_admin),
        Queries::GetURL(short_code) => get_url(pool.get()?, &short_code),
        Queries::StoreNewURL(url_data) => store_url(pool.get()?, &url_data),
    })
    .map_err(AWError::from)
}
