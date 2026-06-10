//! pulse presentation layer (UI) root module.
//!
//! **Taxonomy Classification**: Interface (TUI / Presentation Layer).

pub mod cards;
pub mod widgets;
pub mod processes;
pub mod overlays;
pub mod spring;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};
use library::interface::tui::theme::{ThemeColors, get_theme};
use library::interface::tui::widgets::{
    draw_title_banner, is_too_small, render_too_small_warning,
};

use crate::app::{App, FocusedSection};
use crate::metrics_format::accent_color_from_hex;
use crate::backend::current as backend;

const MIN_W: u16 = 100;
const MIN_H: u16 = 35;

pub fn current_theme(app: &App) -> ThemeColors {
    let accent = accent_color_from_hex(backend::get_win_accent_color());
    get_theme(app.power.is_dark(), accent)
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.area();
    let theme = current_theme(app);

    if is_too_small(size, (MIN_W, MIN_H)) {
        render_too_small_warning(
            f,
            size,
            (size.width, size.height),
            (MIN_W, MIN_H),
            " ⚠️  Terminal Sizing Warning ",
            theme.warning,
        );
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title banner
            Constraint::Length(6), // Top stat cards
            Constraint::Min(10),   // Full-width context table
            Constraint::Length(3), // Footer status bar
        ])
        .split(size);

    let (help_btn, quit_btn) = draw_title_banner(
        f,
        chunks[0],
        &theme,
        " Rust System Monitor ",
        "pulse",
        env!("CARGO_PKG_VERSION"),
        &app.username,
        &app.host_name,
        &app.os_str,
    );

    app.help_btn = help_btn;
    app.quit_btn = quit_btn;

    render_panels(f, app, chunks.clone());

    // Render modal overlays
    if let Some(ref details) = app.selected_process_details {
        overlays::render_process_details_modal(f, size, details);
    }
    if let Some(pid_val) = app.kill_confirm_pid {
        let name = app.kill_confirm_name.clone().unwrap_or_default();
        overlays::render_kill_confirm_modal(f, size, pid_val, &name);
    }
    if app.show_help {
        overlays::render_help_modal(f, size);
    }
    if app.show_markdown.is_some() {
        overlays::render_markdown_modal(f, size, app);
    }

    // Draw status bar
    overlays::render_status_bar(f, chunks[3], app);
}

pub fn render_panels(f: &mut Frame, app: &mut App, chunks: std::rc::Rc<[Rect]>) {
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(chunks[1]);

    let card_w = top_chunks[0].width.saturating_sub(4);
    let card_w_usize = card_w as usize;

    let cpu_border = cards::border_color(app.focus, FocusedSection::Cpu, app.theme.accent, app.theme.border);
    let mem_border = cards::border_color(app.focus, FocusedSection::Memory, app.theme.accent, app.theme.border);
    let disk_border = cards::border_color(app.focus, FocusedSection::Disk, app.theme.accent, app.theme.border);
    let gpu_border = cards::border_color(app.focus, FocusedSection::Gpu, app.theme.accent, app.theme.border);
    let net_border = cards::border_color(app.focus, FocusedSection::Network, app.theme.accent, app.theme.border);

    cards::render_cpu_card(f, top_chunks[0], app, card_w, cpu_border);
    cards::render_memory_card(f, top_chunks[1], app, card_w, mem_border);
    cards::render_disk_card(f, top_chunks[2], app, card_w, disk_border);
    cards::render_gpu_card(f, top_chunks[3], app, card_w, card_w_usize, gpu_border);
    cards::render_network_card(f, top_chunks[4], app, card_w, net_border);

    render_context_table(f, chunks[2], app);
}

fn render_context_table(f: &mut Frame, area: Rect, app: &mut App) {
    let theme = &*app.theme;
    let border_color = theme.accent;

    let details_height: u16 = match app.focus {
        FocusedSection::Cpu => {
            let num_cpus = app.sys.cpus().len();
            let inner = area.width as usize;
            let cols = (inner / 10).max(1);
            let rows = num_cpus.div_ceil(cols);
            (rows as u16 + 2).clamp(4, 15)
        }
        FocusedSection::Memory => 11,
        FocusedSection::Disk => (app.disks.len() as u16 + 4).clamp(6, 15),
        FocusedSection::Gpu => (app.gpu_names.len() as u16 + 4).clamp(6, 12),
        FocusedSection::Network => (app.networks.len() as u16 + 4).clamp(6, 15),
    };

    let sub_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(details_height), Constraint::Min(4)])
        .split(area);

    match app.focus {
        FocusedSection::Cpu => widgets::render_cpu_details(f, sub_chunks[0], app, border_color),
        FocusedSection::Memory => widgets::render_memory_details(f, sub_chunks[0], app, border_color),
        FocusedSection::Disk => widgets::render_disk_details(f, sub_chunks[0], app, border_color),
        FocusedSection::Gpu => widgets::render_gpu_details(f, sub_chunks[0], app, border_color),
        FocusedSection::Network => widgets::render_network_details(f, sub_chunks[0], app, border_color),
    }

    processes::render_processes_table(f, sub_chunks[1], app);
}
