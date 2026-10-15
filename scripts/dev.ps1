Write-Host "=== Starting Kusuri Development Environment ===" -ForegroundColor Cyan

# Check if DB container is running, start if stopped
if (Get-Command docker -ErrorAction SilentlyContinue) {
    Write-Host "Checking PostgreSQL database..." -ForegroundColor Cyan
    docker compose up -d kusuri-db
}

# Start backend in a separate background job or window
Write-Host "Starting Backend server (Axum)..." -ForegroundColor Green
$backendJob = Start-Job -ScriptBlock {
    Set-Location $using:PWD
    cargo run --bin backend
}

# Start web frontend in foreground
Write-Host "Starting Web Dashboard (Bun + React)..." -ForegroundColor Green
try {
    bun --cwd apps/web dev
} finally {
    Write-Host "Stopping backend job..." -ForegroundColor Yellow
    Stop-Job $backendJob -ErrorAction SilentlyContinue
    Remove-Job $backendJob -ErrorAction SilentlyContinue
}
