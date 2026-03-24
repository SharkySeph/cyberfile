# CYBERFILE — Cyberpunk File Management System

## Project Overview

**Codename:** CYBERFILE  
**Platform:** Linux (X11/Wayland)  
**Type:** Desktop file manager with immersive cyberpunk UI  
**Aesthetic Sources:** Cyberpunk 2077, Ghost in the Shell, Hackers, Neon Genesis Evangelion

A file management system that replaces the conventional desktop metaphor with a cyberpunk terminal/HUD interface — files are navigated, manipulated, and visualized as if you're jacked into a futuristic operating system.

---

## 1. Visual Design Language

### 1.1 Color Palette

| Role | Color | Hex | Source Influence |
|------|-------|-----|------------------|
| Primary | Hot cyan | `#00F0FF` | Ghost in the Shell data streams |
| Secondary | Neon magenta | `#FF2079` | Cyberpunk 2077 UI accents |
| Tertiary | Acid yellow | `#F7F32A` | Cyberpunk 2077 warning text |
| Background | Deep black | `#0A0A0F` | Universal |
| Surface | Dark navy | `#0D1117` | NERV terminal screens |
| Danger | MAGI red | `#FF3333` | Evangelion MAGI alerts |
| Success | Matrix green | `#39FF14` | Hackers terminal output |
| Text Primary | Cool white | `#E0E0E8` | — |
| Text Dim | Faded cyan | `#4A7A7F` | Ghost in the Shell subtitles |

### 1.2 Typography

- **Primary UI Font:** Monospaced, custom or based on JetBrains Mono / Share Tech Mono
- **Headers/Titles:** Rajdhani or Orbitron — angular, condensed, futuristic
- **System Messages:** OCR-A or a custom bitmap-style font for "machine readout" feel
- **Japanese Glyphs:** Noto Sans JP for decorative kanji overlays (Evangelion/GitS influence)

### 1.3 Core Visual Elements

- **Scanlines:** Subtle horizontal CRT scanline overlay (togglable)
- **Glitch Effects:** Micro-glitch on transitions and error states (chromatic aberration, horizontal displacement)
- **HUD Borders:** Thin-line geometric borders with corner brackets `[ ]` instead of rounded corners
- **Data Rain:** Optional background particle effect — falling characters (GitS/Matrix style)
- **Holographic Noise:** Subtle animated noise texture on panels (low opacity)
- **NERV-style Labels:** Uppercase system labels with decorative kanji watermarks behind panels
- **Boot Sequence:** Startup shows a POST-style boot log before UI renders

### 1.4 Iconography

- Wireframe/outline icons only — no filled icons
- Geometric, angular style (triangles, hexagons, chevrons)
- File type icons use abstract glyphs, not skeuomorphic representations
- Folders represented as "data nodes" or "sectors"

---

## 2. Architecture

### 2.1 Technology Stack

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| Language | Rust | Performance, safety, Linux-native |
| UI Framework | iced-rs or egui | GPU-accelerated, Rust-native, custom rendering |
| Rendering | wgpu | Shader effects (scanlines, glitch, glow) |
| File System | std::fs + inotify | Native Linux FS access + real-time watching |
| Config | TOML | Human-readable, Rust ecosystem standard |
| Theming | Custom shader pipeline | CRT effects, glow, chromatic aberration |
| Audio | rodio | UI feedback sounds |
| IPC | D-Bus | Desktop integration |

### 2.2 Module Architecture

