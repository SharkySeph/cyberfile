use std::process::{Command, Stdio};
use std::sync::OnceLock;

// ── Audio Sink/Source Types ────────────────────────────────

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AudioSink {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub volume_percent: u32,
    pub muted: bool,
    pub is_default: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AudioSource {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub volume_percent: u32,
    pub muted: bool,
    pub is_default: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AudioStream {
    pub id: u32,
    pub app_name: String,
    pub volume_percent: u32,
    pub muted: bool,
    pub sink_id: Option<u32>,
}

// ── Snapshot ───────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct AudioSnapshot {
    pub sinks: Vec<AudioSink>,
    pub sources: Vec<AudioSource>,
    pub streams: Vec<AudioStream>,
    pub default_sink_volume: u32,
    pub default_sink_muted: bool,
    pub default_source_muted: bool,
    pub backend: AudioBackend,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum AudioBackend {
    Pipewire,
    Pulseaudio,
    #[default]
    None,
}

// ── Detection ──────────────────────────────────────────────

static AUDIO_BACKEND: OnceLock<AudioBackend> = OnceLock::new();

pub fn detect_audio_backend() -> AudioBackend {
    *AUDIO_BACKEND.get_or_init(detect_audio_backend_impl)
}

fn detect_audio_backend_impl() -> AudioBackend {
    if Command::new("wpctl").arg("--version").output().is_ok() {
        if let Ok(output) = Command::new("wpctl").arg("status").output() {
            if output.status.success() {
                return AudioBackend::Pipewire;
            }
        }
    }
    if Command::new("pactl").arg("--version").output().is_ok() {
        return AudioBackend::Pulseaudio;
    }
    AudioBackend::None
}

// ── Snapshot Collection ────────────────────────────────────

pub fn collect_audio_snapshot() -> AudioSnapshot {
    let backend = detect_audio_backend();
    match backend {
        AudioBackend::Pipewire => collect_pipewire_snapshot(),
        AudioBackend::Pulseaudio => collect_pulse_snapshot(),
        AudioBackend::None => AudioSnapshot::default(),
    }
}

fn collect_pipewire_snapshot() -> AudioSnapshot {
    let mut snap = AudioSnapshot {
        backend: AudioBackend::Pipewire,
        ..Default::default()
    };

    // Default sink volume + mute
    if let Ok(output) = Command::new("wpctl").args(["get-volume", "@DEFAULT_AUDIO_SINK@"]).output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        // "Volume: 0.45 [MUTED]" or "Volume: 0.45"
        if let Some(vol_str) = text.strip_prefix("Volume: ") {
            let muted = vol_str.contains("[MUTED]");
            let vol_num: f64 = vol_str
                .split_whitespace()
                .next()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.0);
            snap.default_sink_volume = (vol_num * 100.0).round() as u32;
            snap.default_sink_muted = muted;
        }
    }

    // Default source mute
    if let Ok(output) =
        Command::new("wpctl").args(["get-volume", "@DEFAULT_AUDIO_SOURCE@"]).output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        snap.default_source_muted = text.contains("[MUTED]");
    }

    // Parse pw-dump for sinks/sources/streams (heavy), fallback to pactl
    if let Ok(output) = Command::new("pactl")
        .args(["--format=json", "list", "sinks"])
        .output()
    {
        let expanded = expand_json(&String::from_utf8_lossy(&output.stdout));
        parse_pactl_sinks(&expanded, &mut snap);
    }
    if let Ok(output) = Command::new("pactl")
        .args(["--format=json", "list", "sources"])
        .output()
    {
        let expanded = expand_json(&String::from_utf8_lossy(&output.stdout));
        parse_pactl_sources(&expanded, &mut snap);
    }
    if let Ok(output) = Command::new("pactl")
        .args(["--format=json", "list", "sink-inputs"])
        .output()
    {
        let expanded = expand_json(&String::from_utf8_lossy(&output.stdout));
        parse_pactl_streams(&expanded, &mut snap);
    }

    snap
}

