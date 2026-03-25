#!/usr/bin/env bash
set -euo pipefail

PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"
SHARE_DIR="$PREFIX/share"
ICON_DIR="$SHARE_DIR/icons/hicolor"
APP_DIR="$SHARE_DIR/applications"

echo "╔══════════════════════════════════════╗"
echo "║       CYBERFILE — DEPLOY PHASE       ║"
echo "╚══════════════════════════════════════╝"
echo ""

# Build release binary (skip if already built)
if command -v cargo &>/dev/null; then
    echo "[1/4] Compiling release binary..."
    cargo build --release
elif [ -f target/release/cyberfile ]; then
    echo "[1/4] Using existing release binary (cargo not in PATH)"
else
    echo "[!] ERROR: cargo not found and no release binary exists."
    echo "    Run 'cargo build --release' first, then re-run this script."
    exit 1
fi

# Install binary
echo "[2/4] Installing binary to $BIN_DIR..."
install -Dm755 target/release/cyberfile "$BIN_DIR/cyberfile"

# Install desktop file
echo "[3/4] Installing desktop entry..."
install -Dm644 cyberfile.desktop "$APP_DIR/cyberfile.desktop"

# Install icon
echo "[4/4] Installing icon..."
if [ -f assets/icon.png ]; then
    install -Dm644 assets/icon.png "$ICON_DIR/256x256/apps/cyberfile.png"
    install -Dm644 assets/icon.png "$ICON_DIR/128x128/apps/cyberfile.png"
    install -Dm644 assets/icon.png "$ICON_DIR/64x64/apps/cyberfile.png"
fi
if [ -f assets/icon.svg ]; then
    install -Dm644 assets/icon.svg "$ICON_DIR/scalable/apps/cyberfile.svg"
fi

# Update icon cache
if command -v gtk-update-icon-cache &>/dev/null; then
    gtk-update-icon-cache -f "$ICON_DIR" 2>/dev/null || true
fi
if command -v update-desktop-database &>/dev/null; then
    update-desktop-database "$APP_DIR" 2>/dev/null || true
fi

echo ""
echo "══════════════════════════════════════"
echo "  CyberFile deployed to $PREFIX"
echo "  Run: cyberfile"
echo "══════════════════════════════════════"
