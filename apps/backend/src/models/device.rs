use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Device {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub device_type: String,
    pub device_token: String,
    pub token_hash: String,
    pub is_online: bool,
    pub last_seen_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub raw_model: Option<String>,
    pub marketing_name: Option<String>,
    pub retail_branding: Option<String>,
    pub chipset: Option<String>,
    pub specs_id: Option<Uuid>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DevicePairingCode {
    pub id: Uuid,
    pub code: String,
    pub user_id: Uuid,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub attempts: i32,
    pub max_attempts: i32,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CreatePairingCodeRequest {
    #[serde(default)]
    pub device_name: Option<String>,
    #[serde(default)]
    pub device_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PairingCodeResponse {
    pub pairing_id: Uuid,
    pub code: String,
    pub expires_at: DateTime<Utc>,
    pub expires_in_seconds: i64,
    pub max_attempts: i32,
}

#[derive(Debug, Deserialize, Default)]
pub struct PairDeviceRequest {
    #[serde(alias = "code")]
    pub pairing_code: String,
    #[serde(default)]
    pub pairing_id: Option<Uuid>,
    #[serde(default, alias = "name")]
    pub device_name: Option<String>,
    #[serde(default, alias = "type")]
    pub device_type: Option<String>, // "android", "windows", "linux", etc.
    #[serde(default, alias = "model")]
    pub raw_model: Option<String>, // e.g. "Infinix-X6881"
}

#[derive(Debug, Serialize)]
pub struct PairDeviceResponse {
    pub device_id: Uuid,
    pub device_token: String,
    pub user_id: Uuid,
    pub device_name: String,
    pub device_type: String,
    pub marketing_name: Option<String>,
    pub chipset: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDeviceRequest {
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DeviceResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub device_type: String,
    pub is_online: bool,
    pub last_seen_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub raw_model: Option<String>,
    pub marketing_name: Option<String>,
    pub retail_branding: Option<String>,
    pub chipset: Option<String>,
}

impl From<Device> for DeviceResponse {
    fn from(device: Device) -> Self {
        Self {
            id: device.id,
            user_id: device.user_id,
            name: device.name,
            device_type: device.device_type,
            is_online: device.is_online,
            last_seen_at: device.last_seen_at,
            created_at: device.created_at,
            raw_model: device.raw_model,
            marketing_name: device.marketing_name,
            retail_branding: device.retail_branding,
            chipset: device.chipset,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryData {
    pub battery_percent: Option<f32>,
    pub is_charging: Option<bool>,
    pub os_details: Option<String>,
    pub memory_used_mb: Option<u64>,
    pub memory_total_mb: Option<u64>,
    pub ip_address: Option<String>,
}
