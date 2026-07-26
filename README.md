# Lincy

> Lightweight clipboard manager for Linux, inspired by [Maccy](https://github.com/p0deje/Maccy).

Simple, fast, keyboard-driven. Built with Rust and GTK4.

[![CI](https://github.com/edumrodrigues/lincy/actions/workflows/ci.yml/badge.svg)](https://github.com/edumrodrigues/lincy/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/edumrodrigues/lincy)](https://github.com/edumrodrigues/lincy/releases)

**Built & tested on:** ![Ubuntu](https://img.shields.io/badge/Ubuntu-passing-brightgreen) ![Fedora](https://img.shields.io/badge/Fedora-passing-brightgreen) ![Arch](https://img.shields.io/badge/Arch-passing-brightgreen) ![Debian](https://img.shields.io/badge/Debian-passing-brightgreen)

## Compatibility

| Desktop | Clipboard | Tray | Hotkey |
|---------|-----------|------|--------|
| GNOME (Wayland) | arboard/XWayland | ksni/D-Bus | gsettings |
| GNOME (X11) | arboard/X11 | ksni/D-Bus | gsettings |
| KDE Plasma | arboard/XWayland | ksni/D-Bus | gsettings |
| XFCE, Cinnamon, MATE | arboard | ksni/D-Bus | gsettings |
| Sway, Hyprland (wlroots) | arboard/XWayland | ksni/D-Bus | manual |

> Requires: GTK4 runtime, XWayland (for clipboard access), D-Bus session (for tray).

## Features

- **Clipboard history** — text + image support, polling every 500ms
- **System tray icon** — accessible from the top bar, menu shows recent items
- **Popup search window** — Maccy-style centered popup, real-time filter
- **Keyboard navigation** — arrows, Enter to copy, Escape to close, Ctrl+P to pin
- **Image thumbnails** — previews in list, copy images back to clipboard
- **Pin items** — keep important items at the top
- **Deduplication** — SHA-256 hash prevents duplicate entries
- **Configurable** — thumbnail size, max items, shortcut, clear history
- **Hotkey** — customizable, auto-registered via gsettings

## Requirements

- Ubuntu 24.04+ or any Linux distribution with GNOME ≥ 44
- Wayland session (recommended) or X11/Xorg

### System packages

```bash
sudo apt install -y libgtk-4-dev libadwaita-1-dev libsqlite3-dev pkg-config
```

- Rust stable: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

## Install

### .deb package (Ubuntu/Debian)

Download the `.deb` from [releases](https://github.com/edumrodrigues/lincy/releases/latest) and:

```bash
sudo apt install ./lincy_*.deb
lincy &
```

### Pre-built binary (any distro)

Download `lincy-linux-x86_64.tar.gz` from [releases](https://github.com/edumrodrigues/lincy/releases/latest):

```bash
tar xf lincy-linux-x86_64.tar.gz
cd lincy && ./install.sh
```

### Build from source

```bash
git clone https://github.com/edumrodrigues/lincy.git
cd lincy && ./scripts/install.sh
```

### cargo install (Rust users)

Once on [crates.io](https://crates.io):
```bash
cargo install lincy
```

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
