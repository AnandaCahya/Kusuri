use crate::auth::AuthenticatedUser;
use crate::error::AppError;
use crate::models::clipboard::{ClipboardResponse, CreateClipboardRequest};
use crate::ws::hub::WsEvent;
use crate::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub async fn list_clipboard(
    auth_user: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ClipboardResponse>>, AppError> {
    let items = sqlx::query_as::<_, ClipboardResponse>(
        r#"SELECT 
            c.id, 
            c.user_id, 
            c.source_device_id, 
            d.name as source_device_name,
            c.content, 
            c.content_type, 
            c.created_at
           FROM clipboard_items c
           LEFT JOIN devices d ON c.source_device_id = d.id
           WHERE c.user_id = $1
           ORDER BY c.created_at DESC
           LIMIT 50"#,
    )
    .bind(auth_user.user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(items))
}

pub async fn create_clipboard(
    auth_user: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateClipboardRequest>,
) -> Result<Json<ClipboardResponse>, AppError> {
    if payload.content.trim().is_empty() {
        return Err(AppError::BadRequest("Content cannot be empty".to_string()));
    }

    let item_id = Uuid::new_v4();
    let content_type = payload.content_type.unwrap_or_else(|| "text/plain".to_string());
    let now = Utc::now();

    sqlx::query(
        r#"INSERT INTO clipboard_items (id, user_id, source_device_id, content, content_type, created_at)
           VALUES ($1, $2, NULL, $3, $4, $5)"#,
    )
    .bind(item_id)
    .bind(auth_user.user_id)
    .bind(&payload.content)
    .bind(&content_type)
    .bind(now)
    .execute(&state.db)
    .await?;

    let response = ClipboardResponse {
        id: item_id,
        user_id: auth_user.user_id,
        source_device_id: None,
        source_device_name: Some(auth_user.username.clone()),
        content: payload.content.clone(),
        content_type: content_type.clone(),
        created_at: now,
    };

    // Broadcast over WebSocket to all user devices
    state
        .hub
        .broadcast_to_user(
            auth_user.user_id,
            &WsEvent::ClipboardUpdated {
                id: item_id,
                content: payload.content,
                content_type,
                source_device_id: None,
                source_device_name: Some(auth_user.username),
                created_at: now.to_rfc3339(),
            },
            None,
        )
        .await;

    Ok(Json(response))
}

pub async fn delete_clipboard(
    auth_user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let res = sqlx::query("DELETE FROM clipboard_items WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(auth_user.user_id)
        .execute(&state.db)
        .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Clipboard item not found".to_string()));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Clipboard item deleted"
    })))
}
