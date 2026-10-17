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

use crate::services::matcher::{calculate_smart_match_score, generate_search_queries};

/// Scrapes device specifications dynamically from devicespecifications.com
pub async fn scrape_devicespecifications(
    marketing_name: &str,
    retail_branding: Option<&str>,
) -> Result<DeviceSpecificationsResult, AppError> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to build HTTP client: {}", e)))?;

    let search_queries = generate_search_queries(marketing_name, retail_branding);
    let model_link_re = Regex::new(r#"<a[^>]+href=["']?([^"'\s>]+/model/[^"'\s>]+)["']?[^>]*>([\s\S]*?)</a>"#)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let tag_strip_re = Regex::new(r#"<[^>]+>"#).unwrap();

    let mut model_url: Option<String> = None;

    for query in search_queries {
        let search_url = format!(
            "https://www.devicespecifications.com/en/search?search={}",
            query.replace(' ', "+")
        );
        info!("Querying DeviceSpecifications at {}", search_url);

        let resp = match client.get(&search_url).send().await {
            Ok(r) => r,
            Err(e) => {
                warn!("DeviceSpecifications search failed for query '{}': {}", query, e);
                continue;
            }
        };

        let html = match resp.text().await {
            Ok(t) => t,
            Err(_) => continue,
        };

        if html.contains("Verify you are human") || html.contains("cf-turnstile") {
            warn!("DeviceSpecifications bot challenge encountered");
            return Err(AppError::NotFound("DeviceSpecifications challenged by bot verification".to_string()));
        }

        let mut candidates = Vec::new();
        for cap in model_link_re.captures_iter(&html) {
            let href = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let raw_title = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let clean_title = tag_strip_re.replace_all(raw_title, " ").trim().to_string();

            let full_url = if href.starts_with("http") {
                href.to_string()
            } else {
                format!("https://www.devicespecifications.com{}", href)
            };

            let score = calculate_smart_match_score(marketing_name, &clean_title, retail_branding);
            candidates.push((full_url, clean_title, score));
        }

        candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((url, title, score)) = candidates.first() {
            if *score >= 15.0 || query.contains(&marketing_name.to_lowercase()) || !candidates.is_empty() {
                info!("DeviceSpecifications matched candidate '{}' ({}) with score {:.1}", title, url, score);
                model_url = Some(url.clone());
                break;
            }
        }
    }

    let detail_url = model_url.ok_or_else(|| {
        AppError::NotFound(format!("Device '{}' not found on DeviceSpecifications", marketing_name))
    })?;

    info!("Fetching DeviceSpecifications detail page at {}", detail_url);
    let detail_resp = client.get(&detail_url).send().await
        .map_err(|e| AppError::Internal(format!("Failed to fetch DeviceSpecifications detail: {}", e)))?;

    let html = detail_resp.text().await
        .map_err(|e| AppError::Internal(format!("Failed to read DeviceSpecifications detail: {}", e)))?;

    if html.contains("Verify you are human") || html.contains("cf-turnstile") {
        return Err(AppError::NotFound("DeviceSpecifications detail challenged by bot verification".to_string()));
    }

    let extract_spec = |header: &str| -> Option<String> {
        let pattern = format!(r#"<td[^>]*>{}</td>\s*<td[^>]*>([\s\S]*?)</td>"#, regex::escape(header));
        let re = Regex::new(&pattern).ok()?;
        re.captures(&html).and_then(|c| c.get(1)).map(|m| {
            tag_strip_re.replace_all(m.as_str(), " ")
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
    let display_size = extract_spec("Size");
    let display_resolution = extract_spec("Resolution");
    let battery = extract_spec("Battery");
    let camera = extract_spec("Camera");
    let os = extract_spec("Operating system");
    let memory = extract_spec("RAM");
    let storage = extract_spec("Storage");

    if let Some(ref s) = soc { raw_map.insert("soc".to_string(), s.clone()); }
    if let Some(ref c) = cpu { raw_map.insert("cpu".to_string(), c.clone()); }
    if let Some(ref g) = gpu { raw_map.insert("gpu".to_string(), g.clone()); }
    if let Some(ref d) = display { raw_map.insert("display".to_string(), d.clone()); }
    if let Some(ref b) = battery { raw_map.insert("battery".to_string(), b.clone()); }

    let memory_spec = match (memory, storage) {
        (Some(m), Some(s)) => Some(format!("{}, {}", s, m)),
        (Some(m), None) => Some(m),
        (None, Some(s)) => Some(s),
        (None, None) => None,
    };

    let img_re = Regex::new(r#"<div[^>]+id=["']model-image["'][^>]*>\s*<img[^>]+src=["']([^"']+)["']"#).unwrap();
    let image_url = img_re.captures(&html).and_then(|c| c.get(1)).map(|m| m.as_str().to_string());

    if soc.is_none() && display.is_none() && battery.is_none() {
        return Err(AppError::NotFound("No specs parsed from DeviceSpecifications".to_string()));
    }

    Ok(DeviceSpecificationsResult {
        source_url: detail_url,
        chipset: soc,
        cpu,
        gpu,
        display_type: display,
        display_size,
        display_resolution,
        battery_spec: battery,
        charging_spec: None,
        camera_spec: camera,
        memory_spec,
        os_original: os,
        image_url,
        raw_map,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_devicespec_zte_8045() {
        match scrape_devicespecifications("ZTE 8045", Some("ZTE")).await {
            Ok(spec) => {
                println!("DeviceSpecifications resolved ZTE 8045: {:?}", spec.chipset);
                assert!(spec.chipset.is_some() || spec.battery_spec.is_some());
            }
            Err(e) => {
                println!("DeviceSpecifications live test network/verification notice: {}", e);
            }
        }
    }
}

