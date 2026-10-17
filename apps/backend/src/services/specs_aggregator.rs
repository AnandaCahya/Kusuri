use crate::error::AppError;
use crate::services::devicespecifications::scrape_devicespecifications;
use crate::services::gsmarena::scrape_gsmarena;
use crate::services::matcher::classify_tokens;
use crate::services::nanoreview::scrape_nanoreview;
use crate::services::wikipedia::fetch_from_wikipedia;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DeviceCatalogSpecs {
    pub id: Uuid,
    pub marketing_name: String,
    pub retail_branding: Option<String>,
    pub gsmarena_url: Option<String>,
    pub chipset: Option<String>,
    pub cpu: Option<String>,
    pub gpu: Option<String>,
    pub display_type: Option<String>,
    pub display_size: Option<String>,
    pub display_resolution: Option<String>,
    pub os_original: Option<String>,
    pub battery_spec: Option<String>,
    pub charging_spec: Option<String>,
    pub camera_spec: Option<String>,
    pub memory_spec: Option<String>,
    pub image_url: Option<String>,
    pub raw_specs: Option<serde_json::Value>,
}

#[derive(Default, Clone, Debug)]
pub struct AggregatedSpecs {
    pub source_url: Option<String>,
    pub verified_sources: Vec<String>,
    pub source_urls: HashMap<String, String>,
    pub chipset: Option<String>,
    pub cpu: Option<String>,
    pub gpu: Option<String>,
    pub display_type: Option<String>,
    pub display_size: Option<String>,
    pub display_resolution: Option<String>,
    pub os_original: Option<String>,
    pub battery_spec: Option<String>,
    pub charging_spec: Option<String>,
    pub camera_spec: Option<String>,
    pub memory_spec: Option<String>,
    pub image_url: Option<String>,
    pub raw_map: HashMap<String, String>,
}

