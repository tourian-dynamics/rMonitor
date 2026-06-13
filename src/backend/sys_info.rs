//! Host system information and theme querying utilities.

pub use crate::backend::shell_terminal::query_shell_and_terminal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerStatus {
    pub ac_online: bool,
    pub battery_percent: u8,
}

impl Default for PowerStatus {
    fn default() -> Self {
        Self {
            ac_online: true,
            battery_percent: 100,
        }
    }
}

impl PowerStatus {
    pub const BATTERY_PERCENT_UNKNOWN: u8 = 255;

    pub fn is_battery_percent_unknown(&self) -> bool {
        self.battery_percent == Self::BATTERY_PERCENT_UNKNOWN
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SystemBiosInfo {
    pub manufacturer: String,
    pub product: String,
    pub model: String,
}

/// Helper structure for caching query results
struct Cached<T> {
    last_updated: std::time::Instant,
    value: T,
    duration: std::time::Duration,
}

impl<T> Cached<T> {
    fn new(value: T, duration: std::time::Duration) -> Self {
        Self {
            last_updated: std::time::Instant::now(),
            value,
            duration,
        }
    }

    fn is_valid(&self) -> bool {
        self.last_updated.elapsed() < self.duration
    }
}

// Cross-platform query_local_ip
pub fn query_local_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

#[path = "sys_info_win.rs"]
#[cfg(target_os = "windows")]
mod win_impl;

#[cfg(target_os = "windows")]
pub use win_impl::*;

#[cfg(not(target_os = "windows"))]
mod fallback_impl {
    use super::{PowerStatus, SystemBiosInfo};

    pub fn query_accent_color() -> (u8, u8, u8) { (0, 245, 255) }
    pub fn get_win_accent_color_hex() -> String { "#00F5FF".to_string() }
    pub fn query_high_contrast() -> bool { false }
    pub fn query_os_version() -> String { "Mock OS".to_string() }
    pub fn query_dark_mode() -> bool { true }
    pub fn query_power_status() -> Option<PowerStatus> {
        Some(PowerStatus {
            ac_online: true,
            battery_percent: 100,
        })
    }
    pub fn query_bios_info() -> Option<SystemBiosInfo> { None }
    pub fn query_gpu_names() -> Vec<String> { vec!["Mock GPU".to_string()] }
    pub fn get_local_time_string() -> String {
        "2026-06-06 12:00:00".to_string()
    }
}

#[cfg(not(target_os = "windows"))]
pub use fallback_impl::*;

#[derive(Debug, Clone, Copy)]
pub struct GlyphMap {
    pub status_ok: &'static str,
    pub status_err: &'static str,
    pub info: &'static str,
    pub warning: &'static str,
    pub cpu: &'static str,
    pub gpu: &'static str,
    pub memory: &'static str,
    pub disk: &'static str,
    pub package: &'static str,
    pub battery: &'static str,
    pub shell: &'static str,
    pub terminal: &'static str,
    pub network: &'static str,
    pub clipboard: &'static str,
    pub play: &'static str,
    pub play_empty: &'static str,
}

impl GlyphMap {
    pub fn load() -> Self {
        let (_, terminal) = query_shell_and_terminal();
        if terminal == "Windows Console Host" {
            Self {
                status_ok: "[OK]",
                status_err: "[ERR]",
                info: "[i]",
                warning: "[!]",
                cpu: "[CPU]",
                gpu: "[GPU]",
                memory: "[RAM]",
                disk: "[DISK]",
                package: "[PKG]",
                battery: "[BAT]",
                shell: "[SH]",
                terminal: "[TERM]",
                network: "[NET]",
                clipboard: "[CLIP]",
                play: "> ",
                play_empty: "  ",
            }
        } else {
            Self {
                status_ok: "✔️",
                status_err: "❌",
                info: "ℹ️",
                warning: "⚠️",
                cpu: "🧠",
                gpu: "🎮",
                memory: "📟",
                disk: "💾",
                package: "📦",
                battery: "🔋",
                shell: "🐚",
                terminal: "📟",
                network: "🌐",
                clipboard: "📋",
                play: "▶ ",
                play_empty: "  ",
            }
        }
    }
}

#[cfg(test)]
#[path = "sys_info_tests.rs"]
mod tests;

