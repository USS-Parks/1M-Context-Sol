# ADR-0006: Strict compaction blocking

- **Status:** Accepted
- **Date:** 2026-07-19
- **Prompt:** CAC-03

## Context

Compaction is lossy and contradicts the core preservation goal. Codex's documented `PreCompact` hook can stop an automatic or manual compaction, but stopping it can also abort the active turn at the hard limit.

## Decision

Before every known compaction attempt, complete and independently verify a reservoir/WAL checkpoint plus transcript snapshot manifest. Then return `continue: false` for automatic and manual `PreCompact` events.

Treat every `PostCompact` event as a policy violation. Do not normalize an emergency recovery capsule as successful compaction.

## Contract

- **Owner:** `cctx::guard`
- **Inputs:** validated lifecycle event, token estimate, database/transcript state, checkpoint verdict
- **Outputs:** checkpoint ID, `continue: false` response, visible diagnostic, rollover/recovery command, incident record when applicable
- **Failure mode:** halt once and preserve all available evidence; never allow compaction after a partial or unverifiable checkpoint

## Consequences

- Same-task liveness is intentionally sacrificed when preservation and liveness conflict.
- Repeated-trigger suppression must prevent restart loops.
- Proactive checkpointing and successor rollover are required before the production threshold.

## Rejected alternatives

- Letting Codex compact after a summary: violates the literal goal.
- Quietly increasing the threshold: delays but does not prevent loss.
- Automatic retry loops: can duplicate tools, costs, and side effects.
