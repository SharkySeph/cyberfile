use eframe::egui::{self, Color32, RichText};
use rodio::Source;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
};
use std::time::{Duration, Instant};

use crate::config::Settings;
use crate::filesystem::{self, EntryKind, FileEntry, SortColumn};
use crate::integrations::journald::LogChannel;
use crate::integrations::media::MediaState;
use crate::launcher::{
    self, CommandSurfaceMode, LauncherAction, LauncherEntry, LoadedProtocolManifest,
};
use crate::scenes::{
    slugify_scene_name, MissionScene, MissionSceneOverlayState, MissionSceneRemoteState,
    MissionSceneSplitState, MissionSceneTab, MissionSceneTerminalState, RecentSceneRecord,
    SceneStore,
};
use crate::theme::{self, CyberTheme};

// ── View Modes ────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ViewMode {
    List,
    Grid,
    HexGrid,
    Hex,
}

// ── Tab State ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) struct Tab {
    pub path: PathBuf,
    pub selected: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ClipboardOp {
    Copy,
    Cut,
}

// ── Undo/Redo ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) enum UndoAction {
    Rename { old_path: PathBuf, new_path: PathBuf },
    Delete { original_path: PathBuf, trash_name: String },
    Copy { copied_to: Vec<PathBuf> },
    Move { sources: Vec<PathBuf>, destinations: Vec<PathBuf> },
    Create { path: PathBuf, kind: EntryKind },
}

type SharedSftpConnection = Arc<Mutex<crate::integrations::sftp::SftpConnection>>;

enum BackgroundTaskResult {
    TerminalFinished {
        job_id: u64,
        lines: Vec<String>,
        duration: Duration,
        success: bool,
    },
    TerminalSpawnFailed {
        job_id: u64,
        message: String,
    },
    ContentSearchFinished {
        request_id: u64,
        query: String,
        results: Vec<(String, u32, String)>,
    },
    SftpConnected {
        request_id: u64,
        connection: SharedSftpConnection,
        display_name: String,
        entries: Vec<crate::integrations::sftp::RemoteEntry>,
        clear_password: bool,
    },
    SftpListed {
        request_id: u64,
        path: String,
        entries: Vec<crate::integrations::sftp::RemoteEntry>,
    },
    SftpDownloaded {
        request_id: u64,
        file_name: String,
    },
    SftpUploaded {
        request_id: u64,
        file_names: Vec<String>,
    },
    SftpFailed {
        request_id: u64,
        message: String,
    },
}

#[derive(Debug, Clone)]
enum StartupSceneRequest {
    ResumeSession,
    RestoreScene(String),
    FreshStart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProcessSortMode {
    Cpu,
    Memory,
    Name,
    Pid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OperatorJobState {
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct OperatorJob {
    pub id: u64,
    pub label: String,
    pub command: String,
    pub cwd: PathBuf,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub duration_ms: Option<u128>,
    pub status: OperatorJobState,
    pub output_tail: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SignalDeckTab {
    #[default]
    Audio,
    Media,
    Clipboard,
    Notifications,
    Power,
}

// ── Sound Types ───────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub(crate) enum SoundType {
    Navigate,
    Select,
    Error,
    Delete,
    CopyComplete,
}

pub struct CyberFile {
    // ── File System State ────────────────────────────────
    pub(crate) current_path: PathBuf,
    pub(crate) entries: Vec<FileEntry>,
    pub(crate) selected: Option<usize>,
    pub(crate) multi_selected: HashSet<usize>,
    pub(crate) history: Vec<PathBuf>,
    pub(crate) history_pos: usize,
    pub(crate) sort_column: SortColumn,
    pub(crate) sort_ascending: bool,

    // ── Type-ahead Search ────────────────────────────────
    pub(crate) type_ahead_buffer: String,
    pub(crate) type_ahead_last_key: Instant,

    // ── UI State ─────────────────────────────────────────
    pub(crate) sidebar_visible: bool,
    pub(crate) show_hidden: bool,
    pub(crate) context_menu_open: bool,
    pub(crate) context_menu_pos: egui::Pos2,
    pub(crate) context_menu_just_opened: bool,
    pub(crate) data_rain_cols: Vec<f32>,

    // ── Command Bar ──────────────────────────────────────
    pub(crate) command_bar_text: String,
    pub(crate) command_bar_active: bool,
    pub(crate) command_surface_mode: CommandSurfaceMode,
    pub(crate) launcher_results: Vec<LauncherEntry>,
    pub(crate) launcher_selected: usize,
    pub(crate) focus_command_bar_next_frame: bool,
    pub(crate) local_protocol_manifest: Option<LoadedProtocolManifest>,

    // ── Rename ───────────────────────────────────────────
    pub(crate) rename_index: Option<usize>,
    pub(crate) rename_text: String,

    // ── New Folder Dialog ────────────────────────────────
    pub(crate) new_folder_dialog: bool,
    pub(crate) new_folder_name: String,

    // ── New File Dialog ──────────────────────────────────
    pub(crate) new_file_dialog: bool,
    pub(crate) new_file_name: String,

    // ── Confirm Delete Dialog ────────────────────────────
    pub(crate) confirm_delete_dialog: bool,
    pub(crate) delete_pending_indices: Vec<usize>,

    // ── Real-time Filter ─────────────────────────────────
    pub(crate) filter_text: String,

    // ── Boot Sequence ────────────────────────────────────
    pub(crate) boot_complete: bool,
    pub(crate) boot_start: Instant,

    // ── Clipboard ────────────────────────────────────────
    pub(crate) clipboard_op: Option<ClipboardOp>,
    pub(crate) clipboard_paths: Vec<PathBuf>,

    // ── Status ───────────────────────────────────────────
    pub(crate) status_message: String,
    pub(crate) error_message: Option<(String, Instant)>,

    // ── Bookmarks ────────────────────────────────────────
    pub(crate) bookmarks: Vec<PathBuf>,

    // ── Theme ────────────────────────────────────────────
    pub(crate) current_theme: CyberTheme,

    // ── Visual Effects ───────────────────────────────────
    pub(crate) scanlines_enabled: bool,
    pub(crate) crt_effect: bool,
    pub(crate) glitch_active: bool,
    pub(crate) glitch_start: Option<Instant>,

    // ── Resource Monitor ─────────────────────────────────
    pub(crate) resource_monitor_visible: bool,
    pub(crate) sys_info: sysinfo::System,
    pub(crate) sys_disks: sysinfo::Disks,
    pub(crate) sys_last_refresh: Instant,
    pub(crate) cpu_history: VecDeque<f32>,
    pub(crate) mem_history: VecDeque<f32>,

    // ── Settings Panel ───────────────────────────────────
    pub(crate) settings_panel_open: bool,
    pub(crate) scene_manager_open: bool,
    pub(crate) process_matrix_open: bool,
    pub(crate) service_deck_open: bool,
    pub(crate) log_viewer_open: bool,
    pub(crate) scene_manager_selected_id: Option<String>,
    pub(crate) scene_capture_name: String,
    pub(crate) scene_store: SceneStore,
    startup_scene_request: Option<StartupSceneRequest>,
    pub(crate) ui_scale_preview: f32,
    pub(crate) process_filter_text: String,
    pub(crate) process_sort_mode: ProcessSortMode,
    pub(crate) process_entries: Vec<crate::integrations::processes::ProcessEntry>,
    pub(crate) process_selected_pid: Option<i32>,
    pub(crate) process_last_refresh: Instant,
    pub(crate) service_filter_text: String,
    pub(crate) service_entries: Vec<crate::integrations::services::ServiceEntry>,
    pub(crate) service_selected_unit: Option<String>,
    pub(crate) service_status_output: Vec<String>,
    pub(crate) service_last_refresh: Instant,
    pub(crate) log_selected_channel_id: Option<String>,
    pub(crate) log_output: Vec<String>,
    pub(crate) log_last_refresh: Instant,

    // ── Signal Deck (Stage 4) ────────────────────────────
    pub(crate) signal_deck_open: bool,
    pub(crate) signal_deck_tab: SignalDeckTab,
    pub(crate) process_matrix_detached: bool,
    pub(crate) service_deck_detached: bool,
    pub(crate) log_viewer_detached: bool,
    pub(crate) signal_deck_detached: bool,
    pub(crate) audio_snapshot: crate::integrations::audio::AudioSnapshot,
    pub(crate) audio_volume_slider: u32,
    pub(crate) audio_last_refresh: Instant,
    pub(crate) power_info: crate::integrations::audio::PowerInfo,
    pub(crate) brightness_percent: Option<u32>,
    pub(crate) clipboard_entries: Vec<crate::integrations::audio::ClipboardEntry>,
    pub(crate) clipboard_last_refresh: Instant,
    pub(crate) notification_entries: Vec<crate::integrations::audio::NotificationEntry>,
    pub(crate) notification_last_refresh: Instant,

    // ── Network Mesh (Stage 5) ───────────────────────────
    pub(crate) network_mesh_open: bool,
    pub(crate) network_mesh_detached: bool,
    pub(crate) network_mesh_tab: crate::ui::network_mesh::NetworkMeshTab,
    pub(crate) network_nmcli_available: bool,
    pub(crate) network_interfaces: Vec<crate::integrations::network::NetworkInterface>,
    pub(crate) network_wifi_list: Vec<crate::integrations::network::WifiNetwork>,
    pub(crate) network_vpn_list: Vec<crate::integrations::network::VpnConnection>,
    pub(crate) network_last_refresh: Instant,
    pub(crate) network_throughput_history: crate::ui::network_mesh::ThroughputHistory,
    pub(crate) network_throughput_prev: std::collections::BTreeMap<String, crate::integrations::network::ThroughputSample>,

    // ── Device Bay (Stage 5) ─────────────────────────────
    pub(crate) device_bay_open: bool,
    pub(crate) device_bay_detached: bool,
    pub(crate) device_udisksctl_available: bool,
    pub(crate) device_entries: Vec<crate::integrations::devices::BlockDevice>,
    pub(crate) device_last_refresh: Instant,
    pub(crate) device_show_all: bool,
    pub(crate) device_error: Option<String>,

    // ── Window Bridge (Stage 6) ──────────────────────────
    pub(crate) window_bridge_open: bool,
    pub(crate) window_bridge_detached: bool,
    pub(crate) window_bridge_tab: crate::ui::window_bridge::WindowBridgeTab,
    pub(crate) wm_backend: Option<crate::integrations::windows::WmBackend>,
    pub(crate) wm_windows: Vec<crate::integrations::windows::WmWindow>,
    pub(crate) wm_workspaces: Vec<crate::integrations::windows::WmWorkspace>,
    pub(crate) wm_last_refresh: Instant,

    // ── Settings ─────────────────────────────────────────
    pub(crate) settings: Settings,

    // ── View Mode ────────────────────────────────────────
    pub(crate) view_mode: ViewMode,

    // ── Tabs ─────────────────────────────────────────────
    pub(crate) tabs: Vec<Tab>,
    pub(crate) active_tab: usize,

    // ── Preview Panel ────────────────────────────────────
    pub(crate) preview_visible: bool,

    // ── Data Rain ────────────────────────────────────────
    pub(crate) data_rain_enabled: bool,

    // ── Media / Music ────────────────────────────────────
    pub(crate) media_state: MediaState,
    pub(crate) media_last_refresh: Instant,
    pub(crate) media_preferred_player: String,
    pub(crate) media_players: Vec<crate::integrations::media::PlayerInfo>,
    pub(crate) media_players_last_refresh: Instant,

    // ── Idle Inhibit ─────────────────────────────────────
    pub(crate) idle_inhibit_child: Option<std::process::Child>,

    // ── fzf Integration ──────────────────────────────────
    pub(crate) fzf_available: bool,
    pub(crate) fzf_results: Vec<PathBuf>,
    pub(crate) fzf_mode: bool,

    // ── Open With Dialog ─────────────────────────────────
    pub(crate) open_with_dialog: bool,
    pub(crate) open_with_text: String,
    pub(crate) open_with_target: Option<PathBuf>,

    // ── Internal ─────────────────────────────────────────
    pub(crate) theme_applied: bool,
    pub(crate) frame_count: u64,

    // ── Cached Disk Info ─────────────────────────────────
    pub(crate) disk_info_cache: Option<(String, String, String, String)>,
    pub(crate) disk_info_last_refresh: Instant,
    pub(crate) disk_info_path: PathBuf,

    // ── Hex Grid Zoom ────────────────────────────────────
    pub(crate) hex_zoom: f32,
    pub(crate) hex_pan_offset: egui::Vec2,

    // ── Thumbnail Cache ──────────────────────────────────
    pub(crate) thumbnail_cache: HashMap<PathBuf, egui::TextureHandle>,
    pub(crate) thumbnail_failed: HashSet<PathBuf>,

    // ── Properties Dialog ────────────────────────────────
    pub(crate) properties_dialog: bool,
    pub(crate) properties_target: Option<PathBuf>,

    // ── Trash View ───────────────────────────────────────
    pub(crate) trash_view_open: bool,
    pub(crate) trash_entries: Vec<(String, PathBuf)>,

    // ── Symlink Creation Dialog ──────────────────────────
    pub(crate) symlink_dialog: bool,
    pub(crate) symlink_name: String,

    // ── Content Search Dialog ────────────────────────────
    pub(crate) content_search_dialog: bool,
    pub(crate) content_search_query: String,
    pub(crate) content_search_results: Vec<(String, u32, String)>,
    pub(crate) content_search_active_query: String,
    pub(crate) content_search_request_id: u64,

    // ── Batch Rename Dialog ──────────────────────────────
    pub(crate) batch_rename_dialog: bool,
    pub(crate) batch_rename_find: String,
    pub(crate) batch_rename_replace: String,
    pub(crate) batch_rename_use_regex: bool,

    // ── Undo/Redo Stack ──────────────────────────────────
    pub(crate) undo_stack: Vec<UndoAction>,
    pub(crate) redo_stack: Vec<UndoAction>,

    // ── Split Pane ("DUAL JACK") ─────────────────────────
    pub(crate) split_pane_active: bool,
    pub(crate) split_pane_path: PathBuf,
    pub(crate) split_pane_entries: Vec<FileEntry>,
    pub(crate) split_pane_selected: Option<usize>,
    pub(crate) split_pane_sort_column: SortColumn,
    pub(crate) split_pane_sort_ascending: bool,

    // ── Rubber Band Selection ────────────────────────────
    pub(crate) rubber_band_start: Option<egui::Pos2>,
    pub(crate) rubber_band_active: bool,

    // ── Drag and Drop ────────────────────────────────────
    pub(crate) drag_source_paths: Vec<PathBuf>,
    pub(crate) dragging: bool,

    // ── Terminal Panel ("NEURAL JACK PORT") ──────────────
    pub(crate) terminal_panel_visible: bool,
    pub(crate) terminal_input: String,
    pub(crate) terminal_history: Vec<String>,
    pub(crate) terminal_output: Vec<String>,
    pub(crate) operator_jobs: Vec<OperatorJob>,
    pub(crate) operator_job_selected_id: Option<u64>,
    pub(crate) next_operator_job_id: u64,
    pub(crate) terminal_task_running: bool,
    pub(crate) terminal_running_command: Option<String>,
    pub(crate) terminal_started_at: Option<Instant>,

    // ── Sound ────────────────────────────────────────────
    pub(crate) sound_enabled: bool,

    // ── Phase D: Visual Effects ──────────────────────────
    pub(crate) neon_glow: bool,
    pub(crate) chromatic_aberration: bool,
    pub(crate) holographic_noise: bool,

    // ── Phase E: Accessibility ───────────────────────────
    pub(crate) reduced_motion: bool,
    pub(crate) high_contrast: bool,

    // ── Phase D: SFTP Remote ─────────────────────────────
    pub(crate) sftp_dialog: bool,
    pub(crate) sftp_host: String,
    pub(crate) sftp_port: String,
    pub(crate) sftp_user: String,
    pub(crate) sftp_password: String,
    pub(crate) sftp_connection: Option<SharedSftpConnection>,
    pub(crate) sftp_display_name: String,
    pub(crate) sftp_remote_path: String,
    pub(crate) sftp_remote_entries: Vec<crate::integrations::sftp::RemoteEntry>,
    pub(crate) sftp_error: Option<String>,
    pub(crate) sftp_busy: bool,
    pub(crate) sftp_request_id: u64,
    pub(crate) sftp_operation_label: String,

    // ── Background Tasks ─────────────────────────────────
    pub(crate) content_search_in_progress: bool,
    background_tx: Sender<BackgroundTaskResult>,
    background_rx: Receiver<BackgroundTaskResult>,
}

impl CyberFile {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut settings = Settings::load();
        let seeded_log_channels = settings.log_channels.is_empty();
        settings.ensure_default_log_channels();
        if seeded_log_channels {
            settings.save();
        }
        let (background_tx, background_rx) = mpsc::channel();
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let theme = CyberTheme::from_id(&settings.theme);

        // Restore bookmarks from settings
        let bookmarks: Vec<PathBuf> = settings
            .bookmarks
            .iter()
            .map(|s| PathBuf::from(s))
            .filter(|p| p.exists())
            .collect();

        // Restore tabs from settings, or default to home
        let saved_tabs: Vec<Tab> = if settings.saved_tabs.is_empty() {
            vec![Tab {
                path: home.clone(),
                selected: None,
            }]
        } else {
            settings
                .saved_tabs
                .iter()
                .map(|s| {
                    let p = PathBuf::from(s);
                    Tab {
                        path: if p.is_dir() { p } else { home.clone() },
                        selected: None,
                    }
                })
                .collect()
        };

        let start_path = if let Ok(cli_path) = std::env::var("CYBERFILE_START_PATH") {
            let p = PathBuf::from(&cli_path);
            if p.is_dir() { p } else { saved_tabs[0].path.clone() }
        } else {
            saved_tabs[0].path.clone()
        };

        let mut sys = sysinfo::System::new();
        sys.refresh_cpu_all();
        sys.refresh_memory();
        let disks = sysinfo::Disks::new_with_refreshed_list();
        let local_protocol_manifest = launcher::load_local_protocol_manifest(&start_path);
        let mut scene_store = SceneStore::load(&settings.saved_scenes);
        scene_store.ensure_default_presets(&start_path);

        if !settings.saved_scenes.is_empty() {
            settings.saved_scenes.clear();
            settings.save();
        }

        let sound_enabled = settings.sound_enabled;
        let ui_scale_preview = settings.font_size;
        let startup_scene_request = if scene_store.session_scene.is_some() {
            Some(StartupSceneRequest::ResumeSession)
        } else {
            Some(StartupSceneRequest::FreshStart)
        };

        Self {
            current_path: start_path.clone(),
            entries: Vec::new(),
            selected: None,
            multi_selected: HashSet::new(),
            history: vec![home.clone()],
            history_pos: 0,
            sort_column: SortColumn::Name,
            sort_ascending: true,
            type_ahead_buffer: String::new(),
            type_ahead_last_key: Instant::now(),
            sidebar_visible: true,
            show_hidden: settings.show_hidden,
            context_menu_open: false,
            context_menu_pos: egui::pos2(0.0, 0.0),
            context_menu_just_opened: false,
            data_rain_cols: (0..80).map(|i| (i as f32 * 7.77) % 100.0).collect(),
            command_bar_text: start_path.to_string_lossy().to_string(),
            command_bar_active: false,
            command_surface_mode: CommandSurfaceMode::Path,
            launcher_results: Vec::new(),
            launcher_selected: 0,
            focus_command_bar_next_frame: false,
            local_protocol_manifest,
            rename_index: None,
            rename_text: String::new(),
            new_folder_dialog: false,
            new_folder_name: String::new(),
            new_file_dialog: false,
            new_file_name: String::new(),
            confirm_delete_dialog: false,
            delete_pending_indices: Vec::new(),
            filter_text: String::new(),
            boot_complete: !settings.boot_sequence,
            boot_start: Instant::now(),
            clipboard_op: None,
            clipboard_paths: Vec::new(),
            status_message: "SYSTEM OPERATIONAL".into(),
            error_message: None,
            bookmarks,
            current_theme: theme,
            scanlines_enabled: settings.scanlines_enabled,
            crt_effect: settings.crt_effect,
            glitch_active: false,
            glitch_start: None,
            resource_monitor_visible: false,
            sys_info: sys,
            sys_disks: disks,
            sys_last_refresh: Instant::now(),
            cpu_history: VecDeque::new(),
            mem_history: VecDeque::new(),
            settings_panel_open: false,
            scene_manager_open: false,
            process_matrix_open: false,
            service_deck_open: false,
            log_viewer_open: false,
            scene_manager_selected_id: scene_store.saved_scenes.first().map(|scene| scene.id.clone()),
            scene_capture_name: String::new(),
            scene_store,
            startup_scene_request,
            ui_scale_preview,
            process_filter_text: String::new(),
            process_sort_mode: ProcessSortMode::Cpu,
            process_entries: Vec::new(),
            process_selected_pid: None,
            process_last_refresh: Instant::now(),
            service_filter_text: String::new(),
            service_entries: Vec::new(),
            service_selected_unit: None,
            service_status_output: Vec::new(),
            service_last_refresh: Instant::now(),
            log_selected_channel_id: settings.log_channels.first().map(|channel| channel.id.clone()),
            log_output: Vec::new(),
            log_last_refresh: Instant::now(),
            signal_deck_open: false,
            signal_deck_tab: SignalDeckTab::Audio,
            process_matrix_detached: false,
            service_deck_detached: false,
            log_viewer_detached: false,
            signal_deck_detached: false,
            audio_snapshot: crate::integrations::audio::AudioSnapshot::default(),
            audio_volume_slider: 50,
            audio_last_refresh: Instant::now(),
            power_info: crate::integrations::audio::PowerInfo::default(),
            brightness_percent: None,
            clipboard_entries: Vec::new(),
            clipboard_last_refresh: Instant::now(),
            notification_entries: Vec::new(),
            notification_last_refresh: Instant::now(),
            network_mesh_open: false,
            network_mesh_detached: false,
            network_mesh_tab: crate::ui::network_mesh::NetworkMeshTab::default(),
            network_nmcli_available: crate::integrations::network::nmcli_available(),
            network_interfaces: Vec::new(),
            network_wifi_list: Vec::new(),
            network_vpn_list: Vec::new(),
            network_last_refresh: Instant::now(),
            network_throughput_history: std::collections::BTreeMap::new(),
            network_throughput_prev: std::collections::BTreeMap::new(),
            device_bay_open: false,
            device_bay_detached: false,
            device_udisksctl_available: crate::integrations::devices::udisksctl_available(),
            device_entries: Vec::new(),
            device_last_refresh: Instant::now(),
            device_show_all: false,
            device_error: None,
            window_bridge_open: false,
            window_bridge_detached: false,
            window_bridge_tab: crate::ui::window_bridge::WindowBridgeTab::default(),
            wm_backend: crate::integrations::windows::detect_wm(),
            wm_windows: Vec::new(),
            wm_workspaces: Vec::new(),
            wm_last_refresh: Instant::now(),
            settings,
            view_mode: ViewMode::List,
            tabs: saved_tabs,
            active_tab: 0,
            preview_visible: false,
            data_rain_enabled: false,
            media_state: MediaState::default(),
            media_last_refresh: Instant::now(),
            media_preferred_player: String::new(),
            media_players: Vec::new(),
            media_players_last_refresh: Instant::now(),
            idle_inhibit_child: None,
            fzf_available: crate::integrations::fzf::is_available(),
            fzf_results: Vec::new(),
            fzf_mode: false,
            open_with_dialog: false,
            open_with_text: String::new(),
            open_with_target: None,
            theme_applied: false,
            frame_count: 0,
            disk_info_cache: None,
            disk_info_last_refresh: Instant::now(),
            disk_info_path: PathBuf::new(),
            hex_zoom: 1.0,
            hex_pan_offset: egui::Vec2::ZERO,
            thumbnail_cache: HashMap::new(),
            thumbnail_failed: HashSet::new(),
            properties_dialog: false,
            properties_target: None,
            trash_view_open: false,
            trash_entries: Vec::new(),
            symlink_dialog: false,
            symlink_name: String::new(),
            content_search_dialog: false,
            content_search_query: String::new(),
            content_search_results: Vec::new(),
            content_search_active_query: String::new(),
            content_search_request_id: 0,
            batch_rename_dialog: false,
            batch_rename_find: String::new(),
            batch_rename_replace: String::new(),
            batch_rename_use_regex: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            split_pane_active: false,
            split_pane_path: home.clone(),
            split_pane_entries: Vec::new(),
            split_pane_selected: None,
            split_pane_sort_column: SortColumn::Name,
            split_pane_sort_ascending: true,
            rubber_band_start: None,
            rubber_band_active: false,
            drag_source_paths: Vec::new(),
            dragging: false,
            terminal_panel_visible: false,
            terminal_input: String::new(),
            terminal_history: Vec::new(),
            terminal_output: Vec::new(),
            operator_jobs: Vec::new(),
            operator_job_selected_id: None,
            next_operator_job_id: 1,
            terminal_task_running: false,
            terminal_running_command: None,
            terminal_started_at: None,
            sound_enabled,
            neon_glow: false,
            chromatic_aberration: false,
            holographic_noise: false,
            reduced_motion: true,
            high_contrast: false,
            sftp_dialog: false,
            sftp_host: String::new(),
            sftp_port: "22".to_string(),
            sftp_user: std::env::var("USER").unwrap_or_default(),
            sftp_password: String::new(),
            sftp_connection: None,
            sftp_display_name: String::new(),
            sftp_remote_path: "/".to_string(),
            sftp_remote_entries: Vec::new(),
            sftp_error: None,
            sftp_busy: false,
            sftp_request_id: 0,
            sftp_operation_label: String::new(),
            content_search_in_progress: false,
            background_tx,
            background_rx,
        }
    }

    fn trim_terminal_output(&mut self) {
        if self.terminal_output.len() > 1000 {
            let drain = self.terminal_output.len() - 1000;
            self.terminal_output.drain(..drain);
        }
    }

    fn sync_command_bar_with_current_path(&mut self) {
        if self.command_surface_mode == CommandSurfaceMode::Path {
            self.command_bar_text = self.current_path.to_string_lossy().to_string();
        }
    }

    pub(crate) fn set_command_surface_mode(&mut self, mode: CommandSurfaceMode) {
        if self.command_surface_mode == mode {
            return;
        }

        self.command_surface_mode = mode;
        match mode {
            CommandSurfaceMode::Path => {
                self.command_bar_text = self.current_path.to_string_lossy().to_string();
            }
            CommandSurfaceMode::Protocol => {
                if self.command_bar_text == self.current_path.to_string_lossy() {
                    self.command_bar_text.clear();
                }
            }
        }
        self.launcher_selected = 0;
        self.refresh_launcher_results();
        self.focus_command_bar_next_frame = true;
    }

    fn refresh_local_protocol_manifest(&mut self) {
        self.local_protocol_manifest = launcher::load_local_protocol_manifest(&self.current_path);
    }

    pub(crate) fn save_scene_store(&self) {
        self.scene_store.save();
    }

    pub(crate) fn open_process_matrix(&mut self) {
        self.process_matrix_open = true;
        self.process_matrix_detached = true;
        self.refresh_process_matrix(true);
    }

    pub(crate) fn open_service_deck(&mut self) {
        self.service_deck_open = true;
        self.service_deck_detached = true;
        self.refresh_service_deck(true);
    }

    pub(crate) fn open_log_viewer(&mut self) {
        self.log_viewer_open = true;
        self.log_viewer_detached = true;
        self.refresh_log_viewer(true);
    }

    pub(crate) fn refresh_process_matrix(&mut self, force: bool) {
        if !force && self.process_last_refresh.elapsed().as_secs() < 3 {
            return;
        }

        match crate::integrations::processes::collect_processes(240) {
            Ok(entries) => {
                self.process_entries = entries;
                if self
                    .process_selected_pid
                    .map(|pid| self.process_entries.iter().any(|entry| entry.pid == pid))
                    .unwrap_or(false)
                    == false
                {
                    self.process_selected_pid = self.process_entries.first().map(|entry| entry.pid);
                }
                self.process_last_refresh = Instant::now();
            }
            Err(error) => self.set_error(error),
        }
    }

    pub(crate) fn filtered_process_entries(&self) -> Vec<crate::integrations::processes::ProcessEntry> {
        let query = self.process_filter_text.trim().to_lowercase();
        let mut entries: Vec<_> = self
            .process_entries
            .iter()
            .filter(|entry| {
                query.is_empty()
                    || entry.name.to_lowercase().contains(&query)
                    || entry.command.to_lowercase().contains(&query)
                    || entry.cwd.to_lowercase().contains(&query)
                    || entry.pid.to_string().contains(&query)
            })
            .cloned()
            .collect();

        entries.sort_by(|left, right| match self.process_sort_mode {
            ProcessSortMode::Cpu => right
                .cpu_percent
                .partial_cmp(&left.cpu_percent)
                .unwrap_or(std::cmp::Ordering::Equal),
            ProcessSortMode::Memory => right.memory_kib.cmp(&left.memory_kib),
            ProcessSortMode::Name => left.name.cmp(&right.name),
            ProcessSortMode::Pid => left.pid.cmp(&right.pid),
        });
        entries
    }

    pub(crate) fn terminate_selected_process(&mut self, force: bool) {
        let Some(pid) = self.process_selected_pid else {
            return;
        };
        match crate::integrations::processes::terminate_process(pid, force) {
            Ok(()) => {
                self.status_message = if force {
                    format!("Process terminated: {} (KILL)", pid)
                } else {
                    format!("Process terminated: {} (TERM)", pid)
                };
                self.refresh_process_matrix(true);
            }
            Err(error) => self.set_error(error),
        }
    }

    pub(crate) fn refresh_service_deck(&mut self, force: bool) {
        if !force && self.service_last_refresh.elapsed().as_secs() < 4 {
            return;
        }

        match crate::integrations::services::list_user_services(240) {
            Ok(entries) => {
                self.service_entries = entries;
                if self.service_selected_unit.is_none() {
                    self.service_selected_unit = self
                        .service_entries
                        .first()
                        .map(|service| service.unit.clone());
                }
                if let Some(selected) = self.service_selected_unit.clone() {
                    self.inspect_service_unit(&selected);
                }
                self.service_last_refresh = Instant::now();
            }
            Err(error) => self.set_error(error),
        }
    }

    pub(crate) fn filtered_service_entries(&self) -> Vec<crate::integrations::services::ServiceEntry> {
        let query = self.service_filter_text.trim().to_lowercase();
        self.service_entries
            .iter()
            .filter(|entry| {
                query.is_empty()
                    || entry.unit.to_lowercase().contains(&query)
                    || entry.description.to_lowercase().contains(&query)
                    || entry.active.to_lowercase().contains(&query)
                    || entry.sub.to_lowercase().contains(&query)
            })
            .cloned()
            .collect()
    }

    pub(crate) fn inspect_service_unit(&mut self, unit: &str) {
        match crate::integrations::services::inspect_user_service(unit) {
            Ok(output) => {
                self.service_status_output = output.lines().map(|line| line.to_string()).collect();
            }
            Err(error) => {
                self.service_status_output = vec![format!("[ERR] {}", error)];
            }
        }
    }

    pub(crate) fn control_selected_service(&mut self, action: crate::integrations::services::ServiceAction) {
        let Some(unit) = self.service_selected_unit.clone() else {
            return;
        };
        match crate::integrations::services::control_user_service(&unit, action) {
            Ok(message) => {
                self.status_message = format!("Service deck: {}", message);
                self.refresh_service_deck(true);
            }
            Err(error) => self.set_error(error),
        }
    }

    pub(crate) fn save_service_log_channel(&mut self, unit: &str) {
        let channel = crate::integrations::journald::service_channel(unit);
        if !self.settings.log_channels.iter().any(|existing| existing.id == channel.id) {
            self.settings.log_channels.push(channel.clone());
            self.settings.save();
        }
        self.log_selected_channel_id = Some(channel.id);
        self.open_log_viewer();
    }

    pub(crate) fn remove_log_channel(&mut self, channel_id: &str) {
        if channel_id == "journal.user" || channel_id == "journal.warnings" {
            self.set_error("Default log channels cannot be removed".into());
            return;
        }

        self.settings.log_channels.retain(|channel| channel.id != channel_id);
        self.log_selected_channel_id = self.settings.log_channels.first().map(|channel| channel.id.clone());
        self.settings.save();
        self.refresh_log_viewer(true);
    }

    pub(crate) fn refresh_log_viewer(&mut self, force: bool) {
        if !force && self.log_last_refresh.elapsed().as_secs() < 4 {
            return;
        }

        if self.log_selected_channel_id.is_none() {
            self.log_selected_channel_id = self.settings.log_channels.first().map(|channel| channel.id.clone());
        }

        let Some(channel_id) = self.log_selected_channel_id.clone() else {
            self.log_output = vec!["No log channels configured".to_string()];
            return;
        };

        let Some(channel) = self
            .settings
            .log_channels
            .iter()
            .find(|channel| channel.id == channel_id)
            .cloned()
        else {
            self.log_output = vec!["Selected log channel no longer exists".to_string()];
            return;
        };

        match crate::integrations::journald::read_channel(&channel) {
            Ok(output) => {
                self.log_output = if output.is_empty() {
                    vec!["No journal output for selected channel".to_string()]
                } else {
                    output
                };
                self.log_last_refresh = Instant::now();
            }
            Err(error) => {
                self.log_output = vec![format!("[ERR] {}", error)];
            }
        }
    }

    // ── Signal Deck (Stage 4) ────────────────────────────

    pub(crate) fn open_signal_deck(&mut self) {
        self.signal_deck_open = true;
        self.signal_deck_detached = true;
        self.refresh_audio_snapshot(true);
        self.refresh_power_info(true);
        self.refresh_clipboard(true);
        self.refresh_notifications(true);
        self.brightness_percent = crate::integrations::audio::get_brightness_percent();
    }

    pub(crate) fn refresh_audio_snapshot(&mut self, force: bool) {
        if !force && self.audio_last_refresh.elapsed().as_secs() < 3 {
            return;
        }
        self.audio_snapshot = crate::integrations::audio::collect_audio_snapshot();
        self.audio_volume_slider = self.audio_snapshot.default_sink_volume;
        self.audio_last_refresh = Instant::now();
    }

    pub(crate) fn refresh_power_info(&mut self, force: bool) {
        if !force && self.audio_last_refresh.elapsed().as_secs() < 10 {
            return;
        }
        self.power_info = crate::integrations::audio::collect_power_info();
        self.brightness_percent = crate::integrations::audio::get_brightness_percent();
    }

    pub(crate) fn refresh_clipboard(&mut self, force: bool) {
        if !force && self.clipboard_last_refresh.elapsed().as_secs() < 5 {
            return;
        }
        self.clipboard_entries = crate::integrations::audio::read_clipboard_entries(30);
        self.clipboard_last_refresh = Instant::now();
    }

    pub(crate) fn refresh_notifications(&mut self, force: bool) {
        if !force && self.notification_last_refresh.elapsed().as_secs() < 5 {
            return;
        }
        self.notification_entries = crate::integrations::audio::read_notification_history(40);
        self.notification_last_refresh = Instant::now();
    }

    // ── Network Mesh (Stage 5) ────────────────────────────

    pub(crate) fn open_network_mesh(&mut self) {
        self.network_mesh_open = true;
        self.network_mesh_detached = true;
        self.refresh_network_mesh(true);
    }

    pub(crate) fn refresh_network_mesh(&mut self, force: bool) {
        if !force && self.network_last_refresh.elapsed().as_secs() < 5 {
            return;
        }
        if !self.network_nmcli_available {
            return;
        }
        self.network_interfaces = crate::integrations::network::list_interfaces().unwrap_or_default();
        self.network_wifi_list = crate::integrations::network::list_wifi_networks().unwrap_or_default();
        self.network_vpn_list = crate::integrations::network::list_vpn_connections().unwrap_or_default();

        // Update throughput deltas
        for iface in &self.network_interfaces {
            if iface.state != "connected" {
                continue;
            }
            if let Some(sample) = crate::integrations::network::read_throughput(&iface.device) {
                if let Some(prev) = self.network_throughput_prev.get(&iface.device) {
                    let rx_delta = sample.rx_bytes.saturating_sub(prev.rx_bytes);
                    let tx_delta = sample.tx_bytes.saturating_sub(prev.tx_bytes);
                    let history = self.network_throughput_history
                        .entry(iface.device.clone())
                        .or_insert_with(VecDeque::new);
                    history.push_back((rx_delta, tx_delta));
                    if history.len() > 30 {
                        history.pop_front();
                    }
                }
                self.network_throughput_prev.insert(iface.device.clone(), sample);
            }
        }

        self.network_last_refresh = Instant::now();
    }

    // ── Device Bay (Stage 5) ─────────────────────────────

    pub(crate) fn open_device_bay(&mut self) {
        self.device_bay_open = true;
        self.device_bay_detached = true;
        self.refresh_device_bay(true);
    }

    pub(crate) fn refresh_device_bay(&mut self, force: bool) {
        if !force && self.device_last_refresh.elapsed().as_secs() < 5 {
            return;
        }
        self.device_entries = crate::integrations::devices::list_block_devices().unwrap_or_default();
        self.device_last_refresh = Instant::now();
    }

    // ── Window Bridge (Stage 6) ──────────────────────────

    pub(crate) fn open_window_bridge(&mut self) {
        self.window_bridge_open = true;
        self.window_bridge_detached = true;
        self.refresh_window_bridge(true);
    }

    pub(crate) fn refresh_window_bridge(&mut self, force: bool) {
        if !force && self.wm_last_refresh.elapsed().as_secs() < 3 {
            return;
        }
        let Some(backend) = self.wm_backend else {
            return;
        };
        self.wm_windows = crate::integrations::windows::list_windows(backend).unwrap_or_default();
        self.wm_workspaces = crate::integrations::windows::list_workspaces(backend).unwrap_or_default();
        self.wm_last_refresh = Instant::now();
    }

    fn start_operator_job(&mut self, command: &str, label: &str, cwd: &std::path::Path) -> u64 {
        let job_id = self.next_operator_job_id;
        self.next_operator_job_id += 1;
        let job = OperatorJob {
            id: job_id,
            label: label.to_string(),
            command: command.to_string(),
            cwd: cwd.to_path_buf(),
            started_at: chrono::Local::now().to_rfc3339(),
            finished_at: None,
            duration_ms: None,
            status: OperatorJobState::Running,
            output_tail: Vec::new(),
        };
        self.operator_jobs.insert(0, job);
        if self.operator_jobs.len() > 24 {
            self.operator_jobs.truncate(24);
        }
        self.operator_job_selected_id = Some(job_id);
        job_id
    }

    fn finish_operator_job(
        &mut self,
        job_id: u64,
        status: OperatorJobState,
        lines: Vec<String>,
        duration: Option<Duration>,
    ) {
        if let Some(job) = self.operator_jobs.iter_mut().find(|job| job.id == job_id) {
            job.status = status;
            job.finished_at = Some(chrono::Local::now().to_rfc3339());
            job.duration_ms = duration.map(|duration| duration.as_millis());
            let mut tail: Vec<String> = lines.into_iter().rev().take(60).collect();
            tail.reverse();
            job.output_tail = tail;
        }
    }

    #[allow(dead_code)]
    pub(crate) fn restart_operator_job(&mut self, job_id: u64) {
        let Some(job) = self.operator_jobs.iter().find(|job| job.id == job_id).cloned() else {
            self.set_error(format!("Operator job not found: {}", job_id));
            return;
        };
        self.run_embedded_shell_command_at(job.command, Some(job.label), job.cwd);
    }

    #[allow(dead_code)]
    fn selected_log_channel(&self) -> Option<LogChannel> {
        let channel_id = self.log_selected_channel_id.as_ref()?;
        self.settings
            .log_channels
            .iter()
            .find(|channel| &channel.id == channel_id)
            .cloned()
    }

    pub(crate) fn ordered_scene_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.scene_store.saved_scenes.len()).collect();
        indices.sort_by(|left, right| {
            let left_scene = &self.scene_store.saved_scenes[*left];
            let right_scene = &self.scene_store.saved_scenes[*right];
            right_scene
                .pinned
                .cmp(&left_scene.pinned)
                .then_with(|| right_scene.updated_at.cmp(&left_scene.updated_at))
        });
        indices
    }

