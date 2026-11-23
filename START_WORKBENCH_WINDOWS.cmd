@echo off
setlocal
cd /d "%~dp0"
if not "%~1"=="" goto with_args
if exist "bin\labquest.exe" (
    "bin\labquest.exe" workbench --open
) else (
    cargo run --locked --offline -p labquest -- workbench --open
)
set "result=%errorlevel%"
if not "%result%"=="0" pause
exit /b %result%
:with_args
if exist "bin\labquest.exe" (
    "bin\labquest.exe" workbench %*
) else (
    cargo run --locked --offline -p labquest -- workbench %*
)
exit /b %errorlevel%
