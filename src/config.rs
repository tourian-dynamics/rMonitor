//! rmonitor application configuration — now backed by rCommon's
//! `AppConfig<T>`.
//!
//! **Taxonomy Classification**: Platform & Architecture (Deployment - Native).

#![allow(dead_code)]

use std::io;

use library::platform::native::config::{AppConfig as GenericAppConfig, ConfigFields};

pub const APP_NAME: &str = "rmonitor";
pub const CONFIG_FILE: &str = "config.yaml";
pub const CONFIG_HEADER: &str = "rMonitor Local Configuration\n-----------------------------";

/// Concrete configuration fields for rmonitor.
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
