use rusqlite::{params, Connection};

use crate::db::models::ClipboardItem;
use crate::error::LincyError;
use sha2::{Digest, Sha256};

/// Computes the SHA-256 hex digest of the given text.
pub fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    // Convert the GenericArray to hex string
    let hex: String = result.iter().map(|b| format!("{:02x}", b)).collect();
    hex
}

/// (rest of the file stays the same)
pub fn insert_or_update(conn: &Connection, content: &str) -> Result<ClipboardItem, LincyError> {
    let content_hash = hash_content(content);

    let updated = conn.execute(
        "UPDATE clipboard_history
         SET last_used_at = datetime('now', 'localtime'),
             usage_count  = usage_count + 1
         WHERE content_hash = ?1",
        params![content_hash],
    )?;

    if updated > 0 {
        let item = conn.query_row(
            "SELECT id, content, content_hash, pinned, created_at, last_used_at, usage_count
             FROM clipboard_history
             WHERE content_hash = ?1",
            params![content_hash],
            |row| {
                Ok(ClipboardItem {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    content_hash: row.get(2)?,
                    pinned: row.get::<_, i64>(3)? != 0,
                    created_at: row.get(4)?,
                    last_used_at: row.get(5)?,
                    usage_count: row.get(6)?,
                })
            },
        )?;
        return Ok(item);
    }

    conn.execute(
        "INSERT INTO clipboard_history (content, content_hash)
         VALUES (?1, ?2)",
        params![content, content_hash],
    )?;

    let id = conn.last_insert_rowid();
    let item = conn.query_row(
        "SELECT id, content, content_hash, pinned, created_at, last_used_at, usage_count
         FROM clipboard_history
         WHERE id = ?1",
        params![id],
        |row| {
            Ok(ClipboardItem {
                id: row.get(0)?,
                content: row.get(1)?,
                content_hash: row.get(2)?,
                pinned: row.get::<_, i64>(3)? != 0,
                created_at: row.get(4)?,
                last_used_at: row.get(5)?,
                usage_count: row.get(6)?,
            })
        },
    )?;

    Ok(item)
}

/// Returns the most recent clipboard entries, pinned first.
pub fn list_recent(conn: &Connection, limit: i64) -> Result<Vec<ClipboardItem>, LincyError> {
    let mut stmt = conn.prepare(
        "SELECT id, content, content_hash, pinned, created_at, last_used_at, usage_count
         FROM clipboard_history
         ORDER BY pinned DESC, last_used_at DESC
         LIMIT ?1",
    )?;

    let items = stmt
        .query_map(params![limit], |row| {
            Ok(ClipboardItem {
                id: row.get(0)?,
                content: row.get(1)?,
                content_hash: row.get(2)?,
                pinned: row.get::<_, i64>(3)? != 0,
                created_at: row.get(4)?,
                last_used_at: row.get(5)?,
                usage_count: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(items)
}

/// Searches the clipboard history by content (LIKE query).
pub fn search(
    conn: &Connection,
    query: &str,
    limit: i64,
) -> Result<Vec<ClipboardItem>, LincyError> {
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT id, content, content_hash, pinned, created_at, last_used_at, usage_count
         FROM clipboard_history
         WHERE content LIKE ?1
         ORDER BY pinned DESC, last_used_at DESC
         LIMIT ?2",
    )?;

    let items = stmt
        .query_map(params![pattern, limit], |row| {
            Ok(ClipboardItem {
                id: row.get(0)?,
                content: row.get(1)?,
                content_hash: row.get(2)?,
                pinned: row.get::<_, i64>(3)? != 0,
                created_at: row.get(4)?,
                last_used_at: row.get(5)?,
                usage_count: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(items)
}

/// Toggles the `pinned` status of an item.
pub fn toggle_pin(conn: &Connection, id: i64) -> Result<(), LincyError> {
    conn.execute(
        "UPDATE clipboard_history
         SET pinned = CASE WHEN pinned = 0 THEN 1 ELSE 0 END
         WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

/// Deletes a single item from the history.
pub fn delete_item(conn: &Connection, id: i64) -> Result<(), LincyError> {
    conn.execute("DELETE FROM clipboard_history WHERE id = ?1", params![id])?;
    Ok(())
}

/// Deletes all unpinned items from the history.
pub fn delete_all_unpinned(conn: &Connection) -> Result<(), LincyError> {
    conn.execute("DELETE FROM clipboard_history WHERE pinned = 0", [])?;
    Ok(())
}

/// Returns the total count of items in the history.
pub fn count(conn: &Connection) -> Result<i64, LincyError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM clipboard_history", [], |row| {
        row.get(0)
    })?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::initialize_db(&conn).unwrap();
        conn
    }

    #[test]
    fn test_insert_and_deduplicate() {
        let conn = setup_test_db();
        let item1 = insert_or_update(&conn, "hello world").unwrap();
        let item2 = insert_or_update(&conn, "hello world").unwrap();
        assert_eq!(item1.id, item2.id);
        assert_eq!(item2.usage_count, 2);
        assert_eq!(count(&conn).unwrap(), 1);
    }

    #[test]
    fn test_search() {
        let conn = setup_test_db();
        insert_or_update(&conn, "apple pie").unwrap();
        insert_or_update(&conn, "banana bread").unwrap();
        insert_or_update(&conn, "apple tart").unwrap();
        let results = search(&conn, "apple", 10).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_toggle_pin() {
        let conn = setup_test_db();
        let item = insert_or_update(&conn, "pinned item").unwrap();
        assert!(!item.pinned);
        toggle_pin(&conn, item.id).unwrap();
        let results = list_recent(&conn, 10).unwrap();
        assert!(results[0].pinned);
    }

    #[test]
    fn test_delete() {
        let conn = setup_test_db();
        insert_or_update(&conn, "to delete").unwrap();
        assert_eq!(count(&conn).unwrap(), 1);
        let items = list_recent(&conn, 10).unwrap();
        delete_item(&conn, items[0].id).unwrap();
        assert_eq!(count(&conn).unwrap(), 0);
    }
}