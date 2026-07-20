# Codex Desktop Sol 1M Context Dial — Canonical Plan / Sequential Prompt Roster

> **Status:** M0 NO-GO — CDS-01 PASSED; CDS-02 BLOCKED — CDS-X1 OR CONTRACT CHANGE REQUIRES USER APPROVAL
>
> **Plan ID:** CDS1M-PSPR-1
>
> **Draft date:** 2026-07-20

## 1. Governance and authorization

### 1.1 Initiative

Deliver the requested capability inside the existing Codex experience in the ChatGPT desktop app on this Windows machine:

- exact model `gpt-5.6-sol`;
- a native 1,050,000-token model window for newly started local Codex tasks;
- automatic compaction at 900,000 tokens, while preserving the user's ability to request manual compaction;
- a small, continuously updated context-remaining dial integrated into the existing prompt pill/composer;
- a minimal local MCP server only where MCP is useful for read-only status and verification.

This is a desktop-host integration. It is not a terminal application, replacement agent, alternate chat client, or durable-memory product.

### 1.2 Authoritative locations

- **Target implementation repository:** `C:\Users\17076\Documents\Codex 1M Context Project`
- **Draft plan location:** `C:\Users\17076\Documents\Codex Context Sidecar\CODEX-DESKTOP-SOL-1M-PSPR.md`
- **Planned canonical path after approval:** `PLANNING/CODEX-DESKTOP-SOL-1M-PSPR.md` in the target repository
- **Target product surface:** local Codex mode in the installed ChatGPT desktop app for Windows
- **Initial target runtime:** the installed Codex host and exact `gpt-5.6-sol` route available on this machine

This plan supersedes the execution authority of `CODEX-CONTEXT-CONTINUUM-PSPR.md` as of 2026-07-20. Prompt CDS-00 records the transition while preserving the earlier plan and DEVLOG as historical evidence.

### 1.3 Authorization boundary

The user approved full STS execution on 2026-07-20. Execution proceeds sequentially through the M0 feasibility stop and then returns to the user for review. This approval does not waive the named special gates below.

The following remain separate special approvals even during STS execution:

- changing the user's real global Codex configuration;
- restarting or controlling the desktop app for live acceptance;
- running long-context requests with material token or monetary cost;
- modifying, repackaging, or replacing installed ChatGPT/Codex desktop binaries;
- creating an upstream issue, pull request, or other external post;
- publishing a release.

### 1.4 Current state at drafting

- Existing published source provides a command-scoped Sol catalog launcher and configuration-management components.
- The existing user-facing path is a separate TUI and does not satisfy this plan.
- The target repository has unfinished local compaction-guard changes; they must be preserved and reconciled before reuse decisions.
- The installed Codex parser has accepted a command-scoped catalog resolving exact `gpt-5.6-sol` to a 1,050,000-token window, 96 percent effective budget, and 900,000 automatic-compaction threshold.
- No live request above 272,000 tokens has yet proven the shipping route.
- No documented MCP or plugin contract currently proves that third-party code can insert a control into the desktop prompt pill.

## 2. Corrected product contract

### 2.1 Required user experience

The completed product must let the user open the normal ChatGPT desktop app, select or enter Codex, and start an ordinary local task. In that existing UX:

1. the task starts on exact `gpt-5.6-sol` with the verified 1,050,000-token catalog policy;
2. the prompt pill contains a small context dial without opening another application;
3. the dial updates from host-authoritative context-usage state and shows remaining capacity relative to the active window;
4. the dial exposes exact remaining/used tokens, model identity, window, and 900,000 automatic-compaction threshold through a compact tooltip or detail affordance;
5. automatic compaction does not occur before the configured 900,000-token threshold;
6. at the threshold, Codex follows its normal supported compaction behavior rather than being blocked by a replacement continuity system;
7. uninstall restores the user's prior configuration and removes every owned integration artifact.

### 2.2 Non-substitution rule

None of the following satisfies the required prompt-pill dial:

- a Codex TUI footer or terminal meter;
- a separate window, tray application, web dashboard, or side panel;
- an MCP tool response that the user must request;
- a slash command, status command, notification, or log file;
- an estimated counter that is not reconciled with host-authoritative usage;
- screenshots or mockups without a live desktop integration.

