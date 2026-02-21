//! Card database: SQLite schema, queries, and data access for MTG card data.
//!
//! This crate manages the SQLite database that stores card data imported from
//! Scryfall bulk data. The schema follows the architecture doc Section 5.2.

pub mod schema;

use rusqlite::Connection;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CardDbError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, CardDbError>;

/// Opens (or creates) a SQLite database at the given path and ensures the schema exists.
pub fn open_database(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    schema::create_tables(&conn)?;
    Ok(conn)
}

/// Opens an in-memory database with the schema applied. Useful for testing.
pub fn open_memory_database() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    schema::create_tables(&conn)?;
    Ok(conn)
}
