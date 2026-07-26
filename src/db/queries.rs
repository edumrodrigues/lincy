use rusqlite::{params, Connection};

use crate::db::models::ClipboardItem;
use crate::error::LincyError;
use sha2::{Digest, Sha256};

/// Computes SHA-256 hex digest for deduplication.
fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn hash_content(content: &str) -> String {
    hash_bytes(content.as_bytes())
}

pub fn hash_image(rgba: &[u8]) -> String {
    hash_bytes(rgba)
}

fn row_to_item(row: &rusqlite::Row) -> rusqlite::Result<ClipboardItem> {
    Ok(ClipboardItem {
        id: row.get(0)?,
        content: row.get(1)?,
        content_hash: row.get(2)?,
        content_type: row.get(3)?,
        pinned: row.get::<_, i64>(4)? != 0,
        created_at: row.get(5)?,
        last_used_at: row.get(6)?,
        usage_count: row.get(7)?,
        image_data: row.get(8)?,
        image_width: row.get(9)?,
        image_height: row.get(10)?,
    })
}

const SELECT_COLS: &str = "id, content, content_hash, content_type, pinned, created_at, last_used_at, usage_count, image_data, image_width, image_height";

/// Inserts a text item (or updates usage if already present).
pub fn insert_text(conn: &Connection, text: &str) -> Result<ClipboardItem, LincyError> {
    let hash = hash_content(text);
    upsert(conn, text, "", &hash, "text", None, None, None)
}

/// Inserts an image item (or updates usage if already present).
pub fn insert_image(
    conn: &Connection,
    rgba: &[u8],
    width: i32,
    height: i32,
) -> Result<ClipboardItem, LincyError> {
    let hash = hash_image(rgba);
    upsert(conn, "", &hash, &hash, "image", Some(rgba), Some(width), Some(height))
}

fn upsert(
    conn: &Connection,
    text: &str,
    text_for_hash: &str,
    hash: &str,
    content_type: &str,
    image_data: Option<&[u8]>,
    image_width: Option<i32>,
    image_height: Option<i32>,
) -> Result<ClipboardItem, LincyError> {
    // Try to update existing
    let updated = conn.execute(
        "UPDATE clipboard_history
         SET last_used_at = datetime('now', 'localtime'),
             usage_count  = usage_count + 1
         WHERE content_hash = ?1",
        params![hash],
    )?;

    if updated > 0 {
        return conn.query_row(
            &format!("SELECT {} FROM clipboard_history WHERE content_hash = ?1", SELECT_COLS),
            params![hash],
            row_to_item,
        ).map_err(LincyError::from);
    }

    // Insert new
    conn.execute(
        "INSERT INTO clipboard_history (content, content_hash, content_type, image_data, image_width, image_height)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![text, hash, content_type, image_data, image_width, image_height],
    )?;

    let id = conn.last_insert_rowid();
    conn.query_row(
        &format!("SELECT {} FROM clipboard_history WHERE id = ?1", SELECT_COLS),
        params![id],
        row_to_item,
    ).map_err(LincyError::from)
}

pub fn list_recent(conn: &Connection, limit: i64) -> Result<Vec<ClipboardItem>, LincyError> {
    let mut stmt = conn.prepare(
        &format!(
            "SELECT {} FROM clipboard_history
             ORDER BY pinned DESC, last_used_at DESC
             LIMIT ?1",
            SELECT_COLS
        ),
    )?;
    stmt.query_map(params![limit], row_to_item)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(LincyError::from)
}

pub fn search(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> Result<Vec<ClipboardItem>, LincyError> {
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        &format!(
            "SELECT {} FROM clipboard_history
             WHERE content LIKE ?1 AND content_type = 'text'
             ORDER BY pinned DESC, last_used_at DESC
             LIMIT ?2",
            SELECT_COLS
        ),
    )?;
    stmt.query_map(params![pattern, limit], row_to_item)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(LincyError::from)
}

pub fn toggle_pin(conn: &Connection, id: i64) -> Result<(), LincyError> {
    conn.execute(
        "UPDATE clipboard_history
         SET pinned = CASE WHEN pinned = 0 THEN 1 ELSE 0 END
         WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

pub fn delete_item(conn: &Connection, id: i64) -> Result<(), LincyError> {
    conn.execute("DELETE FROM clipboard_history WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn delete_all_unpinned(conn: &Connection) -> Result<(), LincyError> {
    conn.execute("DELETE FROM clipboard_history WHERE pinned = 0", [])?;
    Ok(())
}

pub fn count(conn: &Connection) -> Result<i64, LincyError> {
    Ok(conn.query_row("SELECT COUNT(*) FROM clipboard_history", [], |r| r.get(0))?)
}
