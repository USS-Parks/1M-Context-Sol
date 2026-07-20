# Context Continuum for GPT-5.6 Sol — Canonical Plan / Sequential Prompt Roster

> **Status:** SUPERSEDED 2026-07-20 — HISTORICAL RECORD — EXECUTION MOVED TO `CODEX-DESKTOP-SOL-1M-PSPR.md`

> **Supersession note:** This plan accurately records the earlier Context Continuum direction and completed CAC evidence, but it is no longer execution authority. The user corrected the product goal to the existing Codex desktop UX with exact Sol-1M configuration, a native prompt-pill context dial, and normal automatic compaction at 900,000 tokens. Do not resume this roster unless the user explicitly restores it through a new addendum.
> **Initiative:** Codex Added Context
> **Plan ID:** CAC-PSPR-2
> **Last updated:** 2026-07-19
> **Authoritative repository:** `https://github.com/USS-Parks/1M-Context-Sol`
> **Authoritative working folder:** `C:\Users\17076\Documents\Codex 1M Context Project`
> **Model scope:** `gpt-5.6-sol` only; no substitute model is permitted
> **Planned product name:** Context Continuum for GPT-5.6 Sol
> **Planned plugin ID:** `context-continuum`
> **Planned executable:** `cctx`

## 1. Authorization and execution state

This document is the reviewable execution plan. Drafting, revising, or approving its wording is not authorization to implement it.

Execution begins only after the user says **`run it STS`** or explicitly approves named prompts or milestones. Until then:

- do not initialize or clone the target repository;
- do not change the user's Codex configuration, hooks, plugins, model catalog, or PATH;
- do not install dependencies or binaries;
- do not create commits, push branches, publish releases, or make community posts;
- do not spend paid API tokens on long-context probes.

After STS approval, prompts run in dependency order. Each prompt gets one focused commit unless its entry explicitly says it is an evidence-only gate. A prompt is not complete merely because code exists; its acceptance gate must pass and its evidence must be recorded.

External publication and potentially expensive live tests retain their explicit gates even after general STS approval.

### Execution authorization record

- **Authorized:** 2026-07-19
- **User instruction:** “I approve this PSPR for full STS status, authorizing all commits and pushes in this session to reach the Goal without fail.”
- **Execution scope:** CAC-00 onward in roster order, including prompt-scoped commits and pushes.
- **Retained special gates:** Undefined paid usage, credential entry, changes to the user's real global Codex installation, release-candidate publication, community posting, and showcase submission remain fail-closed at their named gates.

## 2. Mission

Build and open-source a Codex runtime extension that gives every newly started Codex task:

1. the exact model **`gpt-5.6-sol`**, with no Terra, Luna, GPT-5.4, or other-model fallback;
2. Sol's documented **1,050,000-token native context window** instead of Codex's current 272,000-token client cap;
3. a strict compaction guard that blocks context compaction and prevents silent, irreversible loss;
4. a durable local Sol continuity reservoir of at least 1,000,000 tokens for rollover, recall, recovery, and longer agentic loops;
5. explicit checkpoint, export, deletion, and successor-task controls; and
6. startup enforcement that stops or visibly fails any task that is not proven to be Sol with the one-million-class window active.

The native-window goal is a total model context window, not a claim of one million input tokens. OpenAI currently documents Sol as 1,050,000 total tokens, with a 922,000-token maximum input and 128,000-token maximum output. Those three numbers must remain distinct in the product and its public claims.

The primary targets are local Codex CLI, Codex desktop, and the Codex IDE extension. The first supported host platform is Windows, followed by Linux and macOS release validation.

## 3. Product terminology

The product is not accurately described by only one of “agent,” “app,” or “MCP.” The settled default description is:

> **Context Continuum is a Codex runtime extension delivered as a Codex plugin plus a companion native CLI/service and local MCP server.**

| Name | Meaning |
|---|---|
| Context Continuum for GPT-5.6 Sol | Public product name |
| `context-continuum` | Codex plugin and MCP server ID |
| `cctx` | Native executable, installer, hook receiver, doctor, store, and MCP host |
| Native window | Tokens simultaneously available to the selected model backend |
| Effective Codex budget | Native window made available by Codex after its safety reserve |
| Reservoir | Durable local append-only context retained outside the active model window |
| Recovery capsule | Small, verified summary/index injected into a verified successor or emergency incident recovery |
| Strict guard | Mode that blocks compaction even when doing so halts the current turn |
| Successor rollover | A fresh Sol task started before the source task reaches compaction pressure |
| Subscription lane | Sol through the user's existing ChatGPT-authenticated Codex backend |
| Direct API lane | Sol through Codex API-key authentication and the standard Responses API |

The product name is an override point until prompt CAC-00. The technical split is not: a plugin-only or MCP-only implementation cannot meet the lifecycle, installation, storage, and diagnostic requirements.

## 4. Feasibility baseline and claim contract

### 4.1 Verified current Codex boundaries

Planning inspection on 2026-07-19 established the following current baseline:

- OpenAI's current GPT-5.6 Sol model page documents a 1,050,000-token context window, a 922,000-token maximum input, and a 128,000-token maximum output.
- The installed Codex CLI is 0.144.5, is logged in through ChatGPT, and already defaults to `gpt-5.6-sol`.
- The installed and current open-source Codex model catalogs nevertheless set Sol's `context_window` and `max_context_window` to 272,000.
- Codex changelog and source evidence show that the Sol/Terra/Luna 272,000 values were an intentional 0.144.x correction, not a display typo.
- The inspected Codex implementation clamps `model_context_window` to the catalog's `max_context_window`, so a config-only `model_context_window = 1050000` cannot lift the present Sol cap.
- A startup-only `model_catalog_json` is an authoritative catalog replacement for the process. It can carry a Sol entry with a different context/max/effective-window policy, making this the first client-side seam to test.
- Codex applies `effective_context_window_percent` before context budgeting; its default is 95 percent.
- The current catalog's 95 percent reserve turns 272,000 into roughly 258,000 usable Codex budget, which explains the user's observed “256k” class behavior.
- Codex supports startup configuration for `model_context_window`, `model_auto_compact_token_limit`, and a custom `model_catalog_json`.
- Codex supports local ChatGPT authentication and local API-key authentication. API-key authentication uses standard API pricing and can access Sol through the documented API surface, subject to account access and limits.
- Sol's current catalog has `use_responses_lite = true`; inspected Codex tests show this requests `reasoning.context = "all_turns"`, which can help multi-turn continuity but does not change the hard context window.
- Codex lifecycle hooks include `SessionStart`, `UserPromptSubmit`, `PostToolUse`, `PreCompact`, `PostCompact`, `Stop`, and related events.
- `PreCompact` can return `continue: false`; this prevents that compaction but can also abort the active turn at the hard limit.
- Hook-injected context is intentionally small, so a million tokens cannot be pushed wholesale through hook stdout. Durable selective retrieval is required.
- MCP tools can retrieve stored context, but MCP does not change the model backend's native limit.

#### CAC-01 execution correction — bundled versus resolved catalog

The CAC-01 probe on 2026-07-19 supersedes one planning shorthand above without weakening the goal:

