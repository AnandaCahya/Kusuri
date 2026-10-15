# Kusuri TypeScript SDK 📦

SDK resmi Kusuri untuk lingkungan JavaScript / TypeScript (Node.js, Bun, Browser, Electron, React Native).

## 🎯 Kegunaan

Memudahkan pembuatan aplikasi pihak ketiga, integrasi script otomatisasi, atau digunakan oleh frontend internal untuk mengakses REST API dan WebSocket Kusuri dengan typed client.

## 💡 Contoh Penggunaan

```typescript
import { KusuriClient } from "@kusuri/sdk";

const client = new KusuriClient({
  baseUrl: "https://kusuri.example.com",
  token: "ksr_devicetoken...",
});

// Menghubungkan ke WebSocket real-time
await client.connect();

// Mendengarkan event clipboard
client.on("clipboard_updated", (event) => {
  console.log("Clipboard baru diterima:", event.content);
});

// Mengirim file
await client.files.upload({
  filePath: "./document.pdf",
  targetDeviceId: "dev_laptop",
});
```
