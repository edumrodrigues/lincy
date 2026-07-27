# Contributing to Lincy

Thanks for your interest in contributing!

## Getting started

```bash
# Clone
git clone https://github.com/edumrodrigues/lincy.git
cd lincy

# Install system dependencies
# Debian/Ubuntu:
sudo apt install -y libgtk-4-dev libadwaita-1-dev libsqlite3-dev pkg-config
# Fedora:
sudo dnf install -y gtk4-devel libadwaita-devel sqlite-devel pkg-config
# Arch:
sudo pacman -S --noconfirm gtk4 libadwaita sqlite pkg-config

# Build
cargo build --release

# Run tests
cargo test

# Run lints
cargo clippy -- -D warnings
```

## Project structure

```
src/
├── main.rs           # Entry point, GTK4 application, event loop
├── config.rs         # XDG paths, settings load/save, shortcut registration
├── error.rs          # Error types (thiserror)
├── clipboard/mod.rs  # arboard polling (text + image capture)
├── db/               # SQLite storage
│   ├── mod.rs        # Database wrapper
│   ├── migrations.rs # Schema creation / migration
│   ├── models.rs     # ClipboardItem struct
│   └── queries.rs    # CRUD operations
├── hotkey/mod.rs     # ashpd global shortcuts portal
├── tray.rs           # ksni D-Bus StatusNotifierItem
└── ui/
    ├── mod.rs        # Popup window, list rendering, actions
    └── settings.rs   # Settings dialog
```

## How it works

1. **Clipboard monitoring** — `arboard` polls the clipboard every 500ms via X11/XWayland. Text and images are detected, deduplicated by SHA-256 hash, and stored in SQLite.

2. **System tray** — `ksni` implements the StatusNotifierItem D-Bus protocol. Left-click opens the popup, right-click shows the clipboard history as a menu.

3. **Popup window** — a GTK4 undecorated window with search entry, list box, and footer. Full clipboard content is stored on each row via `glib` object data.

4. **Hotkey** — `ashpd` tries the xdg-desktop-portal GlobalShortcuts API (best-effort). Settings fallback uses `gsettings` to register a GNOME custom keybinding.

5. **Settings** — stored as JSON in `~/.config/lincy/settings.json`. Thumbnail size, max items, and shortcut are configurable via the Settings dialog.

## CI

GitHub Actions runs on every push to `main`:

| Job | What it does |
|-----|-------------|
| `test-ubuntu` | Build + test on Ubuntu (native) |
| `test-fedora` | Build + test on Fedora (container) |
| `test-arch` | Build + test on Arch (container) |
| `test-debian` | Build + test on Debian (container) |
| `lint` | `cargo clippy -D warnings` |

The `release` workflow builds `.deb` and `.tar.gz` artifacts on version tags (`v*`).

## Running

```bash
# Start
lincy &

# Open popup
lincy

# Run with debug logging
RUST_LOG=debug lincy
```

## Creating a release

```bash
git checkout v1.0
git tag -a v0.X.Y -m "v0.X.Y"
git push origin v0.X.Y
```

The CI will build the `.deb` and `.tar.gz` and attach them to a new GitHub Release.