- installed `codex-cli 0.144.5` bundles a Sol entry at 372,000 / 372,000;
- the same binary's runtime-resolved catalog provides Sol at 272,000 / 272,000;
- the current open-source Codex `main` catalog at commit `c86b1be3cdbe12307843bcc9e7a44c1904ddcdf1` also provides 272,000 / 272,000;
- the resolved 95 percent effective policy yields 258,400, reproducing the observed “256k class” window; and
- the implementation must inspect bundled, resolved, and replacement catalogs separately because catalog refresh can override bundled metadata.

The original bullet saying the “installed catalog” is 272,000 is retained as planning history but must now be read as the runtime-resolved catalog, not the bundled catalog. Machine-readable evidence is recorded under `docs/evidence/CAC-01/`.

### 4.2 Normative public claim

The release may use this claim only after all corresponding gates pass:

> **GPT-5.6 Sol with its native 1.05M window. Compaction blocked. Durable continuity beyond the window.**

The words have exact meanings:

- **Sol native 1.05M** means the active model slug is `gpt-5.6-sol`, the model/backend specification is 1,050,000 total tokens, Codex has loaded the intended catalog policy, and a live request has exceeded the ordinary 272,000-token Codex boundary. It never means one million input tokens; Sol's documented maximum input is 922,000.
- **Effective Codex budget** is reported separately from total window and maximum input. A catalog percentage that makes a status screen say one million is not sufficient evidence by itself.
- **Durable continuity** means an integrity-checked local Sol reservoir can ingest, retain, search, and exactly retrieve a corpus whose tokenizer count is at least 1,000,000 tokens. It is required robustness, not a substitute for the native gate.
- **No silent context loss** means every compaction attempt creates and verifies a checkpoint before the product either blocks compaction or explicitly allows a recoverable continuity transition.
- **Every new task** means installation is global and automatically active for fresh local Codex tasks. Any non-Sol model selection, missing 1M catalog policy, or unproven auth route fails closed with a visible remediation path.

### 4.3 Native 1M hypothesis to prove

The first implementation path is configuration-based, not a Codex fork and not a model substitution:

1. keep every fresh task on exact slug `gpt-5.6-sol`;
2. derive a one-model, versioned custom catalog from the installed Sol entry so current instructions and feature flags are preserved;
3. set Sol's total context/max metadata to the documented 1,050,000 window and calibrate its effective budget without exceeding the documented 922,000 maximum input;
4. test the user's existing ChatGPT-authenticated Codex route above 272,000 input tokens;
5. if that service route enforces 272,000, use Codex's supported API-key authentication and direct Sol Responses route;
6. set a conservative pre-limit threshold, checkpoint proactively, and block every PreCompact event;
7. roll long agentic work into a fresh, identically configured Sol task before the input ceiling rather than compacting the source task; and
8. have `cctx doctor` prove the model, auth lane, total window, maximum input, effective budget, guard, and reservoir state at every startup.

A 1,050,000 catalog window with a 96 percent effective budget yields 1,008,000 internal Codex tokens, but Sol's documented input ceiling remains 922,000. Therefore 96 percent is only a candidate metadata setting; the operational request and proactive-rollover thresholds must be calibrated below 922,000 with room for hidden instructions, tool state, and output. Arithmetic alone cannot settle a shipping value.

If this path does not pass, the escalation ladder is:

1. distinguish a local catalog/config failure from a ChatGPT Codex service limit using progressively larger, cost-bounded Sol probes;
2. retry through Codex's supported API-key authentication against the official Sol Responses endpoint;
3. test a companion launcher with a narrowly patched, reproducible build of open-source Codex if client budgeting or compaction logic still blocks Sol's documented window;
4. prepare an upstream Codex issue or pull request for supported Sol long-context metadata, compaction-disable, or successor-rollover controls; and
5. keep the native-1M gate open rather than substituting another model or relabeling the reservoir.

No alternative model is permitted anywhere in that ladder. The reservoir and compaction work may still ship as a clearly labeled preview if valuable, but the one-goal completion state remains open until Sol's native one-million-class window is actually demonstrated in Codex or the user explicitly changes the goal.

## 5. Governance

### 5.1 Source of truth

After CAC-00, execution truth is recorded in:

1. `PLANNING/CODEX-CONTEXT-CONTINUUM-PSPR.md` — canonical scope and ordered roster;
2. `docs/sessions/CODEX-CONTEXT-CONTINUUM-DEVLOG.md` — prompt, files, commands, results, commit SHA, and remote SHA;
3. `docs/VERIFICATION.md` — reproducible verification commands and current gate state;
4. `docs/evidence/CAC-*/` — machine-readable measurements, sanitized logs, hashes, and screenshots where required; and
5. Git history and GitHub Actions — immutable publication evidence.

The PSPR is copied into the repository at CAC-00. Later changes are append-only status changes or explicit addenda; history is never silently rewritten.

### 5.2 Repository and branch discipline

- Canonical remote: `USS-Parks/1M-Context-Sol`.
- Canonical default branch: `main`.
- Canonical implementation branch: `main`, per the user's 2026-07-19 repository override. Earlier feature-branch publication remains recorded in the DEVLOG as history.
- One prompt per focused commit, or one tightly justified evidence-only entry with no source commit.
- The DEVLOG records local commit SHA and remote SHA immediately after each authorized push.
- Parallel sessions, if later authorized, use separate Git worktrees and never share an index.
- User-owned or unrelated files are not modified.
- No force push, history rewrite, broad cleanup, or destructive reset is authorized by this plan.

### 5.3 Settled stack

The v0.1 default stack is intentionally narrow:

- one Rust executable with internal modules rather than multiple services or micro-crates;
- SQLite in WAL mode with bundled FTS5 for the local reservoir;
- a Rust MCP server hosted by the same executable;
- Codex plugin hooks for lifecycle capture and guard behavior;
- Codex global configuration and a generated model-catalog overlay for automatic enrollment;
- GitHub Actions for tests, packaging, provenance, SBOM, checksums, and releases;
- Apache-2.0 as the default open-source license, subject to user override at CAC-00.

Dependency choices and versions are frozen only after prompt-local evidence. The implementation should prefer well-maintained standard crates and avoid a separate database process, web server, Electron shell, cloud backend, or mandatory embedding model.

### 5.4 Prerequisites

- working Git and GitHub authentication with write access to the canonical repository;
- a current Codex CLI installation with stable hooks and plugin support;
- user access to GPT-5.6 Sol in Codex for native live tests;
- an OpenAI API project/key and explicit spend ceiling if the existing ChatGPT-authenticated Codex route enforces 272,000;
- Rust stable toolchain and required Windows build tools;
- explicit approval before global Codex configuration changes;
- explicit cost ceiling before any million-token live probe; and
- explicit preview approval before posting to the Codex community or submitting a showcase.

### 5.5 Explicit exclusions for v0.1

- changing OpenAI server-side entitlements or bypassing provider limits;
- claiming native 1M for a backend that reports or proves a smaller hard cap;
- replacing Codex's reasoning loop or tool permission system;
- relying on undocumented transcript parsing as the only source of truth;
- mandatory cloud storage, telemetry, or embeddings; an API key is permitted only for the direct official Sol lane and is never stored in the repository or reservoir;
- a graphical desktop application when CLI status and diagnostics suffice;
- automatically exfiltrating prompts, tool output, credentials, or repository data;
- silently blocking a task without a visible reason and recovery command;
- publishing under wording that implies OpenAI endorsement; and
- external forum posts, showcase submissions, or paid tests without their named gate.

