//! Application initialization and lifecycle guards.

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use ratatui::{Terminal, backend::TermwizBackend};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, SetSize, disable_raw_mode, enable_raw_mode},
    event::{EnableMouseCapture, DisableMouseCapture},
};
use crate::backend::sys_info;
use crate::logger;

static APP_SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

/// Signal that the application is shutting down.
pub fn set_app_shutting_down(val: bool) {
    APP_SHUTTING_DOWN.store(val, Ordering::Relaxed);
}

/// Check if the application is shutting down.
pub fn is_app_shutting_down() -> bool {
    APP_SHUTTING_DOWN.load(Ordering::Relaxed)
}

/// registers a panic handler that restores the terminal before exiting.
pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            crossterm::cursor::Show
        );

        let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.as_str()
        } else {
            "Box<dyn Any>"
        };

        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let crash_report = format!("Application panicked at {}: {}", location, msg);
        logger::log_message("PANIC", &crash_report);

        eprintln!("\n══════════════════════════════════════════════════════════════");
        eprintln!(" ⚠️  FATAL ERROR: Application Panicked");
        eprintln!("══════════════════════════════════════════════════════════════");
        eprintln!("Location : {}", location);
        eprintln!("Error    : {}", msg);
        eprintln!("══════════════════════════════════════════════════════════════");
        eprintln!("Restored terminal to normal mode. Exiting.\n");

        std::process::exit(1);
    }));
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn ctrl_handler(ctrl_type: u32) -> windows_sys::Win32::Foundation::BOOL {
    use windows_sys::Win32::Foundation::FALSE;
    use windows_sys::Win32::System::Console::{
        CTRL_C_EVENT, CTRL_BREAK_EVENT, CTRL_CLOSE_EVENT, CTRL_LOGOFF_EVENT, CTRL_SHUTDOWN_EVENT
    };

    match ctrl_type {
        CTRL_C_EVENT | CTRL_BREAK_EVENT | CTRL_CLOSE_EVENT | CTRL_LOGOFF_EVENT | CTRL_SHUTDOWN_EVENT => {
            set_app_shutting_down(true);
            FALSE
        }
        _ => FALSE,
    }
}

pub use crate::bootstrap_guards::{SingleInstanceGuard, ConsoleTitleGuard, BorderlessConsole};

#[derive(Debug, Clone)]
pub struct Config {
    pub title: &'static str,
    pub size: (u16, u16),
    pub enforce_single_instance: bool,
    pub borderless: bool,
    pub install_panic_hook: bool,
}

impl Config {
    pub fn new(title: &'static str) -> Self {
        Self {
            title,
            size: (100, 35),
            enforce_single_instance: true,
            borderless: false,
            install_panic_hook: true,
        }
    }
}

pub struct ConsoleGuard {
    active: bool,
}

impl ConsoleGuard {
    pub fn new() -> Self {
        Self { active: true }
    }
    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

impl Drop for ConsoleGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = disable_raw_mode();
            let _ = execute!(
                io::stdout(),
                LeaveAlternateScreen,
                DisableMouseCapture
            );
        }
    }
}

pub struct Guards {
    pub _instance_guard: Option<SingleInstanceGuard>,
    pub _title_guard: ConsoleTitleGuard,
    pub _borderless: Option<BorderlessConsole>,
    pub _console_guard: ConsoleGuard,
}

pub fn init(config: Config) -> io::Result<(Terminal<TermwizBackend>, Guards)> {
    set_app_shutting_down(false);

    #[cfg(target_os = "windows")]
    unsafe {
        let _ = windows_sys::Win32::System::Console::SetConsoleCtrlHandler(
            Some(ctrl_handler),
            windows_sys::Win32::Foundation::TRUE,
        );
    }

    if config.install_panic_hook {
        set_panic_hook();
    }

    let _instance_guard = if config.enforce_single_instance {
        Some(SingleInstanceGuard::try_new_or_exit(config.title))
    } else {
        None
    };

    let _title_guard = ConsoleTitleGuard::new(config.title);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let _ = execute!(stdout, SetSize(config.size.0, config.size.1));
    if let Err(e) = execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
        let _ = disable_raw_mode();
        return Err(e);
    }

    let _borderless = if config.borderless {
        Some(BorderlessConsole::enable())
    } else {
        None
    };

    std::thread::sleep(Duration::from_millis(50));

    #[cfg(target_os = "windows")]
    {
        if _borderless.is_none() {
            unsafe {
                use windows_sys::Win32::System::Console::GetConsoleWindow;
                use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindowRect, IsWindowVisible, SetWindowPos, SWP_NOSIZE, SWP_NOZORDER, SWP_NOACTIVATE};
                use windows_sys::Win32::Graphics::Gdi::{MonitorFromWindow, GetMonitorInfoW, MONITORINFO, MONITOR_DEFAULTTONEAREST};

                let hwnd = GetConsoleWindow();
                if !hwnd.is_null() && IsWindowVisible(hwnd) != 0 {
                    let mut rect = std::mem::zeroed();
                    if GetWindowRect(hwnd, &mut rect) != 0 {
                        let width = rect.right - rect.left;
                        let height = rect.bottom - rect.top;
                        let h_monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
                        if !h_monitor.is_null() {
                            let mut mi: MONITORINFO = std::mem::zeroed();
                            mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
                            if GetMonitorInfoW(h_monitor, &mut mi as *mut _ as *mut _) != 0 {
                                let monitor_w = mi.rcWork.right - mi.rcWork.left;
                                let monitor_h = mi.rcWork.bottom - mi.rcWork.top;
                                let x = mi.rcWork.left + (monitor_w - width) / 2;
                                let y = mi.rcWork.top + (monitor_h - height) / 2;
                                SetWindowPos(hwnd, std::ptr::null_mut(), x, y, width, height, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
                            }
                        }
                    }
                }
            }
        }
    }

    let backend = TermwizBackend::new().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let _console_guard = ConsoleGuard::new();

    Ok((
        terminal,
        Guards {
            _instance_guard,
            _title_guard,
            _borderless,
            _console_guard,
        },
    ))
}

pub fn shutdown(_terminal: &mut Terminal<TermwizBackend>) -> io::Result<()> {
    set_app_shutting_down(true);
    let _ = disable_raw_mode();
    let _ = execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    );
    Ok(())
}
