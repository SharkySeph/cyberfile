use std::process::{Command, Stdio};

#[derive(Debug, Clone, Default)]
pub struct MediaState {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub playing: bool,
    pub available: bool,
    pub player_name: String,
    pub player_id: String,
    pub position_secs: f64,
    pub duration_secs: f64,
}

#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub id: String,
    pub display_name: String,
    pub status: String,
}

/// List all available MPRIS players with their status.
/// Uses a single playerctl call to fetch all statuses at once.
pub fn list_players() -> Vec<PlayerInfo> {
    // playerctl --all-players status --format '{{playerInstance}}\t{{status}}'
    let output = match Command::new("playerctl")
        .args(["--all-players", "status", "--format", "{{playerInstance}}\t{{status}}"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| {
            let mut parts = line.splitn(2, '\t');
            let id = parts.next()?.trim().to_string();
            let status = parts.next().map(|s| s.trim().to_string()).unwrap_or_else(|| "Unknown".into());
            if id.is_empty() {
                return None;
            }
            let display_name = friendly_name(&id);
            Some(PlayerInfo {
                id,
                display_name,
                status,
            })
        })
        .collect()
}

/// Seek to an absolute position (seconds) on the given player.
pub fn seek_to(player_id: &str, position_secs: f64) {
    // playerctl position sets absolute position in seconds
    let _ = Command::new("playerctl")
        .args(["--player", player_id, "position", &format!("{:.1}", position_secs)])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output();
}

/// Get state for a specific player (or auto-detect if empty).
pub fn get_state_for_player(preferred: &str) -> MediaState {
    let player = if preferred.is_empty() {
        match get_active_player() {
            Some(p) => p,
            None => return MediaState::default(),
        }
    } else {
        preferred.to_string()
    };
    get_state_impl(&player)
}

/// Detect the currently active MPRIS player (if any).
/// Returns the player instance name (e.g. "spotify", "firefox.instance123", "vlc").
/// Uses a single playerctl call to get all player statuses at once.
pub fn get_active_player() -> Option<String> {
    let output = Command::new("playerctl")
        .args(["--all-players", "status", "--format", "{{playerInstance}}\t{{status}}"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let entries: Vec<(String, String)> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, '\t');
            let id = parts.next()?.trim().to_string();
            let status = parts.next().map(|s| s.trim().to_string()).unwrap_or_default();
            if id.is_empty() { None } else { Some((id, status)) }
        })
        .collect();

    if entries.is_empty() {
        return None;
    }

    // Prefer Playing, then Paused, then first available.
    // Deprioritise relay-style bridges (e.g. kdeconnect) that are not
    // directly controllable as local players.
    let is_relay = |id: &str| id.starts_with("kdeconnect");

    // Playing local player first
    if let Some((id, _)) = entries.iter().find(|(id, s)| s == "Playing" && !is_relay(id)) {
        return Some(id.clone());
    }
    // Playing relay as fallback
    if let Some((id, _)) = entries.iter().find(|(_, s)| s == "Playing") {
        return Some(id.clone());
    }
    // Paused local player
    if let Some((id, _)) = entries.iter().find(|(id, s)| s == "Paused" && !is_relay(id)) {
        return Some(id.clone());
    }
    // Paused relay
    if let Some((id, _)) = entries.iter().find(|(_, s)| s == "Paused") {
        return Some(id.clone());
    }
    Some(entries[0].0.clone())
}

/// Friendly display name from a player instance ID
fn friendly_name(player: &str) -> String {
    // playerctl names like "spotify", "firefox.instance12345", "vlc", "chromium.instance999"
    let base = player.split('.').next().unwrap_or(player);
    base.to_uppercase()
}

fn get_state_impl(player: &str) -> MediaState {
    let status = Command::new("playerctl")
        .args(["--player", player, "status"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        });

    let playing = match status.as_deref() {
        Some("Playing") => true,
        Some("Paused") => false,
        _ => return MediaState::default(),
    };

    let metadata = Command::new("playerctl")
        .args([
            "--player",
            player,
            "metadata",
            "--format",
            "{{title}}\n{{artist}}\n{{album}}",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).to_string())
            } else {
                None
            }
        });

    let (title, artist, album) = match metadata {
        Some(m) => {
            let lines: Vec<&str> = m.lines().collect();
            (
                lines.first().unwrap_or(&"").to_string(),
                lines.get(1).unwrap_or(&"").to_string(),
                lines.get(2).unwrap_or(&"").to_string(),
            )
        }
        None => (String::new(), String::new(), String::new()),
    };

    // Position (seconds, float)
    let position_secs = Command::new("playerctl")
        .args(["--player", player, "position"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<f64>().ok())
        .unwrap_or(0.0);

    // Duration (microseconds from metadata)
    let duration_secs = Command::new("playerctl")
        .args(["--player", player, "metadata", "mpris:length"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<f64>().ok())
        .map(|us| us / 1_000_000.0)
        .unwrap_or(0.0);

    MediaState {
        title,
        artist,
        album,
        playing,
        available: true,
        player_name: friendly_name(player),
        player_id: player.to_string(),
        position_secs,
        duration_secs,
    }
}

pub fn play_pause_player(player_id: &str) {
    let _ = Command::new("playerctl")
        .args(["--player", player_id, "play-pause"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output();
}

pub fn next_track_player(player_id: &str) {
    let _ = Command::new("playerctl")
        .args(["--player", player_id, "next"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output();
}

pub fn previous_track_player(player_id: &str) {
    let _ = Command::new("playerctl")
        .args(["--player", player_id, "previous"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output();
}
