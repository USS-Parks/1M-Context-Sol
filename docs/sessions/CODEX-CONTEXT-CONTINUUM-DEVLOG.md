# Context Continuum Development Log

This append-only ledger records execution of `CAC-PSPR-2`. A prompt is complete only when its acceptance gate passes.

## CAC-00 — Bootstrap the canonical repository and governance

- **Date:** 2026-07-19
- **Status:** Complete
- **Authorization:** Full STS execution, commits, and pushes authorized by the user.
- **Remote:** `https://github.com/USS-Parks/Codex-Added-Context`
- **Branch:** `codex/context-continuum-v0.1`
- **Scope:** Governance, documentation placeholders, Apache-2.0 license, editor settings, and minimal Rust skeleton only.
- **Product behavior:** None implemented.
- **Files changed:** `.editorconfig`, `.gitattributes`, `.gitignore`, `AGENTS.md`, `Cargo.toml`, `Cargo.lock`, `CONTRIBUTING.md`, `LICENSE`, `PLANNING/CODEX-CONTEXT-CONTINUUM-PSPR.md`, `README.md`, `SECURITY.md`, `docs/VERIFICATION.md`, this DEVLOG, and the `src/` skeleton.
- **Verification:**
  - `cargo fmt --all -- --check` — passed
  - `cargo check --all-targets` — passed
  - `cargo clippy --all-targets -- -D warnings` — passed
  - `cargo test --all-targets` — passed; 1 test
  - `git diff --check` — passed
  - PSPR validator — 47 prompts, 47 unique IDs, approved STS status
  - Remote probe — `USS-Parks/Codex-Added-Context` exists, public, empty at bootstrap, default branch `main`
  - Toolchain — Git 2.54.0.windows.1; Cargo/Rust 1.96.1
- **Implementation commit:** `f46254ca0967d0a9f04bfc9e900b525390432619`
- **Published remote SHA:** `f46254ca0967d0a9f04bfc9e900b525390432619` on both `main` and `codex/context-continuum-v0.1`

## CAC-01 — Freeze the Codex capability baseline

- **Date:** 2026-07-19
- **Status:** Complete
- **Scope:** Read-only `cctx probe`, sanitized schema/evidence, catalog compatibility fixtures, baseline document, and explicit bundled-versus-resolved PSPR correction.
- **Observed runtime:** `codex-cli 0.144.5`; ChatGPT-authenticated; exact configured model `gpt-5.6-sol`; hooks/plugins stable and enabled.
- **Observed catalogs:** Bundled Sol 372,000 / 372,000 with 353,400 effective; resolved Sol 272,000 / 272,000 with 258,400 effective.
- **Official Sol limits:** 1,050,000 total; 922,000 maximum input; 128,000 maximum output.
- **Upstream freeze:** Codex `main` `c86b1be3cdbe12307843bcc9e7a44c1904ddcdf1`; model-catalog blob `a43af2a54ed82719b011f6e8498f9028f340a5ce`.
- **Safety:** No config/auth mutation and no model request.
- **Files changed:** `Cargo.toml`/`Cargo.lock`, `src/probe.rs`, CLI/library wiring, four catalog fixtures, evidence integration tests, capability JSON schema, sanitized evidence, source freeze, capability baseline, README, PSPR correction, and verification ledgers.
- **Verification:**
  - `cargo fmt --all -- --check` — passed
  - `cargo check --all-targets` — passed
  - `cargo clippy --all-targets -- -D warnings` — passed
  - `cargo test --all-targets` — passed; 11 tests
  - `cargo run --quiet -- probe` — passed twice against the installed Codex runtime
  - Codex config SHA-256 before/after probe — unchanged at `6F274971BD736B79CDEE52DA94A584134217528420C2CDBFEBCAD6F5D5CB0BDA`
  - Live report — resolved 272,000 / 258,400 effective; bundled 372,000 / 353,400 effective; no model request
  - Official model source — exact `gpt-5.6-sol` at 1,050,000 total / 922,000 max input / 128,000 max output
  - Current upstream source freeze — Sol 272,000 / 272,000 at the recorded commit/blob
  - `git diff --check` — passed
