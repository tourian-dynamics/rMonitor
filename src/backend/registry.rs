//! Windows registry queries for system properties (brand name, CPU names, etc.).
//! With Unix file-based emulation fallback.

#![allow(dead_code)]
#![allow(unused_imports)]

#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Registry::*;
#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::*;

#[cfg(target_os = "windows")]
pub use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CLASSES_ROOT, HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, KEY_ALL_ACCESS,
};

#[cfg(not(target_os = "windows"))]
pub type HKEY = isize;
#[cfg(not(target_os = "windows"))]
pub const HKEY_CLASSES_ROOT: HKEY = 0;
#[cfg(not(target_os = "windows"))]
pub const HKEY_CURRENT_USER: HKEY = 1;
#[cfg(not(target_os = "windows"))]
pub const HKEY_LOCAL_MACHINE: HKEY = 2;
#[cfg(not(target_os = "windows"))]
pub const HKEY_USERS: HKEY = 3;
#[cfg(not(target_os = "windows"))]
pub const KEY_READ: u32 = 0;
#[cfg(not(target_os = "windows"))]
pub const KEY_WRITE: u32 = 0;
#[cfg(not(target_os = "windows"))]
pub const KEY_ALL_ACCESS: u32 = 0;

pub const REG_SZ: u32 = 1;
pub const REG_DWORD: u32 = 4;
pub const REG_BINARY: u32 = 3;

#[cfg(target_os = "windows")]
fn to_utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(target_os = "windows")]
fn from_utf16(slice: &[u16]) -> String {
    if let Some(pos) = slice.iter().position(|&x| x == 0) {
        String::from_utf16_lossy(&slice[..pos])
    } else {
        String::from_utf16_lossy(slice)
    }
}

#[derive(Debug, Clone)]
pub struct RegValueRaw {
    pub vtype: u32,
    pub bytes: Vec<u8>,
}

pub struct RegKey {
    pub hkey: HKEY,
    pub owned: bool,
}

impl RegKey {
    pub fn predef(hkey: HKEY) -> Self {
        Self { hkey, owned: false }
    }

    pub fn open_subkey(&self, path: &str) -> Result<RegKey, std::io::Error> {
        self.open_subkey_with_flags(path, KEY_READ)
    }

