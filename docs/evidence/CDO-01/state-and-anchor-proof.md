# CDO-01 state and anchor proof

Date: 2026-07-20

## Result

The read-only spike passed. It selected the current root Codex Desktop rollout without a pinned task ID, parsed the newest host `token_count` event, used active-context rather than cumulative-session usage, and found the real Codex top-level window from the interactive Windows desktop.

No Codex input, window position, installed file, configuration, or repository remote was changed.

## State calculation

The parser follows the official Codex TUI semantics:

- baseline: `12000`
- active context: `last_token_usage.total_tokens`
- effective window: `model_context_window - 12000`
- adjusted used: `max(active context - 12000, 0)`
- remaining: `max(effective window - adjusted used, 0)`
- displayed percentage: nearest whole percentage of effective window remaining

The cumulative `total_token_usage.total_tokens` field is deliberately ignored for the dial.

## Automated check

Command:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\overlay\Test-ContextOverlay.ps1
```

Result: `Context overlay core tests passed.`

The focused check proves:

- active-context selection ignores a deliberately larger cumulative total;
- the 12,000-token baseline and remaining percentage are exact;
- malformed input fails closed;
- stale input is labeled;
- an active-context decrease is labeled as compaction;
- the freshest root task wins;
- close candidates are labeled ambiguous;
- explicit task selection is auditable and non-ambiguous.

## Live automatic-selection dry-run

The helper ran once through a current-user, interactive, hidden PowerShell task because the command runner cannot enumerate the user's interactive window station. The temporary scheduled task was removed immediately after collection.

Observed output:

| Field | Value |
|---|---:|
| Session | `019f7e5d-bbfa-7922-ba42-71b52f309a39` |
| Short ID | `2f309a39` |
| Selection ambiguous | `false` |
| Active context | `60438` |
| Context window | `258400` |
| Effective window | `246400` |
| Remaining | `197962` |
| Percent remaining | `80` |
| Event age | `5` seconds |
| Stale | `false` |
| Window handle | `0x17C053C` |
| Window owner PID | `25612` |
| Window title/class | `ChatGPT` / `Chrome_WidgetWin_1` |
| Window rectangle | `-7,-7,1543,823` |
| Foreground | `true` |
| Computed dial rectangle | left `1319`, top `647`, width `72`, height `72` |

This pre-existing task correctly reports its current `258400` host window. Sol-1M installation and fresh-task proof belong to CDO-03/CDO-04; CDO-01 does not relabel this task as a 1M task.

## Gate decision

CDO-01 passes. The state seam and real-window anchor are proven. The next prompt may build the one visible focusless overlay without adding a new application framework.
