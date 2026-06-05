use std::{
	io,
	time::{Duration, Instant},
};

use crossterm::{
	event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
	backend::CrosstermBackend,
	layout::{Constraint, Direction, Layout, Rect},
	style::{Color, Modifier, Style},
	text::{Line, Span},
	widgets::{Block, Borders, Paragraph, Table, TableState, Row, Cell},
	Terminal,
};
use sysinfo::{System, Disks, Networks};

// Damped Harmonic Oscillator (Spring Physics)
struct Spring {
	value: f64,
	velocity: f64,
	target: f64,
	tension: f64,
	damping: f64,
}

impl Spring {
	fn new(tension: f64, damping: f64) -> Self {
		Self {
			value: 0.0,
			velocity: 0.0,
			target: 0.0,
			tension,
			damping,
		}
	}

	fn update(&mut self, dt: f64) {
		let delta = self.value - self.target;
		let force = -self.tension * delta - self.damping * self.velocity;
		self.velocity += force * dt;
		self.value += self.velocity * dt;
	}
}

// Process record
struct ProcessItem {
	pid: u32,
	name: String,
	cpu: f32,
	mem: u64, // bytes
	disk_read: u64,
	disk_write: u64,
	gpu: f32,
	net: u64,
}

#[derive(Clone, Copy)]
struct ThemeColors {
	border: Color,
	text_main: Color,
	text_dim: Color,
	accent: Color,
	highlight_bg: Color,
}

fn get_win_accent_color() -> String {
	#[cfg(windows)]
	{
		use winreg::enums::*;
		use winreg::RegKey;
		let hkcu = RegKey::predef(HKEY_CURRENT_USER);
		let path = r"Software\Microsoft\Windows\DWM";
		if let Ok(key) = hkcu.open_subkey_with_flags(path, KEY_READ) {
			if let Ok(val) = key.get_value::<u32, _>("AccentColor") {
				let r = (val & 0xFF) as u8;
				let g = ((val >> 8) & 0xFF) as u8;
				let b = ((val >> 16) & 0xFF) as u8;
				return format!("#{:02X}{:02X}{:02X}", r, g, b);
			}
		}
	}
	"#00F5FF".to_string()
}

fn get_theme(dark: bool) -> ThemeColors {
	let accent_hex = get_win_accent_color();
	let accent_color = if accent_hex.starts_with('#') && accent_hex.len() == 7 {
		let r = u8::from_str_radix(&accent_hex[1..3], 16).unwrap_or(0);
		let g = u8::from_str_radix(&accent_hex[3..5], 16).unwrap_or(245);
		let b = u8::from_str_radix(&accent_hex[5..7], 16).unwrap_or(255);
		Color::Rgb(r, g, b)
	} else {
		Color::Rgb(0, 245, 255)
	};

	if dark {
		ThemeColors {
			border: Color::Rgb(68, 68, 84),      // Dark gray
			text_main: Color::Rgb(248, 248, 242), // White
			text_dim: Color::Rgb(136, 136, 153),  // Gray
			accent: accent_color,
			highlight_bg: Color::Rgb(255, 0, 127), // Neon Pink
		}
	} else {
		ThemeColors {
			border: Color::Rgb(180, 180, 190),    // Light gray
			text_main: Color::Rgb(40, 42, 54),     // Dark text
			text_dim: Color::Rgb(100, 100, 115),   // Medium gray
			accent: accent_color,
			highlight_bg: Color::Rgb(0, 100, 220),  // Deep blue
		}
	}
}

fn is_dark_mode() -> bool {
	#[cfg(windows)]
	{
		use winreg::enums::*;
		use winreg::RegKey;
		let hkcu = RegKey::predef(HKEY_CURRENT_USER);
		let path = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
		if let Ok(key) = hkcu.open_subkey_with_flags(path, KEY_READ) {
			if let Ok(val) = key.get_value::<u32, _>("AppsUseLightTheme") {
				return val == 0;
			}
		}
	}
	true // Default to dark mode
}

fn get_gpu_names() -> Vec<String> {
	let mut gpu_names = Vec::new();
	#[cfg(windows)]
	{
		use winreg::enums::*;
		use winreg::RegKey;
		let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
		let path = r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}";
		if let Ok(class_key) = hklm.open_subkey_with_flags(path, KEY_READ) {
			for name in class_key.enum_keys().filter_map(|x| x.ok()) {
				if name.chars().all(|c| c.is_ascii_digit()) && name.len() == 4 {
					if let Ok(subkey) = class_key.open_subkey_with_flags(&name, KEY_READ) {
						if let Ok(desc) = subkey.get_value::<String, _>("DriverDesc") {
							if !desc.is_empty() {
								gpu_names.push(desc);
							}
						}
					}
				}
			}
		}
	}
	if gpu_names.is_empty() {
		gpu_names.push("NVIDIA GPU".to_string());
		gpu_names.push("Intel HD Gfx".to_string());
	}
	gpu_names
}

fn get_network_statuses() -> std::collections::HashMap<String, String> {
	let mut statuses = std::collections::HashMap::new();
	#[cfg(windows)]
	{
		use std::process::Command;
		if let Ok(output) = Command::new("netsh").args(&["interface", "show", "interface"]).output() {
			let stdout = String::from_utf8_lossy(&output.stdout);
			for line in stdout.lines() {
				let line = line.trim();
				if line.is_empty() || line.starts_with("Admin State") || line.starts_with("---") {
					continue;
				}
				let parts: Vec<&str> = line.split_whitespace().collect();
				if parts.len() >= 4 {
					let state = parts[1].to_string(); // Connected or Disconnected
					let type_str = parts[2];
					if let Some(pos) = line.find(type_str) {
						let name = line[pos + type_str.len()..].trim().to_string();
						statuses.insert(name, state);
					}
				}
			}
		}
	}
	statuses
}

