# 1M Context Ticker macOS Release Development Log

This append-only ledger records execution of `1MCT-M1`. The user approved STS execution on 2026-07-21 and separately authorized focused commits and normal pushes to `main`. GitHub Actions edits, real macOS Codex or login-item changes, tags, releases, and public artifact publication retain their named special gates.

## MAC-00 - Freeze the automated macOS contract

- **Date:** 2026-07-21
- **Status:** Complete locally.
- **Authority:** Approved `PLANNING/1M-CONTEXT-TICKER-MACOS-DMG-ADDENDUM.md`; MAC-00 through MAC-03 are active in dependency order.
- **Baseline:** Canonical `main` checkout, one registered worktree, HEAD `8c678f7f5954a644f4cb517daaa33e4ffb56fed2`, with the three protected Rust files excluded and unchanged.
- **Contract:** Exact `com.openai.codex` host, `~/.codex/sessions` state with `Codex Desktop` originator and automatic subagent exclusion, `last_token_usage.total_tokens`, required `1008000` host window, four owned configuration keys, passive `NSPanel`, `SMAppService.mainApp`, `macos-15`, and universal `arm64` plus `x86_64` output.
- **Current-host boundary:** The ticker retains a macOS 13 deployment target; OpenAI's current combined desktop host requires macOS 14 or later.
- **Automated-only boundary:** No physical-Mac placement, real login launch, Gatekeeper interaction, or live Codex configuration is claimed.
- **Files:** macOS addendum approval record, `AGENTS.md`, verification ledger, machine contract/checker, architecture contract, and `docs/evidence/MAC-00/automated-contract.md`.
- **Verification:** `python ticker/macos/verify-contract.py`; `git diff --check`; protected-file hash comparison.
- **Implementation commit:** `81e8895950e97d79e0b6dfe8c42909150ca59923`.
- **Remote SHA:** Pending normal push to `origin/main`.
