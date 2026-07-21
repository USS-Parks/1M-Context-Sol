# RLS-04 native executable lifecycle

Date: 2026-07-21

## Result

W-G3 passed. The user-local lifecycle now installs, upgrades, starts, stops, inspects, rolls back, and uninstalls the native `1M-Context-Ticker-Windows-x64.exe` without controlling Codex Desktop or its app-server.

The installed final state is the native executable. The accepted PowerShell implementation and its schema-1 manifest are retained under `state\powershell-reference` and `state\install-manifest.before-native.json` for exact rollback.

## Implementation

- `overlay/manage-overlay.ps1` now supports `Upgrade` and `Rollback` in addition to clean native install, start, stop, status, and uninstall.
- Runtime process control is fail-closed: a PowerShell PID must identify the installed reference script, and a native PID must resolve to the exact installed executable path.
- Native executable identity, version, and SHA-256 are verified before installation and through the schema-2 install manifest.
- Upgrade snapshots the reference files and exact manifest before changing runtime files or shortcuts. Failure restores the reference transactionally.
- Rollback verifies every saved reference-file hash before restoring it.
- Startup and Start Menu shortcuts migrate from the legacy `Codex Context Dial` name to official `1M Context Ticker` shortcuts and target the installed executable directly.
- Clean install and uninstall retain the existing four-key configuration ownership model, exact backup, conflict refusal, exact-byte restore, and later-edit preservation.

## Ticker-shape correction

The user's report that the ticker was pinched was traced to `CreateEllipticRgn` clipping a wide ticker window as an ellipse. Both the native runtime and retained reference now use a rounded-rectangle window region. The face has uniform 5-pixel padding around the entire readout and a matching pill radius.

The final live native window measured 198 by 30 device pixels, displayed the active task without error, remained `WS_EX_NOACTIVATE`, and used the sampled dark prompt background `#2D2D2D`.

## Automated verification

- Native build and shared self-test passed with in-box .NET Framework 4.8 compiler.
- `overlay/Test-ContextOverlay.ps1` passed the shared behavior fixture.
- `overlay/Test-OverlayInstaller.ps1` passed clean native install, status, exact config restore, later-edit preservation, owned-key conflict refusal, schema-1 upgrade, retained-reference verification, exact-manifest rollback, and post-rollback uninstall.
- Windows PowerShell parser reported no errors for `manage-overlay.ps1`.
- `git diff --check` passed.

The repository-wide Rust baseline cannot currently pass without changing the explicitly excluded compaction-guard tree:

- `cargo fmt --all -- --check` reports formatting only in preserved `src/precompact_guard.rs`.
- `cargo clippy --all-targets -- -D warnings` stops on the same preserved file at lines 399 and 432 because `format!("{digest:x}")` formats the hasher rather than a finalized digest.
- `cargo test --all-targets` is consequently not an RLS-04 result.

All three protected Rust file hashes still match the CDS-00 baseline below. RLS-04 does not modify, format, stage, or absorb that separate tree.

Tested local executable:

- size: 37,376 bytes
- version: `0.1.0.0`
- SHA-256: `76733b8a1c600dd9ed90485152c4cee21df492c8acf8c8b6141cfc0e86475834`

RLS-05 still owns reproducible artifact normalization, final checksum/manifest production, and the Windows CI job.

## Real lifecycle proof

The real installed reference was upgraded, started, inspected, rolled back, restarted as the PowerShell reference, upgraded again, and left running as the native executable.

- A leftover RLS-03 preview process at `C:\tmp\1mct-r1-build\1M-Context-Ticker-Windows-x64.exe` was verified by exact executable path and stopped because it held the product mutex.
- First installed native PID: `22848`.
- Rollback stopped only that verified ticker process and restored the schema-1 reference manifest and PowerShell shortcut.
- Reference PID after rollback: `7356`.
- A later duplicate native launch preserved the same installed PID and exactly one installed native process.
- Native `Stop` reached `not-running`; native `Start` returned to a healthy live state.
- Final installed native PID at evidence capture: `46360`.
- Final active task: `019f840f-d35e-7993-a11a-c6c68917c7c0`.
- Final host state at capture: 116,708 used, 891,292 remaining, effective window 1,008,000, fresh and unambiguous.
- Installed and source manager SHA-256 both equal `5455c5afcceb9260bbbb8e747a9da326ea4abf1b876e181614fdcb40149e34be`.
- Installed and tested executable SHA-256 both equal `76733b8a1c600dd9ed90485152c4cee21df492c8acf8c8b6141cfc0e86475834`.
- Current configuration SHA-256 remained `0372166b62a83e9f038d0c1a53c037c387a3df6cf7624e38e1b44db9f1793eb3` across live rollback.
- The manifest's original installed-config snapshot no longer equals the whole current config because Codex/user-owned values changed later. The lifecycle preserved those later changes and verified the four owned policy values independently.
- Codex process IDs were identical before and after every final lifecycle sequence; no Codex process was stopped, started, or restarted.

Final shortcuts:

- `%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\1M Context Ticker.lnk`
- `%APPDATA%\Microsoft\Windows\Start Menu\Programs\1M Context Ticker.lnk`
- target: `%LOCALAPPDATA%\CodexContextOverlay\1M-Context-Ticker-Windows-x64.exe`
- legacy shortcut names absent

## Preservation

The unrelated pre-existing dirty files remain byte-identical to the CDS-00 baseline:

- `src/lib.rs`: `47a0c5211ab3c57aac7cda7ae753ec9bb0f61564b0a9e381f0802a07769aa7e1`
- `src/main.rs`: `8e115b381cb09f8e39bbc1fe626f31f34f2bd4e7b790abc1d2ed342af7da9d0b`
- `src/precompact_guard.rs`: `48a4926c8047712669f89de7cb49366e11250db34701aefc622358003a81adc3`

One canonical worktree remains. No dependency tree, worktree, scheduled task, Codex binary change, push, tag, or public release was created by RLS-04.
