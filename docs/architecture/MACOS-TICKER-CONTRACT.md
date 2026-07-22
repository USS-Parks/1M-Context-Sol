# macOS ticker contract

This contract freezes MAC-00 for the unsigned community build. It is intentionally narrower than a general macOS client.

## Supported baseline

- The ticker deployment target is macOS 13.0. OpenAI's current macOS desktop host requires macOS 14 or later, so current end-to-end use starts at macOS 14 even though the ticker binary itself remains 13-compatible.
- The supported host bundle identifier is `com.openai.codex`. This identifier is checked at runtime. A renamed, beta, patched, or future host is unsupported until its identifier and window behavior are reviewed.
- The host must be the frontmost application reported by `NSWorkspace.shared.frontmostApplication`. A name match alone is never enough.
- Window bounds come from `CGWindowListCopyWindowInfo`, filtered to the frontmost host PID, on-screen normal windows, and a single unambiguous candidate. The ticker never calls a window-image or screen-capture API.
- Codex state is read below `~/.codex/sessions/`. A candidate must begin with `session_meta`, have `payload.originator == "Codex Desktop"`, and, during automatic selection, must not have `payload.thread_source == "subagent"`. Token state comes only from the newest valid `token_count` record's `payload.info.last_token_usage.total_tokens`. Its host window comes from `payload.info.model_context_window`.
- The normal face is allowed only when the host window is exactly `1008000`. Any other value produces `Context: !` and an invalid status record.

The OpenAI configuration sample currently documents all four owned keys: `model_context_window`, `model_auto_compact_token_limit`, `model_auto_compact_token_limit_scope`, and the startup-only `model_catalog_json`. The ticker owns those keys only; `model = "gpt-5.6-sol"` remains user-owned and must already be exact.

## Native surface

The app creates one `NSPanel` with `borderless` and `nonactivatingPanel` style masks, floating level, no shadow-dependent hit area, and `ignoresMouseEvents = true`. It has no menu bar, tooltip, hover handler, focus path, input capture, transcript view, network client, or screen-capture permission request. It shows only while the supported host is frontmost and its placement inputs are valid.

`SMAppService.mainApp` is the only start-at-login API. Registration, unregistration, and status must report the framework result without inventing success. The app never starts, stops, restarts, or activates Codex.

## Fail-closed decisions

| Condition | Required result |
|---|---|
| Unknown host bundle identifier | Hide the panel and report `unsupported_host` |
| Host not frontmost, minimized, or without one usable window | Hide the panel |
| Missing, unreadable, stale, or malformed rollout state | Show `Context: !` only when placement remains safe; otherwise hide |
| Ambiguous automatic root-task selection | Preserve the shared warning state; never silently choose a subagent |
| Host window other than `1008000` | Show `Context: !`; set `one_m_context_verified` false |
| Pre-existing owned config key | Refuse installation without changing the config |
| Current owned value differs during uninstall | Preserve the user's later edit and remove only still-owned values |
| Login-item status is not enabled | Report the actual `SMAppService` state |

## Build and evidence boundary

The focused hosted runner is `macos-15` with its installed Xcode. The app is built for `arm64` and `x86_64` with `MACOSX_DEPLOYMENT_TARGET=13.0`; `lipo -verify_arch arm64 x86_64` is the architecture gate. The same shared behavior fixture and Sol catalog used by Windows are inputs, not copied forks.

Automated evidence covers parsing, selection, layout arithmetic, AppKit construction, isolated lifecycle behavior, bundle structure, architecture slices, and artifact integrity. It does not prove placement over a real Codex composer, real login launch, Gatekeeper interaction, or live Codex configuration on a physical Mac. Those claims must remain absent from release evidence.

## Source snapshot

Checked 2026-07-21:

- [OpenAI Codex configuration sample](https://learn.chatgpt.com/docs/config-file/config-sample.md)
- [OpenAI macOS host requirements](https://help.openai.com/en/articles/9395554-what-are-the-system-requirements-for-the-chatgpt-macos-app)
- [Observed `com.openai.codex` bundle metadata in the OpenAI Codex repository](https://github.com/openai/codex/issues/25166)
- [Apple `NSPanel`](https://developer.apple.com/documentation/appkit/nspanel)
- [Apple `NSWindow.ignoresMouseEvents`](https://developer.apple.com/documentation/appkit/nswindow/ignoresmouseevents)
- [Apple `NSWorkspace.frontmostApplication`](https://developer.apple.com/documentation/appkit/nsworkspace/frontmostapplication)
- [Apple `CGWindowListCopyWindowInfo`](https://developer.apple.com/documentation/coregraphics/cgwindowlistcopywindowinfo(_:_:))
- [Apple `SMAppService`](https://developer.apple.com/documentation/servicemanagement/smappservice)
- [GitHub-hosted runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)
