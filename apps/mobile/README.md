# Kusuri Mobile Client 📱

Aplikasi mobile native untuk **Android** dalam ekosistem Kusuri, berfungsi sebagai jembatan sinkronisasi data antar smartphone dan komputer.

## 🎯 Tanggung Jawab & Fitur Utama

- **Foreground Service**: Mempertahankan koneksi WebSocket persisten tanpa dihentikan agresif oleh battery manager OS.
- **Accessibility / Clipboard Service**: Mendeteksi teks/tautan yang disalin dan mengirimkannya ke perangkat terhubung.
- **Share Target Integration**: Muncul di menu "Bagikan..." (Android Share Sheet) untuk langsung mengirim gambar/dokumen ke PC.
- **Pairing via QR Code**: Kamera scanner untuk memindai QR code pairing dari Kusuri Web Dashboard atau Desktop.
- **Biometric Authentication**: Otentikasi sidik jari/wajah sebelum mengakses file sensitif atau menyetujui pairing perangkat baru.

## 🛠️ Rekomendasi Arsitektur & Tech Stack

- **Framework**: Kotlin dengan [Jetpack Compose](https://developer.android.com/jetpack/compose) untuk modern UI.
- **Networking**: Ktor Client / OkHttp (WebSocket) + Retrofit untuk REST API.
- **Local Database**: Room / DataStore.
- **Background Work**: WorkManager & Foreground Services.

## 📂 Struktur Direktori yang Disarankan

```
apps/mobile/
├── app/
│   ├── src/main/java/app/kusuri/
│   │   ├── data/       # Repositories, API client, WS service
│   │   ├── di/         # Dependency injection (Hilt / Koin)
│   │   ├── service/    # Persistent foreground sync service
│   │   └── ui/         # Jetpack Compose screens & components
│   └── build.gradle.kts
├── gradle/
├── build.gradle.kts
└── settings.gradle.kts
```

## 🚀 Status Pengembangan

> [!NOTE]
> Modul ini disiapkan sebagai wadah implementasi Android client Kusuri.
