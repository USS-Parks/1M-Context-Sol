[CmdletBinding()]
param(
    [string] $OutputDirectory
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ($PSVersionTable.PSEdition -ne 'Desktop') {
    throw 'Windows release verification must run under Windows PowerShell 5.1.'
}
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
    $repositoryRoot = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
    $OutputDirectory = Join-Path $repositoryRoot 'dist'
}

$buildScript = Join-Path $PSScriptRoot 'build.ps1'
$verificationRoot = Join-Path ([IO.Path]::GetTempPath()) ('1m-context-ticker-release-' + [guid]::NewGuid().ToString('N'))
$firstDirectory = Join-Path $verificationRoot 'first'
$secondDirectory = Join-Path $verificationRoot 'second'
$expectedFiles = @('1M-Context-Ticker-Windows-x64.exe','1M-Context-Ticker-Windows-x64.exe.sha256','artifact-manifest.json')

function Get-ArtifactHashes([string] $Directory) {
    $hashes = [ordered]@{}
    foreach ($name in $expectedFiles) {
        $path = Join-Path $Directory $name
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Release artifact is missing: $path" }
        $hashes[$name] = (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash.ToLowerInvariant()
    }
    $actualFiles = @(Get-ChildItem -LiteralPath $Directory -File | ForEach-Object { $_.Name } | Sort-Object)
    $difference = @(Compare-Object -ReferenceObject ($expectedFiles | Sort-Object) -DifferenceObject $actualFiles)
    if ($difference.Count -ne 0) { throw ('Release directory contains an unexpected file set: ' + ($difference | Out-String)) }
    $hashes
}

New-Item -ItemType Directory -Path $firstDirectory,$secondDirectory -Force | Out-Null
try {
    $first = & $buildScript -OutputDirectory $firstDirectory
    $second = & $buildScript -OutputDirectory $secondDirectory
    $firstHashes = Get-ArtifactHashes $firstDirectory
    $secondHashes = Get-ArtifactHashes $secondDirectory
    foreach ($name in $expectedFiles) {
        if ($firstHashes[$name] -ne $secondHashes[$name]) {
            throw "Source-identical clean builds differ for $name."
        }
    }

    $final = & $buildScript -OutputDirectory $OutputDirectory
    $finalHashes = Get-ArtifactHashes $OutputDirectory
    foreach ($name in $expectedFiles) {
        if ($firstHashes[$name] -ne $finalHashes[$name]) {
            throw "Final release output differs from clean reproducibility proof for $name."
        }
    }

    $manifest = Get-Content -Raw -LiteralPath (Join-Path $OutputDirectory 'artifact-manifest.json') | ConvertFrom-Json
    $checksumLine = (Get-Content -Raw -LiteralPath (Join-Path $OutputDirectory '1M-Context-Ticker-Windows-x64.exe.sha256')).Trim()
    $expectedChecksumLine = "$($manifest.sha256)  1M-Context-Ticker-Windows-x64.exe"
    if ($checksumLine -ne $expectedChecksumLine) { throw 'Checksum file does not match the artifact manifest.' }

    [pscustomobject]@{
        Passed = $true
        ReproducibleBuilds = 2
        Artifact = $final.Executable
        Bytes = $manifest.bytes
        Sha256 = $manifest.sha256
        ManifestSha256 = $finalHashes['artifact-manifest.json']
        ChecksumSha256 = $finalHashes['1M-Context-Ticker-Windows-x64.exe.sha256']
        Architecture = $manifest.architecture
        Version = $manifest.version
        ManagedDependencies = $manifest.managed_dependencies
        SelfTest = $manifest.self_test
        ModuleMvid = $manifest.compiler.deterministic_normalization.module_mvid
    }
}
finally {
    Remove-Item -LiteralPath $verificationRoot -Recurse -Force -ErrorAction SilentlyContinue
}
