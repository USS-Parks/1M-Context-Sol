# Codex Desktop Sol 1M Working Agreements

## Authority

`PLANNING/CODEX-DESKTOP-SOL-1M-PSPR.md`, approved overlay addendum `PLANNING/CODEX-DESKTOP-SOL-1M-OVERLAY-ADDENDUM.md`, completed Windows release addendum `PLANNING/1M-CONTEXT-TICKER-DESKTOP-RELEASE-ADDENDUM.md`, and approved macOS addendum `PLANNING/1M-CONTEXT-TICKER-MACOS-DMG-ADDENDUM.md` are the canonical active authority. The base PSPR records the native prompt-pill no-go; the overlay addendum records the accepted reference; the Windows roster remains completed history; execute the MAC roster in dependency order. Do not mark a MAC prompt complete until its gate passes and evidence is recorded in `docs/sessions/1M-CONTEXT-TICKER-MACOS-RELEASE-DEVLOG.md`.

`PLANNING/CODEX-CONTEXT-CONTINUUM-PSPR.md` and `docs/sessions/CODEX-CONTEXT-CONTINUUM-DEVLOG.md` are preserved historical records of the superseded product direction. They are not execution authority.

## Non-negotiable product scope

- The only permitted model is exact slug `gpt-5.6-sol`.
- Never add a Terra, Luna, GPT-5.4, or other-model fallback.
- Keep total context (1,050,000), maximum input (922,000), maximum output (128,000), effective Codex budget, used/remaining context, and the 900,000 automatic-compaction threshold distinct.
- A catalog override is not live native-window proof. Do not claim Sol-1M until G2 passes in the actual desktop UX.
- Automatic compaction must use Codex's normal supported path at 900,000 tokens. Do not ship the superseded strict checkpoint-and-block behavior.
- The accepted UX is one focusless native overlay per supported desktop platform, visually anchored over the existing prompt pill. A TUI, replacement client, dashboard, side panel, ordinary window, slash command, or MCP text response remains unacceptable.
- MCP is auxiliary and read-only. It does not own model selection, token accounting, compaction, or desktop rendering.

## Execution discipline

- Prefer one focused commit per CDS prompt; use a small follow-up ledger commit only when recording the first commit's immutable SHA requires it.
- Record prompt ID, files, exact verification commands, results, local commit, and remote SHA in the DEVLOG.
- Preserve user-owned files, dirty work, and configuration. Global Codex changes, desktop restart/control, credentials, paid probes, installed-binary changes, releases, and external posts require their named special gates.
- Preserve the M0 native-seam no-go as history. The only approved continuation is the narrow floating overlay defined by `CDS1M-A1`; do not expand it into another client or dashboard.
- The accepted Windows `.exe` remains unchanged. The approved macOS lane is the narrow Swift/AppKit implementation in `1MCT-M1`; imagery, signing, notarization, tags, releases, and public artifact publication remain parked. GitHub Actions changes require separate approval immediately before editing or use.

## Baseline Rust checks

Run `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`, and `git diff --check` unless a prompt prescribes a stricter gate.
