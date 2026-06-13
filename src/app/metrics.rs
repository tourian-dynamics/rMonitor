//! Metrics query and process detail snapshot logic for pulse.

use std::time::Instant;
use crate::app::{App, ProcessItem, ProcessDetails, FocusedSection, AppTheme};

impl App {
    pub fn refresh_theme(&mut self) {
        let base_theme = crate::ui::theme::get_theme(
            self.power.is_dark(),
            ratatui::style::Color::Rgb(
                self.power.accent_color.0,
                self.power.accent_color.1,
                self.power.accent_color.2,
            ),
        );
        self.theme = AppTheme::from_base(
            base_theme,
            ratatui::style::Color::Rgb(255, 0, 127),
            ratatui::style::Color::Rgb(0, 100, 220),
        );
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
            FocusedSection::Memory => self.processes.sort_by_key(|b| std::cmp::Reverse(b.mem)),
            FocusedSection::Disk => self
                .processes
                .sort_by_key(|b| std::cmp::Reverse(b.disk_read + b.disk_write)),
            FocusedSection::Gpu => self
                .processes
                .sort_by(|a, b| b.gpu.total_cmp(&a.gpu)),
            FocusedSection::Network => self.processes.sort_by_key(|b| std::cmp::Reverse(b.net)),
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

    pub fn show_process_details(&mut self, pid_val: u32) {
        self.sys.refresh_processes();
        let pid = crate::backend::sysinfo_shim::Pid::from_u32(pid_val);
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
}
