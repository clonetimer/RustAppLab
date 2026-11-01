# 原生 Windows CI / 维护者使用；不是学员作业，也不把参考实现发给学习目录。
$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $Root
& cargo build --offline -p labquest --target-dir (Join-Path $Root 'target')
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$Exe = Join-Path $Root 'target\debug\labquest.exe'
$Temp = Join-Path ([IO.Path]::GetTempPath()) ('RustAppLab 学习 '+[guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $Temp | Out-Null
try {
    foreach ($Id in @('unitlab','quizdeck','textscope','ledgerlite','badgeforge','batchmeter','termgarden','storedesk','minikv','netfetch','servicedesk','asyncboard')) {
        $Project = Join-Path $Temp $Id
        & $Exe studio start $Id $Project
        if ($LASTEXITCODE -ne 0) { throw "init failed: $Id" }
        & $Exe studio check $Project 1
        if ($LASTEXITCODE -eq 0) { throw "starter unexpectedly passed: $Id" }
        $Reference = [System.IO.File]::ReadAllText((Join-Path $Root "apps\$Id\src\lib.rs"))
        if ($Id -eq 'termgarden') { $Reference = $Reference -replace '(?m)^pub mod workbench;\r?\n', '' }
        [System.IO.File]::WriteAllText((Join-Path $Project 'src\lib.rs'), $Reference, (New-Object System.Text.UTF8Encoding($false)))
        & $Exe studio check $Project all
        if ($LASTEXITCODE -ne 0) { throw "reference failed: $Id" }
    }
    Write-Host 'Twelve native Windows Studio red-to-green flows passed.'
} finally {
    # 只清理本脚本自己创建的唯一临时目录，不碰学员目录。
    Remove-Item -LiteralPath $Temp -Recurse -Force
}