If the prompt-pill UI cannot be implemented through a supported or explicitly approved desktop seam, the product is blocked. The roster may not silently substitute another UX.

### 2.3 Role of MCP

MCP is deliberately narrow. It may expose read-only tools such as:

- `get_context_status`;
- `verify_sol_1m_policy`;
- `explain_context_thresholds`.

MCP does not own model selection, model-catalog resolution, host token accounting, automatic compaction, or desktop rendering. Those remain Codex host/configuration responsibilities.

### 2.4 Settled numerical semantics

- exact model slug: `gpt-5.6-sol`;
- total model window: 1,050,000 tokens;
- maximum model input: 922,000 tokens;
- maximum model output: 128,000 tokens;
- requested automatic-compaction threshold: 900,000 tokens;
- effective Codex budget must be measured and displayed separately from the total window;
- the dial must label the actual host counter and scope it receives; it may not call an estimate “exact.”

The 900,000 threshold initially uses Codex's documented `total` scope unless current-host evidence proves that a different supported scope is required to match the prompt-pill counter. Any change requires a recorded addendum and user approval.

## 3. Settled stack and authority boundaries

- Shared Codex `config.toml` for supported model, window, catalog, and compaction settings.
- Versioned exact-Sol model-catalog overlay only when the installed host requires it.
- Minimal native helper only for atomic install, verification, rollback, and MCP transport.
- STDIO MCP server for bounded read-only status tools.
- The existing desktop/app-server protocol or another documented desktop extension seam for live token state and prompt-pill rendering.
- Rust may be retained for the small helper/MCP server when reuse is cheaper than replacement.
- Desktop UI code must use the actual host's supported language/build system; this PSPR does not preselect a framework before CDS-02 proves the seam.

Codex remains authoritative for authentication, task lifecycle, request construction, token-budget accounting, compaction, and rendering. The integration may configure or display those host-owned values but may not invent a parallel authority.

## 4. Explicit exclusions

The following are outside this initiative unless the user approves a later addendum:

- a replacement TUI or chat client;
- a durable transcript reservoir, SQLite/FTS store, retrieval engine, recall capsules, or successor-task orchestrator;
- blocking all compaction or treating every `PostCompact` event as an incident;
- alternate models, fallback models, third-party providers, or model routers;
- an OpenAI-compatible proxy or Responses transport shim;
- IDE-specific or web-specific UI;
- Linux or macOS support;
- public marketplace, community, showcase, or marketing work;
- telemetry, cloud storage, analytics, or transcript upload;
- a broad security platform or general-purpose Codex management suite.

## 5. Reuse ledger

| Existing asset | Decision | Bounded use |
|---|---|---|
| `src/model_catalog.rs` | Extract/reuse | Exact-Sol catalog parsing, overlay generation, version/hash validation |
| `src/config_manager.rs` | Extract/reuse | Atomic owned-key edit, backup, conflict refusal, exact rollback |
| `src/doctor.rs` | Reduce/reuse | Only checks needed to prove active desktop Sol/window/threshold state |
| `src/probe.rs` | Reduce/reuse | Installed version, auth lane, and catalog evidence needed by acceptance |
| `src/claim_contract.rs` | Reduce | Preserve truthful numerical labels without carrying the old product scope |
| `src/sol_launcher.rs` | Test-harness only | May help compare catalog resolution; must not ship as the user experience |
| `src/startup_policy.rs` | Park by default | Reuse only if a narrow startup check is necessary and non-disruptive |
| `src/precompact_guard.rs` | Do not ship | Strict checkpoint-and-block behavior conflicts with compaction at 900,000 |
| Planned reservoir/retrieval work | Exclude | Not required for this goal |
| Existing architecture/PSPR documents | Historical evidence | Do not treat the former scope as current product requirements |

New work is permitted only for the proven desktop UI integration, host-state bridge, minimal MCP server, installer/rollback, and their tests.

## 6. Verification gates

A prompt is incomplete when its gate is missing, skipped, flaky, or contradicted by live desktop evidence.

### G0 — Scope and preservation

