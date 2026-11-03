# Windows-native evidence collection, PowerShell 5.1+. No Python or account needed.
[CmdletBinding()]
param([string]$OutputDirectory = '')
$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $Root
$Utf8 = New-Object System.Text.UTF8Encoding($false)
[Console]::OutputEncoding = $Utf8
$OutputEncoding = $Utf8
$Version = [IO.File]::ReadAllText((Join-Path $Root 'VERSION')).Trim()
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
    $OutputDirectory = Join-Path (Split-Path -Parent $Root) ('RustAppLab-native-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0,8))
}
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Evidence directory already exists; choose a NEW directory.' }
$Out = (New-Item -ItemType Directory -Path $OutputDirectory).FullName
$Steps = New-Object System.Collections.ArrayList
$Apps = @('filequest','loglens','localhub','safesync','labquest','unitlab','quizdeck','textscope','ledgerlite','badgeforge','batchmeter','termgarden','storedesk','minikv','netfetch','servicedesk','asyncboard')
function Source-Id {
    $Names = [string[]]@(Get-ChildItem -LiteralPath $Root -Recurse -File -Force | ForEach-Object {
        $Rel = $_.FullName.Substring($Root.Length + 1).Replace('\','/')
        if ($Rel -cnotmatch '(^|/)(target|\.git|__pycache__)/' -and
            (($Rel -notmatch '/') -or ($Rel -cmatch '^(apps|scripts|\.github|course|examples)/') -or ($Rel -cmatch '^projects/(advanced-b|advanced-c1)/')) -and
            (($Rel -ceq 'VERSION') -or ($_.Extension -cin @('.rs','.toml','.lock','.py','.ps1','.cmd','.sh','.html','.css','.js','.yml','.yaml','.md','.tsv','.txt','.json','.kv','.svg')))) { $Rel }
    })
    [Array]::Sort($Names, [StringComparer]::Ordinal)
    $Buffer = New-Object IO.MemoryStream
    $Hash = [Security.Cryptography.SHA256]::Create()
    try {
        foreach ($Name in $Names) {
            $Bytes = $Utf8.GetBytes($Name + [char]0)
            $Buffer.Write($Bytes,0,$Bytes.Length)
            $Digest = $Hash.ComputeHash([IO.File]::ReadAllBytes((Join-Path $Root $Name)))
            $Buffer.Write($Digest,0,$Digest.Length)
        }
        return ([BitConverter]::ToString($Hash.ComputeHash($Buffer.ToArray()))).Replace('-','').ToLowerInvariant()
    } finally { $Hash.Dispose(); $Buffer.Dispose() }
}
function Quote-Native([string]$Value) {
    if ($Value.Length -gt 0 -and $Value -notmatch '[\s"]') { return $Value }
    $Escaped = [regex]::Replace($Value, '(\\*)"', '$1$1\"')
    $Escaped = [regex]::Replace($Escaped, '(\\+)$', '$1$1')
    return '"' + $Escaped + '"'
}
function Step([string]$Name, [string]$Program, [string[]]$Arguments, [bool]$TestSummary = $false, [int]$MinimumPassed = 200, [int]$ExpectedIgnored = 15) {
    Write-Host "[$Name] $Program $($Arguments -join ' ')" -ForegroundColor Cyan
    $Stdout = Join-Path $Out ($Name+'.stdout.txt')
    $Stderr = Join-Path $Out ($Name+'.stderr.txt')
    $Log = Join-Path $Out ($Name+'.txt')
    $Timer = [Diagnostics.Stopwatch]::StartNew()
    $File = (Get-Command $Program -ErrorAction Stop).Source
    $Quoted = ($Arguments | ForEach-Object { Quote-Native $_ }) -join ' '
    $Process = Start-Process -FilePath $File -ArgumentList $Quoted -WorkingDirectory $Root -PassThru -NoNewWindow -RedirectStandardOutput $Stdout -RedirectStandardError $Stderr
    $null = $Process.Handle
    $TimedOut = -not $Process.WaitForExit(600000)
    if ($TimedOut) {
        & taskkill.exe /PID $Process.Id /T /F | Out-Null
        $Process.WaitForExit()
        $Code = 124
    } else { $Process.WaitForExit(); $Code = $Process.ExitCode }
    $Text = [IO.File]::ReadAllText($Stdout,$Utf8) + "`n" + [IO.File]::ReadAllText($Stderr,$Utf8)
    [IO.File]::WriteAllText($Log, ('$ '+$Program+' '+$Quoted+"`n"+$Text),$Utf8)
    $Passed = ($Code -eq 0)
    $Result = [ordered]@{step=$Name;command=@($Program)+$Arguments;exit_code=$Code;timed_out=$TimedOut;seconds=[math]::Round($Timer.Elapsed.TotalSeconds,3);log=[IO.Path]::GetFileName($Log);log_sha256=(Get-FileHash -LiteralPath $Log -Algorithm SHA256).Hash.ToLowerInvariant();passed=$Passed}
    if ($TestSummary) {
        $Matches = [regex]::Matches($Text,'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;')
        $Good=0; $Bad=0; $Ignored=0; $AllOk=$true
        foreach ($M in $Matches) {
            $Good += [int]$M.Groups[2].Value; $Bad += [int]$M.Groups[3].Value; $Ignored += [int]$M.Groups[4].Value
            if ($M.Groups[1].Value -cne 'ok') { $AllOk=$false }
        }
        $Result.tests = @{suites=$Matches.Count;passed=$Good;failed=$Bad;ignored=$Ignored}
        $Result.passed = $Passed -and $Matches.Count -gt 0 -and $Good -ge $MinimumPassed -and $Bad -eq 0 -and $Ignored -eq $ExpectedIgnored -and $AllOk
    }
    [void]$Steps.Add([pscustomobject]$Result)
    if (-not $Result.passed) { Write-Host $Text; throw "$Name failed (exit=$Code). Log: $Log" }
    Write-Host "PASS $Name" -ForegroundColor Green
}
$StartId = Source-Id
$Report = [ordered]@{schema=1;version=$Version;source_id=$StartId;system='Windows';platform=[Environment]::OSVersion.VersionString;machine=$env:PROCESSOR_ARCHITECTURE;generated_utc=[DateTime]::UtcNow.ToString('o');engineering_gate='RUNNING';scope='Native engineering only; browser and learner outcomes are separate';unchecked=@('Real browser HTTP navigation','MSRV unless rustc is 1.74.0','Real learner outcomes');steps=@()}
$Exit = 1
$env:RUST_BACKTRACE='0'
$env:CARGO_TERM_COLOR='never'
$env:CARGO_TARGET_DIR=Join-Path $Root 'target'
Remove-Item Env:CARGO_BUILD_TARGET -ErrorAction SilentlyContinue
try {
    Step 'rustc' 'rustc' @('-Vv')
    Step 'cargo' 'cargo' @('-V')
    # Check optional toolchain components before beginning a long release check.
    # Missing rustfmt/clippy is an environment prerequisite failure, not a Rust source failure.
    try { Step 'rustfmt-component' 'cargo' @('fmt','--version') }
    catch {
        throw ('Missing or unusable rustfmt. Run: rustup component add rustfmt ; then verify: cargo fmt --version. Details: ' + $_.Exception.Message)
    }
    try { Step 'clippy-component' 'cargo' @('clippy','--version') }
    catch {
        throw ('Missing or unusable clippy. Run: rustup component add clippy ; then verify: cargo clippy --version. Details: ' + $_.Exception.Message)
    }
    Step 'format' 'cargo' @('fmt','--all','--','--check')
    Step 'debug-clippy' 'cargo' @('clippy','--locked','--offline','--workspace','--all-targets','--','-D','warnings')
    Step 'release-clippy' 'cargo' @('clippy','--locked','--offline','--release','--workspace','--all-targets','--','-D','warnings')
    Step 'debug-tests' 'cargo' @('test','--locked','--offline','--workspace','--all-targets','--','--test-threads=2') $true
    Step 'release-tests' 'cargo' @('test','--locked','--offline','--release','--workspace','--all-targets','--','--test-threads=2') $true
    Step 'doctests' 'cargo' @('test','--locked','--offline','--workspace','--doc') $true 2 0
    Step 'release-build' 'cargo' @('build','--locked','--offline','--workspace','--release')
    # Match release_check.py advanced_b_steps(): this reference is a separate workspace.
    $ReferenceManifest = 'projects/advanced-b/reference/Cargo.toml'
    $ReferenceCommon = @('--locked','--offline','--manifest-path',$ReferenceManifest,'--workspace')
    $Configurations = @(
        @{Name='default'; Features=@()},
        @{Name='strict'; Features=@('--all-features')}
    )
    Step 'advanced-b-format' 'cargo' @('fmt','--manifest-path',$ReferenceManifest,'--all','--','--check')
    foreach ($Configuration in $Configurations) {
        Step ('advanced-b-clippy-'+$Configuration.Name) 'cargo' (@('clippy')+$ReferenceCommon+@('--all-targets')+$Configuration.Features+@('--','-D','warnings'))
    }
    foreach ($BuildProfile in @(@{Name='debug'; Flags=@()}, @{Name='release'; Flags=@('--release')})) {
        foreach ($Configuration in $Configurations) {
            Step ('advanced-b-'+$BuildProfile.Name+'-'+$Configuration.Name) 'cargo' (@('test')+$ReferenceCommon+$BuildProfile.Flags+@('--all-targets')+$Configuration.Features+@('--','--test-threads=2')) $true 12 0
        }
    }
    Step 'advanced-b-no-default' 'cargo' (@('test')+$ReferenceCommon+@('--all-targets','--no-default-features','--','--test-threads=2')) $true 12 0
    foreach ($Configuration in $Configurations) {
        Step ('advanced-b-doctests-'+$Configuration.Name) 'cargo' (@('test')+$ReferenceCommon+$Configuration.Features+@('--doc')) $true 1 0
    }
    # Match release_check.py advanced_b4_steps(): 22 behavior tests, one doctest, platform E2E.
    $JsonManifest = 'projects/advanced-b/reference/json-export/Cargo.toml'
    $JsonCommon = @('--locked','--offline','--manifest-path',$JsonManifest,'--workspace')
    Step 'advanced-b4-format' 'cargo' @('fmt','--manifest-path',$JsonManifest,'--all','--','--check')
    foreach ($Configuration in $Configurations) {
        Step ('advanced-b4-clippy-'+$Configuration.Name) 'cargo' (@('clippy')+$JsonCommon+@('--all-targets')+$Configuration.Features+@('--','-D','warnings'))
    }
    foreach ($BuildProfile in @(@{Name='debug'; Flags=@()}, @{Name='release'; Flags=@('--release')})) {
        foreach ($Configuration in $Configurations) {
            Step ('advanced-b4-'+$BuildProfile.Name+'-'+$Configuration.Name) 'cargo' (@('test')+$JsonCommon+$BuildProfile.Flags+@('--all-targets')+$Configuration.Features+@('--','--test-threads=2')) $true 20 0
        }
    }
    foreach ($Configuration in $Configurations) {
        Step ('advanced-b4-doctests-'+$Configuration.Name) 'cargo' (@('test')+$JsonCommon+$Configuration.Features+@('--doc')) $true 1 0
    }
    Step 'advanced-b4-platform-e2e' 'cargo' @('test','--locked','--offline','-p','labquest','--features','b4-integration','--test','advanced_b_json_e2e','--','--test-threads=1') $true 5 0
    foreach ($Script in @('windows_beginner_smoke','windows_database_smoke','windows_studio_smoke')) {
        Step $Script 'powershell.exe' @('-NoLogo','-NoProfile','-ExecutionPolicy','Bypass','-File',(Join-Path $PSScriptRoot ($Script+'.ps1')))
    }
    $EndId = Source-Id
    $Report.source_id_after = $EndId
    if ($EndId -ne $StartId) { throw 'Source changed during validation; results do not qualify this tree.' }
    $Bin = Join-Path $Out 'windows-native-programs'
    New-Item -ItemType Directory -Path $Bin | Out-Null
    $Hashes = New-Object System.Collections.ArrayList
    foreach ($App in $Apps) {
        $Source = Join-Path $Root ('target\release\'+$App+'.exe')
        if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) { throw "Missing binary: $Source" }
        Copy-Item -LiteralPath $Source -Destination $Bin
        [void]$Hashes.Add(((Get-FileHash -LiteralPath $Source -Algorithm SHA256).Hash.ToLowerInvariant()+'  '+$App+'.exe'))
    }
    [IO.File]::WriteAllText((Join-Path $Bin 'SHA256SUMS.txt'),(($Hashes -join "`n")+"`n"),$Utf8)
    Copy-Item -LiteralPath (Join-Path $Root 'LICENSE') -Destination $Bin
    [IO.File]::WriteAllText((Join-Path $Bin 'README.txt'),"Rust AppLab $Version locally built Windows programs.`r`nNative engineering passed on this machine. Browser/manual acceptance is NOT certified.`r`nLearner checks still need Rust/Cargo. Do not expose HTTP servers to the Internet.`r`n",$Utf8)
    $Report.engineering_gate='PASS'; $Exit=0
} catch {
    $Report.engineering_gate='FAIL'
    $Report.error=$_.Exception.Message
    Write-Host $Report.error -ForegroundColor Red
} finally {
    $Report.steps = $Steps.ToArray()
    [IO.File]::WriteAllText((Join-Path $Out 'engineering.json'),($Report|ConvertTo-Json -Depth 10),$Utf8)
    try {
        Compress-Archive -Path (Join-Path $Out '*') -DestinationPath ($Out+'.zip') -ErrorAction Stop
        Write-Host "Evidence bundle: $Out.zip"
    } catch { Write-Host "ZIP failed; logs remain in: $Out" }
    Write-Host "Native engineering: $($Report.engineering_gate). Browser and overall release qualification are separate."
}
exit $Exit