```
cyberfile/
├── src/
│   ├── main.rs                  # Entry point, boot sequence
│   ├── app.rs                   # Application state machine
│   ├── config/
│   │   ├── mod.rs
│   │   ├── settings.rs          # User preferences
│   │   └── keybinds.rs          # Keyboard shortcut mapping
│   ├── core/
│   │   ├── mod.rs
│   │   ├── filesystem.rs        # FS operations (CRUD, permissions, watch)
│   │   ├── search.rs            # File search engine
│   │   ├── clipboard.rs         # Cut/copy/paste state
│   │   ├── bookmarks.rs         # Saved locations ("neural links")
│   │   └── trash.rs             # Soft-delete / "quarantine zone"
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── shell.rs             # Main window chrome / HUD frame
│   │   ├── nav_panel.rs         # Left sidebar — directory tree ("network map")
│   │   ├── file_grid.rs         # Main content area — file listing
│   │   ├── file_list.rs         # Alternative list/detail view
│   │   ├── preview_panel.rs     # Right panel — file preview ("data scan")
│   │   ├── status_bar.rs        # Bottom bar — path, stats, system readout
│   │   ├── command_bar.rs       # Top command input ("neural interface")
│   │   ├── context_menu.rs      # Right-click actions
│   │   ├── dialog.rs            # Modal dialogs (confirm, rename, properties)
│   │   ├── breadcrumb.rs        # Path breadcrumb with glitch transitions
│   │   ├── tabs.rs              # Multi-tab support
│   │   └── boot_screen.rs       # Startup POST sequence
│   ├── fx/
│   │   ├── mod.rs
│   │   ├── shaders/
│   │   │   ├── scanline.wgsl    # CRT scanline overlay
│   │   │   ├── glow.wgsl        # Neon glow / bloom
│   │   │   ├── glitch.wgsl      # Chromatic aberration + displacement
│   │   │   ├── noise.wgsl       # Holographic noise texture
│   │   │   └── vignette.wgsl    # Edge darkening
│   │   ├── animations.rs        # Transition animations
│   │   ├── particles.rs         # Data rain / floating particles
│   │   └── audio.rs             # UI sound effects
│   ├── widgets/
│   │   ├── mod.rs
│   │   ├── cyber_button.rs      # Styled button with hover glow
│   │   ├── cyber_input.rs       # Text input with cursor blink effect
│   │   ├── cyber_scrollbar.rs   # Thin neon scrollbar
│   │   ├── cyber_tooltip.rs     # Tooltip with bracket styling
│   │   ├── progress_bar.rs      # Segmented progress (file operations)
│   │   ├── hex_viewer.rs        # Binary file hex view
│   │   └── data_graph.rs        # Disk usage visualization
│   └── integration/
│       ├── mod.rs
│       ├── dbus.rs              # Desktop environment integration
│       ├── xdg.rs               # XDG directory compliance
│       ├── thumbnails.rs        # Thumbnail generation
│       └── open_with.rs         # Application launcher
├── assets/
│   ├── fonts/
│   ├── icons/
│   ├── sounds/
│   └── shaders/
├── themes/
│   ├── default.toml             # "Night City" — Cyberpunk 2077
│   ├── gits.toml                # "Section 9" — Ghost in the Shell
│   ├── nerv.toml                # "MAGI System" — Evangelion
│   └── hackers.toml             # "Gibson" — Hackers
├── Cargo.toml
└── README.md
```

---

## 3. Feature Specification

### 3.1 Core File Management

| Feature | Description | Cyberpunk Flavor |
|---------|-------------|------------------|
| Browse | Navigate directories | Directories are "sectors", drives are "nodes" |
| Create | New files/folders | "Initialize new data construct" |
| Rename | Rename files | "Reassign identifier" |
| Copy/Move | File transfer | Progress shown as "data transfer" with segmented bar |
| Delete | Move to trash | "Quarantine" — trash is the "containment zone" |
| Search | Find files | "Neural scan" — command bar with fuzzy matching |
| Permissions | View/edit perms | "Access level" display in octal + visual badge |
| Properties | File metadata | "Data profile" panel |
| Bulk Operations | Multi-select actions | "Batch protocol" |

### 3.2 Navigation

- **Command Bar (top):** Quick-nav by typing paths or search queries — styled as "neural interface" input with autocomplete dropdown
- **Directory Tree (left):** Collapsible tree with neon connector lines between nodes — labeled "NETWORK MAP"
- **Breadcrumb Path:** Each segment is a clickable chip with `/` separators rendered as chevron `›` glyphs
- **Tabs:** Multiple location tabs across the top — styled as terminal session indicators
- **Bookmarks:** Saved paths called "Neural Links" — quick-access sidebar section
- **History:** Back/forward navigation with a timeline view

### 3.3 File Views

#### Grid View ("Construct Array")

- Thumbnail cards with thin cyan borders
- Hover state: glow effect + expanded info
- Selection: magenta highlight border with corner brackets

#### List View ("Data Stream")

- Dense row-based view with columns: Name, Size, Modified, Type, Permissions
- Column headers styled as system readout labels
- Alternating row opacity for readability
- Sort indicators are small animated chevrons

#### Hex View ("Raw Decode")

- Binary file viewer with hex + ASCII columns
- Syntax highlighting for known binary structures
- Offset column in dim cyan

### 3.4 Preview Panel ("Data Scan")

- Right sidebar, togglable
- **Images:** Rendered with scanline overlay
- **Text/Code:** Syntax-highlighted preview with line numbers
- **Audio:** Waveform visualizer with neon gradient
- **Video:** Thumbnail + metadata
- **Archives:** Contents listing
- **Unknown:** Hex preview + file signature analysis

### 3.5 Visual Effects System

All effects are independently togglable and have intensity sliders:

| Effect | Default | Performance Cost |
|--------|---------|-----------------|
| Scanlines | On (20% opacity) | Low |
| Neon Glow | On | Medium |
| Glitch Transitions | On | Low |
| Data Rain Background | Off | Medium |
| Chromatic Aberration | On (subtle) | Low |
| CRT Vignette | On | Low |
| Holographic Noise | Off | Medium |
| Ambient Sound | Off | None (audio) |

