//! pulse application configuration — now backed by library's
//! `AppConfig<T>`.
//!
//! **Taxonomy Classification**: Platform & Architecture (Deployment - Native).

#![allow(dead_code)]

use std::io;

use library::platform::native::config::{AppConfig as GenericAppConfig, ConfigFields};

pub const APP_NAME: &str = "app/pulse";
pub const CONFIG_FILE: &str = "config.yaml";
pub const CONFIG_HEADER: &str = "pulse Local Configuration\n-----------------------------";

/// Concrete configuration fields for pulse.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub theme_mode: String,
    pub refresh_rate_ms: u32,
    pub enable_borderless: bool,
    pub enable_toasts: bool,
    pub enable_event_log: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme_mode: "auto".to_string(),
            refresh_rate_ms: 100,
            enable_borderless: false,
            enable_toasts: true,
            enable_event_log: true,
        }
    }
}

impl ConfigFields for AppConfig {
    fn parse_field(&mut self, key: &str, val: &str) {
        match key {
            "theme_mode" => self.theme_mode = val.to_string(),
            "refresh_rate_ms" => {
                if let Ok(ms) = val.parse::<u32>() {
                    self.refresh_rate_ms = ms;
                }
            }
            "enable_borderless" => self.enable_borderless = val == "true",
            "enable_toasts" => self.enable_toasts = val == "true",
            "enable_event_log" => self.enable_event_log = val == "true",
            _ => {}
        }
    }

    fn serialize_fields(&self) -> Vec<(String, String)> {
        vec![
            ("theme_mode".to_string(), self.theme_mode.clone()),
            ("refresh_rate_ms".to_string(), self.refresh_rate_ms.to_string()),
            ("enable_borderless".to_string(), self.enable_borderless.to_string()),
            ("enable_toasts".to_string(), self.enable_toasts.to_string()),
            ("enable_event_log".to_string(), self.enable_event_log.to_string()),
        ]
    }
}

impl AppConfig {
    /// Load config from disk; on missing/invalid file, write defaults and return them.
    pub fn load() -> Self {
        let existed = Self::config_path().map(|p| p.exists()).unwrap_or(false);
        let cfg = GenericAppConfig::<AppConfig>::load(APP_NAME, CONFIG_FILE);
        if !existed {
            let _ = GenericAppConfig { fields: cfg.fields.clone() }
                .save(APP_NAME, CONFIG_FILE, CONFIG_HEADER);
        }
        cfg.fields
    }

    pub fn config_path() -> Option<std::path::PathBuf> {
        GenericAppConfig::<AppConfig>::config_path(APP_NAME, CONFIG_FILE)
    }

    /// Save the current config to disk.
    #[allow(dead_code)]
    pub fn save(&self) -> io::Result<()> {
        GenericAppConfig { fields: self.clone() }.save(APP_NAME, CONFIG_FILE, CONFIG_HEADER)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.theme_mode, "auto");
        assert_eq!(cfg.refresh_rate_ms, 100);
        assert!(!cfg.enable_borderless);
        assert!(cfg.enable_toasts);
        assert!(cfg.enable_event_log);
    }

    #[test]
    fn test_config_parse_field() {
        let mut cfg = AppConfig::default();
        cfg.parse_field("theme_mode", "light");
        cfg.parse_field("refresh_rate_ms", "250");
        cfg.parse_field("enable_borderless", "true");
        cfg.parse_field("enable_toasts", "false");
        cfg.parse_field("enable_event_log", "false");

        assert_eq!(cfg.theme_mode, "light");
        assert_eq!(cfg.refresh_rate_ms, 250);
        assert!(cfg.enable_borderless);
        assert!(!cfg.enable_toasts);
        assert!(!cfg.enable_event_log);
    }

    #[test]
    fn test_config_serialize() {
        let mut cfg = AppConfig::default();
        cfg.theme_mode = "dark".to_string();
        cfg.refresh_rate_ms = 500;
        cfg.enable_borderless = true;

        let fields = cfg.serialize_fields();
        let theme_field = fields.iter().find(|(k, _)| k == "theme_mode").unwrap();
        let rate_field = fields.iter().find(|(k, _)| k == "refresh_rate_ms").unwrap();
        let borderless_field = fields.iter().find(|(k, _)| k == "enable_borderless").unwrap();

        assert_eq!(theme_field.1, "dark");
        assert_eq!(rate_field.1, "500");
        assert_eq!(borderless_field.1, "true");
    }
}

