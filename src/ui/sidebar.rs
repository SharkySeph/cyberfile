use eframe::egui::{self, RichText};
use std::path::PathBuf;

use crate::app::CyberFile;
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
                // ── Quick Access ────────────────────────────────
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

                let mut nav_to: Option<PathBuf> = None;

                for (label, dir) in &quick_access {
                    if let Some(path) = dir {
                        let is_current = self.current_path == *path;
                        let text = RichText::new(*label)
                            .color(if is_current { t.primary() } else { t.text_primary() })
                            .monospace()
                            .size(12.5);

                        if ui.selectable_label(is_current, text).clicked() {
                            nav_to = Some(path.clone());
                        }
                    }
                }

                ui.add_space(8.0);
                cyber_separator_themed(ui, t.border_dim());
                ui.add_space(4.0);

                // ── Neural Links (Bookmarks) ───────────────────
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
                                .color(if is_current { t.primary() } else { t.text_primary() })
                                .monospace()
                                .size(12.0);

                            if ui.selectable_label(is_current, text).clicked() {
                                nav_to = Some(bm.clone());
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

                ui.add_space(8.0);
                cyber_separator_themed(ui, t.border_dim());
                ui.add_space(4.0);

                // ── Disk Info ──────────────────────────────────
                section_header(ui, "SYSTEM STATUS", t.primary());
                ui.add_space(4.0);

                // Show basic disk stats for current path (cached, refreshed every 10s)
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

                ui.add_space(8.0);
                cyber_separator_themed(ui, t.border_dim());
                ui.add_space(4.0);

                // ── Containment Zone (Trash) ─────────────────
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

                ui.add_space(8.0);
                cyber_separator_themed(ui, t.border_dim());
                ui.add_space(4.0);

                // ── Remote Access ──────────────────────────────
                section_header(ui, "NET RUNNER", t.primary());
                ui.add_space(4.0);

                if let Some(ref conn) = self.sftp_connection {
                    ui.label(
                        RichText::new(format!("  ◉ {}", conn.display_name()))
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

                ui.add_space(8.0);
                cyber_separator_themed(ui, t.border_dim());
                ui.add_space(4.0);

                // ── Music Widget ───────────────────────────────
                self.render_music_widget(ui);

                // Apply navigation from sidebar clicks
                if let Some(path) = nav_to {
                    self.navigate_to(path);
                }
            });
    }
}