### 3.6 Sound Design

- **Navigation click:** Short electronic blip
- **File select:** Soft tonal ping
- **Error:** Harsh digital buzz (Evangelion alarm inspired)
- **Delete/Quarantine:** Low descending tone
- **Copy complete:** Ascending confirmation chime
- **Boot sequence:** Layered synth startup sequence
- All sounds optional with master volume control

### 3.7 Themed Experiences

#### "Night City" (Default — Cyberpunk 2077)

- Hot cyan + magenta palette
- Yellow warning accents
- Aggressive angular UI elements
- Glitch effects prominent

#### "Section 9" (Ghost in the Shell)

- Teal + desaturated green palette
- Data stream background particles
- Cleaner, more minimal UI
- Floating translucent panels

#### "MAGI System" (Neon Genesis Evangelion)

- Orange + red warning palette on dark backgrounds
- Bold uppercase labels with Japanese text overlays
- NERV diamond logo as watermark
- System status indicators everywhere (MELCHIOR / BALTHASAR / CASPAR style readouts for disk health)
- Alert borders that pulse red on errors

#### "Gibson" (Hackers)

- Green-on-black terminal aesthetic
- More playful, 90s-rave influenced accents
- ASCII art decorative elements
- "Hack the planet" loading messages
- Wireframe 3D globe decoration element

---

## 4. UX Specifications

### 4.1 Boot Sequence

On launch, display a 2-3 second boot screen (skippable):

```
[SYSTEM] CYBERFILE v0.1.0
[SYSTEM] Initializing kernel interface... OK
[SYSTEM] Mounting filesystem nodes...
[  OK  ] /home — USER DATA SECTOR
[  OK  ] /media — EXTERNAL NODES
[  OK  ] /tmp — VOLATILE CACHE
[SYSTEM] Loading neural interface...
[SYSTEM] Scanning file signatures... 142,387 constructs indexed
[SYSTEM] STATUS: OPERATIONAL
[ BOOT ] ████████████████████████████ 100%

> WELCOME BACK, OPERATOR.
```

### 4.2 Keyboard-First Design

| Shortcut | Action | Display Name |
|----------|--------|-------------|
| `/` or `Ctrl+K` | Open command bar | "Neural Interface" |
| `Ctrl+L` | Focus path bar | "Set coordinates" |
| `Space` | Quick preview | "Data scan" |
| `Enter` | Open file/folder | "Access" |
| `Delete` | Quarantine file | "Quarantine" |
| `Ctrl+C/V/X` | Copy/Paste/Cut | Standard |
| `Ctrl+Shift+N` | New folder | "Init sector" |
| `F2` | Rename | "Reassign ID" |
| `Ctrl+H` | Toggle hidden files | "Reveal cloaked" |
| `Tab` | Next panel focus | "Cycle interface" |
| `Ctrl+1/2/3` | Switch views | Grid/List/Hex |
| `Ctrl+T` | New tab | "Open channel" |
| `Ctrl+W` | Close tab | "Terminate channel" |
| `Ctrl+B` | Toggle sidebar | "Toggle map" |
| `Ctrl+P` | Toggle preview | "Toggle scan" |

### 4.3 Context Menu

Styled as a translucent dark panel with neon border, items include:

```
┌─[ ACTIONS ]──────────────────┐
│  ▸ Open                      │
│  ▸ Open with...              │
│  ─────────────────────────── │
│  ▸ Copy         Ctrl+C       │
│  ▸ Cut          Ctrl+X       │
│  ▸ Paste        Ctrl+V       │
│  ─────────────────────────── │
│  ▸ Reassign ID  F2           │
│  ▸ Quarantine   Del          │
│  ─────────────────────────── │
│  ▸ Data Profile              │
│  ▸ Access Levels             │
│  ▸ Compress → Archive        │
└──────────────────────────────┘
```

### 4.4 Status Bar Layout

```
[ /home/user/Documents ]  ◈ 47 constructs  |  ◈ 2.3 GB sector  |  ◈ 128 GB free  |  DISK: ████░░ 67%  |  12:47:33
```

---

## Current Implementation Status

> **Last Updated:** Functionality audit — Dolphin parity analysis complete

### Implemented & Working

