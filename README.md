# Context Continuum for GPT-5.6 Sol

Context Continuum is an open-source Codex runtime extension in active development for exact model `gpt-5.6-sol`. The planned product combines a Codex plugin, native `cctx` CLI/service, local MCP server, strict compaction guard, and durable local context reservoir.

## Current status

Foundation work is underway under the [canonical PSPR](PLANNING/CODEX-CONTEXT-CONTINUUM-PSPR.md). No native one-million-window claim has passed its live gate yet, and this repository does not currently provide an installable release.

The frozen [CAC-01 capability baseline](docs/architecture/CODEX-CAPABILITY-BASELINE.md) shows `codex-cli 0.144.5` bundling Sol at 372,000 tokens but resolving Sol at 272,000 with a 258,400-token effective budget. That reproduces the current “256k class” behavior and remains far below Sol's official 1,050,000-token total window.

The release contract, once proven, is:

> GPT-5.6 Sol with its native 1.05M window. Compaction blocked. Durable continuity beyond the window.

That sentence is a gated target, not a current performance claim. The project always reports Sol's 1,050,000-token total window separately from its 922,000-token maximum input and 128,000-token maximum output.

## Governance

Implementation follows stable CAC prompt IDs, prompt-local acceptance gates, and an auditable [development log](docs/sessions/CODEX-CONTEXT-CONTINUUM-DEVLOG.md). See [verification status](docs/VERIFICATION.md) before relying on any capability.

## License

Apache-2.0. See [LICENSE](LICENSE).
