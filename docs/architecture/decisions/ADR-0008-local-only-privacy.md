# ADR-0008: Local-only persistence and explicit privacy controls

- **Status:** Accepted
- **Date:** 2026-07-19
- **Prompt:** CAC-03

## Context

Hook and transcript data may contain source code, personal data, credentials, command output, and proprietary context. Durable retention expands the impact of path, permission, export, and deletion defects.

## Decision

Store ordinary context only in a user-local state directory with restrictive platform permissions. Disable telemetry and cloud synchronization. Apply configurable redaction before persistence, bound retained fields, and make scoped export, retention, and `forget --dry-run` first-class operations.

API credentials remain exclusively in Codex authentication and are excluded from all cctx inputs, evidence, logs, and storage.

## Contract

- **Owner:** `cctx::security` for policy and `cctx::reservoir` for enforcement
- **Inputs:** canonical event plus classification, redaction, retention, and explicit export/delete scope
- **Outputs:** permission-checked local record, sanitized export, deletion plan/audit
- **Failure mode:** deny persistence/export/deletion when permissions, canonical paths, or scope cannot be proven

## Consequences

- Windows ACLs, Unix modes, junctions/symlinks, backups, and crash artifacts need platform tests.
- Same-user malware and a compromised operating system remain outside the achievable trust boundary and must be documented.
- Exact deletion claims are limited to product-owned, enumerated copies and operating-system storage realities.

## Rejected alternatives

- Mandatory cloud continuity: adds exfiltration, tenancy, and availability risks.
- Store everything then redact on export: leaves secrets at rest.
- Silent retention: incompatible with informed user control.