| Module | Status | Details |
|--------|--------|---------|
| **Core File Browser** | ✅ Complete | Directory listing, navigation, breadcrumbs, sorting (name/size/date/perms) |
| **File Operations** | ✅ Complete | Copy, cut, paste, delete-to-trash, rename, create folder |
| **4 View Modes** | ✅ Complete | List (Ctrl+1), Grid (Ctrl+2), HIVE/HexGrid (Ctrl+3), Hex Viewer (Ctrl+4) |
| **Command Bar** | ✅ Complete | Path navigation, fzf search, neural interface input |
| **Tab System** | ✅ Complete | Unlimited tabs, per-tab path/selection, Ctrl+T/W, close button |
| **Sidebar** | ✅ Complete | Quick access (8 dirs), bookmarks ("Neural Links"), disk stats, music widget |
| **Status Bar** | ✅ Complete | Path, count, size, view mode, fzf status, clock, selection info |
| **Boot Sequence** | ✅ Complete | 11-line POST animation with progress bar, skippable |
| **Keyboard Shortcuts** | ✅ Complete | 30+ shortcuts: F1-F12, Ctrl combos, arrow nav, Backspace/Enter |
| **Context Menu** | ✅ Complete | Themed menu + HIVE-mode hex variant with ⬡ bullets and hex border |
| **Config System** | ✅ Complete | TOML persistence via XDG config dir (theme, view, toggles, terminal, openers) |
| **Theme Engine** | ✅ Complete | 4 themes with dynamic color system via CyberTheme enum |
| **Settings Panel** | ✅ Complete | Hex-themed UI: theme cards, LED toggles, collapsible shortcut ref |
| **Resource Monitor** | ✅ Complete | CPU (per-core), RAM, swap, disk usage, sparklines, threat level |
| **Visual Effects** | ✅ Complete | Scanlines (F11), CRT vignette (F12), data rain (F10), glitch transitions, HUD brackets |
| **Terminal Integration** | ✅ Complete | Auto-detect 8 terminals, "JACK IN" context menu, manual config |
| **External Tools** | ✅ Complete | "ROUTE TO..." dialog, quick apps, protocol bindings, custom openers |
| **Music Widget** | ✅ Complete | MPRIS/playerctl integration, playback controls in sidebar |
| **Preview Panel** | ✅ Complete | Text preview with metadata, togglable right panel (Ctrl+P) |
| **fzf Integration** | ✅ Complete | Fuzzy file search via external fzf (5-level depth, Ctrl+F) |
| **App Icon** | ✅ Complete | SVG + PNG (256/128/64/48px) hexagonal cyber-folder design |

### Partially Implemented

| Feature | Status | What Works | What's Missing |
|---------|--------|------------|----------------|
| **Selection** | ⚠️ Partial | Single-select, `multi_selected` HashSet exists | No Ctrl+Click, Shift+Click, rubber band, Ctrl+A |
| **Search** | ⚠️ Partial | fzf-based fuzzy find | No live filtering, regex, per-column search |
| **Preview** | ⚠️ Partial | Text content + metadata display | No image thumbnails, media preview, syntax highlighting |
| **Properties** | ⚠️ Partial | Sidebar metadata (size, modified, perms in octal) | No dedicated dialog, no owner name, no extended attrs |
| **Window State** | ⚠️ Partial | Sidebar/panel width persisted | No window position/size persistence, no tab state save |

### Not Implemented (Dolphin Parity Gaps)

| Feature | Priority | Dolphin Equivalent |
|---------|----------|--------------------|
| **Multi-select** | P0 | Ctrl+Click, Shift+Click, rubber band, Select All |
| **Create file** | P0 | Right-click → New → File |
| **Confirm delete dialog** | P0 | Modal "Are you sure?" (setting exists but unenforced) |
| **Drag and drop** | P0 | DnD move/copy between panels, to/from desktop |
| **Real-time filter** | P1 | Type-ahead filter bar that hides non-matching entries |
| **Image thumbnails** | P1 | Grid/icon view shows image previews |
| **Properties dialog** | P1 | Dedicated window: permissions editor, owner, timestamps, xattr |
| **Trash management** | P1 | View trashed items, restore, empty trash, original path display |
| **Split/dual pane** | P1 | F3 split view for side-by-side browsing |
| **Undo/Redo** | P1 | Ctrl+Z to undo file moves/deletes/renames |
| **System clipboard** | P1 | Sync internal clipboard with OS (xclip/wl-copy) |
| **Symlink creation** | P2 | Create symlinks via menu or Ctrl+Shift+drag |
| **Permissions editing** | P2 | chmod GUI with checkboxes, owner/group change |
| **Archive handling** | P2 | Browse into ZIP/TAR, extract, compress selection |
| **Embedded terminal** | P2 | Terminal panel within the file manager window |
| **Network/remote FS** | P3 | SFTP, SMB, FTP as "remote nodes" |
| **Bookmarks/state persist** | P2 | Save open tabs + bookmark order across sessions |
| **Tab reorder** | P2 | Drag tabs to reorder |
| **Batch rename** | P2 | Rename multiple files with pattern |
| **Custom sort** | P2 | Reverse sort, natural number sort, folders-mixed mode |

