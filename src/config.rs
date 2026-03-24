use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_view")]
    pub default_view: String,
    #[serde(default)]
    pub show_hidden: bool,
    #[serde(default = "default_true")]
    pub confirm_delete: bool,
    #[serde(default = "default_true")]
    pub boot_sequence: bool,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: f32,
    #[serde(default)]
    pub scanlines_enabled: bool,
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
        }
    }
}

impl Settings {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cyberfile")
            .join("config.toml")
    }

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
