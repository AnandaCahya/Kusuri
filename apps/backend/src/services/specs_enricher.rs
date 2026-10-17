use crate::error::AppError;
use crate::services::google_devices::lookup_device_by_codename;
use crate::services::gsmarena::{get_or_fetch_gsmarena_specs, DeviceCatalogSpecs};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeHardwareTelemetryRequest {
    pub raw_model: Option<String>, // e.g. "Infinix-X6881"
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HardwareTelemetryRow {
    pub device_id: Uuid,
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
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedDeviceSpecsResponse {
    pub device_id: Uuid,
    pub name: String,
    pub device_type: String,
    pub raw_model: Option<String>,
    pub marketing_name: Option<String>,
    pub retail_branding: Option<String>,
    pub chipset: Option<String>,
    pub is_online: bool,
    pub catalog_specs: Option<DeviceCatalogSpecs>,
    pub live_hardware: Option<HardwareTelemetryRow>,
}

pub async fn enrich_device_and_store(
    pool: &PgPool,
    device_id: Uuid,
    telemetry: NativeHardwareTelemetryRequest,
) -> Result<EnrichedDeviceSpecsResponse, AppError> {
    let mut marketing_name: Option<String> = None;
    let mut retail_branding: Option<String> = None;
    let mut chipset: Option<String> = None;
    let mut specs_id: Option<Uuid> = None;

    if let Some(raw_model) = &telemetry.raw_model {
        info!("Resolving device model '{}' via Google Play catalog...", raw_model);
        if let Some(matched) = lookup_device_by_codename(pool, raw_model).await {
            info!(
                "Matched: Branding='{}', Marketing='{}' for raw model '{}'",
                matched.retail_branding, matched.marketing_name, raw_model
            );
            retail_branding = Some(matched.retail_branding.clone());
            marketing_name = Some(matched.marketing_name.clone());

            // Fetch or get cached GSM Arena specifications
            match get_or_fetch_gsmarena_specs(pool, &matched.marketing_name, Some(&matched.retail_branding)).await {
                Ok(specs) => {
                    chipset = specs.chipset;
                    specs_id = Some(specs.id);
                }
                Err(e) => {
                    warn!("Failed fetching specs for {}: {}", matched.marketing_name, e);
                }
            }
        }
    }

    if specs_id.is_none() {
        if let Some(raw_model) = &telemetry.raw_model {
            let clean_model = raw_model.replace('-', " ");
            // Infer brand if retail_branding is not yet known (e.g. "ZTE 8045" -> brand "ZTE")
            let inferred_brand = retail_branding.clone().or_else(|| {
                clean_model.split_whitespace().next().map(|s| s.to_string())
            });

            if let Ok(specs) = get_or_fetch_gsmarena_specs(pool, &clean_model, inferred_brand.as_deref()).await {
                chipset = specs.chipset;
                specs_id = Some(specs.id);
                let resolved_comm = specs
                    .raw_specs
                    .as_ref()
                    .and_then(|r| r.get("commercial_name"))
                    .and_then(|c| c.as_str())
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.to_string());

                if let Some(comm) = resolved_comm {
                    marketing_name = Some(comm);
                } else if marketing_name.is_none() {
                    marketing_name = Some(specs.marketing_name);
                }
                if retail_branding.is_none() {
                    retail_branding = specs.retail_branding.or(inferred_brand);
                }
            }
        }
    }

    if chipset.is_none() {
        if let Some(ref cpu) = telemetry.cpu_model {
            chipset = Some(cpu.clone());
        }
    }

    // Synthesize rich collective specifications for Desktop / PC devices if no online catalog matched
    if specs_id.is_none() {
        let dev_type_res = sqlx::query_scalar::<_, String>("SELECT device_type FROM devices WHERE id = $1")
            .bind(device_id)
            .fetch_optional(pool)
            .await?
            .unwrap_or_default()
            .to_lowercase();

        if dev_type_res == "windows" || dev_type_res == "linux" || dev_type_res == "desktop" {
            let synth_marketing = marketing_name
                .clone()
                .or_else(|| telemetry.raw_model.clone())
                .unwrap_or_else(|| format!("{} Desktop", dev_type_res));
            let synth_branding = retail_branding
                .clone()
                .or_else(|| telemetry.hostname.clone())
                .unwrap_or_else(|| "PC System".to_string());
            let synth_cpu = telemetry.cpu_model.clone().map(|brand| {
                if let (Some(c), Some(t)) = (telemetry.cpu_cores, telemetry.cpu_logical_cores) {
                    format!("{} ({} Cores, {} Threads)", brand, c, t)
                } else {
                    brand
                }
            });
            let synth_gpu = telemetry.gpu_model.clone();
            let synth_display = telemetry.screen_resolution.clone().map(|res| {
                if let Some(hz) = telemetry.screen_refresh_rate {
                    format!("{} @ {:.0}Hz", res, hz)
                } else {
                    res
                }
            });
            let synth_memory = if let (Some(r), Some(s)) = (telemetry.ram_total_mb, telemetry.storage_total_mb) {
                Some(format!("{:.1} GB RAM / {:.1} GB Storage", r as f64 / 1024.0, s as f64 / 1024.0))
            } else {
                None
            };
            let synth_os = telemetry.os_version.clone().or_else(|| Some(dev_type_res.clone()));

            let synth_raw_specs = serde_json::json!({
                "sources": ["Local Desktop Hardware", "System BIOS & WMI"],
                "source_urls": {},
                "disks": telemetry.disks_info,
                "network_interfaces": telemetry.network_interfaces,
                "hostname": telemetry.hostname,
                "architecture": telemetry.architecture,
                "uptime": telemetry.uptime_formatted,
            });

            let sid = Uuid::new_v4();
            let synth_inserted = sqlx::query_scalar::<_, Uuid>(
                r#"INSERT INTO device_catalog_specs (
                    id, marketing_name, retail_branding, chipset, cpu, gpu,
                    display_type, display_size, display_resolution, os_original,
                    battery_spec, charging_spec, camera_spec, memory_spec, image_url, raw_specs,
                    created_at, updated_at
                ) VALUES (
                    $1, $2, $3, $4, $5, $6,
                    'Desktop Monitor / Built-in Display', $7, $8, $9,
                    $10, 'AC Power Supply / Adapter', 'Webcam / External USB Camera', $11, NULL, $12,
                    NOW(), NOW()
                )
                ON CONFLICT (marketing_name) DO UPDATE SET
                    retail_branding = COALESCE(EXCLUDED.retail_branding, device_catalog_specs.retail_branding),
                    chipset = COALESCE(EXCLUDED.chipset, device_catalog_specs.chipset),
                    cpu = COALESCE(EXCLUDED.cpu, device_catalog_specs.cpu),
                    gpu = COALESCE(EXCLUDED.gpu, device_catalog_specs.gpu),
                    display_size = COALESCE(EXCLUDED.display_size, device_catalog_specs.display_size),
                    display_resolution = COALESCE(EXCLUDED.display_resolution, device_catalog_specs.display_resolution),
                    os_original = COALESCE(EXCLUDED.os_original, device_catalog_specs.os_original),
                    battery_spec = COALESCE(EXCLUDED.battery_spec, device_catalog_specs.battery_spec),
                    memory_spec = COALESCE(EXCLUDED.memory_spec, device_catalog_specs.memory_spec),
                    raw_specs = EXCLUDED.raw_specs,
                    updated_at = NOW()
                RETURNING id"#,
            )
            .bind(sid)
            .bind(&synth_marketing)
            .bind(&synth_branding)
            .bind(&chipset)
            .bind(&synth_cpu)
            .bind(&synth_gpu)
            .bind(&synth_display)
            .bind(&telemetry.screen_resolution)
            .bind(&synth_os)
            .bind(telemetry.battery_health.as_deref().or(Some("Direct AC Power")))
            .bind(&synth_memory)
            .bind(Some(synth_raw_specs))
            .fetch_one(pool)
            .await?;

            specs_id = Some(synth_inserted);
            if marketing_name.is_none() {
                marketing_name = Some(synth_marketing);
            }
            if retail_branding.is_none() {
                retail_branding = Some(synth_branding);
            }
        }
    }

    // Update devices table
    sqlx::query(
        r#"UPDATE devices 
           SET raw_model = COALESCE($1, raw_model),
               marketing_name = COALESCE($2, marketing_name),
               retail_branding = COALESCE($3, retail_branding),
               chipset = COALESCE($4, chipset),
               specs_id = COALESCE($5, specs_id),
               last_seen_at = NOW()
           WHERE id = $6"#,
    )
    .bind(&telemetry.raw_model)
    .bind(&marketing_name)
    .bind(&retail_branding)
    .bind(&chipset)
    .bind(specs_id)
    .bind(device_id)
    .execute(pool)
    .await?;

    // Upsert live hardware telemetry
    sqlx::query(
        r#"INSERT INTO device_hardware_telemetry (
            device_id, imei1, imei2, serial_number,
            ram_total_mb, ram_used_mb, ram_free_mb,
            storage_total_mb, storage_used_mb, storage_free_mb,
            battery_percent, battery_charging, battery_temp_c, battery_health,
            os_version, android_sdk, kernel_version,
            screen_resolution, screen_refresh_rate,
            ip_address, network_type,
            cpu_model, cpu_cores, cpu_logical_cores, cpu_frequency_mhz,
            gpu_model, hostname, architecture, uptime_formatted,
            disks_info, network_interfaces,
            updated_at
        ) VALUES (
            $1, $2, $3, $4,
            $5, $6, $7,
            $8, $9, $10,
            $11, $12, $13, $14,
            $15, $16, $17,
            $18, $19,
            $20, $21,
            $22, $23, $24, $25,
            $26, $27, $28, $29,
            $30, $31,
            NOW()
        )
        ON CONFLICT (device_id) DO UPDATE SET
            imei1 = COALESCE(EXCLUDED.imei1, device_hardware_telemetry.imei1),
            imei2 = COALESCE(EXCLUDED.imei2, device_hardware_telemetry.imei2),
            serial_number = COALESCE(EXCLUDED.serial_number, device_hardware_telemetry.serial_number),
            ram_total_mb = COALESCE(EXCLUDED.ram_total_mb, device_hardware_telemetry.ram_total_mb),
            ram_used_mb = COALESCE(EXCLUDED.ram_used_mb, device_hardware_telemetry.ram_used_mb),
            ram_free_mb = COALESCE(EXCLUDED.ram_free_mb, device_hardware_telemetry.ram_free_mb),
            storage_total_mb = COALESCE(EXCLUDED.storage_total_mb, device_hardware_telemetry.storage_total_mb),
            storage_used_mb = COALESCE(EXCLUDED.storage_used_mb, device_hardware_telemetry.storage_used_mb),
            storage_free_mb = COALESCE(EXCLUDED.storage_free_mb, device_hardware_telemetry.storage_free_mb),
            battery_percent = COALESCE(EXCLUDED.battery_percent, device_hardware_telemetry.battery_percent),
            battery_charging = COALESCE(EXCLUDED.battery_charging, device_hardware_telemetry.battery_charging),
            battery_temp_c = COALESCE(EXCLUDED.battery_temp_c, device_hardware_telemetry.battery_temp_c),
            battery_health = COALESCE(EXCLUDED.battery_health, device_hardware_telemetry.battery_health),
            os_version = COALESCE(EXCLUDED.os_version, device_hardware_telemetry.os_version),
            android_sdk = COALESCE(EXCLUDED.android_sdk, device_hardware_telemetry.android_sdk),
            kernel_version = COALESCE(EXCLUDED.kernel_version, device_hardware_telemetry.kernel_version),
            screen_resolution = COALESCE(EXCLUDED.screen_resolution, device_hardware_telemetry.screen_resolution),
            screen_refresh_rate = COALESCE(EXCLUDED.screen_refresh_rate, device_hardware_telemetry.screen_refresh_rate),
            ip_address = COALESCE(EXCLUDED.ip_address, device_hardware_telemetry.ip_address),
            network_type = COALESCE(EXCLUDED.network_type, device_hardware_telemetry.network_type),
            cpu_model = COALESCE(EXCLUDED.cpu_model, device_hardware_telemetry.cpu_model),
            cpu_cores = COALESCE(EXCLUDED.cpu_cores, device_hardware_telemetry.cpu_cores),
            cpu_logical_cores = COALESCE(EXCLUDED.cpu_logical_cores, device_hardware_telemetry.cpu_logical_cores),
            cpu_frequency_mhz = COALESCE(EXCLUDED.cpu_frequency_mhz, device_hardware_telemetry.cpu_frequency_mhz),
            gpu_model = COALESCE(EXCLUDED.gpu_model, device_hardware_telemetry.gpu_model),
            hostname = COALESCE(EXCLUDED.hostname, device_hardware_telemetry.hostname),
            architecture = COALESCE(EXCLUDED.architecture, device_hardware_telemetry.architecture),
            uptime_formatted = COALESCE(EXCLUDED.uptime_formatted, device_hardware_telemetry.uptime_formatted),
            disks_info = COALESCE(EXCLUDED.disks_info, device_hardware_telemetry.disks_info),
            network_interfaces = COALESCE(EXCLUDED.network_interfaces, device_hardware_telemetry.network_interfaces),
            updated_at = NOW()"#,
    )
    .bind(device_id)
    .bind(&telemetry.imei1)
    .bind(&telemetry.imei2)
    .bind(&telemetry.serial_number)
    .bind(telemetry.ram_total_mb)
    .bind(telemetry.ram_used_mb)
    .bind(telemetry.ram_free_mb)
    .bind(telemetry.storage_total_mb)
    .bind(telemetry.storage_used_mb)
    .bind(telemetry.storage_free_mb)
    .bind(telemetry.battery_percent)
    .bind(telemetry.battery_charging)
    .bind(telemetry.battery_temp_c)
    .bind(&telemetry.battery_health)
    .bind(&telemetry.os_version)
    .bind(telemetry.android_sdk)
    .bind(&telemetry.kernel_version)
    .bind(&telemetry.screen_resolution)
    .bind(telemetry.screen_refresh_rate)
    .bind(&telemetry.ip_address)
    .bind(&telemetry.network_type)
    .bind(&telemetry.cpu_model)
    .bind(telemetry.cpu_cores)
    .bind(telemetry.cpu_logical_cores)
    .bind(telemetry.cpu_frequency_mhz)
    .bind(&telemetry.gpu_model)
    .bind(&telemetry.hostname)
    .bind(&telemetry.architecture)
    .bind(&telemetry.uptime_formatted)
    .bind(&telemetry.disks_info)
    .bind(&telemetry.network_interfaces)
    .execute(pool)
    .await?;

    get_device_full_specs(pool, device_id).await
}

