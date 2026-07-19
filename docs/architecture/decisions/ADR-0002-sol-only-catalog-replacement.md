# ADR-0002: Sol-only authoritative catalog replacement

- **Status:** Accepted
- **Date:** 2026-07-19
- **Prompt:** CAC-03

## Context

CAC-01 proves that Codex 0.144.5 bundles Sol at 372,000 tokens but resolves it at 272,000 and 258,400 effective tokens, while OpenAI documents a 1,050,000-token total Sol window. A plain `model_context_window` override is bounded by catalog policy.

## Decision

Generate a deterministic, version-pinned `model_catalog_json` containing exactly the installed `gpt-5.6-sol` entry with only approved context-policy fields changed. Preserve the installed Sol instructions and feature flags. Install the catalog through owned, atomic config.

The overlay is client policy, not live backend proof. G2 still requires returned Sol identity and an above-cap request.

## Contract

- **Owner:** `cctx::model_catalog`
- **Inputs:** bundled/resolved catalog, Codex version/schema, official Sol limits, measured CAC-14 calibration
- **Outputs:** one-model catalog, normalized hash, source hash, compatibility verdict
- **Failure mode:** any unknown schema, source drift, non-Sol entry, reduced official limit, or ambiguous field blocks generation and installation

## Consequences

- Model-picker fallback cannot be mislabeled compliant.
- Codex updates require revalidation before an overlay is reused.
- The product cannot bypass a service-side entitlement or hard model limit.

## Rejected alternatives

- Editing the bundled catalog in place: not atomic or update-safe.
- Setting only `model_context_window`: current catalog maximums clamp it.
- Routing to another model: violates the goal and claim contract.
