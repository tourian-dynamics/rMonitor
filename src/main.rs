//! pulse entry point: top-level CLI dispatch + render loop.

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event};
use crate::logger as file_log;
use crate::win32_relaunch::hide_console_at_startup;
use crate::bootstrap::{init, shutdown, Config as BootstrapConfig};

mod app;
mod backend;
mod bootstrap;
mod bootstrap_guards;
mod chrome;
mod clipboard;
mod config;
mod diagnostics;
mod gpu_names;
mod json;
mod logger;
mod metrics_format;
mod network_statuses;
mod ui;
mod utils;
mod win32_relaunch;

#[cfg(test)]
mod tests_perf;

use crate::app::App;
use crate::config::AppConfig;

const MIN_W: u16 = 100;
const MIN_H: u16 = 35;

fn run_ui() -> io::Result<()> {
    file_log::set_log_app_name("app/pulse");
    let _hwnd = hide_console_at_startup();

    let config = AppConfig::load();

    let mut tui_config = BootstrapConfig::new("pulse");
    tui_config.borderless = config.enable_borderless;
    tui_config.size = (MIN_W, MIN_H);

    let (mut terminal, _guards) = init(tui_config)?;

        crate::backend::window::show_console_window();

    let mut app = App::new(config);
    let tick_rate = Duration::from_millis(app.config.refresh_rate_ms as u64);
    let mut last_tick = Instant::now();
    let mut last_refresh = Instant::now();

    while !app.should_quit {
        if crate::bootstrap::is_app_shutting_down() {
            break;
        }
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

        terminal.draw(|f| ui::draw(f, &mut app))?;

        let current_tick = app.power.effective_tick_rate(tick_rate);
        let timeout = current_tick
            .checked_sub(now.elapsed())
            .unwrap_or(Duration::from_secs(0));

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => app::keys::handle_key(&mut app, key),
                Event::Mouse(mouse) => app::mouse::handle_mouse(&mut app, mouse.kind, mouse.column, mouse.row),
                _ => {}
            }
        }
    }

    shutdown(&mut terminal)?;
    file_log::log_message("INFO", "pulse clean shutdown complete.");
    Ok(())
}

fn main() -> io::Result<()> {
    file_log::set_log_app_name("app/pulse");
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
    let _ = crate::chrome::embedded_docs::doc_for_f_key(1);
    run_ui()
}
