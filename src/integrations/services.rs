use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceEntry {
    pub unit: String,
    pub load: String,
    pub active: String,
    pub sub: String,
    pub description: String,
    pub enabled: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceAction {
    Start,
    Stop,
    Restart,
    Enable,
    Disable,
}

impl ServiceAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Restart => "restart",
            Self::Enable => "enable",
            Self::Disable => "disable",
        }
    }
}

pub fn list_user_services(limit: usize) -> Result<Vec<ServiceEntry>, String> {
    let enabled_map = list_unit_file_states()?;
    let output = Command::new("systemctl")
        .args([
            "--user",
            "list-units",
            "--type=service",
            "--all",
            "--no-pager",
            "--no-legend",
            "--plain",
        ])
        .output()
        .map_err(|error| format!("systemctl failed: {}", error))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("systemctl failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut rows = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let Some(unit) = parts.next() else {
            continue;
        };
        let load = parts.next().unwrap_or_default().to_string();
        let active = parts.next().unwrap_or_default().to_string();
        let sub = parts.next().unwrap_or_default().to_string();
        let description = parts.collect::<Vec<_>>().join(" ");
        rows.push(ServiceEntry {
            unit: unit.to_string(),
            load,
            active,
            sub,
            description,
            enabled: enabled_map
                .get(unit)
                .cloned()
                .unwrap_or_else(|| "unknown".to_string()),
        });
    }

    if rows.len() > limit {
        rows.truncate(limit);
    }

    Ok(rows)
}

fn list_unit_file_states() -> Result<HashMap<String, String>, String> {
    let output = Command::new("systemctl")
        .args([
            "--user",
            "list-unit-files",
            "--type=service",
            "--no-pager",
            "--no-legend",
            "--plain",
        ])
        .output()
        .map_err(|error| format!("systemctl failed: {}", error))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("systemctl failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut states = HashMap::new();
    for line in stdout.lines() {
        let mut parts = line.split_whitespace();
        let Some(unit) = parts.next() else {
            continue;
        };
        let state = parts.next().unwrap_or("unknown");
        states.insert(unit.to_string(), state.to_string());
    }
    Ok(states)
}

pub fn inspect_user_service(unit: &str) -> Result<String, String> {
    let output = Command::new("systemctl")
        .args(["--user", "status", "--", unit, "--no-pager", "--lines=40"])
        .output()
        .map_err(|error| format!("systemctl failed: {}", error))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    if combined.trim().is_empty() {
        Err(format!("No status output returned for {}", unit))
    } else {
        Ok(combined)
    }
}

pub fn control_user_service(unit: &str, action: ServiceAction) -> Result<String, String> {
    let output = Command::new("systemctl")
        .args(["--user", action.label(), "--", unit])
        .output()
        .map_err(|error| format!("systemctl failed: {}", error))?;

    if output.status.success() {
        Ok(format!("{} {}", action.label(), unit))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("{} failed for {}: {}", action.label(), unit, stderr.trim()))
    }
}