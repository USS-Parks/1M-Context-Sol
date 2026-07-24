# 1M Context Ticker Windows Release Development Log

This append-only ledger records execution of `1MCT-R1`. Full STS execution for RLS-00 through RLS-06 was approved by the user on 2026-07-20. The user separately authorized publication through RLS-05 and its CI repair on 2026-07-21; public release, signing, DMG/macOS, and imagery remain outside authorization.

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

## RLS-03 - Build the native Windows executable

- **Date:** 2026-07-20
- **Status:** Complete; native self-test and interactive parity passed.
- **Source:** `ticker/windows/Program.cs`, `State.cs`, `Native.cs`, `TickerWindow.cs`, `SelfTest.cs`, and `build.ps1`.
- **Build:** In-box .NET Framework 4.8 C# compiler, x64 WPF `winexe`, warnings-as-errors, no third-party runtime dependency.
- **Artifact proof:** 37,376 bytes, version `0.1.0.0`, AMD64; first hash `f3e3697611b95b92c2244bf965609ecbfa48b64e54e273bfdd971b8101c8f733`, clean canonical rebuild hash `7be18efe2266d4e779808f52ea8056cefc3588c14cf13cb15a7f2870ec2fc2b9`.
- **Open artifact gate:** Source-identical hashes differ; deterministic/normalized output remains required by RLS-05 and is not claimed at RLS-03.
- **Fixture proof:** Native executable consumed the same 5 token, 3 selection, and 4 layout cases as the PowerShell reference.
- **Live proof:** Exact current task/window, `190x18` footprint, theme/sidebar center, focusless styles, heartbeat, duplicate rejection, no error, and no PowerShell child process passed.
- **Rollback boundary:** Installed PowerShell reference files and shortcuts remain intact pending RLS-04 lifecycle migration.
- **Evidence:** `docs/evidence/RLS-03/native-executable-parity.md`.
- **Implementation commit:** `c0389222d9e95d6a8a82607d56160d9de165392f`
- **Remote SHA:** Not published; push authorization has not been granted.

## RLS-02 - Freeze executable parity fixtures

- **Date:** 2026-07-20
- **Status:** Complete for the accepted reference; executable consumption is an RLS-03 gate.
- **Files:** `ticker/fixtures/behavior-cases.json`, `overlay/ContextOverlay.Core.psm1`, `overlay/Test-ContextOverlay.ps1`, and `docs/evidence/RLS-02/shared-behavior-fixtures.md`.
- **Coverage:** Active-vs-cumulative usage, baseline, exhaustion, stale, compaction, malformed input, bounded tails, task selection, subagent exclusion, pinning, and sidebar-aware layout centers.
- **Verification:** `overlay/Test-ContextOverlay.ps1` passed all shared cases; `git diff --check` passed.
- **Implementation commit:** `368fcbba1a91006c37ab1b1b6da4ef60525e452c`
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

## RLS-04 - Integrate safe executable lifecycle

- **Date:** 2026-07-21
- **Status:** Complete; W-G3 passed.
- **Files:** Native-aware `overlay/manage-overlay.ps1`, lifecycle tests, rounded-pill correction in both runtimes, and `docs/evidence/RLS-04/native-executable-lifecycle.md`.
- **Lifecycle:** Clean native install, schema-1 PowerShell upgrade, fail-closed process inspection, start, duplicate rejection, stop, status, exact rollback, uninstall, and later-config-edit preservation are implemented and tested.
- **Real upgrade:** The running PowerShell reference was upgraded, the installed native executable was started and inspected, rollback restored the reference exactly, and a final upgrade left the native executable active.
- **Official identity:** Startup and Start Menu shortcuts migrated from legacy `Codex Context Dial` names to `1M Context Ticker` and now target the installed executable directly.
- **User correction:** The pinched elliptical clip was replaced by a true rounded-rectangle pill with uniform 5-pixel padding. Final live geometry was `198x30`, with no activation, fresh unambiguous host state, and no runtime error.
- **Installed proof:** Final installed executable/source hash `76733b8a1c600dd9ed90485152c4cee21df492c8acf8c8b6141cfc0e86475834`; installed and source manager hash `5455c5afcceb9260bbbb8e747a9da326ea4abf1b876e181614fdcb40149e34be`; exactly one installed native process; Codex process IDs unchanged.
- **Verification:** Native build/self-test passed; shared fixture suite passed; lifecycle suite passed; Windows PowerShell parser passed; `git diff --check` passed; live upgrade/rollback/duplicate/stop/start/shortcut/status checks passed.
- **Preserved-tree blocker:** Repository-wide Rust format/Clippy/tests cannot pass without changing excluded `src/precompact_guard.rs`; format differences and two `format!("{digest:x}")` type errors are recorded in the RLS-04 evidence. All three protected Rust hashes still match CDS-00 and were not staged.
- **Implementation commit:** `c512d2016257eaf852a54655e209054f9a374bae`.
- **Remote SHA:** Not published; push authorization has not been granted.