### Available Themes

| Theme | Primary | Accent | Inspiration |
|-------|---------|--------|-------------|
| **NIGHT CITY** | Cyan `#00F0FF` | Magenta `#FF2079` | Cyberpunk 2077 Arasaka |
| **SECTION 9** | Teal `#00D4AA` | Violet `#9B59B6` | Ghost in the Shell |
| **MAGI SYSTEM** | Orange `#FF6B00` | Crimson `#DC143C` | Evangelion NERV |
| **GIBSON** | Amber `#FFB000` | Green `#00FF41` | Classic hacker terminal |

### Keyboard Shortcuts (Current)

| Key | Action |
|-----|--------|
| F1 | Settings panel |
| F2 | Rename selected |
| F3 | Resource monitor toggle |
| F5 | Refresh directory |
| F9 | Toggle HUD overlay |
| F10 | Toggle data rain |
| F11 | Scanlines toggle |
| F12 | CRT effect toggle |
| Ctrl+1/2/3/4 | List / Grid / HIVE / Hex view |
| Ctrl+T | New tab |
| Ctrl+W | Close tab |
| Ctrl+B | Sidebar toggle |
| Ctrl+P | Preview panel toggle |
| Ctrl+H | Hidden files toggle |
| Ctrl+F | fzf search |
| Ctrl+C/X/V | Copy / Cut / Paste |
| Ctrl+Shift+N | New folder |
| Backspace | Navigate up |
| Delete | Quarantine (trash) |
| Arrow keys | Selection navigation |
| Enter | Open file/folder |
| Escape | Close overlays |

### Architecture (Actual)

```
cyberfile/
├── Cargo.toml                   # eframe 0.31, sysinfo 0.32, serde, toml, chrono, etc.
├── assets/
│   ├── icon.svg                 # Hexagonal cyber-folder icon (512×512)
│   ├── icon-256.png
│   ├── icon-128.png
│   ├── icon-64.png
│   └── icon-48.png
├── src/
│   ├── main.rs                  # Entry point, 1280×800 window
│   ├── app.rs                   # CyberFile struct, state machine, eframe::App impl (~1350 LOC)
│   ├── config.rs                # Settings with TOML persistence
│   ├── filesystem.rs            # FileEntry, read/sort/CRUD operations
│   ├── theme.rs                 # CyberTheme engine, 4 themes, apply_cyber_theme
│   ├── integrations/
│   │   ├── mod.rs
│   │   ├── fzf.rs               # fzf fuzzy search integration
│   │   └── media.rs             # MPRIS/playerctl music detection
│   └── ui/
│       ├── mod.rs
│       ├── boot_screen.rs       # POST-style boot animation
│       ├── command_bar.rs       # Top nav bar with path input + view toggles
│       ├── data_rain.rs         # Matrix-style falling character effect
│       ├── effects.rs           # Scanlines, CRT vignette, glitch, HUD brackets
│       ├── file_view.rs         # List view: breadcrumbs + sortable column listing
│       ├── grid_view.rs         # Grid view: thumbnail card layout
│       ├── hex_viewer.rs        # Hex dump view for binary files
│       ├── hud_overlay.rs       # Fullscreen HUD overlay (F9)
│       ├── music_widget.rs      # MPRIS music player controls
│       ├── preview_panel.rs     # Right-side file preview panel
│       ├── resource_monitor.rs  # CPU/RAM/disk vital signs panel
│       ├── settings_panel.rs    # Configuration manifest window (hex-themed)
│       ├── sidebar.rs           # Quick access + Neural Links + disk stats
│       ├── status_bar.rs        # Bottom info bar with clock
│       └── tabs.rs              # Multi-tab management
└── themes/
    └── default.toml
```

---

## 5. Dolphin Feature Parity — Gap Analysis

> Comprehensive comparison against KDE Dolphin as the reference desktop file manager.
> Features are scored: ✅ Implemented | ⚠️ Partial | ❌ Missing

### 5.1 Feature Matrix

