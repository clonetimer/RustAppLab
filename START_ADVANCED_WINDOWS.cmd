@echo off
setlocal DisableDelayedExpansion
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\advanced_windows.ps1" %*
set "CODE=%ERRORLEVEL%"
echo.
echo Advanced guide exited with code %CODE%.
pause
exit /b %CODE%
