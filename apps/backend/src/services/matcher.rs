use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct CandidateDevice {
    pub title: String,
    pub url_path: String,
    pub image_url: Option<String>,
    pub score: f64,
}

/// Normalizes strings for matching: lowercase, handles '+' -> 'plus', handles plural typos (e.g. 'hots' -> 'hot')
pub fn normalize_for_matching(text: &str) -> String {
    let lower = text.to_lowercase();
    let replaced_plus = lower.replace('+', " plus ");
    let mut clean = String::new();
    for ch in replaced_plus.chars() {
        if ch.is_alphanumeric() || ch.is_whitespace() {
            clean.push(ch);
        } else {
            clean.push(' ');
        }
    }

    clean
        .split_whitespace()
        .map(|w| {
            // Normalize known plural/typo variants:
            if w == "hots" {
                "hot"
            } else if w == "phones" {
                "phone"
            } else if w.len() > 4 && w.ends_with('s') && !w.ends_with("ss") {
                &w[..w.len() - 1]
            } else {
                w
            }
        })
        .collect::<Vec<&str>>()
        .join(" ")
}

#[derive(Debug, Default, Clone)]
pub struct ClassifiedTokens {
    pub brand: Option<String>,
    pub numbers: Vec<String>,
    pub year: Option<u32>,
    pub tier_keywords: HashSet<String>,
    pub network_tags: HashSet<String>,
    pub general_tokens: HashSet<String>,
}

pub fn classify_tokens(text: &str, brand_hint: Option<&str>) -> ClassifiedTokens {
    let normalized = normalize_for_matching(text);
    let mut classified = ClassifiedTokens::default();

    let known_brands = [
        "infinix", "samsung", "apple", "xiaomi", "redmi", "poco", "oppo", "vivo",
        "realme", "motorola", "google", "oneplus", "sony", "huawei", "honor",
        "asus", "zte", "tecno", "itel", "lenovo", "nokia", "nothing",
    ];

    let brand_hint_lower = brand_hint.map(|b| b.trim().to_lowercase()).unwrap_or_default();

    for token in normalized.split_whitespace() {
        // Stop words
        if ["the", "a", "an", "and", "edition", "version", "series", "smartphone", "mobile", "phone"].contains(&token) {
            continue;
        }

        // Extract any contiguous numbers (e.g. 50, 24 from s24, 6881 from x6881, 8045 from zte 8045, 2020 from a5 2020)
        let mut curr_num = String::new();
        for ch in token.chars() {
            if ch.is_ascii_digit() {
                curr_num.push(ch);
            } else if !curr_num.is_empty() {
                if !classified.numbers.contains(&curr_num) {
                    classified.numbers.push(curr_num.clone());
                }
                curr_num.clear();
            }
        }
        if !curr_num.is_empty() && !classified.numbers.contains(&curr_num) {
            classified.numbers.push(curr_num);
        }

        // Pure numbers
        if token.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        // Tier keywords
        if ["pro", "plus", "ultra", "max", "lite", "mini", "neo", "play", "prime", "se", "fe", "gt", "vita"].contains(&token) {
            classified.tier_keywords.insert(token.to_string());
            continue;
        }

        // Network tags
        if ["4g", "5g", "lte", "nfc", "3g", "global", "china", "india"].contains(&token) {
            classified.network_tags.insert(token.to_string());
            continue;
        }

        // Brand
        if known_brands.contains(&token) || (!brand_hint_lower.is_empty() && brand_hint_lower.contains(token)) {
            if classified.brand.is_none() {
                classified.brand = Some(token.to_string());
            }
            continue;
        }

        // General series tokens (e.g. hot, note, galaxy, blade, edge)
        classified.general_tokens.insert(token.to_string());
    }

    // Detect 4-digit release year (e.g. 2020 in Oppo A5 2020, 2016 in A57 2016)
    // and separate it from core model numbers:
    let mut year_found = None;
    classified.numbers.retain(|num_str| {
        if num_str.len() == 4 {
            if let Ok(y) = num_str.parse::<u32>() {
                if (1995..=2035).contains(&y) {
                    year_found = Some(y);
                    return false; // Remove year from general model numbers
                }
            }
        }
        true
    });
    classified.year = year_found;

    classified
}

