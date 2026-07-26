use rusqlite::Connection;

use crate::error::LincyError;

/// Creates the database tables if they don't exist and runs migrations.
pub fn initialize_db(conn: &Connection) -> Result<(), LincyError> {
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS clipboard_history (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            content      TEXT    NOT NULL DEFAULT '',
            content_hash TEXT    NOT NULL,
            content_type TEXT    NOT NULL DEFAULT 'text',
            pinned       INTEGER NOT NULL DEFAULT 0,
            created_at   TEXT    NOT NULL DEFAULT (datetime('now', 'localtime')),
            last_used_at TEXT    NOT NULL DEFAULT (datetime('now', 'localtime')),
            usage_count  INTEGER NOT NULL DEFAULT 1,
            image_data   BLOB,
            image_width  INTEGER,
            image_height INTEGER
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

    // Add columns for existing databases (ignore errors if columns exist)
    for col in &[
        "ALTER TABLE clipboard_history ADD COLUMN content_type TEXT NOT NULL DEFAULT 'text'",
        "ALTER TABLE clipboard_history ADD COLUMN image_data BLOB",
        "ALTER TABLE clipboard_history ADD COLUMN image_width INTEGER",
        "ALTER TABLE clipboard_history ADD COLUMN image_height INTEGER",
    ] {
        let _ = conn.execute(col, []);
    }

    Ok(())
}