- the corrected contract and exclusions are explicit;
- all existing dirty, committed, and remote work is inventoried before changes;
- the prior plan/DEVLOG remain preserved as history;
- no old feature is deleted merely to simplify the new lane.

### G1 — Desktop seam feasibility

- the exact installed desktop and Codex host versions are recorded;
- a supported path is proven for desktop adoption of model/window/compaction configuration in a fresh task;
- a supported or explicitly approvable path is proven for reading host-authoritative context usage and rendering inside the prompt pill;
- an MCP-only status response, separate UI, or TUI does not pass;
- if either required seam is absent, execution stops before implementation and produces a bounded blocker report.

### G2 — Native Sol 1M truth

- a fresh task in the normal desktop UX reports exact `gpt-5.6-sol`;
- the active desktop task resolves 1,050,000 total, 922,000 maximum input, 128,000 maximum output, and the measured effective budget as distinct values;
- a controlled desktop request above 272,000 tokens completes without client-side truncation or premature compaction;
- the returned model identity and host usage evidence are recorded;
- the final costly probe runs only under an approved ceiling.

### G3 — Compaction threshold truth

- the active desktop task resolves automatic compaction at 900,000 tokens using the declared scope;
- low-threshold tests prove the same host code path without requiring an expensive full-scale run;
- production-threshold evidence proves no automatic compaction before 900,000 and normal supported compaction at or after the threshold;
- manual user-requested compaction remains available;
- no custom guard blocks compaction or starts a replacement TUI/task.

### G4 — Native dial UX

- the dial is visibly integrated into the existing desktop prompt pill;
- it updates during an active task without a manual MCP call or page refresh;
- its value is derived from or reconciled to host-authoritative token-budget state;
- it distinguishes total window, effective budget, used, remaining, and 900,000 compaction threshold;
- boundary, resize, theme, scaling, keyboard, and accessibility checks pass;
- no separate UI is needed for ordinary use.

### G5 — Install and rollback safety

- install is one bounded operation with a dry run;
- only declared config, catalog, MCP, and UI-integration artifacts are owned;
- an exact backup is created before mutation;
- user changes and unrelated MCP/config entries are preserved;
- update, interrupted install, conflict refusal, uninstall, and exact rollback pass;
- a clean desktop restart/new-task path is documented.

### G6 — Local product acceptance

- an ordinary newly created desktop Codex task automatically receives exact Sol-1M policy, the native dial, the 900,000 compaction threshold, and the minimal MCP tools;
- acceptance is performed in the user's actual desktop UX, not a developer harness;
- the same build passes install, use, restart, fresh-task, compaction, and uninstall/restore checks;
- the repository and evidence ledger agree on the accepted commit;
- no release claim exceeds the evidence.

## 7. Milestones

| Milestone | Usable cut | Prompts | Approval significance |
|---|---|---|---|
| M0 — Feasibility | Proven desktop configuration and prompt-pill UI seams | CDS-00–CDS-03 | Hard go/no-go before product implementation |
| M1 — Minimal host integration | Safe desktop config plus minimal MCP status | CDS-10–CDS-12 | First functional but not yet UX-complete cut |
| M2 — Requested desktop UX | Live prompt-pill dial and 900k compaction behavior | CDS-20–CDS-22 | The requested interaction exists in the real app |
| M3 — Local completion | Live Sol-1M proof, safe install/rollback, final acceptance | CDS-30–CDS-33 | Product is complete locally on this machine |

M0 is mandatory. Failure at M0 stops the roster; it does not authorize a substitute product.

## 8. Sequential prompt roster

### Phase 0 — Correct scope and prove the required seams

#### **CDS-00 — Establish the corrected execution baseline**

**Objective:** Preserve prior work, suspend the former product direction, and make this approved PSPR the sole active roster without rewriting history.

**Deliverables:** Repository/branch/remote/worktree inventory; dirty-file preservation record; old-plan supersession note; new DEVLOG header; exact installed desktop/Codex versions; source/config locations; no-delete reuse inventory.

**Acceptance gate:** G0 passes; the earlier plan and evidence remain accessible; no implementation or global configuration change occurs.

#### **CDS-01 — Prove desktop adoption of host configuration**

