use eframe::egui::{self, Color32, RichText, Stroke};

use crate::app::CyberFile;

impl CyberFile {
    /// Render tab bar with NERV-style segment indicators
    pub(crate) fn render_tab_bar(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;
        let tab_count = self.tabs.len();

        ui.horizontal(|ui| {
            // NERV-style section label
            ui.label(
                RichText::new("┌─ CHANNELS ─┐")
                    .color(t.primary())
                    .monospace()
                    .size(10.0),
            );

            ui.add_space(8.0);

            let mut close_tab: Option<usize> = None;
            let mut switch_tab: Option<usize> = None;
            let mut drag_from: Option<usize> = None;
            let mut drop_to: Option<usize> = None;

            for i in 0..tab_count {
                let is_active = i == self.active_tab;
                let tab = &self.tabs[i];
                let label = tab
                    .path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| tab.path.to_string_lossy().to_string());

                // Truncate long names
                let display = if label.len() > 18 {
                    format!("{}…", &label[..17])
                } else {
                    label
                };

                let seg_label = format!("SEG.{}", i + 1);

                // NERV segment indicator style
                let (bg, border, text_color) = if is_active {
                    (
                        Color32::from_rgba_premultiplied(
                            t.primary().r(),
                            t.primary().g(),
                            t.primary().b(),
                            40,
                        ),
                        t.primary(),
                        t.text_primary(),
                    )
                } else {
                    (Color32::TRANSPARENT, t.border_dim(), t.text_dim())
                };

                let resp = egui::Frame::new()
                    .fill(bg)
                    .stroke(Stroke::new(if is_active { 1.5 } else { 0.5 }, border))
                    .inner_margin(egui::Margin::symmetric(6, 2))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Segment number (like SEG.1, SEG.2 from EVA)
                            ui.label(
                                RichText::new(&seg_label)
                                    .color(if is_active { t.warning() } else { t.text_dim() })
                                    .monospace()
                                    .size(8.0),
                            );
                            ui.label(
                                RichText::new(&display)
                                    .color(text_color)
                                    .monospace()
                                    .size(11.0),
                            );
                            // Close button (only if more than 1 tab)
                            if tab_count > 1 {
                                if ui
                                    .small_button(
                                        RichText::new("✕").color(t.danger()).size(9.0),
                                    )
                                    .clicked()
                                {
                                    close_tab = Some(i);
                                }
                            }
                        });
                    })
                    .response
                    .interact(egui::Sense::click_and_drag());

                if resp.clicked() {
                    switch_tab = Some(i);
                }

                // Tab drag-to-reorder
                if resp.dragged() {
                    drag_from = Some(i);
                }
                if resp.hovered() && ui.input(|inp| inp.pointer.any_released()) {
                    drop_to = Some(i);
                }
            }

            // New tab button
            if ui
                .button(
                    RichText::new("+ OPEN CHANNEL")
                        .color(t.primary_dim())
                        .monospace()
                        .size(10.0),
                )
                .clicked()
            {
                self.open_new_tab();
            }

            // Apply actions
            if let Some(i) = switch_tab {
                self.switch_to_tab(i);
            }
            if let Some(i) = close_tab {
                self.close_tab(i);
            }
            // Tab reorder via drag
            if let (Some(from), Some(to)) = (drag_from, drop_to) {
                if from != to && from < self.tabs.len() && to < self.tabs.len() {
                    let tab = self.tabs.remove(from);
                    self.tabs.insert(to, tab);
                    // Adjust active_tab index
                    if self.active_tab == from {
                        self.active_tab = to;
                    } else if from < self.active_tab && to >= self.active_tab {
                        self.active_tab -= 1;
                    } else if from > self.active_tab && to <= self.active_tab {
                        self.active_tab += 1;
                    }
                }
            }
        });

        // NERV-style border line beneath tabs
        let rect = ui.available_rect_before_wrap();
        let y = rect.top();
        ui.painter().line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            Stroke::new(1.0, t.primary()),
        );
        // "BORDER LINE" label (EVA style)
        let line_label_pos = egui::pos2(rect.center().x - 30.0, y - 7.0);
        ui.painter().text(
            line_label_pos,
            egui::Align2::CENTER_BOTTOM,
            "BORDER LINE",
            egui::FontId::monospace(7.0),
            t.primary_dim(),
        );
        ui.add_space(3.0);
    }
}
