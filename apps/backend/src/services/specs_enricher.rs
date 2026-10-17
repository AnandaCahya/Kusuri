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
                if marketing_name.is_none() {
                    marketing_name = Some(specs.marketing_name);
                }
                if retail_branding.is_none() {
                    retail_branding = specs.retail_branding.or(inferred_brand);
                }
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
            ip_address, network_type, updated_at
        ) VALUES (
            $1, $2, $3, $4,
            $5, $6, $7,
            $8, $9, $10,
            $11, $12, $13, $14,
            $15, $16, $17,
            $18, $19,
            $20, $21, NOW()
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
            ip_address, network_type, updated_at
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