- **Implementation commit:** `8ff3cd7d0e8ed98f09dc095832cc9b6848602759`
- **Published remote SHA:** `8ff3cd7d0e8ed98f09dc095832cc9b6848602759` on `codex/context-continuum-v0.1`

## CAC-02 — Freeze the claim contract and terminology

- **Date:** 2026-07-19
- **Status:** Complete
- **Scope:** Normative claim document, machine-readable vocabulary/schema, typed validator, limitation table, README linkage, and negative wording/model/dimension tests.
- **Contract:** Exact `gpt-5.6-sol` only; aliases and fallback forbidden; all eleven public claims disabled until their named gates pass.
- **Dimensions:** Native total, maximum input, maximum output, effective Codex budget, operational input threshold, and durable reservoir capacity are separate and non-substitutable.
- **Files changed:** `contracts/capability-vocabulary.json`, `schemas/capability-vocabulary.schema.json`, `docs/architecture/CLAIM-CONTRACT.md`, `src/claim_contract.rs`, `tests/claim_contract.rs`, README, and ledgers.
- **Verification:**
  - `cargo fmt --all -- --check` — passed
  - `cargo check --all-targets` — passed
  - `cargo clippy --locked --all-targets -- -D warnings` — passed
  - `cargo test --locked --all-targets` — passed; 22 tests total, including 11 claim-contract tests
  - Six dimension IDs exist exactly once and have empty substitution sets
  - Eleven public claims map to named G0–G9 gates and are disabled before gate completion
  - Non-Sol model evidence, conflated totals, low effective/durable capacity, unsafe threshold, missing G4, and ambiguous wording all fail
  - Checked-in contract and schema parse as JSON; README wording passes the validator
- **Implementation commit:** `d21716c34bda05e80f9d328c7814c77cf626be4b`
- **Published remote SHA:** `d21716c34bda05e80f9d328c7814c77cf626be4b` on `codex/context-continuum-v0.1`

## CAC-03 — Record architecture and threat decisions

- **Date:** 2026-07-19
- **Status:** Complete
- **Scope:** Nine accepted ADRs, sole-authority/component contract, twenty trust transitions, Mermaid data flow, repository-scoped threat model, README/security linkage, and structural tests.
- **Authority result:** Codex retains the agent loop and credentials; `cctx` has one named owner for install, model policy, lifecycle capture, durable context, recall, compaction, rollover, MCP, and compliance verdicts.
- **Excluded parallel systems:** No replacement agent orchestrator, second durable store, cloud transcript backend, or alternate-model router.
- **Threat model:** Repository scope; generated under Codex Security threat-model guidance; cached under `C:\tmp\codex-security-scans\Codex 1M Context Project\threat_model.md` with target/version footer.
- **Files changed:** `contracts/architecture-boundaries.json`, nine `docs/architecture/decisions/ADR-*` files, data-flow/trust-boundary document, `docs/security/THREAT-MODEL.md`, `tests/architecture_boundaries.rs`, README, SECURITY, and ledgers.
- **Verification:**
  - `cargo fmt --all -- --check` — passed
  - `cargo check --locked --all-targets` — passed
  - `cargo clippy --locked --all-targets -- -D warnings` — passed
  - `cargo test --locked --all-targets` — passed; 28 tests total, including 6 architecture-boundary tests
  - 11 unique components each have owner, inputs, outputs, and failure mode
  - 6 authorities resolve to existing sole owners
  - 20 trust transitions have owned endpoints, validation controls, and fail-closed outcomes
  - all 9 ADRs have accepted status and normalized contract fields
  - threat model contains all required repository-scope sections and exact cache footer
