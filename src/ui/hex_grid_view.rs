use eframe::egui::{self, Color32, RichText, Stroke};

use crate::app::CyberFile;

/// Pre-collected display data for a hex grid cell
struct HexCell {
    index: usize,
    name: String,
    is_dir: bool,
    is_symlink: bool,
    is_hidden: bool,
    size: String,
    ext: String,
}

/// Draw a regular hexagon (flat-top) centered at (cx, cy) with given radius.
fn hex_points(cx: f32, cy: f32, radius: f32) -> Vec<egui::Pos2> {
    (0..6)
        .map(|i| {
            let angle = std::f32::consts::FRAC_PI_3 * i as f32;
            egui::pos2(cx + radius * angle.cos(), cy + radius * angle.sin())
        })
        .collect()
}

/// Compute circular/spiral hex positions using axial hex coordinates.
/// Ring 0 = center (1 cell), ring 1 = 6 cells, ring 2 = 12 cells, etc.
/// Returns (offset_x, offset_y) relative to the center of the layout.
fn circular_hex_positions(count: usize, hex_radius: f32) -> Vec<(f32, f32)> {
    if count == 0 {
        return vec![];
    }

    let mut positions = Vec::with_capacity(count);

    // For flat-top hexagons, axial to pixel:
    //   x = hex_radius * (3/2 * q)
    //   y = hex_radius * (sqrt(3)/2 * q + sqrt(3) * r)
    let sqrt3 = 3.0_f32.sqrt();

    let axial_to_pixel = |q: i32, r: i32| -> (f32, f32) {
        let x = hex_radius * (1.5 * q as f32);
        let y = hex_radius * (sqrt3 * 0.5 * q as f32 + sqrt3 * r as f32);
        (x, y)
    };

    // Ring 0: center
    positions.push((0.0, 0.0));
    if positions.len() >= count {
        return positions;
    }

    // Axial direction vectors for traversing hex rings (flat-top)
    let axial_dirs: [(i32, i32); 6] = [
        (1, 0),
        (0, 1),
        (-1, 1),
        (-1, 0),
        (0, -1),
        (1, -1),
    ];

    let mut ring = 1;
    'outer: loop {
        // Start at axial coordinate (ring, -ring) which is the "top-right" corner
        let mut q: i32 = ring as i32;
        let mut r: i32 = -(ring as i32);

        for (_dir_idx, &(dq, dr)) in axial_dirs.iter().enumerate() {
            for _step in 0..ring {
                let (px, py) = axial_to_pixel(q, r);
                positions.push((px, py));
                if positions.len() >= count {
                    break 'outer;
                }
                q += dq;
                r += dr;
            }
        }

        ring += 1;
        if ring > 100 {
            break;
        }
    }

    positions
}

