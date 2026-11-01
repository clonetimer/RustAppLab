@echo off
setlocal
chcp 65001 >nul
cd /d "%~dp0"
echo Rust AppLab real browser acceptance. This is a maintainer check, not a learner step.
echo It does not disable browser policy. A blocked localhost request remains BLOCKED.
powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\verify_browser_windows.ps1"
set "RESULT=%ERRORLEVEL%"
echo.
echo Browser acceptance exited with code %RESULT%.
pause
exit /b %RESULT%
