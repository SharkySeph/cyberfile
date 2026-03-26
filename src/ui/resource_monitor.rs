use eframe::egui::{self, Color32, RichText, Stroke};
use std::collections::VecDeque;

use crate::app::CyberFile;
use crate::theme::{self, CyberTheme};

impl CyberFile {
    pub(crate) fn render_resource_monitor(&mut self, ctx: &egui::Context) {
        let t = self.current_theme;

        egui::SidePanel::right("resource_monitor_panel")
            .default_width(280.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(t.surface())
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .stroke(Stroke::new(1.0, t.border_dim())),
            )
            .show(ctx, |ui| {
                // ── Header ─────────────────────────────────────
                ui.label(
                    RichText::new("\u{250C}\u{2500} VITAL SIGNS \u{2500}\u{2510}")
                        .color(t.primary())
                        .monospace()
                        .size(14.0)
                        .strong(),
                );
                ui.label(
                    RichText::new("  SYSTEM DIAGNOSTIC v2.1")
                        .color(t.text_dim())
                        .monospace()
                        .size(10.0),
                );
                ui.add_space(6.0);

                // ── CPU ────────────────────────────────────────
                let global_cpu = self.sys_info.global_cpu_usage();
                let cpu_color = threat_color(global_cpu, t);

                ui.label(
                    RichText::new("\u{25C8} CPU LOAD")
                        .color(t.primary())
                        .monospace()
                        .size(11.0)
                        .strong(),
                );

                // CPU bar
                ui.horizontal(|ui| {
                    let bar = render_bar(global_cpu, 20);
                    ui.label(
                        RichText::new(format!("  {} {:.1}%", bar, global_cpu))
                            .color(cpu_color)
                            .monospace()
                            .size(11.0),
                    );
                });

                // Per-core mini view
                let cpus: Vec<(String, f32)> = self
                    .sys_info
                    .cpus()
                    .iter()
                    .enumerate()
                    .map(|(i, cpu)| (format!("C{:02}", i), cpu.cpu_usage()))
                    .collect();

                if !cpus.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        for (name, usage) in &cpus {
                            let mini_bar = render_bar(*usage, 6);
                            let color = threat_color(*usage, t);
                            ui.label(
                                RichText::new(format!("{}:{}", name, mini_bar))
                                    .color(color)
                                    .monospace()
                                    .size(9.0),
                            );
                        }
                    });
                }

                ui.add_space(4.0);

                // CPU sparkline
                if !self.cpu_history.is_empty() {
                    ui.label(
                        RichText::new(format!("  ECG: {}", render_sparkline(&self.cpu_history)))
                            .color(t.primary_dim())
                            .monospace()
                            .size(9.0),
                    );
                }

                ui.add_space(6.0);
                theme::cyber_separator_themed(ui, t.border_dim());
                ui.add_space(2.0);

                // ── MEMORY ─────────────────────────────────────
                let total_mem = self.sys_info.total_memory();
                let used_mem = self.sys_info.used_memory();
                let mem_pct = if total_mem > 0 {
                    used_mem as f32 / total_mem as f32 * 100.0
                } else {
                    0.0
                };
                let mem_color = threat_color(mem_pct, t);

                ui.label(
                    RichText::new("\u{25C8} MEMORY BANK")
                        .color(t.primary())
                        .monospace()
                        .size(11.0)
                        .strong(),
                );

                ui.horizontal(|ui| {
                    let bar = render_bar(mem_pct, 20);
                    ui.label(
                        RichText::new(format!("  {} {:.1}%", bar, mem_pct))
                            .color(mem_color)
                            .monospace()
                            .size(11.0),
                    );
                });

                ui.label(
                    RichText::new(format!(
                        "  {} / {}",
                        bytesize::ByteSize(used_mem),
                        bytesize::ByteSize(total_mem)
                    ))
                    .color(t.text_dim())
                    .monospace()
                    .size(10.0),
                );

                // Swap
                let total_swap = self.sys_info.total_swap();
                let used_swap = self.sys_info.used_swap();
                if total_swap > 0 {
                    let swap_pct = used_swap as f32 / total_swap as f32 * 100.0;
                    ui.label(
                        RichText::new(format!(
                            "  SWAP: {} / {} ({:.0}%)",
                            bytesize::ByteSize(used_swap),
                            bytesize::ByteSize(total_swap),
                            swap_pct
                        ))
                        .color(t.text_dim())
                        .monospace()
                        .size(10.0),
                    );
                }

