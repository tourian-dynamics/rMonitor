//! Monitor / display enumeration utilities.

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{BOOL, LPARAM, RECT};
#[cfg(target_os = "windows")]
use windows_sys::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};

// Full Windows enumeration.
#[cfg(target_os = "windows")]
unsafe extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _lprect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let monitor_list = lparam as *mut Vec<String>;
    unsafe {
        let mut info: MONITORINFOEXW = std::mem::zeroed();
        info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

        let ok = GetMonitorInfoW(hmonitor, &mut info as *mut MONITORINFOEXW as *mut _);
        if ok != 0 {
            let w = (info.monitorInfo.rcMonitor.right - info.monitorInfo.rcMonitor.left).abs();
            let h = (info.monitorInfo.rcMonitor.bottom - info.monitorInfo.rcMonitor.top).abs();

            let mut dpi_x = 0;
            let mut dpi_y = 0;
            let hr = GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
            let scale = if hr == 0 {
                (dpi_x as f32 / 96.0 * 100.0).round() as u32
            } else {
                100
            };

            let name_u16 = &info.szDevice;
            let len = name_u16
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(name_u16.len());
            let name = String::from_utf16_lossy(&name_u16[..len]);

            let display_num = name.trim_start_matches(r"\\.\DISPLAY");
            let display_label = if !display_num.is_empty() {
                format!("Display {}", display_num)
            } else {
                name
            };

            (*monitor_list).push(format!("{}: {}x{} ({}% DPI)", display_label, w, h, scale));
        }
    }
    1
}

#[cfg(target_os = "windows")]
fn get_all_monitors_uncached() -> Vec<String> {
    let mut monitors = Vec::new();
    unsafe {
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            Some(monitor_enum_proc),
            &mut monitors as *mut Vec<String> as LPARAM,
        );
    }
    if monitors.is_empty() {
        monitors.push("Unknown Display".to_string());
    }
    monitors
}

#[cfg(not(target_os = "windows"))]
fn get_all_monitors_uncached() -> Vec<String> {
    let mut monitors = Vec::new();
    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let path = entry.path();
                let status_path = path.join("status");
                if status_path.exists() {
                    if let Ok(status) = std::fs::read_to_string(&status_path) {
                        if status.trim() == "connected" {
                            let connector_name = path
                                .file_name()
                                .map(|f| f.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Unknown".to_string());
                            let modes_path = path.join("modes");
                            let mut resolution = "Unknown resolution".to_string();
                            if modes_path.exists() {
                                if let Ok(modes) = std::fs::read_to_string(modes_path) {
                                    if let Some(first_mode) = modes.lines().next() {
                                        resolution = first_mode.trim().to_string();
                                    }
                                }
                            }
                            monitors.push(format!("{}: {}", connector_name, resolution));
                        }
                    }
                }
            }
        }
    }
    if monitors.is_empty() {
        monitors.push("Virtual Monitor: 1920x1080 (100% DPI)".to_string());
    }
    monitors
}

static MONITORS_CACHE: std::sync::Mutex<Option<(std::time::Instant, Vec<String>)>> = std::sync::Mutex::new(None);

pub fn get_all_monitors() -> Vec<String> {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    let mut lock = MONITORS_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((last_updated, val)) = &*lock {
        if last_updated.elapsed() < std::time::Duration::from_millis(2000) {
            return val.clone();
        }
    }
    let val = match catch_unwind(AssertUnwindSafe(get_all_monitors_uncached)) {
        Ok(v) => v,
        Err(_) => {
            if let Some((_, prev)) = lock.take() { prev } else { Vec::new() }
        }
    };
    *lock = Some((std::time::Instant::now(), val.clone()));
    val
}
