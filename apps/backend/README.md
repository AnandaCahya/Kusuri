# Kusuri Backend 🚀

Backend server performa tinggi berbasis **Rust (Axum + Tokio + SQLx + PostgreSQL)** untuk menghubungkan berbagai perangkat seperti **Android, Windows, dan Linux**.

Mendukung skenario **self-hosted**, **multi-user**, serta pertukaran data real-time melalui **WebSocket** dan **REST API**.

---

## 🌟 Fitur Utama

1. **Multi-User & Multi-Platform**:
   - Pengguna dapat membuat akun (Registrasi/Login berbasis JWT + Argon2 password hashing).
   - Menghubungkan perangkat Android, Windows, Linux, atau perangkat lainnya.
2. **Device Pairing Sederhana (6-Digit Code)**:
   - Pengguna menghasilkan kode pairing 6-digit dari dashboard/perangkat utama.
   - Perangkat baru cukup memasukkan kode pairing untuk mendapatkan `device_token` permanen (tanpa perlu menyimpan password akun di perangkat).
3. **Sinkronisasi Clipboard & Teks Real-Time**:
   - Kirim teks atau tautan secara instan antar perangkat yang terhubung.
   - Riwayat clipboard tersimpan di database PostgreSQL.
4. **Transfer File & Media**:
   - Upload file via REST API multipart (`/api/files/upload`).
   - Notifikasi ketersediaan file disiarkan via WebSocket (`file_available`) dengan tautan download streaming hemat memori.
5. **Resolusi Spesifikasi Pabrik & GSM Arena Scraper**:
   - Mendeteksi codename perangkat mentah dari Android (misal: `Infinix-X6881`).
   - Melakukan validasi & mapping otomatis terhadap **Google Play Supported Devices Catalog** (`storage.googleapis.com/play_public/supported_devices.csv`) untuk mendapatkan **Consumer Product Name** (contoh: `Infinix HOT 50 Pro`).
   - Mengambil spesifikasi mendalam dari **GSM Arena** (misal: Chipset `Mediatek Helio G100 (6 nm)`, CPU, GPU, Display, Kamera, Baterai, dan Foto resmi) dan menyimpannya secara terstruktur di database PostgreSQL (dengan caching otomatis).
   - Menghubungkan telemetri live native Android (RAM, Storage, Suhu & Persentase Baterai, IMEI 1 & 2, Android SDK, Kernel) dengan spesifikasi katalog pabrikan.
6. **Remote Command & Telemetri Perangkat**:
   - Kirim perintah jarak jauh ke perangkat tertentu (misal: ping, lock screen, buka URL, get status).
   - Perangkat target mengeksekusi dan melaporkan hasil kembali via WebSocket/REST.
   - Sinkronisasi status baterai, RAM, IP, dan OS detail secara berkala.
7. **Pub/Sub Relay Pesan Kustom**:
   - Kirim event JSON kustom ke perangkat tertentu atau broadcast ke semua perangkat milik pengguna.
8. **Database Migration Otomatis**:
   - Migrasi skema PostgreSQL dieksekusi secara otomatis saat server pertama kali dijalankan.
9. **Siap Docker & Self-Hosted**:
   - Dilengkapi `Dockerfile` multi-stage dan `docker-compose.yml` siap pakai.

---

## 🏗️ Struktur Proyek

