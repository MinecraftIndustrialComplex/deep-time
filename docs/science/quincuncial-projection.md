# Quincuncial projections (prototype only)

> **Retired 2026-10-24** (cube plan §7.4, 2026-10-24; Ben, decisions log 2026-10-24: "Cube only (Recommended)"). The prototype is deleted: the dev-only crate
> `crates/deeptime-quincuncial` and the scripts `tools/proto/quincuncial/`. SCL-070 to SCL-077 are
> retired (`merged`, the text kept). The page is kept as the record of the comparison.

Status of this page: `stage 1 (written with the prototype, 2026-10-03; nothing read in its primary
source except Wikipedia's Guyou page)`. The shipped world uses Lambert's cylindrical equal-area
projection (decisions log 2026-10-02); this page describes the dev-only prototype of
[`../design/seamless-wrapping.md`](../design/seamless-wrapping.md).

Ledger ids on this page: `SCL-070` to `SCL-077` (see [claims.md](claims.md)).

## What you see in the world

The prototype world is a rectangle of 16,384 × 8,192 blocks: two squares, each holding one hemisphere,
side by side (Guyou's arrangement). The equator runs straight across the middle (z = 0). The poles sit
on the long edges, a quarter of the way along each. Walking off any edge brings the player back in
through an Immersive Portals portal: east and west edges slide (like today's east–west wrap), the long
edges turn the player 180° about the edge's midpoint, so walking north over the north pole comes out
heading south on the far side of the pole. The map is conformal: shapes are true locally, but areas are
not. A piece of land is drawn largest near four points at 45° latitude, where the projection is
singular; the prototype turns the planet so that those four points fall in open sea.

## The model in plain words

1. Peirce's map (1879) sends the sphere to a square. The planet point's stereographic coordinate ζ
   (the plane the sphere is projected onto from its south pole) is the Jacobi elliptic function cn of
   the block's position w in the square, with parameter m = ½ (ledger SCL-070, written from memory,
   checked by tests).
2. The function cn with m = ½ repeats with the periods 4K and 2K ± 2iK and is even. So the same
   formula covers the whole plane, and each edge of the square is glued to itself by a half-turn about
   its midpoint. That gluing is what makes the world seamless: the map continued past an edge is
   exactly the map inside, turned 180°.
3. Guyou's aspect (1887) is the same map with the sphere turned so that the map's pole lies on the
   equator, and the block plane turned 45° (SCL-071; the Wikipedia statements were read on 2026-10-03).
4. The world data the game reads is resampled from a normal Deep Time bake: every raster cell of the
   new world is mapped to its planet point and sampled there in the bake's own projection. Continuous
   fields are sampled bilinearly, categories and rock stacks from the nearest cell.

## Equations and constants

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| SCL-070 | the map | ζ = cn(K + (2K/a)(x − i z) \| ½) | dimensionless | Peirce 1879; Lee 1976 | `from-memory` |
| SCL-071 | Guyou aspect | frame = 45° turn of (x + b/2, z); sphere turned so the frame's pole is (0°, λc) | blocks, degrees | Guyou 1887 via Wikipedia | `verified` (qualitative) |
| SCL-072 | K(½) | 1.854074677301372 | dimensionless | none | `from-memory` |
| SCL-073 | sn, cn, dn, R_F, arccn | AGM; addition formulas; duplication | dimensionless | A&S 1964; DLMF; Carlson 1995 | `from-memory` |
| SCL-074 | singular points | Peirce: edge midpoints on the equator; Guyou: hemisphere-square corners at ±45° | blocks | Wikipedia (Guyou) | `verified` (corners) |
| SCL-075 | area distortion | min 0.457, median 0.79, 99 % 4.6, 99.9 % 14.5 (blocks per planet area / mean) | dimensionless | computed | `calibrated` |
| SCL-076 | prototype size | b = 8192, λc = 31°, strip C′ = 32,768 | blocks | agent | `game-choice` |
| SCL-077 | portals | 1,200 tall at y = 64; half-turn quaternion (0, 0, 1, 0) | blocks | 2026-10-02 prototype | `game-choice` |

The linear scale, in radians of the planet per block, is L = (2K/a) · 2|sn(w)·dn(w)| / (1 + |cn(w)|²),
the same in every direction. It is √2·(2K/a) at the map's two poles and falls to zero like the
distance at the four singular points (the zeros of sn·dn).

## Earth against the mod

Not applicable: the projection is geometry, not a physical claim. The measured distortion (SCL-075)
agrees with the 2026-10-02 research note's own computation (`world-projection-options.md` §2.1).

## Open questions

- The prototype's formulas were written from memory. The tests prove they are a conformal,
  orientation-preserving map of the sphere onto the square that agrees with itself across every seam,
  which is all the prototype needs; they do not prove it is Peirce's own normalisation (it could differ
  by a turn of the sphere, which would not matter).
- None of the cited works was read in full.
- Which projection production should use is surveyed in
  [`../research/notes/seamless-projection-survey.md`](../research/notes/seamless-projection-survey.md)
  (2026-10-03): with quarter-turn seams an equal-area cube does better than any pillowcase map; its
  constants are ledgered as SCL-078 to SCL-085 (`not-yet-coded`). Its measurements of Peirce
  reproduce SCL-075 by an independent route (Schwarz triangle functions).
