<div align="center">

# 🌌 Kusuri (薬)

**Ekosistem sinkronisasi multi-perangkat real-time, clipboard bersama, dan transfer berkas cross-platform berperforma tinggi.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Bun](https://img.shields.io/badge/Bun-1.1+-fbf0df.svg?logo=bun)](https://bun.sh/)
[![React](https://img.shields.io/badge/React-19-61dafb.svg?logo=react)](https://react.dev/)
[![TailwindCSS](https://img.shields.io/badge/TailwindCSS-v4-38bdf8.svg?logo=tailwindcss)](https://tailwindcss.com/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16-336791.svg?logo=postgresql)](https://www.postgresql.org/)

[Fitur Utama](#-fitur-utama) •
[Struktur Monorepo](#-struktur-monorepo) •
[Memulai Cepat](#-memulai-cepat-quick-start) •
[Self-Hosting](#-self-hosting-dengan-docker) •
[Dokumentasi](#-dokumentasi-lengkap) •
[Kontribusi](#-kontribusi)

---

</div>

## 📖 Tentang Kusuri

**Kusuri** adalah platform sinkronisasi perangkat pribadi *open source* yang mengintegrasikan alur kerja harian Anda antar berbagai perangkat (Android, Windows, Linux, dan Web) secara instan, aman, dan tanpa ketergantungan pada cloud pihak ketiga (*self-hostable*).

Kusuri bertindak sebagai jembatan data berlatensi rendah untuk:
- 📋 **Universal Clipboard Sync**: Salin teks atau tautan di smartphone Android, tempel langsung di PC Windows atau laptop Linux secara instan.
- 📁 **File & Media Streaming**: Kirim berkas berukuran besar antar perangkat secara langsung dengan verifikasi integritas checksum SHA-256.
- 📱 **Device Management & Telemetry**: Pantau spesifikasi hardware perangkat (diperkaya otomatis dengan GSM Arena catalog), status baterai, jaringan, dan status online secara real-time.
- ⚡ **Remote Command Dispatch**: Eksekusi perintah aman antar perangkat terpairing.
- 🛡️ **Zero-Trust Security**: Otentikasi ganda (JWT untuk sesi Web Dashboard + 64-karakter token SHA-256 untuk perangkat native), proteksi password dengan Argon2id, dan isolasi ketat antar pengguna.

---

## 🏗️ Struktur Monorepo

Repositori ini dikelola menggunakan model **Unified Polyglot Monorepo** menggabungkan **Cargo Workspaces** (untuk ekosistem Rust) dan **Bun Workspaces** (untuk ekosistem JavaScript/TypeScript):

```text
kusuri/
├── apps/                          # 📱 Aplikasi & Layanan Utama
│   ├── backend/                   # Rust Axum Server (Tokio + SQLx + WebSocket)
│   ├── web/                       # Web Dashboard (React 19 + Tailwind CSS v4 + Bun)
│   ├── desktop/                   # [Placeholder] Client Windows & Linux (Tauri / Rust)
│   ├── mobile/                    # [Placeholder] Client Android (Kotlin / Jetpack Compose)
│   └── cli/                       # [Placeholder] Kusuri CLI & Terminal Daemon
│
├── packages/                      # 📦 Pustaka & Modul Bersama (Shared)
│   ├── protocol/                  # Spesifikasi skema event WebSocket & IPC
│   ├── types/                     # Shared TypeScript & Rust data contracts (DTOs)
│   ├── sdk-ts/                    # Kusuri client SDK untuk TypeScript/JavaScript
│   └── sdk-rust/                  # Kusuri client SDK crate untuk Rust
│
├── docs/                          # 📚 Pusat Dokumentasi Teknis
│   ├── architecture/              # Diagram dan penjelasan arsitektur sistem
│   ├── api/                       # Dokumentasi REST API & WebSocket
│   ├── guides/                    # Panduan Self-Hosting dan Pengembangan Lokal
│   └── rfcs/                      # Request for Comments & proposal teknis
│
├── deploy/                        # 🚀 Konfigurasi Produksi & Server
│   ├── docker/                    # Dockerfile & Docker Compose overrides
│   ├── caddy/                     # Reverse proxy config (Auto SSL & WebSocket proxy)
│   ├── nginx/                     # Alternatif konfigurasi Nginx produksi
│   └── systemd/                   # Linux systemd service units
│
├── scripts/                       # 🛠️ Skrip Otomasi Pengembang (Cross-Platform)
│   ├── setup.ps1 / setup.sh       # Skrip setup awal & pengecekan dependensi
│   ├── dev.ps1 / dev.sh           # Menjalankan backend + web secara simultan
│   └── build.ps1 / build.sh       # Kompilasi seluruh artefak rilis
│
├── .github/                       # 🤖 Otomasi GitHub & CI/CD
│   ├── workflows/                 # CI pipelines (Backend Rust, Web Bun, Docker)
│   ├── ISSUE_TEMPLATE/            # Template issue bug & fitur
│   └── PULL_REQUEST_TEMPLATE.md   # Template pengajuan PR
│
├── Cargo.toml                     # Root Cargo Workspace
├── package.json                   # Root Bun Workspace
├── docker-compose.yml             # Orkestrasi Docker lokal (DB + Backend + Web)
├── .editorconfig                  # Standar format kode lintas editor
├── .gitignore                     # Filter git komprehensif
└── CONTRIBUTING.md                # Panduan tata cara kontribusi
```

---

## ⚡ Memulai Cepat (Quick Start)

### 📋 Prasyarat
- [Rust](https://rustup.rs/) (v1.80+)
- [Bun](https://bun.sh/) (v1.1+)
- [Docker](https://www.docker.com/) (untuk PostgreSQL)

### 1. Inisialisasi Environment
Jalankan skrip inisialisasi yang secara otomatis memeriksa perkakas, menginstal dependensi web, dan menyiapkan berkas `.env`:

```powershell
# Windows (PowerShell)
.\scripts\setup.ps1

# Linux / macOS (Bash)
./scripts/setup.sh
```

### 2. Jalankan Lingkungan Pengembangan
Jalankan seluruh layanan (Database, Backend, dan Web) dengan satu perintah:

```powershell
# Windows (PowerShell)
.\scripts\dev.ps1

# Linux / macOS (Bash)
./scripts/dev.sh
```

Atau jalankan secara independen melalui script workspace:
```bash
# Menjalankan Web Dashboard (port 3000)
bun run dev:web

# Menjalankan Backend Server (port 8080)
bun run dev:backend
```

Akses aplikasi di:
- 🌐 **Web Dashboard**: `http://localhost:3000`
- ⚙️ **Backend Healthcheck**: `http://localhost:8080/api/health`
- 📡 **WebSocket Gateway**: `ws://localhost:8080/ws`

---

## 🐳 Self-Hosting dengan Docker

Untuk menjalankan Kusuri secara penuh di server produksi mandiri:

```bash
# 1. Clone repositori
git clone https://github.com/kusuri-app/kusuri.git
cd kusuri

# 2. Siapkan file environment
cp apps/backend/.env.example apps/backend/.env
# Edit apps/backend/.env dan ganti JWT_SECRET & POSTGRES_PASSWORD

# 3. Jalankan container
docker compose up -d
```

Kusuri siap digunakan dalam hitungan detik! Untuk setup domain dan sertifikat SSL gratis, baca [Panduan Reverse Proxy Caddy](file:///c:/Users/Ananda/Documents/Coding/Aplikasi/Kusuri/deploy/caddy/Caddyfile).

---

## 📚 Dokumentasi Lengkap

- 🏛️ **[Arsitektur Sistem & Alur Kerja](file:///c:/Users/Ananda/Documents/Coding/Aplikasi/Kusuri/docs/architecture/overview.md)**
- 🔌 **[Spesifikasi Lengkap REST API & WebSocket](file:///c:/Users/Ananda/Documents/Coding/Aplikasi/Kusuri/docs/api/README.md)**
- 🏠 **[Panduan Deployment & Self-Hosting](file:///c:/Users/Ananda/Documents/Coding/Aplikasi/Kusuri/docs/guides/self-hosting.md)**
- 💻 **[Panduan Pengembangan Lokal](file:///c:/Users/Ananda/Documents/Coding/Aplikasi/Kusuri/docs/guides/development.md)**
- 📝 **[Template Usulan Desain (RFC)](file:///c:/Users/Ananda/Documents/Coding/Aplikasi/Kusuri/docs/rfcs/0000-template.md)**

---

## 🗺️ Roadmap Pengembangan

- [x] **v0.1.0 (MVP Backend & Web Dashboard)**
  - [x] Arsitektur Monorepo Cargo + Bun
  - [x] Rust Axum high-performance backend
  - [x] Real-time bi-directional WebSocket broker
  - [x] Responsive React 19 + Tailwind v4 Web Dashboard
  - [x] Hardware spec enrichment scraper
  - [x] Docker & Caddy production manifests
- [ ] **v0.2.0 (Native Clients Expansion)**
  - [ ] Implementasi Tauri Desktop Client untuk Windows & Linux (`apps/desktop`)
  - [ ] Implementasi Android Native Client dengan Background Service (`apps/mobile`)
  - [ ] Implementasi Kusuri CLI (`apps/cli`)
  - [ ] Ekstraksi protocol buffers & shared SDKs (`packages/sdk-ts`, `packages/sdk-rust`)
- [ ] **v0.3.0 (Security & Advanced Sync)**
  - [ ] End-to-End Encryption (E2EE) antar perangkat menggunakan X25519 & ChaCha20-Poly1305
  - [ ] LAN P2P direct file streaming via WebRTC / Local Socket jika berada dalam satu WiFi

---

## 🤝 Kontribusi

Kontribusi dari komunitas sangat disambut hangat! Silakan baca panduan lengkap kami di:
- 📖 **[CONTRIBUTING.md](file:///c:/Users/Ananda/Documents/Coding/Aplikasi/Kusuri/CONTRIBUTING.md)** untuk alur kerja branch, commit convention, dan pengujian.
- 📜 **[CODE_OF_CONDUCT.md](file:///c:/Users/Ananda/Documents/Coding/Aplikasi/Kusuri/CODE_OF_CONDUCT.md)** untuk pedoman etika komunitas.
- 🔒 **[SECURITY.md](file:///c:/Users/Ananda/Documents/Coding/Aplikasi/Kusuri/SECURITY.md)** untuk pelaporan kerentanan keamanan secara bertanggung jawab.

---

## 📄 Lisensi

Kusuri dirilis di bawah lisensi terbuka [MIT License](file:///c:/Users/Ananda/Documents/Coding/Aplikasi/Kusuri/LICENSE).
