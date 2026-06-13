use crate::backend::sys_info::{GlyphMap, query_os_version};

pub fn run_doctor() {
    println!("===================================================");
    println!("             pulse Diagnostic Doctor              ");
    println!("===================================================\n");

    let glyphs = GlyphMap::load();

    // 1. Check OS
    let os = query_os_version();
    println!("{} OS: {}", glyphs.info, os);

    let gpu_names = crate::gpu_names::get_gpu_names_sorted();
    println!("\nDetected GPUs ({}):", gpu_names.len());
    for (i, name) in gpu_names.iter().enumerate() {
        println!("  - GPU{}: {}", i + 1, name);
    }

    let net_statuses = crate::network_statuses::get_network_statuses();
    println!("\nNetwork Statuses (via netsh):");
    if net_statuses.is_empty() {
        println!("  No active statuses found or netsh failed.");
    } else {
        for (name, status) in &net_statuses {
            println!("  - {}: {}", name, status);
        }
    }

    println!("\nChecking Log File...");
    if let Some(log_file) = crate::logger::get_appdata_log_path() {
        println!("  Log File Path:     {}", log_file.to_string_lossy());
        println!(
            "  Log Status:        {}",
            if log_file.exists() {
                "Active"
            } else {
                "Not created yet"
            }
        );
    }

    println!("\n===================================================");
    println!("Diagnostics Complete.");
    println!("===================================================");
}

pub fn run_install() {
    println!("==================================================");
    println!("           pulse -- Windows Installation           ");
    println!("==================================================");

    #[cfg(windows)]
    let exe_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            println!("Failed to locate current executable: {}", e);
            return;
        }
    };
    #[cfg(windows)]
    let exe_dir = exe_path
        .parent()
        .unwrap_or(std::path::Path::new(""))
        .to_path_buf();

    println!("Creating Start Menu shortcut...");
    #[cfg(windows)]
    {
        let ps_script = format!(
            r#"$WshShell = New-Object -ComObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut("$env:APPDATA\Microsoft\Windows\Start Menu\Programs\pulse.lnk"); $Shortcut.TargetPath = "{}"; $Shortcut.WorkingDirectory = "{}"; $Shortcut.IconLocation = "{},0"; $Shortcut.Save();"#,
            exe_path.to_string_lossy().replace('\\', "\\\\"),
            exe_dir.to_string_lossy().replace('\\', "\\\\"),
            exe_path.to_string_lossy().replace('\\', "\\\\")
        );
        let status = std::process::Command::new("powershell")
            .args(["-Command", &ps_script])
            .status();
        match status {
            Ok(s) if s.success() => println!(
                "  [OK] Start Menu shortcut created successfully at %APPDATA%\\Microsoft\\Windows\\Start Menu\\Programs\\pulse.lnk"
            ),
            _ => println!("  [ERROR] Failed to run PowerShell shortcut creation script"),
        }
    }

    println!("Registering in user App Paths (Run Dialog)...");
    #[cfg(windows)]
    {
        use crate::backend::registry::RegKey;
        use crate::backend::registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE, KEY_ALL_ACCESS};
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = r"Software\Microsoft\Windows\CurrentVersion\App Paths\pulse.exe";
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
    println!("You can now launch 'pulse' from the Start Menu");
    println!("or by pressing Win+R and entering 'pulse'!");
}
