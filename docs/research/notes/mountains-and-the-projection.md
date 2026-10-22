# Mountains and the projection: which map on the cube

Researched 2026-10-22 for Deep Time, on the branch `worktree-agent-a996493d601a564e5`. Ben's question: "In general,
I'm having some second guesses about the projection we are using, mountains especialyl are feeling hit hard. What
would be a better way to go about this?" During the study he decided (decisions log, 2026-10-22): "Fuck it, remove
equal are restriction and move to a cube." This note's job is therefore to pick the map on the cube and back the
pick with numbers, for the cube plan (`docs/plans/cube-world-plan.md`). It changes no shipped code.

**Tags.** **[C]** computed by me with the tools in `tools/research/mountains-projection/` on the test VM (§7 says
how); **[V]** read by me in the source; **[SA]** stated by a source I did not read myself (the survey or the plan
cites it); **[I]** my inference or derivation, reasoning given; **[M]** from memory, not checked.

"This planet" is one earthlike_quick_16k run (supercontinent start, 1,000 Myr), baked as the Lambert world and
projected to the Petroff-Guyou world the game plays (b = 6,656, λc 62.5°). "15 planets" adds the land of 14 older
bakes (11 test runs, Ben's screenshot world and two play worlds), read from their Petroff-Guyou heightmaps.
"Mountain land" is land 1 km or more above sea level.

---

## 0. The answer in plain words

- **Pick: the conformal cube for the world, turned freely so all eight corners sit far out at sea.** Shapes on
  land are true (stretch at most 1.02, the table's error). Land is drawn between 0.79 and 1.92 times its true area
  on the worst of 15 planets (median planet: within ×1.68; this planet ×1.67, its mountains ×1.59).
  The corners land 5.3° to 9.6° from the nearest land on all 15 planets. [C]
- **The turn is part of the pick.** The plan keeps the polar axis (poles at the face centres). On the conformal map
  that is a bad match: on this planet a corner then sits 0.18° from land and land is drawn up to 9.9 times too big
  (17 times on the worst of 15 planets). **If the polar axis must stay, pick the equal-area Moser cube instead.** No
  in-between map rescues the polar axis (taken half way, Moser's flow still draws land up to 9.8 times too big on the
  worst planet, 6.0 on this one). [C]
- **The simulation's own cube: the equal-area Moser map, on the polar axis.** The simulation's corners cannot be put
  at sea (continents drift over every point in 1,000 Myr), and the conformal map's corner cells are tiny: at the
  simulation's n = 240 the corner cell has 0.044 of the mean area and a shortest edge 0.18 of the mean. One global
  explicit diffusion step would be 31 times shorter, an advection step up to 5.6 times. The plan's "about 0.16 of
  the mean area, so explicit steps halve" reads Putman and Lin's length ratio as an area, and their halved
  advection step (one test flow) as a bound for every solver. Moser's corner cell is 0.91 of the mean. [C, SA]
- **Strict rule (exactly equal area):** only Moser meets it. **Relaxed rule (no land drawn more than a factor too
  big or too small), free turn, worst of 15 planets:** the conformal cube meets ×2 (1.92), Moser's flow taken half
  way (τ = 0.5) meets ×1.5 (1.46), three quarters of the way (τ = 0.75) meets ×1.25 (1.23). [C]
- **What a player notices more: the stretch within one place.** At 35 m per block vertically, the conformal map's
  slopes vary only between places: on this planet's mountains the slope exaggeration runs from 8.15 to 9.37 (5th to
  95th percentile; 6.67 to 9.46 at the extremes) and changes by 2.8 % (median) over 256 blocks. Moser and
  Petroff-Guyou vary within one place, by direction: Moser draws a mountain 1.26 times longer one way than the other
  (median; up to 1.42), Petroff-Guyou 1.49 (up to 2.64). A player sees the second in a single view (round craters
  drawn as ellipses, slopes steeper on one axis); the first hides inside real relief, whose true slopes vary 2.9
  times between the median and the 90th percentile of mountain land. [C for the numbers, I for the judgement]
- **Why mountains feel hit hard today** (parts 1, 2): mostly the simulation's Lambert texture, carved in block
  metric (median stretch 3.6 at 55° to 70°, 10.2 above 70°), then Petroff-Guyou's redraw (median 1.49 on mountain
  land). Moving the simulation onto a cube removes the first, whichever map the world takes. Part 4 lists what no map
  fixes: narrow belts (median full width 9 to 15 km) and the 8.4 times vertical exaggeration that makes a true 3.8°
  slope a 29° slope in blocks. [C]
- **Fallbacks, in order:** conformal with the pole tilted at most 30° from a face centre (land within ×2.05); Moser
  on the polar axis (equal area, shapes up to 1.42). The in-between maps are this study's own construction, cost the
  full metric tensor like Moser, and still need the free turn; I do not recommend them. [C, I]

---

## 1. Where mountains get distorted today (short)

Two maps act one after the other [C]:

