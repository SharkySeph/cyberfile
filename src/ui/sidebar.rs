use eframe::egui::{self, RichText};
use std::path::PathBuf;

use crate::app::CyberFile;
use crate::config::SidebarWidget;
use crate::theme::*;

impl CyberFile {
    pub(crate) fn render_sidebar(&mut self, ctx: &egui::Context) {
        let t = self.current_theme;
        egui::SidePanel::left("sidebar_panel")
            .default_width(self.settings.sidebar_width)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(t.surface())
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .stroke(egui::Stroke::new(1.0, t.border_dim())),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                let mut nav_to: Option<PathBuf> = None;

                // Build the ordered widget list from settings
                let layout: Vec<(SidebarWidget, bool)> = self
                    .settings
                    .sidebar_layout
                    .iter()
                    .map(|e| (e.widget, e.visible))
                    .collect();

                let mut first = true;
                for (widget, visible) in &layout {
                    if !visible {
                        continue;
                    }
                    if !first {
                        ui.add_space(8.0);
                        cyber_separator_themed(ui, t.border_dim());
                        ui.add_space(4.0);
                    }
                    first = false;
                    match widget {
                        SidebarWidget::QuickAccess => self.render_sidebar_quick_access(ui, &mut nav_to),
                        SidebarWidget::NeuralLinks => self.render_sidebar_neural_links(ui, &mut nav_to),
                        SidebarWidget::MissionScenes => self.render_sidebar_mission_scenes(ui),
                        SidebarWidget::SystemStatus => self.render_sidebar_system_status(ui),
                        SidebarWidget::ContainmentZone => self.render_sidebar_containment_zone(ui),
                        SidebarWidget::NetRunner => self.render_sidebar_net_runner(ui),
                        SidebarWidget::OperatorDeck => self.render_sidebar_operator_deck(ui),
                        SidebarWidget::MusicWidget => self.render_music_widget(ui),
                        SidebarWidget::NetworkMesh => self.render_sidebar_network_mesh(ui),
                        SidebarWidget::DeviceBay => self.render_sidebar_device_bay(ui),
                        SidebarWidget::WindowBridge => self.render_sidebar_window_bridge(ui),
                    }
                }

                // Apply navigation from sidebar clicks
                if let Some(path) = nav_to {
                    self.navigate_to(path);
                }
                }); // ScrollArea
            });
    }

    // ── Quick Access ────────────────────────────────────
    fn render_sidebar_quick_access(&self, ui: &mut egui::Ui, nav_to: &mut Option<PathBuf>) {
        let t = self.current_theme;
        section_header(ui, "QUICK ACCESS", t.primary());
        ui.add_space(4.0);

        let quick_access: Vec<(&str, Option<PathBuf>)> = vec![
            ("⌂  HOME", dirs::home_dir()),
            ("▼  DOWNLOADS", dirs::download_dir()),
            ("◆  DOCUMENTS", dirs::document_dir()),
            ("♫  AUDIO", dirs::audio_dir()),
            ("◈  IMAGES", dirs::picture_dir()),
            ("▸  VIDEOS", dirs::video_dir()),
            ("⚙  CONFIG", dirs::config_dir()),
            ("/  ROOT", Some(PathBuf::from("/"))),
        ];

        for (label, dir) in &quick_access {
            if let Some(path) = dir {
                let is_current = self.current_path == *path;
                let text = RichText::new(*label)
                    .color(if is_current { t.text_primary() } else { t.text_primary() })
                    .monospace()
                    .size(12.5);

                if ui.selectable_label(is_current, text).clicked() {
                    *nav_to = Some(path.clone());
                }
            }
        }
    }

    // ── Neural Links (Bookmarks) ───────────────────────
    fn render_sidebar_neural_links(&mut self, ui: &mut egui::Ui, nav_to: &mut Option<PathBuf>) {
        let t = self.current_theme;
        section_header(ui, "NEURAL LINKS", t.primary());
        ui.add_space(4.0);

        if self.bookmarks.is_empty() {
            ui.label(
                RichText::new("  No links saved")
                    .color(t.text_dim())
                    .monospace()
                    .size(11.0),
            );
        } else {
            let bookmarks_snapshot = self.bookmarks.clone();
            let mut remove_idx: Option<usize> = None;

            for (i, bm) in bookmarks_snapshot.iter().enumerate() {
                let label = bm
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| bm.to_string_lossy().to_string());

                let is_current = self.current_path == *bm;

                ui.horizontal(|ui| {
                    let text = RichText::new(format!("▪ {}", label))
                        .color(if is_current { t.text_primary() } else { t.text_primary() })
                        .monospace()
                        .size(12.0);

                    if ui.selectable_label(is_current, text).clicked() {
                        *nav_to = Some(bm.clone());
                    }

                    // Remove button
                    if ui
                        .small_button(RichText::new("✕").color(t.danger()).size(10.0))
                        .clicked()
                    {
                        remove_idx = Some(i);
                    }
                });
            }

            if let Some(idx) = remove_idx {
                self.bookmarks.remove(idx);
            }
        }

        ui.add_space(4.0);

        // Add current path as bookmark button
        if ui
            .button(
                RichText::new("+ SAVE NEURAL LINK")
                    .color(t.primary_dim())
                    .monospace()
                    .size(11.0),
            )
            .clicked()
        {
            let current = self.current_path.clone();
            if !self.bookmarks.contains(&current) {
                self.bookmarks.push(current);
            }
        }
    }

    // ── Mission Scenes ───────────────────────────────
    fn render_sidebar_mission_scenes(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;
        section_header(ui, "MISSION SCENES", t.primary());
        ui.add_space(4.0);

        let scene_rows: Vec<(String, String, bool)> = self
            .ordered_scene_indices()
            .into_iter()
            .take(4)
            .filter_map(|index| {
                self.scene_store.saved_scenes.get(index).map(|scene| {
                    (scene.id.clone(), scene.name.clone(), scene.pinned)
                })
            })
            .collect();

        if scene_rows.is_empty() {
            ui.label(
                RichText::new("  No mission scenes captured")
                    .color(t.text_dim())
                    .monospace()
                    .size(11.0),
            );
        } else {
            for (index, (scene_id, name, pinned)) in scene_rows.into_iter().enumerate() {
                let label = if pinned {
                    format!("{} ★ {}", index + 1, name)
                } else {
                    format!("{} ▪ {}", index + 1, name)
                };
                if ui
                    .button(
                        RichText::new(label)
                            .color(if pinned { t.warning() } else { t.text_primary() })
                            .monospace()
                            .size(11.0),
                    )
                    .clicked()
                {
                    self.restore_scene(&scene_id);
                }
            }
        }

        ui.add_space(4.0);
        ui.label(
            RichText::new("  Alt+1..4 restore quick scene slots")
                .color(t.text_dim())
                .monospace()
                .size(9.0),
        );
        ui.add_space(2.0);
        if ui
            .button(
                RichText::new("⟐ OPEN SCENE MANAGER")
                    .color(t.primary_dim())
                    .monospace()
                    .size(11.0),
            )
            .clicked()
        {
            self.open_scene_manager();
        }
    }

    // ── System Status (Disk Info) ──────────────────────
    fn render_sidebar_system_status(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;
        section_header(ui, "SYSTEM STATUS", t.primary());
        ui.add_space(4.0);

        let needs_refresh = self.disk_info_cache.is_none()
            || self.disk_info_last_refresh.elapsed().as_secs() >= 10
            || self.disk_info_path != self.current_path;

        if needs_refresh {
            if let Ok(output) = std::process::Command::new("df")
                .arg("-h")
                .arg("--output=size,used,avail,pcent")
                .arg(&self.current_path)
                .output()
            {
                let text = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = text.lines().collect();
                if lines.len() >= 2 {
                    let parts: Vec<&str> = lines[1].split_whitespace().collect();
                    if parts.len() >= 4 {
                        self.disk_info_cache = Some((
                            parts[0].to_string(),
                            parts[1].to_string(),
                            parts[2].to_string(),
                            parts[3].to_string(),
                        ));
                    }
                }
            }
            self.disk_info_last_refresh = std::time::Instant::now();
            self.disk_info_path = self.current_path.clone();
        }

        if let Some((total, used, free, load)) = &self.disk_info_cache {
            ui.label(
                RichText::new(format!("  TOTAL: {}", total))
                    .color(t.text_dim())
                    .monospace()
                    .size(11.0),
            );
            ui.label(
                RichText::new(format!("  USED:  {}", used))
                    .color(t.text_dim())
                    .monospace()
                    .size(11.0),
            );
            ui.label(
                RichText::new(format!("  FREE:  {}", free))
                    .color(t.success())
                    .monospace()
                    .size(11.0),
            );
            ui.label(
                RichText::new(format!("  LOAD:  {}", load))
                    .color(t.warning())
                    .monospace()
                    .size(11.0),
            );
        }

        if let Some(msg) = &self.error_message {
            ui.add_space(8.0);
            ui.label(
                RichText::new(format!("⚠ {}", msg.0))
                    .color(t.danger())
                    .monospace()
                    .size(11.0),
            );
        }
    }

    // ── Containment Zone (Trash) ─────────────────────
    fn render_sidebar_containment_zone(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;
        section_header(ui, "CONTAINMENT ZONE", t.primary());
        ui.add_space(4.0);

        let trash_count = crate::filesystem::list_trash().len();
        let trash_label = if trash_count == 0 {
            "  Zone clear".to_string()
        } else {
            format!("  {} quarantined", trash_count)
        };
        let trash_color = if trash_count == 0 { t.text_dim() } else { t.warning() };

        ui.label(
            RichText::new(&trash_label)
                .color(trash_color)
                .monospace()
                .size(11.0),
        );

        ui.add_space(4.0);
        if ui
            .button(
                RichText::new("⟐ OPEN CONTAINMENT")
                    .color(if trash_count > 0 { t.danger() } else { t.primary_dim() })
                    .monospace()
                    .size(11.0),
            )
            .clicked()
        {
            self.trash_entries = crate::filesystem::list_trash();
            self.trash_view_open = true;
        }
    }

    // ── Net Runner (Remote Access) ─────────────────────
    fn render_sidebar_net_runner(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;
        section_header(ui, "NET RUNNER", t.primary());
        ui.add_space(4.0);

        if self.sftp_connection.is_some() {
            ui.label(
                RichText::new(format!("  ◉ {}", self.sftp_display_name))
                    .color(t.success())
                    .monospace()
                    .size(11.0),
            );
            ui.add_space(2.0);
        } else {
            ui.label(
                RichText::new("  No active uplink")
                    .color(t.text_dim())
                    .monospace()
                    .size(11.0),
            );
            ui.add_space(2.0);
        }

        if ui
            .button(
                RichText::new("⟐ REMOTE NODE [F9]")
                    .color(if self.sftp_connection.is_some() {
                        t.success()
                    } else {
                        t.primary_dim()
                    })
                    .monospace()
                    .size(11.0),
            )
            .clicked()
        {
            self.sftp_dialog = true;
        }
    }

    // ── Operator Deck ──────────────────────────────────
    fn render_sidebar_operator_deck(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;
        section_header(ui, "OPERATOR DECK", t.primary());
        ui.add_space(4.0);

        if ui
            .button(
                RichText::new("⟐ PROCESS MATRIX [Ctrl+Shift+P]")
                    .color(if self.process_matrix_open { t.accent() } else { t.primary_dim() })
                    .monospace()
                    .size(11.0),
            )
            .clicked()
        {
            if self.process_matrix_open {
                self.process_matrix_open = false;
            } else {
                self.open_process_matrix();
            }
        }

        if ui
            .button(
                RichText::new("⟐ SERVICE DECK [Ctrl+D]")
                    .color(if self.service_deck_open { t.accent() } else { t.primary_dim() })
                    .monospace()
                    .size(11.0),
            )
            .clicked()
        {
            if self.service_deck_open {
                self.service_deck_open = false;
            } else {
                self.open_service_deck();
            }
        }

        if ui
            .button(
                RichText::new("⟐ LOG VIEWER [Ctrl+J]")
                    .color(if self.log_viewer_open { t.accent() } else { t.primary_dim() })
                    .monospace()
                    .size(11.0),
            )
            .clicked()
        {
            if self.log_viewer_open {
                self.log_viewer_open = false;
            } else {
                self.open_log_viewer();
            }
        }

        if ui
            .button(
                RichText::new("⟐ SIGNAL DECK [Ctrl+Shift+D]")
                    .color(if self.signal_deck_open { t.accent() } else { t.primary_dim() })
                    .monospace()
                    .size(11.0),
            )
            .clicked()
        {
            if self.signal_deck_open {
                self.signal_deck_open = false;
            } else {
                self.open_signal_deck();
            }
        }
    }

    // ── Network Mesh (sidebar widget) ───────────────────
    fn render_sidebar_network_mesh(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;
        section_header(ui, "NETWORK MESH", t.primary());
        ui.add_space(4.0);

        if !self.network_nmcli_available {
            ui.label(RichText::new("nmcli not available").color(t.text_dim()).monospace().size(11.0));
            return;
        }

        // Show connected interfaces summary
        for iface in &self.network_interfaces {
            if iface.state != "connected" {
                continue;
            }
            let icon = match iface.iface_type.as_str() {
                "wifi" => "📶",
                "ethernet" => "🔌",
                _ => "📡",
            };
            ui.label(
                RichText::new(format!("{} {} → {}", icon, iface.device, iface.connection))
                    .color(t.success())
                    .monospace()
                    .size(11.0),
            );
        }

        // Open full panel button
        if ui
            .button(
                RichText::new("⟐ NETWORK MESH [Ctrl+Shift+N]")
                    .color(if self.network_mesh_open { t.accent() } else { t.primary_dim() })
                    .monospace()
                    .size(11.0),
            )
            .clicked()
        {
            if self.network_mesh_open {
                self.network_mesh_open = false;
            } else {
                self.open_network_mesh();
            }
        }
    }

    // ── Device Bay (sidebar widget) ─────────────────────
    fn render_sidebar_device_bay(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;
        section_header(ui, "DEVICE BAY", t.primary());
        ui.add_space(4.0);

        // Show removable/mounted devices summary
        let removable: Vec<_> = self
            .device_entries
            .iter()
            .filter(|d| d.removable || !d.mountpoint.is_empty())
            .collect();

        if removable.is_empty() {
            ui.label(RichText::new("No removable devices").color(t.text_dim()).monospace().size(11.0));
        } else {
            for dev in removable.iter().take(5) {
                let label = if !dev.label.is_empty() {
                    format!("💾 {} \"{}\" {}", dev.name, dev.label, dev.size)
                } else {
                    format!("💾 {} {}", dev.name, dev.size)
                };
                let color = if dev.mountpoint.is_empty() { t.text_dim() } else { t.success() };
                ui.label(RichText::new(label).color(color).monospace().size(11.0));
            }
        }

        if ui
            .button(
                RichText::new("⟐ DEVICE BAY [Ctrl+Shift+B]")
                    .color(if self.device_bay_open { t.accent() } else { t.primary_dim() })
                    .monospace()
                    .size(11.0),
            )
            .clicked()
        {
            if self.device_bay_open {
                self.device_bay_open = false;
            } else {
                self.open_device_bay();
            }
        }
    }

    // ── Window Bridge (sidebar widget) ──────────────────
    fn render_sidebar_window_bridge(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;
        section_header(ui, "TACTICAL BRIDGE", t.primary());
        ui.add_space(4.0);

        if let Some(backend) = self.wm_backend {
            ui.label(
                RichText::new(format!("WM: {}", backend.label()))
                    .color(t.success())
                    .monospace()
                    .size(11.0),
            );

            let ws_count = self.wm_workspaces.len();
            let win_count = self.wm_windows.len();
            ui.label(
                RichText::new(format!("{} workspace{}, {} window{}",
                    ws_count, if ws_count == 1 { "" } else { "s" },
                    win_count, if win_count == 1 { "" } else { "s" },
                ))
                    .color(t.text_dim())
                    .monospace()
                    .size(11.0),
            );
        } else {
            ui.label(RichText::new("No WM detected").color(t.text_dim()).monospace().size(11.0));
        }

        if ui
            .button(
                RichText::new("⟐ TACTICAL BRIDGE [Ctrl+Shift+W]")
                    .color(if self.window_bridge_open { t.accent() } else { t.primary_dim() })
                    .monospace()
                    .size(11.0),
            )
            .clicked()
        {
            if self.window_bridge_open {
                self.window_bridge_open = false;
            } else {
                self.open_window_bridge();
            }
        }
    }
}
