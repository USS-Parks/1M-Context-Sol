[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('Plan', 'Install', 'Uninstall', 'Start', 'Stop', 'Status')]
    [string] $Action,

    [string] $InstallRoot = (Join-Path $env:LOCALAPPDATA 'CodexContextOverlay'),
    [string] $ConfigPath = (Join-Path $env:USERPROFILE '.codex\config.toml'),
    [string] $SourceRoot = $PSScriptRoot,
    [string] $ShortcutName = 'Codex Context Dial',
    [switch] $SkipShortcut
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$ownedKeys = @(
    'model_context_window',
    'model_auto_compact_token_limit',
    'model_auto_compact_token_limit_scope',
    'model_catalog_json'
)
$requiredSourceFiles = @(
    'context-overlay.ps1',
    'ContextOverlay.Core.psm1',
    'manage-overlay.ps1',
    'sol-1m-models.json',
    'sol-1m-catalog-manifest.json'
)
$manifestPath = Join-Path $InstallRoot 'state\install-manifest.json'
$backupPath = Join-Path $InstallRoot 'state\config.before.toml'
$runtimeStatusPath = Join-Path $InstallRoot 'state\runtime-status.json'
$startupShortcut = Join-Path ([Environment]::GetFolderPath('Startup')) ($ShortcutName + '.lnk')
$startMenuShortcut = Join-Path ([Environment]::GetFolderPath('Programs')) ($ShortcutName + '.lnk')

