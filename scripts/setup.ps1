Write-Host "=== Setting up Kusuri Monorepo Environment ===" -ForegroundColor Cyan

# 1. Check Bun
if (Get-Command bun -ErrorAction SilentlyContinue) {
    Write-Host "[✓] Bun is installed: $(bun --version)" -ForegroundColor Green
} else {
    Write-Host "[!] Bun not found. Please install Bun from https://bun.sh" -ForegroundColor Yellow
}

# 2. Check Cargo
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    Write-Host "[✓] Rust/Cargo is installed: $(cargo --version)" -ForegroundColor Green
} else {
    Write-Host "[!] Cargo not found. Please install Rust from https://rustup.rs" -ForegroundColor Yellow
}

# 3. Check Docker
if (Get-Command docker -ErrorAction SilentlyContinue) {
    Write-Host "[✓] Docker is installed: $(docker --version)" -ForegroundColor Green
} else {
    Write-Host "[!] Docker not found (optional for local DB): https://docker.com" -ForegroundColor Yellow
}

# 4. Install Web dependencies
Write-Host "Installing web dashboard dependencies..." -ForegroundColor Cyan
Set-Location apps/web
bun install
Set-Location ../..

# 5. Environment copy if needed
if (-not (Test-Path apps/backend/.env)) {
    Copy-Item apps/backend/.env.example apps/backend/.env
    Write-Host "[✓] Created apps/backend/.env from .env.example" -ForegroundColor Green
}

Write-Host "=== Setup completed! Run scripts/dev.ps1 to start development. ===" -ForegroundColor Green