**Objective:** Determine, through read-only inspection and isolated test configuration, whether a fresh local desktop Codex task honors the supported model, model-catalog, context-window, and automatic-compaction settings.

**Deliverables:** Configuration precedence map; exact supported keys and value types; restart/new-task semantics; isolated desktop-start evidence; unsupported-key behavior; rollback instructions.

**Acceptance gate:** A fresh desktop task—not the TUI—demonstrably resolves exact `gpt-5.6-sol`, 1,050,000 total, the measured effective budget, and 900,000 compaction policy, or the prompt records the exact blocker and stops.

**Special gate:** Requires explicit approval immediately before any real global configuration write or desktop restart/control.

#### **CDS-02 — Prove the prompt-pill UI and token-state seam**

**Objective:** Identify and demonstrate the smallest supported or explicitly approvable seam that can read host-authoritative context status and render a small dial inside the existing desktop prompt pill.

**Deliverables:** Current official-surface review; installed-client/app-server read-only inspection; token-status event/field contract; prompt-pill component/extension contract; minimal non-shipping spike in the actual desktop surface; maintenance and compatibility assessment.

**Acceptance gate:** A live desktop spike places a changing test dial in the real prompt pill and feeds it host-authoritative test state. A mockup, TUI, side panel, separate window, MCP response, or DOM automation does not pass.

**Hard stop:** If no supported seam exists, record the exact missing capability and stop before CDS-10. Continue only after the user explicitly approves CDS-X1 or waives/changes the native-dial requirement through a PSPR addendum.

#### **CDS-03 — Freeze the minimal architecture and acceptance fixtures**

**Objective:** Convert the proven seams into the smallest implementation design, with no parallel runtime or speculative subsystem.

**Deliverables:** Component ownership map; desktop-host/config/MCP/UI data flow; threat notes; version compatibility boundary; dial state contract; 900k semantics; synthetic token-status fixtures; exact acceptance command list.

**Acceptance gate:** G1 passes in full; every component exists only to satisfy a required outcome; the user receives the M0 evidence and explicit go/no-go result.

### Phase 1 — Minimal desktop host integration

#### **CDS-10 — Extract the smallest reusable core**

**Objective:** Retain only the proven catalog, configuration, and diagnostic logic needed by the corrected product.

**Deliverables:** Focused modules or crate boundaries; old TUI marked non-shipping; strict compaction guard excluded; tests for exact Sol, 1.05M, 900k, ownership, and rollback; provenance mapping from reused code.

**Acceptance gate:** Build, format, lint, unit, and focused integration tests pass; no reservoir, replacement client, alternate model, or unapproved global write is introduced.

#### **CDS-11 — Implement safe desktop configuration install and rollback**

**Objective:** Apply the proven desktop-host settings atomically while preserving every user-owned configuration value.

**Deliverables:** Plan/apply/verify/restore operations; exact-byte backup; owned-setting manifest; compatible Sol-only catalog artifact; conflict detection; restart/new-task guidance.

**Acceptance gate:** Isolated clean/missing/commented/CRLF/conflicting configurations install and restore exactly; unsupported desktop/Codex versions fail closed; G5 configuration requirements pass.

#### **CDS-12 — Implement the minimal read-only MCP server**

**Objective:** Provide bounded inspection tools without assigning MCP authority it does not possess.

**Deliverables:** STDIO MCP server; `get_context_status`, `verify_sol_1m_policy`, and `explain_context_thresholds`; strict JSON schemas; timeouts; sanitized errors; desktop MCP registration owned by the installer.

**Acceptance gate:** The desktop lists the server after restart; all tools are read-only and schema-valid; status agrees with the host bridge; disabling MCP does not change model/window/compaction behavior or break the dial.

### Phase 2 — Implement the requested native desktop experience

#### **CDS-20 — Implement the prompt-pill context dial**

**Objective:** Add the smallest production-quality context dial to the existing desktop prompt pill through the seam proven in CDS-02.

**Deliverables:** Native component; host-state subscription; remaining-capacity arc; compact exact-value tooltip/detail; loading/unavailable/error states; light/dark/high-contrast styling; accessibility label.

