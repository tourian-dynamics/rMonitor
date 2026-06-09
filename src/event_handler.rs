//! Top-level draw dispatcher and keyboard/mouse event handling.
//!
//! **Taxonomy Classification**: Interface (TUI / Presentation Layer).

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton};
use ratatui::{layout::Constraint, layout::Direction, layout::Layout, Frame};
use library::clipboard::copy_text_to_clipboard;
use library::interface::tui::markdown::parse_markdown_to_lines;
use library::interface::tui::theme::ThemeColors;
use library::interface::tui::widgets::{
    draw_title_banner, is_too_small, render_too_small_warning,
};

use crate::app::{App, FocusedSection};
use crate::helpers::accent_color_from_hex;
use crate::modals::DOC_FILES;
use crate::win32;

const MIN_W: u16 = 100;
const MIN_H: u16 = 35;

pub fn current_theme(app: &App) -> ThemeColors {
    let accent = accent_color_from_hex(win32::get_win_accent_color());
    library::interface::tui::theme::get_theme(app.power.is_dark(), accent)
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.area();
    let theme = current_theme(app);

    if is_too_small(size, (MIN_W, MIN_H)) {
        render_too_small_warning(
            f,
            size,
            (size.width, size.height),
            (MIN_W, MIN_H),
            " ⚠️  Terminal Sizing Warning ",
            theme.warning,
        );
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title banner
            Constraint::Length(6), // Top stat cards
            Constraint::Min(10),   // Full-width context table
            Constraint::Length(3), // Footer status bar
        ])
        .split(size);

    let (help_btn, quit_btn) = draw_title_banner(
        f,
        chunks[0],
        &theme,
        " Rust System Monitor ",
        "pulse",
        env!("CARGO_PKG_VERSION"),
        &app.username,
        &app.host_name,
        &app.os_str,
    );
    app.help_btn = help_btn;
    app.quit_btn = quit_btn;

    crate::panels::render(f, app, chunks.clone());
    crate::modals::render_status_bar(f, chunks[3], app);

    if let Some(details) = app.selected_process_details.clone() {
        crate::modals::render_process_details_modal(f, size, &details);
    }
    if let Some(pid_val) = app.kill_confirm_pid {
        let name = app
            .kill_confirm_name
            .clone()
            .unwrap_or_else(|| "<unknown>".to_string());
        crate::modals::render_kill_confirm_modal(f, size, pid_val, &name);
    }
    if app.show_help {
        crate::modals::render_help_modal(f, size);
    }
    if app.show_markdown.is_some() {
        crate::modals::render_markdown_modal(f, size, app);
    }

    if app.selection.is_active() {
        app.selection.highlight(f);
    }
    if let Some(text) = app.selection.take_copy_text(f) {
        let _ = copy_text_to_clipboard(&text);
        let preview = if text.len() > 30 {
            format!("{}...", &text[..27].replace('\n', " "))
        } else {
            text.replace('\n', " ")
        };
        app.set_status(format!("📋 Copied selection to clipboard: {}", preview));
    }
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }
    if let Some(pid) = app.kill_confirm_pid {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                let sys_pid = sysinfo::Pid::from_u32(pid);
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
            if let Some(idx) = app.process_state.selected() {
                if idx < app.processes.len() {
                    let item = &app.processes[idx];
                    app.show_process_details(item.pid);
                }
            }
        }
        KeyCode::F(9) | KeyCode::Char('K') | KeyCode::Delete => {
            if let Some(idx) = app.process_state.selected() {
                if idx < app.processes.len() {
                    let item = &app.processes[idx];
                    app.kill_confirm_pid = Some(item.pid);
                    app.kill_confirm_name = Some(item.name.clone());
                    app.set_status(format!("Confirm kill: {} (PID: {})? [y/n]", item.name, item.pid));
                }
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
        "README.md" => include_str!("../README.md"),
        "SUPPORT.md" => include_str!("../SUPPORT.md"),
        "LICENSE.md" => include_str!("../LICENSE.md"),
        "COPYRIGHT.md" => include_str!("../COPYRIGHT.md"),
        "PRIVACY.md" => include_str!("../PRIVACY.md"),
        "SECURITY.md" => include_str!("../SECURITY.md"),
        "CONTRIBUTING.md" => include_str!("../CONTRIBUTING.md"),
        _ => return,
    };
    let dark = app.power.is_dark();
    let accent = {
        let (r, g, b) = library::platform::native::sys_info::query_accent_color();
        ratatui::style::Color::Rgb(r, g, b)
    };
    let theme = library::interface::tui::theme::get_theme(dark, accent);
    app.markdown_lines = parse_markdown_to_lines(content, &theme);
    app.show_markdown = Some(name.to_string());
    app.markdown_scroll = 0;
    app.set_status(format!("Opened document: {}", name));
}