pub async fn get_device_full_specs(
    pool: &PgPool,
    device_id: Uuid,
) -> Result<EnrichedDeviceSpecsResponse, AppError> {
    let dev = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, Option<String>, Option<String>, bool, Option<Uuid>)>(
        r#"SELECT name, device_type, raw_model, marketing_name, retail_branding, chipset, is_online, specs_id
           FROM devices WHERE id = $1"#,
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

    let (name, dev_type, raw_model, marketing_name, retail_branding, chipset, is_online, specs_id) = dev;

    let catalog_specs = if let Some(sid) = specs_id {
        sqlx::query_as::<_, DeviceCatalogSpecs>(
            r#"SELECT 
                id, marketing_name, retail_branding, gsmarena_url, chipset, cpu, gpu,
                display_type, display_size, display_resolution, os_original,
                battery_spec, charging_spec, camera_spec, memory_spec, image_url, raw_specs
               FROM device_catalog_specs WHERE id = $1"#,
        )
        .bind(sid)
        .fetch_optional(pool)
        .await?
    } else {
        None
    };

    let live_hardware = sqlx::query_as::<_, HardwareTelemetryRow>(
        r#"SELECT 
            device_id, imei1, imei2, serial_number,
            ram_total_mb, ram_used_mb, ram_free_mb,
            storage_total_mb, storage_used_mb, storage_free_mb,
            battery_percent, battery_charging, battery_temp_c, battery_health,
            os_version, android_sdk, kernel_version,
            screen_resolution, screen_refresh_rate,
            ip_address, network_type,
            cpu_model, cpu_cores, cpu_logical_cores, cpu_frequency_mhz,
            gpu_model, hostname, architecture, uptime_formatted,
            disks_info, network_interfaces,
            updated_at
           FROM device_hardware_telemetry WHERE device_id = $1"#,
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await?;

    Ok(EnrichedDeviceSpecsResponse {
        device_id,
        name,
        device_type: dev_type,
        raw_model,
        marketing_name,
        retail_branding,
        chipset,
        is_online,
        catalog_specs,
        live_hardware,
    })
}