| # | Feature Area | Dolphin | Cyberfile | Gap |
|---|-------------|---------|-----------|-----|
| **FILE OPERATIONS** | | | | |
| 1 | Copy / Cut / Paste | ✅ | ✅ | — |
| 2 | Delete to Trash | ✅ | ✅ | — |
| 3 | Rename (F2) | ✅ | ✅ | — |
| 4 | Create Folder | ✅ | ✅ | — |
| 5 | Create File | ✅ | ❌ | No "New File" option |
| 6 | Create Symlink | ✅ | ❌ | — |
| 7 | Undo / Redo | ✅ | ❌ | No operation history stack |
| 8 | Batch Rename | ✅ | ❌ | — |
| **NAVIGATION** | | | | |
| 9 | Back / Forward | ✅ | ✅ | — |
| 10 | Go Up (parent) | ✅ | ✅ | — |
| 11 | Breadcrumb path bar | ✅ | ✅ | — |
| 12 | Editable path bar | ✅ | ✅ | — |
| 13 | Tabs | ✅ | ✅ | — |
| 14 | Tab reorder (drag) | ✅ | ❌ | Tabs not draggable |
| 15 | Split view (F3) | ✅ | ❌ | — |
| 16 | Bookmarks sidebar | ✅ | ✅ | "Neural Links" |
| **SELECTION** | | | | |
| 17 | Click to select | ✅ | ✅ | — |
| 18 | Ctrl+Click multi | ✅ | ❌ | HashSet infra exists, no input handling |
| 19 | Shift+Click range | ✅ | ❌ | — |
| 20 | Ctrl+A select all | ✅ | ❌ | — |
| 21 | Rubber band | ✅ | ❌ | — |
| **SEARCH & FILTER** | | | | |
| 22 | Search bar | ✅ | ✅ | via fzf |
| 23 | Live filter | ✅ | ❌ | No type-ahead filtering |
| 24 | Regex search | ✅ | ❌ | — |
| 25 | Content search | ✅ | ❌ | — |
| **VIEW MODES** | | | | |
| 26 | Icon/Grid view | ✅ | ✅ | — |
| 27 | List/Detail view | ✅ | ✅ | — |
| 28 | Compact view | ✅ | ✅ | HIVE mode |
| 29 | Column sorting | ✅ | ✅ | Name/Size/Date/Perms |
| **PREVIEW** | | | | |
| 30 | Image thumbnails | ✅ | ❌ | Grid shows glyph icons only |
| 31 | Text/code preview | ✅ | ⚠️ | Basic text, no syntax highlighting |
| 32 | Media preview | ✅ | ❌ | — |
| 33 | Hover tooltip info | ✅ | ⚠️ | Status bar shows info, no hover tooltip |
| **PROPERTIES** | | | | |
| 34 | Properties dialog | ✅ | ❌ | Metadata in sidebar only |
| 35 | Permissions editor | ✅ | ❌ | View-only octal display |
| 36 | Owner/group display | ✅ | ❌ | — |
| **DRAG & DROP** | | | | |
| 37 | DnD within app | ✅ | ❌ | — |
| 38 | DnD to/from desktop | ✅ | ❌ | — |
| **TRASH** | | | | |
| 39 | Delete to trash | ✅ | ✅ | — |
| 40 | View trash contents | ✅ | ❌ | — |
| 41 | Restore from trash | ✅ | ❌ | — |
| 42 | Empty trash | ✅ | ❌ | — |
| 43 | Confirm delete dialog | ✅ | ❌ | Setting exists, not enforced |
| **CLIPBOARD** | | | | |
| 44 | Internal copy/paste | ✅ | ✅ | — |
| 45 | System clipboard sync | ✅ | ❌ | No xclip/wl-copy integration |
| **ARCHIVES** | | | | |
| 46 | Browse into archives | ✅ | ❌ | — |
| 47 | Extract/compress | ✅ | ❌ | — |
| **NETWORK** | | | | |
| 48 | SFTP / SSH | ✅ | ❌ | — |
| 49 | SMB / NFS | ✅ | ❌ | — |
| **STATE PERSISTENCE** | | | | |
| 50 | Window size/position | ✅ | ❌ | — |
| 51 | Open tabs on restart | ✅ | ❌ | — |
| 52 | Last directory | ✅ | ❌ | Always opens at $HOME |

### 5.2 Cyberfile Exclusive Features (Not in Dolphin)

| Feature | Description |
|---------|-------------|
| **4 Cyberpunk Themes** | Night City, Section 9, MAGI System, Gibson — full UI recoloring |
| **HIVE View Mode** | Hexagonal grid file layout with hex context menu |
| **Hex Dump Viewer** | Built-in binary hex viewer (Ctrl+4) |
| **Visual Effects** | Scanlines, CRT vignette, data rain, glitch transitions, HUD overlay |
| **Boot Sequence** | Themed POST-style startup animation |
| **Resource Monitor** | Real-time CPU/RAM/disk with sparklines and threat assessment |
| **Music Widget** | MPRIS sidebar music controls (Spotify, VLC, Firefox, etc.) |
| **fzf Integration** | External fuzzy finder integration |
| **Cyberpunk Naming** | Quarantine, Neural Links, Jack In, HIVE Protocol, etc. |

### 5.3 Summary Scorecard

