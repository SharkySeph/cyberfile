use eframe::egui::{self, RichText, ScrollArea, Stroke};

use crate::app::{CyberFile, SignalDeckTab};

impl CyberFile {
    pub(crate) fn render_signal_deck(&mut self, ctx: &egui::Context) {
        let t = self.current_theme;
        let mut open = self.signal_deck_open;

        egui::Window::new(
            RichText::new("┌─ SIGNAL DECK ─┐")
                .color(t.primary())
                .monospace()
                .strong(),
        )
        .open(&mut open)
        .default_width(640.0)
        .default_height(460.0)
        .resizable(true)
        .frame(
            egui::Frame::new()
                .fill(t.surface())
                .inner_margin(egui::Margin::symmetric(10, 8))
                .stroke(Stroke::new(1.0, t.border_dim())),
        )
        .show(ctx, |ui| {
            // ── Tab Selector ───────────────────────────────
            ui.horizontal(|ui| {
                let tabs = [
                    (SignalDeckTab::Audio, "♫ AUDIO"),
                    (SignalDeckTab::Clipboard, "◫ CLIPBOARD"),
                    (SignalDeckTab::Notifications, "⚡ ALERTS"),
                    (SignalDeckTab::Power, "⏻ POWER"),
                ];
                for (tab, label) in tabs {
                    let active = self.signal_deck_tab == tab;
                    let color = if active { t.primary() } else { t.text_dim() };
                    if ui
                        .button(RichText::new(label).color(color).monospace().size(11.0))
                        .clicked()
                    {
                        self.signal_deck_tab = tab;
                    }
                }
            });
            ui.add_space(6.0);

            match self.signal_deck_tab {
                SignalDeckTab::Audio => self.render_signal_audio(ui),
                SignalDeckTab::Clipboard => self.render_signal_clipboard(ui),
                SignalDeckTab::Notifications => self.render_signal_notifications(ui),
                SignalDeckTab::Power => self.render_signal_power(ui),
            }
        });

        self.signal_deck_open = open;
    }

    // ── Audio Tab ──────────────────────────────────────────

