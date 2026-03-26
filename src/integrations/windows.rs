use std::process::Command;

// ── WM Detection ───────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WmBackend {
    Hyprland,
    Sway,
    I3,
}

impl WmBackend {
    pub fn label(self) -> &'static str {
        match self {
            Self::Hyprland => "Hyprland",
            Self::Sway => "Sway",
            Self::I3 => "i3",
        }
    }

    fn command(self) -> &'static str {
        match self {
            Self::Hyprland => "hyprctl",
            Self::Sway => "swaymsg",
            Self::I3 => "i3-msg",
        }
    }
}

/// Detect which window manager is running (check env vars then fall back to binary probing).
pub fn detect_wm() -> Option<WmBackend> {
    // Hyprland sets HYPRLAND_INSTANCE_SIGNATURE
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return Some(WmBackend::Hyprland);
    }
    // Sway sets SWAYSOCK
    if std::env::var("SWAYSOCK").is_ok() {
        return Some(WmBackend::Sway);
    }
    // i3 sets I3SOCK
    if std::env::var("I3SOCK").is_ok() {
        return Some(WmBackend::I3);
    }
    None
}

// ── Data Structs ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct WmWindow {
    pub id: String,
    pub title: String,
    pub class: String,
    pub workspace: String,
    pub focused: bool,
}

#[derive(Debug, Clone)]
pub struct WmWorkspace {
    pub id: String,
    pub name: String,
    pub focused: bool,
    pub window_count: usize,
}

// ── Window Listing ─────────────────────────────────────────

pub fn list_windows(backend: WmBackend) -> Result<Vec<WmWindow>, String> {
    match backend {
        WmBackend::Hyprland => list_windows_hyprland(),
        WmBackend::Sway => list_windows_sway_i3("swaymsg"),
        WmBackend::I3 => list_windows_sway_i3("i3-msg"),
    }
}

