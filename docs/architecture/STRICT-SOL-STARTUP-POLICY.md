# Strict Sol-only startup policy

CAC-13 adds one narrow hook handler:

```text
cctx hook startup-policy --audit-dir <ABSOLUTE_DIR>
```

It accepts only the current documented `SessionStart` and `UserPromptSubmit` command-hook envelopes. The handler compares Codex's hook-reported active model with the same deterministic report used by `cctx doctor`, writes a sanitized per-task audit, and returns a documented hook response. It never changes the model, configuration, catalog, authentication, or prompt.

This handler is not installed globally at CAC-13. Plugin packaging, hook trust review, clean-user installation, and real global enrollment remain later roster gates.

## Why both events are enforced

`SessionStart` is the earliest supported thread-scope lifecycle seam. It blocks a new, resumed, cleared, or post-compact task when policy is not ready. `UserPromptSubmit` repeats the check before each ordinary prompt, so a task-level model override cannot bypass a startup result.

The current Codex hook reference says:

- every command hook receives one JSON object on stdin, including `session_id`, `cwd`, `hook_event_name`, and active `model`;
- `SessionStart` adds `source` and supports `continue: false`, `stopReason`, and `systemMessage`;
- `UserPromptSubmit` adds `turn_id` and `prompt`, and supports `decision: "block"` with a visible `reason`;
- non-managed command hooks must be reviewed and trusted before they run; and
- matching hooks run concurrently, so this policy does not claim authority over unrelated hooks.

The implementation is pinned to Codex 0.144.5's documented fields. Unknown fields, unknown permission modes, mixed event fields, malformed JSON, and inputs above 8 MiB fail closed. CAC-30 later generalizes the bounded dispatcher for the remaining lifecycle events.

## Green means every required invariant is green

The policy permits an event only when all of the following are simultaneously true:

1. the active `model` in the hook envelope is exact `gpt-5.6-sol`;
2. it matches the root configured model, so a per-task override is absent;
3. doctor state is `policy_ready`, exit code 0, and no doctor check failed;
4. authentication is an observed ChatGPT or API-key lane with access;
5. the supported catalog contains exactly one model, exact Sol, at 1,050,000 total/max with at least a 1,000,000-token Effective Codex budget;
6. the explicit configured model window is 1,050,000 with no profile model override;
7. the operational input threshold is below the separate 922,000 maximum input; and
8. the strict compaction guard is observed blocking at the same checkpoint threshold.

The evaluator rechecks those fields even if a malformed or fabricated report says `policy_ready`. Catalog/config policy still is not live backend proof: the audit preserves `live_native_window_proven = false`, and CAC-15/CAC-16 remain responsible for G2.

The current installed 272,000-token Sol catalog therefore blocks correctly. The 1.05M candidate overlay also blocks until CAC-14 calibration and the strict guard evidence make doctor genuinely green. There is no temporary allow-list, model alias, Terra fallback, or "warn and continue" path.

## Visible blocks and remediation

Blocked events identify stable check IDs and state that no ordinary prompt was released. The message includes applicable commands from this bounded set:

```text
cctx doctor --json
codex login
cctx catalog generate --codex codex --output <ABS_SOL_CATALOG> --manifest <ABS_CATALOG_MANIFEST>
cctx config plan --config <ABS_CONFIG> --state-dir <ABS_STATE_DIR> --catalog <ABS_SOL_CATALOG> --cctx <ABS_CCTX>
```

The configuration command remains a plan until the operator separately runs the reviewed apply command. `cctx` never accepts or stores a credential; `codex login` remains Codex-owned. A failed audit write also blocks the event instead of allowing an unaudited task.

## Per-task override audit

Each valid event writes one atomic JSON record under a SHA-256-derived session directory. The record contains:

- event/source and permission mode;
- hashes of session ID, turn ID, and working directory;
- active and configured model slugs plus the override verdict;
- sanitized doctor/auth/catalog values;
- decision, blocking check IDs, and remediation commands; and
- explicit omission flags for prompt, transcript path, and credentials.

The prompt is validated as a string while parsing and immediately discarded. Raw prompt text, raw session/turn IDs, raw working directory, transcript path, credentials, arbitrary config, and model instructions do not enter the audit or its filename. Audit records conform to [`startup-policy-audit.schema.json`](../../schemas/startup-policy-audit.schema.json).

## Hook-source shape

Packaging later supplies the absolute executable and state paths. The intended source contains both handlers and uses the same command for each:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup|resume|clear|compact",
        "hooks": [
          {
            "type": "command",
            "command": "<ABS_CCTX> hook startup-policy --audit-dir <ABS_AUDIT_DIR>",
            "timeout": 10,
            "statusMessage": "Verifying exact Sol policy"
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "<ABS_CCTX> hook startup-policy --audit-dir <ABS_AUDIT_DIR>",
            "timeout": 10,
            "statusMessage": "Rechecking exact Sol policy"
          }
        ]
      }
    ]
  }
}
```

Authoritative reference: [Codex hooks](https://learn.chatgpt.com/docs/hooks).
