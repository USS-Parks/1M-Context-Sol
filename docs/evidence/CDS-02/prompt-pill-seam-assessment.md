# CDS-02 prompt-pill seam assessment

## Verdict

Blocked at the native prompt-pill gate on 2026-07-20.

The installed Codex host exposes the authoritative token state needed by a dial, but neither the supported plugin/MCP contract nor the public Codex source exposes a dynamic component insertion seam for the Desktop prompt pill. A compliant live spike therefore cannot be built without an additional OpenAI-provided seam or an explicitly approved modification/repackaging of the proprietary Desktop renderer.

Per `CDS1M-PSPR-1`, this is a mandatory hard stop before CDS-03 or CDS-10. No TUI, MCP response, embedded app panel, separate window, static icon, or DOM automation is accepted as a substitute.

## Proven host token-state seam

The app-server protocol provides `thread/tokenUsage/updated`. Its notification contains:

```text
threadId
turnId
tokenUsage.last.inputTokens
tokenUsage.last.cachedInputTokens
tokenUsage.last.outputTokens
tokenUsage.last.reasoningOutputTokens
tokenUsage.last.totalTokens
tokenUsage.total.<same breakdown>
tokenUsage.modelContextWindow
```

This is adequate host-authoritative state for calculating a live remaining-capacity dial. The current public definition is [ThreadTokenUsageUpdatedNotification](https://github.com/openai/codex/blob/eceb3eeaf3a68d732596fd8c0e8a6807f9166770/codex-rs/app-server-protocol/schema/typescript/v2/ThreadTokenUsageUpdatedNotification.ts).

## Supported UI surfaces inspected

### Plugin manifest

The current official plugin manifest accepts presentation metadata including a static `interface.composerIcon` asset path. The field is an image path, not a component registration, state subscription, render callback, or mutable per-thread value. The official field guide lists no prompt-pill widget/component field and states that unsupported manifest fields are rejected.

The installed bundled Browser plugin demonstrates the supported shape:

```json
{
  "interface": {
    "composerIcon": "./assets/composer-icon.png"
  }
}
```

This can brand a plugin affordance in composer UX. It cannot render a continuously changing context dial.

Official manifest reference: [plugin-json-spec.md](https://github.com/openai/codex/blob/eceb3eeaf3a68d732596fd8c0e8a6807f9166770/codex-rs/skills/src/assets/samples/plugin-creator/references/plugin-json-spec.md).

### MCP and Apps SDK UI

The current Codex manual describes MCP UI as an optional embedded component, modal, fullscreen view, or other app interaction inside ChatGPT. It does not define a host-shell component slot in the native prompt pill, nor a way for an MCP server to subscribe a renderer directly to app-server token notifications.

Those supported MCP UI surfaces fail the PSPR's non-substitution rule even if they display the same number.

### Public source

The official `openai/codex` repository at commit [`eceb3eeaf3a68d732596fd8c0e8a6807f9166770`](https://github.com/openai/codex/commit/eceb3eeaf3a68d732596fd8c0e8a6807f9166770) contains app-server protocol/backend code, CLI, SDKs, and the Rust TUI. It does not contain the installed Desktop Electron renderer. Current code search for `composer` resolves to TUI composer files and plugin metadata, not a Desktop prompt-pill component API.

### Installed client

- Package: `OpenAI.Codex_26.715.7063.0_x64__2p2nqsd0c76g0`.
- Application: `app/ChatGPT.exe`, `Windows.FullTrustApplication`.
- Renderer archive: `app/resources/app.asar`, approximately 201 MB.
- App-server binary: `app/resources/codex.exe`.
- Bundled plugin manifests expose skills, apps, MCP servers, lifecycle configuration, presentation assets, and static `composerIcon` values.
- No declared external renderer-component or prompt-pill widget contract was found.

The renderer is delivered inside the signed Store package. Editing or replacing `app.asar` would be an installed-binary patch, not a supported plugin seam; it would introduce signing, update, compatibility, rollback, and maintenance risks. The Store update that occurred during CDS-01 already demonstrates that installed client versions can change independently during normal operation.

## Exact missing capabilities

A supported implementation needs all of the following, none of which is currently exposed:

1. A plugin or extension API that registers a component inside the native Codex prompt pill.
2. A host bridge that provides that component with the active thread's `thread/tokenUsage/updated` stream.
3. A per-thread lifecycle contract for mounting, updating, resetting, and unmounting the component.
4. A dynamic rendering contract; `composerIcon` is a static asset path.
5. Compatibility/versioning guarantees for the prompt-pill slot.

Because item 1 is absent, no compliant live Desktop spike can be produced through supported means. DOM injection/automation would not create a maintainable product seam and is explicitly non-accepting under the PSPR.

## Stop condition and available escalation

CDS-02 does not pass G1's UI half. Execution stops before architecture freeze and implementation.

The only roster-compliant continuation is separately approved CDS-X1, choosing exactly one route:

- request an official prompt-pill extension slot and token-state binding from OpenAI;
- propose an upstream public UI contract if OpenAI publishes/accepts the relevant renderer source;
- explicitly authorize investigation of a local Desktop patch/fork, accepting Store signing/update and maintenance consequences.

No CDS-X1 route was authorized during this prompt. No external post, reverse engineering, package extraction, renderer modification, repackaging, or binary replacement occurred.
