# The young landscape and the amplifier

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

The deep-time simulation works on cells several kilometres wide. Two later steps bring the terrain down to the size of a block. The young landscape reruns erosion for the last stretch of history on a finer grid, so that valleys and ridges are sharp where the land is young. The amplifier then adds block-scale form: it carves branching valleys with a stream-power kernel (after Schott and others 2024), limits slopes, and gives mountains a character that depends on their rock and age. Nearly every amplifier constant was tuned by eye to look right and is marked `game-choice`; this page must say so plainly.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [YNG](ledger/YNG.md) | young landscape | YNG-001 to YNG-014 (14) |
| [YNG](ledger/YNG.md) | detail noise | YNG-015 to YNG-021 (7) |
| [AMP](ledger/AMP.md) | amplifier kernels | AMP-001 to AMP-022 (22) |
| [AMP](ledger/AMP.md) | incision | AMP-023 to AMP-033 (11) |
| [AMP](ledger/AMP.md) | mountain character | AMP-034 to AMP-066 (33) |
| [AMP](ledger/AMP.md) | resampling | AMP-067 to AMP-071 (5) |

Rows by status: `cited-unverified` 9, `calibrated` 1, `game-choice` 62, `uncited` 20.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/AMP.md](extraction-notes/AMP.md), [extraction-notes/HYD.md](extraction-notes/HYD.md).
- Work package in the verification plan: WP-6 ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- `coarse_iteration_factor` is coded 2.0 where the build log and plan say 3 (extraction notes AMP, flag 1).
- The thermal slope comment says "Schott: tan 30 degrees = 0.57" and the default is 0.6. In Schott's published shader 0.57 is used only on a non-default branch; the default threshold is a tangent of 0.9 to 1.4 (AMP-034, AMP-038).
