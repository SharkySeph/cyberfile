use eframe::egui::{self, Color32, RichText, Stroke};
use std::path::Path;

use crate::app::CyberFile;

impl CyberFile {
    /// File preview panel — "DATA SCAN" module
    /// Shows text preview, image info, hex peek, or metadata
    pub(crate) fn render_preview_panel(&mut self, ctx: &egui::Context) {
        let t = self.current_theme;

        egui::SidePanel::right("preview_panel")
            .default_width(320.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(t.bg_dark())
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .stroke(Stroke::new(1.0, t.border_dim())),
            )
            .show(ctx, |ui| {
                // NERV-style header
                ui.label(
                    RichText::new("┌─── DATA SCAN ─── ACTIVE ───┐")
                        .color(t.primary())
                        .monospace()
                        .size(11.0)
                        .strong(),
                );
                ui.add_space(4.0);

                let entry = self
                    .selected
                    .and_then(|idx| self.entries.get(idx).cloned());

                match entry {
                    None => {
                        ui.add_space(40.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("NO TARGET SELECTED")
                                    .color(t.text_dim())
                                    .monospace()
                                    .size(13.0),
                            );
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("Select a construct to\ninitiate data scan")
                                    .color(t.text_dim())
                                    .monospace()
                                    .size(10.0),
                            );
                        });
                    }
                    Some(entry) => {
                        // File identity card
                        self.render_preview_identity(ui, &entry.name, &entry.path, entry.is_dir);
                        ui.add_space(6.0);

                        // Separator
                        let rect = ui.available_rect_before_wrap();
                        ui.painter().line_segment(
                            [
                                egui::pos2(rect.left(), rect.top()),
                                egui::pos2(rect.right(), rect.top()),
                            ],
                            Stroke::new(0.5, t.border_dim()),
                        );
                        ui.add_space(6.0);

                        // Metadata block
                        self.render_preview_metadata(ui, &entry.path, entry.is_dir, entry.size);

                        ui.add_space(8.0);

                        // Content preview
                        if !entry.is_dir {
                            self.render_preview_content(ui, &entry.path);
                        } else {
                            self.render_preview_dir_stats(ui, &entry.path);
                        }
                    }
                }

