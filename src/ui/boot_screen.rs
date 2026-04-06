use eframe::egui::{self, Color32, RichText};

use crate::app::CyberFile;
use crate::theme::CyberTheme;

// ── CRT Star Wipe ──────────────────────────────────────────────────────
// Simulates an old CRT powering on: a bright horizontal slit in the
// center that expands vertically while a star/diamond shape blooms
// outward, then the whole screen "opens up" to reveal the boot text.
const STAR_WIPE_DURATION_MS: u64 = 700;

struct BootLine {
    time_ms: u64,
    text: &'static str,
    /// Which semantic color to use: 'd' = text_dim, 's' = success, 'p' = primary
    kind: char,
}

fn boot_color(kind: char, theme: CyberTheme) -> Color32 {
    match kind {
        's' => theme.success(),
        'p' => theme.primary(),
        _ => theme.text_dim(),
    }
}

const BOOT_LINES: &[BootLine] = &[
    BootLine { time_ms: 700,  text: "",                                              kind: 'v' },
    BootLine { time_ms: 850,  text: "[SYSTEM] Initializing kernel interface... OK",  kind: 'd' },
    BootLine { time_ms: 1050, text: "[SYSTEM] Mounting filesystem nodes...",         kind: 'd' },
    BootLine { time_ms: 1250, text: "[  OK  ] /home — USER DATA SECTOR",             kind: 's' },
    BootLine { time_ms: 1400, text: "[  OK  ] /media — EXTERNAL NODES",              kind: 's' },
    BootLine { time_ms: 1550, text: "[  OK  ] /tmp — VOLATILE CACHE",                kind: 's' },
    BootLine { time_ms: 1750, text: "[SYSTEM] Loading neural interface...",           kind: 'd' },
    BootLine { time_ms: 2050, text: "[SYSTEM] Indexing data constructs...",           kind: 'd' },
    BootLine { time_ms: 2300, text: "[SYSTEM] STATUS: OPERATIONAL",                  kind: 'p' },
    BootLine { time_ms: 2700, text: "",                                              kind: 'd' },
    BootLine { time_ms: 2900, text: "> WELCOME BACK, OPERATOR.",                     kind: 'p' },
];

const BOOT_DURATION_MS: u64 = 3500;

