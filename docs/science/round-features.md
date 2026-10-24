# Round features on the planet

Status of this page: `stage 1 (written with the fix, 2026-10-23; the geometry is exact, its one source not yet re-read)`.

Ledger ids on this page: `SCL-230` to `SCL-236` (see [ledger/SCL.md](ledger/SCL.md)); it also changes IMP-135,
DEF-043, DEF-049, BIO-105, BIO-106 and BIO-182.

## What you see in the world

A crater, a volcano, a caldera, a flood-basalt province, a granite pluton, a stock or a volcanic plug is round on the
planet at every latitude: a 10 km crater at 55° N is 10 km across east-west and north-south alike. On the cube world,
which is conformal (it keeps shapes locally; Ben's world since 2026-10-22), it is round in world blocks too, scaled by
the projection's local scale. Volcanoes along an arc are spaced by the same distance on the ground at the equator and
near the poles.

The Petroff-Guyou world is equal-area, not conformal: it draws every shape with its own local stretch, 1.78 to 1 at
the median and 2.67 to 1 at most over the planet (the projection survey's figures,
`docs/research/notes/seamless-projection-survey.md` §3.1), and a round feature there shows that stretch as every
mountain and coast does. Until 2026-10-23 every one of these features was drawn round in the simulation's own blocks,
so the planet showed it long north-south by 1/cos² of its latitude (1.33 to 1 at 30°, 3.0 to 1 at 55°, 8.5 to 1 at
70°), and the Petroff-Guyou world added its own stretch on top. Volcanoes far from the equator stood twice as close
east-west as north-south. A player would have seen the egg-shaped craters as plainly wrong. Measured on a 16,384-block
Petroff-Guyou planet (λc 30°, 72 longitudes a latitude; `cargo test -p deeptime-core --lib -- --ignored --nocapture
petroff_guyou_shape_ratio_of_round_features`, 2026-10-23), a crater's shape ratio in world blocks (long axis over
short):

| Latitude | Now: median / p90 / max | Before: median / p90 / max |
|---|---|---|
| 0° | 1.79 / 2.26 / 2.29 | the same |
| 30° | 1.42 / 2.48 / 2.63 | 1.54 / 3.28 / 3.51 |
| 55° | 2.01 / 2.52 / 2.66 | 2.53 / 6.96 / 8.08 |
| 70° | 2.38 / 2.54 / 2.56 | 10.75 / 21.07 / 21.85 |

So on Petroff-Guyou a crater is never more stretched than the map itself (2.67 to 1 at most), where it reached 22 to
1 at 70° before; on the cube it is round.

On the Lambert world preset (a test layout, not a played one), whose blocks are the simulation's, the same features
are now wide east-west by 1/cos² of the latitude: that is what a circle on the planet looks like in Lambert's map.

## The model in plain words

The simulation runs on Lambert's cylindrical equal-area grid (`sphere::BlockFrame`): x is longitude times a
constant and z is the sine of latitude times the planet's radius R = C / 2π. A block of x is cos φ blocks of the
planet's surface and a block of z is 1/cos φ of them, so the grid keeps area but not shape. A feature drawn with a
radius in grid blocks (its distance the block offset times metres a block) is therefore an ellipse on the planet.

The fix measures on the planet instead (`sphere::Around`, SCL-230). About a feature's centre, a block's offset is
its position in the azimuthal equidistant frame at that centre: the great-circle distance from the centre, laid
along the bearing, as metres east and south. Every stage that stamps a radius now calls it:

1. **Impacts** (impact stage): a crater's imprint and overprint on the coarse grid (`impacts::mod`, through
   `volcanoes::for_cells`), and the breccia lens body under its floor (SCL-233), whose plan is the ellipse the grid
   makes of the 0.95-rim-radius circle.
