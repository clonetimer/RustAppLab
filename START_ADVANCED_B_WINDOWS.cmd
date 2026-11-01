@echo off
setlocal DisableDelayedExpansion
cd /d "%~dp0"
echo Rust AppLab Advanced B - TaskPocket Pro
echo This creates a NEW project, never overwrite old work.
set /p "PROJECT=New folder name (default my-taskpocket-pro): "
if not defined PROJECT set "PROJECT=my-taskpocket-pro"
cargo run --locked --offline -p labquest -- advanced-b start "%CD%\%PROJECT%"
if errorlevel 1 (echo Failed. Keep output for support.& pause & exit /b 1)
echo Open %CD%\%PROJECT%\START_HERE.md
cargo run --locked --offline -p labquest -- advanced-b next "%CD%\%PROJECT%"
pause