- **Implementation commit:** `907254aca8a7228d5af7561004cb220921590362`
- **Published remote SHA:** `907254aca8a7228d5af7561004cb220921590362` on `codex/context-continuum-v0.1`

## CAC-04 — Establish CI and evidence contracts

- **Date:** 2026-07-19
- **Status:** Complete
- **Scope:** Read-only CI workflow, pinned toolchain/actions, format/check/Clippy/tests/docs, full-history secret scan, cargo-deny policy, structural PSPR validation, Draft 2020-12 evidence schemas, complete evidence mapping, and negative gate proofs.
- **Credentials:** Workflow permissions are `contents: read`. No release, deployment, OpenAI, or user-managed GitHub credential is required.
- **Pinned tools:** checkout v7.0.0 SHA, rust-toolchain action SHA with Rust 1.96.1, cargo-deny action v2.1.1 SHA, Gitleaks 8.30.1 archive SHA-256, and `jsonschema` 0.48.1 with network/file resolution disabled.
- **Files changed:** CI workflow, `deny.toml`, two CI scripts, governance/schema tests, source-freeze schema, evidence-schema map, deliberate invalid fixtures, CI/evidence documentation, Cargo dependency lock, README, and ledgers.
- **Local verification:**
  - `cargo fmt --all -- --check` — passed
  - `cargo check --locked --all-targets` — passed
  - `cargo clippy --locked --all-targets -- -D warnings` — passed
  - `cargo test --locked --all-targets` — passed; 33 tests total, including 5 governance/schema tests
  - all mapped schemas compile explicitly as Draft 2020-12
  - every JSON evidence document is mapped exactly once and validates
  - canonical PSPR has 47 unique prompts with required objectives/gates/sections
  - deliberate format, Clippy, test, rustdoc, PSPR, and evidence failures were each blocked
  - `git diff --check` — passed
