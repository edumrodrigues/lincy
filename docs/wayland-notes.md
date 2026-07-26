# Architecture Notes — Lincy

## Overview

Lincy runs as a **single GTK4 application** with:
- **GDK4 clipboard monitoring** (event-based, via `gdk4::Clipboard::connect_changed`)
- **System tray** via D-Bus StatusNotifierItem (`ksni` crate, pure Rust, no GTK3 conflict)
- **Hotkey** via xdg-desktop-portal GlobalShortcuts (`ashpd`, best-effort)
- **Popup UI** via GTK4 window (Maccy-style overlay)

## Why No Daemon Process?

Previous attempts used a separate daemon + UI process with `arboard` polling. This fails on Wayland because:
1. A headless process has no Wayland surface → compositor won't deliver clipboard data
2. `arboard` creates its own Wayland connection without a surface → no clipboard access

The fix: run a single GTK4 application. GTK4 creates the necessary Wayland surface, and `gdk4::Clipboard` uses it for proper clipboard monitoring.

## Clipboard Monitoring

```rust
let clipboard = gdk4::Display::default().clipboard();
clipboard.connect_changed(|clipboard| {
    // GDK emits this signal whenever another app owns the clipboard
    glib::spawn_future_local(async move {
        match clipboard.read_text_future().await {
            Ok(Some(text)) => store_in_db(text),
            _ => {}
        }
    });
});
```

- **Event-based** — no polling; GDK emits `changed` when the compositor notifies us
- **Ignores own changes** — tracks the last text we set to avoid re-storing
- **Async** — uses `glib::spawn_future_local` for non-blocking text reading

## System Tray

Uses `ksni`, a pure Rust implementation of the KDE/Freedesktop [StatusNotifierItem](https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/) D-Bus protocol.

- No GTK3 dependency → no GTK3/GTK4 symbol conflict
- D-Bus based → works on any desktop with SNI support (GNOME + Ubuntu AppIndicators, KDE, etc.)
- Runs on a background tokio thread
- Communicates with the GTK main loop via `std::sync::mpsc` channel

On Ubuntu, StatusNotifierItem support comes from the `ubuntu-appindicators` GNOME Shell extension (pre-installed).

## Hotkey

```
Try: ashpd GlobalShortcuts portal (D-Bus)
  └── Requires: xdg-desktop-portal-gnome, GNOME ≥ 44, .desktop file installed
      └── If fails → manual GNOME custom shortcut
```

The portal error "An app id is required" is expected when running outside of a proper desktop launch context. Install the .desktop file and run via autostart for the portal to work.

### Manual Fallback

GNOME Settings → Keyboard → View and Customize Shortcuts → Custom Shortcuts:
- Name: `Lincy`
- Command: `lincy`
- Shortcut: `Ctrl+Shift+C`

## Compatibility

| Feature | Wayland (GNOME/Mutter) | X11/Xorg |
|---------|------------------------|----------|
| Clipboard monitor | ✅ gdk4::Clipboard (event-based) | ✅ gdk4::Clipboard |
| System tray | ✅ ksni (D-Bus SNI) | ✅ ksni (D-Bus SNI) |
| Hotkey | ✅ Portal (best-effort) | ✅ Portal or X11 grab |
| UI | ✅ GTK4 | ✅ GTK4 |
