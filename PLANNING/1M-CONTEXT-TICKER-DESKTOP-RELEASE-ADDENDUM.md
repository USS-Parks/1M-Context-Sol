# 1M Context Ticker Windows Release Addendum

> **Status:** APPROVED - FULL STS EXECUTION ACTIVE
>
> **Addendum ID:** 1MCT-R1
>
> **Approved:** 2026-07-20

## 1. Initiative and official identity

The product's official name is **1M Context Ticker**.

This addendum turns the accepted Windows PowerShell/WPF reference into one real Windows artifact:

- `1M-Context-Ticker-Windows-x64.exe`

It also replaces the stale repository welcome page with accurate 1M Context Ticker copy. macOS, DMG, logo/imagery work, signing, publishing, pushing, and GitHub Release creation are parked.

The authoritative repository is `C:\Users\17076\Documents\Codex 1M Context Project` on `main`. The installed PowerShell ticker remains the rollback reference until executable parity passes.

## 2. Authorization and preservation

The user's `Commence STS` authorizes RLS-00 through RLS-06 in dependency order, including local compilation, test execution, safe overlay-only start/stop, user-local executable installation, ordinary shortcut updates, README replacement, focused commits, and local artifact generation.

It does not authorize:

- stopping, restarting, or modifying Codex Desktop or its app-server;
- deleting or absorbing `src/lib.rs`, `src/main.rs`, or `src/precompact_guard.rs`;
- DMG/macOS work or repository imagery;
- signing certificates, credentials, push, tag, public release, or external communication;
- requests above the previously approved live-token boundary.

One canonical worktree remains sufficient. No new worktree or dependency tree is permitted.

## 3. Current baseline

- Installed reference: one live PowerShell/WPF helper under `%LOCALAPPDATA%\CodexContextOverlay`.
- Host state: exact `gpt-5.6-sol`, catalog total `1050000`, effective host budget `1008000`, automatic compaction `900000` with scope `total`.
- Accepted face: `Context: <active tokens> / 1M`, muted prompt color, text-sized footprint, centered in the composer region, sidebar-aware, focusless, topmost, and hidden outside foreground Codex.
- Windows compiler: in-box .NET Framework 4.8 C# compiler `4.8.9221.0`.
- Local .NET SDK: absent and not required.
- Canonical branch has unpublished commits; publication remains out of scope.
- Three legacy Rust files remain byte-preserved and excluded.

## 4. Product contract

1M Context Ticker is a lightweight Codex Desktop companion that:

- displays host-authored `last_token_usage.total_tokens` as `Context: <tokens> / 1M`;
- follows the freshest active root Codex Desktop task without reading transcript content for display;
- adapts to prompt color and right-sidebar state;
- hides when Codex is absent, minimized, or not foreground;
- uses a named single-instance lock;
- applies only the supported Sol catalog/window/compaction configuration with reversible ownership;
- never proxies prompts, replaces Codex, blocks normal compaction, or controls Codex processes.

## 5. Settled Windows stack

- Native C# WPF/Win32 executable targeting .NET Framework 4.8.
- Framework-dependent x64 GUI executable; no PowerShell runtime, browser shell, Electron, Tauri, web server, or third-party runtime DLL.
- In-box `JavaScriptSerializer` for bounded JSONL parsing.
- Native Win32 window enumeration, focus detection, DPI conversion, no-activate/tool-window styles, screen color sampling, and user-local single-instance mutex.
- Existing JSONL semantics, Sol catalog, golden cases, and ownership rules are reused; the rejected TUI/MCP/strict-compaction product is not.
- Binary artifact and SHA-256 manifest live under `dist/` for this local preview. Public release placement remains a later decision.

## 6. Repository welcome-page copy

The root README must replace the stale Context Continuum/TUI page with:

1. centered **1M Context Ticker** heading;
2. one-sentence description;
3. compact Windows download/build status;
4. **What it does** bullets;
5. tasteful placement description: text-sized, muted, centered between composer controls, responsive to sidebar changes;
6. installation, ordinary restart, startup, status, uninstall, build, verification, privacy, and limitations sections;
7. exact distinction between 1,050,000 catalog total, 1,008,000 effective host budget, live active tokens, and 900,000 automatic compaction;
8. no imagery until the user supplies the final repository treatment.

## 7. Verification gates

### W-G1 - Identity and behavior contract

