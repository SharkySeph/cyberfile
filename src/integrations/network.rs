use std::process::Command;

// ── Data Structs ───────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkInterface {
    pub device: String,
    pub iface_type: String,
    pub state: String,
    pub connection: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal: u8,
    pub security: String,
    pub in_use: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VpnConnection {
    pub name: String,
    pub vpn_type: String,
    pub active: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ThroughputSample {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

// ── Interface Listing ──────────────────────────────────────

pub fn list_interfaces() -> Result<Vec<NetworkInterface>, String> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "DEVICE,TYPE,STATE,CONNECTION", "device", "status"])
        .output()
        .map_err(|e| format!("nmcli failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("nmcli error: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut interfaces = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 4 {
            continue;
        }
        // Skip loopback
        if parts[0] == "lo" {
            continue;
        }
        interfaces.push(NetworkInterface {
            device: parts[0].to_string(),
            iface_type: parts[1].to_string(),
            state: parts[2].to_string(),
            connection: parts[3].to_string(),
        });
    }

    Ok(interfaces)
}

// ── WiFi Scan ──────────────────────────────────────────────

pub fn list_wifi_networks() -> Result<Vec<WifiNetwork>, String> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "IN-USE,SSID,SIGNAL,SECURITY", "device", "wifi", "list"])
        .output()
        .map_err(|e| format!("nmcli wifi list failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("nmcli wifi error: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut networks = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.splitn(4, ':').collect();
        if parts.len() < 4 {
            continue;
        }
        let ssid = parts[1].trim().to_string();
        if ssid.is_empty() {
            continue;
        }
        networks.push(WifiNetwork {
            in_use: parts[0].trim() == "*",
            ssid,
            signal: parts[2].trim().parse().unwrap_or(0),
            security: parts[3].trim().to_string(),
        });
    }

    // Sort by signal strength descending
    networks.sort_by(|a, b| b.signal.cmp(&a.signal));
    Ok(networks)
}

// ── VPN Connections ────────────────────────────────────────

pub fn list_vpn_connections() -> Result<Vec<VpnConnection>, String> {
    // Get all connections of type vpn or wireguard
    let output = Command::new("nmcli")
        .args(["-t", "-f", "NAME,TYPE,ACTIVE", "connection", "show"])
        .output()
        .map_err(|e| format!("nmcli vpn list failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("nmcli error: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut vpns = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 3 {
            continue;
        }
        let conn_type = parts[1];
        if !conn_type.contains("vpn") && !conn_type.contains("wireguard") {
            continue;
        }
        vpns.push(VpnConnection {
            name: parts[0].to_string(),
            vpn_type: conn_type.to_string(),
            active: parts[2].trim() == "yes",
        });
    }

    Ok(vpns)
}

// ── Throughput Reading ─────────────────────────────────────

/// Read current byte counters from /sys/class/net for a given device.
pub fn read_throughput(device: &str) -> Option<ThroughputSample> {
    let rx = std::fs::read_to_string(format!("/sys/class/net/{}/statistics/rx_bytes", device))
        .ok()?
        .trim()
        .parse()
        .ok()?;
    let tx = std::fs::read_to_string(format!("/sys/class/net/{}/statistics/tx_bytes", device))
        .ok()?
        .trim()
        .parse()
        .ok()?;
    Some(ThroughputSample { rx_bytes: rx, tx_bytes: tx })
}

// ── Actions ────────────────────────────────────────────────

pub fn connect_wifi(ssid: &str, password: Option<&str>) -> Result<(), String> {
    let mut args = vec!["device", "wifi", "connect", ssid];
    let pw;
    if let Some(p) = password {
        pw = p.to_string();
        args.push("password");
        args.push(&pw);
    }
    let output = Command::new("nmcli")
        .args(&args)
        .output()
        .map_err(|e| format!("nmcli connect failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().to_string());
    }
    Ok(())
}

pub fn disconnect_device(device: &str) -> Result<(), String> {
    let output = Command::new("nmcli")
        .args(["device", "disconnect", device])
        .output()
        .map_err(|e| format!("nmcli disconnect failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().to_string());
    }
    Ok(())
}

pub fn toggle_vpn(name: &str, activate: bool) -> Result<(), String> {
    let action = if activate { "up" } else { "down" };
    let output = Command::new("nmcli")
        .args(["connection", action, name])
        .output()
        .map_err(|e| format!("nmcli vpn {} failed: {}", action, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().to_string());
    }
    Ok(())
}

/// Check if nmcli is available on the system.
pub fn nmcli_available() -> bool {
    Command::new("nmcli")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
