# Security Policy

## Supported versions

No released version exists yet. Security fixes apply to the active development branch until the first release.

## Reporting a vulnerability

Do not open a public issue for suspected credential exposure, transcript disclosure, path escape, config corruption, or remote-code execution. Use GitHub's private vulnerability reporting for `USS-Parks/1M-Context-Sol` when available, or contact the repository owner privately through their published GitHub contact channel.

Include the affected commit/version, platform, minimal reproduction, impact, and whether sensitive data may already have been exposed. Do not include real API keys or private transcripts.

## Project security invariants

The ordinary runtime is local-only and telemetry-off. Credentials must never enter the repository, reservoir, evidence bundle, or logs. Recalled content is untrusted data. Unknown Codex schemas and catalog drift fail closed.

The repository-scoped [threat model](docs/security/THREAT-MODEL.md) defines assets, trust boundaries, attacker-controlled inputs, security objectives, and severity calibration. The [machine boundary contract](contracts/architecture-boundaries.json) is structurally tested.
