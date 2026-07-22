[CmdletBinding()]
param(
    [string] $OutputDirectory
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ($PSVersionTable.PSEdition -ne 'Desktop') {
    throw 'The native release build must run under Windows PowerShell 5.1.'
}
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
    $repositoryRoot = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
    $OutputDirectory = Join-Path $repositoryRoot 'dist'
}

$compiler = 'C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe'
if (-not (Test-Path -LiteralPath $compiler -PathType Leaf)) {
    throw "The in-box .NET Framework 4.8 compiler is missing: $compiler"
}

$fixture = Join-Path (Split-Path $PSScriptRoot -Parent) 'fixtures\behavior-cases.json'
if (-not (Test-Path -LiteralPath $fixture -PathType Leaf)) {
    throw "Shared behavior fixture is missing: $fixture"
}

$artifactName = '1M-Context-Ticker-Windows-x64.exe'
$sourceFileNames = @('Program.cs','State.cs','Native.cs','TickerWindow.cs','SelfTest.cs')
$sourcePaths = @($sourceFileNames | ForEach-Object { Join-Path $PSScriptRoot $_ })
foreach ($sourcePath in $sourcePaths) {
    if (-not (Test-Path -LiteralPath $sourcePath -PathType Leaf)) { throw "Native source file is missing: $sourcePath" }
}

function Get-BytesSha256([byte[]] $Bytes) {
    $hasher = [Security.Cryptography.SHA256]::Create()
    try { ([BitConverter]::ToString($hasher.ComputeHash($Bytes))).Replace('-', '').ToLowerInvariant() }
    finally { $hasher.Dispose() }
}

function Get-SourceSeed([string[]] $Paths) {
    $stream = New-Object IO.MemoryStream
    try {
        foreach ($path in $Paths) {
            $nameBytes = [Text.Encoding]::UTF8.GetBytes([IO.Path]::GetFileName($path))
            $contentBytes = [IO.File]::ReadAllBytes($path)
            $stream.Write($nameBytes, 0, $nameBytes.Length)
            $stream.WriteByte(0)
            $stream.Write($contentBytes, 0, $contentBytes.Length)
            $stream.WriteByte(0)
        }
        $hasher = [Security.Cryptography.SHA256]::Create()
        try { ,$hasher.ComputeHash($stream.ToArray()) }
        finally { $hasher.Dispose() }
    }
    finally { $stream.Dispose() }
}

function Find-ByteSequence([byte[]] $Bytes, [byte[]] $Needle) {
    $locations = New-Object System.Collections.Generic.List[int]
    for ($offset = 0; $offset -le $Bytes.Length - $Needle.Length; $offset++) {
        $matches = $true
        for ($index = 0; $index -lt $Needle.Length; $index++) {
            if ($Bytes[$offset + $index] -ne $Needle[$index]) { $matches = $false; break }
        }
        if ($matches) { $locations.Add($offset) }
    }
    $locations.ToArray()
}

