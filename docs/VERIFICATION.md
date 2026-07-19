# Verification Ledger

## Current release claim status

No release claim is active. G0–G9 remain open until their prescribed evidence passes on the same release candidate.

## Prompt status

| Prompt | Gate | Status | Evidence |
|---|---|---|---|
| CAC-00 | Repository identity, scope, skeleton checks, first push | Passed | `f46254ca0967d0a9f04bfc9e900b525390432619` and DEVLOG entry |
| CAC-01 | Installed Codex/Sol discrepancy, sanitized probe, fixtures, source freeze | Passed | `8ff3cd7d0e8ed98f09dc095832cc9b6848602759`, `docs/evidence/CAC-01/`, and capability baseline |
| CAC-02 | Claim vocabulary, limitation matrix, gate map, and fail-closed wording/model tests | Passed | `d21716c34bda05e80f9d328c7814c77cf626be4b`, capability vocabulary, and claim-contract tests |
| CAC-03 | ADRs, sole authorities, trust transitions, data flow, repository threat model | Passed | `907254aca8a7228d5af7561004cb220921590362`, architecture contract/tests, and threat model |
| CAC-04 | Rust/docs, PSPR/schema, secrets, supply-chain CI plus negative proofs | Passed | `10c749c5dcecab66ce51ee008dff669ff2952fdd`, repair `6645fce3793a8e74dda72765084a507de1ba1f91`, and green Actions run [29704751317](https://github.com/USS-Parks/Codex-Added-Context/actions/runs/29704751317) |
| CAC-10 | Versioned Sol-only catalog generation, preservation, drift guards, installed-parser proof | Passed | `359c572fb2775a4f68608ff8e3a333dfdc31abfb`, catalog schemas/tests, CAC-10 manifest, and green Actions run [29705566278](https://github.com/USS-Parks/Codex-Added-Context/actions/runs/29705566278) |
| CAC-11 | Atomic owned-setting apply, conflict refusal, and exact-byte rollback | Passed | `279819fe11c00a589dce82569fc4ac5631d06af5`, ownership schema, 12 config-manager tests, isolated CLI proof, and green Actions run [29707154346](https://github.com/USS-Parks/Codex-Added-Context/actions/runs/29707154346) |
| CAC-12 onward | PSPR prompt-local gates | Not started | — |

## Baseline local checks

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
git diff --check
```

Live Sol long-context evidence is intentionally absent at CAC-00.
