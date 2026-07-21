# CDO-03 safe install and rollback

Date: 2026-07-20

## Result

The revised user-local installer passed and is active. It does not stop, restart, or launch Codex Desktop or its app-server. The user performed the Codex restart normally.

## Installed surface

- Install root: `C:\Users\17076\AppData\Local\CodexContextOverlay`
- Startup shortcut: `Codex Context Dial.lnk`
- Start Menu shortcut: `Codex Context Dial.lnk`
- Scheduled task: absent
- Catalog SHA-256: `aa0044cb49656689bffdf55c0334826118a9afcb5cee29fabebe3e0b356b6707`
- Model: exact `gpt-5.6-sol`
- Catalog total window: `1050000`
- Expected effective host window: `1008000`
- Automatic-compaction threshold: `900000`, scope `total`

The installer owns only:

1. `model_context_window`
2. `model_auto_compact_token_limit`
3. `model_auto_compact_token_limit_scope`
4. `model_catalog_json`

The pre-existing `model = "gpt-5.6-sol"` line remains user-owned.

## Safety evidence

- Original config hash: `90329a7d57f48c5472e5357fad5906c37eee5869c0faa72db0aea4e6d8d1c80f`.
- First installed snapshot hash: `e8fe487b212c4b121c3fb3a1d5aa2a5576745cac8136162a8dca9bcdfb853835`.
- The real uninstall restored the original hash exactly, removed both shortcuts and the install root, and left zero owned keys.
- The packaged app-server PID remained `38484` across plan, install, uninstall, and final reinstall. It changed only after the user later restarted Codex normally.
- A final reinstall left the product active.
- The isolated installer suite also proved later unrelated config edits are preserved and a pre-existing owned key is refused.
- Source inspection found no scheduled-task or app-server-control path in the accepted installer. `Stop-Process` is limited to a PID whose command line exactly identifies the installed overlay script.

## Post-reopen defect and correction

The first user screenshot exposed two implementation defects: duplicate overlay processes and synchronous `Get-Content -Tail` work on the WPF UI thread. Two stale helpers were confirmed. The correction:

- adds a named per-session mutex;
- replaces whole-file tail behavior with bounded seek-from-end reads at 256 KiB, 1 MiB, 4 MiB, and 16 MiB;
- calibrates the default anchor to the observed prompt pill (`RightOffset = 560`, `BottomOffset = 140`);
- stops only verified overlay PowerShell processes during hot replacement.

Live parser timing after the fix was 218 ms cold and 6-21 ms warm. A duplicate shortcut launch left exactly one helper process.

Final live state after the user's normal restart:

| Field | Value |
|---|---:|
| Overlay process count | `1` |
| Active task | `019f7e5d-bbfa-7922-ba42-71b52f309a39` |
| Host context window | `1008000` |
| Active context ticker | `Context: 216062 / 1M` at final placement proof |
| Diagnostic used fraction | `20` percent |
| Host remaining | `800674` tokens / `80` percent |
| Native overlay rectangle | `819,961,1057,996` |
| Sampled prompt background | `#2D2D2D`, dark |
| Status refresh | advanced across a three-second sample |
| Selection stale/ambiguous | `false` / `false` |

The final user-directed face is a 190-by-28-DIP regular-weight ticker displaying exact `Context: used tokens / 1M`. It contains no circle, progress bar, or border/frame and sits on the same baseline as the prompt pill's approval and model controls. Remaining capacity and the 900,000-token compaction threshold are tooltip details rather than face text. The background is sampled from the live Codex prompt pill and the muted foreground is derived from that sample, so light and dark Codex themes are handled without a separate hard-coded overlay theme.

## Verification commands

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\overlay\Test-ContextOverlay.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\overlay\Test-OverlayInstaller.ps1
```

Both focused suites passed. The installed state remains active; user visual confirmation of the final compact ticker placement is the remaining CDO-04 acceptance item.
