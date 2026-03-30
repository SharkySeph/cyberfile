use std::path::PathBuf;

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
