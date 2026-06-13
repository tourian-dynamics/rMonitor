//! Markdown parser and renderer widgets for ratatui TUIs.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use super::theme::ThemeColors;

/// A lightweight, custom terminal markdown parser returning styled console Spans and Lines.
pub fn parse_markdown_to_lines(content: &str, theme: &ThemeColors) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut in_code_block = false;
    let mut current_paragraph = String::new();

    let flush_paragraph = |para: &mut String, lines: &mut Vec<Line<'static>>| {
        if !para.is_empty() {
            lines.push(Line::from(Span::styled(
                para.clone(),
                Style::default().fg(theme.text_main),
            )));
            para.clear();
        }
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(Color::Rgb(150, 240, 150)),
            )));
            continue;
        }

        if trimmed.is_empty() {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(""));
            continue;
        }

        if let Some(header) = trimmed.strip_prefix("# ") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("=== {} ===", header.to_uppercase()),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
        } else if let Some(header) = trimmed.strip_prefix("## ") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("--- {} ---", header),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
        } else if let Some(header) = trimmed.strip_prefix("### ") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(Span::styled(
                header.to_string(),
                Style::default().fg(theme.accent),
            )));
        } else if let Some(item) = trimmed
            .strip_prefix("* ")
            .or_else(|| trimmed.strip_prefix("- "))
        {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(vec![
                Span::styled(" • ", Style::default().fg(theme.accent)),
                Span::styled(item.to_string(), Style::default().fg(theme.text_main)),
            ]));
        } else if let Some((num_str, rest)) = trimmed.split_once(". ").filter(|(num_str, _)| !num_str.is_empty() && num_str.chars().all(|c| c.is_ascii_digit())) {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {}. ", num_str),
                    Style::default().fg(theme.accent),
                ),
                Span::styled(
                    rest.to_string(),
                    Style::default().fg(theme.text_main),
                ),
            ]));
        } else if let Some(quote) = trimmed.strip_prefix("> ") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(Span::styled(
                format!("  │ {}", quote),
                Style::default()
                    .fg(theme.text_dim)
                    .add_modifier(Modifier::ITALIC),
            )));
        } else {
            if !current_paragraph.is_empty() {
                current_paragraph.push(' ');
            }
            current_paragraph.push_str(trimmed);
        }
    }
    flush_paragraph(&mut current_paragraph, &mut lines);
    lines
}

