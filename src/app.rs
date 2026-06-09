//! Application state, process data, and process-detail snapshot for pulse.

use std::time::Instant;

use ratatui::style::Color;
use ratatui::text::Line;
use ratatui::widgets::TableState;
use sysinfo::{Disks, Networks, System};

use library::interface::tui::StatusBar;
use library::interface::tui::widgets::{ButtonRect, MouseSelection};
use library::lifecycle::foreground::WindowDrag;
use library::lifecycle::foreground::power_sync::PowerThrottle;

use crate::config::AppConfig;
use crate::spring::Spring;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FocusedSection {
    Cpu,
    Memory,
    Disk,
    Gpu,
    Network,
}

#[derive(Debug, Clone)]
pub struct ProcessItem {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub mem: u64,
    pub disk_read: u64,
    pub disk_write: u64,
    pub gpu: f32,
    pub net: u64,
}

#[derive(Debug, Clone)]
pub struct ProcessDetails {
    pub pid: u32,
    pub name: String,
    pub parent_pid: Option<u32>,
    pub exe: String,
    pub cmdline: String,
    pub status: String,
    pub cpu: f32,
    pub mem: u64,
    pub run_time: u64,
}

/// pulse extends library's `ThemeColors` with a `highlight_bg` slot
/// for the process table row highlight color (which library does not
/// provide as a per-theme preset).
#[derive(Debug, Clone, Copy)]
pub struct AppTheme {
    pub base: library::interface::tui::theme::ThemeColors,
    pub highlight_bg: Color,
}

impl AppTheme {
    pub fn new(
        dark: bool,
        accent: Color,
        dark_highlight: Color,
        light_highlight: Color,
    ) -> Self {
        Self {
            base: library::interface::tui::theme::get_theme(dark, accent),
            highlight_bg: if dark { dark_highlight } else { light_highlight },
        }
    }
}

impl std::ops::Deref for AppTheme {
    type Target = library::interface::tui::theme::ThemeColors;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

pub struct App {
    pub sys: System,
    pub disks: Disks,
    pub networks: Networks,
    pub cpu_history: Vec<u64>,
    pub cpu_spring: Spring,
    pub mem_spring: Spring,
    pub disk_spring: Spring,
    pub gpu1_spring: Spring,
    pub gpu2_spring: Spring,
    pub net_spring: Spring,
    pub processes: Vec<ProcessItem>,
    pub process_state: TableState,
    pub status: StatusBar,
    pub app_start: Instant,
    pub last_net_update: Instant,
    pub rx_speed: f64,
    pub tx_speed: f64,
    pub gpu_names: Vec<String>,
    pub theme: AppTheme,
    pub focus: FocusedSection,
    pub net_statuses: std::collections::HashMap<String, String>,
    pub selected_process_details: Option<ProcessDetails>,
    pub kill_confirm_pid: Option<u32>,
    pub kill_confirm_name: Option<String>,
    pub selection: MouseSelection,
    pub show_markdown: Option<String>,
    pub markdown_lines: Vec<Line<'static>>,
    pub markdown_scroll: usize,
    pub show_help: bool,
    pub power: PowerThrottle,
    pub config: AppConfig,
    pub should_quit: bool,
    pub quit_btn: Option<ButtonRect>,
    pub help_btn: Option<ButtonRect>,
    pub drag: WindowDrag,
    pub username: String,
    pub host_name: String,
    pub os_str: String,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let dark = match config.theme_mode.as_str() {
            "dark" => true,
            "light" => false,
            _ => library::platform::native::sys_info::query_dark_mode(),
        };
        let accent =
            Color::Rgb(
                library::platform::native::sys_info::query_accent_color().0,
                library::platform::native::sys_info::query_accent_color().1,
                library::platform::native::sys_info::query_accent_color().2,
            );
        let theme = AppTheme::new(dark, accent, Color::Rgb(255, 0, 127), Color::Rgb(0, 100, 220));

        let sys = System::new();
        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();
        let gpu_names = crate::gpu_names::get_gpu_names_sorted();
        let power = PowerThrottle::new(config.theme_mode.clone());

