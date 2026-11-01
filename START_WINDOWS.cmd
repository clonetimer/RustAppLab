@echo off
setlocal
chcp 65001 >nul
cd /d "%~dp0"
echo Rust AppLab - Windows Beginner Launcher
echo This launcher only creates a new learning folder in this project.
echo It will not install software or delete files.
echo.
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\start_windows.ps1"
set "exitcode=%ERRORLEVEL%"
echo.
if not "%exitcode%"=="0" echo The guide exited with code %exitcode%. See the messages above.
echo You can reopen START_HERE.html for help.
pause
exit /b %exitcode%
