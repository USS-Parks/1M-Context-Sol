<h1 align="center">1M Context Ticker</h1>

<p align="center">
  A lightweight Windows companion for Codex Desktop that shows the exact active-context count inside the composer and enables GPT-5.6 Sol's 1M catalog policy.
</p>

<p align="center">
  <strong>Windows preview</strong> · Native executable in progress · macOS parked
</p>

## What it does

- Displays the live host-authored count as `Context: <tokens> / 1M`.
- Follows the freshest active root Codex Desktop task.
- Centers itself in the composer region and responds when the right sidebar opens or closes.
- Samples the Codex prompt color for a subdued light/dark appearance.
- Hides when Codex is absent, minimized, or not foreground.
- Uses one focusless, non-activating, single-instance overlay.
- Applies only the supported Sol catalog/window/compaction settings with reversible ownership.

1M Context Ticker never proxies prompts, replaces the Codex interface, blocks normal compaction, or stops/restarts Codex.

## Placement

The ticker sits on the Codex prompt pill's lower control row, centered between the approval and model controls. Its protected window is sized to the rendered text rather than a large fixed badge, so it stays visually quiet and avoids the surrounding controls.

## Capacity terms

| Term | Value | Meaning |
|---|---:|---|
| Catalog total | 1,050,000 | GPT-5.6 Sol model-catalog context window |
| Effective Codex budget | 1,008,000 | Host-reported task budget after Codex's 96% allowance |
| Ticker value | Live | `last_token_usage.total_tokens` for the active context |
| Automatic compaction | 900,000 | Normal Codex compaction threshold, scope `total` |
| Maximum input | 922,000 | Model input limit, distinct from total context |
| Maximum output | 128,000 | Model output limit |

The face deliberately says `/ 1M` for readability while the tooltip and verification records retain the exact dimensions.

## Install the current Windows preview

The accepted reference currently uses Windows PowerShell/WPF. A native `.exe` is being produced under the [Windows release addendum](PLANNING/1M-CONTEXT-TICKER-DESKTOP-RELEASE-ADDENDUM.md).

From Windows PowerShell in this repository:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\overlay\manage-overlay.ps1 -Action Plan
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\overlay\manage-overlay.ps1 -Action Install
```

Installation:

- copies the ticker under `%LOCALAPPDATA%\CodexContextOverlay`;
- creates ordinary Start Menu and Startup shortcuts;
- adds exactly four owned Codex configuration keys;
- preserves a byte-exact pre-install backup;
- does not stop, restart, or otherwise control Codex.

Quit and reopen Codex normally when convenient, then open **Codex Context Dial** from the Start menu once. Future Windows sign-ins launch the helper automatically; it remains hidden until Codex is foreground.

## Status, stop, and uninstall

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\overlay\manage-overlay.ps1 -Action Status
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\overlay\manage-overlay.ps1 -Action Stop
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\overlay\manage-overlay.ps1 -Action Uninstall
```

Uninstall stops only a verified ticker process, removes its two shortcuts and install directory, and restores owned configuration. If unrelated user/app settings changed after installation, those later changes are preserved.

## Verification

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\overlay\Test-ContextOverlay.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\overlay\Test-OverlayInstaller.ps1
```

Detailed local evidence is in:

- [Windows release verification](docs/VERIFICATION.md)
- [Desktop/overlay development log](docs/sessions/CODEX-DESKTOP-SOL-1M-DEVLOG.md)
- [Windows executable release log](docs/sessions/1M-CONTEXT-TICKER-RELEASE-DEVLOG.md)
- [Safe install and rollback evidence](docs/evidence/CDO-03/safe-install-and-rollback.md)

## Privacy and safety

- Reads local Codex rollout metadata and `token_count` events.
- Does not display or transmit transcript content.
- Sends no model requests and uses no alternate API lane.
- Does not modify the signed Codex package.
- Does not terminate Codex or its app-server.
- Uses no scheduled task, MCP server, browser service, or replacement chat client.

## Current limitations

- Windows only.
- The visible surface is a separate focusless overlay, not a native Codex prompt-pill component.
- The installed reference currently depends on Windows PowerShell; the native executable release is tracked by `1MCT-R1`.
- Public signing, GitHub Release publication, imagery, and macOS packaging are not part of the current release scope.
- Configuration and host-budget proof do not substitute for an unrun paid request above the previously tested live boundary.

## Historical implementation

Earlier Context Continuum/TUI, MCP, reservoir, and strict-compaction work remains in repository history and preserved source, but it is not the active product or installation path.

## License

Apache-2.0. See [LICENSE](LICENSE).
