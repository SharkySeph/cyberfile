use eframe::egui::{self, RichText};

use crate::app::{CyberFile, ViewMode};
use crate::theme::*;

impl CyberFile {
    pub(crate) fn render_command_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("command_bar_panel")
            .frame(
                egui::Frame::new()
                    .fill(BG_DARK)
                    .inner_margin(egui::Margin::symmetric(10, 6))
                    .stroke(egui::Stroke::new(1.0, BORDER_DIM)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Title / brand
                    ui.label(
                        RichText::new("CYBERFILE")
                            .color(CYAN)
                            .monospace()
                            .size(14.0)
                            .strong(),
                    );
                    ui.label(
                        RichText::new("//")
                            .color(TEXT_DIM)
                            .monospace()
                            .size(14.0),
                    );

                    // Navigation buttons
                    if ui
                        .button(RichText::new("◀").color(CYAN).monospace().size(13.0))
                        .on_hover_text("Go back")
                        .clicked()
                    {
                        self.go_back();
                    }
                    if ui
                        .button(RichText::new("▶").color(CYAN).monospace().size(13.0))
                        .on_hover_text("Go forward")
                        .clicked()
                    {
                        self.go_forward();
                    }
                    if ui
                        .button(RichText::new("▲").color(CYAN).monospace().size(13.0))
                        .on_hover_text("Go up")
                        .clicked()
                    {
                        self.go_up();
                    }
                    if ui
                        .button(RichText::new("⟳").color(CYAN).monospace().size(13.0))
                        .on_hover_text("Refresh")
                        .clicked()
                    {
                        self.load_current_directory();
                    }

                    ui.add_space(8.0);

                    // Path / search input
                    ui.label(
                        RichText::new("▸")
                            .color(if self.command_bar_active { MAGENTA } else { CYAN })
                            .monospace()
                            .size(14.0),
                    );

                    let response = ui.add_sized(
                        [ui.available_width() - 120.0, 20.0],
                        egui::TextEdit::singleline(&mut self.command_bar_text)
                            .font(egui::FontId::monospace(13.0))
                            .text_color(TEXT_PRIMARY)
                            .hint_text(
                                RichText::new("NEURAL INTERFACE // type path or search query")
                                    .color(TEXT_DIM),
                            ),
                    );

                    if response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        self.execute_command();
                    }

                    self.command_bar_active = response.has_focus();

                    // Toggle hidden
                    ui.add_space(4.0);
                    let hidden_label = if self.show_hidden { "👁" } else { "◌" };
                    if ui
                        .button(
                            RichText::new(hidden_label)
                                .color(if self.show_hidden { YELLOW } else { TEXT_DIM })
                                .monospace()
                                .size(13.0),
                        )
                        .on_hover_text(if self.show_hidden {
                            "Hide cloaked files"
                        } else {
                            "Reveal cloaked files"
                        })
                        .clicked()
                    {
                        self.show_hidden = !self.show_hidden;
                        self.load_current_directory();
                    }

                    // Toggle sidebar
                    let sidebar_label = if self.sidebar_visible { "◧" } else { "▣" };
                    if ui
                        .button(
                            RichText::new(sidebar_label)
                                .color(CYAN_DIM)
                                .monospace()
                                .size(13.0),
                        )
                        .on_hover_text("Toggle network map")
                        .clicked()
                    {
                        self.sidebar_visible = !self.sidebar_visible;
                    }

                    // View mode switcher
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
                                    .color(if is_active { CYAN } else { TEXT_DIM })
                                    .monospace()
                                    .size(13.0),
                            )
                            .on_hover_text(tip)
                            .clicked()
                        {
                            self.view_mode = mode;
                        }
                    }

                    // Preview panel toggle
                    if ui
                        .button(
                            RichText::new("◫")
                                .color(if self.preview_visible { CYAN } else { TEXT_DIM })
                                .monospace()
                                .size(13.0),
                        )
                        .on_hover_text("Toggle data scan (Ctrl+P)")
                        .clicked()
                    {
                        self.preview_visible = !self.preview_visible;
                    }

                    // fzf button
                    if self.fzf_available {
                        if ui
                            .button(
                                RichText::new("⌕")
                                    .color(MAGENTA)
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
            });
    }
}
