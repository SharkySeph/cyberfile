#!/usr/bin/env bash
# CYBERFILE — UNINSTALL
# Removes a user-local install (from install.sh)
set -euo pipefail

PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"
SHARE_DIR="$PREFIX/share"
ICON_DIR="$SHARE_DIR/icons/hicolor"
APP_DIR="$SHARE_DIR/applications"

echo "╔══════════════════════════════════════╗"
echo "║     CYBERFILE — UNINSTALL PHASE      ║"
echo "╚══════════════════════════════════════╝"
echo ""

rm -fv "$BIN_DIR/cyberfile"
rm -fv "$APP_DIR/cyberfile.desktop"
rm -fv "$ICON_DIR/256x256/apps/cyberfile.png"
rm -fv "$ICON_DIR/128x128/apps/cyberfile.png"
rm -fv "$ICON_DIR/64x64/apps/cyberfile.png"
rm -fv "$ICON_DIR/48x48/apps/cyberfile.png"
rm -fv "$ICON_DIR/scalable/apps/cyberfile.svg"

if command -v gtk-update-icon-cache &>/dev/null; then
    gtk-update-icon-cache -f "$ICON_DIR" 2>/dev/null || true
fi
if command -v update-desktop-database &>/dev/null; then
    update-desktop-database "$APP_DIR" 2>/dev/null || true
fi

echo ""
echo "  CyberFile removed from $PREFIX"
echo ""
echo "  To remove system-wide (.deb): sudo dpkg -r cyberfile"
echo "  To remove system-wide (.rpm): sudo rpm -e cyberfile"
