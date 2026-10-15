# Changelog

Semua perubahan penting pada proyek **Kusuri** akan didokumentasikan di berkas ini.

Format berkas ini mengacu pada [Keep a Changelog](https://keepachangelog.com/id/1.1.0/)
dan proyek ini mematuhi [Semantic Versioning](https://semver.org/lang/id/).

## [Unreleased]

### Added
- Arsitektur monorepo terpadu berbasis Cargo Workspaces dan Bun Workspaces.
- Dokumentasi arsitektur, panduan self-hosting, spesifikasi API, dan RFC template.
- Skrip otomasi pengembang (`scripts/setup`, `scripts/dev`, `scripts/build`) untuk PowerShell dan Bash.
- Workflow GitHub Actions CI untuk backend Rust, web frontend Bun, dan Docker build.
- Konfigurasi Docker Compose orkestrasi penuh (Postgres, Backend, Web Dashboard).
- Wadah placeholder untuk aplikasi Desktop (`apps/desktop`), Mobile (`apps/mobile`), CLI (`apps/cli`), dan shared packages (`packages/types`, `packages/protocol`, `packages/sdk-ts`, `packages/sdk-rust`).

## [0.1.0] - 2026-10-15

### Added
- **Kusuri Backend**:
  - Server performa tinggi berbasis Rust Axum 0.8, Tokio, dan SQLx (PostgreSQL).
  - Autentikasi JWT untuk sesi browser dan SHA-256 hashed 64-character tokens untuk perangkat native.
  - Saluran WebSocket real-time (`/ws`) dua arah dengan penanganan virtual session.
  - Streaming upload/download berkas dengan validasi SHA-256 checksum.
  - Engine integrasi spesifikasi perangkat otomatis (GSM Arena scraping + Google Play catalog).
- **Kusuri Web Dashboard**:
  - Dashboard web responsif berbasis React 19, Tailwind CSS v4, dan Bun runtime.
  - Manajemen perangkat, telemetry monitor, dan antarmuka pairing perangkat.
  - Riwayat clipboard real-time, file transfer explorer, dan command history viewer.
  - Self-hosted instance administrative panel.
