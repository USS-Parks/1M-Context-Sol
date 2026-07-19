#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
gate="${1:-}"
scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT

expect_failure() {
  local label="$1"
  shift
  set +e
  "$@" >"$scratch/$label.log" 2>&1
  local code=$?
  set -e
  if [[ $code -eq 0 ]]; then
    echo "negative gate unexpectedly passed: $label"
    cat "$scratch/$label.log"
    exit 1
  fi
  echo "negative gate blocked as required: $label (exit $code)"
}

write_manifest() {
  local directory="$1"
  mkdir -p "$directory/src"
  printf '%s\n' \
    '[package]' \
    'name = "intentional-ci-failure"' \
    'version = "0.0.0"' \
    'edition = "2024"' \
    >"$directory/Cargo.toml"
}

case "$gate" in
  format)
    mkdir -p "$scratch/format"
    printf '%s\n' 'fn main(){println!("intentional");}' >"$scratch/format/unformatted.rs"
    expect_failure format rustfmt --check "$scratch/format/unformatted.rs"
    ;;
  clippy)
    write_manifest "$scratch/clippy"
    printf '%s\n' \
      'pub fn needless(value: bool) -> bool {' \
      '    if value { true } else { false }' \
      '}' \
      >"$scratch/clippy/src/lib.rs"
    expect_failure clippy cargo clippy \
      --manifest-path "$scratch/clippy/Cargo.toml" \
      --target-dir "$scratch/target-clippy" \
      -- -D warnings
    ;;
  test)
    write_manifest "$scratch/test"
    printf '%s\n' \
      '#[cfg(test)]' \
      'mod tests {' \
      '    #[test]' \
      '    fn intentional_failure() { assert!(false); }' \
      '}' \
      >"$scratch/test/src/lib.rs"
    expect_failure test cargo test \
      --manifest-path "$scratch/test/Cargo.toml" \
      --target-dir "$scratch/target-test"
    ;;
  docs)
    write_manifest "$scratch/docs"
    printf '%s\n' \
      '#![deny(rustdoc::broken_intra_doc_links)]' \
      '/// Deliberately broken link to [MissingType].' \
      'pub fn documented() {}' \
      >"$scratch/docs/src/lib.rs"
    expect_failure docs env RUSTDOCFLAGS="-D warnings" cargo doc \
      --manifest-path "$scratch/docs/Cargo.toml" \
      --target-dir "$scratch/target-docs" \
      --no-deps
    ;;
  pspr)
    expect_failure pspr env \
      CCTX_PSPR_PATH="$repo_root/tests/fixtures/ci-failures/invalid-pspr.md" \
      cargo test --manifest-path "$repo_root/Cargo.toml" --locked \
      --test governance_contracts canonical_pspr_is_structurally_valid -- --exact
    ;;
  evidence)
    expect_failure evidence env \
      CCTX_EVIDENCE_PATH="$repo_root/tests/fixtures/ci-failures/invalid-capability-evidence.json" \
      cargo test --manifest-path "$repo_root/Cargo.toml" --locked \
      --test governance_contracts mapped_evidence_conforms_to_schema -- --exact
    ;;
  *)
    echo "usage: $0 {format|clippy|test|docs|pspr|evidence}" >&2
    exit 2
    ;;
esac
