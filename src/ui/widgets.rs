//! Details view panels (CPU, Memory, Disk, GPU) rendering.
//!
//! **Taxonomy Classification**: UI Rendering (Detail Panes).

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};
use crate::app::App;

// CPU usage thresholds for color coding
const CPU_HIGH_USAGE: f32 = 80.0;
const CPU_MEDIUM_USAGE: f32 = 40.0;

// Byte conversion constants
const BYTES_PER_GB: f64 = 1024.0 * 1024.0 * 1024.0;

// Re-export network details rendering
pub use crate::ui::widgets_net::render_network_details;

pub fn render_cpu_details(f: &mut Frame, area: Rect, app: &App, border_color: Color) {
    let theme = &*app.theme;
    let inner = area.width as usize;
    let item_w = 10;
    let max_cols = (inner / item_w).max(1);
    let cpus = app.sys.cpus();
    let mut lines = Vec::new();
    for chunk in cpus.chunks(max_cols) {
        let mut line_spans = Vec::new();
        for (i, cpu) in chunk.iter().enumerate() {
            let idx = cpus.iter().position(|x| std::ptr::eq(x, cpu)).unwrap_or(0);
            let usage = cpu.cpu_usage();
            let color = if usage > CPU_HIGH_USAGE {
                Color::Rgb(255, 85, 85)
            } else if usage > CPU_MEDIUM_USAGE {
                Color::Rgb(255, 215, 0)
            } else {
                Color::Rgb(80, 250, 123)
            };
            line_spans.push(Span::styled(
                format!("C{:02}:{:3.0}%", idx, usage),
                Style::default().fg(color),
            ));
            if i < chunk.len() - 1 {
                line_spans.push(Span::styled(" │ ", Style::default().fg(theme.border)));
            }
        }
        lines.push(Line::from(line_spans));
    }
    let p = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" CPU ")
            .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(p, area);
}

pub fn render_memory_details(f: &mut Frame, area: Rect, app: &App, border_color: Color) {
    let pink = Color::Rgb(255, 121, 198);
    let theme = &*app.theme;
    let total_ram = app.sys.total_memory() as f64 / BYTES_PER_GB;
    let used_ram = app.sys.used_memory() as f64 / BYTES_PER_GB;
    let free_ram = app.sys.free_memory() as f64 / BYTES_PER_GB;
    let avail_ram = app.sys.available_memory() as f64 / BYTES_PER_GB;
    let total_swap = app.sys.total_swap() as f64 / BYTES_PER_GB;
    let used_swap = app.sys.used_swap() as f64 / BYTES_PER_GB;
    let free_swap = app.sys.free_swap() as f64 / BYTES_PER_GB;
    let header = Row::new(vec!["Metric Type", "Allocated Value"])
        .style(Style::default().fg(pink).add_modifier(Modifier::BOLD))
        .bottom_margin(1);
    let rows = vec![
        ("Total Physical RAM", format!("{:.2} GB", total_ram)),
        ("Used Physical RAM", format!("{:.2} GB", used_ram)),
        ("Free Physical RAM", format!("{:.2} GB", free_ram)),
        ("Available Physical RAM", format!("{:.2} GB", avail_ram)),
        ("Total Pagefile Swap (Swapfile)", format!("{:.2} GB", total_swap)),
        ("Used Pagefile Swap (Swapfile)", format!("{:.2} GB", used_swap)),
        ("Free Pagefile Swap (Swapfile)", format!("{:.2} GB", free_swap)),
    ]
    .into_iter()
    .map(|(m, v)| {
        Row::new(vec![
            Cell::from(m).style(Style::default().fg(theme.text_main).add_modifier(Modifier::BOLD)),
            Cell::from(v).style(Style::default().fg(pink)),
        ])
    });
    let table = Table::new(rows, [Constraint::Percentage(60), Constraint::Percentage(40)])
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Memory ")
                .title_style(Style::default().fg(pink).add_modifier(Modifier::BOLD))
                .border_style(Style::default().fg(border_color)),
        );
    f.render_widget(table, area);
}

pub fn render_disk_details(f: &mut Frame, area: Rect, app: &App, border_color: Color) {
    let purple = Color::Rgb(160, 32, 240);
    let theme = &*app.theme;
    let header = Row::new(vec!["Partition", "Format", "Used Space", "Total", "Free"])
        .style(Style::default().fg(purple).add_modifier(Modifier::BOLD))
        .bottom_margin(1);
    let rows = app.disks.iter().map(|disk| {
        let total = disk.total_space() as f64 / BYTES_PER_GB;
        let avail = disk.available_space() as f64 / BYTES_PER_GB;
        let used = total - avail;
        Row::new(vec![
            Cell::from(disk.mount_point().to_string_lossy().to_string())
                .style(Style::default().fg(theme.text_main).add_modifier(Modifier::BOLD)),
            Cell::from(disk.file_system().to_string_lossy().to_string())
                .style(Style::default().fg(theme.text_dim)),
            Cell::from(format!("{:.1} GB", used)).style(Style::default().fg(purple)),
            Cell::from(format!("{:.1} GB", total)).style(Style::default().fg(theme.text_dim)),
            Cell::from(format!("{:.1} GB", avail)).style(Style::default().fg(Color::Rgb(80, 250, 123))),
        ])
    });
    let table = Table::new(
        rows,
        [
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Storage ")
            .title_style(Style::default().fg(purple).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(table, area);
}

pub fn render_gpu_details(f: &mut Frame, area: Rect, app: &App, border_color: Color) {
    let gold = Color::Rgb(255, 215, 0);
    let theme = &*app.theme;
    let header = Row::new(vec!["Index", "Display Adapter Desc", "Engine Load"])
        .style(Style::default().fg(gold).add_modifier(Modifier::BOLD))
        .bottom_margin(1);
    let rows = app.gpu_names.iter().enumerate().map(|(idx, name)| {
        let load = if idx == 0 { app.gpu1_spring.value } else { app.gpu2_spring.value };
        Row::new(vec![
            Cell::from(format!("GPU{}", idx + 1)).style(Style::default().fg(theme.text_dim)),
            Cell::from(name.clone()).style(
                Style::default().fg(theme.text_main).add_modifier(Modifier::BOLD),
            ),
            Cell::from(format!("{:.1}%", load)).style(Style::default().fg(gold)),
        ])
    });
    let table = Table::new(
        rows,
        [
            Constraint::Percentage(15),
            Constraint::Percentage(60),
            Constraint::Percentage(25),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" GPU ")
            .title_style(Style::default().fg(gold).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(table, area);
}
