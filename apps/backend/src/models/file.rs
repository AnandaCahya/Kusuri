use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SharedFile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub source_device_id: Option<Uuid>,
    pub target_device_id: Option<Uuid>,
    pub original_filename: String,
    pub stored_filename: String,
    pub file_size: i64,
    pub mime_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SharedFileResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub source_device_id: Option<Uuid>,
    pub source_device_name: Option<String>,
    pub target_device_id: Option<Uuid>,
    pub original_filename: String,
    pub file_size: i64,
    pub mime_type: String,
    pub download_url: String,
    pub created_at: DateTime<Utc>,
}
