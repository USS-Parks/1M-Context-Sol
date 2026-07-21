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

## RLS-02 - Freeze executable parity fixtures

- **Date:** 2026-07-20
- **Status:** Complete for the accepted reference; executable consumption is an RLS-03 gate.
- **Files:** `ticker/fixtures/behavior-cases.json`, `overlay/ContextOverlay.Core.psm1`, `overlay/Test-ContextOverlay.ps1`, and `docs/evidence/RLS-02/shared-behavior-fixtures.md`.
- **Coverage:** Active-vs-cumulative usage, baseline, exhaustion, stale, compaction, malformed input, bounded tails, task selection, subagent exclusion, pinning, and sidebar-aware layout centers.
- **Verification:** `overlay/Test-ContextOverlay.ps1` passed all shared cases; `git diff --check` passed.
- **Implementation commit:** Pending focused RLS-02 commit.
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
