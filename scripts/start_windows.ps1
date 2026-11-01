# Rust AppLab 0.11.1 - Windows zero-prerequisite console guidance (except Rust installation).
# Executed by START_WINDOWS.cmd; never deletes files, silently installs software or writes outside the repo.
$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $PSScriptRoot
$ProjectName = 'my-first-card'
$Project = Join-Path $Root $ProjectName

function Header([string]$Title) {
    Write-Host "`n==========================================================" -ForegroundColor Cyan
    Write-Host $Title -ForegroundColor Cyan
    Write-Host "==========================================================" -ForegroundColor Cyan
}
function Pause-Lesson([string]$Message) {
    $null = Read-Host $Message
}
function Run-LabQuest([string[]]$Arguments) {
    & cargo run --offline -p labquest -- @Arguments | Out-Host
    return [int]$LASTEXITCODE
}
function Help-Install {
    Write-Host '电脑还不能识别 Rust 工具。请先完成安装：' -ForegroundColor Yellow
    Write-Host '  1. 浏览器打开 https://rustup.rs/ ，按 Windows 安装说明安装 Rust。'
    Write-Host '  2. 如提示安装 MSVC C++ Build Tools，请安装「使用 C++ 的桌面开发」组件。'
    Write-Host '  3. 完成后关掉这个黑色窗口，再次双击 START_WINDOWS.cmd。'
    Write-Host '  4. 不确定时可先打开同目录的 START_HERE.html 看图文说明。'
}
try {
    Set-Location -LiteralPath $Root
    Header '第 0 关：30 分钟做出你的第一个 Rust 小软件'
    Write-Host '你只需要跟着提示操作；不必先理解所有术语。'
    Write-Host '本向导不会删除现有目录，也不会下载陌生程序。'
    Write-Host "当前工程目录：$Root"
    Pause-Lesson '准备好后按 Enter，先检查 Rust 是否装好'

    $Cargo = Get-Command cargo -ErrorAction SilentlyContinue
    $Rust = Get-Command rustc -ErrorAction SilentlyContinue
    if (-not $Cargo -or -not $Rust) {
        Help-Install
        exit 2
    }
    Write-Host '`n检测到 Rust：' -ForegroundColor Green
    & rustc --version
    & cargo --version
    if ($LASTEXITCODE -ne 0) { Help-Install; exit 2 }

    Header '第 1 步：自动创建你的第一个练习项目'
    if (Test-Path -LiteralPath $Project) {
        if (-not (Test-Path -LiteralPath (Join-Path $Project 'src\main.rs'))) {
            Write-Host "文件夹 $Project 已存在，但不是预期练习；为保护你的文件，本向导不会覆盖它。" -ForegroundColor Yellow
            Write-Host '请把现有文件夹自行改名后重新启动，或采用课程中的手动命令选择新目录。'
            exit 2
        }
        Write-Host '发现之前的练习文件；不会覆盖，直接继续。' -ForegroundColor Yellow
    } else {
        # 使用绝对路径，避免受工作目录及旧版相对路径判断的影响。
        $Code = Run-LabQuest @('beginner', 'start', $Project)
        if ($Code -ne 0) {
            Write-Host '无法创建练习工程。请保留上方具体报错；这是创建目录阶段，不需要重新安装 MSVC。' -ForegroundColor Yellow
            exit $Code
        }
    }

    Header '第 2 步：先运行没有完成的程序，观察变化前的状态'
    Write-Host '下面会编译并运行第一次任务卡。你应看到「等待完成第一张任务卡」。'
    & cargo run --offline --manifest-path (Join-Path $Project 'Cargo.toml') -- '小林' '写 Rust' | Out-Host
    if ($LASTEXITCODE -ne 0) {
        Write-Host '第一次编译未通过。如果包含 link.exe / linker 错误，请先安装 Rust 官方建议的 Windows C++ 构建工具。' -ForegroundColor Yellow
        exit 2
    }

    Header '第 3 步：只修改你要练习的唯一一行'
    $Source = Join-Path $Project 'src\main.rs'
    Write-Host "马上用记事本打开：$Source"
    Write-Host '找到 println!("等待完成第一张任务卡"); 这一行。'
    Write-Host '目标：让程序显示「今日任务卡 | 小林 | 写 Rust」。'
    Write-Host '修改后请按 Ctrl+S 保存，然后回到这个窗口。'
    Pause-Lesson '按 Enter 打开记事本'
    try {
        Start-Process -FilePath 'notepad.exe' -ArgumentList ('"' + $Source + '"')
    } catch {
        Write-Host "无法自动打开记事本。可以自行用 VS Code/记事本打开：$Source" -ForegroundColor Yellow
    }

    Header '第 4 步：运行真正的 Rust 自动检查'
    Write-Host '检查会试多个名字，不能只写死一句输出。失败很正常，修改并保存后可以反复试。'
    while ($true) {
        $InputText = Read-Host '保存文件后按 Enter 检查；输入 H 查看提示；输入 Q 退出'
        if ($InputText -eq 'Q' -or $InputText -eq 'q') {
            Write-Host '练习文件已保留。下次可重新双击入口继续。'
            exit 0
        }
        if ($InputText -eq 'H' -or $InputText -eq 'h') {
            $Tier = Read-Host '选提示级别 1/2/3（先看 1，再看 2 和 3）'
            if ($Tier -notin @('1', '2', '3')) { $Tier = '1' }
            $null = Run-LabQuest @('beginner', 'hint', $Tier)
            continue
        }
        $Code = Run-LabQuest @('beginner', 'check', $Project)
        if ($Code -eq 0) { break }
        Write-Host '没有通过不等于你失败。先看上方「期待 / 实际」，只改那一行再试。' -ForegroundColor Yellow
    }

    Header '恭喜：你的第一款 Rust 软件已经通过检查'
    Write-Host "完成记录：$ProjectName\FIRST_WIN.md" -ForegroundColor Green
    Write-Host '现在进入自己的工程目录，试着运行：'
    Write-Host ('  cd "' + $Project + '"')
    Write-Host '  cargo run --offline -- "我的名字" "今天的任务"'
    Write-Host '接下来双击 CONTINUE_WINDOWS.cmd，从 UnitLab 单位换算器开始，先不要跳进大型工程。'
    exit 0
} catch {
    Write-Host "`n发生了意外：$($_.Exception.Message)" -ForegroundColor Red
    Write-Host '当前练习文件没有被主动清理。查看 START_HERE.html 的常见错误章节。'
    exit 2
}