/// Calculates a selective smart matching score between a target device query and a candidate name.
/// Accurately discriminates between "Infinix Hot 50 Pro 4G", "Infinix Hot 50 Pro+ 4G", "Oppo A5 (2020)" vs "Oppo A5 (AX5)".
pub fn calculate_smart_match_score(
    target: &str,
    candidate_title: &str,
    brand_hint: Option<&str>,
) -> f64 {
    let target_norm = normalize_for_matching(target);
    let cand_norm = normalize_for_matching(candidate_title);

    // Exact normalized match gets highest score
    if target_norm == cand_norm {
        return 200.0;
    }

    let target_tokens = classify_tokens(target, brand_hint);
    let cand_tokens = classify_tokens(candidate_title, brand_hint);

    let mut score = 0.0;

    // 0. Release Year Disambiguation (CRITICAL for homonyms like Oppo A5 (2020) vs Oppo A5 2018 / A57 2016)
    match (target_tokens.year, cand_tokens.year) {
        (Some(ty), Some(cy)) => {
            if ty == cy {
                score += 50.0; // Exact release year match!
            } else {
                return -1000.0; // Conflicting release year (e.g. 2020 vs 2016) -> Instant Disqualification!
            }
        }
        (Some(_), None) => {
            // Target requested an explicit year (e.g. A5 2020), but candidate has no year
            score -= 40.0;
        }
        (None, Some(_)) => {
            score -= 10.0;
        }
        (None, None) => {}
    }

    // 1. Number tokens (CRITICAL: model number mismatch is an immediate disqualifier)
    if !target_tokens.numbers.is_empty() {
        let mut all_numbers_matched = true;
        for num in &target_tokens.numbers {
            if cand_tokens.numbers.contains(num) {
                score += 35.0;
            } else {
                all_numbers_matched = false;
            }
        }

        if !all_numbers_matched {
            // If candidate doesn't match the required model number, penalize heavily
            return -1000.0;
        }

        // If candidate has unexpected different numbers
        for cand_num in &cand_tokens.numbers {
            if !target_tokens.numbers.contains(cand_num) {
                score -= 20.0;
            }
        }
    }

    // 2. Tier keyword precision (Pro vs Pro+ vs Ultra vs Lite vs Base)
    let target_has_pro = target_tokens.tier_keywords.contains("pro");
    let target_has_plus = target_tokens.tier_keywords.contains("plus");
    let cand_has_pro = cand_tokens.tier_keywords.contains("pro");
    let cand_has_plus = cand_tokens.tier_keywords.contains("plus");

    if target_has_pro {
        if cand_has_pro {
            score += 30.0;
            if !target_has_plus && cand_has_plus {
                // Target is 'Pro', but candidate is 'Pro+' -> Penalize heavily!
                score -= 30.0;
            } else if target_has_plus && cand_has_plus {
                score += 30.0; // Both are 'Pro+'
            }
        } else {
            // Target is Pro, candidate is not Pro -> Penalize!
            score -= 35.0;
        }
    } else if target_has_plus {
        if cand_has_plus {
            score += 30.0;
        } else {
            score -= 35.0;
        }
    } else {
        // Target is the base model (no 'pro', no 'plus')
        if cand_has_pro || cand_has_plus {
            // Candidate is a higher tier Pro/Plus when target requested base model
            score -= 25.0;
        } else {
            score += 15.0;
        }
    }

    // Ultra check
    if target_tokens.tier_keywords.contains("ultra") {
        if cand_tokens.tier_keywords.contains("ultra") {
            score += 35.0;
        } else {
            score -= 40.0;
        }
    } else if cand_tokens.tier_keywords.contains("ultra") {
        score -= 30.0;
    }

    // 3. Network variant tolerance (e.g. 4G / 5G)
    // GSM Arena often tags phones as "Hot 50 Pro 4G", whereas the official Google CSV only says "Hot 50 Pro".
    let target_has_4g = target_tokens.network_tags.contains("4g");
    let target_has_5g = target_tokens.network_tags.contains("5g");
    let cand_has_4g = cand_tokens.network_tags.contains("4g");
    let cand_has_5g = cand_tokens.network_tags.contains("5g");

    if target_has_5g {
        if cand_has_5g {
            score += 25.0;
        } else if cand_has_4g {
            score -= 25.0;
        }
    } else if target_has_4g {
        if cand_has_4g {
            score += 25.0;
        } else if cand_has_5g {
            score -= 25.0;
        }
    } else {
        // Target did not specify 4G/5G: both are acceptable network variants
        if cand_has_4g || cand_has_5g {
            score += 10.0;
        }
    }

    // 4. Brand match
    if let Some(ref target_brand) = target_tokens.brand {
        if let Some(ref cand_brand) = cand_tokens.brand {
            if target_brand == cand_brand {
                score += 20.0;
            } else {
                return -1000.0; // Completely different brand
            }
        }
    }

    // 5. General series tokens (e.g. 'hot', 'note', 'galaxy')
    for token in &target_tokens.general_tokens {
        if cand_tokens.general_tokens.contains(token) {
            score += 20.0;
        } else {
            score -= 10.0;
        }
    }

    // 6. Jaccard word similarity on normalized strings
    let target_set: HashSet<&str> = target_norm.split_whitespace().collect();
    let cand_set: HashSet<&str> = cand_norm.split_whitespace().collect();
    let intersection = target_set.intersection(&cand_set).count();
    let union = target_set.union(&cand_set).count();
    if union > 0 {
        let jaccard = intersection as f64 / union as f64;
        score += jaccard * 15.0;
    }

    score
}

