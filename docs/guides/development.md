# Panduan Pengembangan Lokal (Local Development) 💻

Panduan bagi pengembang yang ingin berkontribusi atau menjalankan Kusuri secara lokal dalam lingkungan pengembangan.

## 🛠️ Kebutuhan Perkakas (Toolchain)

- **Rust**: Versi 1.80+ (dapat diinstal melalui [rustup.rs](https://rustup.rs/))
- **Bun**: Versi 1.1+ ([bun.sh](https://bun.sh))
- **PostgreSQL**: Versi 16 (atau via Docker container)
- **Git**

## 🏃 Menjalankan Aplikasi Secara Cepat

### 1. Jalankan PostgreSQL
Anda dapat menggunakan Docker untuk menjalankan database:
```bash
docker compose up -d kusuri-db
```

### 2. Jalankan Backend (Rust Axum)
```bash
# Dari root monorepo
cargo run --bin backend

# Atau menggunakan Bun task
bun run dev:backend
```
Server backend akan aktif di `http://127.0.0.1:8080`.

### 3. Jalankan Web Dashboard (React + Bun)
```bash
# Di terminal baru
bun run dev:web
```
Dashboard akan aktif di `http://localhost:3000` dengan fitur Hot Module Replacement (HMR).

### 4. Menjalankan Pengecekan Kode (Lint & Typecheck)
```bash
# Cek tipe TypeScript
bun run check:web

# Cek kode Rust
cargo check
cargo clippy
```