| Category | Implemented | Partial | Missing | Parity % |
|----------|------------|---------|---------|----------|
| File Operations | 4 | 0 | 4 | 50% |
| Navigation | 5 | 0 | 3 | 63% |
| Selection | 1 | 0 | 4 | 20% |
| Search & Filter | 1 | 0 | 3 | 25% |
| View Modes | 4 | 0 | 0 | 100% |
| Preview | 0 | 2 | 2 | 25% |
| Properties | 0 | 0 | 3 | 0% |
| Drag & Drop | 0 | 0 | 2 | 0% |
| Trash | 1 | 0 | 4 | 20% |
| Clipboard | 1 | 0 | 1 | 50% |
| Archives | 0 | 0 | 2 | 0% |
| Network | 0 | 0 | 2 | 0% |
| State Persistence | 0 | 0 | 3 | 0% |
| **TOTAL** | **17** | **2** | **33** | **35%** |

---

## 6. Prioritized Development Roadmap

> Priority levels: **P0** = Blocks daily use | **P1** = Important for parity | **P2** = Nice-to-have | **P3** = Post-1.0

### Phase A — Daily Driver (P0) — "OPERATIONAL STATUS"

**Goal:** Make cyberfile usable as a primary file manager for daily tasks.

- [ ] **Multi-select: Ctrl+Click** — Toggle individual items in `multi_selected` HashSet
- [ ] **Multi-select: Shift+Click** — Range select from last-clicked to current
- [ ] **Multi-select: Ctrl+A** — Select all visible entries
- [ ] **Multi-select visual** — Highlight all selected items across all view modes
- [ ] **Confirm delete dialog** — Modal "PURGE PROTOCOL" confirmation when `confirm_delete` is true
- [ ] **Create file** — Right-click → "Initialize construct" / Ctrl+N for new empty file
- [ ] **Drag and drop (internal)** — Move/copy files between sidebar locations and main view
- [ ] **Real-time filter bar** — Type-ahead that hides non-matching entries instantly (no fzf spawn)
- [ ] **Image thumbnails** — Load + cache image previews for Grid/HIVE view (JPEG, PNG, SVG, WebP)
- [ ] **System clipboard sync** — Bridge internal clipboard to xclip / wl-copy for cross-app paste

### Phase B — Power User (P1) — "NEURAL UPGRADE"

**Goal:** Feature parity with Dolphin's most-used power features.

- [ ] **Split/dual pane view** — F3 to split main area, independent navigation per pane ("Dual Jack")
- [ ] **Undo/Redo stack** — Track copy/move/rename/delete operations, Ctrl+Z to reverse
- [ ] **Properties dialog** — Dedicated window: size, timestamps, permissions checkboxes, owner/group, xattr
- [ ] **Permissions editor** — Visual chmod with read/write/execute checkboxes for user/group/other
- [ ] **Trash management view** — Browse `~/.local/share/Trash/`, show original path, restore, empty all
- [ ] **Syntax-highlighted preview** — Basic highlighting for common languages (Rust, Python, JS, TOML, etc.)
- [ ] **Hover tooltips** — File info tooltip on hover (size, type, modified date)
- [ ] **Rubber band selection** — Click+drag rectangle to select multiple items in Grid/HIVE view
- [ ] **Window state persistence** — Save/restore window size, position, last directory per tab

### Phase C — Extended Functionality (P2) — "AUGMENTATION"

**Goal:** Quality-of-life features that round out the experience.

- [ ] **Archive handling** — Browse into ZIP/TAR/GZ (read-only initially), extract, compress selection
- [ ] **Symlink creation** — Context menu "Create neural link" for symlinks
- [ ] **Batch rename** — Multi-file rename with pattern substitution
- [ ] **Tab reorder** — Drag tabs to rearrange
- [ ] **Tab/bookmark persistence** — Save open tabs and bookmark order across sessions
- [ ] **Content search** — grep/ripgrep integration for searching inside files
- [ ] **Custom sort options** — Reverse sort, natural number sort, mix files and folders
- [ ] **Embedded terminal panel** — Terminal emulator panel within the window ("Neural Jack Port")
- [ ] **Sound effects** — UI audio feedback: nav clicks, errors, delete, copy complete
- [ ] **Owner/group name display** — Resolve UID/GID to names in metadata

### Phase D — Remote & Advanced (P3) — "NET RUNNER"

**Goal:** Network filesystems and plugin architecture.

- [ ] **SFTP/SSH browsing** — Connect to remote "nodes" via SSH, browse as local
- [ ] **SMB/NFS mounting** — Browse Windows shares and NFS mounts
- [ ] **FTP support** — Basic FTP/FTPS file browsing
- [ ] **Plugin system** — Lua/WASM scripting for custom actions and context menu extensions
- [ ] **D-Bus integration** — Desktop environment file manager protocol support
- [ ] **Neon glow / bloom** — Post-processing shader effect for UI elements
- [ ] **Chromatic aberration** — Subtle color fringing shader effect
- [ ] **Holographic noise** — Animated noise texture overlay

