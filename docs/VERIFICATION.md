# Verification Ledger

## Current release claim status

No release claim is active. G0–G9 remain open until their prescribed evidence passes on the same release candidate.

## Prompt status

### Corrected desktop initiative (`CDS1M-PSPR-1`)

| Prompt | Gate | Status | Evidence |
|---|---|---|---|
| CDS-00 | Corrected scope, preservation, sole active roster | Passed | `43b606e293399e1ff4caa67e30c649df9c2021f7`, ledger follow-up `467d46f`, and `docs/evidence/CDS-00/` |
| CDS-01 | Fresh Desktop task adopts exact Sol-1M host configuration | Passed | `docs/evidence/CDS-01/desktop-config-adoption.md`; task `019f7fd8-2c21-7753-a4bf-6b8988326e8f` reported exact Sol and `1008000` effective tokens |
| CDS-02 | Host token state plus live native prompt-pill component seam | Blocked / hard stop | Token event exists; no supported dynamic prompt-pill insertion contract. See `docs/evidence/CDS-02/` |
| CDS-03 onward | Architecture freeze and product implementation | Not run | G1 prerequisite failed at CDS-02; see `docs/evidence/M0/feasibility-verdict.md` |
| CDO-00 | Approved floating-overlay contract and execution authority | Passed | `919aba898399ee409f0b54318d6cb5583eb9d463`, addendum, and DEVLOG |
| CDO-01 | Active-context parser, root-task selection, and real-window anchor | Passed | `f7496d357de2d111a6aa1e005a4e4fe9a558c3f0`, focused test, and `docs/evidence/CDO-01/` |
| CDO-02 | Focusless floating context dial | Passed | `882351c7f5e6daf8bb01f6b625536bc4369ff426` and live no-activate/DPI/region/lifecycle proof |
| CDO-03 | Safe four-key install, user shortcuts, rollback, and corrected live refresh | Passed | `01157263c7e9b7090882e58ffc9ccc29a51b7a41`, UX correction `557b0dedf8bdb8d152db6d952512f565e100190e`, and `docs/evidence/CDO-03/` |

### Windows executable release (`1MCT-R1`)

| Prompt | Gate | Status | Evidence |
|---|---|---|---|
| RLS-00 | Windows-only authority, preservation, toolchain, and exclusions | Passed | `d0d9cacb504f946f6537c22e8c7c89ae2200b4aa`, release addendum, and release DEVLOG |
| RLS-01 | Official repository welcome-page copy without imagery | Passed | `2a7211911dee95d500524f5429dd196b01a8813d`, root `README.md`, and release DEVLOG |
| RLS-02 | Shared active-context, selection, and layout fixtures | Passed for reference | `368fcbba1a91006c37ab1b1b6da4ef60525e452c`, shared fixtures, and `docs/evidence/RLS-02/`; executable consumption required by RLS-03 |
| RLS-03 | Native C# WPF executable and live parity | Passed | `c0389222d9e95d6a8a82607d56160d9de165392f` and `docs/evidence/RLS-03/native-executable-parity.md` |
| RLS-04 | Native executable lifecycle, rollback, and installed pill correction | Passed | `c512d2016257eaf852a54655e209054f9a374bae` and `docs/evidence/RLS-04/native-executable-lifecycle.md` |
| RLS-05 | Reproducible Windows artifact, checksum, manifest, and CI job | Passed locally and hosted | `e3d8e39028446e37eeb25d8cbb74335b0df39622`, governance repair `29b57c94ecfa101584ccdddfac837f165ad492a5`, green run [29836629916](https://github.com/USS-Parks/1M-Context-Sol/actions/runs/29836629916), and `docs/evidence/RLS-05/windows-release-artifact.md` |
| RLS-06 | Final Windows acceptance and closeout | Passed locally | Exact-window fail-closed ticker, measured full face, no hover UI, click-through input, direct `one_m_context_verified` status, lifecycle/rebuild/reinstall proof, and `docs/evidence/RLS-06/windows-acceptance-and-closeout.md` |

### Historical Context Continuum initiative

| Prompt | Gate | Status | Evidence |
|---|---|---|---|
| CAC-00 | Repository identity, scope, skeleton checks, first push | Passed | `f46254ca0967d0a9f04bfc9e900b525390432619` and DEVLOG entry |
| CAC-01 | Installed Codex/Sol discrepancy, sanitized probe, fixtures, source freeze | Passed | `8ff3cd7d0e8ed98f09dc095832cc9b6848602759`, `docs/evidence/CAC-01/`, and capability baseline |
| CAC-02 | Claim vocabulary, limitation matrix, gate map, and fail-closed wording/model tests | Passed | `d21716c34bda05e80f9d328c7814c77cf626be4b`, capability vocabulary, and claim-contract tests |
| CAC-03 | ADRs, sole authorities, trust transitions, data flow, repository threat model | Passed | `907254aca8a7228d5af7561004cb220921590362`, architecture contract/tests, and threat model |
| CAC-04 | Rust/docs, PSPR/schema, secrets, supply-chain CI plus negative proofs | Passed | `10c749c5dcecab66ce51ee008dff669ff2952fdd`, repair `6645fce3793a8e74dda72765084a507de1ba1f91`, and green Actions run [29704751317](https://github.com/USS-Parks/Codex-Added-Context/actions/runs/29704751317) |
| CAC-10 | Versioned Sol-only catalog generation, preservation, drift guards, installed-parser proof | Passed | `359c572fb2775a4f68608ff8e3a333dfdc31abfb`, catalog schemas/tests, CAC-10 manifest, and green Actions run [29705566278](https://github.com/USS-Parks/Codex-Added-Context/actions/runs/29705566278) |
| CAC-11 | Atomic owned-setting apply, conflict refusal, and exact-byte rollback | Passed | `279819fe11c00a589dce82569fc4ac5631d06af5`, ownership schema, 12 config-manager tests, isolated CLI proof, and green Actions run [29707154346](https://github.com/USS-Parks/Codex-Added-Context/actions/runs/29707154346) |
| CAC-12 | Claim-safe native-window doctor/status, exit codes, schemas, and eight golden scenarios | Passed | `ad14add5edc520aa423148a6f013ded07cec6dc9`, doctor/status schemas, golden hashes, live read-only baseline proof, and green Actions run [29708215480](https://github.com/USS-Parks/Codex-Added-Context/actions/runs/29708215480) |
| CAC-13 | Strict exact-Sol SessionStart/UserPromptSubmit policy and per-task override audit | Passed | `d8dad4ff80747d8cb69e69db53d3c5b4b8781be3`, 11 startup-policy tests, dual-boundary live 272k block proof, and green Actions run [29709432946](https://github.com/USS-Parks/Codex-Added-Context/actions/runs/29709432946) |
| CAC-14 | Sol-1M catalog calibration, measured overhead, probe bands, meter, and executable Codex launcher | In progress | Codex parser resolved exact `1050000/1050000/96/900000`; live request and compaction-block proof remain open |
| CAC-15 onward | PSPR prompt-local gates | Not started | — |

## Baseline local checks

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
git diff --check
```

Live Sol long-context evidence is intentionally absent at CAC-00.
