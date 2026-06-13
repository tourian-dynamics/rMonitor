use super::*;
use std::sync::Mutex;
use std::fs;

static CONFIG_ENV_MUTEX: Mutex<()> = Mutex::new(());

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct MockFields {
    pub host: String,
    pub port: u16,
    pub debug: bool,
}

impl ConfigFields for MockFields {
    fn parse_field(&mut self, key: &str, val: &str) {
        match key {
            "host" => self.host = val.to_string(),
            "port" => {
                if let Ok(p) = val.parse::<u16>() {
                    self.port = p;
                }
            }
            "debug" => self.debug = val == "true",
            _ => {}
        }
    }

    fn serialize_fields(&self) -> Vec<(String, String)> {
        vec![
            ("host".to_string(), self.host.clone()),
            ("port".to_string(), self.port.to_string()),
            ("debug".to_string(), self.debug.to_string()),
        ]
    }
}

fn get_test_dir() -> std::path::PathBuf {
    let dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("target")
        .join("test_config_outputs");
    let _ = fs::create_dir_all(&dir);
    dir
}

#[test]
fn test_config_path_resolves() {
    let _guard = CONFIG_ENV_MUTEX.lock().unwrap();

    let orig_xdg = std::env::var("XDG_CONFIG_HOME").ok();
    let orig_home = std::env::var("HOME").ok();
    let orig_appdata = std::env::var("APPDATA").ok();

    unsafe {
        std::env::set_var("XDG_CONFIG_HOME", "/tmp/mock_config");
        std::env::set_var("APPDATA", "/tmp/mock_appdata");
    }

    let path = AppConfig::<MockFields>::config_path("testapp", "config.yaml");
    assert!(path.is_some());
    let path_str = path.unwrap().to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    assert!(path_str.contains("mock_appdata"));
    #[cfg(not(target_os = "windows"))]
    assert!(path_str.contains("mock_config"));

    unsafe {
        if let Some(val) = orig_xdg { std::env::set_var("XDG_CONFIG_HOME", val); } else { std::env::remove_var("XDG_CONFIG_HOME"); }
        if let Some(val) = orig_home { std::env::set_var("HOME", val); } else { std::env::remove_var("HOME"); }
        if let Some(val) = orig_appdata { std::env::set_var("APPDATA", val); } else { std::env::remove_var("APPDATA"); }
    }
}

#[test]
fn test_config_path_fallback_home() {
    let _guard = CONFIG_ENV_MUTEX.lock().unwrap();

    let orig_xdg = std::env::var("XDG_CONFIG_HOME").ok();
    let orig_home = std::env::var("HOME").ok();
    let orig_appdata = std::env::var("APPDATA").ok();

    unsafe {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::set_var("HOME", "/tmp/mock_home");
        std::env::set_var("APPDATA", "/tmp/mock_appdata");
    }

    let path = AppConfig::<MockFields>::config_path("testapp", "config.yaml");
    assert!(path.is_some());
    let path_str = path.unwrap().to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    assert!(path_str.contains("mock_appdata"));
    #[cfg(not(target_os = "windows"))]
    {
        assert!(path_str.contains("mock_home"));
        assert!(path_str.contains(".config"));
    }

    unsafe {
        if let Some(val) = orig_xdg { std::env::set_var("XDG_CONFIG_HOME", val); } else { std::env::remove_var("XDG_CONFIG_HOME"); }
        if let Some(val) = orig_home { std::env::set_var("HOME", val); } else { std::env::remove_var("HOME"); }
        if let Some(val) = orig_appdata { std::env::set_var("APPDATA", val); } else { std::env::remove_var("APPDATA"); }
    }
}

#[test]
fn test_write_file_atomic_success() {
    let test_dir = get_test_dir();
    let file_path = test_dir.join("atomic_test.txt");
    
    let content = b"atomic content test";
    let res = write_file_atomic(&file_path, content);
    assert!(res.is_ok());

    let read_back = fs::read(&file_path).unwrap();
    assert_eq!(read_back, content);

    // Clean up
    let _ = fs::remove_file(file_path);
}

#[test]
fn test_write_file_atomic_invalid_dir() {
    // A path inside a non-existent parent directory should fail since write_file_atomic
    // does not create parent directories, it only writes to path.parent().
    let test_dir = get_test_dir().join("nonexistent_parent_dir_1234");
    let file_path = test_dir.join("atomic_test.txt");
    
    let content = b"should fail";
    let res = write_file_atomic(&file_path, content);
    assert!(res.is_err());
}

