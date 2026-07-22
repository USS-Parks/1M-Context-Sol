#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--artifact", required=True, type=Path)
parser.add_argument("--sha256", required=True)
parser.add_argument("--source-commit", required=True)
parser.add_argument("--output", required=True, type=Path)
args = parser.parse_args()

artifact = args.artifact.resolve(strict=True)
if artifact.name != "1M-Context-Ticker-macOS-universal.dmg":
    raise SystemExit(f"unexpected artifact name: {artifact.name}")
if len(args.sha256) != 64 or any(character not in "0123456789abcdef" for character in args.sha256):
    raise SystemExit("sha256 must be 64 lowercase hexadecimal characters")
if len(args.source_commit) != 40 or any(
    character not in "0123456789abcdef" for character in args.source_commit
):
    raise SystemExit("source commit must be 40 lowercase hexadecimal characters")

manifest = {
    "schema_version": 1,
    "artifact": artifact.name,
    "bytes": artifact.stat().st_size,
    "sha256": args.sha256,
    "source_commit": args.source_commit,
    "bundle_identifier": "com.ussparks.1m-context-ticker",
    "minimum_macos": "13.0",
    "architectures": ["arm64", "x86_64"],
    "dmg_contents": ["1M Context Ticker.app", "Applications"],
    "signed": False,
    "notarized": False,
    "verification": "github-hosted-macos-only",
    "physical_mac_acceptance": False,
}
args.output.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
