use crate::api::{self, ApiClient};
use crate::client::DesktopClient;
use crate::config::ClientConfig;
use crate::telemetry::collector::TelemetryCollector;
use crate::telemetry::DesktopTelemetrySnapshot;
use chrono::Utc;
use eframe::egui::{self, Color32, CornerRadius, Margin, ProgressBar, RichText};
use muda::{Menu, MenuItem, PredefinedMenuItem};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;
use tracing::{error, info};
use tray_icon::menu::MenuEvent;
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

#[derive(PartialEq, Clone, Copy)]
enum GuiTab {
    Dashboard,
    Hardware,
    Notifications,
    Pairing,
    Logs,
    Settings,
}

#[derive(Clone)]
struct LogMessage {
    timestamp: String,
    message: String,
    is_error: bool,
}

pub struct KusuriGuiApp {
    current_tab: GuiTab,
    config: ClientConfig,
    telemetry_cache: Arc<Mutex<Option<DesktopTelemetrySnapshot>>>,
    logs: Arc<Mutex<Vec<LogMessage>>>,
    is_service_running: Arc<AtomicBool>,
    is_ws_connected: Arc<AtomicBool>,

    // Form inputs
    input_server_url: String,
    input_pairing_code: String,
    input_device_name: String,
    pairing_status_msg: Option<(String, bool)>,
    is_pairing_busy: bool,

    // Runtime handle for async background tasks
    rt: Arc<Runtime>,

    // System Tray integration
    #[allow(dead_code)]
    tray_icon: Option<TrayIcon>,
    tray_open_id: muda::MenuId,
    tray_toggle_id: muda::MenuId,
    tray_quit_id: muda::MenuId,
    window_visible: bool,
    request_quit: bool,
}

fn create_pill_icon() -> Icon {
    let width = 32u32;
    let height = 32u32;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let cx = x as f32 - 15.5;
            let cy = y as f32 - 15.5;
            let dist = (cx * cx + cy * cy).sqrt();

            if dist <= 14.0 {
                // Diagonal split for pill shape: top-left cyan, bottom-right indigo
                if x + y <= 31 {
                    rgba.extend_from_slice(&[56, 189, 248, 255]); // Sky 400
                } else {
                    rgba.extend_from_slice(&[99, 102, 241, 255]); // Indigo 500
                }
            } else if dist <= 15.5 {
                // Anti-aliased border
                rgba.extend_from_slice(&[30, 41, 59, 200]);
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]); // Transparent
            }
        }
    }

    Icon::from_rgba(rgba, width, height).unwrap_or_else(|_| {
        let fallback = vec![56, 189, 248, 255];
        Icon::from_rgba(fallback, 1, 1).unwrap()
    })
}