impl CyberFile {
    pub(crate) fn render_boot_screen(&mut self, ctx: &egui::Context) {
        let elapsed_ms = self.boot_start.elapsed().as_millis() as u64;
        let t = self.current_theme;
        let quick_slots = self.boot_scene_slots();
        let has_session_resume = self.has_session_resume();
        let mut restore_scene_id: Option<String> = None;
        let mut resume_session = false;
        let mut fresh_start = false;

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(t.bg_dark()).inner_margin(40.0))
            .show(ctx, |ui| {
                // ── CRT Star Wipe overlay ──────────────────────
                // During the first STAR_WIPE_DURATION_MS milliseconds we draw
                // a black overlay with a diamond/star-shaped hole expanding
                // from the centre, mimicking an old CRT powering on.
                if elapsed_ms < STAR_WIPE_DURATION_MS {
                    let screen = ctx.screen_rect();
                    let cx = screen.center().x;
                    let cy = screen.center().y;
                    let half_w = screen.width() * 0.5;
                    let half_h = screen.height() * 0.5;

                    // t_wipe goes 0→1 over the wipe duration (ease-out cubic)
                    let linear = elapsed_ms as f32 / STAR_WIPE_DURATION_MS as f32;
                    let ease = 1.0 - (1.0 - linear) * (1.0 - linear) * (1.0 - linear);

                    // Diamond radius in each axis
                    let rx = half_w * ease * 1.3; // a bit oversized so it fully clears
                    let ry = half_h * ease * 1.3;

                    let painter = ctx.layer_painter(egui::LayerId::new(
                        egui::Order::Foreground,
                        egui::Id::new("crt_star_wipe"),
                    ));

                    // Black overlay everywhere; punch out a diamond via 4 triangles
                    // Strategy: cover top, bottom, left, right trapezoids around
                    // the diamond hole.  We use a mesh for the star/diamond.
                    let black = Color32::BLACK;
                    let pc = t.primary();

                    // Four triangular "cover" regions outside the diamond:
                    // Top triangle: top-left corner → top-right corner → diamond-top
                    let d_top   = egui::pos2(cx, cy - ry);
                    let d_bot   = egui::pos2(cx, cy + ry);
                    let d_left  = egui::pos2(cx - rx, cy);
                    let d_right = egui::pos2(cx + rx, cy);

                    let tl = screen.left_top();
                    let tr = screen.right_top();
                    let bl = screen.left_bottom();
                    let br = screen.right_bottom();

                    // Top region (two triangles forming a quad from top edge → diamond top)
                    painter.add(egui::Shape::convex_polygon(
                        vec![tl, tr, d_right, d_top, d_left],
                        black,
                        egui::Stroke::NONE,
                    ));
                    // Bottom region
                    painter.add(egui::Shape::convex_polygon(
                        vec![bl, br, d_right, d_bot, d_left],
                        black,
                        egui::Stroke::NONE,
                    ));
                    // Left region
                    painter.add(egui::Shape::convex_polygon(
                        vec![tl, bl, d_left],
                        black,
                        egui::Stroke::NONE,
                    ));
                    // Right region
                    painter.add(egui::Shape::convex_polygon(
                        vec![tr, br, d_right],
                        black,
                        egui::Stroke::NONE,
                    ));

                    // Bright leading edge of the diamond (CRT phosphor glow)
                    let edge_alpha = ((1.0 - ease) * 200.0).min(180.0).max(0.0) as u8;
                    let glow = Color32::from_rgba_premultiplied(
                        (pc.r() as f32 * edge_alpha as f32 / 255.0) as u8,
                        (pc.g() as f32 * edge_alpha as f32 / 255.0) as u8,
                        (pc.b() as f32 * edge_alpha as f32 / 255.0) as u8,
                        edge_alpha,
                    );
                    painter.add(egui::Shape::closed_line(
                        vec![d_top, d_right, d_bot, d_left],
                        egui::Stroke::new(2.0 + (1.0 - ease) * 3.0, glow),
                    ));

                    // Bright horizontal scan-slit at centre (the classic CRT power-on line)
                    if ease < 0.5 {
                        let slit_alpha = ((0.5 - ease) * 2.0 * 255.0).min(220.0) as u8;
                        let slit_h = 2.0 + (1.0 - ease * 2.0) * 1.0;
                        let slit_w = screen.width() * (ease * 2.0).min(1.0);
                        let slit_color = Color32::from_rgba_premultiplied(
                            (pc.r() as f32 * slit_alpha as f32 / 255.0) as u8,
                            (pc.g() as f32 * slit_alpha as f32 / 255.0) as u8,
                            (pc.b() as f32 * slit_alpha as f32 / 255.0) as u8,
                            slit_alpha,
                        );
                        painter.rect_filled(
                            egui::Rect::from_center_size(
                                screen.center(),
                                egui::vec2(slit_w, slit_h),
                            ),
                            0.0,
                            slit_color,
                        );
                    }
                }

                ui.add_space(40.0);

                // Render boot lines that have appeared
                for line in BOOT_LINES {
                    if elapsed_ms >= line.time_ms {
                        if line.kind == 'v' {
                            // Dynamic version line from Cargo.toml
                            ui.label(
                                RichText::new(format!(
                                    "[SYSTEM] CYBERFILE v{}",
                                    env!("CARGO_PKG_VERSION")
                                ))
                                .color(boot_color('d', t))
                                .monospace()
                                .size(14.0),
                            );
                        } else if line.text.is_empty() {
                            ui.add_space(8.0);
                        } else {
                            ui.label(
                                RichText::new(line.text)
                                    .color(boot_color(line.kind, t))
                                    .monospace()
                                    .size(14.0),
                            );
                        }
                    }
                }

                ui.add_space(16.0);

                // Progress bar
                let progress = (elapsed_ms as f32 / BOOT_DURATION_MS as f32).min(1.0);
                let bar_width = 400.0;
                let filled = (bar_width * progress) as usize / 10;
                let empty = (bar_width as usize / 10).saturating_sub(filled);
                let bar = format!(
                    "[ BOOT ] {}{}  {:.0}%",
                    "█".repeat(filled),
                    "░".repeat(empty),
                    progress * 100.0
                );
                ui.label(RichText::new(bar).color(t.primary()).monospace().size(14.0));

                ui.add_space(24.0);

                if elapsed_ms > 1600 {
                    ui.label(
                        RichText::new(if has_session_resume {
                            "[ENTER] Resume last session deck   [0] Fresh start   [1-4] Restore quick scene"
                        } else {
                            "[ENTER] Continue   [1-4] Restore quick scene"
                        })
                        .color(t.text_dim())
                        .monospace()
                        .size(11.0),
                    );

                    ui.add_space(10.0);
                    egui::Frame::new()
                        .fill(t.surface())
                        .stroke(egui::Stroke::new(1.0, t.border_dim()))
                        .inner_margin(egui::Margin::symmetric(10, 8))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("BOOT DECK")
                                    .color(t.primary())
                                    .monospace()
                                    .size(10.5),
                            );
                            ui.add_space(4.0);

                            if has_session_resume {
                                if ui
                                    .button(
                                        RichText::new("[ENTER] RESUME LAST SESSION")
                                            .color(t.success())
                                            .monospace()
                                            .size(11.0),
                                    )
                                    .clicked()
                                {
                                    resume_session = true;
                                }
                            }

                            for (index, scene) in quick_slots.iter().enumerate() {
                                let label = if scene.pinned {
                                    format!("[{}] ★ {}", index + 1, scene.name)
                                } else {
                                    format!("[{}] {}", index + 1, scene.name)
                                };
                                if ui
                                    .button(
                                        RichText::new(label)
                                            .color(if scene.pinned { t.warning() } else { t.primary() })
                                            .monospace()
                                            .size(11.0),
                                    )
                                    .clicked()
                                {
                                    restore_scene_id = Some(scene.scene_id.clone());
                                }
                                if !scene.summary.trim().is_empty() {
                                    ui.label(
                                        RichText::new(&scene.summary)
                                            .color(t.text_dim())
                                            .monospace()
                                            .size(9.5),
                                    );
                                }
                            }

                            if has_session_resume {
                                ui.add_space(6.0);
                                if ui
                                    .button(
                                        RichText::new("[0] FRESH START")
                                            .color(t.text_dim())
                                            .monospace()
                                            .size(10.5),
                                    )
                                    .clicked()
                                {
                                    fresh_start = true;
                                }
                            }
                        });

