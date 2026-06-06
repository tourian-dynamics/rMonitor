#![allow(dead_code, non_snake_case)]
use std::ffi::c_void;
use std::io;

// FFI declarations for Mutex and Handles
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateMutexW(
        lp_mutex_attributes: *const c_void,
        b_initial_owner: i32,
        lp_name: *const u16,
    ) -> *mut c_void;

    fn CloseHandle(h_object: *mut c_void) -> i32;
    fn GetLastError() -> u32;
}

// ==========================================
// 1. Single Instance Mutex Guard
// ==========================================
pub struct SingleInstanceGuard {
    #[allow(dead_code)]
    handle: *mut c_void,
}

impl SingleInstanceGuard {
    pub fn try_new() -> Result<Self, String> {
        let name: Vec<u16> = "Local\\rmon_SingleInstanceMutex_2026\0"
            .encode_utf16()
            .collect();
        let handle = unsafe { CreateMutexW(std::ptr::null(), 1, name.as_ptr()) };
        if handle.is_null() {
            return Err("Failed to create single-instance mutex.".to_string());
        }

        let err = unsafe { GetLastError() };
        if err == 183 {
            // ERROR_ALREADY_EXISTS = 183
            unsafe { CloseHandle(handle) };
            return Err("Another instance of this application is already running.".to_string());
        }

        Ok(SingleInstanceGuard { handle })
    }
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { CloseHandle(self.handle) };
        }
    }
}

// ==========================================
// 2. Windows Application Event Log Sync
// ==========================================
#[link(name = "advapi32")]
unsafe extern "system" {
    fn RegisterEventSourceW(
        lp_unc_server_name: *const u16,
        lp_source_name: *const u16,
    ) -> *mut c_void;

    fn ReportEventW(
        h_event_log: *mut c_void,
        w_type: u16,
        w_category: u16,
        dw_event_id: u32,
        lp_user_sid: *mut c_void,
        w_num_strings: u16,
        dw_data_size: u32,
        lp_strings: *const *const u16,
        lp_raw_data: *mut c_void,
    ) -> i32;

    fn DeregisterEventSource(h_event_log: *mut c_void) -> i32;
}

pub fn log_windows_event(source_name: &str, event_type: u16, event_id: u32, message: &str) {
    unsafe {
        let source_w: Vec<u16> = source_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let handle = RegisterEventSourceW(std::ptr::null(), source_w.as_ptr());
        if !handle.is_null() {
            let message_w: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
            let strings: [*const u16; 1] = [message_w.as_ptr()];

            ReportEventW(
                handle,
                event_type,
                0, // category
                event_id,
                std::ptr::null_mut(), // user sid
                1,                    // num strings
                0,                    // data size
                strings.as_ptr(),
                std::ptr::null_mut(), // raw data
            );
            DeregisterEventSource(handle);
        }
    }
}

// ==========================================
// 3. Raw Win32 Console Title Recovery
// ==========================================
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetConsoleTitleW(lp_console_title: *mut u16, n_size: u32) -> u32;
    fn SetConsoleTitleW(lp_console_title: *const u16) -> i32;
}

pub struct ConsoleTitleGuard {
    original_title: Option<Vec<u16>>,
}

impl ConsoleTitleGuard {
    pub fn new(new_title: &str) -> Self {
        let mut buf = [0u16; 512];
        let len = unsafe { GetConsoleTitleW(buf.as_mut_ptr(), buf.len() as u32) };
        let original_title = if len > 0 {
            Some(buf[..len as usize].to_vec())
        } else {
            None
        };

        let title_w: Vec<u16> = new_title.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            SetConsoleTitleW(title_w.as_ptr());
        }

        ConsoleTitleGuard { original_title }
    }
}

impl Drop for ConsoleTitleGuard {
    fn drop(&mut self) {
        if let Some(ref title) = self.original_title {
            let mut title_null = title.clone();
            title_null.push(0);
            unsafe {
                SetConsoleTitleW(title_null.as_ptr());
            }
        }
    }
}

// ==========================================
// 4. Clipboard API Operations
// ==========================================
#[link(name = "user32")]
unsafe extern "system" {
    fn OpenClipboard(h_wnd_new_owner: *mut c_void) -> i32;
    fn EmptyClipboard() -> i32;
    fn SetClipboardData(u_format: u32, h_mem: *mut c_void) -> *mut c_void;
    fn CloseClipboard() -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GlobalAlloc(u_flags: u32, dw_bytes: usize) -> *mut c_void;
    fn GlobalLock(h_mem: *mut c_void) -> *mut c_void;
    fn GlobalUnlock(h_mem: *mut c_void) -> i32;
    fn GlobalFree(h_mem: *mut c_void) -> *mut c_void;
}

