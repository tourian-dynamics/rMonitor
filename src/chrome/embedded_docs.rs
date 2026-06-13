//! F1..F7 documentation bindings + lookup helpers.

use crossterm::event::KeyCode;

/// Filenames bound to F1..F7 in display order.
pub const DOC_FILES: &[&str] = &[
    "README.md",
    "SUPPORT.md",
    "LICENSE.md",
    "COPYRIGHT.md",
    "PRIVACY.md",
    "SECURITY.md",
    "CONTRIBUTING.md",
];

/// Total number of F-key docs.
pub const DOC_COUNT: u8 = 7;

/// Returns the doc filename for F-key `n` (1..=7), or None otherwise.
pub fn doc_for_f_key(n: u8) -> Option<&'static str> {
    if n >= 1 && n <= DOC_COUNT {
        Some(DOC_FILES[(n - 1) as usize])
    } else {
        None
    }
}

/// Returns the doc filename if `code` is one of F1..F7.
pub fn is_doc_f_key(code: KeyCode) -> Option<&'static str> {
    if let KeyCode::F(n) = code {
        doc_for_f_key(n)
    } else {
        None
    }
}

/// Convenience: returns Some(filename) if this key is F1..F7. Otherwise None.
pub fn open_embedded_markdown(code: KeyCode) -> Option<&'static str> {
    is_doc_f_key(code)
}

/// Look up a doc filename by exact name. Returns None if `name` is not in
/// the F1..F7 list.
pub fn doc(name: &str) -> Option<&'static str> {
    DOC_FILES.iter().find(|&&f| f == name).copied()
}
