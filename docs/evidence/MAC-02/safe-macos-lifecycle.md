# MAC-02 safe macOS lifecycle evidence

## Result

M-G3 passed on the GitHub-hosted `macos-15` runner at commit `c0ba24cf1a583c75e3e50c6d52be3254f5a98bf0`.

- Workflow run: [29886681553](https://github.com/USS-Parks/1M-Context-Sol/actions/runs/29886681553)
- Job: `macOS native ticker`
- Release build: passed with Swift warnings denied
- Lifecycle tests: nine, zero failures
- Full Swift suite: fifteen tests, zero failures
- Full workflow: all twelve jobs passed

## Lifecycle covered

Each test used a unique temporary home, temporary `~/.codex/config.toml`, the shared Sol catalog, and an injected login-item service. The suite passed:

- first install and idempotent first launch;
- exact four-key conflict refusal without a config change;
- exact `gpt-5.6-sol` prerequisite refusal;
- byte-exact backup plus exact restoration when the installed config was otherwise unchanged;
- later user-edit preservation with removal of only values still owned by the ticker;
- catalog and manifest hash verification;
- upgrade without replacing the original backup;
- registration-failure rollback of the config and every named app-owned file;
- start-at-login status, including `requiresApproval`;
- stop through an injected ticker-only process controller;
- uninstall, including login-item unregistration and app-owned status cleanup.

The checked `Info.plist` fixes bundle identifier `com.ussparks.1m-context-ticker`, background-app behavior through `LSUIElement`, and macOS 13.0 as the deployment minimum. The production adapter calls only `SMAppService.mainApp.register()` and `unregister()` for login ownership.

## Scope boundary

The hosted tests did not read or change a real user's home, Codex configuration, login items, or running applications. The system process adapter filters only the ticker bundle identifier and excludes its own process. No path starts, stops, restarts, or activates Codex.
