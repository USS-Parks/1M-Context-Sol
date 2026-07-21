Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'ContextOverlay.Core.psm1') -Force

function Assert-Equal {
    param($Expected, $Actual, [string] $Label)
    if ($Expected -ne $Actual) {
        throw "$Label expected '$Expected' but got '$Actual'."
    }
}

$now = [datetime]'2026-07-20T12:00:30Z'
$lines = @(
    '{"timestamp":"2026-07-20T12:00:00Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"total_tokens":999999},"last_token_usage":{"total_tokens":112000},"model_context_window":1008000}}}',
    'malformed-json'
)
$state = ConvertFrom-ContextTokenEvent -Lines $lines -Now $now -StaleAfterSeconds 60
Assert-Equal 112000 $state.UsedTokens 'active context uses last_token_usage'
Assert-Equal 996000 $state.EffectiveWindow 'effective window subtracts baseline'
Assert-Equal 896000 $state.RemainingTokens 'remaining tokens'
Assert-Equal 90 $state.PercentRemaining 'rounded remaining percent'
Assert-Equal $false $state.IsStale 'fresh event'

$stale = ConvertFrom-ContextTokenEvent -Lines $lines -Now ([datetime]'2026-07-20T12:02:00Z') -StaleAfterSeconds 60 -PreviousUsedTokens 120000
Assert-Equal $true $stale.IsStale 'stale event'
Assert-Equal $true $stale.WasCompacted 'used-token decrease marks compaction'

$invalidFailed = $false
try {
    ConvertFrom-ContextTokenEvent -Lines @('not-json') | Out-Null
}
catch {
    $invalidFailed = $true
}
Assert-Equal $true $invalidFailed 'malformed input fails closed'

$tailFixture = Join-Path ([IO.Path]::GetTempPath()) ('context-overlay-tail-' + [guid]::NewGuid().ToString('N') + '.jsonl')
try {
    $largePrefix = ('x' * 300000) + "`n"
    $tailEvent = '{"timestamp":"2026-07-20T12:00:00Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"total_tokens":999999},"last_token_usage":{"total_tokens":112000},"model_context_window":1008000}}}'
    [IO.File]::WriteAllText($tailFixture, ($largePrefix + $tailEvent + "`n"), (New-Object Text.UTF8Encoding($false)))
    $boundedTail = @(Read-ContextRolloutTail -Path $tailFixture -MaximumBytes 262144)
    $boundedState = ConvertFrom-ContextTokenEvent -Lines $boundedTail -Now $now
    Assert-Equal 112000 $boundedState.UsedTokens 'bounded seek-from-end parser'
}
finally {
    Remove-Item -LiteralPath $tailFixture -Force -ErrorAction SilentlyContinue
}

$testRoot = Join-Path ([IO.Path]::GetTempPath()) ('context-overlay-test-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $testRoot | Out-Null
try {
    $olderId = '00000000-0000-0000-0000-000000000001'
    $newerId = '00000000-0000-0000-0000-000000000002'
    $olderPath = Join-Path $testRoot "rollout-$olderId.jsonl"
    $newerPath = Join-Path $testRoot "rollout-$newerId.jsonl"
    '{"type":"session_meta","payload":{"id":"' + $olderId + '","originator":"Codex Desktop","thread_source":"user"}}' | Set-Content -LiteralPath $olderPath -Encoding UTF8
    '{"type":"session_meta","payload":{"id":"' + $newerId + '","originator":"Codex Desktop","thread_source":"user"}}' | Set-Content -LiteralPath $newerPath -Encoding UTF8
    (Get-Item -LiteralPath $olderPath).LastWriteTimeUtc = [datetime]'2026-07-20T12:00:00Z'
    (Get-Item -LiteralPath $newerPath).LastWriteTimeUtc = [datetime]'2026-07-20T12:00:10Z'

    $selection = Select-ContextRollout -SessionsRoot $testRoot
    Assert-Equal $newerId $selection.SessionId 'freshest root task wins'
    Assert-Equal $true $selection.Ambiguous 'close candidates are labeled ambiguous'

    $pinned = Select-ContextRollout -SessionsRoot $testRoot -ThreadId $olderId
    Assert-Equal $olderId $pinned.SessionId 'explicit task selection'
    Assert-Equal $false $pinned.Ambiguous 'explicit selection is not ambiguous'
}
finally {
    Remove-Item -LiteralPath $testRoot -Recurse -Force
}

Write-Output 'Context overlay core tests passed.'
