use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Check if fzf is available on the system
pub fn is_available() -> bool {
    Command::new("which")
        .arg("fzf")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Use `fzf --filter` to fuzzy-match files under a directory.
/// Returns up to 50 matching paths sorted by relevance.
pub fn fuzzy_search(dir: &Path, query: &str, max_depth: u32) -> Vec<PathBuf> {
    if query.trim().is_empty() {
        return Vec::new();
    }

    let find_output = Command::new("find")
        .arg(dir)
        .arg("-maxdepth")
        .arg(max_depth.to_string())
        .arg("-not")
        .arg("-path")
        .arg("*/.*")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    let find_bytes = match find_output {
        Ok(o) if o.status.success() => o.stdout,
        _ => return Vec::new(),
    };

    let mut child = match Command::new("fzf")
        .arg("--filter")
        .arg(query)
        .arg("--no-sort")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    if let Some(ref mut stdin) = child.stdin {
        let _ = stdin.write_all(&find_bytes);
    }

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .take(50)
        .map(|l| PathBuf::from(l.trim()))
        .filter(|p| p.exists())
        .collect()
}

/// Launch fzf interactively in an external terminal.
/// Returns the selected path, or None if cancelled.
/// `preferred_terminal` — if non-empty, try this terminal first.
pub fn launch_interactive(dir: &Path, preferred_terminal: &str) -> Option<PathBuf> {
    let tmp = std::env::temp_dir().join("cyberfile_fzf_result");
    let _ = std::fs::remove_file(&tmp); // clean up any stale result

    // Write a safe launcher script to avoid shell injection via path names
    let script_path = std::env::temp_dir().join("cyberfile_fzf_launch.sh");
    let script_content = format!(
        "#!/bin/sh\ncd \"$1\" && find . -maxdepth 5 2>/dev/null | fzf --height=100% --border --header='CYBERFILE // SELECT TARGET' > \"$2\"\n"
    );
    if std::fs::write(&script_path, &script_content).is_err() {
        return None;
    }
    let _ = std::fs::set_permissions(
        &script_path,
        std::fs::Permissions::from_mode(0o700),
    );

    // Try common terminal emulators — preferred first, then auto-detect
    let mut terminals: Vec<(&str, &[&str])> = vec![
        ("kitty", &["--", "sh"]),
        ("alacritty", &["-e", "sh"]),
        ("wezterm", &["start", "--", "sh"]),
        ("foot", &["sh"]),
        ("gnome-terminal", &["--", "sh"]),
        ("konsole", &["-e", "sh"]),
        ("xfce4-terminal", &["-e", "sh"]),
        ("xterm", &["-e", "sh"]),
    ];

    // Move preferred terminal to front if specified
    if !preferred_terminal.is_empty() {
        // Find matching args or default to "-e sh"
        let args: &[&str] = match preferred_terminal {
            "kitty" => &["--", "sh"],
            "wezterm" => &["start", "--", "sh"],
            "foot" => &["sh"],
            "gnome-terminal" => &["--", "sh"],
            _ => &["-e", "sh"],
        };
        terminals.retain(|(t, _)| *t != preferred_terminal);
        terminals.insert(0, (preferred_terminal, args));
    }

    for (term, args) in &terminals {
        let mut cmd = Command::new(term);
        for a in *args {
            cmd.arg(a);
        }
        cmd.arg(script_path.as_os_str());
        cmd.arg(dir.as_os_str());
        cmd.arg(tmp.as_os_str());

        if let Ok(mut child) = cmd.spawn() {
            let _ = child.wait();
            if tmp.exists() {
                if let Ok(result) = std::fs::read_to_string(&tmp) {
                    let _ = std::fs::remove_file(&tmp);
                    let selected = result.trim();
                    if !selected.is_empty() {
                        let path = if selected.starts_with("./") {
                            dir.join(&selected[2..])
                        } else {
                            dir.join(selected)
                        };
                        return Some(path);
                    }
                }
                return None;
            }
        }
    }

    None
}
