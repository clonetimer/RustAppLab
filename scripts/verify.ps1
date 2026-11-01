$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error 'Cargo not found. Install Rust with rustup first.'
    exit 127
}
Write-Host '[1/4] Format (all packages)'
& cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host '[2/4] Clippy (all packages)'
& cargo clippy --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host '[3/4] Tests (all packages)'
& cargo test --locked --offline --workspace --all-targets -- --test-threads=2
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host 'MiniKV borrow doctests'
& cargo test --offline -p minikv --doc
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host 'Release full workspace tests'
& cargo test --locked --offline --release --workspace --all-targets -- --test-threads=2
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host '[4/4] Release (all packages)'
& cargo build --workspace --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host 'Completed. Manual application and cross-platform verification are still required.'
