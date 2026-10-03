# The pole cap

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

The world wraps east to west like a planet but cannot wrap over the poles: a flat grid of blocks cannot be folded into a sphere there without mirroring whatever crosses. So the playable world ends at a latitude short of each pole (75 degrees by default, a hard stop at 80), with a storm that turns players back. The map is Lambert's cylindrical equal-area projection (Ben, 2026-10-02, "Lambert for now"; SCL-049, POL-017): every block covers the same area of the planet, so east to west a block is 300·cos φ m of planet and north to south 300 / cos φ m; shapes are true only on the equator. The cap keeps the same 3.4 % of the planet behind 75 degrees as before, but its band is narrower in blocks: 49 blocks from 75 to 80 degrees at C = 16,384 (98 at 32,768), against 228 on the old equirectangular map (POL-006). This is geometry and game design, not physics, and the page says what it distorts.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [POL](ledger/POL.md) | pole cap | POL-001 to POL-016 (16) |
| [POL](ledger/POL.md) | projection | POL-017 to POL-021 (5) |

Rows by status: `game-choice` 18, `uncited` 3.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/SCL.md](extraction-notes/SCL.md).
- Research note: `docs/research/notes/wrapped_planet_worlds.md`.
- Work package in the verification plan: WP-12 ([claims.md](claims.md#verification-plan-for-stage-2)).
