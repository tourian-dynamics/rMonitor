//! Terminal-too-small layout guard widget for ratatui TUIs.
//!
//! **Taxonomy Classification**: Interface (Presentation Layer).

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::ui::layout_helpers::centered_rect;

/// Renders a centered "Terminal too small" warning. Returns early after drawing so
/// the caller can `return;` from its `draw_ui` closure.
#[allow(clippy::too_many_arguments)]
pub fn render_too_small_warning(
    f: &mut Frame,
    area: Rect,
    current_size: (u16, u16),
    min_size: (u16, u16),
    title: &str,
    accent_color: Color,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent_color))
        .title(Span::styled(
            title,
            Style::default().fg(accent_color).add_modifier(Modifier::BOLD),
        ));

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Layout Constraints Not Met",
            Style::default().fg(accent_color).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("  Current Terminal Size: {}x{}", current_size.0, current_size.1)),
        Line::from(format!("  Minimum Required Size: {}x{}", min_size.0, min_size.1)),
        Line::from(""),
        Line::from(
            "  Please resize or maximize your terminal window to resume standard rendering.",
        ),
    ];

    let popup = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    let popup_area = centered_rect(80, 50, area);
    f.render_widget(Clear, popup_area);
    f.render_widget(popup, popup_area);
}

/// Returns true if the given area is below the minimum required size.
pub fn is_too_small(area: Rect, min: (u16, u16)) -> bool {
    area.width < min.0 || area.height < min.1
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn test_is_too_small_below_min() {
        assert!(is_too_small(Rect::new(0, 0, 80, 20), (100, 35)));
    }

    #[test]
    fn test_is_too_small_at_min() {
        assert!(!is_too_small(Rect::new(0, 0, 100, 35), (100, 35)));
    }

    #[test]
    fn test_is_too_small_above_min() {
        assert!(!is_too_small(Rect::new(0, 0, 200, 80), (100, 35)));
    }
}
