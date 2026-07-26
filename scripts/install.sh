#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

BOLD="$(tput bold 2>/dev/null || echo "")"
GREEN="$(tput setaf 2 2>/dev/null || echo "")"
RESET="$(tput sgr0 2>/dev/null || echo "")"

echo "${BOLD}=== Lincy Installer ===${RESET}"
echo "Lightweight Clipboard Manager for GNOME/Wayland"
echo ""

# Ensure ~/.local/bin is in PATH for this session
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"

# ── Check system dependencies ──────────────────────────────
echo "[1/5] Checking system dependencies..."

MISSING=""
for pkg in libgtk-4-dev libadwaita-1-dev libsqlite3-dev; do
    if ! dpkg -s "$pkg" &>/dev/null; then
        MISSING="$MISSING $pkg"
    fi
done

if [ -n "$MISSING" ]; then
    echo "  Missing packages:$MISSING"
    echo "  Installing..."
    sudo apt install -y $MISSING
else
    echo "  ${GREEN}✓${RESET} All system dependencies present"
fi

# ── Build ──────────────────────────────────────────────────
echo "[2/5] Building release binary..."
cd "$PROJECT_DIR"
cargo build --release 2>&1 | sed 's/^/  /'
echo "  ${GREEN}✓${RESET} Build complete"

# ── Install binary ─────────────────────────────────────────
echo "[3/5] Installing binary..."
mkdir -p "$HOME/.local/bin"
cp -v target/release/lincy "$HOME/.local/bin/lincy"
chmod 755 "$HOME/.local/bin/lincy"
echo "  ${GREEN}✓${RESET} Installed to ~/.local/bin/lincy"

# ── Install icons ──────────────────────────────────────────
echo "[4/5] Installing icons..."
for size in scalable symbolic; do
    src="$PROJECT_DIR/data/icons/hicolor/$size/apps"
    if [ -d "$src" ]; then
        dest="$HOME/.local/share/icons/hicolor/$size/apps"
        mkdir -p "$dest"
        cp -v "$src"/*.svg "$dest/"
    fi
done
# Also copy scalable icon to fixed sizes for tray compatibility
for px in 24 32 48; do
    mkdir -p "$HOME/.local/share/icons/hicolor/${px}x${px}/apps"
    cp "$PROJECT_DIR/data/icons/hicolor/scalable/apps/lincy.svg" \
       "$HOME/.local/share/icons/hicolor/${px}x${px}/apps/lincy.svg"
done
gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor/" 2>/dev/null || true
echo "  ${GREEN}✓${RESET} Icons installed"

# ── Install desktop entry + autostart ──────────────────────
echo "[5/5] Installing desktop entry and autostart..."
mkdir -p "$HOME/.local/share/applications" "$HOME/.config/autostart"

DESKTOP_SRC="$PROJECT_DIR/data/lincy.desktop"
if [ -f "$DESKTOP_SRC" ]; then
    # Substitute LINCY_BIN with the actual binary path
    sed "s|LINCY_BIN|$HOME/.local/bin/lincy|" "$DESKTOP_SRC" \
        > "$HOME/.local/share/applications/lincy.desktop"
    cp "$HOME/.local/share/applications/lincy.desktop" \
       "$HOME/.config/autostart/lincy.desktop"
    update-desktop-database "$HOME/.local/share/applications/" 2>/dev/null || true
    echo "  ${GREEN}✓${RESET} Desktop entry and autostart configured"
else
    echo "  ! data/lincy.desktop not found, skipping"
fi

# Register global shortcut (Ctrl+Shift+C)
echo "     Registering global shortcut Ctrl+Shift+C..."
SHORTCUT_PATH="/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/lincy/"
CURRENT=$(gsettings get org.gnome.settings-daemon.plugins.media-keys custom-keybindings 2>/dev/null || echo "@as []")
if ! echo "$CURRENT" | grep -q "lincy"; then
    if [ "$CURRENT" = "@as []" ]; then
        gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings "['$SHORTCUT_PATH']"
    else
        NEW_LIST=$(echo "$CURRENT" | sed "s|]|, '$SHORTCUT_PATH']|")
        gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings "$NEW_LIST"
    fi
fi
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$SHORTCUT_PATH name "Lincy"
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$SHORTCUT_PATH command "$HOME/.local/bin/lincy"
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$SHORTCUT_PATH binding "<Control><Shift>c"
echo "  ${GREEN}✓${RESET} Shortcut Ctrl+Shift+C → Lincy"

# ── Summary ────────────────────────────────────────────────
echo ""
echo "${BOLD}${GREEN}=== Installation complete! ===${RESET}"
echo ""
echo "Lincy is installed to:  ~/.local/bin/lincy"
echo "Icons:                  ~/.local/share/icons/hicolor/"
echo "Desktop entry:          ~/.local/share/applications/lincy.desktop"
echo "Autostart:              ~/.config/autostart/lincy.desktop"
echo "Database:               ~/.local/share/lincy/history.db"
echo ""
echo "${BOLD}To start now:${RESET}"
echo "  lincy &"
echo ""
echo "${BOLD}Hotkey:${RESET}"
echo "  Lincy tries to register Ctrl+Shift+C automatically."
echo "  If that fails, set it manually:"
echo "    GNOME Settings → Keyboard → Custom Shortcuts"
echo "    Name=Lincy, Command=lincy, Shortcut=Ctrl+Shift+C"
echo ""
echo "${BOLD}To uninstall:${RESET}"
echo "  ./scripts/uninstall.sh"

# Check PATH
if ! echo "$PATH" | tr ':' '\n' | grep -qFx "$HOME/.local/bin"; then
    echo ""
    echo "${BOLD}Note:${RESET} ~/.local/bin is not in your PATH."
    echo "  Add this to your ~/.bashrc or ~/.zshrc:"
    echo '    export PATH="$HOME/.local/bin:$PATH"'
fi
