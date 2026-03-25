use eframe::egui::{self, RichText, Stroke, Color32};

use crate::app::CyberFile;
use crate::theme::CyberTheme;

impl CyberFile {
    pub(crate) fn render_settings_panel(&mut self, ctx: &egui::Context) {
        let t = self.current_theme;
        let mut open = self.settings_panel_open;

        egui::Window::new(
            RichText::new("\u{2B22} SYSTEM CONFIGURATION")
                .color(t.primary())
                .monospace()
                .strong(),
        )
        .open(&mut open)
        .default_width(460.0)
        .default_height(520.0)
        .resizable(true)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::new()
                .fill(t.bg_dark())
                .stroke(Stroke::new(1.5, t.primary_dim()))
                .inner_margin(12.0),
        )
        .show(ctx, |ui| {
            // ── Header bar ──
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("OPERATOR ACCESS // v0.1")
                        .color(t.text_dim())
                        .monospace()
                        .size(9.0),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (led_rect, _) =
                        ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                    ui.painter()
                        .circle_filled(led_rect.center(), 3.5, t.success());
                    ui.label(
                        RichText::new("LIVE")
                            .color(t.success())
                            .monospace()
                            .size(9.0),
                    );
                });
            });

            ui.add_space(4.0);
            Self::hex_accent_line(ui, t);
            ui.add_space(6.0);

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // ═══════════════════════════════════════
                    // VISUAL PROTOCOL
                    // ═══════════════════════════════════════
                    Self::section_hex_header(ui, t, "VISUAL PROTOCOL", t.primary());
                    ui.add_space(4.0);

                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        for &theme_opt in CyberTheme::all() {
                            let selected = self.current_theme == theme_opt;
                            let frame_stroke = if selected {
                                Stroke::new(1.5, theme_opt.primary())
                            } else {
                                Stroke::new(0.5, t.border_dim())
                            };
                            let fill = if selected {
                                Color32::from_rgba_premultiplied(
                                    theme_opt.primary().r(),
                                    theme_opt.primary().g(),
                                    theme_opt.primary().b(),
                                    15,
                                )
                            } else {
                                t.surface()
                            };

                            let resp = egui::Frame::new()
                                .fill(fill)
                                .stroke(frame_stroke)
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.set_width(90.0);
                                    // Color bar
                                    let (bar_rect, _) = ui.allocate_exact_size(
                                        egui::vec2(90.0, 4.0),
                                        egui::Sense::hover(),
                                    );
                                    let half = bar_rect.width() / 2.0;
                                    ui.painter().rect_filled(
                                        egui::Rect::from_min_size(
                                            bar_rect.min,
                                            egui::vec2(half, 4.0),
                                        ),
                                        0.0,
                                        theme_opt.primary(),
                                    );
                                    ui.painter().rect_filled(
                                        egui::Rect::from_min_size(
                                            bar_rect.min + egui::vec2(half, 0.0),
                                            egui::vec2(half, 4.0),
                                        ),
                                        0.0,
                                        theme_opt.accent(),
                                    );

                                    ui.add_space(4.0);
                                    let label_color = if selected {
                                        theme_opt.primary()
                                    } else {
                                        t.text_dim()
                                    };
                                    ui.label(
                                        RichText::new(theme_opt.name())
                                            .color(label_color)
                                            .monospace()
                                            .size(10.0)
                                            .strong(),
                                    );
                                    ui.label(
                                        RichText::new(theme_opt.description())
                                            .color(t.text_dim())
                                            .monospace()
                                            .size(8.0),
                                    );
                                })
                                .response;

                            if resp.interact(egui::Sense::click()).clicked() {
                                self.current_theme = theme_opt;
                                self.theme_applied = false;
                                self.settings.theme = theme_opt.id().to_string();
                            }
                        }
                    });

                    ui.add_space(8.0);
                    Self::hex_accent_line(ui, t);
                    ui.add_space(6.0);

                    // ═══════════════════════════════════════
                    // DISPLAY MATRIX
                    // ═══════════════════════════════════════
                    Self::section_hex_header(ui, t, "DISPLAY MATRIX", t.primary());
                    ui.add_space(4.0);

                    let display_toggles: Vec<(&str, bool, &str, Color32)> = vec![
                        ("SCANLINES", self.scanlines_enabled, "F11", t.primary()),
                        ("CRT EFFECT", self.crt_effect, "F12", t.primary()),
                        ("CLOAKED FILES", self.show_hidden, "Ctrl+H", t.warning()),
                        ("BOOT SEQUENCE", self.settings.boot_sequence, "", t.primary()),
                        ("DATA RAIN", self.data_rain_enabled, "F10", t.success()),
                        ("NEON GLOW", self.neon_glow, "F8", t.accent()),
                        ("CHROM ABERR", self.chromatic_aberration, "F6", t.danger()),
                        ("HOLO NOISE", self.holographic_noise, "", t.primary()),
                        ("LOW MOTION", self.reduced_motion, "", t.text_dim()),
                        ("HI CONTRAST", self.high_contrast, "", t.warning()),
                    ];

                    for (i, (name, state, key, on_color)) in display_toggles.iter().enumerate() {
                        if Self::toggle_row(ui, t, name, *state, key, *on_color) {
                            match i {
                                0 => {
                                    self.scanlines_enabled = !self.scanlines_enabled;
                                    self.settings.scanlines_enabled = self.scanlines_enabled;
                                }
                                1 => {
                                    self.crt_effect = !self.crt_effect;
                                    self.settings.crt_effect = self.crt_effect;
                                }
                                2 => {
                                    self.show_hidden = !self.show_hidden;
                                    self.settings.show_hidden = self.show_hidden;
                                    self.load_current_directory();
                                }
                                3 => {
                                    self.settings.boot_sequence = !self.settings.boot_sequence;
                                }
                                4 => {
                                    self.data_rain_enabled = !self.data_rain_enabled;
                                }
                                5 => {
                                    self.neon_glow = !self.neon_glow;
                                    self.settings.neon_glow = self.neon_glow;
                                }
                                6 => {
                                    self.chromatic_aberration = !self.chromatic_aberration;
                                    self.settings.chromatic_aberration = self.chromatic_aberration;
                                }
                                7 => {
                                    self.holographic_noise = !self.holographic_noise;
                                    self.settings.holographic_noise = self.holographic_noise;
                                }
                                8 => {
                                    self.reduced_motion = !self.reduced_motion;
                                    self.settings.reduced_motion = self.reduced_motion;
                                    // When reduced motion enabled, disable animated effects
                                    if self.reduced_motion {
                                        self.data_rain_enabled = false;
                                        self.chromatic_aberration = false;
                                        self.holographic_noise = false;
                                    }
                                }
                                9 => {
                                    self.high_contrast = !self.high_contrast;
                                    self.settings.high_contrast = self.high_contrast;
                                }
                                _ => {}
                            }
                        }
                    }

                    ui.add_space(8.0);
                    Self::hex_accent_line(ui, t);
                    ui.add_space(6.0);

                    // ═══════════════════════════════════════
                    // INTERFACE
                    // ═══════════════════════════════════════
                    Self::section_hex_header(ui, t, "INTERFACE", t.primary());
                    ui.add_space(4.0);

                    let interface_toggles: Vec<(&str, bool, &str, Color32)> = vec![
                        ("SIDEBAR", self.sidebar_visible, "Ctrl+B", t.primary()),
                        ("VITAL SIGNS", self.resource_monitor_visible, "F3", t.primary()),
                        ("CONFIRM DEL", self.settings.confirm_delete, "", t.success()),
                        ("SOUND FX", self.sound_enabled, "", t.accent()),
                    ];

                    for (i, (name, state, key, on_color)) in interface_toggles.iter().enumerate() {
                        if Self::toggle_row(ui, t, name, *state, key, *on_color) {
                            match i {
                                0 => self.sidebar_visible = !self.sidebar_visible,
                                1 => {
                                    self.resource_monitor_visible =
                                        !self.resource_monitor_visible;
                                }
                                2 => {
                                    self.settings.confirm_delete = !self.settings.confirm_delete;
                                }
                                3 => {
                                    self.sound_enabled = !self.sound_enabled;
                                    self.settings.sound_enabled = self.sound_enabled;
                                }
                                _ => {}
                            }
                        }
                    }

                    ui.add_space(8.0);
                    Self::hex_accent_line(ui, t);
                    ui.add_space(6.0);

                    // ═══════════════════════════════════════
                    // SUBSYSTEM LINKS
                    // ═══════════════════════════════════════
                    Self::section_hex_header(ui, t, "SUBSYSTEM LINKS", t.accent());
                    ui.add_space(4.0);

                    // Terminal emulator
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("JACK-IN:")
                                .color(t.text_dim())
                                .monospace()
                                .size(10.0),
                        );
                        ui.add_sized(
                            [160.0, 18.0],
                            egui::TextEdit::singleline(&mut self.settings.terminal_emulator)
                                .font(egui::FontId::monospace(10.0))
                                .text_color(t.text_primary())
                                .hint_text(
                                    RichText::new("auto-detect").color(t.text_dim()),
                                ),
                        );
                        if let Some(detected) = self.settings.resolved_terminal() {
                            let (led_rect, _) =
                                ui.allocate_exact_size(egui::vec2(6.0, 6.0), egui::Sense::hover());
                            ui.painter()
                                .circle_filled(led_rect.center(), 3.0, t.success());
                            ui.label(
                                RichText::new(detected)
                                    .color(t.success())
                                    .monospace()
                                    .size(9.0),
                            );
                        } else {
                            let (led_rect, _) =
                                ui.allocate_exact_size(egui::vec2(6.0, 6.0), egui::Sense::hover());
                            ui.painter()
                                .circle_filled(led_rect.center(), 3.0, t.danger());
                            ui.label(
                                RichText::new("OFFLINE")
                                    .color(t.danger())
                                    .monospace()
                                    .size(9.0),
                            );
                        }
                    });

                    ui.add_space(4.0);

                    // Protocol bindings
                    ui.label(
                        RichText::new("PROTOCOL BINDINGS:")
                            .color(t.text_dim())
                            .monospace()
                            .size(10.0),
                    );

                    let openers: Vec<(String, String)> = self
                        .settings
                        .custom_openers
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();

                    let mut to_remove: Option<String> = None;

                    if openers.is_empty() {
                        ui.label(
                            RichText::new("  No bindings \u{2014} use ROUTE TO... to assign")
                                .color(t.text_dim())
                                .monospace()
                                .size(9.0),
                        );
                    } else {
                        for (ext, app) in &openers {
                            ui.horizontal(|ui| {
                                ui.add_space(8.0);
                                let badge_text = format!(" .{} ", ext.to_uppercase());
                                ui.label(
                                    RichText::new(&badge_text)
                                        .color(t.bg_dark())
                                        .background_color(t.warning())
                                        .monospace()
                                        .size(9.0),
                                );
                                ui.label(
                                    RichText::new("\u{2192}")
                                        .color(t.border_dim())
                                        .monospace()
                                        .size(10.0),
                                );
                                ui.label(
                                    RichText::new(app)
                                        .color(t.text_primary())
                                        .monospace()
                                        .size(10.0),
                                );
                                if ui
                                    .button(
                                        RichText::new("\u{2715}")
                                            .color(t.danger())
                                            .monospace()
                                            .size(9.0),
                                    )
                                    .on_hover_text("Unbind protocol")
                                    .clicked()
                                {
                                    to_remove = Some(ext.clone());
                                }
                            });
                        }
                    }
                    if let Some(key) = to_remove {
                        self.settings.custom_openers.remove(&key);
                    }

                    ui.add_space(8.0);
                    Self::hex_accent_line(ui, t);
                    ui.add_space(6.0);

                    // ═══════════════════════════════════════
                    // STORAGE
                    // ═══════════════════════════════════════
                    Self::section_hex_header(ui, t, "STORAGE", t.text_dim());
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(format!(
                            "  CONFIG: {}",
                            crate::config::Settings::config_path().display()
                        ))
                        .color(t.text_dim())
                        .monospace()
                        .size(9.0),
                    );

                    ui.add_space(10.0);

                    // ── Action Bar ──
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                RichText::new("\u{2B22} SAVE CONFIGURATION")
                                    .color(t.success())
                                    .monospace()
                                    .size(11.0),
                            )
                            .clicked()
                        {
                            self.settings.save();
                            self.status_message = "Configuration manifest saved".into();
                        }
                        ui.add_space(8.0);
                        if ui
                            .button(
                                RichText::new("\u{2715} CLOSE")
                                    .color(t.text_dim())
                                    .monospace()
                                    .size(11.0),
                            )
                            .clicked()
                        {
                            self.settings_panel_open = false;
                        }
                    });

                    ui.add_space(8.0);

                    // ═══════════════════════════════════════
                    // NEURAL SHORTCUTS (collapsible)
                    // ═══════════════════════════════════════
                    let id = ui.make_persistent_id("shortcuts_collapsible");
                    egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(),
                        id,
                        false,
                    )
                    .show_header(ui, |ui| {
                        ui.label(
                            RichText::new("\u{2B21} NEURAL SHORTCUTS")
                                .color(t.primary_dim())
                                .monospace()
                                .size(10.0),
                        );
                    })
                    .body(|ui| {
                        let shortcuts = [
                            ("F1", "Configuration"),
                            ("F2", "Rename"),
                            ("F3", "Vital Signs"),
                            ("F5", "Refresh"),
                            ("F10", "Data Rain"),
                            ("F11", "Scanlines"),
                            ("F12", "CRT Effect"),
                            ("Ctrl+1/2/3/4", "List/Grid/Hive/Viewer"),
                            ("Ctrl+B", "Sidebar"),
                            ("Ctrl+F", "fzf Search"),
                            ("Ctrl+H", "Hidden Files"),
                            ("Ctrl+P", "Preview Panel"),
                            ("Ctrl+T", "New Tab"),
                            ("Ctrl+W", "Close Tab"),
                            ("Ctrl+C/X/V", "Copy/Cut/Paste"),
                            ("Ctrl+Shift+N", "New Folder"),
                            ("Backspace", "Go Up"),
                            ("Delete", "Quarantine"),
                        ];

                        ui.columns(2, |cols| {
                            let mid = (shortcuts.len() + 1) / 2;
                            for (key, action) in &shortcuts[..mid] {
                                cols[0].label(
                                    RichText::new(format!("{:>12}  {}", key, action))
                                        .color(t.text_dim())
                                        .monospace()
                                        .size(9.0),
                                );
                            }
                            for (key, action) in &shortcuts[mid..] {
                                cols[1].label(
                                    RichText::new(format!("{:>12}  {}", key, action))
                                        .color(t.text_dim())
                                        .monospace()
                                        .size(9.0),
                                );
                            }
                        });
                    });
                });
        });

        self.settings_panel_open = open;
    }

    // ── Settings panel helpers ─────────────────────────────

    /// Hex-accented section header: ⬢ LABEL ────────
    fn section_hex_header(ui: &mut egui::Ui, t: CyberTheme, label: &str, color: Color32) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("\u{2B22}")
                    .color(color)
                    .monospace()
                    .size(12.0),
            );
            ui.label(
                RichText::new(label)
                    .color(color)
                    .monospace()
                    .size(11.0)
                    .strong(),
            );
            let remaining = ui.available_width() - 4.0;
            if remaining > 10.0 {
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(remaining, 1.0), egui::Sense::hover());
                let y = rect.center().y;
                ui.painter().line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    Stroke::new(0.5, t.border_dim()),
                );
            }
        });
    }

    /// Decorative hex-dot accent line between sections
    fn hex_accent_line(ui: &mut egui::Ui, t: CyberTheme) {
        let dot_color = Color32::from_rgba_premultiplied(
            t.primary().r(),
            t.primary().g(),
            t.primary().b(),
            60,
        );
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 8.0), egui::Sense::hover());
        let painter = ui.painter();
        let spacing = 12.0;
        let count = (width / spacing) as usize;
        for i in 0..count {
            let x = rect.left() + (i as f32) * spacing + spacing * 0.5;
            let y = rect.center().y;
            if i % 3 == 0 {
                // Hex dot
                let r = 2.5;
                let pts: Vec<egui::Pos2> = (0..6)
                    .map(|k| {
                        let angle = std::f32::consts::FRAC_PI_3 * k as f32;
                        egui::pos2(x + r * angle.cos(), y + r * angle.sin())
                    })
                    .collect();
                painter.add(egui::Shape::convex_polygon(
                    pts,
                    dot_color,
                    Stroke::NONE,
                ));
            } else {
                painter.circle_filled(egui::pos2(x, y), 1.0, dot_color);
            }
        }
    }

    /// Toggle row with LED indicator: [LED] label [ON/OFF] shortcut
    fn toggle_row(
        ui: &mut egui::Ui,
        t: CyberTheme,
        name: &str,
        state: bool,
        key: &str,
        on_color: Color32,
    ) -> bool {
        let resp = ui.horizontal(|ui| {
            ui.add_space(4.0);

            // Status LED
            let (led_rect, _) =
                ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
            let led_color = if state {
                on_color
            } else {
                Color32::from_rgba_premultiplied(
                    t.text_dim().r(),
                    t.text_dim().g(),
                    t.text_dim().b(),
                    80,
                )
            };
            ui.painter()
                .circle_filled(led_rect.center(), 3.0, led_color);
            if state {
                ui.painter().circle_stroke(
                    led_rect.center(),
                    4.5,
                    Stroke::new(
                        0.5,
                        Color32::from_rgba_premultiplied(
                            on_color.r(),
                            on_color.g(),
                            on_color.b(),
                            60,
                        ),
                    ),
                );
            }

            // Name
            let text_color = if state { t.text_primary() } else { t.text_dim() };
            ui.label(
                RichText::new(format!("{:<14}", name))
                    .color(text_color)
                    .monospace()
                    .size(11.0),
            );

            // Status badge
            let (badge_text, badge_color) = if state {
                ("ON ", on_color)
            } else {
                ("OFF", t.text_dim())
            };
            ui.label(
                RichText::new(format!("[{}]", badge_text))
                    .color(badge_color)
                    .monospace()
                    .size(10.0),
            );

            // Shortcut hint
            if !key.is_empty() {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(key)
                            .color(t.text_dim())
                            .monospace()
                            .size(9.0),
                    );
                });
            }
        });

        resp.response.interact(egui::Sense::click()).clicked()
    }
}