### RLS-04 publication closeout

- **Authorization:** User explicitly said `Commit and push RLS-04` on 2026-07-21.
- **Push:** `origin/main` advanced normally from `26553c41488e0a22e06993a5c989717116d63e9d` to RLS-04 ledger SHA `767d505c1d221f7864a82212e043dc499390989f`.
- **Verification:** `git ls-remote origin refs/heads/main` matched local `HEAD` exactly after push; the no-slop pre-push gate passed.
- **Boundary:** No tag, GitHub Release, pull request, signing, or external post was created.

## RLS-05 - Produce the local release artifact

- **Date:** 2026-07-21
- **Status:** Complete; W-G4 passed locally and on hosted CI.
- **Files:** Deterministic `ticker/windows/build.ps1`, new `verify-release.ps1`, independent Windows CI job, exact three-file `dist`, updated README/attributes, and `docs/evidence/RLS-05/windows-release-artifact.md`.
- **Root cause:** The in-box compiler rejects `/deterministic+`; raw builds differed only in PE timestamp, one compiler-generated identity string, and its matching module MVID.
- **Correction:** Build normalization derives one stable source-seeded GUID, replaces exactly one identity/MVID pair, zeros the PE timestamp, and refuses unexpected binary structure before self-test and assembly inspection.
- **Reproducibility:** Two fresh builds plus final `dist` matched byte-for-byte for executable, checksum, and canonical-LF manifest.
- **Artifact:** `1M-Context-Ticker-Windows-x64.exe`, 37,376 bytes, AMD64, version `0.1.0.0`, SHA-256 `29051b7ba096f466e3361796c4bb674a9b5be22e3d484d65cf10cb9f506830e3`.
- **Supporting hashes:** Checksum file `0d0440646d4b5acb668282e8e2d87438882292d1d7c24fcb0189289e0fe59ebb`; artifact manifest `c92cef1a77dfcc6fa036063c6adf4172c32d506b949b9bebed053b226c43749b`.
- **Self-test/dependencies:** Shared 5 token, 3 selection, and 4 layout cases passed; exact dependency allowlist contains only seven .NET Framework assemblies recorded in the manifest.
- **Installed acceptance:** The exact artifact hash matched source, installed file, and install manifest; one live native PID, zero children, zero PowerShell ticker processes, `198x30` no-activate pill, fresh unambiguous state, and unchanged Codex process IDs.
- **CI:** `windows-executable` runs the same two-build verifier on `windows-latest` in runner-temporary storage without changing existing jobs. GitHub Actions run `29836629916` passed all 11 jobs.
- **Preservation:** Excluded `src/lib.rs`, `src/main.rs`, and `src/precompact_guard.rs` remain unstaged and byte-preserved.
- **Implementation commit:** `e3d8e39028446e37eeb25d8cbb74335b0df39622`.
- **Remote SHA:** Published through governance repair SHA `29b57c94ecfa101584ccdddfac837f165ad492a5`.

### RLS-05 CI repair closeout