- **Remote verification:** GitHub Actions run [29704751317](https://github.com/USS-Parks/Codex-Added-Context/actions/runs/29704751317) passed all 10 jobs: Rust/docs, contracts, six negative gates, Gitleaks clean/negative proof, and cargo-deny clean/negative proof.
- **Repair evidence:** The first remote run correctly exposed a stale generated Gitleaks fixture. Commit `6645fce3793a8e74dda72765084a507de1ba1f91` aligned the runtime-only fake value with the exact pinned v8.30.1 OpenAI rule; the clean rerun then passed.
- **Implementation commit:** `10c749c5dcecab66ce51ee008dff669ff2952fdd`
- **Published remote SHA:** `6645fce3793a8e74dda72765084a507de1ba1f91` on `codex/context-continuum-v0.1`

## CAC-10 — Implement versioned model-catalog parsing and overlay generation

- **Date:** 2026-07-19
- **Status:** Complete
- **Scope:** Strict Codex 0.144.5 catalog profile, exact-one-Sol selection, deterministic overlay/manifest serialization, official-limit and policy validation, live installed-catalog capture, CLI generation, Draft 2020-12 schemas, drift/malformed fixtures, and preservation tests.
- **Candidate policy:** 1,050,000 context, 1,050,000 maximum, 96% effective (1,008,000 internal budget), with `auto_compact_token_limit` omitted until CAC-14 calibration.
- **Preservation result:** Only `context_window`, `max_context_window`, and `effective_context_window_percent` changed in the live candidate. Installed base-instruction bytes, model-message semantics, and reviewed feature flags were equivalent.
- **Fail-closed result:** Unknown Codex version, root or Sol schema drift, malformed JSON, missing/duplicate Sol, official limit regression, sub-million effective budget, and a threshold above Codex's 90% clamp all fail.
- **Files changed:** `src/model_catalog.rs`, CLI/library wiring, two catalog schemas, current/drift/malformed fixtures, catalog tests, checked-in CAC-10 manifest, architecture documentation, README, evidence map, and ledgers.
- **Verification:**
  - `cargo fmt --all -- --check` — passed
  - `cargo check --locked --all-targets` — passed
  - `cargo clippy --locked --all-targets -- -D warnings` — passed
  - `cargo test --locked --all-targets` — passed; 40 tests total, including 7 catalog tests
  - installed `codex-cli 0.144.5` resolved catalog parsed under `codex-model-catalog/0.144.5-v1`
  - generated output contained exactly one exact `gpt-5.6-sol` entry
  - Codex's own debug parser accepted the command-scoped overlay and reported `1050000,1050000,96`
  - live output catalog SHA-256: `eceabc60cee218fc5a4bd2042ecccd9330be7986a8095ed26905779a15687081`
  - no model request, user configuration change, credential, or install occurred
  - GitHub Actions run [29705566278](https://github.com/USS-Parks/Codex-Added-Context/actions/runs/29705566278) — passed all 10 jobs
- **Implementation commit:** `359c572fb2775a4f68608ff8e3a333dfdc31abfb`
- **Published remote SHA:** `359c572fb2775a4f68608ff8e3a333dfdc31abfb` on `codex/context-continuum-v0.1`

## CAC-11 — Implement atomic Codex configuration management

- **Date:** 2026-07-19
- **Status:** Complete
- **Scope:** Explicit-path `cctx config plan|apply|restore|uninstall`, comment-preserving TOML edits, owned-leaf-only dry-run output, same-directory atomic replacement, manager locking, timestamped exact-byte backups and installed snapshots, versioned ownership manifest, first-backup retention across managed updates, and fail-closed conflict handling.
- **Owned settings:** Exact `gpt-5.6-sol`, 1,050,000-token model window, optional paired automatic-compaction limit/scope, catalog path, hook/plugin enablement, and the required `context_continuum` MCP command, arguments, enablement, and startup timeout.
- **Safety result:** Apply rechecks exact config and manifest bytes under a lock. Restore requires the current config to equal the hash-verified installed snapshot, restores the hash-verified original bytes when one existed, and refuses concurrent edits, later user edits, MCP ownership collisions, invalid TOML, non-Sol policy, limits above the 90% clamp, ambiguous lifecycle state, or corrupted ownership state.
- **Files changed:** `Cargo.toml`/`Cargo.lock`, `src/config_manager.rs`, CLI/library wiring, `tests/config_manager.rs`, ownership JSON Schema, atomic configuration architecture documentation, README, and ledgers.
- **Verification:**
  - `cargo fmt --all -- --check` — passed
  - `cargo check --locked --all-targets` — passed
  - `cargo clippy --locked --all-targets -- -D warnings` — passed
  - `cargo test --locked --all-targets` — passed; 52 tests total, including 12 config-manager tests
  - `RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --document-private-items` — passed
  - `cargo deny check` — advisories, bans, licenses, and sources passed
  - missing, partial, commented, and CRLF configs round-tripped to exact pre-install bytes
  - concurrent plan/config edits, manager-lock contention, later user edits, unowned MCP collision, invalid TOML/table shape, non-Sol model, unpaired compaction values, and above-clamp threshold were refused without overwriting config
  - emitted ownership manifest validated as Draft 2020-12; managed no-op did not rewrite config or manifest; managed update retained the first backup
  - isolated CLI fixture `C:\tmp\cctx-cac11-proof-019f7be3` completed plan/apply/restore with no plan-time state creation and exact before/after SHA-256 `BBB1506DA4F27BC345B29BA5E6C5ED8164711B553F968D8382FA401BF9CD88CD`
  - no global Codex configuration, credential, catalog installation, or model request was touched
  - GitHub Actions run [29707154346](https://github.com/USS-Parks/Codex-Added-Context/actions/runs/29707154346) — passed all 10 jobs
- **Implementation commit:** `279819fe11c00a589dce82569fc4ac5631d06af5`
- **Published remote SHA:** `279819fe11c00a589dce82569fc4ac5631d06af5` on `codex/context-continuum-v0.1`
