//! pulse entry point: top-level CLI dispatch + render loop.

use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use library::lifecycle::background::file_log;
use library::lifecycle::foreground::window::{
    BorderlessConsole, ConsoleTitleGuard, SingleInstanceGuard, center_console_window,
    hide_console_at_startup,
};

mod app;
mod config;
mod diagnostics;
mod docs;
mod event_handler;
mod gpu_names;
mod helpers;
mod json;
mod logger;
mod modals;
mod network_statuses;
mod panels;
mod spring;
mod win32;

use crate::app::App;
use crate::config::AppConfig;
use crate::event_handler as eh;

const MIN_W: u16 = 100;
const MIN_H: u16 = 35;

fn run_tui() -> io::Result<()> {
    file_log::set_log_app_name("pulse");
    let _hwnd = hide_console_at_startup();
    let _instance_guard = match SingleInstanceGuard::try_new() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let _title_guard = ConsoleTitleGuard::new("pulse");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let _ = execute!(stdout, crossterm::terminal::SetSize(MIN_W, MIN_H));
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let config = AppConfig::load();
    let _borderless = if config.enable_borderless {
        Some(BorderlessConsole::enable())
    } else {
        None
    };
    std::thread::sleep(Duration::from_millis(50));
    if _borderless.is_none() {
        center_console_window();
    }

    // Re-show console after TUI is up (parity with helm/ignite/scout).
    #[cfg(windows)]
    {
        unsafe extern "system" {
            fn ShowWindow(hWnd: *mut std::ffi::c_void, nCmdShow: i32) -> i32;
            fn SetForegroundWindow(hWnd: *mut std::ffi::c_void) -> i32;
        }
        let h = hide_console_at_startup().unwrap_or(std::ptr::null_mut());
        if !h.is_null() {
            unsafe {
                ShowWindow(h, 5);
                SetForegroundWindow(h);
            }
        }
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(config);
    let tick_rate = Duration::from_millis(app.config.refresh_rate_ms as u64);
    let mut last_tick = Instant::now();
    let mut last_refresh = Instant::now();

    while !app.should_quit {
        app.status.tick();
        if let Some(state) = app.power.tick_power() {
            file_log::log_message("INFO", &format!("Power status changed: {}", state));
        }
        if app.power.tick_theme() {
            app.refresh_theme();
        }
        app.on_battery_set(app.power.on_battery);

        let now = Instant::now();
        let dt = now.duration_since(last_tick).as_secs_f64();
        app.update_physics(dt);
        last_tick = now;

        let refresh_interval = app.power.effective_tick_rate(Duration::from_millis(1500));
        if last_refresh.elapsed() > refresh_interval {
            app.update_metrics();
            last_refresh = Instant::now();
        }

        terminal.draw(|f| eh::draw(f, &mut app))?;

        let current_tick = app.power.effective_tick_rate(tick_rate);
        let timeout = current_tick
            .checked_sub(now.elapsed())
            .unwrap_or(Duration::from_secs(0));

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => eh::handle_key(&mut app, key),
                Event::Mouse(mouse) => eh::handle_mouse(&mut app, mouse.kind, mouse.column, mouse.row),
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    file_log::log_message("INFO", "pulse clean shutdown complete.");
    Ok(())
}

fn main() -> io::Result<()> {
    file_log::set_log_app_name("pulse");
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--json" => {
                json::print();
                return Ok(());
            }
            "--doctor" | "doctor" => {
                diagnostics::run_doctor();
                return Ok(());
            }
            "--install" | "install" => {
                diagnostics::run_install();
                return Ok(());
            }
            _ => {}
        }
    }
    let _ = docs::doc_for_f_key(1);
    run_tui()
}
