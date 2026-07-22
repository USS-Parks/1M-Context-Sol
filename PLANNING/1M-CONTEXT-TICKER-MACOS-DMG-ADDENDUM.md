# 1M Context Ticker macOS DMG Addendum

> **Status:** APPROVED v3 - STS ACTIVE THROUGH MAC-03
>
> **Addendum ID:** 1MCT-M1
>
> **Extends:** Completed Windows release addendum `1MCT-R1`
>
> **Approval record:** The user approved STS execution on 2026-07-21 and separately authorized focused commits and normal pushes to `main`. GitHub Actions edits, real macOS Codex or login-item changes, tags, releases, and public artifact publication retain their named special gates.

## 1. Initiative

Add one macOS distribution option for the existing **1M Context Ticker**:

- `1M-Context-Ticker-macOS-universal.dmg`

This is the same product on macOS, not a redesign. The accepted Windows `.exe` remains unchanged. The macOS build must show the same `Context: <active tokens> / 1M` face, enforce the same exact Sol/window/compaction policy, and provide the same safe install, status, startup, and uninstall outcomes.

The DMG is an unsigned GitHub community build. No Apple account, App Store submission, Developer ID, signing, or notarization is part of this plan.

No physical-Mac or live Codex-on-macOS acceptance is planned. That verification boundary is recorded in technical evidence only; it does not add a label to the DMG filename, app UI, or GitHub download listing.

The authoritative repository remains `C:\Users\17076\Documents\Codex 1M Context Project` on `main`. This draft preserves `1MCT-R1` and its evidence as completed history; it does not rewrite that roster.

## 2. Authorization boundary

Drafting or approving this addendum does not authorize implementation. Execution begins only after the user approves `1MCT-M1` and authorizes named prompts or says `run it STS`.

Ordinary STS for MAC-00 through MAC-03 may authorize source edits and isolated automated tests. The following always require separate explicit approval immediately before use:

- pushing commits or changing GitHub Actions;
- modifying the real macOS Codex configuration or login-item state;
- creating a tag or GitHub Release;
- publishing the `.exe` or `.dmg` publicly.

The existing protected `src/lib.rs`, `src/main.rs`, and `src/precompact_guard.rs` lane remains excluded and must not be staged, reformatted, absorbed, or deleted.

## 3. Minimal scope

### Included

- one native Swift/AppKit ticker app;
- one universal macOS app containing `arm64` and `x86_64` slices;
- one drag-to-Applications `.dmg`;
- the existing shared behavior fixture and Sol catalog;
- exact-window fail-closed behavior (`1008000` host budget required for the normal `/ 1M` face);
- a borderless, nonactivating, click-through overlay with no tooltip or hover UI;
- foreground-Codex detection, active root-task selection, and responsive placement;
- reversible ownership of the same four Codex configuration keys;
- status, start-at-login, stop, and uninstall behavior;
- checksum, artifact manifest, focused macOS CI, and a minimal community-feedback note;
- a GitHub download table offering the Windows `.exe` and macOS `.dmg`.

### Explicitly excluded

- a settings application, menu-bar product, dashboard, updater, telemetry, or network service;
- a new shared framework, Rust rewrite, Tauri/Electron shell, or refactor of the Windows implementation;
- Linux, Mac App Store, `.pkg`, Homebrew, or automatic updating;
- Apple Developer Program enrollment, Developer ID signing, or notarization;
- screen-pixel color sampling or any design that requires Screen Recording permission;
- new branding, logo, screenshots, or marketing campaign;
- changes to Codex Desktop, its app-server, prompts, or process lifecycle.

## 4. Settled implementation

- **Language/UI:** Swift plus AppKit.
- **Overlay:** borderless `NSPanel`, nonactivating, floating, `ignoresMouseEvents = true`, hidden unless Codex is foreground.
- **Appearance:** compact system dark/light treatment; no screen capture and no hover popup.
- **State:** bounded reads of the same local Codex rollout metadata and `token_count` records used by Windows.
- **Configuration:** `~/.codex/config.toml` plus a catalog under `~/Library/Application Support/1M Context Ticker/`; byte-exact backup and owned-key-only restore.
- **Startup:** `SMAppService.mainApp` on macOS 13 or later.
- **Packaging:** Xcode command-line build, universal `.app`, drag-to-Applications DMG, SHA-256, and JSON manifest.
- **Public distribution:** unsigned community DMG on GitHub with SHA-256 verification and concise, honest instructions for macOS's unidentified-developer prompt. Instructions must never disable Gatekeeper globally.
- **Minimum target:** macOS 13.0. Raising this minimum requires an addendum; lowering it is not required.

## 5. Reuse ledger

- **Reuse unchanged:** `ticker/fixtures/behavior-cases.json`, `overlay/sol-1m-models.json`, capacity terminology, exact-window guard, privacy rules, and acceptance wording.
- **Translate narrowly:** the parser/task-selection logic from `ticker/windows/State.cs`, the passive face from `TickerWindow.cs`, and owned-config lifecycle semantics from `overlay/manage-overlay.ps1`.
- **Implement new only where platform-required:** AppKit window discovery/placement, Swift app bundle, `SMAppService`, Xcode build, and DMG packaging.
- **Do not create:** a cross-platform UI abstraction or second product architecture.