fn collect_pulse_snapshot() -> AudioSnapshot {
    let mut snap = AudioSnapshot {
        backend: AudioBackend::Pulseaudio,
        ..Default::default()
    };

    if let Ok(output) = Command::new("pactl")
        .args(["--format=json", "list", "sinks"])
        .output()
    {
        let expanded = expand_json(&String::from_utf8_lossy(&output.stdout));
        parse_pactl_sinks(&expanded, &mut snap);
    }
    if let Ok(output) = Command::new("pactl")
        .args(["--format=json", "list", "sources"])
        .output()
    {
        let expanded = expand_json(&String::from_utf8_lossy(&output.stdout));
        parse_pactl_sources(&expanded, &mut snap);
    }
    if let Ok(output) = Command::new("pactl")
        .args(["--format=json", "list", "sink-inputs"])
        .output()
    {
        let expanded = expand_json(&String::from_utf8_lossy(&output.stdout));
        parse_pactl_streams(&expanded, &mut snap);
    }

    // Default volumes from first default sink/source
    if let Some(def) = snap.sinks.iter().find(|s| s.is_default) {
        snap.default_sink_volume = def.volume_percent;
        snap.default_sink_muted = def.muted;
    }
    if let Some(def) = snap.sources.iter().find(|s| s.is_default) {
        snap.default_source_muted = def.muted;
    }

    snap
}

// ── Minimal JSON parsing (no serde_json dep) ──────────────

