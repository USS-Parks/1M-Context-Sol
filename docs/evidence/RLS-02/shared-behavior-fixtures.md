# RLS-02 shared behavior fixtures

Date: 2026-07-20

## Result

The accepted PowerShell reference now consumes one platform-neutral JSON contract at `ticker/fixtures/behavior-cases.json`.

The contract freezes:

- active `last_token_usage.total_tokens` versus deliberately different cumulative usage;
- 12,000-token baseline handling;
- remaining tokens and rounded remaining percentage;
- exhausted-window clamping;
- stale-state labeling;
- compaction detection from an active-count decrease;
- malformed input failure;
- bounded seek-from-end parsing;
- freshest root-task selection and ambiguity;
- automatic subagent exclusion;
- explicit task pinning;
- composer centers for sidebar-open/sidebar-closed and scaled/offset windows.

## Verification

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\overlay\Test-ContextOverlay.ps1
```

Result: `Context overlay shared-fixture tests passed.`

The native executable must load the same fixture in its self-test before RLS-03 can pass.