/// Generates prioritized search query variants for online catalogs.
/// Handles cases like:
/// - "Infinix Hots 50 Pro" -> ["infinix hot 50 pro", "infinix hot 50", "infinix hots 50 pro"]
/// - Obscure model codes like "ZTE 8045" -> ["zte 8045", "zte blade 8045", "zte nubia 8045", "8045"]
pub fn generate_search_queries(marketing_name: &str, retail_branding: Option<&str>) -> Vec<String> {
    let mut queries = Vec::new();
    let brand = retail_branding.unwrap_or("").trim().to_lowercase();
    let norm = normalize_for_matching(marketing_name);

    // 1. With brand prepended if not already in the query
    let base_with_brand = if !brand.is_empty() && !norm.contains(&brand) {
        format!("{} {}", brand, norm)
    } else {
        norm.clone()
    };
    queries.push(base_with_brand.clone());

    // 2. Clean query without brand prefix
    if !queries.contains(&norm) {
        queries.push(norm.clone());
    }

    // 3. Original raw marketing name
    let raw = marketing_name.trim().to_string();
    if !queries.contains(&raw) {
        queries.push(raw);
    }

    let tokens = classify_tokens(&base_with_brand, retail_branding);

    // 4. Broader fallback query: Brand + series + number (e.g. "infinix hot 50")
    if let (Some(b), Some(num)) = (tokens.brand.as_ref(), tokens.numbers.first()) {
        for general_token in &tokens.general_tokens {
            let broader = format!("{} {} {}", b, general_token, num);
            if !queries.contains(&broader) {
                queries.push(broader);
            }
        }
    }

    // 5. Obscure model number disambiguation:
    // If the input is just brand + number (e.g. "ZTE 8045" or "Haitech HPAD_IP8045")
    // Expand with the manufacturer's primary consumer smartphone families:
    if tokens.general_tokens.is_empty() {
        if let (Some(b), Some(num)) = (tokens.brand.as_ref(), tokens.numbers.first()) {
            let series_hints: &[&str] = match b.as_str() {
                "zte" => &["blade", "nubia", "axon", "blade v40"],
                "samsung" => &["galaxy"],
                "xiaomi" | "redmi" => &["redmi note", "redmi", "poco"],
                "infinix" => &["hot", "note", "smart"],
                "motorola" => &["moto", "edge"],
                "oppo" => &["reno", "a"],
                "vivo" => &["y", "v"],
                "realme" => &["narzo", "c"],
                _ => &[],
            };

            for series in series_hints {
                let hint_query = format!("{} {}", series, num);
                if !queries.contains(&hint_query) {
                    queries.push(hint_query);
                }
                let hint_with_brand = format!("{} {} {}", b, series, num);
                if !queries.contains(&hint_with_brand) {
                    queries.push(hint_with_brand);
                }
            }

            // Bare number query for spec/model lookup
            if !queries.contains(num) {
                queries.push(num.clone());
            }
        }
    }

    queries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_selective_infinix_hot_50_pro() {
        let target = "Infinix Hots 50 Pro";
        let brand = Some("Infinix");

        let score_pro_4g = calculate_smart_match_score(target, "Infinix Hot 50 Pro 4G", brand);
        let score_pro_plus = calculate_smart_match_score(target, "Infinix Hot 50 Pro+ 4G", brand);
        let score_hot_50_5g = calculate_smart_match_score(target, "Infinix Hot 50 5G", brand);
        let score_hot_50_base = calculate_smart_match_score(target, "Infinix Hot 50", brand);
        let score_hot_40_pro = calculate_smart_match_score(target, "Infinix Hot 40 Pro", brand);

        assert!(score_pro_4g > score_pro_plus, "Pro 4G should score higher than Pro+ 4G");
        assert!(score_pro_4g > score_hot_50_5g, "Pro 4G should score higher than base 5G");
        assert!(score_pro_4g > score_hot_50_base, "Pro 4G should score higher than base");
        assert!(score_hot_40_pro < 0.0, "Hot 40 Pro should be rejected");
    }

    #[test]
    fn test_smart_selective_samsung_galaxy_s24_ultra() {
        let target = "samsung s24 ultra";
        let brand = Some("Samsung");

        let score_ultra = calculate_smart_match_score(target, "Samsung Galaxy S24 Ultra", brand);
        let score_base = calculate_smart_match_score(target, "Samsung Galaxy S24", brand);
        let score_s23_ultra = calculate_smart_match_score(target, "Samsung Galaxy S23 Ultra", brand);

        assert!(score_ultra > score_base);
        assert!(score_s23_ultra < 0.0);
    }

    #[test]
    fn test_generate_search_queries_zte_8045() {
        let queries = generate_search_queries("ZTE 8045", Some("ZTE"));
        println!("Generated queries for ZTE 8045: {:?}", queries);
        assert!(queries.iter().any(|q| q.contains("blade") && q.contains("8045")));
        assert!(queries.contains(&"8045".to_string()));
    }

    #[test]
    fn test_smart_selective_oppo_a5_2020() {
        let target = "OPPO A5 2020";
        let brand = Some("Oppo");

        let score_a5_2020 = calculate_smart_match_score(target, "Oppo A5 (2020)", brand);
        let score_a5_legacy = calculate_smart_match_score(target, "Oppo A5 (AX5)", brand);
        let score_a5_pro = calculate_smart_match_score(target, "Oppo A5 Pro", brand);
        let score_a53 = calculate_smart_match_score(target, "Oppo A53", brand);
        let score_a57_2016 = calculate_smart_match_score(target, "Oppo A57 (2016)", brand);

        assert!(score_a5_2020 > score_a5_legacy, "A5 2020 must beat legacy A5");
        assert!(score_a5_2020 > score_a5_pro, "A5 2020 must beat A5 Pro");
        assert!(score_a53 < 0.0, "A53 must be rejected due to number mismatch (53 vs 5)");
        assert!(score_a57_2016 < 0.0, "Conflicting year (2016 vs 2020) must be rejected");
    }
}
