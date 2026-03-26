use eframe::egui::{self, RichText, ScrollArea, Stroke};
use std::collections::VecDeque;

use crate::app::CyberFile;
use crate::theme::*;

impl CyberFile {
    pub(crate) fn render_network_mesh(&mut self, ctx: &egui::Context) {
        if self.network_mesh_detached {
            let t = self.current_theme;
            let viewport_id = egui::ViewportId::from_hash_of("network_mesh_viewport");
            let builder = egui::ViewportBuilder::default()
                .with_title("CYBERFILE // NETWORK MESH")
                .with_inner_size([780.0, 560.0])
                .with_min_inner_size([400.0, 300.0]);

            ctx.show_viewport_immediate(viewport_id, builder, |ctx, _class| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.network_mesh_detached = false;
                    self.network_mesh_open = false;
                }
                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(t.surface())
                            .inner_margin(egui::Margin::symmetric(10, 8)),
                    )
                    .show(ctx, |ui| {
                        self.render_network_mesh_content(ui, true);
                    });
            });
        } else {
            let t = self.current_theme;
            let mut open = self.network_mesh_open;

            egui::Window::new(
                RichText::new("┌─ NETWORK MESH ─┐")
                    .color(t.primary())
                    .monospace()
                    .strong(),
            )
            .open(&mut open)
            .default_width(760.0)
            .default_height(540.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(t.surface())
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .stroke(Stroke::new(1.0, t.border_dim())),
            )
            .show(ctx, |ui| {
                self.render_network_mesh_content(ui, false);
            });

            self.network_mesh_open = open;
        }
    }

    fn render_network_mesh_content(&mut self, ui: &mut egui::Ui, detached: bool) {
        let t = self.current_theme;

        // ── Header Row ─────────────────────────────
        ui.horizontal(|ui| {
            section_header(ui, "NETWORK MESH", t.primary());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !detached {
                    if ui
                        .button(RichText::new("⏏").color(t.text_dim()).monospace())
                        .on_hover_text("Detach to viewport")
                        .clicked()
                    {
                        self.network_mesh_detached = true;
                    }
                }
                if ui
                    .button(RichText::new("⟳").color(t.accent()).monospace())
                    .on_hover_text("Refresh")
                    .clicked()
                {
                    self.refresh_network_mesh(true);
                }
            });
        });
        ui.add_space(4.0);

        // Auto-refresh every 5 seconds
        if self.network_last_refresh.elapsed().as_secs() >= 5 {
            self.refresh_network_mesh(false);
        }

        if !self.network_nmcli_available {
            ui.label(
                RichText::new("⚠ nmcli not found — NetworkManager required")
                    .color(t.text_dim())
                    .monospace(),
            );
            return;
        }

        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            // ── Tab Row ────────────────────────────
            ui.horizontal(|ui| {
                for tab in &[NetworkMeshTab::Interfaces, NetworkMeshTab::Wifi, NetworkMeshTab::Vpn, NetworkMeshTab::Throughput] {
                    let selected = self.network_mesh_tab == *tab;
                    let label = tab.label();
                    let color = if selected { t.primary() } else { t.text_dim() };
                    let resp = ui.selectable_label(
                        selected,
                        RichText::new(label).color(color).monospace().strong(),
                    );
                    if resp.clicked() {
                        self.network_mesh_tab = *tab;
                    }
                }
            });
            ui.add_space(4.0);
            cyber_separator_themed(ui, t.border_dim());
            ui.add_space(4.0);

            match self.network_mesh_tab {
                NetworkMeshTab::Interfaces => self.render_network_interfaces(ui),
                NetworkMeshTab::Wifi => self.render_network_wifi(ui),
                NetworkMeshTab::Vpn => self.render_network_vpn(ui),
                NetworkMeshTab::Throughput => self.render_network_throughput(ui),
            }
        });
    }

    fn render_network_interfaces(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;

        if self.network_interfaces.is_empty() {
            ui.label(RichText::new("No interfaces detected").color(t.text_dim()).monospace());
            return;
        }

        for iface in self.network_interfaces.clone() {
            let state_color = match iface.state.as_str() {
                "connected" => t.success(),
                "disconnected" => t.text_dim(),
                _ => t.warning(),
            };

            ui.horizontal(|ui| {
                let icon = match iface.iface_type.as_str() {
                    "wifi" => "📶",
                    "ethernet" => "🔌",
                    "bridge" => "🌉",
                    "loopback" => "↩",
                    _ => "📡",
                };
                ui.label(RichText::new(icon).monospace());
                ui.label(
                    RichText::new(&iface.device)
                        .color(t.text_primary())
                        .monospace()
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("[{}]", iface.iface_type))
                        .color(t.text_dim())
                        .monospace(),
                );
                ui.label(RichText::new(&iface.state).color(state_color).monospace());
                if !iface.connection.is_empty() && iface.connection != "--" {
                    ui.label(
                        RichText::new(format!("→ {}", iface.connection))
                            .color(t.accent())
                            .monospace(),
                    );
                }
                if iface.state == "connected" {
                    if ui
                        .button(RichText::new("✖").color(t.danger()).monospace().small())
                        .on_hover_text("Disconnect")
                        .clicked()
                    {
                        let _ = crate::integrations::network::disconnect_device(&iface.device);
                        self.refresh_network_mesh(true);
                    }
                }
            });
        }
    }

    fn render_network_wifi(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;

        if self.network_wifi_list.is_empty() {
            ui.label(RichText::new("No Wi-Fi networks visible").color(t.text_dim()).monospace());
            return;
        }

        for net in self.network_wifi_list.clone() {
            ui.horizontal(|ui| {
                // Signal strength bar
                let bars = match net.signal {
                    0..=25 => "▂___",
                    26..=50 => "▂▄__",
                    51..=75 => "▂▄▆_",
                    _ => "▂▄▆█",
                };
                let signal_color = match net.signal {
                    0..=25 => t.danger(),
                    26..=50 => t.warning(),
                    _ => t.success(),
                };
                ui.label(RichText::new(bars).color(signal_color).monospace());

                let name_color = if net.in_use { t.primary() } else { t.text_primary() };
                ui.label(RichText::new(&net.ssid).color(name_color).monospace().strong());

                if net.in_use {
                    ui.label(RichText::new("●").color(t.success()).monospace());
                }

                ui.label(
                    RichText::new(format!("[{}]", net.security))
                        .color(t.text_dim())
                        .monospace(),
                );

                ui.label(
                    RichText::new(format!("{}%", net.signal))
                        .color(t.text_dim())
                        .monospace(),
                );

                if !net.in_use {
                    if ui
                        .button(RichText::new("CONNECT").color(t.accent()).monospace().small())
                        .clicked()
                    {
                        // For open networks, connect directly; for secured, this would need a dialog.
                        // For now, attempt open connect; user can use nmcli manually for password.
                        let _ = crate::integrations::network::connect_wifi(&net.ssid, None);
                        self.refresh_network_mesh(true);
                    }
                }
            });
        }
    }

    fn render_network_vpn(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;

        if self.network_vpn_list.is_empty() {
            ui.label(RichText::new("No VPN connections configured").color(t.text_dim()).monospace());
            return;
        }

        for vpn in self.network_vpn_list.clone() {
            ui.horizontal(|ui| {
                let icon = if vpn.active { "🔒" } else { "🔓" };
                let state_color = if vpn.active { t.success() } else { t.text_dim() };
                ui.label(RichText::new(icon).monospace());
                ui.label(
                    RichText::new(&vpn.name)
                        .color(t.text_primary())
                        .monospace()
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("[{}]", vpn.vpn_type))
                        .color(t.text_dim())
                        .monospace(),
                );
                ui.label(
                    RichText::new(if vpn.active { "ACTIVE" } else { "DOWN" })
                        .color(state_color)
                        .monospace(),
                );

                let btn_label = if vpn.active { "DISCONNECT" } else { "CONNECT" };
                let btn_color = if vpn.active { t.warning() } else { t.accent() };
                if ui
                    .button(RichText::new(btn_label).color(btn_color).monospace().small())
                    .clicked()
                {
                    let _ = crate::integrations::network::toggle_vpn(&vpn.name, !vpn.active);
                    self.refresh_network_mesh(true);
                }
            });
        }
    }

    fn render_network_throughput(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;

        if self.network_throughput_history.is_empty() {
            ui.label(
                RichText::new("Collecting throughput data...")
                    .color(t.text_dim())
                    .monospace(),
            );
            return;
        }

        for (device, history) in &self.network_throughput_history {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("📡 {}", device))
                        .color(t.text_primary())
                        .monospace()
                        .strong(),
                );
            });

            if let Some(latest) = history.back() {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("  ↓ {}/s", format_bytes(latest.0)))
                            .color(t.success())
                            .monospace(),
                    );
                    ui.label(
                        RichText::new(format!("  ↑ {}/s", format_bytes(latest.1)))
                            .color(t.accent())
                            .monospace(),
                    );
                });

                // Mini sparkline
                if history.len() > 1 {
                    let max_rx = history.iter().map(|h| h.0).max().unwrap_or(1).max(1);
                    let bars: String = history
                        .iter()
                        .map(|h| {
                            let frac = h.0 as f64 / max_rx as f64;
                            match (frac * 8.0) as u8 {
                                0 => ' ',
                                1 => '▁',
                                2 => '▂',
                                3 => '▃',
                                4 => '▄',
                                5 => '▅',
                                6 => '▆',
                                7 => '▇',
                                _ => '█',
                            }
                        })
                        .collect();
                    ui.label(RichText::new(format!("  RX ▕{}▏", bars)).color(t.success()).monospace());
                }
            }
            ui.add_space(2.0);
        }
    }
}

// ── Tab enum ───────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkMeshTab {
    Interfaces,
    Wifi,
    Vpn,
    Throughput,
}

impl Default for NetworkMeshTab {
    fn default() -> Self {
        Self::Interfaces
    }
}

impl NetworkMeshTab {
    fn label(self) -> &'static str {
        match self {
            Self::Interfaces => "INTERFACES",
            Self::Wifi => "WI-FI",
            Self::Vpn => "VPN TUNNELS",
            Self::Throughput => "THROUGHPUT",
        }
    }
}

// ── Throughput delta type ──────────────────────────────────
// Stored in app: VecDeque<(rx_delta_bytes, tx_delta_bytes)> per device
pub type ThroughputHistory = std::collections::BTreeMap<String, VecDeque<(u64, u64)>>;

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
