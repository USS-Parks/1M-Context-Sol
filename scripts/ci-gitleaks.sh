#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT

version="8.30.1"
archive="$scratch/gitleaks.tar.gz"
binary="$scratch/gitleaks"
url="https://github.com/gitleaks/gitleaks/releases/download/v${version}/gitleaks_${version}_linux_x64.tar.gz"
expected_sha256="551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb"

curl --fail --location --silent --show-error "$url" --output "$archive"
printf '%s  %s\n' "$expected_sha256" "$archive" | sha256sum --check -
tar --extract --gzip --file "$archive" --directory "$scratch" gitleaks

"$binary" git --no-banner --redact "$repo_root"

mkdir -p "$scratch/negative"
prefix="sk-proj-"
body="ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789ABCDEFGH"
printf 'OPENAI_API_KEY=%s%s\n' "$prefix" "$body" >"$scratch/negative/generated.env"

set +e
"$binary" dir --no-banner --redact "$scratch/negative" >"$scratch/negative.log" 2>&1
code=$?
set -e
if [[ $code -ne 1 ]]; then
  echo "Gitleaks negative fixture returned $code instead of the required finding exit code 1"
  cat "$scratch/negative.log"
  exit 1
fi
echo "Gitleaks clean scan passed and generated negative fixture was blocked"
