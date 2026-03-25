use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct LogChannel {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default = "default_lines")]
    pub lines: usize,
    #[serde(default)]
    pub since: String,
    #[serde(default)]
    pub priority: Option<String>,
}

fn default_lines() -> usize {
    120
}

pub fn default_log_channels() -> Vec<LogChannel> {
    vec![
        LogChannel {
            id: "journal.user".to_string(),
            name: "USER JOURNAL".to_string(),
            unit: None,
            lines: 120,
            since: "-30min".to_string(),
            priority: None,
        },
        LogChannel {
            id: "journal.warnings".to_string(),
            name: "RECENT WARNINGS".to_string(),
            unit: None,
            lines: 120,
            since: "-60min".to_string(),
            priority: Some("warning".to_string()),
        },
    ]
}

pub fn read_channel(channel: &LogChannel) -> Result<Vec<String>, String> {
    let mut command = Command::new("journalctl");
    command.args(["--user", "--no-pager", "--output=short-iso"]);
    if let Some(unit) = &channel.unit {
        command.args(["-u", unit]);
    }
    if !channel.since.trim().is_empty() {
        command.args(["--since", channel.since.trim()]);
    }
    if let Some(priority) = &channel.priority {
        command.args(["-p", priority]);
    }
    command.args(["-n", &channel.lines.to_string()]);

    let output = command
        .output()
        .map_err(|error| format!("journalctl failed: {}", error))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("journalctl failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().map(|line| line.to_string()).collect())
}

pub fn service_channel(unit: &str) -> LogChannel {
    LogChannel {
        id: format!("journal.service.{}", unit),
        name: format!("SERVICE // {}", unit),
        unit: Some(unit.to_string()),
        lines: 120,
        since: "-30min".to_string(),
        priority: None,
    }
}