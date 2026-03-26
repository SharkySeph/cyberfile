use eframe::egui::{self, RichText, ScrollArea, Stroke};

use crate::app::{CyberFile, SignalDeckTab};
use crate::integrations::media;

impl CyberFile {
    pub(crate) fn render_signal_deck(&mut self, ctx: &egui::Context) {
        if self.signal_deck_detached {
            let t = self.current_theme;
            let viewport_id = egui::ViewportId::from_hash_of("signal_deck_viewport");
            let builder = egui::ViewportBuilder::default()
                .with_title("CYBERFILE // SIGNAL DECK")
                .with_inner_size([660.0, 480.0])
                .with_min_inner_size([400.0, 300.0]);

            ctx.show_viewport_immediate(viewport_id, builder, |ctx, _class| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.signal_deck_detached = false;
                    self.signal_deck_open = false;
                }
                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(t.surface())
                            .inner_margin(egui::Margin::symmetric(10, 8)),
                    )
                    .show(ctx, |ui| {
                        self.render_signal_deck_content(ui, true);
                    });
            });
        } else {
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
                self.render_signal_deck_content(ui, false);
            });

            self.signal_deck_open = open;
        }
    }

    fn render_signal_deck_content(&mut self, ui: &mut egui::Ui, detached: bool) {
        let t = self.current_theme;
        // ── Tab Selector ───────────────────────────────
            ui.horizontal(|ui| {
                let tabs = [
                    (SignalDeckTab::Audio, "♫ AUDIO"),
                    (SignalDeckTab::Media, "▶ MEDIA"),
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

                ui.separator();

                let detach_label = if detached { "⬡ ATTACH" } else { "⬡ DETACH" };
                let detach_tip = if detached { "Dock back into main window" } else { "Open in separate window" };
                if ui
                    .button(RichText::new(detach_label).color(t.accent()).monospace().size(10.0))
                    .on_hover_text(detach_tip)
                    .clicked()
                {
                    self.signal_deck_detached = !self.signal_deck_detached;
                }
            });
            ui.add_space(6.0);

            match self.signal_deck_tab {
                SignalDeckTab::Audio => self.render_signal_audio(ui),
                SignalDeckTab::Media => self.render_signal_media(ui),
                SignalDeckTab::Clipboard => self.render_signal_clipboard(ui),
                SignalDeckTab::Notifications => self.render_signal_notifications(ui),
                SignalDeckTab::Power => self.render_signal_power(ui),
            }
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

    // ── Media Tab ──────────────────────────────────────────

    fn render_signal_media(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;

        // Refresh player list periodically
        if self.media_players_last_refresh.elapsed().as_secs() >= 5 {
            self.media_players = media::list_players();
            self.media_players_last_refresh = std::time::Instant::now();
        }

        // Refresh media state (more frequently for position tracking)
        if self.media_last_refresh.elapsed().as_millis() >= 1000 {
            self.media_state = media::get_state_for_player(&self.media_preferred_player);
            self.media_last_refresh = std::time::Instant::now();
        }

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("⟐ MEDIA BUS")
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
                    self.media_players = media::list_players();
                    self.media_players_last_refresh = std::time::Instant::now();
                    self.media_state = media::get_state_for_player(&self.media_preferred_player);
                    self.media_last_refresh = std::time::Instant::now();
                }
            });
        });
        ui.add_space(4.0);

        // ── Player Selector ────────────────────────────────
        if !self.media_players.is_empty() {
            ui.label(
                RichText::new("── ACTIVE PLAYERS ──")
                    .color(t.primary())
                    .monospace()
                    .size(10.0)
                    .strong(),
            );
            ui.add_space(2.0);

            let players = self.media_players.clone();
            for p in &players {
                let is_selected = self.media_preferred_player == p.id
                    || (self.media_preferred_player.is_empty()
                        && self.media_state.player_id == p.id);
                let icon = if is_selected { "◉" } else { "○" };
                let status_color = match p.status.as_str() {
                    "Playing" => t.success(),
                    "Paused" => t.warning(),
                    _ => t.text_dim(),
                };
                let name_color = if is_selected { t.accent() } else { t.text_primary() };

                ui.horizontal(|ui| {
                    if ui
                        .button(
                            RichText::new(format!(
                                "{} {} [{}]",
                                icon, p.display_name, p.status
                            ))
                            .color(name_color)
                            .monospace()
                            .size(10.0),
                        )
                        .clicked()
                    {
                        self.media_preferred_player = p.id.clone();
                        self.media_state = media::get_state_for_player(&p.id);
                        self.media_last_refresh = std::time::Instant::now();
                    }
                    let _ = status_color;
                });
            }
            ui.add_space(6.0);
        } else {
            ui.label(
                RichText::new("  No MPRIS players detected.\n  Start a media player (Spotify, Firefox, VLC, mpv, etc.)")
                    .color(t.text_dim())
                    .monospace()
                    .size(11.0),
            );
            return;
        }

        // ── Now Playing ────────────────────────────────────
        let state = &self.media_state;
        if !state.available {
            ui.label(
                RichText::new("  No active playback")
                    .color(t.text_dim())
                    .monospace()
                    .size(11.0),
            );
            return;
        }

        ui.label(
            RichText::new("── NOW PLAYING ──")
                .color(t.primary())
                .monospace()
                .size(10.0)
                .strong(),
        );
        ui.add_space(2.0);

        let status_label = if state.playing { "▶ STREAMING" } else { "⏸ PAUSED" };
        let status_color = if state.playing { t.success() } else { t.warning() };

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("♫ {} FEED", state.player_name))
                    .color(t.primary())
                    .monospace()
                    .size(11.0)
                    .strong(),
            );
            ui.label(
                RichText::new(status_label)
                    .color(status_color)
                    .monospace()
                    .size(10.0),
            );
        });
        ui.add_space(2.0);

        if !state.title.is_empty() {
            ui.label(
                RichText::new(truncate_str(&state.title, 50))
                    .color(t.text_primary())
                    .monospace()
                    .size(12.0)
                    .strong(),
            );
        }
        if !state.artist.is_empty() {
            ui.label(
                RichText::new(truncate_str(&state.artist, 50))
                    .color(t.accent())
                    .monospace()
                    .size(11.0),
            );
        }
        if !state.album.is_empty() {
            ui.label(
                RichText::new(truncate_str(&state.album, 50))
                    .color(t.text_dim())
                    .monospace()
                    .size(10.0),
            );
        }
        ui.add_space(4.0);

        // ── Progress Bar / Seek ────────────────────────────
        let duration = state.duration_secs;
        let position = state.position_secs;
        let player_id = state.player_id.clone();

        if duration > 0.0 {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format_time(position))
                        .color(t.text_dim())
                        .monospace()
                        .size(10.0),
                );

                let bar_width = ui.available_width() - 60.0;
                let mut pos_frac = (position / duration).clamp(0.0, 1.0) as f32;
                let slider = egui::Slider::new(&mut pos_frac, 0.0..=1.0)
                    .show_value(false)
                    .custom_formatter(|_, _| String::new());
                let response = ui.add_sized([bar_width.max(80.0), 14.0], slider);
                if response.changed() {
                    let new_pos = pos_frac as f64 * duration;
                    media::seek_to(&player_id, new_pos);
                }

                ui.label(
                    RichText::new(format_time(duration))
                        .color(t.text_dim())
                        .monospace()
                        .size(10.0),
                );
            });
            ui.add_space(4.0);
        }

        // ── Transport Controls ─────────────────────────────
        ui.horizontal(|ui| {
            let btn_size = 16.0;
            if ui
                .button(RichText::new("⏮").color(t.primary()).monospace().size(btn_size))
                .on_hover_text("Previous track")
                .clicked()
            {
                media::previous_track_player(&player_id);
            }

            let play_icon = if state.playing { "⏸" } else { "▶" };
            if ui
                .button(RichText::new(play_icon).color(t.success()).monospace().size(btn_size))
                .on_hover_text("Play / Pause")
                .clicked()
            {
                media::play_pause_player(&player_id);
            }

            if ui
                .button(RichText::new("⏭").color(t.primary()).monospace().size(btn_size))
                .on_hover_text("Next track")
                .clicked()
            {
                media::next_track_player(&player_id);
            }
        });
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

        ui.add_space(8.0);

        // ── Idle Inhibit ───────────────────────────────────
        let is_inhibited = self.idle_inhibit_child.is_some();
        let inhibit_icon = if is_inhibited { "☕" } else { "💤" };
        let inhibit_label = if is_inhibited {
            "IDLE INHIBIT: ACTIVE"
        } else {
            "IDLE INHIBIT: OFF"
        };
        let inhibit_color = if is_inhibited { t.warning() } else { t.text_dim() };

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{} {}", inhibit_icon, inhibit_label))
                    .color(inhibit_color)
                    .monospace()
                    .size(11.0),
            );

            let btn_label = if is_inhibited { "DISABLE" } else { "ENABLE" };
            let btn_color = if is_inhibited { t.danger() } else { t.accent() };
            if ui
                .button(
                    RichText::new(btn_label)
                        .color(btn_color)
                        .monospace()
                        .size(10.0),
                )
                .on_hover_text("Prevent system from going to sleep/idle")
                .clicked()
            {
                if is_inhibited {
                    if let Some(ref mut child) = self.idle_inhibit_child {
                        crate::integrations::audio::stop_idle_inhibit(child);
                    }
                    self.idle_inhibit_child = None;
                } else {
                    self.idle_inhibit_child = crate::integrations::audio::start_idle_inhibit();
                }
            }
        });
    }
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

fn format_time(secs: f64) -> String {
    let total = secs as u64;
    let m = total / 60;
    let s = total % 60;
    format!("{:02}:{:02}", m, s)
}
