//! Identity helpers (username, hostname, OS string) for banner rendering.

use crate::backend::sys_info::query_os_version;

/// Returns the current OS username (Windows: `$USERNAME`, POSIX: `$USER`).
pub fn username() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "user".to_string())
}

/// Returns the current hostname (Windows: `$COMPUTERNAME`, POSIX: `$HOSTNAME`).
pub fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "localhost".to_string())
}

/// Returns `"{username}@{hostname}"` formatted for the title banner.
pub fn user_host() -> String {
    format!("{}@{}", username(), hostname())
}

/// Returns the OS version string (delegates to cached query).
pub fn os_str() -> String {
    query_os_version()
}

/// Returns the user's default shell (Windows: PowerShell v7.4 if `$PSModulePath`
/// is set, else `cmd.exe`; POSIX: `$SHELL` env var, default `/bin/bash`).
pub fn shell_name() -> String {
    if cfg!(target_os = "windows") {
        if std::env::var("PSModulePath").is_ok() {
            "PowerShell v7.4".to_string()
        } else {
            "cmd.exe".to_string()
        }
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
    }
}