## 6. Architecture

### 6.1 Runtime flow

```text
Fresh Codex task
    |
    +-- SessionStart hook --> cctx validates model/config/plugin/store
    |                            |
    |                            +-- injects a small status/recovery index
    |
    +-- UserPromptSubmit ------> append prompt + selective bounded recall
    |
    +-- PostToolUse -----------> append structured tool input/output
    |
    +-- Stop ------------------> append assistant output + checkpoint turn
    |
    +-- PreCompact ------------> fsync + transcript snapshot + hash manifest
                                     |
                                     +-- block compaction visibly
                                     +-- preserve the source task unchanged
                                     +-- offer verified successor rollover

Unexpected PostCompact --------> mark policy violation + preserve evidence

GPT-5.6 Sol <---- bounded recalled excerpts ---- cctx MCP <---- SQLite reservoir
```

### 6.2 Components

#### Native window controller

- generates a version-pinned model-catalog overlay rather than hand-editing opaque catalog JSON;
- applies user-owned config changes atomically with a dry run, backup, and exact rollback manifest;
- contains exactly the current `gpt-5.6-sol` entry and prevents model-picker fallback from being mistaken for compliance;
- supports a ChatGPT-authenticated Sol probe lane and a direct API-key Sol lane, with strict no-compaction behavior in both;
- detects catalog/schema drift and fails closed instead of carrying a stale override forward;
- reports auth lane, model slug, total window, maximum input, maximum output, configured, effective, auto-compaction, and observed limits separately.

#### Lifecycle capture

- consumes documented structured hook fields first;
- records prompts, assistant messages, tool inputs, tool results, session metadata, compaction events, and explicit user pins;
- takes a raw transcript snapshot at Stop and PreCompact as forensic fallback;
- treats transcript format as versioned and unstable, never as an unguarded parser contract;
- keeps hook execution fast and writes atomically.

#### Local reservoir

- uses append-only content-addressed events with SHA-256 hashes and monotonic sequence numbers;
- stores session/task namespaces, turns, artifacts, checkpoints, pins, and retrieval provenance;
- uses SQLite WAL plus FTS5, with idempotent migrations and corruption checks;
- provides exact fetch by ID/hash and ranked search by lexical match, recency, pin weight, session, path, tool, and event type;
- supports export, retention, redaction, and cryptographic deletion of configured sensitive fields where feasible;
- keeps optional embeddings out of the required path.

#### Recall controller

- injects only a bounded, labeled context capsule through hooks;
- exposes deeper retrieval through MCP tools;
- marks recalled content as historical/untrusted data, not executable instructions;
- includes source event IDs and hashes so the model or user can fetch exact originals;
- favors pins and exact identifiers over lossy summaries.

#### Compaction guard

- checkpoints the database and transcript before every known compaction attempt;
- validates record counts and hashes before returning control;
- returns `continue: false` for both automatic and manual PreCompact events and explains that the active turn was halted to preserve context;
- checkpoints proactively below Sol's maximum-input boundary and initiates or recommends a verified successor task before compaction pressure;
- treats any PostCompact event as a policy violation to diagnose rather than a successful operating mode;
- never describes strict blocking as unlimited same-thread liveness; long-running work continues through a fresh Sol successor with durable provenance.

#### MCP surface

The initial tool surface is deliberately small:

- `context_status` — native/effective/durable capacity and health;
- `context_search` — bounded ranked search with filters;
- `context_fetch` — exact events or artifacts by ID/hash;
- `context_timeline` — compact chronological reconstruction;
- `context_pin` — durable user/agent-selected facts and artifacts;
- `context_checkpoint` — explicit checkpoint and integrity manifest;
- `context_export` — sanitized portable export;
- `context_forget` — scoped deletion with a dry run and audit record.

No MCP tool may change the active model, silently mutate global config, or present stored text as trusted instructions.

#### Installer and doctor

- installs the executable, plugin, hooks, MCP declaration, and global Sol-1M policy;
- uses plugin-owned data paths and user-specific secure state paths;
- performs atomic, surgical config changes and removes only values it owns;
- validates plugin trust, hook discovery, binary path, database permissions, catalog compatibility, active model, and effective token budget;
- offers a complete dry run and machine-readable JSON output;
- makes uninstall and rollback first-class acceptance paths.

### 6.3 Planned repository shape

The official plugin scaffolder decides exact manifest placement at CAC-40. The intended logical shape is:

```text
Cargo.toml
src/
  cli.rs
  config.rs
  doctor.rs
  hooks.rs
  mcp.rs
  model_catalog.rs
  recall.rs
  reservoir.rs
  security.rs
  token_count.rs
plugin/
  .codex-plugin/plugin.json
  hooks/hooks.json
  .mcp.json
docs/
  architecture/
  evidence/
  sessions/
  VERIFICATION.md
PLANNING/
benchmarks/
tests/
scripts/
.github/workflows/
```

## 7. Reuse ledger

| Capability | Treatment | Source/seam | Rationale |
|---|---|---|---|
| GPT-5.6 Sol 1.05M window | Expose through supported auth/config/catalog seams | Sol model spec, Codex auth, config, and model catalog | The native backend exists; Codex currently applies a lower client catalog cap |
| Direct Sol API fallback | Reuse | Codex API-key authentication and Responses API | Preserves Codex while avoiding a ChatGPT service-lane cap if proven |
| Lifecycle interception | Reuse | Stable Codex hooks | Provides startup, prompt, tool, stop, and compaction events |
| Distribution | Reuse | Codex plugin packaging | Automatic discovery and community installation |
| Model-callable recall | Implement at existing seam | MCP | Standard bounded retrieval interface |
| Durable storage | Reuse proven primitive | SQLite WAL + FTS5 | Local, portable, auditable, no service dependency |
| Token accounting | Extend a standard tokenizer | GPT-5 family tokenizer plus conservative fallback | Capacity claims must be reproducible |
| Transcript recovery | Extend cautiously | Codex transcript path | Forensic fallback only because format is unstable |
| In-process context contributors | Park/extract later | Open-source Codex extension APIs | Potential upstream/fork lane, not required for first public MVP |
| Agent-loop rollover | Implement only at proven seam | Hooks, CLI launcher, or app-server | Avoid inventing an orchestration layer before public seams are tested |
| Embedding search | Park | Optional local provider | FTS/exact retrieval must pass without model downloads or additional model/API dependencies |
| GUI dashboard | Park | Future status surface | CLI/JSON can prove v0.1 behavior with much less attack surface |

## 8. Verification gates

These gates govern every roster prompt. A prompt is incomplete when its prescribed gate is missing, flaky, skipped without approval, or contradicted by live evidence.

### G0 — Provenance and reproducibility

- record Codex version, model-catalog hash, OS, toolchain, commit, configuration diff, and exact command;
- generate deterministic fixtures and sanitized machine-readable evidence;
- make any external version or account dependency explicit.

### G1 — Code quality

- formatting, lint/clippy with warnings denied, unit tests, integration tests, and documentation checks pass;
- `git diff --check` passes;
- no unexplained generated files or vendored binaries enter the repository.

### G2 — Native context truth

