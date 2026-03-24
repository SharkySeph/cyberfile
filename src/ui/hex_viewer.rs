use eframe::egui::{self, RichText, Stroke};

use crate::app::CyberFile;

impl CyberFile {
    /// Full hex viewer — "RAW DECODE" mode
    /// Shows file bytes in traditional hex dump format with NERV styling
    pub(crate) fn render_hex_view(&mut self, ui: &mut egui::Ui) {
        let t = self.current_theme;

        let entry = self
            .selected
            .and_then(|idx| self.entries.get(idx).cloned());

        match entry {
            None => {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("[ SELECT CONSTRUCT FOR RAW DECODE ]")
                            .color(t.text_dim())
                            .monospace()
                            .size(14.0),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("No target selected for hex analysis")
                            .color(t.text_dim())
                            .monospace()
                            .size(11.0),
                    );
                });
            }
            Some(entry) if entry.is_dir => {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("[ SECTORS CANNOT BE HEX-DECODED ]")
                            .color(t.warning())
                            .monospace()
                            .size(14.0),
                    );
                });
            }
            Some(entry) => {
                // Header with NERV styling
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("┌─── RAW DECODE ─── HEX ANALYSIS ───┐")
                            .color(t.primary())
                            .monospace()
                            .size(11.0)
                            .strong(),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("│ TARGET: {}", entry.name))
                            .color(t.text_primary())
                            .monospace()
                            .size(10.0),
                    );
                    ui.label(
                        RichText::new(format!("│ SIZE: {}", bytesize::ByteSize(entry.size)))
                            .color(t.warning())
                            .monospace()
                            .size(10.0),
                    );
                });

                ui.add_space(4.0);

                // Column header
                ui.label(
                    RichText::new("OFFSET   │ 00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F │ ASCII")
                        .color(t.primary())
                        .monospace()
                        .size(9.5)
                        .strong(),
                );

                // Separator
                let rect = ui.available_rect_before_wrap();
                ui.painter().line_segment(
                    [
                        egui::pos2(rect.left(), rect.top()),
                        egui::pos2(rect.right(), rect.top()),
                    ],
                    Stroke::new(0.5, t.primary()),
                );
                ui.add_space(2.0);

                // Read file bytes safely (cap read at 8KB, check size first)
                let file_size = std::fs::metadata(&entry.path)
                    .map(|m| m.len())
                    .unwrap_or(0);
                let read_limit: u64 = 8192;
                let read_size = file_size.min(read_limit) as usize;

                let read_result = (|| -> Result<Vec<u8>, std::io::Error> {
                    use std::io::Read;
                    let mut f = std::fs::File::open(&entry.path)?;
                    let mut buf = vec![0u8; read_size];
                    let n = f.read(&mut buf)?;
                    buf.truncate(n);
                    Ok(buf)
                })();

                match read_result {
                    Ok(bytes) => {
                        let display_bytes = &bytes[..];

                        egui::ScrollArea::vertical()
                            .auto_shrink(false)
                            .show(ui, |ui| {
                                for (chunk_idx, chunk) in display_bytes.chunks(16).enumerate() {
                                    let offset = chunk_idx * 16;

                                    // Hex portion with mid-gap
                                    let hex_left: String = chunk
                                        .iter()
                                        .take(8)
                                        .map(|b| format!("{:02X}", b))
                                        .collect::<Vec<_>>()
                                        .join(" ");
                                    let hex_right: String = chunk
                                        .iter()
                                        .skip(8)
                                        .map(|b| format!("{:02X}", b))
                                        .collect::<Vec<_>>()
                                        .join(" ");

                                    // Pad if short
                                    let hex_left_padded = format!("{:<23}", hex_left);
                                    let hex_right_padded = format!("{:<23}", hex_right);

                                    // ASCII portion
                                    let ascii: String = chunk
                                        .iter()
                                        .map(|b| {
                                            let c = *b as char;
                                            if c.is_ascii_graphic() || c == ' ' {
                                                c
                                            } else {
                                                '.'
                                            }
                                        })
                                        .collect();

                                    let line = format!(
                                        "{:06X}   │ {} {} │ {}",
                                        offset, hex_left_padded, hex_right_padded, ascii
                                    );

                                    // Alternate row color for readability
                                    let color = if chunk_idx % 2 == 0 {
                                        t.text_primary()
                                    } else {
                                        t.text_dim()
                                    };

                                    ui.label(
                                        RichText::new(line)
                                            .color(color)
                                            .monospace()
                                            .size(9.5),
                                    );
                                }

                                if bytes.len() > 8192 {
                                    ui.add_space(8.0);
                                    ui.label(
                                        RichText::new(format!(
                                            "│ TRUNCATED — showing 8192 of {} bytes",
                                            bytes.len()
                                        ))
                                        .color(t.warning())
                                        .monospace()
                                        .size(10.0),
                                    );
                                }
                            });
                    }
                    Err(e) => {
                        ui.add_space(20.0);
                        ui.label(
                            RichText::new(format!("│ ACCESS DENIED: {}", e))
                                .color(t.danger())
                                .monospace()
                                .size(11.0),
                        );
                    }
                }
            }
        }
    }
}