- Official name and capacity terms are consistent in source, assembly metadata, artifact names, README, and manifests.
- Shared fixtures prove active usage comes from `last_token_usage`, never cumulative session totals.
- Malformed, missing, stale, compacted, ambiguous, and task-switch cases fail visibly safe.

### W-G2 - Native executable parity

- The executable launches without `powershell.exe` in its process tree.
- One instance tracks the correct root task, reports exact tokens, adapts theme/sidebar placement, hides with Codex focus, and does not activate or obstruct composer input.
- Duplicate launch exits without stacking.
- Runtime status identifies process, task, token/window state, placement, theme, and errors without transcript content.

### W-G3 - Safe lifecycle

- Clean install, startup shortcut, manual start, stop, status, upgrade from PowerShell reference, uninstall, exact-byte rollback, and later-edit preservation pass.
- Install/uninstall never controls Codex or creates a scheduled task.
- The PowerShell reference remains recoverable until executable acceptance passes.

### W-G4 - Artifact integrity

- A clean local build produces exact `dist\1M-Context-Ticker-Windows-x64.exe`.
- Artifact architecture is x64; assembly identity/version is correct; no unexpected runtime dependency is present.
- Self-test, focused fixtures, installed live acceptance, SHA-256 checksum, artifact manifest, and file size are recorded on one commit.

### W-G5 - Repository welcome page

- README renders cleanly on GitHub without stale TUI/MCP/Context Continuum product instructions.
- Functions, placement, installation, rollback, limitations, and verification are concise and source-faithful.
- Imagery is deliberately absent and marked parked rather than silently omitted.

## 8. Sequential prompt roster

### RLS-00 - Approve Windows release authority

**Objective:** Narrow the former cross-platform draft, record STS authorization, preserve state, and establish this roster.

**Gate:** Scope, exclusions, compiler, installed reference, dirty hashes, worktree state, and no-push boundary are recorded.

### RLS-01 - Replace repository welcome-page copy

**Objective:** Rebrand the root README as 1M Context Ticker without adding imagery.

**Gate:** W-G1 and W-G5 copy review passes; all commands and links resolve to real repository paths.

### RLS-02 - Freeze executable parity fixtures

**Objective:** Define platform-neutral active-context, task-selection, and layout expectations consumed by the PowerShell reference and C# executable.

**Gate:** Golden cases cover fresh/stale/malformed/cumulative-vs-active/compaction/task selection, and both implementations agree.

### RLS-03 - Build the native Windows executable

**Objective:** Port the accepted runtime to one compact C# WPF/Win32 executable.

**Gate:** W-G2 passes in self-test and interactive Desktop use; process inspection proves no PowerShell dependency.

### RLS-04 - Integrate safe executable lifecycle

**Objective:** Update the user-local manager and shortcuts to install, upgrade, start, stop, inspect, and uninstall the executable safely.

**Gate:** W-G3 passes, including real upgrade from the running PowerShell reference and rollback without Codex process control.

### RLS-05 - Produce the local release artifact

**Objective:** Make the Windows build reproducible and create the `.exe`, checksum, and artifact manifest.

**Gate:** W-G4 passes from a clean build directory; CI gains a Windows executable job without weakening preserved legacy gates.

### RLS-06 - Run Windows acceptance and close out

**Objective:** Verify ordinary user flow and reconcile source, binary, README, installation, evidence, Git, and retained state.

**Gate:** W-G1 through W-G5 pass on one source/artifact commit; exact uninstall and rebuild commands, artifact size/hash, installed process identity, unpublished commits, preserved dirty files, and remaining parked scope are reported.

## 9. Milestones

- **WRM1 - Branded source:** RLS-00 through RLS-02.
- **WRM2 - Native Windows preview:** RLS-03 through RLS-05.
- **WRM3 - Locally accepted Windows release candidate:** RLS-06.

## 10. Reuse ledger

- **Reuse:** Sol catalog, safe ownership rules, golden rollout fixtures, installed UX, native window/palette/layout formulas.
- **Translate:** PowerShell runtime behavior into C#; do not host or invoke PowerShell from the executable.
- **Extend:** installer lifecycle to support the executable and rollback to the accepted reference.
- **New:** C# executable source, deterministic build script, self-test mode, checksum/manifest, Windows artifact CI job.
- **Park:** DMG/macOS, logo/imagery, signing, push, tag, GitHub Release, and legacy-product deletion.

## 11. Completion boundary

This addendum completes a local Windows executable release candidate and accurate repository welcome-page copy. It does not complete public publication, signing, macOS support, imagery, or paid/large-token proof.
