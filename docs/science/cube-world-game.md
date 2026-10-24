# The game on a cube world

Status of this page: `stage 1 (written with the feature, cube plan phase 2, 2026-10-23; nothing here
is physical, every row is a game choice)`.

Ledger ids on this page: `SCL-210`, `SCL-211`, `POL-031`, with `SCL-108`, `SCL-127` and `POL-022`
to `POL-024` from earlier pages (see [claims.md](claims.md)).

## What you see in the world

A cube world (world-data format 0.7, the level types `deeptime:planet_cube` at 16k and
`deeptime:planet_cube_32k`, opt-in on the Create World screen) is the planet drawn on six square
faces of side s blocks (3,840 at 16k, 7,424 at 32k) laid out as a cross: four faces round the
equator, F3, F0, F1 and F2, wrap east to west through Toroidal World with a period of 4s, and one
face lies above F0 (N) and one below it (S). The cube is turned freely on the planet, so the
poles lie somewhere inside N and S, not at their centres, and north is not the -z direction
everywhere.

Six of the cube's twelve edges are joined in the net itself: five with the faces side by side, one
(F2 to F3) by the east-west slide. Each of the other six appears twice in the net, as two
"turning edges" (round N and S, and the tops and bottoms of F1, F2 and F3, twelve in all) where one
face's ground continues on another face turned by a quarter or half turn. Until the wrap engine joins
them, a temporary border stops a player a block inside each turning edge, with a warning on the
action bar within 24 blocks of it. Looking past the border, the land is not cut off: every chunk
off the faces holds a copy of the ground it would join, turned (its partner), so the view across
a turning edge shows the right hills and coasts, but without trees, ores placed as features or
structures. Past the corners of the net, where no face's ground reaches, lies deep ocean 64 blocks
deep.

A player who arrives far outside the planet (a teleport, an ender pearl, a portal exit more than
16 blocks out) is moved to the place the copy under them shows, with their heading turned the same
way; one who walks into the border is put back at the nearest point inside it. Spectators pass.
The world spawn keeps 64 blocks from any turning edge and 256 blocks from any corner of the cube.

## The model in plain words

There is no physical model here: this page is geometry and game rules. The pieces, in the order
the game uses them:

1. **The net and the fold** (`worlddata/CubeLayout`, from phase 0, SCL-108). Every block (x, z) of
   the level is either on a face, in the band beyond one turning edge (the band reaches a face
   width out; where two bands meet, in the square void by each corner of F0, the nearer edge
   wins), or in no band. A block in a band has a partner on the planet: the block the edge's
   motion (a quarter or half turn about a chunk corner, or the slide) takes it to.
2. **The fill off the faces** (`worldgen/CubeFill`, SCL-210). The faces are chunk aligned, so a
   chunk is wholly on a face or wholly off. A chunk on a face is generated as on any world. A chunk
   off the faces is never filled from noise: at the biome step it takes its partners' biomes (a
   motion about a chunk corner maps 4 x 4 biome cells onto biome cells), and at the surface step
   each partner chunk its columns map to is generated in a scratch chunk by the same steps the game
   runs for that chunk (biomes, noise fill, surface rules, Deep Time's surface passes, the air
   carvers, the water's wake). The scratch chunk is placed where Toroidal World generates the
   partner, so it is the same terrain. Its columns are copied block for block, each block state
   turned (stairs, logs, rails face the right way after the turn). The chunk then gets no
   structure starts, features or mobs of its own.
3. **Deep ocean past the corners** (SCL-211). Columns in no band are the level's default rock up
   to 64 blocks below sea level and water above, in the deep ocean biome.
4. **The border** (`wrap/CubeBorder`, `wrap/EdgeBorder`, POL-031 with POL-022 and POL-023). The
   playable area A(d) is the six faces shrunk by d at every turning edge. Each tick a survival,
   adventure or creative player (or the vehicle they ride) outside A(1) is moved to the nearest
   point of A(1.05); a player more than 16 blocks outside the planet is first folded to their
   partner point instead. Within 24 blocks of a turning edge (measured through every east-west
   period image) the action bar warns.
5. **Spawn** (`server/SpawnHandler`, SCL-127, POL-024). A spawn candidate must lie in A(64), 64
   blocks from every turning edge, and 256 blocks from every image of a cube corner in the net.

## Equations and constants

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| SCL-108 | bands beyond the turning edges | a face width deep, nearer edge wins, ties to the earlier edge | blocks | cube-world-plan.md section 1.4 | `game-choice` |
| SCL-210 | the fill off the faces | partners' biomes; partners' terrain copied and turned at the surface step; no starts, features or mobs | rule | docs/design/cube-game.md section 2 | `game-choice` |
| SCL-211 | deep ocean past the corners | OCEAN_DEPTH = 64 below sea level | blocks | docs/design/cube-game.md section 2 | `game-choice` |
| POL-022 | the stop line's inset | 1 block inside each turning edge; put back 0.05 block inside it | blocks | FORMAT.md "The world's layout" | `game-choice` |
| POL-023 | the warning band | 24 blocks, every 40 ticks | blocks, ticks | code | `game-choice` |
| POL-031 | arrivals folded | FOLD_DEPTH = 16 outside the planet | blocks | docs/design/cube-game.md section 5 | `game-choice` |
| POL-024, SCL-127 | spawn margins | 64 from a turning edge, 256 from a corner image | blocks | cube-world-plan.md B4, W5 | `game-choice` |

The playable area for an inset d, with h = s/2 and x taken into Toroidal World's frame [-2s, 2s):

    A(d) = { |z| <= h - d }  union  { |x| <= h - d and |z| <= 3h - d }  union  { |x| <= h and |z| <= h }

The first set is the equatorial band, the second N and S shrunk at their edges, and the third F0
whole, so that F0's corners (where N's and S's side edges meet F1's and F3's top and bottom edges)
stay inside. A heading of yaw y turns with a motion of k quarter turns as the vector (-sin y, cos y)
does under the motion.

## Earth against the mod

Nothing physical is modelled, so there is nothing to compare with Earth. The GameTests
(`CubeWorldGameTests` on the 4k test planet baked as a cube, s = 1,024) check that the fill equals
its partners block for block and biome for biome; their numbers go in the build log.

## Simplifications and limits

- The copy off the faces has no structures and no features: trees, ore veins placed as features,
  springs and villages stop at the turning edges in the view across them. The wrap engine's band
  (cube plan E3) shows the partner's real chunks instead and supersedes the fill there.
- Terrain that a structure adapts round its pieces (the beardifier) is not copied: the scratch
  chunk does not know the partner's structure starts.
- Lava flows (G16) and the far terrain of Distant Horizons (G6) do not yet cross turning seams.
- The detail noise (G3) is not yet blended across turning seams, so the ground can step by the
  detail term's amplitude at a turning edge (the bake blends its own relief noises, SCL-125).
- Sable ships are not stopped by the cube border yet.

## Game choices

All rows above are game choices made by the cube lane's agent on 2026-10-23 under Ben's rule of
2026-10-16 (undoable choices are the coordinator's call), within Ben's decisions of 2026-10-22 (a
cube world, the conformal cube turned freely, format 0.7, the wrap engine in parallel).

## Open questions

- Whether the fill should place features (trees) from the partner: the wrap engine's band makes
  it moot where the engine runs.
- Whether the two level types replace `deeptime:planet` and `deeptime:planet_32k` (cube plan
  section 7.4, the coordinator's call after phase 2 is gated).
