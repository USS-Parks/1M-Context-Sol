# Codex Desktop Sol 1M Development Log

This append-only ledger records execution of `CDS1M-PSPR-1`. A prompt is complete only when its acceptance gate passes. Full STS execution was approved by the user on 2026-07-20; the mandatory M0 feasibility stop and all named special approvals remain in force.

## CDS-00 — Establish the corrected execution baseline

- **Date:** 2026-07-20
- **Status:** Complete
- **Authorization:** Full STS execution through the M0 feasibility stop.
- **Canonical repository:** `C:\Users\17076\Documents\Codex 1M Context Project`
- **Branch/remote:** `main` tracking `origin/main` at `26553c41488e0a22e06993a5c989717116d63e9d`; zero unpublished commits at capture.
- **Installed host:** `OpenAI.Codex` package `26.715.4045.0`; `ChatGPT.exe` product/file version `150.0.7871.124`; separately installed `codex-cli 0.144.5`.
- **Scope:** Promote the corrected PSPR, supersede the former execution authority without rewriting history, preserve all dirty work, and freeze the exact desktop target before feasibility inspection.
- **Preserved dirty work:** `src/lib.rs`, `src/main.rs`, and `src/precompact_guard.rs`; exact hashes are recorded in `docs/evidence/CDS-00/execution-baseline.md`.
- **Implementation:** None. No source behavior, user configuration, credentials, desktop process, installed package, or remote state changed.
- **Verification:**
  - approved plan contains 15 unique CDS prompt IDs and retains the M0 feasibility stop
  - `AGENTS.md` names the corrected PSPR/DEVLOG as sole active authority
  - the earlier PSPR and DEVLOG carry an explicit historical supersession record
  - all three pre-existing dirty-file SHA-256 values match the capture record after governance promotion
  - `HEAD` and `origin/main` both remain `26553c41488e0a22e06993a5c989717116d63e9d`; zero unpublished commits before the CDS-00 commit
  - global Codex config SHA-256 recorded as `439752EEBF9FE64D6E1C4A6AA9A2459555C89839ED0DB8E26BE219611625306E`; no global write occurred
  - `git diff --check` — passed
  - no new worktree, dependency tree, model request, credential access, desktop control, or remote mutation occurred
- **Implementation commit:** `43b606e293399e1ff4caa67e30c649df9c2021f7`
- **Remote SHA:** Not published; push authorization has not been granted for this lane.

## CDS-01 - Prove desktop adoption of host configuration

- **Date:** 2026-07-20
- **Status:** Complete; configuration-adoption gate passed.
- **Authorization:** User separately approved the temporary real-profile change and packaged app-server restart/control.
- **Evidence:** `docs/evidence/CDS-01/desktop-config-adoption.md`.
- **Result:** A disposable task created through the native Codex Desktop task API completed on exact `gpt-5.6-sol`; its local rollout reports `Codex Desktop`, app-server `0.145.0-alpha.18`, and host-authoritative `model_context_window = 1008000`, matching 96 percent of the configured 1,050,000-token catalog window.
- **Compaction setting:** Startup configuration used `model_auto_compact_token_limit = 900000` and `model_auto_compact_token_limit_scope = "total"`. The task rollout does not echo this field; actual boundary behavior remains gated by G3.
- **Rollback:** All four temporary owned keys were removed. The original bytes were not blindly restored because a simultaneous Store update changed five app-owned runtime values; those later changes were preserved. Final temporary-owned-key count is zero.
- **Host drift:** The installed package updated during the approved restart from `26.715.4045.0` to `26.715.7063.0`; the accepted evidence names the final package.
- **Implementation:** No product source change. No installed binary, credential, long-context request, or remote state changed.
- **Implementation commit:** Pending focused CDS-01 evidence commit.
- **Remote SHA:** Not published; push authorization has not been granted for this lane.
