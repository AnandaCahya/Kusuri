#!/usr/bin/env bash
set -e

echo "=== Building Kusuri Monorepo Artifacts ==="

echo "1. Building Web Dashboard..."
bun --cwd apps/web build

echo "2. Building Backend Release Binary..."
cargo build --release

echo "=== Build Completed Successfully! ==="
