//! pulse: netsh-based network interface status lookup.
//!
//! Kept app-specific (library does not wrap `netsh`).

#[cfg(windows)]
pub fn get_network_statuses() -> std::collections::HashMap<String, String> {
    let mut statuses = std::collections::HashMap::new();
    use std::process::Command;
    if let Ok(output) = Command::new("netsh")
        .args(&["interface", "show", "interface"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("Admin State") || line.starts_with("---") {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let state = parts[1].to_string();
                let type_str = parts[2];
                if let Some(pos) = line.find(type_str) {
                    let name = line[pos + type_str.len()..].trim().to_string();
                    statuses.insert(name, state);
                }
            }
        }
    }
    statuses
}

#[cfg(not(windows))]
pub fn get_network_statuses() -> std::collections::HashMap<String, String> {
    std::collections::HashMap::new()
}
