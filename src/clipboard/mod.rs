use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::db::Database;

fn hash_bytes(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(data).iter().map(|b| format!("{:02x}", b)).collect()
}

pub struct ClipboardManager {
    inner: Mutex<arboard::Clipboard>,
    last_set_hash: Mutex<String>,
}

impl ClipboardManager {
    pub fn new() -> Result<Self, crate::error::LincyError> {
        let clipboard = arboard::Clipboard::new()
            .map_err(|e| crate::error::LincyError::Clipboard(e.to_string()))?;
        log::info!("Clipboard manager ready (arboard)");
        Ok(Self { inner: Mutex::new(clipboard), last_set_hash: Mutex::new(String::new()) })
    }

    pub fn set_text(&self, text: &str) {
        let mut c = self.inner.lock().expect("clipboard lock");
        if let Ok(mut last) = self.last_set_hash.lock() { *last = hash_bytes(text.as_bytes()); }
        let _ = c.set_text(text.to_string());
    }

    pub fn set_image(&self, rgba: &[u8], width: i32, height: i32) {
        let mut c = self.inner.lock().expect("clipboard lock");
        if let Ok(mut last) = self.last_set_hash.lock() { *last = hash_bytes(rgba); }
        let img = arboard::ImageData {
            width: width as usize, height: height as usize,
            bytes: std::borrow::Cow::Borrowed(rgba),
        };
        let _ = c.set_image(img);
    }

    pub fn start_monitoring(self: &Arc<Self>, db: Arc<Database>, interval_ms: u64) -> PollGuard {
        let mgr = Arc::clone(self);
        let last_seen_hash = Arc::new(Mutex::new(String::new()));

        let source_id = gtk4::glib::timeout_add_local(Duration::from_millis(interval_ms), move || {
            let mgr = Arc::clone(&mgr);
            let db = Arc::clone(&db);
            let last_seen_hash = Arc::clone(&last_seen_hash);
            let mut c = mgr.inner.lock().expect("clipboard lock");

            if let Ok(text) = c.get_text() && !text.is_empty() {
                drop(c);
                let h = hash_bytes(text.as_bytes());
                if !skip(&h, &mgr.last_set_hash, &last_seen_hash) {
                    log::info!("📋 Captured text: {} chars", text.len());
                    let _ = db.insert_or_update(&text);
                }
                return gtk4::glib::ControlFlow::Continue;
            }

            if let Ok(img) = c.get_image() {
                drop(c);
                let h = hash_bytes(&img.bytes);
                if !skip(&h, &mgr.last_set_hash, &last_seen_hash) {
                    log::info!("🖼 Captured image: {}x{}", img.width, img.height);
                    let _ = db.insert_image(&img.bytes, img.width as i32, img.height as i32);
                }
            }

            gtk4::glib::ControlFlow::Continue
        });

        log::info!("Clipboard polling started (every {}ms)", interval_ms);
        PollGuard { source_id: Some(source_id) }
    }
}

fn skip(hash: &str, last_set: &Mutex<String>, last_seen: &Mutex<String>) -> bool {
    if let Ok(our) = last_set.lock() && *our == *hash { return true; }
    if let Ok(mut prev) = last_seen.lock() {
        if *prev == *hash { return true; }
        *prev = hash.to_string();
    }
    false
}

pub struct PollGuard {
    source_id: Option<gtk4::glib::SourceId>,
}

impl Drop for PollGuard {
    fn drop(&mut self) {
        if let Some(id) = self.source_id.take() { id.remove(); }
    }
}
