#!/usr/bin/env bash
# ╔══════════════════════════════════════════════╗
# ║  CYBERFILE — PACKAGE BUILD SYSTEM            ║
# ║  Builds .deb, .rpm, .tar.gz, PKGBUILD        ║
# ╚══════════════════════════════════════════════╝
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# ── Extract version from Cargo.toml ─────────────────────
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)  DEB_ARCH="amd64"; RPM_ARCH="x86_64" ;;
    aarch64) DEB_ARCH="arm64";  RPM_ARCH="aarch64" ;;
    armv7l)  DEB_ARCH="armhf";  RPM_ARCH="armv7hl" ;;
    *)       DEB_ARCH="$ARCH";  RPM_ARCH="$ARCH" ;;
esac

PACKAGE="cyberfile"
RELEASE_BIN="target/release/$PACKAGE"
OUT_DIR="dist"

echo "╔══════════════════════════════════════════════╗"
echo "║  CYBERFILE PACKAGE BUILD v${VERSION}"
echo "╚══════════════════════════════════════════════╝"
echo ""

# ── Build release binary ────────────────────────────────
echo "[1/5] Building release binary..."
cargo build --release

if [ ! -f "$RELEASE_BIN" ]; then
    echo "ERROR: Release binary not found at $RELEASE_BIN"
    exit 1
fi

BINARY_SIZE=$(du -h "$RELEASE_BIN" | cut -f1)
echo "       Binary: $RELEASE_BIN ($BINARY_SIZE)"

mkdir -p "$OUT_DIR"

