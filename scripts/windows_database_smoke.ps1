$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $Root
foreach ($Script in @('studio_windows.ps1','windows_studio_smoke.ps1','start_windows.ps1')) {
    $Tokens = $null; $Errors = $null
    $null = [System.Management.Automation.Language.Parser]::ParseFile((Join-Path $PSScriptRoot $Script), [ref]$Tokens, [ref]$Errors)
    if ($Errors.Count -ne 0) { throw "PowerShell parse failed: $Script $Errors" }
}
& rustc -vV
& cargo build --locked --offline -p storedesk -p termgarden --target-dir (Join-Path $Root 'target')
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$Store = Join-Path $Root 'target\debug\storedesk.exe'
$Garden = Join-Path $Root 'target\debug\termgarden.exe'
& $Store doctor
if ($LASTEXITCODE -ne 0) { throw 'SQLite runtime unavailable; read course/SQLITE_SETUP.md' }
$Temp = Join-Path ([IO.Path]::GetTempPath()) ('RustAppLab 数据 '+[guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $Temp | Out-Null
try {
    $Db = Join-Path $Temp '我的 库存.db'
    & $Store init $Db
    if ($LASTEXITCODE -ne 0) { throw 'init failed' }
    & $Store add $Db PEN '学习用笔' 10
    if ($LASTEXITCODE -ne 0) { throw 'add failed' }
    & $Store adjust $Db PEN -2
    if ($LASTEXITCODE -ne 0) { throw 'adjust failed' }
    & $Store adjust $Db PEN -99
    if ($LASTEXITCODE -eq 0) { throw 'negative stock was accepted' }
    $Output = & $Store list $Db
    if ($LASTEXITCODE -ne 0 -or ($Output -join "`n") -notmatch "PEN`t学习用笔`t8") { throw 'inventory mismatch' }
    & $Garden demo (Join-Path $Temp '终端 花园')
    if ($LASTEXITCODE -ne 0) { throw 'garden demo failed' }
    Write-Host 'Native SQLite and terminal smoke passed; this is not a full GUI usability evaluation.'
} finally { Remove-Item -LiteralPath $Temp -Force -Recurse }
