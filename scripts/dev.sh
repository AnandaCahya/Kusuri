#!/usr/bin/env bash
set -e

echo "=== Starting Kusuri Development Environment ==="

if command -v docker >/dev/null 2>&1; then
    echo "Starting database via docker compose..."
    docker compose up -d kusuri-db
fi

echo "Starting Backend (Rust Axum) in background..."
cargo run --bin backend &
BACKEND_PID=$!

cleanup() {
    echo "Stopping background backend process ($BACKEND_PID)..."
    kill $BACKEND_PID 2>/dev/null || true
}
trap cleanup EXIT

echo "Starting Web Dashboard (Bun + React)..."
bun --cwd apps/web dev