    pub fn open_subkey_with_flags(&self, path: &str, _flags: u32) -> Result<RegKey, std::io::Error> {
        #[cfg(target_os = "windows")]
        unsafe {
            let path_w = to_utf16(path);
            let mut subkey: HKEY = 0;
            let res = RegOpenKeyExW(self.hkey, path_w.as_ptr(), 0, _flags, &mut subkey);
            if res == 0 {
                Ok(RegKey { hkey: subkey, owned: true })
            } else {
                Err(std::io::Error::from_raw_os_error(res as i32))
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (path, _flags);
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Not Windows"))
        }
    }

    pub fn create_subkey(&self, path: &str) -> Result<(RegKey, u32), std::io::Error> {
        #[cfg(target_os = "windows")]
        unsafe {
            let path_w = to_utf16(path);
            let mut subkey: HKEY = 0;
            let mut disp: u32 = 0;
            let res = RegCreateKeyExW(
                self.hkey,
                path_w.as_ptr(),
                0,
                std::ptr::null_mut(),
                0,
                KEY_ALL_ACCESS,
                std::ptr::null_mut(),
                &mut subkey,
                &mut disp
            );
            if res == 0 {
                Ok((RegKey { hkey: subkey, owned: true }, disp))
            } else {
                Err(std::io::Error::from_raw_os_error(res as i32))
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = path;
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Not Windows"))
        }
    }

    pub fn get_value<T: RegValue, _K: AsRef<str>>(&self, key: _K) -> Result<T, std::io::Error> {
        T::read_from_key(self.hkey, key.as_ref())
    }

    pub fn set_value<T: RegValue, _K: AsRef<str>>(&self, key: _K, val: &T) -> Result<(), std::io::Error> {
        val.write_to_key(self.hkey, key.as_ref())
    }

    pub fn get_raw_value(&self, key: &str) -> Result<RegValueRaw, std::io::Error> {
        #[cfg(target_os = "windows")]
        unsafe {
            let key_w = to_utf16(key);
            let mut val_type: u32 = 0;
            let mut buf_size: u32 = 0;
            let mut res = RegQueryValueExW(
                self.hkey,
                key_w.as_ptr(),
                std::ptr::null_mut(),
                &mut val_type,
                std::ptr::null_mut(),
                &mut buf_size
            );
            if res != 0 {
                return Err(std::io::Error::from_raw_os_error(res as i32));
            }
            let mut buf = vec![0u8; buf_size as usize];
            res = RegQueryValueExW(
                self.hkey,
                key_w.as_ptr(),
                std::ptr::null_mut(),
                &mut val_type,
                buf.as_mut_ptr(),
                &mut buf_size
            );
            if res == 0 {
                buf.truncate(buf_size as usize);
                Ok(RegValueRaw { vtype: val_type, bytes: buf })
            } else {
                Err(std::io::Error::from_raw_os_error(res as i32))
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = key;
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Not Windows"))
        }
    }

    pub fn set_raw_value(&self, key: &str, val: &RegValueRaw) -> Result<(), std::io::Error> {
        #[cfg(target_os = "windows")]
        unsafe {
            let key_w = to_utf16(key);
            let res = RegSetValueExW(
                self.hkey,
                key_w.as_ptr(),
                0,
                val.vtype,
                val.bytes.as_ptr(),
                val.bytes.len() as u32
            );
            if res == 0 {
                Ok(())
            } else {
                Err(std::io::Error::from_raw_os_error(res as i32))
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (key, val);
            Ok(())
        }
    }

    pub fn delete_value(&self, key: &str) -> Result<(), std::io::Error> {
        #[cfg(target_os = "windows")]
        unsafe {
            let key_w = to_utf16(key);
            let res = RegDeleteValueW(self.hkey, key_w.as_ptr());
            if res == 0 {
                Ok(())
            } else {
                Err(std::io::Error::from_raw_os_error(res as i32))
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = key;
            Ok(())
        }
    }

    pub fn enum_keys(&self) -> KeyIterator {
        KeyIterator {
            hkey: self.hkey,
            index: 0,
        }
    }

    pub fn enum_values(&self) -> ValueIterator {
        ValueIterator {
            hkey: self.hkey,
            index: 0,
        }
    }
}

impl Drop for RegKey {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        if self.owned && self.hkey != 0 {
            unsafe {
                RegCloseKey(self.hkey);
            }
        }
    }
}

pub trait RegValue: Sized {
    fn read_from_key(hkey: HKEY, key: &str) -> Result<Self, std::io::Error>;
    fn write_to_key(&self, hkey: HKEY, key: &str) -> Result<(), std::io::Error>;
}

impl RegValue for String {
    fn read_from_key(hkey: HKEY, key: &str) -> Result<Self, std::io::Error> {
        #[cfg(target_os = "windows")]
        unsafe {
            let key_w = to_utf16(key);
            let mut val_type: u32 = 0;
            let mut buf_size: u32 = 0;
            let mut res = RegQueryValueExW(
                hkey,
                key_w.as_ptr(),
                std::ptr::null_mut(),
                &mut val_type,
                std::ptr::null_mut(),
                &mut buf_size
            );
            if res != 0 {
                return Err(std::io::Error::from_raw_os_error(res as i32));
            }
            let mut buf = vec![0u16; (buf_size as usize + 1) / 2];
            res = RegQueryValueExW(
                hkey,
                key_w.as_ptr(),
                std::ptr::null_mut(),
                &mut val_type,
                buf.as_mut_ptr() as *mut u8,
                &mut buf_size
            );
            if res == 0 {
                Ok(from_utf16(&buf))
            } else {
                Err(std::io::Error::from_raw_os_error(res as i32))
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (hkey, key);
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Not Windows"))
        }
    }

    fn write_to_key(&self, hkey: HKEY, key: &str) -> Result<(), std::io::Error> {
        #[cfg(target_os = "windows")]
        unsafe {
            let key_w = to_utf16(key);
            let val_w = to_utf16(self);
            let len = (val_w.len() * 2) as u32;
            let res = RegSetValueExW(
                hkey,
                key_w.as_ptr(),
                0,
                REG_SZ,
                val_w.as_ptr() as *const u8,
                len
            );
            if res == 0 {
                Ok(())
            } else {
                Err(std::io::Error::from_raw_os_error(res as i32))
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (hkey, key);
            Ok(())
        }
    }
}

impl RegValue for u32 {
    fn read_from_key(hkey: HKEY, key: &str) -> Result<Self, std::io::Error> {
        #[cfg(target_os = "windows")]
        unsafe {
            let key_w = to_utf16(key);
            let mut val_type: u32 = 0;
            let mut val: u32 = 0;
            let mut buf_size = std::mem::size_of::<u32>() as u32;
            let res = RegQueryValueExW(
                hkey,
                key_w.as_ptr(),
                std::ptr::null_mut(),
                &mut val_type,
                &mut val as *mut u32 as *mut u8,
                &mut buf_size
            );
            if res == 0 {
                Ok(val)
            } else {
                Err(std::io::Error::from_raw_os_error(res as i32))
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (hkey, key);
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Not Windows"))
        }
    }

    fn write_to_key(&self, hkey: HKEY, key: &str) -> Result<(), std::io::Error> {
        #[cfg(target_os = "windows")]
        unsafe {
            let key_w = to_utf16(key);
            let res = RegSetValueExW(
                hkey,
                key_w.as_ptr(),
                0,
                REG_DWORD,
                self as *const u32 as *const u8,
                std::mem::size_of::<u32>() as u32
            );
            if res == 0 {
                Ok(())
            } else {
                Err(std::io::Error::from_raw_os_error(res as i32))
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (hkey, key);
            Ok(())
        }
    }
}

pub struct KeyIterator {
    hkey: HKEY,
    index: u32,
}

impl Iterator for KeyIterator {
    type Item = Result<String, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        #[cfg(target_os = "windows")]
        unsafe {
            let mut name_buf = vec![0u16; 256];
            let mut name_len = name_buf.len() as u32;
            let res = RegEnumKeyExW(
                self.hkey,
                self.index,
                name_buf.as_mut_ptr(),
                &mut name_len,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut()
            );
            if res == 0 {
                self.index += 1;
                Some(Ok(from_utf16(&name_buf[..name_len as usize])))
            } else {
                None
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            None
        }
    }
}

pub struct ValueIterator {
    hkey: HKEY,
    index: u32,
}

impl Iterator for ValueIterator {
    type Item = Result<(String, RegValueRaw), std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        #[cfg(target_os = "windows")]
        unsafe {
            let mut name_buf = vec![0u16; 16384];
            let mut name_len = name_buf.len() as u32;
            let mut val_type: u32 = 0;
            let mut data_len: u32 = 0;
            let res = RegEnumValueW(
                self.hkey,
                self.index,
                name_buf.as_mut_ptr(),
                &mut name_len,
                std::ptr::null_mut(),
                &mut val_type,
                std::ptr::null_mut(),
                &mut data_len
            );
            if res == 259 {
                return None;
            }
            if res != 0 && res != 234 {
                return Some(Err(std::io::Error::from_raw_os_error(res as i32)));
            }
            let mut data_buf = vec![0u8; data_len as usize];
            name_len = name_buf.len() as u32;
            let res = RegEnumValueW(
                self.hkey,
                self.index,
                name_buf.as_mut_ptr(),
                &mut name_len,
                std::ptr::null_mut(),
                &mut val_type,
                data_buf.as_mut_ptr(),
                &mut data_len
            );
            if res == 0 {
                self.index += 1;
                let name = from_utf16(&name_buf[..name_len as usize]);
                data_buf.truncate(data_len as usize);
                Some(Ok((name, RegValueRaw { vtype: val_type, bytes: data_buf })))
            } else {
                None
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Thread-Local Test Override
// ---------------------------------------------------------------------------

thread_local! {
    pub static TEST_PATH_OVERRIDE: std::cell::RefCell<Option<std::path::PathBuf>> = const { std::cell::RefCell::new(None) };
}

pub fn set_test_path_override(path: Option<std::path::PathBuf>) {
    TEST_PATH_OVERRIDE.with(|p| *p.borrow_mut() = path);
}

// ---------------------------------------------------------------------------
// Unified Free Helper Functions
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
pub fn read_string(hive: HKEY, path: &str, key: &str) -> Option<String> {
    let root = RegKey::predef(hive);
    let subkey = root.open_subkey(path).ok()?;
    subkey.get_value(key).ok()
}

#[cfg(target_os = "windows")]
pub fn write_string(hive: HKEY, path: &str, key: &str, val: &str) -> std::io::Result<()> {
    let root = RegKey::predef(hive);
    let (subkey, _) = root.create_subkey(path)?;
    subkey.set_value(key, &val.to_string())
}

#[cfg(target_os = "windows")]
pub fn read_u32(hive: HKEY, path: &str, key: &str) -> Option<u32> {
    let root = RegKey::predef(hive);
    let subkey = root.open_subkey(path).ok()?;
    subkey.get_value(key).ok()
}

#[cfg(target_os = "windows")]
pub fn write_u32(hive: HKEY, path: &str, key: &str, val: u32) -> std::io::Result<()> {
    let root = RegKey::predef(hive);
    let (subkey, _) = root.create_subkey(path)?;
    subkey.set_value(key, &val)
}

#[cfg(target_os = "windows")]
pub fn list_values(hive: HKEY, path: &str) -> Option<Vec<(String, String)>> {
    let root = RegKey::predef(hive);
    let subkey = root.open_subkey(path).ok()?;
    let mut values = Vec::new();
    for item in subkey.enum_values() {
        if let Ok((name, _)) = item {
            if let Ok(val) = subkey.get_value::<String, _>(&name) {
                values.push((name, val));
            }
        }
    }
    Some(values)
}

#[cfg(target_os = "windows")]
pub fn read_binary(hive: HKEY, path: &str, key: &str) -> Option<Vec<u8>> {
    let root = RegKey::predef(hive);
    let subkey = root.open_subkey(path).ok()?;
    let val_raw = subkey.get_raw_value(key).ok()?;
    Some(val_raw.bytes)
}

#[cfg(target_os = "windows")]
pub fn write_binary(hive: HKEY, path: &str, key: &str, val: &[u8]) -> std::io::Result<()> {
    let root = RegKey::predef(hive);
    let (subkey, _) = root.create_subkey(path)?;
    let val_raw = RegValueRaw {
        vtype: REG_BINARY,
        bytes: val.to_vec(),
    };
    subkey.set_raw_value(key, &val_raw)
}

#[cfg(target_os = "windows")]
pub fn delete_value(hive: HKEY, path: &str, key: &str) -> std::io::Result<()> {
    let root = RegKey::predef(hive);
    let subkey = root.open_subkey_with_flags(path, KEY_WRITE)?;
    subkey.delete_value(key)
}

// ---------------------------------------------------------------------------
// Non-Windows Fallback Emulation Implementation
// ---------------------------------------------------------------------------

#[cfg(not(target_os = "windows"))]
mod fallback_impl {
    use super::{HKEY, REG_SZ, REG_DWORD, REG_BINARY, TEST_PATH_OVERRIDE};
    use std::collections::HashMap;
    use std::io;

    fn get_registry_file_path() -> Option<std::path::PathBuf> {
        if let Some(override_path) = TEST_PATH_OVERRIDE.with(|p| p.borrow().clone()) {
            return Some(override_path);
        }
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            Some(std::path::PathBuf::from(xdg).join("local76").join("registry.conf"))
        } else if let Ok(home) = std::env::var("HOME") {
            Some(std::path::PathBuf::from(home).join(".config").join("local76").join("registry.conf"))
        } else {
            None
        }
    }

    struct FileLock {
        lock_path: std::path::PathBuf,
        acquired: bool,
    }

    impl FileLock {
        fn acquire(file_path: &std::path::Path) -> Self {
            let lock_path = file_path.with_extension("lock");
            for _ in 0..20 {
                if std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&lock_path)
                    .is_ok()
                {
                    return Self { lock_path, acquired: true };
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Self { lock_path, acquired: false }
        }
    }

    impl Drop for FileLock {
        fn drop(&mut self) {
            if self.acquired {
                let _ = std::fs::remove_file(&self.lock_path);
            }
        }
    }

    fn read_entry(hive: HKEY, path: &str, key: &str) -> Option<(char, String)> {
        let file_path = get_registry_file_path()?;
        let _lock = FileLock::acquire(&file_path);
        if !file_path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(file_path).ok()?;
        let prefix = format!("{}::{}::{}=", hive, path.to_lowercase(), key.to_lowercase());
        for line in content.lines() {
            if line.starts_with(&prefix) {
                let val_part = &line[prefix.len()..];
                if val_part.len() >= 2 && val_part.as_bytes()[1] == b':' {
                    if let Some(vtype) = val_part.chars().next() {
                        let value = val_part[2..].to_string();
                        return Some((vtype, value));
                    }
                }
            }
        }
        None
    }

    fn write_entry(hive: HKEY, path: &str, key: &str, vtype: char, val: &str) -> io::Result<()> {
        let file_path = match get_registry_file_path() {
            Some(p) => p,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "No home directory resolved")),
        };
        if let Some(parent) = file_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        
        let _lock = FileLock::acquire(&file_path);
        let mut lines = Vec::new();
        if file_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                lines = content.lines().map(|s| s.to_string()).collect();
            }
        }
        
        let prefix = format!("{}::{}::{}=", hive, path.to_lowercase(), key.to_lowercase());
        let new_line = format!("{}{}:{}", prefix, vtype, val);
        
        let mut found = false;
        for line in &mut lines {
            if line.starts_with(&prefix) {
                *line = new_line.clone();
                found = true;
                break;
            }
        }
        if !found {
            lines.push(new_line);
        }
        
        std::fs::write(&file_path, lines.join("
"))?;
        Ok(())
    }

    fn delete_entry(hive: HKEY, path: &str, key: &str) -> io::Result<()> {
        let file_path = match get_registry_file_path() {
            Some(p) => p,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "No home directory resolved")),
        };
        let _lock = FileLock::acquire(&file_path);
        if !file_path.exists() {
            return Ok(());
        }
        let content = std::fs::read_to_string(&file_path)?;
        let prefix = format!("{}::{}::{}=", hive, path.to_lowercase(), key.to_lowercase());
        let mut lines = Vec::new();
        for line in content.lines() {
            if !line.starts_with(&prefix) {
                lines.push(line.to_string());
            }
        }
        std::fs::write(&file_path, lines.join("
"))?;
        Ok(())
    }

    pub fn read_string(hive: HKEY, path: &str, key: &str) -> Option<String> {
        read_entry(hive, path, key).and_then(|(t, v)| if t == 'S' { Some(v) } else { None })
    }

    pub fn write_string(hive: HKEY, path: &str, key: &str, val: &str) -> io::Result<()> {
        write_entry(hive, path, key, 'S', val)
    }

    pub fn read_u32(hive: HKEY, path: &str, key: &str) -> Option<u32> {
        read_entry(hive, path, key).and_then(|(t, v)| if t == 'D' { v.parse::<u32>().ok() } else { None })
    }

    pub fn write_u32(hive: HKEY, path: &str, key: &str, val: u32) -> io::Result<()> {
        write_entry(hive, path, key, 'D', &val.to_string())
    }

    pub fn list_values(hive: HKEY, path: &str) -> Option<Vec<(String, String)>> {
        let file_path = get_registry_file_path()?;
        if !file_path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(file_path).ok()?;
        let prefix = format!("{}::{}::", hive, path.to_lowercase());
        let mut values = Vec::new();
        for line in content.lines() {
            if line.starts_with(&prefix) {
                let rest = &line[prefix.len()..];
                if let Some(idx) = rest.find('=') {
                    let key_name = &rest[..idx];
                    let val_part = &rest[idx+1..];
                    if val_part.len() >= 2 && val_part.as_bytes()[1] == b':' {
                        if let Some(vtype) = val_part.chars().next() {
                            let val_str = &val_part[2..];
                            if vtype == 'S' {
                                values.push((key_name.to_string(), val_str.to_string()));
                            }
                        }
                    }
                }
            }
        }
        Some(values)
    }

    pub fn read_binary(hive: HKEY, path: &str, key: &str) -> Option<Vec<u8>> {
        read_entry(hive, path, key).and_then(|(t, v)| {
            if t == 'B' {
                let mut bytes = Vec::new();
                let mut chars = v.chars();
                while let (Some(c1), Some(c2)) = (chars.next(), chars.next()) {
                    if let Ok(b) = u8::from_str_radix(&format!("{}{}", c1, c2), 16) {
                        bytes.push(b);
                    }
                }
                Some(bytes)
            } else {
                None
            }
        })
    }

    pub fn write_binary(hive: HKEY, path: &str, key: &str, val: &[u8]) -> io::Result<()> {
        let hex: String = val.iter().map(|b| format!("{:02x}", b)).collect();
        write_entry(hive, path, key, 'B', &hex)
    }

    pub fn delete_value(hive: HKEY, path: &str, key: &str) -> io::Result<()> {
        delete_entry(hive, path, key)
    }
}

#[cfg(not(target_os = "windows"))]
pub use fallback_impl::{
    read_string, write_string, read_u32, write_u32, list_values, read_binary, write_binary, delete_value,
};

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