                // Bottom closer
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(
                        RichText::new("└─── END SCAN ───────────────┘")
                            .color(t.border_dim())
                            .monospace()
                            .size(10.0),
                    );
                });
            });
    }

    fn render_preview_identity(
        &self,
        ui: &mut egui::Ui,
        name: &str,
        path: &Path,
        is_dir: bool,
    ) {
        let t = self.current_theme;

        // Type badge — NERV segment style
        let type_label = if is_dir { "DIR" } else { "FILE" };
        let type_color = if is_dir { t.primary() } else { t.accent() };

        ui.horizontal(|ui| {
            egui::Frame::new()
                .fill(Color32::from_rgba_premultiplied(
                    type_color.r(),
                    type_color.g(),
                    type_color.b(),
                    30,
                ))
                .stroke(Stroke::new(1.0, type_color))
                .inner_margin(egui::Margin::symmetric(6, 1))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(type_label)
                            .color(type_color)
                            .monospace()
                            .size(9.0)
                            .strong(),
                    );
                });

            // Extension badge
            if !is_dir {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_uppercase();
                    egui::Frame::new()
                        .fill(Color32::from_rgba_premultiplied(
                            t.warning().r(),
                            t.warning().g(),
                            t.warning().b(),
                            20,
                        ))
                        .stroke(Stroke::new(0.5, t.warning()))
                        .inner_margin(egui::Margin::symmetric(4, 1))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(ext_str)
                                    .color(t.warning())
                                    .monospace()
                                    .size(9.0),
                            );
                        });
                }
            }
        });

        ui.add_space(4.0);

        // Name
        ui.label(
            RichText::new(name)
                .color(t.text_primary())
                .monospace()
                .size(14.0)
                .strong(),
        );

        // Full path (dimmed)
        ui.label(
            RichText::new(path.to_string_lossy().as_ref())
                .color(t.text_dim())
                .monospace()
                .size(9.0),
        );
    }

    fn render_preview_metadata(
        &self,
        ui: &mut egui::Ui,
        path: &Path,
        is_dir: bool,
        size: u64,
    ) {
        let t = self.current_theme;

        ui.label(
            RichText::new("│ CONSTRUCT PROFILE")
                .color(t.primary())
                .monospace()
                .size(10.0)
                .strong(),
        );
        ui.add_space(2.0);

        let meta_line = |ui: &mut egui::Ui, key: &str, val: &str, color: Color32| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("│ {}:", key))
                        .color(t.text_dim())
                        .monospace()
                        .size(10.0),
                );
                ui.label(
                    RichText::new(val)
                        .color(color)
                        .monospace()
                        .size(10.0),
                );
            });
        };

        if !is_dir {
            meta_line(ui, "SIZE", &bytesize::ByteSize(size).to_string(), t.text_primary());
        }

        if let Ok(meta) = std::fs::metadata(path) {
            if let Ok(modified) = meta.modified() {
                let dt: chrono::DateTime<chrono::Local> = modified.into();
                meta_line(ui, "MODIFIED", &dt.format("%Y-%m-%d %H:%M:%S").to_string(), t.text_primary());
            }
            if let Ok(accessed) = meta.accessed() {
                let dt: chrono::DateTime<chrono::Local> = accessed.into();
                meta_line(ui, "ACCESSED", &dt.format("%Y-%m-%d %H:%M:%S").to_string(), t.text_dim());
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                meta_line(ui, "INODE", &format!("{}", meta.ino()), t.text_dim());
                let uid = meta.uid();
                let gid = meta.gid();
                let user_name = uzers::get_user_by_uid(uid)
                    .map(|u| u.name().to_string_lossy().to_string())
                    .unwrap_or_else(|| uid.to_string());
                let group_name = uzers::get_group_by_gid(gid)
                    .map(|g| g.name().to_string_lossy().to_string())
                    .unwrap_or_else(|| gid.to_string());
                meta_line(ui, "OWNER", &format!("{} ({})", user_name, uid), t.text_dim());
                meta_line(ui, "GROUP", &format!("{} ({})", group_name, gid), t.text_dim());
                meta_line(
                    ui,
                    "ACCESS",
                    &format!("{:04o}", meta.mode() & 0o7777),
                    t.warning(),
                );
            }
        }
    }

    fn render_preview_content(&self, ui: &mut egui::Ui, path: &Path) {
        let t = self.current_theme;

        // Detect type by extension
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let is_text = matches!(
            ext.as_str(),
            "txt" | "md" | "rs" | "py" | "js" | "ts" | "toml" | "yaml" | "yml"
                | "json" | "xml" | "html" | "css" | "sh" | "bash" | "zsh"
                | "conf" | "cfg" | "ini" | "log" | "csv" | "c" | "cpp" | "h"
                | "hpp" | "java" | "go" | "rb" | "lua" | "vim" | "el" | "tex"
                | "makefile" | "dockerfile" | "gitignore" | ""
        );

        let is_image = matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" | "ico");
        let is_audio = matches!(ext.as_str(), "mp3" | "flac" | "wav" | "ogg" | "m4a" | "aac" | "opus");
        let is_video = matches!(ext.as_str(), "mp4" | "mkv" | "avi" | "webm" | "mov" | "flv");
        let is_archive = matches!(ext.as_str(), "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "zst");

        if is_text {
            ui.label(
                RichText::new("│ CONTENT DECODE ─────────────")
                    .color(t.primary())
                    .monospace()
                    .size(10.0)
                    .strong(),
            );
            ui.add_space(2.0);

            // Read first 80 lines (bounded: only read first 64KB)
            let read_result = (|| -> Result<String, std::io::Error> {
                use std::io::Read;
                let mut f = std::fs::File::open(path)?;
                let mut buf = vec![0u8; 65536]; // 64KB max
                let n = f.read(&mut buf)?;
                buf.truncate(n);
                Ok(String::from_utf8_lossy(&buf).to_string())
            })();

            match read_result {
                Ok(content) => {
                    egui::ScrollArea::vertical()
                        .max_height(ui.available_height() - 30.0)
                        .show(ui, |ui| {
                            let keywords = Self::keywords_for_ext(&ext);
                            for (i, line) in content.lines().take(80).enumerate() {
                                ui.horizontal(|ui| {
                                    // Line number
                                    ui.label(
                                        RichText::new(format!("{:>4} │", i + 1))
                                            .color(t.text_dim())
                                            .monospace()
                                            .size(9.5),
                                    );
                                    // Syntax-colored line
                                    Self::render_syntax_line(ui, line, &keywords, t);
                                });
                            }
                        });
                }
                Err(_) => {
                    ui.label(
                        RichText::new("│ ACCESS DENIED — cannot decode")
                            .color(t.danger())
                            .monospace()
                            .size(10.0),
                    );
                }
            }
        } else if is_image {
            self.render_preview_type_badge(ui, "IMAGE CONSTRUCT", "Visual data — preview via external viewer", t.accent());
        } else if is_audio {
            self.render_preview_type_badge(ui, "AUDIO STREAM", "Waveform data detected", t.success());
        } else if is_video {
            self.render_preview_type_badge(ui, "VIDEO FEED", "Motion capture data", t.warning());
        } else if is_archive {
            self.render_preview_type_badge(ui, "COMPRESSED ARCHIVE", "Packed data container", t.primary());

            // Try to list ZIP contents
            if ext == "zip" {
                match crate::filesystem::list_zip_contents(path) {
                    Ok(entries) => {
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(format!("│ {} entries in archive", entries.len()))
                                .color(t.warning())
                                .monospace()
                                .size(10.0),
                        );
                        ui.add_space(2.0);
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                for (name, size, is_dir) in entries.iter().take(50) {
                                    let icon = if *is_dir { "◆" } else { "◇" };
                                    let size_str = if *is_dir {
                                        String::new()
                                    } else {
                                        format!(" ({})", bytesize::ByteSize(*size))
                                    };
                                    ui.label(
                                        RichText::new(format!("  {} {}{}", icon, name, size_str))
                                            .color(if *is_dir { t.primary() } else { t.text_primary() })
                                            .monospace()
                                            .size(9.0),
                                    );
                                }
                                if entries.len() > 50 {
                                    ui.label(
                                        RichText::new(format!("  ... and {} more", entries.len() - 50))
                                            .color(t.text_dim())
                                            .monospace()
                                            .size(9.0),
                                    );
                                }
                            });
                    }
                    Err(e) => {
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(format!("│ Cannot read archive: {}", e))
                                .color(t.danger())
                                .monospace()
                                .size(9.0),
                        );
                    }
                }
            } else {
                // Non-ZIP archives — show file info
                if let Ok(output) = std::process::Command::new("file")
                    .arg(path)
                    .output()
                {
                    let info = String::from_utf8_lossy(&output.stdout);
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!("│ {}", info.trim()))
                            .color(t.text_dim())
                            .monospace()
                            .size(9.0),
                    );
                }
            }
        } else {
            // Binary — show hex peek
            self.render_hex_peek(ui, path);
        }
    }

    fn render_preview_type_badge(&self, ui: &mut egui::Ui, label: &str, desc: &str, color: Color32) {
        let t = self.current_theme;
        ui.add_space(12.0);
        ui.vertical_centered(|ui| {
            egui::Frame::new()
                .fill(Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 20))
                .stroke(Stroke::new(1.0, color))
                .inner_margin(egui::Margin::symmetric(12, 6))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(label)
                            .color(color)
                            .monospace()
                            .size(12.0)
                            .strong(),
                    );
                });
            ui.add_space(4.0);
            ui.label(
                RichText::new(desc)
                    .color(t.text_dim())
                    .monospace()
                    .size(10.0),
            );
        });
    }

    fn render_hex_peek(&self, ui: &mut egui::Ui, path: &Path) {
        let t = self.current_theme;

        ui.label(
            RichText::new("│ RAW DECODE ── HEX SCAN ─────")
                .color(t.primary())
                .monospace()
                .size(10.0)
                .strong(),
        );
        ui.add_space(2.0);

        // Bounded read: only read first 256 bytes for hex peek
        let read_result = (|| -> Result<Vec<u8>, std::io::Error> {
            use std::io::Read;
            let mut f = std::fs::File::open(path)?;
            let mut buf = vec![0u8; 256];
            let n = f.read(&mut buf)?;
            buf.truncate(n);
            Ok(buf)
        })();

        match read_result {
            Ok(bytes) => {
                let show = bytes.iter().take(256);
                let lines: Vec<String> = show
                    .collect::<Vec<_>>()
                    .chunks(16)
                    .enumerate()
                    .map(|(i, chunk)| {
                        let hex: String = chunk
                            .iter()
                            .map(|b| format!("{:02X}", b))
                            .collect::<Vec<_>>()
                            .join(" ");
                        let ascii: String = chunk
                            .iter()
                            .map(|b| {
                                let c = **b as char;
                                if c.is_ascii_graphic() || c == ' ' {
                                    c
                                } else {
                                    '.'
                                }
                            })
                            .collect();
                        format!("{:06X} │ {:<48} │ {}", i * 16, hex, ascii)
                    })
                    .collect();

                egui::ScrollArea::vertical()
                    .max_height(ui.available_height() - 30.0)
                    .show(ui, |ui| {
                        for line in &lines {
                            ui.label(
                                RichText::new(line)
                                    .color(t.text_primary())
                                    .monospace()
                                    .size(9.0),
                            );
                        }
                    });
            }
            Err(_) => {
                ui.label(
                    RichText::new("│ ACCESS DENIED")
                        .color(t.danger())
                        .monospace()
                        .size(10.0),
                );
            }
        }
    }

    fn render_preview_dir_stats(&self, ui: &mut egui::Ui, path: &Path) {
        let t = self.current_theme;

        ui.label(
            RichText::new("│ SECTOR ANALYSIS ────────────")
                .color(t.primary())
                .monospace()
                .size(10.0)
                .strong(),
        );
        ui.add_space(4.0);

        // Count children
        match std::fs::read_dir(path) {
            Ok(entries) => {
                let mut dirs = 0u32;
                let mut files = 0u32;
                let mut hidden = 0u32;
                let mut total_size: u64 = 0;

                for e in entries.flatten() {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.starts_with('.') {
                        hidden += 1;
                    }
                    if let Ok(m) = e.metadata() {
                        if m.is_dir() {
                            dirs += 1;
                        } else {
                            files += 1;
                            total_size += m.len();
                        }
                    }
                }

                let kv = |ui: &mut egui::Ui, k: &str, v: &str, c: Color32| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("│ {}:", k)).color(t.text_dim()).monospace().size(10.0));
                        ui.label(RichText::new(v).color(c).monospace().size(10.0));
                    });
                };

                kv(ui, "SUB-SECTORS", &dirs.to_string(), t.primary());
                kv(ui, "CONSTRUCTS", &files.to_string(), t.text_primary());
                kv(ui, "CLOAKED", &hidden.to_string(), t.text_dim());
                kv(ui, "SECTOR SIZE", &bytesize::ByteSize(total_size).to_string(), t.warning());
            }
            Err(_) => {
                ui.label(
                    RichText::new("│ SECTOR ACCESS DENIED")
                        .color(t.danger())
                        .monospace()
                        .size(10.0),
                );
            }
        }
    }

    /// Get keywords for syntax highlighting based on file extension.
    fn keywords_for_ext(ext: &str) -> Vec<&'static str> {
        match ext {
            "rs" => vec![
                "fn", "let", "mut", "pub", "use", "mod", "struct", "enum", "impl", "trait",
                "self", "Self", "super", "crate", "const", "static", "type", "where",
                "if", "else", "match", "for", "while", "loop", "break", "continue", "return",
                "as", "in", "ref", "move", "async", "await", "unsafe", "dyn", "true", "false",
            ],
            "py" => vec![
                "def", "class", "import", "from", "return", "if", "elif", "else",
                "for", "while", "break", "continue", "with", "as", "try", "except",
                "finally", "raise", "yield", "lambda", "pass", "None", "True", "False",
                "self", "async", "await", "in", "not", "and", "or", "is",
            ],
            "js" | "ts" => vec![
                "function", "const", "let", "var", "return", "if", "else", "for",
                "while", "do", "switch", "case", "break", "continue", "class",
                "extends", "import", "export", "from", "default", "new", "this",
                "async", "await", "try", "catch", "finally", "throw", "typeof",
                "true", "false", "null", "undefined",
            ],
            "c" | "cpp" | "h" | "hpp" => vec![
                "int", "float", "double", "char", "void", "bool", "long", "short",
                "unsigned", "signed", "const", "static", "extern", "struct", "union",
                "enum", "typedef", "class", "public", "private", "protected",
                "if", "else", "for", "while", "do", "switch", "case", "break",
                "continue", "return", "include", "define", "true", "false", "nullptr",
            ],
            "go" => vec![
                "func", "package", "import", "return", "if", "else", "for", "range",
                "switch", "case", "break", "continue", "struct", "interface", "type",
                "var", "const", "defer", "go", "chan", "map", "select", "nil", "true", "false",
            ],
            "sh" | "bash" | "zsh" => vec![
                "if", "then", "else", "elif", "fi", "for", "while", "do", "done",
                "case", "esac", "function", "return", "exit", "echo", "export",
                "local", "readonly", "set", "unset", "source", "true", "false",
            ],
            "toml" | "yaml" | "yml" | "ini" | "conf" | "cfg" => vec![
                "true", "false", "null", "yes", "no",
            ],
            "java" => vec![
                "public", "private", "protected", "class", "interface", "extends",
                "implements", "static", "final", "void", "int", "boolean", "String",
                "return", "if", "else", "for", "while", "do", "switch", "case",
                "break", "continue", "new", "this", "super", "import", "package",
                "try", "catch", "finally", "throw", "throws", "true", "false", "null",
            ],
            "rb" => vec![
                "def", "class", "module", "end", "if", "elsif", "else", "unless",
                "while", "until", "for", "do", "begin", "rescue", "ensure", "raise",
                "return", "yield", "require", "include", "attr_accessor", "nil",
                "true", "false", "self", "super",
            ],
            "html" | "xml" => vec![
                "html", "head", "body", "div", "span", "class", "id", "style",
                "script", "link", "meta", "title", "src", "href", "type",
            ],
            "css" => vec![
                "color", "background", "border", "margin", "padding", "display",
                "position", "width", "height", "font", "text", "align", "flex",
                "grid", "important", "none", "auto",
            ],
            _ => vec!["true", "false", "null", "nil", "none"],
        }
    }

    /// Render a single line with basic syntax coloring.
    fn render_syntax_line(
        ui: &mut egui::Ui,
        line: &str,
        keywords: &[&str],
        t: crate::theme::CyberTheme,
    ) {
        use eframe::egui::RichText;

        let trimmed = line.trim_start();

        // Comment detection (single-line)
        if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("--") {
            ui.label(
                RichText::new(line)
                    .color(t.text_dim())
                    .monospace()
                    .size(9.5),
            );
            return;
        }

        // String detection — whole line starts inside a string
        if trimmed.starts_with('"') || trimmed.starts_with('\'') || trimmed.starts_with('`') {
            ui.label(
                RichText::new(line)
                    .color(t.success())
                    .monospace()
                    .size(9.5),
            );
            return;
        }

        // Keyword coloring: check if any word in the line is a keyword
        let mut has_keyword = false;
        for word in line.split(|c: char| !c.is_alphanumeric() && c != '_') {
            if keywords.contains(&word) {
                has_keyword = true;
                break;
            }
        }

        if has_keyword {
            // Build a colored layout using LayoutJob for mixed colors
            let mut job = egui::text::LayoutJob::default();
            let mut last_end = 0;

            let chars: Vec<char> = line.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                // Skip non-word chars
                if !chars[i].is_alphanumeric() && chars[i] != '_' {
                    i += 1;
                    continue;
                }

                let word_start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[word_start..i].iter().collect();

                // Append everything before this word as normal text
                if word_start > last_end {
                    let prefix: String = chars[last_end..word_start].iter().collect();
                    job.append(
                        &prefix,
                        0.0,
                        egui::TextFormat {
                            font_id: egui::FontId::monospace(9.5),
                            color: t.text_primary(),
                            ..Default::default()
                        },
                    );
                }

                // Color the word
                let color = if keywords.contains(&word.as_str()) {
                    t.primary()
                } else if word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    t.warning()
                } else if word.chars().all(|c| c.is_ascii_digit()) {
                    t.accent()
                } else {
                    t.text_primary()
                };

                job.append(
                    &word,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::monospace(9.5),
                        color,
                        ..Default::default()
                    },
                );
                last_end = i;
            }

            // Any trailing content
            if last_end < chars.len() {
                let trailing: String = chars[last_end..].iter().collect();
                job.append(
                    &trailing,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::monospace(9.5),
                        color: t.text_primary(),
                        ..Default::default()
                    },
                );
            }

            ui.label(job);
        } else {
            // Plain line — check for numbers
            ui.label(
                RichText::new(line)
                    .color(t.text_primary())
                    .monospace()
                    .size(9.5),
            );
        }
    }
}