**Acceptance gate:** G4 passes against synthetic boundary fixtures and a live desktop task; the dial adds no separate window, panel, TUI, or manual invocation.

#### **CDS-21 — Bind the dial to exact host semantics**

**Objective:** Ensure the displayed value and thresholds reflect what Codex actually uses for request admission and compaction.

**Deliverables:** Token-status adapter; total/effective/used/remaining fields; 900k threshold marker; stale-state detection; model/task change reset; reconciliation diagnostics that omit transcript content.

**Acceptance gate:** Recorded host events and dial values agree at zero, ordinary use, 272k, warning bands, 899,999, 900,000, post-compaction, new-task, and error boundaries; estimates are visibly labeled and cannot be mistaken for exact values.

#### **CDS-22 — Prove compaction occurs at 900k, not before**

**Objective:** Validate normal Codex compaction behavior under the requested threshold without reviving the old strict-block policy.

**Deliverables:** Low-threshold test harness using the same host path; automatic/manual compaction cases; one-shot UI transition; no-loop proof; production 900k configuration verification; sanitized evidence.

**Acceptance gate:** G3 passes; automatic compaction is absent below the threshold and occurs through the normal host path at/after it; manual compaction remains available; the dial transitions correctly.

### Phase 3 — Prove and complete the local product

#### **CDS-30 — Run the controlled desktop Sol-1M live proof**

**Objective:** Prove the actual desktop shipping route above the legacy 272k boundary and near the safe operating limit.

**Deliverables:** Cost estimate and ceiling; progressive deterministic canaries; returned model identity; host token-status trace; dial trace; compaction trace; sanitized report.

**Acceptance gate:** G2 passes on the same installed build/configuration intended for local use; any backend/account limit remains a blocker rather than being hidden by catalog arithmetic.

**Special gate:** Requires explicit approval of the exact probe, credential lane, maximum input, and monetary/token ceiling.

#### **CDS-31 — Package one-operation local install and uninstall**

**Objective:** Make the accepted integration reproducibly installable on this machine without developer commands or a second client.

**Deliverables:** Bounded installer/uninstaller; version manifest; desktop compatibility check; config/catalog/MCP/UI ownership; dry run; interrupted-install recovery; exact rollback.

**Acceptance gate:** G5 passes in an isolated Windows user profile or equivalent clean environment, followed by approved verification on the user's real profile.

#### **CDS-32 — Run real desktop acceptance**

**Objective:** Demonstrate the requested workflow end to end in the user's normal Codex desktop UX.

**Deliverables:** Fresh desktop task; exact-Sol/window proof; visible prompt-pill dial; progressive usage update; 900k behavior evidence; MCP status agreement; restart; second fresh task; uninstall/restore rehearsal.

**Acceptance gate:** G6 passes with no TUI or developer checkout involved; any remaining discrepancy is classified as release-blocking.

**Special gate:** Requires explicit approval immediately before controlling/restarting the desktop app or changing the user's real global Codex installation.

#### **CDS-33 — Close the local product milestone**

**Objective:** Reconcile code, evidence, documentation, and local installation state without expanding into public release work.

**Deliverables:** Final local verification report; exact commit; DEVLOG closeout; known limitations; support/rollback instructions; clean Git status; remote reconciliation only if push was authorized.

**Acceptance gate:** All G0–G6 requirements pass on one commit/build; the user confirms that the actual desktop UX matches the contract; retained worktrees and generated-data footprint are inventoried.

### Optional escalation — only if CDS-02 proves no usable desktop UI seam

#### **CDS-X1 — Obtain or create an authorized native desktop seam**

**Trigger:** CDS-02 proves that documented MCP, plugin, configuration, and app-server surfaces cannot render inside the prompt pill or expose the required host token state.

**Objective:** Pursue exactly one user-approved route: an OpenAI feature request/support escalation, a minimal upstream Codex app-server/UI proposal where source is available, or a legally and technically maintainable desktop patch/fork.

**Deliverables:** Exact missing API/component; smallest proposed contract; security and update implications; maintenance cost; upstream/support draft or patch spike; go/no-go recommendation.

**Acceptance gate:** The approved route produces a stable prompt-pill rendering and host-state seam that satisfies CDS-02, after which execution returns to CDS-03.