```text
backend/
├── Cargo.toml
├── Dockerfile
├── docker-compose.yml
├── .env.example
├── .gitignore
├── README.md
├── migrations/
│   └── 0001_init.sql         # Skema database & tabel PostgreSQL
└── src/
    ├── main.rs               # Entrypoint & inisialisasi server
    ├── config.rs             # Konfigurasi environment (.env)
    ├── error.rs              # Error handling terpadu (JSON response)
    ├── auth.rs               # JWT, Argon2 hashing & extractor auth
    ├── models/
    │   ├── user.rs           # Model user & DTO auth
    │   ├── device.rs         # Model perangkat & pairing code
    │   ├── clipboard.rs      # Model clipboard sync
    │   ├── file.rs           # Model berkas bersama
    │   ├── command.rs        # Model perintah jarak jauh
    │   └── mod.rs
    ├── routes/
    │   ├── auth.rs           # /api/auth/*
    │   ├── device.rs         # /api/devices/*
    │   ├── clipboard.rs      # /api/clipboard/*
    │   ├── file.rs           # /api/files/*
    │   ├── command.rs        # /api/commands/*
    │   └── mod.rs            # Router aggregator & health check
    └── ws/
        ├── hub.rs            # In-memory routing & pub/sub per user
        ├── handler.rs        # WebSocket connection lifecycle & dispatcher
        └── mod.rs
```

---

## 🚀 Panduan Menjalankan (Self-Hosted)

### Cara 1: Menggunakan Docker Compose (Direkomendasikan)

Pastikan Docker & Docker Compose sudah terinstal di server/komputer Anda:

```bash
# 1. Jalankan database PostgreSQL dan Kusuri Backend
docker compose up -d

# 2. Cek status container
docker compose ps

# 3. Pantau log server
docker compose logs -f kusuri-backend
```

Backend akan aktif di `http://localhost:8080` dan siap digunakan!

---

### Cara 2: Menjalankan Langsung (Bare-metal / Development)

1. Pastikan PostgreSQL sudah aktif. Buat database bernama `kusuri`.
2. Salin konfigurasi environment:
   ```bash
   cp .env.example .env
   ```
3. Sesuaikan `DATABASE_URL` di dalam file `.env`:
   ```env
   DATABASE_URL=postgres://postgres:kusurisecret@localhost:5432/kusuri
   ```
4. Jalankan aplikasi:
   ```bash
   cargo run --release
   ```
   *Migrasi database (`migrations/0001_init.sql`) akan berjalan otomatis saat startup.*

---

## 📡 REST API Reference

Semua response error memiliki format:
```json
{
  "error": true,
  "message": "Deskripsi kesalahan"
}
```

### 1. Health Check
- **`GET /api/health`**
  - Response: `{"status": "healthy", "service": "kusuri-backend", "version": "0.1.0"}`

### 2. Autentikasi User
- **`POST /api/auth/register`**
  - Body: `{"username": "ananda", "password": "mypassword123"}`
  - Response: `{"token": "JWT_TOKEN", "user": {"id": "UUID", "username": "ananda", ...}}`
- **`POST /api/auth/login`**
  - Body: `{"username": "ananda", "password": "mypassword123"}`
  - Response: `{"token": "JWT_TOKEN", "user": {"id": "UUID", ...}}`
- **`GET /api/auth/me`**
  - Header: `Authorization: Bearer <JWT_TOKEN>`

### 3. Pairing Perangkat & Manajemen
- **`POST /api/devices/pairing-code`**
  - Header: `Authorization: Bearer <JWT_TOKEN>`
  - Body: `{"device_name": "Opsional", "device_type": "android"}`
  - Response:
    ```json
    {
      "pairing_id": "UUID_SESI",
      "code": "482910",
      "expires_at": "...",
      "expires_in_seconds": 300,
      "max_attempts": 3
    }
    ```
  - *Catatan Keamanan: Kode valid selama 5 menit. Percobaan dibatasi maksimal 3 kali salah. Jika salah 3 kali, kode otomatis dibakar (dihapus).*
