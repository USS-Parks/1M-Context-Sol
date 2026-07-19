# ADR-0001: One native binary

- **Status:** Accepted
- **Date:** 2026-07-19
- **Prompt:** CAC-03

## Context

The product needs a CLI, installer, hook receiver, local MCP server, doctor, storage engine, and rollover helper. Splitting those responsibilities into services or separately installed runtimes would multiply update, path, permission, and failure surfaces.

## Decision

Ship one Rust executable, `cctx`, with internal modules. A mode or subcommand may run as a short-lived hook process, an MCP stdio server, or an operator CLI, but all modes use the same versioned schemas and storage code.

Codex remains the only reasoning-loop owner. `cctx` does not become an alternative agent orchestrator.

## Contract

- **Owner:** `cctx` package and release workflow
- **Inputs:** CLI arguments, bounded hook JSON, MCP frames, local config/catalog/store files
- **Outputs:** atomic owned files, structured hook/MCP responses, diagnostics, and local durable records
- **Failure mode:** the invoked mode exits nonzero or emits its protocol-defined error without launching an implicit background service

## Consequences

- Installation, rollback, signing, and provenance cover one executable.
- Internal module boundaries remain testable without network services.
- A single crash cannot be isolated as a microservice failure, so each hook and MCP request needs transaction and timeout boundaries.

## Rejected alternatives

- Electron or GUI host: unnecessary attack and packaging surface.
- Multiple daemons or micro-crates: no independent scaling or authority requirement.
- Replacement agent runtime: duplicates Codex permissions and reasoning authority.
