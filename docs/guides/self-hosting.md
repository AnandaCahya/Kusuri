# Panduan Self-Hosting Kusuri 🏠

Panduan ini menjelaskan cara melakukan deploy Kusuri di server VPS pribadi (misal: Debian, Ubuntu, Arch) menggunakan Docker Compose dan Caddy sebagai reverse proxy dengan sertifikat SSL/TLS otomatis.

## 📋 Prasyarat

1. Server VPS dengan Linux x86_64 / ARM64.
2. Domain atau Subdomain (misal: `kusuri.domainkamu.com`) yang telah di-point ke IP publik server (A Record).
3. [Docker](https://docs.docker.com/engine/install/) & Docker Compose terinstal.
4. Port `80` dan `443` terbuka di firewall server.

## 🚀 Langkah 1: Kloning Repositori

```bash
git clone https://github.com/kusuri-app/kusuri.git /opt/kusuri
cd /opt/kusuri
```

## ⚙️ Langkah 2: Konfigurasi Environment

Salin file contoh konfigurasi:

```bash
cp apps/backend/.env.example apps/backend/.env
```

Edit variabel penting:
- `JWT_SECRET`: Buat string acak minimal 32 karakter.
- `POSTGRES_PASSWORD`: Ganti kata sandi database dengan sandi kuat.

## 🐳 Langkah 3: Menjalankan Servis

Jalankan container menggunakan root `docker-compose.yml`:

```bash
docker compose up -d
```

Verifikasi seluruh container berjalan sehat:

```bash
docker compose ps
```

## 🔒 Langkah 4: Setup SSL Reverse Proxy (Caddy)

Caddy secara otomatis mengurus penerbitan dan pembaruan sertifikat Let's Encrypt:

1. Salin konfigurasi Caddy dari [deploy/caddy/Caddyfile](file:///c:/Users/Ananda/Documents/Coding/Aplikasi/Kusuri/deploy/caddy/Caddyfile):
```bash
sudo cp deploy/caddy/Caddyfile /etc/caddy/Caddyfile
```
2. Ganti `kusuri.example.com` dengan domain milik Anda.
3. Muat ulang servis Caddy:
```bash
sudo systemctl reload caddy
```

Buka `https://kusuri.domainkamu.com` melalui browser Anda!
