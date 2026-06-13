//! Initialization guards for title, borderless console, and single-instance behavior.

use crate::logger;

/// Ensures only one instance of the application is active at any time.
pub struct SingleInstanceGuard {
    #[cfg(target_os = "windows")]
    handle: windows_sys::Win32::Foundation::HANDLE,
    #[cfg(target_os = "linux")]
    _file: std::fs::File,
}

impl SingleInstanceGuard {
    pub fn try_new(title: &str) -> Result<Self, String> {
        #[cfg(target_os = "windows")]
        unsafe {
            let mutex_name = format!("Local\\{}_SingleInstanceMutex", title);
            let name: Vec<u16> = mutex_name.encode_utf16().chain(std::iter::once(0)).collect();
            let handle = windows_sys::Win32::System::Threading::CreateMutexW(
                std::ptr::null(),
                1,
                name.as_ptr(),
            );
            if handle.is_null() {
                return Err("Failed to create single-instance mutex.".to_string());
            }

            let err = windows_sys::Win32::Foundation::GetLastError();
            if err == windows_sys::Win32::Foundation::ERROR_ALREADY_EXISTS {
                windows_sys::Win32::Foundation::CloseHandle(handle);
                return Err("Another instance of this application is already running.".to_string());
            }

            Ok(Self { handle })
        }
        #[cfg(target_os = "linux")]
        {
            use std::fs::OpenOptions;
            use std::os::unix::io::AsRawFd;
            let socket_path = format!("/tmp/{}_single_instance.sock", title);
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&socket_path)
                .map_err(|e| format!("Failed to open lock file: {}", e))?;

            // LOCK_EX = 2, LOCK_NB = 4, LOCK_EX | LOCK_NB = 6
            unsafe extern "C" {
                fn flock(fd: std::os::raw::c_int, operation: std::os::raw::c_int) -> std::os::raw::c_int;
            }
            let res = unsafe { flock(file.as_raw_fd(), 6) };
            if res < 0 {
                return Err("Another instance of this application is already running.".to_string());
            }
            Ok(Self { _file: file })
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            let _ = title;
            Ok(Self {})
        }
    }

    pub fn try_new_or_exit(title: &str) -> Self {
        match Self::try_new(title) {
            Ok(g) => g,
            Err(e) => {
                logger::log_message("ERROR", &format!("SingleInstanceGuard blocked launch of {}: {}", title, e));
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        unsafe {
            if !self.handle.is_null() {
                windows_sys::Win32::Foundation::CloseHandle(self.handle);
            }
        }
    }
}

pub struct ConsoleTitleGuard {
    #[cfg(target_os = "windows")]
    original_title: Option<Vec<u16>>,
}

impl ConsoleTitleGuard {
    pub fn new(new_title: &str) -> Self {
        #[cfg(target_os = "windows")]
        {
            let mut buf = [0u16; 512];
            let len = unsafe {
                windows_sys::Win32::System::Console::GetConsoleTitleW(
                    buf.as_mut_ptr(),
                    buf.len() as u32,
                )
            };
            let original_title = if len > 0 {
                let safe_len = (len as usize).min(buf.len());
                Some(buf[..safe_len].to_vec())
            } else {
                None
            };

            let title_w: Vec<u16> = new_title.encode_utf16().chain(std::iter::once(0)).collect();
            unsafe {
                windows_sys::Win32::System::Console::SetConsoleTitleW(title_w.as_ptr());
            }

            ConsoleTitleGuard { original_title }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = new_title;
            ConsoleTitleGuard {}
        }
    }
}

impl Drop for ConsoleTitleGuard {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        if let Some(ref title) = self.original_title {
            let mut title_null = title.clone();
            title_null.push(0);
            unsafe {
                windows_sys::Win32::System::Console::SetConsoleTitleW(title_null.as_ptr());
            }
        }
    }
}

pub struct BorderlessConsole {
    #[cfg(target_os = "windows")]
    hwnd: *mut std::ffi::c_void,
    #[cfg(target_os = "windows")]
    original_style: isize,
    #[cfg(target_os = "windows")]
    original_rect: windows_sys::Win32::Foundation::RECT,
    #[cfg(target_os = "windows")]
    active: bool,
}

impl BorderlessConsole {
    pub fn enable() -> Self {
        #[cfg(target_os = "windows")]
        unsafe {
            use windows_sys::Win32::System::Console::GetConsoleWindow;
            use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowRect, GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_STYLE, WS_CAPTION, WS_THICKFRAME, WS_MINIMIZEBOX, WS_MAXIMIZEBOX, WS_SYSMENU, SWP_FRAMECHANGED, SWP_NOZORDER, SWP_NOACTIVATE};
            use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
            use windows_sys::Win32::Graphics::Gdi::{MonitorFromWindow, GetMonitorInfoW, MONITORINFO, MONITOR_DEFAULTTONEAREST};

            let (_, terminal) = crate::backend::sys_info::query_shell_and_terminal();
            if terminal != "Windows Console Host" {
                return Self { hwnd: std::ptr::null_mut(), original_style: 0, original_rect: std::mem::zeroed(), active: false };
            }

            let hwnd = GetConsoleWindow();
            if hwnd.is_null() {
                return Self { hwnd: std::ptr::null_mut(), original_style: 0, original_rect: std::mem::zeroed(), active: false };
            }

            let original_style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            let mut original_rect = std::mem::zeroed();
            GetWindowRect(hwnd, &mut original_rect);

            let new_style = original_style & !((WS_CAPTION | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU) as isize);
            SetWindowLongPtrW(hwnd, GWL_STYLE, new_style);

            let dpi = GetDpiForWindow(hwnd);
            let scale = dpi as f32 / 96.0;
            let width = (900.0 * scale) as i32;
            let height = (900.0 * scale) as i32;

            let mut x = 100;
            let mut y = 100;
            let h_monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
            if !h_monitor.is_null() {
                let mut mi: MONITORINFO = std::mem::zeroed();
                mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
                if GetMonitorInfoW(h_monitor, &mut mi as *mut _ as *mut _) != 0 {
                    let monitor_w = mi.rcWork.right - mi.rcWork.left;
                    let monitor_h = mi.rcWork.bottom - mi.rcWork.top;
                    x = mi.rcWork.left + (monitor_w - width) / 2;
                    y = mi.rcWork.top + (monitor_h - height) / 2;
                }
            }

            SetWindowPos(hwnd, std::ptr::null_mut(), x, y, width, height, SWP_FRAMECHANGED | SWP_NOZORDER | SWP_NOACTIVATE);
            Self { hwnd, original_style, original_rect, active: true }
        }
        #[cfg(not(target_os = "windows"))]
        {
            Self {}
        }
    }
}

impl Drop for BorderlessConsole {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        unsafe {
            if self.active && !self.hwnd.is_null() {
                use windows_sys::Win32::UI::WindowsAndMessaging::{SetWindowLongPtrW, SetWindowPos, GWL_STYLE, SWP_FRAMECHANGED, SWP_NOZORDER, SWP_NOACTIVATE};
                SetWindowLongPtrW(self.hwnd, GWL_STYLE, self.original_style);
                let width = self.original_rect.right - self.original_rect.left;
                let height = self.original_rect.bottom - self.original_rect.top;
                SetWindowPos(self.hwnd, std::ptr::null_mut(), self.original_rect.left, self.original_rect.top, width, height, SWP_FRAMECHANGED | SWP_NOZORDER | SWP_NOACTIVATE);
            }
        }
    }
}