impl KusuriGuiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = ClientConfig::load();
        let rt = Arc::new(Runtime::new().expect("Failed creating Tokio runtime for GUI"));

        let telemetry_cache = Arc::new(Mutex::new(None));
        let logs = Arc::new(Mutex::new(Vec::new()));
        let is_service_running = Arc::new(AtomicBool::new(false));
        let is_ws_connected = Arc::new(AtomicBool::new(false));

        // Initial snapshot
        {
            let mut collector = TelemetryCollector::new();
            let snap = collector.collect_snapshot();
            *telemetry_cache.lock().unwrap() = Some(snap);
        }

        let input_server_url = if config.server_url.trim().is_empty() {
            "http://127.0.0.1:8080".to_string()
        } else {
            config.server_url.clone()
        };
        let input_device_name = config.device_name.clone().unwrap_or_default();

        // Bangun Context Menu untuk Tray Icon
        let tray_menu = Menu::new();
        let item_open = MenuItem::new("Buka Dashboard Kusuri", true, None);
        let item_toggle = MenuItem::new("Aktifkan / Jeda 24/7 Sync", true, None);
        let item_sep = PredefinedMenuItem::separator();
        let item_quit = MenuItem::new("Keluar dari Kusuri", true, None);

        let tray_open_id = item_open.id().clone();
        let tray_toggle_id = item_toggle.id().clone();
        let tray_quit_id = item_quit.id().clone();

        let _ = tray_menu.append(&item_open);
        let _ = tray_menu.append(&item_toggle);
        let _ = tray_menu.append(&item_sep);
        let _ = tray_menu.append(&item_quit);

        // Buat tray icon
        let tray_icon = match TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("Kusuri Desktop Client • 24/7 Sync")
            .with_icon(create_pill_icon())
            .build()
        {
            Ok(ti) => {
                info!("Tray icon berhasil diinisialisasi");
                Some(ti)
            }
            Err(e) => {
                error!("Gagal membuat TrayIcon: {}", e);
                None
            }
        };

        let start_minimized = config.start_minimized;

        let app = Self {
            current_tab: GuiTab::Dashboard,
            config,
            telemetry_cache: telemetry_cache.clone(),
            logs: logs.clone(),
            is_service_running: is_service_running.clone(),
            is_ws_connected: is_ws_connected.clone(),

            input_server_url,
            input_pairing_code: String::new(),
            input_device_name,
            pairing_status_msg: None,
            is_pairing_busy: false,
            rt: rt.clone(),

            tray_icon,
            tray_open_id,
            tray_toggle_id,
            tray_quit_id,
            window_visible: !start_minimized,
            request_quit: false,
        };

        // Jika sudah di-pair, otomatis mulai 24/7 background telemetry sync
        if app.config.is_paired() {
            app.start_background_service();
        }

        // Spawn timer pengumpul telemetri lokal setiap 2 detik untuk animasi UI
        let telem_cache_clone = telemetry_cache.clone();
        std::thread::spawn(move || {
            let mut collector = TelemetryCollector::new();
            loop {
                std::thread::sleep(Duration::from_secs(2));
                let snap = collector.collect_snapshot();
                if let Ok(mut lock) = telem_cache_clone.lock() {
                    *lock = Some(snap);
                }
            }
        });

        app
    }

    fn add_log(&self, msg: &str, is_error: bool) {
        let entry = LogMessage {
            timestamp: Utc::now().format("%H:%M:%S").to_string(),
            message: msg.to_string(),
            is_error,
        };
        if let Ok(mut l) = self.logs.lock() {
            if l.len() > 300 {
                l.remove(0);
            }
            l.push(entry);
        }
    }

    fn start_background_service(&self) {
        if self.is_service_running.load(Ordering::SeqCst) {
            return;
        }

        self.is_service_running.store(true, Ordering::SeqCst);
        let mut cfg = self.config.clone();
        cfg.normalize_ws_url();

        let running_flag = self.is_service_running.clone();
        let ws_flag = self.is_ws_connected.clone();
        let logs_clone = self.logs.clone();
        let rt = self.rt.clone();

        std::thread::spawn(move || {
            rt.block_on(async move {
                let mut client = DesktopClient::new(cfg);
                ws_flag.store(true, Ordering::SeqCst);

                {
                    if let Ok(mut l) = logs_clone.lock() {
                        l.push(LogMessage {
                            timestamp: Utc::now().format("%H:%M:%S").to_string(),
                            message: "Koneksi 24/7 aktif: Menghubungkan WebSocket dan telemetri periodik...".to_string(),
                            is_error: false,
                        });
                    }
                }

                tokio::select! {
                    _ = client.run_forever() => {},
                    _ = async {
                        while running_flag.load(Ordering::SeqCst) {
                            tokio::time::sleep(Duration::from_millis(500)).await;
                        }
                    } => {}
                }

                ws_flag.store(false, Ordering::SeqCst);
                running_flag.store(false, Ordering::SeqCst);
            });
        });

        self.add_log("Service background 24/7 diaktifkan.", false);
    }

    fn stop_background_service(&self) {
        self.is_service_running.store(false, Ordering::SeqCst);
        self.is_ws_connected.store(false, Ordering::SeqCst);
        self.add_log("Service background dihentikan sementara.", false);
    }
}

