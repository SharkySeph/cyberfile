use std::path::PathBuf;
use std::process::Command;

/// Check if a D-Bus session bus is available
#[allow(dead_code)]
pub fn is_dbus_available() -> bool {
    std::env::var("DBUS_SESSION_BUS_ADDRESS").is_ok()
}

/// Reveal a file in the default file manager via D-Bus FileManager1
/// This allows other apps to ask cyberfile to show a specific path
#[allow(dead_code)]
pub fn reveal_in_file_manager(path: &std::path::Path) -> Result<(), String> {
    let uri = format!("file://{}", path.display());
    Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.freedesktop.FileManager1",
            "--object-path",
            "/org/freedesktop/FileManager1",
            "--method",
            "org.freedesktop.FileManager1.ShowItems",
            &format!("['{}']", uri),
            "",
        ])
        .output()
        .map(|_| ())
        .map_err(|e| format!("D-Bus call failed: {}", e))
}

/// Parse command-line arguments for D-Bus-style invocation.
/// Supports: cyberfile [path] or cyberfile --show-item <path>
pub fn parse_cli_path() -> Option<PathBuf> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        return None;
    }

    // --show-item <path> (FileManager1 style)
    if args.len() >= 3 && args[1] == "--show-item" {
        let path = PathBuf::from(&args[2]);
        if path.exists() {
            return Some(if path.is_file() {
                path.parent().unwrap_or(&path).to_path_buf()
            } else {
                path
            });
        }
    }

    // Plain path argument
    let path = PathBuf::from(&args[1]);
    if path.exists() {
        return Some(if path.is_file() {
            path.parent().unwrap_or(&path).to_path_buf()
        } else {
            path
        });
    }

    None
}
