# Panduan Kontribusi (Contributing Guide) 🤝

Terima kasih atas ketertarikan Anda untuk berkontribusi pada **Kusuri**! Dokumen ini memandu Anda dalam alur kerja pengembangan monorepo, standar kode, dan proses pengajuan Pull Request.

## 🧭 Alur Kerja Monorepo

Kusuri menggunakan struktur monorepo terpadu:
- **Rust Backend & Crates**: Dikelola oleh [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html).
- **Web & TypeScript Packages**: Dikelola oleh [Bun Workspaces](https://bun.sh/docs/install/workspaces).

## 🚀 Memulai (Getting Started)

1. **Fork** repositori ini ke akun GitHub Anda.
2. **Clone** hasil fork ke mesin lokal Anda:
   ```bash
   git clone https://github.com/<username-anda>/kusuri.git
   cd kusuri
   ```
3. Jalankan skrip setup otomatis:
   - **Windows (PowerShell)**: `.\scripts\setup.ps1`
   - **Linux / macOS (Bash)**: `./scripts/setup.sh`
4. Jalankan server lokal:
   - **Windows**: `.\scripts\dev.ps1`
   - **Linux / macOS**: `./scripts/dev.sh`

## 🌿 Strategi Percabangan (Branching Strategy)

Gunakan nama branch yang deskriptif dengan awalan tipe:
- `feat/nama-fitur` (misal: `feat/desktop-tray-menu`)
- `fix/deskripsi-bug` (misal: `fix/ws-reconnect-backoff`)
- `docs/deskripsi-dokumen` (misal: `docs/pairing-guide`)
- `refactor/area-kode` (misal: `refactor/file-service-streams`)

## ✍️ Standar Pesan Commit (Conventional Commits)

Kami menggunakan format [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <deskripsi singkat>

[opsional isi penjelasan]

[opsional footer: Fixes #123]
```

**Pilihan Tipe:**
- `feat`: Fitur baru bagi pengguna
- `fix`: Perbaikan bug
- `docs`: Perubahan atau penambahan dokumentasi
- `refactor`: Refactoring kode tanpa mengubah fungsionalitas
- `perf`: Optimasi performa
- `test`: Penambahan atau perbaikan unit/integration test
- `chore`: Pemeliharaan dependensi atau konfigurasi build

**Contoh:**
```
feat(backend): add streaming sha256 calculation for large uploads
fix(web): prevent duplicate toast on rapid websocket reconnect
docs(api): document device telemetry payload format
```

## 🔍 Pengecekan Kualitas Kode Sebelum Kirim PR

Pastikan seluruh linter dan test berhasil lolos di lokal:

```bash
# Validasi TypeScript
bun run check:web

# Validasi Rust
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

## 📬 Mengajukan Pull Request

1. Buka Pull Request ke branch `main`.
2. Isi template PR dengan lengkap: jelaskan perubahan, komponen yang terdampak, dan sertakan screenshot jika relevan.
3. Pastikan seluruh workflow GitHub Actions CI berstatus hijau (pass).
