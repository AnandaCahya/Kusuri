use crate::auth::{create_jwt, hash_password, verify_password, AuthenticatedUser};
use crate::error::AppError;
use crate::models::user::{AuthResponse, LoginRequest, RegisterRequest, User, UserResponse};
use crate::AppState;
use axum::{extract::State, Json};
use std::sync::Arc;
use uuid::Uuid;

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    if payload.username.trim().is_empty() || payload.password.len() < 6 {
        return Err(AppError::BadRequest(
            "Username cannot be empty and password must be at least 6 characters".to_string(),
        ));
    }

    // Check if username is already taken
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)",
    )
    .bind(&payload.username)
    .fetch_one(&state.db)
    .await?;

    if exists {
        return Err(AppError::BadRequest("Username already taken".to_string()));
    }

    let password_hash = hash_password(&payload.password)?;
    let user_id = Uuid::new_v4();

    let user = sqlx::query_as::<_, User>(
        r#"INSERT INTO users (id, username, password_hash, created_at)
           VALUES ($1, $2, $3, NOW())
           RETURNING id, username, password_hash, created_at"#,
    )
    .bind(user_id)
    .bind(&payload.username)
    .bind(password_hash)
    .fetch_one(&state.db)
    .await?;

    let token = create_jwt(&user.id, &user.username, &state.config.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, created_at FROM users WHERE username = $1",
    )
    .bind(&payload.username)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid username or password".to_string()))?;

    let valid = verify_password(&payload.password, &user.password_hash)?;
    if !valid {
        return Err(AppError::Unauthorized("Invalid username or password".to_string()));
    }

    let token = create_jwt(&user.id, &user.username, &state.config.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

pub async fn me(
    auth_user: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<UserResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, created_at FROM users WHERE id = $1",
    )
    .bind(auth_user.user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user.into()))
}
