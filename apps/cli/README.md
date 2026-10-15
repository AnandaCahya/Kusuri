# Kusuri CLI 💻

Antarmuka baris perintah (CLI) untuk Kusuri, ditujukan bagi sysadmin, developer, dan server tanpa antarmuka grafis (headless).

## 🎯 Tanggung Jawab & Fitur Utama

- **Headless Device Daemon**: Berjalan sebagai daemon sistem (systemd / Windows service) tanpa GUI.
- **Pipe-Friendly Clipboard**: Mengalirkan data terminal (`cat file.txt | kusuri clip send`) langsung ke clipboard perangkat target.
- **Direct File Send**: Mengirim berkas dari terminal (`kusuri file send ./report.pdf --to laptop-kerja`).
- **Device Management**: Melihat status perangkat yang terdaftar (`kusuri devices list`).
- **Scriptable Automation**: Integrasi dengan skrip CI/CD, cron jobs, dan terminal workflows.

## 🛠️ Rekomendasi Tech Stack

- **Bahasa**: Rust (menggunakan `clap` v4 untuk argument parsing, `tokio`, `reqwest`, `tokio-tungstenite`).
- Menggunakan crate shared dari `packages/sdk-rust`.

## 📂 Struktur Direktori yang Disarankan

```
apps/cli/
├── src/
│   ├── commands/      # Subcommands: auth, clip, file, devices, daemon
│   ├── config.rs      # Konfigurasi token & server endpoint (~/.config/kusuri/config.toml)
│   └── main.rs
├── Cargo.toml
└── README.md
```
