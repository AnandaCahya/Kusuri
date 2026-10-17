mod api;
mod client;
mod commands;
mod config;
mod gui;
mod telemetry;

use api::ApiClient;
use clap::{Parser, Subcommand};
use client::DesktopClient;
use config::ClientConfig;
use telemetry::collector::TelemetryCollector;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(name = "kusuri-desktop")]
#[command(author = "Kusuri Contributors")]
#[command(version = "0.1.0")]
#[command(about = "24/7 Desktop Telemetry & Sync Client untuk Windows dan Linux", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Jalankan GUI dalam kondisi terminimalisasi di background / system tray
    #[arg(short, long, global = true)]
    minimized: bool,

    /// Tingkat log verbose (debug)
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Buka antarmuka grafis (GUI) desktop interaktif
    Gui {
        /// Jalankan GUI langsung terminimalisasi ke System Tray
        #[arg(short, long)]
        minimized: bool,
    },

    /// Pasangkan perangkat ini dengan akun Kusuri menggunakan kode 6-digit
    Pair {
        /// 6-digit kode pairing dari dashboard Kusuri Web
        #[arg(short, long)]
        code: String,

        /// URL Server Kusuri Backend (contoh: http://127.0.0.1:8080)
        #[arg(short, long, default_value = "http://127.0.0.1:8080")]
        server: String,

        /// Nama kustom untuk perangkat ini (opsional)
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Jalankan service telemetri 24/7 di latar belakang
    Run {
        /// Override URL Server Kusuri Backend
        #[arg(short, long)]
        server: Option<String>,

        /// Interval pengiriman telemetri dalam detik (default: 30)
        #[arg(short, long)]
        interval: Option<u64>,
    },

    /// Periksa status konfigurasi dan koneksi ke backend
    Status,

    /// Tampilkan metrik telemetri & spesifikasi lengkap perangkat saat ini
    Telemetry {
        /// Tampilkan dalam format JSON mentah
        #[arg(short, long)]
        json: bool,
    },

    /// Hapus kredensial pairing lokal dari perangkat ini
    Unpair,
}

fn main() {
    let cli = Cli::parse();

    let log_level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap_or_default();

    match cli.command {
        None => {
            let start_minimized = cli.minimized;
            if let Err(e) = gui::run_gui(start_minimized) {
                eprintln!("Gagal membuka antarmuka grafis GUI Kusuri: {}", e);
            }
        }
        Some(Commands::Gui { minimized }) => {
            let start_minimized = minimized || cli.minimized;
            if let Err(e) = gui::run_gui(start_minimized) {
                eprintln!("Gagal membuka antarmuka grafis GUI Kusuri: {}", e);
            }
        }
        Some(cmd) => {
            let rt = tokio::runtime::Runtime::new().expect("Gagal menginisialisasi Tokio runtime");
            rt.block_on(async {
                match cmd {
                    Commands::Pair { code, server, name } => {
                        handle_pair(&code, &server, name).await;
                    }
                    Commands::Run { server, interval } => {
                        handle_run(server, interval).await;
                    }
                    Commands::Status => {
                        handle_status().await;
                    }
                    Commands::Telemetry { json } => {
                        handle_telemetry(json).await;
                    }
                    Commands::Unpair => {
                        handle_unpair().await;
                    }
                    Commands::Gui { .. } => unreachable!(),
                }
            });
        }
    }
}

