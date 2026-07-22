#!/usr/bin/env python3
import hashlib
import json
import sys
from pathlib import Path

if len(sys.argv) != 2:
    raise SystemExit("usage: verify-release.py OUTPUT_DIRECTORY")

output = Path(sys.argv[1]).resolve(strict=True)
artifact_name = "1M-Context-Ticker-macOS-universal.dmg"
checksum_name = f"{artifact_name}.sha256"
manifest_name = "1M-Context-Ticker-macOS-universal.manifest.json"
expected_files = {artifact_name, checksum_name, manifest_name}
actual_files = {path.name for path in output.iterdir()}
if actual_files != expected_files:
    raise SystemExit(f"unexpected release files: {sorted(actual_files)}")

artifact = output / artifact_name
digest = hashlib.sha256(artifact.read_bytes()).hexdigest()
checksum = (output / checksum_name).read_text(encoding="utf-8")
if checksum != f"{digest}  {artifact_name}\n":
    raise SystemExit("checksum file does not match the final DMG bytes")

manifest = json.loads((output / manifest_name).read_text(encoding="utf-8"))
expected_manifest = {
    "schema_version": 1,
    "artifact": artifact_name,
    "bytes": artifact.stat().st_size,
    "sha256": digest,
    "bundle_identifier": "com.ussparks.1m-context-ticker",
    "minimum_macos": "14.0",
    "architectures": ["arm64", "x86_64"],
    "dmg_contents": ["1M Context Ticker.app", "Applications"],
    "signed": False,
    "notarized": False,
    "verification": "github-hosted-macos-only",
    "physical_mac_acceptance": False,
}
for key, value in expected_manifest.items():
    if manifest.get(key) != value:
        raise SystemExit(f"manifest mismatch for {key}: {manifest.get(key)!r}")
source_commit = manifest.get("source_commit", "")
if len(source_commit) != 40 or any(character not in "0123456789abcdef" for character in source_commit):
    raise SystemExit("manifest source_commit is not a full lowercase Git SHA")

print(
    f"MAC-03 release verified: {artifact.name}, {artifact.stat().st_size} bytes, "
    f"sha256 {digest}, arm64+x86_64, unsigned automated-only evidence"
)
