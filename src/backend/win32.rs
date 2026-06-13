//! Re-export shim that lets the rest of the crate keep writing
//! `win32::query_power_status()` etc. while delegating to local implementations.

#[allow(unused_imports)]
pub use crate::clipboard::copy_text_to_clipboard;
#[allow(unused_imports)]
pub use crate::backend::identity::{hostname, os_str, user_host, username};
#[allow(unused_imports)]
pub use crate::backend::window::{
    get_window_rect, query_cursor_pos, set_window_pos, WindowDrag,
};
#[allow(unused_imports)]
pub use crate::bootstrap_guards::{
    BorderlessConsole, ConsoleTitleGuard, SingleInstanceGuard,
};
#[allow(unused_imports)]
pub use crate::backend::monitors::get_all_monitors;
#[allow(unused_imports)]
pub use crate::backend::sys_info::{
    query_bios_info, query_dark_mode as is_dark_mode, query_gpu_names,
    query_os_version, query_power_status, query_shell_and_terminal,
};

/// Windows-specific accent color reading (kept local because the legacy
/// `print` module expects a hex `String` instead of the (u8,u8,u8) tuple
/// library provides).
pub fn get_win_accent_color() -> String {
    use crate::backend::registry::RegKey;
    use crate::backend::registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE, KEY_ALL_ACCESS};
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
