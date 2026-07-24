# 1M Context Sol Linux/Paseo DEVLOG

## LNX-01 — Headless Linux/Paseo policy lifecycle

Status: passed locally; hosted CI and live Linux/Paseo budget proof remain unclaimed.

Scope:

- add `scripts/linux/manage-sol-policy` and its Python standard-library manager;
- derive a one-model catalog from exact Codex 0.145.0 source metadata;
- own only the four approved Codex config keys;
- add plan/install/status/verify/uninstall, documentation, tests, and `ubuntu-latest` CI;
- preserve the existing user-owned dirty Rust tree without modification.

Verification passed on 2026-07-23: 11 Linux lifecycle/negative tests, Python compilation, Bash syntax, ShellCheck 0.11.0, official Codex 0.145.0 catalog generation, `cargo fmt --all -- --check`, warnings-denied Clippy, all Rust targets, warnings-denied rustdoc, `cargo deny check`, and `git diff --check`. The first full Rust run exposed an AGENTS routing-string regression; the focused governance test and subsequent full run passed after correction. No live model request or Paseo daemon restart was performed. Hosted CI and live host-budget proof remain pending.