## 6. Verification gates

### M-G1 - Behavioral parity

- the same token, selection, stale, malformed, compaction, and wrong-window cases pass;
- active usage comes only from `last_token_usage.total_tokens`;
- a non-`1008000` host window shows `Context: !`, never a normal `/ 1M` claim.

### M-G2 - Passive macOS UX

- AppKit construction and unit tests require one borderless, nonactivating, click-through panel with the complete face;
- source and tests contain no tooltip, hover handler, focus activation, input capture, Screen Recording API, or transcript display;
- foreground detection, window-bound calculations, hiding rules, and layout cases pass against fixtures;
- actual placement over the real macOS Codex composer is outside the automated gate; downloader reports may create later follow-up work.

### M-G3 - Safe lifecycle

- install, first-launch, status, stop, upgrade, uninstall, exact restore, and later-edit preservation pass against isolated temporary homes/configs;
- the app bundle and `SMAppService` registration path are structurally verified; real login launch is not exercised;
- only the four owned Codex keys and named app-owned files are changed;
- Codex is never stopped or restarted by the ticker.

### M-G4 - DMG integrity

- the `.app` contains both `arm64` and `x86_64` slices;
- the DMG contains the app and Applications link only, plus any minimal install note required for clarity;
- checksum and manifest match the final bytes;
- expected unsigned Gatekeeper behavior and a bounded per-app open flow are documented without claiming live acceptance;
- instructions require SHA-256 verification first and never disable system-wide protections.

### M-G5 - Dual-platform GitHub release

- one release presents the existing Windows `.exe` and new macOS `.dmg` with separate checksums and clear platform instructions;
- both artifacts match their recorded commits/manifests;
- the release listing uses the plain macOS download name; linked technical evidence records that verification was automated only;
- no unsupported platform or native-1M claim exceeds live evidence.

## 7. Sequential prompt roster

### MAC-00 - Freeze the automated macOS contract

**Objective:** Freeze the supported assumptions, Codex paths/state shape, AppKit surfaces, Xcode runner, and automated-only verification boundary before implementation.

**Gate:** Current official Codex configuration keys and Apple APIs are recorded; shared fixtures define the required `1008000` task state; unsupported assumptions fail closed; verification evidence states that no physical-Mac acceptance was performed.

### MAC-01 - Build the native macOS ticker

**Objective:** Implement the smallest Swift/AppKit peer of the accepted Windows ticker.

**Gate:** M-G1 and M-G2 pass on a GitHub-hosted macOS runner; the same shared fixture is consumed; AppKit structure and layout tests pass without claiming real Codex placement.

### MAC-02 - Add safe macOS lifecycle

**Objective:** Implement reversible config ownership, app-owned state, start at login, status, stop, upgrade, and uninstall.

**Gate:** M-G3 passes in isolated temporary homes/configs and bundle checks without touching a real Codex installation.

### MAC-03 - Produce the universal DMG

**Objective:** Build the universal app and one deterministic-as-practical DMG with checksum and manifest, then add a focused macOS CI job.

**Gate:** Both architecture slices and automated tests pass on hosted macOS; DMG contents are minimal; checksum verification and expected unsigned Gatekeeper behavior are documented. CI/push remains a special approval.

### MAC-04 - Publish the dual-platform release

**Objective:** Publish one approved GitHub Release offering the Windows `.exe` and macOS `.dmg`.

**Gate:** M-G5 passes; assets, checksums, manifests, install/uninstall instructions, feedback route, release notes, tag, source commit, and DEVLOG agree.

**Special gate:** Requires explicit user approval of the final assets and release notes immediately before tag/release publication.

## 8. Milestones

- **MM1 - macOS implementation:** MAC-00 through MAC-02.
- **MM2 - CI-built DMG:** MAC-03.
- **MM3 - community dual-platform release:** MAC-04.

## 9. Execution discipline

- Execute one MAC prompt at a time in dependency order.
- Use focused commits and append prompt results to a dedicated macOS release DEVLOG.
- Do not create a new Windows worktree. A normal Mac checkout may be created only after the required baseline commits are safely published and its purpose/retirement condition is recorded.
- Compilation and CI prove only the recorded automated gates; they are not live macOS acceptance.
- Community reports after publication are feedback, not retroactive evidence for the release gate. Confirmed defects become separately approved follow-up work.
- Stop at every named special gate.

## 10. Completion boundary

This addendum is complete only when the existing Windows `.exe` and one unsigned universal macOS `.dmg` are offered together on GitHub with matching automated evidence and truthful checksum, Gatekeeper/open, install, uninstall, and feedback instructions. Physical-Mac acceptance is not required and must not be claimed in the technical evidence.

## 11. Approval options

- **`Approve 1MCT-M1 and run MAC-00 STS.`** Freeze the no-live-acceptance contract and automated seams only.
- **`Approve 1MCT-M1 and run it STS through MAC-03.`** Build the DMG and stop before public release.
- **`Approve MAC-04 for publication.`** Publish only the previously reviewed assets and release notes.
- **`Revise 1MCT-M1 as follows: ...`** Update this draft only; do not execute.
