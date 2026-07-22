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

### MAC-00 publication closeout

- **Authorization:** The user explicitly authorized commits and normal pushes to `main` on 2026-07-21.
- **Push:** `origin/main` advanced normally from `7961497cd1f40b54332c71629c1a6267494b8bb7` to MAC-00 ledger SHA `c49abed21ab9164563e14cdaa7e9b04b2a4c0380`.
- **Remote proof:** `git ls-remote origin refs/heads/main` matched the pushed ledger SHA exactly.
- **Boundary:** No GitHub Actions file, tag, release, public artifact, real macOS configuration, or login item was changed.

## MAC-01 - Build the native macOS ticker

- **Date:** 2026-07-21
- **Status:** Candidate; hosted macOS gate pending.
- **Authorization:** The user separately approved the focused GitHub Actions edit and hosted use on 2026-07-21.
- **Implementation:** Swift Package Manager executable with a Foundation parser/selector, exact-window guard, AppKit host-window discovery, one passive `NSPanel`, system appearance, bounded status output, and responsive placement arithmetic.
- **Shared parity:** Tests consume `ticker/fixtures/behavior-cases.json` directly for five token, three selection, and four layout cases. A separate wrong-window test requires `Context: !` behavior.
- **Passive boundary:** The panel is borderless, nonactivating, floating, click-through, and non-key. Source gates reject capture APIs, tracking areas, global event monitors, tooltip wiring, and transcript surfaces.
- **Local verification:** `python ticker/macos/verify-contract.py`; `python ticker/macos/verify-source.py`; `git diff --check`.
- **Hosted gate:** The approved `macos-ticker` job builds the executable and runs the AppKit/shared-fixture suite on `macos-15` with Swift warnings denied.
- **Implementation commit:** Pending candidate commit.
- **Remote SHA:** Pending normal push and hosted result.

### MAC-01 hosted closeout

- **Status:** Complete; M-G1 and M-G2 passed on hosted macOS.
- **Candidate commit:** `e84f25de8f5202fd091020b21922d130659989e7`.
- **Test repair:** `daeaf965822b692bbd502c75eb9a820787ae296a`; moved one throwing fixture lookup outside an XCTest autoclosure.
- **Governance repair:** `14eafa7f9bafb814b86e8582070b4f00ed71d80d`; preserved the completed Windows roster and added structural validation for active `MAC-00` through `MAC-04`.
- **Hosted proof:** Run [29886220336](https://github.com/USS-Parks/1M-Context-Sol/actions/runs/29886220336) passed all thirteen jobs. The macOS job built the release executable with warnings denied and ran six tests with zero failures.
- **Evidence:** `docs/evidence/MAC-01/native-macos-ticker.md`.
- **Boundary:** Real composer placement, physical-Mac use, real login-item registration, and live Codex configuration remain unclaimed.
