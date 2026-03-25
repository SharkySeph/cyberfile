use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{self, LocalProtocolManifest, ProtocolCommand, Settings};
use crate::scenes::MissionScene;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandSurfaceMode {
    Path,
    Protocol,
}

impl Default for CommandSurfaceMode {
    fn default() -> Self {
        Self::Path
    }
}

impl CommandSurfaceMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Path => "PATH",
            Self::Protocol => "PROTO",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::Path => "NEURAL INTERFACE // path or file query",
            Self::Protocol => "PROTOCOL LAUNCHER // action, scene, system command",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Protocol => "protocol",
        }
    }
}

#[derive(Debug, Clone)]
pub enum LauncherAction {
    OpenTerminalHere,
    ToggleSidebar,
    TogglePreview,
    ToggleHidden,
    ToggleResourceMonitor,
    ToggleEmbeddedTerminal,
    OpenSettings,
    OpenSceneManager,
    SaveMissionScene,
    RestoreMissionScene(String),
    StartDeepScan(String),
    TriggerFzf,
    OpenSftpDialog,
    RefreshRemoteNode,
    DisconnectRemoteNode,
    UploadSelectedToRemote,
    TailPath(PathBuf),
    LaunchExternalProgram {
        label: String,
        program: String,
        args: Vec<String>,
        cwd: Option<PathBuf>,
    },
    RunProtocolCommand {
        label: String,
        command: String,
        run_in_terminal: bool,
    },
    OpenPath(PathBuf),
}

#[derive(Debug, Clone)]
pub struct LauncherEntry {
    pub title: String,
    pub subtitle: String,
    pub section: String,
    pub action: LauncherAction,
    search_text: String,
}

impl LauncherEntry {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        subtitle: impl Into<String>,
        section: impl Into<String>,
        keywords: &[&str],
        action: LauncherAction,
    ) -> Self {
        let id = id.into();
        let title = title.into();
        let subtitle = subtitle.into();
        let section = section.into();
        let mut search_text = format!("{} {} {} {}", id, title, subtitle, section).to_lowercase();
        for keyword in keywords {
            search_text.push(' ');
            search_text.push_str(&keyword.to_lowercase());
        }

        Self {
            title,
            subtitle,
            section,
            action,
            search_text,
        }
    }

    pub fn matches(&self, query: &str) -> bool {
        let trimmed = query.trim().to_lowercase();
        trimmed.is_empty() || self.search_text.contains(&trimmed)
    }
}