1. **The simulation's texture.** Erosion, rivers and sediment are carved on the Lambert raster in block metric, as
   if every cell were a 300 m square. In truth a Lambert cell at latitude φ is cos φ wide and 1/cos φ tall, so the
   texture is stretched by 1/cos² φ.
2. **The bake's redraw on Petroff-Guyou.** Petroff-Guyou is equal-area, so it stretches shapes: median 1.78 on the
   whole planet, 1.49 on mountain land, at most 2.64 to 2.68.

Stretch (longest over shortest length of a small circle as drawn in the world), on mountain land of this planet:

| Latitude | Share of mountain land | Petroff-Guyou alone, median / p90 | Simulation texture as drawn, median / p90 |
|---|---|---|---|
| 0-20° | 30 % | 1.74 / 2.36 | 1.86 / 2.49 |
| 20-40° | 47 % | 1.38 / 1.93 | 1.26 / 2.35 |
| 40-55° | 10 % | 1.65 / 1.97 | 1.29 / 2.92 |
| 55-70° | 11 % | 2.20 / 2.38 | 3.59 / 8.20 |
| 70-90° | 2 % | 2.32 / 2.38 | 10.2 / 22.3 |
| All mountain land | 100 % | 1.49 / 2.31 (max 2.64) | 1.36 / 3.00 (p99 12.1) |

## 2. What it does to mountains (short)

- **Slopes depend on direction.** With 294.6 m per block horizontally and 35 m vertically the nominal exaggeration
  is 8.42. On Petroff-Guyou mountain land the steepest-drawn direction gets a median 10.3 (p95 13.0, max 13.7), the
  gentlest a median 6.9 (p5 5.5). [C]
- **Ranges come out combed.** In truth the simulation's slopes are steeper east-west than north-south: the ratio
  of RMS slopes is 1.11, 1.21, 1.75, 2.31 and 6.26 in the bands 0-20°, 20-40°, 40-55°, 55-70° and 70-85°: ridges and valleys line up north-south at high latitude,
  the grain of the Lambert cells. The renders (§3.5) show it as streaks in every panel, because every panel samples
  the same simulation. [C]

## 3. The cube maps compared

### 3.1 The family

- **Conformal cube** (τ = 0; Lee 1976, Rančić, Purser and Mesinger 1996 [SA, via the survey §2.2]). On each face
  the unique conformal map of the square onto the spherical square of the cube face that keeps the square's eight
  symmetries. Built from the Schwarz triangle map of the eighth of the face, triangle (centre, edge midpoint, corner)
  with angles (45°, 90°, 45°), onto the spherical triangle with angles (45°, 90°, 60°), extended by reflection
  (survey `tools/research/projection-survey/conformal.py`, `moser.conformal_tile("cube", ...)`) [V].
