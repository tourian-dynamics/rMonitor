use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

/// A custom vertical scrollbar highlighted with the accent color.
/// Renders a vertical line with scroll thumb on the right edge of the given area.
#[derive(Debug, Clone)]
pub struct AccentScrollbar {
    pub scroll_position: usize,
    pub content_length: usize,
    pub viewport_height: usize,
    pub accent_color: Color,
    pub dim_color: Color,
}

impl AccentScrollbar {
    pub fn new(
        scroll_position: usize,
        content_length: usize,
        viewport_height: usize,
        accent_color: Color,
        dim_color: Color,
    ) -> Self {
        Self {
            scroll_position,
            content_length,
            viewport_height,
            accent_color,
            dim_color,
        }
    }
}

impl Widget for AccentScrollbar {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        if self.content_length <= self.viewport_height || self.viewport_height == 0 || area.height < 2 {
            return;
        }

        let x = area.x + area.width.saturating_sub(1);
        let y_start = area.y;
        let height = area.height;

        // Draw top and bottom arrows
        if let Some(cell) = buf.cell_mut((x, y_start)) {
            cell.set_symbol("▲");
            cell.set_style(Style::default().fg(self.dim_color));
        }
        if let Some(cell) = buf.cell_mut((x, y_start + height - 1)) {
            cell.set_symbol("▼");
            cell.set_style(Style::default().fg(self.dim_color));
        }

        // Draw track and thumb
        let track_height = height.saturating_sub(2) as usize;
        if track_height == 0 {
            return;
        }

        // Math for thumb position
        let max_scroll = self.content_length.saturating_sub(self.viewport_height);
        let thumb_ratio = (self.viewport_height as f64 / self.content_length as f64).clamp(0.1, 1.0);
        let thumb_height = ((track_height as f64 * thumb_ratio) as usize).clamp(1, track_height);
        
        let scroll_ratio = if max_scroll > 0 {
            self.scroll_position as f64 / max_scroll as f64
        } else {
            0.0
        };
        let thumb_start_offset = ((track_height - thumb_height) as f64 * scroll_ratio) as usize;

        for i in 0..track_height {
            let cell_y = y_start + 1 + i as u16;
            if let Some(cell) = buf.cell_mut((x, cell_y)) {
                if i >= thumb_start_offset && i < thumb_start_offset + thumb_height {
                    // Thumb
                    cell.set_symbol("█");
                    cell.set_style(Style::default().fg(self.accent_color));
                } else {
                    // Track
                    cell.set_symbol("│");
                    cell.set_style(Style::default().fg(self.dim_color));
                }
            }
        }
    }
}
