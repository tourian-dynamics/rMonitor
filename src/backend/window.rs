//! Window management and drag-to-move utilities for pulse.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Foreground) + Platform (Native).

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(C)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// State for an in-progress drag-to-move gesture.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowDrag {
    pub active: bool,
    /// Cursor position (x, y) at drag start.
    pub start_cursor: Option<(i32, i32)>,
    /// Window top-left (x, y) at drag start.
    pub start_window: Option<(i32, i32)>,
}

impl WindowDrag {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if a drag gesture is currently in progress.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Begin a drag if `row` is in the title bar. Returns true on drag-start.
    pub fn try_begin(&mut self, row: u16) -> bool {
        if row > 2 {
            return false;
        }
        if let (Some(cursor), Some(rect)) = (query_cursor_pos(), get_window_rect()) {
            self.active = true;
            self.start_cursor = Some(cursor);
            self.start_window = Some((rect.left, rect.top));
            true
        } else {
            false
        }
    }

    /// Update the window position based on the current cursor delta.
    /// No-op if not currently dragging.
    pub fn update(&mut self) {
        if !self.active {
            return;
        }
        let (Some(start_cursor), Some(start_window)) = (self.start_cursor, self.start_window) else {
            return;
        };
        if let Some(curr_cursor) = query_cursor_pos() {
            let dx = curr_cursor.0 - start_cursor.0;
            let dy = curr_cursor.1 - start_cursor.1;
            set_window_pos(start_window.0 + dx, start_window.1 + dy);
        }
    }

    /// End the current drag gesture.
    pub fn end(&mut self) {
        self.active = false;
        self.start_cursor = None;
        self.start_window = None;
    }
}

#[cfg(target_os = "windows")]
mod win_impl {
    use super::RECT;

    pub fn get_window_rect() -> Option<RECT> {
        let hwnd = unsafe { windows_sys::Win32::System::Console::GetConsoleWindow() };
        if hwnd.is_null() {
            return None;
        }
        let mut rect = RECT::default();
        let ok = unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect(
                hwnd,
                &mut rect as *mut RECT as *mut windows_sys::Win32::Foundation::RECT,
            )
        };
        if ok != 0 {
            Some(rect)
        } else {
            None
        }
    }

    pub fn set_window_pos(x: i32, y: i32) {
        let hwnd = unsafe { windows_sys::Win32::System::Console::GetConsoleWindow() };
        if !hwnd.is_null() {
            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::SetWindowPos(
                    hwnd,
                    std::ptr::null_mut(),
                    x,
                    y,
                    0,
                    0,
                    windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOSIZE
                        | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                        | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
                );
            }
        }
    }

    pub fn query_cursor_pos() -> Option<(i32, i32)> {
        let mut pt = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };
        let ok = unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut pt) };
        if ok != 0 {
            Some((pt.x, pt.y))
        } else {
            None
        }
    }

    pub fn hide_console_at_startup() -> Option<*mut std::ffi::c_void> {
        let (_, terminal) = crate::backend::sys_info::query_shell_and_terminal();
        if terminal != "Windows Console Host" {
            return None;
        }
        let hwnd = unsafe { windows_sys::Win32::System::Console::GetConsoleWindow() };
        if hwnd.is_null() {
            return None;
        }
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(
                hwnd,
                windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE,
            );
        }
        Some(hwnd as *mut std::ffi::c_void)
    }

    pub fn show_console_window() {
        let hwnd = unsafe { windows_sys::Win32::System::Console::GetConsoleWindow() };
        if !hwnd.is_null() {
            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(
                    hwnd,
                    windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOW,
                );
                windows_sys::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd);
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod fallback_impl {
    use super::RECT;

    pub fn get_window_rect() -> Option<RECT> {
        None
    }

    pub fn set_window_pos(_x: i32, _y: i32) {}

    pub fn query_cursor_pos() -> Option<(i32, i32)> {
        None
    }

    pub fn hide_console_at_startup() -> Option<*mut std::ffi::c_void> {
        None
    }

    pub fn show_console_window() {}
}

#[cfg(target_os = "windows")]
pub use win_impl::*;

#[cfg(not(target_os = "windows"))]
pub use fallback_impl::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drag_initial() {
        let drag = WindowDrag::new();
        assert!(!drag.is_active());
    }
}