impl eframe::App for KusuriGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Polling event klik pada tray icon
        if let Ok(tray_event) = TrayIconEvent::receiver().try_recv() {
            if let TrayIconEvent::Click { button, button_state, .. } = tray_event {
                if button == MouseButton::Left && button_state == MouseButtonState::Up {
                    self.window_visible = !self.window_visible;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(self.window_visible));
                    if self.window_visible {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    }
                }
            }
        }

        // Polling event menu dari tray icon
        if let Ok(menu_event) = MenuEvent::receiver().try_recv() {
            if menu_event.id == self.tray_open_id {
                self.window_visible = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            } else if menu_event.id == self.tray_toggle_id {
                if self.is_service_running.load(Ordering::SeqCst) {
                    self.stop_background_service();
                } else {
                    self.start_background_service();
                }
            } else if menu_event.id == self.tray_quit_id {
                self.request_quit = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        // Tangani event penutupan jendela [X]
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.config.minimize_to_tray && !self.request_quit {
                // Jangan keluar dari proses! Batalkan penutupan jendela dan sembunyikan ke system tray
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.window_visible = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                self.add_log("Jendela diminimalkan ke System Tray. Service 24/7 tetap berjalan.", false);
            }
        }

        // Jika jendela tersembunyi, minta repaint setiap 500ms agar event loop tetap memproses tray clicks
        if !self.window_visible {
            ctx.request_repaint_after(Duration::from_millis(200));
            return;
        }

        // Skala DPI zoom 1.15 agar tampilan sangat lega dan tajam
        ctx.set_pixels_per_point(1.15);

        // Dark theme styling modern khas Kusuri
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(Color32::from_rgb(241, 245, 249)); // Slate 100
        visuals.panel_fill = Color32::from_rgb(15, 23, 42); // Slate 900
        visuals.window_fill = Color32::from_rgb(30, 41, 59); // Slate 800
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(30, 41, 59);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(51, 65, 85);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(71, 85, 105);
        visuals.widgets.active.bg_fill = Color32::from_rgb(14, 165, 233);
        ctx.set_visuals(visuals);

        // Header Panel Atas yang Luas dan Elegan
        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::NONE.fill(Color32::from_rgb(15, 23, 42)).inner_margin(Margin::symmetric(24, 16)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("💊").size(34.0));
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.heading(RichText::new("Kusuri Desktop Client").color(Color32::from_rgb(56, 189, 248)).size(22.0).strong());
                        ui.label(RichText::new("24/7 Native Telemetry & Multi-Device Sync • Windows & Linux").size(12.5).color(Color32::from_rgb(148, 163, 184)));
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Status indicator badge
                        let (badge_text, badge_bg, badge_fg) = if self.config.is_paired() {
                            if self.is_ws_connected.load(Ordering::SeqCst) {
                                ("● ONLINE 24/7", Color32::from_rgb(6, 78, 59), Color32::from_rgb(52, 211, 153))
                            } else if self.is_service_running.load(Ordering::SeqCst) {
                                ("○ MENGHUBUNGKAN...", Color32::from_rgb(120, 53, 15), Color32::from_rgb(251, 191, 36))
                            } else {
                                ("⏸ STANDBY (PAUSED)", Color32::from_rgb(51, 65, 85), Color32::from_rgb(203, 213, 225))
                            }
                        } else {
                            ("✕ BELUM DI-PAIR", Color32::from_rgb(127, 29, 29), Color32::from_rgb(248, 113, 113))
                        };

                        egui::Frame::NONE
                            .fill(badge_bg)
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(Margin::symmetric(14, 8))
                            .show(ui, |ui| {
                                ui.label(RichText::new(badge_text).color(badge_fg).strong().size(13.0));
                            });

                        ui.add_space(8.0);

                        // Toggle Service Button
                        if self.config.is_paired() {
                            if self.is_service_running.load(Ordering::SeqCst) {
                                if ui.button(RichText::new("⏸ Jeda Service").size(13.0).color(Color32::from_rgb(251, 146, 60))).clicked() {
                                    self.stop_background_service();
                                }
                            } else {
                                if ui.button(RichText::new("▶ Aktifkan 24/7").size(13.0).color(Color32::from_rgb(52, 211, 153))).clicked() {
                                    self.start_background_service();
                                }
                            }
                        }
                    });
                });
            });

        // Tab Navigation Bar yang Lega
        egui::TopBottomPanel::top("tab_bar")
            .frame(egui::Frame::NONE.fill(Color32::from_rgb(30, 41, 59)).inner_margin(Margin::symmetric(24, 10)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;

                    let tabs = [
                        (GuiTab::Dashboard, "📊 Dashboard"),
                        (GuiTab::Hardware, "🖥️ Spesifikasi Hardware"),
                        (GuiTab::Notifications, "🔔 Notifikasi"),
                        (GuiTab::Pairing, "🔗 Pemasangan (Pairing)"),
                        (GuiTab::Logs, "📜 Log Aktivitas"),
                        (GuiTab::Settings, "⚙️ Pengaturan"),
                    ];

                    for (tab, label) in tabs {
                        let selected = self.current_tab == tab;
                        let text = RichText::new(label).size(13.5).strong();
                        let text = if selected {
                            text.color(Color32::from_rgb(56, 189, 248))
                        } else {
                            text.color(Color32::from_rgb(148, 163, 184))
                        };

                        if ui.selectable_label(selected, text).clicked() {
                            self.current_tab = tab;
                        }
                    }
                });
            });

        // Bottom Footer Bar
        egui::TopBottomPanel::bottom("bottom_nav")
            .frame(egui::Frame::NONE.fill(Color32::from_rgb(15, 23, 42)).inner_margin(Margin::symmetric(24, 10)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let dev_name = self.config.device_name.as_deref().unwrap_or("Kusuri Desktop");
                    let dev_id_str = self.config.device_id.map(|i| i.to_string()).unwrap_or_else(|| "Belum Di-pair".to_string());
                    ui.label(RichText::new(format!("Perangkat: {}  •  ID: {}  •  Server: {}", dev_name, &dev_id_str[..dev_id_str.len().min(12)], self.config.server_url)).size(11.5).color(Color32::from_rgb(100, 116, 139)));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("Kusuri Desktop Client v0.1.0 • 24/7 Engine").size(11.5).color(Color32::from_rgb(100, 116, 139)));
                    });
                });
            });

        // Central Content Area
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::from_rgb(15, 23, 42)).inner_margin(Margin::symmetric(24, 18)))
            .show(ctx, |ui| {
                let snap_opt = self.telemetry_cache.lock().unwrap().clone();

                match self.current_tab {
                    GuiTab::Dashboard => {
                        self.render_dashboard(ui, snap_opt.as_ref());
                    }
                    GuiTab::Hardware => {
                        self.render_hardware_specs(ui, snap_opt.as_ref());
                    }
                    GuiTab::Notifications => {
                        self.render_notifications_tab(ui);
                    }
                    GuiTab::Pairing => {
                        self.render_pairing_tab(ui, snap_opt.as_ref());
                    }
                    GuiTab::Logs => {
                        self.render_logs_tab(ui);
                    }
                    GuiTab::Settings => {
                        self.render_settings_tab(ui);
                    }
                }
            });

        // Refresh frame rate
        ctx.request_repaint_after(Duration::from_millis(500));
    }
}

