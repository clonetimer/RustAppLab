# 新作品入口。只创建全新目录，不下载软件，不覆盖学员源文件。
$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $PSScriptRoot
$Ids = @('unitlab','quizdeck','textscope','ledgerlite','badgeforge','batchmeter','termgarden','storedesk','minikv','netfetch','servicedesk','asyncboard')
$Titles = @('单位换算器：先做这个','知识闯关：自己的题库','文本观察镜：词频与搜索','生活账本：文件持久化','SVG 作品卡：trait 与泛型','并行统计台：借用与线程','任务花园：状态与交互','库存台：真实 SQLite 事务','键值仓库：错误、迭代器与共享状态','本机下载器：HTTP、TCP与文件保存','本地工单服务：业务状态与浏览器API','异步提醒工作台：Future、await与Waker')
function Run-Studio([string[]]$Arguments) {
    & cargo run --offline -p labquest -- studio @Arguments | Out-Host
    return [int]$LASTEXITCODE
}
try {
    Set-Location -LiteralPath $Root
    Write-Host 'Rust AppLab 教学开发版 · 第一次任务卡之后，从这里继续' -ForegroundColor Cyan
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Host '没有找到 Cargo。请先完成 START_HERE.html 的安装步骤。' -ForegroundColor Yellow
        exit 2
    }
    for ($i=0; $i -lt $Ids.Count; $i++) { Write-Host ("{0}. {1}" -f ($i+1),$Titles[$i]) }
    $Choice = Read-Host '选择 1–12；基础较弱请按 Enter 选 1。Q 退出'
    if ($Choice -eq 'Q') { exit 0 }
    if ([string]::IsNullOrWhiteSpace($Choice)) { $Choice = '1' }
    if ($Choice -notin @('1','2','3','4','5','6','7','8','9','10','11','12')) { Write-Host '请输入 1–12，下次启动继续。'; exit 2 }
    $Id = $Ids[[int]$Choice-1]
    if ($Id -eq 'storedesk') {
        Write-Host '先检查本机 SQLite 链接与运行环境；这里还不检查你的练习代码。' -ForegroundColor Cyan
        & cargo run --offline -p storedesk -- doctor
        if ($LASTEXITCODE -ne 0) {
            Write-Host '请先读 course/SQLITE_SETUP.md。Windows MSVC 需要 Windows 10/11 SDK 的 winsqlite3.lib；不要为此删除练习或重装 Rust。' -ForegroundColor Yellow
            exit 2
        }
    }
    $Project = Join-Path $Root ("my-"+$Id)
    if (Test-Path -LiteralPath $Project) {
        $Marker = Join-Path $Project '.applab-studio'
        if (-not (Test-Path -LiteralPath $Marker -PathType Leaf)) { Write-Host '同名目录已存在但不是学习工程；不会覆盖，请自行换名或使用手动命令。'; exit 2 }
        $Expected = "RustAppLab Studio v1`n$Id`n"
        $Actual = [IO.File]::ReadAllText($Marker).Replace("`r`n","`n")
        if ($Actual -ne $Expected) { Write-Host '同名目录属于其它工程；不会覆盖。'; exit 2 }
        Write-Host "继续已有工程，不会覆盖：$Project" -ForegroundColor Yellow
    } else {
        $Code = Run-Studio @('start',$Id,$Project)
        if ($Code -ne 0) { exit $Code }
    }
    Write-Host '每款作品分三步。先打开讲义，只修改 src/lib.rs 的当前 TODO。' -ForegroundColor Green
    try { Start-Process -FilePath (Join-Path $Project 'START.html') } catch { Write-Host "请自行打开 $Project\START.html" }
    $Source = Join-Path $Project 'src\lib.rs'
    try { Start-Process -FilePath 'notepad.exe' -ArgumentList ('"'+$Source+'"') } catch { Write-Host "请自行用编辑器打开 $Source" }
    $Step = 1
    while ($true) {
        Write-Host "`n当前推荐：第 $Step 步。编辑并 Ctrl+S 保存后检查。"
        $InputText = Read-Host 'Enter 检查当前步；1/2/3 选阶段；H 提示；O 重开讲义；Q 退出'
        if ($InputText -eq 'Q') { Write-Host '源码已保留，下次双击继续。'; exit 0 }
        if ($InputText -eq 'H') { $null = Run-Studio @('hint',$Id); continue }
        if ($InputText -eq 'O') { Start-Process -FilePath (Join-Path $Project 'START.html'); continue }
        if ($InputText -in @('1','2','3')) { $Step = [int]$InputText }
        elseif (-not [string]::IsNullOrWhiteSpace($InputText)) { Write-Host '请输入有效选项。'; continue }
        $Code = Run-Studio @('check',$Project,[string]$Step)
        if ($Code -eq 0) {
            if ($Step -lt 3) { $Step++; Write-Host '当前阶段通过。先读下一步讲义，再修改对应 TODO。' -ForegroundColor Green }
            else {
                Write-Host '三步功能验收通过。现在用你自己的数据实际运行软件并保存作品。' -ForegroundColor Green
                Write-Host ('在新 PowerShell 中进入：cd "'+$Project+'"')
                Write-Host '查看命令：cargo run --offline -- help'
                Write-Host '交付编译：cargo build --offline --release'
                Write-Host '完成 LEARNING_LOG.md；下一款作品仍从 CONTINUE_WINDOWS.cmd 选择。'
                exit 0
            }
        } else { Write-Host '先处理第一个 FAIL。不要同时改三个 TODO，不要删除测试。' -ForegroundColor Yellow }
    }
} catch {
    Write-Host "发生错误：$($_.Exception.Message)" -ForegroundColor Red
    Write-Host '源码未被本向导主动清理。保留上方报错，或使用 WORKSHOP.html 的手动命令。'
    exit 2
}
