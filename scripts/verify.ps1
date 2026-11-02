$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error 'Cargo not found. Install Rust with rustup first.'
    exit 127
}
Write-Host '[1/5] Format (all packages)'
& cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host '[2/5] Clippy (all packages)'
& cargo clippy --locked --offline --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host '[3/5] Tests (all packages)'
& cargo test --locked --offline --workspace --all-targets -- --test-threads=2
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host 'Doctests (all packages)'
& cargo test --locked --offline --workspace --doc
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host 'Release full workspace tests'
& cargo test --locked --offline --release --workspace --all-targets -- --test-threads=2
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host '[4/5] Release (all packages)'
& cargo build --locked --offline --workspace --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host '[5/5] Advanced B reference workspace and feature configurations'
$ReferenceManifest = 'projects/advanced-b/reference/Cargo.toml'
$ReferenceCommon = @('--locked','--offline','--manifest-path',$ReferenceManifest,'--workspace')
& cargo fmt --manifest-path $ReferenceManifest --all -- --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
foreach ($Configuration in @(@{Name='default'; Features=@()}, @{Name='strict'; Features=@('--all-features')})) {
    Write-Host ('Advanced B clippy: '+$Configuration.Name)
    $CargoArguments = @('clippy')+$ReferenceCommon+@('--all-targets')+$Configuration.Features+@('--','-D','warnings')
    & cargo @CargoArguments
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    foreach ($BuildProfile in @(@{Name='debug'; Flags=@()}, @{Name='release'; Flags=@('--release')})) {
        Write-Host ('Advanced B tests: '+$BuildProfile.Name+' '+$Configuration.Name)
        $CargoArguments = @('test')+$ReferenceCommon+$BuildProfile.Flags+@('--all-targets')+$Configuration.Features+@('--','--test-threads=2')
        & cargo @CargoArguments
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }
    Write-Host ('Advanced B doctests: '+$Configuration.Name)
    $CargoArguments = @('test')+$ReferenceCommon+$Configuration.Features+@('--doc')
    & cargo @CargoArguments
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
Write-Host 'Advanced B tests: no default features'
$CargoArguments = @('test')+$ReferenceCommon+@('--all-targets','--no-default-features','--','--test-threads=2')
& cargo @CargoArguments
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host 'Advanced B4 independently prepared JSON workspace (offline)'
$JsonManifest = 'projects/advanced-b/reference/json-export/Cargo.toml'
$JsonCommon = @('--locked','--offline','--manifest-path',$JsonManifest,'--workspace')
& cargo fmt --manifest-path $JsonManifest --all -- --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
foreach ($Configuration in @(@{Name='default'; Features=@()}, @{Name='strict'; Features=@('--all-features')})) {
    $CargoArguments = @('clippy')+$JsonCommon+@('--all-targets')+$Configuration.Features+@('--','-D','warnings')
    & cargo @CargoArguments
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    foreach ($BuildProfile in @(@{Name='debug'; Flags=@()}, @{Name='release'; Flags=@('--release')})) {
        $CargoArguments = @('test')+$JsonCommon+$BuildProfile.Flags+@('--all-targets')+$Configuration.Features+@('--','--test-threads=2')
        & cargo @CargoArguments
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }
    $CargoArguments = @('test')+$JsonCommon+$Configuration.Features+@('--doc')
    & cargo @CargoArguments
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
& cargo test --locked --offline -p labquest --features b4-integration --test advanced_b_json_e2e -- --test-threads=1
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host 'Completed. Manual application and cross-platform verification are still required.'
