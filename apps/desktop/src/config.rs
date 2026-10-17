use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub server_url: String,
    #[serde(default)]
    pub ws_url: Option<String>,
    #[serde(default)]
    pub device_id: Option<Uuid>,
    #[serde(default)]
    pub device_token: Option<String>,
    #[serde(default)]
    pub auth_token: Option<String>,
    #[serde(default)]
    pub device_name: Option<String>,
    #[serde(default)]
    pub device_type: Option<String>,
    #[serde(default = "default_interval")]
    pub telemetry_interval_sec: u64,
    #[serde(default = "default_auto_reconnect")]
    pub auto_reconnect: bool,
    #[serde(default = "default_true")]
    pub minimize_to_tray: bool,
    #[serde(default)]
    pub start_minimized: bool,
}

fn default_true() -> bool {
    true
}

fn default_interval() -> u64 {
    30
}

fn default_auto_reconnect() -> bool {
    true
}

impl Default for ClientConfig {
    fn default() -> Self {
        let detected_type = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else {
            "desktop"
        };

        Self {
            server_url: "http://127.0.0.1:8080".to_string(),
            ws_url: None,
            device_id: None,
            device_token: None,
            auth_token: None,
            device_name: None,
            device_type: Some(detected_type.to_string()),
            telemetry_interval_sec: 30,
            auto_reconnect: true,
            minimize_to_tray: true,
            start_minimized: false,
        }
    }
}

impl ClientConfig {
    pub fn config_dir() -> PathBuf {
        if let Some(config_base) = dirs::config_dir() {
            config_base.join("kusuri")
        } else {
            PathBuf::from(".kusuri")
        }
    }

    pub fn config_file_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_file_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<ClientConfig>(&content) {
                    Ok(mut cfg) => {
                        cfg.normalize_ws_url();
                        return cfg;
                    }
                    Err(e) => {
                        warn!("Gagal membaca config dari {}: {}. Menggunakan default.", path.display(), e);
                    }
                },
                Err(e) => {
                    warn!("Gagal membuka file config {}: {}. Menggunakan default.", path.display(), e);
                }
            }
        }

        let mut def = Self::default();
        def.normalize_ws_url();
        def
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let dir = Self::config_dir();
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }

        let path = Self::config_file_path();
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(&path, json)?;
        info!("Konfigurasi tersimpan ke: {}", path.display());
        Ok(())
    }

    pub fn normalize_ws_url(&mut self) {
        self.server_url = crate::api::normalize_server_url(&self.server_url);
        self.ws_url = Some(crate::api::derive_ws_url(&self.server_url));
    }

    pub fn is_paired(&self) -> bool {
        self.device_id.is_some() && self.device_token.as_ref().is_some_and(|t| !t.trim().is_empty())
    }

    pub fn clear_credentials(&mut self) {
        self.device_id = None;
        self.device_token = None;
        self.auth_token = None;
    }
}