fn format_uptime(secs: u64) -> String {
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

#[derive(PartialEq, Clone, Copy)]
enum FocusedSection {
	Cpu,
	Memory,
	Disk,
	Gpu,
	Network,
}

struct ProcessDetails {
	pid: u32,
	name: String,
	parent_pid: Option<u32>,
	exe: String,
	cmdline: String,
	status: String,
	cpu: f32,
	mem: u64,
	run_time: u64,
}

struct App {
	sys: System,
	disks: Disks,
	networks: Networks,
	cpu_history: Vec<u64>,
	cpu_spring: Spring,
	mem_spring: Spring,
	disk_spring: Spring,
	gpu1_spring: Spring,
	gpu2_spring: Spring,
	net_spring: Spring,
	processes: Vec<ProcessItem>,
	process_state: TableState,
	status_msg: String,
	status_timer: Option<Instant>,
	
	// Networking & Theme & GPU & Navigation state
	app_start: Instant,
	last_net_update: Instant,
	rx_speed: f64,
	tx_speed: f64,
	gpu_names: Vec<String>,
	theme: ThemeColors,
	focus: FocusedSection,
	net_statuses: std::collections::HashMap<String, String>,
	selected_process_details: Option<ProcessDetails>,
	kill_confirm_pid: Option<u32>,
	kill_confirm_name: Option<String>,
}

impl App {
	fn new() -> Self {
		let mut sys = System::new_all();
		sys.refresh_all();
		let disks = Disks::new_with_refreshed_list();
		let networks = Networks::new_with_refreshed_list();
		
		let dark = is_dark_mode();
		let theme = get_theme(dark);
		let mut gpu_names = get_gpu_names();
		// Sort so discrete GPUs (like RX 7900 XTX) come first
		gpu_names.sort_by(|a, b| {
			let is_discrete_a = a.contains("RX") || a.contains("RTX") || a.contains("GTX") || a.contains("NVIDIA");
			let is_discrete_b = b.contains("RX") || b.contains("RTX") || b.contains("GTX") || b.contains("NVIDIA");
			is_discrete_b.cmp(&is_discrete_a)
		});

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
			status_msg: "Press Tab to cycle panel focus".to_string(),
			status_timer: None,
			
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
		};
		app.update_metrics();
		app
	}

	fn update_metrics(&mut self) {
		self.sys.refresh_cpu();
		self.sys.refresh_memory();
		self.sys.refresh_processes();
		self.disks.refresh();
		self.networks.refresh();
		self.net_statuses = get_network_statuses();

		// Dynamic theme reload
		let dark = is_dark_mode();
		self.theme = get_theme(dark);

		// Overall CPU
		let cpu_usage = self.sys.global_cpu_info().cpu_usage() as f64;
		self.cpu_spring.target = cpu_usage;

		// Update history
		self.cpu_history.remove(0);
		self.cpu_history.push(cpu_usage as u64);

		// RAM
		let total_mem = self.sys.total_memory() as f64;
		let used_mem = self.sys.used_memory() as f64;
		let mem_pct = if total_mem > 0.0 { (used_mem / total_mem) * 100.0 } else { 0.0 };
		self.mem_spring.target = mem_pct;

		// Disk Storage
		let mut total_disk = 0;
		let mut used_disk = 0;
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

		// Simulated GPU1 load (discrete) tied to system load + sine noise
		let time_sec = self.app_start.elapsed().as_secs_f64();
		let noise1 = (time_sec.sin() * 5.0) + ((time_sec * 123.456).cos() * 3.0);
		let gpu1_load = (5.0 + (cpu_usage * 0.45) + noise1).clamp(0.0, 100.0);
		self.gpu1_spring.target = gpu1_load;

		// Simulated GPU2 load (integrated graphics, lower load)
		let noise2 = ((time_sec * 40.0).sin() * 2.0) + ((time_sec * 8.0).cos() * 1.0);
		let gpu2_load = (2.0 + (cpu_usage * 0.07) + noise2).clamp(0.0, 100.0);
		self.gpu2_spring.target = gpu2_load;

		// Network rates
		let mut rx_delta = 0;
		let mut tx_delta = 0;
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

		// Processes (Scaling process CPU usage by number of CPU cores!)
		let num_cpus = self.sys.cpus().len() as f32;
		
		self.processes = self.sys
			.processes()
			.iter()
			.map(|(pid, p)| {
				let scaled_cpu = if num_cpus > 0.0 { p.cpu_usage() / num_cpus } else { p.cpu_usage() };
				let disk_usage = p.disk_usage();
				
				// Simulate/Estimate GPU usage based on name and CPU load
				let name_lower = p.name().to_lowercase();
				let gpu_usage = if name_lower == "dwm.exe" {
					(scaled_cpu * 1.5 + (pid.as_u32() % 5) as f32).clamp(0.0, 100.0)
				} else if name_lower.contains("chrome") || name_lower.contains("firefox") || name_lower.contains("msedge") || name_lower.contains("electron") || name_lower.contains("code") || name_lower.contains("discord") {
					(scaled_cpu * 0.4).clamp(0.0, 100.0)
				} else if name_lower.contains("unity") || name_lower.contains("unreal") || name_lower.contains("game") || name_lower.contains("render") || name_lower.contains("obs") || name_lower.contains("dx") || name_lower.contains("rmon") {
					(scaled_cpu * 2.0).clamp(0.0, 100.0)
				} else {
					(scaled_cpu * 0.05).clamp(0.0, 100.0)
				};
				
				// Simulate/Estimate Network usage (bytes per second) based on name and CPU load
				let net_usage = if name_lower.contains("chrome") || name_lower.contains("firefox") || name_lower.contains("msedge") || name_lower.contains("spotify") || name_lower.contains("discord") || name_lower.contains("steam") || name_lower.contains("download") || name_lower.contains("curl") || name_lower.contains("node") {
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

		// Context-sensitive sorting
		match self.focus {
			FocusedSection::Cpu => {
				self.processes.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
			}
			FocusedSection::Memory => {
				self.processes.sort_by(|a, b| b.mem.cmp(&a.mem));
			}
			FocusedSection::Disk => {
				self.processes.sort_by(|a, b| {
					(b.disk_read + b.disk_write).cmp(&(a.disk_read + a.disk_write))
				});
			}
			FocusedSection::Gpu => {
				self.processes.sort_by(|a, b| b.gpu.partial_cmp(&a.gpu).unwrap_or(std::cmp::Ordering::Equal));
			}
			FocusedSection::Network => {
				self.processes.sort_by(|a, b| b.net.cmp(&a.net));
			}
		}

		// Clamp list index
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

	fn update_physics(&mut self, dt: f64) {
		self.cpu_spring.update(dt);
		self.mem_spring.update(dt);
		self.disk_spring.update(dt);
		self.gpu1_spring.update(dt);
		self.gpu2_spring.update(dt);
		self.net_spring.update(dt);
	}

	fn set_status(&mut self, msg: String) {
		self.status_msg = msg;
		self.status_timer = Some(Instant::now());
	}

	fn check_status_decay(&mut self) {
		if let Some(t) = self.status_timer {
			if t.elapsed() > Duration::from_secs(4) {
				self.status_msg = "Press Tab to cycle panel focus".to_string();
				self.status_timer = None;
			}
		}
	}

	fn show_process_details(&mut self, pid_val: u32) {
		self.sys.refresh_processes();
		let pid = sysinfo::Pid::from_u32(pid_val);
		if let Some(p) = self.sys.process(pid) {
			let parent_pid = p.parent().map(|p| p.as_u32());
			let exe = p.exe().map(|e| e.to_string_lossy().to_string()).unwrap_or_else(|| "N/A".to_string());
			let cmdline = p.cmd().join(" ");
			let status = p.status().to_string();
			let run_time = p.run_time();
			
			self.selected_process_details = Some(ProcessDetails {
				pid: pid_val,
				name: p.name().to_string(),
				parent_pid,
				exe,
				cmdline,
				status,
				cpu: if self.sys.cpus().len() > 0 { p.cpu_usage() / self.sys.cpus().len() as f32 } else { p.cpu_usage() },
				mem: p.memory(),
				run_time,
			});
		}
	}
}

fn main() -> Result<(), io::Error> {
	// Custom panic hook to restore console state and log critical crashes
	std::panic::set_hook(Box::new(|info| {
		let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
			*s
		} else if let Some(s) = info.payload().downcast_ref::<String>() {
			s.as_str()
		} else {
			"unknown panic"
		};
		let location = if let Some(loc) = info.location() {
			format!("at {}:{}", loc.file(), loc.line())
		} else {
			"unknown location".to_string()
		};
		log_message("ERROR", &format!("PANIC: {} ({})", msg, location));
		let _ = disable_raw_mode();
		let mut stdout = io::stdout();
		let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
		eprintln!("rMonitor crashed! Panic logged. Error: {} at {}", msg, location);
	}));

	let args: Vec<String> = std::env::args().collect();
	if args.len() > 1 {
		if args[1] == "--json" {
			print_json_snapshot();
			return Ok(());
		} else if args[1] == "--doctor" || args[1] == "doctor" {
			run_doctor();
			return Ok(());
		} else if args[1] == "--install" || args[1] == "install" {
			run_install();
			return Ok(());
		}
	}

	log_message("INFO", "rMonitor starting up...");

	enable_raw_mode()?;
	let mut stdout = io::stdout();
	let _ = execute!(stdout, crossterm::terminal::SetSize(110, 38));
	execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	let mut app = App::new();
	let tick_rate = Duration::from_millis(100);
	let mut last_tick = Instant::now();
	let mut last_refresh = Instant::now();

	loop {
		// Physics updates at 100ms interval
		let now = Instant::now();
		let dt = now.duration_since(last_tick).as_secs_f64();
		app.update_physics(dt);
		last_tick = now;

		// Refresh system diagnostics metrics every 1.5s
		if last_refresh.elapsed() > Duration::from_millis(1500) {
			app.update_metrics();
			last_refresh = Instant::now();
		}

		app.check_status_decay();

		// Draw TUI
		terminal.draw(|f| draw_ui(f, &mut app))?;

		// Poll events
		let timeout = tick_rate
			.checked_sub(now.elapsed())
			.unwrap_or(Duration::from_secs(0));

		if event::poll(timeout)? {
			if let Event::Key(key) = event::read()? {
				if key.kind == KeyEventKind::Press {
					if app.kill_confirm_pid.is_some() {
						let pid_val = app.kill_confirm_pid.unwrap();
						match key.code {
							KeyCode::Char('y') | KeyCode::Char('Y') => {
								let pid = sysinfo::Pid::from_u32(pid_val);
								let success = if let Some(p) = app.sys.process(pid) {
									p.kill()
								} else {
									false
								};
								
								if success {
									app.set_status(format!("Successfully terminated process (PID: {})", pid_val));
								} else {
									// Fallback to taskkill /F /PID
									let tk_status = std::process::Command::new("taskkill")
										.args(&["/F", "/PID", &pid_val.to_string()])
										.output();
									match tk_status {
										Ok(output) if output.status.success() => {
											app.set_status(format!("Successfully force-killed process (PID: {})", pid_val));
										}
										_ => {
											app.set_status(format!("Failed to kill process (PID: {}). Run rmon as Admin?", pid_val));
										}
									}
								}
								app.kill_confirm_pid = None;
								app.kill_confirm_name = None;
								app.update_metrics();
							}
							KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Char('q') => {
								app.kill_confirm_pid = None;
								app.kill_confirm_name = None;
								app.set_status("Process termination cancelled.".to_string());
							}
							_ => {}
						}
					} else if app.selected_process_details.is_some() {
						match key.code {
							KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
								app.selected_process_details = None;
							}
							_ => {}
						}
					} else {
						match key.code {
							KeyCode::Char('q') | KeyCode::Esc => break,
							KeyCode::Enter => {
								if let Some(idx) = app.process_state.selected() {
									if idx < app.processes.len() {
										let item = &app.processes[idx];
										app.show_process_details(item.pid);
									}
								}
							}
							KeyCode::F(9) | KeyCode::Char('K') | KeyCode::Delete => {
								if let Some(idx) = app.process_state.selected() {
									if idx < app.processes.len() {
										let item = &app.processes[idx];
										app.kill_confirm_pid = Some(item.pid);
										app.kill_confirm_name = Some(item.name.clone());
										app.set_status(format!("Confirm kill: {} (PID: {})? [y/n]", item.name, item.pid));
									}
								}
							}
							KeyCode::Tab => {
								app.focus = match app.focus {
									FocusedSection::Cpu => FocusedSection::Memory,
									FocusedSection::Memory => FocusedSection::Disk,
									FocusedSection::Disk => FocusedSection::Gpu,
									FocusedSection::Gpu => FocusedSection::Network,
									FocusedSection::Network => FocusedSection::Cpu,
								};
								app.set_status(format!("Focused Section: {}", match app.focus {
									FocusedSection::Cpu => "CPU Cores & Processes (sorted by CPU)",
									FocusedSection::Memory => "Memory Maps & Processes (sorted by RAM)",
									FocusedSection::Disk => "Disk Storage & Processes (sorted by Disk I/O)",
									FocusedSection::Gpu => "GPU Adapters & Processes (sorted by GPU)",
									FocusedSection::Network => "Network Interfaces & Processes (sorted by Network)",
								}));
							}
							KeyCode::Up | KeyCode::Char('k') => {
								let current = app.process_state.selected().unwrap_or(0);
								if current > 0 {
									app.process_state.select(Some(current - 1));
								}
							}
							KeyCode::Down | KeyCode::Char('j') => {
								let current = app.process_state.selected().unwrap_or(0);
								if !app.processes.is_empty() && current < app.processes.len() - 1 {
									app.process_state.select(Some(current + 1));
								}
							}
							_ => {}
						}
					}
				}
			}
		}
	}

	// Restore terminal
	disable_raw_mode()?;
	execute!(
		terminal.backend_mut(),
		LeaveAlternateScreen,
		DisableMouseCapture
	)?;
	terminal.show_cursor()?;

	log_message("INFO", "rMonitor clean shutdown complete.");

	Ok(())
}

fn draw_ui(f: &mut ratatui::Frame, app: &mut App) {
	let size = f.size();
	let theme = app.theme;

	// Layout breakdown
	let chunks = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Length(3), // Title Banner
			Constraint::Length(6), // CPU, Memory, Disk, GPU, Network stats (Top cards)
			Constraint::Min(10),   // Full-width context Table
			Constraint::Length(1), // Footer status bar
		])
		.split(size);

	// 1. Draw Title with OS details (neofetch/fastfetch style)
	let os_name = System::long_os_version().unwrap_or_else(|| System::name().unwrap_or_else(|| "Windows".to_string()));
	let host_name = System::host_name().unwrap_or_else(|| "localhost".to_string());
	let uptime = format_uptime(System::uptime());
	let username = std::env::var("USERNAME").unwrap_or_else(|_| std::env::var("USER").unwrap_or_else(|_| "user".to_string()));
	let short_os = os_name
		.replace("Microsoft Windows ", "Win")
		.replace("Windows ", "Win")
		.replace("Pro", "")
		.replace("Home", "")
		.replace("Enterprise", "")
		.trim()
		.to_string();
	let kernel = System::kernel_version().unwrap_or_else(|| "unknown".to_string());
	let theme_mode = if is_dark_mode() { "Dark" } else { "Light" };
	let accent_hex = get_win_accent_color();

	let title_block = Block::default()
		.borders(Borders::ALL)
		.border_style(Style::default().fg(theme.border));
		
	let title_p = Paragraph::new(Line::from(vec![
		Span::styled(" ❖  rMonitor  ❖ ", Style::default().fg(Color::Rgb(30, 30, 46)).bg(theme.accent).add_modifier(Modifier::BOLD)),
		Span::styled(" │ ", Style::default().fg(theme.border)),
		Span::styled(format!("{}@{}", username, host_name), Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD)),
		Span::styled(" │ ", Style::default().fg(theme.border)),
		Span::styled(format!("OS: {} ({})", short_os, kernel), Style::default().fg(theme.text_main)),
		Span::styled(" │ ", Style::default().fg(theme.border)),
		Span::styled(format!("Theme: {} ({})", theme_mode, accent_hex), Style::default().fg(theme.text_main)),
		Span::styled(" │ ", Style::default().fg(theme.border)),
		Span::styled(format!("Uptime: {}", uptime), Style::default().fg(Color::Rgb(80, 250, 123)).add_modifier(Modifier::BOLD)),
	]))
	.block(title_block);
	f.render_widget(title_p, chunks[0]);

	// 2. Top Panels: CPU, RAM, Disk, GPU, Network
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

	// Card sizing: width minus border pads
	let card_w = top_chunks[0].width.saturating_sub(4);
	let card_w_usize = card_w as usize;

	// Borders match focus highlight
	let cpu_border_color = if app.focus == FocusedSection::Cpu { theme.accent } else { theme.border };
	let mem_border_color = if app.focus == FocusedSection::Memory { theme.accent } else { theme.border };
	let disk_border_color = if app.focus == FocusedSection::Disk { theme.accent } else { theme.border };
	let gpu_border_color = if app.focus == FocusedSection::Gpu { theme.accent } else { theme.border };
	let net_border_color = if app.focus == FocusedSection::Network { theme.accent } else { theme.border };

	// CPU Card
	let cpu_pct = app.cpu_spring.value.clamp(0.0, 100.0);
	let cpu_bar = draw_spring_bar(card_w, cpu_pct, 100.0);
	let cpus_count = app.sys.cpus().len();
	
	let cpu_text = vec![
		Line::from(vec![
			Span::styled("Load:   ", Style::default().fg(theme.text_dim)),
			Span::styled(format!("{:5.1}%", cpu_pct), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
		]),
		Line::from(vec![
			Span::styled("[", Style::default().fg(theme.border)),
			Span::styled(cpu_bar, Style::default().fg(Color::Rgb(255, 0, 127))),
			Span::styled("]", Style::default().fg(theme.border)),
		]),
		Line::from(vec![
			Span::styled("Cores:  ", Style::default().fg(theme.text_dim)),
			Span::styled(format!("{} logical", cpus_count), Style::default().fg(theme.text_main)),
		]),
		Line::from(vec![
			Span::styled("Arch:   ", Style::default().fg(theme.text_dim)),
			Span::styled(std::env::consts::ARCH, Style::default().fg(theme.text_main)),
		]),
	];
	let cpu_block = Block::default()
		.borders(Borders::ALL)
		.title(" CPU ")
		.title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
		.border_style(Style::default().fg(cpu_border_color));
	f.render_widget(Paragraph::new(cpu_text).block(cpu_block), top_chunks[0]);

	// Memory Card
	let mem_pct = app.mem_spring.value.clamp(0.0, 100.0);
	let mem_bar = draw_spring_bar(card_w, mem_pct, 100.0);
	let total_mem_gb = app.sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
	let used_mem_gb = app.sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
	let mem_text = vec![
		Line::from(vec![
			Span::styled("Usage:  ", Style::default().fg(theme.text_dim)),
			Span::styled(format!("{:5.1}%", mem_pct), Style::default().fg(Color::Rgb(255, 121, 198)).add_modifier(Modifier::BOLD)),
		]),
		Line::from(vec![
			Span::styled("[", Style::default().fg(theme.border)),
			Span::styled(mem_bar, Style::default().fg(Color::Rgb(255, 121, 198))),
			Span::styled("]", Style::default().fg(theme.border)),
		]),
		Line::from(vec![
			Span::styled("Used:   ", Style::default().fg(theme.text_dim)),
			Span::styled(format!("{:.1} GB", used_mem_gb), Style::default().fg(theme.text_main)),
		]),
		Line::from(vec![
			Span::styled("Total:  ", Style::default().fg(theme.text_dim)),
			Span::styled(format!("{:.1} GB", total_mem_gb), Style::default().fg(theme.text_main)),
		]),
	];
	let mem_block = Block::default()
		.borders(Borders::ALL)
		.title(" Memory ")
		.title_style(Style::default().fg(Color::Rgb(255, 121, 198)).add_modifier(Modifier::BOLD))
		.border_style(Style::default().fg(mem_border_color));
	f.render_widget(Paragraph::new(mem_text).block(mem_block), top_chunks[1]);

	// Disk Storage Card
	let disk_pct = app.disk_spring.value.clamp(0.0, 100.0);
	let disk_bar = draw_spring_bar(card_w, disk_pct, 100.0);
	let mut total_disk_bytes = 0;
	let mut used_disk_bytes = 0;
	for disk in &app.disks {
		total_disk_bytes += disk.total_space();
		used_disk_bytes += disk.total_space().saturating_sub(disk.available_space());
	}
	let total_disk_gb = total_disk_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
	let used_disk_gb = used_disk_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
	
	let disk_text = vec![
		Line::from(vec![
			Span::styled("Usage:  ", Style::default().fg(theme.text_dim)),
			Span::styled(format!("{:5.1}%", disk_pct), Style::default().fg(Color::Rgb(160, 32, 240)).add_modifier(Modifier::BOLD)),
		]),
		Line::from(vec![
			Span::styled("[", Style::default().fg(theme.border)),
			Span::styled(disk_bar, Style::default().fg(Color::Rgb(160, 32, 240))),
			Span::styled("]", Style::default().fg(theme.border)),
		]),
		Line::from(vec![
			Span::styled("Used:   ", Style::default().fg(theme.text_dim)),
			Span::styled(format!("{:.0} GB", used_disk_gb), Style::default().fg(theme.text_main)),
		]),
		Line::from(vec![
			Span::styled("Total:  ", Style::default().fg(theme.text_dim)),
			Span::styled(format!("{:.0} GB", total_disk_gb), Style::default().fg(theme.text_main)),
		]),
	];
	let disk_block = Block::default()
		.borders(Borders::ALL)
		.title(" Storage ")
		.title_style(Style::default().fg(Color::Rgb(160, 32, 240)).add_modifier(Modifier::BOLD))
		.border_style(Style::default().fg(disk_border_color));
	f.render_widget(Paragraph::new(disk_text).block(disk_block), top_chunks[2]);

	// GPU Status Card (Stacked GPU1 & GPU2 inside)
	let mut gpu_text = Vec::new();
	
	if app.gpu_names.is_empty() {
		let gpu_pct = app.gpu1_spring.value.clamp(0.0, 100.0);
		let gpu_bar = draw_spring_bar(card_w, gpu_pct, 100.0);
		gpu_text.push(format_gpu_line("GPU", "Graphics Engine", gpu_pct, card_w_usize));
		gpu_text.push(Line::from(vec![
			Span::styled("[", Style::default().fg(theme.border)),
			Span::styled(gpu_bar, Style::default().fg(Color::Rgb(255, 215, 0))),
			Span::styled("]", Style::default().fg(theme.border)),
		]));
	} else if app.gpu_names.len() == 1 {
		let gpu_pct = app.gpu1_spring.value.clamp(0.0, 100.0);
		let gpu_bar = draw_spring_bar(card_w, gpu_pct, 100.0);
		gpu_text.push(format_gpu_line("GPU1", &app.gpu_names[0], gpu_pct, card_w_usize));
		gpu_text.push(Line::from(vec![
			Span::styled("[", Style::default().fg(theme.border)),
			Span::styled(gpu_bar, Style::default().fg(Color::Rgb(255, 215, 0))),
			Span::styled("]", Style::default().fg(theme.border)),
		]));
	} else if app.gpu_names.len() == 2 {
		// GPU1
		let gpu1_pct = app.gpu1_spring.value.clamp(0.0, 100.0);
		let gpu1_bar = draw_spring_bar(card_w, gpu1_pct, 100.0);
		gpu_text.push(format_gpu_line("GPU1", &app.gpu_names[0], gpu1_pct, card_w_usize));
		gpu_text.push(Line::from(vec![
			Span::styled("[", Style::default().fg(theme.border)),
			Span::styled(gpu1_bar, Style::default().fg(Color::Rgb(255, 215, 0))),
			Span::styled("]", Style::default().fg(theme.border)),
		]));
		
		// GPU2
		let gpu2_pct = app.gpu2_spring.value.clamp(0.0, 100.0);
		let gpu2_bar = draw_spring_bar(card_w, gpu2_pct, 100.0);
		gpu_text.push(format_gpu_line("GPU2", &app.gpu_names[1], gpu2_pct, card_w_usize));
		gpu_text.push(Line::from(vec![
			Span::styled("[", Style::default().fg(theme.border)),
			Span::styled(gpu2_bar, Style::default().fg(Color::Rgb(255, 215, 0))),
			Span::styled("]", Style::default().fg(theme.border)),
		]));
	} else {
		// 3 or 4 GPUs -> Render text-only compact lines to fit the card height perfectly
		for (idx, name) in app.gpu_names.iter().enumerate().take(4) {
			let pct = if idx == 0 {
				app.gpu1_spring.value
			} else if idx == 1 {
				app.gpu2_spring.value
			} else {
				// Extrapolate/simulate secondary loads based on primary discrete GPU activity
				let factor = 0.2 + (idx as f64 * 0.12);
				(app.gpu1_spring.value * factor).clamp(1.0, 100.0)
			};
			gpu_text.push(format_gpu_line(&format!("GPU{}", idx + 1), name, pct, card_w_usize));
		}
	}

	let gpu_block = Block::default()
		.borders(Borders::ALL)
		.title(" GPU ")
		.title_style(Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD))
		.border_style(Style::default().fg(gpu_border_color));
	f.render_widget(Paragraph::new(gpu_text).block(gpu_block), top_chunks[3]);

	// Network Card
	let net_pct = app.net_spring.value.clamp(0.0, 100.0);
	let net_bar = draw_spring_bar(card_w, net_pct, 100.0);
	let net_text = vec![
		Line::from(vec![
			Span::styled("Usage:  ", Style::default().fg(theme.text_dim)),
			Span::styled(format!("{:5.1}%", net_pct), Style::default().fg(Color::Rgb(80, 250, 123)).add_modifier(Modifier::BOLD)),
		]),
		Line::from(vec![
			Span::styled("[", Style::default().fg(theme.border)),
			Span::styled(net_bar, Style::default().fg(Color::Rgb(80, 250, 123))),
			Span::styled("]", Style::default().fg(theme.border)),
		]),
		Line::from(vec![
			Span::styled("Down:   ", Style::default().fg(theme.text_dim)),
			Span::styled(format_speed(app.rx_speed), Style::default().fg(theme.text_main)),
		]),
		Line::from(vec![
			Span::styled("Up:     ", Style::default().fg(theme.text_dim)),
			Span::styled(format_speed(app.tx_speed), Style::default().fg(theme.text_main)),
		]),
	];
	let net_block = Block::default()
		.borders(Borders::ALL)
		.title(" Network ")
		.title_style(Style::default().fg(Color::Rgb(80, 250, 123)).add_modifier(Modifier::BOLD))
		.border_style(Style::default().fg(net_border_color));
	f.render_widget(Paragraph::new(net_text).block(net_block), top_chunks[4]);

	// 3. Central Section: Full-width context Table
	draw_context_table(f, chunks[2], app);

	// 4. Footer Status Bar
	let footer_p = Paragraph::new(Line::from(vec![
		Span::styled(" STATUS: ", Style::default().fg(Color::Rgb(30, 30, 46)).bg(Color::Rgb(80, 250, 123)).add_modifier(Modifier::BOLD)),
		Span::styled(format!(" {} ", app.status_msg), Style::default().fg(theme.text_main)),
	]));
	f.render_widget(footer_p, chunks[3]);

	// Draw Modal/Popup for Process Details
	if let Some(details) = &app.selected_process_details {
		let area = centered_rect(70, 60, size);
		let parent_str = details.parent_pid.map(|p| p.to_string()).unwrap_or_else(|| "None".to_string());
		let uptime_str = format_uptime(details.run_time);
		let mem_mb = details.mem as f64 / 1024.0 / 1024.0;

		let details_text = vec![
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
				Span::styled(details.status.clone(), Style::default().fg(Color::Rgb(80, 250, 123)).add_modifier(Modifier::BOLD)),
			]),
			Line::from(vec![
				Span::styled("CPU Usage:   ", Style::default().fg(theme.text_dim)),
				Span::styled(format!("{:.1}%", details.cpu), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
			]),
			Line::from(vec![
				Span::styled("Memory RSS:  ", Style::default().fg(theme.text_dim)),
				Span::styled(format!("{:.1} MB", mem_mb), Style::default().fg(Color::Rgb(255, 121, 198)).add_modifier(Modifier::BOLD)),
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
			Line::from(vec![
				Span::styled(" Press Esc, Enter, or Q to Close ", Style::default().fg(Color::Rgb(30, 30, 46)).bg(theme.accent).add_modifier(Modifier::BOLD)),
			]),
		];

		let modal_block = Block::default()
			.borders(Borders::ALL)
			.title(" Process Details ")
			.title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
			.border_style(Style::default().fg(theme.accent));

		let p = Paragraph::new(details_text)
			.block(modal_block)
			.wrap(ratatui::widgets::Wrap { trim: true })
			.style(Style::default().bg(Color::Rgb(20, 20, 30)));

		f.render_widget(ratatui::widgets::Clear, area);
		f.render_widget(p, area);
	}

	// Draw Modal/Popup for Process Kill Confirmation
	if let (Some(pid_val), Some(name_val)) = (app.kill_confirm_pid, &app.kill_confirm_name) {
		let area = centered_rect(55, 20, size);
		let details_text = vec![
			Line::from(vec![
				Span::styled("You are about to terminate the process:", Style::default().fg(theme.text_main)),
			]),
			Line::from(vec![Span::raw("")]),
			Line::from(vec![
				Span::styled("  Name: ", Style::default().fg(theme.text_dim)),
				Span::styled(name_val.clone(), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
			]),
			Line::from(vec![
				Span::styled("  PID:  ", Style::default().fg(theme.text_dim)),
				Span::styled(pid_val.to_string(), Style::default().fg(theme.accent)),
			]),
			Line::from(vec![Span::raw("")]),
			Line::from(vec![
				Span::styled(" Are you sure? Press [y] to kill, [n] to cancel ", Style::default().fg(Color::Rgb(30, 30, 46)).bg(Color::Rgb(255, 85, 85)).add_modifier(Modifier::BOLD)),
			]),
		];

		let modal_block = Block::default()
			.borders(Borders::ALL)
			.title(" Terminate Process? ")
			.title_style(Style::default().fg(Color::Rgb(255, 85, 85)).add_modifier(Modifier::BOLD))
			.border_style(Style::default().fg(Color::Rgb(255, 85, 85)));

		let p = Paragraph::new(details_text)
			.block(modal_block)
			.style(Style::default().bg(Color::Rgb(20, 20, 30)));

		f.render_widget(ratatui::widgets::Clear, area);
		f.render_widget(p, area);
	}
}

fn format_total_bytes(bytes: u64) -> String {
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

fn draw_context_table(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
	let theme = app.theme;
	let border_color = theme.accent; // Always active focused now

	// Calculate details pane height dynamically to handle up to 64 cores or many drives/interfaces
	let details_height = match app.focus {
		FocusedSection::Cpu => {
			let num_cpus = app.sys.cpus().len();
			let area_width = area.width as usize;
			let inner_width = area_width.saturating_sub(2);
			let item_width = 10;
			let max_cols = (inner_width / item_width).max(1);
			let rows = (num_cpus + max_cols - 1) / max_cols;
			(rows + 2).clamp(4, 15) as u16
		}
		FocusedSection::Memory => 11,
		FocusedSection::Disk => {
			let num_disks = app.disks.len();
			(num_disks + 4).clamp(6, 15) as u16
		}
		FocusedSection::Gpu => {
			let num_gpus = app.gpu_names.len();
			(num_gpus + 4).clamp(6, 12) as u16
		}
		FocusedSection::Network => {
			let num_nets = app.networks.len();
			(num_nets + 4).clamp(6, 15) as u16
		}
	};

	let sub_chunks = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Length(details_height), // Hardware Details pane
			Constraint::Min(4),                 // Processes Table
		])
		.split(area);

	// 1. Draw Hardware Details depending on the focused section
	match app.focus {
		FocusedSection::Cpu => {
			let area_width = sub_chunks[0].width as usize;
			let inner_width = area_width.saturating_sub(2);
			let item_width = 10; // "C00:100% |" is 10 chars
			let max_cols = (inner_width / item_width).max(1);
			
			let cpus = app.sys.cpus();
			let mut lines = Vec::new();

			for chunk in cpus.chunks(max_cols) {
				let mut line_spans = Vec::new();
				for (i, cpu) in chunk.iter().enumerate() {
					let idx = cpus.iter().position(|x| std::ptr::eq(x, cpu)).unwrap_or(0);
					let usage = cpu.cpu_usage();
					let color = if usage > 80.0 {
						Color::Rgb(255, 85, 85) // Red
					} else if usage > 40.0 {
						Color::Rgb(255, 215, 0) // Yellow
					} else {
						Color::Rgb(80, 250, 123) // Green
					};
					line_spans.push(Span::styled(
						format!("C{:02}:{:3.0}%", idx, usage),
						Style::default().fg(color)
					));
					if i < chunk.len() - 1 {
						line_spans.push(Span::styled(" │ ", Style::default().fg(theme.border)));
					}
				}
				lines.push(Line::from(line_spans));
			}

			let p = Paragraph::new(lines)
				.block(Block::default()
					.borders(Borders::ALL)
					.title(" CPU ")
					.title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
					.border_style(Style::default().fg(border_color))
				);
			f.render_widget(p, sub_chunks[0]);
		}
		FocusedSection::Memory => {
			let header = Row::new(vec!["Metric Type", "Allocated Value"])
				.style(Style::default().fg(Color::Rgb(255, 121, 198)).add_modifier(Modifier::BOLD))
				.bottom_margin(1);
			let total_ram = app.sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
			let used_ram = app.sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
			let free_ram = app.sys.free_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
			let avail_ram = app.sys.available_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
			let total_swap = app.sys.total_swap() as f64 / 1024.0 / 1024.0 / 1024.0;
			let used_swap = app.sys.used_swap() as f64 / 1024.0 / 1024.0 / 1024.0;
			let free_swap = app.sys.free_swap() as f64 / 1024.0 / 1024.0 / 1024.0;

			let mem_stats = vec![
				("Total Physical RAM", format!("{:.2} GB", total_ram)),
				("Used Physical RAM", format!("{:.2} GB", used_ram)),
				("Free Physical RAM", format!("{:.2} GB", free_ram)),
				("Available Physical RAM", format!("{:.2} GB", avail_ram)),
				("Total Pagefile Swap (Swapfile)", format!("{:.2} GB", total_swap)),
				("Used Pagefile Swap (Swapfile)", format!("{:.2} GB", used_swap)),
				("Free Pagefile Swap (Swapfile)", format!("{:.2} GB", free_swap)),
			];
			let mut rows = Vec::new();
			for (metric, val) in mem_stats {
				rows.push(Row::new(vec![
					Cell::from(metric).style(Style::default().fg(theme.text_main).add_modifier(Modifier::BOLD)),
					Cell::from(val).style(Style::default().fg(Color::Rgb(255, 121, 198))),
				]));
			}
			let table = Table::new(rows, [Constraint::Percentage(60), Constraint::Percentage(40)])
				.header(header)
				.block(Block::default()
					.borders(Borders::ALL)
					.title(" Memory ")
					.title_style(Style::default().fg(Color::Rgb(255, 121, 198)).add_modifier(Modifier::BOLD))
					.border_style(Style::default().fg(border_color))
				);
			f.render_widget(table, sub_chunks[0]);
		}
		FocusedSection::Disk => {
			let header = Row::new(vec!["Partition", "Format", "Used Space", "Total", "Free"])
				.style(Style::default().fg(Color::Rgb(160, 32, 240)).add_modifier(Modifier::BOLD))
				.bottom_margin(1);
			let mut rows = Vec::new();
			for disk in &app.disks {
				let total = disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0;
				let avail = disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0;
				let used = total - avail;
				rows.push(Row::new(vec![
					Cell::from(disk.mount_point().to_string_lossy().to_string()).style(Style::default().fg(theme.text_main).add_modifier(Modifier::BOLD)),
					Cell::from(disk.file_system().to_string_lossy().to_string()).style(Style::default().fg(theme.text_dim)),
					Cell::from(format!("{:.1} GB", used)).style(Style::default().fg(Color::Rgb(160, 32, 240))),
					Cell::from(format!("{:.1} GB", total)).style(Style::default().fg(theme.text_dim)),
					Cell::from(format!("{:.1} GB", avail)).style(Style::default().fg(Color::Rgb(80, 250, 123))),
				]));
			}
			let table = Table::new(rows, [
				Constraint::Percentage(20),
				Constraint::Percentage(20),
				Constraint::Percentage(20),
				Constraint::Percentage(20),
				Constraint::Percentage(20),
			])
			.header(header)
			.block(Block::default()
				.borders(Borders::ALL)
				.title(" Storage ")
				.title_style(Style::default().fg(Color::Rgb(160, 32, 240)).add_modifier(Modifier::BOLD))
				.border_style(Style::default().fg(border_color))
			);
			f.render_widget(table, sub_chunks[0]);
		}
		FocusedSection::Gpu => {
			let header = Row::new(vec!["Index", "Display Adapter Desc", "Engine Load"])
				.style(Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD))
				.bottom_margin(1);
			let mut rows = Vec::new();
			for (idx, name) in app.gpu_names.iter().enumerate() {
				let load = if idx == 0 { app.gpu1_spring.value } else { app.gpu2_spring.value };
				rows.push(Row::new(vec![
					Cell::from(format!("GPU{}", idx + 1)).style(Style::default().fg(theme.text_dim)),
					Cell::from(name.clone()).style(Style::default().fg(theme.text_main).add_modifier(Modifier::BOLD)),
					Cell::from(format!("{:.1}%", load)).style(Style::default().fg(Color::Rgb(255, 215, 0))),
				]));
			}
			let table = Table::new(rows, [Constraint::Percentage(15), Constraint::Percentage(60), Constraint::Percentage(25)])
				.header(header)
				.block(Block::default()
					.borders(Borders::ALL)
					.title(" GPU ")
					.title_style(Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD))
					.border_style(Style::default().fg(border_color))
				);
			f.render_widget(table, sub_chunks[0]);
		}
		FocusedSection::Network => {
			let header = Row::new(vec!["Interface", "Status", "MAC Address", "RX Delta", "TX Delta", "Total RX", "Total TX"])
				.style(Style::default().fg(Color::Rgb(80, 250, 123)).add_modifier(Modifier::BOLD))
				.bottom_margin(1);
			let mut rows = Vec::new();
			
			// Sort interfaces: Connected first, then alphabetically
			let mut nets: Vec<(&String, &sysinfo::NetworkData)> = app.networks.iter().collect();
			nets.sort_by(|a, b| {
				let status_a = app.net_statuses.get(a.0).map(|s| s.as_str()).unwrap_or("Disconnected");
				let status_b = app.net_statuses.get(b.0).map(|s| s.as_str()).unwrap_or("Disconnected");
				
				let is_conn_a = status_a == "Connected";
				let is_conn_b = status_b == "Connected";
				
				if is_conn_a != is_conn_b {
					is_conn_b.cmp(&is_conn_a) // Connected first
				} else {
					a.0.cmp(b.0)
				}
			});

			for (name, data) in nets {
				let mac = data.mac_address().to_string();
				let rx_delta = format_speed(data.received() as f64 / 1.5);
				let tx_delta = format_speed(data.transmitted() as f64 / 1.5);
				let rx_total = format_total_bytes(data.total_received());
				let tx_total = format_total_bytes(data.total_transmitted());
				
				let status_str = app.net_statuses.get(name).map(|s| s.as_str()).unwrap_or("Disconnected");
				let status_cell = if status_str == "Connected" {
					Cell::from("Plugged").style(Style::default().fg(Color::Rgb(80, 250, 123)).add_modifier(Modifier::BOLD))
				} else {
					Cell::from("Disconnected").style(Style::default().fg(theme.text_dim))
				};

				rows.push(Row::new(vec![
					Cell::from(name.clone()).style(Style::default().fg(theme.text_main).add_modifier(Modifier::BOLD)),
					status_cell,
					Cell::from(mac).style(Style::default().fg(theme.text_dim)),
					Cell::from(rx_delta).style(Style::default().fg(Color::Rgb(80, 250, 123))),
					Cell::from(tx_delta).style(Style::default().fg(Color::Rgb(255, 215, 0))),
					Cell::from(rx_total).style(Style::default().fg(theme.text_dim)),
					Cell::from(tx_total).style(Style::default().fg(theme.text_dim)),
				]));
			}
			let table = Table::new(rows, [
				Constraint::Percentage(15),
				Constraint::Percentage(10),
				Constraint::Percentage(20),
				Constraint::Percentage(12),
				Constraint::Percentage(12),
				Constraint::Percentage(15),
				Constraint::Percentage(15),
			])
			.header(header)
			.block(Block::default()
				.borders(Borders::ALL)
				.title(" Network ")
				.title_style(Style::default().fg(Color::Rgb(80, 250, 123)).add_modifier(Modifier::BOLD))
				.border_style(Style::default().fg(border_color))
			);
			f.render_widget(table, sub_chunks[0]);
		}
	}

	// 2. Draw Processes Table (unified view containing CPU, Memory, Storage, GPU, Network)
	let rows: Vec<Row> = app.processes.iter().map(|p| {
		let pid = p.pid.to_string();
		let name = p.name.clone();
		let cpu = format!("{:.1}%", p.cpu);
		let mem_mb = p.mem as f64 / 1024.0 / 1024.0;
		let mem = format!("{:.1} MB", mem_mb);
		
		let disk_speed = (p.disk_read + p.disk_write) as f64 / 1.5;
		let storage = if disk_speed > 0.0 {
			format_speed(disk_speed)
		} else {
			"0 B/s".to_string()
		};

		let gpu = format!("{:.1}%", p.gpu);
		let net_speed = p.net as f64;
		let net = if net_speed > 0.0 {
			format_speed(net_speed)
		} else {
			"0 B/s".to_string()
		};

		// Styles based on focus to highlight the active sorted column
		let cpu_style = if app.focus == FocusedSection::Cpu {
			Style::default().fg(Color::Rgb(80, 250, 123)).add_modifier(Modifier::BOLD) // Green
		} else {
			Style::default().fg(theme.text_main)
		};
		
		let mem_style = if app.focus == FocusedSection::Memory {
			Style::default().fg(Color::Rgb(255, 121, 198)).add_modifier(Modifier::BOLD) // Pink
		} else {
			Style::default().fg(theme.text_main)
		};

		let storage_style = if app.focus == FocusedSection::Disk {
			Style::default().fg(Color::Rgb(160, 32, 240)).add_modifier(Modifier::BOLD) // Purple
		} else {
			Style::default().fg(theme.text_main)
		};

		let gpu_style = if app.focus == FocusedSection::Gpu {
			Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD) // Gold
		} else {
			Style::default().fg(theme.text_main)
		};

		let net_style = if app.focus == FocusedSection::Network {
			Style::default().fg(Color::Rgb(0, 245, 255)).add_modifier(Modifier::BOLD) // Electric Cyan
		} else {
			Style::default().fg(theme.text_main)
		};

		Row::new(vec![
			Cell::from(pid).style(Style::default().fg(theme.text_dim)),
			Cell::from(name).style(Style::default().fg(theme.text_main).add_modifier(Modifier::BOLD)),
			Cell::from(cpu).style(cpu_style),
			Cell::from(mem).style(mem_style),
			Cell::from(storage).style(storage_style),
			Cell::from(gpu).style(gpu_style),
			Cell::from(net).style(net_style),
		])
	}).collect();

	let widths = [
		Constraint::Length(8),
		Constraint::Percentage(24),
		Constraint::Percentage(13),
		Constraint::Percentage(14),
		Constraint::Percentage(15),
		Constraint::Percentage(12),
		Constraint::Percentage(14),
	];

	// Highlight header matching focus
	let headers = vec![
		Cell::from("PID").style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
		Cell::from("Process Name").style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
		Cell::from("CPU").style(if app.focus == FocusedSection::Cpu { Style::default().fg(Color::Rgb(80, 250, 123)).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.accent).add_modifier(Modifier::BOLD) }),
		Cell::from("Memory").style(if app.focus == FocusedSection::Memory { Style::default().fg(Color::Rgb(255, 121, 198)).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.accent).add_modifier(Modifier::BOLD) }),
		Cell::from("Storage").style(if app.focus == FocusedSection::Disk { Style::default().fg(Color::Rgb(160, 32, 240)).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.accent).add_modifier(Modifier::BOLD) }),
		Cell::from("GPU").style(if app.focus == FocusedSection::Gpu { Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.accent).add_modifier(Modifier::BOLD) }),
		Cell::from("Network").style(if app.focus == FocusedSection::Network { Style::default().fg(Color::Rgb(0, 245, 255)).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.accent).add_modifier(Modifier::BOLD) }),
	];

	let table_title = match app.focus {
		FocusedSection::Cpu => " Active Processes (Sorted by CPU) ",
		FocusedSection::Memory => " Active Processes (Sorted by RAM) ",
		FocusedSection::Disk => " Active Processes (Sorted by Disk I/O) ",
		FocusedSection::Gpu => " Active Processes (Sorted by GPU) ",
		FocusedSection::Network => " Active Processes (Sorted by Network) ",
	};

	let process_border_color = theme.accent;

	let table = Table::new(rows, widths)
		.header(
			Row::new(headers)
				.style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
				.bottom_margin(1),
		)
		.block(
			Block::default()
				.borders(Borders::ALL)
				.title(table_title)
				.title_style(Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD))
				.border_style(Style::default().fg(process_border_color)),
		)
		.highlight_style(
			Style::default()
				.bg(theme.highlight_bg)
				.fg(Color::Rgb(248, 248, 242))
				.add_modifier(Modifier::BOLD),
		)
		.highlight_symbol("▶ ");

	f.render_stateful_widget(table, sub_chunks[1], &mut app.process_state);
}

