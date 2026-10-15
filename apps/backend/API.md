# Kusuri API Documentation 📖

Dokumentasi lengkap REST API dan Real-time WebSocket untuk Kusuri Backend.

---

## 🌐 Base URL & Network Access

| Lingkungan | URL Dasar |
|------------|-----------|
| **Localhost** | `http://127.0.0.1:8080` |
| **LAN Host (Wi-Fi/Ethernet)** | `http://192.168.30.102:8080` *(Sesuaikan dengan IP komputer host)* |
| **WebSocket** | `ws://<HOST>:8080/ws?token=<TOKEN>` |

---

## 🔐 Model Autentikasi

Kusuri mendukung dua tipe kredensial melalui header `Authorization`:

1. **User JWT Token** (diperoleh saat register/login user dashboard):
   ```http
   Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
   ```
2. **Device Token** (diperoleh perangkat fisik saat proses pairing):
   ```http
   Authorization: Bearer ksr_a1b2c3d4e5f6...
   ```
   > 💡 *Pada endpoint unduhan file streaming (`/api/files/{id}/download`), token dapat disematkan via query parameter:* `?token=<JWT_ATAU_DEVICE_TOKEN>`.

---

## ⚠️ Standar Format Respons Error

Semua kegagalan HTTP (4xx / 5xx) mengembalikan format JSON konsisten:

```json
{
  "error": true,
  "message": "Deskripsi kesalahan atau validasi"
}
```

### Arti HTTP Status Code Utama
- `200 OK` / `201 Created`: Permintaan berhasil.
- `400 Bad Request`: Parameter atau input tidak valid secara logika.
- `401 Unauthorized`: Token tidak disertakan atau sudah kedaluwarsa.
- `403 Forbidden`: Akses ditolak (misal: IP diblokir sementara karena 3x salah pairing).
- `404 Not Found`: Entitas tidak ditemukan.
- `422 Unprocessable Entity`: Body JSON tidak dapat dibaca oleh parser (biasanya karena nama field/tipe data tidak sesuai schema).
- `500 Internal Server Error`: Kesalahan server atau koneksi database.

---

## 📋 Daftar Endpoint REST API

### 1. System & Health Check

#### `GET /api/health`
Mengecek status kesehatan server dan versi aplikasi.

* **Autentikasi:** Tidak perlu (Publik)
* **Respons Berhasil (`200 OK`):**
  ```json
  {
    "service": "kusuri-backend",
    "status": "healthy",
    "timestamp": "2026-10-12T04:30:00.000000Z",
    "version": "0.1.0"
  }
  ```

---

### 2. Autentikasi Pengguna (`/api/auth`)

#### `POST /api/auth/register`
Mendaftarkan akun pengguna baru.

* **Autentikasi:** Tidak perlu (Publik)
* **Headers:** `Content-Type: application/json`
* **Request Body:**
  ```json
  {
    "username": "ananda",
    "password": "Password123!",
    "email": "ananda@example.com"
  }
  ```
  | Field | Tipe | Wajib? | Catatan |
  |-------|------|--------|---------|
  | `username` | string | **Ya** | Minimal 3 karakter |
  | `password` | string | **Ya** | Minimal 8 karakter |
  | `email` | string | Opsional | Format email valid |

* **Respons Berhasil (`201 Created`):**
  ```json
  {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user": {
      "id": "e674512b-3c35-4cb1-807d-94bb9a2d8a43",
      "username": "ananda",
      "email": "ananda@example.com",
      "created_at": "2026-10-12T04:30:00Z"
    }
  }
  ```

#### `POST /api/auth/login`
Masuk dengan username dan password yang sudah terdaftar.

* **Autentikasi:** Tidak perlu (Publik)
* **Headers:** `Content-Type: application/json`
* **Request Body:**
  ```json
  {
    "username": "ananda",
    "password": "Password123!"
  }
  ```
* **Respons Berhasil (`200 OK`):**
  ```json
  {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user": {
      "id": "e674512b-3c35-4cb1-807d-94bb9a2d8a43",
      "username": "ananda",
      "email": "ananda@example.com",
      "created_at": "2026-10-12T04:30:00Z"
    }
  }
  ```

#### `GET /api/auth/me`
Mengambil profil akun user yang sedang aktif.

* **Autentikasi:** User JWT (`Bearer <JWT>`)
* **Respons Berhasil (`200 OK`):**
  ```json
  {
    "id": "e674512b-3c35-4cb1-807d-94bb9a2d8a43",
    "username": "ananda",
    "email": "ananda@example.com",
    "created_at": "2026-10-12T04:30:00Z"
  }
  ```

---

### 3. Pairing & Manajemen Perangkat (`/api/devices`)

