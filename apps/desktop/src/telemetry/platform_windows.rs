use std::process::Command;
use tracing::debug;
use windows_sys::Win32::Graphics::Gdi::{
    EnumDisplaySettingsW, DEVMODEW, ENUM_CURRENT_SETTINGS,
};
use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
};

pub struct WindowsHardwareInfo {
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

pub fn get_windows_hardware_info() -> WindowsHardwareInfo {
    let (manufacturer, product_name) = query_bios_registry();
    let (display_resolution, display_refresh_rate) = query_display_metrics();
    let (battery_percent, is_charging, power_source, battery_health, has_battery) = query_power_status();
    let gpu_name = query_gpu_registry();

    WindowsHardwareInfo {
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

fn query_gpu_registry() -> String {
    let output = Command::new("reg")
        .args([
            "query",
            r"HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}",
            "/s",
            "/v",
            "DriverDesc",
        ])
        .output();

    let mut gpus = Vec::new();
    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            let line = line.trim();
            if line.contains("DriverDesc") {
                if let Some(val) = line.split("REG_SZ").nth(1) {
                    let name = val.trim().to_string();
                    if !name.is_empty() && !gpus.contains(&name) {
                        gpus.push(name);
                    }
                }
            }
        }
    }

    if gpus.is_empty() {
        "Integrated Graphics".to_string()
    } else {
        gpus.join(", ")
    }
}

fn query_bios_registry() -> (String, String) {
    // Coba membaca registry HKLM\HARDWARE\DESCRIPTION\System\BIOS via reg query
    let output = Command::new("reg")
        .args(["query", r"HKLM\HARDWARE\DESCRIPTION\System\BIOS"])
        .output();

    let mut mfg = String::new();
    let mut prod = String::new();

    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            let line = line.trim();
            if line.contains("SystemManufacturer") {
                if let Some(val) = line.split("REG_SZ").nth(1) {
                    mfg = val.trim().to_string();
                }
            } else if line.contains("SystemProductName") {
                if let Some(val) = line.split("REG_SZ").nth(1) {
                    prod = val.trim().to_string();
                }
            }
        }
    }

    if mfg.is_empty() || mfg.eq_ignore_ascii_case("System manufacturer") {
        mfg = "Microsoft Corporation".to_string();
    }
    if prod.is_empty() || prod.eq_ignore_ascii_case("System Product Name") {
        prod = "Windows PC".to_string();
    }

    (mfg, prod)
}

fn query_display_metrics() -> (String, f32) {
    unsafe {
        let width = GetSystemMetrics(SM_CXSCREEN);
        let height = GetSystemMetrics(SM_CYSCREEN);

        let mut devmode: DEVMODEW = std::mem::zeroed();
        devmode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;

        let mut refresh_rate = 60.0f32;
        if EnumDisplaySettingsW(std::ptr::null(), ENUM_CURRENT_SETTINGS, &mut devmode) != 0 {
            if devmode.dmDisplayFrequency > 0 {
                refresh_rate = devmode.dmDisplayFrequency as f32;
            }
        }

        let resolution = if width > 0 && height > 0 {
            format!("{} x {}", width, height)
        } else {
            "1920 x 1080".to_string()
        };

        (resolution, refresh_rate)
    }
}

fn query_power_status() -> (Option<f32>, bool, String, String, bool) {
    unsafe {
        let mut status: SYSTEM_POWER_STATUS = std::mem::zeroed();
        if GetSystemPowerStatus(&mut status) != 0 {
            let ac_line = status.ACLineStatus; // 0: offline, 1: online, 255: unknown
            let flag = status.BatteryFlag; // 128: no battery
            let life_pct = status.BatteryLifePercent;

            let has_battery = (flag & 128) == 0 && life_pct != 255;
            let is_charging = ac_line == 1;

            if !has_battery {
                // Desktop PC (tanpa baterai internal)
                (
                    Some(100.0),
                    true,
                    "AC Power (Desktop)".to_string(),
                    "Normal / Desktop".to_string(),
                    false,
                )
            } else {
                let pct = if life_pct <= 100 {
                    life_pct as f32
                } else {
                    100.0
                };

                let power_source = if is_charging {
                    "AC Adapter (Charging)".to_string()
                } else {
                    "Battery Power".to_string()
                };

                let health = if (flag & 4) != 0 {
                    "Critical".to_string()
                } else if (flag & 2) != 0 {
                    "Low".to_string()
                } else if (flag & 1) != 0 {
                    "High".to_string()
                } else {
                    "Good".to_string()
                };

                (Some(pct), is_charging, power_source, health, true)
            }
        } else {
            debug!("GetSystemPowerStatus gagal, menggunakan fallback");
            (Some(100.0), true, "AC Power".to_string(), "Normal".to_string(), false)
        }
    }
}