async fn handle_pair(code: &str, server_url: &str, custom_name: Option<String>) {
    info!("Memulai proses pairing dengan server: {}", server_url);

    let mut collector = TelemetryCollector::new();
    let snap = collector.collect_snapshot();

    let device_name = custom_name.unwrap_or_else(|| snap.system.raw_model.clone());
    let detected_type = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "desktop"
    };

    let api = ApiClient::new(server_url);
    match api
        .pair_device(
            code,
            Some(device_name.clone()),
            Some(detected_type.to_string()),
            Some(snap.system.raw_model.clone()),
        )
        .await
    {
        Ok(res) => {
            info!("============================================================");
            info!("  PAIRING BERHASIL! 🎉");
            info!("  Device ID      : {}", res.device_id);
            info!("  Device Name    : {}", res.device_name);
            info!("  Device Type    : {}", res.device_type);
            if let Some(mkt) = res.marketing_name {
                info!("  Marketing Name : {}", mkt);
            }
            if let Some(chip) = res.chipset {
                info!("  Chipset        : {}", chip);
            }
            info!("============================================================");

            let mut config = ClientConfig::load();
            config.server_url = server_url.trim_end_matches('/').to_string();
            config.device_id = Some(res.device_id);
            config.device_token = Some(res.device_token);
            config.auth_token = res.auth_token;
            config.device_name = Some(res.device_name);
            config.device_type = Some(res.device_type);
            config.normalize_ws_url();

            if let Err(e) = config.save() {
                error!("Gagal menyimpan konfigurasi: {}", e);
            } else {
                info!("Konfigurasi berhasil disimpan.");
                info!("Untuk menjalankan client 24/7, gunakan perintah:");
                info!("  kusuri-desktop run");
            }
        }
        Err(e) => {
            error!("Pairing gagal: {}", e);
            error!("Pastikan server berjalan dan kode 6-digit belum kedaluwarsa.");
        }
    }
}

async fn handle_run(server_override: Option<String>, interval_override: Option<u64>) {
    let mut config = ClientConfig::load();

    if let Some(srv) = server_override {
        config.server_url = srv;
        config.ws_url = None;
        config.normalize_ws_url();
    }
    if let Some(inv) = interval_override {
        config.telemetry_interval_sec = inv;
    }

    let mut client = DesktopClient::new(config);

    // Setup graceful shutdown pada sinyal Ctrl+C
    tokio::select! {
        _ = client.run_forever() => {}
        _ = tokio::signal::ctrl_c() => {
            info!("\nSinyal shutdown (Ctrl+C) diterima. Menutup Kusuri Desktop Client...");
        }
    }
}

async fn handle_status() {
    let config = ClientConfig::load();
    let config_path = ClientConfig::config_file_path();

    println!("============================================================");
    println!("  Kusuri Desktop Client Status");
    println!("============================================================");
    println!("  Config File : {}", config_path.display());
    println!("  Server URL  : {}", config.server_url);
    println!("  WS URL      : {}", config.ws_url.as_deref().unwrap_or("auto"));
    println!("  Device Name : {}", config.device_name.as_deref().unwrap_or("Belum di-set"));
    println!("  Device Type : {}", config.device_type.as_deref().unwrap_or("desktop"));
    println!("  Interval    : {} detik", config.telemetry_interval_sec);

    if let Some(id) = config.device_id {
        println!("  Device ID   : {}", id);
        println!("  Status      : TERPASANG (Paired) ✅");
    } else {
        println!("  Status      : BELUM TERPASANG (Unpaired) ❌");
        println!("\n  Gunakan 'kusuri-desktop pair --code <6-digit>' untuk menghubungkan.");
    }

    println!("------------------------------------------------------------");
    let api = ApiClient::new(&config.server_url);
    print!("  Koneksi Server: ");
    match api.check_health().await {
        Ok(true) => println!("ONLINE ✅"),
        Ok(false) => println!("RESPON TIDAK VALID ⚠️"),
        Err(e) => println!("OFFLINE / TIDAK TERJANGKAU ❌ ({})", e),
    }
    println!("============================================================");
}

