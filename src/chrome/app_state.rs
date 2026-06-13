//! Common key predicates shared by all apps.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Returns true if the (code, mods) pair is a quit-trigger:
///
/// - Ctrl-C
/// - 'q' or 'Q'
/// - Esc
pub fn is_quit_key(code: KeyCode, mods: KeyModifiers) -> bool {
    if mods.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
        return true;
    }
    matches!(code, KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc)
}

/// Convenience: returns true if this `KeyEvent` is a key-press AND it's
/// a quit-trigger.
pub fn is_quit_key_event(key: &KeyEvent) -> bool {
    key.kind == KeyEventKind::Press && is_quit_key(key.code, key.modifiers)
}

/// Returns true if the key toggles the help overlay: 'h', 'H', or F1.
pub fn is_help_toggle_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('h') | KeyCode::Char('H'))
}

/// Apply markdown-viewer scroll math for one keystroke.
pub fn scroll_for_key(
    code: KeyCode,
    scroll: usize,
    line_count: usize,
    viewport_h: usize,
) -> Option<usize> {
    let vp = viewport_h.max(1);
    let max_scroll = line_count.saturating_sub(vp + 10);
    let arrow_step = 1usize;
    let page_step = vp.clamp(1, 15);

    match code {
        KeyCode::Up | KeyCode::Char('k') => Some(scroll.saturating_sub(arrow_step)),
        KeyCode::Down | KeyCode::Char('j') => {
            if scroll < max_scroll {
                Some((scroll + arrow_step).min(max_scroll))
            } else {
                Some(scroll)
            }
        }
        KeyCode::PageUp => Some(scroll.saturating_sub(page_step)),
        KeyCode::PageDown => {
            if scroll < max_scroll {
                Some((scroll + page_step).min(max_scroll))
            } else {
                Some(scroll)
            }
        }
        _ => None,
    }
}