- **Observed failure:** Run `29829535681` failed both Rust tests and the dedicated governance job on one stale assertion requiring the historical Context Continuum PSPR's former active status.
- **Repair:** Governance now validates that the historical PSPR is explicitly superseded and independently proves the approved `1MCT-R1` addendum contains contiguous RLS-00 through RLS-06 prompts with objectives/gates and is routed by `AGENTS.md`.
- **Local proof:** Clean-HEAD format, check, warnings-denied Clippy, all tests, rustdoc, six positive governance tests, and the invalid-PSPR negative proof passed.
- **Commit:** `29b57c94ecfa101584ccdddfac837f165ad492a5`.
- **Hosted proof:** Run [29836629916](https://github.com/USS-Parks/1M-Context-Sol/actions/runs/29836629916) passed every job, including Windows native executable, Rust quality/docs, PSPR/evidence contracts, all negative gates, secrets, and supply chain.

## RLS-06 - Run Windows acceptance and close out

- **Date:** 2026-07-21
- **Status:** Complete locally; W-G1 through W-G5 passed for the local Windows release candidate.
- **Acceptance discoveries:** The exact documented manager command failed to initialize the default source path under Windows PowerShell 5.1; the dynamic face clipped the trailing `1M`; the default WPF tooltip obstructed the composer and caused palette feedback after hover; and a smaller host window could have retained a misleading hardcoded `/ 1M` face.
- **Corrections:** Initialize `SourceRoot` after parameter binding; measure every face and add a fixed safety margin; remove tooltip/right-click capture; apply `WS_EX_TRANSPARENT`; require exact host window `1008000` in both parsers; render `Context: !` on invalid state; and expose `required_host_window`, `one_m_context_verified`, and `display_state` in status.
- **User flow:** Real stop/start/duplicate/status, uninstall, deterministic rebuild, plan/install/start, exact restore, later-edit preservation, shortcut, process identity, and active-task checks passed without controlling Codex.
- **Live final state:** One native PID `21076`; zero children; zero PowerShell ticker processes; `213x30`; visible in foreground Codex; no activate; click-through input; dark theme; fresh and unambiguous current task; `1008000` host window; no error.
- **Artifact:** 38,400 bytes; SHA-256 `f62558811f95866c4284ea2f68ce06355805230179735c74cbae1244c0337f56`; checksum-file SHA-256 `aa895166094c4886165e2acf3b69dd8bb8aa3cfeb20ebc868830886f1b1ecda4`; manifest SHA-256 `a94bbd774177cb669ed1af1a3a65903f69221143a35e2363a351de89f74491c9`.
- **Focused verification:** Shared fixtures passed; lifecycle suite passed; two-build release verifier passed; five token, three selection, four layout, two face-width, and one wrong-window guard cases passed; PowerShell parsing, README links/identity, Clippy, and `git diff --check` passed.
- **Baseline boundary:** Rust format remains blocked only by excluded `src/precompact_guard.rs`; the reused governance-test binary retained a stale temporary compile-time manifest path. These non-results do not alter the Windows gates, and published pre-RLS-06 SHA `7961497` remains green across 11 hosted jobs.
- **Preservation:** Protected hashes for `src/lib.rs`, `src/main.rs`, and `src/precompact_guard.rs` remain unchanged and unstaged. One canonical worktree remains; `target` is approximately 922 MiB and `dist` approximately 39 KiB.
- **Evidence:** `docs/evidence/RLS-06/windows-acceptance-and-closeout.md`.
- **Implementation commit:** `41063488866daca71fa3776a80a5aa2f493fcd58`.
- **Remote SHA:** `7961497cd1f40b54332c71629c1a6267494b8bb7`; RLS-06 push is not authorized.
- **Parked:** Push, tag, GitHub Release, signing, imagery, macOS/DMG, external communication, and paid above-boundary proof.

## Post-release repair - Restore the live count

- **Date:** 2026-07-23
- **Problem:** The shipped parsers refused any host window that was not exactly `1008000`, so a healthy task could render `Context: !`, and status could report `error-or-unverified` while the 1M policy was live. An uncommitted session had additionally rebuilt the ticker to show `Context: ? / 1M` with frozen root selection whenever two root tasks advanced together, forced `cctx sol run` onto an `OPENAI_API_KEY`-only provider lane, wired an unstaged PreCompact handler that blocked every compaction, and upgraded the local install to that build.
- **Working-tree cleanup:** Reverted the uncommitted `src/lib.rs`, `src/main.rs`, `src/sol_launcher.rs`, and ticker changes to HEAD and deleted the never-committed `src/precompact_guard.rs`; the ChatGPT-authenticated launch route and normal compaction behavior stand.
- **Repair:** Both parsers now accept any usable host window (greater than the 12,000-token baseline) and report it as read. The face denominator is derived from the host window: 1M-class budgets render `/ 1M`, smaller windows render their real size, and `Context: !` remains only for unreadable state. Status verifies `one_m_context_verified` against a 1,000,000-token minimum and exposes `minimum_one_m_window`. Upgrade handles an installed native ticker in place and waits on the stopped process handle instead of racing a PID recheck.
- **Files:** `ticker/windows/State.cs`, `ticker/windows/TickerWindow.cs`, `ticker/windows/SelfTest.cs`, `overlay/ContextOverlay.Core.psm1`, `overlay/context-overlay.ps1`, `overlay/manage-overlay.ps1`, `overlay/Test-ContextOverlay.ps1`, `overlay/Test-OverlayInstaller.ps1`, `README.md`, `dist/*`.
- **Verification:** `cargo fmt --check` clean; 76 Rust tests passed across 11 suites; shared-fixture overlay tests passed; installer lifecycle tests passed; two source-identical clean builds matched; native self-test passed with 5 token, 3 selection, 4 layout, 3 face-width, 1 idle-I/O, and 1 window-guard cases.
- **Deployment:** Upgraded the local install in place to SHA-256 `cecbd31045608606eb03f1a04f878e43c3ac31d3c0be6ee0b54f423349e29baa` (44,544 bytes); one native PID running with no-activate click-through styles; owned config values match. The reference module read the live rollout end to end: 170,108 of 1,008,000 host tokens, unambiguous.
- **Boundary:** `ticker/macos` still carries the exact-window contract and was not modified; the same guard change applies there when a Mac build environment is available.

## Live boundary test - Proven to 355,074 tokens

- **Date:** 2026-07-23 (test events 2026-07-24T02:01Z through 03:03Z)
- **Test:** A user-driven controlled deep-context run in a live root Codex Desktop task under the installed 1M policy, with nine parallel subagent loader threads, followed by a full rollout sweep.
- **Evidence:** Root session `019f91a3-3bc6-7762-85a6-3d5cc128eebd` carried host window `1,008,000` across all 202 token events. It crossed `258,400` at 02:06:25Z (`265,537`), `272,000` at 02:07:12Z (`275,849`), and `353,400` at 02:59:31Z (`353,784`), and completed its deepest turn at `355,074` tokens at 03:00:04Z. No context-exhaustion failure occurred; the session ended three minutes later with a deliberate reset (`355,074 -> 0`, no error event), which is distinct from the pre-install failure signature.
- **Contrast:** Pre-install sessions on 2026-07-17 and 2026-07-19 ended cold at `226,505`, `228,910`, and `239,993` of a `258,400` budget with no further turns and no logged error - the "ran out of usable context" walls this policy removes.
- **Subagent behavior:** All nine loader threads compacted at roughly `226,000`-`231,600` down to about `25,000` and continued working. Thread-level compaction is governed separately from the root task's `900,000` threshold; single-thread depth belongs in the root task.
- **Conclusion:** Server-proven acceptance for exact `gpt-5.6-sol` under this policy extends from the prior ~`252K` observation to `355,074` tokens. The `355K`-to-`900K` range remains unproven, and `900,000` root auto-compaction remains the designed ceiling behavior.