pub fn builtin_entries(
    current_path: &Path,
    scenes: &[MissionScene],
    fzf_available: bool,
) -> Vec<LauncherEntry> {
    let path_label = current_path.to_string_lossy();
    let mut entries = vec![
        LauncherEntry::new(
            "protocol.open_terminal_here",
            "JACK IN HERE",
            format!("Launch terminal in {}", path_label),
            "PROTOCOLS",
            &["terminal", "shell", "jack", "cwd"],
            LauncherAction::OpenTerminalHere,
        ),
        LauncherEntry::new(
            "protocol.toggle_sidebar",
            "TOGGLE NETWORK MAP",
            "Show or hide the sidebar control plane",
            "PANELS",
            &["sidebar", "map", "panel", "layout"],
            LauncherAction::ToggleSidebar,
        ),
        LauncherEntry::new(
            "protocol.toggle_preview",
            "TOGGLE DATA SCAN",
            "Show or hide the preview panel",
            "PANELS",
            &["preview", "scan", "panel"],
            LauncherAction::TogglePreview,
        ),
        LauncherEntry::new(
            "protocol.toggle_hidden",
            "REVEAL CLOAKED CONSTRUCTS",
            "Toggle hidden file visibility in the current sector",
            "FILES",
            &["hidden", "dotfiles", "show", "visibility"],
            LauncherAction::ToggleHidden,
        ),
        LauncherEntry::new(
            "protocol.toggle_resource_monitor",
            "TOGGLE RESOURCE MONITOR",
            "Show CPU, memory, and disk telemetry",
            "SIGNALS",
            &["resource", "cpu", "memory", "telemetry", "monitor"],
            LauncherAction::ToggleResourceMonitor,
        ),
        LauncherEntry::new(
            "protocol.toggle_embedded_terminal",
            "TOGGLE NEURAL JACK PORT",
            "Open or close the embedded terminal panel",
            "SIGNALS",
            &["terminal", "panel", "command", "console"],
            LauncherAction::ToggleEmbeddedTerminal,
        ),
        LauncherEntry::new(
            "protocol.open_settings",
            "OPEN SYSTEM CONFIGURATION",
            "Jump to the settings manifest",
            "SYSTEM",
            &["settings", "config", "preferences"],
            LauncherAction::OpenSettings,
        ),
        LauncherEntry::new(
            "protocol.open_scene_manager",
            "OPEN SCENE MANAGER",
            "Inspect, pin, rename, and restore mission scenes",
            "SCENES",
            &["scene", "manager", "mission", "workspace"],
            LauncherAction::OpenSceneManager,
        ),
        LauncherEntry::new(
            "protocol.save_scene",
            "CAPTURE MISSION SCENE",
            "Snapshot tabs, panels, remotes, and command state",
            "SCENES",
            &["scene", "snapshot", "save", "mission", "workspace"],
            LauncherAction::SaveMissionScene,
        ),
        LauncherEntry::new(
            "protocol.open_remote_node",
            "CONNECT REMOTE NODE",
            "Open the SFTP uplink console",
            "NET",
            &["remote", "sftp", "ssh", "uplink", "connect"],
            LauncherAction::OpenSftpDialog,
        ),
    ];

    if fzf_available {
        entries.push(LauncherEntry::new(
            "protocol.launch_fzf",
            "OPEN FZF SCAN",
            "Launch interactive fuzzy scan in the current sector",
            "FILES",
            &["fzf", "search", "scan", "fuzzy"],
            LauncherAction::TriggerFzf,
        ));
    }

    for scene in scenes {
        entries.push(LauncherEntry::new(
            format!("scene.restore.{}", scene.id),
            format!("RESTORE {}", scene.name),
            scene.summary.clone(),
            "SCENES",
            &["scene", "restore", "mission", "workspace"],
            LauncherAction::RestoreMissionScene(scene.id.clone()),
        ));
    }

    entries
}

pub struct RemoteProviderState<'a> {
    pub connected: bool,
    pub busy: bool,
    pub display_name: &'a str,
    pub remote_path: &'a str,
    pub uploadable_count: usize,
}

pub fn remote_entries(state: &RemoteProviderState<'_>) -> Vec<LauncherEntry> {
    let mut entries = vec![LauncherEntry::new(
        "remote.open_dialog",
        if state.connected {
            "OPEN REMOTE NODE"
        } else {
            "CONNECT REMOTE NODE"
        },
        if state.connected {
            format!("{} // {}", state.display_name, state.remote_path)
        } else {
            "Open SFTP uplink dialog".to_string()
        },
        "NET",
        &["remote", "sftp", "ssh", "uplink", "connect"],
        LauncherAction::OpenSftpDialog,
    )];

    if state.connected {
        entries.push(LauncherEntry::new(
            "remote.refresh",
            "REFRESH REMOTE INDEX",
            if state.busy {
                "Remote listing already in progress".to_string()
            } else {
                format!("Refresh {}", state.remote_path)
            },
            "NET",
            &["remote", "refresh", "reload", "index", "uplink"],
            LauncherAction::RefreshRemoteNode,
        ));
        entries.push(LauncherEntry::new(
            "remote.disconnect",
            "DISCONNECT UPLINK",
            format!("Sever active link to {}", state.display_name),
            "NET",
            &["remote", "disconnect", "uplink", "ssh"],
            LauncherAction::DisconnectRemoteNode,
        ));

        if state.uploadable_count > 0 {
            entries.push(LauncherEntry::new(
                "remote.upload_selected",
                "START TRANSFER TO UPLINK",
                format!(
                    "Upload {} selected construct(s) to {}",
                    state.uploadable_count, state.remote_path
                ),
                "NET",
                &["upload", "transfer", "remote", "uplink", "sftp"],
                LauncherAction::UploadSelectedToRemote,
            ));
        }
    }

    entries
}

