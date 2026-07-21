# 1M Context Ticker Windows Release Development Log

This append-only ledger records execution of `1MCT-R1`. Full STS execution for RLS-00 through RLS-06 was approved by the user on 2026-07-20. Push, public release, signing, DMG/macOS, and imagery remain outside authorization.

## RLS-00 - Approve Windows release authority

- **Date:** 2026-07-20
- **Status:** Complete.
- **Authorization:** User said `Commence STS` after narrowing scope to a Windows `.exe` and repository welcome-page copy.
- **Canonical repository:** `C:\Users\17076\Documents\Codex 1M Context Project`, branch `main`, one registered worktree.
- **Baseline HEAD:** `66664d025a542ddd7982ba65c790a93c1929247b` with 17 unpublished commits.
- **Installed reference:** One live PowerShell/WPF ticker, exact host window `1008000`, no runtime error, installed size 95,979 bytes.
- **Compiler:** In-box .NET Framework 4.8 C# compiler `4.8.9221.0`; no .NET SDK dependency.
- **Preserved dirty work:** `src/lib.rs`, `src/main.rs`, and `src/precompact_guard.rs` remain excluded with their established hashes.
- **Scope:** Windows x64 executable, safe executable lifecycle, reproducible local artifact/checksum/manifest, Windows CI job, and README text rebrand.
- **Parked:** DMG/macOS, imagery/logo, signing, push, tag, GitHub Release, external communication, legacy cleanup, and large-token probes.
- **Implementation commit:** `d0d9cacb504f946f6537c22e8c7c89ae2200b4aa`
- **Remote SHA:** Not published; push authorization has not been granted.

## RLS-03 - Build the native Windows executable

- **Date:** 2026-07-20
- **Status:** Complete; native self-test and interactive parity passed.
- **Source:** `ticker/windows/Program.cs`, `State.cs`, `Native.cs`, `TickerWindow.cs`, `SelfTest.cs`, and `build.ps1`.
- **Build:** In-box .NET Framework 4.8 C# compiler, x64 WPF `winexe`, warnings-as-errors, no third-party runtime dependency.
- **Artifact proof:** 37,376 bytes, version `0.1.0.0`, AMD64; first hash `f3e3697611b95b92c2244bf965609ecbfa48b64e54e273bfdd971b8101c8f733`, clean canonical rebuild hash `7be18efe2266d4e779808f52ea8056cefc3588c14cf13cb15a7f2870ec2fc2b9`.
- **Open artifact gate:** Source-identical hashes differ; deterministic/normalized output remains required by RLS-05 and is not claimed at RLS-03.
- **Fixture proof:** Native executable consumed the same 5 token, 3 selection, and 4 layout cases as the PowerShell reference.
- **Live proof:** Exact current task/window, `190x18` footprint, theme/sidebar center, focusless styles, heartbeat, duplicate rejection, no error, and no PowerShell child process passed.
- **Rollback boundary:** Installed PowerShell reference files and shortcuts remain intact pending RLS-04 lifecycle migration.
- **Evidence:** `docs/evidence/RLS-03/native-executable-parity.md`.
- **Implementation commit:** `c0389222d9e95d6a8a82607d56160d9de165392f`
- **Remote SHA:** Not published; push authorization has not been granted.

## RLS-02 - Freeze executable parity fixtures

- **Date:** 2026-07-20
- **Status:** Complete for the accepted reference; executable consumption is an RLS-03 gate.
- **Files:** `ticker/fixtures/behavior-cases.json`, `overlay/ContextOverlay.Core.psm1`, `overlay/Test-ContextOverlay.ps1`, and `docs/evidence/RLS-02/shared-behavior-fixtures.md`.
- **Coverage:** Active-vs-cumulative usage, baseline, exhaustion, stale, compaction, malformed input, bounded tails, task selection, subagent exclusion, pinning, and sidebar-aware layout centers.
- **Verification:** `overlay/Test-ContextOverlay.ps1` passed all shared cases; `git diff --check` passed.
- **Implementation commit:** `368fcbba1a91006c37ab1b1b6da4ef60525e452c`
- **Remote SHA:** Not published; push authorization has not been granted.

## RLS-01 - Replace repository welcome-page copy

- **Date:** 2026-07-20
- **Status:** Complete.
- **Scope:** Replace the stale Context Continuum/TUI welcome page with official 1M Context Ticker copy; imagery remains parked.
- **Copy:** Centered name/value statement, functions, placement, exact capacity terms, current preview install, status/stop/uninstall, verification, privacy/safety, limitations, and historical note.
- **Claim boundary:** Windows preview only; native executable is explicitly in progress until RLS-03/RLS-05; macOS, imagery, signing, push, and public release remain parked.
- **Verification:** All repository-relative README links resolve; install/status/uninstall commands name existing scripts/actions; stale active-product TUI/MCP claims are absent; `git diff --check` passed.
- **Implementation commit:** `2a7211911dee95d500524f5429dd196b01a8813d`
- **Remote SHA:** Not published; push authorization has not been granted.

## RLS-04 - Integrate safe executable lifecycle

- **Date:** 2026-07-21
- **Status:** Complete; W-G3 passed.
- **Files:** Native-aware `overlay/manage-overlay.ps1`, lifecycle tests, rounded-pill correction in both runtimes, and `docs/evidence/RLS-04/native-executable-lifecycle.md`.
- **Lifecycle:** Clean native install, schema-1 PowerShell upgrade, fail-closed process inspection, start, duplicate rejection, stop, status, exact rollback, uninstall, and later-config-edit preservation are implemented and tested.
- **Real upgrade:** The running PowerShell reference was upgraded, the installed native executable was started and inspected, rollback restored the reference exactly, and a final upgrade left the native executable active.
- **Official identity:** Startup and Start Menu shortcuts migrated from legacy `Codex Context Dial` names to `1M Context Ticker` and now target the installed executable directly.
- **User correction:** The pinched elliptical clip was replaced by a true rounded-rectangle pill with uniform 5-pixel padding. Final live geometry was `198x30`, with no activation, fresh unambiguous host state, and no runtime error.
- **Installed proof:** Final installed executable/source hash `76733b8a1c600dd9ed90485152c4cee21df492c8acf8c8b6141cfc0e86475834`; installed and source manager hash `5455c5afcceb9260bbbb8e747a9da326ea4abf1b876e181614fdcb40149e34be`; exactly one installed native process; Codex process IDs unchanged.
- **Verification:** Native build/self-test passed; shared fixture suite passed; lifecycle suite passed; Windows PowerShell parser passed; `git diff --check` passed; live upgrade/rollback/duplicate/stop/start/shortcut/status checks passed.
- **Preserved-tree blocker:** Repository-wide Rust format/Clippy/tests cannot pass without changing excluded `src/precompact_guard.rs`; format differences and two `format!("{digest:x}")` type errors are recorded in the RLS-04 evidence. All three protected Rust hashes still match CDS-00 and were not staged.
- **Implementation commit:** `c512d2016257eaf852a54655e209054f9a374bae`.
- **Remote SHA:** Not published; push authorization has not been granted.
