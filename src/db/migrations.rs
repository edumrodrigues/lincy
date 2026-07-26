use rusqlite::Connection;

use crate::error::LincyError;

/// Creates the database tables if they don't exist and sets WAL mode.
pub fn initialize_db(conn: &Connection) -> Result<(), LincyError> {
    // Enable WAL mode for concurrent reads (daemon writes, UI reads)
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS clipboard_history (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            content      TEXT    NOT NULL,
            content_hash TEXT    NOT NULL,
            pinned       INTEGER NOT NULL DEFAULT 0,
            created_at   TEXT    NOT NULL DEFAULT (datetime('now', 'localtime')),
            last_used_at TEXT    NOT NULL DEFAULT (datetime('now', 'localtime')),
            usage_count  INTEGER NOT NULL DEFAULT 1
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_content_hash
            ON clipboard_history(content_hash);

        CREATE INDEX IF NOT EXISTS idx_created_at
            ON clipboard_history(created_at DESC);

        CREATE INDEX IF NOT EXISTS idx_pinned
            ON clipboard_history(pinned);

        CREATE INDEX IF NOT EXISTS idx_last_used_at
            ON clipboard_history(last_used_at DESC);
        ",
    )?;

    Ok(())
}