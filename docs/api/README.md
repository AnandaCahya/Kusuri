# Kusuri API & Protocol Specification 🔌

Kusuri menyediakan antarmuka ganda:
1. **REST API** untuk operasi CRUD, autentikasi, status perangkat, dan upload file.
2. **WebSocket (`/ws`)** untuk stream event dua arah real-time berlatensi rendah.

Untuk spesifikasi endpoint lengkap, header otentikasi, contoh request/response body, dan kode status HTTP, silakan merujuk ke dokumentasi teknis di:

👉 **[apps/backend/API.md](file:///c:/Users/Ananda/Documents/Coding/Aplikasi/Kusuri/apps/backend/API.md)**

## ⚡ Ikhtisar Endpoint Utama

### 🔑 Autentikasi
- `POST /api/auth/register` — Pendaftaran akun baru
- `POST /api/auth/login` — Login dan penerbitan token JWT
- `GET /api/auth/me` — Informasi profil pengguna

### 📱 Manajemen Perangkat
- `GET /api/devices` — Daftar perangkat terdaftar beserta telemetry status
- `POST /api/devices/pair` — Pairing perangkat baru via token/kode
- `DELETE /api/devices/{device_id}` — Pencabutan otorisasi perangkat

### 📋 Clipboard & Berkas
- `GET /api/clipboard` — Riwayat clipboard tersinkronisasi
- `POST /api/clipboard` — Penambahan item clipboard baru
- `POST /api/files/upload` — Upload multipart file dengan verifikasi checksum
- `GET /api/files/{file_id}/download` — Download streaming berkas hemat memori

### 🌐 Real-Time WebSocket
- `GET /ws?token={JWT_OR_DEVICE_TOKEN}` — Upgrade koneksi WebSocket dua arah
