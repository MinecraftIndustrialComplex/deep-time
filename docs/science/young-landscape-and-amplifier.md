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

## One-cell spires (margins and spires task, 2026-10-03)

**What was wrong.** The vertical-scale study saw needles one block wide and 20 to 50 blocks tall on steep ground.
Counted on the baked terrain (an L3 cell, two blocks, standing above all eight neighbours), master 00ef214c had about
450 land cells more than 8 blocks above all their neighbours on each quick 16k planet and 50 to 60 more than 16
(tallest 31 to 33 blocks at 20 m per block); master f545c0bc 132 and 167 over 8 blocks, 12 and 10 over 16, tallest
42 and 20. All of them were in the elevation layer itself, not the detail noise.

**Cause.** Switching the amplifier's terms off one at a time on one planet's young checkpoint: the mountain
character (`amplify.ruggedness` 0) removed them (over 8 blocks: 450 → 3); within it the gullies (450 → 21); the glacial
pass (450 → 67). Two inputs of the character varied from one cell to the next: (1) the fade over glacial troughs was
taken from each cell's own glacial cut, which jumps between neighbours at the ice margin and over the bed's rills, so
single cells kept the full character between faded ones (removing that fade alone: 777 → 64 on the run that was
used); (2) each gully octave took its direction from a one-cell gradient of the terrain plus the coarser octaves,
which turns freely at summits, saddles and the coarser gullies' crests and axes, so neighbouring cells read
unrelated stripes (with both gone: 3).

**Fix.** The trough fade is a box mean of the cut over three cells (AMP-072); each gully octave takes the fall line
at a quarter of its wavelength from five gradients and fades where they disagree (AMP-073). Test
`surface::amplify::tests::a_glaciated_young_range_has_no_one_cell_spires`: on a young, steep, glaciated range at
ruggedness 2 no cell stands more than 6 blocks (120 m at the test's 20 m per block) above all eight neighbours; the
old code left 14. Meant exceptions, not on that planet: a volcano's summit and an impact crater's central peak.

**Measured** (master f545c0bc before, this change after; land cells over all eight neighbours by more than 8 / 16
blocks, tallest): planet A 132 / 12 / 41.6 → 3 / 0 / 11.6; planet B 167 / 10 / 19.8 → 0 / 0 / 6.0.

