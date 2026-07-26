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

/// Register the GNOME custom shortcut. Idempotent — safe to call on every startup.
pub fn register_shortcut(binding: &str) {
    let base_path = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/lincy/";
    let schema_key = "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding";

    // Ensure our path is in the custom-keybindings list
    if let Ok(output) = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.settings-daemon.plugins.media-keys", "custom-keybindings"])
        .output()
    {
        let current = String::from_utf8_lossy(&output.stdout);
        if !current.contains("lincy") {
            let new_list = if current.trim() == "@as []" || current.trim().is_empty() {
                format!("['{}']", base_path.trim_end_matches('/'))
            } else {
                format!("{}, '{}']", current.trim().trim_end_matches(']'), base_path.trim_end_matches('/'))
            };
            let _ = std::process::Command::new("gsettings")
                .args(["set", "org.gnome.settings-daemon.plugins.media-keys", "custom-keybindings", &new_list])
                .output();
        }
    }

    // Detect installed binary path (check common locations)
    let bin = ["/usr/bin/lincy", "/usr/local/bin/lincy"].iter()
        .find(|p| std::path::Path::new(p).exists())
        .copied()
        .unwrap_or("$HOME/.local/bin/lincy");

    let schema_path = format!("{}:{}", schema_key, base_path);
    for (key, val) in [("binding", binding), ("command", bin), ("name", "Lincy")] {
        let _ = std::process::Command::new("gsettings")
            .args(["set", &schema_path, key, val])
            .output();
    }
    log::info!("Shortcut: {} → {}", binding, bin);
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
