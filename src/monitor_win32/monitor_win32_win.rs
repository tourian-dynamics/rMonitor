#![allow(dead_code)]
// Power query delegates to rcommon (avoids duplicating GetSystemPowerStatus + windows_sys power feature bloat here).
// Local PowerStatus struct kept for API compatibility with the rest of rMonitor.

#[derive(Debug, Clone, Copy, Default)]
pub struct PowerStatus {
    pub ac_online: bool,
    pub battery_percent: u8,
}

pub fn query_power_status() -> PowerStatus {
    if let Some(p) = rcommon::sys_info::query_power_status() {
        PowerStatus {
            ac_online: p.ac_online,
            battery_percent: p.battery_percent,
        }
    } else {
        PowerStatus {
            ac_online: true,
            battery_percent: 255,
        }
    }
}

pub fn get_win_accent_color() -> String {
    use winreg::RegKey;
    use winreg::enums::*;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\DWM";
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

pub fn is_dark_mode() -> bool {
    use winreg::RegKey;
    use winreg::enums::*;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
    if let Ok(key) = hkcu.open_subkey_with_flags(path, KEY_READ) {
        if let Ok(val) = key.get_value::<u32, _>("AppsUseLightTheme") {
            return val == 0;
        }
    }
    true
}
