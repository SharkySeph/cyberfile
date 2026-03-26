use eframe::egui::{self, RichText, ScrollArea, Stroke, TextEdit};

use crate::app::CyberFile;
use crate::integrations::services::ServiceAction;

impl CyberFile {
    pub(crate) fn render_service_deck(&mut self, ctx: &egui::Context) {
        if self.service_deck_detached {
            let t = self.current_theme;
            let viewport_id = egui::ViewportId::from_hash_of("service_deck_viewport");
            let builder = egui::ViewportBuilder::default()
                .with_title("CYBERFILE // SERVICE DECK")
                .with_inner_size([740.0, 520.0])
                .with_min_inner_size([400.0, 300.0]);

            ctx.show_viewport_immediate(viewport_id, builder, |ctx, _class| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.service_deck_detached = false;
                    self.service_deck_open = false;
                }
                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(t.surface())
                            .inner_margin(egui::Margin::symmetric(10, 8)),
                    )
                    .show(ctx, |ui| {
                        self.render_service_deck_content(ui, true);
                    });
            });
        } else {
            let t = self.current_theme;
            let mut open = self.service_deck_open;

            egui::Window::new(
                RichText::new("┌─ SERVICE DECK ─┐")
                    .color(t.primary())
                    .monospace()
                    .strong(),
            )
            .open(&mut open)
            .default_width(720.0)
            .default_height(500.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(t.surface())
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .stroke(Stroke::new(1.0, t.border_dim())),
            )
            .show(ctx, |ui| {
                self.render_service_deck_content(ui, false);
            });

            self.service_deck_open = open;
        }
    }

    fn render_service_deck_content(&mut self, ui: &mut egui::Ui, detached: bool) {
        let t = self.current_theme;
            // ── Toolbar ────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("⟐ FILTER:")
                        .color(t.text_dim())
                        .monospace()
                        .size(11.0),
                );
                let filter = TextEdit::singleline(&mut self.service_filter_text)
                    .desired_width(200.0)
                    .font(egui::TextStyle::Monospace)
                    .text_color(t.text_primary());
                ui.add(filter);

                ui.separator();

                if ui
                    .button(
                        RichText::new("⟳ REFRESH")
                            .color(t.accent())
                            .monospace()
                            .size(10.0),
                    )
                    .clicked()
                {
                    self.refresh_service_deck(true);
                }

                ui.separator();

                let detach_label = if detached { "⬡ ATTACH" } else { "⬡ DETACH" };
                let detach_tip = if detached { "Dock back into main window" } else { "Open in separate window" };
                if ui
                    .button(RichText::new(detach_label).color(t.accent()).monospace().size(10.0))
                    .on_hover_text(detach_tip)
                    .clicked()
                {
                    self.service_deck_detached = !self.service_deck_detached;
                }
            });
            ui.add_space(4.0);

            // ── Column Headers ─────────────────────────────
            ui.horizontal(|ui| {
                let hw = 11.0;
                ui.label(
                    RichText::new(format!("{:<30}", "UNIT"))
                        .color(t.primary())
                        .monospace()
                        .size(hw),
                );
                ui.label(
                    RichText::new(format!("{:<8}", "ACTIVE"))
                        .color(t.primary())
                        .monospace()
                        .size(hw),
                );
                ui.label(
                    RichText::new(format!("{:<8}", "SUB"))
                        .color(t.primary())
                        .monospace()
                        .size(hw),
                );
                ui.label(
                    RichText::new(format!("{:<10}", "ENABLED"))
                        .color(t.primary())
                        .monospace()
                        .size(hw),
                );
                ui.label(
                    RichText::new("DESCRIPTION")
                        .color(t.primary())
                        .monospace()
                        .size(hw),
                );
            });
            ui.add_space(2.0);

            let entries = self.filtered_service_entries();
            let selected_unit = self.service_selected_unit.clone();

            // ── Split: service list (top) + status (bottom) ─
            let available = ui.available_height();
            let list_height = (available * 0.55).max(100.0);

            // Service list
            ScrollArea::vertical()
                .id_salt("svc_list")
                .max_height(list_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for entry in &entries {
                        let is_selected = selected_unit.as_deref() == Some(&entry.unit);
                        let text_color = if is_selected { t.primary() } else { t.text_primary() };
                        let active_color = match entry.active.as_str() {
                            "active" => t.accent(),
                            "inactive" => t.text_dim(),
                            "failed" => t.danger(),
                            _ => t.warning(),
                        };
                        let enabled_color = match entry.enabled.as_str() {
                            "enabled" => t.accent(),
                            "disabled" => t.text_dim(),
                            "static" => t.text_dim(),
                            _ => t.warning(),
                        };

                        let short_unit = if entry.unit.len() > 30 {
                            format!("{}…", &entry.unit[..29])
                        } else {
                            entry.unit.clone()
                        };

                        ui.horizontal(|ui| {
                            let response = ui.selectable_label(
                                is_selected,
                                RichText::new(format!("{:<30}", short_unit))
                                    .color(text_color)
                                    .monospace()
                                    .size(11.0),
                            );
                            if response.clicked() {
                                self.service_selected_unit = Some(entry.unit.clone());
                                self.inspect_service_unit(&entry.unit);
                            }

                            ui.label(
                                RichText::new(format!("{:<8}", entry.active))
                                    .color(active_color)
                                    .monospace()
                                    .size(11.0),
                            );
                            ui.label(
                                RichText::new(format!("{:<8}", entry.sub))
                                    .color(text_color)
                                    .monospace()
                                    .size(11.0),
                            );
                            ui.label(
                                RichText::new(format!("{:<10}", entry.enabled))
                                    .color(enabled_color)
                                    .monospace()
                                    .size(11.0),
                            );
                            ui.label(
                                RichText::new(&entry.description)
                                    .color(t.text_dim())
                                    .monospace()
                                    .size(10.0),
                            );
                        });
                    }
                });

            ui.add_space(4.0);

            // ── Actions ────────────────────────────────────
            ui.horizontal(|ui| {
                let has_selection = self.service_selected_unit.is_some();
                let actions = [
                    (ServiceAction::Start, "▶ START", t.accent()),
                    (ServiceAction::Stop, "■ STOP", t.warning()),
                    (ServiceAction::Restart, "⟳ RESTART", t.primary()),
                    (ServiceAction::Enable, "◉ ENABLE", t.accent()),
                    (ServiceAction::Disable, "○ DISABLE", t.text_dim()),
                ];
                for (action, label, color) in actions {
                    let btn_color = if has_selection { color } else { t.text_dim() };
                    if ui
                        .add_enabled(
                            has_selection,
                            egui::Button::new(
                                RichText::new(label).color(btn_color).monospace().size(10.0),
                            ),
                        )
                        .clicked()
                    {
                        self.control_selected_service(action);
                    }
                }

                ui.separator();

                // "View logs" button
                if ui
                    .add_enabled(
                        has_selection,
                        egui::Button::new(
                            RichText::new("⟐ LOGS")
                                .color(if has_selection { t.primary() } else { t.text_dim() })
                                .monospace()
                                .size(10.0),
                        ),
                    )
                    .clicked()
                {
                    if let Some(ref unit) = self.service_selected_unit {
                        let unit = unit.clone();
                        self.save_service_log_channel(&unit);
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{} services", entries.len()))
                            .color(t.text_dim())
                            .monospace()
                            .size(10.0),
                    );
                });
            });

            ui.add_space(4.0);

            // ── Status Output ──────────────────────────────
            if !self.service_status_output.is_empty() {
                ui.label(
                    RichText::new("── STATUS OUTPUT ──")
                        .color(t.primary())
                        .monospace()
                        .size(10.0)
                        .strong(),
                );
                ui.add_space(2.0);

                ScrollArea::vertical()
                    .id_salt("svc_status")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for line in &self.service_status_output {
                            let color = if line.contains("Active: active") {
                                t.accent()
                            } else if line.contains("Active: failed") || line.contains("[ERR]") {
                                t.danger()
                            } else if line.contains("Active: inactive") {
                                t.text_dim()
                            } else {
                                t.text_primary()
                            };
                            ui.label(
                                RichText::new(line).color(color).monospace().size(10.0),
                            );
                        }
                    });
            }
    }
}
