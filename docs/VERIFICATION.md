# Verification Ledger

## Current release claim status

No release claim is active. G0–G9 remain open until their prescribed evidence passes on the same release candidate.

## Prompt status

| Prompt | Gate | Status | Evidence |
|---|---|---|---|
| CAC-00 | Repository identity, scope, skeleton checks, first push | Local gate passed; push pending | DEVLOG entry |
| CAC-01 onward | PSPR prompt-local gates | Not started | — |

## Baseline local checks

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
git diff --check
```

Live Sol long-context evidence is intentionally absent at CAC-00.
