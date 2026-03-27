use eframe::egui::{self, Color32, RichText, ScrollArea, Stroke, TextEdit};

use crate::app::{CyberFile, ProcessSortMode};

impl CyberFile {
    pub(crate) fn render_process_matrix(&mut self, ctx: &egui::Context) {
        if self.process_matrix_detached {
            let t = self.current_theme;
            let viewport_id = egui::ViewportId::from_hash_of("process_matrix_viewport");
            let builder = egui::ViewportBuilder::default()
                .with_title("CYBERFILE // PROCESS MATRIX")
                .with_inner_size([700.0, 500.0])
                .with_min_inner_size([400.0, 300.0]);

            ctx.show_viewport_immediate(viewport_id, builder, |ctx, _class| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.process_matrix_detached = false;
                    self.process_matrix_open = false;
                }
                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(t.surface())
                            .inner_margin(egui::Margin::symmetric(10, 8)),
                    )
                    .show(ctx, |ui| {
                        self.render_process_matrix_content(ui, true);
                    });
            });
        } else {
            let t = self.current_theme;
            let mut open = self.process_matrix_open;

            egui::Window::new(
                RichText::new("┌─ PROCESS MATRIX ─┐")
                    .color(t.primary())
                    .monospace()
                    .strong(),
            )
            .open(&mut open)
            .default_width(680.0)
            .default_height(480.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(t.surface())
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .stroke(Stroke::new(1.0, t.border_dim())),
            )
            .show(ctx, |ui| {
                self.render_process_matrix_content(ui, false);
            });

            self.process_matrix_open = open;
        }
    }

    fn render_process_matrix_content(&mut self, ui: &mut egui::Ui, detached: bool) {
        let t = self.current_theme;
            // ── Toolbar ────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("⟐ FILTER:")
                        .color(t.text_dim())
                        .monospace()
                        .size(11.0),
                );
                let filter = TextEdit::singleline(&mut self.process_filter_text)
                    .desired_width(180.0)
                    .font(egui::TextStyle::Monospace)
                    .text_color(t.text_primary());
                ui.add(filter);

                ui.separator();

                let sorts = [
                    (ProcessSortMode::Cpu, "CPU"),
                    (ProcessSortMode::Memory, "MEM"),
                    (ProcessSortMode::Name, "NAME"),
                    (ProcessSortMode::Pid, "PID"),
                ];
                for (mode, label) in sorts {
                    let active = self.process_sort_mode == mode;
                    let color = if active { t.primary() } else { t.text_dim() };
                    if ui
                        .button(RichText::new(label).color(color).monospace().size(10.0))
                        .clicked()
                    {
                        self.process_sort_mode = mode;
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
                    self.refresh_process_matrix(true);
                }

                ui.separator();

                let detach_label = if detached { "⬡ ATTACH" } else { "⬡ DETACH" };
                let detach_tip = if detached { "Dock back into main window" } else { "Open in separate window" };
                if ui
                    .button(RichText::new(detach_label).color(t.accent()).monospace().size(10.0))
                    .on_hover_text(detach_tip)
                    .clicked()
                {
                    self.process_matrix_detached = !self.process_matrix_detached;
                }
            });
            ui.add_space(4.0);

            // ── Column Headers ─────────────────────────────
            ui.horizontal(|ui| {
                let hw = 11.0;
                ui.label(
                    RichText::new(format!("{:<7}", "PID"))
                        .color(t.primary())
                        .monospace()
                        .size(hw),
                );
                ui.label(
                    RichText::new(format!("{:<20}", "NAME"))
                        .color(t.primary())
                        .monospace()
                        .size(hw),
                );
                ui.label(
                    RichText::new(format!("{:>6}", "CPU%"))
                        .color(t.primary())
                        .monospace()
                        .size(hw),
                );
                ui.label(
                    RichText::new(format!("{:>8}", "MEM"))
                        .color(t.primary())
                        .monospace()
                        .size(hw),
                );
                ui.label(
                    RichText::new(format!("{:>5}", "STAT"))
                        .color(t.primary())
                        .monospace()
                        .size(hw),
                );
            });
            ui.add_space(2.0);

            // ── Process List ───────────────────────────────
            let entries = self.filtered_process_entries();
            let selected_pid = self.process_selected_pid;

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for entry in &entries {
                        let is_selected = selected_pid == Some(entry.pid);
                        let bg = if is_selected {
                            t.surface_raised()
                        } else {
                            Color32::TRANSPARENT
                        };
                        let text_color = if is_selected { t.primary() } else { t.text_primary() };
                        let cpu_color = if entry.cpu_percent > 50.0 {
                            t.danger()
                        } else if entry.cpu_percent > 15.0 {
                            t.warning()
                        } else {
                            t.text_dim()
                        };

                        let mem_str = if entry.memory_kib >= 1_048_576 {
                            format!("{:.1}G", entry.memory_kib as f64 / 1_048_576.0)
                        } else if entry.memory_kib >= 1024 {
                            format!("{:.0}M", entry.memory_kib as f64 / 1024.0)
                        } else {
                            format!("{}K", entry.memory_kib)
                        };

                        let row_text = format!(
                            "{:<7} {:<20} {:>5.1}% {:>7} {:>5}",
                            entry.pid,
                            truncate_str(&entry.name, 20),
                            entry.cpu_percent,
                            mem_str,
                            entry.status,
                        );

                        let response = ui.selectable_label(
                            is_selected,
                            RichText::new(&row_text).color(text_color).monospace().size(11.0).background_color(bg),
                        );

                        if response.clicked() {
                            self.process_selected_pid = Some(entry.pid);
                        }

                        // Tooltip with full command
                        if response.hovered() {
                            egui::show_tooltip(ui.ctx(), ui.layer_id(), egui::Id::new(("proc_tip", entry.pid)), |ui| {
                                ui.label(
                                    RichText::new(format!("CMD: {}", entry.command))
                                        .color(t.text_primary())
                                        .monospace()
                                        .size(10.0),
                                );
                                ui.label(
                                    RichText::new(format!("CWD: {}", entry.cwd))
                                        .color(t.text_dim())
                                        .monospace()
                                        .size(10.0),
                                );
                                if entry.child_count > 0 {
                                    ui.label(
                                        RichText::new(format!("CHILDREN: {}", entry.child_count))
                                            .color(cpu_color)
                                            .monospace()
                                            .size(10.0),
                                    );
                                }
                            });
                        }
                    }
                });

            ui.add_space(4.0);

            // ── Actions ────────────────────────────────────
            ui.horizontal(|ui| {
                let has_selection = self.process_selected_pid.is_some();
                let btn_color = if has_selection { t.warning() } else { t.text_dim() };

                if ui
                    .add_enabled(
                        has_selection,
                        egui::Button::new(
                            RichText::new("⟐ TERM")
                                .color(btn_color)
                                .monospace()
                                .size(10.0),
                        ),
                    )
                    .on_hover_text("Send SIGTERM (graceful)")
                    .clicked()
                {
                    self.terminate_selected_process(false);
                }

                if ui
                    .add_enabled(
                        has_selection,
                        egui::Button::new(
                            RichText::new("⟐ KILL")
                                .color(if has_selection { t.danger() } else { t.text_dim() })
                                .monospace()
                                .size(10.0),
                        ),
                    )
                    .on_hover_text("Send SIGKILL (force)")
                    .clicked()
                {
                    self.terminate_selected_process(true);
                }

                ui.separator();

                // Check if selected process is stopped
                let selected_stopped = self.process_selected_pid.map(|pid| {
                    self.process_entries.iter()
                        .find(|e| e.pid == pid)
                        .map(|e| e.status.starts_with('T'))
                        .unwrap_or(false)
                }).unwrap_or(false);

                if selected_stopped {
                    if ui
                        .add_enabled(
                            has_selection,
                            egui::Button::new(
                                RichText::new("▶ CONT")
                                    .color(if has_selection { t.success() } else { t.text_dim() })
                                    .monospace()
                                    .size(10.0),
                            ),
                        )
                        .on_hover_text("Send SIGCONT (resume)")
                        .clicked()
                    {
                        self.continue_selected_process();
                    }
                } else {
                    if ui
                        .add_enabled(
                            has_selection,
                            egui::Button::new(
                                RichText::new("⏸ STOP")
                                    .color(if has_selection { t.accent() } else { t.text_dim() })
                                    .monospace()
                                    .size(10.0),
                            ),
                        )
                        .on_hover_text("Send SIGSTOP (pause)")
                        .clicked()
                    {
                        self.stop_selected_process();
                    }
                }

                if ui
                    .add_enabled(
                        has_selection,
                        egui::Button::new(
                            RichText::new("⟐ HUP")
                                .color(if has_selection { t.primary() } else { t.text_dim() })
                                .monospace()
                                .size(10.0),
                        ),
                    )
                    .on_hover_text("Send SIGHUP (reload config)")
                    .clicked()
                {
                    self.signal_selected_process("HUP");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{} processes", entries.len()))
                            .color(t.text_dim())
                            .monospace()
                            .size(10.0),
                    );
                });
            });

            // ── Priority Controls ──────────────────────────
            if let Some(pid) = self.process_selected_pid {
                let nice = crate::integrations::processes::get_nice_value(pid);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "PID {} │ NICE: {}",
                            pid,
                            nice.map(|n| n.to_string()).unwrap_or_else(|| "?".into()),
                        ))
                        .color(t.text_dim())
                        .monospace()
                        .size(10.0),
                    );
                    ui.separator();
                    if ui
                        .button(RichText::new("▲ +PRI").color(t.accent()).monospace().size(10.0))
                        .on_hover_text("Increase priority (lower nice)")
                        .clicked()
                    {
                        let new_nice = nice.unwrap_or(0) - 5;
                        self.renice_selected_process(new_nice);
                    }
                    if ui
                        .button(RichText::new("▼ −PRI").color(t.text_dim()).monospace().size(10.0))
                        .on_hover_text("Decrease priority (higher nice)")
                        .clicked()
                    {
                        let new_nice = nice.unwrap_or(0) + 5;
                        self.renice_selected_process(new_nice);
                    }
                });
            }
    }
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
