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

```text
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

```text
[SYSTEM] CYBERFILE v1.1.2
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

```text
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

```text
[ /home/user/Documents ]  ◈ 47 constructs  |  ◈ 2.3 GB sector  |  ◈ 128 GB free  |  DISK: ████░░ 67%  |  12:47:33
```

---

## Current Implementation Status

> **Last Updated:** Phase E complete — 96% Dolphin parity (50/0/2)

### Implemented & Working

| Module | Status | Details |
|--------|--------|---------|
| **Core File Browser** | ✅ Complete | Directory listing, navigation, breadcrumbs, sorting (name/size/date/perms) |
| **File Operations** | ✅ Complete | Copy, cut, paste, delete-to-trash, rename, create folder, undo/redo |
| **4 View Modes** | ✅ Complete | List (Ctrl+1), Grid (Ctrl+2), HIVE/HexGrid (Ctrl+3), Hex Viewer (Ctrl+4) |
| **Command Bar** | ✅ Complete | Path navigation, fzf search, neural interface input |
| **Tab System** | ✅ Complete | Unlimited tabs, per-tab path/selection, Ctrl+T/W, close button, drag-to-reorder |
| **Sidebar** | ✅ Complete | Quick access (8 dirs), bookmarks ("Neural Links"), disk stats, music widget |
| **Status Bar** | ✅ Complete | Path, count, size, view mode, fzf status, clock, selection info |
| **Boot Sequence** | ✅ Complete | 11-line POST animation with progress bar, skippable |
| **Keyboard Shortcuts** | ✅ Complete | 35+ shortcuts: F1-F12, Ctrl combos, arrow nav, Backspace/Enter |
| **Context Menu** | ✅ Complete | Themed menu + HIVE-mode hex variant with ⬡ bullets and hex border |
| **Config System** | ✅ Complete | TOML persistence via XDG config dir (theme, view, toggles, terminal, openers, sound) |
| **Theme Engine** | ✅ Complete | 8 themes with dynamic color system via CyberTheme enum |
| **Settings Panel** | ✅ Complete | Hex-themed UI: theme cards, LED toggles, sound toggle, collapsible shortcut ref |
| **Resource Monitor** | ✅ Complete | CPU (per-core), RAM, swap, disk usage, sparklines, threat level |
| **Visual Effects** | ✅ Complete | Scanlines (F11), CRT vignette (F12), data rain (F10), neon glow (F8), chromatic aberration (F6), holographic noise, glitch transitions, HUD brackets, high contrast mode |
| **Terminal Integration** | ✅ Complete | Auto-detect 8 terminals, "JACK IN" context menu, manual config |
| **External Tools** | ✅ Complete | "ROUTE TO..." dialog, quick apps, protocol bindings, custom openers |
| **Music Widget** | ✅ Complete | MPRIS/playerctl integration, playback controls in sidebar |
| **Preview Panel** | ✅ Complete | Text preview with metadata, togglable right panel (Ctrl+P) |
| **fzf Integration** | ✅ Complete | Fuzzy file search via external fzf (5-level depth, Ctrl+F) |
| **App Icon** | ✅ Complete | SVG + PNG (256/128/64/48px) hexagonal cyber-folder design |
| **Undo/Redo** | ✅ Complete | Ctrl+Z/Ctrl+Shift+Z for rename, delete, copy, move, create operations |
| **Split Pane** | ✅ Complete | F4 dual pane with independent navigation, DnD drop zones |
| **Rubber Band Selection** | ✅ Complete | Click+drag rectangle select in Grid view |
| **Drag and Drop** | ✅ Complete | Internal DnD: drag from Grid to split pane directories |
| **Archive Compress** | ✅ Complete | COMPRESS → ARCHIVE context menu, ZIP with deflate, recursive |
| **Embedded Terminal** | ✅ Complete | Neural Jack Port panel (F7), command runner with output display |
| **Sound Effects** | ✅ Complete | rodio sine wave synthesis, 5 sound types, togglable in settings |
| **SFTP/SSH Remote** | ✅ Complete | SSH key + password auth, remote file browser dialog (F9), download, sidebar status |
| **D-Bus Integration** | ✅ Complete | CLI path args, --show-item, reveal_in_file_manager |
| **Accessibility** | ✅ Complete | Reduced motion mode (disables animations), high contrast mode (boosted borders + overlay) |
| **.desktop + Install** | ✅ Complete | cyberfile.desktop, install.sh with PREFIX, icon installation |
| **README** | ✅ Complete | Full feature docs, keyboard shortcuts, install instructions |

### Partially Implemented

| Feature | Status | What Works | What's Missing |
|---------|--------|------------|----------------|
| **Selection** | ✅ Complete | Ctrl+Click toggle, Shift+Click range, Ctrl+A select all, rubber band select in Grid view | — |
| **Search** | ✅ Complete | fzf-based fuzzy find + real-time type-ahead filter bar + DEEP SCAN content search (grep/rg) | — |
| **Preview** | ✅ Complete | Text with syntax highlighting, metadata, image thumbnails in Grid view, ZIP archive contents listing | No media preview |
| **Properties** | ✅ Complete | CONSTRUCT PROFILE dialog with all metadata, visual chmod editor, resolved owner/group names | No extended attrs |
| **Window State** | ✅ Complete | Window size + last directory + tabs + bookmarks persisted | — |

### Not Implemented (Dolphin Parity Gaps)

| Feature | Priority | Dolphin Equivalent |
|---------|----------|--------------------|
| ~~**Drag and drop**~~ | ~~P0~~ | ~~Done: Internal DnD from Grid to split pane directories~~ |
| ~~**Properties dialog**~~ | ~~P1~~ | ~~Done: CONSTRUCT PROFILE dialog~~ |
| ~~**Trash management**~~ | ~~P1~~ | ~~Done: CONTAINMENT ZONE dialog~~ |
| ~~**Split/dual pane**~~ | ~~P1~~ | ~~Done: F4 split view with independent navigation~~ |
| ~~**Undo/Redo**~~ | ~~P1~~ | ~~Done: Ctrl+Z / Ctrl+Shift+Z for all operations~~ |
| ~~**Symlink creation**~~ | ~~P2~~ | ~~Done: NEURAL LINK dialog~~ |
| ~~**Permissions editing**~~ | ~~P2~~ | ~~Done: Visual chmod in Properties dialog~~ |
| ~~**Archive handling**~~ | ~~P2~~ | ~~Done: ZIP contents listing + extract + compress~~ |
| ~~**Embedded terminal**~~ | ~~P2~~ | ~~Done: Neural Jack Port panel (F7)~~ |
| **Network/remote FS** | P3 | SFTP, SMB, FTP as "remote nodes" — SFTP done, SMB/FTP remaining |
| ~~**Bookmarks/state persist**~~ | ~~P2~~ | ~~Done: Tabs + bookmarks saved across sessions~~ |
| ~~**Tab reorder**~~ | ~~P2~~ | ~~Done: Drag tabs to reorder~~ |
| ~~**Batch rename**~~ | ~~P2~~ | ~~Done: MASS REASSIGN dialog with find/replace/regex~~ |
| ~~**Custom sort**~~ | ~~P2~~ | ~~Done: Natural number sort, Extension column sort~~ |
| **DnD to/from desktop** | P3 | Platform-level drag-drop via winit |
| **Media preview** | P3 | Audio/video preview in panel |
| ~~**Sound effects**~~ | ~~P2~~ | ~~Done: rodio sine wave UI sounds (togglable)~~ |
| ~~**Rubber band selection**~~ | ~~P1~~ | ~~Done: Click+drag rectangle select in Grid view~~ |

### Available Themes

| Theme | Primary | Accent | Inspiration |
|-------|---------|--------|-------------|
| **NIGHT CITY** | Cyan `#00F0FF` | Magenta `#FF2079` | Cyberpunk 2077 Arasaka |
| **SECTION 9** | Teal `#00D4AA` | Violet `#9B59B6` | Ghost in the Shell |
| **MAGI SYSTEM** | Orange `#FF6B00` | Crimson `#DC143C` | Evangelion NERV |
| **GIBSON** | Amber `#FFB000` | Green `#00FF41` | Classic hacker terminal |
| **TYRELL** | Steel blue `#4A90D9` | Gold `#D4A520` | Blade Runner |
| **AKIRA** | Capsule red `#FF1744` | Silver `#E0E0F0` | Akira / Neo-Tokyo |
| **WINTERMUTE** | Ice blue `#88CCFF` | Chrome `#C0C8D8` | Neuromancer AI |
| **OUTRUN** | Hot pink `#FF6EC7` | Electric blue `#00BFFF` | Synthwave / retrowave |