- `cctx doctor --json` reports exact slug `gpt-5.6-sol`, auth lane, total window, maximum input, maximum output, catalog, configured, effective, compaction threshold, and observed limits independently;
- the active catalog/config resolve to a 1,050,000 total window and at least a 1,000,000 internal Codex budget while respecting Sol's separate 922,000 maximum input;
- a newly started Sol task completes controlled needle probes above 272,000 and near the safe maximum-input operating threshold without compaction;
- the returned model identity is recorded and proves Sol rather than an alias or substitute;
- the final full-scale proof is run only within an approved cost ceiling;
- any non-Sol model, stale catalog, missing long-window policy, or unproven auth route fails closed.

### G3 — Durable 1M integrity

- ingest a deterministic corpus of at least 1,000,000 tokenizer-counted tokens;
- preserve event counts and full-corpus manifest hashes across restart, migration, export, and restore;
- exact ID/hash retrieval is 100 percent;
- pinned and exact-needle queries are 100 percent;
- the published semantic/lexical benchmark target is frozen before measurement and cannot be weakened after a failure.

### G4 — Compaction safety

- force compaction at a deliberately low test threshold;
- prove PreCompact checkpoint completion before the block response;
- prove both automatic and manual compaction attempts do not complete and report the liveness tradeoff;
- prove a proactive successor rollover starts another correctly configured Sol-1M task before the source task reaches the production threshold;
- prove any unexpected PostCompact event is detected as a policy violation;
- preserve canaries from early, middle, and late transcript regions through at least three blocked attempts or successor rollovers.

### G5 — Fresh-task enrollment

- install from a clean user-level test environment;
- start fresh Codex CLI, desktop, and IDE tasks without per-task model/context flags;
- prove exact Sol identity, hook, MCP, store, compaction guard, and native-window status are active automatically;
- repeat resume, clear, compact, and uninstall/rollback paths;
- do not rely on the developer checkout being on PATH.

### G6 — Security and privacy

- restrict store and backup permissions on Windows, Linux, and macOS;
- test path traversal, symlink/junction escape, SQL injection, malformed hook JSON, oversized events, prompt injection in recalled content, and malicious transcripts;
- verify redaction, retention, export, deletion, and no-telemetry defaults;
- run dependency audit, secret scan, license scan, and artifact/SBOM generation;
- document residual risks without euphemism.

### G7 — Performance and liveness

Frozen initial targets, adjustable only by a documented addendum before measurement:

- ordinary append-only hook write p95 under 100 ms on the reference Windows machine;
- bounded auto-recall p95 under 250 ms with a one-million-token reservoir;
- no hook emits more than its configured context budget;
- database recovery after forced process termination loses no committed events;
- strict mode never loops or repeatedly relaunches a stopped turn.

### G8 — Cross-platform release

- Windows x86_64 is live-tested;
- Linux x86_64 and macOS arm64/x86_64 packages build and pass automated install/doctor/uninstall tests;
- release assets carry checksums, SBOM, provenance, and documented signature verification;
- clean-room installation instructions work from GitHub Releases.

### G9 — Publication truth

- README, limitation matrix, demo, release notes, community post, and showcase submission use the same claim contract;
- every numeric claim links to reproducible evidence;
- no community post or showcase submission occurs until the user approves its final text and visuals;
- post-publication corrections are appended transparently.

## 9. Milestones

| Milestone | Usable cut | Prompts | Approval significance |
|---|---|---|---|
| M0 — Foundation | Reproducible repo, truth contract, and capability probe | CAC-00–CAC-04 | Confirms the project can proceed without speculative claims |
| M1 — Sol Native 1M | Automatic Sol-only policy and live native proof | CAC-10–CAC-16 | Core one-million-class Sol window gate |
| M2 — Durable Sol 1M+ | Local lossless reservoir with tested retrieval | CAC-20–CAC-27 | Sol continuity foundation beyond one active window |
| M3 — No-compaction continuity | Checkpoint, strict block, and successor rollover | CAC-30–CAC-36 | Prevents compaction instead of normalizing it |
| M4 — Productized runtime | MCP, plugin, installer, and fresh-task enrollment | CAC-40–CAC-45 | Installable end-to-end product |
| M5 — Release | Cross-platform, security-reviewed, documented v0.1 | CAC-50–CAC-56 | Public GitHub release candidate and release |
| M6 — Community deployment | Approved Codex community and showcase publication | CAC-60–CAC-63 | External publication; separate approval gate |

M1, M2, and M3 are independently reviewable but all are required for the final product claim. M6 is not a substitute for technical acceptance.

## 10. Sequential prompt roster

### Phase 0 — Foundation and evidence freeze

#### **CAC-00 — Bootstrap the canonical repository and governance**

**Objective:** Turn the empty public repository into the authoritative, minimally scaffolded project without implementing product behavior.

**Deliverables:** Clone the canonical repository into the named working folder; create the approved branch; add the PSPR under `PLANNING/`; add `AGENTS.md`, DEVLOG, verification ledger, README claim placeholder, license, contribution guide, security policy, editor settings, and minimal Rust package skeleton.

**Acceptance gate:** Remote identity, branch, file scope, license, plan status, and no-product-code boundary are verified; format/check/test commands run on the skeleton; DEVLOG records the initial commit and push SHA.

#### **CAC-01 — Freeze the Codex capability baseline**

**Objective:** Build a read-only capability probe that captures the installed Codex version, feature set, model catalog, relevant config, hook support, and source references.

**Deliverables:** `cctx probe` or a temporary repo script, sanitized JSON schema, fixtures for current and older/unknown catalogs, and `docs/architecture/CODEX-CAPABILITY-BASELINE.md`.

**Acceptance gate:** The checked-in evidence reproduces the exact discrepancy between Sol's official 1,050,000/922,000/128,000 model limits and Codex's current 272,000/272,000 catalog values, plus the effective-window calculation, current ChatGPT auth lane, and hook/plugin availability, without changing user configuration.

#### **CAC-02 — Freeze the claim contract and terminology**

**Objective:** Convert Sections 2–4 into a public, test-linked contract that code and documentation can share.

**Deliverables:** `docs/architecture/CLAIM-CONTRACT.md`, machine-readable capability vocabulary, limitation table, and wording tests that reject ambiguous native/durable/effective labels.

**Acceptance gate:** Every planned public claim maps to a named gate; total window, maximum input, maximum output, effective budget, operational threshold, and durable capacity cannot be conflated; any non-Sol example fails compliance tests.

#### **CAC-03 — Record architecture and threat decisions**

**Objective:** Freeze the minimum component boundaries, data authority, trust boundaries, and escalation ladder before implementation.

**Deliverables:** ADRs for single-binary architecture, Sol-only model-catalog replacement, ChatGPT-versus-API auth lanes, SQLite/FTS5 reservoir, hook/MCP separation, strict compaction blocking, successor rollover, local-only privacy, and fork/upstream fallback; a threat model and data-flow diagram.

**Acceptance gate:** Each component has one owner, one input/output contract, and a failure mode; no parallel storage or orchestration subsystem is introduced; security review finds no unowned trust transition.

#### **CAC-04 — Establish CI and evidence contracts**

**Objective:** Make every later prompt produce reproducible, reviewable evidence from the beginning.