2. **Volcanoes** (volcano stage): a cone's, caldera's and summit crater's footprint, the ignimbrite sheet, the
   flood-basalt province (`for_cells`), the spacing test that places arc and hotspot volcanoes (`place_arcs`,
   `dist_m`, SCL-232), and each grid cell's share of a cone, which samples the cell's true extent on the planet
   (`cell_extent_m`, `cell_pad_m`, SCL-231: a cell near a pole is narrow east-west and tall north-south).
3. **Structure**: a pluton's outcrop and aureole (`plutons::for_each_cell_grown`) and its body
   (`Pluton::plan_blocks`), a stock's and a plug's window (`deform::intrude`, SCL-235). The outline is set on the
   planet (semi-axes in metres, a stock's or pluton's strike an azimuth on the planet) and stamped as the ellipse the
   grid makes of it (`Around::ellipse_blocks`).
4. **Amplify**: the block-scale crater and cone (`amplify::impact`, `amplify::volcano`) measure the same distance.
5. **Bake**: an intrusion's window goes to the world as the exact image of its ellipse (SCL-236), and every body on
   the Petroff-Guyou world goes as the exact image of its plan (the cube world's rule, SCL-130). Before, both
   stretched each axis by the length of its own image and kept the two perpendicular, which is exact only where the
   map does not shear the ellipse's axes: on a conformal world it is off by 15 to 28 % in radius for a yawed ellipse
   at 55 to 70° (the Lambert grid's shear), and on Petroff-Guyou, whose own shear adds to it, by more than 5 % at
   55° in the unit test.
6. **The game**: the volcanic biome zones (BIO-182) take the stretch from world blocks to the planet's tangent plane
   at the vent, so they lie on the cone as the terrain draws it. `deeptime volcano-census` measures in the same
   frame.

Lambert is equal-area, so a feature's area, its volume and the rock it moves are the same as before; only its
outline changed. The pool runs (plates, climate, surface and volcanoes) see the change through the volcano stage:
the cones' footprints and the arc spacing.

## Equations and constants

The offset of point u (a unit vector) from centre u0, with east e and north n the tangent vectors at u0:

    d = R_m · atan2(|u × u0|, u · u0)            (great-circle distance, m)
    (east_m, south_m) = d · (u·e, −u·n) / |(u·e, u·n)|

R_m is the planet's radius in metres (C / 2π blocks times `m_per_block_h`). A circle of radius r on the planet,
drawn in Lambert blocks at latitude φ, has the semi-axes

    r / (m cos φ)  along x,     r cos φ / m  along z        (m = metres a block)

and an ellipse with semi-axes a, c, its first axis at yaw ψ (from east toward south), is the ellipse with conjugate
semi-diameters p = K R(ψ) (a, 0) and q = K R(ψ) (0, c), K = diag(1/(m cos φ), cos φ / m). Its axes are the
eigenvectors of M Mᵀ (M = [p q]) and its semi-axes the square roots of the eigenvalues (`sphere::conjugate_axes`):
the major from the larger eigenvalue and the minor as |det M| over it, which keeps its digits for a very long ellipse
near a pole. The same function gives a window's or a body's exact image under the world's Jacobian J (p and q are then
J times the source ellipse's semi-axes). Near a pole cos φ is held at max(cos φ, max(a, c) / (m C / 2), 10⁻⁶)
(SCL-234), so an ellipse spans at most half the circumference east-west and keeps its area.

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| SCL-230 | a feature's distance from its centre | great-circle distance along the bearing (azimuthal equidistant frame) | m | Snyder 1987 ch. 25 (from memory); spherical geometry | `from-memory` |
| SCL-231 | a grid cell's extent on the planet | north-south R (asin(z₁/R) − asin(z₀/R)) m; east-west its Lambert area over that | m | Lambert's equal-area property (Snyder 1987 ch. 10, read) | `game-choice` (the 4 × 4 sampling) |
| SCL-232 | volcano spacing | great-circle distance | m | as SCL-230 | `game-choice` |
| SCL-233 | breccia lens plan | the Lambert ellipse of the 0.95-rim-radius circle | blocks | as SCL-230 | `game-choice` |
| SCL-234 | pole guard | cos φ held at max(a, c) / (m C / 2) and 10⁻⁶ or more in `ellipse_blocks` | - | none (numerical) | `game-choice` |
| SCL-235 | pluton, stock and plug outlines | set on the planet, stamped as their Lambert ellipse | m, blocks | as SCL-230 | `game-choice` |
| SCL-236 | projecting an intrusion window and a Petroff-Guyou body | exact image of the ellipse (eigen-decomposition of M Mᵀ) | blocks | linear algebra | `game-choice` |

## The audit

Every feature the simulation, the bake or the game stamps with a radius or a round footprint, found by a sweep of the
code on 2026-10-23 (`impacts`, `volcanoes`, `structure`, `deposits`, `surface`, `hydro`, `amplify`, `bake` and the
Java world generation). Sizes are for `earthlike_quick_16k` (300 m a block, R = 782 km). "Before" is the outline's
aspect on the planet (north-south over east-west) at 30, 55 and 70°; a feature drawn round in Lambert blocks had
1.33, 3.04 and 8.55. "Matters" says whether a player could see it at its size.

| Feature | Drawn in | Size (16k) | Before at 30 / 55 / 70° | After | Matters | Done |
|---|---|---|---|---|---|---|
| Crater imprint and overprint (bowl, rim, ejecta, melt, hydrothermal ring) | sim: impacts | 2 km across and up (7 blocks and up) | 1.33 / 3.04 / 8.55 | 1.00 | yes: the bowl and rim are the most visible round thing | fixed, SCL-230 |
| Breccia lens body | sim: impacts (bodies at the bake) | 0.95 of the rim radius | 1.33 / 3.04 / 8.55 | 1.00 | yes: it shows in the crater floor and walls | fixed, SCL-233 |
| Block-scale crater (L3) | amplify | 1 to 3 crater radii | 1.33 / 3.04 / 8.55 | 1.00 | yes | fixed, SCL-230 |
| Volcanic cone (arc, shield, silicic) | sim: volcanoes | basal radius about 20, 80 and 70 km | 1.33 / 3.04 / 8.55 | 1.00 | yes | fixed, SCL-230 |
| Caldera and summit crater | sim: volcanoes | caldera 3 to 9 km in radius | 1.33 / 3.04 / 8.55 | 1.00 | yes | fixed, SCL-230 |
| Ignimbrite sheet | sim: volcanoes | out to 4 rim radii (12 to 36 km) | 1.33 / 3.04 / 8.55 | 1.00 | yes, as rock at the surface | fixed, SCL-230 |
| Flood-basalt province | sim: volcanoes | `lip_radius_km` 900 scaled to the planet, about 110 km | 1.33 / 3.04 / 8.55 | 1.00 | yes | fixed, SCL-230 |
| A cell's share of a cone | sim: volcanoes | one L1 cell (4.8 km) | the cell's Lambert square | its planet extent | near the poles: a cell's samples fell outside the true cell | fixed, SCL-231 |
| Arc and hotspot spacing | sim: volcanoes | 60 and 70 km | east-west cos φ of the spacing, north-south 1/cos φ of it: 1.33 / 3.04 / 8.55 | 1.00 | yes: volcanoes crowded east-west poleward | fixed, SCL-232 |
| Block-scale cone (rim wobble, crater cut, flank gullies' ring) | amplify | as the cone | 1.33 / 3.04 / 8.55 | 1.00 | yes | fixed, SCL-230 |
| Plutons and their aureoles | sim: structure | 8 to 80 km long, 0.5 to 1 as wide | their ellipse stretched 1.33 / 3.04 / 8.55 north-south | the planet's ellipse | yes, where they crop out | fixed, SCL-235 |
| Stocks | sim: structure | 2 to 10 km in radius (median 4), 1 to 1.6 elongated | as plutons | the planet's ellipse | yes, as granite outcrops | fixed, SCL-235 |
| Plugs | sim: structure | 2 to 5 blocks in radius | 1.33 / 3.04 / 8.55 | 1.00 | yes: a plug's neck is a landmark | fixed, SCL-235 |
| Intrusion windows in the world | bake | as the stocks and plugs | yawed ellipses off by up to 15 % (55°) and 28 % (70°) in radius on a conformal world | exact | yes, for stocks | fixed, SCL-236 |
| Bodies on the Petroff-Guyou world | bake | every body | as above for a yawed ellipse | exact | yes, for plutons | fixed, SCL-236 (the cube already, SCL-130) |
| Volcanic biome zones | game | as the cone | followed the Lambert cone (BIO-182) | follow the planet's cone | yes: they must sit on the cone | fixed, BIO-182 |
| Vent gullies' and cone relief's ring | bake: relief | as the cone | already on the sphere | - | - | already round |
| Hotspot plumes and the plates' flood basalts | sim: plates | 150 km and up | already on the sphere | - | - | already round |
| Hot-spring pools, vents and patches | game | 2 to 11 blocks | round in world blocks | - | no: a few blocks, drawn in the world | round in the world (on the cube round on the planet too) |
| Magma chambers and lava lakes | game | 4 to 15 and 2 to 10 blocks | round in world blocks | - | no: as above | round in the world |
| Lava flows | game | 1 to 2 blocks wide | measured on the planet already (VOL-160) | - | - | already right |
| Hot-spring field and thermal-cave zones, the water cave shield | game | 8 to 16 blocks | discs in world blocks | - | no: as above | round in the world |
| Dykes, fold trains and faults | sim: structure | dykes 15 km long, folds and faults up to hundreds of km | their length and trend in Lambert blocks, so stretched north-south | unchanged | some: a fault's or fold train's trend is off poleward | not round features: deferred |
| Fold-belt reach from a plate edge, glacier footprint, the glacial amplify box, the isostasy smoothing (50 km) | sim: surface solvers | tens of km | Lambert distances | unchanged | partly: their grain runs north-south poleward | deferred: the solvers' grid (sphere-native grids research) |
| Deposit clusters, camp spacing and cause search | sim: deposits | 1 to 600 km | Lambert distances | unchanged | little: scatter is statistical | deferred (by design, plan section 5.2) |
| Deposit bodies (porphyry, blankets, veins, coal, placer) | sim: deposits | up to km | their plan stretched like the plutons' | unchanged | some: a round porphyry is long north-south poleward | deferred: a dipping body's planet ellipsoid needs a full orientation the body format does not hold |
| Petroleum migration (50 km), BIF basins, placer reach (30 km), adjoining deposits | sim: deposits | 30 to 300 km | Lambert distances | unchanged | little | deferred |
| Delta plains | bake: water | Nienhuis's area law | area right, shape Lambert | unchanged | little: a plain's outline follows the coast | deferred |

Lake and basin seeds and maars were looked for and not found: lakes fill from the terrain, and there are no maars.

## Earth against the mod

Not a calibration: a feature is round on the planet by construction, and the unit tests below show it.

- `sphere::around`: `a_circle_on_the_planet_is_wide_in_lambert_blocks_by_one_over_cos_squared`,
  `a_planet_ellipse_in_blocks_comes_back_round_and_keeps_its_area`, `the_offset_is_the_great_circle_distance_along_the_bearing`.
- Impacts: `a_crater_is_round_on_the_planet_at_high_latitude` (amplify), the breccia lens check in
  `fields_map_and_bodies_come_from_the_catalogue`.
- Volcanoes: `a_cone_is_round_on_the_planet_at_high_latitude` (amplify), `arcs_are_spaced_on_the_planet_at_high_latitude`.
- Structure: `a_pluton_is_round_on_the_planet_at_high_latitude`, `a_stock_and_a_plug_are_round_on_the_planet_at_high_latitude`.
- `sphere::around`: `an_ellipse_at_a_pole_is_finite_and_keeps_its_area` (SCL-234).
- Bake: `an_intrusion_window_is_the_exact_image_of_its_ellipse` (on the Petroff-Guyou world at 30, 55 and 70° a plug's
  window is the image of its circle on the planet; a 1.5 : 1 stock yawed 34° lies on its projected ellipse; the old
  rule missed it by more than 5 %), `on_the_cube_a_circle_on_the_planet_is_a_disc_in_world_blocks` (a plug and a
  breccia lens are discs in the cube world's blocks to 0.3 % at 30, 55 and 70°, away from the corners; the old
  Lambert circle is long by 1 / cos² φ there).
- Game: `VolcanicZonesTest.onAPetroffGuyouWorldTheZonesAreTheConesCircleOnThePlanet` (the zones are the cone's
  circle on the planet, stretched as the map stretches there), `onALambertWorldTheZonesAreThePlanetsCirclesWideEastWest`;
  the GameTest `aCraterIsABowlOverBreccia` measures a crater's floor, rim and ground by the great-circle distance on
  the 4,096-block test planet. That planet is a Petroff-Guyou world, so its plain world frame would read a round
  crater with the map's stretch (1.4 to 2.7 to 1); the sphere's frame is the right one there, and on the cube world it
  is the plain world frame up to the local scale.

The pool's check that the planet's history did not go wrong with the change (8 runs, base3, 2026-10-23) is in the
build log (STATUS.md).

## Simplifications and limits

- The offsets are exact on the sphere; the ellipse a feature is stamped as is the frame's linear map at its centre,
  so a feature hundreds of km across (a flood-basalt province near a pole) is drawn by its distances, not by that
  ellipse, and stays round; bodies and windows, which are ellipses by format, are exact only to first order across
  their extent (the error grows with the feature's size over R / tan φ; not measured on a bake).
- The cube world is conformal, so round on the planet is round in world blocks up to the projection's local scale,
  which varies across a large feature (and is singular at the cube's eight corners). The Petroff-Guyou world is
  equal-area: a round feature carries its stretch (above), as every landform there does.
- Linear features (dykes, folds, faults), the solvers' kernels and the deposits keep the Lambert metric (the audit
  above).
- Near a pole (cos φ under 10⁻⁶, SCL-234) the ellipse's east-west width is held finite.

## Game choices

- Measuring on the planet rather than drawing after the projection: agent, 2026-10-23 (the coordinator's crater task):
  the features feed the simulation's own fields (relief, rock, strata), which live on the Lambert grid, so they are
  stamped there with the planet's metric; the projection then carries them to the world. Round on the planet rather
  than round in a Petroff-Guyou world's blocks: the same call. The simulation does not know the world's map, the
  cube (conformal) is the world Ben chose on 2026-10-22, and on Petroff-Guyou every other landform carries the map's
  stretch too.
- The 4 × 4 samples of a cell's share (SCL-231), the pole guard (SCL-234): agent, 2026-10-23.

## Open questions

- Whether a Petroff-Guyou world should draw its craters and cones round in its own blocks (the map's inverse stretch
  at each feature, against the terrain round it, which keeps the map's stretch). Recommended: no, the cube world
  replaces it.

- The cube world's bodies and the Petroff-Guyou world's now share the exact rule; the Lambert world preset draws the
  planet's circles wide east-west, which is right for its map but looks odd to a player. It is a test layout.
- Deposit bodies would need a full 3D orientation in the body format to be drawn on the planet when they dip (a
  format change, Ben's to decide).

## Sources

- `Snyder1987`: chapter 10 (the cylindrical equal-area projection) read in the USGS PDF on 2026-10-23 by another agent
  (see bibliography.md); chapter 25 (the azimuthal equidistant projection) cited from memory for SCL-230, not re-read.
