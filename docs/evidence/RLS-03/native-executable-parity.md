# RLS-03 native executable parity

Date: 2026-07-20

## Build result

- Artifact: `1M-Context-Ticker-Windows-x64.exe`
- Target: native C# WPF/Win32, .NET Framework 4.8, x64
- Version: `0.1.0.0`
- Size: `37376` bytes
- First-build SHA-256: `f3e3697611b95b92c2244bf965609ecbfa48b64e54e273bfdd971b8101c8f733`
- Clean canonical rebuild SHA-256: `7be18efe2266d4e779808f52ea8056cefc3588c14cf13cb15a7f2870ec2fc2b9`
- PE machine: `0x8664` (AMD64)
- Third-party runtime assemblies: none

The executable was compiled with warnings-as-errors using the in-box .NET Framework 4.8 compiler and GAC WPF assemblies. No .NET SDK or downloaded dependency was added.

The two source-identical builds have equal size/version/architecture and both pass self-test, but their hashes differ. Reproducible binary normalization/deterministic compilation remains an explicit RLS-05 gate; RLS-03 does not claim reproducibility.

## Shared self-test

The executable consumed `ticker/fixtures/behavior-cases.json` directly:

- token cases: 5 passed
- selection cases: 3 passed
- layout cases: 4 passed
- malformed input failure: passed

## Live Desktop parity

The accepted PowerShell helper was stopped without changing its installed files or shortcuts. The native executable then ran against the same active Codex task.

| Field | Native result |
|---|---:|
| PID | `18360` |
| Process | `1M-Context-Ticker-Windows-x64.exe` |
| Session | `019f7e5d-bbfa-7922-ba42-71b52f309a39` |
| Active tokens | `364096` |
| Host context window | `1008000` |
| Remaining tokens | `643904` |
| Visible | `true` |
| Native rectangle | `820,967,1010,985` |
| Prompt background | `#2D2D2D` |
| Prompt center | `915` |
| Sidebar open | `true` |
| No-activate/tool-window styles | `true` / `true` |
| Runtime error | none |

The heartbeat advanced across a three-second sample. A duplicate launch exited `0` and left exactly one native process. Process inspection found zero child processes and zero PowerShell children.

## Gate decision

W-G2 passes. The native executable matches the accepted runtime behavior without hosting or invoking PowerShell. RLS-04 may migrate the user-local lifecycle and shortcuts while retaining rollback to the reference files.
