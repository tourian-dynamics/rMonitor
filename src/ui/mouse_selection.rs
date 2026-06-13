//! Mouse drag-to-select + clipboard copy helper for ratatui TUIs.
//!
//! **Taxonomy Classification**: Interface (Presentation Layer).

use ratatui::{Frame, style::Color};

/// Mouse-drag selection state.
#[derive(Debug, Default, Clone, Copy)]
pub struct MouseSelection {
    /// `(col, row)` where the left button was pressed.
    pub start: Option<(u16, u16)>,
    /// `(col, row)` where the cursor is now (set during Drag, finalized on Up).
    pub end: Option<(u16, u16)>,
    /// Set true between Up and the next paint, signaling `take_copy_text` should consume it.
    pub pending_copy: bool,
}

impl MouseSelection {
    pub fn new() -> Self {
        Self::default()
    }

    /// User pressed the left button outside the title bar. Begin a fresh selection.
    pub fn on_press(&mut self, col: u16, row: u16) {
        self.start = Some((col, row));
        self.end = Some((col, row));
        self.pending_copy = false;
    }

    /// User dragged while holding the left button. Extend the selection.
    pub fn on_drag(&mut self, col: u16, row: u16) {
        if self.start.is_some() && !self.pending_copy {
            self.end = Some((col, row));
        }
    }

    /// User released the left button. If start != end, request a copy.
    pub fn on_release(&mut self) {
        if let (Some(start), Some(end)) = (self.start, self.end) {
            if start != end {
                self.pending_copy = true;
            } else {
                self.start = None;
                self.end = None;
            }
        }
    }

    /// Returns true if a selection is currently active.
    pub fn is_active(&self) -> bool {
        self.start.is_some() && self.end.is_some()
    }

    /// Highlight the selected cells in the current frame's buffer.
    /// Returns true if a highlight was applied.
    pub fn highlight(&self, f: &mut Frame) -> bool {
        let (Some(start), Some(end)) = (self.start, self.end) else {
            return false;
        };
        let buf = f.buffer_mut();
        let width = buf.area.width;
        let height = buf.area.height;
        let (col1, row1) = start;
        let (col2, row2) = end;
        for y in 0..height {
            for x in 0..width {
                if point_in_rect(x, y, col1, row1, col2, row2) {
                    let cell = &mut buf[(x, y)];
                    cell.set_bg(Color::Rgb(0, 120, 215));
                    cell.set_fg(Color::White);
                }
            }
        }
        true
    }

    /// If a copy is pending, extract the selected text from the frame buffer and
    /// reset the state. Returns `Some(text)` if a copy was performed, `None` otherwise.
    pub fn take_copy_text(&mut self, f: &mut Frame) -> Option<String> {
        if !self.pending_copy {
            return None;
        }
        let (start, end) = (self.start?, self.end?);
        // Highlight uses buffer_mut; for reading the symbol we can borrow the same buffer.
        let buf = f.buffer_mut();
        let width = buf.area.width;
        let height = buf.area.height;
        let (col1, row1) = start;
        let (col2, row2) = end;

        let mut selected_text = String::new();
        let mut current_row: Option<u16> = None;
        let mut current_line = String::new();

        for y in 0..height {
            for x in 0..width {
                if point_in_rect(x, y, col1, row1, col2, row2) {
                    let symbol = buf[(x, y)].symbol().to_string();
                    if current_row != Some(y) {
                        if current_row.is_some() {
                            selected_text.push_str(current_line.trim_end());
                            selected_text.push('\n');
                            current_line.clear();
                        }
                        current_row = Some(y);
                    }
                    current_line.push_str(&symbol);
                }
            }
        }
        if !current_line.is_empty() {
            selected_text.push_str(current_line.trim_end());
        }

        self.start = None;
        self.end = None;
        self.pending_copy = false;

        if selected_text.is_empty() {
            None
        } else {
            Some(selected_text)
        }
    }
}

fn point_in_rect(x: u16, y: u16, col1: u16, row1: u16, col2: u16, row2: u16) -> bool {
    let (row_start, col_start, row_end, col_end) = if row1 < row2 || (row1 == row2 && col1 <= col2) {
        (row1, col1, row2, col2)
    } else {
        (row2, col2, row1, col1)
    };

    if y < row_start || y > row_end {
        false
    } else if row_start == row_end {
        y == row_start && x >= col_start && x <= col_end
    } else if y == row_start {
        x >= col_start
    } else if y == row_end {
        x <= col_end
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state_is_inactive() {
        let s = MouseSelection::new();
        assert!(!s.is_active());
    }

    #[test]
    fn test_press_then_release_same_point_clears() {
        let mut s = MouseSelection::new();
        s.on_press(5, 5);
        s.on_release();
        assert!(!s.is_active());
        assert!(!s.pending_copy);
    }

    #[test]
    fn test_press_then_release_diff_points_pends_copy() {
        let mut s = MouseSelection::new();
        s.on_press(5, 5);
        s.on_drag(10, 5);
        s.on_release();
        assert!(s.pending_copy);
    }
}
