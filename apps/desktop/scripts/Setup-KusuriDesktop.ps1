<#
.SYNOPSIS
    Kusuri Desktop Installer & Setup Script for Windows
.DESCRIPTION
    Installs Kusuri Desktop client into %LOCALAPPDATA%\Programs\KusuriDesktop,
    creates Start Menu & Desktop shortcuts, registers the application in Windows
    Add/Remove Programs (Installed Apps), and optionally configures 24/7 background autostart.
#>

param(
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\KusuriDesktop",
    [switch]$NoAutoStart,
    [switch]$NoDesktopShortcut,
    [switch]$NoStartMenu,
    [switch]$NoLaunch,
    [switch]$Uninstall,
    [switch]$Quiet
)

$ErrorActionPreference = "Stop"

$AppName = "Kusuri Desktop"
$AppVersion = "0.1.0"
$AppPublisher = "Kusuri Contributors"
$AppDescription = "24/7 Kusuri Desktop Telemetry & Management Client"
$RegistryKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\KusuriDesktop"
$AutoStartKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$AutoStartVal = "KusuriDesktop"

function Write-Header {
    if (-not $Quiet) {
        Write-Host ""
        Write-Host "============================================================" -ForegroundColor Cyan
        Write-Host "             KUSURI DESKTOP CLIENT SETUP                    " -ForegroundColor White -BackgroundColor DarkBlue
        Write-Host "                   Versi $AppVersion                        " -ForegroundColor Gray
        Write-Host "============================================================" -ForegroundColor Cyan
        Write-Host ""
    }
}

if ($Uninstall) {
    Write-Header
    Write-Host "[-] Memulai proses uninstall $AppName..." -ForegroundColor Yellow

    # 1. Hentikan proses yang sedang berjalan
    Write-Host "[-] Menghentikan proses $AppName..." -ForegroundColor Gray
    Get-Process -Name "kusuri-desktop" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

    # 2. Hapus Windows Task Scheduler jika ada
    Unregister-ScheduledTask -TaskName "KusuriDesktop24x7" -Confirm:$false -ErrorAction SilentlyContinue | Out-Null

    # 3. Hapus AutoStart Registry
    if (Get-ItemProperty -Path $AutoStartKey -Name $AutoStartVal -ErrorAction SilentlyContinue) {
        Remove-ItemProperty -Path $AutoStartKey -Name $AutoStartVal -ErrorAction SilentlyContinue
        Write-Host "[-] Auto-start registry key dihapus." -ForegroundColor Gray
    }

    # 4. Hapus Shortcut Desktop & Start Menu
    $DesktopShortcut = Join-Path ([Environment]::GetFolderPath("Desktop")) "$AppName.lnk"
    if (Test-Path $DesktopShortcut) {
        Remove-Item -Path $DesktopShortcut -Force -ErrorAction SilentlyContinue
        Write-Host "[-] Shortcut Desktop dihapus." -ForegroundColor Gray
    }

    $StartMenuDir = Join-Path ([Environment]::GetFolderPath("Programs")) "Kusuri"
    if (Test-Path $StartMenuDir) {
        Remove-Item -Path $StartMenuDir -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "[-] Shortcut Start Menu dihapus." -ForegroundColor Gray
    }

    # 5. Hapus Registry Uninstall
    if (Test-Path $RegistryKey) {
        Remove-Item -Path $RegistryKey -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "[-] Pendaftaran Windows Add/Remove Programs dihapus." -ForegroundColor Gray
    }

    # 6. Hapus direktori instalasi
    if (Test-Path $InstallDir) {
        Start-Sleep -Seconds 1
        Write-Host "[-] Menghapus file instalasi di $InstallDir..." -ForegroundColor Gray
        $CmdClean = 'timeout /t 1 /nobreak >nul & rd /s /q "' + $InstallDir + '"'
        Start-Process cmd.exe -ArgumentList "/c $CmdClean" -WindowStyle Hidden
    }

    Write-Host ""
    Write-Host "[OK] $AppName berhasil di-uninstall dari komputer Anda!" -ForegroundColor Green
    Write-Host ""
    exit 0
}

