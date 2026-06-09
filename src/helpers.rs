//! Draw helpers shared between the pulse panels and modals.

use ratatui::style::Color;
use ratatui::text::Line;

/// Format an uptime in seconds as "Xd Yh Zm" / "Yh Zm" / "Zm".
pub fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

/// Render a horizontal spring bar with block characters.
pub fn draw_spring_bar(width: u16, value: f64, max: f64) -> String {
    if width == 0 {
        return String::new();
    }
    let pct = (value / max).clamp(0.0, 1.0);
    let blocks_count_float = pct * width as f64;
    let total_blocks = blocks_count_float as usize;
    let mut bar = "█".repeat(total_blocks);
    if total_blocks < width as usize {
        let fraction = blocks_count_float - total_blocks as f64;
        let fraction_idx = (fraction * 8.0) as usize;
        const BLOCKS: [char; 9] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
        bar.push(BLOCKS[fraction_idx.clamp(0, 8)]);
    }
    while bar.chars().count() < width as usize {
        bar.push(' ');
    }
    bar
}

/// Format a bytes-per-second value with adaptive units.
pub fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1024.0 * 1024.0 {
        format!("{:.1} MB/s", bytes_per_sec / 1024.0 / 1024.0)
    } else if bytes_per_sec >= 1024.0 {
        format!("{:.1} KB/s", bytes_per_sec / 1024.0)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}

/// Format a byte total with adaptive units.
pub fn format_total_bytes(bytes: u64) -> String {
    let kb = bytes as f64 / 1024.0;
    let mb = kb / 1024.0;
    let gb = mb / 1024.0;
    if gb >= 1.0 {
        format!("{:.2} GB", gb)
    } else if mb >= 1.0 {
        format!("{:.1} MB", mb)
    } else if kb >= 1.0 {
        format!("{:.1} KB", kb)
    } else {
        format!("{} B", bytes)
    }
}

/// Compose a single GPU name + percentage line (truncated to fit card width).
pub fn format_gpu_line(label: &str, name: &str, pct: f64, width: usize) -> Line<'static> {
    let pct_str = format!(" {:5.1}%", pct);
    let avail_w = width.saturating_sub(label.len() + pct_str.len() + 3);
    let mut display_name = name.to_string();
    display_name = display_name
        .replace("GeForce ", "")
        .replace("Laptop GPU", "")
        .replace("Graphics", "Gfx")
        .replace("Corporation", "")
        .replace("Intel(R) ", "")
        .replace("NVIDIA ", "");
    if display_name.len() > avail_w && avail_w > 4 {
        display_name = display_name[0..avail_w - 3].to_string() + "...";
    } else if display_name.len() > avail_w {
        display_name.truncate(avail_w);
    }
    Line::from(vec![
        ratatui::text::Span::styled(
            format!("{}: ", label),
            ratatui::style::Style::default().fg(Color::Rgb(136, 136, 153)),
        ),
        ratatui::text::Span::styled(
            display_name,
            ratatui::style::Style::default().fg(Color::Rgb(248, 248, 242)),
        ),
        ratatui::text::Span::styled(
            pct_str,
            ratatui::style::Style::default()
                .fg(Color::Rgb(255, 215, 0))
                .add_modifier(ratatui::style::Modifier::BOLD),
        ),
    ])
}

/// Convert a hex color string to a `Color::Rgb`.
pub fn accent_color_from_hex(hex: String) -> Color {
    if hex.starts_with('#') && hex.len() == 7 {
        let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(245);
        let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(255);
        Color::Rgb(r, g, b)
    } else {
        Color::Rgb(0, 245, 255)
    }
}
