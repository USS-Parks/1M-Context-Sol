# M0 feasibility verdict

## Decision

**NO-GO for implementation under the current supported Codex Desktop extension surfaces.**

The host-configuration half is feasible and was proven in the real Desktop app. The required native prompt-pill UI half is blocked because there is no supported dynamic insertion seam.

| Required seam | Result | Evidence |
|---|---|---|
| Fresh Desktop task adopts exact `gpt-5.6-sol` | Passed | CDS-01 disposable task |
| 1,050,000 total / 1,008,000 effective host policy | Passed at configuration/adoption layer | CDS-01 catalog and rollout evidence |
| 900,000 automatic-compaction configuration | Passed at startup configuration layer; boundary behavior not yet tested | CDS-01 config evidence; G3 remains open |
| Host-authoritative live token state | Passed | `thread/tokenUsage/updated` protocol contract |
| Dynamic component inside native prompt pill | Blocked | No supported plugin/MCP/renderer extension contract |
| Live changing dial in the prompt pill | Not runnable | Depends on the missing component seam |

## Consequence

CDS-03 and all CDS-10 through CDS-33 implementation prompts are not authorized to run because their prerequisite G1 did not pass. The repository must not claim a deployable product.

The 1M host configuration can work locally, but shipping it alone would omit the specifically required native dial. The PSPR forbids declaring a TUI, static composer icon, MCP status response, embedded MCP app, separate panel, or separate window equivalent.

## Local-machine state at stop

- The temporary real-profile keys were removed.
- Codex Desktop was restarted onto the restored profile.
- The five later values written by the Microsoft Store update were preserved.
- The disposable acceptance task was archived.
- No installed package was modified.
- No long-context/costly probe was run.
- No external issue, pull request, release, or repository push was made.
- The historical dirty source files remain preserved and excluded from CDS commits.

## User decision required

Proceed only through a separately approved CDS-X1 route, or revise the product contract to remove/change the native prompt-pill requirement. Until then the initiative is blocked, not deployable.