pub fn app_catalog_entries(
    settings: &Settings,
    current_path: &Path,
    selected_target: Option<&Path>,
) -> Vec<LauncherEntry> {
    let mut entries = Vec::new();
    let target = selected_target.unwrap_or(current_path);
    let target_label = target.to_string_lossy().to_string();

    if let Some(editor_program) = detect_editor_program() {
        let editor_name = Path::new(&editor_program)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| editor_program.clone());
        entries.push(LauncherEntry::new(
            format!("apps.editor.{}", editor_name),
            format!("LAUNCH {}", editor_name.to_uppercase()),
            format!("Open {} in external editor", target_label),
            "APPS",
            &["editor", "code", "zed", "text", "app", "launch"],
            LauncherAction::LaunchExternalProgram {
                label: editor_name,
                program: editor_program,
                args: vec![target.to_string_lossy().to_string()],
                cwd: Some(current_path.to_path_buf()),
            },
        ));
    }

    if let Some(system_opener) = config::resolve_executable("xdg-open") {
        entries.push(LauncherEntry::new(
            "apps.system_default",
            "OPEN WITH SYSTEM DEFAULT",
            format!("Open {} using xdg-open", target_label),
            "APPS",
            &["open", "system", "default", "launch", "app"],
            LauncherAction::LaunchExternalProgram {
                label: "system default".to_string(),
                program: system_opener,
                args: vec![target.to_string_lossy().to_string()],
                cwd: Some(current_path.to_path_buf()),
            },
        ));
    }

    if let Some(file_manager) = config::resolve_executable("xdg-open") {
        entries.push(LauncherEntry::new(
            "apps.file_manager",
            "OPEN CURRENT SECTOR IN SYSTEM FILE MANAGER",
            current_path.to_string_lossy().to_string(),
            "APPS",
            &["file", "manager", "system", "directory", "sector"],
            LauncherAction::LaunchExternalProgram {
                label: "file manager".to_string(),
                program: file_manager,
                args: vec![current_path.to_string_lossy().to_string()],
                cwd: Some(current_path.to_path_buf()),
            },
        ));
    }

    if let Some(terminal_path) = settings.resolved_terminal_path() {
        let terminal_name = Path::new(&terminal_path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| terminal_path.clone());
        entries.push(LauncherEntry::new(
            format!("apps.terminal.{}", terminal_name),
            format!("LAUNCH {}", terminal_name.to_uppercase()),
            format!("Start terminal in {}", current_path.to_string_lossy()),
            "APPS",
            &["terminal", "shell", "app", "launch", "cwd"],
            LauncherAction::LaunchExternalProgram {
                label: terminal_name,
                program: terminal_path,
                args: Vec::new(),
                cwd: Some(current_path.to_path_buf()),
            },
        ));
    }

    entries
}

pub fn file_tool_entries(selected_paths: &[PathBuf]) -> Vec<LauncherEntry> {
    let mut entries = Vec::new();
    if let Some(path) = selected_paths.iter().find(|path| path.is_file()) {
        entries.push(LauncherEntry::new(
            format!("file.tail.{}", path.to_string_lossy()),
            "TAIL SELECTED CONSTRUCT",
            format!("Read the last 80 lines of {}", path.to_string_lossy()),
            "TOOLS",
            &["tail", "log", "file", "selected", "read"],
            LauncherAction::TailPath(path.clone()),
        ));
    }
    entries
}

