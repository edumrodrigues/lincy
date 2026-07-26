pub mod migrations;
pub mod models;
pub mod queries;

use rusqlite::Connection;
use std::sync::Mutex;

use crate::config;
use crate::error::LincyError;

/// Opens a connection to the SQLite database, creating the data directory
/// and running migrations as needed.
pub fn open_connection() -> Result<Connection, LincyError> {
    let db_path = config::db_path();

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(&db_path)?;
    migrations::initialize_db(&conn)?;

    log::info!("Database opened at: {}", db_path.display());
    Ok(conn)
}

/// A thread-safe wrapper around the database connection.
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new() -> Result<Self, LincyError> {
        let conn = open_connection()?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn insert_or_update(&self, content: &str) -> Result<models::ClipboardItem, LincyError> {
        let conn = self.conn.lock().unwrap();
        queries::insert_or_update(&conn, content)
    }

    pub fn list_recent(&self, limit: i64) -> Result<Vec<models::ClipboardItem>, LincyError> {
        let conn = self.conn.lock().unwrap();
        queries::list_recent(&conn, limit)
    }

    pub fn search(&self, query: &str, limit: i64) -> Result<Vec<models::ClipboardItem>, LincyError> {
        let conn = self.conn.lock().unwrap();
        queries::search(&conn, query, limit)
    }

    pub fn toggle_pin(&self, id: i64) -> Result<(), LincyError> {
        let conn = self.conn.lock().unwrap();
        queries::toggle_pin(&conn, id)
    }

    pub fn delete_item(&self, id: i64) -> Result<(), LincyError> {
        let conn = self.conn.lock().unwrap();
        queries::delete_item(&conn, id)
    }

    pub fn delete_all_unpinned(&self) -> Result<(), LincyError> {
        let conn = self.conn.lock().unwrap();
        queries::delete_all_unpinned(&conn)
    }

    pub fn count(&self) -> Result<i64, LincyError> {
        let conn = self.conn.lock().unwrap();
        queries::count(&conn)
    }
}