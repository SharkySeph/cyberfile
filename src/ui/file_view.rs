use eframe::egui::{self, Color32, RichText};
use std::path::PathBuf;

use crate::app::CyberFile;
use crate::filesystem::SortColumn;

/// Pre-collected display data for a single file entry row.
/// Avoids borrow conflicts in closures.
struct DisplayRow {
    index: usize,
    name: String,
    is_dir: bool,
    is_symlink: bool,
    is_hidden: bool,
    size: String,
    modified: String,
    permissions: String,
}

impl CyberFile {
    pub(crate) fn render_file_view(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;
        // ── Breadcrumb ─────────────────────────────────────────
        let breadcrumb_nav = self.render_breadcrumb(ui);
        if let Some(path) = breadcrumb_nav {
            self.navigate_to(path);
            return;
        }

        ui.add_space(2.0);

        // ── Column Headers ─────────────────────────────────────
        let mut sort_action: Option<SortColumn> = None;

        ui.horizontal(|ui| {
            ui.add_space(28.0); // icon column space

            let sort_indicator = |col: SortColumn, current: SortColumn, asc: bool| -> &'static str {
                if col == current {
                    if asc { " ▲" } else { " ▼" }
                } else {
                    ""
                }
            };

            let sort_col = self.sort_column;
            let sort_asc = self.sort_ascending;

            if ui
                .selectable_label(
                    false,
                    RichText::new(format!(
                        "NAME{}",
                        sort_indicator(SortColumn::Name, sort_col, sort_asc)
                    ))
                    .color(t.primary())
                    .monospace()
                    .size(11.0)
                    .strong(),
                )
                .clicked()
            {
                sort_action = Some(SortColumn::Name);
            }

            ui.add_space(ui.available_width() - 320.0);

            if ui
                .selectable_label(
                    false,
                    RichText::new(format!(
                        "SIZE{}",
                        sort_indicator(SortColumn::Size, sort_col, sort_asc)
                    ))
                    .color(t.primary())
                    .monospace()
                    .size(11.0)
                    .strong(),
                )
                .clicked()
            {
                sort_action = Some(SortColumn::Size);
            }

            ui.add_space(12.0);

            if ui
                .selectable_label(
                    false,
                    RichText::new(format!(
                        "EXT{}",
                        sort_indicator(SortColumn::Extension, sort_col, sort_asc)
                    ))
                    .color(t.primary())
                    .monospace()
                    .size(11.0)
                    .strong(),
                )
                .clicked()
            {
                sort_action = Some(SortColumn::Extension);
            }

            ui.add_space(12.0);

            if ui
                .selectable_label(
                    false,
                    RichText::new(format!(
                        "MODIFIED{}",
                        sort_indicator(SortColumn::Modified, sort_col, sort_asc)
                    ))
                    .color(t.primary())
                    .monospace()
                    .size(11.0)
                    .strong(),
                )
                .clicked()
            {
                sort_action = Some(SortColumn::Modified);
            }

            ui.add_space(20.0);

            ui.label(
                RichText::new("ACCESS")
                    .color(t.primary())
                    .monospace()
                    .size(11.0)
                    .strong(),
            );
        });

        // Apply sort
        if let Some(col) = sort_action {
            if self.sort_column == col {
                self.sort_ascending = !self.sort_ascending;
            } else {
                self.sort_column = col;
                self.sort_ascending = true;
            }
            self.sort_entries();
        }

        // Thin separator
        let sep_rect = ui.available_rect_before_wrap();
        ui.painter().line_segment(
            [
                egui::pos2(sep_rect.left(), sep_rect.top()),
                egui::pos2(sep_rect.right(), sep_rect.top()),
            ],
            egui::Stroke::new(0.5, t.primary_dim()),
        );
        ui.add_space(3.0);