**Deliverables:** CI for format, clippy, tests, docs, secret scan, dependency/license checks, structural PSPR validation, and evidence schema validation.

**Acceptance gate:** A deliberately failing fixture proves each required job blocks; a clean run passes; no release or deployment credential is required for pull-request CI.

### Phase 1 — Native one-million-token active window

#### **CAC-10 — Implement versioned model-catalog parsing and overlay generation**

**Objective:** Generate an authoritative one-model `gpt-5.6-sol` catalog from the installed Codex catalog while preserving current Sol instructions and capabilities.

**Deliverables:** Typed catalog parser, schema/version guards, overlay generator, deterministic serialization, catalog hash manifest, and fixtures for drift and malformed data.

**Acceptance gate:** Round-trip tests prove that only Sol's context/max/effective/compaction policy fields change; the output contains no fallback model; current instructions and feature flags are byte- or semantic-equivalent as specified; unknown schema or official Sol-limit regression fails closed.

#### **CAC-11 — Implement atomic Codex configuration management**

**Objective:** Apply and reverse only Context Continuum-owned config values safely.

**Deliverables:** Dry-run diff, atomic write, timestamped backup, ownership manifest, conflict detection, restore, and uninstall logic for model, window, catalog path, hook/plugin, and MCP settings.

**Acceptance gate:** Property/integration tests cover missing, partial, commented, concurrent, and user-edited configs; rollback restores exact pre-install bytes when safe and refuses to overwrite later user changes.

#### **CAC-12 — Implement native-window doctor and status output**

**Objective:** Make Sol identity, auth lane, official limits, catalog values, effective budget, operational threshold, compaction guard, and observed values inspectable by humans and automation.

**Deliverables:** `cctx doctor`, `cctx status`, JSON schemas, exit-code contract, and failure guidance.

**Acceptance gate:** Golden tests cover ChatGPT-authenticated Sol at 272k, Sol with the 1.05M replacement catalog, direct-API Sol, a non-Sol override, missing access, stale catalogs, invalid config, and unsupported Codex versions; labels satisfy the claim contract.

#### **CAC-13 — Implement strict Sol-only startup policy**

**Objective:** Ensure every fresh task uses exact `gpt-5.6-sol` with a proven Sol-1M policy or fails closed before ordinary work begins.

**Deliverables:** SessionStart/UserPromptSubmit enforcement, exact model/auth/catalog validation, visible block messages, remediation commands, and per-task override audit.

**Acceptance gate:** A non-Sol model, 272k Sol catalog, stale Sol overlay, or unproven auth lane cannot run a normal prompt without a clear block and remediation; compliant Sol proceeds only when doctor is green.

#### **CAC-14 — Calibrate Sol catalog, input, and no-compaction thresholds**

**Objective:** Freeze a truthful Sol catalog policy with a 1,050,000 total window, at least 1,000,000 internal Codex budget, and operational thresholds below the separate 922,000 maximum input.

**Deliverables:** Offline/client probe harness, deterministic needles, hidden-instruction/tool overhead estimator, configurable output allowance, pre-limit checkpoint threshold, rollover threshold, and frozen catalog policy.

**Acceptance gate:** Codex resolves the intended total and effective values from the replacement catalog; the request/rollover thresholds remain conservatively below 922,000 after measured overhead; auto-compaction cannot occur before the strict hook has checkpointed and blocked; no value ships solely from arithmetic.

#### **CAC-15 — Probe the existing ChatGPT-authenticated Sol route**

**Objective:** Determine whether the user's existing ChatGPT Codex backend accepts Sol requests above the 272,000 product-catalog cap once the client uses the calibrated Sol catalog.

**Deliverables:** Isolated one-off configuration, progressive 300k/600k/near-threshold needle probes, returned model identity, usage/error records, catalog/config hashes, and restoration proof.

**Acceptance gate:** Either the subscription route repeatedly succeeds above 272,000 without compaction and qualifies as a shipping lane, or the exact service-side rejection/limit is captured and CAC-16 becomes mandatory. A service-lane cap is not treated as proof that Sol itself lacks 1.05M.

**Special gate:** Requires explicit approval of the isolated live probe and a maximum ChatGPT-credit/token budget. No permanent user config or auth change is made.

#### **CAC-16 — Prove the direct-API Sol 1.05M shipping lane**

**Objective:** Establish the headline native capability through Codex's supported API-key authentication if the ChatGPT-authenticated route does not meet the gate, and validate it as a second lane if budget permits.

**Deliverables:** Isolated Codex home/auth, secure credential procedure, clean-start Sol transcript/evidence, progressive and near-threshold early/middle/late needles, tokenizer manifest, returned model identity, doctor/status output, catalog/config hashes, repeat-run results, and exact API usage/cost evidence.

**Acceptance gate:** A newly started Codex task uses exact `gpt-5.6-sol`, resolves a 1,050,000 total context window, completes a near-safe-maximum-input needle probe well above 272,000 without compaction, and preserves all canaries. At least one full-scale run and smaller repeat runs must pass, or G2 remains open.

**Special gate:** Requires secure user-provided API authentication and an explicit monetary/token ceiling. Long-context Sol requests above 272,000 receive the documented premium pricing. No public Sol-1M claim before CAC-15 or CAC-16 establishes a shipping lane.

### Phase 2 — Durable local one-million-plus reservoir

#### **CAC-20 — Implement canonical event and provenance schemas**

**Objective:** Represent prompts, replies, tools, artifacts, lifecycle events, pins, and checkpoints without lossy transcript parsing.

**Deliverables:** Versioned Rust/JSON schemas, monotonic sequence contract, content hashes, session/turn IDs, provenance fields, size limits, and migration fixtures.

**Acceptance gate:** Round-trip and malformed-input tests pass; duplicate delivery is idempotent; unknown fields/versions follow a documented compatibility policy; content hashes are stable across platforms.

#### **CAC-21 — Implement the SQLite WAL/FTS5 reservoir**

**Objective:** Store canonical events locally with restart safety, integrity checks, and indexed retrieval.

**Deliverables:** Schema, migrations, WAL settings, transactions, FTS tables, integrity command, backup/restore primitives, and concurrency tests.

**Acceptance gate:** Crash/restart tests preserve committed events; migrations are forward-only and idempotent; integrity and foreign-key checks pass; concurrent hook/MCP access cannot corrupt state.

#### **CAC-22 — Implement reproducible token accounting**

**Objective:** Count stored and injected context consistently enough to make one-million-token claims auditable and safe.

**Deliverables:** GPT-5-family tokenizer integration, versioned tokenizer metadata, conservative fallback, per-event and aggregate counts, and corpus generator.

**Acceptance gate:** Counts match the selected reference tokenizer on frozen fixtures; fallback never understates capacity use; one-million threshold evidence records tokenizer name/version and content hash.

#### **CAC-23 — Implement exact and ranked retrieval**

**Objective:** Retrieve authoritative originals and useful bounded context without mandatory embeddings.

**Deliverables:** Exact fetch, FTS search, timeline reconstruction, filters, pin weighting, deterministic ranking/tie breaks, pagination, and provenance-rich results.

**Acceptance gate:** Exact ID/hash retrieval is 100 percent; frozen early/middle/late needle tests pass; ranking is deterministic; malformed queries, SQL metacharacters, and oversized limits are safe.

