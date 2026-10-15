# Kusuri Desktop Client 🖥️

Aplikasi desktop native untuk sistem operasi **Windows** dan **Linux** yang bertindak sebagai endpoint tersinkronisasi dalam ekosistem Kusuri.

## 🎯 Tanggung Jawab & Fitur Utama

- **System Tray Integration**: Berjalan ringan di latar belakang dengan indikator status koneksi dan menu cepat.
- **Local Clipboard Watcher**: Mendeteksi perubahan clipboard lokal dan menyiarkannya ke perangkat lain via WebSocket secara aman.
- **Remote Clipboard Receiver**: Menerima pembaruan clipboard dari perangkat lain dan menyimpannya ke clipboard sistem lokal.
- **Direct & Relay File Transfer**: Mengirim dan menerima berkas tanpa batasan ukuran file, terintegrasi dengan file explorer.
- **Remote Command Executor**: Menjalankan perintah yang diizinkan pengguna (dengan mekanisme perizinan ketat).
- **Native Notifications**: Menampilkan toast / notification bubble saat pairing, file masuk, atau pesan broadcast.

## 🛠️ Rekomendasi Arsitektur & Tech Stack

- **Pilihan 1 (Direkomendasikan)**: [Tauri v2](https://v2.tauri.app/) (Rust core + Web frontend). Sangat hemat memori (~20MB RAM) dan dapat menggunakan kembali logika Rust dari `packages/sdk-rust` serta komponen UI dari web.
- **Pilihan 2**: C# / .NET 8 (WPF / WinUI 3) untuk Windows native ultra-performant.
- **Pilihan 3**: Flutter Desktop.

## 📂 Struktur Direktori yang Disarankan

```
apps/desktop/
├── src-tauri/         # Rust backend core (Tauri v2)
│   ├── src/           # Tray, clipboard listeners, IPC handlers
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/               # Frontend UI (React / Svelte / Vanilla)
├── package.json
└── README.md
```

## 🚀 Status Pengembangan

> [!NOTE]
> Modul ini disiapkan sebagai wadah implementasi client desktop Kusuri. Silakan mulai inisialisasi project sesuai rekomendasi di atas.