- **Equal-area Moser cube** (τ = 1; the survey's recommendation, §4 there). The conformal cube made equal-area by
  Moser's flow: world points move along dx/dt = -∇u / ((1 - t) + t ρ), with Δu = ρ - 1 and ρ the conformal map's
  planet area per world area (mean 1) [V].
- **In-between maps** (0 < τ < 1, **my own construction**): the same flow stopped at time τ, G_τ = F ∘ φ_τ. Its
  blocks per planet area at a planet point are τ + (1 - τ) · bpa₀, where bpa₀ is the conformal map's: the conformal
  area error moved a share τ of the way to 1 [I, from the flow's definition; checked numerically, C].
- Every map with τ < 1 has unbounded blocks per planet area at the corner vertex itself; only the corner's
  neighbourhood matters, which is why the corners' placement decides everything below. [C]

Whole-planet figures, no placement (blocks per planet area relative to the mean; stretch = shape ratio) [C]:

| τ | Area: min / median / p99 / p99.9 | Planet within ×1.25 / ×1.5 | Stretch: median / p99 / max |
|---|---|---|---|
| 0 (conformal) | 0.79 / 0.92 / 2.17 / 3.84 | 86.4 % / 95.5 % | 1.00 / 1.00 / 1.02 |
| 0.25 | 0.84 / 0.94 / 1.88 / 3.13 | 92.6 % / 97.1 % | 1.05 / 1.09 / 1.10 |
| 0.5 | 0.90 / 0.96 / 1.59 / 2.43 | 95.5 % / 98.6 % | 1.11 / 1.19 / 1.20 |
| 0.625 | 0.92 / 0.97 / 1.44 / 2.05 | 97.1 % / 99.3 % | 1.14 / 1.25 / 1.25 |
| 0.75 | 0.95 / 0.98 / 1.29 / 1.71 | 98.6 % / 99.7 % | 1.18 / 1.30 / 1.30 |
| 0.875 | 0.97 / 0.99 / 1.15 / 1.35 | 99.7 % / 99.97 % | 1.22 / 1.36 / 1.36 |
| 1 (Moser) | 1.00 (table error 0.97 at a corner) | 100 % / 100 % | 1.26 / 1.41 / 1.73 at the corner point |

How the conformal map's area error falls off with distance from the nearest corner (largest value at or beyond
that distance) [C]:

| Distance from the nearest corner | 2° | 3° | 5° | 8° | 10° | 15° | 20° |
|---|---|---|---|---|---|---|---|
| Blocks per planet area, max (mean = 1) | 3.07 | 2.52 | 1.96 | 1.56 | 1.40 | 1.16 | 1.04 |
| World area nearer than this | 1.1 % | 1.9 % | 4.1 % | 8.2 % | 11.5 % | 21 % | |

Relative to the face centre (0.79) these are 2.5, 1.8 and 1.3 at 5°, 10° and 20°, the plan's §1.6 figures.

### 3.2 Where the corners go

The corner score of a turn is the smallest signed distance from the eight corners to land (positive at sea,
capped at 20° like SCL-087), ties broken by the sum. Three ways to turn the cube were searched [C]:

- **Polar aspect**: the poles at face centres, only the turn λ0 about the axis is free (the plan's B3; 0.25° to 1°
  steps over 90°).
- **Tilt-limited**: the pole may lean at most 10°, 20° or 30° from the centre of its face (a grid of tilts in 2°,
  tilt directions in 10° and turns in 1° steps, then a local random walk that refuses turns past the limit; a lower
  bound on what a finer search finds).
- **Free turn**: any rotation (60,000 uniform random rotations on this planet, 30,000 on the others, the best 24
  refined by a shrinking random walk).

Smallest corner distance to land, degrees, over 15 planets [C]:

| Turn | Range | Median | Notes |
|---|---|---|---|
| Polar aspect | -1.44 to 4.79 | 1.95 | all eight at sea on 14 of 15; 11 of 15 closer than 3°; this planet 0.18 |
| Tilt ≤ 10° | 2.81 to 7.75 | 4.74 | this planet 2.81 |
| Tilt ≤ 20° | 2.93 to 8.64 | 5.65 | this planet 2.93 |
| Tilt ≤ 30° | 4.58 to 8.93 | 5.65 | this planet 4.70 |
| Free turn | **5.28 to 9.57** | **6.91** | pole tilt 7° to 46° (median 38°); this planet 6.98 at 40.8° |
| Petroff-Guyou's 4 cone points today (λc search) | 2.48 to 10.88 | 6.0 | this planet 4.3 |

How often all eight corners fit at sea [C]:

- Random rotations: 4.2 % to 25.6 % of them (this planet 9.45 %; 0.5 % keep all eight 3° out).
- The polar λ0 grid: 0 % to 30 % (this planet 0.56 %, and none 3° out).
- For comparison, Petroff-Guyou's λc search put all four cone points at sea on 21.8 % of its candidates on this
  planet.
- Land within 5° of a corner: none on any of the 15 planets with the free turn (every corner is 5.28° or more out).
  On the 14 other planets: at most 0.035 % of land with a tilt ≤ 20° or 30°, 0.4 % with tilt ≤ 10°, up to 1.9 % on
  the polar aspect.

The polar aspect fails because its eight corners sit at ±35.3° latitude in two fixed rings, where the planets'
land is; a free turn can put them anywhere. [C for the shares, I for the reason]

### 3.3 Area per block on land

Worst land on each planet, as the relaxed factor max(largest blocks per planet area, 1 / smallest): the worst of
15 planets, with the median planet in brackets [C]:

| τ | Polar aspect | Tilt ≤ 10° | Tilt ≤ 20° | Tilt ≤ 30° | Free turn |
|---|---|---|---|---|---|
| 0 (conformal) | 17.0 (3.14) | 2.63 (2.02) | 2.57 (1.84) | 2.05 (1.84) | **1.92 (1.68)** |
| 0.25 | 11.9 (2.60) | 2.22 (1.75) | 2.18 (1.64) | 1.79 (1.64) | 1.68 (1.51) |
| 0.5 | 9.75 (2.06) | 1.81 (1.50) | 1.79 (1.43) | 1.52 (1.43) | 1.46 (1.34) |
| 0.625 | 7.01 (1.79) | 1.61 (1.38) | 1.59 (1.32) | 1.40 (1.32) | 1.34 (1.25) |
| 0.75 | 4.56 (1.52) | 1.41 (1.25) | 1.39 (1.21) | 1.26 (1.21) | 1.23 (1.17) |
| 0.875 | 2.95 (1.27) | 1.20 (1.13) | 1.20 (1.11) | 1.13 (1.11) | 1.11 (1.08) |
| 1 (Moser) | 1.67 (1.00) | 1.02 | 1.02 | 1.02 | 1.02 |

Moser's 1.67 and 1.02 are the table's error where a corner sits on or near land; the map itself is 1.00. Leaving
out each planet's worst 0.1 % of land, the free-turn column becomes 1.77, 1.58, 1.39, 1.29, 1.19, 1.10, 1.00. [C]

This planet, free turn [C]:

| τ | Land: min to max | Mountain land max | Stretch on mountain land: median / max | Land median metres per block at s = 3,840 |
|---|---|---|---|---|
| 0 | 0.79 to 1.67 (p99 1.34) | 1.59 | 1.00 / 1.02 | about 310 |
| 0.5 | 0.90 to 1.33 (p99 1.17) | 1.30 | 1.12 / 1.20 | about 302 |
| 0.75 | 0.95 to 1.17 (p99 1.08) | 1.15 | 1.19 / 1.30 | about 298 |
| 1 | 1.00 | 1.00 | 1.26 / 1.42 | 294.8 |

The free turn pushes land towards the face centres, where the conformal map draws fewer blocks per area than the
mean (median land 0.905), so land blocks are about 5 % longer than the planet mean. [C]

### 3.4 Slopes at the fixed 35 m per block

The vertical scale is fixed at 35 m per block (ledgered; `docs/research/notes/vertical-scale-study.md`), so the
slope exaggeration at a point and direction is the horizontal metres per block in that direction over 35. A map
that varies the area per block varies it between places; a map that stretches shapes varies it between directions
at one place. On this planet's mountain land, free turn [C]:

| τ | Exaggeration, isotropic part: p5 / median / p95 (min, max) | Between places: p95/p5 (max/min) | Within one place, by direction: median / p90 / max | Change over 256 blocks: median / p99 | Everything: steepest over gentlest anywhere |
|---|---|---|---|---|---|
| 0 (conformal) | 8.15 / 8.83 / 9.37 (6.67, 9.46) | 1.15 (1.42) | 1.00 / 1.00 / 1.02 | 1.028 / 1.100 | 1.42 |
| 0.25 | 8.22 / 8.72 / 9.10 (6.99, 9.16) | 1.11 (1.31) | 1.06 / 1.09 / 1.09 | 1.020 / 1.080 | 1.34 |
| 0.5 | 8.28 / 8.62 / 8.86 (7.38, 8.89) | 1.07 (1.20) | 1.12 / 1.18 / 1.20 | 1.013 / 1.057 | 1.34 |
| 0.75 | 8.35 / 8.52 / 8.63 (7.86, 8.65) | 1.03 (1.10) | 1.19 / 1.29 / 1.30 | 1.006 / 1.032 | 1.35 |
| 0.875 | 8.38 / 8.47 / 8.52 (8.12, 8.53) | 1.02 (1.05) | 1.22 / 1.34 / 1.36 | 1.003 / 1.016 | 1.36 |
| 1 (Moser) | 8.42 everywhere | 1.00 | 1.26 / 1.40 / 1.42 | 1.000 / 1.000 | 1.42 |
| Petroff-Guyou today | 8.42 everywhere | 1.00 | 1.49 / 2.31 / 2.64 | 1.000 | steepest median 10.3 (max 13.7), gentlest median 6.9 (p5 5.5) |

(256 blocks is about 75 km, 5.5° of this 782 km-radius planet.)

Read the last column first: every cube map spreads slopes over about the same total range, 1.34 to 1.42. The maps
differ in where the spread goes. **Which does a player notice more?** [I]

- **Within one place it is visible in one view.** A round crater or volcano becomes an ellipse (1.26 to 1 on Moser
  at the median, up to 1.42; 1.49 on Petroff-Guyou), and slopes facing one way are steeper than those facing at
  right angles. Ben chose to correct this for volcanoes and craters on Petroff-Guyou (D1, fix 6), which says it is
  noticed.
- **Between places it is not.** The conformal map changes the scale by 2.8 % over 256 blocks (10 % at the 99th
  percentile), smoothly, the same in every direction. Two ranges 15 % apart in exaggeration are usually hundreds of
  kilometres apart, and true slopes on this planet's mountain land already vary 2.9 times between the median (3.8°)
  and the 90th percentile (11.1°) [C]. A 15 % difference disappears in that.
- So the conformal map's cost is the one a player is least likely to see, and its gain (true shapes) is the one
  most visible.

An option that removes even that cost [I, not measured in game]: a local vertical scale, 35 m × k / k̄ (k the local
horizontal metres per block, k̄ the mean), keeps the exaggeration at 8.42 everywhere on the conformal map. On this
planet's mountains it would run from 27.7 to 39.3 m per block, and a level surface 2 km up would tilt by about 0.4°
(1.3° at the 99th percentile).
Every height conversion (strata, sea floor, structures, climate heights) would then need k too. Not recommended for
now; listed so the plan can weigh it.

### 3.5 Renders

Same three ranges through every map, 1,024 × 1,024 blocks, rings at 50 and 150 km, arrow pointing north. Panels:
today as baked (with detail), today's Petroff-Guyou (no detail), conformal cube, τ = 0.5, τ = 0.75, Moser cube, and
the true shape (azimuthal equal-area). The cube panels use this planet's free turn, so north points a different way
in each range. All panels sample the same Lambert simulation, so the north-south combing of part 2 shows in all of
them: that is the simulation's texture, not the cube's. Files (gitignored, in this worktree):

- `review/out/mountains-projection/range-low-relief.png`, `range-low-oblique.png`: 10.9° S, 339.0° E (mean height
  4,739 m within 25 km; Petroff-Guyou block -6464, 776).
- `review/out/mountains-projection/range-mid-relief.png`, `range-mid-oblique.png`: 38.7° N, 249.7° E (4,810 m;
  block 3656, -1680).
- `review/out/mountains-projection/range-high-relief.png`, `range-high-oblique.png`: 62.6° N, 267.0° E (5,761 m;
  block 4072, -2576).
- `review/out/mountains-projection/cube-nets-free.png` and `cube-nets-polar.png`: this planet's land on the cross
  net for τ = 0, 0.5 and 1 with contours of blocks per planet area, for the free turn and the polar aspect (the
  render's net does not apply the quarter-turn choice of §5.4).
- `review/out/mountains-projection/cube-maps-by-tau.png`: area and stretch over one face for each τ.
- `review/out/mountains-projection/numbers.json`, `renders.json`: every number in this note.

What they show [C, read off the pictures]: Petroff-Guyou squeezes the 150 km ring into an ellipse about 2.3 to 1 at
62.6° N; the conformal panel draws it round; the Moser panel draws it about 1.2 to 1; τ = 0.5 and 0.75 lie between.
The conformal ring is slightly larger, the local area factor.

### 3.6 What the solvers need on each map

Face coordinates (u, v), cells of side h in them; g the planet metric pulled back to the face [I unless tagged].

**Conformal (τ = 0).** g = σ² (du² + dv²): one scale factor σ per point, no direction.

- Gradient and slope: |∇h| = |∂h| / σ; a D8 step of d cells has slope Δh / (σ h d). Conformal maps keep angles, so
  the eight D8 directions are at true 45° steps.
- Laplacian: Δ_g = σ⁻² (∂²/∂u² + ∂²/∂v²). In finite volumes the Dirichlet energy ∫|∇h|² dA is conformally
  invariant, so the stiffness matrix is the flat 5-point Laplacian unchanged (each face's transmissibility,
  true edge length over true centre distance, tends to 1) and only the mass matrix carries σ² h². The system stays
  symmetric positive definite and an M-matrix: conjugate gradients with a Jacobi preconditioner work as on a
  flat grid.
- Drainage area: Σ σ² h² over the upstream cells. Stream power and transport laws take true area and true slope.
- Advection: velocity in cells per time is u_true / σ; the step limit is σ h / |u|, smallest at the corner cells.
- Corner cells [C]: the corner vertex joins three cells, not four (7 D8 neighbours there, on every cube map). The
  corner cell of an n × n face has this share of the mean cell:

  | n | 60 | 120 | 240 | 480 | 3,840 (extrapolated) |
  |---|---|---|---|---|---|
  | Area | 0.111 | 0.070 | 0.044 | 0.028 | 0.0070 |
  | Shortest edge | 0.284 | 0.226 | 0.179 | 0.142 | 0.071 |

  Area scales as n^(-2/3) and length as n^(-1/3): near a corner the map behaves like z^(4/3), because a 90° corner
  of the square becomes a 120° corner on the sphere [I; the fitted exponents agree, C]. At n = 360 the shortest edge
  is 0.156, which matches Putman and Lin's "minimum over mean cell size 0.16 at c360" (as read for
  `sphere-native-grids.md` §2.2 [SA]): a **length** ratio. They found the conformal grid needed half the advection
  time step of the gnomonic grid in their tests [SA]; the bound from the cell size alone is larger. With one global
  explicit step at n = 240, an advection step is up to 1 / 0.179 = 5.6 times shorter (if the fastest flow can cross
  a corner cell) and a diffusion step 1 / 0.179² = 31 times shorter, since diffusion's limit is the smallest cell
  whatever the flow. Ways out: implicit diffusion (the system above), local time steps, or a different map for the
  simulation (§5.5). `sphere-native-grids.md` already advised against the plain conformal cube for the simulation
  on the same grounds.

**Moser (τ = 1).** √det g is constant (equal cell areas), but g has off-diagonal terms.

- Laplacian: Δ_g = ∂_i (g^ij ∂_j) (√g is constant). The cross terms g^12 make the 5-point stencil inconsistent: a
  9-point stencil, or a two-point flux with a non-orthogonal correction, is needed. The cell skew is bounded:
  |g12| / √(g11 g22) ≤ (s² - 1) / (s² + 1) = 0.34 at the largest stretch s = 1.42: grid lines meet at 70° at
  worst, 20° off square, near the corners where the stretch lies along the face diagonal (the survey's largest
  angular distortion, 19.9°, is the same bound). Finite-volume cubed-sphere models run on the gnomonic grid, whose
  skew is larger, so this is ordinary [I].
- D8: the eight neighbour distances differ per cell and direction, and the directions are skewed up to 20° from
  true; distances from the cells' corner points fix the slopes, not the directions.
- Drainage area: cell count times one area. Step limits: uniform (corner cell 0.913 of the mean area, shortest edge
  0.862 at n = 240 [C]).

**In between (0 < τ < 1).** Both: σ varies and g has cross terms. The full tensor is needed for consistency. A
cheap approximation treats the cell as conformal with σ = (det g)^(1/4); its error in a gradient is at most the
local stretch (1.20 at τ = 0.5). Corner cells at n = 240 [C]: τ = 0.5 area 0.103, edge 0.274 (steps 3.6 and 13
times shorter); τ = 0.75 area 0.215, edge 0.400 (2.5 and 6.3 times).

### 3.7 The equal-area rule

Ben's rule of 2026-10-02, "Use an equal area projects FFS", answered equirectangular maps that inflate area with
latitude (2 times at 60°, without bound towards the poles) and stretch shapes by the same factor. Under it, the
**strict** reading (every block the same planet area) leaves only the Moser cube. Ben dropped it as a strict rule on
2026-10-22, which the decisions log records as "the area per block may vary within a bound on land, shapes as true as
possible". Under that reading the
conformal cube with a free turn fits better than what the rule was made against: its land error (at most 1.92 on 15
planets) is tied to distance from a corner at sea, not to latitude, and it draws shapes true where equirectangular
stretched them 2 times at 60°. [C for the numbers, I for the comparison]

Which bound each map meets on land, worst of 15 planets [C]:

| Map | Strict (×1) | ×1.25 | ×1.5 | ×2 | Shapes on land |
|---|---|---|---|---|---|
| Conformal, free turn | no | no | no | **yes (1.92)** | true (1.02) |
| Conformal, tilt ≤ 30° | no | no | no | no (2.05) | true |
| Conformal, polar aspect | no | no | no | no (17.0) | true |
| τ = 0.5, free turn | no | no | **yes (1.46)** | yes | up to 1.20 |
| τ = 0.75, free turn | no | **yes (1.23)** | yes | yes | up to 1.30 |
| Any τ < 1, polar aspect | no | no | no | no (2.95 at τ = 0.875) | |
| Moser, any turn | **yes** | yes | yes | yes | up to 1.42 |

---

## 4. What else makes mountains weak (short)

Not the map, and no map fixes them [C, this planet]:

- **Height.** Summits reach 6,469 m; 7.1 % of land is above 3 km, 2.3 % above 4 km, 0.13 % above 5 km.
- **Width.** Mountain belts (land 1 km up) are narrow: full widths median / p90 / max of 15 / 73 / 170 km at 0-30°,
  10 / 42 / 229 km at 30-55°, 9 / 56 / 125 km at 55-75°. A 10 km belt is 34 blocks.
- **Steepness is the vertical scale's.** True simulated slopes on mountain land: median 3.8°, p90 11.1°, p99 18.7°.
  At 8.4 times exaggeration the world draws median 29° and p90 59°; 28 % of mountain land and 15.7 % of all land is
  steeper than 45° in blocks. The look of a range is set more by 35 m per block than by any projection.
- **The texture.** The combing of part 2 comes from the simulation's grid; it goes when the simulation moves to a
  cube (plan §3), not when the world does.

---

## 5. Recommendation and the numbers the implementation needs

### 5.1 The pick

1. **World: the conformal cube (τ = 0), turned freely** by the corner search of §5.4. [C: shapes 1.02, land ×1.92
   worst of 15, corners ≥ 5.28° out at sea]
2. **Simulation: the equal-area Moser cube (τ = 1) on the polar axis**, its own cube, independent of the world's
   turn. The bake resamples simulation cube to world cube through the planet point. [C: corner cell 0.91 of the
   mean; I: robust to corners on land]
3. **If the coordinator keeps the world's polar axis** (plan B3), the world takes Moser too, and both cubes share one
   table. Fix 6 then stays.

Why conformal over the half-way map (τ = 0.5, free turn): it draws shapes true where τ = 0.5 stretches them up to
1.20 (the error a player sees, §3.4); it is a published map with a closed construction, evaluated to Newton
precision at each table node with no PDE solve; its Jacobian is one number and one angle, so every metres-to-blocks
conversion and every footprint (a circle, not an ellipse) is simpler; and fix 6 is dropped. τ = 0.5 wins only on the
area bound (1.46 against 1.92) and on the between-place spread (1.07 against 1.15). [C, I]

Why not conformal for the simulation too: the simulation's corners cannot be placed (continents cross every point in
1,000 Myr), so continents would sometimes be carved on cells 5.6 times finer in length than elsewhere, with
explicit steps shortened to match (§3.6). Resolution-dependent laws (stream power, hillslope thresholds) would leave
a mark where a corner sat under land [I]. Moser's cells are within 9 % of one another everywhere.