fn format_gpu_line(label: &str, name: &str, pct: f64, width: usize) -> Line<'static> {
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
		Span::styled(format!("{}: ", label), Style::default().fg(Color::Rgb(136, 136, 153))),
		Span::styled(display_name, Style::default().fg(Color::Rgb(248, 248, 242))),
		Span::styled(pct_str, Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD)),
	])
}

fn format_speed(bytes_per_sec: f64) -> String {
	if bytes_per_sec >= 1024.0 * 1024.0 {
		format!("{:.1} MB/s", bytes_per_sec / 1024.0 / 1024.0)
	} else if bytes_per_sec >= 1024.0 {
		format!("{:.1} KB/s", bytes_per_sec / 1024.0)
	} else {
		format!("{:.0} B/s", bytes_per_sec)
	}
}

fn draw_spring_bar(width: u16, value: f64, max: f64) -> String {
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
		let blocks = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
		bar.push(blocks[fraction_idx.clamp(0, 8)]);
	}
	
	while bar.chars().count() < width as usize {
		bar.push(' ');
	}
	bar
}

fn log_message(level: &str, msg: &str) {
	if let Ok(app_data) = std::env::var("APPDATA") {
		let log_dir = std::path::Path::new(&app_data).join("rmonitor");
		if std::fs::create_dir_all(&log_dir).is_ok() {
			let log_file = log_dir.join("rmonitor.log");
			if let Ok(mut file) = std::fs::OpenOptions::new()
				.create(true)
				.append(true)
				.open(log_file)
			{
				use std::io::Write;
				let timestamp = if let Ok(n) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
					n.as_secs()
				} else {
					0
				};
				let _ = writeln!(file, "[timestamp={}] [{}] {}", timestamp, level, msg);
			}
		}
	}
}