pub fn query_entries(query: &str) -> Vec<LauncherEntry> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    vec![LauncherEntry::new(
        "protocol.deep_scan",
        format!("DEEP SCAN \"{}\"", trimmed),
        "Run content search in the current sector",
        "FILES",
        &["search", "grep", "ripgrep", "scan"],
        LauncherAction::StartDeepScan(trimmed.to_string()),
    )]
}

pub fn protocol_entries(
    protocols: &[ProtocolCommand],
    section_label: &str,
    source_hint: &str,
) -> Vec<LauncherEntry> {
    protocols
        .iter()
        .filter(|protocol| !protocol.name.trim().is_empty() && !protocol.command.trim().is_empty())
        .map(|protocol| {
            let title = if protocol.icon.trim().is_empty() {
                protocol.name.clone()
            } else {
                format!("{} {}", protocol.icon.trim(), protocol.name)
            };
            let subtitle = if protocol.subtitle.trim().is_empty() {
                protocol.command.clone()
            } else {
                protocol.subtitle.clone()
            };
            let mut keywords: Vec<&str> = protocol.tags.iter().map(String::as_str).collect();
            keywords.push(source_hint);
            LauncherEntry::new(
                if protocol.id.trim().is_empty() {
                    format!("protocol.{}.{}", source_hint, protocol.name)
                } else {
                    protocol.id.clone()
                },
                title,
                subtitle,
                if protocol.section.trim().is_empty() {
                    section_label.to_string()
                } else {
                    protocol.section.clone()
                },
                &keywords,
                LauncherAction::RunProtocolCommand {
                    label: protocol.name.clone(),
                    command: protocol.command.clone(),
                    run_in_terminal: protocol.run_in_terminal,
                },
            )
        })
        .collect()
}

pub fn path_entry(title: String, subtitle: String, section: &str, path: PathBuf) -> LauncherEntry {
    LauncherEntry::new(
        format!("path.{}", path.to_string_lossy()),
        title,
        subtitle,
        section.to_string(),
        &["path", "file", "directory", "sector"],
        LauncherAction::OpenPath(path),
    )
}

#[derive(Debug, Clone)]
pub struct LoadedProtocolManifest {
    pub path: PathBuf,
    pub name: String,
    pub protocols: Vec<ProtocolCommand>,
}

pub fn load_local_protocol_manifest(current_path: &Path) -> Option<LoadedProtocolManifest> {
    let start_dir = if current_path.is_dir() {
        current_path
    } else {
        current_path.parent()?
    };

    for dir in start_dir.ancestors() {
        let candidate = dir.join(".cyberfile.toml");
        if !candidate.is_file() {
            continue;
        }

        let Ok(content) = fs::read_to_string(&candidate) else {
            continue;
        };
        let Ok(manifest) = toml::from_str::<LocalProtocolManifest>(&content) else {
            continue;
        };

        return Some(LoadedProtocolManifest {
            path: candidate,
            name: if manifest.meta.name.trim().is_empty() {
                dir.file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_else(|| "LOCAL MANIFEST".to_string())
            } else {
                manifest.meta.name
            },
            protocols: manifest.protocols,
        });
    }

    None
}

fn detect_editor_program() -> Option<String> {
    if let Ok(editor) = std::env::var("VISUAL") {
        if let Some(resolved) = config::resolve_executable(&editor) {
            return Some(resolved);
        }
    }
    if let Ok(editor) = std::env::var("EDITOR") {
        if let Some(resolved) = config::resolve_executable(&editor) {
            return Some(resolved);
        }
    }

    config::detect_first_available(&["code", "codium", "zed", "kate", "gedit"])
}

pub fn filter_entries(entries: &[LauncherEntry], query: &str, limit: usize) -> Vec<LauncherEntry> {
    entries
        .iter()
        .filter(|entry| entry.matches(query))
        .take(limit)
        .cloned()
        .collect()
}