    pub(crate) fn open_scene_manager(&mut self) {
        self.scene_manager_open = true;
        if self.scene_manager_selected_id.is_none() {
            self.scene_manager_selected_id = self
                .scene_store
                .saved_scenes
                .first()
                .map(|scene| scene.id.clone());
        }
    }

    pub(crate) fn has_session_resume(&self) -> bool {
        self.scene_store.session_scene.is_some()
    }

    pub(crate) fn boot_scene_slots(&self) -> Vec<RecentSceneRecord> {
        let mut slots = Vec::new();
        let mut seen = HashSet::new();

        for index in self.ordered_scene_indices() {
            let scene = &self.scene_store.saved_scenes[index];
            if scene.pinned && seen.insert(scene.id.clone()) {
                slots.push(RecentSceneRecord::from_scene(scene, scene.updated_at.clone()));
            }
        }

        for record in &self.scene_store.recent_scenes {
            if !seen.insert(record.scene_id.clone()) {
                continue;
            }
            if let Some(scene) = self
                .scene_store
                .saved_scenes
                .iter()
                .find(|scene| scene.id == record.scene_id)
            {
                slots.push(RecentSceneRecord::from_scene(scene, record.last_used_at.clone()));
            }
        }

        for index in self.ordered_scene_indices() {
            let scene = &self.scene_store.saved_scenes[index];
            if seen.insert(scene.id.clone()) {
                slots.push(RecentSceneRecord::from_scene(scene, scene.updated_at.clone()));
            }
        }

        slots.truncate(4);
        slots
    }

    pub(crate) fn queue_boot_resume(&mut self) {
        self.startup_scene_request = Some(StartupSceneRequest::ResumeSession);
        self.boot_complete = true;
    }

    pub(crate) fn queue_boot_scene_restore(&mut self, scene_id: String) {
        self.startup_scene_request = Some(StartupSceneRequest::RestoreScene(scene_id));
        self.boot_complete = true;
    }

    pub(crate) fn queue_boot_fresh_start(&mut self) {
        self.startup_scene_request = Some(StartupSceneRequest::FreshStart);
        self.boot_complete = true;
    }

    fn process_startup_scene_request(&mut self) {
        match self.startup_scene_request.take() {
            Some(StartupSceneRequest::ResumeSession) => {
                if let Some(scene) = self.scene_store.session_scene.clone() {
                    self.restore_scene_snapshot(scene, false, true);
                } else {
                    self.load_current_directory();
                }
            }
            Some(StartupSceneRequest::RestoreScene(scene_id)) => self.restore_scene(&scene_id),
            Some(StartupSceneRequest::FreshStart) => self.load_current_directory(),
            None => {
                if self.entries.is_empty() {
                    self.load_current_directory();
                }
            }
        }
    }

    fn record_recent_scene_use(&mut self, scene: &MissionScene) {
        let record = RecentSceneRecord::from_scene(scene, chrono::Local::now().to_rfc3339());
        self.scene_store
            .recent_scenes
            .retain(|recent| recent.scene_id != record.scene_id);
        self.scene_store.recent_scenes.insert(0, record);
        if self.scene_store.recent_scenes.len() > 8 {
            self.scene_store.recent_scenes.truncate(8);
        }
    }

    fn restore_quick_scene_slot(&mut self, slot_index: usize) {
        if let Some(scene_id) = self
            .boot_scene_slots()
            .get(slot_index)
            .map(|scene| scene.scene_id.clone())
        {
            self.restore_scene(&scene_id);
        }
    }

    fn infer_scene_summary(&self, remote: &MissionSceneRemoteState) -> String {
        let remote_label = if remote.connected {
            "uplink staged"
        } else {
            "local deck"
        };
        format!(
            "{} tabs | {} | {}",
            self.tabs.len(),
            self.current_path.display(),
            remote_label
        )
    }

    fn capture_remote_scene_state(&self) -> MissionSceneRemoteState {
        MissionSceneRemoteState {
            connected: self.sftp_connection.is_some(),
            host: self.sftp_host.clone(),
            port: self.sftp_port.clone(),
            user: self.sftp_user.clone(),
            display_name: self.sftp_display_name.clone(),
            remote_path: self.sftp_remote_path.clone(),
        }
    }

    fn capture_terminal_output_tail(&self) -> Vec<String> {
        let mut output_tail: Vec<String> = self
            .terminal_output
            .iter()
            .rev()
            .take(40)
            .cloned()
            .collect();
        output_tail.reverse();
        output_tail
    }

    fn build_scene_snapshot(
        &self,
        scene_id: String,
        scene_name: String,
        summary: String,
        notes: String,
        pinned: bool,
        tags: Vec<String>,
    ) -> MissionScene {
        let now = chrono::Local::now();
        MissionScene {
            id: scene_id,
            name: scene_name,
            summary,
            notes,
            pinned,
            tags,
            current_path: self.current_path.to_string_lossy().to_string(),
            active_tab: self.active_tab,
            tabs: self
                .tabs
                .iter()
                .map(|tab| MissionSceneTab {
                    path: tab.path.to_string_lossy().to_string(),
                    selected: tab.selected,
                })
                .collect(),
            split: MissionSceneSplitState {
                active: self.split_pane_active,
                path: self.split_pane_path.to_string_lossy().to_string(),
                selected: self.split_pane_selected,
            },
            overlays: MissionSceneOverlayState {
                sidebar_visible: self.sidebar_visible,
                preview_visible: self.preview_visible,
                resource_monitor_visible: self.resource_monitor_visible,
                terminal_panel_visible: self.terminal_panel_visible,
                settings_panel_open: self.settings_panel_open,
                process_matrix_visible: self.process_matrix_open,
                service_deck_visible: self.service_deck_open,
                log_viewer_visible: self.log_viewer_open,
                signal_deck_visible: self.signal_deck_open,
                network_mesh_visible: self.network_mesh_open,
                device_bay_visible: self.device_bay_open,
                window_bridge_visible: self.window_bridge_open,
                data_rain_enabled: self.data_rain_enabled,
            },
            terminal: MissionSceneTerminalState {
                input: self.terminal_input.clone(),
                history: self.terminal_history.iter().rev().take(20).cloned().collect::<Vec<_>>().into_iter().rev().collect(),
                running_command: self.terminal_running_command.clone(),
                output_tail: self.capture_terminal_output_tail(),
            },
            remote: self.capture_remote_scene_state(),
            filter_text: self.filter_text.clone(),
            command_text: self.command_bar_text.clone(),
            command_mode: self.command_surface_mode.id().to_string(),
            theme_id: self.current_theme.id().to_string(),
            view_mode: self.current_view_mode_id().to_string(),
            updated_at: now.to_rfc3339(),
        }
    }

