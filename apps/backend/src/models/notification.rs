use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DeviceNotification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_id: Option<Uuid>,
    pub package_or_app_name: String,
    pub app_title: Option<String>,
    pub title: String,
    pub content: String,
    pub subtext: Option<String>,
    pub posted_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestNotificationRequest {
    pub device_id: Option<Uuid>,
    pub package_or_app_name: String,
    pub app_title: Option<String>,
    pub title: String,
    pub content: String,
    pub subtext: Option<String>,
    pub posted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_id: Option<Uuid>,
    pub device_name: Option<String>,
    pub package_or_app_name: String,
    pub app_title: Option<String>,
    pub title: String,
    pub content: String,
    pub subtext: Option<String>,
    pub posted_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NotificationQuery {
    pub device_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub search: Option<String>,
}
