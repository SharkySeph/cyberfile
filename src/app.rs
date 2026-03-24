use eframe::egui::{self, Color32, RichText};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Instant;

use crate::config::Settings;
use crate::filesystem::{self, FileEntry, SortColumn};
use crate::integrations::media::MediaState;
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

    // ── UI State ─────────────────────────────────────────
    pub(crate) sidebar_visible: bool,
    pub(crate) show_hidden: bool,
    pub(crate) context_menu_open: bool,
    pub(crate) context_menu_pos: egui::Pos2,
    pub(crate) data_rain_cols: Vec<f32>,

    // ── Command Bar ──────────────────────────────────────
    pub(crate) command_bar_text: String,
    pub(crate) command_bar_active: bool,

    // ── Rename ───────────────────────────────────────────
    pub(crate) rename_index: Option<usize>,
    pub(crate) rename_text: String,

    // ── New Folder Dialog ────────────────────────────────
    pub(crate) new_folder_dialog: bool,
    pub(crate) new_folder_name: String,

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
    pub(crate) cpu_history: Vec<f32>,
    pub(crate) mem_history: Vec<f32>,

    // ── Settings Panel ───────────────────────────────────
    pub(crate) settings_panel_open: bool,

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
}

impl CyberFile {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let settings = Settings::load();
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let theme = CyberTheme::from_id(&settings.theme);

        let mut sys = sysinfo::System::new();
        sys.refresh_cpu_all();
        sys.refresh_memory();
        let disks = sysinfo::Disks::new_with_refreshed_list();

        Self {
            current_path: home.clone(),
            entries: Vec::new(),
            selected: None,
            multi_selected: HashSet::new(),
            history: vec![home.clone()],
            history_pos: 0,
            sort_column: SortColumn::Name,
            sort_ascending: true,
            sidebar_visible: true,
            show_hidden: settings.show_hidden,
            context_menu_open: false,
            context_menu_pos: egui::pos2(0.0, 0.0),
            data_rain_cols: (0..80).map(|i| (i as f32 * 7.77) % 100.0).collect(),
            command_bar_text: String::new(),
            command_bar_active: false,
            rename_index: None,
            rename_text: String::new(),
            new_folder_dialog: false,
            new_folder_name: String::new(),
            boot_complete: !settings.boot_sequence,
            boot_start: Instant::now(),
            clipboard_op: None,
            clipboard_paths: Vec::new(),
            status_message: "SYSTEM OPERATIONAL".into(),
            error_message: None,
            bookmarks: Vec::new(),
            current_theme: theme,
            scanlines_enabled: settings.scanlines_enabled,
            crt_effect: settings.crt_effect,
            glitch_active: false,
            glitch_start: None,
            resource_monitor_visible: false,
            sys_info: sys,
            sys_disks: disks,
            sys_last_refresh: Instant::now(),
            cpu_history: Vec::new(),
            mem_history: Vec::new(),
            settings_panel_open: false,
            settings,
            view_mode: ViewMode::List,
            tabs: vec![Tab {
                path: home.clone(),
                selected: None,
            }],
            active_tab: 0,
            preview_visible: false,
            data_rain_enabled: false,
            media_state: MediaState::default(),
            media_last_refresh: Instant::now(),
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
        }
    }

    // ── Navigation ────────────────────────────────────────────

    pub(crate) fn navigate_to(&mut self, path: PathBuf) {
        if !path.is_dir() {
            self.set_error(format!("Cannot navigate: not a sector ({})", path.display()));
            return;
        }

        self.current_path = path.clone();
        self.selected = None;
        self.multi_selected.clear();
        self.context_menu_open = false;

        self.history.truncate(self.history_pos + 1);
        self.history.push(path);
        self.history_pos = self.history.len() - 1;

        self.load_current_directory();
        self.command_bar_text = self.current_path.to_string_lossy().to_string();
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
        if let Some(term) = self.settings.resolved_terminal() {
            let result = std::process::Command::new(&term)
                .current_dir(&dir)
                .spawn();
            if let Err(e) = result {
                self.set_error(format!("Jack-in failed [{}]: {}", term, e));
            }
        } else {
            self.set_error("No terminal subsystem detected — configure in SYSTEM CONFIGURATION".into());
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
                if let Err(e) = filesystem::delete_to_trash(&entry.path) {
                    errors.push(format!("{}: {}", entry.name, e));
                }
            }
        }

        if !errors.is_empty() {
            self.set_error(format!("Quarantine errors: {}", errors.join("; ")));
        } else {
            self.status_message = "Constructs quarantined".into();
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

        for src in &paths {
            let result = match op {
                ClipboardOp::Copy => filesystem::copy_file(src, &dest).map(|_| ()),
                ClipboardOp::Cut => filesystem::move_file(src, &dest).map(|_| ()),
            };
            if let Err(e) = result {
                errors.push(format!("{}", e));
            }
        }

        if op == ClipboardOp::Cut {
            self.clipboard_paths.clear();
            self.clipboard_op = None;
        }

        if !errors.is_empty() {
            self.set_error(format!("Transfer errors: {}", errors.join("; ")));
        } else {
            self.status_message = "Data transfer complete".into();
        }

        self.load_current_directory();
    }

    pub(crate) fn execute_command(&mut self) {
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
            Ok(_) => {
                self.status_message = format!("Sector \"{}\" initialized", name);
                self.load_current_directory();
            }
            Err(e) => {
                self.set_error(format!("Failed to init sector: {}", e));
            }
        }

        self.new_folder_dialog = false;
        self.new_folder_name.clear();
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
        self.command_bar_text = self.current_path.to_string_lossy().to_string();
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
            self.cpu_history.push(cpu);
            if self.cpu_history.len() > 60 {
                self.cpu_history.remove(0);
            }

            let total_mem = self.sys_info.total_memory().max(1) as f32;
            let used_mem = self.sys_info.used_memory() as f32;
            let mem_pct = used_mem / total_mem * 100.0;
            self.mem_history.push(mem_pct);
            if self.mem_history.len() > 60 {
                self.mem_history.remove(0);
            }

            self.sys_last_refresh = Instant::now();
        }
    }

    pub(crate) fn trigger_glitch(&mut self) {
        self.glitch_active = true;
        self.glitch_start = Some(Instant::now());
    }

    // ── Keyboard Shortcuts ────────────────────────────────────

    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        if self.command_bar_active {
            return;
        }

        ctx.input(|input| {
            let ctrl = input.modifiers.ctrl;

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
            if ctrl && input.modifiers.shift && input.key_pressed(egui::Key::N) {
                self.new_folder_dialog = true;
            }
            if input.key_pressed(egui::Key::Delete) && self.selected.is_some() {
                self.delete_selected();
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
            }
            if input.key_pressed(egui::Key::F3) {
                self.resource_monitor_visible = !self.resource_monitor_visible;
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
                }
            }
            // Preview panel
            if ctrl && input.key_pressed(egui::Key::P) {
                self.preview_visible = !self.preview_visible;
            }
            // fzf interactive
            if ctrl && input.key_pressed(egui::Key::F) {
                self.fzf_interactive();
            }
            // Data rain toggle
            if input.key_pressed(egui::Key::F10) {
                self.data_rain_enabled = !self.data_rain_enabled;
            }
            if input.key_pressed(egui::Key::Escape) {
                self.context_menu_open = false;
                self.settings_panel_open = false;
            }
        });
    }
}

