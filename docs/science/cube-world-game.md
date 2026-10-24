# The game on a cube world

Status of this page: `stage 1 (written with the feature, cube plan phase 2, 2026-10-23; nothing here
is physical, every row is a game choice)`.

Ledger ids on this page: `SCL-210`, `SCL-211`, `SCL-215`, `POL-031`, with `SCL-108`, `SCL-127` and `POL-022`
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
A Sable ship that reaches the border loses its speed outward and is blown back inside; it is never
moved, so the players on its deck stay on it.
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

   Every biome search of a cube world's chunks (the biome step on and off the faces, and the surface
   and carver steps) starts cold (`climate/ColdBiomeSearch`), so a quart's biome depends on the
   quart alone. Vanilla's search starts from the entry the thread found last and keeps it when
   another entry is exactly as near, so at a climate tie a chunk holds whatever its thread met
   first. A chunk off the faces asks its partner's quarts in another order (turned), so at a tie it
   held the other biome: under Expanded Ecosphere, 2 of the test's 18,432 quarts at depth 9,500
   (y = -96) sat exactly between Wythers' underground and deep underground (fitness 250,000 to
   each), and the copy held deep underground where the partner held underground. The polar worlds
   keep vanilla's warm search.
3. **Deep ocean past the corners** (SCL-211). Columns in no band are the level's default rock up
   to 64 blocks below sea level and water above, in the deep ocean biome.