#### **CAC-24 — Implement pins, artifacts, and checkpoint manifests**

**Objective:** Give users and agents durable, exact control over high-value facts and files.

**Deliverables:** Pin/unpin semantics, immutable artifact references, checkpoint manifests, transcript snapshot references, and exportable source index.

**Acceptance gate:** Pins survive restart/migration/export/restore; checkpoint verification detects missing, altered, or reordered content; snapshots cannot escape the authorized state directory.

#### **CAC-25 — Implement privacy, retention, redaction, and deletion**

**Objective:** Make local persistence safe for transcripts that may contain credentials or sensitive repository data.

**Deliverables:** Secure directory creation, OS permission checks, configurable redaction, retention policies, scoped export, `forget --dry-run`, deletion audit, and telemetry-off invariant.

**Acceptance gate:** Platform permission tests and malicious-path fixtures pass; known secret fixtures are redacted; deletion/export scopes are exact; no network request occurs during ordinary store/search/recall operations.

#### **CAC-26 — Implement bounded recall capsules**

**Objective:** Convert reservoir results into small, provenance-rich, injection-resistant context suitable for hooks and MCP.

**Deliverables:** Capsule schema, token budgeter, pin/recency/query selection, untrusted-data delimiters, source IDs/hashes, truncation rules, and duplicate suppression.

**Acceptance gate:** Capsules never exceed the configured token/byte budget; malicious stored instructions remain data; exact originals remain fetchable; golden fixtures are deterministic.

#### **CAC-27 — Pass the durable one-million-token benchmark**

**Objective:** Prove that the reservoir, not a mock or compressed approximation, retains and retrieves at least one million tokens.

**Deliverables:** Deterministic public benchmark corpus/generator, ingest report, restart/migration/export/restore report, retrieval metrics, latency distribution, and integrity hashes.

**Acceptance gate:** G3 and G7 pass on the reference Windows machine; exact and pinned queries are perfect; any frozen ranked-retrieval target is met without post-failure relaxation.

### Phase 3 — Compaction prevention and continuity

#### **CAC-30 — Implement documented hook transport and lifecycle adapters**

**Objective:** Connect Codex hook JSON to the canonical event pipeline without depending on unstable transcript text.

**Deliverables:** Hook dispatcher for SessionStart, UserPromptSubmit, PostToolUse, Stop, PreCompact, and PostCompact; bounded stdin/stdout handling; timeouts; structured errors; test fixtures from official schemas.

**Acceptance gate:** Each event maps losslessly to canonical fields; malformed/oversized input fails safely; stdout remains valid hook protocol; normal event latency meets G7.

#### **CAC-31 — Implement the PreCompact checkpoint protocol**

**Objective:** Make it impossible for Context Continuum to knowingly allow compaction before durable state verification.

**Deliverables:** Transaction barrier, WAL checkpoint, transcript snapshot, manifest hash, fsync policy, retry ceiling, and mandatory block decision record.

**Acceptance gate:** Fault injection at every write step proves compaction is not allowed after a partial checkpoint; successful checkpoints verify independently after process restart.

#### **CAC-32 — Implement strict no-compaction mode**

**Objective:** Prevent automatic and manual compaction literally; preservation is the product default and release contract.

**Deliverables:** `continue: false` PreCompact decision, visible diagnostic, checkpoint ID, recovery/rollover command, repeated-trigger suppression, and audit event.

**Acceptance gate:** Forced integration test proves compaction does not complete; the active turn stops once, not in a loop; all pre-stop context is retrievable; documentation states the hard-limit consequence plainly.

#### **CAC-33 — Detect prohibited compaction and recover as an incident**

**Objective:** Detect any PostCompact event as a policy violation and preserve enough evidence/state for emergency recovery without treating compaction as normal operation.

**Deliverables:** PostCompact violation record, SessionStart source=`compact` detector, checkpoint verifier, high-visibility alarm, emergency recovery capsule, exact-source fetch instructions, and diagnostic bundle.

**Acceptance gate:** A test-only bypass that produces PostCompact is detected immediately, the task is marked noncompliant, early/middle/late canaries remain retrievable from the last checkpoint, and normal continuation does not silently resume.

#### **CAC-34 — Implement proactive threshold warning and checkpointing**

**Objective:** Warn and checkpoint before the hard limit so long agentic loops do not discover strict-mode liveness loss only at compaction time.

**Deliverables:** Conservative token-usage estimator, configurable thresholds, one-shot warning state, automatic pre-limit checkpoint, and user/agent remediation commands.

**Acceptance gate:** Boundary fixtures show no warning spam, undercount, or recursive hook injection; a long synthetic task checkpoints before the configured guard threshold.

#### **CAC-35 — Prototype compaction-free successor rollover**

**Objective:** Determine whether supported Codex CLI/app-server seams can continue an agentic loop in a successor task without compacting the predecessor.

**Deliverables:** Time-boxed spike, thread/working-directory/config continuity contract, checkpoint handoff, successor recovery capsule, and go/no-go ADR.

**Acceptance gate:** Either a live successor continues from verified state with exact Sol-1M policy and no compaction of the source task, or the ADR documents the exact public-seam blocker and routes implementation to CAC-X1 without weakening CAC-32/CAC-33.

**Scope rule:** This prompt may not grow into a replacement agent orchestrator. Any unsupported fork work moves to CAC-X1.

#### **CAC-36 — Pass compaction and long-loop acceptance**

**Objective:** Validate strict prevention, proactive checkpointing, prohibited-compaction detection, and the approved successor-rollover path in realistic Sol tasks.

**Deliverables:** Forced-low-threshold suite, long-running tool loop, restart/resume/clear cases, canary matrix, failure injection, and sanitized live evidence.

**Acceptance gate:** G4 and applicable G7 requirements pass; no silent loss is observed; unsupported lifecycle cases are visible and release-blocking unless explicitly excluded by addendum.

### Phase 4 — MCP, plugin, installation, and automatic enrollment

#### **CAC-40 — Implement the local MCP server**

**Objective:** Expose the minimal reservoir operations to Codex through a stable, bounded MCP interface.

**Deliverables:** The eight tools in Section 6.2, JSON schemas, pagination, timeouts, authorization boundaries, structured errors, and MCP protocol tests.

**Acceptance gate:** Tool schemas and responses validate; queries cannot escape the current authorized store/scope; prompt-injection fixtures remain untrusted data; load tests meet latency limits.

#### **CAC-41 — Scaffold and validate the Codex plugin**

**Objective:** Package hooks, MCP configuration, metadata, and usage instructions in the supported Codex plugin format.

**Deliverables:** Officially scaffolded plugin, validated manifest, default-discovered `hooks/hooks.json`, `.mcp.json`, icons/README where supported, and repository-hosted marketplace metadata if the current plugin specification supports it.

**Acceptance gate:** Official plugin validation passes; the package uses `PLUGIN_ROOT`/`PLUGIN_DATA` correctly; Windows and POSIX hook commands resolve the installed `cctx`; no unsupported manifest field is invented.

#### **CAC-42 — Implement safe install, update, rollback, and uninstall**

**Objective:** Make the full runtime installable without manual file surgery.

**Deliverables:** `cctx install`, `update`, `rollback`, and `uninstall`; dry runs; platform paths; version manifest; plugin trust guidance; binary/catalog/config ownership records.

