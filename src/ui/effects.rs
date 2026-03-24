use eframe::egui::{self, Color32, Stroke};

use crate::app::CyberFile;

impl CyberFile {
    pub(crate) fn render_effects(&self, ctx: &egui::Context) {
        let screen = ctx.screen_rect();

        // ── Scanlines ──────────────────────────────────────
        if self.scanlines_enabled {
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("scanlines"),
            ));
            let mut y = screen.top();
            while y < screen.bottom() {
                painter.line_segment(
                    [egui::pos2(screen.left(), y), egui::pos2(screen.right(), y)],
                    Stroke::new(1.0, Color32::from_rgba_premultiplied(0, 0, 0, 18)),
                );
                y += 3.0;
            }
        }

        // ── CRT Vignette ───────────────────────────────────
        if self.crt_effect {
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("crt_vignette"),
            ));

            let steps: usize = 15;
            let edge: f32 = 80.0;
            let step_size = edge / steps as f32;

            for i in 0..steps {
                let alpha = ((steps - i) as f32 / steps as f32 * 60.0) as u8;
                let offset = i as f32 * step_size;
                let color = Color32::from_rgba_premultiplied(0, 0, 0, alpha);

                // Top edge
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(screen.left(), screen.top() + offset),
                        egui::pos2(screen.right(), screen.top() + offset + step_size),
                    ),
                    0.0,
                    color,
                );
                // Bottom edge
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(screen.left(), screen.bottom() - offset - step_size),
                        egui::pos2(screen.right(), screen.bottom() - offset),
                    ),
                    0.0,
                    color,
                );
                // Left edge
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(screen.left() + offset, screen.top()),
                        egui::pos2(screen.left() + offset + step_size, screen.bottom()),
                    ),
                    0.0,
                    color,
                );
                // Right edge
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(screen.right() - offset - step_size, screen.top()),
                        egui::pos2(screen.right() - offset, screen.bottom()),
                    ),
                    0.0,
                    color,
                );
            }

            // Corner darkening (extra dark in corners)
            let corner_size: f32 = 120.0;
            let corner_steps: usize = 10;
            let block = egui::vec2(corner_size / corner_steps as f32, corner_size / corner_steps as f32);

            for cx in 0..corner_steps {
                for cy in 0..corner_steps {
                    let fx = cx as f32 / corner_steps as f32;
                    let fy = cy as f32 / corner_steps as f32;
                    let dist = (fx * fx + fy * fy).sqrt() / 1.414;
                    let alpha = ((1.0 - dist) * 30.0).max(0.0) as u8;
                    if alpha == 0 {
                        continue;
                    }
                    let color = Color32::from_rgba_premultiplied(0, 0, 0, alpha);
                    let dx = cx as f32 * block.x;
                    let dy = cy as f32 * block.y;

                    // Top-left
                    painter.rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(screen.left() + dx, screen.top() + dy),
                            block,
                        ),
                        0.0,
                        color,
                    );
                    // Top-right
                    painter.rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(screen.right() - dx - block.x, screen.top() + dy),
                            block,
                        ),
                        0.0,
                        color,
                    );
                    // Bottom-left
                    painter.rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(screen.left() + dx, screen.bottom() - dy - block.y),
                            block,
                        ),
                        0.0,
                        color,
                    );
                    // Bottom-right
                    painter.rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(screen.right() - dx - block.x, screen.bottom() - dy - block.y),
                            block,
                        ),
                        0.0,
                        color,
                    );
                }
            }
        }

        // ── Glitch Effect ──────────────────────────────────
        if self.glitch_active {
            if let Some(start) = self.glitch_start {
                let elapsed = start.elapsed().as_millis();
                if elapsed < 80 {
                    let painter = ctx.layer_painter(egui::LayerId::new(
                        egui::Order::Foreground,
                        egui::Id::new("glitch_effect"),
                    ));
                    let intensity = 1.0 - (elapsed as f32 / 80.0);
                    let t = self.current_theme;

                    // Subtle flash overlay
                    let alpha = (intensity * 12.0) as u8;
                    painter.rect_filled(
                        screen,
                        0.0,
                        Color32::from_rgba_premultiplied(
                            t.primary().r(),
                            t.primary().g(),
                            t.primary().b(),
                            alpha,
                        ),
                    );

                    // Thin glitch bars (fewer, subtler)
                    let bar_count = (intensity * 3.0) as usize;
                    let bar_height = 1.0;
                    for i in 0..bar_count {
                        let y = screen.top() + (i as f32 * 97.3) % screen.height();
                        let shift = (intensity * 8.0) * if i % 2 == 0 { 1.0 } else { -1.0 };
                        painter.rect_filled(
                            egui::Rect::from_min_size(
                                egui::pos2(screen.left() + shift, y),
                                egui::vec2(screen.width(), bar_height),
                            ),
                            0.0,
                            Color32::from_rgba_premultiplied(
                                t.accent().r(),
                                t.accent().g(),
                                t.accent().b(),
                                (intensity * 20.0) as u8,
                            ),
                        );
                    }

                    ctx.request_repaint();
                }
            }
        }

        // ── HUD Corner Brackets ────────────────────────────
        {
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("hud_brackets"),
            ));
            let t = self.current_theme;
            let inset = 4.0;
            let size = 20.0;
            let s = Stroke::new(1.0, t.border_dim());
            let r = egui::Rect::from_min_max(
                egui::pos2(screen.left() + inset, screen.top() + inset),
                egui::pos2(screen.right() - inset, screen.bottom() - inset),
            );

            // Top-left
            painter.line_segment([r.left_top(), egui::pos2(r.left() + size, r.top())], s);
            painter.line_segment([r.left_top(), egui::pos2(r.left(), r.top() + size)], s);
            // Top-right
            painter.line_segment([egui::pos2(r.right() - size, r.top()), r.right_top()], s);
            painter.line_segment([r.right_top(), egui::pos2(r.right(), r.top() + size)], s);
            // Bottom-left
            painter.line_segment(
                [egui::pos2(r.left(), r.bottom() - size), r.left_bottom()],
                s,
            );
            painter.line_segment(
                [r.left_bottom(), egui::pos2(r.left() + size, r.bottom())],
                s,
            );
            // Bottom-right
            painter.line_segment(
                [egui::pos2(r.right() - size, r.bottom()), r.right_bottom()],
                s,
            );
            painter.line_segment(
                [egui::pos2(r.right(), r.bottom() - size), r.right_bottom()],
                s,
            );
        }
    }
}
