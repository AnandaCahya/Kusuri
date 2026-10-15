# Kusuri Backend Analysis & REQ.md Alignment Report

Dokumen ini memetakan analisa sistem backend Kusuri (`backend/` & `BACKEND.md`) terhadap kebutuhan spesifikasi fungsional dan teknis di `REQ.md`.

---

## 1. Ringkasan Arsitektur Backend

- **Stack**: Rust (Axum, Tokio, SQLx, PostgreSQL).
- **Protokol**:
  - REST API (`/api/*`) berbasis JSON & multipart/form-data.
  - WebSocket (`/ws?token=<JWT_OR_DEVICE_TOKEN>`) dua arah dengan penanganan virtual session bagi web dashboard client (`username-web`, type `web`).
- **Autentikasi**:
  - Pengguna: JWT Token (ditransmisikan via header `Authorization: Bearer <token>` atau query param untuk file download/WS). Password di-hash menggunakan Argon2.
  - Perangkat: 64-character random device token (`ksr_...`, 256-bit entropy). Di backend disimpan dalam bentuk SHA-256 `token_hash`.
  - Anti-brute force: Batas percobaan pairing 3x dalam 5 menit, IP rate limiting 5 menit lockout jika gagal beruntun, serta 500ms delay.

---

## 2. Pemetaan Fitur & Kontrak API Terhadap REQ.md

### 2.1. Authentication & Account
| Fitur REQ.md | Endpoint Backend | Status Backend | Keterangan Frontend |
|---|---|---|---|
| Register | `POST /api/auth/register` | ✅ Didukung | Body: `{ username, password }` |
| Login | `POST /api/auth/login` | ✅ Didukung | Body: `{ username, password }` -> `{ token, user }` |
| Current User Info | `GET /api/auth/me` | ✅ Didukung | Header: `Authorization: Bearer <token>` |
| Logout | Client-side revocation | ✅ Didukung | Hapus token lokal & tutup koneksi WebSocket |
| Ubah Password / Profil | Belum ada endpoint | ⚠️ Belum ada di backend | Sesuai REQ.md Sec. 17: Tampilkan status read-only & informatif, tidak membuat endpoint palsu |

### 2.2. Device Management & Pairing
| Fitur REQ.md | Endpoint / Event Backend | Status Backend | Keterangan Frontend |
|---|---|---|---|
| Generate Pairing Code | `POST /api/devices/pairing-code` | ✅ Didukung | Body: `{ device_name?, device_type? }`. Return: 6-digit code, expires_in 300s, max 3 attempts |
| Pair Perangkat Baru | `POST /api/devices/pair` | ✅ Didukung | Digunakan oleh client native (Android/Linux/Windows) |
| Daftar Perangkat | `GET /api/devices` | ✅ Didukung | Menampilkan `id`, `name`, `device_type`, `is_online`, `last_seen_at`, `marketing_name`, `chipset` |
| Rename Perangkat | `PATCH /api/devices/{id}` | ✅ Didukung | Body: `{ name: "..." }` |
| Hapus / Putus Perangkat | `DELETE /api/devices/{id}` | ✅ Didukung | Memerlukan konfirmasi pengguna di UI |
| Spesifikasi & Telemetri Lengkap | `GET /api/devices/{id}/specs` | ✅ Didukung | Kombinasi Google Devices Catalog, GSM Arena scraper, dan hardware telemetry |
| Update Telemetri Live | `POST /api/devices/{id}/telemetry` | ✅ Didukung | Live RAM, Storage, Battery %, Temp, OS, Android SDK |
| Manual Catalog Sync | `POST /api/devices/sync-catalog` | ✅ Didukung | Sinkronisasi database dengan Google Play Supported Devices CSV |

### 2.3. Clipboard History
| Fitur REQ.md | Endpoint / Event Backend | Status Backend | Keterangan Frontend |
|---|---|---|---|
| Daftar Riwayat | `GET /api/clipboard` | ✅ Didukung | Mengembalikan 50 riwayat terbaru per user |
| Kirim / Buat Teks Baru | `POST /api/clipboard` | ✅ Didukung | Body: `{ content, content_type }`. Otomatis broadcast via WS |
| Hapus Item Clipboard | `DELETE /api/clipboard/{id}` | ✅ Didukung | Konfirmasi sebelum hapus permanen |
| Real-time Sync | WS Event: `clipboard_updated` | ✅ Didukung | Teks diperbarui instan tanpa reload |

### 2.4. File Management & Transfer
| Fitur REQ.md | Endpoint / Event Backend | Status Backend | Keterangan Frontend |
|---|---|---|---|
| Upload Berkas | `POST /api/files/upload` | ✅ Didukung | Multipart/form-data: `file` & opsional `target_device_id` |
| Daftar Berkas | `GET /api/files` | ✅ Didukung | Mengembalikan daftar file yang di-share dengan ukuran & MIME type |
| Unduh Berkas | `GET /api/files/{id}/download?token=...` | ✅ Didukung | Streaming download via browser atau `Authorization` header |
| Hapus Berkas | `DELETE /api/files/{id}` | ✅ Didukung | Menghapus file fisik di storage dan record DB |
| Notifikasi File Tersedia | WS Event: `file_available` | ✅ Didukung | Pop-up/toast ketika berkas baru selesai diunggah |

