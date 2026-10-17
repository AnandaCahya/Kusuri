use std::fs;
use std::path::Path;

pub struct LinuxHardwareInfo {
    pub manufacturer: String,
    pub product_name: String,
    pub display_resolution: String,
    pub display_refresh_rate: f32,
    pub battery_percent: Option<f32>,
    pub is_charging: bool,
    pub power_source: String,
    pub battery_health: String,
    pub has_battery: bool,
    pub gpu_name: String,
}

pub fn get_linux_hardware_info() -> LinuxHardwareInfo {
    let (manufacturer, product_name) = query_linux_dmi();
    let (display_resolution, display_refresh_rate) = query_linux_display();
    let (battery_percent, is_charging, power_source, battery_health, has_battery) = query_linux_power();
    let gpu_name = query_linux_gpu();

    LinuxHardwareInfo {
        manufacturer,
        product_name,
        display_resolution,
        display_refresh_rate,
        battery_percent,
        is_charging,
        power_source,
        battery_health,
        has_battery,
        gpu_name,
    }
}

fn query_linux_gpu() -> String {
    if let Ok(output) = std::process::Command::new("lspci").output() {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.contains("VGA compatible controller") || line.contains("3D controller") {
                if let Some((_, name)) = line.split_once(':') {
                    return name.trim().to_string();
                }
            }
        }
    }
    "Linux Display Controller".to_string()
}

fn read_trimmed_file(path: &str) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn query_linux_dmi() -> (String, String) {
    let mfg = read_trimmed_file("/sys/class/dmi/id/sys_vendor")
        .unwrap_or_else(|| "Linux System".to_string());
    let prod = read_trimmed_file("/sys/class/dmi/id/product_name")
        .unwrap_or_else(|| "Linux Desktop".to_string());
    (mfg, prod)
}

fn query_linux_display() -> (String, f32) {
    let drm_path = Path::new("/sys/class/drm");
    if drm_path.exists() {
        if let Ok(entries) = fs::read_dir(drm_path) {
            for entry in entries.flatten() {
                let modes_file = entry.path().join("modes");
                if modes_file.exists() {
                    if let Ok(content) = fs::read_to_string(modes_file) {
                        if let Some(first_mode) = content.lines().next() {
                            let mode = first_mode.trim();
                            if !mode.is_empty() {
                                let parts: Vec<&str> = mode.split('x').collect();
                                if parts.len() == 2 {
                                    return (format!("{} x {}", parts[0], parts[1]), 60.0);
                                }
                                return (mode.to_string(), 60.0);
                            }
                        }
                    }
                }
            }
        }
    }

    ("1920 x 1080".to_string(), 60.0)
}

fn query_linux_power() -> (Option<f32>, bool, String, String, bool) {
    let power_path = Path::new("/sys/class/power_supply");
    if power_path.exists() {
        if let Ok(entries) = fs::read_dir(power_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = entry.file_name().to_string_lossy().to_string();

                if file_name.starts_with("BAT") {
                    let cap = read_trimmed_file(&format!("{}/capacity", path.display()))
                        .and_then(|s| s.parse::<f32>().ok());
                    let status = read_trimmed_file(&format!("{}/status", path.display()))
                        .unwrap_or_else(|| "Unknown".to_string());

                    let is_charging = status.eq_ignore_ascii_case("Charging")
                        || status.eq_ignore_ascii_case("Full");

                    let power_source = if is_charging {
                        "AC Adapter (Charging)".to_string()
                    } else {
                        "Battery Power".to_string()
                    };

                    let health = read_trimmed_file(&format!("{}/health", path.display()))
                        .unwrap_or_else(|| "Good".to_string());

                    return (cap.or(Some(100.0)), is_charging, power_source, health, true);
                }
            }
        }
    }

    // Default desktop Linux tanpa baterai
    (
        Some(100.0),
        true,
        "AC Power (Desktop)".to_string(),
        "Normal / Desktop".to_string(),
        false,
    )
}
