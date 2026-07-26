# Lincy

> Lightweight clipboard manager for Ubuntu/Linux, inspired by [Maccy](https://github.com/p0deje/Maccy).

Simple, fast, keyboard-driven. Built with Rust, GTK4, and Libadwaita for native GNOME/Wayland integration.

## Features

- **Event-based clipboard monitoring** — uses GDK4's native clipboard signals (no polling)
- **System tray icon** — always accessible from the GNOME top bar
- **Popup search window** — Maccy-style centered popup with instant search
- **Keyboard navigation** — arrow keys, Enter to copy, Escape to close, Ctrl+P to pin
- **Pin items** — keep important items at the top of the list
- **Deduplication** — SHA-256 hash prevents duplicate entries
- **Hotkey** — Ctrl+Shift+C (via xdg-desktop-portal, best-effort on Wayland)

## Requirements

- Ubuntu 24.04+ or any Linux distribution with GNOME ≥ 44
- Wayland session (recommended) or X11/Xorg

### System packages

```bash
sudo apt install -y libgtk-4-dev libadwaita-1-dev libsqlite3-dev pkg-config
```

- Rust stable: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

## Quick Install

```bash
git clone git@github.com:edumrodrigues/lincy.git
cd lincy
./scripts/install.sh
```

The installer:
1. Checks/installs system dependencies
2. Builds the release binary
3. Installs to `~/.local/bin/lincy`
4. Installs icons to `~/.local/share/icons/`
5. Creates desktop entry and autostart

## Usage

```bash
# Start Lincy (runs in background with tray icon)
lincy &

# The app will auto-start on next login via ~/.config/autostart
```

### Keyboard Shortcuts (in popup)

| Key | Action |
|-----|--------|
| `↑` `↓` | Navigate items |
| `Enter` | Copy selected item to clipboard |
| `Escape` | Close popup |
| `Delete` | Delete selected item |
| `Ctrl+P` | Pin/unpin selected item |
| Type text | Filter history in real time |

### Tray Menu

- **Show Lincy** — open the popup window
- **Clear History** — remove all unpinned items
- **Quit** — exit Lincy

### Global Hotkey

Lincy tries to register `Ctrl+Shift+C` automatically via the xdg-desktop-portal GlobalShortcuts API.

If the portal is unavailable, set it up manually:
1. GNOME Settings → Keyboard → View and Customize Shortcuts
2. Scroll to Custom Shortcuts → Add
3. Name: `Lincy`, Command: `lincy`, Shortcut: `Ctrl+Shift+C`

## Architecture

```
lincy
├── Single GTK4 application (runs continuously)
│   ├── Clipboard monitor (gdk4::Clipboard changed signal)
│   ├── System tray icon (D-Bus StatusNotifierItem via ksni)
│   ├── Popup window (Maccy-style search & paste)
│   └── Hotkey listener (ashpd GlobalShortcuts portal)
├── SQLite database (~/.local/share/lincy/history.db)
└── Event-based (no polling — GDK4 clipboard changed signal)
```

## Database

Stored at `~/.local/share/lincy/history.db` (SQLite, WAL mode):

```sql
CREATE TABLE clipboard_history (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    content      TEXT    NOT NULL,
    content_hash TEXT    NOT NULL,  -- SHA-256 for dedup
    pinned       INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT    NOT NULL DEFAULT (datetime('now','localtime')),
    last_used_at TEXT    NOT NULL DEFAULT (datetime('now','localtime')),
    usage_count  INTEGER NOT NULL DEFAULT 1
);
```

## Uninstall

```bash
./scripts/uninstall.sh
```

## License

MIT
