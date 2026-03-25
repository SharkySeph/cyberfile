use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct MissionSceneTab {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub selected: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct MissionSceneSplitState {
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub selected: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct MissionSceneOverlayState {
    #[serde(default)]
    pub sidebar_visible: bool,
    #[serde(default)]
    pub preview_visible: bool,
    #[serde(default)]
    pub resource_monitor_visible: bool,
    #[serde(default)]
    pub terminal_panel_visible: bool,
    #[serde(default)]
    pub settings_panel_open: bool,
    #[serde(default)]
    pub process_matrix_visible: bool,
    #[serde(default)]
    pub service_deck_visible: bool,
    #[serde(default)]
    pub log_viewer_visible: bool,
    #[serde(default)]
    pub signal_deck_visible: bool,
    #[serde(default)]
    pub data_rain_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct MissionSceneTerminalState {
    #[serde(default)]
    pub input: String,
    #[serde(default)]
    pub history: Vec<String>,
    #[serde(default)]
    pub running_command: Option<String>,
    #[serde(default)]
    pub output_tail: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct MissionSceneRemoteState {
    #[serde(default)]
    pub connected: bool,
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: String,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub remote_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MissionScene {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub current_path: String,
    #[serde(default)]
    pub active_tab: usize,
    #[serde(default)]
    pub tabs: Vec<MissionSceneTab>,
    #[serde(default)]
    pub split: MissionSceneSplitState,
    #[serde(default)]
    pub overlays: MissionSceneOverlayState,
    #[serde(default)]
    pub terminal: MissionSceneTerminalState,
    #[serde(default)]
    pub remote: MissionSceneRemoteState,
    #[serde(default)]
    pub filter_text: String,
    #[serde(default)]
    pub command_text: String,
    #[serde(default = "default_command_mode")]
    pub command_mode: String,
    #[serde(default = "default_theme")]
    pub theme_id: String,
    #[serde(default = "default_view")]
    pub view_mode: String,
    #[serde(default)]
    pub updated_at: String,
}

impl Default for MissionScene {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            summary: String::new(),
            notes: String::new(),
            pinned: false,
            tags: Vec::new(),
            current_path: String::new(),
            active_tab: 0,
            tabs: Vec::new(),
            split: MissionSceneSplitState::default(),
            overlays: MissionSceneOverlayState::default(),
            terminal: MissionSceneTerminalState::default(),
            remote: MissionSceneRemoteState::default(),
            filter_text: String::new(),
            command_text: String::new(),
            command_mode: default_command_mode(),
            theme_id: default_theme(),
            view_mode: default_view(),
            updated_at: String::new(),
        }
    }
}

impl MissionScene {
    pub fn display_label(&self) -> String {
        if self.summary.trim().is_empty() {
            self.name.clone()
        } else {
            format!("{} // {}", self.name, self.summary)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RecentSceneRecord {
    #[serde(default)]
    pub scene_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub last_used_at: String,
}

impl RecentSceneRecord {
    pub fn from_scene(scene: &MissionScene, last_used_at: String) -> Self {
        Self {
            scene_id: scene.id.clone(),
            name: scene.name.clone(),
            summary: scene.summary.clone(),
            pinned: scene.pinned,
            last_used_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SceneStore {
    #[serde(default)]
    pub saved_scenes: Vec<MissionScene>,
    #[serde(default)]
    pub recent_scenes: Vec<RecentSceneRecord>,
    #[serde(default)]
    pub session_scene: Option<MissionScene>,
}

impl SceneStore {
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cyberfile")
            .join("scenes.toml")
    }

    pub fn load(legacy_saved_scenes: &[MissionScene]) -> Self {
        let path = Self::path();
        let mut store = if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|content| toml::from_str(&content).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        };

        if store.saved_scenes.is_empty() && !legacy_saved_scenes.is_empty() {
            store.saved_scenes = legacy_saved_scenes.to_vec();
            let now = chrono::Local::now().to_rfc3339();
            store.recent_scenes = store
                .saved_scenes
                .iter()
                .take(4)
                .map(|scene| RecentSceneRecord::from_scene(scene, now.clone()))
                .collect();
            store.save();
        }

        store
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(content) = toml::to_string_pretty(self) {
            let _ = fs::write(path, content);
        }
    }

    pub fn ensure_default_presets(&mut self, root: &Path) {
        if !self.saved_scenes.is_empty() {
            return;
        }

        self.saved_scenes = default_pinned_scenes(root);
        let now = chrono::Local::now().to_rfc3339();
        self.recent_scenes = self
            .saved_scenes
            .iter()
            .take(4)
            .map(|scene| RecentSceneRecord::from_scene(scene, now.clone()))
            .collect();
        self.save();
    }
}

fn default_command_mode() -> String {
    "path".to_string()
}

fn default_theme() -> String {
    "night_city".to_string()
}

fn default_view() -> String {
    "list".to_string()
}

pub fn slugify_scene_name(input: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }

    slug.trim_matches('-').to_string()
}

fn choose_existing_dir(candidates: &[PathBuf], fallback: &Path) -> PathBuf {
    candidates
        .iter()
        .find(|path| path.is_dir())
        .cloned()
        .unwrap_or_else(|| fallback.to_path_buf())
}

fn preset_scene(
    id: &str,
    name: &str,
    summary: &str,
    notes: &str,
    tags: &[&str],
    path: &Path,
    view_mode: &str,
    command_mode: &str,
    command_text: &str,
    split_active: bool,
    split_path: &Path,
    overlays: MissionSceneOverlayState,
) -> MissionScene {
    let now = chrono::Local::now().to_rfc3339();
    MissionScene {
        id: id.to_string(),
        name: name.to_string(),
        summary: summary.to_string(),
        notes: notes.to_string(),
        pinned: true,
        tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
        current_path: path.to_string_lossy().to_string(),
        active_tab: 0,
        tabs: vec![MissionSceneTab {
            path: path.to_string_lossy().to_string(),
            selected: None,
        }],
        split: MissionSceneSplitState {
            active: split_active,
            path: split_path.to_string_lossy().to_string(),
            selected: None,
        },
        overlays,
        terminal: MissionSceneTerminalState {
            input: String::new(),
            history: Vec::new(),
            running_command: None,
            output_tail: Vec::new(),
        },
        remote: MissionSceneRemoteState::default(),
        filter_text: String::new(),
        command_text: command_text.to_string(),
        command_mode: command_mode.to_string(),
        theme_id: default_theme(),
        view_mode: view_mode.to_string(),
        updated_at: now,
    }
}

pub fn default_pinned_scenes(root: &Path) -> Vec<MissionScene> {
    let home = dirs::home_dir().unwrap_or_else(|| root.to_path_buf());
    let projects = choose_existing_dir(
        &[
            home.join("Desktop").join("Projects"),
            home.join("Projects"),
            root.to_path_buf(),
        ],
        &home,
    );
    let downloads = choose_existing_dir(&[home.join("Downloads"), root.to_path_buf()], &home);
    let documents = choose_existing_dir(&[home.join("Documents"), root.to_path_buf()], &home);

    vec![
        preset_scene(
            "preset-code-ops",
            "Code Ops",
            "Pinned starter deck for active project work",
            "Terminal-forward workspace tuned for source trees and project roots.",
            &["preset", "code", "ops"],
            &projects,
            "list",
            "protocol",
            "cargo check",
            true,
            &documents,
            MissionSceneOverlayState {
                sidebar_visible: true,
                preview_visible: false,
                resource_monitor_visible: false,
                terminal_panel_visible: true,
                settings_panel_open: false,
                process_matrix_visible: false,
                service_deck_visible: false,
                log_viewer_visible: false,
                signal_deck_visible: false,
                data_rain_enabled: false,
            },
        ),
        preset_scene(
            "preset-media-intake",
            "Media Intake",
            "Pinned starter deck for downloads triage and preview",
            "Preview-heavy scene for sorting assets, screenshots, and incoming media.",
            &["preset", "media", "intake"],
            &downloads,
            "grid",
            "path",
            downloads.to_string_lossy().as_ref(),
            false,
            &downloads,
            MissionSceneOverlayState {
                sidebar_visible: true,
                preview_visible: true,
                resource_monitor_visible: false,
                terminal_panel_visible: false,
                settings_panel_open: false,
                process_matrix_visible: false,
                service_deck_visible: false,
                log_viewer_visible: false,
                signal_deck_visible: false,
                data_rain_enabled: false,
            },
        ),
        preset_scene(
            "preset-remote-maintenance",
            "Remote Maintenance",
            "Pinned starter deck for uplink and service work",
            "Protocol-first scene that keeps the terminal and sidebar open for remote operations.",
            &["preset", "remote", "maintenance"],
            &projects,
            "list",
            "protocol",
            "remote",
            true,
            &home,
            MissionSceneOverlayState {
                sidebar_visible: true,
                preview_visible: false,
                resource_monitor_visible: true,
                terminal_panel_visible: true,
                settings_panel_open: false,
                process_matrix_visible: false,
                service_deck_visible: false,
                log_viewer_visible: false,
                signal_deck_visible: false,
                data_rain_enabled: false,
            },
        ),
        preset_scene(
            "preset-archive-recovery",
            "Archive Recovery",
            "Pinned starter deck for archives, logs, and raw inspection",
            "Split-pane scene for comparing extracted data, logs, and recovered constructs.",
            &["preset", "archive", "recovery"],
            &documents,
            "hex",
            "protocol",
            "tail log",
            true,
            &downloads,
            MissionSceneOverlayState {
                sidebar_visible: true,
                preview_visible: true,
                resource_monitor_visible: false,
                terminal_panel_visible: true,
                settings_panel_open: false,
                process_matrix_visible: false,
                service_deck_visible: false,
                log_viewer_visible: false,
                signal_deck_visible: false,
                data_rain_enabled: false,
            },
        ),
    ]
}