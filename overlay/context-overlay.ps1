[CmdletBinding()]
param(
    [switch] $DryRun,
    [string] $SessionsRoot = (Join-Path $env:USERPROFILE '.codex\sessions'),
    [string] $ThreadId,
    [int] $StaleAfterSeconds = 300,
    [int] $RightOffset = 152,
    [int] $BottomOffset = 104,
    [string] $OutputPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'ContextOverlay.Core.psm1') -Force

if (-not $DryRun) {
    throw 'CDO-01 is a read-only spike. Use -DryRun.'
}

$selection = Select-ContextRollout -SessionsRoot $SessionsRoot -ThreadId $ThreadId
$state = Get-ContextOverlayState -RolloutPath $selection.Path -StaleAfterSeconds $StaleAfterSeconds
$anchor = Get-CodexWindowAnchor -RightOffset $RightOffset -BottomOffset $BottomOffset

$result = [pscustomobject]@{
    Mode             = 'dry-run'
    SessionId        = $state.SessionId
    ShortId          = $state.ShortId
    SourcePath       = $state.SourcePath
    SelectionIsAmbiguous = $selection.Ambiguous
    UsedTokens       = $state.UsedTokens
    ContextWindow    = $state.ContextWindow
    EffectiveWindow  = $state.EffectiveWindow
    RemainingTokens  = $state.RemainingTokens
    PercentRemaining = $state.PercentRemaining
    EventTimestampUtc = $state.EventTimestampUtc
    AgeSeconds       = $state.AgeSeconds
    IsStale          = $state.IsStale
    WasCompacted     = $state.WasCompacted
    Window           = $anchor
} | ConvertTo-Json -Depth 5

if ($OutputPath) {
    [IO.File]::WriteAllText($OutputPath, $result, (New-Object Text.UTF8Encoding($false)))
}
else {
    $result
}
