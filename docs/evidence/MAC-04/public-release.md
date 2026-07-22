# MAC-04 public release evidence

## Result

M-G5 passed. The dual-platform v0.1.0 release is public:

- Release: https://github.com/USS-Parks/1M-Context-Sol/releases/tag/v0.1.0
- Release ID: `357774461`
- Tag: `v0.1.0`
- Tag and target SHA: `ef7b779c794f654c431a8c0f9fddf1040d7a3ed9`
- Draft: false
- Prerelease: false

## Published assets

| Asset | Bytes | SHA-256 |
|---|---:|---|
| `1M-Context-Ticker-Windows-x64.exe` | 38,400 | `f62558811f95866c4284ea2f68ce06355805230179735c74cbae1244c0337f56` |
| `1M-Context-Ticker-Windows-x64.exe.sha256` | 100 | `aa895166094c4886165e2acf3b69dd8bb8aa3cfeb20ebc868830886f1b1ecda4` |
| `artifact-manifest.json` | 2,344 | `a94bbd774177cb669ed1af1a3a65903f69221143a35e2363a351de89f74491c9` |
| `1M-Context-Ticker-macOS-universal.dmg` | 295,092 | `0a92ce6382793be6852e9b7484a6393bcd76e57d07cc1e7e5b3b5fbc2867c788` |
| `1M-Context-Ticker-macOS-universal.dmg.sha256` | 104 | `07159c022d49302afa418c1583f6a30c8a6c82c010612d4fdb927663e8263064` |
| `1M-Context-Ticker-macOS-universal.manifest.json` | 575 | `ea76fe82f7aaeef775904f3c183b75c6ba3fc215f436ae22b06113ee2641a87e` |

GitHub's release API reported the same size and digest for every uploaded asset. The macOS DMG was independently downloaded from green run `29890495321` before publication and passed the final-byte verifier. Its manifest requires macOS 14.0 and records both `arm64` and `x86_64` slices.

The README links directly to the permanent release downloads. The release notes provide checksum-first installation, per-app Gatekeeper opening, lifecycle behavior, exact-window failure behavior, and links to the macOS instructions and green CI.

No physical-Mac composer placement or real login-item/configuration test is claimed.
