use super::*;
use std::sync::Mutex;

static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_event_log_toggle_true() {
    set_event_log_enabled(true);
    assert!(is_event_log_enabled());
}

#[test]
fn test_event_log_toggle_false() {
    set_event_log_enabled(false);
    assert!(!is_event_log_enabled());
}

#[test]
fn test_get_log_app_name_default() {
    let name = get_log_app_name();
    assert!(!name.is_empty());
}

#[test]
fn test_set_log_app_name_signature() {
    // Calling it multiple times is safe (OnceLock will just ignore subsequent sets)
    set_log_app_name("pulse_test_app");
    let name = get_log_app_name();
    assert!(!name.is_empty());
}

#[test]
fn test_log_message_does_not_panic() {
    // Calling log_message should not panic under any circumstance.
    log_message("DEBUG", "Testing silent logger implementation");
}

#[test]
fn test_get_appdata_log_path_with_xdg_data_home() {
    let _guard = ENV_MUTEX.lock().unwrap();
    
    // Save original environment
    let orig_xdg = std::env::var("XDG_DATA_HOME").ok();
    let orig_home = std::env::var("HOME").ok();
    let orig_appdata = std::env::var("APPDATA").ok();

    // Set mock XDG_DATA_HOME
    unsafe {
        std::env::set_var("XDG_DATA_HOME", "/tmp/mock_xdg_data_home");
        // Also set APPDATA on Windows to keep cross-platform behavior consistent if windows
        std::env::set_var("APPDATA", "/tmp/mock_appdata");
    }

    let path = get_appdata_log_path();
    assert!(path.is_some());
    let path_str = path.unwrap().to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    assert!(path_str.contains("mock_appdata"));
    #[cfg(not(target_os = "windows"))]
    assert!(path_str.contains("mock_xdg_data_home"));

    // Restore environment
    unsafe {
        if let Some(val) = orig_xdg { std::env::set_var("XDG_DATA_HOME", val); } else { std::env::remove_var("XDG_DATA_HOME"); }
        if let Some(val) = orig_home { std::env::set_var("HOME", val); } else { std::env::remove_var("HOME"); }
        if let Some(val) = orig_appdata { std::env::set_var("APPDATA", val); } else { std::env::remove_var("APPDATA"); }
    }
}

#[test]
fn test_get_appdata_log_path_with_home_fallback() {
    let _guard = ENV_MUTEX.lock().unwrap();
    
    // Save original environment
    let orig_xdg = std::env::var("XDG_DATA_HOME").ok();
    let orig_home = std::env::var("HOME").ok();
    let orig_appdata = std::env::var("APPDATA").ok();

    // Clear XDG_DATA_HOME and set HOME
    unsafe {
        std::env::remove_var("XDG_DATA_HOME");
        std::env::set_var("HOME", "/tmp/mock_home");
        // Also set APPDATA on Windows
        std::env::set_var("APPDATA", "/tmp/mock_appdata");
    }

    let path = get_appdata_log_path();
    assert!(path.is_some());
    let path_str = path.unwrap().to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    assert!(path_str.contains("mock_appdata"));
    #[cfg(not(target_os = "windows"))]
    {
        assert!(path_str.contains("mock_home"));
        assert!(path_str.contains(".local/share"));
    }

    // Restore environment
    unsafe {
        if let Some(val) = orig_xdg { std::env::set_var("XDG_DATA_HOME", val); } else { std::env::remove_var("XDG_DATA_HOME"); }
        if let Some(val) = orig_home { std::env::set_var("HOME", val); } else { std::env::remove_var("HOME"); }
        if let Some(val) = orig_appdata { std::env::set_var("APPDATA", val); } else { std::env::remove_var("APPDATA"); }
    }
}

#[test]
fn test_get_appdata_log_path_none_when_empty() {
    let _guard = ENV_MUTEX.lock().unwrap();
    
    // Save original environment
    let orig_xdg = std::env::var("XDG_DATA_HOME").ok();
    let orig_home = std::env::var("HOME").ok();
    let orig_appdata = std::env::var("APPDATA").ok();

    // Clear all
    unsafe {
        std::env::remove_var("XDG_DATA_HOME");
        std::env::remove_var("HOME");
        std::env::remove_var("APPDATA");
    }

    let path = get_appdata_log_path();
    assert!(path.is_none());

    // Restore environment
    unsafe {
        if let Some(val) = orig_xdg { std::env::set_var("XDG_DATA_HOME", val); } else { std::env::remove_var("XDG_DATA_HOME"); }
        if let Some(val) = orig_home { std::env::set_var("HOME", val); } else { std::env::remove_var("HOME"); }
        if let Some(val) = orig_appdata { std::env::set_var("APPDATA", val); } else { std::env::remove_var("APPDATA"); }
    }
}
