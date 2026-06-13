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

#[test]
fn test_config_parse_invalid_key() {
    let mut cfg = AppConfig::default();
    let original = cfg.clone();
    
    cfg.parse_field("invalid_field_name", "some_value");
    
    // Config should be completely unchanged
    assert_eq!(cfg.theme_mode, original.theme_mode);
    assert_eq!(cfg.refresh_rate_ms, original.refresh_rate_ms);
    assert_eq!(cfg.enable_borderless, original.enable_borderless);
    assert_eq!(cfg.enable_toasts, original.enable_toasts);
    assert_eq!(cfg.enable_event_log, original.enable_event_log);
}

#[test]
fn test_config_parse_invalid_refresh_rate() {
    let mut cfg = AppConfig::default();
    cfg.parse_field("refresh_rate_ms", "not_a_number");
    
    // Refresh rate should retain its default (100) instead of being overridden or causing a panic
    assert_eq!(cfg.refresh_rate_ms, 100);
}

#[test]
fn test_config_path_structure() {
    let path = AppConfig::config_path();
    assert!(path.is_some());
    let path_str = path.unwrap().to_string_lossy().to_string();
    assert!(path_str.contains("config.yaml"));
    assert!(path_str.contains("app/pulse") || path_str.contains("app\\pulse"));
}

#[test]
fn test_config_serialize_length() {
    let cfg = AppConfig::default();
    let fields = cfg.serialize_fields();
    assert_eq!(fields.len(), 5);
}