Write-Header

# 1. Cari file executable release atau debug
$ProjectRoot = Resolve-Path "$PSScriptRoot\..\..\.."
$ReleaseExe = Join-Path $ProjectRoot "target\release\kusuri-desktop.exe"
$DebugExe = Join-Path $ProjectRoot "target\debug\kusuri-desktop.exe"

$SourceExe = $null
if (Test-Path $ReleaseExe) {
    $SourceExe = $ReleaseExe
} elseif (Test-Path $DebugExe) {
    $SourceExe = $DebugExe
} else {
    Write-Host "[*] Executable belum ditemukan. Mengompilasi Kusuri Desktop (Release mode)..." -ForegroundColor Yellow
    Push-Location $ProjectRoot
    try {
        & cargo build --release -p kusuri-desktop
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Gagal mengompilasi binary Kusuri Desktop!"
            exit 1
        }
        $SourceExe = $ReleaseExe
    } finally {
        Pop-Location
    }
}

if (-not (Test-Path $SourceExe)) {
    Write-Error "File binary tidak ditemukan di: $SourceExe"
    exit 1
}

Write-Host "[1/5] Mempersiapkan folder instalasi..." -ForegroundColor Cyan
Write-Host "      Direktori: $InstallDir" -ForegroundColor Gray

# Matikan proses berjalan sebelum overwrite file
Get-Process -Name "kusuri-desktop" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$TargetExe = Join-Path $InstallDir "kusuri-desktop.exe"
Write-Host "[2/5] Menyalin file aplikasi ke tujuan..." -ForegroundColor Cyan
Copy-Item -Path $SourceExe -Destination $TargetExe -Force

# Salin installer script ke install dir untuk uninstaller
$TargetSetupScript = Join-Path $InstallDir "Setup-KusuriDesktop.ps1"
$CurrentScriptPath = if ($PSCommandPath) { $PSCommandPath } else { $MyInvocation.MyCommand.Path }
if ($CurrentScriptPath -and (Test-Path $CurrentScriptPath)) {
    Copy-Item -Path $CurrentScriptPath -Destination $TargetSetupScript -Force
}

# Buat uninstaller script wrapper di install dir
$UninstallerScript = Join-Path $InstallDir "Uninstall.ps1"
$UninstallLines = @(
    '# Kusuri Desktop Uninstaller',
    '$SetupScript = Join-Path $PSScriptRoot "Setup-KusuriDesktop.ps1"',
    'if (Test-Path $SetupScript) {',
    '    & $SetupScript -Uninstall',
    '} else {',
    '    Write-Host "Menghentikan kusuri-desktop..." -ForegroundColor Gray',
    '    Get-Process -Name "kusuri-desktop" -ErrorAction SilentlyContinue | Stop-Process -Force',
    '    Write-Host "Kusuri Desktop telah dihentikan." -ForegroundColor Green',
    '}'
)
Set-Content -Path $UninstallerScript -Value ($UninstallLines -join "`r`n") -Force

Write-Host "[3/5] Membuat shortcut aplikasi..." -ForegroundColor Cyan
$WshShell = New-Object -ComObject WScript.Shell

if (-not $NoStartMenu) {
    $StartMenuDir = Join-Path ([Environment]::GetFolderPath("Programs")) "Kusuri"
    if (-not (Test-Path $StartMenuDir)) {
        New-Item -ItemType Directory -Path $StartMenuDir -Force | Out-Null
    }
    $StartMenuShortcutPath = Join-Path $StartMenuDir "$AppName.lnk"
    $Shortcut = $WshShell.CreateShortcut($StartMenuShortcutPath)
    $Shortcut.TargetPath = $TargetExe
    $Shortcut.WorkingDirectory = $InstallDir
    $Shortcut.Description = $AppDescription
    $Shortcut.IconLocation = "$TargetExe,0"
    $Shortcut.Save()
    Write-Host "      [+] Start Menu: $StartMenuShortcutPath" -ForegroundColor Gray
}

