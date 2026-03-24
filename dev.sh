#!/usr/bin/env bash
# CYBERFILE // DEV LAUNCHER
# Quick launch script for development work
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

usage() {
    echo "CYBERFILE // Dev Launcher"
    echo ""
    echo "Usage: ./dev.sh [command]"
    echo ""
    echo "Commands:"
    echo "  run          Build and run (debug mode, default)"
    echo "  release      Build and run (release/optimized)"
    echo "  build        Build only (debug)"
    echo "  check        Fast type-check (no codegen)"
    echo "  watch        Rebuild on file changes (requires cargo-watch)"
    echo "  clean        Clean build artifacts"
    echo "  deps         Check for optional runtime dependencies"
    echo ""
}

check_deps() {
    echo "=== CYBERFILE Dependency Check ==="
    echo ""

    echo -n "  Rust toolchain: "
    if command -v rustc &>/dev/null; then
        echo "$(rustc --version)"
    else
        echo "MISSING (install from https://rustup.rs)"
    fi

    echo -n "  cargo:          "
    if command -v cargo &>/dev/null; then
        echo "$(cargo --version)"
    else
        echo "MISSING"
    fi

    echo ""
    echo "  Optional runtime tools:"

    echo -n "    playerctl (media control): "
    if command -v playerctl &>/dev/null; then
        echo "$(playerctl --version)"
    else
        echo "not found (install for media widget)"
    fi

    echo -n "    fzf (fuzzy finder):        "
    if command -v fzf &>/dev/null; then
        echo "$(fzf --version 2>&1 | head -1)"
    else
        echo "not found (install for fuzzy file search)"
    fi

    echo -n "    cargo-watch:               "
    if command -v cargo-watch &>/dev/null; then
        echo "installed"
    else
        echo "not found (install for 'watch' command: cargo install cargo-watch)"
    fi

    echo ""
}

CMD="${1:-run}"

case "$CMD" in
    run)
        echo "[CYBERFILE] Building and launching (debug)..."
        cargo run 2>&1
        ;;
    release)
        echo "[CYBERFILE] Building and launching (release)..."
        cargo run --release 2>&1
        ;;
    build)
        echo "[CYBERFILE] Building (debug)..."
        cargo build 2>&1
        ;;
    check)
        echo "[CYBERFILE] Type-checking..."
        cargo check 2>&1
        ;;
    watch)
        if ! command -v cargo-watch &>/dev/null; then
            echo "cargo-watch not installed. Install with: cargo install cargo-watch"
            exit 1
        fi
        echo "[CYBERFILE] Watching for changes..."
        cargo watch -x run
        ;;
    clean)
        echo "[CYBERFILE] Cleaning build artifacts..."
        cargo clean
        echo "Done."
        ;;
    deps)
        check_deps
        ;;
    -h|--help|help)
        usage
        ;;
    *)
        echo "Unknown command: $CMD"
        usage
        exit 1
        ;;
esac
