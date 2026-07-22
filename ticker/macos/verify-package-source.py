#!/usr/bin/env python3
import json
import plistlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MAC = ROOT / "ticker" / "macos"

build = (MAC / "build-release.sh").read_text(encoding="utf-8")
manifest = (MAC / "create-manifest.py").read_text(encoding="utf-8")
release_verifier = (MAC / "verify-release.py").read_text(encoding="utf-8")
instructions = (ROOT / "docs/MACOS.md").read_text(encoding="utf-8")
with (MAC / "Info.plist").open("rb") as handle:
    info = plistlib.load(handle)

for required in [
    "arm64-apple-macosx14.0",
    "x86_64-apple-macosx14.0",
    "-verify_arch arm64 x86_64",
    "1M-Context-Ticker-macOS-universal.dmg",
    'ln -s /Applications',
    'logical_count',
    'hdiutil create',
    'hdiutil attach',
    'shasum -a 256',
]:
    assert required in build, f"missing package gate: {required}"

for forbidden in [
    "codesign",
    "notarytool",
    "altool",
    "spctl --master-disable",
    "xattr -d com.apple.quarantine",
]:
    assert forbidden not in build, f"forbidden package operation: {forbidden}"

assert info["CFBundleIdentifier"] == "com.ussparks.1m-context-ticker"
assert info["LSMinimumSystemVersion"] == "14.0"
assert info["LSUIElement"] is True

for required in [
    '"architectures": ["arm64", "x86_64"]',
    '"signed": False',
    '"notarized": False',
    '"physical_mac_acceptance": False',
]:
    assert required in manifest, f"missing manifest boundary: {required}"

assert "shasum -a 256 -c 1M-Context-Ticker-macOS-universal.dmg.sha256" in instructions
assert "Control-click" in instructions
assert "System Settings > Privacy & Security" in instructions
assert "--action uninstall" in instructions
assert "never disable Gatekeeper system-wide" in instructions

contract = json.loads((MAC / "contract.json").read_text(encoding="utf-8"))
assert contract["build"]["artifact"] == "1M-Context-Ticker-macOS-universal.dmg"
assert contract["product"]["architectures"] == ["arm64", "x86_64"]
assert 'expected_files = {artifact_name, checksum_name, manifest_name}' in release_verifier
assert 'checksum file does not match the final DMG bytes' in release_verifier

print("MAC-03 package source passed: universal app, minimal DMG, checksum, manifest, safe open flow")
