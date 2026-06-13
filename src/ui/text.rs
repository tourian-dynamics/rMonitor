//! Text wrapping and paragraph alignment utility helpers.

/// Wraps text into lines that do not exceed `max_width` characters, wrapping at word boundaries.
/// Maintains existing explicit newlines from the input.
pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    if max_width == 0 {
        return vec![text.to_string()];
    }
    
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut current_line = String::new();
        let mut current_len = 0;
        for word in paragraph.split_whitespace() {
            let word_len = word.chars().count();
            if current_line.is_empty() {
                if word_len > max_width {
                    let chars: Vec<char> = word.chars().collect();
                    let mut start = 0;
                    while start < chars.len() {
                        let end = (start + max_width).min(chars.len());
                        lines.push(chars[start..end].iter().collect());
                        start = end;
                    }
                } else {
                    current_line.push_str(word);
                    current_len = word_len;
                }
            } else if current_len + 1 + word_len <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
                current_len += 1 + word_len;
            } else {
                lines.push(current_line);
                current_line = word.to_string();
                current_len = word_len;
                if current_len > max_width {
                    let chars: Vec<char> = current_line.chars().collect();
                    let mut start = 0;
                    while start < chars.len() {
                        let end = (start + max_width).min(chars.len());
                        lines.push(chars[start..end].iter().collect());
                        start = end;
                    }
                    current_line.clear();
                    current_len = 0;
                }
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        } else if paragraph.is_empty() {
            lines.push(String::new());
        }
    }
    lines
}
