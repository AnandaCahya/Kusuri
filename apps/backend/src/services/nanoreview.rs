use crate::error::AppError;
use crate::services::matcher::normalize_for_matching;
use regex::Regex;
use std::collections::HashMap;
use tracing::{info, warn};

#[derive(Default, Clone, Debug)]
pub struct NanoReviewSpecResult {
    pub source_url: String,
    pub chipset: Option<String>,
    pub cpu: Option<String>,
    pub gpu: Option<String>,
    pub display_type: Option<String>,
    pub display_size: Option<String>,
    pub battery_spec: Option<String>,
    pub charging_spec: Option<String>,
    pub camera_spec: Option<String>,
    pub memory_spec: Option<String>,
    pub os_original: Option<String>,
    pub image_url: Option<String>,
    pub raw_map: HashMap<String, String>,
}

/// Scrapes device specifications dynamically from nanoreview.net
pub async fn scrape_nanoreview(
    marketing_name: &str,
    retail_branding: Option<&str>,
) -> Result<NanoReviewSpecResult, AppError> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(6))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to build HTTP client: {}", e)))?;

    let norm = normalize_for_matching(marketing_name);
    let brand = retail_branding.unwrap_or("").to_lowercase();
    let slug_base = if !brand.is_empty() && !norm.contains(&brand) {
        format!("{}-{}", brand, norm)
    } else {
        norm
    };
    let slug = slug_base.replace(' ', "-");

    let url = format!("https://nanoreview.net/en/phone/{}", slug);
    info!("Querying NanoReview at {}", url);

    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => return Err(AppError::Internal(e.to_string())),
    };

    if !resp.status().is_success() {
        return Err(AppError::NotFound(format!("NanoReview returned status {}", resp.status())));
    }

    let html = resp.text().await.map_err(|e| AppError::Internal(e.to_string()))?;
    if html.contains("Just a moment...") || html.contains("404:") {
        warn!("NanoReview challenged or page not found for slug '{}'", slug);
        return Err(AppError::NotFound("NanoReview page not found or challenged".to_string()));
    }

    let extract_cell = |label: &str| -> Option<String> {
        let pattern = format!(r#"<td class="cell-h">{}</td>\s*<td[^>]*>([\s\S]*?)</td>"#, regex::escape(label));
        let re = Regex::new(&pattern).ok()?;
        re.captures(&html).and_then(|c| c.get(1)).map(|m| {
            let tag_re = Regex::new(r#"<[^>]+>"#).unwrap();
            tag_re.replace_all(m.as_str(), " ")
                .replace("&amp;", "&")
                .replace("&nbsp;", " ")
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join(" ")
        })
    };

    let mut raw_map = HashMap::new();

    let chipset = extract_cell("Chipset");
    let cpu = extract_cell("CPU");
    let gpu = extract_cell("Graphics");
    let ram = extract_cell("RAM size");
    let storage = extract_cell("Storage size");
    let battery = extract_cell("Capacity");
    let charging = extract_cell("Fast charging");
    let os = extract_cell("Operating system");
    let display = extract_cell("Matrix").or_else(|| extract_cell("Type"));
    let display_size = extract_cell("Diagonal");
    let camera = extract_cell("Megapixels");

    if let Some(ref c) = chipset { raw_map.insert("chipset".to_string(), c.clone()); }
    if let Some(ref g) = gpu { raw_map.insert("gpu".to_string(), g.clone()); }
    if let Some(ref r) = ram { raw_map.insert("ram".to_string(), r.clone()); }
    if let Some(ref s) = storage { raw_map.insert("storage".to_string(), s.clone()); }
    if let Some(ref b) = battery { raw_map.insert("battery".to_string(), b.clone()); }
    if let Some(ref d) = display { raw_map.insert("display".to_string(), d.clone()); }

    // Extract device image if present
    let img_re = Regex::new(r#"<img[^>]+src=["'](https://nanoreview\.net/content/phone/[^"']+)["']"#).unwrap();
    let image_url = img_re.captures(&html).and_then(|c| c.get(1)).map(|m| m.as_str().to_string());

    if chipset.is_none() && display.is_none() {
        return Err(AppError::NotFound("No specs parsed from NanoReview".to_string()));
    }

    Ok(NanoReviewSpecResult {
        source_url: url,
        chipset,
        cpu,
        gpu,
        display_type: display,
        display_size,
        battery_spec: battery,
        charging_spec: charging,
        camera_spec: camera,
        memory_spec: ram.map(|r| format!("RAM: {}, Storage: {}", r, storage.unwrap_or_default())),
        os_original: os,
        image_url,
        raw_map,
    })
}
