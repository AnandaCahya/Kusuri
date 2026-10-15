#!/usr/bin/env bash
set -e

echo "=== Setting up Kusuri Monorepo Environment ==="

command -v bun >/dev/null 2>&1 && echo "[✓] Bun is installed: $(bun --version)" || echo "[!] Bun not found. Install from https://bun.sh"
command -v cargo >/dev/null 2>&1 && echo "[✓] Cargo is installed: $(cargo --version)" || echo "[!] Cargo not found. Install from https://rustup.rs"
command -v docker >/dev/null 2>&1 && echo "[✓] Docker is installed: $(docker --version)" || echo "[!] Docker not found."

echo "Installing web dashboard dependencies..."
(cd apps/web && bun install)

if [ ! -f "apps/backend/.env" ]; then
    cp apps/backend/.env.example apps/backend/.env
    echo "[✓] Created apps/backend/.env from .env.example"
fi

echo "=== Setup completed! Run ./scripts/dev.sh to start development. ==="
