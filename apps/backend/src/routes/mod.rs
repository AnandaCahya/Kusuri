pub mod auth;
pub mod clipboard;
pub mod command;
pub mod device;
pub mod file;

use crate::ws::handler::ws_handler;
use crate::AppState;
use axum::{
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde_json::json;
use std::sync::Arc;

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "kusuri-backend",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

pub fn create_router(state: Arc<AppState>) -> Router {
    let api_routes = Router::new()
        .route("/health", get(health_check))
        // Auth
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/me", get(auth::me))
        .route("/devices/pairing-code", post(device::create_pairing_code))
        .route("/devices/pair", post(device::pair_device))
        .route("/devices/sync-catalog", post(device::sync_catalog))
        .route("/devices", get(device::list_devices))
        .route("/devices/{id}", patch(device::update_device))
        .route("/devices/{id}", delete(device::delete_device))
        .route("/devices/{id}/specs", get(device::get_device_specs))
        .route("/devices/{id}/telemetry", post(device::update_device_telemetry))
        // Clipboard
        .route("/clipboard", get(clipboard::list_clipboard))
        .route("/clipboard", post(clipboard::create_clipboard))
        .route("/clipboard/{id}", delete(clipboard::delete_clipboard))
        // Files
        .route("/files/upload", post(file::upload_file))
        .route("/files/{id}/download", get(file::download_file))
        .route("/files", get(file::list_files))
        .route("/files/{id}", delete(file::delete_file))
        // Commands
        .route("/commands", post(command::send_command))
        .route("/commands", get(command::list_commands))
        .route("/commands/{id}/result", post(command::report_command_result));

    Router::new()
        .nest("/api", api_routes)
        .route("/ws", get(ws_handler))
        .with_state(state)
}
