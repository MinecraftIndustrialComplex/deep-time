# Folds, faults and intrusions

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

After rock is laid down it is bent into folds, broken along faults (fractures along which the two sides have slipped) and cut by sheets and plugs of magma. The mod stores each such event and, for any column of the world, undoes them in reverse order to find which original layer ends up there. The method and its mathematics are a port of Noddy, a kinematic structural-geology program (Jessell 1981; Jessell & Valenta 1996), checked bit for bit against the original. What Noddy does not supply is where, how many and how large: those counts, sizes, dips and their dependence on the plate history are this mod's own rules.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [DEF](ledger/DEF.md) | folds | DEF-001 to DEF-012 (12) |
| [DEF](ledger/DEF.md) | faults | DEF-013 to DEF-039 (27) |
| [DEF](ledger/DEF.md) | intrusions (Noddy) | DEF-040 to DEF-053 (14) |
| [DEF](ledger/DEF.md) | structure on the ground | DEF-054 to DEF-062 (9) |
| [DEF](ledger/DEF.md) | Noddy port | DEF-063 to DEF-074 (12) |

Rows by status: `cited-unverified` 16, `game-choice` 54, `uncited` 4.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/RCK.md](extraction-notes/RCK.md).
- Build log: STATUS.md 2026-10-01, "P2-01 stage A" and "P2-01 stage B".
- Attribution: `NOTICE.md`; the GPL original is used only as a test oracle under `tools/validation/noddy`.
- Work package in the verification plan: WP-8 ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- Fault dips cited to Anderson 1951 (55 to 70 degrees normal, 20 to 40 thrust) are applied in block space, which is stretched 15 times vertically, so the real dips are about 6.6 and 2.2 degrees (extraction notes RCK, flag 3).
- Slip ratios are coded 0.05 / 0.08 / 0.05 against a cited value of about 0.03 (flag 4).
- Noddy's licence: the snapshot the port names (Loop3D/noddyverse) is MIT; other releases of the same program are GPL. An LGPL port of the MIT snapshot is compatible if its notices are kept (spot check DEF-063).
