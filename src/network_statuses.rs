//! pulse: netsh-based network interface status lookup.
//!
//! Kept app-specific (library does not wrap `netsh`).

use std::collections::HashMap;

/// Parses netsh interface stdout output into a map of Name -> State
pub fn parse_netsh_output(stdout: &str) -> HashMap<String, String> {
    let mut statuses = HashMap::new();
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
    statuses
}

#[cfg(windows)]
pub fn get_network_statuses() -> HashMap<String, String> {
    use std::process::Command;
    if let Ok(output) = Command::new("netsh")
        .args(["interface", "show", "interface"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_netsh_output(&stdout)
    } else {
        HashMap::new()
    }
}

#[cfg(not(windows))]
pub fn get_network_statuses() -> HashMap<String, String> {
    HashMap::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_netsh_output() {
        let sample = "\
Admin State    State          Type             Interface Name
-------------------------------------------------------------------------
Enabled        Connected      Dedicated        Wi-Fi
Disabled       Disconnected   Dedicated        Ethernet 2
";
        let parsed = parse_netsh_output(sample);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed.get("Wi-Fi").map(|s| s.as_str()), Some("Connected"));
        assert_eq!(parsed.get("Ethernet 2").map(|s| s.as_str()), Some("Disconnected"));
    }
}
