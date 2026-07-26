use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("lincy")
}

pub fn db_path() -> PathBuf {
    data_dir().join("history.db")
}

pub fn settings_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".config"))
        .join("lincy")
        .join("settings.json")
}

pub fn app_id() -> &'static str {
    "com.github.edumrodrigues.lincy"
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    #[serde(default = "default_thumb_size")]
    pub thumb_size: i32,
    #[serde(default = "default_max_items")]
    pub max_items: i64,
    #[serde(default = "default_shortcut")]
    pub shortcut: String,
}

fn default_thumb_size() -> i32 { 24 }
fn default_max_items() -> i64 { 100 }
fn default_shortcut() -> String { "Ctrl+Shift+C".into() }

impl Default for Settings {
    fn default() -> Self {
        Self {
            thumb_size: default_thumb_size(),
            max_items: default_max_items(),
            shortcut: default_shortcut(),
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let path = settings_path();
        if path.exists() {
            std::fs::read_to_string(&path).ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        if let Some(parent) = settings_path().parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(settings_path(), serde_json::to_string_pretty(self).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let s = Settings::default();
        assert!(s.thumb_size >= 16);
        assert!(s.max_items >= 5);
        assert!(!s.shortcut.is_empty());
    }

    #[test]
    fn test_roundtrip() {
        let s = Settings { thumb_size: 32, max_items: 200, shortcut: "Ctrl+Alt+X".into() };
        let json = serde_json::to_string(&s).unwrap();
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.thumb_size, 32);
        assert_eq!(restored.max_items, 200);
        assert_eq!(restored.shortcut, "Ctrl+Alt+X");
    }

    #[test]
    fn test_partial_json() {
        // Missing fields use defaults
        let json = r#"{"thumb_size": 48}"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(s.thumb_size, 48);
        assert_eq!(s.max_items, default_max_items());
        assert_eq!(s.shortcut, default_shortcut());
    }
}
