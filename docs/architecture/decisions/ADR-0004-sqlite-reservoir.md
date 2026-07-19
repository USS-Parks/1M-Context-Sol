# ADR-0004: SQLite WAL and FTS5 as the sole durable reservoir

- **Status:** Accepted
- **Date:** 2026-07-19
- **Prompt:** CAC-03

## Context

The product must retain at least one million tokenizer-counted tokens, survive process restarts and crashes, support exact retrieval and lexical search, and remain local without a database service.

## Decision

Use one SQLite database in WAL mode with bundled FTS5. Canonical events are append-only, content-addressed, monotonically sequenced, and transactionally linked to indexes, pins, artifacts, and checkpoint manifests.

SQLite is the only durable context authority. Transcript snapshots are forensic inputs/artifacts, not a second mutable store. Optional embeddings cannot become a required or authoritative path.

## Contract

- **Owner:** `cctx::reservoir`
- **Inputs:** validated canonical events and scoped, parameterized commands
- **Outputs:** committed records, exact/hash fetches, ranked rows, backups, and verified checkpoint manifests
- **Failure mode:** rollback, integrity failure, or locked/unavailable store; callers receive a visible error and compaction remains blocked

## Consequences

- No database daemon, cloud tenancy, or network availability is required.
- WAL, permissions, backup, migration, and concurrency need adversarial tests.
- Deletion must cover primary rows, FTS content, exports, backups, and retained policy artifacts as documented.

## Rejected alternatives

- Vector database or mandatory embeddings: extra model, service, and privacy dependencies.
- Flat transcript files: weak concurrency, migration, and query guarantees.
- Multiple stores by feature: ambiguous deletion and checkpoint authority.
