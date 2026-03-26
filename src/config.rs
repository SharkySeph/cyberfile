use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::integrations::journald::LogChannel;
use crate::scenes::MissionScene;

// ── Sidebar Layout ──────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SidebarWidget {
    QuickAccess,
    NeuralLinks,
    MissionScenes,
    SystemStatus,
    ContainmentZone,
    NetRunner,
    OperatorDeck,
    MusicWidget,
    NetworkMesh,
    DeviceBay,
    WindowBridge,
}

impl SidebarWidget {
    /// Full ordered list of every widget (used as the default layout).
    pub fn all() -> Vec<SidebarWidget> {
        vec![
            Self::QuickAccess,
            Self::NeuralLinks,
            Self::MissionScenes,
            Self::SystemStatus,
            Self::ContainmentZone,
            Self::NetRunner,
            Self::OperatorDeck,
            Self::MusicWidget,
            Self::NetworkMesh,
            Self::DeviceBay,
            Self::WindowBridge,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::QuickAccess => "QUICK ACCESS",
            Self::NeuralLinks => "NEURAL LINKS",
            Self::MissionScenes => "MISSION SCENES",
            Self::SystemStatus => "SYSTEM STATUS",
            Self::ContainmentZone => "CONTAINMENT ZONE",
            Self::NetRunner => "NET RUNNER",
            Self::OperatorDeck => "OPERATOR DECK",
            Self::MusicWidget => "MUSIC WIDGET",
            Self::NetworkMesh => "NETWORK MESH",
            Self::DeviceBay => "DEVICE BAY",
            Self::WindowBridge => "TACTICAL BRIDGE",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarEntry {
    pub widget: SidebarWidget,
    pub visible: bool,
}

pub fn default_sidebar_layout() -> Vec<SidebarEntry> {
    SidebarWidget::all()
        .into_iter()
        .map(|w| SidebarEntry { widget: w, visible: true })
        .collect()
}

/// Ensure every known widget appears exactly once (handles config from older versions).
pub fn normalize_sidebar_layout(layout: &mut Vec<SidebarEntry>) {
    let all = SidebarWidget::all();
    // Remove duplicates (keep first occurrence)
    let mut seen = std::collections::HashSet::new();
    layout.retain(|e| seen.insert(e.widget));
    // Append any missing widgets at the end (visible by default)
    for w in all {
        if !layout.iter().any(|e| e.widget == w) {
            layout.push(SidebarEntry { widget: w, visible: true });
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProtocolCommand {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub section: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub run_in_terminal: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LocalProtocolManifestMeta {
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LocalProtocolManifest {
    #[serde(default)]
    pub meta: LocalProtocolManifestMeta,
    #[serde(default)]
    pub protocols: Vec<ProtocolCommand>,
    #[serde(default)]
    #[allow(dead_code)]
    pub log_channels: Vec<LogChannel>,
}

/// Persisted operator configuration loaded from `config.toml`.
///
/// Field names are intentionally stable because they are used as serialized keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Stable theme identifier that maps to `CyberTheme::id()`.
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Default file view mode identifier used during startup.
    #[serde(default = "default_view")]
    pub default_view: String,
    /// Whether dotfiles and other hidden entries are visible in listings.
    #[serde(default)]
    pub show_hidden: bool,
    /// Whether delete operations require confirmation before quarantine.
    #[serde(default = "default_true")]
    pub confirm_delete: bool,
    /// Whether the startup boot animation should run on launch.
    #[serde(default = "default_true")]
    pub boot_sequence: bool,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: f32,
    /// Ordered list of sidebar widgets with per-widget visibility.
    #[serde(default = "default_sidebar_layout")]
    pub sidebar_layout: Vec<SidebarEntry>,
    /// Overlay subtle CRT scanlines across the viewport.
    #[serde(default)]
    pub scanlines_enabled: bool,
    /// Apply the CRT-style screen-edge vignette effect.
    #[serde(default)]
    pub crt_effect: bool,

    /// Preferred terminal emulator (e.g. "kitty", "alacritty", "wezterm").
    /// If empty, auto-detect from available system terminals.
    #[serde(default)]
    pub terminal_emulator: String,

    /// Custom "open with" application mappings by extension.
    /// e.g. { "rs" = "code", "png" = "gimp", "md" = "code" }
    #[serde(default)]
    pub custom_openers: BTreeMap<String, String>,

    // ── Window State Persistence ─────────────────────────
    #[serde(default = "default_window_width")]
    pub window_width: f32,
    #[serde(default = "default_window_height")]
    pub window_height: f32,
    #[serde(default)]
    pub last_directory: String,

    // ── Bookmark & Tab Persistence ───────────────────────
    #[serde(default)]
    pub bookmarks: Vec<String>,
    #[serde(default)]
    pub saved_tabs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub saved_scenes: Vec<MissionScene>,
    #[serde(default)]
    pub protocols: Vec<ProtocolCommand>,
    #[serde(default)]
    pub log_channels: Vec<LogChannel>,

    // ── Sound ────────────────────────────────────────────
    /// Enable synthesized UI feedback sounds.
    #[serde(default)]
    pub sound_enabled: bool,

    // ── Phase D: Visual Effects ──────────────────────────
    /// Render edge glow/bloom accents around the interface.
    #[serde(default)]
    pub neon_glow: bool,
    /// Render color-shift artifacts for the chromatic glitch effect.
    #[serde(default)]
    pub chromatic_aberration: bool,
    /// Overlay sparse animated holographic noise cells.
    #[serde(default)]
    pub holographic_noise: bool,

    // ── Phase E: Accessibility ───────────────────────────
    /// Disable higher-motion visual effects for accessibility.
    #[serde(default)]
    pub reduced_motion: bool,
    /// Strengthen readability-focused overlays and contrast cues.
    #[serde(default)]
    pub high_contrast: bool,
}

fn default_theme() -> String {
    "night_city".into()
}
fn default_view() -> String {
    "list".into()
}
fn default_true() -> bool {
    true
}
fn default_font_size() -> f32 {
    14.0
}
fn default_sidebar_width() -> f32 {
    220.0
}
fn default_window_width() -> f32 {
    1280.0
}
fn default_window_height() -> f32 {
    800.0
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            default_view: default_view(),
            show_hidden: false,
            confirm_delete: true,
            boot_sequence: true,
            font_size: default_font_size(),
            sidebar_width: default_sidebar_width(),
            sidebar_layout: default_sidebar_layout(),
            scanlines_enabled: false,
            crt_effect: false,
            terminal_emulator: String::new(),
            custom_openers: BTreeMap::new(),
            window_width: default_window_width(),
            window_height: default_window_height(),
            last_directory: String::new(),
            bookmarks: Vec::new(),
            saved_tabs: Vec::new(),
            saved_scenes: Vec::new(),
            protocols: Vec::new(),
            log_channels: Vec::new(),
            sound_enabled: false,
            neon_glow: false,
            chromatic_aberration: false,
            holographic_noise: false,
            reduced_motion: true,
            high_contrast: false,
        }
    }
}

impl Settings {
    /// Absolute path to the operator configuration manifest.
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cyberfile")
            .join("config.toml")
    }

    /// Load persisted settings or fall back to defaults when the manifest is missing or invalid.
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(mut settings) = toml::from_str::<Settings>(&content) {
                    normalize_sidebar_layout(&mut settings.sidebar_layout);
                    return settings;
                }
            }
        }
        Self::default()
    }

    /// Persist the current configuration back to disk.
    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(content) = toml::to_string_pretty(self) {
            let _ = std::fs::write(path, content);
        }
    }

