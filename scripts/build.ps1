Write-Host "=== Building Kusuri Monorepo Artifacts ===" -ForegroundColor Cyan

Write-Host "1. Building Web Dashboard..." -ForegroundColor Green
bun --cwd apps/web build

Write-Host "2. Building Backend Release Binary..." -ForegroundColor Green
cargo build --release

Write-Host "=== Build Completed Successfully! ===" -ForegroundColor Green
