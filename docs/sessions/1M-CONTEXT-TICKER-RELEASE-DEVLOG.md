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
