Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'ContextOverlay.Core.psm1') -Force

function Assert-Equal {
    param($Expected, $Actual, [string] $Label)
    if ($Expected -ne $Actual) {
        throw "$Label expected '$Expected' but got '$Actual'."
    }
}

$fixturePath = Join-Path (Split-Path $PSScriptRoot -Parent) 'ticker\fixtures\behavior-cases.json'
$fixtures = Get-Content -Raw -LiteralPath $fixturePath | ConvertFrom-Json
Assert-Equal 1 $fixtures.schema_version 'fixture schema version'
Assert-Equal 12000 $fixtures.baseline_tokens 'fixture baseline'

foreach ($case in $fixtures.token_cases) {
    $event = [ordered]@{
        timestamp = $case.timestamp
        type = 'event_msg'
        payload = [ordered]@{
            type = 'token_count'
            info = [ordered]@{
                total_token_usage = [ordered]@{ total_tokens = [long]$case.cumulative_total_tokens }
                last_token_usage = [ordered]@{ total_tokens = [long]$case.active_total_tokens }
                model_context_window = [long]$case.context_window
            }
        }
    } | ConvertTo-Json -Compress -Depth 8
    $parameters = @{
        Lines = @($event, 'malformed-json')
        Now = [datetime]$case.now
        StaleAfterSeconds = [int]$case.stale_after_seconds
    }
    if ($null -ne $case.previous_used_tokens) {
        $parameters.PreviousUsedTokens = [long]$case.previous_used_tokens
    }
    $state = ConvertFrom-ContextTokenEvent @parameters
    Assert-Equal ([long]$case.expected.used_tokens) $state.UsedTokens "$($case.id) used"
    Assert-Equal ([long]$case.expected.effective_window) $state.EffectiveWindow "$($case.id) effective"
    Assert-Equal ([long]$case.expected.remaining_tokens) $state.RemainingTokens "$($case.id) remaining"
    Assert-Equal ([int]$case.expected.percent_remaining) $state.PercentRemaining "$($case.id) percent"
    Assert-Equal ([bool]$case.expected.is_stale) $state.IsStale "$($case.id) stale"
    Assert-Equal ([bool]$case.expected.was_compacted) $state.WasCompacted "$($case.id) compacted"
}

$invalidFailed = $false
try { ConvertFrom-ContextTokenEvent -Lines @('not-json') | Out-Null } catch { $invalidFailed = $true }
Assert-Equal $true $invalidFailed 'malformed input fails closed'

$wrongWindowFailed = $false
$wrongWindowEvent = '{"timestamp":"2026-07-20T12:00:00Z","type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"total_tokens":112000},"model_context_window":258400}}}'
try { ConvertFrom-ContextTokenEvent -Lines @($wrongWindowEvent) -Now ([datetime]'2026-07-20T12:00:01Z') | Out-Null } catch { $wrongWindowFailed = $true }
Assert-Equal $true $wrongWindowFailed 'non-1M host window fails closed'

$tailFixture = Join-Path ([IO.Path]::GetTempPath()) ('context-overlay-tail-' + [guid]::NewGuid().ToString('N') + '.jsonl')
try {
    $largePrefix = ('x' * 300000) + "`n"
    $tailEvent = '{"timestamp":"2026-07-20T12:00:00Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"total_tokens":999999},"last_token_usage":{"total_tokens":112000},"model_context_window":1008000}}}'
    [IO.File]::WriteAllText($tailFixture, ($largePrefix + $tailEvent + "`n"), (New-Object Text.UTF8Encoding($false)))
    $boundedState = ConvertFrom-ContextTokenEvent -Lines @(Read-ContextRolloutTail -Path $tailFixture -MaximumBytes 262144) -Now ([datetime]'2026-07-20T12:00:30Z')
    Assert-Equal 112000 $boundedState.UsedTokens 'bounded seek-from-end parser'
}
finally { Remove-Item -LiteralPath $tailFixture -Force -ErrorAction SilentlyContinue }

$testRoot = Join-Path ([IO.Path]::GetTempPath()) ('context-overlay-test-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $testRoot | Out-Null
try {
    $baseTime = [datetime]'2026-07-20T12:00:00Z'
    foreach ($case in $fixtures.selection_cases) {
        $caseRoot = Join-Path $testRoot $case.id
        New-Item -ItemType Directory -Path $caseRoot | Out-Null
        foreach ($candidate in $case.candidates) {
            $path = Join-Path $caseRoot "rollout-$($candidate.session_id).jsonl"
            $meta = [ordered]@{ type = 'session_meta'; payload = [ordered]@{ id = $candidate.session_id; originator = 'Codex Desktop'; thread_source = $candidate.thread_source } } | ConvertTo-Json -Compress -Depth 5
            [IO.File]::WriteAllText($path, $meta, (New-Object Text.UTF8Encoding($false)))
            (Get-Item -LiteralPath $path).LastWriteTimeUtc = $baseTime.AddSeconds([int]$candidate.last_write_offset_seconds)
        }
        $selectionParams = @{ SessionsRoot = $caseRoot }
        if ($null -ne $case.explicit_thread_id) { $selectionParams.ThreadId = [string]$case.explicit_thread_id }
        $selection = Select-ContextRollout @selectionParams
        Assert-Equal ([string]$case.expected_session_id) $selection.SessionId "$($case.id) selected id"
        Assert-Equal ([bool]$case.expected_ambiguous) $selection.Ambiguous "$($case.id) ambiguous"
    }
}
finally { Remove-Item -LiteralPath $testRoot -Recurse -Force }

foreach ($case in $fixtures.layout_cases) {
    $center = Get-CodexComposerCenter -WindowLeft ([int]$case.window_left) -WindowRight ([int]$case.window_right) -SidebarOpen ([bool]$case.sidebar_open)
    Assert-Equal ([int]$case.expected_center) $center "$($case.id) center"
}

Write-Output 'Context overlay shared-fixture tests passed.'
