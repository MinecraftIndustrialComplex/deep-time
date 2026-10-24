# The pole cap

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

The world wraps east to west like a planet but cannot wrap over the poles: a flat grid of blocks cannot be folded into a sphere there without mirroring whatever crosses. So the playable world ends at a latitude short of each pole (a storm from 70 degrees by default, a hard stop at 80), with a storm that turns players back. The map is Lambert's cylindrical equal-area projection (Ben, 2026-10-02, "Lambert for now"; SCL-049, POL-017): every block covers the same area of the planet, so east to west a block is 300·cos φ m of planet and north to south 300 / cos φ m; shapes are true only on the equator. From 75 degrees the band would be only 49 blocks at C = 16,384 on this map, so Ben moved the cap line to 70 degrees (2026-10-03): 118 blocks from 70 to 80 degrees (235 at 32,768), with 6.0 % of the planet behind the cap line; on the old equirectangular map the 75–80 band was 228 blocks (POL-006). This is geometry and game design, not physics, and the page says what it distorts.

**Since format 0.5 (2026-10-04) the pole cap applies to Lambert worlds only.** The default world is the
seamless Petroff–Guyou world ([seamless-world.md](seamless-world.md); Ben, decisions log 2026-10-04),
whose poles are ordinary, unstretched ground on its north and south edges; it has no pole cap and
nothing "beyond the pole". Its north and south edges carry a temporary border instead until the wrap
engine joins them (POL-022 to POL-024).

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [POL](ledger/POL.md) | pole cap | POL-001 to POL-016 (16) |
| [POL](ledger/POL.md) | projection | POL-017 to POL-021 (5) |
| [POL](ledger/POL.md) | seamless world's edge border (game side) | POL-022 to POL-024 (3) |
| [POL](ledger/POL.md) | cube world's border (game side; page [cube-world-game.md](cube-world-game.md)) | POL-031 (1) |

Rows by status: `game-choice` 21, `uncited` 3.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/SCL.md](extraction-notes/SCL.md).
- Research note: `docs/research/notes/wrapped_planet_worlds.md`.
- Work package in the verification plan: WP-12 ([claims.md](claims.md#verification-plan-for-stage-2)).