fn list_windows_hyprland() -> Result<Vec<WmWindow>, String> {
    let output = Command::new("hyprctl")
        .args(["clients", "-j"])
        .output()
        .map_err(|e| format!("hyprctl failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("hyprctl error: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut windows = Vec::new();

    // Minimal JSON array parser for hyprctl clients output
    // Each object has: "address", "title", "class", "workspace": {"id":N, "name":"..."}, "focusHistoryID"
    for obj in split_json_objects(&stdout) {
        let title = extract_json_str(&obj, "title");
        let class = extract_json_str(&obj, "class");
        let address = extract_json_str(&obj, "address");
        let workspace_name = extract_nested_json_str(&obj, "workspace", "name");
        let focus_id = extract_json_str(&obj, "focusHistoryID");
        if title.is_empty() && class.is_empty() {
            continue;
        }
        windows.push(WmWindow {
            id: address,
            title,
            class,
            workspace: workspace_name,
            focused: focus_id == "0",
        });
    }

    Ok(windows)
}

fn list_windows_sway_i3(cmd: &str) -> Result<Vec<WmWindow>, String> {
    let output = Command::new(cmd)
        .args(["-t", "get_tree"])
        .output()
        .map_err(|e| format!("{} failed: {}", cmd, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{} error: {}", cmd, stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut windows = Vec::new();
    collect_sway_i3_windows(&stdout, &mut windows);
    Ok(windows)
}

fn collect_sway_i3_windows(json: &str, windows: &mut Vec<WmWindow>) {
    // Walk the tree looking for nodes with "type":"con" and a non-empty "name"
    // that also have no "nodes" children (leaf containers = windows)
    for obj in split_json_objects(json) {
        let node_type = extract_json_str(&obj, "type");
        let name = extract_json_str(&obj, "name");
        let app_id = extract_json_str(&obj, "app_id");
        let wm_class = extract_json_str(&obj, "window_properties");
        let focused = extract_json_str(&obj, "focused") == "true";
        let id_str = extract_json_str(&obj, "id");
        let workspace = extract_json_str(&obj, "workspace");

        if (node_type == "con" || node_type == "floating_con") && !name.is_empty() {
            let class = if !app_id.is_empty() {
                app_id
            } else {
                wm_class
            };
            windows.push(WmWindow {
                id: id_str,
                title: name,
                class,
                workspace,
                focused,
            });
        }

        // Recurse into "nodes" and "floating_nodes"
        if let Some(pos) = obj.find("\"nodes\"") {
            collect_sway_i3_windows(&obj[pos..], windows);
        }
        if let Some(pos) = obj.find("\"floating_nodes\"") {
            collect_sway_i3_windows(&obj[pos..], windows);
        }
    }
}

// ── Workspace Listing ──────────────────────────────────────

pub fn list_workspaces(backend: WmBackend) -> Result<Vec<WmWorkspace>, String> {
    match backend {
        WmBackend::Hyprland => list_workspaces_hyprland(),
        WmBackend::Sway | WmBackend::I3 => list_workspaces_sway_i3(backend.command()),
    }
}

fn list_workspaces_hyprland() -> Result<Vec<WmWorkspace>, String> {
    let output = Command::new("hyprctl")
        .args(["workspaces", "-j"])
        .output()
        .map_err(|e| format!("hyprctl failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("hyprctl error: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Get active workspace
    let active_ws = get_active_workspace_hyprland().unwrap_or_default();

    let mut workspaces = Vec::new();
    for obj in split_json_objects(&stdout) {
        let id = extract_json_str(&obj, "id");
        let name = extract_json_str(&obj, "name");
        let windows_str = extract_json_str(&obj, "windows");
        let window_count: usize = windows_str.parse().unwrap_or(0);
        workspaces.push(WmWorkspace {
            focused: id == active_ws,
            id,
            name,
            window_count,
        });
    }

    workspaces.sort_by(|a, b| {
        a.id.parse::<i64>()
            .unwrap_or(999)
            .cmp(&b.id.parse::<i64>().unwrap_or(999))
    });
    Ok(workspaces)
}

fn get_active_workspace_hyprland() -> Option<String> {
    let output = Command::new("hyprctl")
        .args(["activeworkspace", "-j"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = extract_json_str(&stdout, "id");
    if id.is_empty() { None } else { Some(id) }
}

fn list_workspaces_sway_i3(cmd: &str) -> Result<Vec<WmWorkspace>, String> {
    let output = Command::new(cmd)
        .args(["-t", "get_workspaces"])
        .output()
        .map_err(|e| format!("{} failed: {}", cmd, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{} error: {}", cmd, stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut workspaces = Vec::new();

    for obj in split_json_objects(&stdout) {
        let id = extract_json_str(&obj, "num");
        let name = extract_json_str(&obj, "name");
        let focused = extract_json_str(&obj, "focused") == "true";
        workspaces.push(WmWorkspace {
            id,
            name,
            focused,
            window_count: 0, // sway/i3 don't directly report this
        });
    }

    Ok(workspaces)
}

// ── Actions ────────────────────────────────────────────────

pub fn focus_window(backend: WmBackend, window_id: &str) -> Result<(), String> {
    match backend {
        WmBackend::Hyprland => {
            run_cmd("hyprctl", &["dispatch", "focuswindow", &format!("address:{}", window_id)])
        }
        WmBackend::Sway => {
            run_cmd("swaymsg", &[&format!("[con_id={}] focus", window_id)])
        }
        WmBackend::I3 => {
            run_cmd("i3-msg", &[&format!("[con_id={}] focus", window_id)])
        }
    }
}

pub fn move_window_to_workspace(
    backend: WmBackend,
    window_id: &str,
    workspace: &str,
) -> Result<(), String> {
    match backend {
        WmBackend::Hyprland => run_cmd(
            "hyprctl",
            &[
                "dispatch",
                "movetoworkspace",
                &format!("{},address:{}", workspace, window_id),
            ],
        ),
        WmBackend::Sway => run_cmd(
            "swaymsg",
            &[&format!("[con_id={}] move to workspace {}", window_id, workspace)],
        ),
        WmBackend::I3 => run_cmd(
            "i3-msg",
            &[&format!("[con_id={}] move to workspace {}", window_id, workspace)],
        ),
    }
}

pub fn switch_workspace(backend: WmBackend, workspace: &str) -> Result<(), String> {
    match backend {
        WmBackend::Hyprland => {
            run_cmd("hyprctl", &["dispatch", "workspace", workspace])
        }
        WmBackend::Sway => run_cmd("swaymsg", &[&format!("workspace {}", workspace)]),
        WmBackend::I3 => run_cmd("i3-msg", &[&format!("workspace {}", workspace)]),
    }
}

pub fn close_window(backend: WmBackend, window_id: &str) -> Result<(), String> {
    match backend {
        WmBackend::Hyprland => {
            run_cmd("hyprctl", &["dispatch", "closewindow", &format!("address:{}", window_id)])
        }
        WmBackend::Sway => {
            run_cmd("swaymsg", &[&format!("[con_id={}] kill", window_id)])
        }
        WmBackend::I3 => {
            run_cmd("i3-msg", &[&format!("[con_id={}] kill", window_id)])
        }
    }
}

// ── Helpers ────────────────────────────────────────────────

fn run_cmd(cmd: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("{} failed: {}", cmd, e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().to_string());
    }
    Ok(())
}

/// Split a JSON array string into individual top-level object strings.
fn split_json_objects(json: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut depth = 0i32;
    let mut start = None;
    let mut in_string = false;
    let mut escape = false;

    for (i, c) in json.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if c == '\\' && in_string {
            escape = true;
            continue;
        }
        if c == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match c {
            '{' => {
                if depth == 0 {
                    start = Some(i);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(s) = start {
                        results.push(json[s..=i].to_string());
                    }
                    start = None;
                }
            }
            _ => {}
        }
    }

    results
}

fn extract_json_str(obj: &str, key: &str) -> String {
    let pattern = format!("\"{}\"", key);
    let Some(pos) = obj.find(&pattern) else {
        return String::new();
    };
    let after = &obj[pos + pattern.len()..];
    let after = after.trim_start();
    let Some(after) = after.strip_prefix(':') else {
        return String::new();
    };
    let after = after.trim_start();

    if after.starts_with("null") || after.starts_with("false") {
        if after.starts_with("false") {
            return "false".to_string();
        }
        return String::new();
    }
    if after.starts_with("true") {
        return "true".to_string();
    }

    if let Some(rest) = after.strip_prefix('"') {
        let mut escaped = false;
        for (i, c) in rest.char_indices() {
            if escaped {
                escaped = false;
                continue;
            }
            if c == '\\' {
                escaped = true;
                continue;
            }
            if c == '"' {
                return rest[..i].to_string();
            }
        }
        String::new()
    } else {
        let end = after
            .find(|c: char| c == ',' || c == '}' || c == ']')
            .unwrap_or(after.len());
        after[..end].trim().to_string()
    }
}

fn extract_nested_json_str(obj: &str, outer_key: &str, inner_key: &str) -> String {
    let pattern = format!("\"{}\"", outer_key);
    let Some(pos) = obj.find(&pattern) else {
        return String::new();
    };
    let after = &obj[pos + pattern.len()..];
    // Find the nested object
    let Some(brace) = after.find('{') else {
        return String::new();
    };
    let nested = &after[brace..];
    // Find matching close brace
    let mut depth = 0;
    let mut end = 0;
    for (i, c) in nested.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = i;
                    break;
                }
            }
            _ => {}
        }
    }
    if end > 0 {
        extract_json_str(&nested[..=end], inner_key)
    } else {
        String::new()
    }
}