        // ── File Listing ───────────────────────────────────────
        // Pre-collect display data (with filter)
        let filter_lower = self.filter_text.to_lowercase();
        let rows: Vec<DisplayRow> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                self.filter_text.is_empty()
                    || e.name.to_lowercase().contains(&filter_lower)
            })
            .map(|(i, e)| DisplayRow {
                index: i,
                name: e.name.clone(),
                is_dir: e.is_dir,
                is_symlink: e.is_symlink,
                is_hidden: e.is_hidden,
                size: e.formatted_size(),
                modified: e.formatted_modified(),
                permissions: e.permission_string(),
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

        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                if rows.is_empty() {
                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("[ SECTOR EMPTY ]")
                                .color(t.text_dim())
                                .monospace()
                                .size(16.0),
                        );
                        ui.label(
                            RichText::new("No data constructs found")
                                .color(t.text_dim())
                                .monospace()
                                .size(12.0),
                        );
                    });
                    return;
                }

                for row in &rows {
                    let is_sel = current_selected == Some(row.index)
                        || current_multi.contains(&row.index);

                    let frame_fill = if is_sel {
                        t.selection_bg()
                    } else {
                        Color32::TRANSPARENT
                    };

                    let frame = egui::Frame::new()
                        .fill(frame_fill)
                        .inner_margin(egui::Margin::symmetric(4, 1));

                    let resp = frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Icon
                            let (icon, icon_color) = if row.is_dir {
                                ("◆", t.primary())
                            } else if row.is_symlink {
                                ("◇", t.accent())
                            } else {
                                ("◇", t.text_dim())
                            };
                            ui.label(
                                RichText::new(icon)
                                    .color(icon_color)
                                    .monospace()
                                    .size(13.0),
                            );
                            ui.add_space(4.0);

                            // Name
                            let name_color = if row.is_dir {
                                t.primary()
                            } else if row.is_symlink {
                                t.accent()
                            } else if row.is_hidden {
                                t.text_dim()
                            } else {
                                t.text_primary()
                            };

                            let name_text = RichText::new(&row.name)
                                .color(name_color)
                                .monospace()
                                .size(13.0);

                            ui.label(name_text);

                            // Right-aligned metadata
                            let meta_color = if is_sel { t.text_primary() } else { t.text_dim() };
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        RichText::new(&row.permissions)
                                            .color(meta_color)
                                            .monospace()
                                            .size(11.0),
                                    );
                                    ui.add_space(16.0);
                                    ui.label(
                                        RichText::new(&row.modified)
                                            .color(meta_color)
                                            .monospace()
                                            .size(11.0),
                                    );
                                    ui.add_space(16.0);
                                    ui.label(
                                        RichText::new(&row.size)
                                            .color(meta_color)
                                            .monospace()
                                            .size(11.0),
                                    );
                                },
                            );
                        });
                    });

                    // Interaction: click entire row area
                    let resp = resp.response.interact(egui::Sense::click());
                    resp.clone().on_hover_text(format!(
                        "{}\n{} │ {} │ {}",
                        row.name, row.size, row.modified, row.permissions
                    ));
                    // Scroll to this row if type-ahead just selected it
                    if current_selected == Some(row.index)
                        && !self.type_ahead_buffer.is_empty()
                        && self.type_ahead_last_key.elapsed().as_millis() < 600
                    {
                        resp.scroll_to_me(Some(egui::Align::Center));
                    }

                    if resp.clicked() {
                        click_action = Some((row.index, ctrl_held, shift_held));
                    }
                    if resp.double_clicked() {
                        open_index = Some(row.index);
                    }
                    if resp.secondary_clicked() {
                        context_index = Some(row.index);
                    }
                }

                // Blank space interaction — right-click opens background context menu,
                // left-click deselects
                let remaining = ui.available_rect_before_wrap();
                let blank_resp = ui.allocate_rect(remaining, egui::Sense::click());
                if blank_resp.clicked() {
                    click_action = Some((usize::MAX, false, false)); // sentinel for deselect
                }
                if blank_resp.secondary_clicked() {
                    blank_space_menu = true;
                }
            });

        // ── Apply Actions ─────────────────────────────────────
        if let Some((idx, ctrl, shift)) = click_action {
            if idx == usize::MAX {
                // Blank click — deselect all
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

    /// Render breadcrumb path navigation. Returns Some(path) if user clicked a segment.
    fn render_breadcrumb(&self, ui: &mut egui::Ui) -> Option<PathBuf> {
        let t = self.current_theme;
        let mut nav_to: Option<PathBuf> = None;

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("◈ PATH")
                    .color(t.primary())
                    .monospace()
                    .size(11.0)
                    .strong(),
            );
            ui.add_space(6.0);

            let mut accumulated = PathBuf::new();
            for component in self.current_path.components() {
                accumulated.push(component);
                let comp_str = component.as_os_str().to_string_lossy();

                ui.label(RichText::new("›").color(t.text_dim()).monospace().size(12.0));

                if ui
                    .link(
                        RichText::new(comp_str.as_ref())
                            .color(t.primary_dim())
                            .monospace()
                            .size(12.0),
                    )
                    .clicked()
                {
                    nav_to = Some(accumulated.clone());
                }
            }
        });

        nav_to
    }
}
