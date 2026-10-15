use crate::auth::{hash_token, AuthenticatedUser};
use crate::error::AppError;
use crate::models::device::{
    CreatePairingCodeRequest, Device, DeviceResponse, PairDeviceRequest, PairDeviceResponse,
    PairingCodeResponse, UpdateDeviceRequest,
};
use crate::services::google_devices::{lookup_device_by_codename, sync_google_devices_catalog};
use crate::services::gsmarena::get_or_fetch_gsmarena_specs;
use crate::services::specs_enricher::{
    enrich_device_and_store, get_device_full_specs, EnrichedDeviceSpecsResponse,
    NativeHardwareTelemetryRequest,
};
use crate::ws::hub::WsEvent;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use chrono::{Duration, Utc};
use rand::RngExt;
use std::sync::Arc;
use uuid::Uuid;

pub async fn create_pairing_code(
    auth_user: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatePairingCodeRequest>,
) -> Result<Json<PairingCodeResponse>, AppError> {
    let pairing_id = Uuid::new_v4();
    let code = {
        let mut rng = rand::rng();
        let code_num: u32 = rng.random_range(100_000..=999_999);
        code_num.to_string()
    };

    // Reduced lifetime: 5 minutes max as recommended in security review
    let expires_at = Utc::now() + Duration::minutes(5);
    let device_name = payload.device_name;
    let device_type = payload.device_type;

    // Clean up any expired pairing codes first
    let _ = sqlx::query("DELETE FROM device_pairing_codes WHERE expires_at < NOW()")
        .execute(&state.db)
        .await;

    sqlx::query(
        r#"INSERT INTO device_pairing_codes (id, code, user_id, device_name, device_type, attempts, max_attempts, expires_at, created_at)
           VALUES ($1, $2, $3, $4, $5, 0, 3, $6, NOW())"#,
    )
    .bind(pairing_id)
    .bind(&code)
    .bind(auth_user.user_id)
    .bind(device_name)
    .bind(device_type)
    .bind(expires_at)
    .execute(&state.db)
    .await?;

    Ok(Json(PairingCodeResponse {
        pairing_id,
        code,
        expires_at,
        expires_in_seconds: 300,
        max_attempts: 3,
    }))
}

