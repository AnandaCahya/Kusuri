# Kusuri Shared Types 🧬

Modul paket bersama untuk definisi tipe data (TypeScript & Rust DTOs) yang digunakan secara lintas proyek di monorepo.

## 🎯 Kegunaan

- Memastikan tipe data akun, perangkat, clipboard item, metadata file, dan respon API identik di frontend (`apps/web`), desktop client (`apps/desktop`), dan SDK (`packages/sdk-ts`).
- Mengurangi redudansi kode tipe manual antar aplikasi.

## 📂 Struktur yang Direkomendasikan

```
packages/types/
├── src/
│   ├── auth.ts        # User, Session, Token types
│   ├── device.ts      # Device, DeviceType, PairingPayload, Telemetry
│   ├── clipboard.ts   # ClipboardItem, ContentType, History
│   ├── file.ts        # FileMetadata, UploadResponse, TransferProgress
│   ├── event.ts       # WebSocket incoming & outgoing event unions
│   └── index.ts       # Public exports
├── package.json
└── tsconfig.json
```
