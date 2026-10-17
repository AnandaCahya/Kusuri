# Kusuri Desktop Client 🖥️

Aplikasi desktop native untuk sistem operasi **Windows** dan **Linux** yang bertindak sebagai endpoint tersinkronisasi 24/7 dalam ekosistem Kusuri, menyuplai telemetri performa, hardware, daya, jaringan, dan menerima sinkronisasi clipboard serta remote commands.

---

## 🎯 Fitur Utama

- **Antarmuka Grafis Native (GUI)**: Dilengkapi antarmuka grafis modern, responsif, dan hemat daya berbasis `egui` dengan tema gelap (Dark Theme), dashboard meter CPU/RAM/Disk/Baterai live, form pairing interaktif, log feed realtime, dan tab spesifikasi hardware.
- **Koneksi Persisten 24/7**: Terhubung non-stop ke `apps/backend` melalui WebSocket dengan automatic reconnect exponential backoff dan heartbeat ping/pong setiap 25 detik.
- **Maksimalisasi Metrik Telemetri Hardware**:
  - **Sistem & BIOS**: Deteksi pabrikan (Manufacturer) & model laptop/motherboard (e.g., *Lenovo ThinkPad*, *ASUS ROG Zephyrus*, *Dell XPS*) untuk integrasi katalog spesifikasi Kusuri.
  - **Prosesor (CPU)**: Brand model, vendor, core fisik & logikal, frekuensi, total CPU usage %, dan per-core usage %.
  - **Memori (RAM & Swap)**: RAM Total, Used, Free, Available (MB & %), serta Swap memory.
  - **Penyimpanan (Disks & Drives)**: Total kapasitas, terpakai, sisa, serta detail seluruh drive/partisi (NTFS, EXT4, Btrfs, SSD/HDD).
  - **Daya & Baterai**: Persentase baterai, status charging (AC/Battery), kesehatan baterai (Health), dan adaptasi otomatis untuk PC Desktop tanpa baterai.
  - **Tampilan (Display)**: Resolusi layar primer dan refresh rate monitor (Hz).
  - **Jaringan & IP**: Deteksi otomatis alamat IPv4 lokal aktif, jenis antarmuka (*WiFi*, *Ethernet*, *Cellular*), serta total transfer Rx/Tx.
  - **Sensor & Suhu**: Pemantauan suhu CPU/termal (bila didukung hardware).
  - **Proses Sistem**: Jumlah proses berjalan dan jejak konsumsi memori aplikasi yang ultra-ringan (< 15MB RAM).
- **Dual Transmisi (WebSocket + HTTP Fallback)**:
  - Transmisi realtime via WebSocket event `hardware_telemetry` dan `telemetry`.
  - Fail-safe otomatis via HTTP API `POST /api/v1/devices/{device_id}/telemetry` apabila jaringan WebSocket terganggu.
- **Remote Command Receiver**:
  - Menerima dan menjalankan perintah: `ping`, `status`, `battery_check`, `system_info`, `notification`, dan `lock_screen`.
- **Clipboard Sync**: Sinkronisasi clipboard multi-perangkat.

---

## 🚀 Panduan Instalasi & Penggunaan

### 📦 1-Click Installer & Setup

#### **Windows:**
Anda dapat menginstal Kusuri Desktop secara mudah layaknya aplikasi Windows standar:
1. **Cara Cepat (Klik 2x)**:
   - Klik dua kali file `setup-desktop.bat` di root direktori atau `apps/desktop/setup.bat`.
