#!/bin/bash
# Kusuri Desktop Client - Linux Installer & Setup Script
# Installs binary to ~/.local/bin, creates .desktop shortcut, and configures systemd 24/7 user service.

set -e

APP_NAME="Kusuri Desktop"
BIN_NAME="kusuri-desktop"
VERSION="0.1.0"
INSTALL_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"
SERVICE_DIR="$HOME/.config/systemd/user"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

echo "============================================================"
echo "          KUSURI DESKTOP CLIENT SETUP (LINUX)               "
echo "                   Versi $VERSION                           "
echo "============================================================"
echo ""

# Handle uninstall flag
if [ "$1" = "--uninstall" ] || [ "$1" = "-u" ]; then
    echo "[-] Memulai proses uninstall $APP_NAME..."
    
    # 1. Stop and disable systemd service
    if systemctl --user is-active --quiet kusuri-desktop.service 2>/dev/null; then
        echo "[-] Menghentikan systemd user service..."
        systemctl --user stop kusuri-desktop.service || true
    fi
    if systemctl --user is-enabled --quiet kusuri-desktop.service 2>/dev/null; then
        echo "[-] Menonaktifkan autostart systemd service..."
        systemctl --user disable kusuri-desktop.service || true
    fi
    rm -f "$SERVICE_DIR/kusuri-desktop.service"
    systemctl --user daemon-reload || true

    # 2. Kill running processes
    pkill -f "$BIN_NAME" 2>/dev/null || true

    # 3. Remove desktop shortcut
    rm -f "$DESKTOP_DIR/kusuri-desktop.desktop"
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true

    # 4. Remove binary
    rm -f "$INSTALL_DIR/$BIN_NAME"

    echo "[✓] $APP_NAME berhasil di-uninstall!"
    exit 0
fi

# Locate or build binary
RELEASE_BIN="$PROJECT_ROOT/target/release/$BIN_NAME"
DEBUG_BIN="$PROJECT_ROOT/target/debug/$BIN_NAME"
TARGET_SOURCE=""

if [ -f "$RELEASE_BIN" ]; then
    TARGET_SOURCE="$RELEASE_BIN"
elif [ -f "$DEBUG_BIN" ]; then
    TARGET_SOURCE="$DEBUG_BIN"
else
    echo "[*] Binary belum ada. Mengompilasi Kusuri Desktop (Release mode)..."
    cd "$PROJECT_ROOT"
    cargo build --release -p kusuri-desktop
    TARGET_SOURCE="$RELEASE_BIN"
fi

echo "[1/4] Mempersiapkan direktori tujuan..."
mkdir -p "$INSTALL_DIR"
mkdir -p "$DESKTOP_DIR"
mkdir -p "$SERVICE_DIR"

echo "[2/4] Menyalin binary ke $INSTALL_DIR/$BIN_NAME..."
pkill -f "$BIN_NAME" 2>/dev/null || true
sleep 1
cp "$TARGET_SOURCE" "$INSTALL_DIR/$BIN_NAME"
chmod +x "$INSTALL_DIR/$BIN_NAME"

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo "      [!] Catatan: Pastikan ~/.local/bin terdaftar di PATH Anda (~/.bashrc atau ~/.zshrc)"
fi

echo "[3/4] Mendaftarkan Desktop Application Entry..."
cat <<EOF > "$DESKTOP_DIR/kusuri-desktop.desktop"
[Desktop Entry]
Version=1.0
Name=Kusuri Desktop
GenericName=System Telemetry Client
Comment=24/7 Kusuri Telemetry & Management Client
Exec=$INSTALL_DIR/$BIN_NAME gui
Icon=utilities-system-monitor
Terminal=false
Type=Application
Categories=Utility;System;Monitor;
StartupNotify=true
EOF
chmod +x "$DESKTOP_DIR/kusuri-desktop.desktop"
update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true

echo "[4/4] Mengonfigurasi systemd user service untuk background 24/7..."
cat <<EOF > "$SERVICE_DIR/kusuri-desktop.service"
[Unit]
Description=Kusuri Desktop 24/7 Live Telemetry Daemon
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=$INSTALL_DIR/$BIN_NAME run
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=default.target
EOF

systemctl --user daemon-reload
systemctl --user enable kusuri-desktop.service
systemctl --user restart kusuri-desktop.service

echo ""
echo "============================================================"
echo " [✓] INSTALASI BERHASIL! Kusuri Desktop Siap Digunakan       "
echo "============================================================"
echo "Binary terpasang  : $INSTALL_DIR/$BIN_NAME"
echo "Desktop Shortcut  : $DESKTOP_DIR/kusuri-desktop.desktop"
echo "Systemd 24/7 Daemon: Aktif (systemctl --user status kusuri-desktop)"
echo ""
echo "Perintah cepat:"
echo "  - Buka GUI       : kusuri-desktop gui"
echo "  - Cek Status     : kusuri-desktop status"
echo "  - Cek Telemetry  : kusuri-desktop telemetry"
echo "  - Uninstall      : bash $0 --uninstall"
echo "============================================================"