# ── .tar.gz (portable) ─────────────────────────────────
build_tarball() {
    echo "[2/5] Building portable tarball..."
    local TARBALL_NAME="${PACKAGE}-${VERSION}-linux-${ARCH}"
    local STAGING="/tmp/${TARBALL_NAME}"
    rm -rf "$STAGING"
    mkdir -p "$STAGING"

    cp "$RELEASE_BIN" "$STAGING/"
    cp cyberfile.desktop "$STAGING/"
    cp install.sh "$STAGING/"
    cp README.md "$STAGING/"
    [ -f LICENSE ] && cp LICENSE "$STAGING/" || true
    mkdir -p "$STAGING/assets"
    cp assets/icon-*.png "$STAGING/assets/" 2>/dev/null || true
    cp assets/icon.svg "$STAGING/assets/" 2>/dev/null || true
    mkdir -p "$STAGING/themes"
    cp themes/*.toml "$STAGING/themes/" 2>/dev/null || true

    tar -czf "$OUT_DIR/${TARBALL_NAME}.tar.gz" -C /tmp "$TARBALL_NAME"
    rm -rf "$STAGING"
    echo "       → $OUT_DIR/${TARBALL_NAME}.tar.gz"
}

# ── .deb (Debian / Ubuntu / Mint / Pop!_OS) ─────────────
build_deb() {
    echo "[3/5] Building .deb package..."
    if ! command -v dpkg-deb &>/dev/null; then
        echo "       SKIP: dpkg-deb not found"
        return
    fi

    local DEB_NAME="${PACKAGE}_${VERSION}_${DEB_ARCH}"
    local DEB_ROOT="/tmp/${DEB_NAME}"
    rm -rf "$DEB_ROOT"

    # Directory structure
    mkdir -p "$DEB_ROOT/DEBIAN"
    mkdir -p "$DEB_ROOT/usr/bin"
    mkdir -p "$DEB_ROOT/usr/share/applications"
    mkdir -p "$DEB_ROOT/usr/share/icons/hicolor/256x256/apps"
    mkdir -p "$DEB_ROOT/usr/share/icons/hicolor/128x128/apps"
    mkdir -p "$DEB_ROOT/usr/share/icons/hicolor/64x64/apps"
    mkdir -p "$DEB_ROOT/usr/share/icons/hicolor/48x48/apps"
    mkdir -p "$DEB_ROOT/usr/share/icons/hicolor/scalable/apps"
    mkdir -p "$DEB_ROOT/usr/share/doc/$PACKAGE"

    # Binary
    install -m 755 "$RELEASE_BIN" "$DEB_ROOT/usr/bin/$PACKAGE"

    # Desktop file (system-wide Exec path)
    sed "s|Exec=cyberfile|Exec=/usr/bin/cyberfile|" cyberfile.desktop \
        > "$DEB_ROOT/usr/share/applications/cyberfile.desktop"

    # Icons
    [ -f assets/icon-256.png ] && cp assets/icon-256.png "$DEB_ROOT/usr/share/icons/hicolor/256x256/apps/cyberfile.png"
    [ -f assets/icon-128.png ] && cp assets/icon-128.png "$DEB_ROOT/usr/share/icons/hicolor/128x128/apps/cyberfile.png"
    [ -f assets/icon-64.png ]  && cp assets/icon-64.png  "$DEB_ROOT/usr/share/icons/hicolor/64x64/apps/cyberfile.png"
    [ -f assets/icon-48.png ]  && cp assets/icon-48.png  "$DEB_ROOT/usr/share/icons/hicolor/48x48/apps/cyberfile.png"
    [ -f assets/icon.svg ]     && cp assets/icon.svg     "$DEB_ROOT/usr/share/icons/hicolor/scalable/apps/cyberfile.svg"

    # Docs
    cp README.md "$DEB_ROOT/usr/share/doc/$PACKAGE/"
    [ -f LICENSE ] && cp LICENSE "$DEB_ROOT/usr/share/doc/$PACKAGE/" || true

    # Installed size (in KiB)
    local INSTALLED_SIZE
    INSTALLED_SIZE=$(du -sk "$DEB_ROOT" | cut -f1)

    # Control file
    cat > "$DEB_ROOT/DEBIAN/control" << CTRL
Package: ${PACKAGE}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${DEB_ARCH}
Installed-Size: ${INSTALLED_SIZE}
Depends: libc6 (>= 2.31), libgcc-s1, libgl1, libegl1, libxkbcommon0, libwayland-client0, libx11-6, libxcursor1, libxrandr2, libxi6
Recommends: playerctl, xdotool, xprop
Suggests: nmcli, udisks2, fzf
Maintainer: SharkySeph <SharkySeph@users.noreply.github.com>
Homepage: https://github.com/SharkySeph/cyberfile
Description: Cyberpunk-themed file manager for Linux
 CyberFile is a GPU-accelerated, cyberpunk-themed file manager
 built with Rust and egui. Features include a hex grid view,
 process matrix, service deck, signal deck, network mesh,
 device bay, and tactical bridge for window management.
CTRL

    # Post-install: update icon cache
    cat > "$DEB_ROOT/DEBIAN/postinst" << 'POSTINST'
#!/bin/sh
set -e
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f /usr/share/icons/hicolor 2>/dev/null || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications 2>/dev/null || true
fi
POSTINST
    chmod 755 "$DEB_ROOT/DEBIAN/postinst"

    # Post-remove: clean icon cache
    cat > "$DEB_ROOT/DEBIAN/postrm" << 'POSTRM'
#!/bin/sh
set -e
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f /usr/share/icons/hicolor 2>/dev/null || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications 2>/dev/null || true
fi
POSTRM
    chmod 755 "$DEB_ROOT/DEBIAN/postrm"

    dpkg-deb --build --root-owner-group "$DEB_ROOT" "$OUT_DIR/${DEB_NAME}.deb"
    rm -rf "$DEB_ROOT"
    echo "       → $OUT_DIR/${DEB_NAME}.deb"
}

# ── .rpm (Fedora / RHEL / openSUSE) ─────────────────────
build_rpm() {
    echo "[4/5] Building .rpm package..."
    if ! command -v rpmbuild &>/dev/null; then
        echo "       SKIP: rpmbuild not found (install rpm-build)"
        return
    fi

    local RPM_BUILD="/tmp/rpmbuild-cyberfile"
    rm -rf "$RPM_BUILD"
    mkdir -p "$RPM_BUILD"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

    # Create tarball source
    local SRC_DIR="${PACKAGE}-${VERSION}"
    local SRC_STAGING="/tmp/${SRC_DIR}"
    rm -rf "$SRC_STAGING"
    mkdir -p "$SRC_STAGING"
    cp "$RELEASE_BIN" "$SRC_STAGING/"
    cp cyberfile.desktop "$SRC_STAGING/"
    cp README.md "$SRC_STAGING/"
    [ -f LICENSE ] && cp LICENSE "$SRC_STAGING/" || true
    mkdir -p "$SRC_STAGING/assets"
    cp assets/icon-*.png "$SRC_STAGING/assets/" 2>/dev/null || true
    cp assets/icon.svg "$SRC_STAGING/assets/" 2>/dev/null || true
    tar -czf "$RPM_BUILD/SOURCES/${SRC_DIR}.tar.gz" -C /tmp "$SRC_DIR"
    rm -rf "$SRC_STAGING"

    # Spec file
    cat > "$RPM_BUILD/SPECS/cyberfile.spec" << SPEC
Name:           ${PACKAGE}
Version:        ${VERSION}
Release:        1%{?dist}
Summary:        Cyberpunk-themed file manager for Linux
License:        MIT
URL:            https://github.com/SharkySeph/cyberfile
Source0:        %{name}-%{version}.tar.gz

Requires:       glibc libgcc mesa-libGL mesa-libEGL libxkbcommon wayland libX11 libXcursor libXrandr libXi

%description
CyberFile is a GPU-accelerated, cyberpunk-themed file manager
built with Rust and egui. Features include a hex grid view,
process matrix, service deck, signal deck, network mesh,
device bay, and tactical bridge for window management.

%prep
%setup -q

%install
install -Dm755 ${PACKAGE} %{buildroot}/usr/bin/${PACKAGE}
sed "s|Exec=cyberfile|Exec=/usr/bin/cyberfile|" cyberfile.desktop > cyberfile-patched.desktop
install -Dm644 cyberfile-patched.desktop %{buildroot}/usr/share/applications/cyberfile.desktop
[ -f assets/icon-256.png ] && install -Dm644 assets/icon-256.png %{buildroot}/usr/share/icons/hicolor/256x256/apps/cyberfile.png
[ -f assets/icon-128.png ] && install -Dm644 assets/icon-128.png %{buildroot}/usr/share/icons/hicolor/128x128/apps/cyberfile.png
[ -f assets/icon-64.png ]  && install -Dm644 assets/icon-64.png  %{buildroot}/usr/share/icons/hicolor/64x64/apps/cyberfile.png
[ -f assets/icon-48.png ]  && install -Dm644 assets/icon-48.png  %{buildroot}/usr/share/icons/hicolor/48x48/apps/cyberfile.png
[ -f assets/icon.svg ]     && install -Dm644 assets/icon.svg     %{buildroot}/usr/share/icons/hicolor/scalable/apps/cyberfile.svg

%files
/usr/bin/${PACKAGE}
/usr/share/applications/cyberfile.desktop
/usr/share/icons/hicolor/*/apps/cyberfile.*

%post
gtk-update-icon-cache -f /usr/share/icons/hicolor 2>/dev/null || true
update-desktop-database /usr/share/applications 2>/dev/null || true

%postun
gtk-update-icon-cache -f /usr/share/icons/hicolor 2>/dev/null || true
update-desktop-database /usr/share/applications 2>/dev/null || true
SPEC

    rpmbuild --define "_topdir $RPM_BUILD" -bb "$RPM_BUILD/SPECS/cyberfile.spec"
    find "$RPM_BUILD/RPMS" -name "*.rpm" -exec cp {} "$OUT_DIR/" \;
    rm -rf "$RPM_BUILD"
    echo "       → $OUT_DIR/*.rpm"
}

