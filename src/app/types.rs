//! Common application-specific type definitions for pulse.

use ratatui::style::Color;
use crate::ui::theme::ThemeColors;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FocusedSection {
    Cpu,
    Memory,
    Disk,
    Gpu,
    Network,
}

#[derive(Debug, Clone)]
pub struct ProcessItem {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub mem: u64,
    pub disk_read: u64,
    pub disk_write: u64,
    pub gpu: f32,
    pub net: u64,
}

#[derive(Debug, Clone)]
pub struct ProcessDetails {
    pub pid: u32,
    pub name: String,
    pub parent_pid: Option<u32>,
    pub exe: String,
    pub cmdline: String,
    pub status: String,
    pub cpu: f32,
    pub mem: u64,
    pub run_time: u64,
}

/// pulse extends ThemeColors with a highlight_bg slot
/// for the process table row highlight color.
#[derive(Debug, Clone, Copy)]
pub struct AppTheme {
    pub base: ThemeColors,
    pub highlight_bg: Color,
}

impl AppTheme {
    /// Build from an already-constructed `ThemeColors`.
    pub fn from_base(
        base: ThemeColors,
        dark_highlight: Color,
        light_highlight: Color,
    ) -> Self {
        let dark = match base.text_main {
            Color::Rgb(r, _, _) => r > 128,
            _ => true,
        };
        Self {
            base,
            highlight_bg: if dark { dark_highlight } else { light_highlight },
        }
    }
}

impl std::ops::Deref for AppTheme {
    type Target = ThemeColors;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