function Normalize-CompilerIdentity([string] $Path, [string[]] $SourceFiles) {
    $bytes = [IO.File]::ReadAllBytes($Path)
    if ($bytes.Length -lt 512 -or $bytes[0] -ne 0x4D -or $bytes[1] -ne 0x5A) { throw 'Compiler output is not a valid PE image.' }
    $peOffset = [BitConverter]::ToInt32($bytes, 0x3C)
    if ($peOffset -lt 0x40 -or $peOffset + 24 -ge $bytes.Length -or [BitConverter]::ToUInt32($bytes, $peOffset) -ne 0x00004550) {
        throw 'Compiler output has an invalid PE header.'
    }
    $machine = [BitConverter]::ToUInt16($bytes, $peOffset + 4)
    if ($machine -ne 0x8664) { throw ('Compiler output machine is 0x{0:X4}, not AMD64.' -f $machine) }

    $ascii = [Text.Encoding]::ASCII.GetString($bytes)
    $matches = [regex]::Matches($ascii, 'PrivateImplementationDetails>\{(?<guid>[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12})\}')
    if ($matches.Count -ne 1) { throw "Expected exactly one compiler-generated identity string; found $($matches.Count)." }

    $sourceSeed = Get-SourceSeed $SourceFiles
    $deterministicGuidBytes = New-Object byte[] 16
    [Array]::Copy($sourceSeed, 0, $deterministicGuidBytes, 0, 16)
    $deterministicGuid = New-Object Guid (,$deterministicGuidBytes)
    $oldGuid = [Guid]::ParseExact($matches[0].Groups['guid'].Value, 'D')
    if ($deterministicGuid -eq $oldGuid) { throw 'Compiler identity unexpectedly already equals the deterministic identity.' }

    $replacementText = $deterministicGuid.ToString('D').ToUpperInvariant()
    $replacementTextBytes = [Text.Encoding]::ASCII.GetBytes($replacementText)
    [Array]::Copy($replacementTextBytes, 0, $bytes, $matches[0].Groups['guid'].Index, $replacementTextBytes.Length)

    $mvidLocations = @(Find-ByteSequence $bytes $oldGuid.ToByteArray())
    if ($mvidLocations.Count -ne 1) { throw "Expected exactly one compiler-generated MVID; found $($mvidLocations.Count)." }
    [Array]::Copy($deterministicGuid.ToByteArray(), 0, $bytes, $mvidLocations[0], 16)

    [Array]::Clear($bytes, $peOffset + 8, 4)
    [IO.File]::WriteAllBytes($Path, $bytes)
    [pscustomobject]@{
        pe_timestamp = 0
        module_mvid = $deterministicGuid.ToString('D')
        source_seed_sha256 = Get-BytesSha256 $sourceSeed
    }
}

