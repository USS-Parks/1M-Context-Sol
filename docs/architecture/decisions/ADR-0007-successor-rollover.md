# ADR-0007: Verified successor rollover, not a replacement orchestrator

- **Status:** Accepted
- **Date:** 2026-07-19
- **Prompt:** CAC-03

## Context

Strictly blocking compaction means a source task cannot remain live forever. Long agentic work still needs a way to continue before maximum input pressure without mutating or compacting the source.

## Decision

Create a verified checkpoint and bounded recovery capsule, then use supported Codex CLI or app-server seams to start a fresh task in the same working directory with the exact Sol-only policy. Verify the successor with doctor before ordinary work.

The source task remains preserved. `cctx` coordinates one handoff but does not schedule tools, reinterpret permissions, or become a second agent loop.

## Contract

- **Owner:** `cctx::rollover`
- **Inputs:** verified checkpoint, source task ID, working directory, catalog/config hashes, bounded capsule
- **Outputs:** successor launch request, successor task ID, source-to-successor provenance, verification verdict
- **Failure mode:** refuse the successor and leave the source stopped/preserved; route an unsupported public seam to CAC-X1

## Consequences

- Public wording must say fresh successor, not infinite same-task context.
- Handoff capsules are historical untrusted data with exact source references.
- Tool side effects are not replayed automatically.

## Rejected alternatives

- Compaction-backed resume: violates the preservation contract.
- A custom autonomous scheduler: duplicates Codex control and approval semantics.
- Blind transcript replay: can re-execute instructions and exceed input limits.
