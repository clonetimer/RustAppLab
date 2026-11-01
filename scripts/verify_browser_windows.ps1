# Rust AppLab browser acceptance (Windows PowerShell 5.1+).
# Real Chromium/Edge -> local Rust HTTP -> disk. Do NOT bypass browser policies.
[CmdletBinding()]
param(
    [string]$OutputDirectory = '',
    [int]$BuildTimeoutSeconds = 900,
    [int]$BrowserTimeoutSeconds = 300
)
$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $Root
$Utf8 = New-Object System.Text.UTF8Encoding($false)
[Console]::OutputEncoding = $Utf8
$OutputEncoding = $Utf8
$Version = [IO.File]::ReadAllText((Join-Path $Root 'VERSION')).Trim()
if ($BuildTimeoutSeconds -lt 30 -or $BrowserTimeoutSeconds -lt 30) {
    throw 'Timeout values must be at least 30 seconds.'
}
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
    $OutputDirectory = Join-Path (Split-Path -Parent $Root) ('RustAppLab-browser-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0,8))
}
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Evidence directory already exists; choose a NEW directory.' }
$Out = [IO.Path]::GetFullPath($OutputDirectory)
$null = New-Item -ItemType Directory -Path $Out -Force
$StatusFile = Join-Path $Out 'launcher.json'
$Steps = New-Object System.Collections.ArrayList
$Exit = 1
$Stage = 'initialization'
function Save-Status([string]$State, [string]$Message) {
    $Data = [ordered]@{
        version=$Version; state=$State; stage=$Stage; message=$Message;
        generated_utc=[DateTime]::UtcNow.ToString('o'); steps=$Steps.ToArray()
    }
    [IO.File]::WriteAllText($StatusFile, ($Data | ConvertTo-Json -Depth 8), $Utf8)
}
# Start-Process on Windows joins -ArgumentList; quote each argument explicitly.
function Quote-WindowsArgument([string]$Value) {
    if ($Value.Length -gt 0 -and $Value -notmatch '[\s"]') { return $Value }
    $Escaped = [regex]::Replace($Value, '(\\*)"', '$1$1\"')
    $Escaped = [regex]::Replace($Escaped, '(\\+)$', '$1$1')
    return '"' + $Escaped + '"'
}
function Invoke-LoggedProcess([string]$Name, [string]$Executable, [string[]]$Arguments, [int]$TimeoutSeconds) {
    $Stdout = Join-Path $Out ($Name + '.stdout.txt')
    $Stderr = Join-Path $Out ($Name + '.stderr.txt')
    $CommandLine = ($Arguments | ForEach-Object { Quote-WindowsArgument $_ }) -join ' '
    Write-Host "[$Name] $Executable $CommandLine" -ForegroundColor Cyan
    Save-Status 'RUNNING' "$Name started"
    $Clock = [Diagnostics.Stopwatch]::StartNew()
    $Process = Start-Process -FilePath $Executable -ArgumentList $CommandLine `
        -WorkingDirectory $Root -PassThru -NoNewWindow `
        -RedirectStandardOutput $Stdout -RedirectStandardError $Stderr -ErrorAction Stop
    $null = $Process.Handle
    $TimedOut = $false
    $NextHeartbeat = 15
    while (-not $Process.WaitForExit(1000)) {
        $Elapsed = [int]$Clock.Elapsed.TotalSeconds
        if ($Elapsed -ge $TimeoutSeconds) {
            $TimedOut = $true
            Write-Host "[$Name] Timeout after ${TimeoutSeconds}s; stopping this process tree." -ForegroundColor Red
            try { & taskkill.exe /PID $Process.Id /T /F | Out-Null } catch { }
            try { if (-not $Process.WaitForExit(5000)) { $Process.Kill() } } catch { }
            break
        }
        if ($Elapsed -ge $NextHeartbeat) {
            Write-Host "[$Name] Still running (${Elapsed}s). Latest output:" -ForegroundColor Yellow
            try {
                if (Test-Path -LiteralPath $Stdout) {
                    Get-Content -LiteralPath $Stdout -Tail 4 -ErrorAction Stop | ForEach-Object { Write-Host "  $_" }
                }
                if (Test-Path -LiteralPath $Stderr) {
                    Get-Content -LiteralPath $Stderr -Tail 4 -ErrorAction Stop | ForEach-Object { Write-Host "  $_" }
                }
            } catch { Write-Host "  Logs are being written to $Out" }
            $NextHeartbeat = $Elapsed + 15
        }
    }
    try { $Process.WaitForExit() } catch { }
    $Code = if ($TimedOut) { 124 } else { $Process.ExitCode }
    $StepResult = [ordered]@{
        name=$Name; exit_code=$Code; timed_out=$TimedOut;
        seconds=[math]::Round($Clock.Elapsed.TotalSeconds,1);
        stdout=[IO.Path]::GetFileName($Stdout); stderr=[IO.Path]::GetFileName($Stderr)
    }
    [void]$Steps.Add([pscustomobject]$StepResult)
    Save-Status 'RUNNING' "$Name exited with code $Code"
    if ($Code -ne 0) {
        Write-Host "[$Name] Failed, exit code $Code. Recent stderr:" -ForegroundColor Red
        try { Get-Content -LiteralPath $Stderr -Tail 25 | ForEach-Object { Write-Host $_ } } catch { }
    } else { Write-Host "[$Name] PASS" -ForegroundColor Green }
    return [int]$Code
}
function Find-Python {
    $Command = Get-Command py.exe -ErrorAction SilentlyContinue
    if ($null -ne $Command) { return @{ exe=$Command.Source; prefix=@('-3') } }
    $Command = Get-Command python.exe -ErrorAction SilentlyContinue
    if ($null -ne $Command) { return @{ exe=$Command.Source; prefix=@() } }
    return $null
}
function Find-Browser {
    foreach ($EnvName in @('CHROME_PATH','EDGE_PATH','CHROMIUM_PATH','ProgramFiles','ProgramFiles(x86)','LOCALAPPDATA')) {
        $Base = [Environment]::GetEnvironmentVariable($EnvName)
        if ([string]::IsNullOrWhiteSpace($Base)) { continue }
        if ($EnvName -in @('CHROME_PATH','EDGE_PATH','CHROMIUM_PATH')) {
            if (Test-Path -LiteralPath $Base -PathType Leaf) { return $Base }
        } else {
            foreach ($Relative in @('Google\Chrome\Application\chrome.exe','Microsoft\Edge\Application\msedge.exe')) {
                $Candidate = Join-Path $Base $Relative
                if (Test-Path -LiteralPath $Candidate -PathType Leaf) { return $Candidate }
            }
        }
    }
    return ''
}
try {
    $Stage = 'preflight'
    Write-Host '[preflight] Checking Python 3 and Playwright BEFORE compiling Rust...' -ForegroundColor Cyan
    $Python = Find-Python
    if ($null -eq $Python) {
        $Exit = 2
        throw 'Python 3 missing. Install Python 3.10+; then run: py -3 -m pip install playwright'
    }
    $PreflightArgs = @($Python.prefix) + @('-c', 'import sys; import playwright.sync_api; assert sys.version_info >= (3, 10); print(sys.version)')
    $PreflightCode = Invoke-LoggedProcess 'python-playwright-preflight' $Python.exe $PreflightArgs 30
    if ($PreflightCode -ne 0) {
        $Exit = 2
        throw 'Python 3 / Playwright preflight failed. Install Playwright: py -3 -m pip install playwright (a browser is also needed).'
    }
    $Browser = Find-Browser
    if ($Browser) { Write-Host "[preflight] Installed browser: $Browser" -ForegroundColor Cyan }
    else { Write-Host '[preflight] No Chrome/Edge found in common paths; Playwright-managed Chromium may still work.' -ForegroundColor Yellow }
    $Cargo = Get-Command cargo.exe -ErrorAction SilentlyContinue
    if ($null -eq $Cargo) { $Exit = 2; throw 'cargo.exe not found on PATH.' }
    $Stage = 'rust-build'
    $BuildArgs = @('build','--locked','--offline','--release','-p','localhub','-p','servicedesk')
    $BuildCode = Invoke-LoggedProcess 'cargo-build' $Cargo.Source $BuildArgs $BuildTimeoutSeconds
    if ($BuildCode -ne 0) { throw 'Rust build failed or timed out. Check cargo-build.stderr.txt in the evidence directory.' }
    $Stage = 'browser-e2e'
    $Script = Join-Path $PSScriptRoot 'browser_e2e.py'
    $BrowserArgs = @($Python.prefix) + @($Script, '--output', (Join-Path $Out 'browser-results'))
    if ($Browser) { $BrowserArgs += @('--browser-path', $Browser) }
    Write-Host '[browser] Real Chrome/Edge testing LocalHub and ServiceDesk over localhost. No mocks or policy bypass.' -ForegroundColor Cyan
    $BrowserCode = Invoke-LoggedProcess 'browser-e2e' $Python.exe $BrowserArgs $BrowserTimeoutSeconds
    if ($BrowserCode -ne 0) {
        $Exit = $BrowserCode
        throw "Browser test failed/blocked or timed out (exit $BrowserCode). Review browser-results/browser.json and browser-e2e logs."
    }
    $Exit = 0
    $Stage = 'complete'
    Save-Status 'PASS' 'Both real browser application flows passed.'
    Write-Host '[result] Browser acceptance PASS' -ForegroundColor Green
} catch {
    if ($Exit -eq 0) { $Exit = 1 }
    Write-Host "[result] $($_.Exception.Message)" -ForegroundColor Red
    $FailureState = if ($Exit -eq 2) { 'BLOCKED_ENVIRONMENT' } elseif ($Exit -eq 124) { 'TIMEOUT' } else { 'FAIL' }
    Save-Status $FailureState $_.Exception.Message
} finally {
    $Zip = $Out + '.zip'
    try {
        Compress-Archive -Path (Join-Path $Out '*') -DestinationPath $Zip -ErrorAction Stop
        Write-Host "Evidence ZIP: $Zip" -ForegroundColor Cyan
    } catch { Write-Host "Could not package evidence: $($_.Exception.Message); files remain in $Out" -ForegroundColor Yellow }
    Write-Host "Browser acceptance exit=$Exit. Evidence directory: $Out"
}
exit $Exit
