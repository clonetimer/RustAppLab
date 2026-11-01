@echo off
setlocal EnableExtensions DisableDelayedExpansion
chcp 65001 >nul
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\learning_session.ps1"
set "result=%ERRORLEVEL%"
if not "%result%"=="0" echo Learning session exited with code %result%. See the message above.
echo.
pause
exit /b %result%
