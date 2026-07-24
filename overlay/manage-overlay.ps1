[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('Plan', 'Install', 'Upgrade', 'Rollback', 'Uninstall', 'Start', 'Stop', 'Status')]
    [string] $Action,

    [string] $InstallRoot = (Join-Path $env:LOCALAPPDATA 'CodexContextOverlay'),
    [string] $ConfigPath = (Join-Path $env:USERPROFILE '.codex\config.toml'),
    [string] $SourceRoot,
    [string] $ExecutablePath,
    [string] $ShortcutName = '1M Context Ticker',
    [switch] $SkipShortcut
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ([string]::IsNullOrWhiteSpace($SourceRoot)) {
    $SourceRoot = $PSScriptRoot
}
if ([string]::IsNullOrWhiteSpace($ExecutablePath)) {
    $ExecutablePath = Join-Path (Split-Path $PSScriptRoot -Parent) 'dist\1M-Context-Ticker-Windows-x64.exe'
}

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
$referenceManifestPath = Join-Path $InstallRoot 'state\install-manifest.before-native.json'
$referenceRoot = Join-Path $InstallRoot 'state\powershell-reference'
$backupPath = Join-Path $InstallRoot 'state\config.before.toml'
$runtimeStatusPath = Join-Path $InstallRoot 'state\runtime-status.json'
$nativeRuntimeStatusPath = Join-Path $InstallRoot 'state\native-runtime-status.json'
$nativeExecutableName = '1M-Context-Ticker-Windows-x64.exe'
$installedExecutablePath = Join-Path $InstallRoot $nativeExecutableName
$startupShortcut = Join-Path ([Environment]::GetFolderPath('Startup')) ($ShortcutName + '.lnk')
$startMenuShortcut = Join-Path ([Environment]::GetFolderPath('Programs')) ($ShortcutName + '.lnk')
$legacyStartupShortcut = Join-Path ([Environment]::GetFolderPath('Startup')) 'Codex Context Dial.lnk'
$legacyStartMenuShortcut = Join-Path ([Environment]::GetFolderPath('Programs')) 'Codex Context Dial.lnk'

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
    $matches = [regex]::Matches($Prefix, "(?m)^[ \t]*$escaped[ \t]*=[ \t]*(?<value>[^\r\n]*?[^ \t\r\n])[ \t]*\r?$")
    if ($matches.Count -gt 1) {
        throw "Duplicate top-level key '$Key' is not safe to manage."
    }
    if ($matches.Count -eq 0) {
        return $null
    }
    $matches[0].Groups['value'].Value.Trim()
}

