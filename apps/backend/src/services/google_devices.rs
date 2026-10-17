use crate::error::AppError;
use sqlx::PgPool;
use tracing::info;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GoogleDeviceMatch {
    pub retail_branding: String,
    pub marketing_name: String,
    pub device: String,
    pub model: String,
}

pub fn decode_utf16le(bytes: &[u8]) -> String {
    let u16s: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    String::from_utf16_lossy(&u16s)
}

/// Download and sync the Google Play Supported Devices list into PostgreSQL
pub async fn sync_google_devices_catalog(pool: &PgPool, force: bool) -> Result<usize, AppError> {
    // Check if table already has data
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM google_supported_devices")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    if count > 0 && !force {
        info!("Google Play supported devices catalog already populated ({} records). Skipping sync.", count);
        return Ok(count as usize);
    }

    info!("Downloading Google Play supported devices list from storage.googleapis.com...");
    let url = "https://storage.googleapis.com/play_public/supported_devices.csv";
    
    let client = reqwest::Client::builder()
        .user_agent("KusuriBackend/1.0")
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to build HTTP client: {}", e)))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch supported devices: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::Internal(format!(
            "Failed to fetch supported devices: HTTP {}",
            response.status()
        )));
    }

    let raw_bytes = response
        .bytes()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read response bytes: {}", e)))?;

    info!("Decoding {} bytes of UTF-16LE CSV...", raw_bytes.len());
    let csv_text = decode_utf16le(&raw_bytes);

    let mut records = Vec::new();

    // Parse CSV lines
    for line in csv_text.lines().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // CSV columns: "Retail Branding","Marketing Name","Device","Model"
        // Simple CSV splitter respecting quotes
        let cols = parse_csv_line(trimmed);
        if cols.len() >= 3 {
            let retail_branding = cols.first().cloned().unwrap_or_default();
            let marketing_name = cols.get(1).cloned().unwrap_or_default();
            let device = cols.get(2).cloned().unwrap_or_default();
            let model = cols.get(3).cloned().unwrap_or_default();

            if !device.is_empty() && !marketing_name.is_empty() {
                records.push((retail_branding, marketing_name, device, model));
            }
        }
    }

    info!("Inserting {} device records into database...", records.len());

    let mut tx = pool.begin().await?;
    sqlx::query("TRUNCATE TABLE google_supported_devices").execute(&mut *tx).await?;

    // Batch insert using sqlx::QueryBuilder
    for chunk in records.chunks(1000) {
        let mut query_builder: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
            "INSERT INTO google_supported_devices (retail_branding, marketing_name, device, model) ",
        );

        query_builder.push_values(chunk, |mut b, (brand, marketing, dev, mdl)| {
            b.push_bind(brand)
                .push_bind(marketing)
                .push_bind(dev)
                .push_bind(mdl);
        });

        query_builder.build().execute(&mut *tx).await?;
    }

    tx.commit().await?;
    info!("Successfully synced {} Google Play supported devices.", records.len());

    Ok(records.len())
}

/// Helper function to parse a single CSV line with quote handling
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if in_quotes && chars.peek() == Some(&'"') {
                    // Escaped quote
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = !in_quotes;
                }
            }
            ',' if !in_quotes => {
                fields.push(current.trim().to_string());
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }
    fields.push(current.trim().to_string());
    fields
}

/// Find a device in the catalog by raw codename or model string
pub async fn lookup_device_by_codename(
    pool: &PgPool,
    raw_query: &str,
) -> Option<GoogleDeviceMatch> {
    let cleaned = raw_query.trim();
    if cleaned.is_empty() {
        return None;
    }

    // 1. Exact match on device codename, model, or marketing name
    let res = sqlx::query_as::<_, (String, String, String, String)>(
        r#"SELECT retail_branding, marketing_name, device, model 
           FROM google_supported_devices 
           WHERE LOWER(device) = LOWER($1) OR LOWER(model) = LOWER($1) OR LOWER(marketing_name) = LOWER($1)
           LIMIT 1"#,
    )
    .bind(cleaned)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    if let Some((b, m, d, mdl)) = res {
        return Some(GoogleDeviceMatch {
            retail_branding: b,
            marketing_name: m,
            device: d,
            model: mdl,
        });
    }

    // 2. Try stripping prefixes/suffixes (e.g. 'Infinix-X6881' -> 'X6881', 'ZTE 8045' -> '8045')
    let clean_delims = cleaned.replace('-', " ");
    if let Some((_, model_suffix)) = clean_delims.split_once(' ') {
        let suffix = model_suffix.trim();
        let res2 = sqlx::query_as::<_, (String, String, String, String)>(
            r#"SELECT retail_branding, marketing_name, device, model 
               FROM google_supported_devices 
               WHERE LOWER(device) = LOWER($1) OR LOWER(model) = LOWER($1) OR LOWER(marketing_name) = LOWER($1)
                  OR LOWER(device) LIKE LOWER($2) OR LOWER(model) LIKE LOWER($2)
               LIMIT 1"#,
        )
        .bind(suffix)
        .bind(format!("%{}%", suffix))
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        if let Some((b, m, d, mdl)) = res2 {
            return Some(GoogleDeviceMatch {
                retail_branding: b,
                marketing_name: m,
                device: d,
                model: mdl,
            });
        }
    }

    None
}