pub fn copy_text_to_clipboard(text: &str) -> io::Result<()> {
    unsafe {
        use std::ptr;
        if OpenClipboard(ptr::null_mut()) == 0 {
            return Err(io::Error::last_os_error());
        }
        if EmptyClipboard() == 0 {
            let _ = CloseClipboard();
            return Err(io::Error::last_os_error());
        }

        let text_w: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let len = text_w.len() * 2;
        let h_mem = GlobalAlloc(0x0002, len); // GMEM_MOVEABLE = 0x0002
        if h_mem.is_null() {
            let _ = CloseClipboard();
            return Err(io::Error::last_os_error());
        }

        let ptr = GlobalLock(h_mem);
        if ptr.is_null() {
            let _ = GlobalFree(h_mem);
            let _ = CloseClipboard();
            return Err(io::Error::last_os_error());
        }

        std::ptr::copy_nonoverlapping(text_w.as_ptr(), ptr as *mut u16, text_w.len());
        GlobalUnlock(h_mem);

        if SetClipboardData(13, h_mem).is_null() {
            // CF_UNICODETEXT = 13
            let _ = GlobalFree(h_mem);
            let _ = CloseClipboard();
            return Err(io::Error::last_os_error());
        }

        CloseClipboard();
    }
    Ok(())
}

// ==========================================
// 5. Battery and Power Query
// ==========================================
#[derive(Debug, Clone, Copy, Default)]
pub struct PowerStatus {
    pub ac_online: bool,
    pub battery_percent: u8,
}

#[repr(C)]
struct SYSTEM_POWER_STATUS {
    ACLineStatus: u8,
    BatteryFlag: u8,
    BatteryLifePercent: u8,
    SystemStatusFlag: u8,
    BatteryLifeTime: u32,
    BatteryFullLifeTime: u32,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetSystemPowerStatus(lp_system_power_status: *mut SYSTEM_POWER_STATUS) -> i32;
}

pub fn query_power_status() -> PowerStatus {
    let mut status: SYSTEM_POWER_STATUS = unsafe { std::mem::zeroed() };
    let ok = unsafe { GetSystemPowerStatus(&mut status) };
    if ok != 0 {
        PowerStatus {
            ac_online: status.ACLineStatus == 1,
            battery_percent: status.BatteryLifePercent,
        }
    } else {
        PowerStatus {
            ac_online: true,
            battery_percent: 255,
        }
    }
}

// ==========================================
// 6. System Themes & Appearance Settings
// ==========================================
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

// ==========================================
// 7. Borderless Console Window Hook (Dynamic Centering)
// ==========================================

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MONITORINFO {
    pub cbSize: u32,
    pub rcMonitor: RECT,
    pub rcWork: RECT,
    pub dwFlags: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn GetWindowRect(hwnd: *mut c_void, lp_rect: *mut RECT) -> i32;
    fn SetWindowLongPtrW(hwnd: *mut c_void, n_index: i32, dw_new_long: isize) -> isize;
    fn GetWindowLongPtrW(hwnd: *mut c_void, n_index: i32) -> isize;
    fn SetWindowPos(
        hwnd: *mut c_void,
        hwnd_insert_after: *mut c_void,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        flags: u32,
    ) -> i32;
    fn GetDpiForWindow(hwnd: *mut c_void) -> u32;
    fn MonitorFromWindow(hwnd: *mut c_void, flags: u32) -> *mut c_void;
    fn GetMonitorInfoW(h_monitor: *mut c_void, lp_mi: *mut MONITORINFO) -> i32;
    fn GetCursorPos(lp_point: *mut POINT) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetConsoleWindow() -> *mut c_void;
}

pub struct BorderlessConsole {
    hwnd: *mut c_void,
    original_style: isize,
    original_rect: RECT,
    active: bool,
}

impl BorderlessConsole {
    pub fn enable() -> Self {
        unsafe {
            let hwnd = GetConsoleWindow();
            if hwnd.is_null() {
                return BorderlessConsole {
                    hwnd: std::ptr::null_mut(),
                    original_style: 0,
                    original_rect: RECT::default(),
                    active: false,
                };
            }

            let original_style = GetWindowLongPtrW(hwnd, -16); // GWL_STYLE = -16
            let mut original_rect = RECT::default();
            GetWindowRect(hwnd, &mut original_rect);

            // Strip border decorations: WS_CAPTION | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU
            let style_mask = 0x00C00000 | 0x00040000 | 0x00020000 | 0x00010000 | 0x00080000;
            let new_style = original_style & !(style_mask as isize);
            SetWindowLongPtrW(hwnd, -16, new_style);

            let dpi = GetDpiForWindow(hwnd);
            let scale = dpi as f32 / 96.0;
            let width = (900.0 * scale) as i32;
            let height = (900.0 * scale) as i32;

            let mut x = 100;
            let mut y = 100;
            let h_monitor = MonitorFromWindow(hwnd, 2); // MONITOR_DEFAULTTONEAREST = 2
            if !h_monitor.is_null() {
                let mut mi = MONITORINFO::default();
                mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
                if GetMonitorInfoW(h_monitor, &mut mi) != 0 {
                    let monitor_w = mi.rcWork.right - mi.rcWork.left;
                    let monitor_h = mi.rcWork.bottom - mi.rcWork.top;
                    x = mi.rcWork.left + (monitor_w - width) / 2;
                    y = mi.rcWork.top + (monitor_h - height) / 2;
                }
            }

            SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                x,
                y,
                width,
                height,
                0x0020 | 0x0004 | 0x0010, // SWP_FRAMECHANGED | SWP_NOZORDER | SWP_NOACTIVATE
            );

            BorderlessConsole {
                hwnd,
                original_style,
                original_rect,
                active: true,
            }
        }
    }
}

