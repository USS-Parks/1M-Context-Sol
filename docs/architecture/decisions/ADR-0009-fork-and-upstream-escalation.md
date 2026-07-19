# ADR-0009: Narrow fork and upstream escalation only after public seams fail

- **Status:** Accepted
- **Date:** 2026-07-19
- **Prompt:** CAC-03

## Context

Supported catalog, config, hooks, MCP, and app-server seams may not expose every required control. A permanent broad Codex fork would create a large security and maintenance obligation before those seams are tested.

## Decision

Use supported seams first. Enter CAC-X1 only when CAC-14, CAC-30, or CAC-35 records a precise blocker. Any fork is a reproducible, minimal patch to the open-source Codex client for the failed gate only. Offer a generalizable minimal upstream change through CAC-X2. Use the Sol-only transport shim in CAC-X3 only for a proven built-in transport gap.

No escalation path may claim to bypass OpenAI backend limits or route to another model/provider.

## Contract

- **Owner:** CAC-X1/CAC-X2/CAC-X3 addendum and the `cctx` release workflow
- **Inputs:** failed-gate evidence, pinned upstream revision, smallest proposed change, security/compatibility analysis
- **Outputs:** reproducible patch/build or upstream proposal with tests and maintenance cost
- **Failure mode:** keep the core gate open; do not silently ship the fork as equivalent or weaken the claim

## Consequences

- Upstream drift and supply-chain cost are explicit release inputs.
- A forked binary needs separate provenance, update, and vulnerability tracking.
- External issue/PR submission retains its user approval gate.

## Rejected alternatives

- Fork first: premature and costly.
- Broad product divergence: obscures upstream security fixes.
- Alternate model/provider proxy: violates exact Sol scope.