impl CyberFile {
    /// Hexagonal grid view — "HIVE ARRAY" mode
    /// Displays files as hexagonal cells in a circular honeycomb layout
    pub(crate) fn render_hex_grid_view(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;
        let zoom = self.hex_zoom;

        // Header with zoom controls
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("┌─ HIVE ARRAY ─── HEX GRID MODE ───┐")
                    .color(t.primary())
                    .monospace()
                    .size(10.0)
                    .strong(),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new(format!("ZOOM: {:.0}%", zoom * 100.0))
                    .color(t.text_dim())
                    .monospace()
                    .size(9.0),
            );
            if ui
                .selectable_label(
                    false,
                    RichText::new(" ⊖ ").color(t.primary()).monospace().size(10.0),
                )
                .clicked()
            {
                self.hex_zoom = (self.hex_zoom - 0.15).max(0.3);
            }
            if ui
                .selectable_label(
                    false,
                    RichText::new(" ⊕ ").color(t.primary()).monospace().size(10.0),
                )
                .clicked()
            {
                self.hex_zoom = (self.hex_zoom + 0.15).min(3.0);
            }
            if ui
                .selectable_label(
                    false,
                    RichText::new(" ⊙ ").color(t.text_dim()).monospace().size(10.0),
                )
                .on_hover_text("Reset zoom & pan (Ctrl+0)")
                .clicked()
            {
                self.hex_zoom = 1.0;
                self.hex_pan_offset = egui::Vec2::ZERO;
            }
        });
        ui.add_space(4.0);

        let filter_lower = self.filter_text.to_lowercase();
        let cells: Vec<HexCell> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                self.filter_text.is_empty()
                    || e.name.to_lowercase().contains(&filter_lower)
            })
            .map(|(i, e)| HexCell {
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
        let current_multi = self.multi_selected.clone();
        let ctrl_held = ui.input(|i| i.modifiers.ctrl);
        let shift_held = ui.input(|i| i.modifiers.shift);
        let mut open_index: Option<usize> = None;
        let mut context_index: Option<usize> = None;
        let mut blank_space_menu = false;
        let mut click_action: Option<(usize, bool, bool)> = None;

        // Hex geometry — flat-top hexagons, scaled by zoom
        let hex_radius: f32 = 56.0 * zoom;
        let _hex_h = hex_radius * 3.0_f32.sqrt();

        // Compute circular positions
        let positions = circular_hex_positions(cells.len(), hex_radius);

        // Pan to selected hex if type-ahead just selected it
        if let Some(sel_idx) = current_selected {
            if !self.type_ahead_buffer.is_empty()
                && self.type_ahead_last_key.elapsed().as_millis() < 600
            {
                // Find the position index matching the selected entry index
                if let Some(pos_i) = cells.iter().position(|c| c.index == sel_idx) {
                    if let Some(&(ox, oy)) = positions.get(pos_i) {
                        // Pan so the selected hex is at center (offset=0 means center)
                        self.hex_pan_offset = egui::vec2(-ox, -oy);
                    }
                }
            }
        }

        egui::ScrollArea::both()
            .auto_shrink(false)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                // Handle Ctrl+scroll zoom
                let scroll_delta = ui.input(|i| {
                    if i.modifiers.ctrl {
                        i.raw_scroll_delta.y
                    } else {
                        0.0
                    }
                });
                if scroll_delta != 0.0 {
                    self.hex_zoom = (self.hex_zoom + scroll_delta * 0.002).clamp(0.3, 3.0);
                }

                if cells.is_empty() {
                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("[ HIVE EMPTY ]")
                                .color(t.text_dim())
                                .monospace()
                                .size(16.0),
                        );
                    });
                    return;
                }

                // Find bounding box of all positions to compute canvas size
                let mut min_x: f32 = 0.0;
                let mut max_x: f32 = 0.0;
                let mut min_y: f32 = 0.0;
                let mut max_y: f32 = 0.0;
                for &(px, py) in &positions {
                    min_x = min_x.min(px);
                    max_x = max_x.max(px);
                    min_y = min_y.min(py);
                    max_y = max_y.max(py);
                }

                let padding = hex_radius * 1.5 + 30.0;
                let content_width = (max_x - min_x) + padding * 2.0;
                let content_height = (max_y - min_y) + padding * 2.0;

                let available_width = ui.available_width();
                let available_height = ui.available_height();
                let canvas_width = content_width.max(available_width);
                let canvas_height = content_height.max(available_height);

                let center_x = canvas_width / 2.0 + self.hex_pan_offset.x;
                let center_y = canvas_height / 2.0 + self.hex_pan_offset.y;

                // Reserve space and get painter — use click + drag sense
                let (canvas_rect, canvas_response) = ui.allocate_exact_size(
                    egui::vec2(canvas_width, canvas_height),
                    egui::Sense::click_and_drag(),
                );

                // Drag-to-pan
                if canvas_response.dragged_by(egui::PointerButton::Middle)
                    || (canvas_response.dragged_by(egui::PointerButton::Primary)
                        && ui.input(|i| i.modifiers.shift))
                {
                    self.hex_pan_offset += canvas_response.drag_delta();
                }

                let painter = ui.painter_at(canvas_rect);
                let origin = canvas_rect.min;

                // Draw faint ring guides
                let max_ring = {
                    let mut r = 0;
                    let mut total = 1;
                    while total < cells.len() {
                        r += 1;
                        total += 6 * r;
                    }
                    r
                };
                let sqrt3 = 3.0_f32.sqrt();
                // Distance from center to the center of a hex at ring N:
                // Ring N has hexes at distance N * hex_radius * sqrt(3) (for flat-top)
                for ring in 1..=max_ring {
                    let guide_radius = ring as f32 * hex_radius * sqrt3;
                    painter.circle_stroke(
                        egui::pos2(origin.x + center_x, origin.y + center_y),
                        guide_radius,
                        Stroke::new(
                            0.3,
                            Color32::from_rgba_premultiplied(
                                t.border_dim().r(),
                                t.border_dim().g(),
                                t.border_dim().b(),
                                40,
                            ),
                        ),
                    );
                }

                // Place hexagons in circular pattern
                for (i, cell) in cells.iter().enumerate() {
                    if i >= positions.len() {
                        break;
                    }

                    let (offset_x, offset_y) = positions[i];
                    let cx = origin.x + center_x + offset_x;
                    let cy = origin.y + center_y + offset_y;

                    let is_sel = current_selected == Some(cell.index)
                        || current_multi.contains(&cell.index);

                    let points = hex_points(cx, cy, hex_radius - 2.0);

                    // Fill color
                    let fill = if is_sel {
                        Color32::from_rgba_premultiplied(
                            t.accent().r(),
                            t.accent().g(),
                            t.accent().b(),
                            50,
                        )
                    } else {
                        t.surface()
                    };

                    let border_col = if is_sel {
                        t.accent()
                    } else {
                        t.border_dim()
                    };
                    let stroke_width = if is_sel { 1.5 } else { 0.5 };

                    // Draw hex fill
                    painter.add(egui::Shape::convex_polygon(
                        points.clone(),
                        fill,
                        Stroke::NONE,
                    ));

                    // Draw hex border
                    let mut border_points = points.clone();
                    border_points.push(border_points[0]);
                    painter.add(egui::Shape::line(
                        border_points,
                        Stroke::new(stroke_width, border_col),
                    ));

                    // ── Inner content (scaled by zoom) ──
                    let icon_size = (20.0 * zoom).max(10.0);
                    let name_size = (9.0 * zoom).max(6.0);
                    let badge_size = (7.0 * zoom).max(5.0);

                    // Type icon
                    let (icon, icon_color) = if cell.is_dir {
                        ("◆", t.primary())
                    } else if cell.is_symlink {
                        ("◇", t.accent())
                    } else {
                        self.file_type_icon(&cell.ext, t)
                    };

                    painter.text(
                        egui::pos2(cx, cy - 14.0 * zoom),
                        egui::Align2::CENTER_CENTER,
                        icon,
                        egui::FontId::monospace(icon_size),
                        icon_color,
                    );

                    // Name (truncated, adapts to zoom)
                    let max_chars = (12.0 * zoom).max(4.0) as usize;
                    let display_name = if cell.name.len() > max_chars {
                        let end = cell.name.char_indices()
                            .nth(max_chars.saturating_sub(1))
                            .map(|(i, _)| i)
                            .unwrap_or(cell.name.len());
                        format!("{}…", &cell.name[..end])
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

                    painter.text(
                        egui::pos2(cx, cy + 8.0 * zoom),
                        egui::Align2::CENTER_CENTER,
                        &display_name,
                        egui::FontId::monospace(name_size),
                        name_color,
                    );

                    // Extension badge or SECTOR label
                    if cell.is_dir {
                        painter.text(
                            egui::pos2(cx, cy - 30.0 * zoom),
                            egui::Align2::CENTER_CENTER,
                            "SECTOR",
                            egui::FontId::monospace(badge_size),
                            Color32::from_rgba_premultiplied(
                                t.primary().r(),
                                t.primary().g(),
                                t.primary().b(),
                                160,
                            ),
                        );
                    } else if !cell.ext.is_empty() {
                        painter.text(
                            egui::pos2(cx, cy - 30.0 * zoom),
                            egui::Align2::CENTER_CENTER,
                            &cell.ext,
                            egui::FontId::monospace(badge_size),
                            Color32::from_rgba_premultiplied(
                                t.warning().r(),
                                t.warning().g(),
                                t.warning().b(),
                                180,
                            ),
                        );
                    }

                    // Size (files only)
                    if !cell.is_dir {
                        painter.text(
                            egui::pos2(cx, cy + 20.0 * zoom),
                            egui::Align2::CENTER_CENTER,
                            &cell.size,
                            egui::FontId::monospace(badge_size),
                            Color32::from_rgba_premultiplied(
                                t.text_dim().r(),
                                t.text_dim().g(),
                                t.text_dim().b(),
                                180,
                            ),
                        );
                    }

                    // Hit testing
                    if let Some(click_pos) = canvas_response.interact_pointer_pos() {
                        let dx = click_pos.x - cx;
                        let dy = click_pos.y - cy;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist < hex_radius - 4.0 {
                            if canvas_response.clicked() {
                                click_action = Some((cell.index, ctrl_held, shift_held));
                            }
                            if canvas_response.double_clicked() {
                                open_index = Some(cell.index);
                            }
                            if canvas_response.secondary_clicked() {
                                context_index = Some(cell.index);
                            }
                        }
                    }

                    // Hover tooltip
                    if let Some(hover_pos) = canvas_response.hover_pos() {
                        let dx = hover_pos.x - cx;
                        let dy = hover_pos.y - cy;
                        if (dx * dx + dy * dy).sqrt() < hex_radius - 4.0 {
                            let tip = if cell.is_dir {
                                format!("{}\nDIR │ {}", cell.name, cell.ext)
                            } else {
                                format!("{}\n{} │ {}", cell.name, cell.size, cell.ext)
                            };
                            canvas_response.clone().on_hover_text_at_pointer(tip);
                        }
                    }
                }

                // If clicked but didn't hit any hex, deselect
                if canvas_response.clicked() && click_action.is_none() {
                    let mut hit_any = false;
                    if let Some(click_pos) = canvas_response.interact_pointer_pos() {
                        for (i, _) in cells.iter().enumerate() {
                            if i >= positions.len() {
                                break;
                            }
                            let (offset_x, offset_y) = positions[i];
                            let cx = origin.x + center_x + offset_x;
                            let cy = origin.y + center_y + offset_y;
                            let dx = click_pos.x - cx;
                            let dy = click_pos.y - cy;
                            if (dx * dx + dy * dy).sqrt() < hex_radius - 4.0 {
                                hit_any = true;
                                break;
                            }
                        }
                    }
                    if !hit_any {
                        click_action = Some((usize::MAX, false, false));
                    }
                }

                if canvas_response.secondary_clicked() && context_index.is_none() {
                    blank_space_menu = true;
                }
            });

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
}
