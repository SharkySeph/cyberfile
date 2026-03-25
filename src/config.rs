use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

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
            scanlines_enabled: false,
            crt_effect: false,
            terminal_emulator: String::new(),
            custom_openers: BTreeMap::new(),
            window_width: default_window_width(),
            window_height: default_window_height(),
            last_directory: String::new(),
            bookmarks: Vec::new(),
            saved_tabs: Vec::new(),
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
                if let Ok(settings) = toml::from_str(&content) {
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

    /// Resolve which terminal to use: configured value, or auto-detect.
    pub fn resolved_terminal(&self) -> Option<String> {
        if !self.terminal_emulator.is_empty() {
            return Some(self.terminal_emulator.clone());
        }
        for term in Self::KNOWN_TERMINALS {
            let ok = std::process::Command::new("which")
                .arg(term)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if ok {
                return Some((*term).to_string());
            }
        }
        None
    }

    /// Look up a custom opener for a given file extension.
    pub fn opener_for_ext(&self, ext: &str) -> Option<&String> {
        self.custom_openers.get(&ext.to_lowercase())
    }
}