    fn capture_session_scene(&self) -> MissionScene {
        let remote = self.capture_remote_scene_state();
        self.build_scene_snapshot(
            "session.resume".to_string(),
            "Last Session".to_string(),
            format!("{} // autosaved session deck", self.infer_scene_summary(&remote)),
            "Automatically maintained resume point for the active command deck.".to_string(),
            false,
            vec!["session".to_string(), "recent".to_string()],
        )
    }

    fn sync_session_scene(&mut self) {
        let snapshot = self.capture_session_scene();
        let changed = self
            .scene_store
            .session_scene
            .as_ref()
            .map(|scene| scene != &snapshot)
            .unwrap_or(true);
        if changed {
            self.scene_store.session_scene = Some(snapshot);
            self.save_scene_store();
        }
    }

    pub(crate) fn toggle_scene_pin(&mut self, scene_id: &str) {
        if let Some(scene) = self
            .scene_store
            .saved_scenes
            .iter_mut()
            .find(|scene| scene.id == scene_id)
        {
            scene.pinned = !scene.pinned;
            scene.updated_at = chrono::Local::now().to_rfc3339();
            if let Some(recent) = self
                .scene_store
                .recent_scenes
                .iter_mut()
                .find(|recent| recent.scene_id == scene_id)
            {
                recent.pinned = scene.pinned;
            }
            self.save_scene_store();
        }
    }

    pub(crate) fn delete_scene(&mut self, scene_id: &str) {
        self.scene_store.saved_scenes.retain(|scene| scene.id != scene_id);
        self.scene_store
            .recent_scenes
            .retain(|scene| scene.scene_id != scene_id);
        self.scene_manager_selected_id = self
            .scene_store
            .saved_scenes
            .first()
            .map(|scene| scene.id.clone());
        self.save_scene_store();
        self.refresh_launcher_results();
    }

    pub(crate) fn refresh_launcher_results(&mut self) {
        if self.command_surface_mode != CommandSurfaceMode::Protocol {
            self.launcher_results.clear();
            self.launcher_selected = 0;
            return;
        }

        let mut results = launcher::query_entries(&self.command_bar_text);
        let mut registry = launcher::builtin_entries(
            &self.current_path,
            &self.scene_store.saved_scenes,
            self.fzf_available,
        );

        let selected_paths = self.get_selected_paths();
        let selected_target = selected_paths.first().map(PathBuf::as_path);

        registry.extend(launcher::protocol_entries(
            &self.settings.protocols,
            "GLOBAL",
            "global",
        ));

        if let Some(local_manifest) = &self.local_protocol_manifest {
            registry.extend(launcher::protocol_entries(
                &local_manifest.protocols,
                &format!("LOCAL // {}", local_manifest.name),
                "local",
            ));
        }

        registry.extend(launcher::app_catalog_entries(
            &self.settings,
            &self.current_path,
            selected_target,
        ));
        registry.extend(launcher::file_tool_entries(&selected_paths));
        registry.extend(launcher::remote_entries(&launcher::RemoteProviderState {
            connected: self.sftp_connection.is_some(),
            busy: self.sftp_busy,
            display_name: &self.sftp_display_name,
            remote_path: &self.sftp_remote_path,
            uploadable_count: selected_paths.iter().filter(|path| path.is_file()).count(),
        }));

        for bookmark in &self.bookmarks {
            let label = bookmark
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| bookmark.to_string_lossy().to_string());
            registry.push(launcher::path_entry(
                format!("BOOKMARK // {}", label),
                bookmark.to_string_lossy().to_string(),
                "BOOKMARKS",
                bookmark.clone(),
            ));
        }

        let query = self.command_bar_text.trim().to_lowercase();
        if !query.is_empty() {
            let mut match_count = 0;
            for entry in &self.entries {
                if entry.name.to_lowercase().contains(&query) {
                    let subtitle = if entry.is_dir {
                        format!("Sector // {}", entry.path.to_string_lossy())
                    } else {
                        format!("Construct // {}", entry.path.to_string_lossy())
                    };
                    registry.push(launcher::path_entry(
                        entry.name.clone(),
                        subtitle,
                        "SECTORS",
                        entry.path.clone(),
                    ));
                    match_count += 1;
                    if match_count >= 10 {
                        break;
                    }
                }
            }
        }