- **`POST /api/devices/pair`** *(Dipanggil oleh perangkat Android/Windows/Linux baru)*
  - Body:
    ```json
    {
      "pairing_code": "482910",
      "pairing_id": "UUID_SESI_OPSIONAL",
      "device_name": "Infinix-X6881",
      "device_type": "android",
      "raw_model": "Infinix-X6881"
    }
    ```
  - Response:
    ```json
    {
      "device_id": "DEVICE_UUID",
      "device_token": "ksr_a1b2c3d4...",
      "user_id": "USER_UUID",
      "device_name": "Infinix HOT 50 Pro",
      "device_type": "android",
      "marketing_name": "Infinix HOT 50 Pro",
      "chipset": "Mediatek Helio G100 (6 nm)"
    }
    ```
  - *Catatan Keamanan:*
    1. **Token Hashing**: Token acak `ksr_...` (256-bit entropy) di-hash menggunakan **SHA-256** sebelum disimpan di database (`token_hash`). Token mentah hanya disimpan di sisi klien dan tidak pernah disimpan di database!
    2. **Anti-Brute Force**: Jika IP yang sama gagal memasukkan kode 3 kali dalam 5 menit, IP akan diblokir sementara selama 5 menit (HTTP 403 Forbidden).
    3. **Timing Delay**: Ditambahkan jeda waktu buatan 500ms pada setiap percobaan gagal untuk mencegah serangan kamus terotomatisasi.
- **`GET /api/devices`**
  - Header: `Authorization: Bearer <JWT_TOKEN>`
  - Response: Daftar perangkat dengan `marketing_name` dan `chipset` jika terdeteksi.
- **`GET /api/devices/{id}/specs`**
  - Header: `Authorization: Bearer <JWT_TOKEN>`
  - Response: Spesifikasi lengkap perangkat (Kombinasi data Google Play Devices, GSM Arena Chipset/Display/Kamera, dan telemetri hardware live).
- **`POST /api/devices/{id}/telemetry`**
  - Body:
    ```json
    {
      "raw_model": "Infinix-X6881",
      "imei1": "861234567890123",
      "imei2": "861234567890124",
      "ram_total_mb": 8192,
      "ram_used_mb": 4250,
      "storage_total_mb": 262144,
      "storage_used_mb": 95200,
      "battery_percent": 87.5,
      "battery_charging": true,
      "battery_temp_c": 32.4,
      "os_version": "Android 14 (XOS 14.5)",
      "android_sdk": 34
    }
    ```
  - Server otomatis memetakan model ke Google Play catalog, scraping GSM Arena jika belum di-cache, mengupdate database, dan menyiarkan `device_specs_updated` via WebSocket.
- **`POST /api/devices/sync-catalog`**
  - Sinkronisasi manual katalog Google Play Supported Devices.
- **`PATCH /api/devices/{id}`**
  - Body: `{"name": "Nama Baru Perangkat"}`
- **`DELETE /api/devices/{id}`**
  - Menghapus perangkat dari akun.

### 4. Sinkronisasi Clipboard
- **`GET /api/clipboard`**
  - Header: `Authorization: Bearer <JWT_TOKEN>`
  - Response: 50 riwayat clipboard terbaru.
- **`POST /api/clipboard`**
  - Header: `Authorization: Bearer <JWT_TOKEN>`
  - Body: `{"content": "Teks yang disalin", "content_type": "text/plain"}`
  - *Catatan: Otomatis disiarkan via WebSocket ke perangkat lain.*
- **`DELETE /api/clipboard/{id}`**

### 5. Transfer File
- **`POST /api/files/upload`**
  - Header: `Authorization: Bearer <JWT_TOKEN>`
  - Content-Type: `multipart/form-data`
  - Form Fields:
    - `file`: Berkas binary
    - `target_device_id`: *(Opsional, UUID perangkat tujuan. Jika kosong, broadcast ke semua perangkat)*
- **`GET /api/files`**
  - Daftar berkas yang dibagikan.
- **`GET /api/files/{id}/download?token=<JWT_ATAU_DEVICE_TOKEN>`**
  - Unduh berkas langsung secara streaming.
- **`DELETE /api/files/{id}`**

### 6. Remote Command
- **`POST /api/commands`**
  - Body:
    ```json
    {
      "target_device_id": "TARGET_DEVICE_UUID",
      "command": "ping",
      "payload": {"param": "value"}
    }
    ```
- **`GET /api/commands`**
  - Riwayat perintah yang dikirim.
