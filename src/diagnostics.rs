//! `pulse doctor` and `pulse install` CLI subcommands.

pub fn run_doctor() {
    println!("==================================================");
    println!("           pulse -- System Diagnostics             ");
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
        println!(
            "Privilege Level:   {}",
            if is_admin {
                "Administrator (Elevated)"
            } else {
                "Standard User"
            }
        );
    }

    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();

    let os_name = sysinfo::System::long_os_version()
        .or_else(|| sysinfo::System::name())
        .unwrap_or_else(|| "Windows".to_string());
    let kernel = sysinfo::System::kernel_version().unwrap_or_else(|| "unknown".to_string());
    let host_name = library::lifecycle::foreground::identity::hostname();
    println!("Operating System:  {}", os_name);
    println!("Kernel Version:    {}", kernel);
    println!("Hostname:          {}", host_name);

    #[cfg(windows)]
    {
        use winreg::RegKey;
        use winreg::enums::*;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let personalize_path =
            r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
        let personalize_ok = hkcu
            .open_subkey_with_flags(personalize_path, KEY_READ)
            .is_ok();
        println!(
            "Registry (Theme):  {}",
            if personalize_ok {
                "Accessible (OK)"
            } else {
                "Failed to access personalize theme key"
            }
        );

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let display_class_path =
            r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}";
        let display_class_ok = hklm
            .open_subkey_with_flags(display_class_path, KEY_READ)
            .is_ok();
        println!(
            "Registry (GPU):    {}",
            if display_class_ok {
                "Accessible (OK)"
            } else {
                "Failed to access display adapters class key"
            }
        );
    }

    let gpu_names = crate::gpu_names::get_gpu_names_sorted();
    println!("Detected GPUs ({}):", gpu_names.len());
    for (i, name) in gpu_names.iter().enumerate() {
        println!("  - GPU{}: {}", i + 1, name);
    }

    let net_statuses = crate::network_statuses::get_network_statuses();
    println!("Network Statuses (via netsh):");
    if net_statuses.is_empty() {
        println!("  No active statuses found or netsh failed.");
    } else {
        for (name, status) in &net_statuses {
            println!("  - {}: {}", name, status);
        }
    }

    if let Ok(app_data) = std::env::var("APPDATA") {
        let log_file = std::path::Path::new(&app_data)
            .join("pulse")
            .join("log.txt");
        println!("Log File Path:     {}", log_file.to_string_lossy());
        println!(
            "Log Status:        {}",
            if log_file.exists() {
                "Active"
            } else {
                "Not created yet"
            }
        );
    }

    let clip_ok = library::lifecycle::background::clipboard::copy_text_to_clipboard("pulse Diagnostic Test").is_ok();
    println!(
        "Windows Clipboard: {}",
        if clip_ok {
            "Accessible (OK)"
        } else {
            "Failed to access clipboard"
        }
    );
    println!("==================================================");
    println!("Diagnostics complete!");
}

pub fn run_install() {
    println!("==================================================");
    println!("           pulse -- Windows Installation           ");
    println!("==================================================");

    let exe_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            println!("Failed to locate current executable: {}", e);
            return;
        }
    };
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
        use winreg::RegKey;
        use winreg::enums::*;
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
