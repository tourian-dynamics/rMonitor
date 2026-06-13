//! Mouse input event handler for pulse.
//!
//! **Taxonomy Classification**: Interface (Presentation Layer).

use crossterm::event::{MouseEventKind, MouseButton};
use crate::backend::window::{get_window_rect, query_cursor_pos, set_window_pos};
use crate::app::App;

pub fn handle_mouse(
    app: &mut App,
    kind: MouseEventKind,
    col: u16,
    row: u16,
) {
    match kind {
        MouseEventKind::Down(MouseButton::Left) => {
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
        MouseEventKind::Drag(MouseButton::Left) => {
            if app.drag.is_active() {
                if let (Some(start_cursor), Some(start_window)) =
                    (app.drag.start_cursor, app.drag.start_window)
                    && let Some(curr) = query_cursor_pos() {
                        let dx = curr.0 - start_cursor.0;
                        let dy = curr.1 - start_cursor.1;
                        set_window_pos(start_window.0 + dx, start_window.1 + dy);
                    }
            } else if app.selection.is_active() {
                app.selection.on_drag(col, row);
            }
        }
        MouseEventKind::Up(MouseButton::Left) => {
            if app.drag.is_active() {
                app.drag.end();
            } else {
                app.selection.on_release();
            }
        }
        #[allow(clippy::collapsible_match, clippy::redundant_pattern_matching)]
        MouseEventKind::ScrollUp => {
            if app.show_markdown.is_some() {
                app.markdown_scroll = app.markdown_scroll.saturating_sub(3);
            }
        }
        #[allow(clippy::collapsible_match, clippy::redundant_pattern_matching)]
        MouseEventKind::ScrollDown => {
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
