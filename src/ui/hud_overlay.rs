use eframe::egui::{self, Color32, Stroke, epaint::StrokeKind};

use crate::app::CyberFile;

impl CyberFile {
    /// NERV-style HUD overlay — painted over the main content area
    /// Shows system telemetry, file composition analysis, and animated indicators
    pub(crate) fn render_hud_overlay(&self, ctx: &egui::Context) {
        let t = self.current_theme;
        let screen = ctx.screen_rect();

        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("hud_overlay"),
        ));

        // ── Top-left: Sector identifier + path depth gauge ──
        {
            let x = screen.left() + 12.0;
            let y = screen.top() + 32.0;

            // Sector ID from path
            let sector_name = self
                .current_path
                .file_name()
                .map(|n| n.to_string_lossy().to_uppercase())
                .unwrap_or_else(|| "ROOT".into());

            let depth = self.current_path.components().count();
            let depth_bar_width: f32 = 60.0;
            let depth_fill = (depth as f32 / 12.0).min(1.0);

            // Sector label
            painter.text(
                egui::pos2(x, y),
                egui::Align2::LEFT_TOP,
                format!("SEC:{}", sector_name),
                egui::FontId::monospace(9.0),
                Color32::from_rgba_premultiplied(
                    t.primary().r(),
                    t.primary().g(),
                    t.primary().b(),
                    100,
                ),
            );

            // Depth bar outline
            let bar_y = y + 12.0;
            painter.rect_stroke(
                egui::Rect::from_min_size(egui::pos2(x, bar_y), egui::vec2(depth_bar_width, 3.0)),
                0.0,
                Stroke::new(
                    0.5,
                    Color32::from_rgba_premultiplied(
                        t.border_dim().r(),
                        t.border_dim().g(),
                        t.border_dim().b(),
                        80,
                    ),
                ),
                StrokeKind::Outside,
            );
            // Depth bar fill
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(x, bar_y),
                    egui::vec2(depth_bar_width * depth_fill, 3.0),
                ),
                0.0,
                Color32::from_rgba_premultiplied(
                    t.primary().r(),
                    t.primary().g(),
                    t.primary().b(),
                    60,
                ),
            );

            painter.text(
                egui::pos2(x + depth_bar_width + 4.0, bar_y - 1.0),
                egui::Align2::LEFT_TOP,
                format!("D:{}", depth),
                egui::FontId::monospace(8.0),
                Color32::from_rgba_premultiplied(
                    t.text_dim().r(),
                    t.text_dim().g(),
                    t.text_dim().b(),
                    80,
                ),
            );
        }

        // ── Bottom-right: File composition ring ──
        {
            let cx = screen.right() - 50.0;
            let cy = screen.bottom() - 55.0;
            let radius: f32 = 28.0;
            let ring_width: f32 = 4.0;

            let mut dirs = 0u32;
            let mut code = 0u32;
            let mut media = 0u32;
            let mut archive = 0u32;
            let mut other = 0u32;

            for entry in &self.entries {
                if entry.is_dir {
                    dirs += 1;
                } else {
                    let ext = entry
                        .path
                        .extension()
                        .map(|e| e.to_string_lossy().to_lowercase())
                        .unwrap_or_default();
                    match ext.as_str() {
                        "rs" | "py" | "js" | "ts" | "c" | "cpp" | "h" | "go" | "java" | "rb"
                        | "lua" | "sh" | "bash" | "zsh" | "toml" | "yaml" | "yml" | "json"
                        | "xml" | "md" | "txt" | "html" | "css" => code += 1,
                        "png" | "jpg" | "jpeg" | "gif" | "svg" | "bmp" | "webp" | "mp3"
                        | "flac" | "wav" | "ogg" | "m4a" | "mp4" | "mkv" | "avi" | "webm" => {
                            media += 1
                        }
                        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "zst" | "deb"
                        | "rpm" => archive += 1,
                        _ => other += 1,
                    }
                }
            }

            let total = dirs + code + media + archive + other;
            if total > 0 {
                let segments: Vec<(f32, Color32)> = vec![
                    (
                        dirs as f32 / total as f32,
                        Color32::from_rgba_premultiplied(
                            t.primary().r(),
                            t.primary().g(),
                            t.primary().b(),
                            120,
                        ),
                    ),
                    (
                        code as f32 / total as f32,
                        Color32::from_rgba_premultiplied(
                            t.success().r(),
                            t.success().g(),
                            t.success().b(),
                            120,
                        ),
                    ),
                    (
                        media as f32 / total as f32,
                        Color32::from_rgba_premultiplied(
                            t.accent().r(),
                            t.accent().g(),
                            t.accent().b(),
                            120,
                        ),
                    ),
                    (
                        archive as f32 / total as f32,
                        Color32::from_rgba_premultiplied(
                            t.warning().r(),
                            t.warning().g(),
                            t.warning().b(),
                            120,
                        ),
                    ),
                    (
                        other as f32 / total as f32,
                        Color32::from_rgba_premultiplied(
                            t.text_dim().r(),
                            t.text_dim().g(),
                            t.text_dim().b(),
                            80,
                        ),
                    ),
                ];

                let mut angle: f32 = -std::f32::consts::FRAC_PI_2;
                for (frac, color) in &segments {
                    if *frac < 0.005 {
                        continue;
                    }

                    let sweep = frac * std::f32::consts::TAU;
                    let steps = (sweep / 0.05).max(2.0) as usize;
                    let step_angle = sweep / steps as f32;

                    for s in 0..steps {
                        let a = angle + s as f32 * step_angle;
                        let inner_r = radius - ring_width;
                        let p1 = egui::pos2(cx + a.cos() * inner_r, cy + a.sin() * inner_r);
                        let p2 = egui::pos2(cx + a.cos() * radius, cy + a.sin() * radius);
                        painter.line_segment([p1, p2], Stroke::new(2.0, *color));
                    }

                    angle += sweep;
                }

                // Center text
                painter.text(
                    egui::pos2(cx, cy),
                    egui::Align2::CENTER_CENTER,
                    format!("{}", total),
                    egui::FontId::monospace(10.0),
                    Color32::from_rgba_premultiplied(
                        t.text_dim().r(),
                        t.text_dim().g(),
                        t.text_dim().b(),
                        100,
                    ),
                );

                // Label
                painter.text(
                    egui::pos2(cx, cy + radius + 8.0),
                    egui::Align2::CENTER_TOP,
                    "COMP",
                    egui::FontId::monospace(7.0),
                    Color32::from_rgba_premultiplied(
                        t.border_dim().r(),
                        t.border_dim().g(),
                        t.border_dim().b(),
                        100,
                    ),
                );
            }
        }

        // ── Top-right: Animated threat/activity level indicator ──
        {
            let x = screen.right() - 90.0;
            let y = screen.top() + 32.0;

            // Activity level based on entry count
            let activity = (self.entries.len() as f32 / 200.0).min(1.0);
            let level_label = if activity > 0.8 {
                "CRITICAL"
            } else if activity > 0.5 {
                "ELEVATED"
            } else if activity > 0.2 {
                "NOMINAL"
            } else {
                "MINIMAL"
            };
            let level_color = if activity > 0.8 {
                t.danger()
            } else if activity > 0.5 {
                t.warning()
            } else {
                t.success()
            };

            painter.text(
                egui::pos2(x, y),
                egui::Align2::LEFT_TOP,
                "DENSITY",
                egui::FontId::monospace(7.0),
                Color32::from_rgba_premultiplied(
                    t.text_dim().r(),
                    t.text_dim().g(),
                    t.text_dim().b(),
                    80,
                ),
            );

            // Segmented bar (8 segments)
            let bar_y = y + 10.0;
            let seg_w: f32 = 7.0;
            let seg_gap: f32 = 2.0;
            let filled_segs = (activity * 8.0).ceil() as usize;

            for i in 0..8 {
                let sx = x + i as f32 * (seg_w + seg_gap);
                let color = if i < filled_segs {
                    Color32::from_rgba_premultiplied(
                        level_color.r(),
                        level_color.g(),
                        level_color.b(),
                        100,
                    )
                } else {
                    Color32::from_rgba_premultiplied(
                        t.border_dim().r(),
                        t.border_dim().g(),
                        t.border_dim().b(),
                        40,
                    )
                };
                painter.rect_filled(
                    egui::Rect::from_min_size(egui::pos2(sx, bar_y), egui::vec2(seg_w, 4.0)),
                    0.0,
                    color,
                );
            }

            painter.text(
                egui::pos2(x, bar_y + 7.0),
                egui::Align2::LEFT_TOP,
                level_label,
                egui::FontId::monospace(7.0),
                Color32::from_rgba_premultiplied(
                    level_color.r(),
                    level_color.g(),
                    level_color.b(),
                    90,
                ),
            );
        }
    }

    /// MAGI-style animated border pulse on the central panel
    /// Pulses based on frame count for a subtle living-interface feel
    pub(crate) fn render_border_pulse(&self, ctx: &egui::Context) {
        let t = self.current_theme;
        let screen = ctx.screen_rect();

        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Middle,
            egui::Id::new("border_pulse"),
        ));

        // Sine-wave pulse (slow breathing effect)
        let phase = (self.frame_count as f32 * 0.02).sin() * 0.5 + 0.5;
        let alpha = (8.0 + phase * 12.0) as u8;

        let inset = 2.0;
        let rect = egui::Rect::from_min_max(
            egui::pos2(screen.left() + inset, screen.top() + inset),
            egui::pos2(screen.right() - inset, screen.bottom() - inset),
        );

        painter.rect_stroke(
            rect,
            0.0,
            Stroke::new(
                0.5 + phase * 0.5,
                Color32::from_rgba_premultiplied(
                    t.primary().r(),
                    t.primary().g(),
                    t.primary().b(),
                    alpha,
                ),
            ),
            StrokeKind::Outside,
        );

        // Corner tick marks (EVA-style corner indicators)
        let tick_len = 8.0;
        let tick_offset = 6.0;
        let tick_color = Color32::from_rgba_premultiplied(
            t.accent().r(),
            t.accent().g(),
            t.accent().b(),
            (20.0 + phase * 15.0) as u8,
        );
        let ts = Stroke::new(1.0, tick_color);

        // Top-left corner ticks
        painter.line_segment(
            [
                egui::pos2(rect.left() + tick_offset, rect.top()),
                egui::pos2(rect.left() + tick_offset + tick_len, rect.top()),
            ],
            ts,
        );
        painter.line_segment(
            [
                egui::pos2(rect.left(), rect.top() + tick_offset),
                egui::pos2(rect.left(), rect.top() + tick_offset + tick_len),
            ],
            ts,
        );

        // Top-right
        painter.line_segment(
            [
                egui::pos2(rect.right() - tick_offset - tick_len, rect.top()),
                egui::pos2(rect.right() - tick_offset, rect.top()),
            ],
            ts,
        );
        painter.line_segment(
            [
                egui::pos2(rect.right(), rect.top() + tick_offset),
                egui::pos2(rect.right(), rect.top() + tick_offset + tick_len),
            ],
            ts,
        );

        // Bottom-left
        painter.line_segment(
            [
                egui::pos2(rect.left() + tick_offset, rect.bottom()),
                egui::pos2(rect.left() + tick_offset + tick_len, rect.bottom()),
            ],
            ts,
        );
        painter.line_segment(
            [
                egui::pos2(rect.left(), rect.bottom() - tick_offset - tick_len),
                egui::pos2(rect.left(), rect.bottom() - tick_offset),
            ],
            ts,
        );

        // Bottom-right
        painter.line_segment(
            [
                egui::pos2(rect.right() - tick_offset - tick_len, rect.bottom()),
                egui::pos2(rect.right() - tick_offset, rect.bottom()),
            ],
            ts,
        );
        painter.line_segment(
            [
                egui::pos2(rect.right(), rect.bottom() - tick_offset - tick_len),
                egui::pos2(rect.right(), rect.bottom() - tick_offset),
            ],
            ts,
        );

        // Request repaint for animation
        ctx.request_repaint();
    }

}
