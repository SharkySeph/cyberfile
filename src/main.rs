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
    let settings = config::Settings::load();

    // Check for CLI path argument (D-Bus FileManager1 style)
    let cli_path = integrations::dbus::parse_cli_path();

    // Store CLI path for the app to pick up
    if let Some(ref path) = cli_path {
        std::env::set_var("CYBERFILE_START_PATH", path.to_string_lossy().as_ref());
    }

    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_inner_size([settings.window_width, settings.window_height])
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
