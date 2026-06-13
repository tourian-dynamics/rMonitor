//! Windows-specific host system information and theme querying utilities.

use crate::backend::sys_info::{PowerStatus, SystemBiosInfo, Cached};
use crate::backend::registry::{HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER};
use crate::backend::registry::RegKey;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub fn query_accent_color() -> (u8, u8, u8) {
    static CACHE: Mutex<Option<Cached<(u8, u8, u8)>>> = Mutex::new(None);
    let mut lock = CACHE.lock().unwrap();
    if let Some(c) = &*lock {
        if c.is_valid() {
            return c.value;
        }
    }
    let mut color: u32 = 0;
    let mut opaque: i32 = 0;
    let hr = unsafe {
        windows_sys::Win32::Graphics::Dwm::DwmGetColorizationColor(&mut color, &mut opaque)
    };
    let val = if hr == 0 {
        (
            ((color >> 16) & 0xFF) as u8,
            ((color >> 8) & 0xFF) as u8,
            (color & 0xFF) as u8,
        )
    } else {
        (0, 245, 255)
    };
    *lock = Some(Cached::new(val, Duration::from_millis(1000)));
    val
}

pub fn get_win_accent_color_hex() -> String {
    let (r, g, b) = query_accent_color();
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

pub fn query_high_contrast() -> bool {
    use windows_sys::Win32::UI::Accessibility::{HIGHCONTRASTW, HCF_HIGHCONTRASTON};
    use windows_sys::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_GETHIGHCONTRAST};

    let mut hc: HIGHCONTRASTW = unsafe { std::mem::zeroed() };
    hc.cbSize = std::mem::size_of::<HIGHCONTRASTW>() as u32;
    let res = unsafe {
        SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            hc.cbSize,
            &mut hc as *mut _ as *mut _,
            0,
        )
    };
    if res == 0 {
        return false;
    }
    (hc.dwFlags & HCF_HIGHCONTRASTON) != 0
}

pub fn query_os_version() -> String {
    static CACHE: Mutex<Option<Cached<String>>> = Mutex::new(None);
    let mut lock = CACHE.lock().unwrap();
    if let Some(c) = &*lock {
        if c.is_valid() {
            return c.value.clone();
        }
    }
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let mut product_name = "Windows".to_string();
    let mut current_build = String::new();
    let mut display_version = String::new();

    if let Ok(key) = hklm.open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion") {
        if let Ok(val) = key.get_value::<String, _>("ProductName") {
            product_name = val;
        }
        if let Ok(val) = key.get_value::<String, _>("CurrentBuild") {
            current_build = val;
        }
        if let Ok(val) = key.get_value::<String, _>("DisplayVersion") {
            display_version = val;
        }
    }

    let mut final_product = product_name;
    if final_product.starts_with("Windows 10") {
        if let Ok(build) = current_build.parse::<u32>() {
            if build >= 22000 {
                final_product = final_product.replace("Windows 10", "Windows 11");
            }
        }
    }

    let mut parts = vec![final_product];
    if !display_version.is_empty() {
        parts.push(display_version);
    }
    if !current_build.is_empty() {
        parts.push(format!("(Build {})", current_build));
    }
    let val = parts.join(" ");
    *lock = Some(Cached::new(val.clone(), Duration::from_secs(10)));
    val
}

pub fn query_dark_mode() -> bool {
    static CACHE: Mutex<Option<Cached<bool>>> = Mutex::new(None);
    let mut lock = CACHE.lock().unwrap();
    if let Some(c) = &*lock {
        if c.is_valid() {
            return c.value;
        }
    }
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let mut val = true;
    if let Ok(key) = hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize") {
        if let Ok(luse) = key.get_value::<u32, _>("AppsUseLightTheme") {
            val = luse == 0;
        }
    }
    *lock = Some(Cached::new(val, Duration::from_millis(500)));
    val
}

pub fn query_power_status() -> Option<PowerStatus> {
    static CACHE: Mutex<Option<Cached<Option<PowerStatus>>>> = Mutex::new(None);
    let mut lock = CACHE.lock().unwrap();
    if let Some(c) = &*lock {
        if c.is_valid() {
            return c.value.clone();
        }
    }
    let mut status = windows_sys::Win32::System::Power::SYSTEM_POWER_STATUS {
        ACLineStatus: 0,
        BatteryFlag: 0,
        BatteryLifePercent: 0,
        SystemStatusFlag: 0,
        BatteryLifeTime: 0,
        BatteryFullLifeTime: 0,
    };
    let val = if unsafe { windows_sys::Win32::System::Power::GetSystemPowerStatus(&mut status) } != 0 {
        Some(PowerStatus {
            ac_online: status.ACLineStatus == 1,
            battery_percent: status.BatteryLifePercent,
        })
    } else {
        None
    };
    *lock = Some(Cached::new(val.clone(), Duration::from_secs(1)));
    val
}

pub fn query_bios_info() -> Option<SystemBiosInfo> {
    static CACHE: Mutex<Option<Cached<Option<SystemBiosInfo>>>> = Mutex::new(None);
    let mut lock = CACHE.lock().unwrap();
    if let Some(c) = &*lock {
        if c.is_valid() {
            return c.value.clone();
        }
    }
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path = r"HARDWARE\DESCRIPTION\System\BIOS";
    let val = if let Ok(key) = hklm.open_subkey(path) {
        let manufacturer = key.get_value::<String, _>("SystemManufacturer").unwrap_or_default();
        let product = key.get_value::<String, _>("SystemProductName").unwrap_or_default();
        let model = key.get_value::<String, _>("BaseBoardProduct").unwrap_or_default();
        Some(SystemBiosInfo {
            manufacturer: manufacturer.trim().to_string(),
            product: product.trim().to_string(),
            model: model.trim().to_string(),
        })
    } else {
        None
    };
    *lock = Some(Cached::new(val.clone(), Duration::from_secs(10)));
    val
}

pub fn query_gpu_names() -> Vec<String> {
    static CACHE: Mutex<Option<Cached<Vec<String>>>> = Mutex::new(None);
    let mut lock = CACHE.lock().unwrap();
    if let Some(c) = &*lock {
        if c.is_valid() {
            return c.value.clone();
        }
    }
    let mut gpus = Vec::new();
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path = r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}";
    if let Ok(class_key) = hklm.open_subkey(path) {
        for subkey_name in class_key.enum_keys().filter_map(|x| x.ok()) {
            if subkey_name.len() == 4 && subkey_name.chars().all(|c| c.is_ascii_digit()) {
                if let Ok(gpu_key) = class_key.open_subkey(&subkey_name) {
                    if let Ok(desc) = gpu_key.get_value::<String, _>("DriverDesc") {
                        gpus.push(desc);
                    }
                }
            }
        }
    }
    *lock = Some(Cached::new(gpus.clone(), Duration::from_secs(5)));
    gpus
}

pub fn get_local_time_string() -> String {
    use windows_sys::Win32::Foundation::SYSTEMTIME;
    use windows_sys::Win32::System::SystemInformation::GetLocalTime;
    let mut time = std::mem::MaybeUninit::<SYSTEMTIME>::uninit();
    unsafe {
        GetLocalTime(time.as_mut_ptr());
    }
    let time = unsafe { time.assume_init() };
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        time.wYear, time.wMonth, time.wDay, time.wHour, time.wMinute, time.wSecond
    )
}
