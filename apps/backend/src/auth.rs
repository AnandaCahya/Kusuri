use crate::error::AppError;
use argon2::password_hash::phc::PasswordHash;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use argon2::Argon2;
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // user_id as string
    pub username: String,
    pub exp: usize,
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes())
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(format!("Password hashing error: {}", e)))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Invalid password hash format: {}", e)))?;
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

pub fn create_jwt(user_id: &Uuid, username: &str, secret: &str) -> Result<String, AppError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(30))
        .expect("Valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("JWT creation error: {}", e)))
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| AppError::Unauthorized(format!("Invalid or expired token: {}", e)))
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub username: String,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedDevice {
    pub device_id: Uuid,
    pub user_id: Uuid,
    pub device_name: String,
    pub device_type: String,
}

#[derive(Debug, Clone)]
pub enum AuthCaller {
    User(AuthenticatedUser),
    Device(AuthenticatedDevice),
}

impl AuthCaller {
    pub fn user_id(&self) -> Uuid {
        match self {
            AuthCaller::User(u) => u.user_id,
            AuthCaller::Device(d) => d.user_id,
        }
    }

    #[allow(dead_code)]
    pub fn device_id(&self) -> Option<Uuid> {
        match self {
            AuthCaller::User(_) => None,
            AuthCaller::Device(d) => Some(d.device_id),
        }
    }
}

#[derive(Deserialize)]
struct TokenQuery {
    token: Option<String>,
}

fn extract_token_from_parts(parts: &Parts) -> Option<String> {
    // 1. Check Authorization header: Bearer <token>
    if let Some(auth_header) = parts.headers.get(AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Some(token.trim().to_string());
            }
        }
    }

    // 2. Check X-Device-Token header
    if let Some(device_token_header) = parts.headers.get("x-device-token") {
        if let Ok(token_str) = device_token_header.to_str() {
            return Some(token_str.trim().to_string());
        }
    }

    // 3. Check query string `?token=...`
    if let Some(query_str) = parts.uri.query() {
        if let Ok(params) = serde_urlencoded::from_str::<TokenQuery>(query_str) {
            if let Some(token) = params.token {
                return Some(token);
            }
        }
    }

    None
}

// Extractor for User (Authenticates either via JWT OR via paired Device Token `ksr_...`)
impl FromRequestParts<std::sync::Arc<crate::AppState>> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &std::sync::Arc<crate::AppState>,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_token_from_parts(parts)
            .ok_or_else(|| AppError::Unauthorized("Missing authentication token".to_string()))?;

        // 1. First, attempt to verify as user JWT
        if let Ok(claims) = verify_jwt(&token, &state.config.jwt_secret) {
            if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
                return Ok(AuthenticatedUser {
                    user_id,
                    username: claims.username,
                });
            }
        }

        // 2. If not a valid JWT, check if it matches a registered device_token (ksr_...)
        let hashed = hash_token(&token);
        let dev_opt = sqlx::query_as::<_, (Uuid, String)>(
            r#"SELECT d.user_id, u.username
               FROM devices d
               JOIN users u ON d.user_id = u.id
               WHERE d.token_hash = $1"#,
        )
        .bind(&hashed)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("Database error during device auth: {}", e)))?;

        if let Some((user_id, username)) = dev_opt {
            return Ok(AuthenticatedUser {
                user_id,
                username,
            });
        }

        Err(AppError::Unauthorized("Invalid or expired authentication credentials".to_string()))
    }
}

pub fn hash_token(raw_token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    hex::encode(hasher.finalize())
}

// Extractor for Caller (Can be User via JWT OR Device via device_token)
// Needs AppState to query DB if it's a device token
pub async fn authenticate_token_against_db(
    pool: &PgPool,
    secret: &str,
    token: &str,
) -> Result<AuthCaller, AppError> {
    // First, attempt to verify as a user JWT
    if let Ok(claims) = verify_jwt(token, secret) {
        if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
            return Ok(AuthCaller::User(AuthenticatedUser {
                user_id,
                username: claims.username,
            }));
        }
    }

    // Otherwise, check if it matches a registered device_token hash
    let hashed = hash_token(token);
    let device_opt = sqlx::query_as::<_, (Uuid, Uuid, String, String)>(
        "SELECT id, user_id, name, device_type FROM devices WHERE token_hash = $1",
    )
    .bind(&hashed)
    .fetch_optional(pool)
    .await?;

    if let Some((device_id, user_id, device_name, device_type)) = device_opt {
        return Ok(AuthCaller::Device(AuthenticatedDevice {
            device_id,
            user_id,
            device_name,
            device_type,
        }));
    }

    Err(AppError::Unauthorized("Invalid authentication credentials".to_string()))
}

// Extractor for AuthCaller directly in Axum handler signatures
impl FromRequestParts<std::sync::Arc<crate::AppState>> for AuthCaller {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &std::sync::Arc<crate::AppState>,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_token_from_parts(parts)
            .ok_or_else(|| AppError::Unauthorized("Missing authentication token".to_string()))?;

        authenticate_token_against_db(&state.db, &state.config.jwt_secret, &token).await
    }
}

