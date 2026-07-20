# Codex Desktop Sol 1M Working Agreements

## Authority

`PLANNING/CODEX-DESKTOP-SOL-1M-PSPR.md` is the canonical active plan. Execute its CDS prompt roster in dependency order. Do not mark a prompt complete until its acceptance gate passes and its evidence is recorded in `docs/sessions/CODEX-DESKTOP-SOL-1M-DEVLOG.md`.

`PLANNING/CODEX-CONTEXT-CONTINUUM-PSPR.md` and `docs/sessions/CODEX-CONTEXT-CONTINUUM-DEVLOG.md` are preserved historical records of the superseded product direction. They are not execution authority.

## Non-negotiable product scope

- The only permitted model is exact slug `gpt-5.6-sol`.
- Never add a Terra, Luna, GPT-5.4, or other-model fallback.
- Keep total context (1,050,000), maximum input (922,000), maximum output (128,000), effective Codex budget, used/remaining context, and the 900,000 automatic-compaction threshold distinct.
- A catalog override is not live native-window proof. Do not claim Sol-1M until G2 passes in the actual desktop UX.
- Automatic compaction must use Codex's normal supported path at 900,000 tokens. Do not ship the superseded strict checkpoint-and-block behavior.
- The required UX is a live context dial inside the existing desktop prompt pill. A TUI, separate window, side panel, slash command, or MCP text response is not an acceptable substitute.
- MCP is auxiliary and read-only. It does not own model selection, token accounting, compaction, or desktop rendering.

## Execution discipline

- Prefer one focused commit per CDS prompt; use a small follow-up ledger commit only when recording the first commit's immutable SHA requires it.
- Record prompt ID, files, exact verification commands, results, local commit, and remote SHA in the DEVLOG.
- Preserve user-owned files, dirty work, and configuration. Global Codex changes, desktop restart/control, credentials, paid probes, installed-binary changes, releases, and external posts require their named special gates.
- Stop at the M0 feasibility review. If CDS-02 cannot prove a real prompt-pill UI seam, record the blocker and do not build a substitute product.

## Baseline Rust checks

Run `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`, and `git diff --check` unless a prompt prescribes a stricter gate.
