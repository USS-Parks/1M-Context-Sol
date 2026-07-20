# Context Continuum for GPT-5.6 Sol

Context Continuum is an open-source Codex runtime extension in active development for exact model `gpt-5.6-sol`. The planned product combines a Codex plugin, native `cctx` CLI/service, local MCP server, strict compaction guard, and durable local context reservoir.

## Current status

Foundation milestone M0 is complete under the [canonical PSPR](PLANNING/CODEX-CONTEXT-CONTINUUM-PSPR.md), and M1 implementation is underway. No native one-million-window claim has passed its live gate yet, and this repository does not currently provide an installable release.

The frozen [CAC-01 capability baseline](docs/architecture/CODEX-CAPABILITY-BASELINE.md) shows `codex-cli 0.144.5` bundling Sol at 372,000 tokens but resolving Sol at 272,000 with a 258,400-token effective budget. That reproduces the current “256k class” behavior and remains far below Sol's official 1,050,000-token total window.

The release contract, once proven, is:

> GPT-5.6 Sol with its native 1.05M window. Compaction blocked. Durable continuity beyond the window.

That sentence is a gated target, not a current performance claim. The project always reports Sol's 1,050,000-token total window separately from its 922,000-token maximum input and 128,000-token maximum output.

## Governance

Implementation follows stable CAC prompt IDs, prompt-local acceptance gates, and an auditable [development log](docs/sessions/CODEX-CONTEXT-CONTINUUM-DEVLOG.md). Public wording and capacity numbers are governed by the [claim contract](docs/architecture/CLAIM-CONTRACT.md) and its [machine-readable vocabulary](contracts/capability-vocabulary.json). See [verification status](docs/VERIFICATION.md) before relying on any capability.

The settled component ownership and trust transitions are documented in the [data-flow contract](docs/architecture/DATA-FLOW-AND-TRUST-BOUNDARIES.md), nine [architecture decisions](docs/architecture/decisions/), and the repository [threat model](docs/security/THREAT-MODEL.md).

The [CI and evidence contract](docs/CI-AND-EVIDENCE-CONTRACT.md) requires clean Rust, documentation, secret, supply-chain, PSPR, and Draft 2020-12 JSON Schema gates plus deliberate failure proofs.

The version-pinned [Sol catalog overlay](docs/architecture/MODEL-CATALOG-OVERLAY.md) now generates an uninstalled one-model candidate while preserving installed instructions and capabilities. That local catalog/parser proof is not live backend acceptance.

The [atomic configuration manager](docs/architecture/ATOMIC-CONFIG-MANAGEMENT.md) can now dry-run, atomically apply, and exactly roll back only its declared model, window, catalog, hook/plugin, and MCP settings. It requires explicit absolute paths and has not been run against the real global Codex configuration.

The claim-safe [`doctor` and `status`](docs/architecture/DOCTOR-AND-STATUS-CONTRACT.md) commands now expose exact Sol identity, authentication lane, official limits, resolved catalog policy, Effective Codex budget, operational threshold, and compaction-guard state in human and JSON forms. They deliberately distinguish configuration-policy readiness from still-open live native-window proof.

The [strict startup policy](docs/architecture/STRICT-SOL-STARTUP-POLICY.md) now implements fail-closed `SessionStart` and `UserPromptSubmit` handling. It blocks any non-Sol active model, model override, non-green doctor state, missing access, 272k/stale catalog, missing calibrated threshold, or unproven compaction guard, and records a prompt-free per-task audit. It is implemented and tested but is not globally installed yet.

The [`cctx sol` catalog launcher](docs/architecture/SOL-1M-CATALOG-LAUNCHER.md) is the first directly usable path. It generates the replacement catalog from the installed Sol entry, requires Codex's own parser to resolve exact policy `1,050,000 / 1,050,000 / 96 / 900,000`, displays the context meter, and launches Codex with command-scoped overrides. It does not change global configuration or invoke another API lane. The catalog path is proven; an actual request above 272,000 remains a separate live gate.

## Run the Sol 1M catalog launcher now

From this repository in Windows PowerShell:

```powershell
cargo build --locked --release
$cctxState = Join-Path $env:LOCALAPPDATA "ContextContinuum\sol"
$codexCommand = Join-Path $env:APPDATA "npm\codex.cmd"
.\target\release\cctx.exe sol verify --codex $codexCommand --state-dir $cctxState
.\target\release\cctx.exe sol run --codex $codexCommand --state-dir $cctxState --
```

`sol run` starts a new Codex TUI with a live footer containing the model, Codex's `context-remaining` counter, and the current directory. The startup meter shows Context Continuum's frozen bands at 840,000 checkpoint, 860,000 rollover, 880,000 admission stop, and 900,000 automatic compaction. There is no embedded desktop meter yet, and an already-running desktop task cannot retroactively adopt the replacement catalog.

The model does not independently know its exact token count. Codex reads the model-catalog values, budgets the task, reports remaining context in the client, and sends requests to the backend, which enforces the native limit.

## License

Apache-2.0. See [LICENSE](LICENSE).
