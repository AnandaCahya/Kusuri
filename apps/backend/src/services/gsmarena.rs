use crate::error::AppError;
use regex::Regex;
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

    if let Some(specs) = cached {
        info!("Found cached GSM Arena specs for '{}'", clean_name);
        return Ok(specs);
    }

    // 2. Fetch from GSM Arena
    info!("Scraping GSM Arena specifications for '{}'...", clean_name);
    let scraped = scrape_gsmarena(clean_name, retail_branding).await;

    let specs_id = Uuid::new_v4();
    let (
        gsm_url,
        chipset,
        cpu,
        gpu,
        disp_type,
        disp_size,
        disp_res,
        os,
        battery,
        charging,
        camera,
        memory,
        image_url,
        raw_map,
    ) = match scraped {
        Ok(data) => (
            Some(data.url),
            data.specs.get("chipset").cloned(),
            data.specs.get("cpu").cloned(),
            data.specs.get("gpu").cloned(),
            data.specs.get("displaytype").cloned(),
            data.specs.get("displaysize").cloned(),
            data.specs.get("displayres").cloned(),
            data.specs.get("os").cloned(),
            data.specs.get("batdescription1").cloned(),
            data.specs.get("charging").cloned(),
            data.specs.get("cam1modules").cloned(),
            data.specs.get("internalmemory").cloned(),
            data.image_url,
            Some(serde_json::to_value(&data.specs).unwrap_or_default()),
        ),
        Err(e) => {
            warn!(
                "Could not fetch GSM Arena specs for '{}': {}. Using basic placeholder.",
                clean_name, e
            );
            (
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            )
        }
    };

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
            chipset = EXCLUDED.chipset,
            updated_at = NOW()
        RETURNING 
            id, marketing_name, retail_branding, gsmarena_url, chipset, cpu, gpu,
            display_type, display_size, display_resolution, os_original,
            battery_spec, charging_spec, camera_spec, memory_spec, image_url, raw_specs"#,
    )
    .bind(specs_id)
    .bind(clean_name)
    .bind(retail_branding)
    .bind(gsm_url)
    .bind(chipset)
    .bind(cpu)
    .bind(gpu)
    .bind(disp_type)
    .bind(disp_size)
    .bind(disp_res)
    .bind(os)
    .bind(battery)
    .bind(charging)
    .bind(camera)
    .bind(memory)
    .bind(image_url)
    .bind(raw_map)
    .fetch_one(pool)
    .await?;

    Ok(inserted)
}

struct ScrapedData {
    url: String,
    image_url: Option<String>,
    specs: HashMap<String, String>,
}

async fn scrape_gsmarena(
    marketing_name: &str,
    _retail_branding: Option<&str>,
) -> Result<ScrapedData, AppError> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to build HTTP client: {}", e)))?;

    // Search query on GSM Arena
    let search_query = marketing_name.replace(' ', "+");
    let search_url = format!(
        "https://m.gsmarena.com/results.php3?sQuickSearch=yes&sName={}",
        search_query
    );

    let search_resp = client
        .get(&search_url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("GSM Arena search request failed: {}", e)))?;

    let search_html = search_resp
        .text()
        .await
        .map_err(|e| AppError::Internal(format!("Failed reading GSM Arena search response: {}", e)))?;

    // Find first phone link, e.g. <a href="infinix_hot_50_pro-13386.php"> or href="infinix_hot_50-13305.php"
    let link_re = Regex::new(r#"href=["']([a-zA-Z0-9_\-]+-[0-9]+\.php)["']"#)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let phone_link = link_re
        .captures(&search_html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| {
            AppError::NotFound(format!("Device '{}' not found on GSM Arena", marketing_name))
        })?;

    let detail_url = format!("https://www.gsmarena.com/{}", phone_link);
    info!("Found GSM Arena device page: {}", detail_url);

    let detail_resp = client
        .get(&detail_url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("GSM Arena detail request failed: {}", e)))?;

    let detail_html = detail_resp
        .text()
        .await
        .map_err(|e| AppError::Internal(format!("Failed reading GSM Arena detail page: {}", e)))?;

    // Extract specs matching data-spec="key">value<
    let spec_re = Regex::new(r#"data-spec=["']([^"']+)["'][^>]*>([^<]+)<"#)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut specs_map = HashMap::new();
    for cap in spec_re.captures_iter(&detail_html) {
        if let (Some(k), Some(v)) = (cap.get(1), cap.get(2)) {
            let key = k.as_str().trim().to_string();
            let val = v.as_str().trim().to_string();
            specs_map.insert(key, val);
        }
    }

    // Extract device image if present
    let img_re = Regex::new(r#"class=["']specs-photo-main["'][^>]*>\s*<a[^>]*>\s*<img[^>]+src=["']([^"']+)["']"#)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let image_url = img_re
        .captures(&detail_html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string());

    Ok(ScrapedData {
        url: detail_url,
        image_url,
        specs: specs_map,
    })
}