#### `POST /api/devices/pairing-code`
Menghasilkan kode pairing 6 digit dari perangkat utama / web dashboard.

* **Autentikasi:** User JWT (`Bearer <JWT>`)
* **Headers:** `Content-Type: application/json`
* **Request Body (Opsional):**
  ```json
  {
    "device_name": "Infinix Hot 50",
    "device_type": "android"
  }
  ```
* **Respons Berhasil (`200 OK`):**
  ```json
  {
    "pairing_id": "99b9cf25-e51c-4b68-b391-4c132860d84c",
    "code": "583921",
    "expires_at": "2026-10-12T04:35:00Z",
    "expires_in_seconds": 300,
    "max_attempts": 3
  }
  ```
  > 🔒 **Keamanan:** Kode hanya berlaku **5 menit** dan maksimal **3 kali percobaan salah**. Setelah 3 kali gagal, kode otomatis dihapus (hangus).

---

#### `POST /api/devices/pair`
Dijalankan oleh perangkat client baru (Android, Windows, Linux) untuk menukarkan kode pairing dengan `device_token` permanen.

* **Autentikasi:** Tidak perlu (Publik / Client Direct)
* **Headers:** `Content-Type: application/json`
* **Request Body:**
  ```json
  {
    "pairing_code": "583921",
    "device_name": "Infinix-X6881",
    "device_type": "android",
    "raw_model": "Infinix-X6881",
    "pairing_id": null
  }
  ```
  | Field | Tipe | Wajib? | Alias Diterima | Catatan |
  |-------|------|--------|----------------|---------|
  | `pairing_code` | string | **Ya** | `code` | 6 digit string kode pairing |
  | `device_name` | string | Opsional | `name` | Nama perangkat (default: `"Unknown Device"` jika kosong) |
  | `device_type` | string | Opsional | `type` | `"android"`, `"windows"`, `"linux"`, `"web"` |
  | `raw_model` | string | Opsional | `model` | Codename hardware pabrikan (contoh: `Infinix-X6881`) |
  | `pairing_id` | UUID | Opsional | - | UUID sesi pairing (opsional, gunakan jika via QR code) |

* **Contoh Minimal Payload (tetap valid):**
  ```json
  {
    "code": "583921",
    "name": "My Phone",
    "type": "android"
  }
  ```

* **Respons Berhasil (`200 OK`):**
  ```json
  {
    "device_id": "7b01d3cb-8c90-4824-9b2f-22a3d027f677",
    "device_token": "ksr_69b768e77a2842419ce5fc00045f2f518e950bc4410a402390a77f989f66bb40",
    "user_id": "e674512b-3c35-4cb1-807d-94bb9a2d8a43",
    "device_name": "Infinix HOT 50 Pro",
    "device_type": "android",
    "marketing_name": "Infinix HOT 50 Pro",
    "chipset": "Mediatek Helio G100 (6 nm)"
  }
  ```
  > ⚠️ **PENTING:** Simpan `device_token` di local storage atau SharedPreferences perangkat Android/klien. Token mentah ini **tidak disimpan di database** (database hanya menyimpan SHA-256 hash). Token ini digunakan untuk login WebSocket dan panggilan REST API berikutnya.

---

#### `GET /api/devices`
Mengambil daftar semua perangkat yang terhubung ke akun user.

* **Autentikasi:** User JWT atau Device Token (`Bearer <TOKEN>`)
* **Respons Berhasil (`200 OK`):**
  ```json
  [
    {
      "id": "7b01d3cb-8c90-4824-9b2f-22a3d027f677",
      "user_id": "e674512b-3c35-4cb1-807d-94bb9a2d8a43",
      "name": "Infinix HOT 50 Pro",
      "device_type": "android",
      "is_online": true,
      "last_seen_at": "2026-10-12T04:30:00Z",
      "raw_model": "Infinix-X6881",
      "marketing_name": "Infinix HOT 50 Pro",
      "retail_branding": "Infinix",
      "chipset": "Mediatek Helio G100 (6 nm)"
    }
  ]
  ```

---

#### `GET /api/devices/{id}/specs`
Mengambil spesifikasi mendalam gabungan dari Google Play Supported Devices Catalog, GSM Arena scraping, dan native hardware telemetry.

