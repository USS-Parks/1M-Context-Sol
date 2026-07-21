[CmdletBinding()]
param(
    [string] $OutputDirectory = (Join-Path (Split-Path $PSScriptRoot -Parent | Split-Path -Parent) 'dist')
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$compiler = 'C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe'
if (-not (Test-Path -LiteralPath $compiler -PathType Leaf)) {
    throw "The in-box .NET Framework 4.8 compiler is missing: $compiler"
}

$fixture = Join-Path (Split-Path $PSScriptRoot -Parent) 'fixtures\behavior-cases.json'
if (-not (Test-Path -LiteralPath $fixture -PathType Leaf)) {
    throw "Shared behavior fixture is missing: $fixture"
}

New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$executable = Join-Path $OutputDirectory '1M-Context-Ticker-Windows-x64.exe'
$selfTestOutput = Join-Path $OutputDirectory 'self-test.json'
$checksumPath = $executable + '.sha256'
$manifestPath = Join-Path $OutputDirectory 'artifact-manifest.json'

Remove-Item -LiteralPath $executable,$selfTestOutput,$checksumPath,$manifestPath -Force -ErrorAction SilentlyContinue

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
$arguments += @(
    (Join-Path $PSScriptRoot 'Program.cs'),
    (Join-Path $PSScriptRoot 'State.cs'),
    (Join-Path $PSScriptRoot 'Native.cs'),
    (Join-Path $PSScriptRoot 'TickerWindow.cs'),
    (Join-Path $PSScriptRoot 'SelfTest.cs')
)

& $compiler @arguments
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $executable -PathType Leaf)) {
    throw "C# compiler failed with exit code $LASTEXITCODE."
}

$selfTest = Start-Process -FilePath $executable -ArgumentList @('--self-test', ('"{0}"' -f $fixture), ('"{0}"' -f $selfTestOutput)) -WindowStyle Hidden -Wait -PassThru
if ($selfTest.ExitCode -ne 0 -or -not (Test-Path -LiteralPath $selfTestOutput -PathType Leaf)) {
    throw "Native executable self-test failed with exit code $($selfTest.ExitCode)."
}
$selfTestResult = Get-Content -Raw -LiteralPath $selfTestOutput | ConvertFrom-Json
if (-not $selfTestResult.passed) { throw "Native executable self-test reported failure: $($selfTestResult.error)" }

$hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $executable).Hash.ToLowerInvariant()
[IO.File]::WriteAllText($checksumPath, ($hash + '  ' + [IO.Path]::GetFileName($executable) + "`n"), (New-Object Text.UTF8Encoding($false)))
$version = [Diagnostics.FileVersionInfo]::GetVersionInfo($executable)
$manifest = [ordered]@{
    schema_version = 1
    product = '1M Context Ticker'
    version = $version.FileVersion
    artifact = [IO.Path]::GetFileName($executable)
    architecture = 'x64'
    target_framework = '.NET Framework 4.8'
    bytes = (Get-Item -LiteralPath $executable).Length
    sha256 = $hash
    self_test = $selfTestResult
    source_files = @('Program.cs','State.cs','Native.cs','TickerWindow.cs','SelfTest.cs')
}
[IO.File]::WriteAllText($manifestPath, ($manifest | ConvertTo-Json -Depth 8), (New-Object Text.UTF8Encoding($false)))

[pscustomobject]@{
    Executable = $executable
    Bytes = $manifest.bytes
    Sha256 = $hash
    Version = $manifest.version
    SelfTest = $selfTestResult.passed
    Manifest = $manifestPath
}
