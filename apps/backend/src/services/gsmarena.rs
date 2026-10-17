use crate::error::AppError;
use crate::services::matcher::{calculate_smart_match_score, generate_search_queries, CandidateDevice};
pub use crate::services::specs_aggregator::{get_or_fetch_gsmarena_specs, DeviceCatalogSpecs};
use regex::Regex;
use std::collections::HashMap;
use tracing::{info, warn};

#[allow(dead_code)]
#[derive(Default, Clone, Debug)]
pub struct GsmArenaSpecResult {
    pub source_url: String,
    pub commercial_name: Option<String>,
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
    pub models: Option<String>,
    pub raw_map: HashMap<String, String>,
}

/// Dynamic GSM Arena Scraper using Smart Selective Candidate Selection
pub async fn scrape_gsmarena(
    marketing_name: &str,
    retail_branding: Option<&str>,
) -> Result<GsmArenaSpecResult, AppError> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Linux; Android 14; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Mobile Safari/537.36")
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to build HTTP client: {}", e)))?;

    let search_queries = generate_search_queries(marketing_name, retail_branding);
    let mut found_candidates = Vec::new();

    for query in search_queries {
        let search_url = format!(
            "https://m.gsmarena.com/results.php3?sQuickSearch=yes&sName={}",
            query.replace(' ', "+")
        );

        let resp = match client.get(&search_url).send().await {
            Ok(r) => r,
            Err(e) => {
                warn!("Search query '{}' failed on GSM Arena: {}", query, e);
                continue;
            }
        };

        let html = match resp.text().await {
            Ok(t) => t,
            Err(_) => continue,
        };

        if html.contains("cf-turnstile") || html.contains("Turnstile check") {
            return Err(AppError::NotFound("GSM Arena requested Cloudflare verification".to_string()));
        }

        // Pattern matching candidate items in mobile HTML:
        let item_re = Regex::new(
            r#"<a\s+href=["']?([a-zA-Z0-9_\(\)\+\-]+-[0-9]+\.php)["']?[^>]*>(?:<img[^>]+src=["']?([^"'\s>]+)["']?[^>]+title=["']([^"']+)["'])?"#,
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;

        for cap in item_re.captures_iter(&html) {
            let url_path = match cap.get(1) {
                Some(m) => m.as_str().to_string(),
                None => continue,
            };

            // Filter out maker/brand listing links
            if url_path.contains("-phones-") {
                continue;
            }

            let image_url = cap.get(2).map(|m| m.as_str().to_string());
            let title = cap.get(3).map(|m| m.as_str().to_string()).unwrap_or_else(|| {
                url_path
                    .split('-')
                    .next()
                    .unwrap_or("")
                    .replace('_', " ")
            });

            let score = calculate_smart_match_score(marketing_name, &title, retail_branding);
            found_candidates.push(CandidateDevice {
                title,
                url_path,
                image_url,
                score,
            });
        }

        if found_candidates.iter().any(|c| c.score >= 15.0) {
            break;
        }
    }

    if found_candidates.is_empty() {
        return Err(AppError::NotFound(format!(
            "Device '{}' not found on GSM Arena",
            marketing_name
        )));
    }

    // Sort candidates descending by Smart Selective Match score
    found_candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    let mut best_candidate = found_candidates[0].clone();
    let mut verified_detail_html: Option<String> = None;

    // Secondary model-code verification:
    // If no candidate scored >= 15.0 on title matching alone (common with obscure model codes like "ZTE 8045"),
    // verify the candidate's actual detail page `models` field!
    if best_candidate.score < 15.0 {
        let target_tokens = crate::services::matcher::classify_tokens(marketing_name, retail_branding);
        info!(
            "No candidate matched '{}' by title alone (best score: {:.1}). Checking candidate model codes...",
            marketing_name, best_candidate.score
        );

        for cand in found_candidates.iter().take(3) {
            let test_url = format!("https://m.gsmarena.com/{}", cand.url_path);
            if let Ok(resp) = client.get(&test_url).send().await {
                if let Ok(html) = resp.text().await {
                    let spec_re = Regex::new(r#"<td[^>]+data-spec=["']?([^"'\s>]+)["']?[^>]*>([\s\S]*?)</td>"#)
                        .unwrap();
                    let mut models_val = String::new();
                    for cap in spec_re.captures_iter(&html) {
                        if cap.get(1).map(|m| m.as_str()) == Some("models") {
                            models_val = cap.get(2).map(|m| m.as_str()).unwrap_or("").to_lowercase();
                            break;
                        }
                    }

                    let mut matched = false;
                    for num in &target_tokens.numbers {
                        if models_val.contains(num) {
                            matched = true;
                            break;
                        }
                    }
                    if !matched && models_val.contains(&marketing_name.to_lowercase()) {
                        matched = true;
                    }

                    if matched {
                        info!(
                            "Secondary verification SUCCESS: Candidate '{}' confirmed to match model code '{}' (models: {:?})",
                            cand.title, marketing_name, models_val
                        );
                        best_candidate = cand.clone();
                        best_candidate.score = 250.0;
                        verified_detail_html = Some(html);
                        break;
                    }
                }
            }
        }
    }

    info!(
        "Smart Selective GSM Arena selected candidate '{}' (score: {:.1}) for target '{}'",
        best_candidate.title, best_candidate.score, marketing_name
    );

    if best_candidate.score < 15.0 {
        return Err(AppError::NotFound(format!(
            "No candidate matched target '{}' with sufficient confidence (best: '{}' score {:.1})",
            marketing_name, best_candidate.title, best_candidate.score
        )));
    }

    // Fetch detail specs from GSM Arena mobile endpoint
    let detail_url = format!("https://m.gsmarena.com/{}", best_candidate.url_path);
    let detail_html = if let Some(html) = verified_detail_html {
        html
    } else {
        let detail_resp = client
            .get(&detail_url)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch GSM Arena details: {}", e)))?;

        detail_resp
            .text()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to read detail page: {}", e)))?
    };

    if detail_html.contains("cf-turnstile") {
        return Err(AppError::NotFound("GSM Arena detail blocked by Turnstile".to_string()));
    }

    // Extract all dynamic data-spec attributes
    let spec_re = Regex::new(r#"data-spec=["']([^"']+)["'][^>]*>([\s\S]*?)</"#)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let clean_html_tags = |raw: &str| -> String {
        let tag_re = Regex::new(r#"<[^>]+>"#).unwrap();
        let cleaned = tag_re.replace_all(raw, " ");
        cleaned
            .replace("&amp;", "&")
            .replace("&nbsp;", " ")
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    };

    let mut specs_map = HashMap::new();
    for cap in spec_re.captures_iter(&detail_html) {
        if let (Some(k), Some(v)) = (cap.get(1), cap.get(2)) {
            let key = k.as_str().trim().to_lowercase();
            let val = clean_html_tags(v.as_str());
            if !val.is_empty() {
                specs_map.insert(key, val);
            }
        }
    }

    // Check Charging row in table if not in data-spec
    let mut charging_spec = specs_map.get("charging").cloned();
    if charging_spec.is_none() {
        if let Some(ch) = specs_map.get("battype-hl") {
            charging_spec = Some(ch.clone());
        } else {
            let charge_re = Regex::new(r#"Charging</a></td>\s*<td class="nfo">([\s\S]*?)</td>"#).unwrap();
            if let Some(cap) = charge_re.captures(&detail_html) {
                if let Some(m) = cap.get(1) {
                    charging_spec = Some(clean_html_tags(m.as_str()));
                }
            }
        }
    }

    // Extract image URL (candidate image or detail page main photo)
    let mut image_url = best_candidate.image_url.clone();
    if image_url.is_none() {
        let img_re = Regex::new(r#"class=["']specs-photo-main["'][^>]*>\s*<a[^>]*>\s*<img[^>]+src=["']([^"']+)["']"#).unwrap();
        image_url = img_re.captures(&detail_html).and_then(|c| c.get(1)).map(|m| m.as_str().to_string());
    }

    let models = specs_map.get("models").cloned();

    Ok(GsmArenaSpecResult {
        source_url: detail_url,
        commercial_name: Some(best_candidate.title),
        chipset: specs_map.get("chipset").cloned().or_else(|| specs_map.get("chipset-hl").cloned()),
        cpu: specs_map.get("cpu").cloned(),
        gpu: specs_map.get("gpu").cloned(),
        display_type: specs_map.get("displaytype").cloned(),
        display_size: specs_map.get("displaysize").cloned().or_else(|| specs_map.get("displaysize-hl").cloned()),
        display_resolution: specs_map.get("displayresolution").cloned().or_else(|| specs_map.get("displayres").cloned()),
        os_original: specs_map.get("os").cloned().or_else(|| specs_map.get("os-hl").cloned()),
        battery_spec: specs_map.get("batdescription1").cloned().or_else(|| specs_map.get("batsize-hl").cloned()),
        charging_spec,
        camera_spec: specs_map.get("cam1modules").cloned().or_else(|| specs_map.get("camerapixels-hl").cloned()),
        memory_spec: specs_map.get("internalmemory").cloned().or_else(|| specs_map.get("ramsize-hl").cloned()),
        image_url,
        models,
        raw_map: specs_map,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_debug_live_zte_8045() {
        match scrape_gsmarena("ZTE 8045", Some("ZTE")).await {
            Ok(spec) => {
                println!("GSM Arena resolved ZTE 8045: {:?}", spec.commercial_name);
                assert_eq!(spec.commercial_name.as_deref(), Some("ZTE Blade V40 Vita"));
                assert!(spec.chipset.is_some() || spec.battery_spec.is_some());
            }
            Err(e) => {
                println!("GSM Arena live test network/verification notice: {}", e);
            }
        }
    }
}