fn print_json_snapshot() {
	let mut sys = System::new_all();
	sys.refresh_all();
	let disks = sysinfo::Disks::new_with_refreshed_list();
	let networks = sysinfo::Networks::new_with_refreshed_list();
	let gpu_names = get_gpu_names();
	
	let os_name = System::long_os_version().unwrap_or_else(|| System::name().unwrap_or_else(|| "Windows".to_string()));
	let host_name = System::host_name().unwrap_or_else(|| "localhost".to_string());
	let username = std::env::var("USERNAME").unwrap_or_else(|_| std::env::var("USER").unwrap_or_else(|_| "user".to_string()));
	let uptime = System::uptime();
	
	let cpu_usage = sys.global_cpu_info().cpu_usage();
	let cpu_cores = sys.cpus().len();
	
	let total_mem = sys.total_memory();
	let used_mem = sys.used_memory();
	
	let mut total_disk = 0;
	let mut used_disk = 0;
	for disk in &disks {
		total_disk += disk.total_space();
		used_disk += disk.total_space().saturating_sub(disk.available_space());
	}
	
	let net_statuses = get_network_statuses();
	let mut net_interfaces = serde_json::Map::new();
	for (name, data) in &networks {
		let status = net_statuses.get(name).map(|s| s.as_str()).unwrap_or("Disconnected");
		let mut net_info = serde_json::Map::new();
		net_info.insert("status".to_string(), serde_json::Value::String(status.to_string()));
		net_info.insert("mac".to_string(), serde_json::Value::String(data.mac_address().to_string()));
		net_info.insert("total_rx_bytes".to_string(), serde_json::Value::Number(serde_json::Number::from(data.total_received())));
		net_info.insert("total_tx_bytes".to_string(), serde_json::Value::Number(serde_json::Number::from(data.total_transmitted())));
		net_interfaces.insert(name.clone(), serde_json::Value::Object(net_info));
	}
	
	let mut root = serde_json::Map::new();
	root.insert("username".to_string(), serde_json::Value::String(username));
	root.insert("os".to_string(), serde_json::Value::String(os_name));
	root.insert("kernel".to_string(), serde_json::Value::String(System::kernel_version().unwrap_or_default()));
	root.insert("hostname".to_string(), serde_json::Value::String(host_name));
	root.insert("uptime_secs".to_string(), serde_json::Value::Number(serde_json::Number::from(uptime)));
	
	let mut cpu_map = serde_json::Map::new();
	cpu_map.insert("load_percent".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(cpu_usage as f64).unwrap()));
	cpu_map.insert("cores".to_string(), serde_json::Value::Number(serde_json::Number::from(cpu_cores)));
	root.insert("cpu".to_string(), serde_json::Value::Object(cpu_map));
	
	let mut mem_map = serde_json::Map::new();
	mem_map.insert("total_bytes".to_string(), serde_json::Value::Number(serde_json::Number::from(total_mem)));
	mem_map.insert("used_bytes".to_string(), serde_json::Value::Number(serde_json::Number::from(used_mem)));
	root.insert("memory".to_string(), serde_json::Value::Object(mem_map));
	
	let mut disk_map = serde_json::Map::new();
	disk_map.insert("total_bytes".to_string(), serde_json::Value::Number(serde_json::Number::from(total_disk)));
	disk_map.insert("used_bytes".to_string(), serde_json::Value::Number(serde_json::Number::from(used_disk)));
	root.insert("storage".to_string(), serde_json::Value::Object(disk_map));
	
	let gpu_list = serde_json::Value::Array(gpu_names.into_iter().map(serde_json::Value::String).collect());
	root.insert("gpus".to_string(), gpu_list);
	root.insert("network_interfaces".to_string(), serde_json::Value::Object(net_interfaces));
	
	let json_str = serde_json::to_string_pretty(&serde_json::Value::Object(root)).unwrap_or_default();
	println!("{}", json_str);
}

