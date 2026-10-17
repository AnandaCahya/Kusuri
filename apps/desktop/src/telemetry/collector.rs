use crate::telemetry::{
    CpuDetails, DesktopTelemetrySnapshot, DiskPartition, DisplayDetails, MemoryDetails,
    NetworkDetails, NetworkInterfaceInfo, PowerDetails, ProcessDetails, SensorDetails,
    StorageDetails, SystemDetails,
};
use chrono::Utc;
use std::net::IpAddr;
use sysinfo::{Components, Disks, Networks, Pid, System};

pub struct TelemetryCollector {
    sys: System,
    disks: Disks,
    networks: Networks,
    components: Components,
}

impl TelemetryCollector {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();
        let components = Components::new_with_refreshed_list();

        Self {
            sys,
            disks,
            networks,
            components,
        }
    }

    pub fn collect_snapshot(&mut self) -> DesktopTelemetrySnapshot {
        // Refresh all dynamic values
        self.sys.refresh_all();
        self.disks.refresh(true);
        self.networks.refresh(true);
        self.components.refresh(true);

        #[cfg(target_os = "windows")]
        let plat_info = super::platform_windows::get_windows_hardware_info();
        #[cfg(target_os = "linux")]
        let plat_info = super::platform_linux::get_linux_hardware_info();
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        let plat_info = FallbackHardwareInfo::default();

        let timestamp_epoch_ms = Utc::now().timestamp_millis();

        // 1. System Details
        let hostname = System::host_name().unwrap_or_else(|| "localhost".to_string());
        let os_name = System::name().unwrap_or_else(|| {
            if cfg!(target_os = "windows") {
                "Windows".to_string()
            } else {
                "Linux".to_string()
            }
        });
        let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
        let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
        let architecture = {
            let arch = System::cpu_arch();
            if arch.is_empty() {
                "x86_64".to_string()
            } else {
                arch
            }
        };
        let uptime_seconds = System::uptime();
        let uptime_formatted = format_uptime(uptime_seconds);

        // Best display name / raw model
        let raw_model = if !plat_info.product_name.is_empty()
            && !plat_info.product_name.eq_ignore_ascii_case("Windows PC")
            && !plat_info.product_name.eq_ignore_ascii_case("Linux Desktop")
        {
            if !plat_info.manufacturer.is_empty()
                && !plat_info.product_name.to_lowercase().contains(&plat_info.manufacturer.to_lowercase())
            {
                format!("{} {}", plat_info.manufacturer, plat_info.product_name)
            } else {
                plat_info.product_name.clone()
            }
        } else {
            format!("{} Desktop ({})", os_name, hostname)
        };

        let system = SystemDetails {
            hostname,
            os_name,
            os_version,
            kernel_version,
            architecture,
            raw_model,
            manufacturer: plat_info.manufacturer,
            gpu_model: plat_info.gpu_name,
            uptime_seconds,
            uptime_formatted,
        };

        // 2. CPU Details
        let cpus = self.sys.cpus();
        let brand = cpus
            .first()
            .map(|c| c.brand().trim().to_string())
            .unwrap_or_else(|| "CPU".to_string());
        let vendor_id = cpus
            .first()
            .map(|c| c.vendor_id().trim().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        let frequency_mhz = cpus.first().map(|c| c.frequency()).unwrap_or(0);
        let logical_cores = cpus.len();
        let physical_cores = self.sys.physical_core_count().unwrap_or(logical_cores);
        let global_usage_percent = self.sys.global_cpu_usage();
        let per_core_usage: Vec<f32> = cpus.iter().map(|c| c.cpu_usage()).collect();

        let cpu = CpuDetails {
            brand,
            vendor_id,
            physical_cores,
            logical_cores,
            frequency_mhz,
            global_usage_percent,
            per_core_usage,
        };

        // 3. Memory Details (Convert Bytes -> MB)
        let ram_total_mb = self.sys.total_memory() / (1024 * 1024);
        let ram_used_mb = self.sys.used_memory() / (1024 * 1024);
        let ram_free_mb = self.sys.free_memory() / (1024 * 1024);
        let ram_available_mb = self.sys.available_memory() / (1024 * 1024);
        let ram_usage_percent = if ram_total_mb > 0 {
            (ram_used_mb as f32 / ram_total_mb as f32) * 100.0
        } else {
            0.0
        };

        let swap_total_mb = self.sys.total_swap() / (1024 * 1024);
        let swap_used_mb = self.sys.used_swap() / (1024 * 1024);
        let swap_free_mb = self.sys.free_swap() / (1024 * 1024);

        let memory = MemoryDetails {
            ram_total_mb,
            ram_used_mb,
            ram_free_mb,
            ram_available_mb,
            ram_usage_percent,
            swap_total_mb,
            swap_used_mb,
            swap_free_mb,
        };

        // 4. Storage & Disks Details
        let mut disks_list = Vec::new();
        let mut total_storage_mb: u64 = 0;
        let mut total_used_mb: u64 = 0;
        let mut total_free_mb: u64 = 0;

        for disk in self.disks.list() {
            let disk_total_mb = disk.total_space() / (1024 * 1024);
            let disk_free_mb = disk.available_space() / (1024 * 1024);
            let disk_used_mb = disk_total_mb.saturating_sub(disk_free_mb);

            total_storage_mb += disk_total_mb;
            total_used_mb += disk_used_mb;
            total_free_mb += disk_free_mb;

            disks_list.push(DiskPartition {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                file_system: disk.file_system().to_string_lossy().to_string(),
                total_mb: disk_total_mb,
                used_mb: disk_used_mb,
                free_mb: disk_free_mb,
                is_removable: disk.is_removable(),
            });
        }

        let storage = StorageDetails {
            total_mb: total_storage_mb,
            used_mb: total_used_mb,
            free_mb: total_free_mb,
            disks: disks_list,
        };

        // 5. Power & Battery Details
        let power = PowerDetails {
            has_battery: plat_info.has_battery,
            battery_percent: plat_info.battery_percent,
            is_charging: plat_info.is_charging,
            power_source: plat_info.power_source,
            battery_health: plat_info.battery_health,
            battery_temp_c: None,
        };

        // 6. Display Details
        let display = DisplayDetails {
            resolution: plat_info.display_resolution,
            refresh_rate_hz: plat_info.display_refresh_rate,
        };

        // 7. Network Details
        let mut interfaces = Vec::new();
        let mut primary_ip: Option<String> = None;
        let mut detected_type = "Offline".to_string();

        for (interface_name, net_data) in self.networks.iter() {
            let mut ips = Vec::new();
            for ip_net in net_data.ip_networks() {
                let ip = ip_net.addr;
                if !ip.is_loopback() {
                    ips.push(ip.to_string());
                }
            }

            interfaces.push(NetworkInterfaceInfo {
                name: interface_name.clone(),
                ip_addresses: ips,
                mac_address: net_data.mac_address().to_string(),
                total_rx_bytes: net_data.total_received(),
                total_tx_bytes: net_data.total_transmitted(),
            });
        }

        // Cari interface utama dengan prioritas: Wi-Fi > Ethernet > lainnya, mengabaikan 169.254 (link-local)
        for iface in &interfaces {
            let lname = iface.name.to_lowercase();
            let is_virtual = lname.contains("npcap")
                || lname.contains("loopback")
                || lname.contains("vethernet")
                || lname.contains("wsl")
                || lname.contains("vmware")
                || lname.contains("virtual");

            for ip_str in &iface.ip_addresses {
                if let Ok(IpAddr::V4(ipv4)) = ip_str.parse::<IpAddr>() {
                    let is_link_local = ipv4.is_link_local();
                    if !is_link_local && (!is_virtual || primary_ip.is_none()) {
                        let is_wifi = lname.contains("wi-fi") || lname.contains("wifi") || lname.contains("wlan");
                        let is_eth = lname.contains("ethernet") || lname.contains("eth") || lname.contains("en");

                        if primary_ip.is_none() || (!is_virtual && (is_wifi || is_eth)) {
                            primary_ip = Some(ip_str.clone());
                            if is_wifi {
                                detected_type = "WiFi".to_string();
                            } else if is_eth {
                                detected_type = "Ethernet".to_string();
                            } else if is_virtual {
                                detected_type = "Virtual".to_string();
                            } else {
                                detected_type = "Connected".to_string();
                            }

                            if !is_virtual && (is_wifi || is_eth) {
                                break;
                            }
                        }
                    }
                }
            }
        }

        let network = NetworkDetails {
            primary_ip,
            network_type: detected_type,
            interfaces,
        };

        // 8. Sensors (CPU Temp)
        let mut cpu_temp: Option<f32> = None;
        for comp in self.components.list() {
            let label = comp.label().to_lowercase();
            if label.contains("cpu") || label.contains("package") || label.contains("core") {
                if let Some(t) = comp.temperature() {
                    cpu_temp = Some(t);
                    break;
                }
            }
        }
        let sensors = SensorDetails {
            cpu_temperature_c: cpu_temp,
        };

        // 9. Process Details
        let current_pid = Pid::from_u32(std::process::id());
        let client_ram_used_mb = self
            .sys
            .process(current_pid)
            .map(|p| p.memory() / (1024 * 1024))
            .unwrap_or(0);

        let process = ProcessDetails {
            running_processes_count: self.sys.processes().len(),
            client_ram_used_mb,
        };

        DesktopTelemetrySnapshot {
            timestamp_epoch_ms,
            system,
            cpu,
            memory,
            storage,
            power,
            display,
            network,
            sensors,
            process,
        }
    }
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else {
        format!("{}m {}s", minutes, secs)
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
#[derive(Default)]
struct FallbackHardwareInfo {
    pub manufacturer: String,
    pub product_name: String,
    pub display_resolution: String,
    pub display_refresh_rate: f32,
    pub battery_percent: Option<f32>,
    pub is_charging: bool,
    pub power_source: String,
    pub battery_health: String,
    pub has_battery: bool,
}
