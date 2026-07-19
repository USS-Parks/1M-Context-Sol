# ADR-0005: Separate lifecycle capture from model-callable retrieval

- **Status:** Accepted
- **Date:** 2026-07-19
- **Prompt:** CAC-03

## Context

Codex hooks provide lifecycle events and blocking decisions but have small output/context budgets. MCP provides model-callable retrieval but does not observe every lifecycle event or control compaction.

## Decision

Use hooks only for bounded lifecycle capture, startup enforcement, proactive warnings, and compaction decisions. Use MCP only for bounded, authorized store/search/fetch/pin/checkpoint/export/forget operations.

Both paths share canonical schemas and the single reservoir. Neither path may change the active model or silently mutate global configuration.

## Contract

- **Owner:** `cctx::hooks` exclusively owns hook transport; `cctx::mcp` exclusively owns MCP transport
- **Inputs:** documented bounded hook JSON or a schema-valid scoped MCP request, routed only to its owning path
- **Outputs:** valid bounded hook protocol or a bounded structured MCP result
- **Failure mode:** malformed, oversized, timed-out, or out-of-scope input produces a protocol-valid denial without partial authority changes

## Consequences

- A million-token corpus is never injected through hook stdout.
- Recall remains explicit, provenance-rich, and budgeted.
- Hook and MCP parsers need separate fuzz and timeout coverage.

## Rejected alternatives

- Hook-only retrieval: output limits make it incomplete and fragile.
- MCP-only capture: misses mandatory lifecycle and compaction control.
- Parallel hook and MCP stores: violates single durable authority.