impl Drop for BorderlessConsole {
    fn drop(&mut self) {
        if self.active && !self.hwnd.is_null() {
            unsafe {
                SetWindowLongPtrW(self.hwnd, -16, self.original_style);
                let width = self.original_rect.right - self.original_rect.left;
                let height = self.original_rect.bottom - self.original_rect.top;
                SetWindowPos(
                    self.hwnd,
                    std::ptr::null_mut(),
                    self.original_rect.left,
                    self.original_rect.top,
                    width,
                    height,
                    0x0020 | 0x0004 | 0x0010, // SWP_FRAMECHANGED | SWP_NOZORDER | SWP_NOACTIVATE
                );
            }
        }
    }
}

pub fn query_cursor_pos() -> Option<(i32, i32)> {
    unsafe {
        let mut pt = POINT::default();
        if GetCursorPos(&mut pt) != 0 {
            Some((pt.x, pt.y))
        } else {
            None
        }
    }
}

pub fn get_window_rect() -> Option<RECT> {
    unsafe {
        let hwnd = GetConsoleWindow();
        if !hwnd.is_null() {
            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect) != 0 {
                return Some(rect);
            }
        }
    }
    None
}

pub fn set_window_pos(x: i32, y: i32) {
    unsafe {
        let hwnd = GetConsoleWindow();
        if !hwnd.is_null() {
            SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                x,
                y,
                0,
                0,
                0x0001 | 0x0004 | 0x0010, // SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE
            );
        }
    }
}

/// Center the console window on the primary display or active monitor.
pub fn center_console_window() {
    unsafe {
        let hwnd = GetConsoleWindow();
        if hwnd.is_null() {
            return;
        }

        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect) != 0 {
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

            let h_monitor = MonitorFromWindow(hwnd, 2); // MONITOR_DEFAULTTONEAREST = 2
            if !h_monitor.is_null() {
                let mut mi = MONITORINFO::default();
                mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
                if GetMonitorInfoW(h_monitor, &mut mi) != 0 {
                    let monitor_w = mi.rcWork.right - mi.rcWork.left;
                    let monitor_h = mi.rcWork.bottom - mi.rcWork.top;

                    let x = mi.rcWork.left + (monitor_w - width) / 2;
                    let y = mi.rcWork.top + (monitor_h - height) / 2;

                    SetWindowPos(
                        hwnd,
                        std::ptr::null_mut(),
                        x,
                        y,
                        width,
                        height,
                        0x0001 | 0x0004 | 0x0010, // SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE
                    );
                }
            }
        }
    }
}

/// If the application is running in a pseudoconsole (like Windows Terminal) and we want it
/// to run as a standalone styled window, relaunch it inside conhost.exe.
pub fn relaunch_in_conhost_if_needed() {
    #[cfg(windows)]
    {
        // 1. Check if we have the --relaunched flag to prevent any potential loops
        let args: Vec<String> = std::env::args().collect();
        if args.iter().any(|arg| arg == "--relaunched") {
            return;
        }

        // 2. Check if there are arguments that request stdout/diagnostic mode
        for arg in &args {
            let lower = arg.to_lowercase();
            if lower == "--json" || lower == "--doctor" || lower == "doctor" ||
               lower == "--install" || lower == "install" {
                return;
            }
        }

        // 3. Detect if we are in conhost or a pseudoconsole (like Windows Terminal)
        let hwnd = unsafe { GetConsoleWindow() };
        let is_conhost = if hwnd.is_null() {
            false
        } else {
            let mut rect = RECT::default();
            let ok = unsafe { GetWindowRect(hwnd, &mut rect) };
            let style = unsafe { GetWindowLongPtrW(hwnd, -16) }; // GWL_STYLE = -16
            ok != 0 && (rect.right - rect.left) > 0 && style != 0
        };

        if !is_conhost {
            // Relaunch in conhost.exe
            let current_exe = std::env::current_exe().unwrap();
            let mut cmd_args = vec![
                "/c".to_string(),
                "start".to_string(),
                "".to_string(),
                "conhost.exe".to_string(),
                current_exe.to_str().unwrap().to_string(),
            ];
            // Pass all original args, plus the --relaunched flag
            for arg in args.into_iter().skip(1) {
                cmd_args.push(arg);
            }
            cmd_args.push("--relaunched".to_string());

            let _ = std::process::Command::new("cmd.exe")
                .args(&cmd_args)
                .spawn();
            std::process::exit(0);
        }
    }
}
