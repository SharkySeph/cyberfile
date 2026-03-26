use std::process::Command;

// ── Data Structs ───────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockDevice {
    pub name: String,
    pub label: String,
    pub size: String,
    pub fstype: String,
    pub mountpoint: String,
    pub model: String,
    pub removable: bool,
    pub dev_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DeviceAction {
    Mount,
    Unmount,
    Eject,
}

#[allow(dead_code)]
impl DeviceAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Mount => "Mount",
            Self::Unmount => "Unmount",
            Self::Eject => "Eject",
        }
    }
}

// ── Device Listing ─────────────────────────────────────────

pub fn list_block_devices() -> Result<Vec<BlockDevice>, String> {
    let output = Command::new("lsblk")
        .args([
            "-J",
            "-o", "NAME,LABEL,SIZE,FSTYPE,MOUNTPOINT,MODEL,RM,TYPE",
        ])
        .output()
        .map_err(|e| format!("lsblk failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("lsblk error: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();
    parse_lsblk_json(&stdout, &mut devices);
    Ok(devices)
}

fn parse_lsblk_json(json: &str, devices: &mut Vec<BlockDevice>) {
    // Minimal JSON parser for lsblk output — avoids serde_json dependency.
    // lsblk -J produces { "blockdevices": [ { ... }, ... ] }
    // We parse flat fields and recurse into "children" arrays.

    let Some(arr_start) = json.find("\"blockdevices\"") else {
        return;
    };
    let slice = &json[arr_start..];
    parse_device_array(slice, devices, false);
}

fn parse_device_array(json: &str, devices: &mut Vec<BlockDevice>, parent_removable: bool) {
    // Find the opening '[' of the array
    let Some(start) = json.find('[') else {
        return;
    };
    let mut depth = 0;
    let mut obj_start = None;

    for (i, c) in json[start..].char_indices() {
        match c {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            '{' if depth == 1 => {
                obj_start = Some(start + i);
            }
            '}' if depth == 1 => {
                if let Some(os) = obj_start {
                    let obj = &json[os..=start + i];
                    parse_single_device(obj, devices, parent_removable);
                    obj_start = None;
                }
            }
            _ => {}
        }
    }
}

fn parse_single_device(obj: &str, devices: &mut Vec<BlockDevice>, parent_removable: bool) {
    let name = extract_json_string(obj, "name");
    let label = extract_json_string(obj, "label");
    let size = extract_json_string(obj, "size");
    let fstype = extract_json_string(obj, "fstype");
    let mountpoint = extract_json_string(obj, "mountpoint");
    let model = extract_json_string(obj, "model");
    let dev_type = extract_json_string(obj, "type");
    let rm_str = extract_json_string(obj, "rm");
    let removable = rm_str == "1" || rm_str == "true" || parent_removable;

    // Only include partitions and disks, skip loop devices
    if dev_type == "loop" || name.starts_with("loop") {
        // Still check children
        if let Some(children_pos) = obj.find("\"children\"") {
            parse_device_array(&obj[children_pos..], devices, removable);
        }
        return;
    }

    // Add this device if it has a filesystem or is a disk
    if !fstype.is_empty() || dev_type == "disk" {
        devices.push(BlockDevice {
            name: name.clone(),
            label,
            size,
            fstype,
            mountpoint,
            model,
            removable,
            dev_type: dev_type.clone(),
        });
    }

    // Recurse into children
    if let Some(children_pos) = obj.find("\"children\"") {
        parse_device_array(&obj[children_pos..], devices, removable);
    }
}

fn extract_json_string(obj: &str, key: &str) -> String {
    let pattern = format!("\"{}\"", key);
    let Some(pos) = obj.find(&pattern) else {
        return String::new();
    };
    let after = &obj[pos + pattern.len()..];
    // Skip whitespace and colon
    let after = after.trim_start();
    let Some(after) = after.strip_prefix(':') else {
        return String::new();
    };
    let after = after.trim_start();

    if after.starts_with("null") {
        return String::new();
    }

    if let Some(rest) = after.strip_prefix('"') {
        // String value — find closing quote (handle escaped quotes)
        let mut escaped = false;
        let mut end = 0;
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
                end = i;
                break;
            }
        }
        rest[..end].to_string()
    } else {
        // Numeric or boolean — read until comma/brace
        let end = after
            .find(|c: char| c == ',' || c == '}' || c == ']')
            .unwrap_or(after.len());
        after[..end].trim().to_string()
    }
}

// ── Actions ────────────────────────────────────────────────

pub fn mount_device(device_name: &str) -> Result<String, String> {
    // Use udisksctl for unprivileged mount
    let dev_path = format!("/dev/{}", device_name);
    let output = Command::new("udisksctl")
        .args(["mount", "-b", &dev_path])
        .output()
        .map_err(|e| format!("udisksctl mount failed: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().to_string());
    }
    Ok(stdout.trim().to_string())
}

pub fn unmount_device(device_name: &str) -> Result<(), String> {
    let dev_path = format!("/dev/{}", device_name);
    let output = Command::new("udisksctl")
        .args(["unmount", "-b", &dev_path])
        .output()
        .map_err(|e| format!("udisksctl unmount failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().to_string());
    }
    Ok(())
}

pub fn eject_device(device_name: &str) -> Result<(), String> {
    // Strip partition number to get parent disk for eject
    let disk_name = device_name
        .trim_end_matches(|c: char| c.is_ascii_digit())
        .to_string();
    let dev_path = format!("/dev/{}", if disk_name.is_empty() { device_name } else { &disk_name });
    let output = Command::new("udisksctl")
        .args(["power-off", "-b", &dev_path])
        .output()
        .map_err(|e| format!("udisksctl eject failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().to_string());
    }
    Ok(())
}

/// Check if udisksctl is available on the system.
pub fn udisksctl_available() -> bool {
    Command::new("udisksctl")
        .arg("help")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
