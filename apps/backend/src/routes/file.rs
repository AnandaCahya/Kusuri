use crate::auth::{authenticate_token_against_db, AuthenticatedUser};
use crate::error::AppError;
use crate::models::file::SharedFileResponse;
use crate::ws::hub::WsEvent;
use crate::AppState;
use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
    Json,
};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct DownloadQuery {
    pub token: Option<String>,
}

pub async fn upload_file(
    auth_user: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<SharedFileResponse>, AppError> {
    let mut original_filename = String::new();
    let mut mime_type = "application/octet-stream".to_string();
    let mut target_device_id: Option<Uuid> = None;
    let mut file_bytes_written: i64 = 0;
    let file_id = Uuid::new_v4();
    let mut stored_filename = String::new();

    // Ensure uploads directory exists
    tokio::fs::create_dir_all(&state.config.upload_dir)
        .await
        .map_err(AppError::Io)?;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to parse multipart field: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "target_device_id" {
            let text = field
                .text()
                .await
                .map_err(|e| AppError::BadRequest(format!("Invalid target_device_id: {}", e)))?;
            if let Ok(id) = Uuid::parse_str(&text) {
                target_device_id = Some(id);
            }
        } else if name == "file" {
            let raw_fname = field
                .file_name()
                .unwrap_or("unnamed_file")
                .to_string();
            // Sanitize filename
            original_filename = std::path::Path::new(&raw_fname)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("unnamed_file")
                .to_string();

            if let Some(content_type) = field.content_type() {
                mime_type = content_type.to_string();
            }

            stored_filename = format!("{}_{}", file_id, original_filename);
            let target_path = PathBuf::from(&state.config.upload_dir).join(&stored_filename);

            let mut out_file = File::create(&target_path)
                .await
                .map_err(AppError::Io)?;

            let mut stream = field;
            while let Some(chunk) = stream
                .chunk()
                .await
                .map_err(|e| AppError::BadRequest(format!("Failed reading file chunk: {}", e)))?
            {
                file_bytes_written += chunk.len() as i64;
                if file_bytes_written as usize > state.config.max_upload_size {
                    // Remove partially written file
                    let _ = tokio::fs::remove_file(&target_path).await;
                    return Err(AppError::BadRequest("File exceeds max upload size limit".to_string()));
                }
                out_file.write_all(&chunk).await.map_err(AppError::Io)?;
            }
            out_file.flush().await.map_err(AppError::Io)?;
        }
    }

    if stored_filename.is_empty() || file_bytes_written == 0 {
        return Err(AppError::BadRequest("No file uploaded or file was empty".to_string()));
    }

    let download_url = format!("/api/files/{}/download", file_id);

    sqlx::query(
        r#"INSERT INTO shared_files (id, user_id, source_device_id, target_device_id, original_filename, stored_filename, file_size, mime_type, created_at)
           VALUES ($1, $2, NULL, $3, $4, $5, $6, $7, NOW())"#,
    )
    .bind(file_id)
    .bind(auth_user.user_id)
    .bind(target_device_id)
    .bind(&original_filename)
    .bind(&stored_filename)
    .bind(file_bytes_written)
    .bind(&mime_type)
    .execute(&state.db)
    .await?;

    let response = SharedFileResponse {
        id: file_id,
        user_id: auth_user.user_id,
        source_device_id: None,
        source_device_name: Some(auth_user.username),
        target_device_id,
        original_filename: original_filename.clone(),
        file_size: file_bytes_written,
        mime_type: mime_type.clone(),
        download_url: download_url.clone(),
        created_at: chrono::Utc::now(),
    };

    // Broadcast or send file event over WebSocket
    let ws_event = WsEvent::FileAvailable {
        file_id,
        original_filename,
        file_size: file_bytes_written,
        mime_type,
        download_url,
        source_device_id: None,
    };

    if let Some(target_id) = target_device_id {
        state
            .hub
            .send_to_device(auth_user.user_id, target_id, &ws_event)
            .await;
    } else {
        state
            .hub
            .broadcast_to_user(auth_user.user_id, &ws_event, None)
            .await;
    }

    Ok(Json(response))
}

pub async fn list_files(
    auth_user: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<SharedFileResponse>>, AppError> {
    let files = sqlx::query_as::<_, SharedFileResponse>(
        r#"SELECT 
            f.id,
            f.user_id,
            f.source_device_id,
            d.name as source_device_name,
            f.target_device_id,
            f.original_filename,
            f.file_size,
            f.mime_type,
            CONCAT('/api/files/', f.id, '/download') as download_url,
            f.created_at
           FROM shared_files f
           LEFT JOIN devices d ON f.source_device_id = d.id
           WHERE f.user_id = $1
           ORDER BY f.created_at DESC
           LIMIT 50"#,
    )
    .bind(auth_user.user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(files))
}

pub async fn download_file(
    Path(file_id): Path<Uuid>,
    headers: HeaderMap,
    Query(query): Query<DownloadQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Response, AppError> {
    // Authenticate via token in query (for direct browser/download manager links) or header
    let token = query
        .token
        .or_else(|| {
            headers
                .get(header::AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "))
                .map(|s| s.trim().to_string())
        })
        .or_else(|| {
            headers
                .get("x-device-token")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.trim().to_string())
        })
        .ok_or_else(|| AppError::Unauthorized("Authentication required for download".to_string()))?;

    let caller = authenticate_token_against_db(&state.db, &state.config.jwt_secret, &token).await?;

    let file_record = sqlx::query_as::<_, (String, String, String, i64, Uuid)>(
        "SELECT stored_filename, original_filename, mime_type, file_size, user_id FROM shared_files WHERE id = $1",
    )
    .bind(file_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    let (stored_filename, original_filename, mime_type, file_size, owner_id) = file_record;

    if owner_id != caller.user_id() {
        return Err(AppError::Forbidden("Access denied to file".to_string()));
    }

    let file_path = PathBuf::from(&state.config.upload_dir).join(&stored_filename);
    let file = File::open(&file_path).await.map_err(|e| {
        tracing::error!("Failed to open stored file at {:?}: {:?}", file_path, e);
        AppError::NotFound("Stored file missing on disk".to_string())
    })?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let content_disposition = format!("attachment; filename=\"{}\"", original_filename);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .header(header::CONTENT_LENGTH, file_size)
        .body(body)
        .map_err(|e| AppError::Internal(format!("Failed to build download response: {}", e)))?;

    Ok(response)
}

pub async fn delete_file(
    auth_user: AuthenticatedUser,
    Path(file_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let file_record = sqlx::query_as::<_, (String,)>(
        "SELECT stored_filename FROM shared_files WHERE id = $1 AND user_id = $2",
    )
    .bind(file_id)
    .bind(auth_user.user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    let file_path = PathBuf::from(&state.config.upload_dir).join(&file_record.0);
    let _ = tokio::fs::remove_file(file_path).await;

    sqlx::query("DELETE FROM shared_files WHERE id = $1 AND user_id = $2")
        .bind(file_id)
        .bind(auth_user.user_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "File deleted successfully"
    })))
}