        results.extend(launcher::filter_entries(&registry, &self.command_bar_text, 12));
        self.launcher_results = results;
        if self.launcher_selected >= self.launcher_results.len() {
            self.launcher_selected = 0;
        }
    }

    fn current_view_mode_id(&self) -> &'static str {
        match self.view_mode {
            ViewMode::List => "list",
            ViewMode::Grid => "grid",
            ViewMode::HexGrid => "hex_grid",
            ViewMode::Hex => "hex",
        }
    }

    fn view_mode_from_id(id: &str) -> ViewMode {
        match id {
            "grid" => ViewMode::Grid,
            "hex_grid" => ViewMode::HexGrid,
            "hex" => ViewMode::Hex,
            _ => ViewMode::List,
        }
    }

    fn resolve_scene_path(candidate: &str, fallback: &PathBuf) -> PathBuf {
        let path = PathBuf::from(candidate);
        if path.is_dir() {
            path
        } else {
            fallback.clone()
        }
    }

    fn capture_current_scene(&self, name: String) -> MissionScene {
        let scene_name = name.trim().to_string();
        let scene_id = format!(
            "{}-{}",
            slugify_scene_name(&scene_name),
            chrono::Local::now().format("%Y%m%d%H%M%S")
        );
        let remote = self.capture_remote_scene_state();
        self.build_scene_snapshot(
            scene_id,
            scene_name,
            self.infer_scene_summary(&remote),
            String::new(),
            false,
            Vec::new(),
        )
    }

    pub(crate) fn save_current_scene(&mut self, explicit_name: Option<String>) {
        let default_name = format!("MISSION {:02}", self.scene_store.saved_scenes.len() + 1);
        let scene_name = explicit_name
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty())
            .unwrap_or(default_name);

        let scene = self.capture_current_scene(scene_name);
        self.scene_store.saved_scenes.insert(0, scene.clone());
        if self.scene_store.saved_scenes.len() > 16 {
            self.scene_store.saved_scenes.truncate(16);
        }
        self.record_recent_scene_use(&scene);
        self.scene_manager_selected_id = Some(scene.id.clone());
        self.save_scene_store();
        self.status_message = format!("Mission scene captured: {}", scene.display_label());
        self.refresh_launcher_results();
        self.trigger_glitch();
    }

    fn restore_scene_snapshot(&mut self, scene: MissionScene, record_recent: bool, is_session: bool) {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let mut restored_tabs: Vec<Tab> = scene
            .tabs
            .iter()
            .map(|tab| Tab {
                path: Self::resolve_scene_path(&tab.path, &home),
                selected: tab.selected,
            })
            .collect();
        if restored_tabs.is_empty() {
            restored_tabs.push(Tab {
                path: Self::resolve_scene_path(&scene.current_path, &home),
                selected: None,
            });
        }

        self.tabs = restored_tabs;
        self.active_tab = scene.active_tab.min(self.tabs.len().saturating_sub(1));
        self.current_path = Self::resolve_scene_path(&scene.current_path, &self.tabs[self.active_tab].path);
        self.selected = self.tabs[self.active_tab].selected;
        self.split_pane_active = scene.split.active;
        self.split_pane_path = Self::resolve_scene_path(&scene.split.path, &self.current_path);
        self.split_pane_selected = scene.split.selected;
        self.sidebar_visible = scene.overlays.sidebar_visible;
        self.preview_visible = scene.overlays.preview_visible;
        self.resource_monitor_visible = scene.overlays.resource_monitor_visible;
        self.terminal_panel_visible = scene.overlays.terminal_panel_visible;
        self.settings_panel_open = scene.overlays.settings_panel_open;
        self.process_matrix_open = scene.overlays.process_matrix_visible;
        self.service_deck_open = scene.overlays.service_deck_visible;
        self.log_viewer_open = scene.overlays.log_viewer_visible;
        self.signal_deck_open = scene.overlays.signal_deck_visible;
        self.network_mesh_open = scene.overlays.network_mesh_visible;
        self.device_bay_open = scene.overlays.device_bay_visible;
        self.window_bridge_open = scene.overlays.window_bridge_visible;
        self.data_rain_enabled = scene.overlays.data_rain_enabled;
        self.terminal_input = scene.terminal.input.clone();
        self.terminal_history = scene.terminal.history.clone();
        self.terminal_output = scene.terminal.output_tail.clone();
        self.terminal_running_command = scene.terminal.running_command.clone();
        self.terminal_task_running = false;
        self.terminal_started_at = None;
        self.sftp_host = scene.remote.host.clone();
        self.sftp_port = scene.remote.port.clone();
        self.sftp_user = scene.remote.user.clone();
        self.sftp_display_name = scene.remote.display_name.clone();
        self.sftp_remote_path = scene.remote.remote_path.clone();
        self.sftp_connection = None;
        self.sftp_dialog = scene.remote.connected;
        self.sftp_error = None;
        self.filter_text = scene.filter_text.clone();
        self.current_theme = CyberTheme::from_id(&scene.theme_id);
        self.settings.theme = scene.theme_id.clone();
        self.theme_applied = false;
        self.view_mode = Self::view_mode_from_id(&scene.view_mode);
        self.load_current_directory();
        self.refresh_local_protocol_manifest();
        self.filter_text = scene.filter_text.clone();

        match scene.command_mode.as_str() {
            "protocol" => {
                self.command_surface_mode = CommandSurfaceMode::Protocol;
                self.command_bar_text = scene.command_text.clone();
                self.refresh_launcher_results();
            }
            _ => {
                self.command_surface_mode = CommandSurfaceMode::Path;
                self.sync_command_bar_with_current_path();
            }
        }

        self.status_message = if scene.remote.connected {
            format!(
                "{}: {} // uplink requires re-auth",
                if is_session {
                    "Session deck restored"
                } else {
                    "Mission scene restored"
                },
                scene.name
            )
        } else {
            format!(
                "{}: {}",
                if is_session {
                    "Session deck restored"
                } else {
                    "Mission scene restored"
                },
                scene.name
            )
        };
        if record_recent {
            self.record_recent_scene_use(&scene);
            self.save_scene_store();
        }
        self.scene_manager_selected_id = Some(scene.id.clone());
        self.trigger_glitch();
    }

    pub(crate) fn restore_scene(&mut self, scene_id: &str) {
        let Some(scene) = self
            .scene_store
            .saved_scenes
            .iter()
            .find(|scene| scene.id == scene_id)
            .cloned()
        else {
            self.set_error(format!("Mission scene not found: {}", scene_id));
            return;
        };

        self.restore_scene_snapshot(scene, true, false);
    }

    pub(crate) fn execute_launcher_action(&mut self, action: LauncherAction) {
        match action {
            LauncherAction::OpenTerminalHere => self.open_terminal_here(),
            LauncherAction::ToggleSidebar => self.sidebar_visible = !self.sidebar_visible,
            LauncherAction::TogglePreview => self.preview_visible = !self.preview_visible,
            LauncherAction::ToggleHidden => {
                self.show_hidden = !self.show_hidden;
                self.load_current_directory();
            }
            LauncherAction::ToggleResourceMonitor => {
                self.resource_monitor_visible = !self.resource_monitor_visible;
            }
            LauncherAction::ToggleEmbeddedTerminal => {
                self.terminal_panel_visible = !self.terminal_panel_visible;
            }
            LauncherAction::OpenSettings => {
                self.settings_panel_open = true;
                self.ui_scale_preview = self.settings.font_size;
            }
            LauncherAction::OpenSceneManager => self.open_scene_manager(),
            LauncherAction::SaveMissionScene => self.save_current_scene(None),
            LauncherAction::RestoreMissionScene(scene_id) => self.restore_scene(&scene_id),
            LauncherAction::StartDeepScan(query) => {
                self.content_search_dialog = true;
                self.content_search_query = query;
                self.content_search_results.clear();
                self.start_content_search();
            }
            LauncherAction::TriggerFzf => self.fzf_interactive(),
            LauncherAction::OpenSftpDialog => {
                self.sftp_dialog = true;
            }
            LauncherAction::RefreshRemoteNode => {
                let path = self.sftp_remote_path.clone();
                self.start_sftp_list_directory(path);
                self.sftp_dialog = true;
            }
            LauncherAction::DisconnectRemoteNode => self.disconnect_sftp(),
            LauncherAction::UploadSelectedToRemote => self.start_sftp_upload_selected(),
            LauncherAction::TailPath(path) => {
                let command = format!("tail -n 80 -- {}", shell_quote(&path.to_string_lossy()));
                self.run_embedded_shell_command(
                    command,
                    Some(format!("tail {}", path.file_name().unwrap_or_default().to_string_lossy())),
                );
            }
            LauncherAction::LaunchExternalProgram {
                label,
                program,
                args,
                cwd,
            } => {
                let mut command = std::process::Command::new(&program);
                command.args(&args);
                if let Some(cwd) = cwd {
                    command.current_dir(cwd);
                }
                match command.spawn() {
                    Ok(_) => {
                        self.status_message = format!("Application launched: {}", label);
                    }
                    Err(error) => {
                        self.set_error(format!("Application launch failed [{}]: {}", label, error));
                    }
                }
            }
            LauncherAction::RunProtocolCommand {
                label,
                command,
                run_in_terminal,
            } => {
                if run_in_terminal {
                    self.run_embedded_shell_command(command, Some(label));
                } else {
                    match std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&command)
                        .current_dir(&self.current_path)
                        .spawn()
                    {
                        Ok(_) => {
                            self.status_message = format!("Protocol engaged: {}", label);
                        }
                        Err(error) => {
                            self.set_error(format!("Protocol launch failed [{}]: {}", label, error));
                        }
                    }
                }
            }
            LauncherAction::OpenPath(path) => {
                if path.is_dir() {
                    self.navigate_to(path);
                } else if path.is_file() {
                    self.open_file(&path);
                }
            }
            LauncherAction::OpenNetworkMesh => self.open_network_mesh(),
            LauncherAction::OpenDeviceBay => self.open_device_bay(),
            LauncherAction::OpenWindowBridge => self.open_window_bridge(),
        }
    }

    fn execute_protocol_launcher(&mut self) {
        self.refresh_launcher_results();
        if let Some(entry) = self.launcher_results.get(self.launcher_selected).cloned() {
            self.execute_launcher_action(entry.action);
        } else {
            self.status_message = "No protocol matched the current query".into();
        }
    }

    fn poll_background_tasks(&mut self) {
        while let Ok(result) = self.background_rx.try_recv() {
            match result {
                BackgroundTaskResult::TerminalFinished {
                    job_id,
                    lines,
                    duration,
                    success,
                } => {
                    self.terminal_task_running = false;
                    self.terminal_started_at = None;
                    let command = self
                        .terminal_running_command
                        .take()
                        .unwrap_or_else(|| "command".to_string());
                    self.terminal_output.extend(lines);
                    self.terminal_output.push(format!(
                        "[SYS] {} completed in {} ms",
                        command,
                        duration.as_millis()
                    ));
                    self.finish_operator_job(
                        job_id,
                        if success {
                            OperatorJobState::Completed
                        } else {
                            OperatorJobState::Failed
                        },
                        self.terminal_output.clone(),
                        Some(duration),
                    );
                    self.trim_terminal_output();
                    self.load_current_directory();
                }
                BackgroundTaskResult::TerminalSpawnFailed { job_id, message } => {
                    self.terminal_task_running = false;
                    self.terminal_started_at = None;
                    self.terminal_running_command = None;
                    self.terminal_output.push(format!("[ERR] {}", message));
                    self.finish_operator_job(
                        job_id,
                        OperatorJobState::Failed,
                        self.terminal_output.clone(),
                        None,
                    );
                    self.trim_terminal_output();
                }
                BackgroundTaskResult::ContentSearchFinished {
                    request_id,
                    query,
                    results,
                } => {
                    if request_id != self.content_search_request_id {
                        continue;
                    }
                    self.content_search_in_progress = false;
                    self.content_search_results = results;
                    self.content_search_active_query = query.clone();
                    self.status_message = format!(
                        "Deep scan: {} matches for \"{}\"",
                        self.content_search_results.len(),
                        query
                    );
                }
                BackgroundTaskResult::SftpConnected {
                    request_id,
                    connection,
                    display_name,
                    entries,
                    clear_password,
                } => {
                    if request_id != self.sftp_request_id {
                        continue;
                    }
                    self.sftp_busy = false;
                    self.sftp_connection = Some(connection);
                    self.sftp_display_name = display_name;
                    self.sftp_remote_entries = entries;
                    self.sftp_error = None;
                    if clear_password {
                        self.sftp_password.clear();
                    }
                    self.status_message = format!("Uplink established: {}", self.sftp_display_name);
                    self.sftp_operation_label.clear();
                    self.trigger_glitch();
                }
                BackgroundTaskResult::SftpListed {
                    request_id,
                    path,
                    entries,
                } => {
                    if request_id != self.sftp_request_id {
                        continue;
                    }
                    self.sftp_busy = false;
                    self.sftp_remote_path = path;
                    self.sftp_remote_entries = entries;
                    self.sftp_error = None;
                    self.sftp_operation_label.clear();
                }
                BackgroundTaskResult::SftpDownloaded {
                    request_id,
                    file_name,
                } => {
                    if request_id != self.sftp_request_id {
                        continue;
                    }
                    self.sftp_busy = false;
                    self.sftp_error = None;
                    self.status_message = format!("Downloaded: {}", file_name);
                    self.sftp_operation_label.clear();
                }
                BackgroundTaskResult::SftpUploaded {
                    request_id,
                    file_names,
                } => {
                    if request_id != self.sftp_request_id {
                        continue;
                    }
                    self.sftp_busy = false;
                    self.sftp_error = None;
                    self.status_message = format!(
                        "Transferred {} construct(s) to uplink",
                        file_names.len()
                    );
                    self.sftp_operation_label.clear();
                }
                BackgroundTaskResult::SftpFailed { request_id, message } => {
                    if request_id != self.sftp_request_id {
                        continue;
                    }
                    self.sftp_busy = false;
                    self.sftp_operation_label.clear();
                    self.status_message = format!("Remote operation failed: {}", message);
                    self.sftp_error = Some(message);
                }
            }
        }
    }

    fn start_content_search(&mut self) {
        let query = self.content_search_query.trim().to_string();
        if query.is_empty() || self.content_search_in_progress {
            return;
        }

        let tx = self.background_tx.clone();
        let dir = self.current_path.clone();
        self.content_search_request_id += 1;
        let request_id = self.content_search_request_id;
        self.content_search_in_progress = true;
        self.content_search_active_query = query.clone();
        self.status_message = format!("Deep scan running for \"{}\"...", query);

        std::thread::spawn(move || {
            let results = crate::filesystem::search_content(&dir, &query, 200);
            let _ = tx.send(BackgroundTaskResult::ContentSearchFinished {
                request_id,
                query,
                results,
            });
        });
    }

    fn next_sftp_request_id(&mut self) -> u64 {
        self.sftp_request_id += 1;
        self.sftp_request_id
    }

    fn start_sftp_connect(&mut self, use_password: bool) {
        if self.sftp_busy {
            return;
        }

        let host = self.sftp_host.trim().to_string();
        let user = self.sftp_user.trim().to_string();
        if host.is_empty() || user.is_empty() {
            self.sftp_error = Some("Host and user are required".to_string());
            return;
        }

        let port = self.sftp_port.parse().unwrap_or(22);
        let remote_path = self.sftp_remote_path.clone();
        let password = self.sftp_password.clone();
        let tx = self.background_tx.clone();
        let request_id = self.next_sftp_request_id();
        self.sftp_busy = true;
        self.sftp_error = None;
        self.sftp_operation_label = format!("Connecting to {}@{}", user, host);
        self.status_message = format!("Connecting to {}@{}...", user, host);

        std::thread::spawn(move || {
            let connect_result = if use_password {
                crate::integrations::sftp::SftpConnection::connect_with_password(
                    &host,
                    port,
                    &user,
                    &password,
                )
            } else {
                crate::integrations::sftp::SftpConnection::connect(&host, port, &user)
            };

            match connect_result {
                Ok(conn) => {
                    let display_name = conn.display_name();
                    match conn.list_directory(&remote_path) {
                        Ok(entries) => {
                            let _ = tx.send(BackgroundTaskResult::SftpConnected {
                                request_id,
                                connection: Arc::new(Mutex::new(conn)),
                                display_name,
                                entries,
                                clear_password: use_password,
                            });
                        }
                        Err(message) => {
                            let _ = tx.send(BackgroundTaskResult::SftpFailed {
                                request_id,
                                message,
                            });
                        }
                    }
                }
                Err(message) => {
                    let _ = tx.send(BackgroundTaskResult::SftpFailed {
                        request_id,
                        message,
                    });
                }
            }
        });
    }

    fn start_sftp_list_directory(&mut self, path: String) {
        if self.sftp_busy {
            return;
        }
        let Some(conn) = self.sftp_connection.as_ref().map(Arc::clone) else {
            return;
        };

        let tx = self.background_tx.clone();
        let request_id = self.next_sftp_request_id();
        self.sftp_busy = true;
        self.sftp_error = None;
        self.sftp_remote_path = path.clone();
        self.sftp_operation_label = format!("Listing {}", path);

        std::thread::spawn(move || {
            let result = match conn.lock() {
                Ok(conn) => conn.list_directory(&path),
                Err(_) => Err("SFTP connection lock failed".to_string()),
            };

            match result {
                Ok(entries) => {
                    let _ = tx.send(BackgroundTaskResult::SftpListed {
                        request_id,
                        path,
                        entries,
                    });
                }
                Err(message) => {
                    let _ = tx.send(BackgroundTaskResult::SftpFailed {
                        request_id,
                        message,
                    });
                }
            }
        });
    }

    fn start_sftp_download(&mut self, remote: String, local_path: PathBuf, file_name: String) {
        if self.sftp_busy {
            return;
        }
        let Some(conn) = self.sftp_connection.as_ref().map(Arc::clone) else {
            return;
        };

        let tx = self.background_tx.clone();
        let request_id = self.next_sftp_request_id();
        self.sftp_busy = true;
        self.sftp_error = None;
        self.sftp_operation_label = format!("Downloading {}", file_name);
        self.status_message = format!("Downloading {}...", file_name);

        std::thread::spawn(move || {
            let result = match conn.lock() {
                Ok(conn) => conn.download_file(&remote, &local_path),
                Err(_) => Err("SFTP connection lock failed".to_string()),
            };

            match result {
                Ok(()) => {
                    let _ = tx.send(BackgroundTaskResult::SftpDownloaded {
                        request_id,
                        file_name,
                    });
                }
                Err(message) => {
                    let _ = tx.send(BackgroundTaskResult::SftpFailed {
                        request_id,
                        message,
                    });
                }
            }
        });
    }

    fn start_sftp_upload_selected(&mut self) {
        if self.sftp_busy {
            return;
        }
        let Some(conn) = self.sftp_connection.as_ref().map(Arc::clone) else {
            self.set_error("No active uplink available for transfer".into());
            return;
        };

        let local_paths: Vec<PathBuf> = self
            .get_selected_paths()
            .into_iter()
            .filter(|path| path.is_file())
            .collect();
        if local_paths.is_empty() {
            self.set_error("Select one or more local files before starting transfer".into());
            return;
        }

        let remote_dir = self.sftp_remote_path.clone();
        let tx = self.background_tx.clone();
        let request_id = self.next_sftp_request_id();
        self.sftp_busy = true;
        self.sftp_error = None;
        self.sftp_operation_label = format!("Uploading {} construct(s)", local_paths.len());
        self.status_message = format!("Transferring {} construct(s)...", local_paths.len());

        std::thread::spawn(move || {
            let result = match conn.lock() {
                Ok(conn) => {
                    let mut uploaded = Vec::new();
                    let mut failure = None;
                    for local_path in &local_paths {
                        let file_name = local_path
                            .file_name()
                            .map(|name| name.to_string_lossy().to_string())
                            .unwrap_or_else(|| "construct".to_string());
                        let remote_path = join_remote_path(&remote_dir, &file_name);
                        if let Err(message) = conn.upload_file(local_path, &remote_path) {
                            failure = Some(message);
                            break;
                        }
                        uploaded.push(file_name);
                    }
                    if let Some(message) = failure {
                        Err(message)
                    } else {
                        Ok(uploaded)
                    }
                }
                Err(_) => Err("SFTP connection lock failed".to_string()),
            };

            match result {
                Ok(file_names) => {
                    let _ = tx.send(BackgroundTaskResult::SftpUploaded {
                        request_id,
                        file_names,
                    });
                }
                Err(message) => {
                    let _ = tx.send(BackgroundTaskResult::SftpFailed {
                        request_id,
                        message,
                    });
                }
            }
        });
    }

    pub(crate) fn disconnect_sftp(&mut self) {
        self.sftp_connection = None;
        self.sftp_display_name.clear();
        self.sftp_remote_entries.clear();
        self.sftp_remote_path = "/".to_string();
        self.sftp_error = None;
        self.sftp_busy = false;
        self.sftp_operation_label.clear();
        self.sftp_request_id += 1;
        self.status_message = "Uplink severed".to_string();
        self.refresh_launcher_results();
    }

    // ── Navigation ────────────────────────────────────────────

    pub(crate) fn navigate_to(&mut self, path: PathBuf) {
        if !path.is_dir() {
            self.set_error(format!("Cannot navigate: not a sector ({})", path.display()));
            return;
        }

        self.play_sound(SoundType::Navigate);

        self.current_path = path.clone();
        self.selected = None;
        self.multi_selected.clear();
        self.context_menu_open = false;

        self.history.truncate(self.history_pos + 1);
        self.history.push(path);
        self.history_pos = self.history.len() - 1;

        self.load_current_directory();
        self.sync_command_bar_with_current_path();
        self.trigger_glitch();

        // Update active tab
        if self.active_tab < self.tabs.len() {
            self.tabs[self.active_tab].path = self.current_path.clone();
        }
    }

    pub(crate) fn go_back(&mut self) {
        if self.history_pos > 0 {
            self.history_pos -= 1;
            let path = self.history[self.history_pos].clone();
            self.current_path = path;
            self.selected = None;
            self.multi_selected.clear();
            self.load_current_directory();
            self.trigger_glitch();
        }
    }

    pub(crate) fn go_forward(&mut self) {
        if self.history_pos + 1 < self.history.len() {
            self.history_pos += 1;
            let path = self.history[self.history_pos].clone();
            self.current_path = path;
            self.selected = None;
            self.multi_selected.clear();
            self.load_current_directory();
        }
    }

    pub(crate) fn go_up(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            let parent = parent.to_path_buf();
            self.navigate_to(parent);
        }
    }

    pub(crate) fn load_current_directory(&mut self) {
        self.refresh_local_protocol_manifest();
        match filesystem::read_directory(&self.current_path, self.show_hidden) {
            Ok(mut entries) => {
                filesystem::sort_entries(&mut entries, self.sort_column, self.sort_ascending);
                self.entries = entries;
                self.status_message = format!(
                    "SECTOR LOADED // {} constructs indexed",
                    self.entries.len()
                );
            }
            Err(e) => {
                self.set_error(format!("Access denied: {}", e));
                self.entries.clear();
            }
        }
        self.filter_text.clear();
        self.thumbnail_cache.clear();
        self.thumbnail_failed.clear();
        self.refresh_launcher_results();
    }

    pub(crate) fn sort_entries(&mut self) {
        filesystem::sort_entries(&mut self.entries, self.sort_column, self.sort_ascending);
    }

    // ── Entry Operations ──────────────────────────────────────

    pub(crate) fn open_entry(&mut self, index: usize) {
        if let Some(entry) = self.entries.get(index).cloned() {
            if entry.is_dir {
                self.navigate_to(entry.path);
            } else {
                self.open_file(&entry.path);
            }
        }
    }

    /// Open a file with the configured opener (if any) or the system default.
    pub(crate) fn open_file(&mut self, path: &std::path::Path) {
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        if let Some(app) = self.settings.opener_for_ext(&ext).cloned() {
            if let Err(e) = std::process::Command::new(&app).arg(path).spawn() {
                self.set_error(format!("Subsystem link failed [{}]: {}", app, e));
            }
        } else if let Err(e) = open::that(path) {
            self.set_error(format!("Access denied: {}", e));
        }
    }

    /// Open a specific file with a chosen application.
    pub(crate) fn open_file_with(&mut self, path: &std::path::Path, app: &str) {
        if let Err(e) = std::process::Command::new(app).arg(path).spawn() {
            self.set_error(format!("Subsystem link failed [{}]: {}", app, e));
        }
    }

    /// Launch the configured (or auto-detected) terminal in a directory.
    pub(crate) fn open_terminal_here(&mut self) {
        let dir = self.current_path.clone();
        if let Some(term_path) = self.settings.resolved_terminal_path() {
            let result = std::process::Command::new(&term_path)
                .current_dir(&dir)
                .spawn();
            if let Err(e) = result {
                self.set_error(format!("Jack-in failed [{}]: {}", term_path, e));
            }
        } else {
            self.set_error(
                "No terminal subsystem detected — use an absolute executable path in SYSTEM CONFIGURATION".into(),
            );
        }
    }

    pub(crate) fn delete_selected(&mut self) {
        let indices: Vec<usize> = if self.multi_selected.is_empty() {
            self.selected.into_iter().collect()
        } else {
            self.multi_selected.iter().copied().collect()
        };

        let mut errors = Vec::new();
        for &idx in indices.iter().rev() {
            if let Some(entry) = self.entries.get(idx) {
                let original_path = entry.path.clone();
                match filesystem::delete_to_trash(&entry.path) {
                    Ok(trash_name) => {
                        self.record_undo(UndoAction::Delete {
                            original_path,
                            trash_name,
                        });
                        self.redo_stack.clear();
                    }
                    Err(e) => {
                        errors.push(format!("{}: {}", entry.name, e));
                    }
                }
            }
        }

        if !errors.is_empty() {
            self.set_error(format!("Quarantine errors: {}", errors.join("; ")));
        } else {
            self.status_message = "Constructs quarantined".into();
            self.play_sound(SoundType::Delete);
        }

        self.selected = None;
        self.multi_selected.clear();
        self.load_current_directory();
    }

    pub(crate) fn copy_selected(&mut self) {
        let paths = self.get_selected_paths();
        if !paths.is_empty() {
            self.clipboard_paths = paths;
            self.clipboard_op = Some(ClipboardOp::Copy);
            self.status_message = format!(
                "Copied {} construct(s) to buffer",
                self.clipboard_paths.len()
            );
            self.sync_to_system_clipboard();
        }
    }

    pub(crate) fn cut_selected(&mut self) {
        let paths = self.get_selected_paths();
        if !paths.is_empty() {
            self.clipboard_paths = paths;
            self.clipboard_op = Some(ClipboardOp::Cut);
            self.status_message = format!(
                "Cut {} construct(s) to buffer",
                self.clipboard_paths.len()
            );
            self.sync_to_system_clipboard();
        }
    }

    pub(crate) fn paste(&mut self) {
        let op = match self.clipboard_op {
            Some(op) => op,
            None => return,
        };

        let paths = self.clipboard_paths.clone();
        let dest = self.current_path.clone();
        let mut errors = Vec::new();
        let mut copied_to = Vec::new();
        let mut move_sources = Vec::new();
        let mut move_dests = Vec::new();

        for src in &paths {
            let _dest_path = dest.join(src.file_name().unwrap_or_default());
            let result = match op {
                ClipboardOp::Copy => filesystem::copy_file(src, &dest).map(|p| {
                    copied_to.push(p);
                }),
                ClipboardOp::Cut => filesystem::move_file(src, &dest).map(|p| {
                    move_sources.push(src.clone());
                    move_dests.push(p);
                }),
            };
            if let Err(e) = result {
                errors.push(format!("{}", e));
            }
        }

        // Record undo action
        match op {
            ClipboardOp::Copy if !copied_to.is_empty() => {
                self.record_undo(UndoAction::Copy { copied_to });
                self.redo_stack.clear();
            }
            ClipboardOp::Cut if !move_sources.is_empty() => {
                self.record_undo(UndoAction::Move {
                    sources: move_sources,
                    destinations: move_dests,
                });
                self.redo_stack.clear();
            }
            _ => {}
        }

        if op == ClipboardOp::Cut {
            self.clipboard_paths.clear();
            self.clipboard_op = None;
        }

        if !errors.is_empty() {
            self.set_error(format!("Transfer errors: {}", errors.join("; ")));
        } else {
            self.status_message = "Data transfer complete".into();
            self.play_sound(SoundType::CopyComplete);
        }

        self.load_current_directory();
    }

    pub(crate) fn execute_command(&mut self) {
        if self.command_surface_mode == CommandSurfaceMode::Protocol {
            self.execute_protocol_launcher();
            return;
        }

        let text = self.command_bar_text.trim().to_string();
        if text.is_empty() {
            return;
        }

        let path = PathBuf::from(&text);
        if path.is_dir() {
            self.navigate_to(path);
        } else if path.is_file() {
            self.open_file(&path);
        } else if self.fzf_available {
            // Use fzf for fuzzy search
            self.fzf_search(&text);
        } else {
            self.status_message = format!("Scanning for \"{}\"...", text);
            let query = text.to_lowercase();
            let all_entries =
                filesystem::read_directory(&self.current_path, self.show_hidden)
                    .unwrap_or_default();
            self.entries = all_entries
                .into_iter()
                .filter(|e| e.name.to_lowercase().contains(&query))
                .collect();
            self.status_message = format!(
                "Neural scan complete // {} matches for \"{}\"",
                self.entries.len(),
                text
            );
        }
    }

    pub(crate) fn create_new_folder(&mut self) {
        let name = self.new_folder_name.trim().to_string();
        if name.is_empty() {
            return;
        }

        match filesystem::create_directory(&self.current_path, &name) {
            Ok(path) => {
                self.status_message = format!("Sector \"{}\" initialized", name);
                self.record_undo(UndoAction::Create {
                    path,
                    kind: EntryKind::Directory,
                });
                self.redo_stack.clear();
                self.load_current_directory();
                self.play_sound(SoundType::Navigate);
            }
            Err(e) => {
                self.set_error(format!("Failed to init sector: {}", e));
            }
        }

        self.new_folder_dialog = false;
        self.new_folder_name.clear();
    }

    pub(crate) fn create_new_file(&mut self) {
        let name = self.new_file_name.trim().to_string();
        if name.is_empty() {
            return;
        }

        match filesystem::create_file(&self.current_path, &name) {
            Ok(path) => {
                self.status_message = format!("Construct \"{}\" initialized", name);
                self.record_undo(UndoAction::Create {
                    path,
                    kind: EntryKind::File,
                });
                self.redo_stack.clear();
                self.load_current_directory();
                self.play_sound(SoundType::Navigate);
            }
            Err(e) => {
                self.set_error(format!("Failed to init construct: {}", e));
            }
        }

        self.new_file_dialog = false;
        self.new_file_name.clear();
    }

    pub(crate) fn request_delete(&mut self) {
        let indices: Vec<usize> = if self.multi_selected.is_empty() {
            self.selected.into_iter().collect()
        } else {
            self.multi_selected.iter().copied().collect()
        };
        if indices.is_empty() {
            return;
        }

        if self.settings.confirm_delete {
            self.delete_pending_indices = indices;
            self.confirm_delete_dialog = true;
        } else {
            self.delete_selected();
        }
    }

    pub(crate) fn confirm_delete_execute(&mut self) {
        let indices = std::mem::take(&mut self.delete_pending_indices);
        let mut errors = Vec::new();
        for &idx in indices.iter().rev() {
            if let Some(entry) = self.entries.get(idx) {
                let original_path = entry.path.clone();
                match filesystem::delete_to_trash(&entry.path) {
                    Ok(trash_name) => {
                        self.record_undo(UndoAction::Delete {
                            original_path,
                            trash_name,
                        });
                        self.redo_stack.clear();
                    }
                    Err(e) => {
                        errors.push(format!("{}: {}", entry.name, e));
                    }
                }
            }
        }

        if !errors.is_empty() {
            self.set_error(format!("Quarantine errors: {}", errors.join("; ")));
        } else {
            self.status_message = "Constructs quarantined".into();
            self.play_sound(SoundType::Delete);
        }

        self.selected = None;
        self.multi_selected.clear();
        self.confirm_delete_dialog = false;
        self.load_current_directory();
    }

    fn sync_to_system_clipboard(&self) {
        if self.clipboard_paths.is_empty() {
            return;
        }
        let paths_text = self
            .clipboard_paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("\n");

        // Try wl-copy first (Wayland), then xclip (X11)
        let result = std::process::Command::new("wl-copy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(paths_text.as_bytes())?;
                }
                child.wait()
            })
            .or_else(|_| {
                std::process::Command::new("xclip")
                    .args(["-selection", "clipboard"])
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .and_then(|mut child| {
                        use std::io::Write;
                        if let Some(stdin) = child.stdin.as_mut() {
                            stdin.write_all(paths_text.as_bytes())?;
                        }
                        child.wait()
                    })
            });

        let _ = result;
    }

    // ── Helpers ───────────────────────────────────────────────

    fn get_selected_paths(&self) -> Vec<PathBuf> {
        if !self.multi_selected.is_empty() {
            self.multi_selected
                .iter()
                .filter_map(|&i| self.entries.get(i).map(|e| e.path.clone()))
                .collect()
        } else if let Some(idx) = self.selected {
            self.entries
                .get(idx)
                .map(|e| vec![e.path.clone()])
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    pub(crate) fn set_error(&mut self, msg: String) {
        self.play_sound(SoundType::Error);
        self.error_message = Some((msg, Instant::now()));
    }

    // ── Tab Management ────────────────────────────────────────

    pub(crate) fn open_new_tab(&mut self) {
        let new_tab = Tab {
            path: self.current_path.clone(),
            selected: None,
        };
        self.tabs.push(new_tab);
        self.active_tab = self.tabs.len() - 1;
    }

    pub(crate) fn close_tab(&mut self, index: usize) {
        if self.tabs.len() <= 1 {
            return;
        }
        // Save current tab state before closing
        self.tabs.remove(index);
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        }
        // Load the tab we switched to
        let path = self.tabs[self.active_tab].path.clone();
        self.current_path = path;
        self.selected = self.tabs[self.active_tab].selected;
        self.load_current_directory();
    }

    pub(crate) fn switch_to_tab(&mut self, index: usize) {
        if index >= self.tabs.len() || index == self.active_tab {
            return;
        }
        // Save current tab state
        if self.active_tab < self.tabs.len() {
            self.tabs[self.active_tab].path = self.current_path.clone();
            self.tabs[self.active_tab].selected = self.selected;
        }
        // Switch
        self.active_tab = index;
        let tab = &self.tabs[index];
        let path = tab.path.clone();
        self.selected = tab.selected;
        self.current_path = path;
        self.load_current_directory();
        self.sync_command_bar_with_current_path();
    }

    // ── fzf Integration ───────────────────────────────────────

    pub(crate) fn fzf_search(&mut self, query: &str) {
        if !self.fzf_available {
            self.set_error("fzf not found — install with: sudo apt install fzf".into());
            return;
        }
        self.fzf_results = crate::integrations::fzf::fuzzy_search(&self.current_path, query, 5);
        self.fzf_mode = !self.fzf_results.is_empty();
        self.status_message = format!(
            "NEURAL SCAN // {} matches via fzf",
            self.fzf_results.len()
        );
    }

    pub(crate) fn fzf_interactive(&mut self) {
        if !self.fzf_available {
            self.set_error("fzf not found — install with: sudo apt install fzf".into());
            return;
        }
        if let Some(path) = crate::integrations::fzf::launch_interactive(
            &self.current_path,
            &self.settings.terminal_emulator,
        ) {
            if path.is_dir() {
                self.navigate_to(path);
            } else if path.is_file() {
                if let Some(parent) = path.parent() {
                    if parent != self.current_path {
                        self.navigate_to(parent.to_path_buf());
                    }
                }
                self.open_file(&path);
            }
        }
    }

    pub(crate) fn refresh_system_info(&mut self) {
        if self.sys_last_refresh.elapsed().as_secs() >= 2 {
            self.sys_info.refresh_cpu_all();
            self.sys_info.refresh_memory();
            self.sys_disks.refresh_list();

            let cpu = self.sys_info.global_cpu_usage();
            self.cpu_history.push_back(cpu);
            if self.cpu_history.len() > 60 {
                self.cpu_history.pop_front();
            }

            let total_mem = self.sys_info.total_memory().max(1) as f32;
            let used_mem = self.sys_info.used_memory() as f32;
            let mem_pct = used_mem / total_mem * 100.0;
            self.mem_history.push_back(mem_pct);
            if self.mem_history.len() > 60 {
                self.mem_history.pop_front();
            }

            self.sys_last_refresh = Instant::now();
        }

        if self.process_matrix_open {
            self.refresh_process_matrix(false);
        }
        if self.service_deck_open {
            self.refresh_service_deck(false);
        }
        if self.log_viewer_open {
            self.refresh_log_viewer(false);
        }
        if self.signal_deck_open {
            self.refresh_audio_snapshot(false);
            self.refresh_power_info(false);
            self.refresh_clipboard(false);
            self.refresh_notifications(false);
        }
    }

    // ── Undo / Redo ────────────────────────────────────────

    fn record_undo(&mut self, action: UndoAction) {
        const MAX_UNDO: usize = 100;
        self.undo_stack.push(action);
        if self.undo_stack.len() > MAX_UNDO {
            self.undo_stack.drain(..self.undo_stack.len() - MAX_UNDO);
        }
    }

    pub(crate) fn undo(&mut self) {
        if let Some(action) = self.undo_stack.pop() {
            match action.clone() {
                UndoAction::Rename { old_path, new_path } => {
                    if let Err(e) = std::fs::rename(&new_path, &old_path) {
                        self.set_error(format!("Undo rename failed: {}", e));
                    } else {
                        self.status_message = "Undo: rename reversed".into();
                        self.redo_stack.push(action);
                    }
                }
                UndoAction::Delete { original_path, trash_name } => {
                    let dest_dir = original_path.parent().unwrap_or(&self.current_path).to_path_buf();
                    match filesystem::restore_from_trash(&trash_name, &dest_dir) {
                        Ok(_) => {
                            self.status_message = format!("Undo: \"{}\" restored from quarantine", trash_name);
                            self.redo_stack.push(action);
                        }
                        Err(e) => self.set_error(format!("Undo delete failed: {}", e)),
                    }
                }
                UndoAction::Copy { copied_to } => {
                    let mut ok = true;
                    for path in &copied_to {
                        if path.is_dir() {
                            if std::fs::remove_dir_all(path).is_err() { ok = false; }
                        } else if std::fs::remove_file(path).is_err() {
                            ok = false;
                        }
                    }
                    if ok {
                        self.status_message = format!("Undo: removed {} copied constructs", copied_to.len());
                        self.redo_stack.push(action);
                    } else {
                        self.set_error("Undo copy: some files could not be removed".into());
                    }
                }
                UndoAction::Move { sources, destinations } => {
                    let mut ok = true;
                    for (dest, src_dir) in destinations.iter().zip(sources.iter()) {
                        let src_parent = src_dir.parent().unwrap_or(src_dir);
                        if filesystem::move_file(dest, src_parent).is_err() {
                            ok = false;
                        }
                    }
                    if ok {
                        self.status_message = "Undo: move reversed".into();
                        self.redo_stack.push(action);
                    } else {
                        self.set_error("Undo move: some files could not be moved back".into());
                    }
                }
                UndoAction::Create { path, kind } => {
                    let result = if kind == EntryKind::Directory {
                        std::fs::remove_dir_all(&path)
                    } else {
                        std::fs::remove_file(&path)
                    };
                    if let Err(e) = result {
                        self.set_error(format!("Undo create failed: {}", e));
                    } else {
                        self.status_message = "Undo: created item removed".into();
                        self.redo_stack.push(action);
                    }
                }
            }
            self.load_current_directory();
            self.play_sound(SoundType::Navigate);
        } else {
            self.status_message = "Nothing to undo".into();
        }
    }

    pub(crate) fn redo(&mut self) {
        if let Some(action) = self.redo_stack.pop() {
            match action.clone() {
                UndoAction::Rename { old_path, new_path } => {
                    if let Err(e) = std::fs::rename(&old_path, &new_path) {
                        self.set_error(format!("Redo rename failed: {}", e));
                    } else {
                        self.status_message = "Redo: rename re-applied".into();
                        self.undo_stack.push(action);
                    }
                }
                UndoAction::Delete {
                    original_path,
                    trash_name: _,
                } => {
                    match filesystem::delete_to_trash(&original_path) {
                        Ok(trash_name) => {
                            self.undo_stack.push(UndoAction::Delete {
                                original_path,
                                trash_name,
                            });
                            self.status_message = "Redo: re-quarantined".into();
                        }
                        Err(e) => self.set_error(format!("Redo delete failed: {}", e)),
                    }
                }
                UndoAction::Copy { copied_to: _copied_to } => {
                    // Cannot redo copy without knowing sources
                    self.status_message = "Redo: copy cannot be re-applied".into();
                }
                UndoAction::Move { sources, destinations } => {
                    let mut ok = true;
                    for (src, dest) in sources.iter().zip(destinations.iter()) {
                        let dest_dir = dest.parent().unwrap_or(dest);
                        if filesystem::move_file(src, dest_dir).is_err() {
                            ok = false;
                        }
                    }
                    if ok {
                        self.status_message = "Redo: move re-applied".into();
                        self.undo_stack.push(action);
                    } else {
                        self.set_error("Redo move failed".into());
                    }
                }
                UndoAction::Create { path, kind } => {
                    let result = filesystem::recreate_entry(&path, kind);
                    match result {
                        Ok(()) => {
                            self.status_message = "Redo: re-created".into();
                            self.undo_stack.push(action);
                        }
                        Err(e) => self.set_error(format!("Redo create failed: {}", e)),
                    }
                }
            }
            self.load_current_directory();
        } else {
            self.status_message = "Nothing to redo".into();
        }
    }

    // ── Split Pane ────────────────────────────────────────────

    pub(crate) fn toggle_split_pane(&mut self) {
        self.split_pane_active = !self.split_pane_active;
        if self.split_pane_active {
            self.split_pane_path = self.current_path.clone();
            self.load_split_pane_directory();
            self.status_message = "DUAL JACK — split view activated".into();
        } else {
            self.split_pane_entries.clear();
            self.status_message = "Split view deactivated".into();
        }
    }

    pub(crate) fn load_split_pane_directory(&mut self) {
        match filesystem::read_directory(&self.split_pane_path, self.show_hidden) {
            Ok(mut entries) => {
                filesystem::sort_entries(
                    &mut entries,
                    self.split_pane_sort_column,
                    self.split_pane_sort_ascending,
                );
                self.split_pane_entries = entries;
            }
            Err(e) => {
                self.set_error(format!("Split pane access denied: {}", e));
                self.split_pane_entries.clear();
            }
        }
        self.split_pane_selected = None;
    }

    pub(crate) fn split_pane_navigate(&mut self, path: PathBuf) {
        if path.is_dir() {
            self.split_pane_path = path;
            self.load_split_pane_directory();
        }
    }

    // ── Sound Effects ─────────────────────────────────────────

    pub(crate) fn play_sound(&self, sound_type: SoundType) {
        if !self.sound_enabled {
            return;
        }
        let freq = match sound_type {
            SoundType::Navigate => 880.0,
            SoundType::Select => 1200.0,
            SoundType::Error => 220.0,
            SoundType::Delete => 330.0,
            SoundType::CopyComplete => 1400.0,
        };
        let duration_ms = match sound_type {
            SoundType::Error => 200,
            SoundType::Delete => 150,
            _ => 60,
        };
        // Spawn a thread to avoid blocking the UI
        std::thread::spawn(move || {
            if let Ok((_stream, handle)) = rodio::OutputStream::try_default() {
                let source = rodio::source::SineWave::new(freq)
                    .take_duration(std::time::Duration::from_millis(duration_ms))
                    .amplify(0.15);
                let _ = handle.play_raw(source);
                std::thread::sleep(std::time::Duration::from_millis(duration_ms + 20));
            }
        });
    }

    // ── Terminal Panel ────────────────────────────────────────

    fn run_embedded_shell_command_at(&mut self, cmd: String, label: Option<String>, cwd: PathBuf) {
        let cmd = cmd.trim().to_string();
        if cmd.is_empty() || self.terminal_task_running {
            return;
        }

        let label = label.unwrap_or_else(|| cmd.clone());
        let job_id = self.start_operator_job(&cmd, &label, &cwd);
        self.terminal_panel_visible = true;
        self.terminal_output.push(format!("$ {}", cmd));
        self.terminal_history.push(cmd.clone());
        if self.terminal_history.len() > 50 {
            let drain = self.terminal_history.len() - 50;
            self.terminal_history.drain(..drain);
        }
        self.terminal_task_running = true;
        self.terminal_running_command = Some(label);
        self.terminal_started_at = Some(Instant::now());
        self.trim_terminal_output();

        let tx = self.background_tx.clone();
        std::thread::spawn(move || {
            let started = Instant::now();
            let result = std::process::Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .current_dir(cwd)
                .output();

            match result {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let mut lines = Vec::new();
                    for line in stdout.lines().take(200) {
                        lines.push(line.to_string());
                    }
                    for line in stderr.lines().take(50) {
                        lines.push(format!("[ERR] {}", line));
                    }
                    if !out.status.success() && stderr.trim().is_empty() {
                        lines.push(format!("[ERR] command exited with status {}", out.status));
                    }
                    let _ = tx.send(BackgroundTaskResult::TerminalFinished {
                        job_id,
                        lines,
                        duration: started.elapsed(),
                        success: out.status.success(),
                    });
                }
                Err(e) => {
                    let _ = tx.send(BackgroundTaskResult::TerminalSpawnFailed {
                        job_id,
                        message: e.to_string(),
                    });
                }
            }
        });
    }

    fn run_embedded_shell_command(&mut self, cmd: String, label: Option<String>) {
        self.run_embedded_shell_command_at(cmd, label, self.current_path.clone());
    }

    pub(crate) fn run_terminal_command(&mut self) {
        let cmd = self.terminal_input.trim().to_string();
        self.run_embedded_shell_command(cmd, None);
        self.terminal_input.clear();
    }

    pub(crate) fn trigger_glitch(&mut self) {
        if self.reduced_motion {
            return;
        }
        self.glitch_active = true;
        self.glitch_start = Some(Instant::now());
    }

    // ── Keyboard Shortcuts ────────────────────────────────────

    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        let open_protocol_launcher = ctx.input(|input| {
            (input.modifiers.ctrl && input.key_pressed(egui::Key::K))
                || (!input.modifiers.ctrl
                    && !input.modifiers.alt
                    && !input.modifiers.shift
                    && input.key_pressed(egui::Key::Slash))
        });
        if open_protocol_launcher {
            self.set_command_surface_mode(CommandSurfaceMode::Protocol);
        }

        let focus_path_surface = ctx
            .input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::L));
        if focus_path_surface {
            self.set_command_surface_mode(CommandSurfaceMode::Path);
        }

        let restore_quick_scene = ctx.input(|input| {
            if !(input.modifiers.alt && !input.modifiers.ctrl && !input.modifiers.shift) {
                return None;
            }

            if input.key_pressed(egui::Key::Num1) {
                Some(0)
            } else if input.key_pressed(egui::Key::Num2) {
                Some(1)
            } else if input.key_pressed(egui::Key::Num3) {
                Some(2)
            } else if input.key_pressed(egui::Key::Num4) {
                Some(3)
            } else {
                None
            }
        });
        if let Some(slot_index) = restore_quick_scene {
            self.restore_quick_scene_slot(slot_index);
        }

        if self.command_bar_active {
            return;
        }

        ctx.input(|input| {
            let ctrl = input.modifiers.ctrl;
            let shift = input.modifiers.shift;

            if ctrl && input.key_pressed(egui::Key::H) {
                self.show_hidden = !self.show_hidden;
                self.load_current_directory();
            }
            if ctrl && input.key_pressed(egui::Key::C) {
                self.copy_selected();
            }
            if ctrl && input.key_pressed(egui::Key::X) {
                self.cut_selected();
            }
            if ctrl && input.key_pressed(egui::Key::V) {
                self.paste();
            }
            // Undo / Redo
            if ctrl && input.modifiers.shift && input.key_pressed(egui::Key::Z) {
                self.redo();
            } else if ctrl && input.key_pressed(egui::Key::Z) {
                self.undo();
            }
            if ctrl && input.modifiers.shift && input.key_pressed(egui::Key::N) {
                self.new_folder_dialog = true;
            }
            if ctrl && !input.modifiers.shift && input.key_pressed(egui::Key::N) {
                self.new_file_dialog = true;
            }
            if ctrl && input.key_pressed(egui::Key::A) {
                self.multi_selected.clear();
                for i in 0..self.entries.len() {
                    self.multi_selected.insert(i);
                }
                if !self.entries.is_empty() && self.selected.is_none() {
                    self.selected = Some(0);
                }
                self.status_message = format!(
                    "Selected {} constructs",
                    self.multi_selected.len()
                );
            }
            if input.key_pressed(egui::Key::Delete)
                && (self.selected.is_some() || !self.multi_selected.is_empty())
            {
                self.request_delete();
            }
            if input.key_pressed(egui::Key::F2) {
                if let Some(idx) = self.selected {
                    if let Some(entry) = self.entries.get(idx) {
                        self.rename_index = Some(idx);
                        self.rename_text = entry.name.clone();
                    }
                }
            }
            if input.key_pressed(egui::Key::F5) {
                self.load_current_directory();
            }
            if input.key_pressed(egui::Key::F1) {
                self.settings_panel_open = !self.settings_panel_open;
                if self.settings_panel_open {
                    self.ui_scale_preview = self.settings.font_size;
                }
            }
            if input.key_pressed(egui::Key::F3) {
                self.resource_monitor_visible = !self.resource_monitor_visible;
            }
            if input.key_pressed(egui::Key::F4) {
                self.toggle_split_pane();
            }
            if input.key_pressed(egui::Key::F7) {
                self.terminal_panel_visible = !self.terminal_panel_visible;
            }
            if input.key_pressed(egui::Key::F11) {
                self.scanlines_enabled = !self.scanlines_enabled;
            }
            if input.key_pressed(egui::Key::F12) {
                self.crt_effect = !self.crt_effect;
            }
            if input.key_pressed(egui::Key::Backspace) {
                self.go_up();
            }
            if input.key_pressed(egui::Key::Enter) {
                if let Some(idx) = self.selected {
                    self.open_entry(idx);
                }
            }
            if input.key_pressed(egui::Key::ArrowDown) {
                let next = self
                    .selected
                    .map(|i| (i + 1).min(self.entries.len().saturating_sub(1)))
                    .unwrap_or(0);
                if !self.entries.is_empty() {
                    self.selected = Some(next);
                }
            }
            if input.key_pressed(egui::Key::ArrowUp) {
                let prev = self.selected.map(|i| i.saturating_sub(1)).unwrap_or(0);
                if !self.entries.is_empty() {
                    self.selected = Some(prev);
                }
            }
            if ctrl && input.key_pressed(egui::Key::B) {
                self.sidebar_visible = !self.sidebar_visible;
            }
            // Tab management
            if ctrl && input.key_pressed(egui::Key::T) {
                self.open_new_tab();
            }
            if ctrl && input.key_pressed(egui::Key::W) {
                let idx = self.active_tab;
                if self.tabs.len() > 1 {
                    self.close_tab(idx);
                }
            }
            // View modes: Ctrl+1/2/3/4
            if ctrl && input.key_pressed(egui::Key::Num1) {
                self.view_mode = ViewMode::List;
            }
            if ctrl && input.key_pressed(egui::Key::Num2) {
                self.view_mode = ViewMode::Grid;
            }
            if ctrl && input.key_pressed(egui::Key::Num3) {
                self.view_mode = ViewMode::HexGrid;
            }
            if ctrl && input.key_pressed(egui::Key::Num4) {
                self.view_mode = ViewMode::Hex;
            }
            // Hex grid zoom: Ctrl+Plus / Ctrl+Minus / Ctrl+0
            if self.view_mode == ViewMode::HexGrid {
                if ctrl && input.key_pressed(egui::Key::Plus) {
                    self.hex_zoom = (self.hex_zoom + 0.15).min(3.0);
                }
                if ctrl && input.key_pressed(egui::Key::Minus) {
                    self.hex_zoom = (self.hex_zoom - 0.15).max(0.3);
                }
                if ctrl && input.key_pressed(egui::Key::Num0) {
                    self.hex_zoom = 1.0;
                    self.hex_pan_offset = egui::Vec2::ZERO;
                }
            }
            // Preview panel
            if ctrl && input.key_pressed(egui::Key::P) {
                self.preview_visible = !self.preview_visible;
            }
            // Properties dialog
            if ctrl && input.key_pressed(egui::Key::I) {
                if let Some(idx) = self.selected {
                    if let Some(entry) = self.entries.get(idx) {
                        self.properties_target = Some(entry.path.clone());
                        self.properties_dialog = true;
                    }
                }
            }
            // fzf interactive
            if ctrl && input.key_pressed(egui::Key::F) {
                self.fzf_interactive();
            }
            if ctrl && input.modifiers.shift && input.key_pressed(egui::Key::S) {
                self.save_current_scene(None);
            }
            if ctrl && input.modifiers.alt && input.key_pressed(egui::Key::S) {
                self.open_scene_manager();
            }
            // Content search (grep/rg)
            if ctrl && input.key_pressed(egui::Key::G) {
                self.content_search_dialog = true;
                self.content_search_results.clear();
            }
            // Batch rename
            if ctrl && input.key_pressed(egui::Key::R) && !self.multi_selected.is_empty() {
                self.batch_rename_dialog = true;
                self.batch_rename_find.clear();
                self.batch_rename_replace.clear();
            }
            // Data rain toggle
            if input.key_pressed(egui::Key::F10) {
                self.data_rain_enabled = !self.data_rain_enabled;
            }
            // Neon glow toggle
            if input.key_pressed(egui::Key::F8) {
                self.neon_glow = !self.neon_glow;
            }
            // Chromatic aberration toggle
            if input.key_pressed(egui::Key::F6) {
                self.chromatic_aberration = !self.chromatic_aberration;
            }
            // SFTP remote connection dialog
            if input.key_pressed(egui::Key::F9) {
                self.sftp_dialog = !self.sftp_dialog;
            }
            // Process matrix (Ctrl+Shift+P)
            if ctrl && shift && input.key_pressed(egui::Key::P) {
                if self.process_matrix_open {
                    self.process_matrix_open = false;
                } else {
                    self.open_process_matrix();
                }
            }
            // Service deck (Ctrl+D)
            if ctrl && !shift && input.key_pressed(egui::Key::D) {
                if self.service_deck_open {
                    self.service_deck_open = false;
                } else {
                    self.open_service_deck();
                }
            }
            // Signal deck (Ctrl+Shift+D)
            if ctrl && shift && input.key_pressed(egui::Key::D) {
                if self.signal_deck_open {
                    self.signal_deck_open = false;
                } else {
                    self.open_signal_deck();
                }
            }
            // Log viewer (Ctrl+J)
            if ctrl && input.key_pressed(egui::Key::J) {
                if self.log_viewer_open {
                    self.log_viewer_open = false;
                } else {
                    self.open_log_viewer();
                }
            }
            // Network mesh (Ctrl+Shift+N)
            if ctrl && shift && input.key_pressed(egui::Key::N) {
                if self.network_mesh_open {
                    self.network_mesh_open = false;
                } else {
                    self.open_network_mesh();
                }
            }
            // Device bay (Ctrl+Shift+B)
            if ctrl && shift && input.key_pressed(egui::Key::B) {
                if self.device_bay_open {
                    self.device_bay_open = false;
                } else {
                    self.open_device_bay();
                }
            }
            // Window bridge (Ctrl+Shift+W)
            if ctrl && shift && input.key_pressed(egui::Key::W) {
                if self.window_bridge_open {
                    self.window_bridge_open = false;
                } else {
                    self.open_window_bridge();
                }
            }
            if input.key_pressed(egui::Key::Escape) {
                self.context_menu_open = false;
                self.settings_panel_open = false;
                self.new_file_dialog = false;
                self.new_folder_dialog = false;
                self.confirm_delete_dialog = false;
                self.delete_pending_indices.clear();
                self.properties_dialog = false;
                self.properties_target = None;
                self.trash_view_open = false;
                self.symlink_dialog = false;
                self.content_search_dialog = false;
                self.batch_rename_dialog = false;
                self.terminal_panel_visible = false;
                self.sftp_dialog = false;
                self.scene_manager_open = false;
                self.process_matrix_open = false;
                self.service_deck_open = false;
                self.log_viewer_open = false;
                self.signal_deck_open = false;
                self.network_mesh_open = false;
                self.device_bay_open = false;
                self.window_bridge_open = false;
                self.type_ahead_buffer.clear();
            }
        });

        // ── Type-ahead search in file list ─────────────────────
        let any_dialog = self.rename_index.is_some()
            || self.new_folder_dialog
            || self.new_file_dialog
            || self.confirm_delete_dialog
            || self.content_search_dialog
            || self.batch_rename_dialog
            || self.settings_panel_open
            || self.sftp_dialog
            || self.open_with_dialog
            || self.properties_dialog
            || self.symlink_dialog
            || self.focus_command_bar_next_frame;

        if !any_dialog {
            let typed_text: String = ctx.input(|input| {
                if input.modifiers.ctrl || input.modifiers.alt {
                    return String::new();
                }
                input.events.iter().filter_map(|e| {
                    if let egui::Event::Text(t) = e { Some(t.as_str()) } else { None }
                }).collect()
            });

            if !typed_text.is_empty() {
                let now = Instant::now();
                let timed_out = now.duration_since(self.type_ahead_last_key).as_millis() > 500;
                if timed_out {
                    self.type_ahead_buffer.clear();
                }

                // Detect repeated single-character press (e.g. "f", "f", "f")
                let single_char = typed_text.len() == 1;
                let repeat_char = single_char
                    && !timed_out
                    && self.type_ahead_buffer.len() == 1
                    && self.type_ahead_buffer == typed_text;

                if repeat_char {
                    // Cycle to next entry starting with this character
                    let ch = typed_text.to_lowercase();
                    let start = self.selected.map(|i| i + 1).unwrap_or(0);
                    let len = self.entries.len();
                    let found = (0..len).find_map(|offset| {
                        let idx = (start + offset) % len;
                        if self.entries[idx].name.to_lowercase().starts_with(&ch) {
                            Some(idx)
                        } else {
                            None
                        }
                    });
                    if let Some(idx) = found {
                        self.selected = Some(idx);
                        self.multi_selected.clear();
                    }
                } else {
                    self.type_ahead_buffer.push_str(&typed_text);
                    let search = self.type_ahead_buffer.to_lowercase();
                    if let Some(idx) = self.entries.iter().position(|e| {
                        e.name.to_lowercase().starts_with(&search)
                    }) {
                        self.selected = Some(idx);
                        self.multi_selected.clear();
                    }
                }
                self.type_ahead_last_key = now;
            }
        }
    }
}

