#!/bin/bash
set -euo pipefail

# Lincy — one-line installer. Downloads the latest release binary.
# Usage: curl -fsSL https://raw.githubusercontent.com/edumrodrigues/lincy/main/scripts/install-remote.sh | bash

BOLD="$(tput bold 2>/dev/null || echo "")"
GREEN="$(tput setaf 2 2>/dev/null || echo "")"
RESET="$(tput sgr0 2>/dev/null || echo "")"

echo "${BOLD}=== Lincy Installer (remote) ===${RESET}"
echo ""

REPO="edumrodrigues/lincy"
BIN_DIR="$HOME/.local/bin"
INSTALL_PATH="$BIN_DIR/lincy"

# ── System dependencies ────────────────────────────────────
echo "[1/4] Checking dependencies..."
MISSING=""
for pkg in libgtk-4-1 libadwaita-1-0 libsqlite3-0; do
    if ! dpkg -s "$pkg" &>/dev/null 2>&1; then
        MISSING="$MISSING $pkg"
    fi
done
if [ -n "$MISSING" ]; then
    echo "  Installing runtime dependencies:$MISSING"
    if command -v apt &>/dev/null; then
        sudo apt install -y $MISSING
    elif command -v dnf &>/dev/null; then
        sudo dnf install -y gtk4 libadwaita sqlite-libs
    elif command -v pacman &>/dev/null; then
        sudo pacman -S --noconfirm gtk4 libadwaita sqlite
    else
        echo "  ! Unknown package manager. Install these manually: gtk4 libadwaita sqlite"
    fi
else
    echo "  ${GREEN}✓${RESET} All dependencies present"
fi

# ── Download latest release ────────────────────────────────
echo "[2/4] Downloading latest release..."
LATEST_URL=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" | grep "browser_download_url.*lincy-linux-x86_64.tar.gz" | cut -d'"' -f4)
if [ -z "$LATEST_URL" ]; then
    echo "  ! No release found. Trying to build from source..."
    echo "  Install Rust first: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "  Then: cargo install lincy"
    exit 1
fi
curl -fsSL "$LATEST_URL" -o /tmp/lincy.tar.gz
echo "  ${GREEN}✓${RESET} Downloaded"

# ── Extract ────────────────────────────────────────────────
echo "[3/4] Installing..."
mkdir -p "$BIN_DIR"
tar xf /tmp/lincy.tar.gz -C /tmp/
cp /tmp/lincy/lincy "$INSTALL_PATH"
chmod 755 "$INSTALL_PATH"

# Install icons (tarball has icons/ not data/icons/)
mkdir -p "$HOME/.local/share/icons/hicolor/scalable/apps" "$HOME/.local/share/icons/hicolor/symbolic/apps"
cp /tmp/lincy/icons/hicolor/scalable/apps/lincy.svg "$HOME/.local/share/icons/hicolor/scalable/apps/" 2>/dev/null || true
cp /tmp/lincy/icons/hicolor/symbolic/apps/lincy-symbolic.svg "$HOME/.local/share/icons/hicolor/symbolic/apps/" 2>/dev/null || true
for px in 24 32 48; do
    mkdir -p "$HOME/.local/share/icons/hicolor/${px}x${px}/apps"
    cp /tmp/lincy/icons/hicolor/scalable/apps/lincy.svg "$HOME/.local/share/icons/hicolor/${px}x${px}/apps/lincy.svg" 2>/dev/null || true
done
gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor/" 2>/dev/null || true

# Desktop entry + autostart (tarball has lincy.desktop at root)
mkdir -p "$HOME/.local/share/applications" "$HOME/.config/autostart"
sed "s|LINCY_BIN|$INSTALL_PATH|" /tmp/lincy/lincy.desktop > "$HOME/.local/share/applications/lincy.desktop"
cp "$HOME/.local/share/applications/lincy.desktop" "$HOME/.config/autostart/lincy.desktop"
update-desktop-database "$HOME/.local/share/applications/" 2>/dev/null || true

echo "  ${GREEN}✓${RESET} Installed to $INSTALL_PATH"

# ── Cleanup ────────────────────────────────────────────────
rm -f /tmp/lincy.tar.gz
rm -rf /tmp/lincy

echo ""
echo "${BOLD}${GREEN}=== Installation complete! ===${RESET}"
echo ""
echo "Lincy is now installed. It will auto-start on next login."
echo ""
echo "  Start now:  lincy &"
echo "  Open popup: lincy"
echo "  Shortcut:   Ctrl+Shift+C (registered on first run)"
echo ""
if ! echo "$PATH" | tr ':' '\n' | grep -qFx "$BIN_DIR"; then
    echo "${BOLD}Note:${RESET} Add $BIN_DIR to your PATH:"
    echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
fi
