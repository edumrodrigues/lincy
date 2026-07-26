use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::db::Database;

/// Manages clipboard access. Uses `arboard` which works via X11/XWayland —
/// on Wayland sessions, the XWayland bridge makes X11 selections available.
///
/// Polling at 500ms (like Maccy does) provides reliable cross-application
/// monitoring without requiring Wayland protocol extensions.
pub struct ClipboardManager {
    inner: Mutex<arboard::Clipboard>,
    /// Last text we wrote — skip it when polling.
    last_set: Mutex<String>,
}

impl ClipboardManager {
    /// Creates a new clipboard manager.
    pub fn new() -> Result<Self, crate::error::LincyError> {
        let clipboard = arboard::Clipboard::new()
            .map_err(|e| crate::error::LincyError::Clipboard(e.to_string()))?;
        log::info!("Clipboard manager ready (arboard)");
        Ok(Self {
            inner: Mutex::new(clipboard),
            last_set: Mutex::new(String::new()),
        })
    }

    /// Sets the clipboard text and records it so the polling loop skips it.
    pub fn set_text(&self, text: &str) {
        let mut clipboard = self.inner.lock().expect("clipboard lock poisoned");
        if let Ok(mut last) = self.last_set.lock() {
            *last = text.to_string();
        }
        if let Err(e) = clipboard.set_text(text.to_string()) {
            log::error!("Failed to set clipboard text: {}", e);
        }
    }

    /// Starts polling the clipboard at the given interval (in milliseconds).
    /// Returns a guard that stops polling when dropped.
    pub fn start_monitoring(
        self: &Arc<Self>,
        db: Arc<Database>,
        interval_ms: u64,
    ) -> PollGuard {
        let mgr = Arc::clone(self);
        let last_seen = Arc::new(Mutex::new(String::new()));
        let first_poll = Arc::new(std::sync::atomic::AtomicBool::new(true));

        let source_id = gtk4::glib::timeout_add_local(
            Duration::from_millis(interval_ms),
            move || {
                let mgr = Arc::clone(&mgr);
                let db = Arc::clone(&db);
                let last_seen = Arc::clone(&last_seen);
                let first_poll = Arc::clone(&first_poll);

                if first_poll.swap(false, std::sync::atomic::Ordering::Relaxed) {
                    log::debug!("First clipboard poll running...");
                }

                // Do the poll synchronously — arboard::get_text() is sync and fast
                let text = {
                    let mut clipboard = mgr.inner.lock().expect("clipboard lock");
                    match clipboard.get_text() {
                        Ok(t) => t,
                        Err(_) => {
                            // Clipboard empty or has non-text content — normal, don't spam
                            String::new()
                        }
                    }
                };

                if !text.is_empty() {
                    let is_new = {
                        // Check against our own set
                        let our = mgr.last_set.lock().expect("last_set lock");
                        if *our == text {
                            false
                        } else {
                            let mut prev = last_seen.lock().expect("last_seen lock");
                            if *prev == text {
                                false
                            } else {
                                *prev = text.clone();
                                true
                            }
                        }
                    };

                    if is_new {
                        log::info!(
                            "📋 Captured: {} chars — \"{}\"",
                            text.len(),
                            &text[..text.len().min(60)]
                        );
                        if let Err(e) = db.insert_or_update(&text) {
                            log::error!("Failed to store clipboard: {}", e);
                        }
                    }
                }

                gtk4::glib::ControlFlow::Continue
            },
        );

        log::info!(
            "Clipboard polling started (every {}ms via arboard)",
            interval_ms
        );

        PollGuard {
            source_id: Some(source_id),
        }
    }
}

/// Keeps the polling timer alive. Stops polling when dropped.
pub struct PollGuard {
    source_id: Option<gtk4::glib::SourceId>,
}

impl Drop for PollGuard {
    fn drop(&mut self) {
        if let Some(id) = self.source_id.take() {
            id.remove();
        }
    }
}
