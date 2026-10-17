use crate::telemetry::NativeHardwareTelemetryRequest;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct PairDeviceRequest {
    pub pairing_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_model: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct PairDeviceResponse {
    pub device_id: Uuid,
    pub device_token: String,
    pub auth_token: Option<String>,
    pub user_id: Uuid,
    pub device_name: String,
    pub device_type: String,
    pub marketing_name: Option<String>,
    pub chipset: Option<String>,
}

pub struct ApiClient {
    client: Client,
    pub base_url: String,
}

/// Menormalkan input URL agar selalu memiliki skema http:// atau https://
pub fn normalize_server_url(server_url: &str) -> String {
    let trimmed = server_url.trim();
    if trimmed.is_empty() {
        return "http://127.0.0.1:8080".to_string();
    }

    let with_scheme = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else if let Some(stripped) = trimmed.strip_prefix("ws://") {
        format!("http://{}", stripped)
    } else if let Some(stripped) = trimmed.strip_prefix("wss://") {
        format!("https://{}", stripped)
    } else {
        format!("http://{}", trimmed)
    };

    with_scheme.trim_end_matches('/').to_string()
}

/// Menurunkan URL WebSocket yang tepat dari HTTP Server URL
pub fn derive_ws_url(server_url: &str) -> String {
    let http_normalized = normalize_server_url(server_url);
    let ws_base = if let Some(stripped) = http_normalized.strip_prefix("https://") {
        format!("wss://{}", stripped)
    } else if let Some(stripped) = http_normalized.strip_prefix("http://") {
        format!("ws://{}", stripped)
    } else {
        format!("ws://{}", http_normalized)
    };

    // Bersihkan path yang mungkin ada, lalu arahkan ke /ws
    let base = ws_base.trim_end_matches('/');
    if base.ends_with("/ws") {
        base.to_string()
    } else if let Some(idx) = base.find("/api") {
        format!("{}/ws", &base[..idx])
    } else {
        format!("{}/ws", base)
    }
}

impl ApiClient {
    pub fn new(server_url: &str) -> Self {
        let base_url = normalize_server_url(server_url);
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        Self { client, base_url }
    }

    pub async fn check_health(&self) -> Result<bool, String> {
        // Coba endpoint /api/health (format rute standar Kusuri backend)
        let primary_url = format!("{}/api/health", self.base_url);
        if let Ok(res) = self.client.get(&primary_url).send().await {
            if res.status().is_success() {
                return Ok(true);
            }
        }

        // Fallback coba /health
        let fallback_url = format!("{}/health", self.base_url);
        let res = self
            .client
            .get(&fallback_url)
            .send()
            .await
            .map_err(|e| format!("Koneksi ke server gagal ({}): {}", self.base_url, e))?;

        Ok(res.status().is_success())
    }

    pub async fn pair_device(
        &self,
        pairing_code: &str,
        device_name: Option<String>,
        device_type: Option<String>,
        raw_model: Option<String>,
    ) -> Result<PairDeviceResponse, String> {
        let payload = PairDeviceRequest {
            pairing_code: pairing_code.to_string(),
            device_name,
            device_type,
            raw_model,
        };

        // Coba endpoint utama /api/devices/pair
        let primary_url = format!("{}/api/devices/pair", self.base_url);
        info!("Mencoba request pairing ke {}", primary_url);

        let mut response = self
            .client
            .post(&primary_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Koneksi ke backend gagal: {}", e))?;

        // Jika rute /api/devices/pair mengembalikan 404, coba fallback ke /api/v1/devices/pair
        if response.status() == StatusCode::NOT_FOUND {
            let fallback_url = format!("{}/api/v1/devices/pair", self.base_url);
            warn!("Rute {} tidak ditemukan (404), mencoba {}", primary_url, fallback_url);
            response = self
                .client
                .post(&fallback_url)
                .json(&payload)
                .send()
                .await
                .map_err(|e| format!("Koneksi ke backend gagal: {}", e))?;
        }

        if !response.status().is_success() {
            let status = response.status();
            let err_body = response.text().await.unwrap_or_default();
            error!("Pairing gagal [{}]: {}", status, err_body);
            return Err(format!("Pairing gagal [{}]: {}", status, err_body));
        }

        let result = response
            .json::<PairDeviceResponse>()
            .await
            .map_err(|e| format!("Gagal mem-parsing balasan pairing dari server: {}", e))?;

        Ok(result)
    }

    pub async fn send_hardware_telemetry(
        &self,
        device_id: Uuid,
        auth_token: &str,
        telemetry: &NativeHardwareTelemetryRequest,
    ) -> Result<(), String> {
        let primary_url = format!("{}/api/devices/{}/telemetry", self.base_url, device_id);

        let mut response = self
            .client
            .post(&primary_url)
            .bearer_auth(auth_token)
            .json(telemetry)
            .send()
            .await
            .map_err(|e| format!("HTTP Telemetry sync gagal: {}", e))?;

        if response.status() == StatusCode::NOT_FOUND {
            let fallback_url = format!("{}/api/v1/devices/{}/telemetry", self.base_url, device_id);
            response = self
                .client
                .post(&fallback_url)
                .bearer_auth(auth_token)
                .json(telemetry)
                .send()
                .await
                .map_err(|e| format!("HTTP Telemetry sync gagal: {}", e))?;
        }

        if response.status().is_success() {
            debug!("HTTP Telemetry sync berhasil");
            Ok(())
        } else {
            let status = response.status();
            let err_text = response.text().await.unwrap_or_default();
            Err(format!("HTTP Telemetry ditolak server [{}]: {}", status, err_text))
        }
    }

    #[allow(dead_code)]
    pub async fn ingest_notification(
        &self,
        token: &str,
        package_or_app_name: &str,
        app_title: Option<&str>,
        title: &str,
        content: &str,
        subtext: Option<&str>,
    ) -> Result<(), String> {
        let url = format!("{}/api/notifications", self.base_url);
        let payload = serde_json::json!({
            "package_or_app_name": package_or_app_name,
            "app_title": app_title,
            "title": title,
            "content": content,
            "subtext": subtext,
        });

        let response = self
            .client
            .post(&url)
            .bearer_auth(token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Gagal mengirim notifikasi: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let err = response.text().await.unwrap_or_default();
            Err(format!("Server menolak notifikasi [{}]: {}", status, err))
        }
    }
}

