use std::process::Command;

// ── WM Detection ───────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WmBackend {
    Hyprland,
    Sway,
    I3,
    KWin,
    X11Ewmh,
}

impl WmBackend {
    pub fn label(self) -> &'static str {
        match self {
            Self::Hyprland => "Hyprland",
            Self::Sway => "Sway",
            Self::I3 => "i3",
            Self::KWin => "KWin",
            Self::X11Ewmh => "X11",
        }
    }

    fn command(self) -> &'static str {
        match self {
            Self::Hyprland => "hyprctl",
            Self::Sway => "swaymsg",
            Self::I3 => "i3-msg",
            Self::KWin => "qdbus",
            Self::X11Ewmh => "xprop",
        }
    }
}

/// Detect which window manager is running.
/// Priority: Hyprland > Sway > i3 > KDE/KWin > generic X11 EWMH.
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
    // KDE/KWin — check for KDE_SESSION_VERSION or KDE desktop and qdbus reachability
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_uppercase();
    if desktop.contains("KDE") || std::env::var("KDE_SESSION_VERSION").is_ok() {
        if cmd_ok("qdbus", &["org.kde.KWin", "/KWin", "org.kde.KWin.currentDesktop"]) {
            return Some(WmBackend::KWin);
        }
    }
    // Generic X11 EWMH — any X11 session with xprop available
    if std::env::var("DISPLAY").is_ok() && cmd_ok("xprop", &["-root", "-len", "0", "_NET_SUPPORTED"]) {
        return Some(WmBackend::X11Ewmh);
    }
    None
}

/// Quick check whether a command succeeds (exit 0).
fn cmd_ok(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
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
        WmBackend::KWin | WmBackend::X11Ewmh => list_windows_x11(),
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
        WmBackend::KWin => list_workspaces_kwin(),
        WmBackend::X11Ewmh => list_workspaces_x11(),
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
        WmBackend::KWin => focus_window_kwin(window_id),
        WmBackend::X11Ewmh => focus_window_x11(window_id),
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
        WmBackend::KWin => move_window_to_workspace_kwin(window_id, workspace),
        WmBackend::X11Ewmh => move_window_to_workspace_x11(window_id, workspace),
    }
}

pub fn switch_workspace(backend: WmBackend, workspace: &str) -> Result<(), String> {
    match backend {
        WmBackend::Hyprland => {
            run_cmd("hyprctl", &["dispatch", "workspace", workspace])
        }
        WmBackend::Sway => run_cmd("swaymsg", &[&format!("workspace {}", workspace)]),
        WmBackend::I3 => run_cmd("i3-msg", &[&format!("workspace {}", workspace)]),
        WmBackend::KWin => switch_workspace_kwin(workspace),
        WmBackend::X11Ewmh => switch_workspace_x11(workspace),
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
        WmBackend::KWin => close_window_kwin(window_id),
        WmBackend::X11Ewmh => close_window_x11(window_id),
    }
}

// ── X11 / EWMH Implementation ──────────────────────────────
// Works on any EWMH-compliant X11 WM: KDE, GNOME, XFCE, MATE,
// Cinnamon, LXQt, Openbox, Fluxbox, etc.