function Get-Sha256([string] $Path) {
    (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToLowerInvariant()
}

function Get-BytesSha256([byte[]] $Bytes) {
    $hasher = [Security.Cryptography.SHA256]::Create()
    try {
        ([BitConverter]::ToString($hasher.ComputeHash($Bytes))).Replace('-', '').ToLowerInvariant()
    }
    finally {
        $hasher.Dispose()
    }
}

function ConvertFrom-Utf8Bytes([byte[]] $Bytes) {
    $offset = 0
    if ($Bytes.Length -ge 3 -and $Bytes[0] -eq 0xEF -and $Bytes[1] -eq 0xBB -and $Bytes[2] -eq 0xBF) {
        $offset = 3
    }
    $encoding = New-Object Text.UTF8Encoding($false, $true)
    $encoding.GetString($Bytes, $offset, ($Bytes.Length - $offset))
}

function ConvertTo-Utf8Bytes([string] $Text, [bool] $WithBom) {
    $encoding = New-Object Text.UTF8Encoding($WithBom)
    $encoding.GetBytes($Text)
}

function Get-TopLevelPrefix([string] $Text) {
    $match = [regex]::Match($Text, '(?m)^\s*\[')
    if ($match.Success) {
        return [pscustomobject]@{ Prefix = $Text.Substring(0, $match.Index); Suffix = $Text.Substring($match.Index) }
    }
    [pscustomobject]@{ Prefix = $Text; Suffix = '' }
}

function Get-TopLevelValue([string] $Prefix, [string] $Key) {
    $escaped = [regex]::Escape($Key)
    $matches = [regex]::Matches($Prefix, "(?m)^[ \t]*$escaped[ \t]*=[ \t]*(?<value>[^\r\n]+)[ \t]*$")
    if ($matches.Count -gt 1) {
        throw "Duplicate top-level key '$Key' is not safe to manage."
    }
    if ($matches.Count -eq 0) {
        return $null
    }
    $matches[0].Groups['value'].Value.Trim()
}

function Assert-SafePaths {
    if (-not [IO.Path]::IsPathRooted($InstallRoot) -or -not [IO.Path]::IsPathRooted($ConfigPath) -or -not [IO.Path]::IsPathRooted($SourceRoot)) {
        throw 'InstallRoot, ConfigPath, and SourceRoot must be absolute paths.'
    }
    $resolvedInstall = [IO.Path]::GetFullPath($InstallRoot).TrimEnd('\')
    $resolvedConfig = [IO.Path]::GetFullPath($ConfigPath)
    if ($resolvedInstall.Length -lt 12 -or $resolvedInstall -eq [IO.Path]::GetPathRoot($resolvedInstall).TrimEnd('\')) {
        throw 'InstallRoot is too broad for safe management.'
    }
    if ($resolvedConfig.StartsWith($resolvedInstall + '\', [StringComparison]::OrdinalIgnoreCase)) {
        throw 'ConfigPath must not be inside InstallRoot.'
    }
}

function Get-DesiredLines([string] $CatalogPath, [string] $Newline) {
    $escapedPath = $CatalogPath.Replace('\', '\\').Replace('"', '\"')
    @(
        'model_context_window = 1050000',
        'model_auto_compact_token_limit = 900000',
        'model_auto_compact_token_limit_scope = "total"',
        ('model_catalog_json = "{0}"' -f $escapedPath)
    ) -join $Newline
}

function Get-InstallPlan {
    Assert-SafePaths
    if (-not (Test-Path -LiteralPath $ConfigPath -PathType Leaf)) {
        throw "Codex config does not exist: $ConfigPath"
    }
    foreach ($file in $requiredSourceFiles) {
        if (-not (Test-Path -LiteralPath (Join-Path $SourceRoot $file) -PathType Leaf)) {
            throw "Required source file is missing: $file"
        }
    }

    $catalog = Get-Content -Raw -LiteralPath (Join-Path $SourceRoot 'sol-1m-models.json') | ConvertFrom-Json
    if ($catalog.models.Count -ne 1 -or $catalog.models[0].slug -ne 'gpt-5.6-sol') {
        throw 'Catalog must contain only exact gpt-5.6-sol.'
    }
    if ([long]$catalog.models[0].context_window -ne 1050000 -or [long]$catalog.models[0].max_context_window -ne 1050000 -or [long]$catalog.models[0].auto_compact_token_limit -ne 900000) {
        throw 'Catalog does not contain the settled 1,050,000/900,000 policy.'
    }

    $configBytes = [IO.File]::ReadAllBytes($ConfigPath)
    $text = ConvertFrom-Utf8Bytes $configBytes
    $parts = Get-TopLevelPrefix $text
    $model = Get-TopLevelValue $parts.Prefix 'model'
    if ($model -ne '"gpt-5.6-sol"') {
        throw 'The existing user-owned model key must already be exact "gpt-5.6-sol"; the installer will not take ownership of it.'
    }

    $conflicts = New-Object System.Collections.Generic.List[string]
    foreach ($key in $ownedKeys) {
        if ($null -ne (Get-TopLevelValue $parts.Prefix $key)) {
            $conflicts.Add($key)
        }
    }
    if ($conflicts.Count -gt 0) {
        throw ('Owned-key conflict: ' + ($conflicts -join ', '))
    }
    if (Test-Path -LiteralPath $InstallRoot) {
        throw "InstallRoot already exists: $InstallRoot"
    }
    if (-not $SkipShortcut -and ((Test-Path -LiteralPath $startupShortcut) -or (Test-Path -LiteralPath $startMenuShortcut))) {
        throw "Overlay shortcut conflict: $ShortcutName"
    }

    $newline = if ($text.Contains("`r`n")) { "`r`n" } else { "`n" }
    $separator = if ($parts.Prefix.Length -eq 0 -or $parts.Prefix.EndsWith("`n")) { '' } else { $newline }
    $desiredLines = Get-DesiredLines (Join-Path $InstallRoot 'sol-1m-models.json') $newline
    $candidateText = $parts.Prefix + $separator + $desiredLines + $newline + $parts.Suffix
    $hasBom = $configBytes.Length -ge 3 -and $configBytes[0] -eq 0xEF -and $configBytes[1] -eq 0xBB -and $configBytes[2] -eq 0xBF
    $candidateBytes = ConvertTo-Utf8Bytes $candidateText $hasBom

    [pscustomobject]@{
        ConfigBytes       = $configBytes
        ConfigSha256      = Get-BytesSha256 $configBytes
        CandidateBytes    = $candidateBytes
        CandidateSha256   = Get-BytesSha256 $candidateBytes
        CatalogPath       = Join-Path $InstallRoot 'sol-1m-models.json'
        OwnedKeys         = $ownedKeys
    }
}

function Write-BytesAtomically([string] $Path, [byte[]] $Bytes) {
    $temporary = $Path + '.context-overlay.' + $PID + '.tmp'
    try {
        [IO.File]::WriteAllBytes($temporary, $Bytes)
        Move-Item -LiteralPath $temporary -Destination $Path -Force
    }
    finally {
        Remove-Item -LiteralPath $temporary -Force -ErrorAction SilentlyContinue
    }
}

function New-OverlayShortcuts {
    if ($SkipShortcut) { return }
    $scriptPath = Join-Path $InstallRoot 'context-overlay.ps1'
    $arguments = '-NoProfile -STA -WindowStyle Hidden -ExecutionPolicy Bypass -File "{0}" -OutputPath "{1}"' -f $scriptPath, $runtimeStatusPath
    $shell = New-Object -ComObject WScript.Shell
    foreach ($shortcutPath in @($startupShortcut, $startMenuShortcut)) {
        $shortcut = $shell.CreateShortcut($shortcutPath)
        $shortcut.TargetPath = 'C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe'
        $shortcut.Arguments = $arguments
        $shortcut.WorkingDirectory = $InstallRoot
        $shortcut.WindowStyle = 7
        $shortcut.Description = 'Show the context-window dial when Codex Desktop is foreground.'
        $shortcut.Save()
    }
}

function Read-Manifest {
    if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
        throw "Overlay install manifest does not exist: $manifestPath"
    }
    Get-Content -Raw -LiteralPath $manifestPath | ConvertFrom-Json
}

function Assert-ManifestMatches($Manifest) {
    if ($Manifest.schema_version -ne 1 -or $Manifest.install_root -ne $InstallRoot -or $Manifest.config_path -ne $ConfigPath -or $Manifest.startup_shortcut -ne $startupShortcut -or $Manifest.start_menu_shortcut -ne $startMenuShortcut) {
        throw 'Install manifest identity does not match the requested paths/shortcuts.'
    }
    if (-not (Test-Path -LiteralPath $backupPath -PathType Leaf) -or (Get-Sha256 $backupPath) -ne $Manifest.original_config_sha256) {
        throw 'Byte-exact config backup is missing or does not match the install manifest.'
    }
}

function Install-Overlay {
    $plan = Get-InstallPlan
    $installed = $false
    try {
        New-Item -ItemType Directory -Path (Join-Path $InstallRoot 'state') -Force | Out-Null
        foreach ($file in $requiredSourceFiles) {
            Copy-Item -LiteralPath (Join-Path $SourceRoot $file) -Destination (Join-Path $InstallRoot $file)
        }
        [IO.File]::WriteAllBytes($backupPath, $plan.ConfigBytes)
        Write-BytesAtomically $ConfigPath $plan.CandidateBytes

        $fileHashes = [ordered]@{}
        foreach ($file in $requiredSourceFiles) {
            $fileHashes[$file] = Get-Sha256 (Join-Path $InstallRoot $file)
        }
        $manifest = [ordered]@{
            schema_version = 1
            install_root = $InstallRoot
            config_path = $ConfigPath
            startup_shortcut = $startupShortcut
            start_menu_shortcut = $startMenuShortcut
            original_config_sha256 = $plan.ConfigSha256
            installed_config_sha256 = $plan.CandidateSha256
            owned_keys = $ownedKeys
            installed_values = [ordered]@{
                model_context_window = '1050000'
                model_auto_compact_token_limit = '900000'
                model_auto_compact_token_limit_scope = '"total"'
                model_catalog_json = '"' + $plan.CatalogPath.Replace('\', '\\').Replace('"', '\"') + '"'
            }
            file_sha256 = $fileHashes
        }
        $manifestJson = $manifest | ConvertTo-Json -Depth 8
        [IO.File]::WriteAllText($manifestPath, $manifestJson, (New-Object Text.UTF8Encoding($false)))
        New-OverlayShortcuts
        $installed = $true
        [pscustomobject]@{
            action = 'install'
            installed = $true
            install_root = $InstallRoot
            config_sha256 = Get-Sha256 $ConfigPath
            owned_keys = $ownedKeys
            startup_shortcut = if ($SkipShortcut) { 'skipped' } else { $startupShortcut }
            start_menu_shortcut = if ($SkipShortcut) { 'skipped' } else { $startMenuShortcut }
            codex_restart_required = $true
            overlay_launch_required = $true
        }
    }
    finally {
        if (-not $installed) {
            if (-not $SkipShortcut) {
                Remove-Item -LiteralPath $startupShortcut,$startMenuShortcut -Force -ErrorAction SilentlyContinue
            }
            if (Test-Path -LiteralPath $backupPath -PathType Leaf) {
                [IO.File]::WriteAllBytes($ConfigPath, [IO.File]::ReadAllBytes($backupPath))
            }
            if (Test-Path -LiteralPath $InstallRoot -PathType Container) {
                Remove-Item -LiteralPath $InstallRoot -Recurse -Force
            }
        }
    }
}

function Get-ConfigWithoutOwnedKeys($Manifest) {
    $currentBytes = [IO.File]::ReadAllBytes($ConfigPath)
    $currentSha = Get-BytesSha256 $currentBytes
    if ($currentSha -eq $Manifest.installed_config_sha256) {
        return [pscustomobject]@{ Bytes = [IO.File]::ReadAllBytes($backupPath); ExactRestore = $true }
    }

    $text = ConvertFrom-Utf8Bytes $currentBytes
    $parts = Get-TopLevelPrefix $text
    foreach ($key in $ownedKeys) {
        $actual = Get-TopLevelValue $parts.Prefix $key
        $expected = [string]$Manifest.installed_values.$key
        if ($actual -ne $expected) {
            throw "Owned key '$key' changed after install; refusing to remove or overwrite it."
        }
        $escaped = [regex]::Escape($key)
        $parts.Prefix = [regex]::Replace($parts.Prefix, "(?m)^[ \t]*$escaped[ \t]*=[^\r\n]*(?:\r?\n|$)", '', 1)
    }
    $hasBom = $currentBytes.Length -ge 3 -and $currentBytes[0] -eq 0xEF -and $currentBytes[1] -eq 0xBB -and $currentBytes[2] -eq 0xBF
    [pscustomobject]@{ Bytes = ConvertTo-Utf8Bytes ($parts.Prefix + $parts.Suffix) $hasBom; ExactRestore = $false }
}

function Stop-OverlayProcess {
    if (-not (Test-Path -LiteralPath $runtimeStatusPath -PathType Leaf)) { return 'absent' }
    $runtime = Get-Content -Raw -LiteralPath $runtimeStatusPath | ConvertFrom-Json
    if ($null -eq $runtime.ProcessId) { return 'absent' }
    $process = Get-CimInstance Win32_Process -Filter "ProcessId=$($runtime.ProcessId)" -ErrorAction SilentlyContinue
    $scriptPath = Join-Path $InstallRoot 'context-overlay.ps1'
    if ($null -eq $process) { return 'not-running' }
    if ($process.Name -ne 'powershell.exe' -or $process.CommandLine -notlike "*$scriptPath*") {
        throw 'Runtime PID does not belong to the installed overlay; refusing to stop it.'
    }
    Stop-Process -Id ([int]$runtime.ProcessId)
    'stopped'
}

function Uninstall-Overlay {
    Assert-SafePaths
    $manifest = Read-Manifest
    Assert-ManifestMatches $manifest
    $restored = Get-ConfigWithoutOwnedKeys $manifest
    $processState = Stop-OverlayProcess
    Write-BytesAtomically $ConfigPath $restored.Bytes
    if (-not $SkipShortcut) {
        Remove-Item -LiteralPath $startupShortcut,$startMenuShortcut -Force -ErrorAction SilentlyContinue
    }
    $restoredHash = Get-Sha256 $ConfigPath
    $expectedOriginal = [string]$manifest.original_config_sha256
    $exactRestoreVerified = $restored.ExactRestore -and $restoredHash -eq $expectedOriginal
    Remove-Item -LiteralPath $InstallRoot -Recurse -Force
    [pscustomobject]@{
        action = 'uninstall'
        removed = $true
        exact_restore = $exactRestoreVerified
        preserved_later_changes = -not $restored.ExactRestore
        config_sha256 = $restoredHash
        overlay_process_before_removal = $processState
        codex_restart_required = $true
    }
}

function Get-OverlayStatus {
    $manifest = Read-Manifest
    Assert-ManifestMatches $manifest
    $runtime = if (Test-Path -LiteralPath $runtimeStatusPath -PathType Leaf) { Get-Content -Raw -LiteralPath $runtimeStatusPath | ConvertFrom-Json } else { $null }
    [pscustomobject]@{
        installed = $true
        config_owned_snapshot_matches = (Get-Sha256 $ConfigPath) -eq $manifest.installed_config_sha256
        startup_shortcut_exists = if ($SkipShortcut) { $null } else { Test-Path -LiteralPath $startupShortcut }
        start_menu_shortcut_exists = if ($SkipShortcut) { $null } else { Test-Path -LiteralPath $startMenuShortcut }
        runtime = $runtime
    }
}

Assert-SafePaths
switch ($Action) {
    'Plan' {
        $plan = Get-InstallPlan
        [pscustomobject]@{
            action = 'plan'
            config_path = $ConfigPath
            install_root = $InstallRoot
            original_config_sha256 = $plan.ConfigSha256
            candidate_config_sha256 = $plan.CandidateSha256
            owned_keys = $plan.OwnedKeys
            startup_shortcut = if ($SkipShortcut) { 'skipped' } else { $startupShortcut }
            start_menu_shortcut = if ($SkipShortcut) { 'skipped' } else { $startMenuShortcut }
            codex_process_control = 'none'
        }
    }
    'Install' { Install-Overlay }
    'Uninstall' { Uninstall-Overlay }
    'Start' {
        if ($SkipShortcut) { throw 'Start is unavailable with -SkipShortcut.' }
        Read-Manifest | ForEach-Object { Assert-ManifestMatches $_ }
        Start-Process -FilePath $startMenuShortcut -WindowStyle Hidden
        [pscustomobject]@{ action = 'start'; launched = $true }
    }
    'Stop' {
        Read-Manifest | ForEach-Object { Assert-ManifestMatches $_ }
        [pscustomobject]@{ action = 'stop'; overlay_process = Stop-OverlayProcess }
    }
    'Status' { Get-OverlayStatus }
}
