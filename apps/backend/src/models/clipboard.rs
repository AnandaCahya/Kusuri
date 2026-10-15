use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ClipboardItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub source_device_id: Option<Uuid>,
    pub content: String,
    pub content_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateClipboardRequest {
    pub content: String,
    pub content_type: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ClipboardResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub source_device_id: Option<Uuid>,
    pub source_device_name: Option<String>,
    pub content: String,
    pub content_type: String,
    pub created_at: DateTime<Utc>,
}
