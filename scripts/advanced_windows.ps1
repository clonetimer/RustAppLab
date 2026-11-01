# Windows PowerShell 5.1 / 7. No Python, no global toolchain changes.
param([string]$Project = '')
$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $Root
try {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw '未找到Cargo；请先确认基础课程的Rust环境可用。' }
    Write-Host '进阶 A：共享工作区、批量编辑、撤销与后台报告。基础已通过，不需重做。' -ForegroundColor Cyan
    if (-not $Project) { $Project = Read-Host '输入新进阶工程目录，或已有进阶目录（Enter=my-workbench）' }
    $Project = $Project.Trim().Trim('"')
    if (-not $Project) { $Project = 'my-workbench' }
    if (-not [System.IO.Path]::IsPathRooted($Project)) { $Project = Join-Path $Root $Project }
    if (Test-Path -LiteralPath $Project) {
        if (-not (Test-Path -LiteralPath (Join-Path $Project '.applab-advanced') -PathType Leaf)) { throw '已有目录不是进阶工程；不会覆盖，请换一个新目录。' }
    } else {
        $Saved = $ErrorActionPreference
        try {
            $ErrorActionPreference = 'Continue'
            & cargo run --locked --offline -p labquest -- advanced start $Project | Out-Host
            $Code = $LASTEXITCODE
        } finally { $ErrorActionPreference = $Saved }
        if ($Code -ne 0) { exit $Code }
    }
    Invoke-Item -LiteralPath (Join-Path $Project 'START.html')
    & (Join-Path $PSScriptRoot 'learning_session.ps1') -Project $Project
} catch {
    Write-Host "未开始：$($_.Exception.Message)" -ForegroundColor Yellow
    exit 2
}