/// Collective Multi-Source Specification Resolver & Cross-Validator
/// Aggregates and cross-validates specs across GSM Arena, NanoReview, DeviceSpecifications, and Wikipedia.
pub async fn get_or_fetch_gsmarena_specs(
    pool: &PgPool,
    marketing_name: &str,
    retail_branding: Option<&str>,
) -> Result<DeviceCatalogSpecs, AppError> {
    let clean_name = marketing_name.trim();

    // 1. Check database cache
    let cached = sqlx::query_as::<_, DeviceCatalogSpecs>(
        r#"SELECT 
            id, marketing_name, retail_branding, gsmarena_url, chipset, cpu, gpu,
            display_type, display_size, display_resolution, os_original,
            battery_spec, charging_spec, camera_spec, memory_spec, image_url, raw_specs
           FROM device_catalog_specs 
           WHERE LOWER(marketing_name) = LOWER($1)
           LIMIT 1"#,
    )
    .bind(clean_name)
    .fetch_optional(pool)
    .await?;

    // If cached record has valid collective specs, return it immediately
    if let Some(ref specs) = cached {
        if specs.chipset.is_some() || specs.cpu.is_some() || specs.display_type.is_some() {
            info!("Found cached valid collective device specs for '{}'", clean_name);
            return Ok(specs.clone());
        }
    }

    info!("Starting collective multi-source specification aggregation for '{}'...", clean_name);
    let mut collective = AggregatedSpecs::default();
    let target_tokens = classify_tokens(clean_name, retail_branding);

    // ---------------------------------------------------------------------------------------------
    // SOURCE 1: GSM Arena (Primary Mobile Engine with Smart Selective Matching)
    // ---------------------------------------------------------------------------------------------
    let mut resolved_alias: Option<String> = None;
    match scrape_gsmarena(clean_name, retail_branding).await {
        Ok(gsm) => {
            // Validation: ensure GSM Arena result did not match a conflicting year
            let gsm_tokens = classify_tokens(&gsm.source_url, retail_branding);
            let mut is_valid = true;
            if let (Some(ty), Some(cy)) = (target_tokens.year, gsm_tokens.year) {
                if ty != cy {
                    warn!("Discarding GSM Arena candidate due to release year conflict (target: {}, candidate: {})", ty, cy);
                    is_valid = false;
                }
            }

            if is_valid {
                info!("Verified and collected specifications from GSM Arena for '{}' (commercial: {:?})", clean_name, gsm.commercial_name);
                collective.verified_sources.push("GSM Arena".to_string());
                collective.source_urls.insert("GSM Arena".to_string(), gsm.source_url.clone());
                if collective.source_url.is_none() {
                    collective.source_url = Some(gsm.source_url);
                }

                if let Some(ref comm) = gsm.commercial_name {
                    resolved_alias = Some(comm.clone());
                }

                merge_if_empty(&mut collective.chipset, gsm.chipset);
                merge_if_empty(&mut collective.cpu, gsm.cpu);
                merge_if_empty(&mut collective.gpu, gsm.gpu);
                merge_if_empty(&mut collective.display_type, gsm.display_type);
                merge_if_empty(&mut collective.display_size, gsm.display_size);
                merge_if_empty(&mut collective.display_resolution, gsm.display_resolution);
                merge_if_empty(&mut collective.os_original, gsm.os_original);
                merge_if_empty(&mut collective.battery_spec, gsm.battery_spec);
                merge_if_empty(&mut collective.charging_spec, gsm.charging_spec);
                merge_if_empty(&mut collective.camera_spec, gsm.camera_spec);
                merge_if_empty(&mut collective.memory_spec, gsm.memory_spec);
                merge_if_empty(&mut collective.image_url, gsm.image_url);

                for (k, v) in gsm.raw_map {
                    collective.raw_map.insert(format!("gsm_{}", k), v);
                }
            }
        }
        Err(e) => {
            warn!("GSM Arena source lookup for '{}' notice: {}", clean_name, e);
        }
    }

    // Determine query name for subsequent sources: use resolved commercial name if available (e.g. "ZTE Blade V40 Vita")
    let target_for_catalogs = resolved_alias.as_deref().unwrap_or(clean_name);

    // ---------------------------------------------------------------------------------------------
    // SOURCE 2: NanoReview (Technical Benchmark & Architecture Engine)
    // ---------------------------------------------------------------------------------------------
    match scrape_nanoreview(target_for_catalogs, retail_branding).await {
        Ok(nano) => {
            info!("Verified and collected specifications from NanoReview for '{}'", target_for_catalogs);
            collective.verified_sources.push("NanoReview".to_string());
            collective.source_urls.insert("NanoReview".to_string(), nano.source_url.clone());

            merge_if_empty(&mut collective.chipset, nano.chipset);
            merge_if_empty(&mut collective.cpu, nano.cpu);
            merge_if_empty(&mut collective.gpu, nano.gpu);
            merge_if_empty(&mut collective.display_type, nano.display_type);
            merge_if_empty(&mut collective.display_size, nano.display_size);
            merge_if_empty(&mut collective.battery_spec, nano.battery_spec);
            merge_if_empty(&mut collective.charging_spec, nano.charging_spec);
            merge_if_empty(&mut collective.camera_spec, nano.camera_spec);
            merge_if_empty(&mut collective.memory_spec, nano.memory_spec);
            merge_if_empty(&mut collective.os_original, nano.os_original);
            merge_if_empty(&mut collective.image_url, nano.image_url);

            for (k, v) in nano.raw_map {
                collective.raw_map.insert(format!("nano_{}", k), v);
            }
        }
        Err(e) => {
            info!("NanoReview source lookup for '{}' notice: {}", target_for_catalogs, e);
        }
    }

    // ---------------------------------------------------------------------------------------------
    // SOURCE 3: DeviceSpecifications (Hardware Catalog Engine)
    // ---------------------------------------------------------------------------------------------
    match scrape_devicespecifications(target_for_catalogs, retail_branding).await {
        Ok(ds) => {
            info!("Verified and collected specifications from DeviceSpecifications for '{}'", target_for_catalogs);
            collective.verified_sources.push("DeviceSpecifications".to_string());
            collective.source_urls.insert("DeviceSpecifications".to_string(), ds.source_url.clone());

            merge_if_empty(&mut collective.chipset, ds.chipset);
            merge_if_empty(&mut collective.cpu, ds.cpu);
            merge_if_empty(&mut collective.gpu, ds.gpu);
            merge_if_empty(&mut collective.display_type, ds.display_type);
            merge_if_empty(&mut collective.display_size, ds.display_size);
            merge_if_empty(&mut collective.display_resolution, ds.display_resolution);
            merge_if_empty(&mut collective.battery_spec, ds.battery_spec);
            merge_if_empty(&mut collective.camera_spec, ds.camera_spec);
            merge_if_empty(&mut collective.memory_spec, ds.memory_spec);
            merge_if_empty(&mut collective.os_original, ds.os_original);
            merge_if_empty(&mut collective.image_url, ds.image_url);

            for (k, v) in ds.raw_map {
                collective.raw_map.insert(format!("ds_{}", k), v);
            }
        }
        Err(e) => {
            info!("DeviceSpecifications lookup for '{}' notice: {}", target_for_catalogs, e);
        }
    }

    // ---------------------------------------------------------------------------------------------
    // SOURCE 4: Wikipedia (Fallback Open Source Infobox Engine)
    // ---------------------------------------------------------------------------------------------
    if collective.chipset.is_none() && collective.display_type.is_none() {
        match fetch_from_wikipedia(target_for_catalogs, retail_branding).await {
            Ok(wiki) => {
                info!("Verified and collected specifications from Wikipedia for '{}'", target_for_catalogs);
                collective.verified_sources.push("Wikipedia".to_string());
                collective.source_urls.insert("Wikipedia".to_string(), wiki.source_url.clone());

                merge_if_empty(&mut collective.chipset, wiki.chipset);
                merge_if_empty(&mut collective.cpu, wiki.cpu);
                merge_if_empty(&mut collective.gpu, wiki.gpu);
                merge_if_empty(&mut collective.display_type, wiki.display_type);
                merge_if_empty(&mut collective.battery_spec, wiki.battery_spec);
                merge_if_empty(&mut collective.charging_spec, wiki.charging_spec);
                merge_if_empty(&mut collective.camera_spec, wiki.camera_spec);
                merge_if_empty(&mut collective.memory_spec, wiki.memory_spec);
                merge_if_empty(&mut collective.os_original, wiki.os_original);
                merge_if_empty(&mut collective.image_url, wiki.image_url);

                for (k, v) in wiki.raw_map {
                    collective.raw_map.insert(format!("wiki_{}", k), v);
                }
            }
            Err(e) => {
                info!("Wikipedia lookup for '{}' notice: {}", target_for_catalogs, e);
            }
        }
    }

    let specs_id = cached.map(|c| c.id).unwrap_or_else(Uuid::new_v4);

    // Build raw_specs JSON containing collective metadata, verified sources list, and source URLs
    let raw_payload = serde_json::json!({
        "sources": collective.verified_sources,
        "source_urls": collective.source_urls,
        "commercial_name": resolved_alias,
        "details": collective.raw_map,
    });

    let inserted = sqlx::query_as::<_, DeviceCatalogSpecs>(
        r#"INSERT INTO device_catalog_specs (
            id, marketing_name, retail_branding, gsmarena_url, chipset, cpu, gpu,
            display_type, display_size, display_resolution, os_original,
            battery_spec, charging_spec, camera_spec, memory_spec, image_url, raw_specs,
            created_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7,
            $8, $9, $10, $11,
            $12, $13, $14, $15, $16, $17,
            NOW(), NOW()
        )
        ON CONFLICT (marketing_name) DO UPDATE SET
            retail_branding = COALESCE(EXCLUDED.retail_branding, device_catalog_specs.retail_branding),
            gsmarena_url = COALESCE(EXCLUDED.gsmarena_url, device_catalog_specs.gsmarena_url),
            chipset = COALESCE(EXCLUDED.chipset, device_catalog_specs.chipset),
            cpu = COALESCE(EXCLUDED.cpu, device_catalog_specs.cpu),
            gpu = COALESCE(EXCLUDED.gpu, device_catalog_specs.gpu),
            display_type = COALESCE(EXCLUDED.display_type, device_catalog_specs.display_type),
            display_size = COALESCE(EXCLUDED.display_size, device_catalog_specs.display_size),
            display_resolution = COALESCE(EXCLUDED.display_resolution, device_catalog_specs.display_resolution),
            battery_spec = COALESCE(EXCLUDED.battery_spec, device_catalog_specs.battery_spec),
            charging_spec = COALESCE(EXCLUDED.charging_spec, device_catalog_specs.charging_spec),
            camera_spec = COALESCE(EXCLUDED.camera_spec, device_catalog_specs.camera_spec),
            memory_spec = COALESCE(EXCLUDED.memory_spec, device_catalog_specs.memory_spec),
            image_url = COALESCE(EXCLUDED.image_url, device_catalog_specs.image_url),
            raw_specs = EXCLUDED.raw_specs,
            updated_at = NOW()
        RETURNING 
            id, marketing_name, retail_branding, gsmarena_url, chipset, cpu, gpu,
            display_type, display_size, display_resolution, os_original,
            battery_spec, charging_spec, camera_spec, memory_spec, image_url, raw_specs"#,
    )
    .bind(specs_id)
    .bind(clean_name)
    .bind(retail_branding)
    .bind(collective.source_url)
    .bind(collective.chipset)
    .bind(collective.cpu)
    .bind(collective.gpu)
    .bind(collective.display_type)
    .bind(collective.display_size)
    .bind(collective.display_resolution)
    .bind(collective.os_original)
    .bind(collective.battery_spec)
    .bind(collective.charging_spec)
    .bind(collective.camera_spec)
    .bind(collective.memory_spec)
    .bind(collective.image_url)
    .bind(Some(raw_payload))
    .fetch_one(pool)
    .await?;

    info!("Successfully stored collective unified specifications for '{}'", clean_name);
    Ok(inserted)
}

fn merge_if_empty(target: &mut Option<String>, source: Option<String>) {
    if target.is_none() || target.as_ref().map(|s| s.trim().is_empty()).unwrap_or(false) {
        if let Some(s) = source {
            if !s.trim().is_empty() {
                *target = Some(s);
            }
        }
    }
}
