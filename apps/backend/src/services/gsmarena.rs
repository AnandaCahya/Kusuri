use crate::error::AppError;
use crate::services::matcher::{calculate_smart_match_score, generate_search_queries, CandidateDevice};
pub use crate::services::specs_aggregator::{get_or_fetch_gsmarena_specs, DeviceCatalogSpecs};
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use base64::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use tracing::{info, warn};

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

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

/// Decrypts AES-128-CBC obfuscated search results from GSM Arena mobile resl.php3
pub fn decrypt_gsmarena_search_payload(html: &str) -> Option<String> {
    let key_re = Regex::new(r#"const\s+KEY\s*=\s*["']([^"']+)["']"#).ok()?;
    let iv_re = Regex::new(r#"const\s+IV\s*=\s*["']([^"']+)["']"#).ok()?;
    let data_re = Regex::new(r#"const\s+DATA\s*=\s*["']([^"']+)["']"#).ok()?;

    let key_b64 = key_re.captures(html)?.get(1)?.as_str();
    let iv_b64 = iv_re.captures(html)?.get(1)?.as_str();
    let data_b64 = data_re.captures(html)?.get(1)?.as_str();

    let key_bytes = BASE64_STANDARD.decode(key_b64).ok()?;
    let iv_bytes = BASE64_STANDARD.decode(iv_b64).ok()?;
    let mut data_bytes = BASE64_STANDARD.decode(data_b64).ok()?;

    if key_bytes.len() != 16 || iv_bytes.len() != 16 {
        return None;
    }

    let cipher = Aes128CbcDec::new_from_slices(&key_bytes, &iv_bytes).ok()?;
    let decrypted = cipher.decrypt_padded_mut::<Pkcs7>(&mut data_bytes).ok()?;
    String::from_utf8(decrypted.to_vec()).ok()
}

/// Helper function to parse candidate device anchors from raw or decrypted HTML
fn parse_candidates_from_html(
    html: &str,
    target_name: &str,
    retail_branding: Option<&str>,
    out_candidates: &mut Vec<CandidateDevice>,
) {
    let anchor_re = match Regex::new(
        r#"<a\s+href=["']?([a-zA-Z0-9_\(\)\+\-]+-[0-9]+\.php)["']?[^>]*>([\s\S]*?)</a>"#,
    ) {
        Ok(r) => r,
        Err(_) => return,
    };

    let title_strong_re = Regex::new(r#"<strong>([^<]+)</strong>"#).unwrap();
    let title_attr_re = Regex::new(r#"title=["']([^"']+)["']"#).unwrap();
    let img_src_re = Regex::new(r#"<img[^>]+src=["']?([^"'\s>]+)"#).unwrap();

    for cap in anchor_re.captures_iter(html) {
        let url_path = match cap.get(1) {
            Some(m) => m.as_str().to_string(),
            None => continue,
        };
        if url_path.contains("-phones-") || out_candidates.iter().any(|c| c.url_path == url_path) {
            continue;
        }

        let inner_html = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let image_url = img_src_re.captures(inner_html).and_then(|c| c.get(1)).map(|m| m.as_str().to_string());

        let title = title_strong_re
            .captures(inner_html)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim().to_string())
            .or_else(|| {
                title_attr_re
                    .captures(inner_html)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().trim().to_string())
            })
            .unwrap_or_else(|| {
                url_path
                    .split('-')
                    .next()
                    .unwrap_or("")
                    .replace('_', " ")
            });

        let score = calculate_smart_match_score(target_name, &title, retail_branding);
        out_candidates.push(CandidateDevice {
            title,
            url_path,
            image_url,
            score,
        });
    }
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

    // 1. Pass 1: Quick Search by marketing name
    for query in &search_queries {
        let search_url = format!(
            "https://m.gsmarena.com/results.php3?sQuickSearch=yes&sName={}",
            query.replace(' ', "+")
        );

        let resp = match client.get(&search_url).send().await {
            Ok(r) => r,
            Err(e) => {
                warn!("Quick search query '{}' failed on GSM Arena: {}", query, e);
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

        // Check if response contains encrypted script payload
        if let Some(decrypted) = decrypt_gsmarena_search_payload(&html) {
            parse_candidates_from_html(&decrypted, marketing_name, retail_branding, &mut found_candidates);
        } else {
            parse_candidates_from_html(&html, marketing_name, retail_branding, &mut found_candidates);
        }

        if found_candidates.iter().any(|c| c.score >= 15.0) {
            break;
        }
    }

    // 2. Pass 2: Fallback to full model-code & specs search (resl.php3) if no high-confidence match found
    if found_candidates.is_empty() || !found_candidates.iter().any(|c| c.score >= 15.0) {
        info!(
            "No high-confidence candidate for '{}' in quick search. Querying GSM Arena full-text search (resl.php3)...",
            marketing_name
        );
        for query in &search_queries {
            let search_url = format!(
                "https://m.gsmarena.com/resl.php3?sSearch={}",
                query.replace(' ', "+")
            );

            let resp = match client.get(&search_url).send().await {
                Ok(r) => r,
                Err(e) => {
                    warn!("Full search query '{}' failed on GSM Arena: {}", query, e);
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

            if let Some(decrypted) = decrypt_gsmarena_search_payload(&html) {
                parse_candidates_from_html(&decrypted, marketing_name, retail_branding, &mut found_candidates);
            } else {
                parse_candidates_from_html(&html, marketing_name, retail_branding, &mut found_candidates);
            }

            if found_candidates.iter().any(|c| c.score >= 15.0) {
                break;
            }
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

        for cand in found_candidates.iter().take(4) {
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

    // Extract official commercial marketing name from detail page h1 if available
    let h1_re = Regex::new(r#"<h1[^>]*class=["']specs-phone-name-title["'][^>]*>([\s\S]*?)</h1>"#).unwrap();
    let detail_commercial_name = h1_re
        .captures(&detail_html)
        .and_then(|c| c.get(1))
        .map(|m| clean_html_tags(m.as_str()))
        .filter(|s| !s.is_empty())
        .unwrap_or(best_candidate.title);

    Ok(GsmArenaSpecResult {
        source_url: detail_url,
        commercial_name: Some(detail_commercial_name),
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

    #[test]
    fn test_decrypt_gsmarena_search_payload() {
        let sample_html = r#"
            <script>
                const KEY  = "TK/S96tWZDDOGQqOSdc9yQ==";
                const IV   = "KNAopee8Zuvy3FeyzIZMug==";
                const DATA = "K1RGG6HYSnZgzbhzyqgNBvSfi8CrreMGfGOLFOSokYbrnQTJeET1A279LtnImXYw3ZpvMFIj/dpOj3clkzJ1GpDI8yHppP//b4WKX7REogsBAxSgFZDPrcYBTBtdTHlNrtkk7CeojcQ04lxTYkWbiQdLNy1Wq7Ko6Zk2QI6nyNQX6h120bSfzllvMonOuWdEvYG+lrhpQ9SsWXlTn1OJicjhaPIz/iWlzFX56b7ZAfmS6B/FQhH1ihyLYIXIaW1xgbeCatsRMjl3sa0UsU11zaheu8SbdAAzMGoX6aF7jEtdoxX8W1JTXQ9JMAkKcnE1dGe4cVld8kMz8PZl060SjmL8BSV3tLbDPDtZEBqLiISCXHpjwQKZhzxGyZ+lgcLfaHFZ8s+pSAUyRPZV2xGKD8pRykvTZu3Fi+OlxXy21qcTgnz3n9GcVP8vXE5ozNlsU5467v9ho6SFQ4eWWk6NLryVuxeC5sp+0nrM8bYW4ovSM36OgEP+m/LaYw745IK+3UVh1izrgPhxHJ1QPz0KFs+xulGoRXshaOxKtZk5yc4YAMHBYfZS6DBoBi4v6vN3gxML7AeXWlAuDogDfHTvtQz8hk8dti+DpJqQC0n01C50CYQyROtiDvRwqJtYfXFw1sj21JOkvyJ1k67NKRx0qAs7TidZ2vksJAPnL+ajkcZtG42gC2ClpIPAwz1OAfehqtHSmF8xxZQOrM2ib6UOL/tV/1hYLUJMBhx/7G+omkgqXoOEmLY9smuDn29qrZ67Br3c9i0Qc1yodcIGX1fRjN2NmpKqOJnhzd7FHeE3ZRT4M/QIL4nu64nXQKKJ/GSERpWVmxbX/pvga07LRUjHRGYWyRoekhZUyyxDt5DjA3U28E9bjkAyVlwJtsM3dyb1UVg31u7EhNeAmm8UdrgknjYtWRnstv3JKzvNgIxVE/saV9bghLbiotm5QmogVz1DMJQNbUq2o/vMldV66vfvSpz3ggZtuC61vNxsfS/hL4o9gZv9gA5JohrLO2sieFUcm6M4YNg0gouDkMsHpRRafgyB+GcK5PKDkSiU7o940PaRAiQbPERihZreOuXjHwlEhEoS97dggoQspWBmIMeZsYtYLmU/iURZqyoYDH3X2ea5QsvBcDpdT633+i/25Lh5KVl17jb+FUiLJ7bG/CGJjCuJymWfcfREWb17m86rNys=";
            </script>
        "#;
        let decrypted = decrypt_gsmarena_search_payload(sample_html).expect("Decryption must succeed");
        assert!(decrypted.contains("ZTE Blade V40 Vita"));
        assert!(decrypted.contains("zte_blade_v40_vita-11579.php"));

        let mut candidates = Vec::new();
        parse_candidates_from_html(&decrypted, "ZTE 8045", Some("ZTE"), &mut candidates);
        assert!(!candidates.is_empty());
        assert_eq!(candidates[0].title, "ZTE Blade V40 Vita");
        assert_eq!(candidates[0].url_path, "zte_blade_v40_vita-11579.php");
    }

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

