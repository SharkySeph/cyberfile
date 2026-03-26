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

        let filter_lower = self.filter_text.to_lowercase();
        let cells: Vec<GridCell> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                self.filter_text.is_empty()
                    || e.name.to_lowercase().contains(&filter_lower)
            })
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

        // Load thumbnails for visible image files (max 2 per frame)
        let image_exts = ["PNG", "JPG", "JPEG", "GIF", "BMP", "WEBP"];
        let mut loaded_this_frame = 0;
        for cell in &cells {
            if loaded_this_frame >= 2 {
                break;
            }
            if image_exts.contains(&cell.ext.as_str()) {
                if let Some(entry) = self.entries.get(cell.index) {
                    let path = entry.path.clone();
                    if !self.thumbnail_cache.contains_key(&path)
                        && !self.thumbnail_failed.contains(&path)
                    {
                        if let Ok(img) = image::open(&path) {
                            let thumb = img.thumbnail(96, 96);
                            let rgba = thumb.to_rgba8();
                            let size = [rgba.width() as usize, rgba.height() as usize];
                            let color_image =
                                egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                            let texture = ui.ctx().load_texture(
                                format!("thumb_{}", path.display()),
                                color_image,
                                egui::TextureOptions::LINEAR,
                            );
                            self.thumbnail_cache.insert(path, texture);
                        } else {
                            self.thumbnail_failed.insert(path);
                        }
                        loaded_this_frame += 1;
                    }
                }
            }
        }

        // Snapshot thumbnail info for use inside closure
        let thumb_textures: std::collections::HashMap<usize, (egui::TextureId, egui::Vec2)> = cells
            .iter()
            .filter_map(|cell| {
                self.entries.get(cell.index).and_then(|entry| {
                    self.thumbnail_cache.get(&entry.path).map(|tex| {
                        (cell.index, (tex.id(), tex.size_vec2()))
                    })
                })
            })
            .collect();

        let current_selected = self.selected;
        let current_multi = self.multi_selected.clone();
        let ctrl_held = ui.input(|i| i.modifiers.ctrl);
        let shift_held = ui.input(|i| i.modifiers.shift);
        let mut open_index: Option<usize> = None;
        let mut context_index: Option<usize> = None;
        let mut blank_space_menu = false;
        let mut click_action: Option<(usize, bool, bool)> = None;

        let available_width = ui.available_width();
        let card_width: f32 = 140.0;
        let card_height: f32 = 110.0;
        let cols = ((available_width / (card_width + 8.0)) as usize).max(1);

        // Collect card rects for rubber band hit-testing
        let mut card_rects: Vec<(usize, egui::Rect)> = Vec::new();

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
                            let is_sel = current_selected == Some(cell.index)
                                || current_multi.contains(&cell.index);

                            let (border_col, fill) = if is_sel {
                                (
                                    t.accent(),
                                    Color32::from_rgba_premultiplied(
                                        t.accent().r(),
                                        t.accent().g(),
                                        t.accent().b(),
                                        50,
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

                            // Large icon or thumbnail
                            if let Some(&(tex_id, tex_size)) = thumb_textures.get(&cell.index) {
                                let available = egui::vec2(card_width - 12.0, 50.0);
                                let scale = (available.x / tex_size.x)
                                    .min(available.y / tex_size.y)
                                    .min(1.0);
                                let display_size = tex_size * scale;
                                child_ui.image(egui::load::SizedTexture::new(
                                    tex_id,
                                    display_size,
                                ));
                            } else {
                                child_ui.label(
                                    RichText::new(icon)
                                        .color(icon_color)
                                        .monospace()
                                        .size(24.0),
                                );
                            }

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

                            // Store card rect for rubber band hit-testing
                            card_rects.push((cell.index, card_rect));

                            if card_response.clicked() {
                                click_action = Some((cell.index, ctrl_held, shift_held));
                            }
                            if card_response.double_clicked() {
                                open_index = Some(cell.index);
                            }
                            if card_response.secondary_clicked() {
                                context_index = Some(cell.index);
                            }

                            // Drag initiation: start dragging when card is dragged
                            if card_response.dragged() && !self.rubber_band_active {
                                if !self.dragging {
                                    self.dragging = true;
                                    self.drag_source_paths.clear();
                                    if self.multi_selected.contains(&cell.index) {
                                        for &idx in &self.multi_selected {
                                            if let Some(e) = self.entries.get(idx) {
                                                self.drag_source_paths.push(e.path.clone());
                                            }
                                        }
                                    } else if let Some(e) = self.entries.get(cell.index) {
                                        self.drag_source_paths.push(e.path.clone());
                                    }
                                }
                            }

                            card_response.on_hover_text(format!(
                                "{}\n{} │ {}",
                                cell.name, cell.size, cell.ext
                            ));

                            // Small gap between cards
                            ui.add_space(4.0);
                        }
                    });
                    ui.add_space(4.0);
                }

                // Blank space interaction
                let remaining = ui.available_rect_before_wrap();
                let blank_resp = ui.allocate_rect(remaining, egui::Sense::click_and_drag());
                if blank_resp.clicked() {
                    click_action = Some((usize::MAX, false, false));
                }
                if blank_resp.secondary_clicked() {
                    blank_space_menu = true;
                }

                // Rubber band: start on primary drag from blank space
                if blank_resp.drag_started_by(egui::PointerButton::Primary) {
                    if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
                        self.rubber_band_start = Some(pos);
                        self.rubber_band_active = true;
                    }
                }

                // Rubber band: draw rectangle while dragging
                if self.rubber_band_active {
                    if let Some(start) = self.rubber_band_start {
                        if let Some(current) = ui.ctx().input(|i| i.pointer.interact_pos()) {
                            let band_rect = egui::Rect::from_two_pos(start, current);
                            // Draw the selection rectangle
                            ui.painter().rect(
                                band_rect,
                                0.0,
                                Color32::from_rgba_premultiplied(
                                    t.accent().r(),
                                    t.accent().g(),
                                    t.accent().b(),
                                    30,
                                ),
                                Stroke::new(1.0, t.accent()),
                                StrokeKind::Outside,
                            );
                        }
                    }
                }
            });

        // Rubber band: finalize selection on release
        if self.rubber_band_active {
            let released = ui.ctx().input(|i| !i.pointer.primary_down());
            if released {
                if let Some(start) = self.rubber_band_start {
                    if let Some(current) = ui.ctx().input(|i| i.pointer.interact_pos()) {
                        let band_rect = egui::Rect::from_two_pos(start, current);
                        // Select all cards that intersect the rubber band
                        self.multi_selected.clear();
                        for &(idx, card_rect) in &card_rects {
                            if band_rect.intersects(card_rect) {
                                self.multi_selected.insert(idx);
                            }
                        }
                        if let Some(&first) = self.multi_selected.iter().next() {
                            self.selected = Some(first);
                        }
                    }
                }
                self.rubber_band_start = None;
                self.rubber_band_active = false;
            }
        }

        // Cancel drag when mouse released
        if self.dragging && ui.ctx().input(|i| !i.pointer.primary_down()) {
            self.dragging = false;
            self.drag_source_paths.clear();
        }

        // Apply
        if let Some((idx, ctrl, shift)) = click_action {
            if idx == usize::MAX {
                self.selected = None;
                self.multi_selected.clear();
            } else if ctrl {
                if self.multi_selected.contains(&idx) {
                    self.multi_selected.remove(&idx);
                } else {
                    self.multi_selected.insert(idx);
                }
            } else if shift {
                let anchor = self.selected.unwrap_or(0);
                let start = anchor.min(idx);
                let end = anchor.max(idx);
                self.multi_selected.clear();
                for i in start..=end {
                    self.multi_selected.insert(i);
                }
            } else {
                self.multi_selected.clear();
                self.selected = Some(idx);
            }
        }
        if let Some(i) = open_index {
            self.open_entry(i);
        }
        if let Some(i) = context_index {
            self.selected = Some(i);
            self.context_menu_open = true;
            self.context_menu_just_opened = true;
            self.context_menu_pos = ui.ctx().input(|inp| {
                inp.pointer.interact_pos().unwrap_or(egui::pos2(100.0, 100.0))
            });
        }
        if blank_space_menu {
            self.selected = None;
            self.context_menu_open = true;
            self.context_menu_just_opened = true;
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