**Special gate:** External communication, installed-binary modification, reverse engineering, repackaging, or fork distribution each requires separate explicit approval. No such action is implied by ordinary STS authorization.

## 9. Prompt and commit discipline

- Execute prompts in dependency order.
- Use one focused implementation commit per prompt after its gate passes.
- Small ledger-only follow-up commits are permitted when an immutable implementation SHA must be recorded.
- Do not mark a prompt complete from mock, parser, catalog, CLI, or TUI evidence when the gate requires the desktop app.
- Keep the DEVLOG append-only with prompt ID, files, exact commands, result, evidence path, commit SHA, and remote SHA when published.
- Stop at M0 for user review even under broad STS authorization because CDS-02 is the fundamental feasibility decision.
- Do not create a new worktree unless the existing worktree cannot safely preserve the dirty historical lane; follow the global worktree inventory and retirement rules if isolation becomes necessary.

## 10. Parked decisions and override points

- **Desktop implementation seam:** deliberately unset until CDS-02 proves it.
- **Exact dial visual:** default is a small circular/arc remaining-capacity indicator inside the prompt pill with an accessible exact-value tooltip; visual tuning occurs only after the native seam works.
- **Warning colors/bands:** unset until the actual host semantics and desktop design tokens are available; they cannot change the 900k threshold.
- **Catalog override necessity:** use only if the installed host does not natively expose the required Sol entry.
- **MCP tool count:** three read-only tools maximum unless a demonstrated acceptance need justifies an addendum.
- **Public distribution:** parked. Local completion on this machine is the goal of this PSPR.
- **Cross-platform support:** parked.

## 11. Definition of done

This initiative is complete only when all statements below are true on the same locally installed build:

1. The user opens the normal ChatGPT desktop app and uses Codex; no replacement TUI or separate client is required.
2. A fresh local task runs exact `gpt-5.6-sol` with a live-proven 1,050,000-token total window.
3. The prompt pill contains the requested small context-remaining dial.
4. The dial updates automatically from host-authoritative state and exposes exact or honestly labeled values.
5. Automatic compaction does not occur before 900,000 tokens and follows the normal supported Codex path at/after the configured threshold.
6. A controlled live desktop request exceeds 272,000 tokens with returned model identity and without premature compaction.
7. The minimal MCP status tools work in the desktop app but are not required to make the window, dial, or compaction policy function.
8. Install, restart, fresh-task use, update/conflict behavior, uninstall, and exact rollback pass.
9. User-owned config, credentials, transcripts, plugins, and unrelated MCP entries remain intact.
10. Code, DEVLOG, evidence, local installed version, commit SHA, and product claims agree.

If the desktop prompt-pill seam is unavailable and CDS-X1 is not approved or does not succeed, the initiative remains blocked. A TUI, separate dashboard, or text-only MCP status command cannot be declared complete.

## 12. Approval options

- **`Approve CDS1M-PSPR-1 and run it STS.`** Begin CDS-00, proceed sequentially, and stop after the M0 feasibility evidence for user review.
- **`Revise the PSPR as follows: ...`** Update this draft only; do not execute.
- **`Approve CDS-00 through CDS-02 only.`** Run the preservation and feasibility prompts, then stop before architecture freeze or implementation.

## 13. Current official-source basis

- Codex desktop, CLI, and IDE share Codex configuration layers, including MCP configuration.
- Supported configuration includes model context-window and automatic-compaction controls.
- MCP connects Codex to tools and context providers; it is not documented as a host-shell UI customization mechanism.
- Plugins may bundle skills, connectors, MCP servers, hooks, and optional app UI, but current public documentation does not establish an API for modifying the native Codex prompt pill.
- GPT-5.6 guidance identifies exact `gpt-5.6-sol` as the flagship model and requires long-context behavior and cost to be validated rather than inferred from a model string.

References:

- <https://learn.chatgpt.com/docs/extend/mcp>
- <https://learn.chatgpt.com/docs/config-file/config-reference>
- <https://learn.chatgpt.com/docs/plugins>
- <https://developers.openai.com/api/docs/guides/latest-model>
