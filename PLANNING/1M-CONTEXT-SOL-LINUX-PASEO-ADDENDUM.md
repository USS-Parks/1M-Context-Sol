# 1M Context Sol headless Linux/Paseo addendum

Status: execution authorized by the user's 2026-07-23 implementation directive; local closeout pending the gates below.

## Governance

- Initiative: extend the GPT-5.6 Sol 1M catalog policy and verification lane to headless Linux and Codex agents launched through Paseo.
- Authoritative repository: `USS-Parks/1M-Context-Sol`, canonical checkout `C:\Users\17076\Documents\Codex 1M Context Project`.
- Settled stack: Python 3.11+ standard library manager behind a Bash entry point; exact Codex 0.145.0 compatibility profile; GitHub Actions `ubuntu-latest` verification.
- Source of truth: this addendum, `docs/LINUX-PASEO.md`, and `docs/sessions/1M-CONTEXT-SOL-LINUX-PASEO-DEVLOG.md`.
- Prerequisites: exact user-owned `model = "gpt-5.6-sol"`, existing `CODEX_HOME`, installed Codex CLI 0.145.0, and the daemon user's filesystem authority.
- Blockers: live model traffic and actual Paseo-host budget proof remain explicit user-operated gates.
- Exclusions: Linux desktop overlay/ticker, TUI replacement, prompt proxy, MCP-owned selection, alternate API lane, automatic Paseo restart, and ownership of model/auth/provider/approval/sandbox settings.

Drafting this addendum alone does not authorize future schema profiles, releases, daemon control, or model requests. The user's directive authorizes only the LNX-01 implementation and its non-live checks; `verify --live` remains separately opt-in at invocation time.

## Verification gates

LNX-01 is complete only when all of these pass:

- plan leaves the tested Codex home byte-for-byte unchanged;
- install owns exactly the four documented keys and preserves the user model;
- unsupported version, root/schema drift, duplicate/missing Sol, and field-type drift fail closed;
- source metadata is preserved outside the three catalog policy fields and hashes are deterministic;
- repeated install is refused;
- unchanged uninstall restores exact bytes, including LF/no-final-newline input;
- unrelated later edits survive uninstall, while owned drift refuses uninstall;
- explicit `CODEX_HOME` and paths containing spaces pass;
- Bash syntax, ShellCheck, Python compilation/tests, and `git diff --check` pass;
- an `ubuntu-latest` CI job runs the Linux suite without credentials or model traffic.

Live/Paseo acceptance is distinct: a user must start a fresh agent and explicitly invoke live verification before any actual host-budget claim is recorded.

## Reuse ledger

| Surface | Classification | Decision |
|---|---|---|
| Desktop owned-key values | Reuse | Keep the same four keys and values. |
| Windows config ownership semantics | Extraction | Reproduce byte backup, exact restore, unrelated-edit preservation, and owned-drift refusal without desktop lifecycle code. |
| Rust 0.144.5 catalog entry | Rejected reuse | Never install it on Codex 0.145.0. |
| Installed Codex 0.145.0 Sol entry | Reuse at runtime | Capture with `codex debug models --bundled`, validate, preserve, and narrowly modify. |
| Desktop ticker | Excluded | Windows/macOS only. |
| Paseo model selection | Existing seam | Document provider/default behavior; do not intercept or rewrite it. |

## Sequential prompt roster

### LNX-01 — Headless Linux/Paseo policy lifecycle

Objective: implement plan/install/status/verify/uninstall, exact 0.145.0 catalog compatibility, narrow config ownership, Paseo operating guidance, tests, and `ubuntu-latest` CI.

Acceptance gate: every verification gate above passes locally; hosted CI and live/Paseo evidence are reported separately and never inferred.

Milestone: one usable headless Linux policy lane, independently releasable from the Windows/macOS ticker artifacts.

## Completion criteria

The addendum is locally complete when LNX-01's non-live gates pass and the DEVLOG records exact commands/results. It is hosted-complete only after the relevant GitHub Actions run is green. It is live-complete only after a fresh Linux/Paseo Codex agent reports real task-budget evidence from an explicitly permitted request.