4. **The border** (`wrap/CubeBorder`, `wrap/EdgeBorder`, POL-031 with POL-022 and POL-023). The
   playable area A(d) is the six faces shrunk by d at every turning edge. Each tick a survival,
   adventure or creative player (or the vehicle they ride) outside A(1) is moved to the nearest
   point of A(1.05); a player more than 16 blocks outside the planet is first folded to their
   partner point instead. Within 24 blocks of a turning edge (measured through every east-west
   period image) the action bar warns. A Sable ship is held by its bounds: when a corner of its
   global bounds is outside A(1), the part of its velocity that points out of the planet is taken
   away and it gets 1 block/s back each tick (POL-011's wind at the stop line), every tick until
   the bounds are inside again (`CubeBorder.outward`, `SablePoleCap.tickCube`). The way out is the
   sum of the outside corners' offsets from their nearest points of A(1), so a ship that runs
   diagonally into F0's corner is pushed straight back out of the void square, and one held at the
   end of a segment slides on along the other side.
5. **Spawn** (`server/SpawnHandler`, SCL-127, POL-024). A spawn candidate must lie in A(64), 64
   blocks from every turning edge, and 256 blocks from every image of a cube corner in the net.
6. **Far terrain** (`farterrain/FarTerrain`, Distant Horizons' level of detail). An area wholly on
   one face is drawn from the world data as on any world. An area that reaches off the faces is
   drawn column by column at the planet block each column shows (the same fold), its block states
   turned and its biomes those the fill stores, so the far view past a turning edge is the fill's
   ground; columns past the corners are the same deep ocean. Distant Horizons' curvature takes the
   radius of the sphere with the six faces' area, s·sqrt(3/(2π)) (SCL-078: 2,653 blocks at 16k,
   5,130 at 32k), not the period's 4s/(2π), and Toroidal World's guard checks the period 4s.
7. **The detail noise** (`worlddata/DetailNoise`, Rust's `detail::DetailSpec::cube_noise`,
   SCL-215). The game adds a small noise term to the baked ground (the detail layer's amplitude
   times a noise of 32-block wavelength). On a cube world that noise is laid on the cube's surface,
   not on the net: a block's centre goes to its face by the fold (a block off the faces to its
   partner's point), and the noise is value noise on a lattice drawn on the cube, whose lines meet
   the cube's edges (m = max(1, round(s/λ)) cells along each edge, so 32 at s = 1,024). Two faces
   meeting at an edge share the lattice points on it, so the noise runs on through every seam and
   corner with no blend, and a block off the faces reads its partner's value exactly. The bake's
   own relief noises (the jag, the cliff bands' levels) are the same function, so the bake and the
   game agree on the ground at a seam. Petroff-Guyou and Lambert worlds keep the plain noise.
8. **The planet and climate APIs** (`api/planet/DeepTimePlanet`, version 1, and
   `api/climate/DeepTimeClimate`, version 6; docs/api/planet.md). Other mods read the planet's
   geometry on the server and on the client as a sphere, with no projection named: latitude and
   longitude, the local frame (north, east and the metres per block along each), great-circle
   distance, bearing and destination, and the canonical position. On a cube world latitude and
   longitude go through the fold and the map, so a block off the faces reads its partner's place;
   north (`northBearingDeg`, degrees clockwise from -z) runs smoothly over a face and on into the
   copy past its edge, and goes all the way round each pole; distances and bearings run on
   across every edge and corner. Past the corners there is no planet: the calls answer NaN or
   empty. The climate API's `latitudeDeg`, `longitudeDeg`, `northBearingDeg` and `metresPerBlock`
   are shorthands for the planet API; `projectionKind` and `circumferenceBlocks` are gone (API
   6).
9. **The F3 line** (`net/PlanetDebugLine`, cube plan G10). On a cube world the debug screen reads
   `Deep Time: 10.144° N 100.920° E · face F0 · north 7.8° · 311 m/block here · <planet> (C 4096,
   276 m/block)`: latitude and longitude, the face (`F1 (folded)` off the faces, for the face the
   block folds onto), north's bearing and the metres a block spans there, the numbers the climate
   API gives, then the nominal scale. Past the corners it says "off the planet", and a client whose
   jar lacks the server's map table says "map mismatch". The client works it out from the planet
   info alone.
10. **The commands** (`server/DeepTimeCommands`, `server/SiteCommands`, cube plan G11).
    - `/deeptime probe pole` names both poles. The cube is turned freely, so they lie somewhere
      inside N and S. For each pole it gives the block (x in Toroidal World's frame, where `/tp`
      goes), whether the border lets a player reach it, the planet distance to it in km, and the
      walk to it under the border. It also gives the source's own latitude, longitude, north,
      metres per block and border zone.
    - `/deeptime probe column` adds where a block folds to: its face and cell, the turning edge it
      lies past with the planet block it shows, or the ocean past the corners. It also gives north
      and the metres a block spans.
    - `/deeptime probe tw` gives the east-west period 4s, the face side, and whether Toroidal
      World wraps at that period. The smoke run checks the last.
    - `/deeptime sites near` gives each site's planet distance in km and its walk, as well as its
      distance in the net.

    The planet distance is the great circle on the equal-area sphere times the nominal metres per
    block. The equal-area sphere's six faces of s² blocks are its whole area, so its radius is
    R = s·√(3/(2π)), 707.6 blocks at s = 1,024. The nominal metres per block make R the planet's
    radius in metres (SCL-107).

    The walk is the shortest path inside A(0), the band and the polar column, whose boundary is
    the border's 12 segments. A(0) is a cross. Its only reflex corners are the four inner corners
    (±h, ±h) of F0, so the shortest path is straight or bends at those corners. It is found
    exactly, as the shortest path through the two ends and those corners in five period images
    (`CubeBorder.walkDistance`).

    The walk is a distance in the net, in blocks. The planet distance runs across faces the border
    closes: from N's centre to F1's it is a quarter of the way round the planet, while the walk
    goes through F0's corner.
11. **The presets** (`preset/Presets.withProjection`, `server/FirstStartGenerator`, cube plan G12).
    The level type chooses the map. On a cube level type any preset of that size bakes as a cube,
    the built-ins too: the first start bakes a copy of the preset with `scale.projection` set to
    `cube`, and its log says so. A dedicated server sets `level-type=deeptime:planet_cube` (or
    `deeptime:planet_cube_32k`) in `server.properties` and names a 16k (or 32k) preset in
    `config/deeptime/server.json`. If the preset's size does not match the level type, the first
    start stops with the presets that fit. The other level types still need a preset of their own
    map.
12. **The Create World screen** (`screen/model/CubeOutline`, `screen/model/NetView`, cube plan G13).
    The timelapse's pictures stay the simulation's latitude-longitude pictures, so the map and the
    globe work as on the other worlds while the planet's history runs. The cube is turned on the
    planet when the world data is baked, on the final land, so a cube world's edges are known only
    when the run is done. From then on the globe draws the cube's twelve edges over the planet. The
    six edges the border closes are thick orange-red lines, and the other six (five joined in the
    net, and the slide) are thin white ones. Each edge is sampled 48 times along its face side
    through the world's own map, so the line is where the world's faces meet. The eight corners
    are dots: white for the four touching F0, which a player can reach, and orange-red for the four
    behind the border. The review page's final map is the net as it is played: the window of 4s by
    3s blocks drawn 4 : 3, 512 by 384 pixels (8 blocks a pixel at 4k), north (-z) up. The faces
    carry the world data's elevation, the void around N and S is transparent, and the border's
    twelve lines are drawn over it with the five joined face sides. The ore-province hints keep
    their latitude-longitude cells and are painted over the faces. The spawn marker sits at its
    point on its face, and hovering shows the latitude, longitude and face. The editor keeps its
    `projection` and `lambda_c_deg` fields until section 7.4.
13. **The developer map** (`devmap/`, never shipped; `devmap/model/PlanetGeometry`, cube plan G14).
    The dev map reads a cube world as its net. Its picture is the window of 4s by 3s blocks from
    x = -3s/2, and the void around N and S is transparent and cannot be clicked: the server clears
    the void in the satellite picture after the natives draw it, and the elevation picture skips
    it. The natives (`deeptime-tiles`, the workbench's renderer) read a cube world's layers, stored
    as six face planes, as the net window's flat raster: each net cell is its face's cell, a void
    cell the cell under the fold of its centre, x wrapping with the period 4s; read as one flat
    raster from F0's corner, S showed F1's plane and F2 F1's east column. The seam is the slide
    (x = -3s/2, the same line as 5s/2, for |z| up to s/2), dashed. The edges are the border's 12
    segments, drawn as one outline with the border's inset dashed inside it, and the five face sides
    the net joins are faint dashed lines. The two poles are single points inside N and S. The cone points are the eight corners at their twelve images in the window: a
    cross in the cone colour on the four corners of F0, which a player can stand on, and in
    orange-red on the four the border keeps a player off. The readout names the face and gives the
    distance to the slide and to the nearest border segment; the minimap's second line reads
    `F0  slide N  border N`. Distances and the player markers wrap x in Toroidal World's frame
    (-2s to 2s), not the window, so a delta across the slide is short. The graticule is cut only
    where a line crosses a turning seam (two faces the net does not join); across the slide and
    the five joined sides it runs on. A teleport by right-click lands at the nearest point at
    least the border's inset and the edge margin inside the segments (`CubeBorder.nearest`), never
    on a copy in the void. The globe draws the cube's outline as the Create World screen does
    (item 12). The shot driver's places on a cube: the spawn, 25 blocks onto F3 from the slide,
    40 blocks inside N's east segment (a quarter-turn seam's border), the north pole, beside F0's
    north-east corner and 40 blocks under the F1, F2 and N corner.
14. **Lava flows across the seams** (`worldgen/LavaFlows`, cube plan G16; the flows themselves are
    [volcanoes-and-magma.md](volcanoes-and-magma.md), item 11). A flow steps half a block at a time
    down the ground. On a cube world each step goes through the net's fold. At the slide, x just
    wraps. A step that ends past a turning edge goes onto the partner face by the edge's motion,
    and the flow's heading, its pond heading and its metric turn with it, so the flow runs on as the
    copy past the edge showed it. The smooth wander that turns a flow is read where the flow would
    be if the faces were laid flat from its vent, so it does not jump at an edge. A painted column
    is kept as the planet block it shows, and every column asked for (by a chunk, by a copy off the
    faces, by far terrain) answers through the same fold, so a column past an edge shows its
    partner's paint. Which volcanoes can reach a column is decided by the straight line between them
    on the cube: a path over the surface is never shorter, so the test holds across edges and round
    corners. A flow that reaches past the net's corners stops there. On Lambert and Petroff-Guyou
    worlds nothing changes.

## Equations and constants

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| SCL-108 | bands beyond the turning edges | a face width deep, nearer edge wins, ties to the earlier edge | blocks | cube-world-plan.md section 1.4 | `game-choice` |
| SCL-210 | the fill off the faces | partners' biomes; partners' terrain copied and turned at the surface step; no starts, features or mobs | rule | docs/design/cube-game.md section 2 | `game-choice` |
| SCL-211 | deep ocean past the corners | OCEAN_DEPTH = 64 below sea level | blocks | docs/design/cube-game.md section 2 | `game-choice` |
| SCL-215 | the detail noise on the cube's surface | value noise on a surface lattice, m = max(1, round(s/λ)) cells per cube edge, fade f²(3 − 2f), octaves as the plain noise | blocks | cube-world-plan.md G3; FORMAT.md "The detail layer" | `game-choice` |
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

The climate API's north and scale come from the map's Jacobian J at the block: its rows are the
planet's (east, north) metres per block of +x and of +z, through the fold's motion off the faces
(`CubeWorld.jacobian`). North in the block frame is the (dx, dz) that J takes to (0, 1),
normalised. Its bearing is atan2(dx, -dz) in degrees, taken into [0, 360). The metres per block
are sqrt(|det J|), with J taken on the sphere of the six faces' area, whose radius is
m_per_block_h · s · sqrt(3/(2π)) metres (SCL-107). The conformal map
has one scale in every direction, so this is the length of a block both ways. On the equal-area
Lambert and Petroff-Guyou worlds the API gives m_per_block_h everywhere, since a block's area is
the same everywhere there.

## Earth against the mod

Nothing physical is modelled, so there is nothing to compare with Earth. The GameTests
(`CubeWorldGameTests` on the 4k test planet baked as a cube, s = 1,024) check that the fill equals
its partners block for block and biome for biome. On 2026-10-23 (the VM, `./gametest.sh
-PgametestOnly=...` with the seven tests): 29 chunks off the faces, 8,257,536 blocks against their
partners turned, 0 differ, and 256 columns past the corners deep ocean; 16 chunks, 18,432 biome quarts,
0 differ; 99 chunks at the face centres, seams and corners (53 off the faces) generate with no hole
to the floor; a point just inside each turning edge and the fold of the point just outside it are
at most 1.05 blocks apart on the sphere (R 707.6 blocks); the border puts a player in a void square
or past the corners back on the planet (4.55 and 92.7 blocks), folds one 100 blocks past N-east
onto F1 with its yaw turned 90°, and leaves both poles and the face centres alone.

Sable ships (W3, 2026-10-24, the dev planet server on the 4k cube: `gametest-planet.sh --cube-4k`
with `sableHullStoppedAtCubeBorder`, `jetCrossesSeam` and the `sable_straddle` scenario,
`build/cube2/sable-test.sh` and `sable-seam.sh`; two runs). Hulls of 10 blocks launched outward at
60 blocks/s from 12 blocks inside the stop line reach at most 2.91 blocks past it at N's east
segment, 1.91 at F1's top segment and 2.91 at S's bottom segment (one tick at 60 blocks/s is 3), and
after 40 ticks are back inside, moving away from the segment at 8.7 to 10.5 blocks/s. A hull run
diagonally from F0 into its north-east corner reaches 2.63 blocks into the void square's edge, is
turned along F1's top segment, and ends 71 blocks east, inside and moving away from it. A hull at
rest 20 blocks inside N's east segment is not moved. Across the slide (Toroidal World's seam in F2,
C = 4,096) a hull pushed at 40 blocks/s crosses in 20 of 20 attempts, whole (10 of 10 blocks) and
folded to x = -2,043.8, and a hull across the seam is the same sub-level, at the same x, after a
save, stop and start.

Far terrain (stage 2, 2026-10-23, the 4k cube, `farTerrainMatchesTheCube` with the other 13 cube and
far terrain tests): against the chunks the game generated, 168 columns in the 16 blocks inside the
turning edges (167 at the same ground height, all within 1 block), 168 past them (165 exact, all
within 1), 40 across the slide and 132 across Toroidal World's seam (all exact), and the 4 corners of
a chunk past the net's corners (deep ocean, exact); water and biome agree in every column. At 16k
(the smoke's `--dh` checks on `earthlike_quick_16k_cube.json`, then `dh-client-check.sh`, the VM's
private Xvfb): Distant Horizons held 400 of 400 columns on each side of the slide (x = -5,760)
with ground within 2 blocks of the generated chunk in 390 and 398 of them, water or dry as
generated in all 800 and the biome in all 800; the real client then received 2,001 far sections, 378
and 228 of them in the 2,048 blocks west and east of the slide and 210 and 362 beside the seam.

The surface noise (G3, 2026-10-23, `build/cube2/g3-checks.sh` on the VM): the 14 cube and far
terrain GameTests above pass with it, and `gametest-planet.sh --cube-4k` reads the same ground
height (to 0.01 block) from the game and from the bake's Rust reader in the 10 probe columns on every
face. Its unit tests check that the noise reads one value from both sides of each of the 14 net
edges (the slide and the twelve turning edges) and steps across them no more than across a lattice
line inside a face, that an outside block reads its fold partner's value exactly, and that Rust and
Java give the same pinned values. The seam steps it gives the bake are on the bake page (B1).

Lava flows (G16, 2026-10-24, `LavaFlowsTest` on the VM, `build/cube2/g16-test.sh`). The test puts a
shield 20 blocks from an edge of a 1,024-block cube, on ground that is a function of the point on the
cube (a cone by the straight-line distance from the vent, tilted 0.02 blocks a block down across the
edge). It compares the flows with those of a flat world on the same ground, where the box round the
vent past the edge is the partner's copy laid flat. Whether a column is painted agrees in every column
of the 521 by 521 box at the slide (54 of 150 flows cross it, 4,161 of 11,610 painted columns lie
past it, every code equal), at N's east edge (a quarter turn, with an anisotropic metric that must
turn with the flow; 39 of 172 flows cross, 1,227 of 6,030 columns past) and at N's top edge (a half
turn; 61 of 157 cross, 2,853 of 7,012 past). Past a turning edge 83 and 647 old columns swap
oxidised and covered: those patches are a hash of the column, and the cube keys the partner's column
where the flat world keys the copy's. Every column past an edge answers as its partner, and every
chunk with paint is one the flows touch, on both faces and in the void. Away from the edges the
cube's flows equal the flat world's column for column. The surface point used for the reach
matches the cube's development across four seams: |A - Q|² = da² + dp² + t² to 1e-6.