### 2.5. Remote Commands
| Fitur REQ.md | Endpoint / Event Backend | Status Backend | Keterangan Frontend |
|---|---|---|---|
| Kirim Perintah | `POST /api/commands` | ✅ Didukung | Target device UUID, command (`ping`, `lock_screen`, dll.), JSON payload |
| Riwayat Perintah | `GET /api/commands` | ✅ Didukung | Daftar 50 perintah terakhir beserta status (`pending`, `sent`, `completed`, `failed`) |
| Lapor Hasil Perintah | `POST /api/commands/{id}/result` | ✅ Didukung | Dipanggil client target; disiarkan ke dashboard via WS `command_result_received` |

### 2.6. Real-Time WebSocket Protocol (`/ws`)
- **URL**: `ws://<backend_host>/ws?token=<JWT_TOKEN>`
- **Ping / Pong**: Client mengirim `{"type": "ping"}` berkala, server merespon `{"type": "pong"}`.
- **Inbound Events**:
  - `device_connected`: Notifikasi perangkat online (`device_id`, `name`, `device_type`).
  - `device_disconnected`: Perangkat offline (`device_id`).
  - `clipboard_updated`: Item clipboard baru masuk.
  - `device_status_updated`: Pembaruan status telemetri dasar.
  - `device_specs_updated`: Pembaruan spesifikasi/GSM Arena mendalam.
  - `file_available`: Berkas baru siap diunduh.
  - `command_result_received`: Respon hasil remote command dari perangkat target.
  - `error`: Pesan kesalahan dari server WS.

### 2.7. Instance Administration & Operasional (Phase 3 in REQ.md)
| Fitur REQ.md | Endpoint Backend | Status Backend | Strategi Frontend |
|---|---|---|---|
| System Health | `GET /api/health` | ✅ Didukung | Menampilkan status service, versi server, dan timestamp |
| Sync Google Devices | `POST /api/devices/sync-catalog` | ✅ Didukung | Action button di panel admin |
| Audit Logs & User List | Belum ada endpoint backend | ⚠️ Belum ada di backend | Sesuai REQ.md: Ditampilkan status informatif bahwa modul manajemen server multi-user membutuhkan backend admin extension |

---

## 3. Rencana Eksekusi Frontend Sesuai REQ.md (Phase 1 MVP)

1. **Infrastruktur API & WebSocket**:
   - `src/services/api.ts`: Konfigurasi base URL dinamis (default `http://localhost:8080`), Bearer token interceptor, generic typed request & error handling.
   - `src/services/websocket.ts`: WebSocket client terpusat dengan exponential backoff reconnection, jitter, heartbeat ping/pong, dan event subscription listener.
   - `src/context/AuthContext.tsx`: Manajemen sesi login, register, token persistence, dan state pengguna.

2. **Komponen Layout & Navigasi**:
   - Sidebar responsif (Desktop) + Drawer/Bottom Bar (Mobile).
   - Header terintegrasi dengan indikator status koneksi WebSocket (Connected, Reconnecting, Disconnected) dan menu akun.
   - Dark mode & Light mode toggle yang konsisten.

3. **Halaman & Fitur**:
   - **Overview Dashboard**: Metrik perangkat, status koneksi, ringkasan aktivitas clipboard & file terbaru, aksi cepat.
   - **Devices Management**: Kartu dan tabel perangkat, status online/offline, marketing name, chipset, modal rename, modal hapus dengan konfirmasi.
   - **Device Detail & Telemetry Modal**: Tampilan spesifikasi GSM Arena lengkap, foto produk resmi jika ada, telemetri RAM/Storage/Baterai/Suhu, IMEI masking dengan toggle privasi.
   - **Pair Device**: Generator pairing code 6-digit dengan countdown timer 5 menit, visualisasi sisa percobaan (max 3), dan instruksi untuk perangkat client.
   - **Clipboard**: Input pengiriman teks baru, daftar riwayat clipboard, salin 1-klik dengan feedback, hapus item, dan auto-update via WS.
   - **Files**: Upload file (drag & drop / file picker) dengan opsi target perangkat atau broadcast, daftar file dengan ukuran & tanggal, unduh streaming aman, hapus file.
   - **Remote Commands**: Pengiriman remote command ke perangkat target, pemantauan status real-time (`sent`, `pending`, `completed`), dan detail hasil response JSON.
   - **Settings**: Info akun, konfigurasi API/WS Base URL untuk self-hosted fleksibilitas, dan sesi logout.
   - **Instance Administration**: Info kesehatan sistem dari `/api/health`, trigger manual sync Google Play Devices Catalog, dan informasi kapabilitas host.

---

Dokumen ini menjadi acuan spesifikasi teknis pengerjaan kode frontend.
