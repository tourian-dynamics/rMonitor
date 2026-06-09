//! Re-export shim that lets the rest of the crate keep writing
//! `win32::query_power_status()` etc. while delegating to library.

#[allow(unused_imports)]
#[allow(unused_imports)]
pub use library::clipboard::copy_text_to_clipboard;
#[allow(unused_imports)]
pub use library::lifecycle::foreground::identity::{hostname, os_str, user_host, username};
#[allow(unused_imports)]
pub use library::lifecycle::foreground::window::{
    center_console_window, get_window_rect, query_cursor_pos, set_window_pos,
    BorderlessConsole, ConsoleTitleGuard, SingleInstanceGuard,
};
#[allow(unused_imports)]
pub use library::platform::native::monitors::get_all_monitors;
#[allow(unused_imports)]
#[allow(unused_imports)]
pub use library::platform::native::sys_info::{
    query_bios_info, query_dark_mode as is_dark_mode, query_disk_drives, query_gpu_names,
    query_network_adapters, query_os_version, query_power_status, query_shell_and_terminal,
};

/// Windows-specific accent color reading (kept local because the legacy
/// `print` module expects a hex `String` instead of the (u8,u8,u8) tuple
/// library provides).
pub fn get_win_accent_color() -> String {
    use winreg::RegKey;
    use winreg::enums::*;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\DWM";
    #[allow(clippy::collapsible_if)]
    if let Ok(key) = hkcu.open_subkey_with_flags(path, KEY_READ) {
        if let Ok(val) = key.get_value::<u32, _>("AccentColor") {
            let r = (val & 0xFF) as u8;
            let g = ((val >> 8) & 0xFF) as u8;
            let b = ((val >> 16) & 0xFF) as u8;
            return format!("#{:02X}{:02X}{:02X}", r, g, b);
        }
    }
    "#00F5FF".to_string()
}
