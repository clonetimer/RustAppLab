# Run only in the generated dependency kit. No user profiles or Cargo credentials are included.
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$Root = $PSScriptRoot
$Out = $Root + '.zip'
if (Test-Path -LiteralPath $Out) { throw "Archive already exists. Rename it before packing again: $Out" }
if (-not (Test-Path -LiteralPath (Join-Path $Root '.rustapplab-network-kit'))) { throw 'Not an AppLab dependency kit.' }
[IO.Compression.ZipFile]::CreateFromDirectory($Root, $Out, [IO.Compression.CompressionLevel]::Optimal, $false)
$Hash = (Get-FileHash -LiteralPath $Out -Algorithm SHA256).Hash.ToLowerInvariant()
[IO.File]::WriteAllText($Out + '.sha256', $Hash + '  ' + [IO.Path]::GetFileName($Out) + "`n", (New-Object Text.UTF8Encoding($false)))
Write-Host "Created $Out"
Write-Host 'STATUS.md records what actually passed. A ZIP alone does not certify the network lessons.'
