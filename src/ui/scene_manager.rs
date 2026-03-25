use eframe::egui::{self, RichText, Stroke};

use crate::app::CyberFile;

impl CyberFile {
    pub(crate) fn render_scene_manager(&mut self, ctx: &egui::Context) {
        let t = self.current_theme;
        let mut open = self.scene_manager_open;

        if self.scene_manager_selected_id.is_none() {
            self.scene_manager_selected_id = self
                .scene_store
                .saved_scenes
                .first()
                .map(|scene| scene.id.clone());
        }

        let scene_list: Vec<(String, String, String, String, bool)> = self
            .ordered_scene_indices()
            .into_iter()
            .filter_map(|index| {
                self.scene_store.saved_scenes.get(index).map(|scene| {
                    (
                        scene.id.clone(),
                        scene.name.clone(),
                        scene.summary.clone(),
                        scene.updated_at.clone(),
                        scene.pinned,
                    )
                })
            })
            .collect();

        let selected_snapshot = self
            .scene_manager_selected_id
            .as_ref()
            .and_then(|scene_id| {
                self.scene_store
                    .saved_scenes
                    .iter()
                    .find(|scene| &scene.id == scene_id)
                    .cloned()
            });

        let mut capture_requested = false;
        let mut capture_name = self.scene_capture_name.clone();
        let mut restore_scene_id: Option<String> = None;
        let mut toggle_pin_scene_id: Option<String> = None;
        let mut delete_scene_id: Option<String> = None;
        let mut metadata_update: Option<(String, String, String, String)> = None;

        egui::Window::new(
            RichText::new("⬢ MISSION SCENE MANAGER")
                .color(t.primary())
                .monospace()
                .strong(),
        )
        .open(&mut open)
        .default_width(860.0)
        .default_height(520.0)
        .resizable(true)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::new()
                .fill(t.bg_dark())
                .stroke(Stroke::new(1.5, t.primary_dim()))
                .inner_margin(12.0),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("SCENES // capture, pin, restore, curate")
                        .color(t.text_dim())
                        .monospace()
                        .size(9.5),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!(
                            "{} SAVED // {} RECENT",
                            self.scene_store.saved_scenes.len(),
                            self.scene_store.recent_scenes.len()
                        ))
                            .color(t.warning())
                            .monospace()
                            .size(9.5),
                    );
                });
            });

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("CAPTURE:")
                        .color(t.text_dim())
                        .monospace()
                        .size(10.0),
                );
                let response = ui.add_sized(
                    [240.0, 20.0],
                    egui::TextEdit::singleline(&mut capture_name)
                        .font(egui::FontId::monospace(11.0))
                        .text_color(t.text_primary())
                        .hint_text(
                            RichText::new("MISSION NAME // blank uses auto ID")
                                .color(t.text_dim()),
                        ),
                );
                if response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                    capture_requested = true;
                }
                if ui
                    .button(
                        RichText::new("CAPTURE CURRENT")
                            .color(t.primary())
                            .monospace()
                            .size(10.0),
                    )
                    .clicked()
                {
                    capture_requested = true;
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            ui.columns(2, |columns| {
                columns[0].vertical(|ui| {
                    ui.label(
                        RichText::new("SCENE INDEX")
                            .color(t.primary())
                            .monospace()
                            .size(10.0),
                    );
                    ui.add_space(4.0);

                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if scene_list.is_empty() {
                                ui.label(
                                    RichText::new("No mission scenes captured")
                                        .color(t.text_dim())
                                        .monospace()
                                        .size(11.0),
                                );
                            }

                            for (scene_id, name, summary, updated_at, pinned) in &scene_list {
                                let selected = self
                                    .scene_manager_selected_id
                                    .as_ref()
                                    .map(|id| id == scene_id)
                                    .unwrap_or(false);
                                let pin = if *pinned { "★" } else { "·" };
                                if ui
                                    .selectable_label(
                                        selected,
                                        RichText::new(format!("{} {}", pin, name))
                                            .color(if selected { t.primary() } else { t.text_primary() })
                                            .monospace()
                                            .size(11.0),
                                    )
                                    .clicked()
                                {
                                    self.scene_manager_selected_id = Some(scene_id.clone());
                                }
                                ui.label(
                                    RichText::new(summary)
                                        .color(t.text_dim())
                                        .monospace()
                                        .size(9.5),
                                );
                                if !updated_at.is_empty() {
                                    ui.label(
                                        RichText::new(updated_at)
                                            .color(t.text_dim())
                                            .monospace()
                                            .size(8.5),
                                    );
                                }
                                ui.add_space(4.0);
                            }
                        });
                });

                columns[1].vertical(|ui| {
                    ui.label(
                        RichText::new("SCENE DETAILS")
                            .color(t.primary())
                            .monospace()
                            .size(10.0),
                    );
                    ui.add_space(4.0);

                    if let Some(scene) = selected_snapshot {
                        let mut edited_name = scene.name.clone();
                        let mut edited_summary = scene.summary.clone();
                        let mut edited_notes = scene.notes.clone();

                        ui.label(
                            RichText::new(format!("ID: {}", scene.id))
                                .color(t.text_dim())
                                .monospace()
                                .size(9.5),
                        );
                        ui.add_space(4.0);

                        ui.label(
                            RichText::new("NAME")
                                .color(t.text_dim())
                                .monospace()
                                .size(9.5),
                        );
                        let name_response = ui.add_sized(
                            [280.0, 22.0],
                            egui::TextEdit::singleline(&mut edited_name)
                                .font(egui::FontId::monospace(11.0))
                                .text_color(t.text_primary()),
                        );

                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("SUMMARY")
                                .color(t.text_dim())
                                .monospace()
                                .size(9.5),
                        );
                        let summary_response = ui.add_sized(
                            [280.0, 22.0],
                            egui::TextEdit::singleline(&mut edited_summary)
                                .font(egui::FontId::monospace(11.0))
                                .text_color(t.text_primary()),
                        );

                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("NOTES")
                                .color(t.text_dim())
                                .monospace()
                                .size(9.5),
                        );
                        let notes_response = ui.add_sized(
                            [320.0, 96.0],
                            egui::TextEdit::multiline(&mut edited_notes)
                                .font(egui::FontId::monospace(10.5))
                                .text_color(t.text_primary()),
                        );

                        if name_response.changed() || summary_response.changed() || notes_response.changed() {
                            metadata_update = Some((
                                scene.id.clone(),
                                edited_name,
                                edited_summary,
                                edited_notes,
                            ));
                        }

                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!("PATH: {}", scene.current_path))
                                .color(t.text_dim())
                                .monospace()
                                .size(9.5),
                        );
                        ui.label(
                            RichText::new(format!("TABS: {}", scene.tabs.len()))
                                .color(t.text_dim())
                                .monospace()
                                .size(9.5),
                        );
                        ui.label(
                            RichText::new(format!("UPDATED: {}", scene.updated_at))
                                .color(t.text_dim())
                                .monospace()
                                .size(9.5),
                        );
                        if !scene.tags.is_empty() {
                            ui.label(
                                RichText::new(format!("TAGS: {}", scene.tags.join(", ")))
                                    .color(t.text_dim())
                                    .monospace()
                                    .size(9.5),
                            );
                        }

                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if ui
                                .button(
                                    RichText::new("RESTORE")
                                        .color(t.primary())
                                        .monospace()
                                        .size(10.0),
                                )
                                .clicked()
                            {
                                restore_scene_id = Some(scene.id.clone());
                            }

                            let pin_label = if scene.pinned { "UNPIN" } else { "PIN" };
                            if ui
                                .button(
                                    RichText::new(pin_label)
                                        .color(if scene.pinned { t.warning() } else { t.text_dim() })
                                        .monospace()
                                        .size(10.0),
                                )
                                .clicked()
                            {
                                toggle_pin_scene_id = Some(scene.id.clone());
                            }

                            if ui
                                .button(
                                    RichText::new("DELETE")
                                        .color(t.danger())
                                        .monospace()
                                        .size(10.0),
                                )
                                .clicked()
                            {
                                delete_scene_id = Some(scene.id.clone());
                            }
                        });
                    } else {
                        ui.label(
                            RichText::new("Select a mission scene to inspect")
                                .color(t.text_dim())
                                .monospace()
                                .size(11.0),
                        );
                    }
                });
            });
        });

        self.scene_manager_open = open;
        self.scene_capture_name = capture_name.clone();

        if capture_requested {
            let explicit = if capture_name.trim().is_empty() {
                None
            } else {
                Some(capture_name.trim().to_string())
            };
            self.save_current_scene(explicit);
            self.scene_capture_name.clear();
            self.scene_manager_open = true;
        }

        if let Some((scene_id, name, summary, notes)) = metadata_update {
            if let Some(scene) = self
                .scene_store
                .saved_scenes
                .iter_mut()
                .find(|scene| scene.id == scene_id)
            {
                scene.name = name.trim().to_string();
                scene.summary = summary.trim().to_string();
                scene.notes = notes;
                scene.updated_at = chrono::Local::now().to_rfc3339();
                self.save_scene_store();
                self.refresh_launcher_results();
            }
        }

        if let Some(scene_id) = toggle_pin_scene_id {
            self.toggle_scene_pin(&scene_id);
            self.scene_manager_selected_id = Some(scene_id);
        }

        if let Some(scene_id) = delete_scene_id {
            self.delete_scene(&scene_id);
        }

        if let Some(scene_id) = restore_scene_id {
            self.restore_scene(&scene_id);
            self.scene_manager_open = true;
        }
    }
}