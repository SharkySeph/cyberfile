mod app;
mod config;
mod filesystem;
mod integrations;
mod theme;
mod ui;

use app::CyberFile;

fn load_icon() -> Option<eframe::egui::IconData> {
    let icon_bytes = include_bytes!("../assets/icon-256.png");
    let img = image::load_from_memory(icon_bytes).ok()?.into_rgba8();
    let (w, h) = img.dimensions();
    Some(eframe::egui::IconData {
        rgba: img.into_raw(),
        width: w,
        height: h,
    })
}

fn main() -> eframe::Result {
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_inner_size([1280.0, 800.0])
        .with_min_inner_size([800.0, 500.0])
        .with_title("CYBERFILE // OPERATOR TERMINAL");

    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "CYBERFILE",
        native_options,
        Box::new(|cc| Ok(Box::new(CyberFile::new(cc)))),
    )
}
