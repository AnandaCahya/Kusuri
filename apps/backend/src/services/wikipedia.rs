use crate::error::AppError;
use crate::services::matcher::{calculate_smart_match_score, generate_search_queries};
use regex::Regex;
use std::collections::HashMap;
use tracing::info;

#[allow(dead_code)]
#[derive(Default, Clone, Debug)]
pub struct WikipediaSpecResult {
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

/// Dynamic Wikipedia API and Infobox scraper
pub async fn fetch_from_wikipedia(
    marketing_name: &str,
    retail_branding: Option<&str>,
) -> Result<WikipediaSpecResult, AppError> {
    let client = reqwest::Client::builder()
        .user_agent("KusuriBot/1.0 (https://github.com/kusuri; kusuri-specs@anandacahya.com)")
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let search_queries = generate_search_queries(marketing_name, retail_branding);
    let mut best_wiki_title: Option<String> = None;
    let mut highest_score = -999.0;

    for query in search_queries.iter().take(2) {
        let search_url = format!(
            "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&format=json",
            query.replace(' ', "+")
        );

        let search_res: serde_json::Value = match client.get(&search_url).send().await {
            Ok(r) => match r.json().await {
                Ok(j) => j,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        if let Some(items) = search_res["query"]["search"].as_array() {
            for item in items {
                if let Some(title) = item["title"].as_str() {
                    let score = calculate_smart_match_score(marketing_name, title, retail_branding);
                    if score > highest_score {
                        highest_score = score;
                        best_wiki_title = Some(title.to_string());
                    }
                }
            }
        }

        if highest_score >= 35.0 {
            break;
        }
    }

    let top_title = match best_wiki_title {
        Some(title) if highest_score >= 20.0 => title,
        _ => return Err(AppError::NotFound("No matching Wikipedia device article found".to_string())),
    };

    info!(
        "Smart Selective Wikipedia matched '{}' (score: {:.1}) for target '{}'",
        top_title, highest_score, marketing_name
    );

    // Fetch REST Summary for description & thumbnail
    let summary_url = format!(
        "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
        top_title.replace(' ', "_")
    );

    let mut image_url: Option<String> = None;
    if let Ok(sum_res) = client.get(&summary_url).send().await {
        if let Ok(sum_json) = sum_res.json::<serde_json::Value>().await {
            image_url = sum_json["thumbnail"]["source"].as_str().map(|s| s.to_string())
                .or_else(|| sum_json["originalimage"]["source"].as_str().map(|s| s.to_string()));
        }
    }

    // Fetch Wikitext Infobox for detailed specs
    let wikitext_url = format!(
        "https://en.wikipedia.org/w/api.php?action=query&prop=revisions&rvprop=content&rvslots=main&titles={}&format=json",
        top_title.replace(' ', "_")
    );

    let wiki_json: serde_json::Value = client.get(&wikitext_url).send().await?.json().await?;
    let mut wikitext = String::new();
    if let Some(pages) = wiki_json["query"]["pages"].as_object() {
        for (_, page) in pages {
            if let Some(revs) = page["revisions"].as_array() {
                if let Some(first_rev) = revs.first() {
                    if let Some(text) = first_rev["slots"]["main"]["*"].as_str() {
                        wikitext = text.to_string();
                        break;
                    }
                }
            }
        }
    }

    let source_url = format!("https://en.wikipedia.org/wiki/{}", top_title.replace(' ', "_"));
    let mut result = WikipediaSpecResult {
        source_url,
        image_url,
        ..Default::default()
    };

    if !wikitext.is_empty() {
        let clean_wiki = |raw: &str| -> String {
            let without_brackets = raw.replace("[[", "").replace("]]", "");
            let without_refs = Regex::new(r#"<ref[^>]*>.*?</ref>"#).unwrap().replace_all(&without_brackets, "");
            let without_tags = Regex::new(r#"<[^>]+>"#).unwrap().replace_all(&without_refs, "");
            without_tags.trim().to_string()
        };

        for line in wikitext.lines() {
            let trimmed = line.trim();
            if let Some((k, v)) = trimmed.strip_prefix('|').and_then(|s| s.split_once('=')) {
                let key = k.trim().to_lowercase();
                let val = clean_wiki(v);
                if val.is_empty() || val.starts_with("<!--") {
                    continue;
                }
                match key.as_str() {
                    "soc" | "chipset" => {
                        result.raw_map.insert("chipset".to_string(), val.clone());
                        result.chipset = Some(val);
                    }
                    "cpu" => {
                        result.raw_map.insert("cpu".to_string(), val.clone());
                        result.cpu = Some(val);
                    }
                    "gpu" => {
                        result.raw_map.insert("gpu".to_string(), val.clone());
                        result.gpu = Some(val);
                    }
                    "display" | "screen" => {
                        result.raw_map.insert("display".to_string(), val.clone());
                        result.display_type = Some(val);
                    }
                    "battery" => {
                        result.raw_map.insert("battery".to_string(), val.clone());
                        result.battery_spec = Some(val);
                    }
                    "charging" => {
                        result.raw_map.insert("charging".to_string(), val.clone());
                        result.charging_spec = Some(val);
                    }
                    "ext_camera" | "camera" => {
                        result.raw_map.insert("camera".to_string(), val.clone());
                        result.camera_spec = Some(val);
                    }
                    "memory" | "ram" => {
                        result.raw_map.insert("memory".to_string(), val.clone());
                        result.memory_spec = Some(val);
                    }
                    "os" | "operating_system" => {
                        result.raw_map.insert("os".to_string(), val.clone());
                        result.os_original = Some(val);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(result)
}
