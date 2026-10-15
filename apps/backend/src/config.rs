use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub upload_dir: String,
    pub max_upload_size: usize,
}

impl Config {
    pub fn from_env() -> Self {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8080);
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment or .env");
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "kusuri_default_jwt_secret_change_me".to_string());
        let upload_dir = env::var("UPLOAD_DIR").unwrap_or_else(|_| "./data/uploads".to_string());
        let max_upload_size = env::var("MAX_UPLOAD_SIZE")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(100 * 1024 * 1024); // 100 MB default

        Self {
            host,
            port,
            database_url,
            jwt_secret,
            upload_dir,
            max_upload_size,
        }
    }
}
