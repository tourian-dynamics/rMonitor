#![allow(dead_code, unused_imports)]
use std::io;
use winreg::RegKey;
use winreg::enums::KEY_READ;
pub use winreg::enums::{HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, HKEY_USERS};

/// Read a string value from the registry.
pub fn read_string(hive: winreg::HKEY, path: &str, key: &str) -> Option<String> {
    let root = RegKey::predef(hive);
    let subkey = root.open_subkey_with_flags(path, KEY_READ).ok()?;
    subkey.get_value::<String, _>(key).ok()
}

/// Write a string value to the registry.
pub fn write_string(hive: winreg::HKEY, path: &str, key: &str, val: &str) -> io::Result<()> {
    let root = RegKey::predef(hive);
    let (subkey, _) = root.create_subkey(path)?;
    subkey.set_value(key, &val.to_string())
}

/// Read a u32 (DWORD) value from the registry.
pub fn read_u32(hive: winreg::HKEY, path: &str, key: &str) -> Option<u32> {
    let root = RegKey::predef(hive);
    let subkey = root.open_subkey_with_flags(path, KEY_READ).ok()?;
    subkey.get_value::<u32, _>(key).ok()
}

/// Write a u32 (DWORD) value to the registry.
pub fn write_u32(hive: winreg::HKEY, path: &str, key: &str, val: u32) -> io::Result<()> {
    let root = RegKey::predef(hive);
    let (subkey, _) = root.create_subkey(path)?;
    subkey.set_value(key, &val)
}

/// Read a boolean value represented as "1" or "0" string in the registry.
pub fn read_bool_str(hive: winreg::HKEY, path: &str, key: &str) -> bool {
    read_string(hive, path, key)
        .map(|s| s.trim() == "1")
        .unwrap_or(false)
}

/// Write a boolean value as a "1" or "0" string to the registry.
pub fn write_bool_str(hive: winreg::HKEY, path: &str, key: &str, val: bool) -> io::Result<()> {
    let str_val = if val { "1" } else { "0" };
    write_string(hive, path, key, str_val)
}
