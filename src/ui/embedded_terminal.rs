use eframe::egui::{self, RichText, ScrollArea, Stroke, TextEdit};
use std::io::Write;
use std::sync::{mpsc, Arc, Mutex};

use crate::app::CyberFile;
use crate::theme::*;

/// Information about an available shell on the system.
#[derive(Debug, Clone)]
pub(crate) struct ShellInfo {
    pub name: String,
    pub path: String,
}

/// A single interactive CLI session backed by a real PTY.
pub(crate) struct CliSession {
    pub shell: ShellInfo,
    pub output_lines: Vec<String>,
    pub scroll_to_bottom: bool,
    pub writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pub output_rx: mpsc::Receiver<String>,
    pub child: Box<dyn portable_pty::Child + Send + Sync>,
    pub alive: bool,
}

impl CyberFile {
    /// Detect available shells by parsing /etc/shells and checking existence.
    pub(crate) fn detect_available_shells(&mut self) {
        let mut shells = Vec::new();
        if let Ok(content) = std::fs::read_to_string("/etc/shells") {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if std::path::Path::new(line).exists() {
                    let name = std::path::Path::new(line)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    // Skip nologin/false shells
                    if name == "nologin" || name == "false" {
                        continue;
                    }
                    shells.push(ShellInfo {
                        name,
                        path: line.to_string(),
                    });
                }
            }
        }

        // Fallback if /etc/shells missing or empty
        if shells.is_empty() {
            for path in &["/bin/bash", "/bin/sh", "/usr/bin/zsh", "/usr/bin/fish"] {
                if std::path::Path::new(path).exists() {
                    let name = std::path::Path::new(path)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    shells.push(ShellInfo {
                        name,
                        path: path.to_string(),
                    });
                }
            }
        }

        // Deduplicate by path
        shells.dedup_by(|a, b| a.path == b.path);

        self.cli_available_shells = shells;

