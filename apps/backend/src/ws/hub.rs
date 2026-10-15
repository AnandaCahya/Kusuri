use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum WsEvent {
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "pong")]
    Pong,
    #[serde(rename = "device_connected")]
    DeviceConnected {
        device_id: Uuid,
        name: String,
        device_type: String,
    },
    #[serde(rename = "device_disconnected")]
    DeviceDisconnected { device_id: Uuid },
    #[serde(rename = "clipboard_sync")]
    ClipboardSync {
        content: String,
        content_type: Option<String>,
    },
    #[serde(rename = "clipboard_updated")]
    ClipboardUpdated {
        id: Uuid,
        content: String,
        content_type: String,
        source_device_id: Option<Uuid>,
        source_device_name: Option<String>,
        created_at: String,
    },
    #[serde(rename = "telemetry")]
    Telemetry(crate::models::device::TelemetryData),
    #[serde(rename = "hardware_telemetry")]
    HardwareTelemetry(crate::services::specs_enricher::NativeHardwareTelemetryRequest),
    #[serde(rename = "device_status_updated")]
    DeviceStatusUpdated {
        device_id: Uuid,
        data: crate::models::device::TelemetryData,
    },
    #[serde(rename = "device_specs_updated")]
    DeviceSpecsUpdated {
        device_id: Uuid,
        specs: crate::services::specs_enricher::EnrichedDeviceSpecsResponse,
    },
    #[serde(rename = "send_command")]
    SendCommand {
        target_device_id: Uuid,
        command: String,
        payload: Option<serde_json::Value>,
    },
    #[serde(rename = "execute_command")]
    ExecuteCommand {
        command_id: Uuid,
        command: String,
        payload: Option<serde_json::Value>,
        source_device_id: Option<Uuid>,
    },
    #[serde(rename = "command_result")]
    CommandResult {
        command_id: Uuid,
        status: String,
        response: Option<serde_json::Value>,
    },
    #[serde(rename = "command_result_received")]
    CommandResultReceived {
        command_id: Uuid,
        status: String,
        response: Option<serde_json::Value>,
        device_id: Uuid,
    },
    #[serde(rename = "file_available")]
    FileAvailable {
        file_id: Uuid,
        original_filename: String,
        file_size: i64,
        mime_type: String,
        download_url: String,
        source_device_id: Option<Uuid>,
    },
    #[serde(rename = "relay_message")]
    RelayMessage {
        target_device_id: Option<Uuid>,
        event: String,
        data: serde_json::Value,
    },
    #[serde(rename = "relay_event")]
    RelayEvent {
        source_device_id: Option<Uuid>,
        event: String,
        data: serde_json::Value,
    },
    #[serde(rename = "error")]
    Error { message: String },
}

pub type Tx = mpsc::UnboundedSender<Message>;

pub struct DeviceSession {
    #[allow(dead_code)]
    pub name: String,
    pub tx: Tx,
}

#[derive(Default)]
pub struct WsHub {
    // Map of user_id -> (device_id -> DeviceSession)
    users: RwLock<HashMap<Uuid, HashMap<Uuid, DeviceSession>>>,
}

impl WsHub {
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(&self, user_id: Uuid, device_id: Uuid, name: String, tx: Tx) {
        let mut users = self.users.write().await;
        let user_devices = users.entry(user_id).or_default();
        user_devices.insert(device_id, DeviceSession { name, tx });
        tracing::info!(
            "Device {} registered for user {}. Total active devices: {}",
            device_id,
            user_id,
            user_devices.len()
        );
    }

    pub async fn unregister(&self, user_id: Uuid, device_id: Uuid) {
        let mut users = self.users.write().await;
        if let Some(user_devices) = users.get_mut(&user_id) {
            user_devices.remove(&device_id);
            if user_devices.is_empty() {
                users.remove(&user_id);
            }
        }
        tracing::info!("Device {} unregistered for user {}", device_id, user_id);
    }

    pub async fn broadcast_to_user(
        &self,
        user_id: Uuid,
        event: &WsEvent,
        exclude_device_id: Option<Uuid>,
    ) {
        let json_str = match serde_json::to_string(event) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to serialize WsEvent: {:?}", e);
                return;
            }
        };

        let users = self.users.read().await;
        if let Some(user_devices) = users.get(&user_id) {
            for (dev_id, session) in user_devices.iter() {
                if let Some(exclude_id) = exclude_device_id {
                    if *dev_id == exclude_id {
                        continue;
                    }
                }
                let _ = session.tx.send(Message::Text(json_str.clone().into()));
            }
        }
    }

    pub async fn send_to_device(
        &self,
        user_id: Uuid,
        target_device_id: Uuid,
        event: &WsEvent,
    ) -> bool {
        let json_str = match serde_json::to_string(event) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to serialize WsEvent: {:?}", e);
                return false;
            }
        };

        let users = self.users.read().await;
        if let Some(user_devices) = users.get(&user_id) {
            if let Some(session) = user_devices.get(&target_device_id) {
                return session.tx.send(Message::Text(json_str.into())).is_ok();
            }
        }
        false
    }

    pub async fn is_device_online(&self, user_id: Uuid, device_id: Uuid) -> bool {
        let users = self.users.read().await;
        if let Some(user_devices) = users.get(&user_id) {
            user_devices.contains_key(&device_id)
        } else {
            false
        }
    }

    pub async fn get_online_devices_for_user(&self, user_id: Uuid) -> Vec<Uuid> {
        let users = self.users.read().await;
        if let Some(user_devices) = users.get(&user_id) {
            user_devices.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }
}
