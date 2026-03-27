use eframe::egui::{self, RichText};

use crate::app::{CyberFile, ViewMode};
use crate::launcher::CommandSurfaceMode;

impl CyberFile {
    pub(crate) fn render_command_bar(&mut self, ctx: &egui::Context) {
        let t = self.current_theme;
        egui::TopBottomPanel::top("command_bar_panel")
            .frame(
                egui::Frame::new()
                    .fill(t.bg_dark())
                    .inner_margin(egui::Margin::symmetric(10, 6))
                    .stroke(egui::Stroke::new(1.0, t.border_dim())),
            )
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("CYBERFILE")
                                .color(t.primary())
                                .monospace()
                                .size(14.0)
                                .strong(),
                        );
                        ui.label(
                            RichText::new("//")
                                .color(t.text_dim())
                                .monospace()
                                .size(14.0),
                        );

                        if ui
                            .button(RichText::new("◀").color(t.primary()).monospace().size(13.0))
                            .on_hover_text("Go back")
                            .clicked()
                        {
                            self.go_back();
                        }
                        if ui
                            .button(RichText::new("▶").color(t.primary()).monospace().size(13.0))
                            .on_hover_text("Go forward")
                            .clicked()
                        {
                            self.go_forward();
                        }
                        if ui
                            .button(RichText::new("▲").color(t.primary()).monospace().size(13.0))
                            .on_hover_text("Go up")
                            .clicked()
                        {
                            self.go_up();
                        }
                        if ui
                            .button(RichText::new("⟳").color(t.primary()).monospace().size(13.0))
                            .on_hover_text("Refresh")
                            .clicked()
                        {
                            self.load_current_directory();
                        }

                        ui.add_space(8.0);

                        let mode_label = self.command_surface_mode.label();
                        if ui
                            .button(
                                RichText::new(mode_label)
                                    .color(match self.command_surface_mode {
                                        CommandSurfaceMode::Path => t.primary(),
                                        CommandSurfaceMode::Protocol => t.accent(),
                                    })
                                    .monospace()
                                    .size(11.0),
                            )
                            .on_hover_text("Toggle command surface mode (Ctrl+L / Ctrl+K)")
                            .clicked()
                        {
                            let next_mode = match self.command_surface_mode {
                                CommandSurfaceMode::Path => CommandSurfaceMode::Protocol,
                                CommandSurfaceMode::Protocol => CommandSurfaceMode::Path,
                            };
                            self.set_command_surface_mode(next_mode);
                        }

                        ui.label(
                            RichText::new("▸")
                                .color(if self.command_bar_active { t.accent() } else { t.primary() })
                                .monospace()
                                .size(14.0),
                        );

                        let input_width = (ui.available_width() - 150.0).max(180.0);
                        let response = ui.add_sized(
                            [input_width, 20.0],
                            egui::TextEdit::singleline(&mut self.command_bar_text)
                                .font(egui::FontId::monospace(13.0))
                                .text_color(t.text_primary())
                                .hint_text(
                                    RichText::new(self.command_surface_mode.hint())
                                        .color(t.text_dim()),
                                ),
                        );

                        if self.focus_command_bar_next_frame {
                            ui.memory_mut(|mem| mem.request_focus(response.id));
                            self.focus_command_bar_next_frame = false;
                        }

                        if response.changed() && self.command_surface_mode == CommandSurfaceMode::Protocol {
                            self.launcher_selected = 0;
                            self.refresh_launcher_results();
                        }

