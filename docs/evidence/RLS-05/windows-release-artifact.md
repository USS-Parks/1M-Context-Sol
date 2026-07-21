# RLS-05 Windows release artifact

Date: 2026-07-21

## Result

W-G4 passed locally. Two clean source-identical builds produced byte-identical executables, checksum files, and artifact manifests. The exact final executable was installed and accepted live without a PowerShell ticker process or any Codex process control.

The new Windows CI job is source-verified but has not run on GitHub because RLS-05 push authorization has not been granted. Hosted CI is pending, not passed.

## Deterministic build

The in-box .NET Framework compiler `4.8.9221.0` rejects `/deterministic+`. Two raw outputs differed only in the PE timestamp, one compiler-generated `PrivateImplementationDetails` GUID string, and the matching module MVID.

`ticker/windows/build.ps1` now derives a stable GUID from the ordered source names and bytes, replaces exactly one matching identity string and MVID, zeros the PE timestamp, and refuses any unexpected binary shape. This fixed-length normalization does not rewrite IL, methods, resources, assembly identity, version, platform, or dependencies.

Verification command:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\ticker\windows\verify-release.ps1 -OutputDirectory .\dist
```

The verifier builds twice in separate fresh temporary directories, compares all release files, then requires final `dist` to match those clean-build hashes. Temporary directories are removed.

## Artifact

- clean reproducible builds: 2
- executable: `dist/1M-Context-Ticker-Windows-x64.exe`
- bytes: 37,376
- SHA-256: `29051b7ba096f466e3361796c4bb674a9b5be22e3d484d65cf10cb9f506830e3`
- checksum SHA-256: `0d0440646d4b5acb668282e8e2d87438882292d1d7c24fcb0189289e0fe59ebb`
- manifest SHA-256: `c92cef1a77dfcc6fa036063c6adf4172c32d506b949b9bebed053b226c43749b`
- architecture: `amd64`
- product/version: `1M Context Ticker` / `0.1.0.0`
- target: `.NET Framework 4.8`
- deterministic MVID: `6ba0642e-ce0b-a1e3-03b4-8dea46604944`
- source-seed SHA-256: `1a97d18f8da20495f0a222e00e9004c62d7a2c0d161cf39a79d164f3a92ca96d`

`dist` contains exactly the executable, its `.sha256` file, and `artifact-manifest.json`. The schema-2 manifest records compiler normalization, exact source hashes, fixture hash, self-test counts, assembly identity, dependency allowlist, size, and artifact hash.

## Verification

- shared self-test: 5 token, 3 selection, and 4 layout cases passed
- exact dependencies: `mscorlib`, `PresentationCore`, `PresentationFramework`, `System`, `System.Core`, `System.Web.Extensions`, and `WindowsBase`
- no third-party runtime DLL or PowerShell runtime dependency
- installed artifact hash matched final artifact and install manifest
- live PID: `18684`; child processes: 0; PowerShell ticker processes: 0
- live geometry: `198x30`; no-activate: true; runtime error: none
- active task: `019f840f-d35e-7993-a11a-c6c68917c7c0`
- live state at capture: 182,520 used, 825,480 remaining, 1,008,000 effective window
- Codex process IDs unchanged across rollback, installation, and start
- PowerShell reference snapshot and exact pre-native manifest remain retained

## CI and boundaries

`.github/workflows/ci.yml` adds an independent `windows-executable` job. It verifies two source-identical builds in a runner-temporary directory and leaves all existing Rust, governance, negative, secret, and supply-chain jobs unchanged.

Hosted compiler security-patch versions can drift, so CI proves reproducibility within its recorded toolchain rather than requiring a cross-toolchain hash match to the local artifact.

The excluded `src/lib.rs`, `src/main.rs`, and `src/precompact_guard.rs` remain uncommitted and byte-preserved. Signing, RLS-05 push, GitHub Release publication, imagery, macOS/DMG, and paid large-token proof remain outside this prompt.
