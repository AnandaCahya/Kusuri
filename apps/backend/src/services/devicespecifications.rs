use crate::error::AppError;
use regex::Regex;
use std::collections::HashMap;
use tracing::{info, warn};

#[allow(dead_code)]
#[derive(Default, Clone, Debug)]
pub struct DeviceSpecificationsResult {
    pub source_url: String,
    pub chipset: Option<String>,
    pub cpu: Option<String>,
    pub gpu: Option<String>,
    pub display_type: Option<String>,
    pub display_size: Option<String>,
    pub display_resolution: Option<String>,
    pub battery_spec: Option<String>,
    pub charging_spec: Option<String>,
    pub camera_spec: Option<String>,
    pub memory_spec: Option<String>,
    pub os_original: Option<String>,
    pub image_url: Option<String>,
    pub raw_map: HashMap<String, String>,
}

/// Scrapes device specifications dynamically from devicespecifications.com
pub async fn scrape_devicespecifications(
    marketing_name: &str,
    retail_branding: Option<&str>,
) -> Result<DeviceSpecificationsResult, AppError> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(6))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to build HTTP client: {}", e)))?;

    let brand = retail_branding.unwrap_or("").trim();
    let query = if !brand.is_empty() && !marketing_name.to_lowercase().contains(&brand.to_lowercase()) {
        format!("{} {}", brand, marketing_name)
    } else {
        marketing_name.to_string()
    };

    let search_url = format!(
        "https://www.devicespecifications.com/en/search?search={}",
        query.replace(' ', "+")
    );
    info!("Querying DeviceSpecifications at {}", search_url);

    let resp = match client.get(&search_url).send().await {
        Ok(r) => r,
        Err(e) => return Err(AppError::Internal(e.to_string())),
    };

    let html = resp.text().await.map_err(|e| AppError::Internal(e.to_string()))?;
    if html.contains("Verify you are human") || html.contains("cf-turnstile") {
        warn!("DeviceSpecifications bot challenge encountered");
        return Err(AppError::NotFound("DeviceSpecifications challenged by bot verification".to_string()));
    }

    let extract_spec = |header: &str| -> Option<String> {
        let pattern = format!(r#"<td[^>]*header[^>]*>{}</td>\s*<td[^>]*>([\s\S]*?)</td>"#, regex::escape(header));
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

    let soc = extract_spec("SoC").or_else(|| extract_spec("Chipset"));
    let cpu = extract_spec("CPU");
    let gpu = extract_spec("GPU");
    let display = extract_spec("Display");
    let battery = extract_spec("Battery");
    let camera = extract_spec("Camera");
    let os = extract_spec("Operating system");

    if let Some(ref s) = soc { raw_map.insert("soc".to_string(), s.clone()); }
    if let Some(ref c) = cpu { raw_map.insert("cpu".to_string(), c.clone()); }
    if let Some(ref g) = gpu { raw_map.insert("gpu".to_string(), g.clone()); }
    if let Some(ref d) = display { raw_map.insert("display".to_string(), d.clone()); }
    if let Some(ref b) = battery { raw_map.insert("battery".to_string(), b.clone()); }

    if soc.is_none() && display.is_none() {
        return Err(AppError::NotFound("No specs parsed from DeviceSpecifications".to_string()));
    }

    Ok(DeviceSpecificationsResult {
        source_url: search_url,
        chipset: soc,
        cpu,
        gpu,
        display_type: display,
        display_size: None,
        display_resolution: None,
        battery_spec: battery,
        charging_spec: None,
        camera_spec: camera,
        memory_spec: None,
        os_original: os,
        image_url: None,
        raw_map,
    })
}
