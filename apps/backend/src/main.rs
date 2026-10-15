mod auth;
mod config;
mod error;
mod models;
mod routes;
mod services;
mod ws;

use config::Config;
use routes::create_router;
use services::google_devices::sync_google_devices_catalog;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use ws::hub::WsHub;

use services::rate_limiter::PairingRateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Config,
    pub hub: Arc<WsHub>,
    pub pairing_limiter: Arc<PairingRateLimiter>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Install the default CryptoProvider (aws-lc-rs) for rustls.
    // Required by jsonwebtoken v11 and sqlx tls-rustls before any TLS/JWT operations.
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls CryptoProvider");

    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,backend=debug,tower_http=info".into()),
        )
        .init();

    let config = Config::from_env();

    tracing::info!("Connecting to PostgreSQL database...");
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await?;

    tracing::info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Database migrations applied successfully.");

    // Sync Google Play supported devices list in background if needed
    let catalog_pool = pool.clone();
    tokio::spawn(async move {
        if let Err(e) = sync_google_devices_catalog(&catalog_pool, false).await {
            tracing::warn!("Background Google Play catalog sync notice: {:?}", e);
        }
    });

    // Ensure upload directory exists
    tokio::fs::create_dir_all(&config.upload_dir).await?;

    let hub = Arc::new(WsHub::new());
    let pairing_limiter = Arc::new(PairingRateLimiter::new());

    let state = Arc::new(AppState {
        db: pool,
        config: config.clone(),
        hub,
        pairing_limiter,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = create_router(state)
        .layer(cors)
        .layer(axum::middleware::from_fn(log_request_middleware))
        .layer(TraceLayer::new_for_http());

    let bind_addr = format!("{}:{}", config.host, config.port);
    tracing::info!("Kusuri backend listening on http://{}", bind_addr);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn log_request_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start = std::time::Instant::now();

    let response = next.run(req).await;

    let duration = start.elapsed();
    let status = response.status();
    tracing::info!(
        "HTTP [{}] {} -> {} ({:.2?})",
        method,
        uri,
        status,
        duration
    );

    response
}