fn list_windows_x11() -> Result<Vec<WmWindow>, String> {
    // Get _NET_CLIENT_LIST from root window
    let output = Command::new("xprop")
        .args(["-root", "_NET_CLIENT_LIST"])
        .output()
        .map_err(|e| format!("xprop failed: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let wids = parse_xprop_window_list(&stdout);
    if wids.is_empty() {
        return Ok(Vec::new());
    }

    // Get the active/focused window
    let active_output = Command::new("xprop")
        .args(["-root", "_NET_ACTIVE_WINDOW"])
        .output()
        .ok();
    let active_wid = active_output
        .as_ref()
        .map(|o| {
            let s = String::from_utf8_lossy(&o.stdout);
            parse_xprop_single_window(&s)
        })
        .unwrap_or_default();

    let mut windows = Vec::new();
    for wid in &wids {
        // Query per-window properties in one xprop call
        let output = Command::new("xprop")
            .args(["-id", wid, "_NET_WM_NAME", "WM_CLASS", "_NET_WM_DESKTOP"])
            .output();
        let Ok(output) = output else { continue };
        let props = String::from_utf8_lossy(&output.stdout);

        let title = parse_xprop_string(&props, "_NET_WM_NAME");
        let class = parse_xprop_wm_class(&props);
        let desktop_str = parse_xprop_cardinal(&props, "_NET_WM_DESKTOP");

        // Skip desktop shell windows (desktop 0xFFFFFFFF / 4294967295)
        if desktop_str == "4294967295" {
            continue;
        }
        // Skip windows with no title and no class
        if title.is_empty() && class.is_empty() {
            continue;
        }

        let workspace = if desktop_str.is_empty() {
            String::new()
        } else {
            // Convert 0-based index to display name
            let idx: usize = desktop_str.parse().unwrap_or(0);
            format!("{}", idx + 1)
        };

        windows.push(WmWindow {
            id: wid.clone(),
            title,
            class,
            workspace,
            focused: *wid == active_wid,
        });
    }
    Ok(windows)
}

fn list_workspaces_kwin() -> Result<Vec<WmWorkspace>, String> {
    // Use KWin VirtualDesktopManager D-Bus for rich info
    let output = Command::new("qdbus")
        .args(["--literal", "org.kde.KWin", "/VirtualDesktopManager",
               "org.kde.KWin.VirtualDesktopManager.desktops"])
        .output()
        .map_err(|e| format!("qdbus failed: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Get current desktop UUID
    let current_output = Command::new("qdbus")
        .args(["org.kde.KWin", "/VirtualDesktopManager",
               "org.kde.KWin.VirtualDesktopManager.current"])
        .output()
        .ok();
    let current_id = current_output
        .as_ref()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    // Count windows per desktop via xprop
    let win_desktop_map = count_windows_per_desktop_x11();

    // Parse qdbus --literal output: [Variant: [Argument: a(uss) {[Argument: (uss) 0, "uuid", "name"], ...}]]
    let mut workspaces = Vec::new();
    // Extract each (uss) tuple: index, uuid, name
    let mut search = stdout.as_ref();
    let mut ws_index: usize = 0;
    while let Some(pos) = search.find("(uss)") {
        let after = &search[pos + 5..];
        // Format: " 0, \"uuid\", \"name\""
        let after = after.trim_start();
        // Parse numeric index
        let comma1 = after.find(',').unwrap_or(after.len());
        let _idx_str = after[..comma1].trim();
        let after = &after[comma1.saturating_add(1)..];
        // Parse UUID
        let uuid = extract_quoted(after);
        let after = &after[after.find(',').unwrap_or(0).saturating_add(1)..];
        // Parse name
        let name = extract_quoted(after);

        let wc = win_desktop_map.get(&ws_index).copied().unwrap_or(0);
        workspaces.push(WmWorkspace {
            id: uuid.clone(),
            name: if name.is_empty() { format!("Desktop {}", ws_index + 1) } else { name },
            focused: uuid == current_id,
            window_count: wc,
        });
        ws_index += 1;
        // Advance past this entry
        let advance = after.find(']').unwrap_or(0).saturating_add(1);
        search = &after[advance..];
    }

    // Fallback: if qdbus parsing returned nothing, try X11 EWMH
    if workspaces.is_empty() {
        return list_workspaces_x11();
    }
    Ok(workspaces)
}

fn list_workspaces_x11() -> Result<Vec<WmWorkspace>, String> {
    let output = Command::new("xprop")
        .args(["-root", "_NET_NUMBER_OF_DESKTOPS", "_NET_CURRENT_DESKTOP", "_NET_DESKTOP_NAMES"])
        .output()
        .map_err(|e| format!("xprop failed: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let num_desktops: usize = parse_xprop_cardinal(&stdout, "_NET_NUMBER_OF_DESKTOPS")
        .parse()
        .unwrap_or(1);
    let current_desktop: usize = parse_xprop_cardinal(&stdout, "_NET_CURRENT_DESKTOP")
        .parse()
        .unwrap_or(0);
    let desktop_names = parse_xprop_string_list(&stdout, "_NET_DESKTOP_NAMES");
    let win_desktop_map = count_windows_per_desktop_x11();

    let mut workspaces = Vec::new();
    for i in 0..num_desktops {
        let name = desktop_names.get(i).cloned()
            .unwrap_or_else(|| format!("Desktop {}", i + 1));
        let wc = win_desktop_map.get(&i).copied().unwrap_or(0);
        workspaces.push(WmWorkspace {
            id: format!("{}", i),
            name,
            focused: i == current_desktop,
            window_count: wc,
        });
    }
    Ok(workspaces)
}

/// Count windows per 0-based desktop index via _NET_WM_DESKTOP.
fn count_windows_per_desktop_x11() -> std::collections::HashMap<usize, usize> {
    let mut map = std::collections::HashMap::new();
    let output = Command::new("xprop")
        .args(["-root", "_NET_CLIENT_LIST"])
        .output()
        .ok();
    let Some(output) = output else { return map };
    let stdout = String::from_utf8_lossy(&output.stdout);
    for wid in parse_xprop_window_list(&stdout) {
        let output = Command::new("xprop")
            .args(["-id", &wid, "_NET_WM_DESKTOP"])
            .output()
            .ok();
        let Some(output) = output else { continue };
        let s = String::from_utf8_lossy(&output.stdout);
        let ds = parse_xprop_cardinal(&s, "_NET_WM_DESKTOP");
        if ds == "4294967295" { continue; }
        if let Ok(idx) = ds.parse::<usize>() {
            *map.entry(idx).or_insert(0) += 1;
        }
    }
    map
}

// ── X11/KWin Actions ───────────────────────────────────────

fn focus_window_x11(window_id: &str) -> Result<(), String> {
    // Use xdotool if available, otherwise wmctrl, otherwise raw X ClientMessage via xprop
    if cmd_ok("xdotool", &["--version"]) {
        return run_cmd("xdotool", &["windowactivate", "--sync", window_id]);
    }
    if cmd_ok("wmctrl", &["-l"]) {
        return run_cmd("wmctrl", &["-i", "-a", window_id]);
    }
    // Fallback: send _NET_ACTIVE_WINDOW client message via xdotool-less approach
    // Use xprop to set _NET_ACTIVE_WINDOW (limited but works on many WMs)
    Err("Install xdotool or wmctrl for window focus support".into())
}

fn close_window_x11(window_id: &str) -> Result<(), String> {
    if cmd_ok("xdotool", &["--version"]) {
        return run_cmd("xdotool", &["windowclose", window_id]);
    }
    if cmd_ok("wmctrl", &["-l"]) {
        return run_cmd("wmctrl", &["-i", "-c", window_id]);
    }
    Err("Install xdotool or wmctrl for window close support".into())
}

// ── KWin-native D-Bus Actions ──────────────────────────────
// Use KWin scripting via D-Bus so we don't need xdotool/wmctrl.

fn focus_window_kwin(window_id: &str) -> Result<(), String> {
    // Try xdotool/wmctrl first (faster), then KWin D-Bus scripting
    if cmd_ok("xdotool", &["--version"]) {
        return run_cmd("xdotool", &["windowactivate", "--sync", window_id]);
    }
    if cmd_ok("wmctrl", &["-l"]) {
        return run_cmd("wmctrl", &["-i", "-a", window_id]);
    }
    // KWin scripting fallback: activate window by X11 window ID
    let wid = parse_hex_window_id(window_id);
    let script = format!(
        "var clients = workspace.clientList();\n\
         for (var i = 0; i < clients.length; i++) {{\n\
             if (clients[i].windowId === {}) {{\n\
                 workspace.activeClient = clients[i];\n\
                 break;\n\
             }}\n\
         }}",
        wid,
    );
    run_kwin_script(&script)
}

fn close_window_kwin(window_id: &str) -> Result<(), String> {
    if cmd_ok("xdotool", &["--version"]) {
        return run_cmd("xdotool", &["windowclose", window_id]);
    }
    if cmd_ok("wmctrl", &["-l"]) {
        return run_cmd("wmctrl", &["-i", "-c", window_id]);
    }
    // KWin scripting fallback: close window by X11 window ID
    let wid = parse_hex_window_id(window_id);
    let script = format!(
        "var clients = workspace.clientList();\n\
         for (var i = 0; i < clients.length; i++) {{\n\
             if (clients[i].windowId === {}) {{\n\
                 clients[i].closeWindow();\n\
                 break;\n\
             }}\n\
         }}",
        wid,
    );
    run_kwin_script(&script)
}

/// Convert hex window ID (e.g. "0x2200011") to a decimal number for KWin scripting.
fn parse_hex_window_id(wid: &str) -> u64 {
    let stripped = wid.trim().trim_start_matches("0x").trim_start_matches("0X");
    u64::from_str_radix(stripped, 16).unwrap_or(0)
}

fn switch_workspace_kwin(workspace: &str) -> Result<(), String> {
    // KWin uses 1-based desktop index for setCurrentDesktop
    // workspace id is 0-based from our listing; convert
    let idx: i32 = workspace.parse::<i32>().unwrap_or(0) + 1;
    run_cmd("qdbus", &[
        "org.kde.KWin", "/KWin",
        "org.kde.KWin.setCurrentDesktop",
        &idx.to_string(),
    ])
}

fn switch_workspace_x11(workspace: &str) -> Result<(), String> {
    if cmd_ok("xdotool", &["--version"]) {
        return run_cmd("xdotool", &["set_desktop", workspace]);
    }
    if cmd_ok("wmctrl", &["-l"]) {
        return run_cmd("wmctrl", &["-s", workspace]);
    }
    Err("Install xdotool or wmctrl for workspace switching".into())
}

fn move_window_to_workspace_kwin(window_id: &str, workspace: &str) -> Result<(), String> {
    // KWin: use xdotool/wmctrl for move + qdbus for desktop number
    let desktop: i32 = workspace.parse::<i32>().unwrap_or(0) + 1;
    if cmd_ok("wmctrl", &["-l"]) {
        return run_cmd("wmctrl", &["-i", "-r", window_id, "-t", &workspace]);
    }
    if cmd_ok("xdotool", &["--version"]) {
        return run_cmd("xdotool", &["set_desktop_for_window", window_id, &workspace]);
    }
    // Fallback via KWin scripting
    let uuid = window_id.trim_start_matches('{').trim_end_matches('}');
    let script = format!(
        "var clients = workspace.windowList();\nfor (var i = 0; i < clients.length; i++) {{\n    if (clients[i].internalId.toString() === \"{{{}}}\") {{\n        clients[i].desktop = {};\n    }}\n}}",
        uuid,
        desktop,
    );
    run_kwin_script(&script)
}

fn move_window_to_workspace_x11(window_id: &str, workspace: &str) -> Result<(), String> {
    if cmd_ok("xdotool", &["--version"]) {
        return run_cmd("xdotool", &["set_desktop_for_window", window_id, workspace]);
    }
    if cmd_ok("wmctrl", &["-l"]) {
        return run_cmd("wmctrl", &["-i", "-r", window_id, "-t", workspace]);
    }
    Err("Install xdotool or wmctrl for move-to-workspace support".into())
}

/// Execute a temporary KWin script via D-Bus scripting.
fn run_kwin_script(script: &str) -> Result<(), String> {
    // Write script to a temp file
    let tmp = "/tmp/cyberfile_kwin_script.js";
    std::fs::write(tmp, script).map_err(|e| format!("Write script: {}", e))?;
    // Load the script — returns a numeric script ID
    let output = Command::new("qdbus")
        .args(["org.kde.KWin", "/Scripting",
               "org.kde.kwin.Scripting.loadScript", tmp, "cyberfile_action"])
        .output()
        .map_err(|e| format!("qdbus loadScript: {}", e))?;
    let script_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !output.status.success() || script_id.is_empty() {
        let _ = std::fs::remove_file(tmp);
        return Err("Failed to load KWin script".into());
    }
    // Run the script on its specific D-Bus object path
    let script_path = format!("/Scripting/Script{}", script_id);
    let _ = Command::new("qdbus")
        .args(["org.kde.KWin", &script_path, "run"])
        .output();
    // Brief wait for script to execute
    std::thread::sleep(std::time::Duration::from_millis(100));
    // Unload the script
    let _ = Command::new("qdbus")
        .args(["org.kde.KWin", &script_path, "stop"])
        .output();
    let _ = Command::new("qdbus")
        .args(["org.kde.KWin", "/Scripting",
               "org.kde.kwin.Scripting.unloadScript", &script_id])
        .output();
    let _ = std::fs::remove_file(tmp);
    Ok(())
}

// ── xprop Parsing Helpers ──────────────────────────────────

/// Parse `_NET_CLIENT_LIST(WINDOW): window id # 0x..., 0x...` into hex window IDs.
fn parse_xprop_window_list(output: &str) -> Vec<String> {
    for line in output.lines() {
        if line.contains("_NET_CLIENT_LIST") && line.contains('#') {
            let after = line.split('#').nth(1).unwrap_or("");
            return after
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| s.starts_with("0x"))
                .collect();
        }
    }
    Vec::new()
}

/// Parse `_NET_ACTIVE_WINDOW(WINDOW): window id # 0x...` into a single ID.
fn parse_xprop_single_window(output: &str) -> String {
    for line in output.lines() {
        if line.contains('#') {
            if let Some(id) = line.split('#').nth(1) {
                let id = id.trim().split(',').next().unwrap_or("").trim();
                if id.starts_with("0x") {
                    return id.to_string();
                }
            }
        }
    }
    String::new()
}

/// Parse `KEY(UTF8_STRING) = "value"` from xprop output.
fn parse_xprop_string(output: &str, key: &str) -> String {
    for line in output.lines() {
        if line.starts_with(key) && line.contains('=') {
            if let Some(val) = line.split('=').nth(1) {
                let val = val.trim();
                if val.starts_with('"') {
                    return val.trim_matches('"').to_string();
                }
            }
        }
    }
    String::new()
}

/// Parse `WM_CLASS(STRING) = "instance", "class"` — return the class part.
fn parse_xprop_wm_class(output: &str) -> String {
    for line in output.lines() {
        if line.starts_with("WM_CLASS") && line.contains('=') {
            if let Some(val) = line.split('=').nth(1) {
                // Second quoted string is the class name
                let parts: Vec<&str> = val.split('"').collect();
                if parts.len() >= 4 {
                    return parts[3].to_string();
                } else if parts.len() >= 2 {
                    return parts[1].to_string();
                }
            }
        }
    }
    String::new()
}

/// Parse `KEY(CARDINAL) = 0` from xprop output.
fn parse_xprop_cardinal(output: &str, key: &str) -> String {
    for line in output.lines() {
        if line.starts_with(key) && line.contains('=') {
            if let Some(val) = line.split('=').nth(1) {
                return val.trim().to_string();
            }
        }
    }
    String::new()
}

/// Parse `KEY(UTF8_STRING) = "a", "b", "c"` from xprop output.
fn parse_xprop_string_list(output: &str, key: &str) -> Vec<String> {
    for line in output.lines() {
        if line.starts_with(key) && line.contains('=') {
            if let Some(val) = line.split('=').nth(1) {
                return val
                    .split('"')
                    .enumerate()
                    .filter(|(i, _)| i % 2 == 1) // odd indices are inside quotes
                    .map(|(_, s)| s.to_string())
                    .collect();
            }
        }
    }
    Vec::new()
}

/// Extract first quoted string from text: ... "value" ...
fn extract_quoted(s: &str) -> String {
    let Some(start) = s.find('"') else { return String::new() };
    let rest = &s[start + 1..];
    let Some(end) = rest.find('"') else { return String::new() };
    rest[..end].to_string()
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