## Simplifications and limits

- The copy off the faces has no structures and no features: trees, ore veins placed as features,
  springs and villages stop at the turning edges in the view across them. The wrap engine's band
  (cube plan E3) shows the partner's real chunks instead and supersedes the fill there.
- Terrain that a structure adapts round its pieces (the beardifier) is not copied: the scratch
  chunk does not know the partner's structure starts.
- A lava flow that runs into the void square past a corner of the net stops there, and the
  oxidised patches of an old flow past an edge are not the ones a flat world would draw (both are
  hashes of the column).
- Far terrain past a turning edge is the fill's ground, so it has no trees or structures either;
  the area is drawn column by column there (its cost there was not measured apart).
- The surface noise's lattice is square on each face in net blocks, so where the map's local
  scale σ varies across a face the noise's texture is stretched with it on the planet, as the
  plain noise's is on the other projections; in blocks it is the same everywhere.
- The Sable border reads a ship's global bounds, a box in x and z, so a ship turned at an angle
  is stopped when the box's corner reaches the line, up to a few blocks before its own blocks do.
- The timelapse cannot show the cube's edges while the run is going: the cube's turn is chosen on
  the final land when the world data is baked.

## Game choices

All rows above are game choices made by the cube lane's agent on 2026-10-23 under Ben's rule of
2026-10-16 (undoable choices are the coordinator's call), within Ben's decisions of 2026-10-22 (a
cube world, the conformal cube turned freely, format 0.7, the wrap engine in parallel).

## Open questions

- Whether the fill should place features (trees) from the partner: the wrap engine's band makes
  it moot where the engine runs.
- Whether the two level types replace `deeptime:planet` and `deeptime:planet_32k` (cube plan
  section 7.4, the coordinator's call after phase 2 is gated).