if (-not $NoDesktopShortcut) {
    $DesktopShortcutPath = Join-Path ([Environment]::GetFolderPath("Desktop")) "$AppName.lnk"
    $Shortcut = $WshShell.CreateShortcut($DesktopShortcutPath)
    $Shortcut.TargetPath = $TargetExe
    $Shortcut.WorkingDirectory = $InstallDir
    $Shortcut.Description = $AppDescription
    $Shortcut.IconLocation = "$TargetExe,0"
    $Shortcut.Save()
    Write-Host "      [+] Desktop   : $DesktopShortcutPath" -ForegroundColor Gray
}

Write-Host "[4/5] Mendaftarkan ke Windows Installed Apps (Control Panel)..." -ForegroundColor Cyan
if (-not (Test-Path $RegistryKey)) {
    New-Item -Path $RegistryKey -Force | Out-Null
}

$FileItem = Get-Item $TargetExe
$FileSizeKb = [math]::Round($FileItem.Length / 1KB)

Set-ItemProperty -Path $RegistryKey -Name "DisplayName" -Value $AppName
Set-ItemProperty -Path $RegistryKey -Name "DisplayVersion" -Value $AppVersion
Set-ItemProperty -Path $RegistryKey -Name "Publisher" -Value $AppPublisher
Set-ItemProperty -Path $RegistryKey -Name "DisplayIcon" -Value "$TargetExe,0"
Set-ItemProperty -Path $RegistryKey -Name "InstallLocation" -Value $InstallDir
Set-ItemProperty -Path $RegistryKey -Name "UninstallString" -Value ('powershell.exe -ExecutionPolicy Bypass -File "' + $UninstallerScript + '"')
Set-ItemProperty -Path $RegistryKey -Name "QuietUninstallString" -Value ('powershell.exe -ExecutionPolicy Bypass -File "' + $UninstallerScript + '" -Quiet')
Set-ItemProperty -Path $RegistryKey -Name "EstimatedSize" -Value $FileSizeKb
Set-ItemProperty -Path $RegistryKey -Name "Comments" -Value $AppDescription

if (-not $NoAutoStart) {
    Write-Host "[5/5] Mengatur Auto-Start 24/7 pada Windows Startup..." -ForegroundColor Cyan
    # Tambahkan ke Run key (menjalankan aplikasi di background / tray saat login)
    Set-ItemProperty -Path $AutoStartKey -Name $AutoStartVal -Value ('"' + $TargetExe + '" gui --minimized')
    Write-Host "      [+] Registry Run: Terdaftar ($AutoStartVal -> gui --minimized)" -ForegroundColor Gray
} else {
    Write-Host "[5/5] Melewati Auto-Start..." -ForegroundColor Gray
}

Write-Host ""
Write-Host "============================================================" -ForegroundColor Green
Write-Host " [OK] INSTALASI BERHASIL! Kusuri Desktop Siap Digunakan       " -ForegroundColor White -BackgroundColor DarkGreen
Write-Host "============================================================" -ForegroundColor Green
Write-Host "Lokasi Terpasang: $InstallDir" -ForegroundColor White
Write-Host ""
Write-Host "Aplikasi dapat dibuka melalui:" -ForegroundColor Cyan
Write-Host "  1. Shortcut di Desktop: '$AppName'" -ForegroundColor White
Write-Host "  2. Start Menu: 'Kusuri -> $AppName'" -ForegroundColor White
Write-Host "  3. Terminal / CMD: & `"$TargetExe`"" -ForegroundColor White
Write-Host ""

if (-not $NoLaunch) {
    Write-Host "[*] Meluncurkan Kusuri Desktop sekarang..." -ForegroundColor Yellow
    Start-Process -FilePath $TargetExe
}