### 5.2 The map: definition and table

- **Definition.** Face coordinates (u, v) ∈ [-1, 1]², with u = 2x / s - 1 and v = 2z / s - 1 in the face's block
  frame. F maps the face square conformally onto the spherical square of the cube face (the central projection of
  the cube face, corners at the cube's vertices (±1, ±1, ±1)/√3), commuting with the square's eight symmetries. It is
  the Schwarz triangle map of 0 ≤ v ≤ u ≤ 1 (angles 45° at the centre, 90° at the edge midpoint (1, 0), 45° at the
  corner (1, 1)) onto the spherical triangle (face centre, edge midpoint, vertex) with angles 45°, 90°, 60°, extended
  by reflection. `moser.conformal_tile("cube", U, V)` evaluates it: hypergeometric functions through mpmath, Newton
  inversion, output in the face's stereographic coordinate. [V]
- **Table.** `python3 tools/research/mountains-projection/cube_table.py OUT 512 0` writes the (n + 1)² node table
  for τ = 0 (for τ = 0 the flow stops at its first step, so the nodes are the conformal tile at the nodes, exact to
  the Newton tolerance; the script still runs the whole flow, which a production generator can skip). The plan's slot (one-eighth triangle, gnomonic a, b tabulated, Catmull-Rom bicubic, two
  Newton steps for the inverse) takes it unchanged. For the simulation's Moser table: `cube_table.py OUT 512 1`.
