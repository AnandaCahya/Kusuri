use crate::api::ApiClient;
use crate::commands;
use crate::config::ClientConfig;
use crate::telemetry::collector::TelemetryCollector;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{interval, sleep};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};

pub struct DesktopClient {
    config: ClientConfig,
    api_client: ApiClient,
    collector: TelemetryCollector,
}

impl DesktopClient {
    pub fn new(config: ClientConfig) -> Self {
        let api_client = ApiClient::new(&config.server_url);
        let collector = TelemetryCollector::new();

        Self {
            config,
            api_client,
            collector,
        }
    }

    /// Menjalankan background service 24 jam dengan auto-reconnect dan periodic telemetry
    pub async fn run_forever(&mut self) {
        if !self.config.is_paired() {
            error!("Perangkat belum di-pair ke Kusuri Backend.");
            error!("Silakan jalankan: kusuri-desktop pair --code <6-DIGIT-CODE> --server <SERVER_URL>");
            return;
        }

        self.config.normalize_ws_url();

        let device_id = self.config.device_id.unwrap();
        let token = self
            .config
            .device_token
            .as_deref()
            .or(self.config.auth_token.as_deref())
            .unwrap()
            .to_string();

        let ws_endpoint = self.config.ws_url.as_deref().unwrap_or("ws://127.0.0.1:8080/ws");
        let ws_full_url = format!("{}?token={}", ws_endpoint, token);

        info!("============================================================");
        info!("  Kusuri Desktop Client 24/7 Service Memulai...");
        info!("  Device ID   : {}", device_id);
        info!("  Server HTTP : {}", self.config.server_url);
        info!("  Server WS   : {}", ws_endpoint);
        info!("  Interval    : {} detik", self.config.telemetry_interval_sec);
        info!("============================================================");

        let mut reconnect_attempt: u32 = 0;

        loop {
            info!("Menghubungkan ke WebSocket Kusuri (Percobaan #{})...", reconnect_attempt + 1);

            match connect_async(&ws_full_url).await {
                Ok((ws_stream, _response)) => {
                    info!("Berhasil terhubung ke Kusuri WebSocket! Status: ONLINE 24/7");
                    reconnect_attempt = 0;

                    let (mut ws_write, mut ws_read) = ws_stream.split();
                    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

                    // Task untuk mengirim pesan ke socket
                    let send_task = tokio::spawn(async move {
                        while let Some(msg) = rx.recv().await {
                            if ws_write.send(Message::Text(msg.into())).await.is_err() {
                                break;
                            }
                        }
                    });

                    // 1. Kirim initial telemetry secara instan begitu connect
                    self.send_telemetry_payloads(&tx);

                    // 2. Channel untuk mengontrol loop heartbeat & telemetry periodic
                    let mut ping_interval = interval(Duration::from_secs(25));
                    let mut telem_interval = interval(Duration::from_secs(self.config.telemetry_interval_sec));

                    // Abaikan tick pertama yang instan dari interval
                    ping_interval.tick().await;
                    telem_interval.tick().await;

                    let mut socket_alive = true;

                    while socket_alive {
                        tokio::select! {
                            // Heartbeat Ping setiap 25 detik untuk menjaga koneksi router/NAT
                            _ = ping_interval.tick() => {
                                let ping_msg = json!({ "type": "ping" }).to_string();
                                if tx.send(ping_msg).is_err() {
                                    warn!("Gagal mengirim Ping heartbeat, koneksi terputus.");
                                    socket_alive = false;
                                } else {
                                    debug!("Heartbeat Ping terkirim.");
                                }
                            }

                            // Telemetri berkala (default 30 detik)
                            _ = telem_interval.tick() => {
                                self.send_telemetry_payloads(&tx);
                            }

                            // Menerima pesan masuk dari server Kusuri
                            msg_opt = ws_read.next() => {
                                match msg_opt {
                                    Some(Ok(Message::Text(text))) => {
                                        self.handle_incoming_message(&text, &tx).await;
                                    }
                                    Some(Ok(Message::Ping(data))) => {
                                        // Auto pong
                                        let _ = tx.send(json!({ "type": "pong" }).to_string());
                                        debug!("Ping frame diterima, membalas pong (data len: {})", data.len());
                                    }
                                    Some(Ok(Message::Close(frame))) => {
                                        warn!("Server menutup koneksi WebSocket: {:?}", frame);
                                        socket_alive = false;
                                    }
                                    Some(Err(e)) => {
                                        error!("Error membaca dari WebSocket: {}", e);
                                        socket_alive = false;
                                    }
                                    None => {
                                        warn!("WebSocket stream berakhir.");
                                        socket_alive = false;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    // Bersihkan send task
                    let _ = send_task.abort();
                }
                Err(e) => {
                    error!("Gagal menghubungkan ke WebSocket: {}", e);

                    // HTTP Fallback: kirim telemetri via HTTP endpoint jika ada auth_token
                    self.try_http_fallback_telemetry().await;
                }
            }

            if !self.config.auto_reconnect {
                info!("Auto-reconnect dinonaktifkan. Menghentikan client.");
                break;
            }

            reconnect_attempt += 1;
            let delay_secs = calculate_backoff(reconnect_attempt);
            info!(
                "Koneksi terputus. Menunggu {} detik sebelum mencoba terhubung kembali (Percobaan #{})...",
                delay_secs, reconnect_attempt
            );
            sleep(Duration::from_secs(delay_secs)).await;
        }
    }

    fn send_telemetry_payloads(&mut self, tx: &mpsc::UnboundedSender<String>) {
        let snap = self.collector.collect_snapshot();
        let hw_req = snap.to_hardware_telemetry_request();
        let live_data = snap.to_telemetry_data();

        // 1. Hardware Telemetry Event (Database & Specs enrichment)
        let hw_event = json!({
            "type": "hardware_telemetry",
            "payload": hw_req
        });

        // 2. Telemetry Live Event (Live dashboard broadcast)
        let live_event = json!({
            "type": "telemetry",
            "payload": live_data
        });

        if tx.send(hw_event.to_string()).is_ok() && tx.send(live_event.to_string()).is_ok() {
            info!(
                "Telemetri terkirim: Model='{}' | RAM={}/{}MB ({}%) | CPU={:.1}% | Battery={}% (Charging: {})",
                snap.system.raw_model,
                snap.memory.ram_used_mb,
                snap.memory.ram_total_mb,
                snap.memory.ram_usage_percent as u32,
                snap.cpu.global_usage_percent,
                snap.power.battery_percent.unwrap_or(100.0) as u32,
                snap.power.is_charging
            );
        }
    }

    async fn handle_incoming_message(&mut self, text: &str, tx: &mpsc::UnboundedSender<String>) {
        let parsed: Result<Value, _> = serde_json::from_str(text);
        let val = match parsed {
            Ok(v) => v,
            Err(e) => {
                warn!("Menerima format JSON tidak valid: {}", e);
                return;
            }
        };

        let msg_type = val.get("type").and_then(|t| t.as_str()).unwrap_or("");
        let payload = val.get("payload");

        match msg_type {
            "pong" => {
                debug!("Pong diterima dari server.");
            }
            "ping" => {
                let _ = tx.send(json!({ "type": "pong" }).to_string());
            }
            "execute_command" => {
                if let Some(p) = payload {
                    let cmd_id = p.get("command_id").and_then(|c| c.as_str()).unwrap_or("");
                    let command = p.get("command").and_then(|c| c.as_str()).unwrap_or("");
                    let cmd_payload = p.get("payload").cloned();

                    let (status, response) =
                        commands::execute_command(command, cmd_payload, &mut self.collector);

                    let result_msg = json!({
                        "type": "command_result",
                        "payload": {
                            "command_id": cmd_id,
                            "status": status,
                            "response": response
                        }
                    });

                    let _ = tx.send(result_msg.to_string());
                }
            }
            "clipboard_updated" => {
                if let Some(p) = payload {
                    let content = p.get("content").and_then(|c| c.as_str()).unwrap_or("");
                    let src_device = p
                        .get("source_device_name")
                        .and_then(|d| d.as_str())
                        .unwrap_or("Remote Device");
                    info!("Clipboard disinkronkan dari [{}]: {} karakter", src_device, content.len());
                }
            }
            "notification_ingested" => {
                if let Some(p) = payload {
                    let app = p.get("app_title").or_else(|| p.get("package_or_app_name")).and_then(|v| v.as_str()).unwrap_or("App");
                    let title = p.get("title").and_then(|v| v.as_str()).unwrap_or("");
                    let content = p.get("content").and_then(|v| v.as_str()).unwrap_or("");
                    let dev = p.get("device_name").and_then(|v| v.as_str()).unwrap_or("Device");
                    info!("🔔 [Notifikasi Masuk dari {} ({})]: {} - {}", dev, app, title, content);
                }
            }
            "device_connected" => {
                if let Some(p) = payload {
                    let name = p.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown");
                    let dtype = p.get("device_type").and_then(|t| t.as_str()).unwrap_or("");
                    info!("Perangkat terhubung di akun Anda: {} ({})", name, dtype);
                }
            }
            "device_disconnected" => {
                if let Some(p) = payload {
                    let id = p.get("device_id").and_then(|i| i.as_str()).unwrap_or("");
                    debug!("Perangkat terputus: {}", id);
                }
            }
            _ => {
                debug!("Event WebSocket lain diterima: {}", msg_type);
            }
        }
    }

    async fn try_http_fallback_telemetry(&mut self) {
        if let (Some(device_id), Some(auth_token)) = (self.config.device_id, &self.config.auth_token) {
            info!("Mencoba sinkronisasi telemetri via HTTP API fallback...");
            let snap = self.collector.collect_snapshot();
            let hw_req = snap.to_hardware_telemetry_request();

            match self
                .api_client
                .send_hardware_telemetry(device_id, auth_token, &hw_req)
                .await
            {
                Ok(_) => {
                    info!("Telemetri berhasil disinkronkan via HTTP fallback.");
                }
                Err(e) => {
                    warn!("HTTP Fallback gagal: {}", e);
                }
            }
        }
    }
}

/// Menghitung delay exponential backoff: 2s, 3s, 5s, 8s, 12s, 18s, 27s, max 60s
fn calculate_backoff(attempt: u32) -> u64 {
    let base = 2.0 * 1.5f64.powi((attempt.saturating_sub(1)).min(8) as i32);
    base.round().clamp(2.0, 60.0) as u64
}