**Acceptance gate:** Clean-room tests install and remove all owned artifacts, preserve user changes, recover from interrupted updates, and leave no hidden state except explicitly retained user data.

#### **CAC-43 — Implement secure cross-platform state handling**

**Objective:** Make paths, locks, permissions, process invocation, and plugin data correct on Windows, Linux, and macOS.

**Deliverables:** Platform abstraction, Windows ACL checks, Unix modes, junction/symlink defenses, lock/retry policy, path canonicalization, and portable database tests.

**Acceptance gate:** CI and live Windows tests cover paths with spaces/non-ASCII text, concurrent Codex processes, junction/symlink attacks, read-only failures, and uninstall while data is retained.

#### **CAC-44 — Prove automatic enrollment in every fresh task**

**Objective:** Demonstrate that installation affects newly started Codex tasks without per-task flags or developer checkout assumptions.

**Deliverables:** Isolated test-user installation, fresh CLI/desktop/IDE tasks in multiple repositories, ChatGPT-lane and direct-API-lane cases as available, deliberate non-Sol/stale-catalog blocks, hook/MCP traces, and rollback proof.

**Acceptance gate:** G5 passes; the first ordinary prompt has exact Sol identity, Sol-1M status, and active lifecycle capture; any non-Sol or 272k/stale configuration is visibly blocked before ordinary task work.

#### **CAC-45 — Dogfood the complete runtime locally**

**Objective:** Use the installed product in real development sessions long enough to surface lifecycle and usability defects before release hardening.

**Deliverables:** Dogfood protocol, sanitized session evidence, issue ledger, performance samples, recovery drills, and resolved/blocking issue list.

**Acceptance gate:** At least three fresh tasks and one long-loop task complete the protocol; no severity-high data-loss or security issue remains open; unresolved limitations are explicit release blockers or documented exclusions.

**Special gate:** Requires explicit approval immediately before changing the user's real global Codex installation.

### Phase 5 — Hardening and open-source release

#### **CAC-50 — Complete the adversarial security test suite**

**Objective:** Validate the product's highest-risk surfaces before distributing an installer that reads Codex transcripts.

**Deliverables:** Threat-model traceability, property/fuzz tests for hook/MCP/config parsers, path and permission attacks, recalled-content injection cases, secret fixtures, and dependency/license reports.

**Acceptance gate:** G6 passes; every high-risk threat has a test or a documented manual proof; no unresolved critical/high finding remains.

#### **CAC-51 — Complete the cross-platform and version matrix**

**Objective:** Establish the exact Codex/OS versions the release supports and how it fails outside them.

**Deliverables:** Matrix CI, compatibility fixtures for catalog/config drift, Windows live report, Linux/macOS install smoke evidence, and support policy.

**Acceptance gate:** G8 platform requirements pass; unknown future Codex schema fails closed with an upgrade diagnostic; no platform is labeled supported from compilation alone.

#### **CAC-52 — Build reproducible release artifacts and supply-chain evidence**

**Objective:** Produce verifiable binaries and plugin assets from tagged source.

**Deliverables:** Release workflow, locked dependencies, reproducible-build notes, checksums, SBOM, provenance attestations, signing plan, and archive contents tests.

**Acceptance gate:** A release candidate builds from a clean tag; archive inspection contains only intended assets; checksum/SBOM verification works from published instructions; no secret enters logs or artifacts.

#### **CAC-53 — Write user, operator, and contributor documentation**

**Objective:** Make installation, modes, limits, recovery, privacy, verification, and development understandable without marketing ambiguity.

**Deliverables:** README, quickstart, architecture, native-vs-durable matrix, strict-mode warning, privacy/security guide, troubleshooting, uninstall, benchmark reproduction, contributor guide, and FAQ.

**Acceptance gate:** Clean-room documentation test completes install/doctor/sample recall/uninstall; every headline claim links to evidence; the current Codex Sol 272k cap, official Sol 1.05M/922k/128k limits, authentication lanes, cost implications, and strict no-compaction behavior are explicit.

#### **CAC-54 — Produce the public demo and evidence bundle**

**Objective:** Show the product working without exposing private transcripts, credentials, or unverifiable theatrics.

**Deliverables:** Deterministic synthetic demo repository/corpus, terminal recording or screenshots, native 1M proof summary, compaction guard demo, durable retrieval demo, and reproducibility links.

**Acceptance gate:** Demo can be rerun from tagged source; all displayed numbers match machine-readable evidence; privacy review passes; no edited sequence conceals a failed gate.

#### **CAC-55 — Cut and verify the v0.1.0 release candidate**

**Objective:** Freeze the first publishable product and run all release gates against the exact candidate commit.

**Deliverables:** Version bump, changelog, release notes draft, full verification report, artifact hashes, known limitations, and rollback rehearsal.

**Acceptance gate:** G0–G8 pass on the candidate SHA; repository status is clean; CI is green; remote SHA matches the DEVLOG; the user receives the candidate evidence for release approval.

#### **CAC-56 — Publish v0.1.0 on GitHub**

**Objective:** Make the verified source, plugin, binaries, checksums, SBOM, provenance, and documentation publicly installable.

**Deliverables:** Approved merge/tag, GitHub Release, release assets, repository topics/description, issue templates, and post-release clean-install verification.

**Acceptance gate:** Public release installation succeeds from the release URL on a clean environment; assets match candidate hashes; main/tag/DEVLOG remote SHAs are reconciled; no draft claim exceeds evidence.

**Special gate:** Requires explicit user approval of the release candidate and release notes before tag/merge/release publication.

### Phase 6 — Codex developer community deployment

#### **CAC-60 — Draft the Codex community launch package**

**Objective:** Prepare a technical, evidence-backed launch for the official Codex community category.

**Deliverables:** Post title/body, concise architecture explanation, native/durable/strict limitation matrix, install command, demo media, GitHub link, evidence links, support boundaries, and feedback questions.

**Acceptance gate:** G9 content review passes; every claim matches v0.1.0 evidence; the draft avoids spam, hype, and OpenAI endorsement implications.

#### **CAC-61 — Publish to the Codex Developer Community board**

**Objective:** Publish the approved launch package to the official Codex category and verify its public rendering.

**Deliverables:** Final post, public URL, screenshot, timestamp, and DEVLOG/publication record.

**Acceptance gate:** The post is publicly accessible, links and media work, claim wording matches the approved draft, and any moderator requirement is recorded.

**Special gate:** External write. Requires the user's explicit approval of the final post immediately before publication.

#### **CAC-62 — Submit the OpenAI developer showcase package**

**Objective:** Submit the verified open-source project through the official developer showcase path if it meets current eligibility.

**Deliverables:** Approved showcase copy, screenshots/demo, repository/release links, privacy-safe submission data, confirmation, and status record.

**Acceptance gate:** Submission confirmation is captured and all submitted claims match v0.1.0 evidence. Acceptance by OpenAI is not under project control and is not required to call the submission complete.

**Special gate:** External write. Requires the user's explicit approval of the exact submission immediately before sending.

#### **CAC-63 — Establish transparent feedback and maintenance triage**

**Objective:** Convert real user feedback into a bounded, auditable post-release queue without silently expanding v0.1 scope.

