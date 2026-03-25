use std::collections::HashMap;
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessEntry {
    pub pid: i32,
    pub parent_pid: Option<i32>,
    pub name: String,
    pub command: String,
    pub cwd: String,
    pub status: String,
    pub cpu_percent: f32,
    pub memory_kib: u64,
    pub child_count: usize,
}

pub fn collect_processes(limit: usize) -> Result<Vec<ProcessEntry>, String> {
    let output = Command::new("ps")
        .args([
            "-eo",
            "pid=,ppid=,pcpu=,rss=,stat=,comm=,args=",
            "--sort=-pcpu",
        ])
        .output()
        .map_err(|error| format!("ps failed: {}", error))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ps failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut rows = Vec::new();
    let mut child_counts: HashMap<i32, usize> = HashMap::new();

    for line in stdout.lines() {
        let mut parts = line.split_whitespace();
        let Some(pid) = parts.next().and_then(|value| value.parse::<i32>().ok()) else {
            continue;
        };
        let parent_pid = parts.next().and_then(|value| value.parse::<i32>().ok());
        let cpu_percent = parts
            .next()
            .and_then(|value| value.parse::<f32>().ok())
            .unwrap_or(0.0);
        let memory_kib = parts
            .next()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(0);
        let status = parts.next().unwrap_or_default().to_string();
        let name = parts.next().unwrap_or_default().to_string();
        let command = parts.collect::<Vec<_>>().join(" ");
        let cwd = fs::read_link(format!("/proc/{}/cwd", pid))
            .ok()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "-".to_string());

        if let Some(parent_pid) = parent_pid {
            *child_counts.entry(parent_pid).or_insert(0) += 1;
        }

        rows.push(ProcessEntry {
            pid,
            parent_pid,
            name,
            command,
            cwd,
            status,
            cpu_percent,
            memory_kib,
            child_count: 0,
        });
    }

    for row in &mut rows {
        row.child_count = child_counts.get(&row.pid).copied().unwrap_or(0);
    }

    if rows.len() > limit {
        rows.truncate(limit);
    }

    Ok(rows)
}

pub fn terminate_process(pid: i32, force: bool) -> Result<(), String> {
    let signal = if force { "-KILL" } else { "-TERM" };
    let output = Command::new("kill")
        .args([signal, &pid.to_string()])
        .output()
        .map_err(|error| format!("kill failed: {}", error))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("kill failed: {}", stderr.trim()))
    }
}