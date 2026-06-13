//! Keyboard input event handler for pulse.
//!
//! **Taxonomy Classification**: Interface (Presentation Layer).

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use crate::ui::markdown::parse_markdown_to_lines;
use crate::app::{App, FocusedSection};
use crate::ui::overlays::DOC_FILES;

pub fn handle_key(app: &mut App, key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }
    if key.code == KeyCode::Char('c') && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
        app.should_quit = true;
        return;
    }
    if let Some(pid) = app.kill_confirm_pid {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                let sys_pid = crate::backend::sysinfo_shim::Pid::from_u32(pid);
                let success = app
                    .sys
                    .process(sys_pid)
                    .map(|p| p.kill())
                    .unwrap_or(false);
                if success {
                    app.set_status(format!("Successfully terminated process (PID: {})", pid));
                } else {
                    let tk_status = std::process::Command::new("taskkill")
                        .args(["/F", "/PID", &pid.to_string()])
                        .output();
                    match tk_status {
                        Ok(out) if out.status.success() => {
                            app.set_status(format!(
                                "Successfully force-killed process (PID: {})",
                                pid
                            ));
                        }
                        _ => {
                            app.set_status(format!(
                                "Failed to kill process (PID: {}). Run pulse as Admin?",
                                pid
                            ));
                        }
                    }
                }
                app.kill_confirm_pid = None;
                app.kill_confirm_name = None;
                app.update_metrics();
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Char('q') => {
                app.kill_confirm_pid = None;
                app.kill_confirm_name = None;
                app.set_status("Process termination cancelled.".to_string());
            }
            _ => {}
        }
        return;
    }
    if app.selected_process_details.is_some() {
        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                app.selected_process_details = None;
            }
            _ => {}
        }
        return;
    }
    if app.show_help {
        handle_key_help(app, key.code);
        return;
    }
    if app.show_markdown.is_some() {
        handle_key_markdown(app, key.code);
        return;
    }
    handle_key_main(app, key.code);
}

fn handle_key_help(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc | KeyCode::Char('h') | KeyCode::Char('H') => {
            app.show_help = false;
            app.set_status("Help overlay closed.".to_string());
        }
        KeyCode::F(n) if (1..=DOC_FILES.len() as u8).contains(&n) => {
            let file = DOC_FILES[(n - 1) as usize];
            app.show_help = false;
            open_doc(app, file);
        }
        _ => {}
    }
}

fn handle_key_markdown(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            app.show_markdown = None;
        }
        KeyCode::F(n) if (1..=DOC_FILES.len() as u8).contains(&n) => {
            let file = DOC_FILES[(n - 1) as usize];
            open_doc(app, file);
        }
        KeyCode::Up => app.markdown_scroll = app.markdown_scroll.saturating_sub(1),
        KeyCode::Down => {
            if app.markdown_scroll + 10 < app.markdown_lines.len() {
                app.markdown_scroll += 1;
            }
        }
        KeyCode::PageUp => app.markdown_scroll = app.markdown_scroll.saturating_sub(15),
        KeyCode::PageDown => {
            if app.markdown_scroll + 15 < app.markdown_lines.len() {
                app.markdown_scroll += 15;
            } else {
                app.markdown_scroll = app.markdown_lines.len().saturating_sub(10);
            }
        }
        _ => {}
    }
}

fn handle_key_main(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Enter => {
            if let Some(idx) = app.process_state.selected()
                && idx < app.processes.len() {
                    let item = &app.processes[idx];
                    app.show_process_details(item.pid);
                }
        }
        KeyCode::F(9) | KeyCode::Char('K') | KeyCode::Delete => {
            if let Some(idx) = app.process_state.selected()
                && idx < app.processes.len() {
                    let item = &app.processes[idx];
                    app.kill_confirm_pid = Some(item.pid);
                    app.kill_confirm_name = Some(item.name.clone());
                    app.set_status(format!("Confirm kill: {} (PID: {})? [y/n]", item.name, item.pid));
                }
        }
        KeyCode::Tab => {
            app.focus = match app.focus {
                FocusedSection::Cpu => FocusedSection::Memory,
                FocusedSection::Memory => FocusedSection::Disk,
                FocusedSection::Disk => FocusedSection::Gpu,
                FocusedSection::Gpu => FocusedSection::Network,
                FocusedSection::Network => FocusedSection::Cpu,
            };
            let label = match app.focus {
                FocusedSection::Cpu => "CPU Cores & Processes (sorted by CPU)",
                FocusedSection::Memory => "Memory Maps & Processes (sorted by RAM)",
                FocusedSection::Disk => "Disk Storage & Processes (sorted by Disk I/O)",
                FocusedSection::Gpu => "GPU Adapters & Processes (sorted by GPU)",
                FocusedSection::Network => "Network Interfaces & Processes (sorted by Network)",
            };
            app.set_status(format!("Focused Section: {}", label));
        }
        KeyCode::F(n) if (1..=DOC_FILES.len() as u8).contains(&n) => {
            let file = DOC_FILES[(n - 1) as usize];
            open_doc(app, file);
        }
        KeyCode::Char('h') | KeyCode::Char('H') => {
            app.show_help = true;
            app.set_status("Help overlay active. Press ESC/q to close.".to_string());
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let cur = app.process_state.selected().unwrap_or(0);
            if cur > 0 {
                app.process_state.select(Some(cur - 1));
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let cur = app.process_state.selected().unwrap_or(0);
            if !app.processes.is_empty() && cur < app.processes.len() - 1 {
                app.process_state.select(Some(cur + 1));
            }
        }
        _ => {}
    }
}

fn open_doc(app: &mut App, name: &str) {
    let content: &str = match name {
        "README.md" => include_str!("../../README.md"),
        "SUPPORT.md" => include_str!("../../SUPPORT.md"),
        "LICENSE.md" => include_str!("../../LICENSE.md"),
        "COPYRIGHT.md" => include_str!("../../COPYRIGHT.md"),
        "PRIVACY.md" => include_str!("../../PRIVACY.md"),
        "SECURITY.md" => include_str!("../../SECURITY.md"),
        "CONTRIBUTING.md" => include_str!("../../CONTRIBUTING.md"),
        _ => return,
    };
    let dark = app.power.is_dark();
    let accent = {
        let (r, g, b) = crate::backend::sys_info::query_accent_color();
        ratatui::style::Color::Rgb(r, g, b)
    };
    let theme = crate::ui::theme::get_theme(dark, accent);
    app.markdown_lines = parse_markdown_to_lines(content, &theme);
    app.show_markdown = Some(name.to_string());
    app.markdown_scroll = 0;
    app.set_status(format!("Opened document: {}", name));
}
