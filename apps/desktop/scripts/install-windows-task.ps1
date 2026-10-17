# Script untuk mendaftarkan Kusuri Desktop sebagai Scheduled Task 24/7 di Windows
param(
    [string]$Action = "install" # "install" atau "uninstall"
)

$TaskName = "KusuriDesktop24x7"
$BinaryPath = "$PSScriptRoot\..\..\target\release\kusuri-desktop.exe"
if (-not (Test-Path $BinaryPath)) {
    $BinaryPath = "$PSScriptRoot\..\..\target\debug\kusuri-desktop.exe"
}

if ($Action -eq "install") {
    if (-not (Test-Path $BinaryPath)) {
        Write-Error "Binary kusuri-desktop.exe belum ditemukan. Silakan jalankan 'cargo build -p kusuri-desktop' terlebih dahulu."
        exit 1
    }

    $ResolvedPath = (Resolve-Path $BinaryPath).Path
    Write-Host "Mendaftarkan Kusuri Desktop ke Windows Task Scheduler..." -ForegroundColor Cyan
    Write-Host "Lokasi executable: $ResolvedPath" -ForegroundColor Gray

    # Jalankan saat user login, restart jika gagal, berjalan tanpa henti
    $ActionObj = New-ScheduledTaskAction -Execute $ResolvedPath -Argument "run"
    $TriggerObj = New-ScheduledTaskTrigger -AtLogOn
    $SettingsObj = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -ExecutionTimeLimit (New-TimeSpan -Days 365) -RestartCount 999 -RestartInterval (New-TimeSpan -Minutes 1)

    Register-ScheduledTask -TaskName $TaskName -Action $ActionObj -Trigger $TriggerObj -Settings $SettingsObj -Description "Kusuri Desktop 24/7 Live Telemetry Daemon" -Force
    Start-ScheduledTask -TaskName $TaskName

    Write-Host "Berhasil! Task '$TaskName' telah terdaftar dan berjalan 24/7 di background." -ForegroundColor Green
    Write-Host "Cek status dengan: Get-ScheduledTask -TaskName $TaskName" -ForegroundColor Gray
}
elseif ($Action -eq "uninstall") {
    Write-Host "Menghapus Windows Task '$TaskName'..." -ForegroundColor Yellow
    Stop-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue
    Write-Host "Task '$TaskName' berhasil dihapus." -ForegroundColor Green
}
else {
    Write-Host "Gunakan: .\install-windows-task.ps1 -Action install|uninstall" -ForegroundColor Red
}