function Assert-SafePaths {
    if (-not [IO.Path]::IsPathRooted($InstallRoot) -or -not [IO.Path]::IsPathRooted($ConfigPath) -or -not [IO.Path]::IsPathRooted($SourceRoot) -or -not [IO.Path]::IsPathRooted($ExecutablePath)) {
        throw 'InstallRoot, ConfigPath, SourceRoot, and ExecutablePath must be absolute paths.'
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

function Assert-NativeExecutable([string] $Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Native ticker executable is missing: $Path"
    }
    $version = [Diagnostics.FileVersionInfo]::GetVersionInfo($Path)
    if ($version.ProductName -ne '1M Context Ticker' -or $version.FileVersion -ne '0.1.0.0') {
        throw 'Native ticker executable identity/version does not match 1M Context Ticker 0.1.0.0.'
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
    Assert-NativeExecutable $ExecutablePath
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

function New-TickerShortcuts([string] $RuntimeKind, [string] $StartupPath = $startupShortcut, [string] $StartMenuPath = $startMenuShortcut) {
    if ($SkipShortcut) { return }
    if ($RuntimeKind -eq 'native-executable') {
        $target = $installedExecutablePath
        $arguments = '--status-path "{0}"' -f $nativeRuntimeStatusPath
    }
    elseif ($RuntimeKind -eq 'powershell-reference') {
        $target = 'C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe'
        $scriptPath = Join-Path $InstallRoot 'context-overlay.ps1'
        $arguments = '-NoProfile -STA -WindowStyle Hidden -ExecutionPolicy Bypass -File "{0}" -OutputPath "{1}"' -f $scriptPath, $runtimeStatusPath
    }
    else {
        throw "Unsupported ticker runtime kind: $RuntimeKind"
    }
    $shell = New-Object -ComObject WScript.Shell
    foreach ($shortcutPath in @($StartupPath, $StartMenuPath)) {
        $shortcut = $shell.CreateShortcut($shortcutPath)
        $shortcut.TargetPath = $target
        $shortcut.Arguments = $arguments
        $shortcut.WorkingDirectory = $InstallRoot
        $shortcut.WindowStyle = 7
        $shortcut.Description = 'Show 1M Context Ticker when Codex Desktop is foreground.'
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
    $currentShortcuts = $Manifest.startup_shortcut -eq $startupShortcut -and $Manifest.start_menu_shortcut -eq $startMenuShortcut
    $legacyShortcuts = $Manifest.schema_version -eq 1 -and $Manifest.startup_shortcut -eq $legacyStartupShortcut -and $Manifest.start_menu_shortcut -eq $legacyStartMenuShortcut
    if ($Manifest.schema_version -notin @(1, 2) -or $Manifest.install_root -ne $InstallRoot -or $Manifest.config_path -ne $ConfigPath -or (-not $currentShortcuts -and -not $legacyShortcuts)) {
        throw 'Install manifest identity does not match the requested paths/shortcuts.'
    }
    if (-not (Test-Path -LiteralPath $backupPath -PathType Leaf) -or (Get-Sha256 $backupPath) -ne $Manifest.original_config_sha256) {
        throw 'Byte-exact config backup is missing or does not match the install manifest.'
    }
    if ($Manifest.schema_version -eq 2) {
        if ($Manifest.runtime_kind -ne 'native-executable' -or $Manifest.executable_path -ne $installedExecutablePath -or -not (Test-Path -LiteralPath $installedExecutablePath -PathType Leaf) -or (Get-Sha256 $installedExecutablePath) -ne $Manifest.executable_sha256) {
            throw 'Installed native executable is missing or does not match the install manifest.'
        }
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
        Copy-Item -LiteralPath $ExecutablePath -Destination $installedExecutablePath
        [IO.File]::WriteAllBytes($backupPath, $plan.ConfigBytes)
        Write-BytesAtomically $ConfigPath $plan.CandidateBytes

        $fileHashes = [ordered]@{}
        foreach ($file in $requiredSourceFiles) {
            $fileHashes[$file] = Get-Sha256 (Join-Path $InstallRoot $file)
        }
        $fileHashes[$nativeExecutableName] = Get-Sha256 $installedExecutablePath
        $manifest = [ordered]@{
            schema_version = 2
            install_root = $InstallRoot
            config_path = $ConfigPath
            startup_shortcut = $startupShortcut
            start_menu_shortcut = $startMenuShortcut
            runtime_kind = 'native-executable'
            executable_path = $installedExecutablePath
            executable_sha256 = $fileHashes[$nativeExecutableName]
            powershell_reference = 'bundled'
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
        New-TickerShortcuts 'native-executable'
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
            runtime_kind = 'native-executable'
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

function Test-ConfigOwnedValues($Manifest) {
    try {
        $text = ConvertFrom-Utf8Bytes ([IO.File]::ReadAllBytes($ConfigPath))
        $prefix = (Get-TopLevelPrefix $text).Prefix
        foreach ($key in $ownedKeys) {
            if ((Get-TopLevelValue $prefix $key) -ne [string]$Manifest.installed_values.$key) {
                return $false
            }
        }
        return $true
    }
    catch {
        return $false
    }
}

function Get-RuntimeKind($Manifest) {
    if ($Manifest.schema_version -eq 2 -and $Manifest.runtime_kind -eq 'native-executable') { return 'native-executable' }
    'powershell-reference'
}

function Get-TickerProcess($Manifest) {
    $runtimeKind = Get-RuntimeKind $Manifest
    $statusPath = if ($runtimeKind -eq 'native-executable') { $nativeRuntimeStatusPath } else { $runtimeStatusPath }
    if (-not (Test-Path -LiteralPath $statusPath -PathType Leaf)) {
        return [pscustomobject]@{ State = 'absent'; RuntimeKind = $runtimeKind; Runtime = $null; Process = $null }
    }
    $runtime = Get-Content -Raw -LiteralPath $statusPath | ConvertFrom-Json
    $processId = if ($runtimeKind -eq 'native-executable') { $runtime.process_id } else { $runtime.ProcessId }
    if ($null -eq $processId) {
        return [pscustomobject]@{ State = 'absent'; RuntimeKind = $runtimeKind; Runtime = $runtime; Process = $null }
    }
    $process = Get-CimInstance Win32_Process -Filter "ProcessId=$processId" -ErrorAction SilentlyContinue
    if ($null -eq $process) {
        return [pscustomobject]@{ State = 'not-running'; RuntimeKind = $runtimeKind; Runtime = $runtime; Process = $null }
    }
    if ($runtimeKind -eq 'native-executable') {
        $actualPath = [IO.Path]::GetFullPath([string]$process.ExecutablePath)
        $expectedPath = [IO.Path]::GetFullPath($installedExecutablePath)
        if (-not $actualPath.Equals($expectedPath, [StringComparison]::OrdinalIgnoreCase)) {
            throw 'Runtime PID does not belong to the installed native ticker; refusing process control.'
        }
    }
    else {
        $scriptPath = Join-Path $InstallRoot 'context-overlay.ps1'
        if ($process.Name -ne 'powershell.exe' -or $process.CommandLine -notlike "*$scriptPath*") {
            throw 'Runtime PID does not belong to the installed PowerShell reference; refusing process control.'
        }
    }
    [pscustomobject]@{ State = 'running'; RuntimeKind = $runtimeKind; Runtime = $runtime; Process = $process }
}

function Stop-TickerProcess($Manifest) {
    $ticker = Get-TickerProcess $Manifest
    if ($ticker.State -ne 'running') { return $ticker.State }
    $processId = [int]$ticker.Process.ProcessId
    Stop-Process -Id $processId -Force
    $process = Get-Process -Id $processId -ErrorAction SilentlyContinue
    if ($process -and -not $process.WaitForExit(10000)) {
        throw "Ticker process $processId did not terminate; refusing to replace a running executable."
    }
    'stopped'
}

function Upgrade-Overlay {
    Assert-SafePaths
    Assert-NativeExecutable $ExecutablePath
    $manifest = Read-Manifest
    Assert-ManifestMatches $manifest
    if ($manifest.schema_version -eq 2 -and $manifest.runtime_kind -eq 'native-executable') {
        $manifestBytes = [IO.File]::ReadAllBytes($manifestPath)
        $backupExecutable = $installedExecutablePath + '.before-upgrade'
        $upgraded = $false
        try {
            Copy-Item -LiteralPath $installedExecutablePath -Destination $backupExecutable -Force
            $processState = Stop-TickerProcess $manifest
            Copy-Item -LiteralPath $ExecutablePath -Destination $installedExecutablePath -Force
            $executableHash = Get-Sha256 $installedExecutablePath
            $manifest.executable_sha256 = $executableHash
            $manifest.file_sha256.PSObject.Properties[$nativeExecutableName].Value = $executableHash
            $manifestJson = $manifest | ConvertTo-Json -Depth 8
            Write-BytesAtomically $manifestPath ((New-Object Text.UTF8Encoding($false)).GetBytes($manifestJson))
            New-TickerShortcuts 'native-executable' $manifest.startup_shortcut $manifest.start_menu_shortcut
            $upgraded = $true
            [pscustomobject]@{
                action = 'upgrade'
                upgraded = $true
                prior_process = $processState
                runtime_kind = 'native-executable'
                executable_path = $installedExecutablePath
                executable_sha256 = $executableHash
                ticker_launch_required = $true
                codex_process_control = 'none'
            }
            return
        }
        finally {
            if (-not $upgraded) {
                if (Test-Path -LiteralPath $backupExecutable -PathType Leaf) {
                    Copy-Item -LiteralPath $backupExecutable -Destination $installedExecutablePath -Force
                }
                Write-BytesAtomically $manifestPath $manifestBytes
                New-TickerShortcuts 'native-executable' $manifest.startup_shortcut $manifest.start_menu_shortcut
            }
            Remove-Item -LiteralPath $backupExecutable -Force -ErrorAction SilentlyContinue
        }
    }
    if ($manifest.schema_version -ne 1) { throw 'Upgrade requires the installed PowerShell reference manifest (schema 1).' }
    foreach ($file in $requiredSourceFiles) {
        if (-not (Test-Path -LiteralPath (Join-Path $InstallRoot $file) -PathType Leaf)) {
            throw "Installed PowerShell reference file is missing: $file"
        }
        if (-not (Test-Path -LiteralPath (Join-Path $SourceRoot $file) -PathType Leaf)) {
            throw "Required source file is missing: $file"
        }
    }

    $manifestBytes = [IO.File]::ReadAllBytes($manifestPath)
    $referenceManifest = ConvertFrom-Utf8Bytes $manifestBytes | ConvertFrom-Json
    $upgraded = $false
    try {
        New-Item -ItemType Directory -Path $referenceRoot -Force | Out-Null
        foreach ($file in $requiredSourceFiles) {
            Copy-Item -LiteralPath (Join-Path $InstallRoot $file) -Destination (Join-Path $referenceRoot $file) -Force
        }
        [IO.File]::WriteAllBytes($referenceManifestPath, $manifestBytes)
        $processState = Stop-TickerProcess $manifest
        foreach ($file in $requiredSourceFiles) {
            Copy-Item -LiteralPath (Join-Path $SourceRoot $file) -Destination (Join-Path $InstallRoot $file) -Force
        }
        Copy-Item -LiteralPath $ExecutablePath -Destination $installedExecutablePath -Force

        $fileHashes = [ordered]@{}
        foreach ($file in $requiredSourceFiles) { $fileHashes[$file] = Get-Sha256 (Join-Path $InstallRoot $file) }
        $fileHashes[$nativeExecutableName] = Get-Sha256 $installedExecutablePath
        $manifest.schema_version = 2
        $manifest | Add-Member -NotePropertyName runtime_kind -NotePropertyValue 'native-executable'
        $manifest | Add-Member -NotePropertyName executable_path -NotePropertyValue $installedExecutablePath
        $manifest | Add-Member -NotePropertyName executable_sha256 -NotePropertyValue $fileHashes[$nativeExecutableName]
        $manifest | Add-Member -NotePropertyName powershell_reference -NotePropertyValue 'retained'
        $manifest.file_sha256 = [pscustomobject]$fileHashes
        $manifest.startup_shortcut = $startupShortcut
        $manifest.start_menu_shortcut = $startMenuShortcut
        New-TickerShortcuts 'native-executable' $startupShortcut $startMenuShortcut
        if (-not $SkipShortcut) {
            if ($referenceManifest.startup_shortcut -ne $startupShortcut) { Remove-Item -LiteralPath $referenceManifest.startup_shortcut -Force -ErrorAction SilentlyContinue }
            if ($referenceManifest.start_menu_shortcut -ne $startMenuShortcut) { Remove-Item -LiteralPath $referenceManifest.start_menu_shortcut -Force -ErrorAction SilentlyContinue }
        }
        [IO.File]::WriteAllText($manifestPath, ($manifest | ConvertTo-Json -Depth 8), (New-Object Text.UTF8Encoding($false)))
        $upgraded = $true
        [pscustomobject]@{
            action = 'upgrade'
            upgraded = $true
            prior_process = $processState
            runtime_kind = 'native-executable'
            executable_path = $installedExecutablePath
            executable_sha256 = $fileHashes[$nativeExecutableName]
            powershell_reference = $referenceRoot
            ticker_launch_required = $true
            codex_process_control = 'none'
        }
    }
    finally {
        if (-not $upgraded) {
            foreach ($file in $requiredSourceFiles) {
                $saved = Join-Path $referenceRoot $file
                if (Test-Path -LiteralPath $saved -PathType Leaf) { Copy-Item -LiteralPath $saved -Destination (Join-Path $InstallRoot $file) -Force }
            }
            Remove-Item -LiteralPath $installedExecutablePath,$nativeRuntimeStatusPath -Force -ErrorAction SilentlyContinue
            Write-BytesAtomically $manifestPath $manifestBytes
            New-TickerShortcuts 'powershell-reference' $referenceManifest.startup_shortcut $referenceManifest.start_menu_shortcut
            if (-not $SkipShortcut) {
                if ($referenceManifest.startup_shortcut -ne $startupShortcut) { Remove-Item -LiteralPath $startupShortcut -Force -ErrorAction SilentlyContinue }
                if ($referenceManifest.start_menu_shortcut -ne $startMenuShortcut) { Remove-Item -LiteralPath $startMenuShortcut -Force -ErrorAction SilentlyContinue }
            }
            Remove-Item -LiteralPath $referenceRoot -Recurse -Force -ErrorAction SilentlyContinue
            Remove-Item -LiteralPath $referenceManifestPath -Force -ErrorAction SilentlyContinue
        }
    }
}

function Rollback-Overlay {
    Assert-SafePaths
    $manifest = Read-Manifest
    Assert-ManifestMatches $manifest
    if ($manifest.schema_version -ne 2 -or $manifest.runtime_kind -ne 'native-executable') {
        throw 'Rollback requires an upgraded native ticker installation.'
    }
    if (-not (Test-Path -LiteralPath $referenceManifestPath -PathType Leaf) -or -not (Test-Path -LiteralPath $referenceRoot -PathType Container)) {
        throw 'PowerShell reference rollback state is missing.'
    }
    $referenceManifestBytes = [IO.File]::ReadAllBytes($referenceManifestPath)
    $referenceManifest = ConvertFrom-Utf8Bytes $referenceManifestBytes | ConvertFrom-Json
    Assert-ManifestMatches $referenceManifest
    if ($referenceManifest.schema_version -ne 1) { throw 'PowerShell reference manifest is not schema 1.' }

    $processState = Stop-TickerProcess $manifest
    foreach ($file in $requiredSourceFiles) {
        $saved = Join-Path $referenceRoot $file
        if (-not (Test-Path -LiteralPath $saved -PathType Leaf)) { throw "PowerShell reference rollback file is missing: $file" }
        if ((Get-Sha256 $saved) -ne $referenceManifest.file_sha256.$file) { throw "PowerShell reference rollback hash does not match: $file" }
        Copy-Item -LiteralPath $saved -Destination (Join-Path $InstallRoot $file) -Force
    }
    New-TickerShortcuts 'powershell-reference' $referenceManifest.startup_shortcut $referenceManifest.start_menu_shortcut
    if (-not $SkipShortcut) {
        if ($referenceManifest.startup_shortcut -ne $manifest.startup_shortcut) { Remove-Item -LiteralPath $manifest.startup_shortcut -Force -ErrorAction SilentlyContinue }
        if ($referenceManifest.start_menu_shortcut -ne $manifest.start_menu_shortcut) { Remove-Item -LiteralPath $manifest.start_menu_shortcut -Force -ErrorAction SilentlyContinue }
    }
    Remove-Item -LiteralPath $installedExecutablePath,$nativeRuntimeStatusPath -Force -ErrorAction SilentlyContinue
    Write-BytesAtomically $manifestPath $referenceManifestBytes
    Remove-Item -LiteralPath $referenceRoot -Recurse -Force
    Remove-Item -LiteralPath $referenceManifestPath -Force
    [pscustomobject]@{
        action = 'rollback'
        rolled_back = $true
        prior_process = $processState
        runtime_kind = 'powershell-reference'
        ticker_launch_required = $true
        codex_process_control = 'none'
    }
}

function Uninstall-Overlay {
    Assert-SafePaths
    $manifest = Read-Manifest
    Assert-ManifestMatches $manifest
    $restored = Get-ConfigWithoutOwnedKeys $manifest
    $processState = Stop-TickerProcess $manifest
    Write-BytesAtomically $ConfigPath $restored.Bytes
    if (-not $SkipShortcut) {
        Remove-Item -LiteralPath $manifest.startup_shortcut,$manifest.start_menu_shortcut -Force -ErrorAction SilentlyContinue
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
        ticker_process_before_removal = $processState
        codex_restart_required = $true
    }
}

function Get-OverlayStatus {
    $manifest = Read-Manifest
    Assert-ManifestMatches $manifest
    $ticker = Get-TickerProcess $manifest
    $oneMVerified = $null
    $displayState = $ticker.State
    if ($ticker.RuntimeKind -eq 'native-executable' -and $null -ne $ticker.Runtime) {
        $windowProperty = $ticker.Runtime.PSObject.Properties['context_window']
        $staleProperty = $ticker.Runtime.PSObject.Properties['is_stale']
        $errorProperty = $ticker.Runtime.PSObject.Properties['error']
        $visibleProperty = $ticker.Runtime.PSObject.Properties['visible']
        $foregroundProperty = $ticker.Runtime.PSObject.Properties['codex_foreground']
        $oneMVerified = $ticker.State -eq 'running' -and
            $null -ne $windowProperty -and $null -ne $windowProperty.Value -and [long]$windowProperty.Value -ge 1000000L -and
            ($null -eq $staleProperty -or -not [bool]$staleProperty.Value) -and
            ($null -eq $errorProperty -or [string]::IsNullOrWhiteSpace([string]$errorProperty.Value))
        if ($null -ne $visibleProperty -and [bool]$visibleProperty.Value) { $displayState = 'visible-in-codex' }
        elseif ($null -ne $foregroundProperty -and -not [bool]$foregroundProperty.Value) { $displayState = 'hidden-outside-foreground-codex' }
        elseif (-not $oneMVerified) { $displayState = 'error-or-unverified' }
        else { $displayState = 'hidden' }
    }
    [pscustomobject]@{
        installed = $true
        runtime_kind = Get-RuntimeKind $manifest
        minimum_one_m_window = 1000000L
        one_m_context_verified = $oneMVerified
        display_state = $displayState
        config_snapshot_matches = (Get-Sha256 $ConfigPath) -eq $manifest.installed_config_sha256
        config_owned_values_match = Test-ConfigOwnedValues $manifest
        startup_shortcut_exists = if ($SkipShortcut) { $null } else { Test-Path -LiteralPath $manifest.startup_shortcut }
        start_menu_shortcut_exists = if ($SkipShortcut) { $null } else { Test-Path -LiteralPath $manifest.start_menu_shortcut }
        process_state = $ticker.State
        process_identity = if ($null -eq $ticker.Process) { $null } else { [pscustomobject]@{ process_id = $ticker.Process.ProcessId; name = $ticker.Process.Name; executable_path = $ticker.Process.ExecutablePath } }
        runtime = $ticker.Runtime
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
    'Upgrade' { Upgrade-Overlay }
    'Rollback' { Rollback-Overlay }
    'Uninstall' { Uninstall-Overlay }
    'Start' {
        if ($SkipShortcut) { throw 'Start is unavailable with -SkipShortcut.' }
        $manifest = Read-Manifest
        Assert-ManifestMatches $manifest
        Start-Process -FilePath $manifest.start_menu_shortcut -WindowStyle Hidden
        [pscustomobject]@{ action = 'start'; launched = $true; runtime_kind = Get-RuntimeKind $manifest }
    }
    'Stop' {
        $manifest = Read-Manifest
        Assert-ManifestMatches $manifest
        [pscustomobject]@{ action = 'stop'; ticker_process = Stop-TickerProcess $manifest; runtime_kind = Get-RuntimeKind $manifest }
    }
    'Status' { Get-OverlayStatus }
}
