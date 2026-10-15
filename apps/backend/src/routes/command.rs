use crate::auth::AuthenticatedUser;
use crate::error::AppError;
use crate::models::command::{
    CommandResultRequest, RemoteCommandResponse, SendCommandRequest,
};
use crate::ws::hub::WsEvent;
use crate::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

pub async fn send_command(
    auth_user: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SendCommandRequest>,
) -> Result<Json<RemoteCommandResponse>, AppError> {
    // Verify target device belongs to the user
    let target_dev = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT id, name FROM devices WHERE id = $1 AND user_id = $2",
    )
    .bind(payload.target_device_id)
    .bind(auth_user.user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Target device not found or does not belong to you".to_string()))?;

    let cmd_id = Uuid::new_v4();
    let is_online = state
        .hub
        .is_device_online(auth_user.user_id, payload.target_device_id)
        .await;

    let initial_status = if is_online { "sent" } else { "pending" };

    sqlx::query(
        r#"INSERT INTO remote_commands (id, user_id, source_device_id, target_device_id, command, payload, status, created_at, updated_at)
           VALUES ($1, $2, NULL, $3, $4, $5, $6, NOW(), NOW())"#,
    )
    .bind(cmd_id)
    .bind(auth_user.user_id)
    .bind(payload.target_device_id)
    .bind(&payload.command)
    .bind(&payload.payload)
    .bind(initial_status)
    .execute(&state.db)
    .await?;

    if is_online {
        state
            .hub
            .send_to_device(
                auth_user.user_id,
                payload.target_device_id,
                &WsEvent::ExecuteCommand {
                    command_id: cmd_id,
                    command: payload.command.clone(),
                    payload: payload.payload.clone(),
                    source_device_id: None,
                },
            )
            .await;
    }

    let response = RemoteCommandResponse {
        id: cmd_id,
        user_id: auth_user.user_id,
        source_device_id: None,
        source_device_name: Some(auth_user.username),
        target_device_id: payload.target_device_id,
        target_device_name: Some(target_dev.1),
        command: payload.command,
        payload: payload.payload,
        status: initial_status.to_string(),
        response: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    Ok(Json(response))
}

pub async fn list_commands(
    auth_user: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<RemoteCommandResponse>>, AppError> {
    let commands = sqlx::query_as::<_, RemoteCommandResponse>(
        r#"SELECT 
            c.id,
            c.user_id,
            c.source_device_id,
            sd.name as source_device_name,
            c.target_device_id,
            td.name as target_device_name,
            c.command,
            c.payload,
            c.status,
            c.response,
            c.created_at,
            c.updated_at
           FROM remote_commands c
           LEFT JOIN devices sd ON c.source_device_id = sd.id
           JOIN devices td ON c.target_device_id = td.id
           WHERE c.user_id = $1
           ORDER BY c.created_at DESC
           LIMIT 50"#,
    )
    .bind(auth_user.user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(commands))
}

pub async fn report_command_result(
    auth_user: AuthenticatedUser,
    Path(cmd_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CommandResultRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let res = sqlx::query(
        r#"UPDATE remote_commands 
           SET status = $1, response = $2, updated_at = NOW() 
           WHERE id = $3 AND user_id = $4"#,
    )
    .bind(&payload.status)
    .bind(&payload.response)
    .bind(cmd_id)
    .bind(auth_user.user_id)
    .execute(&state.db)
    .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Command not found".to_string()));
    }

    state
        .hub
        .broadcast_to_user(
            auth_user.user_id,
            &WsEvent::CommandResultReceived {
                command_id: cmd_id,
                status: payload.status,
                response: payload.response,
                device_id: auth_user.user_id,
            },
            None,
        )
        .await;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Command result recorded"
    })))
}