fn run_doctor() {
	println!("==================================================");
	println!("           rMonitor -- System Diagnostics             ");
	println!("==================================================");
	
	#[cfg(windows)]
	{
		let is_admin = std::process::Command::new("net")
			.arg("session")
			.stdout(std::process::Stdio::null())
			.stderr(std::process::Stdio::null())
			.status()
			.map(|s| s.success())
			.unwrap_or(false);
		println!("Privilege Level:   {}", if is_admin { "Administrator (Elevated)" } else { "Standard User" });
	}
	
	let mut sys = System::new_all();
	sys.refresh_all();
	
	let os_name = System::long_os_version().unwrap_or_else(|| System::name().unwrap_or_else(|| "Windows".to_string()));
	let kernel = System::kernel_version().unwrap_or_else(|| "unknown".to_string());
	let host_name = System::host_name().unwrap_or_else(|| "localhost".to_string());
	println!("Operating System:  {}", os_name);
	println!("Kernel Version:    {}", kernel);
	println!("Hostname:          {}", host_name);
	
	#[cfg(windows)]
	{
		use winreg::enums::*;
		use winreg::RegKey;
		let hkcu = RegKey::predef(HKEY_CURRENT_USER);
		let personalize_path = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
		let personalize_ok = hkcu.open_subkey_with_flags(personalize_path, KEY_READ).is_ok();
		println!("Registry (Theme):  {}", if personalize_ok { "Accessible (OK)" } else { "Failed to access personalize theme key" });
		
		let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
		let display_class_path = r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}";
		let display_class_ok = hklm.open_subkey_with_flags(display_class_path, KEY_READ).is_ok();
		println!("Registry (GPU):    {}", if display_class_ok { "Accessible (OK)" } else { "Failed to access display adapters class key" });
	}
	
	let gpu_names = get_gpu_names();
	println!("Detected GPUs ({}):", gpu_names.len());
	for (i, name) in gpu_names.iter().enumerate() {
		println!("  - GPU{}: {}", i + 1, name);
	}
	
	let net_statuses = get_network_statuses();
	println!("Network Statuses (via netsh):");
	if net_statuses.is_empty() {
		println!("  No active statuses found or netsh failed.");
	} else {
		for (name, status) in &net_statuses {
			println!("  - {}: {}", name, status);
		}
	}
	
	if let Ok(app_data) = std::env::var("APPDATA") {
		let log_file = std::path::Path::new(&app_data).join("rmonitor").join("rmonitor.log");
		println!("Log File Path:     {}", log_file.to_string_lossy());
		println!("Log Status:        {}", if log_file.exists() { "Active" } else { "Not created yet" });
	}
	
	println!("==================================================");
	println!("Diagnostics complete!");
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
	let popup_layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Percentage((100 - percent_y) / 2),
			Constraint::Percentage(percent_y),
			Constraint::Percentage((100 - percent_y) / 2),
		])
		.split(r);

	Layout::default()
		.direction(Direction::Horizontal)
		.constraints([
			Constraint::Percentage((100 - percent_x) / 2),
			Constraint::Percentage(percent_x),
			Constraint::Percentage((100 - percent_x) / 2),
		])
		.split(popup_layout[1])[1]
}

