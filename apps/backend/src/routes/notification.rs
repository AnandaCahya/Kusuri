use crate::auth::AuthCaller;
use crate::error::AppError;
use crate::models::notification::{IngestNotificationRequest, NotificationQuery, NotificationResponse};
use crate::ws::hub::WsEvent;
use crate::AppState;
use axum::{
    extract::{Query, State},
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub async fn ingest_notification(
    caller: AuthCaller,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<IngestNotificationRequest>,
) -> Result<Json<NotificationResponse>, AppError> {
    if payload.title.trim().is_empty() && payload.content.trim().is_empty() {
        return Err(AppError::BadRequest("Notification title and content cannot both be empty".to_string()));
    }

    let user_id = caller.user_id();
    let device_id = caller.device_id().or(payload.device_id);
    let posted_at = payload.posted_at.unwrap_or_else(Utc::now);
    let notif_id = Uuid::new_v4();

    sqlx::query(
        r#"INSERT INTO device_notifications 
           (id, user_id, device_id, package_or_app_name, app_title, title, content, subtext, posted_at, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())"#,
    )
    .bind(notif_id)
    .bind(user_id)
    .bind(device_id)
    .bind(&payload.package_or_app_name)
    .bind(&payload.app_title)
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(&payload.subtext)
    .bind(posted_at)
    .execute(&state.db)
    .await?;

    let device_name = if let Some(d_id) = device_id {
        sqlx::query_scalar::<_, String>("SELECT name FROM devices WHERE id = $1")
            .bind(d_id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten()
    } else {
        None
    };

    let response = NotificationResponse {
        id: notif_id,
        user_id,
        device_id,
        device_name,
        package_or_app_name: payload.package_or_app_name,
        app_title: payload.app_title,
        title: payload.title,
        content: payload.content,
        subtext: payload.subtext,
        posted_at,
        created_at: Utc::now(),
    };

    // Broadcast over WebSocket to all active dashboards and connected devices of this user
    state
        .hub
        .broadcast_to_user(
            user_id,
            &WsEvent::NotificationIngested(response.clone()),
            device_id,
        )
        .await;

    Ok(Json(response))
}

pub async fn list_notifications(
    caller: AuthCaller,
    State(state): State<Arc<AppState>>,
    Query(query): Query<NotificationQuery>,
) -> Result<Json<Vec<NotificationResponse>>, AppError> {
    let user_id = caller.user_id();
    let limit = query.limit.unwrap_or(50).min(200);

    let responses: Vec<NotificationResponse> = if let Some(search) = &query.search {
        let pattern = format!("%{}%", search.trim());
        if let Some(target_dev_id) = query.device_id {
            sqlx::query_as::<_, NotificationResponse>(
                r#"SELECT n.id, n.user_id, n.device_id, d.name as device_name, n.package_or_app_name, n.app_title, n.title, n.content, n.subtext, n.posted_at, n.created_at
                   FROM device_notifications n
                   LEFT JOIN devices d ON n.device_id = d.id
                   WHERE n.user_id = $1 AND n.device_id = $2 AND (n.title ILIKE $3 OR n.content ILIKE $3 OR n.package_or_app_name ILIKE $3 OR n.app_title ILIKE $3)
                   ORDER BY n.posted_at DESC
                   LIMIT $4"#,
            )
            .bind(user_id)
            .bind(target_dev_id)
            .bind(pattern)
            .bind(limit)
            .fetch_all(&state.db)
            .await?
        } else {
            sqlx::query_as::<_, NotificationResponse>(
                r#"SELECT n.id, n.user_id, n.device_id, d.name as device_name, n.package_or_app_name, n.app_title, n.title, n.content, n.subtext, n.posted_at, n.created_at
                   FROM device_notifications n
                   LEFT JOIN devices d ON n.device_id = d.id
                   WHERE n.user_id = $1 AND (n.title ILIKE $2 OR n.content ILIKE $2 OR n.package_or_app_name ILIKE $2 OR n.app_title ILIKE $2)
                   ORDER BY n.posted_at DESC
                   LIMIT $3"#,
            )
            .bind(user_id)
            .bind(pattern)
            .bind(limit)
            .fetch_all(&state.db)
            .await?
        }
    } else if let Some(target_dev_id) = query.device_id {
        sqlx::query_as::<_, NotificationResponse>(
            r#"SELECT n.id, n.user_id, n.device_id, d.name as device_name, n.package_or_app_name, n.app_title, n.title, n.content, n.subtext, n.posted_at, n.created_at
               FROM device_notifications n
               LEFT JOIN devices d ON n.device_id = d.id
               WHERE n.user_id = $1 AND n.device_id = $2
               ORDER BY n.posted_at DESC
               LIMIT $3"#,
        )
        .bind(user_id)
        .bind(target_dev_id)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, NotificationResponse>(
            r#"SELECT n.id, n.user_id, n.device_id, d.name as device_name, n.package_or_app_name, n.app_title, n.title, n.content, n.subtext, n.posted_at, n.created_at
               FROM device_notifications n
               LEFT JOIN devices d ON n.device_id = d.id
               WHERE n.user_id = $1
               ORDER BY n.posted_at DESC
               LIMIT $2"#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    };

    Ok(Json(responses))
}
