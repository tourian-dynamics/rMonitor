//! Generic file-based diagnostics logging.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use std::sync::atomic::{AtomicBool, Ordering};

static LOG_FILE: OnceLock<Mutex<Option<File>>> = OnceLock::new();
static LOG_APP_NAME: OnceLock<String> = OnceLock::new();
static EVENT_LOG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Set the per-app log file folder name (e.g. `"helm"`).
pub fn set_log_app_name(name: &str) {
    let _ = LOG_APP_NAME.set(name.to_string());
}

/// Enable or disable event logging syncing. Stubbed for self-contained helm.
pub fn set_event_log_enabled(enabled: bool) {
    EVENT_LOG_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Check if event logging syncing is enabled. Stubbed for self-contained helm.
pub fn is_event_log_enabled() -> bool {
    EVENT_LOG_ENABLED.load(Ordering::Relaxed)
}

fn get_log_app_name() -> &'static str {
    LOG_APP_NAME.get_or_init(|| "helm".to_string())
}

/// Helper to resolve the standard AppData folder for diagnostics logging.
pub fn get_appdata_log_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(|appdata| {
            std::path::PathBuf::from(appdata)
                .join("local76")
                .join(get_log_app_name())
                .join("log.txt")
        })
    }
    #[cfg(not(target_os = "windows"))]
    {
        let base = std::env::var("XDG_DATA_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME").ok().map(|home| {
                    PathBuf::from(home).join(".local").join("share")
                })
            });
        base.map(|b| b.join("local76").join(get_log_app_name()).join("log.txt"))
    }
}

fn get_log_file() -> &'static Mutex<Option<File>> {
    LOG_FILE.get_or_init(|| {
        let file_opt = get_appdata_log_path().and_then(|path| {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .ok()
        });
        Mutex::new(file_opt)
    })
}

/// Thread-safe silent logger helper that appends diagnostic logs to a local file.
pub fn log_message(level: &str, msg: &str) {
    let mutex = get_log_file();
    if let Ok(mut guard) = mutex.lock() {
        if let Some(ref mut file) = *guard {
            let epoch = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let _ = writeln!(file, "[{}] [{}] {}", epoch, level, msg);
        }
    }
}

#[cfg(test)]
#[path = "logger_tests.rs"]
mod tests;

