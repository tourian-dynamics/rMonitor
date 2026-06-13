//! Application state and loop drivers for pulse.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Foreground).

use std::time::Instant;

use ratatui::style::Color;
use ratatui::text::Line;
use ratatui::widgets::TableState;
use crate::backend::sysinfo_shim::{Disks, Networks, System};

use crate::config::AppConfig;
use crate::ui::spring::Spring;
use crate::ui::status_bar::StatusBar;
use crate::ui::layout_helpers::ButtonRect;
use crate::ui::mouse_selection::MouseSelection;
use crate::backend::window::WindowDrag;
use crate::backend::ebpf::EbpfTracker;

pub mod keys;
pub mod mouse;
pub mod power;
pub mod types;
pub mod metrics;

pub use power::PowerThrottle;
pub use types::{FocusedSection, ProcessDetails, ProcessItem, AppTheme};

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
    pub ebpf: EbpfTracker,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let base_theme = crate::ui::theme::get_theme(
            config.theme_mode == "dark",
            Color::Rgb(0, 245, 255),
        );
        let theme = AppTheme::from_base(
            base_theme,
            Color::Rgb(255, 0, 127),
            Color::Rgb(0, 100, 220),
        );

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
            username: crate::backend::identity::username(),
            host_name: crate::backend::identity::hostname(),
            os_str: crate::backend::identity::os_str(),
            ebpf: {
                let mut tracker = EbpfTracker::new();
                let _ = tracker.start_tracking();
                tracker
            },
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

    pub fn update_physics(&mut self, dt: f64) {
        self.cpu_spring.update(dt);
        self.mem_spring.update(dt);
        self.disk_spring.update(dt);
        self.gpu1_spring.update(dt);
        self.gpu2_spring.update(dt);
        self.net_spring.update(dt);
    }

    #[allow(dead_code)]
    pub fn open_embedded_markdown(&mut self, title: &str, content: &str) {
        #[allow(unused_variables)]
        let _ = (title, content);
    }
}
