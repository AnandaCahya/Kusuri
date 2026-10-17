pub mod collector;
#[cfg(target_os = "linux")]
pub mod platform_linux;
#[cfg(target_os = "windows")]
pub mod platform_windows;

use serde::{Deserialize, Serialize};

/// Payload yang dikirim ke endpoint backend `update_device_telemetry`
/// dan via WebSocket `WsEvent::HardwareTelemetry`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeHardwareTelemetryRequest {
    pub raw_model: Option<String>,
    pub imei1: Option<String>,
    pub imei2: Option<String>,
    pub serial_number: Option<String>,
    pub ram_total_mb: Option<i64>,
    pub ram_used_mb: Option<i64>,
    pub ram_free_mb: Option<i64>,
    pub storage_total_mb: Option<i64>,
    pub storage_used_mb: Option<i64>,
    pub storage_free_mb: Option<i64>,
    pub battery_percent: Option<f32>,
    pub battery_charging: Option<bool>,
    pub battery_temp_c: Option<f32>,
    pub battery_health: Option<String>,
    pub os_version: Option<String>,
    pub android_sdk: Option<i32>,
    pub kernel_version: Option<String>,
    pub screen_resolution: Option<String>,
    pub screen_refresh_rate: Option<f32>,
    pub ip_address: Option<String>,
    pub network_type: Option<String>,
    // Rich desktop telemetry fields
    pub cpu_model: Option<String>,
    pub cpu_cores: Option<i32>,
    pub cpu_logical_cores: Option<i32>,
    pub cpu_frequency_mhz: Option<i64>,
    pub gpu_model: Option<String>,
    pub hostname: Option<String>,
    pub architecture: Option<String>,
    pub uptime_formatted: Option<String>,
    pub disks_info: Option<serde_json::Value>,
    pub network_interfaces: Option<serde_json::Value>,
}

/// Payload ringkas untuk broadcast live status ke dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryData {
    pub battery_percent: Option<f32>,
    pub is_charging: Option<bool>,
    pub os_details: Option<String>,
    pub memory_used_mb: Option<u64>,
    pub memory_total_mb: Option<u64>,
    pub ip_address: Option<String>,
}

/// Snapshot telemetri komprehensif maksimal untuk desktop Windows & Linux
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopTelemetrySnapshot {
    pub timestamp_epoch_ms: i64,
    pub system: SystemDetails,
    pub cpu: CpuDetails,
    pub memory: MemoryDetails,
    pub storage: StorageDetails,
    pub power: PowerDetails,
    pub display: DisplayDetails,
    pub network: NetworkDetails,
    pub sensors: SensorDetails,
    pub process: ProcessDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDetails {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub architecture: String,
    pub raw_model: String,
    pub manufacturer: String,
    pub gpu_model: String,
    pub uptime_seconds: u64,
    pub uptime_formatted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuDetails {
    pub brand: String,
    pub vendor_id: String,
    pub physical_cores: usize,
    pub logical_cores: usize,
    pub frequency_mhz: u64,
    pub global_usage_percent: f32,
    pub per_core_usage: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDetails {
    pub ram_total_mb: u64,
    pub ram_used_mb: u64,
    pub ram_free_mb: u64,
    pub ram_available_mb: u64,
    pub ram_usage_percent: f32,
    pub swap_total_mb: u64,
    pub swap_used_mb: u64,
    pub swap_free_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskPartition {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_mb: u64,
    pub used_mb: u64,
    pub free_mb: u64,
    pub is_removable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDetails {
    pub total_mb: u64,
    pub used_mb: u64,
    pub free_mb: u64,
    pub disks: Vec<DiskPartition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerDetails {
    pub has_battery: bool,
    pub battery_percent: Option<f32>,
    pub is_charging: bool,
    pub power_source: String,
    pub battery_health: String,
    pub battery_temp_c: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayDetails {
    pub resolution: String,
    pub refresh_rate_hz: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterfaceInfo {
    pub name: String,
    pub ip_addresses: Vec<String>,
    pub mac_address: String,
    pub total_rx_bytes: u64,
    pub total_tx_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDetails {
    pub primary_ip: Option<String>,
    pub network_type: String,
    pub interfaces: Vec<NetworkInterfaceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorDetails {
    pub cpu_temperature_c: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDetails {
    pub running_processes_count: usize,
    pub client_ram_used_mb: u64,
}

impl DesktopTelemetrySnapshot {
    pub fn to_hardware_telemetry_request(&self) -> NativeHardwareTelemetryRequest {
        NativeHardwareTelemetryRequest {
            raw_model: Some(self.system.raw_model.clone()),
            imei1: None,
            imei2: None,
            serial_number: None,
            ram_total_mb: Some(self.memory.ram_total_mb as i64),
            ram_used_mb: Some(self.memory.ram_used_mb as i64),
            ram_free_mb: Some(self.memory.ram_free_mb as i64),
            storage_total_mb: Some(self.storage.total_mb as i64),
            storage_used_mb: Some(self.storage.used_mb as i64),
            storage_free_mb: Some(self.storage.free_mb as i64),
            battery_percent: self.power.battery_percent,
            battery_charging: Some(self.power.is_charging),
            battery_temp_c: self.power.battery_temp_c,
            battery_health: Some(self.power.battery_health.clone()),
            os_version: Some(self.system.os_version.clone()),
            android_sdk: None,
            kernel_version: Some(self.system.kernel_version.clone()),
            screen_resolution: Some(self.display.resolution.clone()),
            screen_refresh_rate: Some(self.display.refresh_rate_hz),
            ip_address: self.network.primary_ip.clone(),
            network_type: Some(self.network.network_type.clone()),
            cpu_model: Some(self.cpu.brand.clone()),
            cpu_cores: Some(self.cpu.physical_cores as i32),
            cpu_logical_cores: Some(self.cpu.logical_cores as i32),
            cpu_frequency_mhz: Some(self.cpu.frequency_mhz as i64),
            gpu_model: Some(self.system.gpu_model.clone()),
            hostname: Some(self.system.hostname.clone()),
            architecture: Some(self.system.architecture.clone()),
            uptime_formatted: Some(self.system.uptime_formatted.clone()),
            disks_info: Some(serde_json::to_value(&self.storage.disks).unwrap_or_default()),
            network_interfaces: Some(serde_json::to_value(&self.network.interfaces).unwrap_or_default()),
        }
    }

    pub fn to_telemetry_data(&self) -> TelemetryData {
        TelemetryData {
            battery_percent: self.power.battery_percent,
            is_charging: Some(self.power.is_charging),
            os_details: Some(format!("{} ({})", self.system.os_name, self.system.os_version)),
            memory_used_mb: Some(self.memory.ram_used_mb),
            memory_total_mb: Some(self.memory.ram_total_mb),
            ip_address: self.network.primary_ip.clone(),
        }
    }
}
