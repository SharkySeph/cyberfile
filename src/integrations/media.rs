use std::process::{Command, Stdio};

#[derive(Debug, Clone, Default)]
pub struct MediaState {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub playing: bool,
    pub available: bool,
    pub player_name: String,
}

/// Detect the currently active MPRIS player (if any).
/// Returns the player instance name (e.g. "spotify", "firefox.instance123", "vlc").
pub fn get_active_player() -> Option<String> {
    // First try to find a playing player
    let output = Command::new("playerctl")
        .args(["--list-all"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let players: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    if players.is_empty() {
        return None;
    }

    // Check each player's status, prefer one that's Playing
    for player in &players {
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

        if status.as_deref() == Some("Playing") {
            return Some(player.clone());
        }
    }

    // If none are playing, check for paused
    for player in &players {
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

        if status.as_deref() == Some("Paused") {
            return Some(player.clone());
        }
    }

    // Fall back to first listed player
    Some(players[0].clone())
}

/// Friendly display name from a player instance ID
fn friendly_name(player: &str) -> String {
    // playerctl names like "spotify", "firefox.instance12345", "vlc", "chromium.instance999"
    let base = player.split('.').next().unwrap_or(player);
    base.to_uppercase()
}

/// Query current media playback state via playerctl MPRIS.
/// Auto-detects the active player.
pub fn get_state() -> MediaState {
    let player = match get_active_player() {
        Some(p) => p,
        None => {
            return MediaState {
                available: false,
                ..Default::default()
            }
        }
    };

    let status = Command::new("playerctl")
        .args(["--player", &player, "status"])
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
        _ => {
            return MediaState {
                available: false,
                ..Default::default()
            }
        }
    };

    let metadata = Command::new("playerctl")
        .args([
            "--player",
            &player,
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

    MediaState {
        title,
        artist,
        album,
        playing,
        available: true,
        player_name: friendly_name(&player),
    }
}

pub fn play_pause() {
    let _ = Command::new("playerctl")
        .args(["play-pause"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

pub fn next_track() {
    let _ = Command::new("playerctl")
        .args(["next"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

pub fn previous_track() {
    let _ = Command::new("playerctl")
        .args(["previous"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}
