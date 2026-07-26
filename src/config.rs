use std::path::PathBuf;

/// Returns the directory where Lincy stores its data (XDG_DATA_HOME).
pub fn data_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("lincy")
}

/// Returns the full path to the SQLite database file.
pub fn db_path() -> PathBuf {
    data_dir().join("history.db")
}

/// Returns the application ID for GTK/D-Bus identification.
pub fn app_id() -> &'static str {
    "com.github.edumrodrigues.lincy"
}