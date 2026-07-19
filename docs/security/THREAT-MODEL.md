# Context Continuum Repository Threat Model

## Overview

Context Continuum is a planned user-local Codex runtime extension for exact `gpt-5.6-sol`. It combines a Rust `cctx` executable, Codex plugin hooks, a local MCP server, an atomic installer/model-catalog controller, a strict compaction guard, successor rollover, and a SQLite WAL/FTS5 context reservoir.

The current repository implements the read-only capability probe and machine-enforced claim contract. Later PSPR prompts add the privileged runtime surfaces. This threat model covers the whole intended repository product, not only the code already implemented.

Primary assets and privileges are:

- Codex prompts, replies, tool inputs/results, source code, paths, and transcript snapshots;
- OpenAI/ChatGPT authentication state, which must remain exclusively under Codex control;
- global user-level Codex config, plugin, hook, MCP, and model-catalog settings;
- the local reservoir, backups, exports, pins, checkpoint manifests, and deletion records;
- model identity and native/effective/durable capacity evidence used for public claims;
- the ability to block compaction or launch a fresh successor task;
- release binaries, plugin assets, checksums, SBOM, provenance, and update metadata.

The core security objectives are confidentiality of local context and credentials; integrity and ordering of durable events/checkpoints; exact Sol-only model identity; atomic/reversible config mutation; untrusted treatment of recalled text; literal compaction blocking; scoped MCP access; and evidence-backed public claims.

## Threat Model, Trust Boundaries, and Assumptions

### Actors and input control

- **Operator-controlled:** installation, login through Codex, config approval, pins, queries, export/delete scope, retention policy, and paid-probe ceiling.
- **Repository or content attacker-controlled:** checked-out source, filenames, symlinks/junctions, tool output, transcripts, stored historical text, MCP arguments originating from the model, and malformed/oversized hook frames.
- **Developer/release-controlled:** source, dependencies, CI workflows, schemas, catalog generation, signing/provenance, and release archives.
- **External service-controlled:** Codex catalog refresh, hook/plugin schemas, model response/usage metadata, account access, and backend limits.

### Trust boundaries

The normative transition ledger is `contracts/architecture-boundaries.json` and the readable flow is `docs/architecture/DATA-FLOW-AND-TRUST-BOUNDARIES.md`. The highest-risk boundaries are:

1. operator input into privileged installer/config mutation;
2. credential entry into Codex authentication, which `cctx` must not cross;
3. Codex hook JSON and workspace/tool content into lifecycle capture;
4. validated events into the sole SQLite durable authority;
5. stored historical text back into MCP/model-visible context;
6. model-originated MCP requests into store/search/export/delete operations;
7. filesystem paths into config, catalog, database, backup, export, and transcript access;
8. PreCompact events into checkpoint and block decisions;
9. source-task state into a fresh successor; and
10. release/update artifacts into executable code on the user's machine.

### Assumptions and invariants

- The operating system, current user account, Codex binary, and official OpenAI endpoint are not already compromised.
- Same-user malware can generally read the user's context and is outside the protection `cctx` alone can provide.
- Codex remains the sole reasoning, tool, permission, and task-lifecycle authority.
- Codex authentication alone sees raw credentials; `cctx` sees only sanitized status.
- Ordinary persistence and retrieval are local-only and telemetry-off.
- The reservoir is the sole mutable durable context authority.
- Stored/recalled text is untrusted data and never acquires instruction authority.
- Unknown schemas, catalog drift, non-Sol identity, ambiguous auth, or unverifiable capacity fail closed.
- No client change can bypass server entitlements or establish native capacity without a live response.
- A partial checkpoint can never authorize compaction or rollover.
- Public claim status is controlled by `contracts/capability-vocabulary.json` and passing evidence, not marketing prose.

Out of scope as direct product guarantees are a compromised OS/kernel, malicious administrator, physical memory attacks, denial of service by a provider outage, OpenAI account policy decisions, and cryptographic erasure guarantees for storage media outside product-owned enumerated files. These do not excuse preventable path, permission, secret-handling, or integrity defects.

## Attack Surface, Mitigations, and Attacker Stories

