//! Layout and positioning utility helpers for ratatui TUIs.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
};
use crate::ui::theme::ThemeColors;
use crate::ui::text::wrap_text;

/// Represents the boundary coordinates of a clickable button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonRect {
    pub y: u16,
    pub x_start: u16,
    pub x_end: u16,
}

impl ButtonRect {
    pub fn new(y: u16, x_start: u16, x_end: u16) -> Self {
        Self { y, x_start, x_end }
    }

    /// Checks if a mouse event coordinate falls inside the button boundary.
    pub fn contains(&self, mouse_row: u16, mouse_col: u16) -> bool {
        mouse_row == self.y && mouse_col >= self.x_start && mouse_col < self.x_end
    }
}


/// Center a rect of specified percentage width and height inside another rect.
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Helper that wraps and aligns a keyboard shortcut helper line.
pub fn format_help_row(
    key: &str,
    description: &str,
    max_desc_width: usize,
    theme: &ThemeColors,
) -> Vec<Line<'static>> {
    let wrapped = wrap_text(description, max_desc_width);
    let mut lines = Vec::new();

    let key_col_width = 18;
    let key_str = format!("  {:<15} ", key);

    if wrapped.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                key_str,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": ", Style::default().fg(theme.text_main)),
        ]));
    } else {
        for (i, chunk) in wrapped.into_iter().enumerate() {
            if i == 0 {
                lines.push(Line::from(vec![
                    Span::styled(
                        key_str.clone(),
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(": ", Style::default().fg(theme.text_main)),
                    Span::styled(chunk, Style::default().fg(theme.text_main)),
                ]));
            } else {
                let padding = " ".repeat(key_col_width + 2);
                lines.push(Line::from(vec![
                    Span::styled(padding, Style::default().fg(theme.text_main)),
                    Span::styled(chunk, Style::default().fg(theme.text_main)),
                ]));
            }
        }
    }
    lines
}