### Phase E — Release (P3) — "DEPLOY"

**Goal:** Distribution and documentation.

- [ ] Performance profiling and optimization
- [ ] Accessibility review (high contrast mode, reduced motion)
- [ ] Linux packaging: .deb, .rpm, AppImage, Flatpak
- [ ] .desktop file with icon for app launcher
- [ ] README with screenshots and feature showcase
- [ ] Public release

---

## 7. Technical Considerations

### 7.1 Performance Targets

| Metric | Target |
|--------|--------|
| Startup (cold) | < 1.5s (boot animation masks load) |
| Directory load (1000 files) | < 100ms |
| File operation feedback | < 16ms (60fps) |
| Memory baseline | < 80MB |
| Thumbnail cache | < 200MB on disk |
| GPU usage (idle, effects on) | < 5% |
| GPU usage (idle, effects off) | < 1% |

### 7.2 Compatibility

- **Display Servers:** X11 and Wayland (via winit)
- **Desktop Environments:** GNOME, KDE, Sway, Hyprland, i3
- **File Systems:** ext4, btrfs, XFS, NTFS (read), FAT32
- **Minimum Resolution:** 1280x720
- **GPU:** Any Vulkan-capable GPU (fallback to software rendering)

### 7.3 Security

- No elevated privileges by default — operate as user
- Sanitize all file paths to prevent traversal attacks
- Sandboxed file preview (no execution of previewed files)
- Respect filesystem permissions strictly
- Optional: integration with SELinux/AppArmor context display

### 7.4 Implementation Notes for Key Features

**Multi-select:** The `multi_selected: HashSet<usize>` field already exists in `CyberFile`. Needs:
- Input handling in `file_view.rs`, `grid_view.rs`, and hex grid view
- Modifier key detection (`ui.input(|i| i.modifiers.ctrl)`)
- `last_clicked_index` field for Shift+Click range calculation
- Update `perform_paste`, `perform_delete`, context menu to operate on `multi_selected`

**Image Thumbnails:** Use the `image` crate to load + resize. Cache as `HashMap<PathBuf, TextureHandle>` with LRU eviction. Generate thumbnails async to avoid blocking UI. Consider XDG thumbnail spec (`~/.cache/thumbnails/`).

**Drag and Drop:** egui supports `egui::DragAndDrop` for internal DnD. For desktop interop, need platform-specific integration via winit drag/drop events. Start with internal-only.

**Undo/Redo:** Maintain `Vec<FileOperation>` stack where each entry records: operation type, source path(s), destination path(s), timestamp. Reverse operations: move→move back, copy→delete copy, rename→rename back, delete→restore from trash.

**System Clipboard:** Shell out to `xclip -selection clipboard` (X11) or `wl-copy`/`wl-paste` (Wayland). Write file URIs as `file:///path/to/file` (freedesktop standard). Detect display server via `$WAYLAND_DISPLAY` env var.

**Split View:** Add `panes: Vec<PaneState>` where each pane has its own `current_dir`, `entries`, `selected`, `view_mode`. Render with `egui::SidePanel` or manual column layout. F3 toggles between 1 and 2 panes.

---

## 8. Inspiration Reference Map

| Element | CP2077 | GitS | Evangelion | Hackers |
|---------|--------|------|------------|---------|
| Color palette | ██ Primary | ██ Alt theme | ██ Alt theme | ██ Alt theme |
| UI chrome/borders | ★★★ | ★★ | ★★ | ★ |
| Typography | ★★ | ★ | ★★★ | ★ |
| Glitch effects | ★★★ | ★★ | ★ | ★★ |
| System readouts | ★★ | ★ | ★★★ | ★ |
| Particle effects | ★ | ★★★ | ★ | ★★ |
| Sound design | ★★ | ★★ | ★★★ | ★ |
| Terminal aesthetic | ★ | ★★ | ★★ | ★★★ |
| Japanese text | ★ | ★★★ | ★★★ | — |

★★★ = Heavy influence | ★★ = Moderate | ★ = Light | — = None

---

## 9. Stretch Goals (Post v1.0)

- **AR overlay mode** — transparent window overlay on desktop
- **AI file assistant** — natural language file search ("find the PDF I downloaded last Tuesday")
- **Customizable boot messages** — user-defined POST sequence text
- **Live wallpaper mode** — data rain as desktop background
- **Mobile companion** — phone app that shows file transfer status

---

*// END OF LINE*