        // Default to user's SHELL env, or first available
        if let Ok(user_shell) = std::env::var("SHELL") {
            if let Some(idx) = self
                .cli_available_shells
                .iter()
                .position(|s| s.path == user_shell)
            {
                self.cli_selected_shell = idx;
            }
        }
    }

    /// Spawn a new interactive CLI session with the selected shell.
    pub(crate) fn spawn_cli_session(&mut self) {
        let shell = match self.cli_available_shells.get(self.cli_selected_shell) {
            Some(s) => s.clone(),
            None => {
                self.set_error("No shell available for CLI session".into());
                return;
            }
        };

        let pty_system = portable_pty::native_pty_system();
        let pair = match pty_system.openpty(portable_pty::PtySize {
            rows: 30,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        }) {
            Ok(p) => p,
            Err(e) => {
                self.set_error(format!("PTY allocation failed: {}", e));
                return;
            }
        };

        let mut cmd = portable_pty::CommandBuilder::new(&shell.path);
        cmd.cwd(&self.current_path);
        cmd.env("TERM", "dumb");

        let child = match pair.slave.spawn_command(cmd) {
            Ok(c) => c,
            Err(e) => {
                self.set_error(format!("Shell spawn failed: {}", e));
                return;
            }
        };

        // Drop slave — the child owns it now
        drop(pair.slave);

        let writer = match pair.master.take_writer() {
            Ok(w) => Arc::new(Mutex::new(w)),
            Err(e) => {
                self.set_error(format!("PTY writer failed: {}", e));
                return;
            }
        };

        let reader = match pair.master.try_clone_reader() {
            Ok(r) => r,
            Err(e) => {
                self.set_error(format!("PTY reader failed: {}", e));
                return;
            }
        };

        let (tx, rx) = mpsc::channel();

        // Background reader thread
        std::thread::spawn(move || {
            use std::io::Read;
            let mut reader = reader;
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let raw = &buf[..n];
                        let stripped = strip_ansi_escapes::strip(raw);
                        let text = String::from_utf8_lossy(&stripped).to_string();
                        if tx.send(text).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let session = CliSession {
            shell,
            output_lines: Vec::new(),
            scroll_to_bottom: true,
            writer,
            output_rx: rx,
            child,
            alive: true,
        };

        self.cli_sessions.push(session);
        self.cli_active_session = self.cli_sessions.len() - 1;
    }

    /// Poll all CLI sessions for new output.
    pub(crate) fn poll_cli_sessions(&mut self) {
        let mut any_output = false;
        for session in &mut self.cli_sessions {
            if !session.alive {
                continue;
            }
            while let Ok(chunk) = session.output_rx.try_recv() {
                any_output = true;
                // Split into lines, preserving partial lines
                let parts: Vec<&str> = chunk.split('\n').collect();
                for (i, part) in parts.iter().enumerate() {
                    // First fragment appends to the last line if it was partial
                    if i == 0 && !part.is_empty() {
                        let should_append = session
                            .output_lines
                            .last()
                            .map(|l| !l.ends_with('\n'))
                            .unwrap_or(false);
                        if should_append {
                            session.output_lines.last_mut().unwrap().push_str(part);
                            continue;
                        }
                    }
                    session.output_lines.push(part.to_string());
                }
                session.scroll_to_bottom = true;

                // Trim if too long
                if session.output_lines.len() > 5000 {
                    let drain = session.output_lines.len() - 5000;
                    session.output_lines.drain(..drain);
                }
            }

            // Check if child is still alive
            if let Ok(Some(_status)) = session.child.try_wait() {
                session.alive = false;
            }
        }
        if any_output {
            // Will be picked up on next poll cycle for repaint
        }
    }

    /// Send input text to the active CLI session.
    fn cli_send_input(&mut self, text: &str) {
        if let Some(session) = self.cli_sessions.get_mut(self.cli_active_session) {
            if session.alive {
                if let Ok(mut writer) = session.writer.lock() {
                    let _ = writer.write_all(text.as_bytes());
                    let _ = writer.flush();
                }
            }
        }
    }

    /// Send a signal byte to the active CLI session (e.g. Ctrl+C = 0x03).
    fn cli_send_signal(&mut self, byte: u8) {
        if let Some(session) = self.cli_sessions.get_mut(self.cli_active_session) {
            if session.alive {
                if let Ok(mut writer) = session.writer.lock() {
                    let _ = writer.write_all(&[byte]);
                    let _ = writer.flush();
                }
            }
        }
    }

    /// Kill the active CLI session.
    fn cli_kill_active_session(&mut self) {
        if let Some(session) = self.cli_sessions.get_mut(self.cli_active_session) {
            let _ = session.child.kill();
            session.alive = false;
        }
    }

    /// Open the embedded CLI (SHELL JACK) panel.
    pub(crate) fn open_shell_jack(&mut self) {
        if self.cli_available_shells.is_empty() {
            self.detect_available_shells();
        }
        self.cli_visible = true;
        // Auto-spawn a session if none exist
        if self.cli_sessions.is_empty() {
            self.spawn_cli_session();
        }
    }

    /// Render the embedded CLI as a detachable viewport / embedded window.
    pub(crate) fn render_shell_jack(&mut self, ctx: &egui::Context) {
        if self.cli_detached {
            let t = self.current_theme;
            let viewport_id = egui::ViewportId::from_hash_of("shell_jack_viewport");
            let builder = egui::ViewportBuilder::default()
                .with_title("CYBERFILE // SHELL JACK")
                .with_inner_size([780.0, 500.0])
                .with_min_inner_size([400.0, 250.0]);

            ctx.show_viewport_immediate(viewport_id, builder, |ctx, _class| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.cli_detached = false;
                    self.cli_visible = false;
                }
                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(t.bg_dark())
                            .inner_margin(egui::Margin::symmetric(10, 8)),
                    )
                    .show(ctx, |ui| {
                        self.render_shell_jack_content(ui, true);
                    });
            });
        } else {
            let t = self.current_theme;
            let mut open = self.cli_visible;

            egui::Window::new(
                RichText::new("┌─ SHELL JACK ─┐")
                    .color(t.primary())
                    .monospace()
                    .strong(),
            )
            .open(&mut open)
            .default_width(720.0)
            .default_height(420.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(t.bg_dark())
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .stroke(Stroke::new(1.0, t.border_dim())),
            )
            .show(ctx, |ui| {
                self.render_shell_jack_content(ui, false);
            });

            self.cli_visible = open;
        }
    }

    /// Inner content renderer for the Shell Jack panel.
    fn render_shell_jack_content(&mut self, ui: &mut egui::Ui, _is_viewport: bool) {
        let t = self.current_theme;

        // ── Toolbar ──────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("⏚ SHELL JACK")
                    .color(t.accent())
                    .monospace()
                    .size(11.0)
                    .strong(),
            );

            ui.separator();

            // Shell selector
            let current_shell_name = self
                .cli_available_shells
                .get(self.cli_selected_shell)
                .map(|s| s.name.clone())
                .unwrap_or_else(|| "none".into());

            egui::ComboBox::from_id_salt("shell_selector")
                .selected_text(
                    RichText::new(format!("⟐ {}", current_shell_name.to_uppercase()))
                        .color(t.primary())
                        .monospace()
                        .size(10.0),
                )
                .show_ui(ui, |ui| {
                    for (idx, shell) in self.cli_available_shells.iter().enumerate() {
                        let label = format!("{} ({})", shell.name, shell.path);
                        if ui
                            .selectable_label(
                                idx == self.cli_selected_shell,
                                RichText::new(&label)
                                    .color(t.text_primary())
                                    .monospace()
                                    .size(10.0),
                            )
                            .clicked()
                        {
                            self.cli_selected_shell = idx;
                        }
                    }
                });

            ui.separator();

            // Session tabs
            let session_count = self.cli_sessions.len();
            if session_count > 1 {
                for i in 0..session_count {
                    let is_active = i == self.cli_active_session;
                    let alive = self.cli_sessions[i].alive;
                    let label = format!(
                        "{}#{}",
                        if alive { "●" } else { "○" },
                        i + 1
                    );
                    let color = if is_active {
                        t.accent()
                    } else if alive {
                        t.text_dim()
                    } else {
                        t.danger()
                    };
                    if ui
                        .selectable_label(
                            is_active,
                            RichText::new(label).color(color).monospace().size(10.0),
                        )
                        .clicked()
                    {
                        self.cli_active_session = i;
                    }
                }
                ui.separator();
            }

            // New session button
            if ui
                .small_button(
                    RichText::new("+ NEW")
                        .color(t.success())
                        .monospace()
                        .size(9.0),
                )
                .clicked()
            {
                self.spawn_cli_session();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Close button
                if ui
                    .small_button(RichText::new("✕").color(t.danger()).monospace())
                    .clicked()
                {
                    self.cli_visible = false;
                    self.cli_detached = false;
                }

                // Detach / Attach button
                let detach_label = if self.cli_detached {
                    "⬡ ATTACH"
                } else {
                    "⬡ DETACH"
                };
                if ui
                    .small_button(
                        RichText::new(detach_label)
                            .color(t.primary())
                            .monospace()
                            .size(9.0),
                    )
                    .clicked()
                {
                    self.cli_detached = !self.cli_detached;
                }

                // Kill session button
                if let Some(session) = self.cli_sessions.get(self.cli_active_session) {
                    if session.alive {
                        if ui
                            .small_button(
                                RichText::new("KILL")
                                    .color(t.danger())
                                    .monospace()
                                    .size(9.0),
                            )
                            .clicked()
                        {
                            self.cli_kill_active_session();
                        }
                    }
                }

                // Clear output button
                if ui
                    .small_button(
                        RichText::new("CLEAR")
                            .color(t.text_dim())
                            .monospace()
                            .size(9.0),
                    )
                    .clicked()
                {
                    if let Some(session) = self.cli_sessions.get_mut(self.cli_active_session) {
                        session.output_lines.clear();
                    }
                }
            });
        });

        ui.add_space(2.0);
        cyber_separator_themed(ui, t.border_dim());
        ui.add_space(2.0);

        // ── Output Area ──────────────────────────────────────
        let active_idx = self.cli_active_session;
        let session_alive = self
            .cli_sessions
            .get(active_idx)
            .map(|s| s.alive)
            .unwrap_or(false);
        let shell_name = self
            .cli_sessions
            .get(active_idx)
            .map(|s| s.shell.name.clone())
            .unwrap_or_default();

        let available_height = ui.available_height();
        let output_height = (available_height - 28.0).max(60.0);

        ScrollArea::vertical()
            .id_salt(format!("cli_output_{}", active_idx))
            .stick_to_bottom(true)
            .max_height(output_height)
            .show(ui, |ui| {
                if let Some(session) = self.cli_sessions.get(active_idx) {
                    if session.output_lines.is_empty() && session.alive {
                        ui.label(
                            RichText::new(format!(
                                "// {} session active — awaiting input",
                                shell_name
                            ))
                            .color(t.text_dim())
                            .monospace()
                            .size(10.0),
                        );
                    }
                    for line in &session.output_lines {
                        ui.label(
                            RichText::new(line)
                                .color(t.text_primary())
                                .monospace()
                                .size(11.0),
                        );
                    }
                } else {
                    ui.label(
                        RichText::new("// no active session")
                            .color(t.text_dim())
                            .monospace()
                            .size(10.0),
                    );
                }
            });

        // ── Input Line ───────────────────────────────────────
        ui.horizontal(|ui| {
            let prompt_color = if session_alive {
                t.success()
            } else {
                t.danger()
            };
            ui.label(
                RichText::new(if session_alive { "▸" } else { "✕" })
                    .color(prompt_color)
                    .monospace()
                    .size(12.0),
            );

            let resp = ui.add(
                TextEdit::singleline(&mut self.cli_input_buffer)
                    .font(egui::FontId::monospace(12.0))
                    .text_color(t.text_primary())
                    .desired_width(ui.available_width() - 80.0)
                    .hint_text(
                        RichText::new(if session_alive {
                            "enter command..."
                        } else {
                            "session ended — start new"
                        })
                        .color(t.text_dim())
                        .monospace(),
                    ),
            );

            // Handle Enter key — send input + newline to the PTY
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if session_alive {
                    let input = format!("{}\n", self.cli_input_buffer);
                    self.cli_send_input(&input);
                    // Store in local history
                    let cmd = self.cli_input_buffer.trim().to_string();
                    if !cmd.is_empty() {
                        self.cli_history.push(cmd);
                        if self.cli_history.len() > 100 {
                            self.cli_history.remove(0);
                        }
                    }
                    self.cli_input_buffer.clear();
                    self.cli_history_pos = None;
                }
                // Re-focus the input field for subsequent commands
                resp.request_focus();
            }

            // Handle Ctrl+C — send interrupt
            if resp.has_focus() && ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::C)) {
                self.cli_send_signal(0x03); // ETX (Ctrl+C)
            }

            // Handle Ctrl+D — send EOF
            if resp.has_focus() && ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::D)) {
                self.cli_send_signal(0x04); // EOT (Ctrl+D)
            }

            // Handle Up/Down for local history navigation
            if resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                if !self.cli_history.is_empty() {
                    let pos = match self.cli_history_pos {
                        Some(p) => p.saturating_sub(1),
                        None => self.cli_history.len() - 1,
                    };
                    self.cli_history_pos = Some(pos);
                    self.cli_input_buffer = self.cli_history[pos].clone();
                }
            }
            if resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                if let Some(pos) = self.cli_history_pos {
                    if pos + 1 < self.cli_history.len() {
                        let new_pos = pos + 1;
                        self.cli_history_pos = Some(new_pos);
                        self.cli_input_buffer = self.cli_history[new_pos].clone();
                    } else {
                        self.cli_history_pos = None;
                        self.cli_input_buffer.clear();
                    }
                }
            }

            // Handle Tab — send to PTY for shell completion
            if resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Tab)) {
                if session_alive && !self.cli_input_buffer.is_empty() {
                    let partial = format!("{}\t", self.cli_input_buffer);
                    self.cli_send_input(&partial);
                    self.cli_input_buffer.clear();
                }
            }

            // Send button
            if ui
                .add_enabled(
                    session_alive,
                    egui::Button::new(
                        RichText::new("SEND")
                            .color(t.success())
                            .monospace()
                            .size(10.0),
                    ),
                )
                .clicked()
            {
                let input = format!("{}\n", self.cli_input_buffer);
                self.cli_send_input(&input);
                self.cli_input_buffer.clear();
            }
        });
    }
}
