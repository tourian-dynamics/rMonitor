//! Network details rendering for pulse.

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};
use crate::app::App;
use crate::metrics_format::{format_speed, format_total_bytes};

pub fn render_network_details(f: &mut Frame, area: Rect, app: &App, border_color: Color) {
    let green = Color::Rgb(80, 250, 123);
    let theme = &*app.theme;

    let sub_chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(area);

    let header = Row::new(vec![
        "Interface",
        "Status",
        "MAC Address",
        "RX Delta",
        "TX Delta",
        "Total RX",
        "Total TX",
    ])
    .style(Style::default().fg(green).add_modifier(Modifier::BOLD))
    .bottom_margin(1);

    let mut nets: Vec<(&String, &crate::backend::sysinfo_shim::NetworkData)> = app.networks.iter().collect();
    nets.sort_by(|a, b| {
        let s_a = app.net_statuses.get(a.0).map(|s| s.as_str()).unwrap_or("Disconnected");
        let s_b = app.net_statuses.get(b.0).map(|s| s.as_str()).unwrap_or("Disconnected");
        let c_a = s_a == "Connected";
        let c_b = s_b == "Connected";
        if c_a != c_b { c_b.cmp(&c_a) } else { a.0.cmp(b.0) }
    });

    let rows = nets.into_iter().map(|(name, data)| {
        let mac = data.mac_address().to_string();
        let rx_delta = format_speed(data.received() as f64 / 1.5);
        let tx_delta = format_speed(data.transmitted() as f64 / 1.5);
        let rx_total = format_total_bytes(data.total_received());
        let tx_total = format_total_bytes(data.total_transmitted());
        let status_str = app
            .net_statuses
            .get(name)
            .map(|s| s.as_str())
            .unwrap_or("Disconnected");
        let status_cell = if status_str == "Connected" {
            Cell::from("Plugged").style(
                Style::default().fg(green).add_modifier(Modifier::BOLD),
            )
        } else {
            Cell::from("Disconnected").style(Style::default().fg(theme.text_dim))
        };
        Row::new(vec![
            Cell::from(name.clone()).style(
                Style::default().fg(theme.text_main).add_modifier(Modifier::BOLD),
            ),
            status_cell,
            Cell::from(mac).style(Style::default().fg(theme.text_dim)),
            Cell::from(rx_delta).style(Style::default().fg(green)),
            Cell::from(tx_delta).style(Style::default().fg(Color::Rgb(255, 215, 0))),
            Cell::from(rx_total).style(Style::default().fg(theme.text_dim)),
            Cell::from(tx_total).style(Style::default().fg(theme.text_dim)),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(20),
            Constraint::Percentage(12),
            Constraint::Percentage(12),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Network Interfaces ")
            .title_style(Style::default().fg(green).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(table, sub_chunks[0]);

    // Render eBPF tracked connections on the right panel
    let connections = app.ebpf.get_active_connections();
    let conn_text = if connections.is_empty() {
        "No active connections tracked (or not supported on this OS).".to_string()
    } else {
        connections.join("\n")
    };
    let ebpf_block = Block::default()
        .borders(Borders::ALL)
        .title(" Active Connections (eBPF) ")
        .title_style(Style::default().fg(green).add_modifier(Modifier::BOLD))
        .border_style(Style::default().fg(border_color));
    let ebpf_p = Paragraph::new(conn_text)
        .block(ebpf_block)
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(ebpf_p, sub_chunks[1]);
}