pub async fn pair_device(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PairDeviceRequest>,
) -> Result<Json<PairDeviceResponse>, AppError> {
    // Extract client IP for anti-brute force rate limiting
    let client_ip = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "client_direct".to_string());

    // 1. Check in-memory rate limiter (blocked if 3 failures within 5 minutes)
    if !state.pairing_limiter.check_allowed(&client_ip).await {
        return Err(AppError::Forbidden(
            "Terlalu banyak percobaan pairing gagal. Akses diblokir sementara selama 5 menit demi keamanan.".to_string(),
        ));
    }

    // 2. Validate pairing code and enforce 3-attempt limit in database
    let (record_id, user_id, opt_name, opt_type) = if let Some(pid) = payload.pairing_id {
        // Pairing with specific session ID (e.g. from QR code)
        let row = sqlx::query_as::<_, (Uuid, String, Uuid, Option<String>, Option<String>, i32, i32)>(
            r#"SELECT id, code, user_id, device_name, device_type, attempts, max_attempts
               FROM device_pairing_codes 
               WHERE id = $1 AND expires_at > NOW()"#,
        )
        .bind(pid)
        .fetch_optional(&state.db)
        .await?;

        match row {
            Some((id, code, uid, d_name, d_type, attempts, max_attempts)) => {
                if code != payload.pairing_code.trim() {
                    let new_attempts = attempts + 1;
                    if new_attempts >= max_attempts {
                        // Max attempts reached, burn the code!
                        let _ = sqlx::query("DELETE FROM device_pairing_codes WHERE id = $1")
                            .bind(id)
                            .execute(&state.db)
                            .await;
                        state.pairing_limiter.record_failure(&client_ip).await;
                        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
                        return Err(AppError::Forbidden(
                            "Kode pairing salah sebanyak 3 kali. Kode telah dibatalkan demi keamanan.".to_string(),
                        ));
                    } else {
                        let _ = sqlx::query("UPDATE device_pairing_codes SET attempts = $1 WHERE id = $2")
                            .bind(new_attempts)
                            .bind(id)
                            .execute(&state.db)
                            .await;
                        state.pairing_limiter.record_failure(&client_ip).await;
                        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
                        return Err(AppError::BadRequest(format!(
                            "Kode pairing salah. Sisa kesempatan: {}",
                            max_attempts - new_attempts
                        )));
                    }
                }
                (id, uid, d_name, d_type)
            }
            None => {
                state.pairing_limiter.record_failure(&client_ip).await;
                tokio::time::sleep(std::time::Duration::from_millis(600)).await;
                return Err(AppError::BadRequest("Sesi pairing tidak valid atau telah kedaluwarsa.".to_string()));
            }
        }
    } else {
        // Direct 6-digit code entry
        let row = sqlx::query_as::<_, (Uuid, Uuid, Option<String>, Option<String>, i32, i32)>(
            r#"SELECT id, user_id, device_name, device_type, attempts, max_attempts 
               FROM device_pairing_codes 
               WHERE code = $1 AND expires_at > NOW()"#,
        )
        .bind(payload.pairing_code.trim())
        .fetch_optional(&state.db)
        .await?;

        match row {
            Some((id, uid, d_name, d_type, _, _)) => (id, uid, d_name, d_type),
            None => {
                let count = state.pairing_limiter.record_failure(&client_ip).await;
                tokio::time::sleep(std::time::Duration::from_millis(600)).await;
                return Err(AppError::BadRequest(format!(
                    "Kode pairing tidak valid atau telah kedaluwarsa. (Gagal: {}/3)",
                    count
                )));
            }
        }
    };

    // Pairing succeeded: reset failure counter & consume code immediately
    state.pairing_limiter.reset(&client_ip).await;
    sqlx::query("DELETE FROM device_pairing_codes WHERE id = $1")
        .bind(record_id)
        .execute(&state.db)
        .await?;

    let device_id = Uuid::new_v4();

    // 3. Generate high-entropy raw device token (ksr_ + 64 hex characters = 256 bits entropy)
    let raw_device_token = format!(
        "ksr_{}{}",
        Uuid::new_v4().simple(),
        Uuid::new_v4().simple()
    );

    // Compute SHA-256 hash for secure database storage
    let token_hash = hash_token(&raw_device_token);

    // Masked token for administrative display (first 8 and last 4 characters)
    let masked_token = format!(
        "{}...{}",
        &raw_device_token[..8],
        &raw_device_token[raw_device_token.len() - 4..]
    );

    let raw_model = payload.raw_model.clone().or_else(|| {
        payload.device_name.as_ref().and_then(|name| {
            if name.contains('-') {
                Some(name.clone())
            } else {
                None
            }
        })
    });

    // Check Google Supported Devices & GSM Arena specs if model is provided
    let mut marketing_name: Option<String> = None;
    let mut retail_branding: Option<String> = None;
    let mut chipset: Option<String> = None;
    let mut specs_id: Option<Uuid> = None;

    if let Some(ref model) = raw_model {
        if let Some(matched) = lookup_device_by_codename(&state.db, model).await {
            retail_branding = Some(matched.retail_branding.clone());
            marketing_name = Some(matched.marketing_name.clone());

            if let Ok(specs) = get_or_fetch_gsmarena_specs(&state.db, &matched.marketing_name, Some(&matched.retail_branding)).await {
                chipset = specs.chipset;
                specs_id = Some(specs.id);
            }
        }
    }

    // Default display name: prefer marketing name (e.g. "Infinix HOT 50 Pro") over raw code name
    let device_name = if let Some(ref mkt) = marketing_name {
        mkt.clone()
    } else if let Some(ref name) = payload.device_name {
        if !name.trim().is_empty() {
            name.clone()
        } else {
            opt_name.unwrap_or_else(|| "Unknown Device".to_string())
        }
    } else {
        opt_name.unwrap_or_else(|| "Unknown Device".to_string())
    };

    let device_type = if let Some(ref dtype) = payload.device_type {
        if !dtype.trim().is_empty() {
            dtype.to_lowercase()
        } else {
            opt_type.unwrap_or_else(|| "other".to_string())
        }
    } else {
        opt_type.unwrap_or_else(|| "other".to_string())
    };

    // Store token_hash (SHA-256) in the database. Raw token is NEVER stored in database!
    sqlx::query(
        r#"INSERT INTO devices (
            id, user_id, name, device_type, device_token, token_hash, is_online, last_seen_at, created_at,
            raw_model, marketing_name, retail_branding, chipset, specs_id
        ) VALUES (
            $1, $2, $3, $4, $5, $6, false, NOW(), NOW(),
            $7, $8, $9, $10, $11
        )"#,
    )
    .bind(device_id)
    .bind(user_id)
    .bind(&device_name)
    .bind(&device_type)
    .bind(&masked_token)
    .bind(&token_hash)
    .bind(&raw_model)
    .bind(&marketing_name)
    .bind(&retail_branding)
    .bind(&chipset)
    .bind(specs_id)
    .execute(&state.db)
    .await?;

    Ok(Json(PairDeviceResponse {
        device_id,
        device_token: raw_device_token, // Provided once to the client for local storage
        user_id,
        device_name,
        device_type,
        marketing_name,
        chipset,
    }))
}