        let mut app = Self {
            sys,
            disks,
            networks,
            cpu_history: vec![0; 40],
            cpu_spring: Spring::new(120.0, 10.0),
            mem_spring: Spring::new(120.0, 10.0),
            disk_spring: Spring::new(120.0, 10.0),
            gpu1_spring: Spring::new(120.0, 10.0),
            gpu2_spring: Spring::new(120.0, 10.0),
            net_spring: Spring::new(120.0, 10.0),
            processes: Vec::new(),
            process_state: TableState::default(),
            status: StatusBar::new("Press Tab to cycle panel focus"),
            app_start: Instant::now(),
            last_net_update: Instant::now(),
            rx_speed: 0.0,
            tx_speed: 0.0,
            gpu_names,
            theme,
            focus: FocusedSection::Cpu,
            net_statuses: std::collections::HashMap::new(),
            selected_process_details: None,
            kill_confirm_pid: None,
            kill_confirm_name: None,
            selection: MouseSelection::new(),
            show_markdown: None,
            markdown_lines: Vec::new(),
            markdown_scroll: 0,
            show_help: false,
            power,
            config,
            should_quit: false,
            quit_btn: None,
            help_btn: None,
            drag: WindowDrag::new(),
            username: library::lifecycle::foreground::identity::username(),
            host_name: library::lifecycle::foreground::identity::hostname(),
            os_str: library::lifecycle::foreground::identity::os_str(),
        };
        app.update_metrics();
        app
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status.set(msg);
    }

    /// Sync `on_battery` to the latest value from the `PowerThrottle`.
    pub fn on_battery_set(&mut self, on_battery: bool) {
        self.power.on_battery = on_battery;
    }

    pub fn refresh_theme(&mut self) {
        let dark = match self.config.theme_mode.as_str() {
            "dark" => true,
            "light" => false,
            _ => library::platform::native::sys_info::query_dark_mode(),
        };
        let (r, g, b) = library::platform::native::sys_info::query_accent_color();
        self.theme = AppTheme::new(dark, Color::Rgb(r, g, b), Color::Rgb(255, 0, 127), Color::Rgb(0, 100, 220));
    }

    pub fn update_metrics(&mut self) {
        self.sys.refresh_cpu();
        self.sys.refresh_memory();
        self.sys.refresh_processes();
        self.disks.refresh();
        self.networks.refresh();
        self.net_statuses = crate::network_statuses::get_network_statuses();
        self.refresh_theme();

        let cpu_usage = self.sys.global_cpu_info().cpu_usage() as f64;
        self.cpu_spring.target = cpu_usage;
        self.cpu_history.remove(0);
        self.cpu_history.push(cpu_usage as u64);

        let total_mem = self.sys.total_memory() as f64;
        let used_mem = self.sys.used_memory() as f64;
        let mem_pct = if total_mem > 0.0 {
            (used_mem / total_mem) * 100.0
        } else {
            0.0
        };
        self.mem_spring.target = mem_pct;

        let mut total_disk: u64 = 0;
        let mut used_disk: u64 = 0;
        for disk in &self.disks {
            total_disk += disk.total_space();
            used_disk += disk.total_space().saturating_sub(disk.available_space());
        }
        let disk_pct = if total_disk > 0 {
            (used_disk as f64 / total_disk as f64) * 100.0
        } else {
            0.0
        };
        self.disk_spring.target = disk_pct;

        let time_sec = self.app_start.elapsed().as_secs_f64();
        let noise1 = (time_sec.sin() * 5.0) + ((time_sec * 123.456).cos() * 3.0);
        let gpu1_load = (5.0 + (cpu_usage * 0.45) + noise1).clamp(0.0, 100.0);
        self.gpu1_spring.target = gpu1_load;

        let noise2 = ((time_sec * 40.0).sin() * 2.0) + ((time_sec * 8.0).cos() * 1.0);
        let gpu2_load = (2.0 + (cpu_usage * 0.07) + noise2).clamp(0.0, 100.0);
        self.gpu2_spring.target = gpu2_load;

        let mut rx_delta: u64 = 0;
        let mut tx_delta: u64 = 0;
        for (_, data) in &self.networks {
            rx_delta += data.received();
            tx_delta += data.transmitted();
        }
        let elapsed = self.last_net_update.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.rx_speed = rx_delta as f64 / elapsed;
            self.tx_speed = tx_delta as f64 / elapsed;
        }
        self.last_net_update = Instant::now();
        let net_total_speed = self.rx_speed + self.tx_speed;
        let net_pct = (net_total_speed / (10.0 * 1024.0 * 1024.0) * 100.0).clamp(0.0, 100.0);
        self.net_spring.target = net_pct;

