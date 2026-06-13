//! Traverses the process hierarchy to identify the current shell and terminal emulator.

#[cfg(target_os = "windows")]
pub fn query_shell_and_terminal() -> (String, String) {
    static CELL: std::sync::OnceLock<(String, String)> = std::sync::OnceLock::new();
    CELL.get_or_init(|| {
        let mut shell = "Unknown Shell".to_string();
        let mut terminal = "Unknown Terminal".to_string();

        use crate::backend::sysinfo_shim::System;
        let mut sys = System::new();
        sys.refresh_processes_specifics(crate::backend::sysinfo_shim::ProcessRefreshKind::new());

        let mut current_pid = crate::backend::sysinfo_shim::get_current_pid().ok();
        let mut depth = 0;

        while let Some(pid) = current_pid {
            if depth > 12 {
                break;
            }
            if let Some(process) = sys.process(pid) {
                let name = process.name().to_lowercase();
                if shell == "Unknown Shell" {
                    if name.contains("powershell") || name.contains("pwsh") {
                        shell = "PowerShell".to_string();
                    } else if name == "cmd.exe" || name == "cmd" {
                        shell = "CMD".to_string();
                    } else if name.contains("bash") || name.contains("sh") || name.contains("zsh") {
                        shell = name.replace(".exe", "");
                    }
                }

                if terminal == "Unknown Terminal" {
                    if name.contains("windowsterminal") || name == "openconsole.exe" {
                        terminal = "Windows Terminal".to_string();
                    } else if name.contains("code") {
                        terminal = "VS Code Terminal".to_string();
                    } else if name.contains("alacritty") {
                        terminal = "Alacritty".to_string();
                    } else if name.contains("wezterm") {
                        terminal = "WezTerm".to_string();
                    } else if name.contains("conhost") {
                        terminal = "Windows Console Host".to_string();
                    }
                }
                current_pid = process.parent();
                depth += 1;
            } else {
                break;
            }
        }
        (shell, terminal)
    }).clone()
}

#[cfg(not(target_os = "windows"))]
pub fn query_shell_and_terminal() -> (String, String) {
    ("sh".to_string(), "Terminal".to_string())
}
