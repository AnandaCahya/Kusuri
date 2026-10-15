use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RemoteCommand {
    pub id: Uuid,
    pub user_id: Uuid,
    pub source_device_id: Option<Uuid>,
    pub target_device_id: Uuid,
    pub command: String,
    pub payload: Option<serde_json::Value>,
    pub status: String,
    pub response: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SendCommandRequest {
    pub target_device_id: Uuid,
    pub command: String,
    pub payload: Option<serde_json::Value>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct CommandResultRequest {
    pub command_id: Uuid,
    pub status: String, // "completed" or "failed"
    pub response: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct RemoteCommandResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub source_device_id: Option<Uuid>,
    pub source_device_name: Option<String>,
    pub target_device_id: Uuid,
    pub target_device_name: Option<String>,
    pub command: String,
    pub payload: Option<serde_json::Value>,
    pub status: String,
    pub response: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
