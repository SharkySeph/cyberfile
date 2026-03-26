use eframe::egui::{self, RichText, ScrollArea, Stroke};

use crate::app::CyberFile;
use crate::theme::*;

impl CyberFile {
    pub(crate) fn render_device_bay(&mut self, ctx: &egui::Context) {
        if self.device_bay_detached {
            let t = self.current_theme;
            let viewport_id = egui::ViewportId::from_hash_of("device_bay_viewport");
            let builder = egui::ViewportBuilder::default()
                .with_title("CYBERFILE // DEVICE BAY")
                .with_inner_size([700.0, 480.0])
                .with_min_inner_size([400.0, 300.0]);

            ctx.show_viewport_immediate(viewport_id, builder, |ctx, _class| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.device_bay_detached = false;
                    self.device_bay_open = false;
                }
                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(t.surface())
                            .inner_margin(egui::Margin::symmetric(10, 8)),
                    )
                    .show(ctx, |ui| {
                        self.render_device_bay_content(ui, true);
                    });
            });
        } else {
            let t = self.current_theme;
            let mut open = self.device_bay_open;

            egui::Window::new(
                RichText::new("┌─ DEVICE BAY ─┐")
                    .color(t.primary())
                    .monospace()
                    .strong(),
            )
            .open(&mut open)
            .default_width(680.0)
            .default_height(460.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(t.surface())
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .stroke(Stroke::new(1.0, t.border_dim())),
            )
            .show(ctx, |ui| {
                self.render_device_bay_content(ui, false);
            });

            self.device_bay_open = open;
        }
    }

    fn render_device_bay_content(&mut self, ui: &mut egui::Ui, detached: bool) {
        let t = self.current_theme;

        // ── Header Row ─────────────────────────────
        ui.horizontal(|ui| {
            section_header(ui, "DEVICE BAY", t.primary());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !detached {
                    if ui
                        .button(RichText::new("⏏").color(t.text_dim()).monospace())
                        .on_hover_text("Detach to viewport")
                        .clicked()
                    {
                        self.device_bay_detached = true;
                    }
                }
                if ui
                    .button(RichText::new("⟳").color(t.accent()).monospace())
                    .on_hover_text("Refresh")
                    .clicked()
                {
                    self.refresh_device_bay(true);
                }
            });
        });
        ui.add_space(4.0);

        // Auto-refresh every 5 seconds
        if self.device_last_refresh.elapsed().as_secs() >= 5 {
            self.refresh_device_bay(false);
        }

        if !self.device_udisksctl_available {
            ui.label(
                RichText::new("⚠ udisksctl not found — udisks2 required for mount/eject")
                    .color(t.text_dim())
                    .monospace(),
            );
        }

        if self.device_entries.is_empty() {
            ui.label(
                RichText::new("No block devices detected")
                    .color(t.text_dim())
                    .monospace(),
            );
            return;
        }

        // ── Filter row ─────────────────────────────
        ui.horizontal(|ui| {
            let show_all_label = if self.device_show_all { "SHOW ALL ●" } else { "REMOVABLE ONLY ○" };
            if ui
                .button(RichText::new(show_all_label).color(t.accent()).monospace().small())
                .clicked()
            {
                self.device_show_all = !self.device_show_all;
            }
        });
        ui.add_space(2.0);
        cyber_separator_themed(ui, t.border_dim());
        ui.add_space(4.0);

        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            let entries: Vec<_> = self
                .device_entries
                .clone()
                .into_iter()
                .filter(|d| self.device_show_all || d.removable || !d.mountpoint.is_empty())
                .collect();

            if entries.is_empty() {
                ui.label(
                    RichText::new("No matching devices (toggle filter)")
                        .color(t.text_dim())
                        .monospace(),
                );
                return;
            }

            for dev in &entries {
                ui.horizontal(|ui| {
                    let icon = if dev.removable {
                        "💾"
                    } else if dev.dev_type == "disk" {
                        "💿"
                    } else {
                        "📦"
                    };
                    ui.label(RichText::new(icon).monospace());

                    // Name
                    ui.label(
                        RichText::new(format!("/dev/{}", dev.name))
                            .color(t.text_primary())
                            .monospace()
                            .strong(),
                    );

                    // Label
                    if !dev.label.is_empty() {
                        ui.label(
                            RichText::new(format!("\"{}\"", dev.label))
                                .color(t.accent())
                                .monospace(),
                        );
                    }

                    // Size
                    ui.label(
                        RichText::new(&dev.size)
                            .color(t.text_dim())
                            .monospace(),
                    );

                    // Filesystem
                    if !dev.fstype.is_empty() {
                        ui.label(
                            RichText::new(format!("[{}]", dev.fstype))
                                .color(t.text_dim())
                                .monospace(),
                        );
                    }
                });

                // Mountpoint + actions
                ui.horizontal(|ui| {
                    ui.add_space(28.0);
                    if !dev.mountpoint.is_empty() {
                        ui.label(
                            RichText::new(format!("→ {}", dev.mountpoint))
                                .color(t.success())
                                .monospace(),
                        );
                        // Navigate to mountpoint
                        if ui
                            .button(RichText::new("OPEN").color(t.primary()).monospace().small())
                            .on_hover_text("Navigate to mountpoint")
                            .clicked()
                        {
                            self.navigate_to(std::path::PathBuf::from(&dev.mountpoint));
                        }
                        // Unmount
                        if self.device_udisksctl_available {
                            if ui
                                .button(RichText::new("UNMOUNT").color(t.warning()).monospace().small())
                                .clicked()
                            {
                                if let Err(e) = crate::integrations::devices::unmount_device(&dev.name) {
                                    self.device_error = Some(e);
                                }
                                self.refresh_device_bay(true);
                            }
                        }
                    } else if !dev.fstype.is_empty() && dev.dev_type != "disk" {
                        ui.label(
                            RichText::new("NOT MOUNTED")
                                .color(t.text_dim())
                                .monospace(),
                        );
                        if self.device_udisksctl_available {
                            if ui
                                .button(RichText::new("MOUNT").color(t.accent()).monospace().small())
                                .clicked()
                            {
                                match crate::integrations::devices::mount_device(&dev.name) {
                                    Ok(msg) => {
                                        self.device_error = Some(format!("✓ {}", msg));
                                    }
                                    Err(e) => {
                                        self.device_error = Some(e);
                                    }
                                }
                                self.refresh_device_bay(true);
                            }
                        }
                    }

                    // Eject for removable
                    if dev.removable && self.device_udisksctl_available {
                        if ui
                            .button(RichText::new("EJECT").color(t.danger()).monospace().small())
                            .on_hover_text("Power off device")
                            .clicked()
                        {
                            if let Err(e) = crate::integrations::devices::eject_device(&dev.name) {
                                self.device_error = Some(e);
                            }
                            self.refresh_device_bay(true);
                        }
                    }
                });

                // Model info
                if !dev.model.is_empty() {
                    ui.horizontal(|ui| {
                        ui.add_space(28.0);
                        ui.label(
                            RichText::new(format!("⊟ {}", dev.model))
                                .color(t.text_dim())
                                .monospace(),
                        );
                    });
                }

                ui.add_space(2.0);
            }

            // Error/status message
            if let Some(err) = &self.device_error {
                ui.add_space(4.0);
                cyber_separator_themed(ui, t.border_dim());
                ui.add_space(4.0);
                let color = if err.starts_with('✓') { t.success() } else { t.danger() };
                ui.label(RichText::new(err).color(color).monospace());
            }
        });
    }
}
