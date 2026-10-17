@echo off
setlocal
title Kusuri Desktop Installer

echo ============================================================
echo           Kusuri Desktop Client - Windows Setup
echo ============================================================
echo.

powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0apps\desktop\scripts\Setup-KusuriDesktop.ps1"

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] Terjadi kesalahan saat instalasi.
    pause
    exit /b %ERRORLEVEL%
)

echo.
pause