function Resolve-GacAssembly([string] $Name, [string] $ArchitectureHint) {
    $matches = @(Get-ChildItem -LiteralPath 'C:\Windows\Microsoft.NET\assembly' -Recurse -File -Filter ($Name + '.dll') -ErrorAction SilentlyContinue)
    if ($ArchitectureHint) {
        $preferred = @($matches | Where-Object { $_.FullName -like ('*\' + $ArchitectureHint + '\*') } | Select-Object -First 1)
        if ($preferred.Count -eq 1) { return $preferred[0].FullName }
    }
    if ($matches.Count -eq 0) { throw "Required in-box assembly is missing: $Name" }
    $matches[0].FullName
}

$references = @(
    (Resolve-GacAssembly 'System.Xaml' 'GAC_MSIL'),
    (Resolve-GacAssembly 'WindowsBase' 'GAC_MSIL'),
    (Resolve-GacAssembly 'PresentationCore' 'GAC_64'),
    (Resolve-GacAssembly 'PresentationFramework' 'GAC_MSIL')
)
foreach ($reference in $references) {
    if (-not (Test-Path -LiteralPath $reference -PathType Leaf)) { throw "Compiler reference is missing: $reference" }
}

New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$executable = Join-Path $OutputDirectory $artifactName
$selfTestOutput = Join-Path $OutputDirectory '.self-test.tmp.json'
$checksumPath = $executable + '.sha256'
$manifestPath = Join-Path $OutputDirectory 'artifact-manifest.json'
$expectedOutputNames = @($artifactName, ($artifactName + '.sha256'), 'artifact-manifest.json')

Remove-Item -LiteralPath $executable,$selfTestOutput,$checksumPath,$manifestPath,(Join-Path $OutputDirectory 'self-test.json') -Force -ErrorAction SilentlyContinue

$arguments = @(
    '/nologo',
    '/target:winexe',
    '/platform:x64',
    '/optimize+',
    '/checked+',
    '/warn:4',
    '/warnaserror+',
    ('/out:' + $executable)
)
$arguments += $references | ForEach-Object { '/reference:' + $_ }
$arguments += $sourcePaths

& $compiler @arguments
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $executable -PathType Leaf)) {
    throw "C# compiler failed with exit code $LASTEXITCODE."
}

$normalization = Normalize-CompilerIdentity $executable $sourcePaths
$selfTestResult = $null
try {
    $selfTest = Start-Process -FilePath $executable -ArgumentList @('--self-test', ('"{0}"' -f $fixture), ('"{0}"' -f $selfTestOutput)) -WindowStyle Hidden -Wait -PassThru
    if ($selfTest.ExitCode -ne 0 -or -not (Test-Path -LiteralPath $selfTestOutput -PathType Leaf)) {
        throw "Native executable self-test failed with exit code $($selfTest.ExitCode)."
    }
    $selfTestResult = Get-Content -Raw -LiteralPath $selfTestOutput | ConvertFrom-Json
    if (-not $selfTestResult.passed) { throw "Native executable self-test reported failure: $($selfTestResult.error)" }
}
finally {
    Remove-Item -LiteralPath $selfTestOutput -Force -ErrorAction SilentlyContinue
}

$version = [Diagnostics.FileVersionInfo]::GetVersionInfo($executable)
if ($version.ProductName -ne '1M Context Ticker' -or $version.FileVersion -ne '0.1.0.0') {
    throw 'Built assembly identity/version does not match 1M Context Ticker 0.1.0.0.'
}
$escapedExecutable = $executable.Replace("'", "''")
$inspectionCommand = '$assembly = [Reflection.Assembly]::ReflectionOnlyLoadFrom(''' + $escapedExecutable + '''); $assembly.GetReferencedAssemblies() | ForEach-Object { $_.Name } | Sort-Object'
$managedDependencies = @(& (Join-Path $PSHOME 'powershell.exe') -NoProfile -Command $inspectionCommand)
if ($LASTEXITCODE -ne 0) { throw "Managed dependency inspection failed with exit code $LASTEXITCODE." }
$expectedDependencies = @('mscorlib','PresentationCore','PresentationFramework','System','System.Core','System.Web.Extensions','WindowsBase')
$dependencyDifference = @(Compare-Object -ReferenceObject $expectedDependencies -DifferenceObject $managedDependencies)
if ($dependencyDifference.Count -ne 0) { throw ('Unexpected managed dependency set: ' + ($dependencyDifference | Out-String)) }

$hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $executable).Hash.ToLowerInvariant()
[IO.File]::WriteAllText($checksumPath, ($hash + '  ' + $artifactName + "`n"), (New-Object Text.UTF8Encoding($false)))

$sourceHashes = [ordered]@{}
foreach ($sourceFileName in $sourceFileNames) {
    $sourceHashes[$sourceFileName] = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $PSScriptRoot $sourceFileName)).Hash.ToLowerInvariant()
}
$selfTestEvidence = [ordered]@{
    passed = $true
    token_cases = [int]$selfTestResult.token_cases
    selection_cases = [int]$selfTestResult.selection_cases
    layout_cases = [int]$selfTestResult.layout_cases
    face_width_cases = [int]$selfTestResult.face_width_cases
    window_guard_cases = [int]$selfTestResult.window_guard_cases
}
$manifest = [ordered]@{
    schema_version = 2
    product = '1M Context Ticker'
    version = $version.FileVersion
    artifact = $artifactName
    architecture = 'amd64'
    target_framework = '.NET Framework 4.8'
    compiler = [ordered]@{
        product_version = [Diagnostics.FileVersionInfo]::GetVersionInfo($compiler).ProductVersion
        deterministic_normalization = $normalization
    }
    bytes = (Get-Item -LiteralPath $executable).Length
    sha256 = $hash
    checksum_file = [IO.Path]::GetFileName($checksumPath)
    shared_fixture_sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $fixture).Hash.ToLowerInvariant()
    self_test = $selfTestEvidence
    managed_dependencies = $managedDependencies
    source_sha256 = $sourceHashes
}
$manifestJson = ($manifest | ConvertTo-Json -Depth 8).Replace("`r`n", "`n")
[IO.File]::WriteAllText($manifestPath, $manifestJson, (New-Object Text.UTF8Encoding($false)))

$actualOutputNames = @(Get-ChildItem -LiteralPath $OutputDirectory -File | ForEach-Object { $_.Name } | Sort-Object)
$outputDifference = @(Compare-Object -ReferenceObject ($expectedOutputNames | Sort-Object) -DifferenceObject $actualOutputNames)
if ($outputDifference.Count -ne 0) { throw ('Release output contains an unexpected file set: ' + ($outputDifference | Out-String)) }

[pscustomobject]@{
    Executable = $executable
    Bytes = $manifest.bytes
    Sha256 = $hash
    Version = $manifest.version
    Architecture = $manifest.architecture
    SelfTest = $selfTestEvidence.passed
    Manifest = $manifestPath
    Checksum = $checksumPath
    ModuleMvid = $normalization.module_mvid
}
