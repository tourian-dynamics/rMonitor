//! Power-status and theme-status throttled poller for render loops.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Foreground) + Platform (Native).

use std::time::{Duration, Instant};
use crate::backend::sys_info;

/// How often to re-query power status (5 s).
pub const POWER_CHECK_INTERVAL: Duration = Duration::from_millis(5000);
/// How often to re-query DWM dark-mode + accent color (2.5 s).
pub const THEME_CHECK_INTERVAL: Duration = Duration::from_millis(2500);
/// Multiplier applied to the app's `tick_rate` when on battery.
pub const BATTERY_THROTTLE_MULTIPLIER: u32 = 2;

/// State for the power-and-theme throttled poller.
#[derive(Debug, Clone)]
pub struct PowerThrottle {
    last_power_check: Instant,
    last_theme_check: Instant,
    pub on_battery: bool,
    pub dark_mode: bool,
    pub accent_color: (u8, u8, u8),
    last_power_state: bool,
    last_dark: bool,
    last_accent: (u8, u8, u8),
    pub theme_mode: String,
}

impl PowerThrottle {
    /// Create a new throttle using the system `theme_mode` config string
    /// ("auto" / "dark" / "light").
    pub fn new(theme_mode: impl Into<String>) -> Self {
        let power = sys_info::query_power_status();
        let on_battery = power.as_ref().map(|p| !p.ac_online).unwrap_or(false);
        let last_power_state = on_battery;
        let mode = theme_mode.into();
        let dark = resolve_dark(&mode);
        let accent = sys_info::query_accent_color();
        Self {
            last_power_check: Instant::now(),
            last_theme_check: Instant::now(),
            on_battery,
            dark_mode: dark,
            accent_color: accent,
            last_power_state,
            last_dark: dark,
            last_accent: accent,
            theme_mode: mode,
        }
    }

    /// Tick the power poller. Logs (and returns `Some(state_string)`) when the
    /// battery state changes; `None` otherwise.
    pub fn tick_power(&mut self) -> Option<String> {
        if self.last_power_check.elapsed() < POWER_CHECK_INTERVAL {
            return None;
        }
        self.last_power_check = Instant::now();
        let power = sys_info::query_power_status();
        let current = power.as_ref().map(|p| !p.ac_online).unwrap_or(false);
        if current == self.last_power_state {
            return None;
        }
        self.last_power_state = current;
        self.on_battery = current;
        let state = if current {
            "Battery (Power-Saving Throttling Enabled)"
        } else {
            "AC Power (Full Speed)"
        };
        Some(state.to_string())
    }

    /// Tick the theme poller. Returns `true` if dark mode or accent changed.
    pub fn tick_theme(&mut self) -> bool {
        if self.last_theme_check.elapsed() < THEME_CHECK_INTERVAL {
            return false;
        }
        self.last_theme_check = Instant::now();
        let current_dark = resolve_dark(&self.theme_mode);
        let current_accent = sys_info::query_accent_color();
        if current_dark == self.last_dark && current_accent == self.last_accent {
            return false;
        }
        let changed = current_dark != self.dark_mode || current_accent != self.accent_color;
        self.last_dark = current_dark;
        self.last_accent = current_accent;
        self.dark_mode = current_dark;
        self.accent_color = current_accent;
        changed
    }

    /// Tick both power and theme pollers. Returns `(power_msg, theme_changed)`.
    pub fn tick(&mut self) -> (Option<String>, bool) {
        (self.tick_power(), self.tick_theme())
    }

    /// Returns the effective tick rate (doubled when on battery).
    pub fn effective_tick_rate(&self, base: Duration) -> Duration {
        if self.on_battery {
            base * BATTERY_THROTTLE_MULTIPLIER
        } else {
            base
        }
    }

    /// Returns the current dark-mode boolean (respects `theme_mode` override).
    pub fn is_dark(&self) -> bool {
        self.dark_mode
    }

    /// Returns the current accent color as an (R, G, B) tuple.
    pub fn accent(&self) -> (u8, u8, u8) {
        self.accent_color
    }
}

fn resolve_dark(theme_mode: &str) -> bool {
    match theme_mode {
        "dark" => true,
        "light" => false,
        _ => sys_info::query_dark_mode(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_throttle_initial() {
        let mut t = PowerThrottle::new("auto");
        // Just constructed; no battery or theme change yet.
        assert!(t.tick_power().is_none());
        assert!(!t.tick_theme());
    }

    #[test]
    fn test_effective_tick_rate() {
        let t = PowerThrottle {
            on_battery: true,
            ..PowerThrottle::new("auto")
        };
        let base = Duration::from_millis(100);
        assert_eq!(t.effective_tick_rate(base), Duration::from_millis(200));

        let t2 = PowerThrottle {
            on_battery: false,
            ..PowerThrottle::new("auto")
        };
        assert_eq!(t2.effective_tick_rate(base), Duration::from_millis(100));
    }
}
