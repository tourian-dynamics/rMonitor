use super::*;
use std::time::Duration;

#[test]
fn test_power_status_default() {
    let p = PowerStatus::default();
    assert!(p.ac_online);
    assert_eq!(p.battery_percent, 100);
}

#[test]
fn test_power_status_is_battery_percent_unknown() {
    let p_known = PowerStatus {
        ac_online: true,
        battery_percent: 50,
    };
    assert!(!p_known.is_battery_percent_unknown());

    let p_unknown = PowerStatus {
        ac_online: false,
        battery_percent: PowerStatus::BATTERY_PERCENT_UNKNOWN,
    };
    assert!(p_unknown.is_battery_percent_unknown());
}

#[test]
fn test_system_bios_info_default() {
    let bios = SystemBiosInfo::default();
    assert_eq!(bios.manufacturer, "");
    assert_eq!(bios.product, "");
    assert_eq!(bios.model, "");
}

#[test]
fn test_cached_new() {
    let duration = Duration::from_secs(5);
    let cached = Cached::new(42, duration);
    assert_eq!(cached.value, 42);
    assert_eq!(cached.duration, duration);
    // last_updated should be very close to now
    assert!(cached.last_updated.elapsed() < Duration::from_secs(1));
}

#[test]
fn test_cached_is_valid_fresh() {
    let cached = Cached::new("test".to_string(), Duration::from_secs(10));
    assert!(cached.is_valid());
}

#[test]
fn test_cached_is_valid_expired() {
    let cached = Cached::new(100, Duration::ZERO);
    // Since duration is ZERO, it should not be valid (or becomes invalid instantly)
    assert!(!cached.is_valid());
}

#[test]
fn test_cached_is_valid_expire_after_sleep() {
    let cached = Cached::new(true, Duration::from_millis(2));
    std::thread::sleep(Duration::from_millis(5));
    assert!(!cached.is_valid());
}

#[test]
fn test_query_local_ip_signature() {
    let res = query_local_ip();
    if let Some(ip_str) = res {
        let parsed: Result<std::net::IpAddr, _> = ip_str.parse();
        assert!(parsed.is_ok(), "Returned IP string was not a valid IP address");
    }
}

#[test]
fn test_glyph_map_load() {
    let map = GlyphMap::load();
    assert!(!map.status_ok.is_empty());
    assert!(!map.status_err.is_empty());
    assert!(!map.cpu.is_empty());
    assert!(!map.play.is_empty());
}

#[cfg(not(target_os = "windows"))]
#[test]
fn test_fallback_impl_values() {
    assert_eq!(query_accent_color(), (0, 245, 255));
    assert_eq!(get_win_accent_color_hex(), "#00F5FF");
    assert!(!query_high_contrast());
    assert_eq!(query_os_version(), "Mock OS");
    assert!(query_dark_mode());
    
    let power = query_power_status();
    assert!(power.is_some());
    let p = power.unwrap();
    assert!(p.ac_online);
    assert_eq!(p.battery_percent, 100);

    assert!(query_bios_info().is_none());
    assert_eq!(query_gpu_names(), vec!["Mock GPU".to_string()]);
    assert_eq!(get_local_time_string(), "2026-06-06 12:00:00");
}
