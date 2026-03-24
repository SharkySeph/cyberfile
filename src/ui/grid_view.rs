use eframe::egui::{self, Color32, RichText, Stroke, epaint::StrokeKind};

use crate::app::CyberFile;

/// Pre-collected display data for a grid cell
struct GridCell {
    index: usize,
    name: String,
    is_dir: bool,
    is_symlink: bool,
    is_hidden: bool,
    size: String,
    ext: String,
}

impl CyberFile {
    /// Grid view — "CONSTRUCT ARRAY" mode
    /// Displays files as NERV-style data cards in a grid
    pub(crate) fn render_grid_view(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;

        // Header
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("┌─ CONSTRUCT ARRAY ─── GRID MODE ───┐")
                    .color(t.primary())
                    .monospace()
                    .size(10.0)
                    .strong(),
            );
        });
        ui.add_space(4.0);

        let cells: Vec<GridCell> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, e)| GridCell {
                index: i,
                name: e.name.clone(),
                is_dir: e.is_dir,
                is_symlink: e.is_symlink,
                is_hidden: e.is_hidden,
                size: e.formatted_size(),
                ext: e
                    .path
                    .extension()
                    .map(|x| x.to_string_lossy().to_uppercase())
                    .unwrap_or_default(),
            })
            .collect();

        let current_selected = self.selected;
        let mut new_selected = self.selected;
        let mut open_index: Option<usize> = None;
        let mut context_index: Option<usize> = None;
        let mut blank_space_menu = false;

        let available_width = ui.available_width();
        let card_width: f32 = 140.0;
        let card_height: f32 = 110.0;
        let cols = ((available_width / (card_width + 8.0)) as usize).max(1);

        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                if cells.is_empty() {
                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("[ SECTOR EMPTY ]")
                                .color(t.text_dim())
                                .monospace()
                                .size(16.0),
                        );
                    });
                    return;
                }

                // Render as rows of cards
                let rows = cells.chunks(cols);
                for row_cells in rows {
                    ui.horizontal(|ui| {
                        for cell in row_cells {
                            let is_sel = current_selected == Some(cell.index);

                            let (border_col, fill) = if is_sel {
                                (
                                    t.accent(),
                                    Color32::from_rgba_premultiplied(
                                        t.accent().r(),
                                        t.accent().g(),
                                        t.accent().b(),
                                        25,
                                    ),
                                )
                            } else {
                                (t.border_dim(), t.surface())
                            };

                            // Allocate a fixed-size region for each card
                            let (card_rect, card_response) = ui.allocate_exact_size(
                                egui::vec2(card_width, card_height),
                                egui::Sense::click(),
                            );

                            // Draw card background + border
                            let stroke_width = if is_sel { 1.5 } else { 0.5 };
                            ui.painter().rect(
                                card_rect,
                                0.0,
                                fill,
                                Stroke::new(stroke_width, border_col),
                                StrokeKind::Outside,
                            );

                            // Render card contents in a child UI within the fixed rect
                            let inner_margin = 6.0;
                            let inner_rect = card_rect.shrink(inner_margin);
                            let mut child_ui = ui.new_child(
                                egui::UiBuilder::new()
                                    .max_rect(inner_rect)
                                    .layout(egui::Layout::top_down(egui::Align::Center)),
                            );

                            // Type icon (large, centered)
                            let (icon, icon_color) = if cell.is_dir {
                                ("◆", t.primary())
                            } else if cell.is_symlink {
                                ("◇", t.accent())
                            } else {
                                self.file_type_icon(&cell.ext, t)
                            };

                            child_ui.add_space(4.0);

                            // Extension badge or DIR label
                            if cell.is_dir {
                                egui::Frame::new()
                                    .fill(Color32::from_rgba_premultiplied(
                                        t.primary().r(),
                                        t.primary().g(),
                                        t.primary().b(),
                                        25,
                                    ))
                                    .stroke(Stroke::new(0.5, t.primary()))
                                    .inner_margin(egui::Margin::symmetric(4, 1))
                                    .show(&mut child_ui, |ui| {
                                        ui.label(
                                            RichText::new("SECTOR")
                                                .color(t.primary())
                                                .monospace()
                                                .size(8.0),
                                        );
                                    });
                            } else if !cell.ext.is_empty() {
                                egui::Frame::new()
                                    .fill(Color32::from_rgba_premultiplied(
                                        t.warning().r(),
                                        t.warning().g(),
                                        t.warning().b(),
                                        15,
                                    ))
                                    .inner_margin(egui::Margin::symmetric(3, 0))
                                    .show(&mut child_ui, |ui| {
                                        ui.label(
                                            RichText::new(&cell.ext)
                                                .color(t.warning())
                                                .monospace()
                                                .size(8.0),
                                        );
                                    });
                            }

                            child_ui.add_space(4.0);

                            // Large icon
                            child_ui.label(
                                RichText::new(icon)
                                    .color(icon_color)
                                    .monospace()
                                    .size(24.0),
                            );

                            child_ui.add_space(4.0);

                            // Name (truncated)
                            let display_name = if cell.name.len() > 16 {
                                format!("{}…", &cell.name[..15])
                            } else {
                                cell.name.clone()
                            };

                            let name_color = if cell.is_dir {
                                t.primary()
                            } else if cell.is_hidden {
                                t.text_dim()
                            } else {
                                t.text_primary()
                            };

                            child_ui.label(
                                RichText::new(&display_name)
                                    .color(name_color)
                                    .monospace()
                                    .size(10.0),
                            );

                            // Size
                            if !cell.is_dir {
                                child_ui.label(
                                    RichText::new(&cell.size)
                                        .color(t.text_dim())
                                        .monospace()
                                        .size(9.0),
                                );
                            }

                            if card_response.clicked() {
                                new_selected = Some(cell.index);
                            }
                            if card_response.double_clicked() {
                                open_index = Some(cell.index);
                            }
                            if card_response.secondary_clicked() {
                                context_index = Some(cell.index);
                            }

                            // Small gap between cards
                            ui.add_space(4.0);
                        }
                    });
                    ui.add_space(4.0);
                }

                // Blank space interaction
                let remaining = ui.available_rect_before_wrap();
                let blank_resp = ui.allocate_rect(remaining, egui::Sense::click());
                if blank_resp.clicked() {
                    new_selected = None;
                }
                if blank_resp.secondary_clicked() {
                    blank_space_menu = true;
                }
            });

        // Apply
        self.selected = new_selected;
        if let Some(i) = open_index {
            self.open_entry(i);
        }
        if let Some(i) = context_index {
            self.selected = Some(i);
            self.context_menu_open = true;
            self.context_menu_pos = ui.ctx().input(|inp| {
                inp.pointer.interact_pos().unwrap_or(egui::pos2(100.0, 100.0))
            });
        }
        if blank_space_menu {
            self.selected = None;
            self.context_menu_open = true;
            self.context_menu_pos = ui.ctx().input(|inp| {
                inp.pointer.interact_pos().unwrap_or(egui::pos2(100.0, 100.0))
            });
        }
    }

    pub(crate) fn file_type_icon(&self, ext: &str, t: crate::theme::CyberTheme) -> (&'static str, Color32) {
        match ext {
            "RS" | "PY" | "JS" | "TS" | "C" | "CPP" | "H" | "GO" | "JAVA" | "RB" | "LUA" => {
                ("⟨⟩", t.success())
            }
            "TXT" | "MD" | "LOG" | "CSV" => ("▤", t.text_primary()),
            "TOML" | "YAML" | "YML" | "JSON" | "XML" | "INI" | "CONF" => ("⚙", t.warning()),
            "PNG" | "JPG" | "JPEG" | "GIF" | "SVG" | "BMP" | "WEBP" => ("◫", t.accent()),
            "MP3" | "FLAC" | "WAV" | "OGG" | "M4A" | "AAC" => ("♫", t.success()),
            "MP4" | "MKV" | "AVI" | "WEBM" | "MOV" => ("▶", t.accent()),
            "ZIP" | "TAR" | "GZ" | "BZ2" | "XZ" | "7Z" | "RAR" | "ZST" => ("◰", t.warning()),
            "PDF" => ("▥", t.danger()),
            "SH" | "BASH" | "ZSH" | "FISH" => ("▸", t.primary()),
            _ => ("◇", t.text_dim()),
        }
    }
}