fn join_remote_path(base: &str, file_name: &str) -> String {
    if base == "/" {
        format!("/{}", file_name)
    } else {
        format!("{}/{}", base.trim_end_matches('/'), file_name)
    }
}

fn shell_quote(input: &str) -> String {
    format!("'{}'", input.replace('\'', "'\\''"))
}

// ── eframe::App Implementation ────────────────────────────────────

impl eframe::App for CyberFile {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count += 1;

        // Apply theme
        if !self.theme_applied {
            theme::apply_cyber_theme(ctx, self.current_theme, self.settings.font_size / 14.0);
            self.theme_applied = true;
        }

        // Clear old errors
        if let Some((_, when)) = &self.error_message {
            if when.elapsed().as_secs() >= 5 {
                self.error_message = None;
            }
        }

        // Boot screen
        if !self.boot_complete {
            self.render_boot_screen(ctx);
            return;
        }

        self.process_startup_scene_request();

        // Refresh system info
        self.refresh_system_info();

        // Keyboard shortcuts
        self.handle_keyboard(ctx);

        // Background task completions
        self.poll_background_tasks();
        if self.terminal_task_running || self.content_search_in_progress || self.sftp_busy {
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        // Render UI
        self.render_command_bar(ctx);
        self.render_status_bar(ctx);

        if self.sidebar_visible {
            self.render_sidebar(ctx);
        }

        if self.resource_monitor_visible {
            self.render_resource_monitor(ctx);
        }

        if self.preview_visible {
            self.render_preview_panel(ctx);
        }

        // Data rain (background layer)
        self.render_data_rain(ctx);

        // Terminal panel (bottom)
        if self.terminal_panel_visible {
            egui::TopBottomPanel::bottom("terminal_panel")
                .resizable(true)
                .default_height(180.0)
                .min_height(80.0)
                .max_height(400.0)
                .frame(
                    egui::Frame::new()
                        .fill(self.current_theme.bg_dark())
                        .stroke(egui::Stroke::new(1.0, self.current_theme.border_dim()))
                        .inner_margin(egui::Margin::symmetric(8, 4)),
                )
                .show(ctx, |ui| {
                    self.render_terminal_panel(ui);
                });
        }

        // Central panel
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(self.current_theme.bg_dark())
                    .inner_margin(egui::Margin::symmetric(8, 6)),
            )
            .show(ctx, |ui| {
                // Tab bar
                if self.tabs.len() > 1 {
                    self.render_tab_bar(ui);
                }

                // Real-time filter bar
                self.render_filter_bar(ui);

                if self.split_pane_active {
                    // Split view: two side-by-side columns
                    let available = ui.available_size();
                    let half_width = (available.x - 8.0) / 2.0;

                    ui.horizontal(|ui| {
                        // Left pane (primary)
                        ui.vertical(|ui| {
                            ui.set_max_width(half_width);
                            ui.set_min_width(half_width);
                            match self.view_mode {
                                ViewMode::List => self.render_file_view(ui),
                                ViewMode::Grid => self.render_grid_view(ui),
                                ViewMode::HexGrid => self.render_hex_grid_view(ui),
                                ViewMode::Hex => self.render_hex_view(ui),
                            }
                        });

                        // Divider
                        ui.separator();

                        // Right pane (split)
                        ui.vertical(|ui| {
                            self.render_split_pane(ui);
                        });
                    });
                } else {
                    // Normal: single view
                    match self.view_mode {
                        ViewMode::List => self.render_file_view(ui),
                        ViewMode::Grid => self.render_grid_view(ui),
                        ViewMode::HexGrid => self.render_hex_grid_view(ui),
                        ViewMode::Hex => self.render_hex_view(ui),
                    }
                }
            });

        // Overlays
        self.render_dialogs(ctx);

        if self.settings_panel_open {
            self.render_settings_panel(ctx);
        }

        if self.scene_manager_open {
            self.render_scene_manager(ctx);
        }

        // SFTP connection dialog
        if self.sftp_dialog {
            self.render_sftp_dialog(ctx);
        }

        // Stage 3 panels
        if self.process_matrix_open {
            self.render_process_matrix(ctx);
        }
        if self.service_deck_open {
            self.render_service_deck(ctx);
        }
        if self.log_viewer_open {
            self.render_log_viewer(ctx);
        }

        // Stage 4 panel
        if self.signal_deck_open {
            self.render_signal_deck(ctx);
        }

        // Stage 5 panels
        if self.network_mesh_open {
            self.render_network_mesh(ctx);
        }
        if self.device_bay_open {
            self.render_device_bay(ctx);
        }

        // Stage 6 panel
        if self.window_bridge_open {
            self.render_window_bridge(ctx);
        }

        // HUD overlay elements (NERV-style indicators)
        self.render_hud_overlay(ctx);
        self.render_border_pulse(ctx);

        // Visual effects (on top of everything)
        self.render_effects(ctx);

        // Expire glitch
        if self.glitch_active {
            if let Some(start) = self.glitch_start {
                if start.elapsed().as_millis() > 80 {
                    self.glitch_active = false;
                    self.glitch_start = None;
                }
            }
        }

