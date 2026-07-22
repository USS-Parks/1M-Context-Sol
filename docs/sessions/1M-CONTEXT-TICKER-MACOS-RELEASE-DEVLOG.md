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
- **Hosted proof:** Run [29886220336](https://github.com/USS-Parks/1M-Context-Sol/actions/runs/29886220336) passed all twelve jobs. The macOS job built the release executable with warnings denied and ran six tests with zero failures.
- **Evidence:** `docs/evidence/MAC-01/native-macos-ticker.md`.
- **Boundary:** Real composer placement, physical-Mac use, real login-item registration, and live Codex configuration remain unclaimed.

## MAC-02 - Add safe macOS lifecycle

- **Date:** 2026-07-21
- **Status:** Candidate; hosted macOS gate pending.
- **Implementation:** App-owned catalog/state paths, byte-exact configuration backup, manifest hashes, four-key conflict refusal, atomic install, first-launch idempotence, upgrade, status, later-edit preservation, rollback on registration failure, stop, and uninstall.
- **Startup:** The production adapter uses `SMAppService.mainApp`; tests inject a fake service and never register a real login item.
- **Process boundary:** Stop and uninstall target only other processes with ticker bundle identifier `com.ussparks.1m-context-ticker`. Codex is never started, stopped, restarted, or activated.
- **Isolated tests:** Every lifecycle case creates its own temporary home and config, uses the shared Sol catalog, and removes the temporary tree afterward.
- **Local verification:** `python ticker/macos/verify-contract.py`; `python ticker/macos/verify-source.py`; `git diff --check`; protected-file hash comparison.
- **Implementation commit:** Pending candidate commit.
- **Remote SHA:** Pending normal push and hosted result.

### MAC-02 hosted closeout

- **Status:** Complete; M-G3 passed on hosted macOS.
- **Lifecycle commit:** `9e37b0379063bbaa05bac9b6228d2e857fa1edea`.
- **Literal-gate completion:** `c0ba24cf1a583c75e3e50c6d52be3254f5a98bf0`; added the injected stop test and checked bundle/login-item structure.
- **Hosted proof:** Run [29886681553](https://github.com/USS-Parks/1M-Context-Sol/actions/runs/29886681553) passed all twelve jobs. The macOS job built with warnings denied and ran nine lifecycle tests plus six ticker tests with zero failures.
- **Evidence:** `docs/evidence/MAC-02/safe-macos-lifecycle.md`.
- **Boundary:** No real home, Codex configuration, login item, running ticker, or Codex process was changed during verification.

## MAC-03 - Produce the universal DMG

- **Date:** 2026-07-21
- **Status:** Candidate; hosted macOS packaging gate pending.
- **Authorization:** The user approved the MAC-03 Actions extension and instructed Codex to commit and push it to `main`.
- **Packaging:** Cross-build `arm64` and `x86_64` release binaries against the macOS 13 target, combine them with `lipo`, assemble the checked app bundle, and create one compressed drag-to-Applications DMG.
- **Integrity:** Mount-time architecture/content checks, SHA-256 checksum, JSON manifest, and an independent final-byte verifier require exactly three output files.
- **Distribution boundary:** The manifest records unsigned, unnotarized, GitHub-hosted-only verification. `docs/MACOS.md` requires checksum verification and a per-app Control-click/Open flow; it never disables Gatekeeper system-wide.
- **Hosted artifact:** The focused job uses immutable `actions/upload-artifact` commit `ea165f8d65b6e75b540449e92b4886f43607fa02` and retains the three-file candidate for fourteen days.
- **Local verification:** `python ticker/macos/verify-contract.py`; `python ticker/macos/verify-source.py`; `python ticker/macos/verify-package-source.py`; Git Bash `bash -n ticker/macos/build-release.sh`; `git diff --check`.
- **Implementation commit:** Pending candidate commit.
- **Remote SHA:** Pending normal push and hosted result.