### Keyboard Shortcuts (Current)

| Key | Action |
|-----|--------|
| F1 | Settings panel |
| F2 | Rename selected |
| F3 | Resource monitor toggle |
| F4 | Split pane toggle |
| F5 | Refresh directory |
| F6 | Chromatic aberration toggle |
| F7 | Embedded terminal toggle |
| F8 | Neon glow toggle |
| F9 | SFTP remote dialog |
| F10 | Toggle data rain |
| F11 | Scanlines toggle |
| F12 | CRT effect toggle |
| Ctrl+1/2/3/4 | List / Grid / HIVE / Hex view |
| Ctrl+T | New tab |
| Ctrl+W | Close tab |
| Ctrl+B | Sidebar toggle |
| Ctrl+P | Preview panel toggle |
| Ctrl+Shift+P | Process Matrix |
| Ctrl+D | Service Deck |
| Ctrl+J | Log Viewer |
| Ctrl+Shift+D | Signal Deck |
| Ctrl+H | Hidden files toggle |
| Ctrl+F | fzf search |
| Ctrl+C/X/V | Copy / Cut / Paste |
| Ctrl+N | New file |
| Ctrl+Shift+N | New folder |
| Ctrl+A | Select all |
| Ctrl+Z | Undo last operation |
| Ctrl+Shift+Z | Redo last undone operation |
| Ctrl+I | Properties dialog (CONSTRUCT PROFILE) |
| Ctrl+G | Content search (DEEP SCAN) |
| Ctrl+R | Batch rename (MASS REASSIGN, multi-select) |
| Backspace | Navigate up |
| Delete | Quarantine (trash) |
| Arrow keys | Selection navigation |
| Enter | Open file/folder |
| Escape | Close overlays / terminal |