pub async fn list_devices(
    auth_user: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<DeviceResponse>>, AppError> {
    let devices = sqlx::query_as::<_, Device>(
        r#"SELECT 
            id, user_id, name, device_type, device_token, token_hash, is_online, last_seen_at, created_at,
            raw_model, marketing_name, retail_branding, chipset, specs_id
           FROM devices WHERE user_id = $1 ORDER BY created_at ASC"#,
    )
    .bind(auth_user.user_id)
    .fetch_all(&state.db)
    .await?;

    let mut response_list = Vec::with_capacity(devices.len());
    for dev in devices {
        let is_online = state.hub.is_device_online(dev.user_id, dev.id).await || dev.is_online;
        response_list.push(DeviceResponse {
            id: dev.id,
            user_id: dev.user_id,
            name: dev.name,
            device_type: dev.device_type,
            is_online,
            last_seen_at: dev.last_seen_at,
            created_at: dev.created_at,
            raw_model: dev.raw_model,
            marketing_name: dev.marketing_name,
            retail_branding: dev.retail_branding,
            chipset: dev.chipset,
        });
    }

    Ok(Json(response_list))
}

pub async fn get_device_specs(
    auth_user: AuthenticatedUser,
    Path(device_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<EnrichedDeviceSpecsResponse>, AppError> {
    // Verify device belongs to user
    let dev_user = sqlx::query_scalar::<_, Uuid>("SELECT user_id FROM devices WHERE id = $1")
        .bind(device_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

    if dev_user != auth_user.user_id {
        return Err(AppError::Forbidden("Access denied to device".to_string()));
    }

    let specs = get_device_full_specs(&state.db, device_id).await?;
    Ok(Json(specs))
}

pub async fn update_device_telemetry(
    auth_user: AuthenticatedUser,
    Path(device_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NativeHardwareTelemetryRequest>,
) -> Result<Json<EnrichedDeviceSpecsResponse>, AppError> {
    let dev_user = sqlx::query_scalar::<_, Uuid>("SELECT user_id FROM devices WHERE id = $1")
        .bind(device_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

    if dev_user != auth_user.user_id {
        return Err(AppError::Forbidden("Access denied to device".to_string()));
    }

    let enriched = enrich_device_and_store(&state.db, device_id, payload).await?;

    // Broadcast over WebSocket to all user's paired devices
    state
        .hub
        .broadcast_to_user(
            auth_user.user_id,
            &WsEvent::DeviceStatusUpdated {
                device_id,
                data: crate::models::device::TelemetryData {
                    battery_percent: enriched.live_hardware.as_ref().and_then(|h| h.battery_percent),
                    is_charging: enriched.live_hardware.as_ref().and_then(|h| h.battery_charging),
                    os_details: enriched.live_hardware.as_ref().and_then(|h| h.os_version.clone()),
                    memory_used_mb: enriched.live_hardware.as_ref().and_then(|h| h.ram_used_mb.map(|m| m as u64)),
                    memory_total_mb: enriched.live_hardware.as_ref().and_then(|h| h.ram_total_mb.map(|m| m as u64)),
                    ip_address: enriched.live_hardware.as_ref().and_then(|h| h.ip_address.clone()),
                },
            },
            Some(device_id),
        )
        .await;

    Ok(Json(enriched))
}

pub async fn sync_catalog(
    _auth_user: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let count = sync_google_devices_catalog(&state.db, true).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Successfully synchronized {} Google Play supported devices", count)
    })))
}

pub async fn update_device(
    auth_user: AuthenticatedUser,
    Path(device_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateDeviceRequest>,
) -> Result<Json<DeviceResponse>, AppError> {
    if let Some(new_name) = payload.name {
        if new_name.trim().is_empty() {
            return Err(AppError::BadRequest("Device name cannot be empty".to_string()));
        }

        let device = sqlx::query_as::<_, Device>(
            r#"UPDATE devices SET name = $1 WHERE id = $2 AND user_id = $3
               RETURNING 
                id, user_id, name, device_type, device_token, token_hash, is_online, last_seen_at, created_at,
                raw_model, marketing_name, retail_branding, chipset, specs_id"#,
        )
        .bind(new_name)
        .bind(device_id)
        .bind(auth_user.user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Device not found".to_string()))?;

        return Ok(Json(device.into()));
    }

    Err(AppError::BadRequest("No fields to update".to_string()))
}

pub async fn delete_device(
    auth_user: AuthenticatedUser,
    Path(device_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    state.hub.unregister(auth_user.user_id, device_id).await;

    let res = sqlx::query("DELETE FROM devices WHERE id = $1 AND user_id = $2")
        .bind(device_id)
        .bind(auth_user.user_id)
        .execute(&state.db)
        .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Device not found".to_string()));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Device successfully deleted"
    })))
}
