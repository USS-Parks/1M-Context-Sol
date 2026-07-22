# MAC-03 universal DMG evidence

## Result

M-G4 passed on the GitHub-hosted `macos-15` runner at source commit `1ff0b541800cd61dc5dd51cf00b4c8b357d5c4f3`.

- Workflow run: [29888694993](https://github.com/USS-Parks/1M-Context-Sol/actions/runs/29888694993)
- Job: `macOS native ticker`
- Full workflow: all twelve jobs passed
- Uploaded artifact ID: `8517461310`
- Artifact name: `1M-Context-Ticker-macOS-universal`
- Artifact archive: 297,870 bytes; SHA-256 `0012e811bdf8e5889b1c323d83c7584d45a0719802c722161ca19bfb5da7c0ab`
- Retention: fourteen days

## Final DMG

- File: `1M-Context-Ticker-macOS-universal.dmg`
- Bytes: 296,637
- SHA-256: `1df281bb9da3fb7b20efca67a8a9dfb0d6e5ab796201f745e9459bc666b2c535`
- Architectures: `arm64` and `x86_64`
- Bundle identifier: `com.ussparks.1m-context-ticker`
- Minimum ticker target: macOS 13.0
- Signing: none
- Notarization: none

The build verified both slices before packaging and again from the mounted DMG. The mounted image contained exactly the app and an `/Applications` link as logical top-level entries. The checked app bundle contained the frozen `Info.plist`, universal executable, and exact shared Sol catalog.

The uploaded candidate contains exactly three files:

- `1M-Context-Ticker-macOS-universal.dmg`
- `1M-Context-Ticker-macOS-universal.dmg.sha256`
- `1M-Context-Ticker-macOS-universal.manifest.json`

After the hosted job completed, the artifact was downloaded into a unique temporary directory and `ticker/macos/verify-release.py` independently recomputed its DMG hash, byte count, checksum-file contents, manifest fields, source commit, filenames, and automated-only boundary. The verification passed.

## Repair history

Candidate `572a8668905b95b51622bbe6cb9ae1293ab4f51c` built both architecture slices and the app, then failed because the filename followed rather than preceded `lipo -verify_arch`. Commit `1ff0b541800cd61dc5dd51cf00b4c8b357d5c4f3` corrected the command order. The next run built, mounted, verified, checksummed, manifested, and uploaded the DMG successfully.

## User-facing boundary

`docs/MACOS.md` requires SHA-256 verification before a per-app Control-click/Open or Privacy & Security approval. It never instructs users to remove quarantine metadata or disable Gatekeeper system-wide. The README lists the exact macOS filename at the top while stating that public download still requires final release approval.

No physical-Mac composer placement, real login launch, real Codex configuration change, Gatekeeper interaction, tag, or GitHub Release is claimed or performed.

## macOS 14 release-floor rebuild

Before publication, the user raised the shipping floor to macOS 14.0. The macOS 13 artifact above remains historical and was not published. Commit `ef7b779c794f654c431a8c0f9fddf1040d7a3ed9` rebuilt the universal app and DMG with a macOS 14.0 target. Run [29890495321](https://github.com/USS-Parks/1M-Context-Sol/actions/runs/29890495321) passed all twelve jobs and produced artifact `8518078853`.

The replacement DMG is 295,092 bytes with SHA-256 `0a92ce6382793be6852e9b7484a6393bcd76e57d07cc1e7e5b3b5fbc2867c788`. Its manifest records macOS 14.0, `arm64` plus `x86_64`, the exact source commit, unsigned/unnotarized status, and automated-only verification. This is the artifact published in v0.1.0.
