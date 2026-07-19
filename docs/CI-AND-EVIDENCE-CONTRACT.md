# CI and Evidence Contract

## Required checks

`.github/workflows/ci.yml` runs on `main`, `codex/**` branches, pull requests, and manual dispatch. It grants only `contents: read` and requires no release, deployment, OpenAI, or user-managed GitHub credential.

| Job | Clean gate | Deliberate failure proof |
|---|---|---|
| Rust quality and docs | format, check, Clippy with warnings denied, tests, rustdoc warnings denied | temporary unformatted source, Clippy violation, failing test, broken rustdoc link |
| PSPR and evidence contracts | 47 unique prompt blocks; required sections/objectives/gates; mapped Draft 2020-12 JSON Schema validation | committed invalid PSPR and invalid capability evidence fixtures |
| Secret scan | full-history Gitleaks scan using a hash-pinned v8.30.1 Linux archive | runtime-generated OpenAI-shaped key must produce finding exit code 1 |
| Supply chain | cargo-deny advisories, bans, registries/sources, and allowed licenses | alternate config explicitly bans `serde` and must fail |

The workflow itself fails if a negative fixture unexpectedly passes. A broken validator cannot produce a green self-test by merely skipping the fixture.

## Pinned external tools

- `actions/checkout` commit `9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0` (v7.0.0)
- `dtolnay/rust-toolchain` commit `4cda84d5c5c54efe2404f9d843567869ab1699d4` with Rust 1.96.1
- `EmbarkStudios/cargo-deny-action` commit `3c6349835b2b7b196a839186cb8b78e02f7b5f25` (v2.1.1)
- Gitleaks v8.30.1 Linux x64 archive SHA-256 `551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb`
- `jsonschema` crate 0.48.1 with network/file resolution features disabled

Pin changes require a focused dependency/tooling update with primary-source release evidence.

## Evidence mapping rule

Every JSON file under `docs/evidence/` must appear exactly once in `contracts/evidence-schema-map.json`. Its mapped schema must compile explicitly as Draft 2020-12 and the document must validate with zero errors.

Current mappings:

- `docs/evidence/CAC-01/capability-baseline.json` → `schemas/capability-probe.schema.json`
- `docs/evidence/CAC-01/source-freeze.json` → `schemas/source-freeze.schema.json`

Adding an unmapped evidence JSON file or an invalid schema blocks CI.

## Local clean gate

```powershell
cargo fmt --all -- --check
cargo check --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
$env:RUSTDOCFLAGS = "-D warnings"
cargo doc --locked --no-deps --document-private-items
Remove-Item Env:RUSTDOCFLAGS
git diff --check
```

Run negative gates through Git Bash or CI:

```bash
for gate in format clippy test docs pspr evidence; do
  bash scripts/ci-negative-gate.sh "$gate"
done
```

`scripts/ci-gitleaks.sh` is Linux-only because its downloaded artifact is hash-pinned for Linux x64. The clean and negative Gitleaks proof is authoritative in GitHub Actions.
