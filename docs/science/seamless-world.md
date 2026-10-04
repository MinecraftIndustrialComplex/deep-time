# The seamless world: Petroff's equal-area map in the Guyou layout

Status of this page: `stage 2 (written with the code, 2026-10-04; Petroff's paper and code read in
full, the map's properties tested; the layout and its sizes are game choices)`.

Ledger ids on this page: `SCL-049`, `SCL-083`, `SCL-084`, `SCL-086` to `SCL-090` (see
[claims.md](claims.md)).

## What you see in the world

A Deep Time world (format 0.5, the default) is a rectangle of 13,312 × 6,656 blocks for a 16k planet
and 26,112 × 13,056 for a 32k one: two squares, each holding one hemisphere, side by side. The equator
runs straight across the middle (z = 0); north is −z along it. Every block covers the same area of
the planet, as on the Lambert world before it, but shapes are stretched in places: half the planet is
drawn with one direction up to 1.78 times longer than the other, and at most 2.67 times (Petroff's
own figures, reproduced by the projection survey). Walking east or west wraps round the planet as
before (Toroidal World). The north and south edges are each glued onto themselves by a half-turn
about their midpoint: walking north over the north pole comes out heading south on the far side, as
on a real globe. Until Deep Time's wrap engine joins those two edges, a temporary border stops
players one block inside them; the terrain beyond the border is already the glued planet, so the view
over it is right.

The poles are ordinary places of the planet. They lie on the north and south edges, a quarter of the
way in (x = ∓b/2), so for now they stand at the border. Four other places on the edges, the cone
points, are where a full turn of planet is drawn in half a turn of world: walking round one takes half
the usual distance, and the map there is stretched without bound. The bake turns the planet about
its axis so these four points lie as far from land as the planet allows.

## The model in plain words

1. The simulation runs as it always has, on its own Lambert grid; only the world is projected, at
   the end of the bake (Ben, decisions log 2026-10-03, "Bake only").
2. **The map.** Petroff (2021) draws the whole sphere in a square with the north pole at its centre,
   equal-area and in closed form (ledger SCL-083). Its northern half is a diamond; tilted 45° it is a
   square showing one hemisphere with the equator round its edge. Two of these squares, the second
   holding the southern hemisphere mirrored, make a 2 : 1 rectangle whose edges are all the same
   circle of the sphere drawn from both sides, so every edge glues exactly (SCL-086).
3. **Guyou's aspect.** The sphere is turned so that the first square's centre is a point of the
   equator at longitude λc and its top-edge midpoint the north pole. The equator then runs straight
   through both squares (SCL-086).
4. **Size.** The rectangle has the planet's area at today's scale: 2b² = 4πR², b = C/√(2π), rounded
   to a multiple of 256 blocks (SCL-084).
5. **λc.** The bake tries every quarter degree and keeps the longitude that puts the four cone points
   farthest from land (SCL-087).
6. **Resampling.** Every world cell finds its point on the sphere and reads the simulation's layers
   there, averaged over a patch that is round on the sphere (SCL-090). Near the poles a Lambert row is
   many blocks tall but its columns a fraction of a block wide; read at a point, that draws radial
   streaks round the poles. The round patch averages across the narrow columns until both directions
   are resolved alike. Categories and rock stacks take the nearest cell.
7. **Things with a position** (rivers, folds and faults, ore bodies, sites, volcanoes, craters, spawn
   points) move through the projection; directions and lengths through its local stretch. A river
   that crosses a turning edge is cut there.

## Equations and constants

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| SCL-083 | Petroff's map | φ0 = 3π/8; ψ0 = asin(1/√(2 − cos²φ0)); h′ = (12/π)(ψ0 + asin(2 sin φ0/√(3 − cos 2φ0)) − π/2); forward and inverse closed forms | radians | Petroff 2021, CC0 code | `verified` |
| SCL-084 | hemisphere side | b = 256·round(C/(256√(2π))): 6,656 (16k), 13,056 (32k) | blocks | — | `game-choice` |
| SCL-086 | the layout and its properties | X = 2x/b + 1, Y = −2z/b; equal area, seams close to 10⁻¹² rad | blocks | Petroff 2021 | `verified` (tested) |
| SCL-087 | λc search | 0.25° steps, distances to land to 20° | degrees | — | `game-choice` |
| SCL-088 | temporary border | 1 block inside the north and south edges | blocks | — | `game-choice` |
| SCL-089 | detail noise at the edges | blend over 32 blocks to the half-turn-symmetric mean | blocks | — | `game-choice` |
| SCL-090 | footprint | D = max(stretch·r, r / cos φ), stretch ≤ 8 | blocks | — | `game-choice` |

The world block's true size is √(4πR²/2b²) metres (294.6 m at 16k, 300.4 m at 32k), written as
`scale.m_per_block_h`.

## Earth against the mod

Not a model of Earth: these are properties of a map. Measured on the bake (see the STATUS build log,
SW-1, for the run and its numbers): the map's land and ice shares against the planet's true shares,
and the pole anisotropy with and without the round footprint.

## Simplifications and limits

- Shapes are not true: up to 2.67 times longer one way than the other, worst near the four cone
  points. Ben accepted this (decisions log 2026-10-04) and may swap to the cube if it feels too bad in
  play.
- The simulation still runs on Lambert's grid, whose polar cells are slivers; the world cannot show
  detail the simulation did not resolve near the poles, so the polar regions are smoother than the
  rest (they are no longer streaked).
- Footprint averaging softens the terrain a little everywhere (a box of a fraction of a cell at the
  equator).
- River chains crossing the north or south edge are cut there; a river does not yet continue through
  the seam.
- Until the wrap engine joins the north and south edges, the poles are reachable only up to the
  border.

## Game choices

- The projection and layout: Ben, decisions log 2026-10-04 ("We'll proceed w/ Petroff–Guyou").
- Projecting only at the bake: Ben, 2026-10-03 ("Bake only").
- The world size at today's blocks per km² (not the prototype's 16,384 × 8,192): agent, 2026-10-04,
  in STATUS "Pending questions".
- λc search, border inset, edge blend, footprint: agent, 2026-10-04 (SCL-087 to SCL-090).

## Open questions

- The world size (above).
- Whether the cone points should also avoid the poles' neighbourhood (they cannot: both lie on the
  same edges by construction).

## Sources

- `petroff2021` (Petroff, M. A. 2021, ACM TSAS 7(4):18; code Zenodo 4661351): read in full (paper by a
  sub-agent, 2026-10-03; code by the implementing agent, 2026-10-04).
- The projection survey, `docs/research/notes/seamless-projection-survey.md`, for Guyou's aspect,
  the shape statistics and the layout.