                        if response.has_focus() && self.command_surface_mode == CommandSurfaceMode::Protocol {
                            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown))
                                && !self.launcher_results.is_empty()
                            {
                                self.launcher_selected =
                                    (self.launcher_selected + 1).min(self.launcher_results.len() - 1);
                            }
                            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp))
                                && !self.launcher_results.is_empty()
                            {
                                self.launcher_selected = self.launcher_selected.saturating_sub(1);
                            }
                        }

                        if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.execute_command();
                        }

                        // Tab-complete paths in Path mode
                        if response.has_focus()
                            && self.command_surface_mode == CommandSurfaceMode::Path
                            && ui.input(|i| i.key_pressed(egui::Key::Tab))
                        {
                            self.tab_complete_path();
                        }

                        self.command_bar_active = response.has_focus();

                        ui.add_space(4.0);
                        let hidden_label = if self.show_hidden { "👁" } else { "◌" };
                        if ui
                            .button(
                                RichText::new(hidden_label)
                                    .color(if self.show_hidden { t.warning() } else { t.text_dim() })
                                    .monospace()
                                    .size(13.0),
                            )
                            .on_hover_text(if self.show_hidden {
                                "Hide hidden files"
                            } else {
                                "Show hidden files"
                            })
                            .clicked()
                        {
                            self.show_hidden = !self.show_hidden;
                            self.load_current_directory();
                        }

                        let sidebar_label = if self.sidebar_visible { "◧" } else { "▣" };
                        if ui
                            .button(
                                RichText::new(sidebar_label)
                                    .color(t.primary_dim())
                                    .monospace()
                                    .size(13.0),
                            )
                            .on_hover_text("Toggle network map")
                            .clicked()
                        {
                            self.sidebar_visible = !self.sidebar_visible;
                        }

                        ui.add_space(4.0);
                        let modes = [
                            (ViewMode::List, "≡", "List view (Ctrl+1)"),
                            (ViewMode::Grid, "▦", "Grid view (Ctrl+2)"),
                            (ViewMode::HexGrid, "⬡", "Hex grid (Ctrl+3)"),
                            (ViewMode::Hex, "⟨⟩", "Hex viewer (Ctrl+4)"),
                        ];
                        for (mode, icon, tip) in modes {
                            let is_active = self.view_mode == mode;
                            if ui
                                .button(
                                    RichText::new(icon)
                                        .color(if is_active { t.primary() } else { t.text_dim() })
                                        .monospace()
                                        .size(13.0),
                                )
                                .on_hover_text(tip)
                                .clicked()
                            {
                                self.view_mode = mode;
                            }
                        }

                        if ui
                            .button(
                                RichText::new("◫")
                                    .color(if self.preview_visible { t.primary() } else { t.text_dim() })
                                    .monospace()
                                    .size(13.0),
                            )
                            .on_hover_text("Toggle data scan (Ctrl+P)")
                            .clicked()
                        {
                            self.preview_visible = !self.preview_visible;
                        }

                        if self.fzf_available {
                            if ui
                                .button(
                                    RichText::new("⌕")
                                        .color(t.accent())
                                        .monospace()
                                        .size(13.0),
                                )
                                .on_hover_text("fzf search (Ctrl+F)")
                                .clicked()
                            {
                                self.fzf_interactive();
                            }
                        }
                    });

                    if self.command_surface_mode == CommandSurfaceMode::Protocol
                        && self.command_bar_active
                    {
                        ui.add_space(6.0);
                        egui::Frame::new()
                            .fill(t.surface())
                            .inner_margin(egui::Margin::symmetric(8, 6))
                            .stroke(egui::Stroke::new(1.0, t.border_dim()))
                            .show(ui, |ui| {
                                let quick_slots = self.boot_scene_slots();
                                if !quick_slots.is_empty() {
                                    let slot_hint = quick_slots
                                        .iter()
                                        .enumerate()
                                        .map(|(index, scene)| {
                                            format!("Alt+{} {}", index + 1, scene.name)
                                        })
                                        .collect::<Vec<_>>()
                                        .join("  |  ");
                                    ui.label(
                                        RichText::new(slot_hint)
                                            .color(t.text_dim())
                                            .monospace()
                                            .size(9.5),
                                    );
                                    ui.add_space(4.0);
                                }

                                if self.launcher_results.is_empty() {
                                    ui.label(
                                        RichText::new("No protocols matched the current query")
                                            .color(t.text_dim())
                                            .monospace()
                                            .size(10.5),
                                    );
                                } else {
                                    let mut clicked_action = None;
                                    for (index, entry) in self.launcher_results.iter().enumerate() {
                                        let selected = index == self.launcher_selected;
                                        if ui
                                            .selectable_label(
                                                selected,
                                                RichText::new(format!(
                                                    "{} // {}",
                                                    entry.section, entry.title
                                                ))
                                                .color(if selected { t.primary() } else { t.text_primary() })
                                                .monospace()
                                                .size(11.0),
                                            )
                                            .clicked()
                                        {
                                            clicked_action = Some((index, entry.action.clone()));
                                        }
                                        ui.label(
                                            RichText::new(&entry.subtitle)
                                                .color(t.text_dim())
                                                .monospace()
                                                .size(10.0),
                                        );
                                        if index + 1 < self.launcher_results.len() {
                                            ui.add_space(3.0);
                                        }
                                    }

                                    if let Some((index, action)) = clicked_action {
                                        self.launcher_selected = index;
                                        self.execute_launcher_action(action);
                                    }
                                }
                            });
                    }
                });
            });
    }
}
