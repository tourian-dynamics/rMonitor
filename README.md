# ❖ rMonitor — Rust System Monitor ❖

`rMonitor` is a lightweight, responsive, and high-performance terminal system monitor (TUI) custom-tailored for Windows, built using **Rust** and the **Ratatui** framework.

![rMonitor Screenshot](rmon_icon_256.png)

## 🚀 Features
*   **Compact CPU Metrics:** Displays global CPU usage, logical core counts, and CPU architecture in a compact core grid supporting up to 64+ cores.
*   **Memory & Swap Mapping:** Shows physical RAM utilization and full Pagefile Swap memory maps.
*   **Disk Storage Breakdown:** Displays drive formats, partitions, total size, and free/used space.
*   **Multi-GPU Adaptability:** Dynamically reads all discrete and integrated GPU adapters directly from the Windows Registry.
*   **Connected Network Sorting:** Checks active network adapters (`Ethernet`, `Wi-Fi`, `Bluetooth`) and displays their connection status (`Plugged` vs `Disconnected`), bubbling active connections to the top of the interface list.
*   **Unified Process Table:** Lists active process PIDs, Names, CPU%, RAM (MB), Storage (Disk I/O), GPU%, and Network speed (bytes/s).
*   **Context-Sensitive Sorting:** Dynamically sorts the process list by the resource matching the currently focused panel.
*   **Neofetch-style Title Banner:** Displays username, hostname, OS version, kernel release, and uptime in a compact layout.
*   **Automatic Resizing:** Auto-resizes the terminal window to a compact 110x38 characters layout upon startup.
*   **Diagnostics & JSON Snapshots:** Built-in subcommands for system health audits (`--doctor`) and structured snapshots (`--json`).

---

## ⌨️ Interface Controls
*   `Tab` : Cycle focus between panels (CPU, Memory, Storage, GPU, Network).
*   `↑` / `k` : Move process list cursor up.
*   `↓` / `j` : Move process list cursor down.
*   `F9` / `K` / `Delete` : Safely terminate (kill) the selected process (opens confirmation dialog).
*   `Enter` : View full process details modal.
*   `Esc` / `q` : Safely exit the application or close active modals/popups.

---

## 🩺 Command Line Subcommands
`rMonitor` can be executed with subcommands directly from your console:

### 1. JSON Data Snapshot (`--json`)
Dumps all current hardware statistics and network details to standard output in clean JSON format:
```powershell
.\rmon.exe --json
```

### 2. Doctor Diagnostic System (`--doctor` / `doctor`)
Runs a quick health check of the monitoring environment, checking execution privilege levels, registry access permissions, GPU hardware paths, and connection interfaces:
```powershell
.\rmon.exe --doctor
```

### 3. Native Windows Installation (`--install` / `install`)
Registers `rMonitor` under the user's `App Paths` registry entry (enabling execution via `Win + R` as `rmon`) and creates a shortcut in the Windows Start Menu programs folder (making it instantly searchable/launchable via the Start Menu):
```powershell
.\rmon.exe --install
```

---

## 📝 Error Logging
All runtime activities, errors, and system crashes (including panics) are automatically captured and logged silently to:
`%APPDATA%\rmonitor\rmonitor.log` (typically `C:\Users\<User>\AppData\Roaming\rmonitor\rmonitor.log`)

---

## 🛠️ Building From Source
Ensure you have the Rust compiler toolchain installed on Windows.

1. Clone the repository and navigate to the folder:
    ```powershell
    cd rmonitor
    ```
2. Build the release binary:
    ```powershell
    .\build.bat
    ```
    This will generate an optimized executable with embedded application resource icons directly at the root (`rmon.exe`).
