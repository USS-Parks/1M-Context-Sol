# MAC-01 native macOS ticker evidence

## Result

M-G1 and M-G2 passed on the GitHub-hosted `macos-15` runner at commit `14eafa7f9bafb814b86e8582070b4f00ed71d80d`.

- Workflow run: [29886220336](https://github.com/USS-Parks/1M-Context-Sol/actions/runs/29886220336)
- Job: `macOS native ticker`
- Runner: macOS 15 arm64
- Release build: passed with Swift warnings denied
- XCTest: six tests, zero failures
- Full workflow: all twelve jobs passed

## Behavior covered

The Swift test target reads `ticker/fixtures/behavior-cases.json` from the repository rather than copying it. The hosted suite passed:

- five active-token cases, including baseline floor, stale state, saturation, and compaction;
- three root-task selection cases, including automatic subagent exclusion and explicit pinning;
- four responsive layout calculations;
- the exact `1008000` host-window guard and the `Context: !` failure path;
- construction of one borderless, nonactivating, floating, click-through `NSPanel` with a complete face;
- a source scan excluding window-image capture, ScreenCaptureKit, pointer tracking, global event monitors, tooltip wiring, and transcript surfaces.

The executable uses `NSWorkspace.shared.frontmostApplication` for the exact `com.openai.codex` host and bounded `CGWindowListCopyWindowInfo` metadata for placement. It reads only local rollout metadata and writes only the app-owned status path.

## Repair history

The first hosted build at `e84f25de8f5202fd091020b21922d130659989e7` compiled the release executable with warnings denied, then found one throwing helper inside an XCTest message autoclosure. Commit `daeaf965822b692bbd502c75eb9a820787ae296a` moved that lookup outside the autoclosure; the macOS job passed.

The same run exposed a stale Windows-only `AGENTS.md` assertion in the Rust governance test. Commit `14eafa7f9bafb814b86e8582070b4f00ed71d80d` preserved the completed Windows roster and added structural validation for active `MAC-00` through `MAC-04`. The following run passed every job.

## Verification boundary

This is hosted compilation and automated AppKit evidence. It does not claim physical-Mac placement over a real composer, real login-item registration, live Codex configuration changes, or Gatekeeper interaction.
