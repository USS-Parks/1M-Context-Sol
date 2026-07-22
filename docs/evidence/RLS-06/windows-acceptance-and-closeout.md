# RLS-06 Windows acceptance and closeout

Date: 2026-07-21

## Result

W-G1 through W-G5 pass for the local Windows release candidate described here. RLS-06 found and corrected three ordinary-user-flow defects before closeout: the documented manager command did not initialize its default source path correctly under Windows PowerShell 5.1, the dynamic ticker face could clip the trailing `1M`, and the default WPF tooltip was obstructive and caused a temporary palette-sampling feedback effect. The accepted build measures the complete face explicitly, adds a safety margin, has no hover UI, passes pointer input through to Codex, and refuses to show the normal `/ 1M` face for a host window other than exactly 1,008,000.

The user-visible proof is now two-layered:

- the passive face is `Context: <active tokens> / 1M` only for an exact, fresh, unambiguous 1M host state; an invalid state renders `Context: !`;
- `overlay/manage-overlay.ps1 -Action Status` reports `required_host_window`, `one_m_context_verified`, and `display_state` explicitly.

## Ordinary user flow

The following documented Windows PowerShell commands were exercised against the real user-local installation:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\overlay\manage-overlay.ps1 -Action Status
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\overlay\manage-overlay.ps1 -Action Stop
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\overlay\manage-overlay.ps1 -Action Start
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\overlay\manage-overlay.ps1 -Action Uninstall
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\ticker\windows\verify-release.ps1 -OutputDirectory .\dist
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\overlay\manage-overlay.ps1 -Action Plan
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\overlay\manage-overlay.ps1 -Action Install
```

The first uninstall preserved later unrelated configuration edits while removing the four owned settings. The final uninstall restored the exact pre-install bytes. Both removed only the verified ticker process, two owned shortcuts, and `C:\Users\17076\AppData\Local\CodexContextOverlay`. Reinstallation recreated the managed state from the checked-in artifact. Codex Desktop and its app-server were not stopped, restarted, or modified; the nine observed Codex/ChatGPT process IDs remained unchanged across the final replacement.

## Gate reconciliation

### W-G1 - Identity and behavior

- Root README, source, assembly metadata, artifact names, and manifest use `1M Context Ticker` consistently.
- Shared PowerShell and native tests continue to use `last_token_usage.total_tokens`, not cumulative totals.
- Five token cases, three selection cases, four layout cases, malformed input, bounded tails, stale state, compaction, and ambiguous selection passed.
- Both parsers now reject a non-1M `model_context_window`; the native self-test records one window-guard case.
- The active task reported host window `1008000`, fresh state, unambiguous selection, and no error.

### W-G2 - Native executable parity and passive UX

- Final installed process: PID `21076`, exact installed executable path, one instance after duplicate launch, zero children, and zero PowerShell ticker processes.
- Live status: `visible-in-codex`, `213x30`, dark prompt palette, sidebar-aware center `915`, no-activate and tool-window styles set.
- `WS_EX_TRANSPARENT` is set and reported as `has_transparent_input: true`; the ticker no longer captures hover or composer input.
- The default WPF tooltip and right-click handler were removed. Exact diagnostics remain in the manager status output.
- Explicit WPF text measurement plus 22 device-independent pixels of combined padding/safety space is tested for `Context: 117,015 / 1M` and `Context: 1,008,000 / 1M`.

### W-G3 - Safe lifecycle

- Clean plan/install/start/status/stop/uninstall, duplicate launch, exact restore, later-edit preservation, and shortcut creation/removal passed.
- The Windows PowerShell 5.1 default-source-path regression is covered by the lifecycle suite without supplying `-SourceRoot` to the plan action.
- Final configuration SHA-256: `0372166b62a83e9f038d0c1a53c037c387a3df6cf7624e38e1b44db9f1793eb3`.
- Final source and installed manager SHA-256: `3b0d6b6a72fe2e90fd2c2a827546a3d8421101d4f3ac084f570c4fa3f60b3c9b`.
- Startup and Start Menu shortcuts exist and target the exact installed native executable.

### W-G4 - Artifact integrity

Two fresh builds and final `dist` matched byte-for-byte:

- executable: `dist/1M-Context-Ticker-Windows-x64.exe`
- bytes: `38,400`
- SHA-256: `f62558811f95866c4284ea2f68ce06355805230179735c74cbae1244c0337f56`
- checksum-file SHA-256: `aa895166094c4886165e2acf3b69dd8bb8aa3cfeb20ebc868830886f1b1ecda4`
- artifact-manifest SHA-256: `a94bbd774177cb669ed1af1a3a65903f69221143a35e2363a351de89f74491c9`
- architecture/version: AMD64 / `0.1.0.0`
- self-test: five token, three selection, four layout, two face-width, and one window-guard cases
- dependencies: the same seven allowlisted .NET Framework assemblies; no third-party runtime or PowerShell dependency

The source artifact, installed executable, and install-manifest hashes match exactly.

### W-G5 - Repository welcome page

- Six repository-relative links resolve.
- Official name, artifact, 1,050,000 catalog total, 1,008,000 host budget, 900,000 compaction threshold, install/status/uninstall flow, exact per-task verification, privacy boundary, and limitations are present.
- The stale TUI/MCP product command is absent.
- Public signing, GitHub Release publication, imagery, and macOS packaging are explicitly parked.

## Verification commands and results

- `overlay/Test-ContextOverlay.ps1` - passed.
- `overlay/Test-OverlayInstaller.ps1` - passed.
- `ticker/windows/verify-release.ps1 -OutputDirectory .\dist` - passed; two reproducible builds.
- Windows PowerShell parser across manager, tests, build, and verifier - no errors.
- README relative-link and identity checks - passed.
- `git diff --check` - passed.
- `cargo clippy --all-targets -- -D warnings` - passed against the preserved tree.
- `cargo fmt --all -- --check` - non-result for RLS scope: it reports formatting differences only in excluded `src/precompact_guard.rs`.
- `cargo test --all-targets` - environment/cache non-result: a reused governance-test binary retained compile-time `CARGO_MANIFEST_DIR=C:\tmp\1mct-full-ci-3aac93eda39e4130968762dfa1a72b46` and could not find repository governance files there. The published pre-RLS-06 source baseline at `7961497cd1f40b54332c71629c1a6267494b8bb7` remains green in all 11 hosted jobs; RLS-06 changes no Rust product source.

## Preserved state and storage

The release work did not stage or alter the protected historical lane:

- `src/lib.rs`: 998 bytes; SHA-256 `47a0c5211ab3c57aac7cda7ae753ec9bb0f61564b0a9e381f0802a07769aa7e1`
- `src/main.rs`: 31,175 bytes; SHA-256 `8e115b381cb09f8e39bbc1fe626f31f34f2bd4e7b790abc1d2ed342af7da9d0b`
- `src/precompact_guard.rs`: 17,477 bytes; SHA-256 `48a4926c8047712669f89de7cb49366e11250db34701aefc622358003a81adc3`

Exactly one registered worktree remains: the canonical `main` checkout at `C:\Users\17076\Documents\Codex 1M Context Project`. It is retained because it is the authoritative checkout and still contains the protected uncommitted lane. Generated data is approximately 922 MiB under `target`; final `dist` is approximately 39 KiB. No worktree or dependency tree was created for RLS-06.

## Publication and remaining scope

The focused RLS-06 implementation/acceptance commit is recorded in the release DEVLOG after creation. Push, tag, GitHub Release publication, signing, imagery, macOS/DMG work, and external communication remain unauthorized and unperformed. The paid request above the prior live-token boundary remains absent and cannot be advertised as proven.