pub fn handle_mouse(
    app: &mut App,
    kind: crossterm::event::MouseEventKind,
    col: u16,
    row: u16,
) {
    use library::lifecycle::foreground::window::{get_window_rect, query_cursor_pos, set_window_pos};
    match kind {
        crossterm::event::MouseEventKind::Down(MouseButton::Left) => {
            let mut clicked = false;
            #[allow(clippy::collapsible_if)]
            if let Some(btn) = app.quit_btn {
                if btn.contains(row, col) {
                    app.should_quit = true;
                    clicked = true;
                }
            }
            #[allow(clippy::collapsible_if)]
            if !clicked && let Some(btn) = app.help_btn {
                if btn.contains(row, col) {
                    app.show_help = !app.show_help;
                    app.set_status(if app.show_help {
                        "Help overlay active. Press ESC/q to close.".to_string()
                    } else {
                        "Help overlay closed.".to_string()
                    });
                    clicked = true;
                }
            }
            if !clicked {
                if mouse_row_is_title(row) {
                    if let (Some(cursor), Some(rect)) = (query_cursor_pos(), get_window_rect()) {
                        app.drag.active = true;
                        app.drag.start_cursor = Some(cursor);
                        app.drag.start_window = Some((rect.left, rect.top));
                    }
                } else {
                    app.selection.on_press(col, row);
                }
            }
        }
        crossterm::event::MouseEventKind::Drag(MouseButton::Left) => {
            if app.drag.is_active() {
                if let (Some(start_cursor), Some(start_window)) =
                    (app.drag.start_cursor, app.drag.start_window)
                {
                    if let Some(curr) = query_cursor_pos() {
                        let dx = curr.0 - start_cursor.0;
                        let dy = curr.1 - start_cursor.1;
                        set_window_pos(start_window.0 + dx, start_window.1 + dy);
                    }
                }
            } else if app.selection.is_active() {
                app.selection.on_drag(col, row);
            }
        }
        crossterm::event::MouseEventKind::Up(MouseButton::Left) => {
            if app.drag.is_active() {
                app.drag.end();
            } else {
                app.selection.on_release();
            }
        }
        #[allow(clippy::collapsible_match, clippy::redundant_pattern_matching)]
        crossterm::event::MouseEventKind::ScrollUp => {
            if app.show_markdown.is_some() {
                app.markdown_scroll = app.markdown_scroll.saturating_sub(3);
            }
        }
        #[allow(clippy::collapsible_match, clippy::redundant_pattern_matching)]
        crossterm::event::MouseEventKind::ScrollDown => {
            if app.show_markdown.is_some() {
                let max = app.markdown_lines.len().saturating_sub(10);
                if app.markdown_scroll < max {
                    app.markdown_scroll = (app.markdown_scroll + 3).min(max);
                }
            }
        }
        _ => {}
    }
}

fn mouse_row_is_title(row: u16) -> bool {
    row <= 2
}