**Deliverables:** Labeled GitHub issues, support template requiring doctor/evidence output, security-report path, compatibility dashboard, and proposed v0.1.x/v0.2 addenda.

**Acceptance gate:** Every received high-signal report is reproducible, classified, or explicitly awaiting information; fixes enter a new approved roster/addendum rather than bypassing governance.

### Optional escalation lane — only if a core public seam fails

#### **CAC-X1 — Prototype the open-source Codex in-process extension seam**

**Trigger:** CAC-14, CAC-30, or CAC-35 proves that supported config/hooks/app-server interfaces cannot meet a required gate.

**Objective:** Test a minimal, reproducible Codex fork using the existing context/turn contributor seams without adding unrelated product changes.

**Acceptance gate:** The fork demonstrably closes the named failed gate, rebases cleanly, carries an explicit maintenance cost, and does not pretend to bypass backend hard limits.

#### **CAC-X2 — Prepare an upstream Codex proposal or pull request**

**Trigger:** CAC-X1 succeeds and the change is generalizable.

**Objective:** Offer the smallest supported upstream capability needed for native effective-window control, lifecycle state, or successor rollover.

**Acceptance gate:** Proposal/PR includes tests, threat/compatibility analysis, and neutral motivation; external submission requires user approval and upstream acceptance is not a completion dependency.

#### **CAC-X3 — Prototype a Sol-only Responses transport shim**

**Trigger:** The official direct-API Sol endpoint meets its documented limits, but the built-in Codex provider/auth path cannot carry the calibrated long-context request.

**Objective:** Build the smallest local Codex-compatible transport that forwards requests only to official `gpt-5.6-sol` Responses API while preserving Codex tools, streaming, usage, and errors.

**Acceptance gate:** The shim cannot route to any other model or provider, adds no content truncation/compaction, preserves request and response semantics under integration tests, and closes only the demonstrated transport gap. Credential and network security review is mandatory.

## 11. Risk register and response rules

| Risk | Detection | Required response |
|---|---|---|
| ChatGPT Codex service keeps Sol at 272k | CAC-15 progressive probe | Preserve evidence and continue to official direct-API Sol lane at CAC-16 |
| Direct-API Sol cannot be carried by stock Codex | CAC-16 exact request/error | Enter narrow fork or Sol-only transport-shim lane; keep G2 open |
| Total window is confused with max input | Claim tests and docs review | Report 1.05M total, 922k max input, and 128k max output together |
| Custom catalog drifts with Codex update | Hash/schema mismatch | Fail closed, restore supported catalog, issue actionable upgrade path |
| Strict compaction blocking halts work | Forced limit and live loop | Preserve checkpoint, stop once, surface rollover/recovery; never auto-loop |
| Prohibited compaction still occurs | Unexpected PostCompact event | Mark task noncompliant, preserve evidence, treat as release-critical defect |
| Hook output/context limits truncate recall | Token/byte budget tests | Keep capsules bounded and use MCP exact fetch |
| Transcript format changes | Parser/version failure | Preserve raw snapshot, use structured hook fields, fail optional parsing safely |
| Reservoir captures secrets | Secret fixtures/user report | Redact, restrict permissions, document retention/delete, no cloud default |
| Recalled text injects instructions | Adversarial recall tests | Mark as untrusted data, preserve provenance, never elevate authority |
| Installer damages user config | Byte-diff/rollback test | Atomic backup, ownership manifest, refuse conflicting rollback |
| Million-token live test is costly | Preflight token/cost estimate | Require explicit ceiling and abort before overrun |
| API credential leaks into logs/store | Secret fixtures and artifact scan | Never persist the key in repo/reservoir/evidence; rotate immediately on exposure |
| Community language outruns proof | G9 review | Block publication until wording matches evidence |
| Codex version changes mid-project | Probe/catalog hash drift | Re-freeze baseline through a PSPR addendum before continuing affected prompts |

Repeated technical difficulty is not permission to weaken a gate. A failed gate produces evidence, a narrow diagnosis, and either a prompt-local fix or an approved addendum. Environment/account failures are recorded separately from product failures.

## 12. Definition of done

The one goal is complete only when all of the following are true on the same released version:

1. a clean install automatically enrolls every fresh local Codex task in exact `gpt-5.6-sol` identity, lifecycle capture, context status, and no-compaction policy;
2. at least one supported Codex authentication lane live-proves Sol's 1,050,000-token total window with at least a 1,000,000 internal Codex budget, while separately and truthfully reporting the 922,000 maximum input and 128,000 maximum output;
3. a live near-threshold Sol probe runs far above Codex's ordinary 272,000 cap without compaction, with returned model identity and canary evidence;
4. every non-Sol, stale-catalog, 272k-capped, or unproven session is blocked before ordinary work and can never be labeled compliant;
5. every compliant Sol task receives a local reservoir proven to retain at least one million tokens with integrity and exact retrieval;
6. every known automatic or manual compaction attempt is checkpointed and visibly blocked; longer loops continue through a verified fresh Sol successor rather than an allowed compaction;
7. install, update, rollback, uninstall, security, privacy, performance, and cross-platform gates pass;
8. the exact verified source and artifacts are published under an approved open-source license in `USS-Parks/1M-Context-Sol`;
9. the release is installed successfully from GitHub rather than the developer checkout;
10. an approved technical launch is published to the official Codex community category;
11. the approved OpenAI developer showcase submission is sent if eligible; and
12. PSPR, DEVLOG, verification ledger, release tag, remote SHA, and public claims all agree.

An external showcase acceptance decision is outside project control. Submission is required; acceptance is not.

## 13. Approval options

The current state is **APPROVED — FULL STS EXECUTION ACTIVE — GPT-5.6 SOL ONLY**.

The approval options below are retained as historical governance text. The user selected full STS execution on 2026-07-19.

The user may choose one of these next instructions:

- **`Approve the PSPR and run it STS.`** Begin CAC-00 and proceed sequentially, stopping at named special gates.
- **`Approve M0 only and run it STS.`** Execute CAC-00 through CAC-04, then stop for review.
- **`Revise the PSPR: ...`** Change the plan only; do not implement.
- **`Reject the PSPR.`** Leave the repository and user environment untouched.

No option is inferred from silence.

## 14. Planning references

- OpenAI GPT-5.6 Sol model: <https://developers.openai.com/api/docs/models/gpt-5.6-sol>
- OpenAI GPT-5.6 guidance: <https://developers.openai.com/api/docs/guides/model-guidance?model=gpt-5.6>
- Codex authentication: <https://learn.chatgpt.com/docs/auth>
- Codex changelog: <https://learn.chatgpt.com/docs/changelog>
- Codex 0.144 Sol context-catalog corrections: <https://github.com/openai/codex/pull/33972> and <https://github.com/openai/codex/pull/34009>
- Codex configuration reference: <https://learn.chatgpt.com/docs/config-file/config-reference>
- Codex hooks: <https://learn.chatgpt.com/docs/hooks>
- Codex plugin building: <https://learn.chatgpt.com/docs/build-plugins>
- Open-source Codex repository: <https://github.com/openai/codex>
- Target repository: <https://github.com/USS-Parks/1M-Context-Sol>
- Codex community category: <https://community.openai.com/c/codex/37>
- OpenAI developer community and showcase: <https://developers.openai.com/community>