#[test]
fn test_generic_app_config_load_nonexistent() {
    let _guard = CONFIG_ENV_MUTEX.lock().unwrap();
    let orig_xdg = std::env::var("XDG_CONFIG_HOME").ok();

    unsafe {
        // Point config path to a unique non-existent path
        std::env::set_var("XDG_CONFIG_HOME", "/tmp/nonexistent_config_dir_path");
    }

    let loaded = AppConfig::<MockFields>::load("testapp", "config.yaml");
    // Should fallback to default MockFields
    assert_eq!(loaded.fields, MockFields::default());

    unsafe {
        if let Some(val) = orig_xdg { std::env::set_var("XDG_CONFIG_HOME", val); } else { std::env::remove_var("XDG_CONFIG_HOME"); }
    }
}

#[test]
fn test_generic_app_config_load_valid() {
    let _guard = CONFIG_ENV_MUTEX.lock().unwrap();
    let orig_xdg = std::env::var("XDG_CONFIG_HOME").ok();

    let test_dir = get_test_dir();
    unsafe {
        std::env::set_var("XDG_CONFIG_HOME", test_dir.to_str().unwrap());
    }

    let app_dir = test_dir.join("local76").join("testapp");
    let _ = fs::create_dir_all(&app_dir);
    let config_file = app_dir.join("config.yaml");

    let yaml_content = "host: 127.0.0.1\nport: 8080\ndebug: true\n";
    fs::write(&config_file, yaml_content).unwrap();

    let loaded = AppConfig::<MockFields>::load("testapp", "config.yaml");
    assert_eq!(loaded.fields.host, "127.0.0.1");
    assert_eq!(loaded.fields.port, 8080);
    assert!(loaded.fields.debug);

    // Clean up
    let _ = fs::remove_file(config_file);
    unsafe {
        if let Some(val) = orig_xdg { std::env::set_var("XDG_CONFIG_HOME", val); } else { std::env::remove_var("XDG_CONFIG_HOME"); }
    }
}

#[test]
fn test_generic_app_config_load_comments_and_spaces() {
    let _guard = CONFIG_ENV_MUTEX.lock().unwrap();
    let orig_xdg = std::env::var("XDG_CONFIG_HOME").ok();

    let test_dir = get_test_dir();
    unsafe {
        std::env::set_var("XDG_CONFIG_HOME", test_dir.to_str().unwrap());
    }

    let app_dir = test_dir.join("local76").join("testapp");
    let _ = fs::create_dir_all(&app_dir);
    let config_file = app_dir.join("config.yaml");

    let yaml_content = r#"
# This is a comment
host: localhost

  # Another comment
port: 9000
#debug: false
debug: false
"#;
    fs::write(&config_file, yaml_content).unwrap();

    let loaded = AppConfig::<MockFields>::load("testapp", "config.yaml");
    assert_eq!(loaded.fields.host, "localhost");
    assert_eq!(loaded.fields.port, 9000);
    assert!(!loaded.fields.debug);

    // Clean up
    let _ = fs::remove_file(config_file);
    unsafe {
        if let Some(val) = orig_xdg { std::env::set_var("XDG_CONFIG_HOME", val); } else { std::env::remove_var("XDG_CONFIG_HOME"); }
    }
}

#[test]
fn test_generic_app_config_save() {
    let _guard = CONFIG_ENV_MUTEX.lock().unwrap();
    let orig_xdg = std::env::var("XDG_CONFIG_HOME").ok();

    let test_dir = get_test_dir();
    unsafe {
        std::env::set_var("XDG_CONFIG_HOME", test_dir.to_str().unwrap());
    }

    let app_dir = test_dir.join("local76").join("testapp");
    let _ = fs::create_dir_all(&app_dir);
    let config_file = app_dir.join("config.yaml");
    let _ = fs::remove_file(&config_file); // Ensure fresh

    let config = AppConfig {
        fields: MockFields {
            host: "saved-host".to_string(),
            port: 3000,
            debug: true,
        }
    };

    let save_res = config.save("testapp", "config.yaml", "Header line 1\n# Header line 2");
    assert!(save_res.is_ok());

    assert!(config_file.exists());
    let saved_content = fs::read_to_string(&config_file).unwrap();

    // Check header was commented properly
    assert!(saved_content.contains("# Header line 1"));
    assert!(saved_content.contains("# Header line 2"));

    // Check properties
    assert!(saved_content.contains("host: saved-host"));
    assert!(saved_content.contains("port: 3000"));
    assert!(saved_content.contains("debug: true"));

    // Clean up
    let _ = fs::remove_file(config_file);
    unsafe {
        if let Some(val) = orig_xdg { std::env::set_var("XDG_CONFIG_HOME", val); } else { std::env::remove_var("XDG_CONFIG_HOME"); }
    }
}
