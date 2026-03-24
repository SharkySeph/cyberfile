use eframe::egui::{self, RichText};

use crate::app::{CyberFile, ViewMode};
use crate::theme::*;

impl CyberFile {
    pub(crate) fn render_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar_panel")
            .frame(
                egui::Frame::new()
                    .fill(BG_DARK)
                    .inner_margin(egui::Margin::symmetric(10, 4))
                    .stroke(egui::Stroke::new(1.0, BORDER_DIM)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Current path
                    ui.label(
                        RichText::new(format!("[ {} ]", self.current_path.display()))
                            .color(CYAN_DIM)
                            .monospace()
                            .size(11.0),
                    );

                    ui.add_space(12.0);
                    ui.label(RichText::new("|").color(BORDER_DIM).monospace().size(11.0));
                    ui.add_space(12.0);

                    // Entry count
                    let count = self.entries.len();
                    ui.label(
                        RichText::new(format!("◈ {} constructs", count))
                            .color(TEXT_DIM)
                            .monospace()
                            .size(11.0),
                    );

                    ui.add_space(12.0);
                    ui.label(RichText::new("|").color(BORDER_DIM).monospace().size(11.0));
                    ui.add_space(12.0);

                    // Total size of visible files
                    let total_size: u64 = self.entries.iter().map(|e| e.size).sum();
                    ui.label(
                        RichText::new(format!("◈ {}", bytesize::ByteSize(total_size)))
                            .color(TEXT_DIM)
                            .monospace()
                            .size(11.0),
                    );

                    ui.add_space(12.0);
                    ui.label(RichText::new("|").color(BORDER_DIM).monospace().size(11.0));
                    ui.add_space(12.0);

                    // View mode indicator
                    let mode_label = match self.view_mode {
                        ViewMode::List => "LIST",
                        ViewMode::Grid => "GRID",
                        ViewMode::HexGrid => "HIVE",
                        ViewMode::Hex => "HEX",
                    };
                    ui.label(
                        RichText::new(format!("◈ {}", mode_label))
                            .color(CYAN)
                            .monospace()
                            .size(11.0),
                    );

                    // fzf indicator
                    if self.fzf_available {
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("FZF")
                                .color(MAGENTA)
                                .monospace()
                                .size(9.0),
                        );
                    }

                    // Right-aligned: selected info + clock
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            // Clock
                            let time = chrono::Local::now().format("%H:%M:%S");
                            ui.label(
                                RichText::new(format!("{}", time))
                                    .color(CYAN_DIM)
                                    .monospace()
                                    .size(11.0),
                            );

                            ui.add_space(12.0);
                            ui.label(
                                RichText::new("|")
                                    .color(BORDER_DIM)
                                    .monospace()
                                    .size(11.0),
                            );
                            ui.add_space(12.0);

                            // Status message or selection info
                            if let Some(idx) = self.selected {
                                if let Some(entry) = self.entries.get(idx) {
                                    ui.label(
                                        RichText::new(format!("◈ SELECTED: {}", entry.name))
                                            .color(MAGENTA)
                                            .monospace()
                                            .size(11.0),
                                    );
                                }
                            } else {
                                ui.label(
                                    RichText::new(&self.status_message)
                                        .color(TEXT_DIM)
                                        .monospace()
                                        .size(11.0),
                                );
                            }
                        },
                    );
                });
            });

        // Request repaint for clock
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}
