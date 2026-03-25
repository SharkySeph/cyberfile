use eframe::egui::{self, RichText, ScrollArea, Stroke, TextEdit};

use crate::app::CyberFile;

impl CyberFile {
    pub(crate) fn render_log_viewer(&mut self, ctx: &egui::Context) {
        let t = self.current_theme;
        let mut open = self.log_viewer_open;

        egui::Window::new(
            RichText::new("┌─ LOG VIEWER ─┐")
                .color(t.primary())
                .monospace()
                .strong(),
        )
        .open(&mut open)
        .default_width(700.0)
        .default_height(420.0)
        .resizable(true)
        .frame(
            egui::Frame::new()
                .fill(t.surface())
                .inner_margin(egui::Margin::symmetric(10, 8))
                .stroke(Stroke::new(1.0, t.border_dim())),
        )
        .show(ctx, |ui| {
            // ── Channel Selector ───────────────────────────
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("⟐ CHANNEL:")
                        .color(t.text_dim())
                        .monospace()
                        .size(11.0),
                );

                let channels: Vec<_> = self.settings.log_channels.clone();
                for channel in &channels {
                    let is_selected = self.log_selected_channel_id.as_deref() == Some(&channel.id);
                    let color = if is_selected { t.primary() } else { t.text_dim() };
                    if ui
                        .button(
                            RichText::new(&channel.name)
                                .color(color)
                                .monospace()
                                .size(10.0),
                        )
                        .clicked()
                    {
                        self.log_selected_channel_id = Some(channel.id.clone());
                        self.refresh_log_viewer(true);
                    }
                }

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
                    self.refresh_log_viewer(true);
                }
            });
            ui.add_space(2.0);

            // ── Remove channel button (for non-default channels) ─
            if let Some(ref selected_id) = self.log_selected_channel_id.clone() {
                if selected_id != "journal.user" && selected_id != "journal.warnings" {
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                RichText::new("✕ REMOVE CHANNEL")
                                    .color(t.warning())
                                    .monospace()
                                    .size(10.0),
                            )
                            .clicked()
                        {
                            self.remove_log_channel(selected_id);
                        }
                    });
                    ui.add_space(2.0);
                }
            }

            // ── Log Output ─────────────────────────────────
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in &self.log_output {
                        let color = if line.contains("ERR") || line.contains("error") {
                            t.danger()
                        } else if line.contains("WARN") || line.contains("warning") {
                            t.warning()
                        } else if line.contains("INFO") || line.contains("notice") {
                            t.accent()
                        } else {
                            t.text_primary()
                        };
                        ui.label(
                            RichText::new(line).color(color).monospace().size(10.0),
                        );
                    }
                });

            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{} lines", self.log_output.len()))
                            .color(t.text_dim())
                            .monospace()
                            .size(10.0),
                    );
                });
            });
        });

        self.log_viewer_open = open;
    }
}