        // Save window dimensions and session state periodically (every 60 frames ≈ 1s)
        if self.frame_count % 60 == 0 {
            let mut needs_save = false;

            if let Some(rect) = ctx.input(|i| i.viewport().inner_rect) {
                let size = rect.size();
                if (size.x - self.settings.window_width).abs() > 5.0
                    || (size.y - self.settings.window_height).abs() > 5.0
                {
                    self.settings.window_width = size.x;
                    self.settings.window_height = size.y;
                    needs_save = true;
                }
            }

            // Sync bookmarks
            let current_bookmarks: Vec<String> = self
                .bookmarks
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            if current_bookmarks != self.settings.bookmarks {
                self.settings.bookmarks = current_bookmarks;
                needs_save = true;
            }

            // Sync tabs
            let current_tabs: Vec<String> = self
                .tabs
                .iter()
                .map(|t| t.path.to_string_lossy().to_string())
                .collect();
            if current_tabs != self.settings.saved_tabs {
                self.settings.saved_tabs = current_tabs;
                needs_save = true;
            }

            // Update last directory
            let last_dir = self.current_path.to_string_lossy().to_string();
            if last_dir != self.settings.last_directory {
                self.settings.last_directory = last_dir;
                needs_save = true;
            }

            if needs_save {
                self.settings.save();
            }
        }

        if self.frame_count % 180 == 0 {
            self.sync_session_scene();
        }
    }
}

