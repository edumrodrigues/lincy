/// Represents a single item in the clipboard history.
#[derive(Debug, Clone)]
pub struct ClipboardItem {
    pub id: i64,
    pub content: String,
    pub content_hash: String,
    pub pinned: bool,
    pub created_at: String,
    pub last_used_at: String,
    pub usage_count: i64,
}