                // Memory sparkline
                if !self.mem_history.is_empty() {
                    ui.label(
                        RichText::new(format!("  ECG: {}", render_sparkline(&self.mem_history)))
                            .color(t.primary_dim())
                            .monospace()
                            .size(9.0),
                    );
                }

                ui.add_space(6.0);
                theme::cyber_separator_themed(ui, t.border_dim());
                ui.add_space(2.0);

                // ── DISKS ──────────────────────────────────────
                ui.label(
                    RichText::new("\u{25C8} STORAGE NODES")
                        .color(t.primary())
                        .monospace()
                        .size(11.0)
                        .strong(),
                );

                for disk in self.sys_disks.list() {
                    let total = disk.total_space();
                    let avail = disk.available_space();
                    if total == 0 {
                        continue;
                    }
                    let used = total - avail;
                    let pct = used as f32 / total as f32 * 100.0;
                    let color = threat_color(pct, t);
                    let mount = disk.mount_point().to_string_lossy();
                    let bar = render_bar(pct, 12);

                    ui.label(
                        RichText::new(format!("  {} {}", mount, bar))
                            .color(color)
                            .monospace()
                            .size(10.0),
                    );
                    ui.label(
                        RichText::new(format!(
                            "    {} / {} ({:.0}%)",
                            bytesize::ByteSize(used),
                            bytesize::ByteSize(total),
                            pct
                        ))
                        .color(t.text_dim())
                        .monospace()
                        .size(9.0),
                    );
                }

                ui.add_space(6.0);
                theme::cyber_separator_themed(ui, t.border_dim());
                ui.add_space(2.0);

                // ── THREAT ASSESSMENT ──────────────────────────
                let threat_level = if global_cpu > 90.0 || mem_pct > 95.0 {
                    ("CRITICAL", t.danger())
                } else if global_cpu > 75.0 || mem_pct > 85.0 {
                    ("ELEVATED", t.warning())
                } else if global_cpu > 50.0 || mem_pct > 70.0 {
                    ("MODERATE", t.primary())
                } else {
                    ("NOMINAL", t.success())
                };

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("\u{25B6} THREAT LEVEL:")
                            .color(t.text_dim())
                            .monospace()
                            .size(11.0),
                    );
                    // Blinking effect for critical
                    let visible = if threat_level.0 == "CRITICAL" {
                        (self.frame_count / 30) % 2 == 0
                    } else {
                        true
                    };
                    if visible {
                        ui.label(
                            RichText::new(threat_level.0)
                                .color(threat_level.1)
                                .monospace()
                                .size(11.0)
                                .strong(),
                        );
                    }
                });

                ui.add_space(8.0);
                ui.label(
                    RichText::new("\u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}")
                        .color(t.border_dim())
                        .monospace()
                        .size(10.0),
                );

                // Request repaint for live updates
                ctx.request_repaint_after(std::time::Duration::from_secs(2));
            });
    }
}

// ── Helper Functions ──────────────────────────────────────────────

fn render_bar(pct: f32, width: usize) -> String {
    let filled = ((pct / 100.0) * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    format!(
        "[{}{}]",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(empty)
    )
}

fn render_sparkline(data: &VecDeque<f32>) -> String {
    let blocks = [
        '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}',
        '\u{2588}',
    ];
    let max = data
        .iter()
        .cloned()
        .fold(f32::MIN, f32::max)
        .max(1.0);
    let recent: Vec<&f32> = data
        .iter()
        .rev()
        .take(30)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    recent
        .iter()
        .map(|&&v| {
            let idx = ((v / max) * 7.0).round() as usize;
            blocks[idx.min(7)]
        })
        .collect()
}

fn threat_color(pct: f32, t: CyberTheme) -> Color32 {
    if pct > 90.0 {
        t.danger()
    } else if pct > 75.0 {
        t.warning()
    } else if pct > 50.0 {
        t.primary()
    } else {
        t.success()
    }
}