impl KusuriGuiApp {
    fn render_dashboard(&mut self, ui: &mut egui::Ui, snap_opt: Option<&DesktopTelemetrySnapshot>) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Some(snap) = snap_opt {
                // 1. Top 4 Metric Cards (Lega & Responsif)
                ui.columns(4, |cols| {
                    // Card 1: Uptime & OS
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(30, 41, 59))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::same(16))
                        .show(&mut cols[0], |ui| {
                            ui.label(RichText::new("⏱️ UPTIME SISTEM").size(11.5).color(Color32::from_rgb(148, 163, 184)).strong());
                            ui.add_space(4.0);
                            ui.heading(RichText::new(&snap.system.uptime_formatted).color(Color32::WHITE).size(18.0));
                            ui.add_space(2.0);
                            ui.label(RichText::new(format!("{} (Kernel {})", snap.system.os_name, snap.system.kernel_version)).size(11.0).color(Color32::from_rgb(148, 163, 184)));
                        });

                    // Card 2: CPU Global
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(30, 41, 59))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::same(16))
                        .show(&mut cols[1], |ui| {
                            ui.label(RichText::new("⚡ BEBAN PROSESOR").size(11.5).color(Color32::from_rgb(148, 163, 184)).strong());
                            ui.add_space(4.0);
                            ui.heading(RichText::new(format!("{:.1}%", snap.cpu.global_usage_percent)).color(Color32::from_rgb(56, 189, 248)).size(20.0));
                            ui.add_space(2.0);
                            ui.label(RichText::new(format!("{} Cores • {} MHz", snap.cpu.logical_cores, snap.cpu.frequency_mhz)).size(11.0).color(Color32::from_rgb(148, 163, 184)));
                        });

                    // Card 3: RAM
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(30, 41, 59))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::same(16))
                        .show(&mut cols[2], |ui| {
                            ui.label(RichText::new("🧠 MEMORI TERPAKAI").size(11.5).color(Color32::from_rgb(148, 163, 184)).strong());
                            ui.add_space(4.0);
                            let used_gb = snap.memory.ram_used_mb as f32 / 1024.0;
                            let total_gb = snap.memory.ram_total_mb as f32 / 1024.0;
                            ui.heading(RichText::new(format!("{:.1} / {:.1} GB", used_gb, total_gb)).color(Color32::from_rgb(192, 132, 252)).size(18.0));
                            ui.add_space(2.0);
                            ui.label(RichText::new(format!("{:.1}% Terpakai", snap.memory.ram_usage_percent)).size(11.0).color(Color32::from_rgb(148, 163, 184)));
                        });

                    // Card 4: Daya & Jaringan
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(30, 41, 59))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::same(16))
                        .show(&mut cols[3], |ui| {
                            ui.label(RichText::new("🔋 DAYA & JARINGAN").size(11.5).color(Color32::from_rgb(148, 163, 184)).strong());
                            ui.add_space(4.0);
                            let pwr_text = if snap.power.has_battery {
                                format!("{:.0}% {}", snap.power.battery_percent.unwrap_or(100.0), if snap.power.is_charging { "⚡" } else { "" })
                            } else {
                                "AC Desktop ⚡".to_string()
                            };
                            ui.heading(RichText::new(pwr_text).color(Color32::from_rgb(52, 211, 153)).size(18.0));
                            ui.add_space(2.0);
                            let ip_str = snap.network.primary_ip.as_deref().unwrap_or("Offline");
                            ui.label(RichText::new(format!("IP: {} ({})", ip_str, snap.network.network_type)).size(11.0).color(Color32::from_rgb(148, 163, 184)));
                        });
                });

                ui.add_space(20.0);

                // 2. Resource Meters Detail Card
                egui::Frame::NONE
                    .fill(Color32::from_rgb(30, 41, 59))
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(Margin::same(20))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("📊 MONITOR SUMBER DAYA SISTEM REALTIME").strong().size(15.0).color(Color32::from_rgb(56, 189, 248)));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new(format!("Model: {}", snap.system.raw_model)).size(12.0).color(Color32::from_rgb(148, 163, 184)));
                            });
                        });
                        ui.add_space(14.0);

                        // CPU Progress Bar (Tinggi 22px)
                        let cpu_ratio = (snap.cpu.global_usage_percent / 100.0).clamp(0.0, 1.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("⚡ Beban CPU (Total):").size(13.5).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new(format!("{:.1}%", snap.cpu.global_usage_percent)).strong().size(13.5));
                            });
                        });
                        ui.add_space(4.0);
                        ui.add(ProgressBar::new(cpu_ratio).fill(Color32::from_rgb(14, 165, 233)).desired_height(20.0));
                        ui.label(RichText::new(format!("Processor: {}", snap.cpu.brand)).size(11.5).color(Color32::from_rgb(148, 163, 184)));

                        ui.add_space(16.0);

                        // RAM Progress Bar
                        let ram_ratio = (snap.memory.ram_usage_percent / 100.0).clamp(0.0, 1.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🧠 Memori RAM:").size(13.5).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new(format!("{} MB / {} MB ({:.1}%)", snap.memory.ram_used_mb, snap.memory.ram_total_mb, snap.memory.ram_usage_percent)).strong().size(13.5));
                            });
                        });
                        ui.add_space(4.0);
                        ui.add(ProgressBar::new(ram_ratio).fill(Color32::from_rgb(168, 85, 247)).desired_height(20.0));
                        ui.label(RichText::new(format!("Tersedia Bebas: {} MB  •  Swap Terpakai: {} MB / {} MB", snap.memory.ram_available_mb, snap.memory.swap_used_mb, snap.memory.swap_total_mb)).size(11.5).color(Color32::from_rgb(148, 163, 184)));

                        ui.add_space(16.0);

                        // Storage Partitions
                        ui.label(RichText::new("💾 Partisi & Drive Penyimpanan:").size(13.5).strong());
                        ui.add_space(4.0);
                        for disk in &snap.storage.disks {
                            let total = disk.total_mb.max(1);
                            let ratio = (disk.used_mb as f32 / total as f32).clamp(0.0, 1.0);
                            let used_gb = disk.used_mb as f32 / 1024.0;
                            let total_gb = disk.total_mb as f32 / 1024.0;
                            let free_gb = disk.free_mb as f32 / 1024.0;

                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("• Drive {} ({}) [{}]", disk.mount_point, disk.name, disk.file_system)).size(13.0));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(format!("{:.1} GB / {:.1} GB (Sisa: {:.1} GB)", used_gb, total_gb, free_gb)).size(12.0).color(Color32::from_rgb(203, 213, 225)));
                                });
                            });
                            ui.add_space(2.0);
                            ui.add(ProgressBar::new(ratio).fill(Color32::from_rgb(234, 179, 8)).desired_height(18.0));
                            ui.add_space(4.0);
                        }
                    });

                ui.add_space(20.0);

                // 3. Quick Actions
                ui.horizontal(|ui| {
                    if ui.add_sized([220.0, 38.0], egui::Button::new(RichText::new("🔄 Perbarui Telemetri").strong().size(13.5))).clicked() {
                        self.add_log("Manual snapshot telemetry diperbarui.", false);
                    }

                    if !self.config.is_paired() {
                        if ui.add_sized([240.0, 38.0], egui::Button::new(RichText::new("🔗 Hubungkan ke Backend (Pair)").strong().size(13.5)).fill(Color32::from_rgb(14, 165, 233))).clicked() {
                            self.current_tab = GuiTab::Pairing;
                        }
                    }
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                    ui.label("Mengambil data telemetri awal...");
                });
            }
        });
    }

    fn render_hardware_specs(&mut self, ui: &mut egui::Ui, snap_opt: Option<&DesktopTelemetrySnapshot>) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(6.0);
            ui.heading(RichText::new("Spesifikasi Lengkap Hardware & Sistem").size(20.0));
            ui.label(RichText::new("Seluruh metrik dan data hardware ini diekstrak secara native dan disinkronkan ke platform Kusuri.").size(12.5).color(Color32::from_rgb(148, 163, 184)));
            ui.add_space(16.0);

            if let Some(snap) = snap_opt {
                egui::Frame::NONE
                    .fill(Color32::from_rgb(30, 41, 59))
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(Margin::same(20))
                    .show(ui, |ui| {
                        egui::Grid::new("specs_grid_large")
                            .num_columns(2)
                            .spacing([50.0, 14.0])
                            .striped(true)
                            .show(ui, |ui| {
                                let rows = [
                                    ("Model Komputer / Laptop:", snap.system.raw_model.clone()),
                                    ("Pabrikan (Manufacturer):", snap.system.manufacturer.clone()),
                                    ("Nama Host (Hostname):", snap.system.hostname.clone()),
                                    ("Sistem Operasi:", format!("{} (Versi: {})", snap.system.os_name, snap.system.os_version)),
                                    ("Versi Kernel:", snap.system.kernel_version.clone()),
                                    ("Arsitektur Sistem:", snap.system.architecture.clone()),
                                    ("Prosesor (CPU):", snap.cpu.brand.clone()),
                                    ("Vendor CPU:", snap.cpu.vendor_id.clone()),
                                    ("Jumlah Inti Prosesor:", format!("{} Inti Fisik / {} Inti Logikal (Frekuensi: {} MHz)", snap.cpu.physical_cores, snap.cpu.logical_cores, snap.cpu.frequency_mhz)),
                                    ("Resolusi Layar Monitor:", format!("{} @ {:.1} Hz", snap.display.resolution, snap.display.refresh_rate_hz)),
                                    ("Kapasitas RAM Fisik:", format!("{} MB ({:.1} GB)", snap.memory.ram_total_mb, snap.memory.ram_total_mb as f32 / 1024.0)),
                                    ("Virtual Memory / Swap:", format!("{} MB Total", snap.memory.swap_total_mb)),
                                    ("Total Penyimpanan (Disk):", format!("{:.1} GB Total ({:.1} GB Terpakai)", snap.storage.total_mb as f32 / 1024.0, snap.storage.used_mb as f32 / 1024.0)),
                                    ("Status Baterai / Catu Daya:", format!("{} • {}", snap.power.power_source, snap.power.battery_health)),
                                    ("Alamat IP Lokal Utama:", snap.network.primary_ip.as_deref().unwrap_or("N/A").to_string()),
                                    ("Jenis Jaringan:", snap.network.network_type.clone()),
                                    ("Jumlah Proses Berjalan:", format!("{} proses", snap.process.running_processes_count)),
                                    ("Jejak Memori Aplikasi Kusuri:", format!("{} MB RAM (Ultra-efisien)", snap.process.client_ram_used_mb)),
                                ];

                                for (label, val) in rows {
                                    ui.label(RichText::new(label).strong().size(13.5).color(Color32::from_rgb(203, 213, 225)));
                                    ui.label(RichText::new(val).size(13.5).color(Color32::WHITE));
                                    ui.end_row();
                                }
                            });
                    });
            }
        });
    }

    fn render_pairing_tab(&mut self, ui: &mut egui::Ui, snap_opt: Option<&DesktopTelemetrySnapshot>) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(6.0);
            ui.heading(RichText::new("Pemasangan Perangkat (Pairing)").size(20.0));
            ui.label(RichText::new("Hubungkan client desktop ini dengan server Kusuri backend Anda menggunakan kode 6-digit.").size(12.5).color(Color32::from_rgb(148, 163, 184)));
            ui.add_space(16.0);

            if self.config.is_paired() {
                egui::Frame::NONE
                    .fill(Color32::from_rgb(30, 41, 59))
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(Margin::same(24))
                    .show(ui, |ui| {
                        ui.label(RichText::new("✅ PERANGKAT INI SUDAH TERPASANG DAN TERAUTENTIKASI").color(Color32::from_rgb(52, 211, 153)).size(15.0).strong());
                        ui.add_space(12.0);

                        egui::Grid::new("paired_info_grid").spacing([30.0, 10.0]).show(ui, |ui| {
                            ui.label(RichText::new("Server Backend URL:").strong().size(13.0));
                            ui.label(RichText::new(&self.config.server_url).size(13.0).color(Color32::from_rgb(56, 189, 248)));
                            ui.end_row();

                            ui.label(RichText::new("WebSocket URL:").strong().size(13.0));
                            ui.label(RichText::new(self.config.ws_url.as_deref().unwrap_or("Auto")).size(13.0).color(Color32::from_rgb(148, 163, 184)));
                            ui.end_row();

                            ui.label(RichText::new("Device ID:").strong().size(13.0));
                            ui.label(RichText::new(self.config.device_id.unwrap_or_default().to_string()).size(13.0).monospace());
                            ui.end_row();

                            ui.label(RichText::new("Nama Perangkat:").strong().size(13.0));
                            ui.label(RichText::new(self.config.device_name.as_deref().unwrap_or("Desktop")).size(13.0));
                            ui.end_row();

                            ui.label(RichText::new("Tipe Perangkat:").strong().size(13.0));
                            ui.label(RichText::new(self.config.device_type.as_deref().unwrap_or("windows")).size(13.0));
                            ui.end_row();
                        });

                        ui.add_space(18.0);
                        if ui.add_sized([220.0, 36.0], egui::Button::new(RichText::new("Putuskan Sambungan (Unpair)").size(13.0)).fill(Color32::from_rgb(185, 28, 28))).clicked() {
                            self.config.clear_credentials();
                            let _ = self.config.save();
                            self.stop_background_service();
                            self.add_log("Kredensial pairing dihapus dari client.", false);
                        }
                    });
            } else {
                egui::Frame::NONE
                    .fill(Color32::from_rgb(30, 41, 59))
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(Margin::same(24))
                    .show(ui, |ui| {
                        ui.label(RichText::new("Form Pairing Perangkat Baru").strong().size(16.0).color(Color32::from_rgb(56, 189, 248)));
                        ui.add_space(4.0);
                        ui.label(RichText::new("Buka dashboard web Kusuri > Devices > 'Pair Device' untuk mendapatkan kode 6-digit.").size(12.0).color(Color32::from_rgb(148, 163, 184)));
                        ui.add_space(16.0);

                        ui.label(RichText::new("URL Server Kusuri Backend:").size(13.0).strong());
                        ui.add_space(4.0);
                        ui.add(egui::TextEdit::singleline(&mut self.input_server_url).desired_width(f32::INFINITY).hint_text("Contoh: http://localhost:8080 atau localhost:8080"));

                        ui.add_space(14.0);
                        ui.label(RichText::new("Kode Pairing 6-Digit:").size(13.0).strong());
                        ui.add_space(4.0);
                        ui.add(
                            egui::TextEdit::singleline(&mut self.input_pairing_code)
                                .desired_width(f32::INFINITY)
                                .font(egui::TextStyle::Heading)
                                .hint_text("Contoh: 123456"),
                        );

                        ui.add_space(14.0);
                        ui.label(RichText::new("Nama Kustom Perangkat (Opsional):").size(13.0).strong());
                        ui.add_space(4.0);
                        ui.add(egui::TextEdit::singleline(&mut self.input_device_name).desired_width(f32::INFINITY).hint_text("Contoh: PC Kantor / Laptop Lenovo"));

                        ui.add_space(20.0);

                        if self.is_pairing_busy {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label(RichText::new("Menghubungi server dan memverifikasi kode...").size(13.5));
                            });
                        } else {
                            if ui.add_sized([260.0, 42.0], egui::Button::new(RichText::new("🚀 Hubungkan Sekarang").size(15.0).strong()).fill(Color32::from_rgb(16, 185, 129))).clicked() {
                                let code = self.input_pairing_code.trim().to_string();
                                let raw_server_input = self.input_server_url.trim();

                                // Normalisasi URL secara otomatis (menambahkan http:// jika belum ada)
                                let normalized_srv = api::normalize_server_url(raw_server_input);

                                let name = if self.input_device_name.trim().is_empty() {
                                    snap_opt.map(|s| s.system.raw_model.clone())
                                } else {
                                    Some(self.input_device_name.trim().to_string())
                                };
                                let raw_model = snap_opt.map(|s| s.system.raw_model.clone());

                                if code.len() < 6 {
                                    self.pairing_status_msg = Some(("Kode pairing harus terdiri dari 6 digit angka.".to_string(), false));
                                } else {
                                    self.is_pairing_busy = true;
                                    let rt = self.rt.clone();
                                    let api = ApiClient::new(&normalized_srv);
                                    let detected_type = if cfg!(target_os = "windows") { "windows" } else { "linux" };

                                    match rt.block_on(api.pair_device(&code, name, Some(detected_type.to_string()), raw_model)) {
                                        Ok(res) => {
                                            self.config.server_url = normalized_srv;
                                            self.config.device_id = Some(res.device_id);
                                            self.config.device_token = Some(res.device_token);
                                            self.config.auth_token = res.auth_token;
                                            self.config.device_name = Some(res.device_name);
                                            self.config.device_type = Some(res.device_type);
                                            self.config.normalize_ws_url();
                                            let _ = self.config.save();

                                            self.pairing_status_msg = Some(("Berhasil dipasangkan! Menghubungkan ke layanan 24/7...".to_string(), true));
                                            self.start_background_service();
                                        }
                                        Err(e) => {
                                            self.pairing_status_msg = Some((format!("Pairing gagal: {}", e), false));
                                        }
                                    }
                                    self.is_pairing_busy = false;
                                }
                            }
                        }

                        if let Some((msg, success)) = &self.pairing_status_msg {
                            ui.add_space(14.0);
                            let (bg_color, fg_color) = if *success {
                                (Color32::from_rgb(6, 78, 59), Color32::from_rgb(52, 211, 153))
                            } else {
                                (Color32::from_rgb(127, 29, 29), Color32::from_rgb(248, 113, 113))
                            };

                            egui::Frame::NONE
                                .fill(bg_color)
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::same(12))
                                .show(ui, |ui| {
                                    ui.label(RichText::new(msg).color(fg_color).strong().size(13.0));
                                });
                        }
                    });
            }
        });
    }

    fn render_logs_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Log Aktivitas Realtime").size(20.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add_sized([130.0, 32.0], egui::Button::new("Bersihkan Log")).clicked() {
                    if let Ok(mut l) = self.logs.lock() {
                        l.clear();
                    }
                }
            });
        });
        ui.add_space(12.0);

        egui::Frame::NONE
            .fill(Color32::from_rgb(10, 15, 29)) // Dark console black
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    if let Ok(logs) = self.logs.lock() {
                        if logs.is_empty() {
                            ui.label(RichText::new("Belum ada catatan aktivitas.").color(Color32::from_rgb(100, 116, 139)).size(13.0));
                        } else {
                            for entry in logs.iter() {
                                let color = if entry.is_error {
                                    Color32::from_rgb(248, 113, 113)
                                } else {
                                    Color32::from_rgb(226, 232, 240)
                                };
                                ui.label(RichText::new(format!("[{}] {}", entry.timestamp, entry.message)).color(color).monospace().size(12.5));
                            }
                        }
                    }
                });
            });
    }

    fn render_notifications_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.heading(RichText::new("🔔 Riwayat & Sinkronisasi Notifikasi").size(20.0));
        });
        ui.add_space(8.0);
        ui.label(
            RichText::new("Kusuri secara otomatis menangkap notifikasi lintas perangkat (ponsel Android & desktop). Seluruh notifikasi tersimpan dan dapat disinkronkan secara real-time.")
                .size(13.0)
                .color(Color32::from_rgb(148, 163, 184)),
        );
        ui.add_space(16.0);

        egui::Frame::NONE
            .fill(Color32::from_rgb(30, 41, 59))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::same(20))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("📱 Status Android Notification Listener:").strong().size(14.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        egui::Frame::NONE
                            .fill(Color32::from_rgb(6, 78, 59))
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(Margin::symmetric(10, 4))
                            .show(ui, |ui| {
                                ui.label(RichText::new("SIAP DISINKRONKAN").color(Color32::from_rgb(52, 211, 153)).size(11.0).strong());
                            });
                    });
                });
                ui.add_space(8.0);
                ui.label(RichText::new("Layanan KusuriNotificationListenerService pada Android secara otomatis membaca notifikasi masuk dan mengirimkannya ke endpoint POST /api/notifications.").size(12.5).color(Color32::from_rgb(203, 213, 225)));
            });

        ui.add_space(16.0);
        ui.heading(RichText::new("Notifikasi Terbaru yang Masuk").size(16.0));
        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Tampilkan baris notifikasi dari log atau stream
            if let Ok(logs) = self.logs.lock() {
                let notif_logs: Vec<_> = logs.iter().filter(|l| l.message.contains("Notifikasi Masuk") || l.message.contains("Notifikasi")).collect();
                if notif_logs.is_empty() {
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(20, 29, 47))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::same(20))
                        .show(ui, |ui| {
                            ui.label(RichText::new("Belum ada notifikasi baru yang diterima dari perangkat mobile atau desktop.").color(Color32::from_rgb(100, 116, 139)).size(13.0));
                        });
                } else {
                    for n in notif_logs.iter().rev() {
                        egui::Frame::NONE
                            .fill(Color32::from_rgb(30, 41, 59))
                            .corner_radius(CornerRadius::same(10))
                            .inner_margin(Margin::same(14))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("🔔").size(16.0));
                                    ui.label(RichText::new(&n.timestamp).color(Color32::from_rgb(56, 189, 248)).size(12.0));
                                    ui.label(RichText::new(&n.message).color(Color32::from_rgb(241, 245, 249)).size(13.0));
                                });
                            });
                        ui.add_space(6.0);
                    }
                }
            }
        });
    }

    fn render_settings_tab(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(6.0);
            ui.heading(RichText::new("Pengaturan Klien Desktop").size(20.0));
            ui.add_space(16.0);

            egui::Frame::NONE
                .fill(Color32::from_rgb(30, 41, 59))
                .corner_radius(CornerRadius::same(12))
                .inner_margin(Margin::same(24))
                .show(ui, |ui| {
                    ui.label(RichText::new("⏱️ Frekuensi Pengiriman Telemetri (Detik):").strong().size(14.0));
                    ui.add_space(4.0);
                    let mut interval = self.config.telemetry_interval_sec as f32;
                    if ui.add(egui::Slider::new(&mut interval, 10.0..=300.0).suffix(" detik")).changed() {
                        self.config.telemetry_interval_sec = interval as u64;
                        let _ = self.config.save();
                    }

                    ui.add_space(18.0);
                    ui.label(RichText::new("🔄 Reconnect Otomatis 24/7:").strong().size(14.0));
                    ui.add_space(4.0);
                    if ui.checkbox(&mut self.config.auto_reconnect, "Sambungkan kembali secara otomatis jika jaringan terputus (Exponential Backoff)").changed() {
                        let _ = self.config.save();
                    }

                    ui.add_space(18.0);
                    ui.label(RichText::new("📥 Integrasi System Tray & Background 24/7:").strong().size(14.0));
                    ui.add_space(4.0);
                    if ui.checkbox(&mut self.config.minimize_to_tray, "Minimalkan ke System Tray saat tombol tutup [X] diklik (aplikasi tetap berjalan)").changed() {
                        let _ = self.config.save();
                    }
                    ui.add_space(6.0);
                    if ui.checkbox(&mut self.config.start_minimized, "Mulai aplikasi dalam kondisi tersembunyi di System Tray (Startup Minimized)").changed() {
                        let _ = self.config.save();
                    }

                    ui.add_space(18.0);
                    ui.label(RichText::new("📁 Lokasi Penyimpanan File Konfigurasi:").strong().size(14.0));
                    ui.add_space(4.0);
                    ui.label(RichText::new(ClientConfig::config_file_path().display().to_string()).color(Color32::from_rgb(56, 189, 248)).monospace().size(12.5));
                });

            ui.add_space(16.0);
            egui::Frame::NONE
                .fill(Color32::from_rgb(30, 41, 59))
                .corner_radius(CornerRadius::same(12))
                .inner_margin(Margin::same(24))
                .show(ui, |ui| {
                    ui.label(RichText::new("ℹ️ Tentang Kusuri Desktop:").strong().size(14.0).color(Color32::from_rgb(56, 189, 248)));
                    ui.add_space(6.0);
                    ui.label(RichText::new("Kusuri Desktop Client dibangun dengan teknologi native Rust super ringan (< 30MB RAM) yang menghubungkan komputer Windows dan Linux ke platform Kusuri secara persisten 24 jam sehari, menyuplai metrik performa komprehensif, mendukung remote commands, dan clipboard sync.").size(12.5).color(Color32::from_rgb(203, 213, 225)));
                });
        });
    }
}

pub fn run_gui(start_minimized: bool) -> Result<(), eframe::Error> {
    info!("Memulai Kusuri Desktop GUI (start_minimized: {})...", start_minimized);

    let config = ClientConfig::load();
    let should_hide = start_minimized || config.start_minimized;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 760.0])
            .with_min_inner_size([920.0, 600.0])
            .with_title("Kusuri Desktop Client • 24/7 Sync")
            .with_visible(!should_hide),
        ..Default::default()
    };

    eframe::run_native(
        "Kusuri Desktop Client",
        options,
        Box::new(|cc| Ok(Box::new(KusuriGuiApp::new(cc)))),
    )
}
