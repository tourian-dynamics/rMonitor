//! Generic configuration parser and writer for yaml-like key-value storage.

use std::io;
use std::path::PathBuf;

/// Trait to be implemented by application-specific configuration field structures.
pub trait ConfigFields: Default {
    /// Update field matching `key` with value `val`.
    fn parse_field(&mut self, key: &str, val: &str);
    /// Serialize fields into a list of key-value string pairs.
    fn serialize_fields(&self) -> Vec<(String, String)>;
}

/// Generic application configuration wrapper.
#[derive(Debug, Clone)]
pub struct AppConfig<T: ConfigFields> {
    pub fields: T,
}

impl<T: ConfigFields> AppConfig<T> {
    /// Resolves path to `%APPDATA%\<app_name>\<filename>`
    pub fn config_path(app_name: &str, filename: &str) -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            std::env::var("APPDATA").ok().map(|appdata| {
                std::path::PathBuf::from(appdata)
                    .join("local76")
                    .join(app_name)
                    .join(filename)
            })
        }
        #[cfg(not(target_os = "windows"))]
        {
            let base = std::env::var("XDG_CONFIG_HOME")
                .ok()
                .map(PathBuf::from)
                .or_else(|| {
                    std::env::var("HOME").ok().map(|home| {
                        PathBuf::from(home).join(".config")
                    })
                });
            base.map(|b| b.join("local76").join(app_name).join(filename))
        }
    }

    /// Load config from file, falling back to defaults on failure or missing file.
    pub fn load(app_name: &str, filename: &str) -> Self {
        let Some(path) = Self::config_path(app_name, filename) else {
            return Self { fields: T::default() };
        };
        let Ok(content) = std::fs::read_to_string(&path) else {
            return Self { fields: T::default() };
        };

        let mut fields = T::default();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(pos) = line.find(':') {
                let key = line[..pos].trim();
                let val = line[pos + 1..].trim();
                fields.parse_field(key, val);
            }
        }
        Self { fields }
    }

    /// Save current config properties to file.
    pub fn save(&self, app_name: &str, filename: &str, header: &str) -> io::Result<()> {
        let Some(path) = Self::config_path(app_name, filename) else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut content = String::new();
        if !header.is_empty() {
            for line in header.lines() {
                let line = line.trim();
                if line.starts_with('#') {
                    content.push_str(line);
                } else {
                    content.push_str(&format!("# {}", line));
                }
                content.push('\n');
            }
            content.push('\n');
        }

        for (k, v) in self.fields.serialize_fields() {
            content.push_str(&format!("{}: {}\n", k, v));
        }

        write_file_atomic(path, content)
    }
}

/// Write to a file atomically by writing to a temporary file first and renaming it.
pub fn write_file_atomic<P: AsRef<std::path::Path>, C: AsRef<[u8]>>(path: P, content: C) -> std::io::Result<()> {
    let path = path.as_ref();
    let temp_name = format!(
        ".tmp_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let temp_path = path.parent()
        .map(|p| p.join(&temp_name))
        .unwrap_or_else(|| std::path::PathBuf::from(temp_name));

    std::fs::write(&temp_path, content)?;
    if let Err(e) = std::fs::rename(&temp_path, path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(e);
    }
    Ok(())
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;