2. **Melalui PowerShell**:
   ```powershell
   .\apps\desktop\scripts\Setup-KusuriDesktop.ps1
   ```
   **Apa yang dilakukan installer Windows:**
   - 📂 Menyalin binary yang dioptimalkan ke `%LOCALAPPDATA%\Programs\KusuriDesktop\` (tanpa perlu hak Administrator).
   - 🖥️ Membuat shortcut di **Desktop**.
   - 📑 Mendaftarkan shortcut di **Start Menu** (`Kusuri -> Kusuri Desktop`).
   - ⚙️ Menambahkan opsi autostart 24/7 di Windows Startup (opsional, bisa dinonaktifkan dengan flag `-NoAutoStart`).
   - 📋 Terdaftar secara resmi di Windows **Installed Apps / Control Panel (Add/Remove Programs)** dengan opsi Uninstall bersih.
   - 🗑️ Menyertakan skrip uninstaller: `.\apps\desktop\scripts\Setup-KusuriDesktop.ps1 -Uninstall`.

#### **Linux:**
```bash
chmod +x apps/desktop/scripts/install-linux.sh
./apps/desktop/scripts/install-linux.sh
```
**Apa yang dilakukan installer Linux:**
- 📂 Memasang executable ke `~/.local/bin/kusuri-desktop`.
- 🖥️ Mendaftarkan `.desktop` launcher di `~/.local/share/applications/kusuri-desktop.desktop` agar muncul di Application Menu desktop environment (GNOME, KDE, XFCE).
- ⚙️ Mengonfigurasi dan menyalakan `systemd --user` service agar berjalan otomatis 24/7 di background.
- 🗑️ Uninstall bersih kapan saja dengan: `./apps/desktop/scripts/install-linux.sh --uninstall`.

---

### 💻 Kompilasi Manual (Manual Build)
Jika Anda ingin mengompilasi sendiri secara manual:
```bash
cargo build --release -p kusuri-desktop
```
Binary executable akan berada di `target/release/kusuri-desktop` (atau `kusuri-desktop.exe` di Windows).

### 2. Membuka Aplikasi Desktop (Mode GUI)
Jalankan tanpa argumen atau dengan perintah `gui`:
```bash
kusuri-desktop
# atau
kusuri-desktop gui
```
> Di Windows, Anda juga dapat langsung mengklik dua kali file `kusuri-desktop.exe` di File Explorer!

### 3. Memasangkan Perangkat (Pairing)
- **Melalui GUI**: Buka tab **🔗 Pairing**, masukkan kode 6-digit dari dashboard Web, lalu klik **"Hubungkan Sekarang"**.
- **Melalui Terminal (CLI)**:
```bash
kusuri-desktop pair --code 123456 --server http://127.0.0.1:8080
```
> Kredensial, token unik `ksr_...`, dan Device ID akan disimpan otomatis secara aman di:
> - **Windows**: `%APPDATA%\kusuri\config.json`
> - **Linux**: `~/.config/kusuri/config.json`

### 4. Menjalankan Service 24/7 di Foreground / Headless
```bash
kusuri-desktop run
```

### 5. Memeriksa Status & Metrik
```bash
# Cek status koneksi dan konfigurasi
kusuri-desktop status

# Tampilkan snapshot telemetri lengkap di terminal
kusuri-desktop telemetry

# Tampilkan dalam format JSON mentah
kusuri-desktop telemetry --json
```

---

## ⚙️ Menjalankan Otomatis di Background 24/7 (Daemon / Service)

### Di Windows (Scheduled Task)
Jalankan PowerShell sebagai Administrator atau user aktif:
```powershell
.\apps\desktop\scripts\install-windows-task.ps1 -Action install
```
Task scheduler akan otomatis menjalankan `kusuri-desktop.exe run` setiap kali user login atau sistem menyala, dengan opsi restart otomatis jika koneksi terputus.

Untuk menghapus:
```powershell
.\apps\desktop\scripts\install-windows-task.ps1 -Action uninstall
```

### Di Linux (Systemd User Service)
```bash
# 1. Salin binary ke PATH user
mkdir -p ~/.local/bin
cp target/release/kusuri-desktop ~/.local/bin/

# 2. Aktifkan service systemd
mkdir -p ~/.config/systemd/user
cp apps/desktop/scripts/kusuri-desktop.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now kusuri-desktop.service

# 3. Cek log service 24 jam
journalctl --user -u kusuri-desktop -f
```
