# Kusuri Rust SDK (`kusuri-client`) 🦀

Crate Rust yang menyediakan client asynchronous siap pakai untuk berinteraksi dengan Kusuri Backend.

## 🎯 Kegunaan

Digunakan bersama oleh:
- `apps/cli`: CLI utility
- `apps/desktop`: Background daemon / Tauri core

## 💡 Contoh Penggunaan

```rust
use kusuri_client::{KusuriClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = KusuriClient::new(Config {
        endpoint: "http://localhost:8080".into(),
        token: "ksr_...".into(),
    });

    // Kirim clipboard
    client.send_clipboard("Halo dari Rust!").await?;

    Ok(())
}
```
