# Deployment Configurations 🚀

Direktori ini berisi berkas konfigurasi siap pakai untuk berbagai strategi deployment server produksi:

- **`docker/`**: Konfigurasi Docker multi-container dan overrides.
- **`caddy/`**: Konfigurasi reverse proxy Caddy dengan auto-SSL (Let's Encrypt) dan dukungan WebSocket streaming.
- **`nginx/`**: Konfigurasi Nginx produksi untuk reverse proxy dan caching static files.
- **`systemd/`**: Unit file systemd untuk menjalankan Kusuri binary secara native di Linux background service.
