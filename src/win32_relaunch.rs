//! Windows-specific conhost relaunch logic.

/// Hides the console window at startup if running in Windows Console Host.
pub fn hide_console_at_startup() -> Option<*mut std::ffi::c_void> {
    #[cfg(target_os = "windows")]
    unsafe {
        let (_, terminal) = crate::backend::sys_info::query_shell_and_terminal();
        if terminal != "Windows Console Host" {
            return None;
        }
        let hwnd = windows_sys::Win32::System::Console::GetConsoleWindow();
        if hwnd.is_null() {
            return None;
        }
        windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(
            hwnd,
            windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE,
        );
        Some(hwnd)
    }
    #[cfg(not(target_os = "windows"))]
    None
}

/// Detects if we are in Windows Terminal or another virtual terminal, and should relaunch in standard conhost.
pub fn should_relaunch_in_conhost() -> bool {
    #[cfg(target_os = "windows")]
    {
        let (_, terminal) = crate::backend::sys_info::query_shell_and_terminal();
        let is_conhost = terminal == "Windows Console Host" && unsafe {
            let hwnd = windows_sys::Win32::System::Console::GetConsoleWindow();
            if !hwnd.is_null() {
                let mut rect = std::mem::zeroed();
                let ok = windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect(
                    hwnd,
                    &mut rect,
                );
                let style = windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                    hwnd,
                    windows_sys::Win32::UI::WindowsAndMessaging::GWL_STYLE,
                );
                (rect.right - rect.left) > 0 && style != 0
            } else {
                false
            }
        };
        !is_conhost
    }
    #[cfg(not(target_os = "windows"))]
    false
}

/// Relaunches the current executable inside a native conhost.exe process.
pub fn relaunch_in_conhost() -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        let current_exe = std::env::current_exe()?;
        let args: Vec<String> = std::env::args().collect();
        let mut con_args = vec![current_exe.to_string_lossy().to_string()];
        con_args.extend(args.into_iter().skip(1));
        con_args.push("--relaunched".to_string());

        std::process::Command::new("conhost.exe")
            .args(&con_args)
            .spawn()?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Not supported on this platform"))
}

/// Relaunches the current process in conhost.exe if it was spawned in a different terminal (like Windows Terminal) and we need standard console host behavior.
pub fn relaunch_in_conhost_if_needed() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--relaunched") {
        return;
    }

    for arg in &args {
        let base_arg = arg.trim_start_matches('-');
        if base_arg == "help" || base_arg == "h"
            || base_arg == "version" || base_arg == "v"
            || base_arg == "doctor"
            || base_arg == "list"
        {
            return;
        }
    }

    if should_relaunch_in_conhost() {
        if relaunch_in_conhost().is_ok() {
            std::process::exit(0);
        } else {
            eprintln!("Warning: Failed to relaunch in conhost.exe, continuing in current terminal.");
        }
    }
}

