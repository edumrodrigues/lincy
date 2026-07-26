# Contributing to Lincy

## Setup

```bash
# System dependencies (Ubuntu)
sudo apt install -y libgtk-4-dev libadwaita-1-dev libsqlite3-dev pkg-config

# Build
cargo build --release

# Run tests
cargo test
```

## Project Structure

```
src/
├── main.rs           # Entry point, GTK4 application, event loop
├── config.rs         # XDG paths, settings load/save
├── error.rs          # Error types (thiserror)
├── clipboard/        # arboard-based clipboard polling + Wayland data-control
├── db/               # SQLite (migrations, models, queries)
├── hotkey/           # ashpd GlobalShortcuts portal registration
├── tray/             # ksni D-Bus StatusNotifierItem
└── ui/
    ├── mod.rs        # Popup window, list, keyboard shortcuts
    └── settings.rs   # Settings dialog
```

## Running

```bash
# Build & install
./scripts/install.sh

# Start (auto-starts on login via ~/.config/autostart)
lincy &

# Open popup
lincy

# Uninstall
./scripts/uninstall.sh
```

## CI

GitHub Actions builds and tests on **Ubuntu, Fedora, Arch, and Debian** on every push to `main` and `dev`.

- `ci.yml` — multi-distro build + test + clippy
- `release.yml` — build and attach binary on version tags (`v*`)
