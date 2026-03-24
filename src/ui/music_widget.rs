use eframe::egui::{self, RichText, Stroke};

use crate::app::CyberFile;
use crate::integrations::media;

impl CyberFile {
    /// Generic MPRIS media player widget — NERV-style media readout
    /// Auto-detects any active audio source (Spotify, Firefox, VLC, mpv, etc.)
    pub(crate) fn render_music_widget(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;

        // Refresh media state periodically (every 3 seconds)
        if self.media_last_refresh.elapsed().as_secs() >= 3 {
            self.media_state = media::get_state();
            self.media_last_refresh = std::time::Instant::now();
        }

        let state = &self.media_state;

        if !state.available {
            // No active media player
            egui::Frame::new()
                .fill(t.surface())
                .stroke(Stroke::new(0.5, t.border_dim()))
                .inner_margin(egui::Margin::symmetric(8, 4))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("♫ AUDIO FEED — OFFLINE")
                            .color(t.text_dim())
                            .monospace()
                            .size(10.0),
                    );
                });
            return;
        }

        egui::Frame::new()
            .fill(t.surface())
            .stroke(Stroke::new(1.0, if state.playing { t.success() } else { t.border_dim() }))
            .inner_margin(egui::Margin::symmetric(8, 5))
            .show(ui, |ui| {
                // Header with player source
                let status_label = if state.playing { "▶ STREAMING" } else { "⏸ PAUSED" };
                let status_color = if state.playing { t.success() } else { t.warning() };

                ui.horizontal(|ui| {
                    let header = if state.player_name.is_empty() {
                        "♫ AUDIO FEED".to_string()
                    } else {
                        format!("♫ {} FEED", state.player_name)
                    };
                    ui.label(
                        RichText::new(&header)
                            .color(t.primary())
                            .monospace()
                            .size(10.0)
                            .strong(),
                    );
                    ui.label(
                        RichText::new(status_label)
                            .color(status_color)
                            .monospace()
                            .size(9.0),
                    );
                });

                ui.add_space(2.0);

                // Track info
                if !state.title.is_empty() {
                    let title_display = if state.title.len() > 28 {
                        format!("{}…", &state.title[..27])
                    } else {
                        state.title.clone()
                    };
                    ui.label(
                        RichText::new(&title_display)
                            .color(t.text_primary())
                            .monospace()
                            .size(11.0)
                            .strong(),
                    );
                }

                if !state.artist.is_empty() {
                    let artist_display = if state.artist.len() > 30 {
                        format!("{}…", &state.artist[..29])
                    } else {
                        state.artist.clone()
                    };
                    ui.label(
                        RichText::new(&artist_display)
                            .color(t.accent())
                            .monospace()
                            .size(10.0),
                    );
                }

                if !state.album.is_empty() {
                    let album_display = if state.album.len() > 30 {
                        format!("{}…", &state.album[..29])
                    } else {
                        state.album.clone()
                    };
                    ui.label(
                        RichText::new(&album_display)
                            .color(t.text_dim())
                            .monospace()
                            .size(9.0),
                    );
                }

                ui.add_space(3.0);

                // Transport controls
                ui.horizontal(|ui| {
                    if ui
                        .button(
                            RichText::new("⏮")
                                .color(t.primary())
                                .monospace()
                                .size(14.0),
                        )
                        .on_hover_text("Previous track")
                        .clicked()
                    {
                        media::previous_track();
                    }

                    let play_icon = if state.playing { "⏸" } else { "▶" };
                    if ui
                        .button(
                            RichText::new(play_icon)
                                .color(t.success())
                                .monospace()
                                .size(14.0),
                        )
                        .on_hover_text("Play/Pause")
                        .clicked()
                    {
                        media::play_pause();
                    }

                    if ui
                        .button(
                            RichText::new("⏭")
                                .color(t.primary())
                                .monospace()
                                .size(14.0),
                        )
                        .on_hover_text("Next track")
                        .clicked()
                    {
                        media::next_track();
                    }
                });
            });
    }
}
