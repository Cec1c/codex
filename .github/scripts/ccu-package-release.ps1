[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$UpstreamVersion,

    [Parameter(Mandatory)]
    [string]$UpstreamTag,

    [Parameter(Mandatory)]
    [ValidatePattern('^[a-f0-9]{40}$')]
    [string]$UpstreamCommit,

    [Parameter(Mandatory)]
    [ValidatePattern('^[a-f0-9]{40}$')]
    [string]$ForkCommit,

    [Parameter(Mandatory)]
    [ValidateRange(1, [int]::MaxValue)]
    [int]$Revision,

    [Parameter(Mandatory)]
    [string]$DisplayVersion,

    [Parameter(Mandatory)]
    [string]$BinaryPath,

    [Parameter(Mandatory)]
    [string]$OutputDirectory
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Test-Path -LiteralPath $BinaryPath -PathType Leaf)) {
    throw "Codex binary does not exist: $BinaryPath"
}
if ($UpstreamTag -ne "rust-v$UpstreamVersion") {
    throw "Upstream tag $UpstreamTag does not match version $UpstreamVersion"
}
if ($DisplayVersion -ne "$UpstreamVersion-ccu.i18n.$Revision") {
    throw "Display version $DisplayVersion does not match the CCU version contract"
}

$target = 'x86_64-pc-windows-msvc'
$releaseTag = "ccu-rust-v$UpstreamVersion-r$Revision"
$assetName = "codex-ccu-i18n-$UpstreamVersion-r$Revision-$target.zip"
$outputRoot = [System.IO.Path]::GetFullPath($OutputDirectory)
$stagingRoot = Join-Path $outputRoot 'staging'
$packageRoot = Join-Path $stagingRoot 'package\bin'
$assetPath = Join-Path $outputRoot $assetName
$manifestPath = Join-Path $outputRoot 'ccu-fork-manifest.json'

New-Item -ItemType Directory -Path $outputRoot -Force | Out-Null
Remove-Item -LiteralPath $stagingRoot -Recurse -Force -ErrorAction SilentlyContinue
foreach ($path in @($assetPath, "$assetPath.sha256", $manifestPath, "$manifestPath.sha256")) {
    Remove-Item -LiteralPath $path -Force -ErrorAction SilentlyContinue
}
New-Item -ItemType Directory -Path $packageRoot -Force | Out-Null
Copy-Item -LiteralPath $BinaryPath -Destination (Join-Path $packageRoot 'codex.exe')
Compress-Archive -LiteralPath (Join-Path $stagingRoot 'package') -DestinationPath $assetPath -CompressionLevel Optimal

$assetFile = Get-Item -LiteralPath $assetPath
$assetHash = (Get-FileHash -LiteralPath $assetPath -Algorithm SHA256).Hash.ToLowerInvariant()
$manifest = [ordered]@{
    schemaVersion = 1
    type = 'codex-ccu-i18n-build'
    releaseTag = $releaseTag
    displayVersion = $DisplayVersion
    upstreamVersion = $UpstreamVersion
    upstreamTag = $UpstreamTag
    upstreamCommit = $UpstreamCommit
    forkCommit = $ForkCommit
    ultraRevision = $Revision
    i18nApiVersion = 1
    platform = $target
    asset = [ordered]@{
        name = $assetName
        size = $assetFile.Length
        sha256 = "sha256:$assetHash"
    }
}

$manifest | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $manifestPath -Encoding utf8NoBOM
"$assetHash  $assetName" | Set-Content -LiteralPath "$assetPath.sha256" -Encoding ascii
$manifestHash = (Get-FileHash -LiteralPath $manifestPath -Algorithm SHA256).Hash.ToLowerInvariant()
"$manifestHash  ccu-fork-manifest.json" | Set-Content -LiteralPath "$manifestPath.sha256" -Encoding ascii
Remove-Item -LiteralPath $stagingRoot -Recurse -Force

$manifest | ConvertTo-Json -Depth 5