async fn handle_telemetry(as_json: bool) {
    let mut collector = TelemetryCollector::new();
    let snap = collector.collect_snapshot();

    if as_json {
        let json_str = serde_json::to_string_pretty(&snap).unwrap_or_default();
        println!("{}", json_str);
        return;
    }

    println!("============================================================");
    println!("  KUSURI TELEMETRY & HARDWARE SNAPSHOT");
    println!("============================================================");
    println!("🖥️  SISTEM & PERANGKAT:");
    println!("   Model       : {}", snap.system.raw_model);
    println!("   Manufacturer: {}", snap.system.manufacturer);
    println!("   Hostname    : {}", snap.system.hostname);
    println!("   OS          : {} (Versi: {})", snap.system.os_name, snap.system.os_version);
    println!("   Kernel      : {}", snap.system.kernel_version);
    println!("   Arsitektur  : {}", snap.system.architecture);
    println!("   Uptime      : {}", snap.system.uptime_formatted);

    println!("\n⚡ PROSESOR (CPU):");
    println!("   Model       : {}", snap.cpu.brand);
    println!("   Vendor      : {}", snap.cpu.vendor_id);
    println!("   Cores       : {} Fisik / {} Logikal", snap.cpu.physical_cores, snap.cpu.logical_cores);
    println!("   Frekuensi   : {} MHz", snap.cpu.frequency_mhz);
    println!("   Total Usage : {:.1}%", snap.cpu.global_usage_percent);

    println!("\n🧠 MEMORI (RAM):");
    println!(
        "   Total / Used: {} MB / {} MB ({:.1}%)",
        snap.memory.ram_total_mb, snap.memory.ram_used_mb, snap.memory.ram_usage_percent
    );
    println!("   Free / Avail: {} MB / {} MB", snap.memory.ram_free_mb, snap.memory.ram_available_mb);
    println!("   Swap        : {} MB / {} MB Used", snap.memory.swap_total_mb, snap.memory.swap_used_mb);

    println!("\n💾 PENYIMPANAN (DISKS):");
    println!("   Total / Used: {} MB / {} MB", snap.storage.total_mb, snap.storage.used_mb);
    for disk in &snap.storage.disks {
        println!(
            "   - {} ({}) [{}] : {} MB / {} MB (Free: {} MB)",
            disk.mount_point, disk.name, disk.file_system, disk.used_mb, disk.total_mb, disk.free_mb
        );
    }

    println!("\n🔋 DAYA & BATERAI:");
    println!("   Ada Baterai : {}", if snap.power.has_battery { "Ya" } else { "Tidak (Desktop PC)" });
    if let Some(pct) = snap.power.battery_percent {
        println!("   Persentase  : {:.0}%", pct);
    }
    println!("   Charging    : {}", if snap.power.is_charging { "Ya ⚡" } else { "Tidak" });
    println!("   Sumber Daya : {}", snap.power.power_source);
    println!("   Status/Kesehatan: {}", snap.power.battery_health);

    println!("\n📺 TAMPILAN (DISPLAY):");
    println!("   Resolusi    : {}", snap.display.resolution);
    println!("   Refresh Rate: {:.1} Hz", snap.display.refresh_rate_hz);

    println!("\n🌐 JARINGAN (NETWORK):");
    println!("   IP Utama    : {}", snap.network.primary_ip.as_deref().unwrap_or("N/A"));
    println!("   Jenis       : {}", snap.network.network_type);
    for iface in &snap.network.interfaces {
        if !iface.ip_addresses.is_empty() {
            println!("   - {}: {:?}", iface.name, iface.ip_addresses);
        }
    }

    if let Some(temp) = snap.sensors.cpu_temperature_c {
        println!("\n🌡️ SENSOR:");
        println!("   Suhu CPU    : {:.1} °C", temp);
    }

    println!("\n⚙️  PROSES:");
    println!("   Running Procs: {}", snap.process.running_processes_count);
    println!("   Client Memory: {} MB", snap.process.client_ram_used_mb);
    println!("============================================================");
}

async fn handle_unpair() {
    let mut config = ClientConfig::load();
    if !config.is_paired() {
        info!("Perangkat memang belum di-pair.");
        return;
    }

    config.clear_credentials();
    if let Err(e) = config.save() {
        error!("Gagal menyimpan konfigurasi: {}", e);
    } else {
        info!("Kredensial pairing berhasil dihapus.");
        info!("Perangkat sekarang berstatus unpaired.");
    }
}