| Surface / attacker story | Impact | Required mitigation and evidence |
|---|---|---|
| Malicious repository uses path traversal, a symlink, or Windows junction to redirect config/store/export access | overwrite or disclose files outside product scope | canonical paths, no-follow/reparse checks, ownership manifest, ACL/mode checks, atomic writes, adversarial platform fixtures |
| Hook sends malformed, deeply nested, or oversized JSON or stalls stdin/stdout | crash, memory exhaustion, Codex hang, lost lifecycle event | bounded reads, schema/version checks, depth/size ceilings, timeout, protocol-valid errors, fuzz/property tests |
| Tool output or recalled transcript contains instructions such as credential requests or destructive commands | prompt injection and unintended agent actions | untrusted delimiters, provenance, exact-source IDs, bounded recall, no authority elevation, adversarial recall tests |
| MCP request escapes session/store scope or injects SQL/FTS syntax | cross-session disclosure, deletion, corruption, denial of service | canonical namespace authorization, prepared statements, allowlisted sort/filter fields, result/timeout limits, dry-run deletion |
| Config update races user edits or follows an attacker-controlled path | corrupt global Codex config or persist malicious hooks | byte precondition, temp file plus atomic replace, backup, ownership manifest, conflict refusal, rollback tests |
| Catalog overlay contains another model, stale instructions, or inflated unproved values | false compliance, wrong model, invalid public claim | exact slug invariant, one-model output, source/overlay hashes, schema guard, doctor and returned-model evidence |
| `cctx` logs, stores, or exports an API key or auth header | account compromise and billable abuse | credential exclusion by design, redaction before persistence, secret fixtures/scans, no raw auth output |
| Database crash, migration, or concurrent hook/MCP access loses or reorders events | silent context loss or false checkpoint | transactions, WAL, monotonic sequence, content hashes, foreign keys, integrity checks, crash/restart/concurrency tests |
| PreCompact checkpoint is partial but guard returns success, or PostCompact is ignored | irreversible context loss | mandatory verified barrier, `continue=false` default, fault injection at each write, PostCompact incident alarm |
| Rollover capsule is tampered, stale, oversized, or replayed into the wrong workspace | contaminated successor or side-effect replay | checkpoint/task/workspace hashes, expiry/freshness, bounded data-only capsule, doctor preflight, no automatic tool replay |
| Malicious release archive, dependency, workflow, or updater executes code | developer/user machine compromise | locked dependencies, least-privilege CI, secret isolation, archive allowlist, checksums, SBOM, provenance/signing plan, clean-tag rebuild |
| Local unprivileged user reads a permissive reservoir or backup | transcript/source disclosure | per-user state path, restrictive ACL/mode validation, backup/export permission parity, documented same-user limits |
| High-volume prompts/tool results fill disk or create quadratic retrieval | availability loss | per-event/store quotas, backpressure, bounded FTS queries, retention, predictable failure without compaction |

Repository-context vulnerability classes with the highest relevance are path traversal/reparse-point abuse, command or argument injection in hook/plugin launchers, SQL/FTS injection, secret leakage, prompt injection through recalled content, authorization/scope errors in MCP, unsafe config replacement, deserialization/resource exhaustion, checkpoint integrity errors, and supply-chain compromise.

Web-specific CSRF, browser XSS, and multi-tenant object authorization are not primary surfaces because v0.1 has no web server or cloud tenancy. They become in scope if a future addendum adds either.

## Severity Calibration (Critical, High, Medium, Low)

### Critical

- Arbitrary code execution through a hook/plugin command, update archive, or path-controlled executable resolution in the normal install/runtime flow.
- Exfiltration of usable OpenAI credentials to an attacker-controlled destination.
- A release-pipeline compromise that publishes an attacker-controlled signed/verified binary.

### High

- Silent compaction or checkpoint logic that irreversibly loses context while reporting preservation.
- Path escape that overwrites user Codex config or reads/writes sensitive files outside product-owned state.
- Cross-session or out-of-scope MCP export/delete of private transcript data.
- Model/capacity spoofing that enables the public native-1M claim with a non-Sol or unproved backend.
- Recalled prompt injection that crosses the untrusted-data boundary and reliably causes privileged tool execution.

### Medium

- Malformed hook/MCP input that crashes or persistently blocks Codex without code execution or data disclosure.
- Redaction or retention failure affecting bounded non-credential sensitive data with an available recovery/deletion path.
- Database/query amplification that exhausts local resources within the user's own session.
- Successor verification failure that visibly blocks work but preserves the source checkpoint.

### Low

- Incorrect diagnostic wording that does not enable compliance or expose data.
- Local performance regression below a published target without integrity, confidentiality, or unbounded-availability impact.
- A documentation or uninstall residue issue limited to non-sensitive metadata and clearly reported owned files.

Severity moves upward when exploitation is reachable from ordinary repository content, a model-generated MCP call, a default install/update, or an unprivileged local actor, and downward when it requires a compromised administrator/OS already capable of the same outcome.

Repository: target_sha256_fd57868a15df2bb7a521c39ba3326e0121373a048d37a819508d58c426f474b7
Version: f21a702ea33b0b7c6adbbd05b830ec54b72a3699