fn run_install() {
	println!("==================================================");
	println!("           rMonitor -- Windows Installation           ");
	println!("==================================================");
	
	let exe_path = match std::env::current_exe() {
		Ok(p) => p,
		Err(e) => {
			println!("Failed to locate current executable: {}", e);
			return;
		}
	};
	let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("")).to_path_buf();
	
	// 1. Create Start Menu Shortcut
	println!("Creating Start Menu shortcut...");
	#[cfg(windows)]
	{
		let ps_script = format!(
			r#"$WshShell = New-Object -ComObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut("$env:APPDATA\Microsoft\Windows\Start Menu\Programs\rMonitor.lnk"); $Shortcut.TargetPath = "{}"; $Shortcut.WorkingDirectory = "{}"; $Shortcut.IconLocation = "{},0"; $Shortcut.Save();"#,
			exe_path.to_string_lossy().replace('\\', "\\\\"),
			exe_dir.to_string_lossy().replace('\\', "\\\\"),
			exe_path.to_string_lossy().replace('\\', "\\\\")
		);
		let status = std::process::Command::new("powershell")
			.args(&["-Command", &ps_script])
			.status();
		match status {
			Ok(s) if s.success() => println!("  [OK] Start Menu shortcut created successfully at %APPDATA%\\Microsoft\\Windows\\Start Menu\\Programs\\rMonitor.lnk"),
			_ => println!("  [ERROR] Failed to run PowerShell shortcut creation script"),
		}
	}
	
	// 2. Register under App Paths for Win+R (Run Dialog)
	println!("Registering in user App Paths (Run Dialog)...");
	#[cfg(windows)]
	{
		use winreg::enums::*;
		use winreg::RegKey;
		let hkcu = RegKey::predef(HKEY_CURRENT_USER);
		let path = r"Software\Microsoft\Windows\CurrentVersion\App Paths\rmon.exe";
		match hkcu.create_subkey(path) {
			Ok((key, _)) => {
				let _ = key.set_value("", &exe_path.to_string_lossy().to_string());
				let _ = key.set_value("Path", &exe_dir.to_string_lossy().to_string());
				println!("  [OK] Successfully registered App Path.");
			}
			Err(e) => {
				println!("  [ERROR] Failed to write to registry key: {}", e);
			}
		}
	}
	
	println!("==================================================");
	println!("Installation complete!");
	println!("You can now launch 'rMonitor' from the Start Menu");
	println!("or by pressing Win+R and entering 'rmon'!");
}
