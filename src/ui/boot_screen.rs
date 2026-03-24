use eframe::egui::{self, RichText};

use crate::app::CyberFile;
use crate::theme::*;

struct BootLine {
    time_ms: u64,
    text: &'static str,
    color: egui::Color32,
}

const BOOT_LINES: &[BootLine] = &[
    BootLine { time_ms: 0,    text: "[SYSTEM] CYBERFILE v0.1.0", color: TEXT_DIM },
    BootLine { time_ms: 150,  text: "[SYSTEM] Initializing kernel interface... OK", color: TEXT_DIM },
    BootLine { time_ms: 350,  text: "[SYSTEM] Mounting filesystem nodes...", color: TEXT_DIM },
    BootLine { time_ms: 550,  text: "[  OK  ] /home — USER DATA SECTOR", color: SUCCESS },
    BootLine { time_ms: 700,  text: "[  OK  ] /media — EXTERNAL NODES", color: SUCCESS },
    BootLine { time_ms: 850,  text: "[  OK  ] /tmp — VOLATILE CACHE", color: SUCCESS },
    BootLine { time_ms: 1050, text: "[SYSTEM] Loading neural interface...", color: TEXT_DIM },
    BootLine { time_ms: 1350, text: "[SYSTEM] Indexing data constructs...", color: TEXT_DIM },
    BootLine { time_ms: 1600, text: "[SYSTEM] STATUS: OPERATIONAL", color: CYAN },
    BootLine { time_ms: 2000, text: "", color: TEXT_DIM },
    BootLine { time_ms: 2200, text: "> WELCOME BACK, OPERATOR.", color: CYAN },
];

const BOOT_DURATION_MS: u64 = 2800;

impl CyberFile {
    pub(crate) fn render_boot_screen(&mut self, ctx: &egui::Context) {
        let elapsed_ms = self.boot_start.elapsed().as_millis() as u64;

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BG_DARK).inner_margin(40.0))
            .show(ctx, |ui| {
                ui.add_space(40.0);

                // Render boot lines that have appeared
                for line in BOOT_LINES {
                    if elapsed_ms >= line.time_ms {
                        if line.text.is_empty() {
                            ui.add_space(8.0);
                        } else {
                            ui.label(
                                RichText::new(line.text)
                                    .color(line.color)
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
                ui.label(RichText::new(bar).color(CYAN).monospace().size(14.0));

                ui.add_space(24.0);

                if elapsed_ms > 800 {
                    ui.label(
                        RichText::new("Press any key or click to skip...")
                            .color(TEXT_DIM)
                            .monospace()
                            .size(11.0),
                    );
                }

                // Check for skip
                if ui.input(|i| {
                    i.pointer.any_click()
                        || i.keys_down.iter().next().is_some()
                        || i.key_pressed(egui::Key::Escape)
                        || i.key_pressed(egui::Key::Enter)
                        || i.key_pressed(egui::Key::Space)
                }) {
                    self.boot_complete = true;
                    self.load_current_directory();
                }

                // Auto-complete boot
                if elapsed_ms >= BOOT_DURATION_MS {
                    self.boot_complete = true;
                    self.load_current_directory();
                }
            });

        // Keep animating
        ctx.request_repaint();
    }
}