                    ui.add_space(12.0);
                    ui.label(
                        RichText::new("Auto-resume engages when boot completes if no manual deck is selected")
                            .color(t.text_dim())
                            .monospace()
                            .size(11.0),
                    );
                }

                if ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space)) {
                    if has_session_resume {
                        resume_session = true;
                    } else {
                        fresh_start = true;
                    }
                }

                if ui.input(|i| i.key_pressed(egui::Key::Num0) || i.key_pressed(egui::Key::Escape)) {
                    fresh_start = true;
                }

                for (index, key) in [egui::Key::Num1, egui::Key::Num2, egui::Key::Num3, egui::Key::Num4]
                    .into_iter()
                    .enumerate()
                {
                    if ui.input(|i| i.key_pressed(key)) {
                        if let Some(scene) = quick_slots.get(index) {
                            restore_scene_id = Some(scene.scene_id.clone());
                        }
                    }
                }

                if restore_scene_id.is_none() && !resume_session && !fresh_start && ui.input(|i| i.pointer.any_click()) {
                    if has_session_resume {
                        resume_session = true;
                    } else {
                        fresh_start = true;
                    }
                }

                // Auto-complete boot
                if elapsed_ms >= BOOT_DURATION_MS {
                    if has_session_resume {
                        resume_session = true;
                    } else {
                        fresh_start = true;
                    }
                }
            });

        if let Some(scene_id) = restore_scene_id {
            self.queue_boot_scene_restore(scene_id);
        } else if resume_session {
            self.queue_boot_resume();
        } else if fresh_start {
            self.queue_boot_fresh_start();
        }

        // Keep animating
        ctx.request_repaint();
    }
}
