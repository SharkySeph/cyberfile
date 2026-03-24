use eframe::egui::{self, Color32, CornerRadius, FontFamily, Stroke, Visuals};
use serde::{Deserialize, Serialize};

// ── CyberTheme Engine ──────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CyberTheme {
    NightCity,
    Section9,
    MagiSystem,
    Gibson,
}

impl Default for CyberTheme {
    fn default() -> Self {
        Self::NightCity
    }
}

impl CyberTheme {
    pub fn all() -> &'static [CyberTheme] {
        &[Self::NightCity, Self::Section9, Self::MagiSystem, Self::Gibson]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::NightCity => "NIGHT CITY",
            Self::Section9 => "SECTION 9",
            Self::MagiSystem => "MAGI SYSTEM",
            Self::Gibson => "GIBSON",
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            Self::NightCity => "night_city",
            Self::Section9 => "section9",
            Self::MagiSystem => "magi",
            Self::Gibson => "gibson",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::NightCity => "Neon cyan & magenta // Arasaka networks",
            Self::Section9 => "Teal & violet // Public Security",
            Self::MagiSystem => "Orange & crimson // NERV terminal",
            Self::Gibson => "Amber phosphor // ICE breaker",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id {
            "section9" => Self::Section9,
            "magi" => Self::MagiSystem,
            "gibson" => Self::Gibson,
            _ => Self::NightCity,
        }
    }

    pub fn primary(&self) -> Color32 {
        match self {
            Self::NightCity => Color32::from_rgb(0x00, 0xF0, 0xFF),
            Self::Section9 => Color32::from_rgb(0x00, 0xD4, 0xAA),
            Self::MagiSystem => Color32::from_rgb(0xFF, 0x6B, 0x00),
            Self::Gibson => Color32::from_rgb(0xFF, 0xB0, 0x00),
        }
    }

    pub fn primary_dim(&self) -> Color32 {
        match self {
            Self::NightCity => Color32::from_rgb(0x00, 0x80, 0x99),
            Self::Section9 => Color32::from_rgb(0x00, 0x66, 0x55),
            Self::MagiSystem => Color32::from_rgb(0x99, 0x40, 0x00),
            Self::Gibson => Color32::from_rgb(0x88, 0x66, 0x00),
        }
    }

    pub fn accent(&self) -> Color32 {
        match self {
            Self::NightCity => Color32::from_rgb(0xFF, 0x20, 0x79),
            Self::Section9 => Color32::from_rgb(0x9B, 0x59, 0xB6),
            Self::MagiSystem => Color32::from_rgb(0xDC, 0x14, 0x3C),
            Self::Gibson => Color32::from_rgb(0x00, 0xFF, 0x41),
        }
    }

    pub fn warning(&self) -> Color32 {
        match self {
            Self::NightCity => Color32::from_rgb(0xF7, 0xF3, 0x2A),
            Self::Section9 => Color32::from_rgb(0x5D, 0xAD, 0xE2),
            Self::MagiSystem => Color32::from_rgb(0xFF, 0xAA, 0x00),
            Self::Gibson => Color32::from_rgb(0xFF, 0xD7, 0x00),
        }
    }

    pub fn bg_dark(&self) -> Color32 {
        match self {
            Self::NightCity => Color32::from_rgb(0x0A, 0x0A, 0x0F),
            Self::Section9 => Color32::from_rgb(0x08, 0x0A, 0x0C),
            Self::MagiSystem => Color32::from_rgb(0x0C, 0x08, 0x04),
            Self::Gibson => Color32::from_rgb(0x00, 0x00, 0x00),
        }
    }

    pub fn surface(&self) -> Color32 {
        match self {
            Self::NightCity => Color32::from_rgb(0x0D, 0x11, 0x17),
            Self::Section9 => Color32::from_rgb(0x0E, 0x10, 0x12),
            Self::MagiSystem => Color32::from_rgb(0x12, 0x0E, 0x0A),
            Self::Gibson => Color32::from_rgb(0x04, 0x04, 0x00),
        }
    }

    pub fn surface_raised(&self) -> Color32 {
        match self {
            Self::NightCity => Color32::from_rgb(0x12, 0x18, 0x22),
            Self::Section9 => Color32::from_rgb(0x15, 0x18, 0x19),
            Self::MagiSystem => Color32::from_rgb(0x1A, 0x14, 0x10),
            Self::Gibson => Color32::from_rgb(0x0A, 0x0A, 0x00),
        }
    }

    pub fn danger(&self) -> Color32 {
        match self {
            Self::NightCity => Color32::from_rgb(0xFF, 0x33, 0x33),
            Self::Section9 => Color32::from_rgb(0xE7, 0x4C, 0x3C),
            Self::MagiSystem => Color32::from_rgb(0xFF, 0x00, 0x00),
            Self::Gibson => Color32::from_rgb(0xFF, 0x00, 0x00),
        }
    }

    pub fn success(&self) -> Color32 {
        match self {
            Self::NightCity => Color32::from_rgb(0x39, 0xFF, 0x14),
            Self::Section9 => Color32::from_rgb(0x1A, 0xBC, 0x9C),
            Self::MagiSystem => Color32::from_rgb(0xAA, 0xFF, 0x00),
            Self::Gibson => Color32::from_rgb(0x00, 0xFF, 0x41),
        }
    }

    pub fn text_primary(&self) -> Color32 {
        match self {
            Self::NightCity => Color32::from_rgb(0xE0, 0xE0, 0xE8),
            Self::Section9 => Color32::from_rgb(0xC8, 0xD0, 0xD8),
            Self::MagiSystem => Color32::from_rgb(0xE0, 0xD8, 0xCC),
            Self::Gibson => Color32::from_rgb(0xFF, 0xB0, 0x00),
        }
    }

    pub fn text_dim(&self) -> Color32 {
        match self {
            Self::NightCity => Color32::from_rgb(0x4A, 0x7A, 0x7F),
            Self::Section9 => Color32::from_rgb(0x4A, 0x55, 0x68),
            Self::MagiSystem => Color32::from_rgb(0x7A, 0x6A, 0x5A),
            Self::Gibson => Color32::from_rgb(0x66, 0x55, 0x00),
        }
    }

    pub fn border_dim(&self) -> Color32 {
        match self {
            Self::NightCity => Color32::from_rgb(0x1A, 0x2A, 0x33),
            Self::Section9 => Color32::from_rgb(0x1A, 0x1E, 0x24),
            Self::MagiSystem => Color32::from_rgb(0x2A, 0x20, 0x18),
            Self::Gibson => Color32::from_rgb(0x1A, 0x14, 0x00),
        }
    }

    pub fn border_active(&self) -> Color32 {
        match self {
            Self::NightCity => Color32::from_rgb(0x00, 0x60, 0x66),
            Self::Section9 => Color32::from_rgb(0x00, 0x88, 0x6A),
            Self::MagiSystem => Color32::from_rgb(0x88, 0x44, 0x00),
            Self::Gibson => Color32::from_rgb(0x88, 0x66, 0x00),
        }
    }

    pub fn selection_bg(&self) -> Color32 {
        match self {
            Self::NightCity => Color32::from_rgba_premultiplied(0xFF, 0x20, 0x79, 0x25),
            Self::Section9 => Color32::from_rgba_premultiplied(0x9B, 0x59, 0xB6, 0x25),
            Self::MagiSystem => Color32::from_rgba_premultiplied(0xFF, 0x6B, 0x00, 0x25),
            Self::Gibson => Color32::from_rgba_premultiplied(0x00, 0xFF, 0x41, 0x25),
        }
    }
}