# ── PKGBUILD (Arch Linux / Manjaro / EndeavourOS) ────────
build_pkgbuild() {
    echo "[5/5] Generating Arch PKGBUILD..."
    local PKGBUILD_DIR="$OUT_DIR/arch"
    mkdir -p "$PKGBUILD_DIR"

    cat > "$PKGBUILD_DIR/PKGBUILD" << 'PKGBUILD'
# Maintainer: SharkySeph <SharkySeph@users.noreply.github.com>
pkgname=cyberfile
pkgver=_VERSION_
pkgrel=1
pkgdesc="Cyberpunk-themed file manager for Linux"
arch=('x86_64' 'aarch64')
url="https://github.com/SharkySeph/cyberfile"
license=('MIT')
depends=('gcc-libs' 'glibc' 'libgl' 'libegl' 'libxkbcommon' 'wayland' 'libx11' 'libxcursor' 'libxrandr' 'libxi')
optdepends=(
    'playerctl: media widget control'
    'xdotool: X11 window management'
    'xorg-xprop: X11 window listing'
    'wmctrl: X11 window management'
    'network-manager: network mesh panel'
    'udisks2: device bay mount/unmount'
    'fzf: fuzzy file search'
)
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/SharkySeph/cyberfile/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$pkgname-$pkgver"
    cargo build --release --locked
}

package() {
    cd "$pkgname-$pkgver"
    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
    install -Dm644 cyberfile.desktop "$pkgdir/usr/share/applications/cyberfile.desktop"
    install -Dm644 assets/icon-256.png "$pkgdir/usr/share/icons/hicolor/256x256/apps/cyberfile.png"
    install -Dm644 assets/icon-128.png "$pkgdir/usr/share/icons/hicolor/128x128/apps/cyberfile.png"
    install -Dm644 assets/icon-64.png "$pkgdir/usr/share/icons/hicolor/64x64/apps/cyberfile.png"
    install -Dm644 assets/icon-48.png "$pkgdir/usr/share/icons/hicolor/48x48/apps/cyberfile.png"
    [ -f assets/icon.svg ] && install -Dm644 assets/icon.svg "$pkgdir/usr/share/icons/hicolor/scalable/apps/cyberfile.svg"
}
PKGBUILD

    # Substitute version
    sed -i "s/_VERSION_/${VERSION}/" "$PKGBUILD_DIR/PKGBUILD"
    echo "       → $PKGBUILD_DIR/PKGBUILD"
}

# ── Run all builders ────────────────────────────────────
echo ""
build_tarball
build_deb
build_rpm
build_pkgbuild

echo ""
echo "══════════════════════════════════════════════"
echo "  BUILD COMPLETE — v${VERSION}"
echo ""
echo "  Artifacts in $OUT_DIR/:"
ls -lh "$OUT_DIR/" 2>/dev/null | grep -v "^total"
echo ""
echo "  Install .deb:    sudo dpkg -i $OUT_DIR/${PACKAGE}_${VERSION}_${DEB_ARCH}.deb"
echo "  Install .rpm:    sudo rpm -i $OUT_DIR/*.rpm"
echo "  Install tarball: tar xzf $OUT_DIR/*.tar.gz && cd ${PACKAGE}-*/ && ./install.sh"
echo "  Install Arch:    cd $OUT_DIR/arch && makepkg -si"
echo "══════════════════════════════════════════════"
