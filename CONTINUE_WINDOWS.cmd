@echo off
setlocal
chcp 65001 >nul
cd /d "%~dp0"
echo Rust AppLab - Continue Learning / Studio
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\studio_windows.ps1"
set "code=%ERRORLEVEL%"
if not "%code%"=="0" echo Studio exited with code %code%. Keep the output above.
pause
exit /b %code%
