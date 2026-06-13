//! Custom, lightweight FFI-based shim replacing the external `sysinfo` crate.
//! Works on Windows (direct Win32 FFI) and Linux (direct /proc and /sys filesystem reads).

#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::*;
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::SystemInformation::*;
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::ProcessStatus::*;
#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::*;

#[cfg(target_os = "linux")]
#[repr(C)]
struct StatVfs {
    f_bsize: u64,
    f_frsize: u64,
    f_blocks: u64,
    f_bfree: u64,
    f_bavail: u64,
    f_files: u64,
    f_ffree: u64,
    f_favail: u64,
    f_fsid: u64,
    f_flag: u64,
    f_namemax: u64,
    __f_spare: [i32; 6],
}

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn statvfs(path: *const u8, buf: *mut StatVfs) -> i32;
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pid(pub u32);

impl Pid {
    pub fn from_u32(val: u32) -> Self {
        Pid(val)
    }
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

pub fn get_current_pid() -> Result<Pid, &'static str> {
    Ok(Pid(std::process::id()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    Run,
    Sleep,
    Idle,
    Unknown,
}

impl std::fmt::Display for ProcessStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ProcessStatus::Run => "Run",
            ProcessStatus::Sleep => "Sleep",
            ProcessStatus::Idle => "Idle",
            ProcessStatus::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DiskUsage {
    pub read_bytes: u64,
    pub written_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct Process {
    pid: Pid,
    parent: Option<Pid>,
    name: String,
    exe: Option<PathBuf>,
    cmd: Vec<String>,
    status: ProcessStatus,
    run_time: u64,
    cpu_usage: f32,
    memory: u64,
    disk_usage: DiskUsage,
}

impl Process {
    pub fn pid(&self) -> Pid { self.pid }
    pub fn parent(&self) -> Option<Pid> { self.parent }
    pub fn name(&self) -> &str { &self.name }
    pub fn exe(&self) -> Option<&Path> { self.exe.as_deref() }
    pub fn cmd(&self) -> &[String] { &self.cmd }
    pub fn status(&self) -> ProcessStatus { self.status }
    pub fn run_time(&self) -> u64 { self.run_time }
    pub fn cpu_usage(&self) -> f32 { self.cpu_usage }
    pub fn memory(&self) -> u64 { self.memory }
    pub fn disk_usage(&self) -> DiskUsage { self.disk_usage }
    
    pub fn kill(&self) -> bool {
        #[cfg(target_os = "windows")]
        unsafe {
            use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
            use windows_sys::Win32::Foundation::CloseHandle;
            let h_proc = OpenProcess(PROCESS_TERMINATE, 0, self.pid.0);
            if h_proc != 0 {
                let res = TerminateProcess(h_proc, 1);
                CloseHandle(h_proc);
                res != 0
            } else {
                false
            }
        }
        #[cfg(not(target_os = "windows"))]
        unsafe {
            unsafe extern "C" {
                fn kill(pid: i32, sig: i32) -> i32;
            }
            kill(self.pid.0 as i32, 9) == 0
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Cpu {
    brand: String,
    cpu_usage: f32,
}

impl Cpu {
    pub fn brand(&self) -> &str { &self.brand }
    pub fn cpu_usage(&self) -> f32 { self.cpu_usage }
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessRefreshKind;
impl ProcessRefreshKind {
    pub fn new() -> Self { Self }
    pub fn everything() -> Self { Self }
}

pub struct System {
    global_cpu: Cpu,
    cpus: Vec<Cpu>,
    total_memory: u64,
    used_memory: u64,
    processes: HashMap<Pid, Process>,
    
    // Last cached CPU time counters
    last_system_time: (u64, u64), // (total, idle)
    last_proc_times: HashMap<Pid, u64>, // pid -> process cpu time
}

impl System {
    pub fn new() -> Self {
        let mut sys = Self {
            global_cpu: Cpu::default(),
            cpus: Vec::new(),
            total_memory: 0,
            used_memory: 0,
            processes: HashMap::new(),
            last_system_time: (0, 0),
            last_proc_times: HashMap::new(),
        };
        let num_cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
        sys.cpus = vec![Cpu::default(); num_cores];
        sys.refresh_all();
        sys
    }

    pub fn new_all() -> Self {
        Self::new()
    }

    pub fn global_cpu_info(&self) -> &Cpu {
        &self.global_cpu
    }

    pub fn cpus(&self) -> &[Cpu] {
        &self.cpus
    }

    pub fn total_memory(&self) -> u64 {
        self.total_memory
    }

    pub fn used_memory(&self) -> u64 {
        self.used_memory
    }

    pub fn free_memory(&self) -> u64 {
        self.total_memory.saturating_sub(self.used_memory)
    }

    pub fn available_memory(&self) -> u64 {
        self.total_memory.saturating_sub(self.used_memory)
    }

    pub fn total_swap(&self) -> u64 {
        0
    }

    pub fn used_swap(&self) -> u64 {
        0
    }

    pub fn free_swap(&self) -> u64 {
        0
    }

    pub fn processes(&self) -> &HashMap<Pid, Process> {
        &self.processes
    }

    pub fn process(&self, pid: Pid) -> Option<&Process> {
        self.processes.get(&pid)
    }

    pub fn refresh_all(&mut self) {
        self.refresh_cpu();
        self.refresh_memory();
        self.refresh_processes();
    }

    pub fn refresh_cpu(&mut self) {
        #[cfg(target_os = "windows")]
        unsafe {
            let mut idle = 0u64;
            let mut kernel = 0u64;
            let mut user = 0u64;
            if GetSystemTimes(&mut idle as *mut _ as *mut _, &mut kernel as *mut _ as *mut _, &mut user as *mut _ as *mut _) != 0 {
                let total = kernel.wrapping_add(user);
                let prev_total = self.last_system_time.0;
                let prev_idle = self.last_system_time.1;
                
                let diff_total = total.wrapping_sub(prev_total);
                let diff_idle = idle.wrapping_sub(prev_idle);
                
                if diff_total > 0 {
                    let pct = ((diff_total.saturating_sub(diff_idle)) as f32 / diff_total as f32) * 100.0;
                    self.global_cpu.cpu_usage = pct.clamp(0.0, 100.0);
                    for cpu in &mut self.cpus {
                        cpu.cpu_usage = pct.clamp(0.0, 100.0);
                    }
                }
                self.last_system_time = (total, idle);
            }
        }
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/stat") {
                if let Some(line) = content.lines().next() {
                    let parts: Vec<&str> = line.split_whitespace().skip(1).collect();
                    let mut times = [0u64; 10];
                    for (i, p) in parts.iter().enumerate() {
                        if i < 10 {
                            times[i] = p.parse().unwrap_or(0);
                        }
                    }
                    let idle = times[3] + times[4];
                    let non_idle = times[0] + times[1] + times[2] + times[5] + times[6] + times[7] + times[8] + times[9];
                    let total = idle + non_idle;
                    
                    let prev_total = self.last_system_time.0;
                    let prev_idle = self.last_system_time.1;
                    
                    let diff_total = total.saturating_sub(prev_total);
                    let diff_idle = idle.saturating_sub(prev_idle);
                    
                    if diff_total > 0 {
                        let pct = ((diff_total.saturating_sub(diff_idle)) as f32 / diff_total as f32) * 100.0;
                        self.global_cpu.cpu_usage = pct.clamp(0.0, 100.0);
                        for cpu in &mut self.cpus {
                            cpu.cpu_usage = pct.clamp(0.0, 100.0);
                        }
                    }
                    self.last_system_time = (total, idle);
                }
            }
        }
        
        if self.global_cpu.brand.is_empty() {
            #[cfg(target_os = "windows")]
            {
                if let Some(brand) = crate::backend::registry::read_string(
                    HKEY_LOCAL_MACHINE,
                    "HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0",
                    "ProcessorNameString"
                ) {
                    self.global_cpu.brand = brand.trim().to_string();
                } else {
                    self.global_cpu.brand = "Windows CPU".to_string();
                }
            }
            #[cfg(target_os = "linux")]
            {
                if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
                    for line in content.lines() {
                        if line.starts_with("model name") {
                            if let Some(brand) = line.split(':').nth(1) {
                                self.global_cpu.brand = brand.trim().to_string();
                                break;
                            }
                        }
                    }
                }
                if self.global_cpu.brand.is_empty() {
                    self.global_cpu.brand = "Linux CPU".to_string();
                }
            }
            let brand = self.global_cpu.brand.clone();
            for cpu in &mut self.cpus {
                cpu.brand = brand.clone();
            }
        }
    }

    pub fn refresh_memory(&mut self) {
        #[cfg(target_os = "windows")]
        unsafe {
            let mut status = MEMORYSTATUSEX {
                dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
                dwMemoryLoad: 0,
                ullTotalPhys: 0,
                ullAvailPhys: 0,
                ullTotalPageFile: 0,
                ullAvailPageFile: 0,
                ullTotalVirtual: 0,
                ullAvailVirtual: 0,
                ullAvailExtendedVirtual: 0,
            };
            if GlobalMemoryStatusEx(&mut status) != 0 {
                self.total_memory = status.ullTotalPhys;
                self.used_memory = status.ullTotalPhys.saturating_sub(status.ullAvailPhys);
            }
        }
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
                let mut total_kb = 0u64;
                let mut avail_kb = 0u64;
                let mut free_kb = 0u64;
                for line in content.lines() {
                    if line.starts_with("MemTotal:") {
                        total_kb = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                    } else if line.starts_with("MemAvailable:") {
                        avail_kb = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                    } else if line.starts_with("MemFree:") {
                        free_kb = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                    }
                }
                if avail_kb == 0 {
                    avail_kb = free_kb;
                }
                self.total_memory = total_kb * 1024;
                self.used_memory = total_kb.saturating_sub(avail_kb) * 1024;
            }
        }
    }

    pub fn refresh_processes(&mut self) {
        let mut new_procs = HashMap::new();
        let mut new_proc_times = HashMap::new();
        
        let num_cores = self.cpus.len().max(1) as f32;
        
        #[cfg(target_os = "windows")]
        unsafe {
            use windows_sys::Win32::System::Diagnostics::ToolHelp::*;
            let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snap != INVALID_HANDLE_VALUE {
                let mut pe = PROCESSENTRY32W {
                    dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                    cntUsage: 0,
                    th32ProcessID: 0,
                    th32DefaultHeapID: 0,
                    th32ModuleID: 0,
                    cntThreads: 0,
                    th32ParentProcessID: 0,
                    pcPriClassBase: 0,
                    dwFlags: 0,
                    szExeFile: [0u16; 260],
                };
                if Process32FirstW(snap, &mut pe) != 0 {
                    loop {
                        let pid = pe.th32ProcessID;
                        let ppid = pe.th32ParentProcessID;
                        let name_len = pe.szExeFile.iter().position(|&c| c == 0).unwrap_or(pe.szExeFile.len());
                        let name = String::from_utf16_lossy(&pe.szExeFile[..name_len]);
                        
                        let mut proc_cpu_usage = 0.0f32;
                        let mut memory = 0u64;
                        let mut disk_io = DiskUsage::default();
                        let mut exe_path = None;
                        
                        let h_proc = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ, 0, pid);
                        if h_proc != 0 {
                            let mut pmc = PROCESS_MEMORY_COUNTERS {
                                cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
                                PageFaultCount: 0,
                                PeakWorkingSetSize: 0,
                                WorkingSetSize: 0,
                                QuotaPeakPagedPoolUsage: 0,
                                QuotaPagedPoolUsage: 0,
                                QuotaPeakNonPagedPoolUsage: 0,
                                QuotaNonPagedPoolUsage: 0,
                                PagefileUsage: 0,
                                PeakPagefileUsage: 0,
                            };
                            if K32GetProcessMemoryInfo(h_proc, &mut pmc, pmc.cb) != 0 {
                                memory = pmc.WorkingSetSize as u64;
                            }
                            
                            let mut io_counters = std::mem::zeroed::<IO_COUNTERS>();
                            if GetProcessIoCounters(h_proc, &mut io_counters) != 0 {
                                disk_io.read_bytes = io_counters.ReadTransferCount;
                                disk_io.written_bytes = io_counters.WriteTransferCount;
                            }
                            
                            let mut exe_buf = [0u16; 1024];
                            let mut exe_len = exe_buf.len() as u32;
                            if QueryFullProcessImageNameW(h_proc, 0, exe_buf.as_mut_ptr(), &mut exe_len) != 0 {
                                exe_path = Some(PathBuf::from(String::from_utf16_lossy(&exe_buf[..exe_len as usize])));
                            }
                            
                            let mut creation = 0u64;
                            let mut exit = 0u64;
                            let mut kernel = 0u64;
                            let mut user = 0u64;
                            if GetProcessTimes(h_proc, &mut creation as *mut _ as *mut _, &mut exit as *mut _ as *mut _, &mut kernel as *mut _ as *mut _, &mut user as *mut _ as *mut _) != 0 {
                                let total_proc_time = kernel.wrapping_add(user);
                                new_proc_times.insert(Pid(pid), total_proc_time);
                                
                                if let Some(&prev_proc_time) = self.last_proc_times.get(&Pid(pid)) {
                                    let diff_proc = total_proc_time.wrapping_sub(prev_proc_time);
                                    proc_cpu_usage = (diff_proc as f32 / 150000.0).clamp(0.0, num_cores * 100.0);
                                }
                            }
                            CloseHandle(h_proc);
                        }
                        
                        new_procs.insert(Pid(pid), Process {
                            pid: Pid(pid),
                            parent: if ppid > 0 { Some(Pid(ppid)) } else { None },
                            name,
                            exe: exe_path,
                            cmd: Vec::new(),
                            status: ProcessStatus::Run,
                            run_time: 0,
                            cpu_usage: proc_cpu_usage,
                            memory,
                            disk_usage: disk_io,
                        });
                        
                        if Process32NextW(snap, &mut pe) == 0 {
                            break;
                        }
                    }
                }
                CloseHandle(snap);
            }
        }
        #[cfg(target_os = "linux")]
        {
            if let Ok(entries) = std::fs::read_dir("/proc") {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.chars().all(|c| c.is_ascii_digit()) {
                            if let Ok(pid_val) = name.parse::<u32>() {
                                let pid = Pid(pid_val);
                                let proc_dir = entry.path();
                                
                                if let Ok(stat_content) = std::fs::read_to_string(proc_dir.join("stat")) {
                                    let stat_parts: Vec<&str> = stat_content.split_whitespace().collect();
                                    if stat_parts.len() >= 24 {
                                        let ppid = stat_parts[3].parse::<u32>().unwrap_or(0);
                                        let proc_name = stat_parts[1].trim_start_matches('(').trim_end_matches(')').to_string();
                                        
                                        let state_char = stat_parts[2].chars().next().unwrap_or('?');
                                        let status = match state_char {
                                            'R' => ProcessStatus::Run,
                                            'S' => ProcessStatus::Sleep,
                                            'D' => ProcessStatus::Sleep,
                                            'T' => ProcessStatus::Idle,
                                            _ => ProcessStatus::Unknown,
                                        };
                                        
                                        let utime: u64 = stat_parts[13].parse().unwrap_or(0);
                                        let stime: u64 = stat_parts[14].parse().unwrap_or(0);
                                        let proc_time = utime + stime;
                                        new_proc_times.insert(pid, proc_time);
                                        
                                        let mut cpu_pct = 0.0f32;
                                        if let Some(&prev_time) = self.last_proc_times.get(&pid) {
                                            let diff_proc = proc_time.saturating_sub(prev_time);
                                            cpu_pct = (diff_proc as f32 / 1.5) * 10.0;
                                            cpu_pct = cpu_pct.clamp(0.0, num_cores * 100.0);
                                        }
                                        
                                        let rss_pages = stat_parts[23].parse::<u64>().unwrap_or(0);
                                        let memory = rss_pages * 4096;
                                        
                                        let mut cmd = Vec::new();
                                        if let Ok(cmd_content) = std::fs::read_to_string(proc_dir.join("cmdline")) {
                                            let cmd_args: Vec<String> = cmd_content.split('\0')
                                                .filter(|s| !s.is_empty())
                                                .map(|s| s.to_string())
                                                .collect();
                                            if !cmd_args.is_empty() {
                                                cmd = cmd_args;
                                            }
                                        }
                                        
                                        let exe = std::fs::read_link(proc_dir.join("exe")).ok();
                                        
                                        let mut disk_usage = DiskUsage::default();
                                        if let Ok(io_content) = std::fs::read_to_string(proc_dir.join("io")) {
                                            for line in io_content.lines() {
                                                if line.starts_with("read_bytes:") {
                                                    disk_usage.read_bytes = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                                                } else if line.starts_with("write_bytes:") {
                                                    disk_usage.written_bytes = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                                                }
                                            }
                                        }
                                        
                                        let start_ticks: u64 = stat_parts[21].parse().unwrap_or(0);
                                        let sys_uptime = Self::uptime();
                                        let run_time = sys_uptime.saturating_sub(start_ticks / 100);
                                        
                                        new_procs.insert(pid, Process {
                                            pid,
                                            parent: if ppid > 0 { Some(Pid(ppid)) } else { None },
                                            name: proc_name,
                                            exe,
                                            cmd,
                                            status,
                                            run_time,
                                            cpu_usage: cpu_pct,
                                            memory,
                                            disk_usage,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        self.processes = new_procs;
        self.last_proc_times = new_proc_times;
    }

    pub fn refresh_processes_specifics(&mut self, _kind: ProcessRefreshKind) {
        self.refresh_processes();
    }

    pub fn uptime() -> u64 {
        #[cfg(target_os = "windows")]
        unsafe {
            GetTickCount64() / 1000
        }
        #[cfg(target_os = "linux")]
        {
            std::fs::read_to_string("/proc/uptime")
                .ok()
                .and_then(|s| s.split_whitespace().next().and_then(|v| v.parse::<f32>().ok()))
                .map(|v| v as u64)
                .unwrap_or(0)
        }
    }

    pub fn host_name() -> Option<String> {
        #[cfg(target_os = "windows")]
        {
            std::env::var("COMPUTERNAME").ok()
        }
        #[cfg(target_os = "linux")]
        {
            std::fs::read_to_string("/proc/sys/kernel/hostname")
                .map(|s| s.trim().to_string())
                .ok()
        }
    }

    pub fn kernel_version() -> Option<String> {
        #[cfg(target_os = "windows")]
        {
            std::env::var("OS").ok()
        }
        #[cfg(target_os = "linux")]
        {
            std::fs::read_to_string("/proc/sys/kernel/osrelease")
                .map(|s| s.trim().to_string())
                .ok()
        }
    }

    pub fn long_os_version() -> Option<String> {
        #[cfg(target_os = "windows")]
        {
            Some("Windows".to_string())
        }
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
                for line in content.lines() {
                    if line.starts_with("PRETTY_NAME=") {
                        return Some(line.split('=').nth(1).unwrap_or("").trim_matches('"').to_string());
                    }
                }
            }
            Some("Linux".to_string())
        }
    }

    pub fn name() -> Option<String> {
        #[cfg(target_os = "windows")]
        {
            Some("Windows".to_string())
        }
        #[cfg(target_os = "linux")]
        {
            Some("Linux".to_string())
        }
    }
}

pub struct Disk {
    mount_point: PathBuf,
    file_system: std::ffi::OsString,
    total_space: u64,
    available_space: u64,
}

impl Disk {
    pub fn mount_point(&self) -> &Path { &self.mount_point }
    pub fn file_system(&self) -> &std::ffi::OsStr { &self.file_system }
    pub fn total_space(&self) -> u64 { self.total_space }
    pub fn available_space(&self) -> u64 { self.available_space }
}

pub struct Disks {
    list: Vec<Disk>,
}

impl Disks {
    pub fn new_with_refreshed_list() -> Self {
        let mut disks = Self { list: Vec::new() };
        disks.refresh();
        disks
    }
    
    pub fn refresh(&mut self) {
        #[cfg(target_os = "windows")]
        unsafe {
            use windows_sys::Win32::Storage::FileSystem::{
                GetLogicalDriveStringsW, GetDiskFreeSpaceExW, GetVolumeInformationW,
            };
            let mut buf = [0u16; 512];
            let len = GetLogicalDriveStringsW(buf.len() as u32, buf.as_mut_ptr());
            if len > 0 && len < buf.len() as u32 {
                let mut drives = Vec::new();
                let mut start = 0;
                for i in 0..len as usize {
                    if buf[i] == 0 {
                        if start < i {
                            let drive_w = &buf[start..i+1];
                            let drive_str = String::from_utf16_lossy(&buf[start..i]);
                            let mut free_avail: u64 = 0;
                            let mut total: u64 = 0;
                            let mut total_free: u64 = 0;
                            let res = GetDiskFreeSpaceExW(
                                drive_w.as_ptr(),
                                &mut free_avail,
                                &mut total,
                                &mut total_free
                            );
                            if res != 0 {
                                let mut fs_name = [0u16; 256];
                                let mut serial = 0u32;
                                let mut max_len = 0u32;
                                let mut flags = 0u32;
                                let fs_res = GetVolumeInformationW(
                                    drive_w.as_ptr(),
                                    std::ptr::null_mut(),
                                    0,
                                    &mut serial,
                                    &mut max_len,
                                    &mut flags,
                                    fs_name.as_mut_ptr(),
                                    fs_name.len() as u32,
                                );
                                let fs_str = if fs_res != 0 {
                                    let len = fs_name.iter().position(|&c| c == 0).unwrap_or(fs_name.len());
                                    String::from_utf16_lossy(&fs_name[..len])
                                } else {
                                    "NTFS".to_string()
                                };
                                drives.push(Disk {
                                    mount_point: PathBuf::from(drive_str),
                                    file_system: std::ffi::OsString::from(fs_str),
                                    total_space: total,
                                    available_space: free_avail,
                                });
                            }
                        }
                        start = i + 1;
                    }
                }
                self.list = drives;
            }
        }
        #[cfg(target_os = "linux")]
        {
            let mut list = Vec::new();
            if let Ok(content) = std::fs::read_to_string("/proc/mounts") {
                for line in content.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        let dev = parts[0];
                        let path = parts[1];
                        let fstype = parts[2];
                        if dev.starts_with("/dev/") || path == "/" {
                            let path_c = match std::ffi::CString::new(path) {
                                Ok(p) => p,
                                Err(_) => continue,
                            };
                            unsafe {
                                let mut stat: StatVfs = std::mem::zeroed();
                                if statvfs(path_c.as_ptr() as *const u8, &mut stat) == 0 {
                                    let block_size = if stat.f_frsize > 0 { stat.f_frsize } else { stat.f_bsize };
                                    list.push(Disk {
                                        mount_point: PathBuf::from(path),
                                        file_system: std::ffi::OsString::from(fstype),
                                        total_space: stat.f_blocks * block_size,
                                        available_space: stat.f_bavail * block_size,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            if list.is_empty() {
                list.push(Disk {
                    mount_point: PathBuf::from("/"),
                    file_system: std::ffi::OsString::from("ext4"),
                    total_space: 100 * 1024 * 1024 * 1024,
                    available_space: 50 * 1024 * 1024 * 1024,
                });
            }
            self.list = list;
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Disk> {
        self.list.iter()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }
}

impl<'a> IntoIterator for &'a Disks {
    type Item = &'a Disk;
    type IntoIter = std::slice::Iter<'a, Disk>;
    fn into_iter(self) -> Self::IntoIter {
        self.list.iter()
    }
}

#[derive(Debug, Clone)]
pub struct NetworkData {
    received: u64,
    transmitted: u64,
    total_received: u64,
    total_transmitted: u64,
    mac_address: [u8; 6],
}

impl NetworkData {
    pub fn received(&self) -> u64 { self.received }
    pub fn transmitted(&self) -> u64 { self.transmitted }
    pub fn total_received(&self) -> u64 { self.total_received }
    pub fn total_transmitted(&self) -> u64 { self.total_transmitted }
    
    pub fn mac_address(&self) -> MacAddr {
        MacAddr(self.mac_address)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MacAddr(pub [u8; 6]);
impl std::fmt::Display for MacAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

pub struct Networks {
    map: HashMap<String, NetworkData>,
}

impl Networks {
    pub fn new_with_refreshed_list() -> Self {
        let mut nets = Self { map: HashMap::new() };
        nets.refresh();
        nets
    }
    
    pub fn refresh(&mut self) {
        #[cfg(target_os = "windows")]
        unsafe {
            use windows_sys::Win32::NetworkManagement::IpHelper::{GetIfTable2, FreeMibTable, MIB_IF_TABLE2};
            let mut table: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
            let res = GetIfTable2(&mut table);
            if res == 0 && !table.is_null() {
                let num_entries = (*table).NumEntries as usize;
                let slice = std::slice::from_raw_parts((*table).Table.as_ptr(), num_entries);
                let mut new_map = HashMap::new();
                for row in slice {
                    let len = row.Description.iter().position(|&c| c == 0).unwrap_or(row.Description.len());
                    let desc = String::from_utf16_lossy(&row.Description[..len]);
                    
                    let rx = row.InOctets;
                    let tx = row.OutOctets;
                    
                    let mut mac = [0u8; 6];
                    let mac_len = row.PhysAddrLength as usize;
                    if mac_len >= 6 {
                        mac.copy_from_slice(&row.PhysAddr[..6]);
                    }
                    
                    let prev = self.map.get(&desc);
                    let rx_delta = if let Some(p) = prev { rx.saturating_sub(p.total_received) } else { 0 };
                    let tx_delta = if let Some(p) = prev { tx.saturating_sub(p.total_transmitted) } else { 0 };
                    
                    new_map.insert(desc, NetworkData {
                        received: rx_delta,
                        transmitted: tx_delta,
                        total_received: rx,
                        total_transmitted: tx,
                        mac_address: mac,
                    });
                }
                self.map = new_map;
                FreeMibTable(table as *mut std::ffi::c_void);
            }
        }
        #[cfg(target_os = "linux")]
        {
            let mut new_map = HashMap::new();
            if let Ok(content) = std::fs::read_to_string("/proc/net/dev") {
                for line in content.lines().skip(2) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 9 {
                        let name = parts[0].trim_end_matches(':').to_string();
                        if let (Ok(rx), Ok(tx)) = (parts[1].parse::<u64>(), parts[9].parse::<u64>()) {
                            let mut mac = [0u8; 6];
                            if let Ok(mac_str) = std::fs::read_to_string(format!("/sys/class/net/{}/address", name)) {
                                let mut idx = 0;
                                for hex in mac_str.trim().split(':') {
                                    if idx < 6 {
                                        if let Ok(b) = u8::from_str_radix(hex, 16) {
                                            mac[idx] = b;
                                        }
                                        idx += 1;
                                    }
                                }
                            }
                            
                            let prev = self.map.get(&name);
                            let rx_delta = if let Some(p) = prev { rx.saturating_sub(p.total_received) } else { 0 };
                            let tx_delta = if let Some(p) = prev { tx.saturating_sub(p.total_transmitted) } else { 0 };
                            
                            new_map.insert(name, NetworkData {
                                received: rx_delta,
                                transmitted: tx_delta,
                                total_received: rx,
                                total_transmitted: tx,
                                mac_address: mac,
                            });
                        }
                    }
                }
            }
            self.map = new_map;
        }
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, NetworkData> {
        self.map.iter()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
}

impl<'a> IntoIterator for &'a Networks {
    type Item = (&'a String, &'a NetworkData);
    type IntoIter = std::collections::hash_map::Iter<'a, String, NetworkData>;
    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}
