# Lincy

> Lightweight clipboard manager for Linux — inspired by [Maccy](https://github.com/p0deje/Maccy).

Simple, fast, keyboard-driven. Built with Rust and GTK4.

[![CI](https://github.com/edumrodrigues/lincy/actions/workflows/ci.yml/badge.svg)](https://github.com/edumrodrigues/lincy/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/edumrodrigues/lincy)](https://github.com/edumrodrigues/lincy/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**Built & tested on:** Ubuntu · Fedora · Arch · Debian

---

## Features

- **Clipboard history** — text and image support, deduplication via SHA-256
- **System tray icon** — always accessible, right-click shows recent items
- **Popup search** — Maccy-style window with real-time filtering
- **Keyboard-driven** — arrows, Enter to copy, Escape to dismiss, Ctrl+P to pin
- **Image thumbnails** — previews in the list, copy images back to clipboard
- **Pin items** — keep important clips at the top
- **Configurable** — thumbnail size, max history, custom shortcut
- **Global hotkey** — auto-registered, customizable

## Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/edumrodrigues/lincy/main/scripts/install-remote.sh | bash
```

Downloads the latest binary, installs everything, and sets up autostart. No git clone, no compilation.

### Other install methods

| Method | Command |
|--------|---------|
| **.deb package** | `sudo apt install ./lincy_*.deb` (from [releases](https://github.com/edumrodrigues/lincy/releases/latest)) |
| **Pre-built tarball** | `tar xf lincy-linux-x86_64.tar.gz && cd lincy && ./install.sh` |
| **Build from source** | `git clone https://github.com/edumrodrigues/lincy.git && cd lincy && ./scripts/install.sh` |
| **cargo** | `cargo install lincy` (once published on [crates.io](https://crates.io)) |

## Usage

Lincy runs in the background and auto-starts on login. Once running:

- **`Ctrl+Shift+C`** — open the popup (customizable in Settings)
- **Click the tray icon** — left-click opens popup, right-click shows history
- **Search** — type to filter items in real time
- **Enter / Click** — copy selected item to clipboard
- **Escape / Click outside** — close the popup

## Compatibility

| Desktop | Clipboard | Tray | Notes |
|---------|-----------|------|-------|
| GNOME (Wayland) | arboard / XWayland | ksni / D-Bus | Full support |
| GNOME (X11) | arboard / X11 | ksni / D-Bus | Full support |
| KDE Plasma | arboard / XWayland | ksni / D-Bus | Full support |
| XFCE, Cinnamon, MATE | arboard | ksni / D-Bus | Full support |
| Sway, Hyprland (wlroots) | arboard / XWayland | ksni / D-Bus | Manual hotkey setup |

**Requirements:** GTK4, XWayland (for clipboard), D-Bus session bus. All major distros ship these by default.

## Building from source

```bash
# Install system dependencies
# Debian/Ubuntu:
sudo apt install -y libgtk-4-dev libadwaita-1-dev libsqlite3-dev pkg-config
# Fedora:
sudo dnf install -y gtk4-devel libadwaita-devel sqlite-devel pkg-config
# Arch:
sudo pacman -S --noconfirm gtk4 libadwaita sqlite pkg-config

# Build
cargo build --release
cargo test
```

Requires [Rust](https://rustup.rs) stable.

## Project structure

```
src/
├── main.rs           # Entry point, GTK4 application
├── config.rs         # Settings, XDG paths, shortcut registration
├── clipboard/        # arboard polling (text + image)
├── db/               # SQLite storage (migrations, queries)
├── hotkey/           # ashpd portal registration
├── tray/             # ksni D-Bus StatusNotifierItem
└── ui/
    ├── mod.rs        # Popup window, list, keyboard handling
    └── settings.rs   # Settings dialog
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE) © 2026 Eduardo Rodrigues