// ── Legacy Color Constants (Night City defaults) ───────────────
pub const CYAN: Color32 = Color32::from_rgb(0x00, 0xF0, 0xFF);
pub const CYAN_DIM: Color32 = Color32::from_rgb(0x00, 0x80, 0x99);
pub const MAGENTA: Color32 = Color32::from_rgb(0xFF, 0x20, 0x79);
pub const YELLOW: Color32 = Color32::from_rgb(0xF7, 0xF3, 0x2A);
pub const BG_DARK: Color32 = Color32::from_rgb(0x0A, 0x0A, 0x0F);
pub const SURFACE: Color32 = Color32::from_rgb(0x0D, 0x11, 0x17);
#[allow(dead_code)]
pub const SURFACE_RAISED: Color32 = Color32::from_rgb(0x12, 0x18, 0x22);
pub const DANGER: Color32 = Color32::from_rgb(0xFF, 0x33, 0x33);
pub const SUCCESS: Color32 = Color32::from_rgb(0x39, 0xFF, 0x14);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xE0, 0xE0, 0xE8);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x4A, 0x7A, 0x7F);
pub const BORDER_DIM: Color32 = Color32::from_rgb(0x1A, 0x2A, 0x33);
#[allow(dead_code)]
pub const BORDER_ACTIVE: Color32 = Color32::from_rgb(0x00, 0x60, 0x66);