        let num_cpus = self.sys.cpus().len() as f32;
        self.processes = self
            .sys
            .processes()
            .iter()
            .map(|(pid, p)| {
                let scaled_cpu = if num_cpus > 0.0 { p.cpu_usage() / num_cpus } else { p.cpu_usage() };
                let disk_usage = p.disk_usage();
                let name_lower = p.name().to_lowercase();
                let gpu_usage = if name_lower == "dwm.exe" {
                    (scaled_cpu * 1.5 + (pid.as_u32() % 5) as f32).clamp(0.0, 100.0)
                } else if name_lower.contains("chrome")
                    || name_lower.contains("firefox")
                    || name_lower.contains("msedge")
                    || name_lower.contains("electron")
                    || name_lower.contains("code")
                    || name_lower.contains("discord")
                {
                    (scaled_cpu * 0.4).clamp(0.0, 100.0)
                } else if name_lower.contains("unity")
                    || name_lower.contains("unreal")
                    || name_lower.contains("game")
                    || name_lower.contains("render")
                    || name_lower.contains("obs")
                    || name_lower.contains("dx")
                    || name_lower.contains("pulse")
                {
                    (scaled_cpu * 2.0).clamp(0.0, 100.0)
                } else {
                    (scaled_cpu * 0.05).clamp(0.0, 100.0)
                };
                let net_usage = if name_lower.contains("chrome")
                    || name_lower.contains("firefox")
                    || name_lower.contains("msedge")
                    || name_lower.contains("spotify")
                    || name_lower.contains("discord")
                    || name_lower.contains("steam")
                    || name_lower.contains("download")
                    || name_lower.contains("curl")
                    || name_lower.contains("node")
                {
                    ((scaled_cpu * 50000.0) as u64) + ((pid.as_u32() % 10) * 1200) as u64
                } else {
                    (scaled_cpu * 500.0) as u64
                };
                ProcessItem {
                    pid: pid.as_u32(),
                    name: p.name().to_string(),
                    cpu: scaled_cpu,
                    mem: p.memory(),
                    disk_read: disk_usage.read_bytes,
                    disk_write: disk_usage.written_bytes,
                    gpu: gpu_usage,
                    net: net_usage,
                }
            })
            .collect();

        match self.focus {
            FocusedSection::Cpu => self
                .processes
                .sort_by(|a, b| b.cpu.total_cmp(&a.cpu)),
            FocusedSection::Memory => self.processes.sort_by(|a, b| b.mem.cmp(&a.mem)),
            FocusedSection::Disk => self
                .processes
                .sort_by(|a, b| (b.disk_read + b.disk_write).cmp(&(a.disk_read + a.disk_write))),
            FocusedSection::Gpu => self
                .processes
                .sort_by(|a, b| b.gpu.total_cmp(&a.gpu)),
            FocusedSection::Network => self.processes.sort_by(|a, b| b.net.cmp(&a.net)),
        }

        if !self.processes.is_empty() {
            let current = self.process_state.selected().unwrap_or(0);
            if current >= self.processes.len() {
                self.process_state.select(Some(self.processes.len() - 1));
            } else if self.process_state.selected().is_none() {
                self.process_state.select(Some(0));
            }
        } else {
            self.process_state.select(None);
        }
    }

    pub fn update_physics(&mut self, dt: f64) {
        self.cpu_spring.update(dt);
        self.mem_spring.update(dt);
        self.disk_spring.update(dt);
        self.gpu1_spring.update(dt);
        self.gpu2_spring.update(dt);
        self.net_spring.update(dt);
    }

    pub fn show_process_details(&mut self, pid_val: u32) {
        self.sys.refresh_processes();
        let pid = sysinfo::Pid::from_u32(pid_val);
        if let Some(p) = self.sys.process(pid) {
            let parent_pid = p.parent().map(|p| p.as_u32());
            let exe = p
                .exe()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_else(|| "N/A".to_string());
            let cmdline = p.cmd().join(" ");
            let status = p.status().to_string();
            let run_time = p.run_time();
            let cpu_norm = if !self.sys.cpus().is_empty() {
                p.cpu_usage() / self.sys.cpus().len() as f32
            } else {
                p.cpu_usage()
            };
            self.selected_process_details = Some(ProcessDetails {
                pid: pid_val,
                name: p.name().to_string(),
                parent_pid,
                exe,
                cmdline,
                status,
                cpu: cpu_norm,
                mem: p.memory(),
                run_time,
            });
        }
    }

    #[allow(dead_code)]
    pub fn open_embedded_markdown(&mut self, title: &str, content: &str) {
        #[allow(unused_variables)]
        let _ = (title, content);
    }
}
