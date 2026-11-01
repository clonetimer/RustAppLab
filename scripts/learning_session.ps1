# Rust AppLab learning session. Windows PowerShell 5.1 / PowerShell 7.
# Only explicit check executes learner code. Read-only actions never modify it.
param(
    [string]$Project = '',
    [ValidateSet('', 'route', 'next', 'lesson', 'check', 'check-all', 'hint1', 'hint2', 'hint3')]
    [string]$Action = ''
)
$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $Root

function Invoke-Learning([string[]]$Arguments) {
    # Native stderr (including normal Compiling messages) is not a PowerShell exception.
    # Never capture/merge 2>&1 with ErrorActionPreference=Stop on PowerShell 5.1.
    $Saved = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
        & cargo run --locked --offline -p labquest -- @Arguments | Out-Host
        $Code = $LASTEXITCODE
        return [int]$Code
    } finally { $ErrorActionPreference = $Saved }
}
function Run-Action([string]$Choice, [string]$Path) {
    switch ($Choice) {
        'route' { return (Invoke-Learning @('learn', 'route')) }
        'next' { return (Invoke-Learning @('learn', 'next', $Path)) }
        'lesson' { return (Invoke-Learning @('learn', 'lesson', $Path)) }
        'check' { return (Invoke-Learning @('learn', 'check', $Path)) }
        'check-all' { return (Invoke-Learning @('learn', 'check', $Path, 'all')) }
        'hint1' { return (Invoke-Learning @('learn', 'hint', $Path, '1')) }
        'hint2' { return (Invoke-Learning @('learn', 'hint', $Path, '2')) }
        'hint3' { return (Invoke-Learning @('learn', 'hint', $Path, '3')) }
        default { Write-Host '未知操作。'; return 2 }
    }
}
try {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Host '没有找到 Cargo。先按 START_HERE.html 安装 Rust，重新打开窗口。' -ForegroundColor Yellow
        exit 2
    }
    if ($Action -eq 'route') { exit (Run-Action 'route' '') }
    Write-Host 'Rust AppLab 学习导航：先看任务，修改保存，再选择检查。' -ForegroundColor Cyan
    Write-Host '无需重新创建已有工程。路径有空格或中文也可以；输入包含学员 Cargo.toml 的目录。'
    if (-not $Project) {
        $Project = Read-Host '学习目录（例如 my-unitlab；直接 Enter 只查看路线）'
    }
    $Project = $Project.Trim().Trim('"')
    if (-not $Project) { exit (Run-Action 'route' '') }
    $Project = (Resolve-Path -LiteralPath $Project -ErrorAction Stop).Path
    if ($Action) { exit (Run-Action $Action $Project) }
    $Code = Run-Action 'next' $Project
    if ($Code -ne 0) { exit $Code }
    while ($true) {
        Write-Host "`n[P] 作品进度总览  [N] 下一步  [L] 当前任务卡  [1/2/3] 三层提示  [C] 真实检查  [S] 指定检查步骤"
        Write-Host '[E] 打开学员源码  [G] 打开完整讲义  [R] 学习路线  [Q] 保存现状并退出'
        $Choice = (Read-Host '选择（Enter=任务卡）').Trim().ToUpperInvariant()
        $Code = 0
        switch ($Choice) {
            'Q' { exit 0 }
            'P' { $Code = Invoke-Learning @('learn', 'overview', (Split-Path -Parent $Project)) }
            'N' { $Code = Run-Action 'next' $Project }
            ''  { $Code = Run-Action 'lesson' $Project }
            'L' { $Code = Run-Action 'lesson' $Project }
            '1' { $Code = Run-Action 'hint1' $Project }
            '2' { $Code = Run-Action 'hint2' $Project }
            '3' { $Code = Run-Action 'hint3' $Project }
            'C' {
                Write-Host '即将编译运行自己的练习；当前步检查会包含前面步骤的回归。' -ForegroundColor Cyan
                $Code = Run-Action 'check' $Project
            }
            'S' {
                $Step = (Read-Host '检查哪一步（1/2/3/all；独立作品3为完整合同）').Trim()
                if ($Step -notin @('1', '2', '3', 'all')) { Write-Host '请输入 1、2、3 或 all。'; continue }
                $Code = Invoke-Learning @('learn', 'check', $Project, $Step)
            }
            'E' {
                $File = Join-Path $Project 'src\lib.rs'
                if (-not (Test-Path -LiteralPath $File)) { $File = Join-Path $Project 'src\main.rs' }
                if (-not (Test-Path -LiteralPath $File -PathType Leaf)) { Write-Host '未找到学员源码。'; continue }
                Write-Host "只编辑这个文件并按 Ctrl+S 保存：$File"
                Start-Process -FilePath 'notepad.exe' -ArgumentList ('"' + $File + '"')
            }
            'G' {
                $File = Join-Path $Project 'START.html'
                if (-not (Test-Path -LiteralPath $File)) { $File = Join-Path $Project '跟着做.md' }
                if (-not (Test-Path -LiteralPath $File -PathType Leaf)) { $File = Join-Path $Project 'CONTRACT.md' }
                if (Test-Path -LiteralPath $File -PathType Leaf) { Invoke-Item -LiteralPath $File }
            }
            'R' { $Code = Run-Action 'route' $Project }
            default { Write-Host '没有识别这个选择，练习文件未被修改。'; continue }
        }
        if ($Code -ne 0) {
            Write-Host "本次操作未通过（退出码 $Code）。先读第一项错误；修改保存后可在此重试。" -ForegroundColor Yellow
            Write-Host '本窗口不会删除或覆盖练习源码。Q 退出只是结束导航，不代表练习通过。'
        }
    }
} catch {
    Write-Host "导航遇到错误：$($_.Exception.Message)" -ForegroundColor Yellow
    Write-Host '请核对路径；输入应为学员工程，不是平台根目录。已有源码未被导航覆盖。'
    exit 2
}
