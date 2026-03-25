# CYBERFILE

**A cyberpunk-themed file manager for Linux.**

> Files are data constructs. Directories are sectors. You are the operator.

Built with Rust and egui — CyberFile replaces the conventional desktop file manager metaphor with an immersive cyberpunk HUD interface inspired by Cyberpunk 2077, Ghost in the Shell, Evangelion, and Hackers.

---

## Features

### Core File Management

- Full CRUD operations: copy, cut, paste, rename, delete, create file/folder, symlinks
- Undo/Redo stack (Ctrl+Z / Ctrl+Shift+Z) for all operations
- Batch rename with find/replace and regex support
- Archive handling: ZIP browsing, extraction, and compression
- System clipboard bridge (xclip / wl-copy)
- Confirm-delete dialog ("PURGE PROTOCOL")
- Soft-delete to trash ("CONTAINMENT ZONE") with restore

### Navigation

- Breadcrumb path bar with editable path input
- Unlimited tabs with drag-to-reorder (Ctrl+T / Ctrl+W)
- Split/dual pane view (F4) with independent navigation
- Bookmarks sidebar ("Neural Links")
- Command bar with fuzzy search via fzf
- Content search via grep/ripgrep ("DEEP SCAN")
- Back/forward history, Go Up (Backspace)

### 4 View Modes

- **List** (Ctrl+1) — Sortable columns: name, size, date, permissions, extension
- **Grid** (Ctrl+2) — Thumbnail cards with image previews
- **HIVE** (Ctrl+3) — Hexagonal grid with hex context menu
- **Hex Viewer** (Ctrl+4) — Binary hex dump view

### Selection

- Click, Ctrl+Click, Shift+Click range, Ctrl+A select all
- Rubber band (click-drag) selection in Grid view
- Internal drag-and-drop to split pane directories

### Preview Panel

- Text/code preview with syntax highlighting (12+ languages)
- Image thumbnails (JPEG, PNG, SVG, WebP)
- ZIP archive contents listing
- File metadata and properties (Ctrl+I)
- Visual permissions editor (chmod checkboxes)

### Visual Effects (All Togglable)

- CRT scanlines (F11)
- CRT vignette (F12)
- Matrix-style data rain (F10)
- Neon glow / bloom (F8)
- Chromatic aberration (F6)
- Holographic noise
- Glitch transitions
- HUD corner brackets
- High contrast mode

### 8 Cyberpunk Themes

| Theme | Inspiration |
|-------|-------------|
| **NIGHT CITY** | Cyberpunk 2077 — cyan + magenta |
| **SECTION 9** | Ghost in the Shell — teal + violet |
| **MAGI SYSTEM** | Evangelion NERV — orange + crimson |
| **GIBSON** | Hackers — amber + green |
| **TYRELL** | Blade Runner — steel blue + gold |
| **AKIRA** | Akira / Neo-Tokyo — capsule red + silver |
| **WINTERMUTE** | Neuromancer — ice blue + chrome |
| **OUTRUN** | Synthwave — hot pink + electric blue |

### Integrations

- **Resource Monitor** (F3) — CPU, RAM, swap, disk with sparklines
- **Music Widget** — MPRIS/playerctl playback controls in sidebar
- **Embedded Terminal** (F7) — Command runner with output display
- **fzf** — Fuzzy file search (Ctrl+F)
- **SFTP/SSH** (F9) — Remote file browsing via SSH key or password auth
- **D-Bus** — CLI path arguments, `--show-item` support
- **Sound Effects** — Synthesized UI audio via rodio (togglable)

### Boot Sequence

POST-style startup animation with progress bar — skippable.

---

## Installation

### From Source

```bash
# Dependencies (Debian/Ubuntu)
sudo apt install libssl-dev libasound2-dev pkg-config

# Build
cargo build --release

# Run
./target/release/cyberfile
```

### System Install

```bash
# Install to /usr/local (requires sudo)
sudo ./install.sh

# Or install to custom prefix
PREFIX=~/.local ./install.sh
```

The install script copies the binary, .desktop file, and icon to the appropriate directories.

---

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| **F1** | Settings panel |
| **F2** | Rename selected |
| **F3** | Resource monitor |
| **F4** | Split pane |
| **F5** | Refresh directory |
| **F6** | Chromatic aberration toggle |
| **F7** | Embedded terminal |
| **F8** | Neon glow toggle |
| **F9** | SFTP remote dialog |
| **F10** | Data rain toggle |
| **F11** | Scanlines toggle |
| **F12** | CRT vignette toggle |
| **Ctrl+1/2/3/4** | List / Grid / HIVE / Hex view |
| **Ctrl+T** | New tab |
| **Ctrl+W** | Close tab |
| **Ctrl+B** | Toggle sidebar |
| **Ctrl+P** | Toggle preview panel |
| **Ctrl+H** | Toggle hidden files |
| **Ctrl+F** | fzf fuzzy search |
| **Ctrl+G** | Content search (DEEP SCAN) |
| **Ctrl+R** | Batch rename (multi-select) |
| **Ctrl+I** | Properties dialog |
| **Ctrl+C/X/V** | Copy / Cut / Paste |
| **Ctrl+N** | New file |
| **Ctrl+Shift+N** | New folder |
| **Ctrl+A** | Select all |
| **Ctrl+Z** | Undo |
| **Ctrl+Shift+Z** | Redo |
| **Backspace** | Navigate up |
| **Delete** | Move to trash |
| **Escape** | Close overlays |

---

## Configuration

Settings are stored in `~/.config/cyberfile/config.toml` and persisted automatically. The settings panel (F1) provides a themed UI for all options.

---

## Requirements

- Linux (X11 or Wayland)
- Rust 1.70+ (for building)
- OpenGL 3.3+ or Vulkan-capable GPU
- Optional: `fzf` for fuzzy search, `playerctl` for music widget, `libssh2` for SFTP

---

## License

MIT
