use eframe::egui::{self, Color32};

use crate::app::CyberFile;

/// Data rain / matrix-style falling characters effect
/// Inspired by Ghost in the Shell data streams
impl CyberFile {
    pub(crate) fn render_data_rain(&self, ctx: &egui::Context) {
        if !self.data_rain_enabled {
            return;
        }

        let screen = ctx.screen_rect();
        let t = self.current_theme;

        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Background,
            egui::Id::new("data_rain"),
        ));

        // Characters to use — mix of katakana, latin, digits
        let chars: &[char] = &[
            'ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク', 'ケ', 'コ',
            'サ', 'シ', 'ス', 'セ', 'ソ', 'タ', 'チ', 'ツ', 'テ', 'ト',
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
            'A', 'B', 'C', 'D', 'E', 'F',
            ':', '.', '/', '\\', '#', '@',
        ];

        let col_width: f32 = 16.0;
        let char_height: f32 = 14.0;
        let num_cols = (screen.width() / col_width) as usize;
        let time = self.frame_count as f32 * 0.05;

        let primary = t.primary();

        for col in 0..num_cols.min(self.data_rain_cols.len()) {
            let col_offset = self.data_rain_cols[col];
            let x = screen.left() + col as f32 * col_width;

            // Each column drops at a slightly different speed
            let speed = 0.3 + (col_offset * 0.02) % 0.5;
            let y_base = ((time * speed + col_offset) * char_height) % (screen.height() + 200.0) - 100.0;

            // Trail of characters
            let trail_len = 8 + (col_offset as usize % 8);
            for i in 0..trail_len {
                let y = y_base - i as f32 * char_height;
                if y < screen.top() || y > screen.bottom() {
                    continue;
                }

                // Alpha fades along trail
                let alpha = if i == 0 {
                    180 // head is brightest
                } else {
                    (120.0 * (1.0 - i as f32 / trail_len as f32)).max(0.0) as u8
                };

                if alpha < 10 {
                    continue;
                }

                // Pick character pseudo-randomly based on position and frame
                let char_idx =
                    ((col * 37 + i * 13 + self.frame_count as usize / 3) % chars.len()) as usize;
                let ch = chars[char_idx];

                let color = if i == 0 {
                    // Head of trail — white-ish
                    Color32::from_rgba_premultiplied(
                        primary.r().saturating_add(80),
                        primary.g().saturating_add(80),
                        primary.b().saturating_add(80),
                        alpha,
                    )
                } else {
                    Color32::from_rgba_premultiplied(
                        primary.r(),
                        primary.g(),
                        primary.b(),
                        alpha,
                    )
                };

                painter.text(
                    egui::pos2(x, y),
                    egui::Align2::LEFT_TOP,
                    ch.to_string(),
                    egui::FontId::monospace(11.0),
                    color,
                );
            }
        }

        // Request continuous repaint for animation
        ctx.request_repaint();
    }
}
