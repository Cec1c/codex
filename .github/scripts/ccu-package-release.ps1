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
    [string]$ReleaseTag,

    [Parameter(Mandatory)]
    [ValidateSet('windows-x64', 'linux-x64', 'linux-arm64', 'macos-x64', 'macos-arm64')]
    [string]$RuntimeId,

    [Parameter(Mandatory)]
    [string]$BinaryPath,

    [Parameter(Mandatory)]
    [string]$OutputDirectory
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Assert-ChildPath {
    param(
        [Parameter(Mandatory)] [string]$Root,
        [Parameter(Mandatory)] [string]$Candidate,
        [Parameter(Mandatory)] [string]$Label
    )

    $resolvedRoot = [System.IO.Path]::GetFullPath($Root)
    $resolvedCandidate = [System.IO.Path]::GetFullPath($Candidate)
    $separator = [System.IO.Path]::DirectorySeparatorChar.ToString()
    $rootPrefix = if ($resolvedRoot.EndsWith($separator, [System.StringComparison]::Ordinal)) {
        $resolvedRoot
    }
    else {
        "$resolvedRoot$separator"
    }
    if (-not $resolvedCandidate.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "$Label must stay inside the output directory: $resolvedCandidate"
    }
    return $resolvedCandidate
}

if (-not (Test-Path -LiteralPath $BinaryPath -PathType Leaf)) {
    throw "Codex binary does not exist: $BinaryPath"
}
if ($UpstreamTag -cne "rust-v$UpstreamVersion") {
    throw "Upstream tag $UpstreamTag does not match version $UpstreamVersion"
}
if ($DisplayVersion -cne "v$UpstreamVersion-CCU.R$Revision") {
    throw "Display version $DisplayVersion does not match the CCU version contract"
}
$stableReleaseTag = "ccu-rust-v$UpstreamVersion-r$Revision"
$releaseTagPattern = '^' + [regex]::Escape($stableReleaseTag) + '(?:-alpha\.[1-9][0-9]*)?$'
if ($ReleaseTag -cnotmatch $releaseTagPattern) {
    throw "Release tag $ReleaseTag does not match the CCU release contract"
}

$platforms = @{
    'windows-x64' = @{
        Target = 'x86_64-pc-windows-msvc'
        BinaryName = 'codex.exe'
        ManifestName = 'ccu-fork-manifest.json'
    }
    'linux-x64' = @{
        Target = 'x86_64-unknown-linux-musl'
        BinaryName = 'codex'
        ManifestName = 'ccu-fork-manifest-linux-x64.json'
    }
    'linux-arm64' = @{
        Target = 'aarch64-unknown-linux-musl'
        BinaryName = 'codex'
        ManifestName = 'ccu-fork-manifest-linux-arm64.json'
    }
    'macos-x64' = @{
        Target = 'x86_64-apple-darwin'
        BinaryName = 'codex'
        ManifestName = 'ccu-fork-manifest-macos-x64.json'
    }
    'macos-arm64' = @{
        Target = 'aarch64-apple-darwin'
        BinaryName = 'codex'
        ManifestName = 'ccu-fork-manifest-macos-arm64.json'
    }
}
$platform = $platforms[$RuntimeId]
$target = $platform.Target
$binaryName = $platform.BinaryName
$manifestName = $platform.ManifestName
$releaseTag = $ReleaseTag
$assetName = "codex-ccu-i18n-$UpstreamVersion-r$Revision-$target.zip"
$outputRoot = [System.IO.Path]::GetFullPath($OutputDirectory)
$stagingRoot = Assert-ChildPath -Root $outputRoot -Candidate (Join-Path $outputRoot 'staging') -Label 'Release staging directory'
$packageRoot = Join-Path (Join-Path $stagingRoot 'package') 'bin'
$assetPath = Assert-ChildPath -Root $outputRoot -Candidate (Join-Path $outputRoot $assetName) -Label 'Release asset'
$manifestPath = Assert-ChildPath -Root $outputRoot -Candidate (Join-Path $outputRoot $manifestName) -Label 'Release manifest'

New-Item -ItemType Directory -Path $outputRoot -Force | Out-Null
Remove-Item -LiteralPath $stagingRoot -Recurse -Force -ErrorAction SilentlyContinue
foreach ($path in @($assetPath, "$assetPath.sha256", $manifestPath, "$manifestPath.sha256")) {
    Remove-Item -LiteralPath $path -Force -ErrorAction SilentlyContinue
}
New-Item -ItemType Directory -Path $packageRoot -Force | Out-Null
Copy-Item -LiteralPath $BinaryPath -Destination (Join-Path $packageRoot $binaryName)
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
"$manifestHash  $manifestName" | Set-Content -LiteralPath "$manifestPath.sha256" -Encoding ascii
Remove-Item -LiteralPath $stagingRoot -Recurse -Force

$manifest | ConvertTo-Json -Depth 5
