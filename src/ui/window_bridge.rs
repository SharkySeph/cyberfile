use eframe::egui::{self, RichText, ScrollArea, Stroke};

use crate::app::CyberFile;
use crate::integrations::windows::WmBackend;
use crate::theme::*;

impl CyberFile {
    pub(crate) fn render_window_bridge(&mut self, ctx: &egui::Context) {
        if self.window_bridge_detached {
            let t = self.current_theme;
            let viewport_id = egui::ViewportId::from_hash_of("window_bridge_viewport");
            let builder = egui::ViewportBuilder::default()
                .with_title("CYBERFILE // TACTICAL BRIDGE")
                .with_inner_size([720.0, 500.0])
                .with_min_inner_size([400.0, 300.0]);

            ctx.show_viewport_immediate(viewport_id, builder, |ctx, _class| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.window_bridge_detached = false;
                    self.window_bridge_open = false;
                }
                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(t.surface())
                            .inner_margin(egui::Margin::symmetric(10, 8)),
                    )
                    .show(ctx, |ui| {
                        self.render_window_bridge_content(ui, true);
                    });
            });
        } else {
            let t = self.current_theme;
            let mut open = self.window_bridge_open;

            egui::Window::new(
                RichText::new("┌─ TACTICAL BRIDGE ─┐")
                    .color(t.primary())
                    .monospace()
                    .strong(),
            )
            .open(&mut open)
            .default_width(700.0)
            .default_height(480.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(t.surface())
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .stroke(Stroke::new(1.0, t.border_dim())),
            )
            .show(ctx, |ui| {
                self.render_window_bridge_content(ui, false);
            });

            self.window_bridge_open = open;
        }
    }

    fn render_window_bridge_content(&mut self, ui: &mut egui::Ui, detached: bool) {
        let t = self.current_theme;

        // ── Header Row ─────────────────────────────
        ui.horizontal(|ui| {
            section_header(ui, "TACTICAL BRIDGE", t.primary());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !detached {
                    if ui
                        .button(RichText::new("⏏").color(t.text_dim()).monospace())
                        .on_hover_text("Detach to viewport")
                        .clicked()
                    {
                        self.window_bridge_detached = true;
                    }
                }
                if ui
                    .button(RichText::new("⟳").color(t.accent()).monospace())
                    .on_hover_text("Refresh")
                    .clicked()
                {
                    self.refresh_window_bridge(true);
                }

                // Show detected backend
                if let Some(backend) = self.wm_backend {
                    ui.label(
                        RichText::new(format!("[{}]", backend.label()))
                            .color(t.success())
                            .monospace(),
                    );
                }
            });
        });
        ui.add_space(4.0);

        // Auto-refresh every 3 seconds
        if self.wm_last_refresh.elapsed().as_secs() >= 3 {
            self.refresh_window_bridge(false);
        }

        let Some(backend) = self.wm_backend else {
            ui.label(
                RichText::new("⚠ No supported WM detected (Hyprland / Sway / i3)")
                    .color(t.text_dim())
                    .monospace(),
            );
            return;
        };

        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            // ── Tab Row ────────────────────────────
            ui.horizontal(|ui| {
                for tab in &[WindowBridgeTab::Windows, WindowBridgeTab::Workspaces] {
                    let selected = self.window_bridge_tab == *tab;
                    let label = tab.label();
                    let color = if selected { t.primary() } else { t.text_dim() };
                    let resp = ui.selectable_label(
                        selected,
                        RichText::new(label).color(color).monospace().strong(),
                    );
                    if resp.clicked() {
                        self.window_bridge_tab = *tab;
                    }
                }
            });
            ui.add_space(4.0);
            cyber_separator_themed(ui, t.border_dim());
            ui.add_space(4.0);

            match self.window_bridge_tab {
                WindowBridgeTab::Windows => self.render_wm_windows(ui, backend),
                WindowBridgeTab::Workspaces => self.render_wm_workspaces(ui, backend),
            }
        });
    }

    fn render_wm_windows(&mut self, ui: &mut egui::Ui, backend: WmBackend) {
        let t = self.current_theme;

        if self.wm_windows.is_empty() {
            ui.label(RichText::new("No windows detected").color(t.text_dim()).monospace());
            return;
        }

        let windows = self.wm_windows.clone();
        for win in &windows {
            ui.horizontal(|ui| {
                let focus_indicator = if win.focused { "▶" } else { " " };
                let focus_color = if win.focused { t.primary() } else { t.text_dim() };
                ui.label(RichText::new(focus_indicator).color(focus_color).monospace());

                // Class/app name
                if !win.class.is_empty() {
                    ui.label(
                        RichText::new(&win.class)
                            .color(t.accent())
                            .monospace()
                            .strong(),
                    );
                }

                // Title (truncated)
                let title_display = if win.title.len() > 50 {
                    format!("{}…", &win.title[..49])
                } else {
                    win.title.clone()
                };
                ui.label(RichText::new(&title_display).color(t.text_primary()).monospace());

                // Workspace badge
                if !win.workspace.is_empty() {
                    ui.label(
                        RichText::new(format!("[{}]", win.workspace))
                            .color(t.text_dim())
                            .monospace(),
                    );
                }
            });

            // Action buttons
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                if !win.focused {
                    if ui
                        .button(RichText::new("FOCUS").color(t.accent()).monospace().small())
                        .clicked()
                    {
                        let _ = crate::integrations::windows::focus_window(backend, &win.id);
                        self.refresh_window_bridge(true);
                    }
                }
                if ui
                    .button(RichText::new("CLOSE").color(t.danger()).monospace().small())
                    .clicked()
                {
                    let _ = crate::integrations::windows::close_window(backend, &win.id);
                    self.refresh_window_bridge(true);
                }

                // Move to workspace — show workspace selector
                if !self.wm_workspaces.is_empty() {
                    let ws_names: Vec<String> = self
                        .wm_workspaces
                        .iter()
                        .map(|ws| ws.name.clone())
                        .collect();
                    for ws_name in &ws_names {
                        if *ws_name != win.workspace {
                            if ui
                                .button(
                                    RichText::new(format!("→{}", ws_name))
                                        .color(t.text_dim())
                                        .monospace()
                                        .small(),
                                )
                                .on_hover_text(format!("Move to workspace {}", ws_name))
                                .clicked()
                            {
                                let _ = crate::integrations::windows::move_window_to_workspace(
                                    backend, &win.id, ws_name,
                                );
                                self.refresh_window_bridge(true);
                            }
                        }
                    }
                }
            });
            ui.add_space(2.0);
        }
    }

    fn render_wm_workspaces(&mut self, ui: &mut egui::Ui, backend: WmBackend) {
        let t = self.current_theme;

        if self.wm_workspaces.is_empty() {
            ui.label(RichText::new("No workspaces detected").color(t.text_dim()).monospace());
            return;
        }

        let workspaces = self.wm_workspaces.clone();
        for ws in &workspaces {
            ui.horizontal(|ui| {
                let focus_indicator = if ws.focused { "▶" } else { " " };
                let name_color = if ws.focused { t.primary() } else { t.text_primary() };
                ui.label(
                    RichText::new(focus_indicator)
                        .color(if ws.focused { t.primary() } else { t.text_dim() })
                        .monospace(),
                );
                ui.label(
                    RichText::new(&ws.name)
                        .color(name_color)
                        .monospace()
                        .strong(),
                );

                if ws.window_count > 0 {
                    ui.label(
                        RichText::new(format!("({} window{})", ws.window_count, if ws.window_count == 1 { "" } else { "s" }))
                            .color(t.text_dim())
                            .monospace(),
                    );
                } else {
                    ui.label(RichText::new("(empty)").color(t.text_dim()).monospace());
                }

                if !ws.focused {
                    if ui
                        .button(RichText::new("SWITCH").color(t.accent()).monospace().small())
                        .clicked()
                    {
                        let _ = crate::integrations::windows::switch_workspace(backend, &ws.name);
                        self.refresh_window_bridge(true);
                    }
                }
            });
        }
    }
}

// ── Tab enum ───────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowBridgeTab {
    Windows,
    Workspaces,
}

impl Default for WindowBridgeTab {
    fn default() -> Self {
        Self::Windows
    }
}

impl WindowBridgeTab {
    fn label(self) -> &'static str {
        match self {
            Self::Windows => "WINDOWS",
            Self::Workspaces => "WORKSPACES",
        }
    }
}
