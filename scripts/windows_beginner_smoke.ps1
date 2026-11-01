# This script runs on a real Windows machine in CI. It is not evidence of a local Windows run.
$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $PSScriptRoot
$Base = Join-Path ([System.IO.Path]::GetTempPath()) ("rust-applab-win-smoke-" + [guid]::NewGuid().ToString('N'))
$Project = Join-Path $Base '学习 空格'
$RelativeName = 'win-relative-' + [guid]::NewGuid().ToString('N')
$RelativeProject = Join-Path $Root $RelativeName
$Env:LOCALE = 'zh_CN.UTF-8'
try {
    New-Item -ItemType Directory -Path $Base -Force | Out-Null
    Set-Location -LiteralPath $Root
    # 与 START_WINDOWS.cmd 最初失败的操作一致：从工程根目录创建单层相对路径。
    & cargo run --offline -p labquest -- beginner start $RelativeName
    if ($LASTEXITCODE -ne 0) { throw 'relative beginner start failed' }
    if (-not (Test-Path -LiteralPath (Join-Path $RelativeProject 'src\main.rs'))) { throw 'relative beginner project missing' }
    & cargo run --offline -p labquest -- beginner start $Project
    if ($LASTEXITCODE -ne 0) { throw 'init failed' }
    & cargo run --offline -p labquest -- beginner check $Project
    if ($LASTEXITCODE -eq 0) { throw 'initial unfinished project unexpectedly passed' }
    $Source = Join-Path $Project 'src\main.rs'
    $Before = [System.IO.File]::ReadAllText($Source, [System.Text.Encoding]::UTF8)
    $Fixed = $Before.Replace('等待完成第一张任务卡', '今日任务卡 | {name} | {task}')
    if ($Fixed -eq $Before) { throw 'starter source marker not found' }
    [System.IO.File]::WriteAllText($Source, $Fixed, (New-Object System.Text.UTF8Encoding($false)))
    & cargo run --offline -p labquest -- beginner check $Project
    if ($LASTEXITCODE -ne 0) { throw 'fixed code did not pass beginner check' }
    if (-not (Test-Path -LiteralPath (Join-Path $Project 'FIRST_WIN.md'))) { throw 'FIRST_WIN.md missing' }
    & cargo run --offline --manifest-path (Join-Path $Project 'Cargo.toml') -- '小林' '写 Rust'
    if ($LASTEXITCODE -ne 0) { throw 'student program failed to run' }
    Write-Host 'WINDOWS BEGINNER SMOKE PASSED' -ForegroundColor Green
} finally {
    Set-Location -LiteralPath $Root
    if (Test-Path -LiteralPath $RelativeProject) { Remove-Item -LiteralPath $RelativeProject -Recurse -Force }
    if (Test-Path -LiteralPath $Base) { Remove-Item -LiteralPath $Base -Recurse -Force }
}
