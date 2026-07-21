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

    Write-Output 'Overlay installer tests passed.'
}
finally {
    Remove-Item -LiteralPath $testRoot -Recurse -Force -ErrorAction SilentlyContinue
}