* **Autentikasi:** User JWT atau Device Token
* **Respons Berhasil (`200 OK`):**
  ```json
  {
    "device_id": "7b01d3cb-8c90-4824-9b2f-22a3d027f677",
    "device_name": "Infinix HOT 50 Pro",
    "device_type": "android",
    "raw_model": "Infinix-X6881",
    "marketing_name": "Infinix HOT 50 Pro",
    "retail_branding": "Infinix",
    "chipset": "Mediatek Helio G100 (6 nm)",
    "gsmarena_specs": {
      "device_name": "Infinix Hot 50 Pro",
      "chipset": "Mediatek Helio G100 (6 nm)",
      "cpu": "Octa-core (2x2.2 GHz Cortex-A76 & 6x2.0 GHz Cortex-A55)",
      "gpu": "Mali-G57 MC2",
      "display_type": "AMOLED, 120Hz, 1800 nits (peak)",
      "display_size": "6.78 inches (~89.3% screen-to-body ratio)",
      "display_resolution": "1080 x 2436 pixels (~393 ppi density)",
      "main_camera": "50 MP, f/1.6, (wide), AF",
      "battery_info": "5000 mAh, non-removable, 33W wired",
      "image_url": "https://fdn2.gsmarena.com/vv/bigpic/infinix-hot-50-pro.jpg"
    },
    "telemetry": {
      "imei1": "861234567890123",
      "imei2": "861234567890124",
      "ram_total_mb": 8192,
      "ram_used_mb": 4120,
      "storage_total_mb": 262144,
      "storage_used_mb": 94300,
      "battery_percent": 88.0,
      "battery_charging": true,
      "battery_temp_c": 32.5,
      "os_version": "Android 14 (XOS 14.5)",
      "android_sdk": 34
    }
  }
  ```

---

#### `POST /api/devices/{id}/telemetry`
Memperbarui telemetri hardware native dari perangkat (misal: Android Native Service mengirimkan RAM, Storage, Baterai, IMEI).

* **Autentikasi:** User JWT atau Device Token
* **Request Body:**
  ```json
  {
    "raw_model": "Infinix-X6881",
    "imei1": "861234567890123",
    "imei2": "861234567890124",
    "ram_total_mb": 8192,
    "ram_used_mb": 4120,
    "storage_total_mb": 262144,
    "storage_used_mb": 94300,
    "battery_percent": 88.0,
    "battery_charging": true,
    "battery_temp_c": 32.5,
    "os_version": "Android 14 (XOS 14.5)",
    "android_sdk": 34
  }
  ```

---

#### `PATCH /api/devices/{id}`
Mengubah nama kustom perangkat.

* **Autentikasi:** User JWT atau Device Token
* **Request Body:**
  ```json
  {
    "name": "HP Kantor Infinix"
  }
  ```

---

#### `DELETE /api/devices/{id}`
Menghapus perangkat dari akun user dan mencabut akses tokennya.

* **Autentikasi:** User JWT

---

### 4. Sinkronisasi Clipboard (`/api/clipboard`)

#### `GET /api/clipboard`
Mengambil riwayat teks clipboard terbaru milik pengguna.

* **Autentikasi:** User JWT atau Device Token
* **Query Parameters:** `?limit=50` (opsional, default 50)
* **Respons Berhasil (`200 OK`):**
  ```json
  [
    {
      "id": "2d1b8214-3a9d-4876-b605-6a5be715db14",
      "user_id": "e674512b-3c35-4cb1-807d-94bb9a2d8a43",
      "source_device_id": "7b01d3cb-8c90-4824-9b2f-22a3d027f677",
      "source_device_name": "Infinix HOT 50 Pro",
      "content": "https://kusuri.dev/docs",
      "content_type": "text/plain",
      "created_at": "2026-10-12T04:32:10Z"
    }
  ]
  ```

#### `POST /api/clipboard`
Menyimpan teks clipboard baru. **Otomatis disiarkan via WebSocket** ke seluruh perangkat lain milik user yang sedang aktif.

* **Autentikasi:** User JWT atau Device Token
* **Request Body:**
  ```json
  {
    "content": "Teks yang disalin dari Android",
    "content_type": "text/plain"
  }
  ```

#### `DELETE /api/clipboard/{id}`
Menghapus riwayat item clipboard tertentu.

---

### 5. Transfer File (`/api/files`)

#### `POST /api/files/upload`
Mengunggah file ke backend. Setelah selesai, server akan menyiarkan event WebSocket `file_available` ke perangkat target.

* **Autentikasi:** User JWT atau Device Token
* **Content-Type:** `multipart/form-data`
* **Form Fields:**
  - `file`: Berkas binary (file nyata)
  - `target_device_id`: *(Opsional)* UUID perangkat penerima. Jika dikosongkan, file dapat diakses oleh semua perangkat user.
* **Respons Berhasil (`200 OK`):**
  ```json
  {
    "id": "31b26ef8-825f-40e1-b472-a541603ba7f2",
    "filename": "document.pdf",
    "file_size": 2048500,
    "mime_type": "application/pdf",
    "download_url": "/api/files/31b26ef8-825f-40e1-b472-a541603ba7f2/download"
  }
  ```

#### `GET /api/files`
Mengambil daftar berkas yang dibagikan.

