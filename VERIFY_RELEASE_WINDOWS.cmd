@echo off
setlocal
chcp 65001 >nul
cd /d "%~dp0"
echo Rust AppLab release verification. Do not close this window.
powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\verify_release_windows.ps1"
set "RESULT=%ERRORLEVEL%"
echo.
echo Verification exited with code %RESULT%. Evidence paths are printed above.
pause
exit /b %RESULT%