### Architecture (Actual)

```text
cyberfile/
├── Cargo.toml                   # eframe 0.31, sysinfo 0.32, serde, toml, chrono, ssh2, etc.
├── README.md                    # Feature docs, install instructions, shortcuts
├── cyberfile.desktop             # Linux .desktop entry
├── install.sh                   # System install script
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
│   ├── theme.rs                 # CyberTheme engine, 8 themes, apply_cyber_theme
│   ├── integrations/
│   │   ├── mod.rs
│   │   ├── dbus.rs              # D-Bus integration, CLI path handling
│   │   ├── fzf.rs               # fzf fuzzy search integration
│   │   ├── media.rs             # MPRIS/playerctl music detection
│   │   └── sftp.rs              # SFTP/SSH remote file browsing
│   └── ui/
│       ├── mod.rs
│       ├── boot_screen.rs       # POST-style boot animation
│       ├── command_bar.rs       # Top nav bar with path input + view toggles
│       ├── data_rain.rs         # Matrix-style falling character effect
│       ├── effects.rs           # Scanlines, CRT vignette, glitch, neon glow, chromatic aberration, holographic noise, high contrast, HUD brackets
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
| 5 | Create File | ✅ | ✅ | Ctrl+N / context menu |
| 6 | Create Symlink | ✅ | ✅ | NEURAL LINK dialog |
| 7 | Undo / Redo | ✅ | ✅ | Ctrl+Z / Ctrl+Shift+Z — tracks rename, delete, copy, move, create |
| 8 | Batch Rename | ✅ | ✅ | MASS REASSIGN dialog (Ctrl+R) |
| **NAVIGATION** | | | | |
| 9 | Back / Forward | ✅ | ✅ | — |
| 10 | Go Up (parent) | ✅ | ✅ | — |
| 11 | Breadcrumb path bar | ✅ | ✅ | — |
| 12 | Editable path bar | ✅ | ✅ | — |
| 13 | Tabs | ✅ | ✅ | — |
| 14 | Tab reorder (drag) | ✅ | ✅ | Drag tabs to reorder |
| 15 | Split view (F4) | ✅ | ✅ | F4 split pane with independent navigation + DnD drop zones |
| 16 | Bookmarks sidebar | ✅ | ✅ | "Neural Links" |
| **SELECTION** | | | | |
| 17 | Click to select | ✅ | ✅ | — |
| 18 | Ctrl+Click multi | ✅ | ✅ | — |
| 19 | Shift+Click range | ✅ | ✅ | — |
| 20 | Ctrl+A select all | ✅ | ✅ | — |
| 21 | Rubber band | ✅ | ✅ | Click+drag rectangle select in Grid view |
| **SEARCH & FILTER** | | | | |
| 22 | Search bar | ✅ | ✅ | via fzf |
| 23 | Live filter | ✅ | ✅ | Real-time filter bar above all views |
| 24 | Regex search | ✅ | ✅ | Regex in batch rename + content search |
| 25 | Content search | ✅ | ✅ | DEEP SCAN dialog (Ctrl+G) via grep/rg |
| **VIEW MODES** | | | | |
| 26 | Icon/Grid view | ✅ | ✅ | — |
| 27 | List/Detail view | ✅ | ✅ | — |
| 28 | Compact view | ✅ | ✅ | HIVE mode |
| 29 | Column sorting | ✅ | ✅ | Name/Size/Date/Perms |
| **PREVIEW** | | | | |
| 30 | Image thumbnails | ✅ | ✅ | Lazy-loaded 96px thumbnails in Grid view |
| 31 | Text/code preview | ✅ | ✅ | Syntax-highlighted preview for 12+ languages |
| 32 | Media preview | ✅ | ❌ | — |
| 33 | Hover tooltip info | ✅ | ✅ | Hover tooltips in List/Grid/HIVE views |
| **PROPERTIES** | | | | |
| 34 | Properties dialog | ✅ | ✅ | CONSTRUCT PROFILE dialog (Ctrl+I) |
| 35 | Permissions editor | ✅ | ✅ | Visual chmod with R/W/X checkboxes |
| 36 | Owner/group display | ✅ | ✅ | Resolved names via uzers crate |
| **DRAG & DROP** | | | | |
| 37 | DnD within app | ✅ | ✅ | Drag files from Grid view to split pane directories |
| 38 | DnD to/from desktop | ✅ | ❌ | Platform DnD requires winit integration |
| **TRASH** | | | | |
| 39 | Delete to trash | ✅ | ✅ | — |
| 40 | View trash contents | ✅ | ✅ | CONTAINMENT ZONE dialog |
| 41 | Restore from trash | ✅ | ✅ | Restore button per item |
| 42 | Empty trash | ✅ | ✅ | PURGE ALL button |
| 43 | Confirm delete dialog | ✅ | ✅ | PURGE PROTOCOL modal |
| **CLIPBOARD** | | | | |
| 44 | Internal copy/paste | ✅ | ✅ | — |
| 45 | System clipboard sync | ✅ | ✅ | wl-copy / xclip bridge |
| **ARCHIVES** | | | | |
| 46 | Browse into archives | ✅ | ✅ | ZIP contents listing in preview panel |
| 47 | Extract/compress | ✅ | ✅ | ZIP extraction + compress via context menu (deflate) |
| **NETWORK** | | | | |
| 48 | SFTP / SSH | ✅ | ✅ | SFTP dialog (F9) with key + password auth |
| 49 | SMB / NFS | ✅ | ❌ | — |
| **STATE PERSISTENCE** | | | | |
| 50 | Window size/position | ✅ | ✅ | Auto-saves window dimensions |
| 51 | Open tabs on restart | ✅ | ✅ | Tabs + bookmarks persisted in config.toml |
| 52 | Last directory | ✅ | ✅ | Saved in config.toml |

### 5.2 Cyberfile Exclusive Features (Not in Dolphin)

| Feature | Description |
|---------|-------------|
| **8 Cyberpunk Themes** | Night City, Section 9, MAGI System, Gibson, Tyrell, Akira, Wintermute, Outrun — full UI recoloring |
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
| File Operations | 8 | 0 | 0 | 100% |
| Navigation | 8 | 0 | 0 | 100% |
| Selection | 5 | 0 | 0 | 100% |
| Search & Filter | 4 | 0 | 0 | 100% |
| View Modes | 4 | 0 | 0 | 100% |
| Preview | 3 | 0 | 1 | 75% |
| Properties | 3 | 0 | 0 | 100% |
| Drag & Drop | 1 | 0 | 1 | 50% |
| Trash | 5 | 0 | 0 | 100% |
| Clipboard | 2 | 0 | 0 | 100% |
| Archives | 2 | 0 | 0 | 100% |
| Network | 1 | 0 | 1 | 50% |
| State Persistence | 3 | 0 | 0 | 100% |
| **TOTAL** | **50** | **0** | **2** | **96%** |

---

## 6. Prioritized Development Roadmap

> Priority levels: **P0** = Blocks daily use | **P1** = Important for parity | **P2** = Nice-to-have | **P3** = Post-1.0

### Phase A — Daily Driver (P0) — "OPERATIONAL STATUS"

**Goal:** Make cyberfile usable as a primary file manager for daily tasks.

- [x] **Multi-select: Ctrl+Click** — Toggle individual items in `multi_selected` HashSet
- [x] **Multi-select: Shift+Click** — Range select from last-clicked to current
- [x] **Multi-select: Ctrl+A** — Select all visible entries
- [x] **Multi-select visual** — Highlight all selected items across all view modes
- [x] **Confirm delete dialog** — Modal "PURGE PROTOCOL" confirmation when `confirm_delete` is true
- [x] **Create file** — Right-click → "Initialize construct" / Ctrl+N for new empty file
- [x] **Drag and drop (internal)** — Drag files from Grid view to split pane directories
- [x] **Real-time filter bar** — Type-ahead that hides non-matching entries instantly (no fzf spawn)
- [x] **Image thumbnails** — Load + cache image previews for Grid/HIVE view (JPEG, PNG, SVG, WebP)
- [x] **System clipboard sync** — Bridge internal clipboard to xclip / wl-copy for cross-app paste

### Phase B — Power User (P1) — "NEURAL UPGRADE"

**Goal:** Feature parity with Dolphin's most-used power features.

- [x] **Split/dual pane view** — F4 to split main area, independent navigation per pane ("Dual Jack")
- [x] **Undo/Redo stack** — Track copy/move/rename/delete/create operations, Ctrl+Z / Ctrl+Shift+Z
- [x] **Properties dialog** — Dedicated window: size, timestamps, permissions checkboxes, owner/group, inode, device
- [x] **Permissions editor** — Visual chmod with read/write/execute checkboxes for user/group/other
- [x] **Trash management view** — Browse trash, restore items to current dir, empty all, sidebar section
- [x] **Syntax-highlighted preview** — Keyword highlighting for Rust, Python, JS/TS, C/C++, Go, Shell, Java, Ruby, HTML, CSS, TOML/YAML
- [x] **Hover tooltips** — File info tooltip on hover in List, Grid, and HIVE views
- [x] **Rubber band selection** — Click+drag rectangle to select multiple items in Grid view
- [x] **Window state persistence** — Save/restore window size, last directory

### Phase C — Extended Functionality (P2) — "AUGMENTATION"

**Goal:** Quality-of-life features that round out the experience.

- [x] **Archive handling** — ZIP browsing (contents listing + extract), TAR/GZ via file command
- [x] **Symlink creation** — Context menu "NEURAL LINK" + radial hex menu, "Create neural link" dialog
- [x] **Batch rename** — "MASS REASSIGN" dialog with find/replace + regex support, preview, Ctrl+R
- [x] **Tab reorder** — Drag tabs to rearrange
- [x] **Tab/bookmark persistence** — Tabs + bookmarks saved/restored across sessions via config.toml
- [x] **Content search** — "DEEP SCAN" dialog with grep/ripgrep integration, Ctrl+G, clickable results
- [x] **Custom sort options** — Natural number sort (file2 < file10), Extension column sort
- [x] **Embedded terminal panel** — Terminal panel within window ("Neural Jack Port", F7)
- [x] **Sound effects** — UI audio via rodio: nav clicks, errors, delete, copy complete (togglable)
- [x] **Owner/group name display** — Resolve UID/GID to names via uzers crate in preview + properties

### Phase D — Remote & Advanced (P3) — "NET RUNNER"

**Goal:** Network filesystems and plugin architecture.

- [x] **SFTP/SSH browsing** — Connect to remote "nodes" via SSH (key + password auth), browse/download files (F9)
- [ ] **SMB/NFS mounting** — Browse Windows shares and NFS mounts
- [ ] **FTP support** — Basic FTP/FTPS file browsing
- [ ] **Plugin system** — Lua/WASM scripting for custom actions and context menu extensions
- [x] **D-Bus integration** — CLI path arguments, --show-item support, reveal_in_file_manager
- [x] **Neon glow / bloom** — Edge glow effect in theme colors (F8)
- [x] **Chromatic aberration** — Animated color-shifted scan bars (F6)
- [x] **Holographic noise** — Sparse animated noise grid overlay

### Phase E — Release (P3) — "DEPLOY"

**Goal:** Distribution and documentation.

- [x] Performance profiling and optimization (zero warnings, dead code cleanup)
- [x] Accessibility review (high contrast mode, reduced motion)
- [x] Linux packaging: install.sh with PREFIX support
- [x] .desktop file with icon for app launcher
- [x] README with feature showcase and keyboard shortcuts
- [ ] Public release

### Parallel Track — Operator Console Expansion — "TACTICAL CONSOLE"

**Goal:** Evolve cyberfile from a themed file manager into a focused in-DE control deck without turning it into a full desktop environment.

#### Product Doctrine

- **Cyberfile remains an operator console, not a shell replacement.** It should orchestrate files, processes, services, devices, and sessions from one surface.
- **Files stay central.** Every non-file subsystem should attach back to directories, projects, remotes, logs, or launch contexts.
- **Integrations must degrade gracefully.** WM- and distro-specific bridges stay optional and auto-detected.
- **Everything should be keyboard-first and stateful.** The command layer, session restore, and saved actions matter more than decorative widgets.

#### Primary Subsystems

| Subsystem | In-universe Name | Purpose | Likely Integration Layer |
|-----------|------------------|---------|--------------------------|
| **FILES** | Data Sectors | Existing file manager core | `std::fs`, trash, preview, archive, SFTP |
| **PROTOCOLS** | Protocol Launcher | Unified command palette for actions, scripts, apps, remotes, and toggles | internal registry + shell commands + D-Bus |
| **SCENES** | Mission Scenes | Save and restore tabs, paths, terminal commands, remotes, and launch context | config persistence + app state snapshots |
| **PROC** | Process Matrix | Inspect, filter, kill, and relaunch running processes and project tasks | `/proc`, `sysinfo`, child-process tracking |
| **SERVICES** | Service Deck | Start/stop/restart user services and inspect logs | `systemd --user`, `journalctl` |
| **SIGNALS** | Signal Deck | Volume, media, mic, clipboard, notifications, power, brightness | D-Bus, `playerctl`, `wpctl`/`pactl`, `brightnessctl` |
| **NET** | Network Mesh | Wi-Fi, VPN, SSH bookmarks, remote mounts, transfer state | NetworkManager D-Bus / `nmcli`, SSH/SFTP |
| **DEVICES** | Device Bay | Mount/eject disks, removable media, phones, cameras | `udisks2`, GVfs, MTP tools |
| **WINDOWS** | Window Bridge | Optional focus/move/launch control for the surrounding compositor | `hyprctl`, `swaymsg`, `i3-msg`, `wmctrl` fallback |

#### Recommended Implementation Order

#### Stage 1 — Command Surface — "PROTOCOL LAYER"

**Outcome:** Cyberfile becomes the place you go to do things, not just browse files.

- [x] Replace the current command bar with a registry-backed launcher that mixes files, commands, apps, bookmarks, remotes, and system actions in one result list
- [x] Add named "protocols" backed by shell commands or built-in actions with arguments and icons
- [x] Add quick actions for common operator tasks: open terminal here, tail log here, start transfer, launch editor, open system apps, and connect remote
- [x] Add per-directory and per-project action presets so Cyberfile feels context-aware

#### Stage 1 Implementation Work Packages

| Work Package | Scope | Files / Modules |
|--------------|-------|-----------------|
| **1A. Surface mode split** | Separate path-navigation mode from protocol-launcher mode, preserving fast path entry | `src/app.rs`, `src/ui/command_bar.rs`, `src/launcher.rs` |
| **1B. Launcher registry** | Introduce a registry of built-in actions and scene restores, with typed actions instead of raw strings | `src/launcher.rs` |
| **1C. Result list UI** | Inline launcher panel under the command bar with keyboard selection and click-to-execute behavior | `src/ui/command_bar.rs` |
| **1D. Action execution bridge** | Map launcher actions to real app state changes: terminal, panels, deep scan, settings, scene capture | `src/app.rs` |
| **1E. Context presets** | Add directory/project scoped protocol manifests via global config and nearest `.cyberfile.toml` override | `src/launcher.rs`, `src/config.rs`, `config.toml`, `.cyberfile.toml` |

#### Current Scaffold Status

- [x] Two-mode command surface scaffolded: `PATH` and `PROTO`
- [x] Built-in launcher registry scaffolded with panel toggles, terminal, deep scan, settings, and scene actions
- [x] Inline protocol results panel scaffolded in the top bar
- [x] Global protocol manifest support via `config.toml`
- [x] Nearest local protocol manifest support via `.cyberfile.toml`
- [x] Scene Manager window scaffolded with capture, restore, pin, rename, and delete flows
- [x] External app launch catalog
- [x] Remote/node-specific protocol providers

#### Stage 2 — Stateful Workspace — "MISSION SCENES"

**Outcome:** Cyberfile can restore a working session as a command deck, not just a last-opened folder.

- [x] Save and restore scene snapshots: open tabs, split panes, focused directories, SFTP nodes, terminal history, filter state, theme, overlays
- [x] Support pinned scenes such as "Code Ops", "Media Intake", "Remote Maintenance", and "Archive Recovery"
- [x] Add one-key scene switching and scene launch from the boot screen / command layer
- [x] Persist recent scenes separately from generic app state

#### Mission Scene Data Model

| Field Group | Stored State | Purpose |
|-------------|--------------|---------|
| **Identity** | `id`, `name`, `summary`, `updated_at`, `tags`, `pinned` | Stable restore target + user-facing label |
| **Sector State** | `current_path`, `tabs[]`, `active_tab`, split pane path/selection | Rebuild the file-manager workspace |
| **Surface State** | `command_text`, `command_mode`, `filter_text`, `view_mode`, `theme_id` | Restore the operator's current control surface |
| **Overlay State** | sidebar, preview, resource monitor, terminal panel, settings panel, data rain | Recreate the current HUD layout |
| **Terminal State** | command input, recent command history, running command label, output tail | Preserve recent terminal context without trying to serialize live processes |
| **Remote State** | SFTP host/user/port/display/path plus connection intent | Restore the remote target and prompt for re-auth when needed |
| **Scene Store** | `saved_scenes`, `recent_scenes`, `session_scene` | Persist named decks, boot quick slots, and the last-session resume point in `scenes.toml` |

#### Scene Restore Rules

- Live process handles and authenticated SFTP sessions are **not** resumed automatically; scenes restore intent, not privileged live handles.
- Invalid paths degrade to the current home directory instead of failing the full scene restore.
- Terminal output is intentionally truncated to a recent tail to keep config persistence small and stable.
- Scene restore should re-apply theme, layout, and command-surface mode before optional future WM hooks fire.
- The boot deck exposes the last-session snapshot plus four quick scene slots backed by pinned and recent scenes.

#### Stage 3 — System Control — "PROCESS + SERVICE DECK"

**Outcome:** Cyberfile becomes a practical control panel for active work.

- [x] Add a live process matrix with search, sort, kill, cwd display, and child-process grouping
- [x] Track commands launched from the embedded terminal as first-class jobs with output history and restart controls
- [x] Add a Service Deck for `systemd --user` units with start/stop/restart/enable and status inspection
- [x] Add a log viewer with saved watch channels backed by `journalctl`

#### Stage 4 — Desktop Signals — "SIGNAL DECK"

**Outcome:** The app starts to feel like a mini DE inside the existing DE.

- [ ] Add audio route controls, mic mute, volume mixer, and current sink/source visibility
- [ ] Expand MPRIS into a generalized media bus: playback, player switch, progress, artwork cache
- [ ] Add notification history and clipboard history panels
- [ ] Add laptop-oriented controls where available: battery, brightness, power profile, idle inhibit

#### Stage 5 — Network + Devices — "FIELD OPS"

**Outcome:** Local, remote, and removable resources all feel part of one interface.

- [ ] Add Network Mesh status for interfaces, SSIDs, VPN tunnels, and transfer throughput
- [ ] Add SSH bookmark vault with connect, reconnect, and scene binding
- [ ] Add Device Bay for removable disks and media with mount/eject actions and health/status readouts
- [ ] Expand remote nodes beyond SFTP to SMB/NFS where practical

#### Stage 6 — Optional WM Hooks — "TACTICAL BRIDGE"

**Outcome:** Cyberfile can coordinate the surrounding desktop without owning it.

- [ ] Add optional window-manager bridges for Hyprland, Sway, and i3
- [ ] Surface launch/focus/move-to-workspace actions from the command layer
- [ ] Support scene actions that open/focus external apps and arrange them loosely through compositor hooks
- [ ] Keep all WM-specific features behind capability detection and settings flags

#### Suggested Module Additions

```text
src/
    launcher.rs        # protocol registry + action filtering
    scenes.rs          # mission scene serialization model
    integrations/
        audio.rs           # volume, mic, sinks/sources
        devices.rs         # udisks2 / removable media state
        journald.rs        # journalctl readers / log filters
        network.rs         # nmcli / D-Bus network state
        processes.rs       # process inventory + task control
        services.rs        # systemd --user integration
        windows.rs         # optional compositor / WM bridge
    ui/
        launcher.rs        # registry-driven command palette
        scene_manager.rs   # save/restore working scenes
        process_matrix.rs  # process browser + task controls
        service_deck.rs    # service list + logs
        signal_deck.rs     # media/audio/notifications/clipboard
        network_mesh.rs    # network + remote status
        device_bay.rs      # mountable devices + removable media
```

#### Non-Goals

- Replacing the Linux panel, launcher, or compositor
- Becoming a full terminal emulator or full IDE
- Re-implementing a distro settings center wholesale
- Depending on one compositor or desktop environment for core usability

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

// END OF LINE
