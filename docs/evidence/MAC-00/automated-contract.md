# MAC-00 automated contract evidence

## Result

Passed locally on 2026-07-21. The macOS implementation contract is frozen before product code is added.

## Verified inputs

- Canonical checkout: `C:\Users\17076\Documents\Codex 1M Context Project`, `main`, one registered worktree.
- Baseline HEAD: `8c678f7f5954a644f4cb517daaa33e4ffb56fed2`, two commits ahead of the previously published `7961497cd1f40b54332c71629c1a6267494b8bb7`.
- Preserved excluded lane: `src/lib.rs`, `src/main.rs`, and `src/precompact_guard.rs`; none is part of MAC-00.
- Installed reference: `codex-cli 0.144.5`.
- Shared fixture: five token cases, three selection cases, and four layout cases.
- Shared catalog: one exact `gpt-5.6-sol` entry with `1050000` total context and `900000` automatic compaction.

## Contract proof

```text
python ticker/macos/verify-contract.py
MAC-00 contract passed: 5 token, 3 selection, 4 layout cases; exact host window 1008000; unsupported live claims disabled
```

The checker proves that the active count comes from `last_token_usage.total_tokens`, every shared token fixture uses the required `1008000` host budget, the four-key configuration ownership is exact, unsupported hosts and ambiguous windows hide the panel, and all physical-Mac-only claims remain false.

## Verification boundary

No macOS binary was compiled or run in MAC-00. No physical Mac, real Codex configuration, real login item, GitHub Actions workflow, tag, release, or public artifact was changed. Apple APIs and the hosted-runner label are source-verified in `docs/architecture/MACOS-TICKER-CONTRACT.md`; their later code paths still require the MAC-01 through MAC-03 gates.
