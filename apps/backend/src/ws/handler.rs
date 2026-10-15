use crate::auth::{authenticate_token_against_db, AuthCaller};
use crate::error::AppError;
use crate::ws::hub::WsEvent;
use crate::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct WsAuthQuery {
    pub token: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsAuthQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let token = query
        .token
        .ok_or_else(|| AppError::Unauthorized("WebSocket requires ?token=<token>".to_string()))?;

    let caller = authenticate_token_against_db(&state.db, &state.config.jwt_secret, &token).await?;

    Ok(ws.on_upgrade(move |socket| handle_ws_socket(socket, state, caller)))
}

async fn handle_ws_socket(socket: WebSocket, state: Arc<AppState>, caller: AuthCaller) {
    let user_id = caller.user_id();
    let (device_id, device_name, device_type, is_device) = match &caller {
        AuthCaller::Device(dev) => (
            dev.device_id,
            dev.device_name.clone(),
            dev.device_type.clone(),
            true,
        ),
        AuthCaller::User(usr) => (
            Uuid::new_v4(), // Virtual session ID for web/dashboard client
            format!("{}-web", usr.username),
            "web".to_string(),
            false,
        ),
    };

    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    tracing::info!(
        "WebSocket CONNECTED: device='{}' ({}), device_id={}, user_id={}",
        device_name,
        device_type,
        device_id,
        user_id
    );

    // Register to hub
    state
        .hub
        .register(user_id, device_id, device_name.clone(), tx)
        .await;

    // If it's a registered physical device, update its status in DB and broadcast to other devices
    if is_device {
        let _ = sqlx::query("UPDATE devices SET is_online = true, last_seen_at = NOW() WHERE id = $1")
            .bind(device_id)
            .execute(&state.db)
            .await;

        state
            .hub
            .broadcast_to_user(
                user_id,
                &WsEvent::DeviceConnected {
                    device_id,
                    name: device_name.clone(),
                    device_type: device_type.clone(),
                },
                Some(device_id),
            )
            .await;
    }

    // Spawn task to forward hub messages to WebSocket client
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Read loop from client
    let hub_clone = state.hub.clone();
    let db_clone = state.db.clone();
    let dev_name = device_name.clone();

    while let Some(Ok(msg)) = ws_receiver.next().await {
        match msg {
            Message::Text(text) => {
                let event: Result<WsEvent, _> = serde_json::from_str(&text);
                match event {
                    Ok(WsEvent::Ping) => {
                        let _ = hub_clone
                            .send_to_device(user_id, device_id, &WsEvent::Pong)
                            .await;
                    }
                    Ok(WsEvent::ClipboardSync {
                        content,
                        content_type,
                    }) => {
                        let content_type = content_type.unwrap_or_else(|| "text/plain".to_string());
                        let item_id = Uuid::new_v4();
                        let src_id = if is_device { Some(device_id) } else { None };

                        let db_res = sqlx::query(
                            r#"INSERT INTO clipboard_items (id, user_id, source_device_id, content, content_type, created_at)
                               VALUES ($1, $2, $3, $4, $5, NOW())"#,
                        )
                        .bind(item_id)
                        .bind(user_id)
                        .bind(src_id)
                        .bind(&content)
                        .bind(&content_type)
                        .execute(&db_clone)
                        .await;

                        if let Err(e) = db_res {
                            tracing::error!("Failed to store clipboard sync: {:?}", e);
                        } else {
                            hub_clone
                                .broadcast_to_user(
                                    user_id,
                                    &WsEvent::ClipboardUpdated {
                                        id: item_id,
                                        content,
                                        content_type,
                                        source_device_id: src_id,
                                        source_device_name: Some(dev_name.clone()),
                                        created_at: Utc::now().to_rfc3339(),
                                    },
                                    Some(device_id),
                                )
                                .await;
                        }
                    }
                    Ok(WsEvent::Telemetry(data)) => {
                        if is_device {
                            let _ = sqlx::query("UPDATE devices SET last_seen_at = NOW() WHERE id = $1")
                                .bind(device_id)
                                .execute(&db_clone)
                                .await;

                            hub_clone
                                .broadcast_to_user(
                                    user_id,
                                    &WsEvent::DeviceStatusUpdated {
                                        device_id,
                                        data,
                                    },
                                    Some(device_id),
                                )
                                .await;
                        }
                    }
                    Ok(WsEvent::HardwareTelemetry(telemetry)) => {
                        if is_device {
                            let db_c = db_clone.clone();
                            let hub_c = hub_clone.clone();
                            tokio::spawn(async move {
                                match crate::services::specs_enricher::enrich_device_and_store(&db_c, device_id, telemetry).await {
                                    Ok(enriched) => {
                                        hub_c
                                            .broadcast_to_user(
                                                user_id,
                                                &WsEvent::DeviceSpecsUpdated {
                                                    device_id,
                                                    specs: enriched,
                                                },
                                                Some(device_id),
                                            )
                                            .await;
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to enrich device telemetry via WS: {:?}", e);
                                    }
                                }
                            });
                        }
                    }
                    Ok(WsEvent::SendCommand {
                        target_device_id,
                        command,
                        payload,
                    }) => {
                        let cmd_id = Uuid::new_v4();
                        let src_id = if is_device { Some(device_id) } else { None };

                        let _ = sqlx::query(
                            r#"INSERT INTO remote_commands (id, user_id, source_device_id, target_device_id, command, payload, status)
                               VALUES ($1, $2, $3, $4, $5, $6, 'sent')"#,
                        )
                        .bind(cmd_id)
                        .bind(user_id)
                        .bind(src_id)
                        .bind(target_device_id)
                        .bind(&command)
                        .bind(&payload)
                        .execute(&db_clone)
                        .await;

                        let delivered = hub_clone
                            .send_to_device(
                                user_id,
                                target_device_id,
                                &WsEvent::ExecuteCommand {
                                    command_id: cmd_id,
                                    command,
                                    payload,
                                    source_device_id: src_id,
                                },
                            )
                            .await;

                        if !delivered {
                            let _ = sqlx::query("UPDATE remote_commands SET status = 'pending' WHERE id = $1")
                                .bind(cmd_id)
                                .execute(&db_clone)
                                .await;
                        }
                    }
                    Ok(WsEvent::CommandResult {
                        command_id,
                        status,
                        response,
                    }) => {
                        let _ = sqlx::query(
                            r#"UPDATE remote_commands SET status = $1, response = $2, updated_at = NOW() WHERE id = $3 AND user_id = $4"#,
                        )
                        .bind(&status)
                        .bind(&response)
                        .bind(command_id)
                        .bind(user_id)
                        .execute(&db_clone)
                        .await;

                        hub_clone
                            .broadcast_to_user(
                                user_id,
                                &WsEvent::CommandResultReceived {
                                    command_id,
                                    status,
                                    response,
                                    device_id,
                                },
                                Some(device_id),
                            )
                            .await;
                    }
                    Ok(WsEvent::RelayMessage {
                        target_device_id,
                        event,
                        data,
                    }) => {
                        let relay_event = WsEvent::RelayEvent {
                            source_device_id: if is_device { Some(device_id) } else { None },
                            event,
                            data,
                        };

                        if let Some(target_id) = target_device_id {
                            let _ = hub_clone
                                .send_to_device(user_id, target_id, &relay_event)
                                .await;
                        } else {
                            hub_clone
                                .broadcast_to_user(user_id, &relay_event, Some(device_id))
                                .await;
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!("Unrecognized WS message: {:?}", e);
                        let _ = hub_clone
                            .send_to_device(
                                user_id,
                                device_id,
                                &WsEvent::Error {
                                    message: format!("Malformed event payload: {}", e),
                                },
                            )
                            .await;
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    // Cleanup on disconnect
    tracing::info!("WebSocket DISCONNECTED: device='{}', device_id={}", dev_name, device_id);
    send_task.abort();
    state.hub.unregister(user_id, device_id).await;

    if is_device {
        let _ = sqlx::query("UPDATE devices SET is_online = false, last_seen_at = NOW() WHERE id = $1")
            .bind(device_id)
            .execute(&state.db)
            .await;

        state
            .hub
            .broadcast_to_user(
                user_id,
                &WsEvent::DeviceDisconnected { device_id },
                Some(device_id),
            )
            .await;
    }
}
