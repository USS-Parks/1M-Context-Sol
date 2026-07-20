# Sol 1.05M Catalog Launcher

`cctx sol` turns the catalog seam into an executable, fail-closed Codex launch path. It generates a one-model catalog from the installed `gpt-5.6-sol` entry, changes only context-policy fields, asks Codex itself to resolve the result, and refuses to launch unless the resolved policy is exactly 1,050,000 context, 1,050,000 maximum, 96 percent effective, and a 900,000-token automatic-compaction boundary.

The command is command-scoped. It does not edit global Codex configuration, install a plugin, enable the unfinished MCP runtime, or route to another model or API transport.

```powershell
cargo build --locked --release
$cctxState = Join-Path $env:LOCALAPPDATA "ContextContinuum\sol"
$codexCommand = Join-Path $env:APPDATA "npm\codex.cmd"
.\target\release\cctx.exe sol verify --codex $codexCommand --state-dir $cctxState
.\target\release\cctx.exe sol run --codex $codexCommand --state-dir $cctxState --
.\target\release\cctx.exe sol meter --used-tokens 600000
```

`prepare` writes only `sol-1m-models.json` and its hash manifest into the explicit state directory. `verify` regenerates those files from the current installed catalog and validates the resolved values through `codex debug models`. `run` performs the same verification, displays the threshold meter, and launches Codex with the supported TUI footer fixed to model, live context remaining, and current directory. Inside the launched TUI, `/status` reports token usage and `/statusline` can inspect or reorder footer fields.

The live 0.144.5 calibration measured 17,766 base-instruction bytes and 18,039 model-message bytes. A conservative three-bytes-per-token estimate plus a separate 16,000-token tool/wrapper reserve fits inside the 42,000-token margin below Sol's 922,000 maximum input. The visible operating bands are 840,000 checkpoint, 860,000 rollover, 880,000 admission stop, and 900,000 compaction boundary.

The model does not introspect its exact live token count. The catalog informs Codex's client budget and Codex's own TUI displays the resulting live context remaining. `sol meter --used-tokens` maps an independently observed count to Context Continuum's checkpoint, rollover, admission, and compaction bands; later lifecycle capture feeds that same `SolMeter` contract automatically.

There is no wheel or embedded meter in the current Codex desktop surface. A task already running in the desktop app keeps the catalog policy it resolved at startup. The currently deployable UX is a new CLI task launched through `cctx sol run`, where the Codex-native footer is live and the Context Continuum meter describes the frozen safety bands.

This is client/catalog proof, not a completed above-272,000 model request. Machine-readable evidence keeps `live_request_proven` false until that live gate actually succeeds.
