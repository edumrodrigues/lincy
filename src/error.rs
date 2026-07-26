use thiserror::Error;

#[derive(Error, Debug)]
pub enum LincyError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Clipboard error: {0}")]
    Clipboard(String),

    #[error("Wayland clipboard error: {0}")]
    Wayland(String),

    #[error("Hotkey registration error: {0}")]
    Hotkey(String),

    #[error("Portal D-Bus error: {0}")]
    Portal(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("GTK error: {0}")]
    Gtk(String),

    #[error("Tray icon error: {0}")]
    Tray(String),

    #[error("Async runtime error: {0}")]
    Async(String),
}