    fn render_signal_audio(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;
        let backend_label = match self.audio_snapshot.backend {
            crate::integrations::audio::AudioBackend::Pipewire => "PIPEWIRE",
            crate::integrations::audio::AudioBackend::Pulseaudio => "PULSEAUDIO",
            crate::integrations::audio::AudioBackend::None => "NO BACKEND",
        };

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("⟐ BACKEND: {}", backend_label))
                    .color(t.text_dim())
                    .monospace()
                    .size(10.0),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(
                        RichText::new("⟳ REFRESH")
                            .color(t.accent())
                            .monospace()
                            .size(10.0),
                    )
                    .clicked()
                {
                    self.refresh_audio_snapshot(true);
                }
            });
        });
        ui.add_space(4.0);

        // ── Master Volume ──────────────────────────────────
        ui.horizontal(|ui| {
            let muted = self.audio_snapshot.default_sink_muted;
            let mute_icon = if muted { "🔇" } else { "🔊" };
            let mute_color = if muted { t.danger() } else { t.accent() };

            if ui
                .button(
                    RichText::new(mute_icon)
                        .color(mute_color)
                        .monospace()
                        .size(14.0),
                )
                .clicked()
            {
                crate::integrations::audio::toggle_default_sink_mute();
                self.refresh_audio_snapshot(true);
            }

            ui.label(
                RichText::new("VOLUME:")
                    .color(t.text_primary())
                    .monospace()
                    .size(11.0),
            );

            let mut vol = self.audio_volume_slider as f32;
            let slider = egui::Slider::new(&mut vol, 0.0..=150.0)
                .text("%")
                .show_value(true);
            if ui.add(slider).changed() {
                self.audio_volume_slider = vol as u32;
                crate::integrations::audio::set_default_sink_volume(self.audio_volume_slider);
            }
        });
        ui.add_space(2.0);

        // ── Mic Mute ──────────────────────────────────────
        ui.horizontal(|ui| {
            let mic_muted = self.audio_snapshot.default_source_muted;
            let mic_icon = if mic_muted { "🎤✕" } else { "🎤" };
            let mic_color = if mic_muted { t.danger() } else { t.accent() };

            if ui
                .button(
                    RichText::new(mic_icon)
                        .color(mic_color)
                        .monospace()
                        .size(12.0),
                )
                .clicked()
            {
                crate::integrations::audio::toggle_default_source_mute();
                self.refresh_audio_snapshot(true);
            }
            ui.label(
                RichText::new(if mic_muted {
                    "MIC: MUTED"
                } else {
                    "MIC: ACTIVE"
                })
                .color(if mic_muted { t.warning() } else { t.text_primary() })
                .monospace()
                .size(11.0),
            );
        });
        ui.add_space(6.0);

        // ── Sinks ──────────────────────────────────────────
        if !self.audio_snapshot.sinks.is_empty() {
            ui.label(
                RichText::new("── OUTPUT DEVICES ──")
                    .color(t.primary())
                    .monospace()
                    .size(10.0)
                    .strong(),
            );
            ui.add_space(2.0);

            let sinks = self.audio_snapshot.sinks.clone();
            for sink in &sinks {
                let label = if sink.description.is_empty() {
                    &sink.name
                } else {
                    &sink.description
                };
                let is_default = sink.is_default;
                let color = if is_default {
                    t.accent()
                } else {
                    t.text_primary()
                };
                let icon = if is_default { "◉" } else { "○" };

                ui.horizontal(|ui| {
                    let btn = ui.button(
                        RichText::new(format!(
                            "{} {} [{}%{}]",
                            icon,
                            truncate_str(label, 35),
                            sink.volume_percent,
                            if sink.muted { " MUTED" } else { "" }
                        ))
                        .color(color)
                        .monospace()
                        .size(10.0),
                    );
                    if btn.clicked() && !is_default {
                        crate::integrations::audio::set_default_sink(&sink.name);
                        self.refresh_audio_snapshot(true);
                    }
                });
            }
            ui.add_space(4.0);
        }

        // ── Active Streams (mixer) ─────────────────────────
        if !self.audio_snapshot.streams.is_empty() {
            ui.label(
                RichText::new("── ACTIVE STREAMS ──")
                    .color(t.primary())
                    .monospace()
                    .size(10.0)
                    .strong(),
            );
            ui.add_space(2.0);

            let streams = self.audio_snapshot.streams.clone();
            for stream in &streams {
                let app = if stream.app_name.is_empty() {
                    format!("Stream #{}", stream.id)
                } else {
                    stream.app_name.clone()
                };
                let mute_label = if stream.muted { " [MUTED]" } else { "" };
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "  ▸ {} — {}%{}",
                            truncate_str(&app, 25),
                            stream.volume_percent,
                            mute_label
                        ))
                        .color(if stream.muted { t.text_dim() } else { t.text_primary() })
                        .monospace()
                        .size(10.0),
                    );
                    if ui
                        .small_button(
                            RichText::new(if stream.muted { "🔇" } else { "🔊" })
                                .size(10.0),
                        )
                        .clicked()
                    {
                        crate::integrations::audio::toggle_stream_mute(stream.id);
                        self.refresh_audio_snapshot(true);
                    }
                });
            }
        }
    }

    // ── Clipboard Tab ──────────────────────────────────────

    fn render_signal_clipboard(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("⟐ CLIPBOARD HISTORY")
                    .color(t.text_dim())
                    .monospace()
                    .size(10.0),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(
                        RichText::new("⟳ REFRESH")
                            .color(t.accent())
                            .monospace()
                            .size(10.0),
                    )
                    .clicked()
                {
                    self.refresh_clipboard(true);
                }
            });
        });
        ui.add_space(4.0);

        if self.clipboard_entries.is_empty() {
            ui.label(
                RichText::new("  No clipboard history available.\n  Install cliphist for Wayland clipboard history.")
                    .color(t.text_dim())
                    .monospace()
                    .size(11.0),
            );
            return;
        }

        let entries = self.clipboard_entries.clone();
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for entry in &entries {
                    let response = ui.selectable_label(
                        false,
                        RichText::new(format!("  {} │ {}", entry.index + 1, entry.preview))
                            .color(t.text_primary())
                            .monospace()
                            .size(10.0),
                    );
                    if response.clicked() {
                        crate::integrations::audio::paste_clipboard_entry(entry.index);
                        self.status_message =
                            format!("Clipboard entry {} copied to clipboard", entry.index + 1);
                    }
                }
            });

        ui.add_space(2.0);
        ui.label(
            RichText::new(format!("{} entries", self.clipboard_entries.len()))
                .color(t.text_dim())
                .monospace()
                .size(10.0),
        );
    }

    // ── Notifications Tab ──────────────────────────────────

    fn render_signal_notifications(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("⟐ NOTIFICATION HISTORY")
                    .color(t.text_dim())
                    .monospace()
                    .size(10.0),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(
                        RichText::new("✕ CLEAR ALL")
                            .color(t.danger())
                            .monospace()
                            .size(10.0),
                    )
                    .clicked()
                {
                    crate::integrations::audio::clear_all_notifications();
                    self.refresh_notifications(true);
                }
                if ui
                    .button(
                        RichText::new("⟳ REFRESH")
                            .color(t.accent())
                            .monospace()
                            .size(10.0),
                    )
                    .clicked()
                {
                    self.refresh_notifications(true);
                }
            });
        });
        ui.add_space(4.0);

        if self.notification_entries.is_empty() {
            ui.label(
                RichText::new("  No notification history.\n  Requires dunst (dunstctl) or swaync.")
                    .color(t.text_dim())
                    .monospace()
                    .size(11.0),
            );
            return;
        }

        let entries = self.notification_entries.clone();
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for entry in &entries {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("[{}]", entry.app))
                                .color(t.accent())
                                .monospace()
                                .size(10.0),
                        );
                        ui.label(
                            RichText::new(&entry.summary)
                                .color(t.text_primary())
                                .monospace()
                                .size(10.0),
                        );
                        if ui
                            .small_button(RichText::new("✕").color(t.danger()).size(10.0))
                            .clicked()
                        {
                            crate::integrations::audio::dismiss_notification(entry.id);
                            self.refresh_notifications(true);
                        }
                    });
                    if !entry.body.is_empty() {
                        ui.label(
                            RichText::new(format!("    {}", truncate_str(&entry.body, 70)))
                                .color(t.text_dim())
                                .monospace()
                                .size(9.0),
                        );
                    }
                    ui.add_space(2.0);
                }
            });

        ui.add_space(2.0);
        ui.label(
            RichText::new(format!("{} notifications", self.notification_entries.len()))
                .color(t.text_dim())
                .monospace()
                .size(10.0),
        );
    }

    // ── Power Tab ──────────────────────────────────────────

    fn render_signal_power(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("⟐ POWER STATUS")
                    .color(t.text_dim())
                    .monospace()
                    .size(10.0),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(
                        RichText::new("⟳ REFRESH")
                            .color(t.accent())
                            .monospace()
                            .size(10.0),
                    )
                    .clicked()
                {
                    self.refresh_power_info(true);
                }
            });
        });
        ui.add_space(6.0);

        // ── Battery ────────────────────────────────────────
        if let Some(percent) = self.power_info.battery_percent {
            let icon = if self.power_info.charging {
                "⚡"
            } else if percent > 80 {
                "🔋"
            } else if percent > 30 {
                "🔋"
            } else {
                "🪫"
            };
            let color = if percent > 50 {
                t.accent()
            } else if percent > 20 {
                t.warning()
            } else {
                t.danger()
            };

            ui.label(
                RichText::new(format!("{} BATTERY: {}%", icon, percent))
                    .color(color)
                    .monospace()
                    .size(14.0)
                    .strong(),
            );
            ui.add_space(2.0);

            // Bar
            let bar_width = ui.available_width().min(400.0);
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(bar_width, 12.0), egui::Sense::hover());
            ui.painter()
                .rect_filled(rect, 2.0, t.surface_raised());
            let fill_w = rect.width() * (percent as f32 / 100.0);
            let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(fill_w, rect.height()));
            ui.painter().rect_filled(fill_rect, 2.0, color);
            ui.add_space(4.0);

            let state_str = if self.power_info.charging {
                "CHARGING"
            } else if self.power_info.on_battery {
                "DISCHARGING"
            } else {
                "AC POWER"
            };
            ui.label(
                RichText::new(format!("  STATE: {}", state_str))
                    .color(t.text_primary())
                    .monospace()
                    .size(11.0),
            );

            if !self.power_info.time_remaining.is_empty() {
                ui.label(
                    RichText::new(format!("  REMAINING: {}", self.power_info.time_remaining))
                        .color(t.text_dim())
                        .monospace()
                        .size(11.0),
                );
            }
        } else {
            ui.label(
                RichText::new("  ⏻ AC POWER (no battery detected)")
                    .color(t.text_dim())
                    .monospace()
                    .size(12.0),
            );
        }
        ui.add_space(8.0);

        // ── Power Profile ──────────────────────────────────
        if !self.power_info.power_profile.is_empty() {
            ui.label(
                RichText::new(format!("  PROFILE: {}", self.power_info.power_profile.to_uppercase()))
                    .color(t.text_primary())
                    .monospace()
                    .size(11.0),
            );
            ui.add_space(4.0);
        }

        // ── Brightness ─────────────────────────────────────
        if let Some(brightness) = self.brightness_percent {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("☀ BRIGHTNESS:")
                        .color(t.text_primary())
                        .monospace()
                        .size(11.0),
                );
                let mut br = brightness as f32;
                let slider = egui::Slider::new(&mut br, 1.0..=100.0)
                    .text("%")
                    .show_value(true);
                if ui.add(slider).changed() {
                    let new_val = br as u32;
                    crate::integrations::audio::set_brightness_percent(new_val);
                    self.brightness_percent = Some(new_val);
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
