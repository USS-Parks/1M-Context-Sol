# Codex Desktop Sol 1M Floating Overlay Addendum

> **Status:** APPROVED - FULL STS EXECUTION ACTIVE
>
> **Addendum ID:** CDS1M-A1
>
> **Approved:** 2026-07-20

## 1. Contract change

The user accepts one narrowly defined substitute for the unavailable native prompt-pill component seam: a small Windows overlay window that visually floats over and tracks the existing Codex Desktop prompt pill.

The overlay is allowed to be technically separate from the signed Desktop renderer only when it:

- remains visually anchored to the Codex composer;
- never opens a replacement chat client, dashboard, panel, terminal, or TUI;
- does not take keyboard focus or obstruct normal prompt entry;
- hides when Codex is hidden, minimized, or not the foreground application;
- displays host-authoritative active-context state and labels stale or ambiguous state;
- uses Codex's supported model/catalog/compaction configuration rather than proxying requests;
- is removable with exact owned-setting rollback.

The CDS-02 native-seam no-go remains historical truth. This addendum does not rewrite or claim that the overlay is a native prompt-pill component.

## 2. Authority and exclusions

The user's approval of this addendum authorizes full sequential execution through local acceptance, including:

- creating and running the project-owned overlay;
- a bounded temporary or installed four-key Sol-1M configuration change with backup and rollback;
- starting/stopping the project-owned overlay process;
- restarting the packaged Codex app-server when startup-only configuration requires it;
- creating and archiving bounded disposable Desktop acceptance tasks.

It does not authorize a request above 272,000 tokens, installed Codex binary modification, package extraction/repacking, external communication, release publication, repository push, or deletion of user-owned work.

No new worktree is required. The canonical checkout remains authoritative and the three pre-existing dirty source files remain byte-preserved and excluded from addendum commits.

## 3. Settled implementation constraints

- Windows-only local overlay for the installed Codex Desktop package.
- Native WPF/Win32 facilities available on Windows; no Tauri, Electron, browser server, or new dependency tree.
- Read only local Codex rollout `token_count` events.
- Match the official Codex TUI calculation: use `last_token_usage.total_tokens`, `model_context_window`, and the 12,000-token baseline for remaining percentage.
- Prefer the freshest root `Codex Desktop` rollout; keep the selected file stable while it is active; expose stale/ambiguous state instead of inventing certainty.
- Show the short task ID in the tooltip so selection can be audited.
- Total catalog window `1050000`; expected effective host window `1008000`; automatic-compaction threshold `900000`, scope `total`.
- MCP is not required for the dial and will not be added unless acceptance proves a missing read-only need.

## 4. Verification gates

### AO-G1 - State truth

- Fixture and live parsing use `last_token_usage`, never cumulative `total_token_usage`, for active context.
- Missing, malformed, stale, compacted, and task-switch states fail visibly safe.
- The displayed task ID and source file are inspectable without transcript content.

### AO-G2 - Overlay behavior

- One small borderless overlay tracks the Codex window.
- It is topmost, non-activating, and click-through outside its own dial affordance.
- It hides when Codex is minimized or not foreground.
- Scaling and position offsets are configurable without code changes.

### AO-G3 - Sol-1M configuration safety

- The installer owns only exact model/window/compaction/catalog keys plus its own overlay startup artifact.
- Apply creates a byte-exact backup and refuses conflicts.
- Restore removes owned keys while preserving later user/app changes.
- No request proxy, alternate model, or compaction blocker is introduced.

### AO-G4 - Local acceptance

- A fresh Desktop task reports exact `gpt-5.6-sol` and `1008000` effective tokens.
- The overlay visibly updates from that task's host event and identifies the task.
- The 900,000 threshold is shown distinctly from remaining capacity.
- Restart and uninstall/restore checks pass.
- No TUI or developer terminal is required for normal installed use.

## 5. Sequential prompt roster

### CDO-00 - Record the approved overlay contract

**Objective:** Preserve the native-seam no-go and make this addendum the active continuation authority.

**Gate:** Governance, authorization, exclusions, dirty-file hashes, worktree state, and no-delete reuse decision are recorded.

### CDO-01 - Prove state selection and window anchoring

**Objective:** Build the smallest non-shipping spike that parses official token events, selects an auditable active root task, and computes an anchor rectangle from the real Codex window.

**Gate:** Parser fixtures and live read-only output agree with the current task; ambiguous/stale cases are visible; a dry-run reports a valid on-screen anchor without moving or controlling Codex.

### CDO-02 - Implement the floating context dial

**Objective:** Turn the proven spike into one focusless WPF/Win32 overlay.

**Gate:** Automated parser/state checks pass; the live overlay tracks/hides correctly, does not steal focus, displays the exact task/window/used/remaining/threshold state, and can be stopped cleanly.

### CDO-03 - Implement install and rollback

**Objective:** Package the overlay and the proven Sol-1M configuration as one bounded local install with exact ownership.

**Gate:** Dry-run, apply, restart, startup launch, conflict refusal, stop, uninstall, and preservation checks pass; a reinstall leaves the accepted product active.

### CDO-04 - Run local Desktop acceptance and close out

**Objective:** Verify the ordinary user flow in the installed Desktop app and reconcile code, evidence, installation, and Git state.

**Gate:** AO-G1 through AO-G4 pass on one commit/build; any unproven long-context or production compaction-boundary claim remains explicitly open; the user receives exact uninstall instructions and retained-state inventory.

## 6. Completion boundary

This addendum completes a local Windows product only. Public release, a paid/large token proof, multi-platform support, and a true native prompt-pill integration remain outside scope.

## 7. User-safety execution amendment - 2026-07-20

The user's later instruction overrides the earlier permission to restart the packaged app-server during installation. Installation and removal must not stop, restart, or otherwise control Codex Desktop or its app-server.

The accepted activation flow is:

- install files under the user's local application-data directory;
- own only the four context/catalog keys while leaving the existing exact `model` key user-owned;
- create ordinary user Startup and Start Menu shortcuts, never a scheduled task;
- require the user to quit/reopen Codex normally for startup configuration adoption;
- start or stop only the overlay helper itself;
- keep a named single-instance lock so repeated shortcut launches cannot stack stale dials.

This amendment is authoritative for CDO-03 and CDO-04. The aborted app-server-control attempt rolled back fully before this safer flow was applied.

## 8. Final ticker display amendment - 2026-07-20

The user rejected both a large remaining-percentage dial and a subsequent progress ring. The accepted face is a compact ticker capsule at the prompt pill's upper-right edge:

- the ticker shows `Context: ` followed by the exact host `last_token_usage.total_tokens` count with thousands separators and literal ` / 1M`;
- no window-size, remaining-percent, or threshold text appears on the face;
- full remaining/window/900,000-threshold details remain available in the tooltip;
- the installed default is 190 by 28 device-independent pixels on the prompt pill's bottom row, centered between the approval and model controls;
- the face has no border/frame and uses a luminance-derived muted foreground matching Codex's subdued composer labels;
- the capsule samples quiet pixels from the actual Codex prompt pill and derives neutral foreground, border, and ring colors from the observed luminance, so Codex light/dark theme—not a separate hard-coded theme—controls its appearance.