fn parse_pactl_sinks(json: &str, snap: &mut AudioSnapshot) {
    // pactl --format=json list sinks returns an array of objects
    // We do a minimal line-based parse to avoid adding serde_json
    let mut current_id: Option<u32> = None;
    let mut current_name = String::new();
    let mut current_desc = String::new();
    let mut current_vol: u32 = 0;
    let mut current_muted = false;
    let mut current_default = false;

    // Get default sink name
    let default_sink = Command::new("pactl")
        .args(["get-default-sink"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    for line in json.lines() {
        let trimmed = line.trim();
        if let Some(val) = extract_json_u32(trimmed, "\"index\"") {
            if current_id.is_some() {
                snap.sinks.push(AudioSink {
                    id: current_id.unwrap_or(0),
                    name: current_name.clone(),
                    description: current_desc.clone(),
                    volume_percent: current_vol,
                    muted: current_muted,
                    is_default: current_default,
                });
            }
            current_id = Some(val);
            current_name.clear();
            current_desc.clear();
            current_vol = 0;
            current_muted = false;
            current_default = false;
        }
        if current_name.is_empty() {
            if let Some(val) = extract_json_str(trimmed, "\"name\"") {
                current_default = val == default_sink;
                current_name = val;
            }
        }
        if current_desc.is_empty() {
            if let Some(val) = extract_json_str(trimmed, "\"description\"") {
                current_desc = val;
            }
        }
        if let Some(val) = extract_json_str(trimmed, "\"value_percent\"") {
            if current_vol == 0 {
                current_vol = val.trim_end_matches('%').parse().unwrap_or(0);
            }
        }
        if let Some(val) = extract_json_bool(trimmed, "\"mute\"") {
            current_muted = val;
        }
    }
    if let Some(id) = current_id {
        snap.sinks.push(AudioSink {
            id,
            name: current_name,
            description: current_desc,
            volume_percent: current_vol,
            muted: current_muted,
            is_default: current_default,
        });
    }
}

fn parse_pactl_sources(json: &str, snap: &mut AudioSnapshot) {
    let mut current_id: Option<u32> = None;
    let mut current_name = String::new();
    let mut current_desc = String::new();
    let mut current_vol: u32 = 0;
    let mut current_muted = false;
    let mut current_default = false;

    let default_source = Command::new("pactl")
        .args(["get-default-source"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    for line in json.lines() {
        let trimmed = line.trim();
        if let Some(val) = extract_json_u32(trimmed, "\"index\"") {
            if current_id.is_some() {
                snap.sources.push(AudioSource {
                    id: current_id.unwrap_or(0),
                    name: current_name.clone(),
                    description: current_desc.clone(),
                    volume_percent: current_vol,
                    muted: current_muted,
                    is_default: current_default,
                });
            }
            current_id = Some(val);
            current_name.clear();
            current_desc.clear();
            current_vol = 0;
            current_muted = false;
            current_default = false;
        }
        if current_name.is_empty() {
            if let Some(val) = extract_json_str(trimmed, "\"name\"") {
                current_default = val == default_source;
                current_name = val;
            }
        }
        if current_desc.is_empty() {
            if let Some(val) = extract_json_str(trimmed, "\"description\"") {
                current_desc = val;
            }
        }
        if let Some(val) = extract_json_str(trimmed, "\"value_percent\"") {
            if current_vol == 0 {
                current_vol = val.trim_end_matches('%').parse().unwrap_or(0);
            }
        }
        if let Some(val) = extract_json_bool(trimmed, "\"mute\"") {
            current_muted = val;
        }
    }
    if let Some(id) = current_id {
        snap.sources.push(AudioSource {
            id,
            name: current_name,
            description: current_desc,
            volume_percent: current_vol,
            muted: current_muted,
            is_default: current_default,
        });
    }
}

fn parse_pactl_streams(json: &str, snap: &mut AudioSnapshot) {
    let mut current_id: Option<u32> = None;
    let mut current_app = String::new();
    let mut current_vol: u32 = 0;
    let mut current_muted = false;
    let mut current_sink: Option<u32> = None;

    for line in json.lines() {
        let trimmed = line.trim();
        if let Some(val) = extract_json_u32(trimmed, "\"index\"") {
            if current_id.is_some() {
                snap.streams.push(AudioStream {
                    id: current_id.unwrap_or(0),
                    app_name: current_app.clone(),
                    volume_percent: current_vol,
                    muted: current_muted,
                    sink_id: current_sink,
                });
            }
            current_id = Some(val);
            current_app.clear();
            current_vol = 0;
            current_muted = false;
            current_sink = None;
        }
        if let Some(val) = extract_json_u32(trimmed, "\"sink\"") {
            current_sink = Some(val);
        }
        if let Some(val) = extract_json_str(trimmed, "\"application.name\"") {
            current_app = val;
        }
        if let Some(val) = extract_json_str(trimmed, "\"value_percent\"") {
            if current_vol == 0 {
                current_vol = val.trim_end_matches('%').parse().unwrap_or(0);
            }
        }
        if let Some(val) = extract_json_bool(trimmed, "\"mute\"") {
            current_muted = val;
        }
    }
    if let Some(id) = current_id {
        snap.streams.push(AudioStream {
            id,
            app_name: current_app,
            volume_percent: current_vol,
            muted: current_muted,
            sink_id: current_sink,
        });
    }
}

// ── Controls ───────────────────────────────────────────────

pub fn set_default_sink_volume(percent: u32) {
    let vol = format!("{}%", percent.min(150));
    let backend = detect_audio_backend();
    match backend {
        AudioBackend::Pipewire => {
            let _ =
                Command::new("wpctl").args(["set-volume", "@DEFAULT_AUDIO_SINK@", &vol]).output();
        }
        AudioBackend::Pulseaudio => {
            let _ = Command::new("pactl")
                .args(["set-sink-volume", "@DEFAULT_SINK@", &vol])
                .output();
        }
        AudioBackend::None => {}
    }
}

pub fn toggle_default_sink_mute() {
    let backend = detect_audio_backend();
    match backend {
        AudioBackend::Pipewire => {
            let _ = Command::new("wpctl")
                .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
                .output();
        }
        AudioBackend::Pulseaudio => {
            let _ = Command::new("pactl")
                .args(["set-sink-mute", "@DEFAULT_SINK@", "toggle"])
                .output();
        }
        AudioBackend::None => {}
    }
}

pub fn toggle_default_source_mute() {
    let backend = detect_audio_backend();
    match backend {
        AudioBackend::Pipewire => {
            let _ = Command::new("wpctl")
                .args(["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"])
                .output();
        }
        AudioBackend::Pulseaudio => {
            let _ = Command::new("pactl")
                .args(["set-source-mute", "@DEFAULT_SOURCE@", "toggle"])
                .output();
        }
        AudioBackend::None => {}
    }
}

pub fn set_default_sink(name: &str) {
    let backend = detect_audio_backend();
    match backend {
        AudioBackend::Pipewire => {
            // wpctl uses node IDs; fall through to pactl
            let _ = Command::new("pactl")
                .args(["set-default-sink", name])
                .output();
        }
        AudioBackend::Pulseaudio => {
            let _ = Command::new("pactl")
                .args(["set-default-sink", name])
                .output();
        }
        AudioBackend::None => {}
    }
}

pub fn set_stream_volume(stream_id: u32, percent: u32) {
    let vol = format!("{}%", percent.min(150));
    let _ = Command::new("pactl")
        .args(["set-sink-input-volume", &stream_id.to_string(), &vol])
        .output();
}

pub fn toggle_stream_mute(stream_id: u32) {
    let _ = Command::new("pactl")
        .args(["set-sink-input-mute", &stream_id.to_string(), "toggle"])
        .output();
}

// ── Power / Battery ────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct PowerInfo {
    pub on_battery: bool,
    pub battery_percent: Option<u32>,
    pub charging: bool,
    pub time_remaining: String,
    pub power_profile: String,
}

pub fn collect_power_info() -> PowerInfo {
    let mut info = PowerInfo::default();

    // upower -i /org/freedesktop/UPower/devices/battery_BAT0
    if let Ok(output) = Command::new("upower")
        .args(["-i", "/org/freedesktop/UPower/devices/battery_BAT0"])
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("percentage:") {
                info.battery_percent = trimmed
                    .split_whitespace()
                    .last()
                    .and_then(|v| v.trim_end_matches('%').parse().ok());
            }
            if trimmed.starts_with("state:") {
                let state = trimmed.split_whitespace().last().unwrap_or("");
                info.charging = state == "charging";
                info.on_battery = state == "discharging";
            }
            if trimmed.starts_with("time to empty:")
                || trimmed.starts_with("time to full:")
            {
                info.time_remaining = trimmed
                    .split(':')
                    .skip(1)
                    .collect::<Vec<_>>()
                    .join(":")
                    .trim()
                    .to_string();
            }
        }
    }

    // Power profile (powerprofilesctl or system76-power)
    if let Ok(output) = Command::new("powerprofilesctl").arg("get").output() {
        if output.status.success() {
            info.power_profile = String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }

    info
}

// ── Brightness ─────────────────────────────────────────────

pub fn get_brightness_percent() -> Option<u32> {
    let output = Command::new("brightnessctl")
        .args(["info", "-m"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    // Format: "device,class,current,max,percent%"
    text.split(',')
        .nth(3)
        .or_else(|| {
            // fallback to percentage field
            text.split(',').last()
        })
        .and_then(|v| v.trim().trim_end_matches('%').parse().ok())
}

pub fn set_brightness_percent(percent: u32) {
    let val = format!("{}%", percent.min(100));
    let _ = Command::new("brightnessctl").args(["set", &val]).output();
}

// ── Idle Inhibit ───────────────────────────────────────────

/// Check whether idle/sleep inhibit is currently active.
/// Looks for any existing inhibitor locks via `systemd-inhibit --list`.
#[allow(dead_code)]
pub fn is_idle_inhibited() -> bool {
    // Check if our own inhibitor process is still alive
    // We use a sentinel approach: look for our inhibitor in the list
    Command::new("systemd-inhibit")
        .args(["--list", "--no-pager"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("cyberfile"))
        .unwrap_or(false)
}

/// Start an idle inhibitor. Returns the child process handle.
/// The inhibitor stays active as long as the process is alive.
pub fn start_idle_inhibit() -> Option<std::process::Child> {
    Command::new("systemd-inhibit")
        .args([
            "--what=idle",
            "--who=cyberfile",
            "--why=User requested idle inhibit",
            "--mode=block",
            "sleep",
            "infinity",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()
}

/// Stop an idle inhibitor by killing the child process.
pub fn stop_idle_inhibit(child: &mut std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}

// ── Clipboard History ──────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ClipboardEntry {
    pub index: usize,
    pub preview: String,
}

/// Read clipboard history from clipman (wl-clipboard) or similar.
/// Falls back to just the current clipboard content.
pub fn read_clipboard_entries(limit: usize) -> Vec<ClipboardEntry> {
    // Try cliphist (common Wayland clipboard manager)
    if let Ok(output) = Command::new("cliphist").arg("list").output() {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            return text
                .lines()
                .take(limit)
                .enumerate()
                .map(|(i, line)| {
                    let preview = if line.len() > 80 {
                        format!("{}…", &line[..80])
                    } else {
                        line.to_string()
                    };
                    ClipboardEntry { index: i, preview }
                })
                .collect();
        }
    }

    // Fallback: just current clipboard
    let current = read_current_clipboard().unwrap_or_default();
    if current.is_empty() {
        return Vec::new();
    }
    let preview = if current.len() > 80 {
        format!("{}…", &current[..80])
    } else {
        current
    };
    vec![ClipboardEntry { index: 0, preview }]
}

fn read_current_clipboard() -> Option<String> {
    // Wayland
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        if let Ok(output) = Command::new("wl-paste").arg("--no-newline").output() {
            if output.status.success() {
                return Some(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
    }
    // X11 fallback
    if let Ok(output) = Command::new("xclip")
        .args(["-selection", "clipboard", "-o"])
        .output()
    {
        if output.status.success() {
            return Some(String::from_utf8_lossy(&output.stdout).to_string());
        }
    }
    None
}

pub fn paste_clipboard_entry(entry_index: usize) {
    // Use cliphist select
    if let Ok(output) = Command::new("cliphist").arg("list").output() {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = text.lines().nth(entry_index) {
                // pipe into cliphist decode | wl-copy
                use std::io::Write;
                use std::process::Stdio;
                if let Ok(mut decode) = Command::new("cliphist")
                    .arg("decode")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                {
                    if let Some(stdin) = decode.stdin.as_mut() {
                        let _ = stdin.write_all(line.as_bytes());
                    }
                    if let Ok(decoded) = decode.wait_with_output() {
                        if let Ok(mut copy) = Command::new("wl-copy").stdin(Stdio::piped()).spawn()
                        {
                            if let Some(stdin) = copy.stdin.as_mut() {
                                let _ = stdin.write_all(&decoded.stdout);
                            }
                            let _ = copy.wait();
                        }
                    }
                }
            }
        }
    }
}

// ── Notification History ───────────────────────────────────

#[derive(Debug, Clone)]
pub struct NotificationEntry {
    pub id: u32,
    pub app: String,
    pub summary: String,
    pub body: String,
}

/// Read notification history from dunst (dunstctl) or swaync.
pub fn read_notification_history(limit: usize) -> Vec<NotificationEntry> {
    // Try dunstctl
    if let Ok(output) = Command::new("dunstctl").arg("history").output() {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            return parse_dunst_history(&text, limit);
        }
    }

    // Try swaync-client
    if let Ok(output) = Command::new("swaync-client").arg("--get-json").output() {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            return parse_swaync_history(&text, limit);
        }
    }

    Vec::new()
}

fn parse_dunst_history(json: &str, limit: usize) -> Vec<NotificationEntry> {
    // dunstctl history outputs JSON with { "type": "...", "data": [[notifications]] }
    let mut entries = Vec::new();
    let mut current_id: Option<u32> = None;
    let mut current_app = String::new();
    let mut current_summary = String::new();
    let mut current_body = String::new();
    let mut in_data = false;

    for line in json.lines() {
        let trimmed = line.trim();
        if trimmed.contains("\"data\"") {
            in_data = true;
        }
        if !in_data {
            continue;
        }
        if let Some(val) = extract_json_u32(trimmed, "\"id\"") {
            if let Some(id) = current_id {
                entries.push(NotificationEntry {
                    id,
                    app: current_app.clone(),
                    summary: current_summary.clone(),
                    body: current_body.clone(),
                });
                if entries.len() >= limit {
                    return entries;
                }
            }
            current_id = Some(val);
            current_app.clear();
            current_summary.clear();
            current_body.clear();
        }
        if let Some(val) = extract_json_nested_str(trimmed, "\"appname\"") {
            current_app = val;
        }
        if let Some(val) = extract_json_nested_str(trimmed, "\"summary\"") {
            current_summary = val;
        }
        if let Some(val) = extract_json_nested_str(trimmed, "\"body\"") {
            current_body = val;
        }
    }
    if let Some(id) = current_id {
        entries.push(NotificationEntry {
            id,
            app: current_app,
            summary: current_summary,
            body: current_body,
        });
    }
    entries
}

fn parse_swaync_history(json: &str, limit: usize) -> Vec<NotificationEntry> {
    // Similar structure — array of notification objects
    let mut entries = Vec::new();
    let mut current_id: Option<u32> = None;
    let mut current_app = String::new();
    let mut current_summary = String::new();
    let mut current_body = String::new();

    for line in json.lines() {
        let trimmed = line.trim();
        if let Some(val) = extract_json_u32(trimmed, "\"id\"") {
            if let Some(id) = current_id {
                entries.push(NotificationEntry {
                    id,
                    app: current_app.clone(),
                    summary: current_summary.clone(),
                    body: current_body.clone(),
                });
                if entries.len() >= limit {
                    return entries;
                }
            }
            current_id = Some(val);
            current_app.clear();
            current_summary.clear();
            current_body.clear();
        }
        if let Some(val) = extract_json_str(trimmed, "\"app_name\"") {
            current_app = val;
        }
        if let Some(val) = extract_json_str(trimmed, "\"summary\"") {
            current_summary = val;
        }
        if let Some(val) = extract_json_str(trimmed, "\"body\"") {
            current_body = val;
        }
    }
    if let Some(id) = current_id {
        entries.push(NotificationEntry {
            id,
            app: current_app,
            summary: current_summary,
            body: current_body,
        });
    }
    entries
}

pub fn dismiss_notification(id: u32) {
    // dunstctl close-by-id
    let _ = Command::new("dunstctl")
        .args(["close", &id.to_string()])
        .output();
}

pub fn clear_all_notifications() {
    let _ = Command::new("dunstctl").arg("close-all").output();
}

// ── Tiny JSON Helpers (no serde_json dep) ──────────────────

/// Expand minified JSON so each key-value sits on its own line.
/// The line-based extractors below rely on one key per line.
fn expand_json(json: &str) -> String {
    let mut out = String::with_capacity(json.len() * 2);
    let mut in_string = false;
    let mut prev = '\0';
    for ch in json.chars() {
        if ch == '"' && prev != '\\' {
            in_string = !in_string;
        }
        if !in_string {
            match ch {
                '{' | '[' => {
                    out.push(ch);
                    out.push('\n');
                    prev = ch;
                    continue;
                }
                '}' | ']' => {
                    out.push('\n');
                    out.push(ch);
                    prev = ch;
                    continue;
                }
                ',' => {
                    out.push(ch);
                    out.push('\n');
                    prev = ch;
                    continue;
                }
                _ => {}
            }
        }
        out.push(ch);
        prev = ch;
    }
    out
}

fn extract_json_str(line: &str, key: &str) -> Option<String> {
    let pos = line.find(key)?;
    let rest = &line[pos + key.len()..];
    let rest = rest.trim_start_matches(|c: char| c == ':' || c == ' ');
    if rest.starts_with('"') {
        let inner = &rest[1..];
        let end = inner.find('"')?;
        Some(inner[..end].to_string())
    } else {
        None
    }
}

fn extract_json_u32(line: &str, key: &str) -> Option<u32> {
    let pos = line.find(key)?;
    let rest = &line[pos + key.len()..];
    let rest = rest.trim_start_matches(|c: char| c == ':' || c == ' ');
    let num_end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    rest[..num_end].parse().ok()
}

fn extract_json_bool(line: &str, key: &str) -> Option<bool> {
    let pos = line.find(key)?;
    let rest = &line[pos + key.len()..];
    let rest = rest.trim_start_matches(|c: char| c == ':' || c == ' ');
    if rest.starts_with("true") {
        Some(true)
    } else if rest.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

fn extract_json_nested_str(line: &str, key: &str) -> Option<String> {
    // dunst nests values as {"type": "string", "data": "actual_value"}
    // Look for "data" after the key
    let pos = line.find(key)?;
    let rest = &line[pos..];
    if let Some(data_pos) = rest.find("\"data\"") {
        let after = &rest[data_pos + 6..];
        let after = after.trim_start_matches(|c: char| c == ':' || c == ' ');
        if after.starts_with('"') {
            let inner = &after[1..];
            let end = inner.find('"')?;
            return Some(inner[..end].to_string());
        }
    }
    None
}
