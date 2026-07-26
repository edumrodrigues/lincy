pub mod migrations;
pub mod models;
pub mod queries;

use rusqlite::Connection;
use std::sync::Mutex;

use crate::config;
use crate::error::LincyError;

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

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new() -> Result<Self, LincyError> {
        let conn = open_connection()?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn insert_text(&self, content: &str) -> Result<models::ClipboardItem, LincyError> {
        queries::insert_text(&self.conn.lock().unwrap(), content)
    }

    pub fn insert_image(
        &self,
        rgba: &[u8],
        width: i32,
        height: i32,
    ) -> Result<models::ClipboardItem, LincyError> {
        queries::insert_image(&self.conn.lock().unwrap(), rgba, width, height)
    }

    // Kept for backward compat in clipboard polling
    pub fn insert_or_update(&self, content: &str) -> Result<models::ClipboardItem, LincyError> {
        self.insert_text(content)
    }

    pub fn list_recent(&self, limit: i64) -> Result<Vec<models::ClipboardItem>, LincyError> {
        queries::list_recent(&self.conn.lock().unwrap(), limit)
    }

    pub fn search(&self, query: &str, limit: i64) -> Result<Vec<models::ClipboardItem>, LincyError> {
        queries::search(&self.conn.lock().unwrap(), query, limit)
    }

    pub fn toggle_pin(&self, id: i64) -> Result<(), LincyError> {
        queries::toggle_pin(&self.conn.lock().unwrap(), id)
    }

    pub fn delete_item(&self, id: i64) -> Result<(), LincyError> {
        queries::delete_item(&self.conn.lock().unwrap(), id)
    }

    pub fn delete_all_unpinned(&self) -> Result<(), LincyError> {
        queries::delete_all_unpinned(&self.conn.lock().unwrap())
    }

    pub fn count(&self) -> Result<i64, LincyError> {
        queries::count(&self.conn.lock().unwrap())
    }
}
