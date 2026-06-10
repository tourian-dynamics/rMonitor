//! Draw helpers shared between the pulse panels and modals.

use ratatui::style::Color;
use ratatui::text::Line;

// Time constants (in seconds)
const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 3600;
const SECONDS_PER_DAY: u64 = 86400;

// Byte constants
const BYTES_PER_KB: f64 = 1024.0;
const BYTES_PER_MB: f64 = 1024.0 * 1024.0;
const BYTES_PER_GB: f64 = 1024.0 * 1024.0 * 1024.0;

// Fractional block constants for smooth rendering
const FRACTION_STEPS: usize = 8;
const FRACTION_STEP_VALUE: f64 = 8.0;

/// Format an uptime in seconds as "Xd Yh Zm" / "Yh Zm" / "Zm".
pub fn format_uptime(secs: u64) -> String {
    let days = secs / SECONDS_PER_DAY;
    let hours = (secs % SECONDS_PER_DAY) / SECONDS_PER_HOUR;
    let minutes = (secs % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
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
    if width == 0 || max == 0.0 {
        return String::new();
    }
    let pct = (value / max).clamp(0.0, 1.0);
    let blocks_count_float = pct * width as f64;
    let total_blocks = blocks_count_float as usize;
    let mut bar = "█".repeat(total_blocks);
    if total_blocks < width as usize {
        let fraction = blocks_count_float - total_blocks as f64;
        let fraction_idx = (fraction * FRACTION_STEP_VALUE) as usize;
        const BLOCKS: [char; 9] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
        bar.push(BLOCKS[fraction_idx.clamp(0, FRACTION_STEPS)]);
    }
    while bar.chars().count() < width as usize {
        bar.push(' ');
    }
    bar
}

/// Format a bytes-per-second value with adaptive units.
pub fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= BYTES_PER_MB {
        format!("{:.1} MB/s", bytes_per_sec / BYTES_PER_MB)
    } else if bytes_per_sec >= BYTES_PER_KB {
        format!("{:.1} KB/s", bytes_per_sec / BYTES_PER_KB)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}

/// Format a byte total with adaptive units.
pub fn format_total_bytes(bytes: u64) -> String {
    let kb = bytes as f64 / BYTES_PER_KB;
    let mb = kb / BYTES_PER_KB;
    let gb = mb / BYTES_PER_KB;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(30), "0m");
        assert_eq!(format_uptime(90), "1m");
        assert_eq!(format_uptime(3600), "1h 0m");
        assert_eq!(format_uptime(3665), "1h 1m");
        assert_eq!(format_uptime(86400), "1d 0h 0m");
        assert_eq!(format_uptime(90065), "1d 1h 1m");
    }

    #[test]
    fn test_draw_spring_bar() {
        let bar_empty = draw_spring_bar(0, 10.0, 10.0);
        assert!(bar_empty.is_empty());

        let bar_zero_max = draw_spring_bar(10, 10.0, 0.0);
        assert!(bar_zero_max.is_empty());

        let bar_full = draw_spring_bar(5, 10.0, 10.0);
        assert_eq!(bar_full, "█████");

        let bar_half = draw_spring_bar(4, 5.0, 10.0);
        assert_eq!(bar_half, "██  ");
    }

    #[test]
    fn test_format_speed() {
        assert_eq!(format_speed(500.0), "500 B/s");
        assert_eq!(format_speed(1024.0), "1.0 KB/s");
        assert_eq!(format_speed(1536.0), "1.5 KB/s");
        assert_eq!(format_speed(1024.0 * 1024.0), "1.0 MB/s");
        assert_eq!(format_speed(1.5 * 1024.0 * 1024.0), "1.5 MB/s");
    }

    #[test]
    fn test_format_total_bytes() {
        assert_eq!(format_total_bytes(500), "500 B");
        assert_eq!(format_total_bytes(1024), "1.0 KB");
        assert_eq!(format_total_bytes(1536), "1.5 KB");
        assert_eq!(format_total_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_total_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_accent_color_from_hex() {
        assert_eq!(accent_color_from_hex("#ff0000".to_string()), Color::Rgb(255, 0, 0));
        assert_eq!(accent_color_from_hex("#00f5ff".to_string()), Color::Rgb(0, 245, 255));
        assert_eq!(accent_color_from_hex("invalid".to_string()), Color::Rgb(0, 245, 255));
    }
}

