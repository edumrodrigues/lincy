/// Represents a single item in the clipboard history.
#[derive(Debug, Clone)]
pub struct ClipboardItem {
    pub id: i64,
    pub content: String,
    pub content_hash: String,
    pub content_type: String, // "text" or "image"
    pub pinned: bool,
    pub created_at: String,
    pub last_used_at: String,
    pub usage_count: i64,
    /// For images: RGBA pixel data
    pub image_data: Option<Vec<u8>>,
    /// For images: width in pixels
    pub image_width: Option<i32>,
    /// For images: height in pixels
    pub image_height: Option<i32>,
}