// ── Apply Theme to egui ────────────────────────────────────────

pub fn apply_cyber_theme(ctx: &egui::Context, theme: CyberTheme) {
    let mut visuals = Visuals::dark();

    visuals.panel_fill = theme.surface();
    visuals.window_fill = theme.bg_dark();
    visuals.extreme_bg_color = theme.bg_dark();
    visuals.faint_bg_color = theme.surface_raised();
    visuals.override_text_color = None;
    visuals.selection.bg_fill = theme.selection_bg();
    visuals.selection.stroke = Stroke::new(1.0, theme.accent());
    visuals.hyperlink_color = theme.primary();

    // Non-interactive
    visuals.widgets.noninteractive.bg_fill = theme.surface();
    visuals.widgets.noninteractive.weak_bg_fill = theme.surface();
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, theme.text_primary());
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(0.5, theme.border_dim());
    visuals.widgets.noninteractive.corner_radius = CornerRadius::ZERO;

    // Inactive
    let r = theme.surface().r().saturating_add(6);
    let g = theme.surface().g().saturating_add(5);
    let b = theme.surface().b().saturating_add(7);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(r, g, b);
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(r, g, b);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, theme.primary());
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, theme.border_active());
    visuals.widgets.inactive.corner_radius = CornerRadius::ZERO;

    // Hovered
    visuals.widgets.hovered.bg_fill = Color32::from_rgba_premultiplied(
        theme.primary().r() / 6,
        theme.primary().g() / 6,
        theme.primary().b() / 6,
        180,
    );
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgba_premultiplied(
        theme.primary().r() / 6,
        theme.primary().g() / 6,
        theme.primary().b() / 6,
        180,
    );
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, theme.primary());
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, theme.primary());
    visuals.widgets.hovered.corner_radius = CornerRadius::ZERO;

    // Active (selected selectables, pressed buttons)
    visuals.widgets.active.bg_fill = Color32::from_rgba_premultiplied(
        theme.primary().r() / 8,
        theme.primary().g() / 8,
        theme.primary().b() / 8,
        200,
    );
    visuals.widgets.active.weak_bg_fill = Color32::from_rgba_premultiplied(
        theme.primary().r() / 8,
        theme.primary().g() / 8,
        theme.primary().b() / 8,
        200,
    );
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, theme.text_primary());
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, theme.accent());
    visuals.widgets.active.corner_radius = CornerRadius::ZERO;

    // Open
    visuals.widgets.open.bg_fill = theme.surface_raised();
    visuals.widgets.open.weak_bg_fill = theme.surface_raised();
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, theme.primary());
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, theme.primary());
    visuals.widgets.open.corner_radius = CornerRadius::ZERO;

    // Window style
    visuals.window_corner_radius = CornerRadius::ZERO;
    visuals.menu_corner_radius = CornerRadius::ZERO;
    visuals.window_shadow = egui::epaint::Shadow::NONE;
    visuals.popup_shadow = egui::epaint::Shadow::NONE;
    visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);
    visuals.striped = true;

    ctx.set_visuals(visuals);

    // Typography: all monospace
    let mut style = (*ctx.style()).clone();
    for (_text_style, font_id) in style.text_styles.iter_mut() {
        font_id.family = FontFamily::Monospace;
    }
    style.spacing.item_spacing = egui::vec2(6.0, 3.0);
    style.spacing.button_padding = egui::vec2(8.0, 3.0);
    ctx.set_style(style);
}

// ── Helper Functions ───────────────────────────────────────────

pub fn section_header(ui: &mut egui::Ui, label: &str) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("\u{25C8} {}", label))
                .color(CYAN)
                .strong()
                .size(13.0),
        );
    });
}

pub fn cyber_separator(ui: &mut egui::Ui) {
    let rect = ui.available_rect_before_wrap();
    let y = rect.top() + 2.0;
    ui.painter().line_segment(
        [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
        Stroke::new(0.5, BORDER_DIM),
    );
    ui.add_space(5.0);
}

pub fn cyber_separator_themed(ui: &mut egui::Ui, color: Color32) {
    let rect = ui.available_rect_before_wrap();
    let y = rect.top() + 2.0;
    ui.painter().line_segment(
        [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
        Stroke::new(0.5, color),
    );
    ui.add_space(5.0);
}
