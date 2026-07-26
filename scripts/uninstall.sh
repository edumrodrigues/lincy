#!/bin/bash
set -euo pipefail

BOLD="$(tput bold 2>/dev/null || echo "")"
RED="$(tput setaf 1 2>/dev/null || echo "")"
GREEN="$(tput setaf 2 2>/dev/null || echo "")"
RESET="$(tput sgr0 2>/dev/null || echo "")"

echo "${BOLD}=== Lincy Uninstaller ===${RESET}"
echo ""

REMOVED=false

# 1. Kill running instance
if pgrep -f "lincy" > /dev/null 2>&1; then
    echo "[1/6] Stopping running instance..."
    pkill -f "lincy" 2>/dev/null || true
    sleep 1
    echo "  ${GREEN}✓${RESET} Stopped"
    REMOVED=true
else
    echo "[1/6] No running instance"
fi

# 2. Remove binary
if [ -f "$HOME/.local/bin/lincy" ]; then
    echo "[2/6] Removing binary..."
    rm -f "$HOME/.local/bin/lincy"
    echo "  ${GREEN}✓${RESET} Removed ~/.local/bin/lincy"
    REMOVED=true
else
    echo "[2/6] Binary not found"
fi

# 3. Remove desktop entry
if [ -f "$HOME/.local/share/applications/lincy.desktop" ]; then
    echo "[3/6] Removing desktop entry..."
    rm -f "$HOME/.local/share/applications/lincy.desktop"
    update-desktop-database "$HOME/.local/share/applications/" 2>/dev/null || true
    echo "  ${GREEN}✓${RESET} Removed"
    REMOVED=true
else
    echo "[3/6] Desktop entry not found"
fi

# 4. Remove autostart
if [ -f "$HOME/.config/autostart/lincy.desktop" ]; then
    echo "[4/6] Removing autostart..."
    rm -f "$HOME/.config/autostart/lincy.desktop"
    echo "  ${GREEN}✓${RESET} Removed"
    REMOVED=true
else
    echo "[4/6] Autostart not found"
fi

# 5. Remove icons
echo "[5/6] Removing icons..."
ICON_REMOVED=false
for size in scalable symbolic; do
    icon="$HOME/.local/share/icons/hicolor/$size/apps/lincy.svg"
    icon_symbolic="$HOME/.local/share/icons/hicolor/$size/apps/lincy-symbolic.svg"
    for f in "$icon" "$icon_symbolic"; do
        if [ -f "$f" ]; then
            rm -f "$f"
            echo "  Removed $f"
            ICON_REMOVED=true
        fi
    done
done
if [ "$ICON_REMOVED" = true ]; then
    gtk-update-icon-cache "$HOME/.local/share/icons/hicolor/" 2>/dev/null || true
    echo "  ${GREEN}✓${RESET} Icons removed"
    REMOVED=true
else
    echo "  No icons found"
fi

# 6. Remove data (ask)
if [ -d "$HOME/.local/share/lincy" ]; then
    echo "[6/6] Data directory found: $HOME/.local/share/lincy"
    read -p "  Remove clipboard history database? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$HOME/.local/share/lincy"
        echo "  ${GREEN}✓${RESET} Data removed"
        REMOVED=true
    else
        echo "  Kept at $HOME/.local/share/lincy"
    fi
else
    echo "[6/6] No data directory"
fi

echo ""
if [ "$REMOVED" = true ]; then
    echo "${BOLD}${GREEN}Lincy has been uninstalled.${RESET}"
else
    echo "${BOLD}Lincy was not installed.${RESET}"
fi