- **The corner.** F behaves like (z - z_corner)^(4/3) at a corner, so the bicubic is only O(h^(4/3)) accurate in
  the table cells touching it and its Jacobian may fold there. The plan's M1 fold check "within one block of a
  corner" will show it; a finer sub-table or the local expansion in the corner cell would fix it. The corners are
  at sea, so accuracy there matters only for continuity. [I]
- **Jacobian.** Scalar times rotation: the slot can return σ (planet metres per block) and the local angle; the
  plan's 2 × 2 form stays valid.

### 5.3 Face size

Keep the plan's s = 256 · round(C / (256 √(6π))): 3,840 at 16k, 7,424 at 32k, 1,024 at 4k. The planet mean is
today's (294.8 m at 16k), but with the free turn land sits towards the face centres: median land about 310 m per
block on this planet, 5 % longer than today's 294.6 m, so slopes in blocks are 5 % steeper (median exaggeration
8.83), which helps mountains. s = 4,096 at 16k would put land back near today's 290 m for 13.8 % more blocks. [C]
Coordinator's call; I recommend 3,840.

### 5.4 The corner placement rule (replaces plan B3)

1. **Land.** The final keyframe's land (elevation at or above sea level), as planet points.
2. **Signed coast distance d(p).** Great-circle distance from p to the nearest land, positive at sea; minus the
   distance to the nearest sea on land; clamped to ±20° (LAND_SEARCH_DEG, SCL-087). Precompute it on a sphere grid
   (the simulation's cells are enough) and look it up.
3. **Score of a rotation Q** (planet to cube frame): with c_k = (±1, ±1, ±1)/√3, score(Q) = min_k d(Qᵀ c_k) +
   0.001 · Σ_k d(Qᵀ c_k). The four reachable and four bordered corners count alike.
4. **Search.** A seeded generator (the planet's seed): 60,000 uniform random rotations plus the best of the polar
   λ0 grid; keep the best 24; refine each over 120 rounds of 32 trials, small random rotations whose size shrinks
   geometrically from 4° to 0.03°, keeping improvements; take the best. About 1.2 million corner lookups. On this
   planet the random stage alone reached 6.36°, the refinement 6.98°. [C]
5. **Tilt limit (optional).** If the coordinator wants north near -z, refuse rotations whose pole leans more than the
   limit from its face centre; 30° costs little (land ×2.05 worst), 10° more (×2.63).
6. **Symmetry.** The cube's 24 rotations give the same corners. Take the one that puts the north pole in face N
   (the face whose centre is nearest the pole), then of its four quarter turns the one with the most land on F0
   (plan B3's rule). Ties by index.
7. **Record.** The rotation (3 × 3, f64, row-major, planet to cube) in the manifest; the 8 corners' latitude,
   longitude and signed distance, the land's largest and 99.9th-percentile blocks per planet area, and the land
   share within 5° of a corner in `bake.stats`. A test on the 12-seed set asserts land ≤ ×2 and corners ≥ 5° out.
8. **Spawn** stays at least 256 blocks from a corner (plan); with corners 5° out at sea it is automatic.

What the free turn costs [I]: latitude no longer runs along the net, so -z is not north and the band faces do not
hold the equator. North already turns across today's Petroff-Guyou world, the sun in Minecraft is not local, and
climate is baked per block, so nothing breaks; the plan's "keeps latitude and the net aligned" is what is given up.
Petroff-Guyou's λc search, which this replaces, had the same freedom about one axis only.

### 5.5 The simulation's cube

- Moser (τ = 1), polar axis, n = 240 at r 16 (plan §3). Its metric from the cells' corner points as the plan says,
  with a 9-point stencil or non-orthogonal correction for the cross terms (skew ≤ 20°), D8 with real neighbour
  distances.
- The bake composes: world block → face (u, v) → conformal F → planet point (Qᵀ) → Moser inverse → simulation cell.
  Java needs the Moser table only if it reads simulation-grid rasters directly.
- If the simulation lane prefers the conformal map's 5-point stencil, it can take τ = 0 on the polar axis with
  implicit diffusion and local or subcycled advection steps at the corners (§3.6), and accept the corner imprint
  risk. The family makes this a data change, as plan §3 says.

### 5.6 What changes in the plan

- §1.6 table, "Solver metric", conformal column: corner cells 0.044 of the mean area at n = 240 (shortest edge
  0.18; Putman and Lin's 0.16 is that edge ratio at c360); explicit diffusion steps 31 times shorter and advection
  steps up to 5.6 times, not halved (the halving is Putman and Lin's measured advection step in their tests).
- §1.6, the sentence on the scale near corners: the 2.5, 1.8 and 1.3 are relative to the face centre; relative to
  the mean they are 1.96, 1.40 and 1.04.
- B3: the free turn (or a tilt limit) replaces the polar axis for a conformal world; the check numbers are §3.2's.
- §3 "One map, two cubes": two maps (Moser for the simulation, conformal for the world) and two turns.
- D1 fix 6 is dropped (shapes ≤ 1.02 on land).
- Every metres-to-blocks conversion uses the local σ: land ranges from 213 to 331 m per block on the worst planet
  (this planet's mountains 233 to 331 m). [C]

### 5.7 Fallbacks

1. Conformal with the pole tilted at most 30° from a face centre: land ×2.05 worst of 15, north within 30° of -z on
   the pole's face. [C]
2. Moser for the world on the polar axis: exactly equal area, stretch median 1.26 and at most 1.42 on land, fix 6
   kept. [C]

---

## 6. A finding on the detail noise

`neoforge/.../worlddata/DetailNoise.java` is value noise at the world block (x, z), with a 32-block blend towards
its half-turn-symmetric mean inside Petroff-Guyou's turning edges (SCL-089) [V]. The seamless projection survey's
line 249 says the detail noise is "sampled at the planet point"; it is not. On the cube either the plan's G3 blend
(a partition of unity at every turning seam) or sampling at the planet point (3D noise on the sphere, seamless with
no blend, feature size in blocks then following the local scale) works [I]; the survey's sentence should be
corrected either way.

## 7. How it was measured

- Tools: `tools/research/mountains-projection/` (`run.sh` verbs planet, cube, analyse, render; `geom.py` the maps
  and the Petroff-Guyou forward; `cube_table.py` the τ tables at n = 512; `analyse.py` the numbers; `render.py`
  the pictures). Run on the test VM, detached and admitted, e.g. `tools/remote/mac.sh -- tools/remote/detach.sh
  start mp-analyse --kind shell:mountains bash tools/research/mountains-projection/run.sh analyse`, then
  `tools/remote/mac.sh --no-sync --pull review/out/mountains-projection -- true`.
- Land samples: every fourth cell of the baked Lambert world's rasters. The other planets' land comes from their
  Petroff-Guyou heightmaps, with latitude and longitude rebuilt from each bake's λc; their Petroff cone distances
  agree with the bakes' recorded ones to a median 0.1° (largest 0.44°, and one far cone 1.9°, larger than recorded,
  probably a small island the coarser heightmap misses [I]), which checks the rebuild.
- Area and stretch come from the tables' finite-difference Jacobians; exaggeration is k / 35 with k = 294.6 / √bpa
  metres per block, and stretch splits it by direction.
- Limits: one planet for the slope figures; the tilt-limited searches are lower bounds; Moser's table errs near its
  corners (1.02, and 1.67 where a corner sits on land); the in-between maps are my construction; nothing was
  checked in game.

## 8. Sources

- Seamless projection survey: `docs/research/notes/seamless-projection-survey.md` (§2.2, §3.1, §4; its tools in
  `tools/research/projection-survey/`).
- Lee, L. P. (1976). Conformal Projections Based on Elliptic Functions. Cartographica Monograph 16. Rančić, M.,
  Purser, R. J. & Mesinger, F. (1996). A global shallow-water model using an expanded spherical cube: gnomonic
  versus conformal coordinates. QJRMS 122:959-982. Neither read; cited through the survey's §2.2 and §10.
- Putman, W. M. & Lin, S.-J. (2007). Finite-volume transport on various cubed-sphere grids. J. Comput. Phys.
  227:55-78. Read by a sub-agent for `sphere-native-grids.md` [SA]; the 0.16 and the halved step are from there.
- Moser, J. (1965). On the volume elements on a manifold. Trans. AMS 120:286-294 [M, as in the survey].
- Deep Time: `docs/plans/cube-world-plan.md` (origin/master), `docs/deeptime-design-decisions.md` (rows 2026-10-02
  and 2026-10-22), `docs/research/notes/vertical-scale-study.md`.
