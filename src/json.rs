//! `pulse --json` snapshot printer.

use sysinfo::{Disks, Networks, System};

use crate::gpu_names;

pub fn print() {
    let mut sys = System::new_all();
    sys.refresh_all();
    let disks = Disks::new_with_refreshed_list();
    let networks = Networks::new_with_refreshed_list();
    let gpu_names = gpu_names::get_gpu_names_sorted();

    let os_name = sysinfo::System::long_os_version()
        .or_else(|| sysinfo::System::name())
        .unwrap_or_else(|| "Windows".to_string());
    let host_name = library::lifecycle::foreground::identity::hostname();
    let username = library::lifecycle::foreground::identity::username();
    let uptime = sysinfo::System::uptime();
    let cpu_usage = sys.global_cpu_info().cpu_usage();
    let cpu_cores = sys.cpus().len();
    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();

    let mut total_disk: u64 = 0;
    let mut used_disk: u64 = 0;
    for disk in &disks {
        total_disk += disk.total_space();
        used_disk += disk.total_space().saturating_sub(disk.available_space());
    }
    let net_statuses = crate::network_statuses::get_network_statuses();
    let mut net_interfaces = serde_json::Map::new();
    for (name, data) in &networks {
        let status = net_statuses
            .get(name)
            .map(|s| s.as_str())
            .unwrap_or("Disconnected");
        let mut info = serde_json::Map::new();
        info.insert("status".into(), serde_json::Value::String(status.to_string()));
        info.insert("mac".into(), serde_json::Value::String(data.mac_address().to_string()));
        info.insert(
            "total_rx_bytes".into(),
            serde_json::Value::Number(serde_json::Number::from(data.total_received())),
        );
        info.insert(
            "total_tx_bytes".into(),
            serde_json::Value::Number(serde_json::Number::from(data.total_transmitted())),
        );
        net_interfaces.insert(name.clone(), serde_json::Value::Object(info));
    }

    let mut root = serde_json::Map::new();
    root.insert("username".into(), serde_json::Value::String(username));
    root.insert("os".into(), serde_json::Value::String(os_name));
    root.insert(
        "kernel".into(),
        serde_json::Value::String(sysinfo::System::kernel_version().unwrap_or_default()),
    );
    root.insert("hostname".into(), serde_json::Value::String(host_name));
    root.insert(
        "uptime_secs".into(),
        serde_json::Value::Number(serde_json::Number::from(uptime)),
    );
    let mut cpu_map = serde_json::Map::new();
    cpu_map.insert(
        "load_percent".into(),
        serde_json::Value::Number(serde_json::Number::from_f64(cpu_usage as f64).unwrap()),
    );
    cpu_map.insert(
        "cores".into(),
        serde_json::Value::Number(serde_json::Number::from(cpu_cores)),
    );
    root.insert("cpu".into(), serde_json::Value::Object(cpu_map));
    let mut mem_map = serde_json::Map::new();
    mem_map.insert(
        "total_bytes".into(),
        serde_json::Value::Number(serde_json::Number::from(total_mem)),
    );
    mem_map.insert(
        "used_bytes".into(),
        serde_json::Value::Number(serde_json::Number::from(used_mem)),
    );
    root.insert("memory".into(), serde_json::Value::Object(mem_map));
    let mut disk_map = serde_json::Map::new();
    disk_map.insert(
        "total_bytes".into(),
        serde_json::Value::Number(serde_json::Number::from(total_disk)),
    );
    disk_map.insert(
        "used_bytes".into(),
        serde_json::Value::Number(serde_json::Number::from(used_disk)),
    );
    root.insert("storage".into(), serde_json::Value::Object(disk_map));
    let gpu_list = serde_json::Value::Array(
        gpu_names.into_iter().map(serde_json::Value::String).collect(),
    );
    root.insert("gpus".into(), gpu_list);
    root.insert("network_interfaces".into(), serde_json::Value::Object(net_interfaces));
    let json_str = serde_json::to_string_pretty(&serde_json::Value::Object(root)).unwrap_or_default();
    println!("{}", json_str);
}
