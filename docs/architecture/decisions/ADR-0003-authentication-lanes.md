# ADR-0003: Codex-owned ChatGPT and official API authentication lanes

- **Status:** Accepted
- **Date:** 2026-07-19
- **Prompt:** CAC-03

## Context

The user's current Codex session is ChatGPT-authenticated. That lane may enforce the resolved 272,000 catalog boundary. Codex also supports official OpenAI API-key authentication, which is separately billed and may expose Sol's documented long context.

## Decision

Test the existing ChatGPT lane first. If it fails the progressive above-cap gate, test Codex's supported official API-key lane under an explicit spend ceiling. Both lanes must return exact `gpt-5.6-sol`.

Codex owns credential entry, storage, and provider authentication. `cctx` observes only a sanitized lane/status verdict and never accepts, logs, stores, exports, or transmits the credential itself.

## Contract

- **Owner:** Codex authentication; `cctx::doctor` owns only compliance reporting
- **Inputs:** operator login through Codex, opaque authenticated session, sanitized auth status
- **Outputs:** authenticated Sol request capability and a non-secret lane label
- **Failure mode:** missing access or an unproven lane is noncompliant; no automatic credential or provider fallback occurs

## Consequences

- ChatGPT and API evidence remain separately attributable.
- Paid API probes require a monetary/token ceiling.
- ChatGPT-only features may differ in the API lane and must be documented.

## Rejected alternatives

- Product-managed key vault: unnecessary credential exposure.
- Environment-variable scraping: leaks authority into hooks and diagnostics.
- Third-party or alternate-provider routing: outside the exact Sol contract.