// ── eframe::App Implementation ────────────────────────────────────

impl eframe::App for CyberFile {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count += 1;

        // Apply theme
        if !self.theme_applied {
            theme::apply_cyber_theme(ctx, self.current_theme);
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

        // Refresh system info
        self.refresh_system_info();

        // Keyboard shortcuts
        self.handle_keyboard(ctx);

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

                // View mode switch
                match self.view_mode {
                    ViewMode::List => self.render_file_view(ui),
                    ViewMode::Grid => self.render_grid_view(ui),
                    ViewMode::HexGrid => self.render_hex_grid_view(ui),
                    ViewMode::Hex => self.render_hex_view(ui),
                }
            });

        // Overlays
        self.render_dialogs(ctx);

        if self.settings_panel_open {
            self.render_settings_panel(ctx);
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
    }
}

impl CyberFile {
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
                                        }
                                    }
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
                                if btn(ui, bullet, "NEW SECTOR", "Ctrl+N", t.success()) {
                                    self.new_folder_dialog = true;
                                    self.context_menu_open = false;
                                }
                                if btn(ui, bullet, "QUARANTINE", "Del", t.danger()) {
                                    self.delete_selected();
                                    self.context_menu_open = false;
                                }
                            } else {
                                if btn(ui, bullet, "NEW SECTOR", "Ctrl+N", t.success()) {
                                    self.new_folder_dialog = true;
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

                // Dismiss on click outside
                let menu_rect = resp.response.rect;
                if ctx.input(|i| i.pointer.any_pressed()) {
                    if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                        if !menu_rect.contains(pos) {
                            self.context_menu_open = false;
                        }
                    }
                }
            }
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
            items.push(RadialItem { label: "NEW DIR", icon: "⬡", key_hint: "Ctrl+N", color: t.success(), action_id: 7 });
            items.push(RadialItem { label: "DELETE", icon: "⦻", key_hint: "Del", color: t.danger(), action_id: 8 });
        } else {
            items.push(RadialItem { label: "NEW DIR", icon: "⬡", key_hint: "Ctrl+N", color: t.success(), action_id: 7 });
            items.push(RadialItem { label: "INJECT", icon: "⬡", key_hint: "Ctrl+V", color: t.text_primary(), action_id: 5 });
            items.push(RadialItem { label: "REFRESH", icon: "⟳", key_hint: "F5", color: t.primary(), action_id: 9 });
            items.push(RadialItem { label: "JACK IN", icon: "⏚", key_hint: "", color: t.accent(), action_id: 10 });
            items.push(RadialItem { label: "HIDDEN", icon: "◌", key_hint: "Ctrl+H", color: t.text_dim(), action_id: 11 });
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
                    self.delete_selected();
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
                _ => {}
            }
            self.context_menu_open = false;
        }

        // Dismiss if clicked outside the radial menu
        let dismiss_radius = ring_radius + hex_radius + 12.0;
        if ctx.input(|i| i.pointer.any_pressed()) {
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

    fn apply_rename(&mut self) {
        if let Some(idx) = self.rename_index {
            if let Some(entry) = self.entries.get(idx) {
                let new_name = self.rename_text.trim();
                if !new_name.is_empty() {
                    let new_path = entry
                        .path
                        .parent()
                        .unwrap_or(&self.current_path)
                        .join(new_name);
                    if let Err(e) = std::fs::rename(&entry.path, &new_path) {
                        self.set_error(format!("Rename failed: {}", e));
                    } else {
                        self.status_message =
                            format!("ID reassigned: {} \u{2192} {}", entry.name, new_name);
                        self.load_current_directory();
                    }
                }
            }
        }
        self.rename_index = None;
        self.rename_text.clear();
    }
}
