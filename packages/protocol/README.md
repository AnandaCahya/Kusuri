# Kusuri Shared Protocol Specification 📡

Direktori ini berisi spesifikasi kontrak komunikasi, format data serialisasi, serta skema pesan real-time WebSocket dan REST antar seluruh komponen Kusuri.

## 🎯 Tujuan

Menjaga satu sumber kebenaran (*single source of truth*) untuk format pesan jaringan, sehingga server, web dashboard, desktop, mobile, dan CLI selalu kompatibel.

## 📦 Isi yang Disarankan

1. **JSON Schemas / TypeBox / Zod definitions**:
   - `events/`
     - `clipboard_changed.json`
     - `device_online.json`
     - `device_offline.json`
     - `file_available.json`
     - `command_requested.json`
     - `command_result.json`
2. **Protocol Buffers (`.proto`)** *(Opsional untuk high-throughput binary sync)*:
   - `kusuri.proto`

## 🔄 Alur Sinkronisasi Pesan

```mermaid
sequenceDiagram
    participant D1 as Device 1 (Android / Desktop)
    participant S as Kusuri Backend (/ws)
    participant D2 as Device 2 (Web / Laptop)

    Note over D1,S: Terkoneksi via WebSocket dengan Token Terenkripsi
    D1->>S: {"type": "clipboard_sync", "payload": {"content": "https://example.com"}}
    S->>S: Validasi kepemilikan akun & filter broadcast
    S-->>D2: {"type": "clipboard_updated", "payload": {"device_id": "d1", "content": "..."}}
```
