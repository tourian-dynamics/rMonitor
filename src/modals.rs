//! Modal overlays: status bar, process details, kill-confirm, help, markdown.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::{App, ProcessDetails};
use crate::helpers::format_uptime;

pub fn render_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let theme = &*app.theme;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            " Status ",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let (text_color, status_text) = if app.status.is_default() {
        (theme.text_dim, "Ready. Press Tab to cycle focus.".to_string())
    } else {
        let msg = app.status.current();
        let lower = msg.to_lowercase();
        let color = if lower.contains("failed") || lower.contains("error") {
            theme.warning
        } else {
            theme.accent
        };
        (color, msg.to_string())
    };
    let p = Paragraph::new(Line::from(vec![Span::styled(
        status_text,
        Style::default().fg(text_color).add_modifier(Modifier::BOLD),
    )]));
    f.render_widget(p, inner);
}

pub fn render_process_details_modal(
    f: &mut Frame,
    area: Rect,
    details: &ProcessDetails,
) {
    use library::interface::tui::layout::centered_rect;
    let area = centered_rect(70, 60, area);
    let theme = library::interface::tui::theme::get_theme(true, Color::Rgb(0, 245, 255));
    let parent_str = details
        .parent_pid
        .map(|p| p.to_string())
        .unwrap_or_else(|| "None".to_string());
    let uptime_str = format_uptime(details.run_time);
    let mem_mb = details.mem as f64 / 1024.0 / 1024.0;
    let pink = Color::Rgb(255, 121, 198);
    let green = Color::Rgb(80, 250, 123);
    let lines = vec![
        Line::from(vec![
            Span::styled("PID:         ", Style::default().fg(theme.text_dim)),
            Span::styled(details.pid.to_string(), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Name:        ", Style::default().fg(theme.text_dim)),
            Span::styled(details.name.clone(), Style::default().fg(theme.text_main).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Parent PID:  ", Style::default().fg(theme.text_dim)),
            Span::styled(parent_str, Style::default().fg(theme.text_main)),
        ]),
        Line::from(vec![
            Span::styled("Status:      ", Style::default().fg(theme.text_dim)),
            Span::styled(details.status.clone(), Style::default().fg(green).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("CPU Usage:   ", Style::default().fg(theme.text_dim)),
            Span::styled(format!("{:.1}%", details.cpu), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Memory RSS:  ", Style::default().fg(theme.text_dim)),
            Span::styled(format!("{:.1} MB", mem_mb), Style::default().fg(pink).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Uptime:      ", Style::default().fg(theme.text_dim)),
            Span::styled(uptime_str, Style::default().fg(theme.text_main)),
        ]),
        Line::from(vec![
            Span::styled("Executable:  ", Style::default().fg(theme.text_dim)),
            Span::styled(details.exe.clone(), Style::default().fg(theme.text_dim)),
        ]),
        Line::from(vec![
            Span::styled("Command:     ", Style::default().fg(theme.text_dim)),
            Span::styled(details.cmdline.clone(), Style::default().fg(theme.text_dim)),
        ]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(
            " Press Esc, Enter, or Q to Close ",
            Style::default().fg(Color::Rgb(30, 30, 46)).bg(theme.accent).add_modifier(Modifier::BOLD),
        )]),
    ];
    let modal = Block::default()
        .borders(Borders::ALL)
        .title(" Process Details ")
        .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .border_style(Style::default().fg(theme.accent));
    let p = Paragraph::new(lines)
        .block(modal)
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(Color::Rgb(20, 20, 30)));
    f.render_widget(Clear, area);
    f.render_widget(p, area);
}

pub fn render_kill_confirm_modal(f: &mut Frame, parent: Rect, pid_val: u32, name_val: &str) {
    use library::interface::tui::layout::centered_rect;
    let area = centered_rect(55, 20, parent);
    let theme = library::interface::tui::theme::get_theme(true, Color::Rgb(0, 245, 255));
    let lines = vec![
        Line::from(vec![Span::styled(
            "You are about to terminate the process:",
            Style::default().fg(theme.text_main),
        )]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![
            Span::styled("  Name: ", Style::default().fg(theme.text_dim)),
            Span::styled(name_val.to_string(), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  PID:  ", Style::default().fg(theme.text_dim)),
            Span::styled(pid_val.to_string(), Style::default().fg(theme.accent)),
        ]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(
            " Are you sure? Press [y] to kill, [n] to cancel ",
            Style::default().fg(Color::Rgb(30, 30, 46)).bg(Color::Rgb(255, 85, 85)).add_modifier(Modifier::BOLD),
        )]),
    ];
    let modal = Block::default()
        .borders(Borders::ALL)
        .title(" Terminate Process? ")
        .title_style(Style::default().fg(Color::Rgb(255, 85, 85)).add_modifier(Modifier::BOLD))
        .border_style(Style::default().fg(Color::Rgb(255, 85, 85)));
    let p = Paragraph::new(lines)
        .block(modal)
        .style(Style::default().bg(Color::Rgb(20, 20, 30)));
    f.render_widget(Clear, area);
    f.render_widget(p, area);
}

pub fn render_help_modal(f: &mut Frame, parent: Rect) {
    use library::interface::tui::layout::{centered_rect, format_help_row};
    let area = centered_rect(65, 75, parent);
    let theme = library::interface::tui::theme::get_theme(true, Color::Rgb(0, 245, 255));
    let popup = Block::default()
        .title(" Keyboard Shortcuts & TUI Commands ")
        .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent));

    let key_col_width = 18;
    let border_padding = 2;
    let total_inner_width = area.width.saturating_sub(border_padding);
    let max_desc_width = (total_inner_width as usize)
        .saturating_sub(key_col_width)
        .saturating_sub(2);

    let mut help_text = Vec::new();
    help_text.push(Line::from(""));
    for (key, desc) in [
        ("Tab", "Cycle active panel focus"),
        ("Enter", "View detailed metrics of the selected process"),
        ("F9 / K / Del", "Terminate (kill) the selected process"),
        ("Esc / q", "Close dialogs / Help Overlay, or Quit application"),
        ("h", "Toggle this help shortcut overlay modal"),
        ("Left Click/Drag", "Highlight text to copy to Windows Clipboard"),
    ] {
        help_text.extend(format_help_row(key, desc, max_desc_width, &theme));
    }
    help_text.push(Line::from(""));
    for (idx, file) in DOC_FILES.iter().enumerate() {
        let key = format!("F{}", idx + 1);
        let desc = format!("View {file} document");
        help_text.extend(format_help_row(&key, &desc, max_desc_width, &theme));
    }
    f.render_widget(Clear, area);
    let p = Paragraph::new(help_text).block(popup);
    f.render_widget(p, area);
}

pub fn render_markdown_modal(f: &mut Frame, parent: Rect, app: &App) {
    use library::interface::tui::layout::centered_rect;
    if let Some(filename) = app.show_markdown.clone() {
        let area = centered_rect(85, 80, parent);
        f.render_widget(Clear, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(0, 245, 255)));
        let paragraph = Paragraph::new(app.markdown_lines.clone())
            .block(block)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left)
            .scroll((app.markdown_scroll as u16, 0));
        f.render_widget(paragraph, area);
        let _ = filename;
    }
}

/// A constant slice of the docs F1..F7 open, used by the help modal.
pub const DOC_FILES: &[&str] = &[
    "README.md",
    "SUPPORT.md",
    "LICENSE.md",
    "COPYRIGHT.md",
    "PRIVACY.md",
    "SECURITY.md",
    "CONTRIBUTING.md",
];