impl CyberFile {
    fn render_filter_bar(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("⟐ FILTER")
                    .color(t.primary())
                    .monospace()
                    .size(10.0),
            );
            let _resp = ui.add_sized(
                [200.0, 18.0],
                egui::TextEdit::singleline(&mut self.filter_text)
                    .font(egui::FontId::monospace(11.0))
                    .text_color(t.text_primary())
                    .hint_text(
                        RichText::new("type to filter...")
                            .color(t.text_dim())
                            .monospace(),
                    ),
            );
            if !self.filter_text.is_empty() {
                if ui
                    .button(RichText::new("✗").color(t.danger()).monospace().size(10.0))
                    .clicked()
                {
                    self.filter_text.clear();
                }
                let filter_lower = self.filter_text.to_lowercase();
                let count = self
                    .entries
                    .iter()
                    .filter(|e| e.name.to_lowercase().contains(&filter_lower))
                    .count();
                ui.label(
                    RichText::new(format!("{}/{}", count, self.entries.len()))
                        .color(t.text_dim())
                        .monospace()
                        .size(10.0),
                );
            }
            // Multi-select count indicator
            if !self.multi_selected.is_empty() {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!(
                            "◈ {} SELECTED",
                            self.multi_selected.len()
                        ))
                        .color(t.accent())
                        .monospace()
                        .size(10.0),
                    );
                });
            }
        });
        ui.add_space(2.0);
    }

    fn render_dialogs(&mut self, ctx: &egui::Context) {
        let t = self.current_theme;

        // New folder dialog
        if self.new_folder_dialog {
            egui::Window::new(
                RichText::new("\u{25C8} INITIALIZE NEW SECTOR")
                    .color(t.primary())
                    .monospace(),
            )
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Sector identifier:")
                        .color(t.text_dim())
                        .monospace()
                        .size(12.0),
                );
                let resp = ui.add_sized(
                    [300.0, 22.0],
                    egui::TextEdit::singleline(&mut self.new_folder_name)
                        .font(egui::FontId::monospace(13.0))
                        .text_color(t.text_primary()),
                );
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.create_new_folder();
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui
                        .button(RichText::new("INITIALIZE").color(t.primary()).monospace())
                        .clicked()
                    {
                        self.create_new_folder();
                    }
                    if ui
                        .button(RichText::new("ABORT").color(t.danger()).monospace())
                        .clicked()
                    {
                        self.new_folder_dialog = false;
                        self.new_folder_name.clear();
                    }
                });
            });
        }

        // Rename dialog
        if let Some(idx) = self.rename_index {
            let entry_name = self
                .entries
                .get(idx)
                .map(|e| e.name.clone())
                .unwrap_or_default();

            egui::Window::new(
                RichText::new("\u{25C8} REASSIGN IDENTIFIER")
                    .color(t.primary())
                    .monospace(),
            )
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(format!("Current ID: {}", entry_name))
                        .color(t.text_dim())
                        .monospace()
                        .size(12.0),
                );
                ui.add_space(4.0);
                let resp = ui.add_sized(
                    [300.0, 22.0],
                    egui::TextEdit::singleline(&mut self.rename_text)
                        .font(egui::FontId::monospace(13.0))
                        .text_color(t.text_primary()),
                );
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.apply_rename();
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui
                        .button(RichText::new("CONFIRM").color(t.primary()).monospace())
                        .clicked()
                    {
                        self.apply_rename();
                    }
                    if ui
                        .button(RichText::new("ABORT").color(t.danger()).monospace())
                        .clicked()
                    {
                        self.rename_index = None;
                        self.rename_text.clear();
                    }
                });
            });
        }

        // Open With dialog
        if self.open_with_dialog {
            let target_name = self
                .open_with_target
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".into());

            egui::Window::new(
                RichText::new("\u{25C8} ROUTE TO SUBSYSTEM")
                    .color(t.primary())
                    .monospace(),
            )
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(format!("\u{2502} TGT: {}", target_name))
                        .color(t.text_dim())
                        .monospace()
                        .size(11.0),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Subsystem process ID:")
                        .color(t.text_dim())
                        .monospace()
                        .size(10.0),
                );
                let resp = ui.add_sized(
                    [300.0, 22.0],
                    egui::TextEdit::singleline(&mut self.open_with_text)
                        .font(egui::FontId::monospace(13.0))
                        .text_color(t.text_primary())
                        .hint_text(RichText::new("enter subsystem // code, gimp, vlc").color(t.text_dim())),
                );
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let app = self.open_with_text.trim().to_string();
                    if let Some(path) = self.open_with_target.clone() {
                        if !app.is_empty() {
                            self.open_file_with(&path, &app);
                        }
                    }
                    self.open_with_dialog = false;
                    self.open_with_text.clear();
                    self.open_with_target = None;
                }

                // Quick-pick common apps
                ui.add_space(6.0);
                ui.label(
                    RichText::new("\u{250C}\u{2500} KNOWN SUBSYSTEMS \u{2500}\u{2500}\u{2500}\u{2500}\u{2510}")
                        .color(t.border_dim())
                        .monospace()
                        .size(9.0),
                );
                let quick_apps = [
                    ("code", "VS Code"),
                    ("vim", "Vim"),
                    ("nano", "Nano"),
                    ("gimp", "GIMP"),
                    ("vlc", "VLC"),
                    ("mpv", "mpv"),
                    ("xdg-open", "System Default"),
                ];
                for (cmd, label) in quick_apps {
                    if ui
                        .selectable_label(
                            false,
                            RichText::new(format!("  \u{25B6} {}", label))
                                .color(t.text_primary())
                                .monospace()
                                .size(11.0),
                        )
                        .clicked()
                    {
                        if let Some(path) = self.open_with_target.clone() {
                            self.open_file_with(&path, cmd);
                        }
                        self.open_with_dialog = false;
                        self.open_with_text.clear();
                        self.open_with_target = None;
                    }
                }

                // Option to save as default for this extension
                if let Some(path) = &self.open_with_target {
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        let app_text = self.open_with_text.trim().to_string();
                        if !app_text.is_empty() {
                            ui.add_space(4.0);
                            if ui
                                .button(
                                    RichText::new(format!("BIND PROTOCOL FOR .{}", ext_str.to_uppercase()))
                                        .color(t.success())
                                        .monospace()
                                        .size(10.0),
                                )
                                .clicked()
                            {
                                self.settings
                                    .custom_openers
                                    .insert(ext_str, app_text.clone());
                                self.settings.save();
                                self.status_message =
                                    format!("Protocol bound: .{} → {}", ext.to_string_lossy().to_uppercase(), app_text);
                            }
                        }
                    }
                }

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui
                        .button(
                            RichText::new("ENGAGE")
                                .color(t.primary())
                                .monospace(),
                        )
                        .clicked()
                    {
                        let app = self.open_with_text.trim().to_string();
                        if let Some(path) = self.open_with_target.clone() {
                            if !app.is_empty() {
                                self.open_file_with(&path, &app);
                            }
                        }
                        self.open_with_dialog = false;
                        self.open_with_text.clear();
                        self.open_with_target = None;
                    }
                    if ui
                        .button(
                            RichText::new("ABORT")
                                .color(t.danger())
                                .monospace(),
                        )
                        .clicked()
                    {
                        self.open_with_dialog = false;
                        self.open_with_text.clear();
                        self.open_with_target = None;
                    }
                });
            });
        }

        // New file dialog
        if self.new_file_dialog {
            egui::Window::new(
                RichText::new("\u{25C8} INITIALIZE NEW CONSTRUCT")
                    .color(t.primary())
                    .monospace(),
            )
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Construct identifier:")
                        .color(t.text_dim())
                        .monospace()
                        .size(12.0),
                );
                let resp = ui.add_sized(
                    [300.0, 22.0],
                    egui::TextEdit::singleline(&mut self.new_file_name)
                        .font(egui::FontId::monospace(13.0))
                        .text_color(t.text_primary()),
                );
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.create_new_file();
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui
                        .button(RichText::new("INITIALIZE").color(t.primary()).monospace())
                        .clicked()
                    {
                        self.create_new_file();
                    }
                    if ui
                        .button(RichText::new("ABORT").color(t.danger()).monospace())
                        .clicked()
                    {
                        self.new_file_dialog = false;
                        self.new_file_name.clear();
                    }
                });
            });
        }

        // Confirm delete dialog — "PURGE PROTOCOL"
        if self.confirm_delete_dialog {
            let count = self.delete_pending_indices.len();
            let names: Vec<String> = self
                .delete_pending_indices
                .iter()
                .filter_map(|&i| self.entries.get(i).map(|e| e.name.clone()))
                .take(5)
                .collect();

            egui::Window::new(
                RichText::new("⚠ PURGE PROTOCOL")
                    .color(t.danger())
                    .monospace(),
            )
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!(
                        "Quarantine {} construct(s)?",
                        count
                    ))
                    .color(t.warning())
                    .monospace()
                    .size(13.0),
                );
                ui.add_space(4.0);
                for name in &names {
                    ui.label(
                        RichText::new(format!("  \u{25B6} {}", name))
                            .color(t.text_dim())
                            .monospace()
                            .size(11.0),
                    );
                }
                if count > 5 {
                    ui.label(
                        RichText::new(format!("  ... and {} more", count - 5))
                            .color(t.text_dim())
                            .monospace()
                            .size(11.0),
                    );
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui
                        .button(
                            RichText::new("CONFIRM PURGE")
                                .color(t.danger())
                                .monospace(),
                        )
                        .clicked()
                    {
                        self.confirm_delete_execute();
                    }
                    if ui
                        .button(RichText::new("ABORT").color(t.primary()).monospace())
                        .clicked()
                    {
                        self.confirm_delete_dialog = false;
                        self.delete_pending_indices.clear();
                    }
                });
            });
        }

        // ── Properties Dialog — "CONSTRUCT PROFILE" ──────
        if self.properties_dialog {
            if let Some(target) = self.properties_target.clone() {
                let mut close_dialog = false;
                egui::Window::new(
                    RichText::new("◈ CONSTRUCT PROFILE")
                        .color(t.primary())
                        .monospace(),
                )
                .collapsible(false)
                .resizable(true)
                .default_width(420.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    let name = target
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| target.to_string_lossy().to_string());

                    ui.label(
                        RichText::new(format!("│ TARGET: {}", name))
                            .color(t.text_primary())
                            .monospace()
                            .size(13.0)
                            .strong(),
                    );
                    ui.label(
                        RichText::new(format!("│ PATH: {}", target.display()))
                            .color(t.text_dim())
                            .monospace()
                            .size(10.0),
                    );
                    ui.add_space(6.0);

                    if let Ok(meta) = std::fs::metadata(&target) {
                        let is_dir = meta.is_dir();
                        let type_str = if is_dir { "DIRECTORY" } else { "FILE" };

                        // ── Type & Size ──
                        ui.label(
                            RichText::new("┌─ IDENTITY ─────────────────┐")
                                .color(t.border_dim())
                                .monospace()
                                .size(9.0),
                        );
                        let prop_line = |ui: &mut egui::Ui, key: &str, val: &str, color: Color32| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("│ {:>12}:", key))
                                        .color(t.text_dim())
                                        .monospace()
                                        .size(11.0),
                                );
                                ui.label(
                                    RichText::new(val)
                                        .color(color)
                                        .monospace()
                                        .size(11.0),
                                );
                            });
                        };

                        prop_line(ui, "TYPE", type_str, t.primary());
                        if !is_dir {
                            prop_line(ui, "SIZE", &bytesize::ByteSize(meta.len()).to_string(), t.text_primary());
                            prop_line(ui, "SIZE (raw)", &format!("{} bytes", meta.len()), t.text_dim());
                        }

                        if let Some(ext) = target.extension() {
                            prop_line(ui, "EXTENSION", &ext.to_string_lossy().to_uppercase(), t.warning());
                        }

                        ui.add_space(4.0);

                        // ── Timestamps ──
                        ui.label(
                            RichText::new("┌─ TEMPORAL DATA ────────────┐")
                                .color(t.border_dim())
                                .monospace()
                                .size(9.0),
                        );
                        if let Ok(modified) = meta.modified() {
                            let dt: chrono::DateTime<chrono::Local> = modified.into();
                            prop_line(ui, "MODIFIED", &dt.format("%Y-%m-%d %H:%M:%S").to_string(), t.text_primary());
                        }
                        if let Ok(accessed) = meta.accessed() {
                            let dt: chrono::DateTime<chrono::Local> = accessed.into();
                            prop_line(ui, "ACCESSED", &dt.format("%Y-%m-%d %H:%M:%S").to_string(), t.text_dim());
                        }
                        if let Ok(created) = meta.created() {
                            let dt: chrono::DateTime<chrono::Local> = created.into();
                            prop_line(ui, "CREATED", &dt.format("%Y-%m-%d %H:%M:%S").to_string(), t.text_dim());
                        }

                        ui.add_space(4.0);

                        // ── Unix Metadata ──
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::MetadataExt;
                            ui.label(
                                RichText::new("┌─ SYSTEM DATA ──────────────┐")
                                    .color(t.border_dim())
                                    .monospace()
                                    .size(9.0),
                            );
                            prop_line(ui, "INODE", &format!("{}", meta.ino()), t.text_dim());
                            prop_line(ui, "DEVICE", &format!("{}", meta.dev()), t.text_dim());
                            prop_line(ui, "HARD LINKS", &format!("{}", meta.nlink()), t.text_dim());
                            let uid = meta.uid();
                            let gid = meta.gid();
                            let user_name = uzers::get_user_by_uid(uid)
                                .map(|u| u.name().to_string_lossy().to_string())
                                .unwrap_or_else(|| uid.to_string());
                            let group_name = uzers::get_group_by_gid(gid)
                                .map(|g| g.name().to_string_lossy().to_string())
                                .unwrap_or_else(|| gid.to_string());
                            prop_line(ui, "OWNER", &format!("{} ({})", user_name, uid), t.text_dim());
                            prop_line(ui, "GROUP", &format!("{} ({})", group_name, gid), t.text_dim());
                            prop_line(ui, "MODE", &format!("{:04o}", meta.mode() & 0o7777), t.warning());

                            ui.add_space(4.0);

                            // ── Permissions Editor ──
                            ui.label(
                                RichText::new("┌─ ACCESS CONTROL ───────────┐")
                                    .color(t.border_dim())
                                    .monospace()
                                    .size(9.0),
                            );

                            let mode = meta.mode();
                            let perm_bits: [(u32, &str); 9] = [
                                (0o400, "Owner Read"),
                                (0o200, "Owner Write"),
                                (0o100, "Owner Execute"),
                                (0o040, "Group Read"),
                                (0o020, "Group Write"),
                                (0o010, "Group Execute"),
                                (0o004, "Other Read"),
                                (0o002, "Other Write"),
                                (0o001, "Other Execute"),
                            ];

                            let mut new_mode = mode & 0o7777;
                            let labels = ["OWNER", "GROUP", "OTHER"];

                            for (chunk_idx, label) in labels.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!("│ {:>8}:", label))
                                            .color(t.text_dim())
                                            .monospace()
                                            .size(10.0),
                                    );
                                    for j in 0..3 {
                                        let idx = chunk_idx * 3 + j;
                                        let (bit, _) = perm_bits[idx];
                                        let mut set = (new_mode & bit) != 0;
                                        let perm_label = ["R", "W", "X"][j];
                                        let color = if set { t.success() } else { t.text_dim() };
                                        if ui.checkbox(&mut set, RichText::new(perm_label).color(color).monospace().size(10.0)).changed() {
                                            if set {
                                                new_mode |= bit;
                                            } else {
                                                new_mode &= !bit;
                                            }
                                        }
                                    }
                                });
                            }

                            if new_mode != (mode & 0o7777) {
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!("│ NEW MODE: {:04o}", new_mode))
                                            .color(t.warning())
                                            .monospace()
                                            .size(11.0),
                                    );
                                    if ui
                                        .button(
                                            RichText::new("APPLY")
                                                .color(t.success())
                                                .monospace(),
                                        )
                                        .clicked()
                                    {
                                        use std::os::unix::fs::PermissionsExt;
                                        let perms = std::fs::Permissions::from_mode(new_mode);
                                        if let Err(e) = std::fs::set_permissions(&target, perms) {
                                            self.set_error(format!("Permission denied: {}", e));
                                        } else {
                                            self.status_message = format!("Access mode updated to {:04o}", new_mode);
                                            self.load_current_directory();
                                        }
                                    }
                                });
                            }
                        }
                    } else {
                        ui.label(
                            RichText::new("│ ACCESS DENIED — cannot read metadata")
                                .color(t.danger())
                                .monospace()
                                .size(11.0),
                        );
                    }

                    ui.add_space(8.0);
                    if ui
                        .button(RichText::new("CLOSE").color(t.primary()).monospace())
                        .clicked()
                    {
                        close_dialog = true;
                    }
                });

                if close_dialog {
                    self.properties_dialog = false;
                    self.properties_target = None;
                }
            }
        }

        // ── Trash View — "CONTAINMENT ZONE" ──────────────
        if self.trash_view_open {
            let mut close_trash = false;
            let mut restore_name: Option<String> = None;
            let mut empty_all = false;

            egui::Window::new(
                RichText::new("◈ CONTAINMENT ZONE")
                    .color(t.danger())
                    .monospace(),
            )
            .collapsible(false)
            .resizable(true)
            .default_width(450.0)
            .default_height(400.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(
                    RichText::new("│ Quarantined constructs awaiting purge")
                        .color(t.text_dim())
                        .monospace()
                        .size(10.0),
                );
                ui.add_space(4.0);

                if self.trash_entries.is_empty() {
                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("[ CONTAINMENT ZONE CLEAR ]")
                                .color(t.text_dim())
                                .monospace()
                                .size(12.0),
                        );
                    });
                } else {
                    ui.label(
                        RichText::new(format!("│ {} constructs in quarantine", self.trash_entries.len()))
                            .color(t.warning())
                            .monospace()
                            .size(11.0),
                    );
                    ui.add_space(4.0);

                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for (name, _path) in &self.trash_entries {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!("  ▸ {}", name))
                                            .color(t.text_primary())
                                            .monospace()
                                            .size(11.0),
                                    );
                                    if ui
                                        .small_button(
                                            RichText::new("RESTORE")
                                                .color(t.success())
                                                .monospace()
                                                .size(9.0),
                                        )
                                        .clicked()
                                    {
                                        restore_name = Some(name.clone());
                                    }
                                });
                            }
                        });
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if !self.trash_entries.is_empty() {
                        if ui
                            .button(
                                RichText::new("PURGE ALL")
                                    .color(t.danger())
                                    .monospace(),
                            )
                            .clicked()
                        {
                            empty_all = true;
                        }
                    }
                    if ui
                        .button(
                            RichText::new("CLOSE")
                                .color(t.primary())
                                .monospace(),
                        )
                        .clicked()
                    {
                        close_trash = true;
                    }
                });
            });

            if let Some(name) = restore_name {
                match crate::filesystem::restore_from_trash(&name, &self.current_path) {
                    Ok(_) => {
                        self.status_message = format!("Restored \"{}\" to current sector", name);
                        self.load_current_directory();
                    }
                    Err(e) => self.set_error(format!("Restore failed: {}", e)),
                }
                self.trash_entries = crate::filesystem::list_trash();
            }

            if empty_all {
                match crate::filesystem::empty_trash() {
                    Ok(count) => {
                        self.status_message = format!("Purged {} constructs from containment", count);
                    }
                    Err(e) => self.set_error(format!("Purge failed: {}", e)),
                }
                self.trash_entries = crate::filesystem::list_trash();
            }

            if close_trash {
                self.trash_view_open = false;
            }
        }

        // ── Symlink Creation Dialog — "NEURAL LINK" ──────
        if self.symlink_dialog {
            let mut close_dialog = false;
            let mut create_link = false;

            egui::Window::new(
                RichText::new("◇ CREATE NEURAL LINK")
                    .color(t.accent())
                    .monospace(),
            )
            .collapsible(false)
            .resizable(false)
            .default_width(380.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                if let Some(idx) = self.selected {
                    if let Some(entry) = self.entries.get(idx) {
                        ui.label(
                            RichText::new(format!("│ TARGET: {}", entry.path.display()))
                                .color(t.text_dim())
                                .monospace()
                                .size(10.0),
                        );
                    }
                }
                ui.add_space(6.0);
                ui.label(
                    RichText::new("│ LINK NAME:")
                        .color(t.text_dim())
                        .monospace()
                        .size(11.0),
                );
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.symlink_name)
                        .font(egui::FontId::monospace(13.0))
                        .text_color(t.text_primary())
                        .hint_text(
                            RichText::new("link name...")
                                .color(t.text_dim())
                                .monospace(),
                        ),
                );
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    create_link = true;
                }
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui
                        .button(RichText::new("CREATE").color(t.success()).monospace())
                        .clicked()
                    {
                        create_link = true;
                    }
                    if ui
                        .button(RichText::new("CANCEL").color(t.danger()).monospace())
                        .clicked()
                    {
                        close_dialog = true;
                    }
                });
            });

            if create_link {
                let name = self.symlink_name.trim().to_string();
                if !name.is_empty() && !name.contains('/') && !name.contains('\0') {
                    if let Some(idx) = self.selected {
                        if let Some(entry) = self.entries.get(idx) {
                            let link_path = self.current_path.join(&name);
                            match crate::filesystem::create_symlink(&entry.path, &link_path) {
                                Ok(()) => {
                                    self.status_message =
                                        format!("Neural link \"{}\" established", name);
                                    self.load_current_directory();
                                }
                                Err(e) => {
                                    self.set_error(format!("Link creation failed: {}", e));
                                }
                            }
                        }
                    }
                }
                close_dialog = true;
            }

            if close_dialog {
                self.symlink_dialog = false;
                self.symlink_name.clear();
            }
        }

        // ── Content Search Dialog — "DEEP SCAN" ──────────
        if self.content_search_dialog {
            let mut close_dialog = false;
            let mut run_search = false;
            let mut nav_to_file: Option<std::path::PathBuf> = None;

            egui::Window::new(
                RichText::new("⟐ DEEP SCAN — CONTENT SEARCH")
                    .color(t.primary())
                    .monospace(),
            )
            .collapsible(false)
            .resizable(true)
            .default_width(550.0)
            .default_height(400.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(format!(
                        "│ SCAN SECTOR: {}",
                        self.current_path.display()
                    ))
                    .color(t.text_dim())
                    .monospace()
                    .size(10.0),
                );
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("│ QUERY:")
                            .color(t.text_dim())
                            .monospace()
                            .size(11.0),
                    );
                    let resp = ui.add_sized(
                        [350.0, 22.0],
                        egui::TextEdit::singleline(&mut self.content_search_query)
                            .font(egui::FontId::monospace(12.0))
                            .text_color(t.text_primary())
                            .hint_text(
                                RichText::new("search pattern...")
                                    .color(t.text_dim())
                                    .monospace(),
                            ),
                    );
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        run_search = true;
                    }
                    if ui
                        .add_enabled(
                            !self.content_search_in_progress,
                            egui::Button::new(
                                RichText::new(if self.content_search_in_progress { "BUSY" } else { "SCAN" })
                                    .color(t.success())
                                    .monospace(),
                            ),
                        )
                        .clicked()
                    {
                        run_search = true;
                    }
                });

                ui.add_space(4.0);

                if self.content_search_in_progress {
                    ui.label(
                        RichText::new(format!(
                            "│ SCAN IN PROGRESS: {}",
                            self.content_search_active_query
                        ))
                            .color(t.accent())
                            .monospace()
                            .size(10.0),
                    );
                    ui.add_space(4.0);
                }

                if !self.content_search_results.is_empty() {
                    ui.label(
                        RichText::new(format!(
                            "│ {} matches found",
                            self.content_search_results.len()
                        ))
                        .color(t.warning())
                        .monospace()
                        .size(10.0),
                    );
                    ui.add_space(4.0);

                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for (file_path, line_num, line_text) in
                                &self.content_search_results
                            {
                                ui.horizontal(|ui| {
                                    // Clickable file path
                                    let short_path = file_path
                                        .strip_prefix(
                                            &self
                                                .current_path
                                                .to_string_lossy()
                                                .to_string(),
                                        )
                                        .unwrap_or(file_path)
                                        .trim_start_matches('/');
                                    if ui
                                        .link(
                                            RichText::new(format!(
                                                "{}:{}",
                                                short_path, line_num
                                            ))
                                            .color(t.primary())
                                            .monospace()
                                            .size(10.0),
                                        )
                                        .clicked()
                                    {
                                        nav_to_file =
                                            Some(std::path::PathBuf::from(file_path));
                                    }
                                    ui.label(
                                        RichText::new(
                                            if line_text.len() > 80 {
                                                format!("{}…", &line_text[..80])
                                            } else {
                                                line_text.clone()
                                            },
                                        )
                                        .color(t.text_primary())
                                        .monospace()
                                        .size(9.5),
                                    );
                                });
                            }
                        });
                }

                ui.add_space(6.0);
                if ui
                    .button(RichText::new("CLOSE").color(t.primary()).monospace())
                    .clicked()
                {
                    close_dialog = true;
                }
            });

            if run_search && !self.content_search_query.is_empty() {
                self.start_content_search();
            }

            if let Some(file_path) = nav_to_file {
                if let Some(parent) = file_path.parent() {
                    if parent != self.current_path {
                        self.navigate_to(parent.to_path_buf());
                    }
                }
                self.open_file(&file_path);
            }

            if close_dialog {
                self.content_search_dialog = false;
            }
        }

        // ── Batch Rename Dialog — "MASS REASSIGN" ────────
        if self.batch_rename_dialog {
            let mut close_dialog = false;
            let mut execute_rename = false;

            egui::Window::new(
                RichText::new("⧉ MASS REASSIGN — BATCH RENAME")
                    .color(t.warning())
                    .monospace(),
            )
            .collapsible(false)
            .resizable(false)
            .default_width(450.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(format!(
                        "│ {} constructs selected",
                        self.multi_selected.len()
                    ))
                    .color(t.text_dim())
                    .monospace()
                    .size(10.0),
                );
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("│ FIND:")
                            .color(t.text_dim())
                            .monospace()
                            .size(11.0),
                    );
                    ui.add_sized(
                        [300.0, 22.0],
                        egui::TextEdit::singleline(&mut self.batch_rename_find)
                            .font(egui::FontId::monospace(12.0))
                            .text_color(t.text_primary())
                            .hint_text(
                                RichText::new("pattern to find...")
                                    .color(t.text_dim())
                                    .monospace(),
                            ),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("│ REPLACE:")
                            .color(t.text_dim())
                            .monospace()
                            .size(11.0),
                    );
                    ui.add_sized(
                        [300.0, 22.0],
                        egui::TextEdit::singleline(&mut self.batch_rename_replace)
                            .font(egui::FontId::monospace(12.0))
                            .text_color(t.text_primary())
                            .hint_text(
                                RichText::new("replacement...")
                                    .color(t.text_dim())
                                    .monospace(),
                            ),
                    );
                });
                ui.horizontal(|ui| {
                    ui.checkbox(
                        &mut self.batch_rename_use_regex,
                        RichText::new("Use regex")
                            .color(t.text_dim())
                            .monospace()
                            .size(10.0),
                    );
                });

                // Preview
                if !self.batch_rename_find.is_empty() {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("│ PREVIEW:")
                            .color(t.warning())
                            .monospace()
                            .size(10.0),
                    );
                    let indices: Vec<usize> =
                        self.multi_selected.iter().copied().collect();
                    let re = if self.batch_rename_use_regex {
                        regex::Regex::new(&self.batch_rename_find).ok()
                    } else {
                        None
                    };
                    for &idx in indices.iter().take(10) {
                        if let Some(entry) = self.entries.get(idx) {
                            let new_name = if let Some(ref re) = re {
                                re.replace_all(
                                    &entry.name,
                                    self.batch_rename_replace.as_str(),
                                )
                                .to_string()
                            } else {
                                entry.name.replace(
                                    &self.batch_rename_find,
                                    &self.batch_rename_replace,
                                )
                            };
                            if new_name != entry.name {
                                ui.label(
                                    RichText::new(format!(
                                        "  {} → {}",
                                        entry.name, new_name
                                    ))
                                    .color(t.accent())
                                    .monospace()
                                    .size(10.0),
                                );
                            }
                        }
                    }
                    if indices.len() > 10 {
                        ui.label(
                            RichText::new(format!(
                                "  ... and {} more",
                                indices.len() - 10
                            ))
                            .color(t.text_dim())
                            .monospace()
                            .size(10.0),
                        );
                    }
                }

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui
                        .button(
                            RichText::new("EXECUTE")
                                .color(t.success())
                                .monospace(),
                        )
                        .clicked()
                    {
                        execute_rename = true;
                    }
                    if ui
                        .button(
                            RichText::new("CANCEL")
                                .color(t.danger())
                                .monospace(),
                        )
                        .clicked()
                    {
                        close_dialog = true;
                    }
                });
            });

            if execute_rename && !self.batch_rename_find.is_empty() {
                let indices: Vec<usize> =
                    self.multi_selected.iter().copied().collect();
                let re = if self.batch_rename_use_regex {
                    regex::Regex::new(&self.batch_rename_find).ok()
                } else {
                    None
                };
                let mut success_count = 0;
                let mut errors = Vec::new();

                for &idx in &indices {
                    if let Some(entry) = self.entries.get(idx) {
                        let new_name = if let Some(ref re) = re {
                            re.replace_all(
                                &entry.name,
                                self.batch_rename_replace.as_str(),
                            )
                            .to_string()
                        } else {
                            entry.name.replace(
                                &self.batch_rename_find,
                                &self.batch_rename_replace,
                            )
                        };
                        if new_name != entry.name
                            && !new_name.is_empty()
                            && !new_name.contains('/')
                            && !new_name.contains('\0')
                        {
                            let new_path = entry
                                .path
                                .parent()
                                .unwrap_or(&self.current_path)
                                .join(&new_name);
                            match std::fs::rename(&entry.path, &new_path) {
                                Ok(()) => success_count += 1,
                                Err(e) => {
                                    errors.push(format!("{}: {}", entry.name, e))
                                }
                            }
                        }
                    }
                }

                if !errors.is_empty() {
                    self.set_error(format!(
                        "Rename errors: {}",
                        errors.join("; ")
                    ));
                }
                self.status_message =
                    format!("Mass reassign: {} constructs renamed", success_count);
                self.multi_selected.clear();
                self.load_current_directory();
                close_dialog = true;
            }

            if close_dialog {
                self.batch_rename_dialog = false;
                self.batch_rename_find.clear();
                self.batch_rename_replace.clear();
            }
        }

        // Context menu — positioned at cursor
        if self.context_menu_open {
            let menu_id = egui::Id::new("context_menu_area");
            let menu_pos = self.context_menu_pos;
            let hive_mode = self.view_mode == ViewMode::HexGrid;

            if hive_mode {
                // ── RADIAL HEX CONTEXT MENU ──
                // Each option is a hexagon arranged in a circle around the cursor
                self.render_radial_hex_context_menu(ctx, t, menu_pos);
            } else {
                // ── CLASSIC LINEAR CONTEXT MENU ──
                let bullet = "\u{25B6}"; // ▶

                let resp = egui::Area::new(menu_id)
                    .fixed_pos(menu_pos)
                    .order(egui::Order::Foreground)
                    .show(ctx, |ui| {
                        egui::Frame::new()
                            .fill(Color32::TRANSPARENT)
                            .stroke(egui::Stroke::NONE)
                            .inner_margin(0.0)
                            .show(ui, |ui| {
                                egui::Frame::new()
                                    .fill(t.bg_dark())
                                    .stroke(egui::Stroke::new(1.5, t.primary()))
                                    .inner_margin(10.0)
                                    .outer_margin(0.0)
                                    .show(ui, |ui| {
                            // Decorative header
                            let header_text = if self.selected.is_some() {
                                "\u{250C}\u{2500}\u{2500} PROTOCOL SELECT \u{2500}\u{2500}\u{2510}"
                            } else {
                                "\u{250C}\u{2500}\u{2500} SECTOR PROTOCOL \u{2500}\u{2500}\u{2510}"
                            };
                            ui.label(
                                RichText::new(header_text)
                                    .color(t.primary())
                                    .monospace()
                                    .size(11.0)
                                    .strong(),
                            );

                            // Target info
                            if let Some(idx) = self.selected {
                                if let Some(entry) = self.entries.get(idx) {
                                    ui.label(
                                        RichText::new(format!("\u{2502} TGT: {}", entry.name))
                                            .color(t.text_dim())
                                            .monospace()
                                            .size(9.0),
                                    );
                                    let type_str = if entry.is_dir { "DIR" } else { "FILE" };
                                    ui.label(
                                        RichText::new(format!("\u{2502} TYPE: {} | ACC: {}", type_str, entry.permission_string()))
                                            .color(t.text_dim())
                                            .monospace()
                                            .size(9.0),
                                    );
                                }
                            } else {
                                ui.label(
                                    RichText::new(format!("\u{2502} LOC: {}", self.current_path.display()))
                                        .color(t.text_dim())
                                        .monospace()
                                        .size(9.0),
                                );
                            }

                            ui.add_space(4.0);

                            let btn = |ui: &mut egui::Ui, icon: &str, label: &str, key: &str, color: Color32| -> bool {
                                let text = format!(" {} {} {:>10}", icon, label, key);
                                ui.selectable_label(
                                    false,
                                    RichText::new(text)
                                        .color(color)
                                        .monospace()
                                        .size(11.0),
                                )
                                .clicked()
                            };

                            let divider = |ui: &mut egui::Ui, t: CyberTheme| {
                                ui.label(
                                    RichText::new("\u{2502}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2502}")
                                        .color(t.border_dim())
                                        .monospace()
                                        .size(9.0),
                                );
                            };

                            if self.selected.is_some() {
                                if btn(ui, bullet, "EXECUTE", "Enter", t.primary()) {
                                    if let Some(idx) = self.selected {
                                        self.open_entry(idx);
                                    }
                                    self.context_menu_open = false;
                                }

                                if let Some(idx) = self.selected {
                                    if let Some(entry) = self.entries.get(idx) {
                                        if !entry.is_dir {
                                            if btn(ui, bullet, "ROUTE TO...", "", t.accent()) {
                                                self.open_with_target = Some(entry.path.clone());
                                                self.open_with_text.clear();
                                                self.open_with_dialog = true;
                                                self.context_menu_open = false;
                                            }
                                            // Extract archive option for ZIP files
                                            let ext = entry
                                                .path
                                                .extension()
                                                .map(|e| e.to_string_lossy().to_lowercase())
                                                .unwrap_or_default();
                                            if ext == "zip" {
                                                if btn(ui, bullet, "EXTRACT ARCHIVE", "", t.success()) {
                                                    let dest = self.current_path.clone();
                                                    match crate::filesystem::extract_zip(&entry.path, &dest) {
                                                        Ok(count) => {
                                                            self.status_message = format!("Extracted {} files from archive", count);
                                                            self.load_current_directory();
                                                        }
                                                        Err(e) => self.set_error(format!("Extract failed: {}", e)),
                                                    }
                                                    self.context_menu_open = false;
                                                }
                                            }
                                        }
                                    }
                                }

                                // Compress selected files to archive
                                if btn(ui, bullet, "COMPRESS → ARCHIVE", "", t.warning()) {
                                    let mut paths: Vec<std::path::PathBuf> = Vec::new();
                                    if !self.multi_selected.is_empty() {
                                        for &idx in &self.multi_selected {
                                            if let Some(e) = self.entries.get(idx) {
                                                paths.push(e.path.clone());
                                            }
                                        }
                                    } else if let Some(idx) = self.selected {
                                        if let Some(e) = self.entries.get(idx) {
                                            paths.push(e.path.clone());
                                        }
                                    }
                                    if !paths.is_empty() {
                                        let archive_name = if paths.len() == 1 {
                                            let stem = paths[0].file_stem()
                                                .map(|s| s.to_string_lossy().to_string())
                                                .unwrap_or_else(|| "archive".to_string());
                                            format!("{}.zip", stem)
                                        } else {
                                            "archive.zip".to_string()
                                        };
                                        let output = self.current_path.join(&archive_name);
                                        match crate::filesystem::create_zip_archive(&paths, &output) {
                                            Ok(count) => {
                                                self.status_message = format!("Compressed {} entries → {}", count, archive_name);
                                                self.load_current_directory();
                                            }
                                            Err(e) => self.set_error(format!("Compress failed: {}", e)),
                                        }
                                    }
                                    self.context_menu_open = false;
                                }

                                divider(ui, t);

                                if btn(ui, bullet, "REPLICATE", "Ctrl+C", t.text_primary()) {
                                    self.copy_selected();
                                    self.context_menu_open = false;
                                }
                                if btn(ui, bullet, "EXTRACT", "Ctrl+X", t.warning()) {
                                    self.cut_selected();
                                    self.context_menu_open = false;
                                }
                                if btn(ui, bullet, "INJECT", "Ctrl+V", t.text_primary()) {
                                    self.paste();
                                    self.context_menu_open = false;
                                }

                                divider(ui, t);

                                if btn(ui, bullet, "REASSIGN ID", "F2", t.accent()) {
                                    if let Some(idx) = self.selected {
                                        if let Some(entry) = self.entries.get(idx) {
                                            self.rename_text = entry.name.clone();
                                            self.rename_index = Some(idx);
                                        }
                                    }
                                    self.context_menu_open = false;
                                }
                                if btn(ui, bullet, "NEW SECTOR", "Ctrl+Shift+N", t.success()) {
                                    self.new_folder_dialog = true;
                                    self.context_menu_open = false;
                                }
                                if btn(ui, bullet, "NEW CONSTRUCT", "Ctrl+N", t.success()) {
                                    self.new_file_dialog = true;
                                    self.context_menu_open = false;
                                }
                                if btn(ui, bullet, "NEURAL LINK", "", t.accent()) {
                                    if let Some(idx) = self.selected {
                                        if let Some(entry) = self.entries.get(idx) {
                                            self.symlink_name = format!("{}_link", entry.name);
                                        }
                                    }
                                    self.symlink_dialog = true;
                                    self.context_menu_open = false;
                                }
                                if btn(ui, bullet, "QUARANTINE", "Del", t.danger()) {
                                    self.request_delete();
                                    self.context_menu_open = false;
                                }

                                divider(ui, t);

                                if btn(ui, bullet, "SCAN PROFILE", "Ctrl+I", t.primary()) {
                                    if let Some(idx) = self.selected {
                                        if let Some(entry) = self.entries.get(idx) {
                                            self.properties_target = Some(entry.path.clone());
                                            self.properties_dialog = true;
                                        }
                                    }
                                    self.context_menu_open = false;
                                }
                                if btn(ui, bullet, "DEEP SCAN", "Ctrl+G", t.primary()) {
                                    self.content_search_dialog = true;
                                    self.content_search_results.clear();
                                    self.context_menu_open = false;
                                }
                                if !self.multi_selected.is_empty() {
                                    if btn(ui, bullet, "MASS REASSIGN", "Ctrl+R", t.warning()) {
                                        self.batch_rename_dialog = true;
                                        self.context_menu_open = false;
                                    }
                                }
                            } else {
                                if btn(ui, bullet, "NEW SECTOR", "Ctrl+Shift+N", t.success()) {
                                    self.new_folder_dialog = true;
                                    self.context_menu_open = false;
                                }
                                if btn(ui, bullet, "NEW CONSTRUCT", "Ctrl+N", t.success()) {
                                    self.new_file_dialog = true;
                                    self.context_menu_open = false;
                                }
                                if btn(ui, bullet, "INJECT", "Ctrl+V", t.text_primary()) {
                                    self.paste();
                                    self.context_menu_open = false;
                                }

                                divider(ui, t);

                                if btn(ui, bullet, "REFRESH", "F5", t.primary()) {
                                    self.load_current_directory();
                                    self.context_menu_open = false;
                                }
                                if btn(ui, bullet, "JACK IN", "", t.accent()) {
                                    self.open_terminal_here();
                                    self.context_menu_open = false;
                                }
                                if btn(ui, bullet, "TOGGLE HIDDEN", "Ctrl+H", t.text_dim()) {
                                    self.show_hidden = !self.show_hidden;
                                    self.load_current_directory();
                                    self.context_menu_open = false;
                                }

                                divider(ui, t);

                                if btn(ui, bullet, "DEEP SCAN", "Ctrl+G", t.primary()) {
                                    self.content_search_dialog = true;
                                    self.content_search_results.clear();
                                    self.context_menu_open = false;
                                }
                            }

                            ui.add_space(2.0);
                            ui.label(
                                RichText::new("\u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}")
                                    .color(t.border_dim())
                                    .monospace()
                                    .size(9.0),
                            );
                        });
                    });
                });

                // Dismiss on click outside (skip the frame menu was opened)
                let menu_rect = resp.response.rect;
                if self.context_menu_just_opened {
                    self.context_menu_just_opened = false;
                } else if ctx.input(|i| i.pointer.any_pressed()) {
                    if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                        if !menu_rect.contains(pos) {
                            self.context_menu_open = false;
                        }
                    }
                }
            }
        }
    }

    /// Render the SFTP/SSH remote connection dialog
    fn render_sftp_dialog(&mut self, ctx: &egui::Context) {
        let t = self.current_theme;

        let mut open = true;
        egui::Window::new(
            RichText::new("◈ NET RUNNER — REMOTE UPLINK")
                .color(t.primary())
                .monospace(),
        )
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(420.0)
        .min_width(380.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            let connected = self.sftp_connection.is_some();

            if !connected {
                // ── Connection Form ────────────────────────
                ui.add_space(4.0);
                ui.label(
                    RichText::new("TARGET NODE")
                        .color(t.text_dim())
                        .monospace()
                        .size(11.0),
                );
                ui.add_space(2.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("HOST:").color(t.text_dim()).monospace().size(12.0));
                    ui.add_sized(
                        [220.0, 20.0],
                        egui::TextEdit::singleline(&mut self.sftp_host)
                            .font(egui::FontId::monospace(12.0))
                            .text_color(t.text_primary()),
                    );
                    ui.label(RichText::new("PORT:").color(t.text_dim()).monospace().size(12.0));
                    ui.add_sized(
                        [50.0, 20.0],
                        egui::TextEdit::singleline(&mut self.sftp_port)
                            .font(egui::FontId::monospace(12.0))
                            .text_color(t.text_primary()),
                    );
                });

                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("USER:").color(t.text_dim()).monospace().size(12.0));
                    ui.add_sized(
                        [300.0, 20.0],
                        egui::TextEdit::singleline(&mut self.sftp_user)
                            .font(egui::FontId::monospace(12.0))
                            .text_color(t.text_primary()),
                    );
                });

                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("PASS:").color(t.text_dim()).monospace().size(12.0));
                    ui.add_sized(
                        [300.0, 20.0],
                        egui::TextEdit::singleline(&mut self.sftp_password)
                            .password(true)
                            .font(egui::FontId::monospace(12.0))
                            .text_color(t.text_primary()),
                    );
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            !self.sftp_busy,
                            egui::Button::new(
                                RichText::new("⟐ JACK IN (KEY AUTH)")
                                    .color(t.primary())
                                    .monospace(),
                            ),
                        )
                        .clicked()
                    {
                        self.start_sftp_connect(false);
                    }
                    if ui
                        .add_enabled(
                            !self.sftp_busy,
                            egui::Button::new(
                                RichText::new("⟐ JACK IN (PASSWORD)")
                                    .color(t.accent())
                                    .monospace(),
                            ),
                        )
                        .clicked()
                    {
                        self.start_sftp_connect(true);
                    }
                });
            } else {
                // ── Connected — Remote File Browser ────────
                ui.add_space(2.0);
                if !self.sftp_display_name.is_empty() {
                    ui.label(
                        RichText::new(format!("◉ UPLINK: {}", self.sftp_display_name))
                            .color(t.success())
                            .monospace()
                            .size(12.0),
                    );
                }

                if self.sftp_busy {
                    ui.label(
                        RichText::new(format!(
                            "⟳ {}",
                            self.sftp_operation_label
                        ))
                            .color(t.accent())
                            .monospace()
                            .size(10.0),
                    );
                }

                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("PATH:").color(t.text_dim()).monospace().size(12.0));
                    let resp = ui.add_sized(
                        [300.0, 20.0],
                        egui::TextEdit::singleline(&mut self.sftp_remote_path)
                            .font(egui::FontId::monospace(12.0))
                            .text_color(t.primary()),
                    );
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let path = self.sftp_remote_path.clone();
                        self.start_sftp_list_directory(path);
                    }
                    if ui
                        .add_enabled(
                            !self.sftp_busy,
                            egui::Button::new(RichText::new("⟳").color(t.primary()).monospace()),
                        )
                        .clicked()
                    {
                        let path = self.sftp_remote_path.clone();
                        self.start_sftp_list_directory(path);
                    }
                });

                // Back button
                ui.add_space(2.0);
                if self.sftp_remote_path != "/" {
                    if ui
                        .button(RichText::new("▲ UP LEVEL").color(t.text_dim()).monospace().size(11.0))
                        .clicked()
                    {
                        let parent = std::path::Path::new(&self.sftp_remote_path)
                            .parent()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| "/".to_string());
                        self.sftp_remote_path = if parent.is_empty() {
                            "/".to_string()
                        } else {
                            parent
                        };
                        let path = self.sftp_remote_path.clone();
                        self.start_sftp_list_directory(path);
                    }
                }

                ui.add_space(4.0);

                // Remote file listing
                let row_h = 20.0;
                let max_h = 300.0;
                egui::ScrollArea::vertical()
                    .max_height(max_h)
                    .show(ui, |ui| {
                        let entries_snapshot = self.sftp_remote_entries.clone();
                        if entries_snapshot.is_empty() {
                            ui.label(
                                RichText::new("  <empty sector>")
                                    .color(t.text_dim())
                                    .monospace()
                                    .size(11.0),
                            );
                        }
                        for entry in &entries_snapshot {
                            ui.horizontal(|ui| {
                                let icon = if entry.is_dir { "▸" } else { "◇" };
                                let color = if entry.is_dir {
                                    t.primary()
                                } else {
                                    t.text_primary()
                                };
                                let size_str = if entry.is_dir {
                                    String::new()
                                } else {
                                    bytesize::ByteSize(entry.size).to_string()
                                };

                                let label = format!("{} {}", icon, entry.name);
                                let resp = ui.add_sized(
                                    [280.0, row_h],
                                    egui::Label::new(
                                        RichText::new(label)
                                            .color(color)
                                            .monospace()
                                            .size(12.0),
                                    )
                                    .sense(egui::Sense::click()),
                                );

                                if !size_str.is_empty() {
                                    ui.label(
                                        RichText::new(size_str)
                                            .color(t.text_dim())
                                            .monospace()
                                            .size(11.0),
                                    );
                                }

                                if resp.double_clicked() {
                                    if entry.is_dir {
                                        self.sftp_remote_path = entry.path.clone();
                                        let path = entry.path.clone();
                                        self.start_sftp_list_directory(path);
                                    }
                                }

                                // Download button for files
                                if !entry.is_dir {
                                    if resp.clicked() {
                                        // Download to ~/Downloads
                                        if let Some(dl_dir) = dirs::download_dir() {
                                            let local_path =
                                                dl_dir.join(&entry.name);
                                            let remote = entry.path.clone();
                                            self.start_sftp_download(
                                                remote,
                                                local_path,
                                                entry.name.clone(),
                                            );
                                        }
                                    }
                                }
                            });
                        }
                    });

                ui.add_space(6.0);

                // Disconnect
                if ui
                    .button(
                        RichText::new("✕ DISCONNECT UPLINK")
                            .color(t.danger())
                            .monospace(),
                    )
                    .clicked()
                {
                    self.disconnect_sftp();
                }
            }

            // Error display
            if let Some(ref err) = self.sftp_error {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("⚠ {}", err))
                        .color(t.danger())
                        .monospace()
                        .size(11.0),
                );
            }
        });

        if !open {
            self.sftp_dialog = false;
        }
    }

    /// Render a radial hexagonal context menu — each option is a hex cell
    /// arranged in a circle around the cursor position.
    fn render_radial_hex_context_menu(
        &mut self,
        ctx: &egui::Context,
        t: crate::theme::CyberTheme,
        center_pos: egui::Pos2,
    ) {
        // Build menu items based on selection state
        struct RadialItem {
            label: &'static str,
            icon: &'static str,
            key_hint: &'static str,
            color: Color32,
            action_id: u8,
        }

        let has_selected = self.selected.is_some();
        let is_file = self.selected.and_then(|i| self.entries.get(i)).map(|e| !e.is_dir).unwrap_or(false);

        let mut items: Vec<RadialItem> = Vec::new();

        if has_selected {
            items.push(RadialItem { label: "EXECUTE", icon: "⬡", key_hint: "Enter", color: t.primary(), action_id: 1 });
            if is_file {
                items.push(RadialItem { label: "ROUTE TO", icon: "⎆", key_hint: "", color: t.accent(), action_id: 2 });
            }
            items.push(RadialItem { label: "REPLICATE", icon: "⧉", key_hint: "Ctrl+C", color: t.text_primary(), action_id: 3 });
            items.push(RadialItem { label: "EXTRACT", icon: "⬡", key_hint: "Ctrl+X", color: t.warning(), action_id: 4 });
            items.push(RadialItem { label: "INJECT", icon: "⬡", key_hint: "Ctrl+V", color: t.text_primary(), action_id: 5 });
            items.push(RadialItem { label: "RENAME", icon: "⟐", key_hint: "F2", color: t.accent(), action_id: 6 });
            items.push(RadialItem { label: "NEW DIR", icon: "⬡", key_hint: "Ctrl+Shift+N", color: t.success(), action_id: 7 });
            items.push(RadialItem { label: "NEW FILE", icon: "◇", key_hint: "Ctrl+N", color: t.success(), action_id: 12 });
            items.push(RadialItem { label: "DELETE", icon: "⦻", key_hint: "Del", color: t.danger(), action_id: 8 });
            items.push(RadialItem { label: "LINK", icon: "◇", key_hint: "", color: t.accent(), action_id: 13 });
        } else {
            items.push(RadialItem { label: "NEW DIR", icon: "⬡", key_hint: "Ctrl+Shift+N", color: t.success(), action_id: 7 });
            items.push(RadialItem { label: "NEW FILE", icon: "◇", key_hint: "Ctrl+N", color: t.success(), action_id: 12 });
            items.push(RadialItem { label: "INJECT", icon: "⬡", key_hint: "Ctrl+V", color: t.text_primary(), action_id: 5 });
            items.push(RadialItem { label: "REFRESH", icon: "⟳", key_hint: "F5", color: t.primary(), action_id: 9 });
            items.push(RadialItem { label: "JACK IN", icon: "⏚", key_hint: "", color: t.accent(), action_id: 10 });
            items.push(RadialItem { label: "HIDDEN", icon: "◌", key_hint: "Ctrl+H", color: t.text_dim(), action_id: 11 });
            items.push(RadialItem { label: "DEEP SCAN", icon: "⟐", key_hint: "Ctrl+G", color: t.primary(), action_id: 14 });
        }

        let item_count = items.len();
        let hex_radius: f32 = 48.0;
        // Scale ring radius based on item count so hexagons never overlap
        let min_gap = hex_radius * 2.2; // minimum center-to-center distance between adjacent hexes
        // For N items on a circle: chord = 2*R*sin(π/N) >= min_gap => R >= min_gap / (2*sin(π/N))
        let ring_radius: f32 = if item_count <= 1 {
            0.0
        } else {
            let angle_step = std::f32::consts::TAU / item_count as f32;
            (min_gap / (2.0 * (angle_step / 2.0).sin())).max(hex_radius * 2.5)
        };
        let total_canvas = (ring_radius + hex_radius) * 2.0 + 40.0;

        // Offset the area so center_pos is at the geometric center
        let area_origin = egui::pos2(
            center_pos.x - total_canvas / 2.0,
            center_pos.y - total_canvas / 2.0,
        );

        let resp = egui::Area::new(egui::Id::new("radial_hex_menu"))
            .fixed_pos(area_origin)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let (canvas_rect, _canvas_resp) = ui.allocate_exact_size(
                    egui::vec2(total_canvas, total_canvas),
                    egui::Sense::hover(),
                );

                let painter = ui.painter_at(canvas_rect);
                let cx = canvas_rect.min.x + total_canvas / 2.0;
                let cy = canvas_rect.min.y + total_canvas / 2.0;

                // Draw a subtle background circle
                painter.circle_filled(
                    egui::pos2(cx, cy),
                    ring_radius + hex_radius + 8.0,
                    Color32::from_rgba_premultiplied(
                        t.bg_dark().r(),
                        t.bg_dark().g(),
                        t.bg_dark().b(),
                        220,
                    ),
                );
                painter.circle_stroke(
                    egui::pos2(cx, cy),
                    ring_radius + hex_radius + 8.0,
                    egui::Stroke::new(1.0, t.border_dim()),
                );

                // Center label
                let center_label = if has_selected {
                    if let Some(idx) = self.selected {
                        if let Some(entry) = self.entries.get(idx) {
                            let name = if entry.name.len() > 8 {
                                format!("{}…", &entry.name[..7])
                            } else {
                                entry.name.clone()
                            };
                            name
                        } else {
                            "CELL".to_string()
                        }
                    } else {
                        "CELL".to_string()
                    }
                } else {
                    "HIVE".to_string()
                };

                // Center hex
                let center_hex = Self::radial_hex_points(cx, cy, 34.0);
                painter.add(egui::Shape::convex_polygon(
                    center_hex.clone(),
                    Color32::from_rgba_premultiplied(
                        t.primary().r(),
                        t.primary().g(),
                        t.primary().b(),
                        20,
                    ),
                    egui::Stroke::new(1.0, t.primary()),
                ));
                painter.text(
                    egui::pos2(cx, cy),
                    egui::Align2::CENTER_CENTER,
                    &center_label,
                    egui::FontId::monospace(10.0),
                    t.primary(),
                );

                // Get current pointer for hover detection
                let pointer_pos = ui.input(|i| i.pointer.interact_pos());

                let mut clicked_action: Option<u8> = None;

                // Place each item as a hexagon around the ring
                for (i, item) in items.iter().enumerate() {
                    let angle = std::f32::consts::TAU * i as f32 / item_count as f32
                        - std::f32::consts::FRAC_PI_2; // start from top
                    let hx = cx + ring_radius * angle.cos();
                    let hy = cy + ring_radius * angle.sin();

                    // Check hover
                    let is_hovered = pointer_pos.map_or(false, |pp| {
                        let dx = pp.x - hx;
                        let dy = pp.y - hy;
                        (dx * dx + dy * dy).sqrt() < hex_radius - 2.0
                    });

                    let hex_pts = Self::radial_hex_points(hx, hy, hex_radius - 2.0);

                    // Fill
                    let fill = if is_hovered {
                        Color32::from_rgba_premultiplied(
                            item.color.r(),
                            item.color.g(),
                            item.color.b(),
                            50,
                        )
                    } else {
                        Color32::from_rgba_premultiplied(
                            t.surface().r(),
                            t.surface().g(),
                            t.surface().b(),
                            200,
                        )
                    };

                    let border_color = if is_hovered { item.color } else { t.border_dim() };
                    let border_width = if is_hovered { 1.5 } else { 0.7 };

                    painter.add(egui::Shape::convex_polygon(
                        hex_pts.clone(),
                        fill,
                        egui::Stroke::NONE,
                    ));

                    let mut border_pts = hex_pts.clone();
                    border_pts.push(border_pts[0]);
                    painter.add(egui::Shape::line(
                        border_pts,
                        egui::Stroke::new(border_width, border_color),
                    ));

                    // Connecting line from center to hex
                    painter.line_segment(
                        [egui::pos2(cx, cy), egui::pos2(hx, hy)],
                        egui::Stroke::new(
                            0.3,
                            Color32::from_rgba_premultiplied(
                                t.border_dim().r(),
                                t.border_dim().g(),
                                t.border_dim().b(),
                                60,
                            ),
                        ),
                    );

                    // Icon
                    let icon_color = if is_hovered { item.color } else { Color32::from_rgba_premultiplied(item.color.r(), item.color.g(), item.color.b(), 160) };
                    painter.text(
                        egui::pos2(hx, hy - 10.0),
                        egui::Align2::CENTER_CENTER,
                        item.icon,
                        egui::FontId::monospace(18.0),
                        icon_color,
                    );

                    // Label
                    let label_color = if is_hovered { item.color } else { t.text_dim() };
                    painter.text(
                        egui::pos2(hx, hy + 8.0),
                        egui::Align2::CENTER_CENTER,
                        item.label,
                        egui::FontId::monospace(9.0),
                        label_color,
                    );

                    // Key hint (below label)
                    if !item.key_hint.is_empty() {
                        painter.text(
                            egui::pos2(hx, hy + 20.0),
                            egui::Align2::CENTER_CENTER,
                            item.key_hint,
                            egui::FontId::monospace(7.0),
                            Color32::from_rgba_premultiplied(
                                t.text_dim().r(),
                                t.text_dim().g(),
                                t.text_dim().b(),
                                120,
                            ),
                        );
                    }

                    // Check click
                    if is_hovered && ctx.input(|i| i.pointer.any_pressed()) {
                        clicked_action = Some(item.action_id);
                    }
                }

                clicked_action
            });

        // Handle action
        if let Some(action_id) = resp.inner {
            match action_id {
                1 => { // EXECUTE
                    if let Some(idx) = self.selected {
                        self.open_entry(idx);
                    }
                }
                2 => { // ROUTE TO
                    if let Some(idx) = self.selected {
                        if let Some(entry) = self.entries.get(idx) {
                            self.open_with_target = Some(entry.path.clone());
                            self.open_with_text.clear();
                            self.open_with_dialog = true;
                        }
                    }
                }
                3 => { // REPLICATE
                    self.copy_selected();
                }
                4 => { // EXTRACT
                    self.cut_selected();
                }
                5 => { // INJECT
                    self.paste();
                }
                6 => { // RENAME
                    if let Some(idx) = self.selected {
                        if let Some(entry) = self.entries.get(idx) {
                            self.rename_text = entry.name.clone();
                            self.rename_index = Some(idx);
                        }
                    }
                }
                7 => { // NEW DIR
                    self.new_folder_dialog = true;
                }
                8 => { // DELETE
                    self.request_delete();
                }
                9 => { // REFRESH
                    self.load_current_directory();
                }
                10 => { // JACK IN
                    self.open_terminal_here();
                }
                11 => { // TOGGLE HIDDEN
                    self.show_hidden = !self.show_hidden;
                    self.load_current_directory();
                }
                12 => { // NEW FILE
                    self.new_file_dialog = true;
                }
                13 => { // NEURAL LINK (symlink)
                    if let Some(idx) = self.selected {
                        if let Some(entry) = self.entries.get(idx) {
                            self.symlink_name = format!("{}_link", entry.name);
                        }
                    }
                    self.symlink_dialog = true;
                }
                14 => { // DEEP SCAN (content search)
                    self.content_search_dialog = true;
                    self.content_search_results.clear();
                }
                _ => {}
            }
            self.context_menu_open = false;
        }

        // Dismiss if clicked outside the radial menu (skip the frame menu was opened)
        let dismiss_radius = ring_radius + hex_radius + 12.0;
        if self.context_menu_just_opened {
            self.context_menu_just_opened = false;
        } else if ctx.input(|i| i.pointer.any_pressed()) {
            if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                let dx = pos.x - center_pos.x;
                let dy = pos.y - center_pos.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > dismiss_radius {
                    self.context_menu_open = false;
                }
            }
        }
    }

    /// Generate hex points for the radial menu (pointy-top for variety)
    fn radial_hex_points(cx: f32, cy: f32, radius: f32) -> Vec<egui::Pos2> {
        (0..6)
            .map(|i| {
                let angle = std::f32::consts::FRAC_PI_3 * i as f32 + std::f32::consts::FRAC_PI_6;
                egui::pos2(cx + radius * angle.cos(), cy + radius * angle.sin())
            })
            .collect()
    }

    // ── Split Pane Rendering ──────────────────────────────────

    fn render_split_pane(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;

        // Path bar
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("DUAL JACK ▸")
                    .color(t.accent())
                    .monospace()
                    .size(10.0)
                    .strong(),
            );
            let path_str = self.split_pane_path.to_string_lossy().to_string();
            ui.label(
                RichText::new(&path_str)
                    .color(t.text_dim())
                    .monospace()
                    .size(10.0),
            );
            if ui
                .small_button(RichText::new("▲").color(t.primary()).monospace())
                .on_hover_text("Go up")
                .clicked()
            {
                if let Some(parent) = self.split_pane_path.parent() {
                    let p = parent.to_path_buf();
                    self.split_pane_navigate(p);
                }
            }
            if ui
                .small_button(RichText::new("⟳").color(t.primary()).monospace())
                .on_hover_text("Refresh")
                .clicked()
            {
                self.load_split_pane_directory();
            }
            if ui
                .small_button(RichText::new("✕").color(t.danger()).monospace())
                .on_hover_text("Close Dual Jack (F4)")
                .clicked()
            {
                self.split_pane_active = false;
                self.split_pane_entries.clear();
                self.status_message = "Split view deactivated".into();
            }
        });
        ui.add_space(2.0);

        // File listing
        egui::ScrollArea::vertical().show(ui, |ui| {
            if self.split_pane_entries.is_empty() {
                ui.label(
                    RichText::new("[ SECTOR EMPTY ]")
                        .color(t.text_dim())
                        .monospace()
                        .size(11.0),
                );
                return;
            }

            let entries_snapshot: Vec<(usize, String, bool, bool, String, PathBuf)> = self
                .split_pane_entries
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    (
                        i,
                        e.name.clone(),
                        e.is_dir,
                        e.is_symlink,
                        e.formatted_size(),
                        e.path.clone(),
                    )
                })
                .collect();

            let mut nav_to: Option<PathBuf> = None;
            let mut open_file: Option<PathBuf> = None;

            for (idx, name, is_dir, is_symlink, size, path) in &entries_snapshot {
                let selected = self.split_pane_selected == Some(*idx);
                let icon = if *is_dir {
                    "◆"
                } else if *is_symlink {
                    "◇"
                } else {
                    "○"
                };
                let color = if selected {
                    t.accent()
                } else if *is_dir {
                    t.primary()
                } else {
                    t.text_primary()
                };

                let resp = ui.selectable_label(
                    selected,
                    RichText::new(format!(" {} {} {}", icon, name, if *is_dir { "" } else { &size }))
                        .color(color)
                        .monospace()
                        .size(11.0),
                );

                if resp.clicked() {
                    self.split_pane_selected = Some(*idx);
                }
                if resp.double_clicked() {
                    if *is_dir {
                        nav_to = Some(path.clone());
                    } else {
                        open_file = Some(path.clone());
                    }
                }

                // Drop zone: if dragging files, show visual
                if self.dragging && !self.drag_source_paths.is_empty() && *is_dir {
                    if resp.hovered() {
                        ui.painter().rect_stroke(
                            resp.rect,
                            0.0,
                            egui::Stroke::new(2.0, t.success()),
                            egui::StrokeKind::Outside,
                        );
                    }
                    if resp.hovered() && ui.input(|i| i.pointer.any_released()) {
                        // Drop: move files to this directory
                        let dest = path.clone();
                        let sources = self.drag_source_paths.clone();
                        let mut errors = Vec::new();
                        for src in &sources {
                            if let Err(e) = filesystem::move_file(src, &dest) {
                                errors.push(format!("{}", e));
                            }
                        }
                        if errors.is_empty() {
                            self.status_message = format!("Moved {} files", sources.len());
                        } else {
                            self.set_error(format!("Drop errors: {}", errors.join("; ")));
                        }
                        self.drag_source_paths.clear();
                        self.dragging = false;
                        self.load_current_directory();
                        self.load_split_pane_directory();
                    }
                }
            }

            if let Some(path) = nav_to {
                self.split_pane_navigate(path);
            }
            if let Some(path) = open_file {
                self.open_file(&path);
            }
        });
    }

    // ── Terminal Panel Rendering ──────────────────────────────

    fn render_terminal_panel(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("⏚ NEURAL JACK PORT")
                    .color(t.accent())
                    .monospace()
                    .size(11.0)
                    .strong(),
            );
            ui.label(
                RichText::new(format!("// {}", self.current_path.display()))
                    .color(t.text_dim())
                    .monospace()
                    .size(9.0),
            );
            if let Some(command) = &self.terminal_running_command {
                let elapsed_ms = self
                    .terminal_started_at
                    .map(|started| started.elapsed().as_millis())
                    .unwrap_or(0);
                ui.label(
                    RichText::new(format!("// RUNNING {} ({} ms)", command, elapsed_ms))
                        .color(t.warning())
                        .monospace()
                        .size(9.0),
                );
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button(RichText::new("✕").color(t.danger()).monospace())
                    .clicked()
                {
                    self.terminal_panel_visible = false;
                }
                if ui
                    .small_button(RichText::new("CLEAR").color(t.text_dim()).monospace().size(9.0))
                    .clicked()
                {
                    self.terminal_output.clear();
                }
            });
        });

        // Output area
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .max_height(ui.available_height() - 24.0)
            .show(ui, |ui| {
                for line in &self.terminal_output {
                    let color = if line.starts_with("[ERR]") {
                        t.danger()
                    } else if line.starts_with("$ ") {
                        t.primary()
                    } else {
                        t.text_primary()
                    };
                    ui.label(
                        RichText::new(line)
                            .color(color)
                            .monospace()
                            .size(11.0),
                    );
                }
            });

        // Input line
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("▸")
                    .color(t.success())
                    .monospace()
                    .size(12.0),
            );
            let resp = ui.add(
                egui::TextEdit::singleline(&mut self.terminal_input)
                    .font(egui::FontId::monospace(12.0))
                    .text_color(t.text_primary())
                    .desired_width(ui.available_width() - 60.0)
                    .hint_text(
                        RichText::new("enter command...")
                            .color(t.text_dim())
                            .monospace(),
                    ),
            );
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.run_terminal_command();
            }
            if ui
                .add_enabled(
                    !self.terminal_task_running,
                    egui::Button::new(
                        RichText::new(if self.terminal_task_running { "BUSY" } else { "RUN" })
                            .color(t.success())
                            .monospace()
                            .size(10.0),
                    ),
                )
                .clicked()
            {
                self.run_terminal_command();
            }
        });
    }

    fn apply_rename(&mut self) {
        if let Some(idx) = self.rename_index {
            if let Some(entry) = self.entries.get(idx) {
                let new_name = self.rename_text.trim();
                if !new_name.is_empty() {
                    if let Err(e) = filesystem::validate_entry_name(new_name) {
                        self.set_error(format!("Rename failed: {}", e));
                        self.rename_index = None;
                        self.rename_text.clear();
                        return;
                    }
                    let old_path = entry.path.clone();
                    let new_path = entry
                        .path
                        .parent()
                        .unwrap_or(&self.current_path)
                        .join(new_name);
                    if let Err(e) = std::fs::rename(&old_path, &new_path) {
                        self.set_error(format!("Rename failed: {}", e));
                    } else {
                        self.status_message =
                            format!("ID reassigned: {} \u{2192} {}", entry.name, new_name);
                        self.record_undo(UndoAction::Rename {
                            old_path,
                            new_path,
                        });
                        self.redo_stack.clear();
                        self.load_current_directory();
                    }
                }
            }
        }
        self.rename_index = None;
        self.rename_text.clear();
    }
}
