@echo off
setlocal
title Kusuri Desktop Installer

echo ============================================================
echo           Memulai Instalasi Kusuri Desktop...
echo ============================================================
echo.

powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\Setup-KusuriDesktop.ps1"

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] Instalasi mengalami kegagalan.
    pause
    exit /b %ERRORLEVEL%
)

echo.
pause
