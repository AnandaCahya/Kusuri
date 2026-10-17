use crate::telemetry::collector::TelemetryCollector;
use chrono::Utc;
use serde_json::{json, Value};
use tracing::info;

pub fn execute_command(
    command: &str,
    payload: Option<Value>,
    collector: &mut TelemetryCollector,
) -> (String, Value) {
    let now = Utc::now().timestamp_millis();
    info!("Mengeksekusi remote command '{}'...", command);

    match command.to_lowercase().as_str() {
        "ping" => {
            let snap = collector.collect_snapshot();
            let res = json!({
                "pong": true,
                "hostname": snap.system.hostname,
                "raw_model": snap.system.raw_model,
                "os": snap.system.os_name,
                "os_version": snap.system.os_version,
                "uptime": snap.system.uptime_formatted,
                "cpu": snap.cpu.brand,
                "ram_used_mb": snap.memory.ram_used_mb,
                "timestamp": now,
            });
            ("completed".to_string(), res)
        }
        "battery_check" => {
            let snap = collector.collect_snapshot();
            let res = json!({
                "has_battery": snap.power.has_battery,
                "battery_percent": snap.power.battery_percent,
                "is_charging": snap.power.is_charging,
                "power_source": snap.power.power_source,
                "battery_health": snap.power.battery_health,
                "timestamp": now,
            });
            ("completed".to_string(), res)
        }
        "status" => {
            let snap = collector.collect_snapshot();
            let res = json!({
                "status": "online",
                "cpu_usage_pct": snap.cpu.global_usage_percent,
                "ram_used_mb": snap.memory.ram_used_mb,
                "ram_total_mb": snap.memory.ram_total_mb,
                "battery_percent": snap.power.battery_percent,
                "is_charging": snap.power.is_charging,
                "primary_ip": snap.network.primary_ip,
                "network_type": snap.network.network_type,
                "timestamp": now,
            });
            ("completed".to_string(), res)
        }
        "system_info" => {
            let snap = collector.collect_snapshot();
            let res = json!({
                "system": snap.system,
                "cpu": snap.cpu,
                "memory": snap.memory,
                "storage": snap.storage,
                "power": snap.power,
                "display": snap.display,
                "network": snap.network,
                "sensors": snap.sensors,
                "timestamp": now,
            });
            ("completed".to_string(), res)
        }
        "notification" => {
            let title = payload
                .as_ref()
                .and_then(|p| p.get("title"))
                .and_then(|t| t.as_str())
                .unwrap_or("Kusuri Remote Alert");
            let message = payload
                .as_ref()
                .and_then(|p| p.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Perintah remote notifikasi diterima dari perangkat lain.");

            info!("NOTIFICATION POPUP: [{}] {}", title, message);

            let res = json!({
                "displayed": true,
                "title": title,
                "message": message,
                "timestamp": now,
            });
            ("completed".to_string(), res)
        }
        "lock_screen" => {
            let locked = trigger_lock_screen();
            let res = json!({
                "action": "lock_screen",
                "success": locked,
                "timestamp": now,
            });
            ("completed".to_string(), res)
        }
        _ => {
            let res = json!({
                "status": "acknowledged",
                "command": command,
                "payload": payload,
                "timestamp": now,
            });
            ("completed".to_string(), res)
        }
    }
}

fn trigger_lock_screen() -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let res = Command::new("rundll32.exe")
            .args(["user32.dll,LockWorkStation"])
            .spawn();
        res.is_ok()
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        if Command::new("loginctl").args(["lock-session"]).status().is_ok() {
            return true;
        }
        if Command::new("xdg-screensaver").args(["lock"]).status().is_ok() {
            return true;
        }
        false
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    false
}
