@echo off
setlocal
cd /d "%~dp0"
echo Optional NETWORK dependency preparation - not required for beginner lessons.
echo This command downloads crates and runs dependency build scripts using Cargo.
echo Output: network-offline-kit. Existing learner projects will not be changed.
cargo run --offline -p labquest -- ecosystem prepare network-offline-kit
set "RESULT=%ERRORLEVEL%"
if not "%RESULT%"=="0" (
  echo Preparation did not pass. Keep network-offline-kit\STATUS.md and logs.
  echo After fixing network/toolchain issues, run this command again.
) else (
  echo Preparation passed on this machine only.
  echo To pack: powershell -NoProfile -ExecutionPolicy Bypass -File network-offline-kit\PACK_WINDOWS.ps1
)
pause
exit /b %RESULT%