- **`POST /api/commands/{id}/result`**
  - Body:
    ```json
    {
      "command_id": "UUID",
      "status": "completed",
      "response": {"output": "pong", "battery": 92}
    }
    ```

---

## ⚡ Protokol Real-Time WebSocket (`/ws`)

Koneksi dibuka ke:
`ws://<host>:8080/ws?token=<DEVICE_TOKEN_ATAU_JWT>`

### Format Event dari Client ke Server:

1. **Heartbeat / Ping**:
   ```json
   {"type": "ping"}
   ```
   *Server membalas dengan `{"type": "pong"}`.*

2. **Sinkronisasi Clipboard**:
   ```json
   {
     "type": "clipboard_sync",
     "payload": {
       "content": "https://kusuri.app",
       "content_type": "text/plain"
     }
   }
   ```

3. **Kirim Telemetri / Status Hardware**:
   ```json
   {
     "type": "telemetry",
     "payload": {
       "battery_percent": 87.5,
       "is_charging": true,
       "os_details": "Ubuntu 24.04 LTS (Kernel 6.8)",
       "memory_used_mb": 4200,
       "memory_total_mb": 16000,
       "ip_address": "192.168.1.15"
     }
   }
   ```

    ```

4. **Kirim Live Hardware Telemetry (Otomatis Diperkaya dengan Google Catalog & GSM Arena Specs)**:
   ```json
   {
     "type": "hardware_telemetry",
     "payload": {
       "raw_model": "Infinix-X6881",
       "imei1": "861234567890123",
       "ram_total_mb": 8192,
       "ram_used_mb": 4250,
       "storage_total_mb": 262144,
       "storage_used_mb": 95200,
       "battery_percent": 87.5,
       "battery_charging": true,
       "battery_temp_c": 32.4,
       "os_version": "Android 14 (XOS 14.5)"
     }
   }
   ```

5. **Kirim Remote Command ke Perangkat Lain**:
   ```json
   {
     "type": "send_command",
     "payload": {
       "target_device_id": "TARGET_UUID",
       "command": "lock_screen",
       "payload": {}
     }
   }
   ```

6. **Lapor Hasil Eksekusi Remote Command**:
   ```json
   {
     "type": "command_result",
     "payload": {
       "command_id": "COMMAND_UUID",
       "status": "completed",
       "response": {"success": true}
     }
   }
   ```

7. **Relay Pesan Kustom (Pub/Sub)**:
   ```json
   {
     "type": "relay_message",
     "payload": {
       "target_device_id": null,
       "event": "custom_sync",
       "data": {"any": "json"}
     }
   }
   ```

---

### Format Event dari Server ke Client:

1. **Perangkat Online / Terhubung**:
   `{"type": "device_connected", "payload": {"device_id": "...", "name": "...", "device_type": "..."}}`
2. **Perangkat Terputus**:
   `{"type": "device_disconnected", "payload": {"device_id": "..."}}`
3. **Clipboard Diperbarui**:
   `{"type": "clipboard_updated", "payload": {"id": "...", "content": "...", "source_device_name": "..."}}`
4. **Status Perangkat Diperbarui (Basic Telemetry)**:
   `{"type": "device_status_updated", "payload": {"device_id": "...", "data": {...}}}`
5. **Spesifikasi & Hardware Lengkap Diperbarui**:
   `{"type": "device_specs_updated", "payload": {"device_id": "...", "specs": {"marketing_name": "Infinix HOT 50 Pro", "chipset": "Mediatek Helio G100 (6 nm)", ...}}}`
6. **Perintah untuk Dieksekusi**:
   `{"type": "execute_command", "payload": {"command_id": "...", "command": "...", "payload": {...}}}`
7. **File Tersedia untuk Diunduh**:
   `{"type": "file_available", "payload": {"file_id": "...", "original_filename": "photo.jpg", "file_size": 1048576, "download_url": "/api/files/.../download"}}`
8. **Relay Event Diterima**:
   `{"type": "relay_event", "payload": {"source_device_id": "...", "event": "...", "data": {...}}}`