    /// The list of known terminal emulators to try when auto-detecting.
    pub const KNOWN_TERMINALS: &[&str] = &[
        "kitty",
        "alacritty",
        "wezterm",
        "foot",
        "gnome-terminal",
        "konsole",
        "xfce4-terminal",
        "xterm",
    ];

    /// Resolve which terminal to show in the UI: configured value, or auto-detect.
    pub fn resolved_terminal(&self) -> Option<String> {
        let configured = self.terminal_emulator.trim();
        if !configured.is_empty() {
            return resolve_executable(configured).map(|_| configured.to_string());
        }
        for term in Self::KNOWN_TERMINALS {
            if resolve_executable(term).is_some() {
                return Some((*term).to_string());
            }
        }
        None
    }

    /// Resolve which terminal executable path to launch.
    pub fn resolved_terminal_path(&self) -> Option<String> {
        let configured = self.terminal_emulator.trim();
        if !configured.is_empty() {
            return resolve_executable(configured);
        }
        for term in Self::KNOWN_TERMINALS {
            if let Some(path) = resolve_executable(term) {
                return Some(path);
            }
        }
        None
    }

    /// Look up a custom opener for a given file extension.
    pub fn opener_for_ext(&self, ext: &str) -> Option<&String> {
        self.custom_openers.get(&ext.to_lowercase())
    }

    pub fn ensure_default_log_channels(&mut self) {
        if self.log_channels.is_empty() {
            self.log_channels = crate::integrations::journald::default_log_channels();
        }
    }
}

pub(crate) fn detect_first_available(commands: &[&str]) -> Option<String> {
    for command in commands {
        if let Some(resolved) = resolve_executable(command) {
            return Some(resolved);
        }
    }
    None
}

pub(crate) fn resolve_executable(command: &str) -> Option<String> {
    let command = command.trim();
    if command.is_empty() {
        return None;
    }

    let direct_path = expand_user_path(command);
    if direct_path.components().count() > 1 || command.starts_with('.') || command.starts_with('~') {
        return is_executable(&direct_path).then(|| direct_path.to_string_lossy().to_string());
    }

    if let Ok(output) = Command::new("which")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        if output.status.success() {
            let resolved = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !resolved.is_empty() && is_executable(&PathBuf::from(&resolved)) {
                return Some(resolved);
            }
        }
    }

    for dir in fallback_bin_dirs() {
        let candidate = dir.join(command);
        if is_executable(&candidate) {
            return Some(candidate.to_string_lossy().to_string());
        }
    }

    None
}

fn expand_user_path(path: &str) -> PathBuf {
    if path == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from(path));
    }
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

fn fallback_bin_dirs() -> Vec<PathBuf> {
    let mut dirs_list = vec![
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/bin"),
    ];

    if let Some(home) = dirs::home_dir() {
        dirs_list.insert(0, home.join(".local/bin"));
        dirs_list.insert(1, home.join(".cargo/bin"));
        dirs_list.insert(2, home.join(".local/kitty.app/bin"));
    }

    dirs_list
}

fn is_executable(path: &PathBuf) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && (metadata.permissions().mode() & 0o111) != 0)
        .unwrap_or(false)
}
