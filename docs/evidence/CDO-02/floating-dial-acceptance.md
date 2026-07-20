# CDO-02 floating dial acceptance

Date: 2026-07-20

## Result

The focusless WPF/Win32 context dial passed its implementation gate in the real interactive Codex Desktop session.

The product surface is one 72-by-72 device-independent-pixel circular window. At the machine's 125 percent scale, Windows reported the expected 90-by-90 device-pixel region. It has no replacement chat surface, dashboard, terminal, or TUI.

## Final live state

| Field | Observed value |
|---|---:|
| Overlay process | `44916` |
| Overlay handle | `0x2B30E16` |
| Native rectangle | `1705,853,1795,943` |
| WPF anchor | `1364,682.4` DIPs |
| Visible | `true` |
| Tool-window style | `true` |
| No-activate style | `true` |
| Elliptical corner passes through | `true` |
| Center belongs to dial | `true` |
| Selected task | `019f7e5d-bbfa-7922-ba42-71b52f309a39` |
| Active context | `98982` |
| Host context window | `258400` |
| Remaining | `159418` |
| Remaining percent | `65` |
| Stale | `false` |
| Selection ambiguous | `false` |
| Displayed compaction threshold | `900000` |
| Runtime error | empty |

This task predates the Sol-1M installed configuration, so the dial truthfully displayed its actual 258,400-token host window. CDO-03/CDO-04 own the installed 1M configuration and fresh-task proof.

## Behavior evidence

- The helper stayed `Running` under Task Scheduler while its WPF dispatcher was active.
- A focus-loss observation reported `Visible = false`, `CodexForeground = false`, and the helper remained running; the dial returned on the next clean Codex-foreground launch.
- Win32 `WS_EX_NOACTIVATE` and `WS_EX_TOOLWINDOW` were present.
- `WindowFromPoint` at the elliptical window's corner resolved through the overlay; the center resolved to the dial.
- The live counter refreshed from `91306` to `91373` active tokens without a restart.
- The 125-percent DPI mismatch found during testing was corrected with WPF's device-to-DIP transform.
- Right-click closes the dial; the scheduled test process was also stopped cleanly.
- All temporary CDO-02 scheduled tasks and generated probe files were removed.

## Verification

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\overlay\Test-ContextOverlay.ps1
```

Result: `Context overlay core tests passed.`

PowerShell parser validation of `overlay/context-overlay.ps1` also passed with zero syntax errors.

## Gate decision

AO-G1 and AO-G2 pass for the overlay implementation. CDO-03 may package the same files and exact behavior with bounded configuration ownership, startup, and rollback.