#### `GET /api/files/{id}/download`
Mengunduh berkas langsung secara streaming hemat RAM.

* **Autentikasi:** Header `Authorization: Bearer <TOKEN>` **atau** Query param `?token=<TOKEN>`

#### `DELETE /api/files/{id}`
Menghapus file dari server dan sistem disk.

---

### 6. Remote Commands (`/api/commands`)

#### `POST /api/commands`
Mengirim instruksi jarak jauh ke perangkat tertentu (misal: ping, lock screen, open url).

* **Autentikasi:** User JWT atau Device Token
* **Request Body:**
  ```json
  {
    "target_device_id": "7b01d3cb-8c90-4824-9b2f-22a3d027f677",
    "command": "open_url",
    "payload": {
      "url": "https://google.com"
    }
  }
  ```

#### `POST /api/commands/{id}/result`
Dilaporkan oleh perangkat eksekutor setelah perintah selesai dijalankan.

* **Autentikasi:** Device Token
* **Request Body:**
  ```json
  {
    "command_id": "90ba95dc-1fa0-424a-9ef5-6dbfb4b4ca81",
    "status": "completed",
    "response": {
      "success": true,
      "message": "Browser opened successfully"
    }
  }
  ```

---

## ⚡ Protokol Real-Time WebSocket (`/ws`)

### 1. Membuka Koneksi

Klien terhubung ke WebSocket menggunakan query parameter `token`:
```text
ws://<HOST>:8080/ws?token=<DEVICE_TOKEN_ATAU_JWT>
```
*Contoh:*
`ws://192.168.30.102:8080/ws?token=ksr_69b768e77a2842419ce5fc00045f2f518e950bc4410a402390a77f989f66bb40`

---

### 2. Format Event: Klien ➡️ Server

#### a. Heartbeat / Ping
```json
{
  "type": "ping"
}
```
*Server akan membalas dengan `{"type": "pong"}`.*

#### b. Sinkronisasi Clipboard
```json
{
  "type": "clipboard_sync",
  "payload": {
    "content": "Teks yang disalin di Android",
    "content_type": "text/plain"
  }
}
```

#### c. Live Hardware Telemetry
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

#### d. Kirim Remote Command
```json
{
  "type": "send_command",
  "payload": {
    "target_device_id": "TARGET_DEVICE_UUID",
    "command": "lock_screen",
    "payload": {}
  }
}
```

#### e. Lapor Hasil Remote Command
```json
{
  "type": "command_result",
  "payload": {
    "command_id": "COMMAND_UUID",
    "status": "completed",
    "response": {
      "success": true
    }
  }
}
```

#### f. Relay Pesan Kustom (Pub/Sub)
```json
{
  "type": "relay_message",
  "payload": {
    "target_device_id": null,
    "event": "sync_contacts",
    "data": { "count": 150 }
  }
}
```

---

### 3. Format Event: Server ➡️ Klien

#### a. Perangkat Terhubung / Online
```json
{
  "type": "device_connected",
  "payload": {
    "device_id": "7b01d3cb-8c90-4824-9b2f-22a3d027f677",
    "name": "Infinix HOT 50 Pro",
    "device_type": "android"
  }
}
```

#### b. Perangkat Terputus / Offline
```json
{
  "type": "device_disconnected",
  "payload": {
    "device_id": "7b01d3cb-8c90-4824-9b2f-22a3d027f677"
  }
}
```

#### c. Clipboard Diperbarui
```json
{
  "type": "clipboard_updated",
  "payload": {
    "id": "2d1b8214-3a9d-4876-b605-6a5be715db14",
    "content": "Teks yang baru disalin",
    "source_device_name": "Infinix HOT 50 Pro"
  }
}
```

#### d. Perintah yang Harus Dieksekusi
```json
{
  "type": "execute_command",
  "payload": {
    "command_id": "90ba95dc-1fa0-424a-9ef5-6dbfb4b4ca81",
    "command": "open_url",
    "payload": {
      "url": "https://kusuri.dev"
    }
  }
}
```

#### e. Notifikasi File Tersedia
```json
{
  "type": "file_available",
  "payload": {
    "file_id": "31b26ef8-825f-40e1-b472-a541603ba7f2",
    "original_filename": "photo.jpg",
    "file_size": 1048576,
    "download_url": "/api/files/31b26ef8-825f-40e1-b472-a541603ba7f2/download"
  }
}
```

#### f. Spesifikasi Hardware Diperbarui
```json
{
  "type": "device_specs_updated",
  "payload": {
    "device_id": "7b01d3cb-8c90-4824-9b2f-22a3d027f677",
    "specs": {
      "marketing_name": "Infinix HOT 50 Pro",
      "chipset": "Mediatek Helio G100 (6 nm)"
    }
  }
}
```
