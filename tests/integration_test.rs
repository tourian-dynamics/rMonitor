use std::process::Command;

#[test]
fn test_pulse_doctor() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--doctor"])
        .output()
        .expect("Failed to execute pulse --doctor");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("STDOUT:\n{}", stdout);
    println!("STDERR:\n{}", stderr);

    assert!(output.status.success(), "pulse --doctor failed to execute successfully");

    // The output should contain header and diagnostic sections
    assert!(stdout.contains("pulse Diagnostic Doctor"), "Output did not contain diagnostic header");
    assert!(stdout.contains("OS:"), "Output did not contain OS check");
    assert!(stdout.contains("Detected GPUs"), "Output did not contain GPU check");
    assert!(stdout.contains("Checking Log File"), "Output did not contain log check");
}

#[test]
fn test_pulse_json() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--json"])
        .output()
        .expect("Failed to execute pulse --json");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("STDOUT:\n{}", stdout);
    println!("STDERR:\n{}", stderr);

    assert!(output.status.success(), "pulse --json failed to execute successfully");

    // Parse output as JSON to verify it's valid
    let json_val: serde_json::Value = serde_json::from_str(&stdout)
        .expect("pulse --json output was not valid JSON");

    // Check presence of top-level keys
    assert!(json_val.get("username").is_some(), "JSON missing username");
    assert!(json_val.get("os").is_some(), "JSON missing os");
    assert!(json_val.get("hostname").is_some(), "JSON missing hostname");
    assert!(json_val.get("uptime_secs").is_some(), "JSON missing uptime_secs");
    assert!(json_val.get("cpu").is_some(), "JSON missing cpu");
    assert!(json_val.get("memory").is_some(), "JSON missing memory");
    assert!(json_val.get("storage").is_some(), "JSON missing storage");
    assert!(json_val.get("gpus").is_some(), "JSON missing gpus");
    assert!(json_val.get("network_interfaces").is_some(), "JSON missing network_interfaces");
}
