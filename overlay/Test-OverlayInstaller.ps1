Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Assert-True([bool] $Condition, [string] $Label) {
    if (-not $Condition) { throw "Assertion failed: $Label" }
}

$manager = Join-Path $PSScriptRoot 'manage-overlay.ps1'
$testRoot = Join-Path ([IO.Path]::GetTempPath()) ('context-overlay-installer-' + [guid]::NewGuid().ToString('N'))
$config = Join-Path $testRoot 'config.toml'
$install = Join-Path $testRoot 'install'
$original = @"
model = "gpt-5.6-sol"
approval_policy = "on-request"

[features]
apps = true
"@

New-Item -ItemType Directory -Path $testRoot | Out-Null
try {
    [IO.File]::WriteAllText($config, $original, (New-Object Text.UTF8Encoding($false)))
    $originalHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $config).Hash

    & $manager -Action Plan -InstallRoot $install -ConfigPath $config -SourceRoot $PSScriptRoot -SkipShortcut | Out-Null
    & $manager -Action Install -InstallRoot $install -ConfigPath $config -SourceRoot $PSScriptRoot -SkipShortcut | Out-Null
    $installedText = Get-Content -Raw -LiteralPath $config
    foreach ($key in 'model_context_window','model_auto_compact_token_limit','model_auto_compact_token_limit_scope','model_catalog_json') {
        Assert-True ($installedText -match "(?m)^$key\s*=") "installed $key"
    }
    Assert-True ($installedText -match '(?m)^model\s*=\s*"gpt-5.6-sol"$') 'model remains user-owned and unchanged'
    Assert-True (Test-Path -LiteralPath (Join-Path $install 'state\config.before.toml')) 'byte backup exists'
    Assert-True (Test-Path -LiteralPath (Join-Path $install '1M-Context-Ticker-Windows-x64.exe')) 'native executable is installed'
    $freshManifest = Get-Content -Raw -LiteralPath (Join-Path $install 'state\install-manifest.json') | ConvertFrom-Json
    Assert-True ($freshManifest.schema_version -eq 2) 'fresh install uses native manifest schema'
    Assert-True ($freshManifest.runtime_kind -eq 'native-executable') 'fresh install selects native runtime'
    $freshStatus = & $manager -Action Status -InstallRoot $install -ConfigPath $config -SourceRoot $PSScriptRoot -SkipShortcut
    Assert-True ($freshStatus.runtime_kind -eq 'native-executable') 'status reports native runtime'

    & $manager -Action Uninstall -InstallRoot $install -ConfigPath $config -SourceRoot $PSScriptRoot -SkipShortcut | Out-Null
    Assert-True ((Get-FileHash -Algorithm SHA256 -LiteralPath $config).Hash -eq $originalHash) 'unchanged uninstall restores exact bytes'
    Assert-True (-not (Test-Path -LiteralPath $install)) 'uninstall removes install root'

    & $manager -Action Install -InstallRoot $install -ConfigPath $config -SourceRoot $PSScriptRoot -SkipShortcut | Out-Null
    $later = (Get-Content -Raw -LiteralPath $config).Replace('approval_policy = "on-request"', 'approval_policy = "never"')
    [IO.File]::WriteAllText($config, $later, (New-Object Text.UTF8Encoding($false)))
    & $manager -Action Uninstall -InstallRoot $install -ConfigPath $config -SourceRoot $PSScriptRoot -SkipShortcut | Out-Null
    $restoredLater = Get-Content -Raw -LiteralPath $config
    Assert-True ($restoredLater -match 'approval_policy = "never"') 'later unrelated edit is preserved'
    Assert-True ($restoredLater -notmatch '(?m)^model_context_window\s*=') 'owned keys are removed after later edit'

    $conflictConfig = Join-Path $testRoot 'conflict.toml'
    $conflictText = $original.Replace('[features]', "model_context_window = 777777`n`n[features]")
    [IO.File]::WriteAllText($conflictConfig, $conflictText, (New-Object Text.UTF8Encoding($false)))
    $conflictFailed = $false
    try {
        & $manager -Action Install -InstallRoot (Join-Path $testRoot 'conflict-install') -ConfigPath $conflictConfig -SourceRoot $PSScriptRoot -SkipShortcut | Out-Null
    }
    catch {
        $conflictFailed = $_.Exception.Message -match 'Owned-key conflict'
    }
    Assert-True $conflictFailed 'pre-existing owned key is refused'

    [IO.File]::WriteAllText($config, $original, (New-Object Text.UTF8Encoding($false)))
    & $manager -Action Install -InstallRoot $install -ConfigPath $config -SourceRoot $PSScriptRoot -SkipShortcut | Out-Null
    $legacyManifestPath = Join-Path $install 'state\install-manifest.json'
    $legacyManifest = Get-Content -Raw -LiteralPath $legacyManifestPath | ConvertFrom-Json
    $legacyManifest.schema_version = 1
    foreach ($property in 'runtime_kind','executable_path','executable_sha256','powershell_reference') {
        $legacyManifest.PSObject.Properties.Remove($property)
    }
    $legacyManifest.file_sha256.PSObject.Properties.Remove('1M-Context-Ticker-Windows-x64.exe')
    [IO.File]::WriteAllText($legacyManifestPath, ($legacyManifest | ConvertTo-Json -Depth 8), (New-Object Text.UTF8Encoding($false)))
    Remove-Item -LiteralPath (Join-Path $install '1M-Context-Ticker-Windows-x64.exe') -Force
    $legacyManifestHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $legacyManifestPath).Hash

    & $manager -Action Upgrade -InstallRoot $install -ConfigPath $config -SourceRoot $PSScriptRoot -SkipShortcut | Out-Null
    $upgradedManifest = Get-Content -Raw -LiteralPath $legacyManifestPath | ConvertFrom-Json
    Assert-True ($upgradedManifest.schema_version -eq 2) 'upgrade selects schema 2'
    Assert-True ($upgradedManifest.runtime_kind -eq 'native-executable') 'upgrade selects native runtime'
    Assert-True (Test-Path -LiteralPath (Join-Path $install 'state\powershell-reference\context-overlay.ps1')) 'upgrade retains PowerShell reference'
    Assert-True (Test-Path -LiteralPath (Join-Path $install 'state\install-manifest.before-native.json')) 'upgrade retains reference manifest'

    & $manager -Action Rollback -InstallRoot $install -ConfigPath $config -SourceRoot $PSScriptRoot -SkipShortcut | Out-Null
    Assert-True ((Get-FileHash -Algorithm SHA256 -LiteralPath $legacyManifestPath).Hash -eq $legacyManifestHash) 'rollback restores exact reference manifest'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $install '1M-Context-Ticker-Windows-x64.exe'))) 'rollback removes native runtime'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $install 'state\powershell-reference'))) 'rollback retires temporary reference snapshot'

    & $manager -Action Uninstall -InstallRoot $install -ConfigPath $config -SourceRoot $PSScriptRoot -SkipShortcut | Out-Null
    Assert-True ((Get-FileHash -Algorithm SHA256 -LiteralPath $config).Hash -eq $originalHash) 'reference uninstall restores exact bytes after rollback'

    Write-Output 'Ticker lifecycle tests passed.'
}
finally {
    Remove-Item -LiteralPath $testRoot -Recurse -Force -ErrorAction SilentlyContinue
}
