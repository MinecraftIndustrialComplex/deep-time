# Which projection for a production seamless world? A survey with measurements

Researched 2026-10-03 for Deep Time, after Ben's answer in the decisions log (2026-10-03): "Send off some research on
what the current best projections would be for this". The question: which projection, layout and seam set should a
production seamless world use, now that seams may be joined by Immersive Portals (translations, half-turns and
quarter-turns), or by Deep Time's own wrap engine, which Ben asked to be started alongside. It follows
[`world-projection-options.md`](world-projection-options.md) (2026-10-02, the projections under Toroidal World's
translation-only rule), [`rotated-seam-worlds-prior-art.md`](rotated-seam-worlds-prior-art.md),
[`immersive-portals-pillowcase-prototype.md`](immersive-portals-pillowcase-prototype.md) and the prototype in
[`../../design/seamless-wrapping.md`](../../design/seamless-wrapping.md). It changes no shipped code: Lambert's
cylindrical equal-area projection stays the shipped world.

**Tags.** **[C]** computed by me with the tools in `tools/research/projection-survey/` on the test VM (§9 says how);
**[V]** read by me in the source; **[SA]** read in the source by a Sonnet sub-agent this session, its reading recorded
in §10; **[I]** my inference or derivation, reasoning given; **[M]** from memory, not checked.

---

## 0. The answer in plain words

- **Quarter-turn seams change the answer.** With translations only, the best seamless world is the Peirce family
  (two half-turn seams, four bad points). With quarter-turns allowed, a **cube** becomes possible: six square faces,
  eight corners. A corner of a cube is a much milder defect than a Peirce singular point (a quarter of a turn missing,
  not half), so every kind of distortion can be smaller (§1). This is a theorem, not a tuning result.
- **Recommendation: an equal-area cube that is smooth across every seam, in a cross-shaped net.** [C]
  - *Projection.* The conformal cube map (Lee 1976, the "conformal cubed sphere" of climate models) made exactly
    equal-area by Moser's flow (§4). Measured: every block covers the same planet area (0.01 % rms, 0.6 % at
    worst, on the 513² table tested); shapes are at most 1.42× too long in one direction (median 1.26); there are no creases: the derivative is
    continuous across every seam and face edge (largest jump 2 %, the table's numerical floor). The eight corners are
    the only defects, and an equal-area map cannot do much better there (≥ 1.33 is forced, §1.3).
  - *Layout.* The cross net: four faces in a band round the equator (the equator one straight east–west line that
    wraps, as today), one face on each pole (the poles are ordinary walkable points). 16k: faces of 3,776 blocks
    (236 chunks); 32k: 7,552 (472 chunks), the same blocks per km² as today.
  - *Seams.* Seven seam pairs: one translation (the band's east–west wrap), four quarter-turns and two half-turns
    (the polar faces' other edges). 26,432 blocks of seam at 16k, against 19,632 for the Guyou prototype.
  - *Cost.* A precomputed table per face (computed once; about 0.4 MB with the cube's symmetry), a bilinear lookup
    of about 13 ns per block [C]. The construction is this note's own composition of two textbook pieces; nobody has
    published it as a projection (§2.4), so it needs its own tests.
- **If a closed formula is preferred: the same cube with QSC.** O'Neill and Laubscher's quadrilateralized spherical
  cube, which I measured to be the same map as Roşca and Plonka's 2011 equal-area cube (and, per the sub-agent's
  check, Snyder's equal-area cube) [C]. Exactly equal-area, shapes at most 1.55× (median 1.32), 16 ns per block in
  Roşca and Plonka's form. But it is creased: a straight ridge or coast crossing a face diagonal kinks by 41° on the
  map, and by up to 13° at a face edge.
- **Runner-up: Petroff's 2021 equal-area square in the Guyou 2:1 layout.** It keeps today's 2:1 grid, the prototype's
  seams (one slide, two half-turns) and all of its tooling: only `peirce.rs` changes. Exactly equal-area and smooth
  across all of its seams; tiny creases (≤ 5°) inside each hemisphere. The price: shapes up to 2.67× (median 1.78),
  about twice the cube's, and four singular points of the strong kind. 370 ns per block.
- **Ruled out for production:**
  - the conformal Peirce/Guyou map of the prototype (area 0.46× to unbounded);
  - HEALPix (a cube in disguise, but with 63° creases on the polar faces);
  - Gringorten (a 62° crease along its seams, iterative);
  - the near-equal-area cube warps used in graphics (S2, equi-angular, COBE, Nowell): creased at every edge, up to
    44°, and not equal-area;
  - Lee's tetrahedral map: a pillowcase like Peirce, and its rectangle has an irrational side ratio;
  - octahedral, icosahedral and Dymaxion nets: they need 60°, 72° or 120° turns, which a block grid cannot do.
- **Questions for Ben** (§8): cube (recommended) or keep the 2:1 Guyou world; the smooth table map (recommended) or
  closed-form QSC; the cross net (recommended) or a 3 × 2 rectangle; today's scale (recommended) or power-of-two
  faces.

---

## 1. What the seams allow: a short piece of geometry

### 1.1 A seamless world is a flat sphere with corners [I, V]

Take any world assembled from flat pieces of block grid whose edges are glued by translations, half-turns and
quarter-turns, and which covers the planet once. Walking round any point and adding up the angle gives 360° except at
finitely many **cone points**, where the total is smaller. By Gauss–Bonnet the missing angles add up to 720°
(Thurston 1998: "the sum of the apex curvatures is 2π times the Euler characteristic" [SA]; for a sphere that is
4π = 720°). Because every gluing is a multiple of 90°, every cone angle is 90°, 180° or 270°. So the possible
worlds are:

| Cone points | Missing angle each | Examples |
|---|---|---|
| 4 × 180° | 180° | the **pillowcase**: Peirce, Guyou, Adams, Gringorten, Collignon quincuncial, Petroff; Lee's tetrahedron (on a skewed lattice) |
| 8 × 270° | 90° | the **cube** and every map built on it: gnomonic, S2, equi-angular, COBE, QSC, HEALPix/rHEALPix, the conformal cube |
| 3 (90°, 90°, 180°) | 270°, 270°, 180° | a square with each pair of adjacent edges glued by a quarter-turn (the p4 orbifold); the worst of all |
| mixtures (e.g. 2 × 180° + 4 × 270°) | | a box with two square ends, and so on; none better than the cube |

Translations alone (Toroidal World) allow only half-turn-free worlds, which is why the 2026-10-02 note found that a
seamless torus needs two copies of the planet. Half-turns allow the pillowcase. **Quarter-turns are what make the cube
possible.** Octahedra (240° corners), icosahedra (300°) and the Dymaxion net need turns of 120° or 72°, which do not map
a block grid onto itself.

### 1.2 Fewer missing degrees per corner is less distortion [I]

Near a cone point of angle θ the planet's full 360° is squeezed into θ of world. Two consequences:

- **Equal-area maps must stretch shapes there.** A world disc of radius ρ round the corner has area θρ²/2 and
  perimeter θρ. Equal area maps it onto a planet patch of the same area, whose perimeter is at least √(2πθ)·ρ (the
  isoperimetric inequality, small discs). So the stretch along the circle averages at least √(2π/θ), and since area
  is kept, the shape ratio a/b (how many times too long one direction is drawn relative to the other) is at least
  **2π/θ** somewhere on every small circle. For the pillowcase (θ = 180°) that is **2**; for the cube (θ = 270°) it is
  **4/3**. (The 2026-10-02 note gave the pillowcase case.)
- **Conformal maps must blow up area there.** The developing map near a cone point is z ↦ z^{θ/2π} (Thurston 1998 §8
  [SA]), so planet distance per block behaves like r^{2π/θ − 1}: like r for the pillowcase (the prototype's "falls to
  zero like the distance"), like r^{1/3} for the cube. Blocks per unit planet area go like r⁻² near a Peirce point but
  only r^{−2/3} near a cube corner. Putman & Lin (2007) see exactly this: the conformal cubed sphere's smallest cell
  shrinks as N^{−4/3} [SA].

**The cube is the finest structure quarter-turns allow, and its corners are the mildest.** That is the case for it.
Everything below measures how close real maps come to these bounds.

### 1.3 What the bounds mean for the candidates [C]

| | Bound near each corner | Best measured |
|---|---|---|
| Pillowcase, equal-area | a/b ≥ 2 | Petroff 2.67, Moser-Peirce 2.85, Gringorten 2.88, Collignon 2.98 |
| Cube, equal-area | a/b ≥ 1.33 | Moser cube 1.42, QSC 1.55, HEALPix 2.41 |
| Pillowcase, conformal | area ∝ r⁻² | Peirce: 0.46× to ∞; 1 % of the planet drawn more than 4.6× too large |
| Cube, conformal | area ∝ r^{−2/3} | conformal cube: 0.79× to ∞; 1 % of the planet more than 2.2× too large |

---

## 2. The candidates

### 2.1 The pillowcase family (half-turn seams; the prototype's class)

- **Peirce quincuncial (1879), Guyou (1887), Adams' squares (1925).** One conformal map in different aspects: the
  prototype's crate implements Peirce and Guyou. Adams' world-in-a-square II puts the four singular points at the two
  poles and two equator points, which is a 90° turn of Peirce's configuration, so it has Peirce's statistics and puts
  singular points on the poles [I]; Adams' hemisphere-in-a-square is Guyou's tile [I]. Not measured separately.
- **Equal-area relatives with the same layout.** Gringorten (1972; d3-geo-projection's port, iterative), the
  Collignon quincuncial (= Lambert azimuthal + Shirley–Chiu concentric square, closed form), and Petroff (2021, closed
  form both ways, CC0 code). Petroff's paper: Gringorten "has a smooth derivative, and thus no cusps, everywhere but
  the equator"; his own "contains no significant visible cusps" [SA].
- **Lee's conformal tetrahedral map (1965/1976).** Four 180° cone points again, but the tetrahedron's lattice is
  triangular. It fits a rectangle of side ratio 4 : √3 (left and right edges slide, the bottom edge folds onto
  itself about its midpoint, the top edge in two halves) [I], which is irrational: a block grid can only approximate
  it, and then the map is no longer Lee's.
- **Peirce made equal-area by Moser's flow** (this note, §4): measured as a check on the method.

### 2.2 The cube family (quarter-turn seams)

- **Gnomonic (central) cube**, and the warps of graphics: **equi-angular cubemap** (Google's VR video, the same as
  S2's tangent projection), **S2 quadratic** (Google S2's default), **Nowell's spherified cube**, **COBE CSC** (Chan &
  O'Neill 1975, a fitted polynomial, nearly equal-area). Formulas and licences in §7.
- **Exactly equal-area cubes.** **QSC** (O'Neill & Laubscher 1976, as Calabretta & Greisen 2002 and PROJ give it),
  **Roşca & Plonka 2011** ("Lambert-type" equal-area cube), and **Snyder's 1992 equal-area polyhedral map on the
  cube**. These are one map: my measurements of QSC and Roşca–Plonka agree in every statistic to the last printed
  digit, and their outputs over 4.2 million blocks sum to the same value to 10⁻⁹ relative [C]; the sub-agent found
  brsr's closed form of Snyder's cube equal to PROJ's QSC to 1.4·10⁻¹³ [SA]. Dimitrijević & Strobl (2025): Roşca and
  Plonka's map "retains the same properties [as QSC] while offering greater numerical stability" [SA].
- **HEALPix (Górski et al. 2005) and rHEALPix.** HEALPix's equatorial zone is cylindrical equal-area and its polar caps
  are four triangular "teeth" each. Measured as a flat surface it has exactly the cube's structure: eight 270° cone
  points, at latitude ±41.8° [C, I]. rHEALPix only folds the teeth into a polar square. So HEALPix is an equal-area
  cube map with its corners moved; it also fits a **2:1 rectangle** (band plus teeth, with the gaps between teeth
  unreachable) whose seams run at 45°, and its band is today's Lambert projection with the standard parallel at
  22.9° (x scaled by cos 22.9° = 0.921) [I].
- **The conformal cube** (Lee 1976; Rančić, Purser & Mesinger 1996; the "RAN" projection of the reviews). Conformal,
  and "differentiable to any order at all points except the eight singularities" (Putman & Lin 2007 [SA]). The only
  open implementation the sub-agent found is CSIRO's CCAM, GPL-3: not used. I computed it independently from Schwarz
  triangle functions (§9).
- **The conformal cube made equal-area by Moser's flow** (this note, §4).

### 2.3 Not grid-compatible

- **Octahedral maps** (TOAST, TEA of McGlynn et al. 2019; the octahedral equal-area partition; Praun & Hoppe's
  geometry images): either 240° corners, or folded into a square as a pillowcase (Collignon, Petroff), which §2.1
  covers.
- **Icosahedral and Dymaxion maps, Snyder's icosahedral equal-area (ISEA):** 300° corners, 72°/120° turns.
- **Rhombic-dodecahedral nets with rhombic faces:** 109.5°/70.5° corners. (HEALPix's twelve base pixels are a rhombic
  dodecahedron drawn with square pixels, which is why it lands in the cube class.)
- **Lambert cylindrical (shipped):** not a closed world at all; one x seam and a pole band. Kept as the reference row.

### 2.4 Recent work (after 2015)

- **Cube-map reviews in graphics and geodesy.** Dimitrijević, Lambers & Rančić (2016) compared cube maps for
  planet-sized terrain rendering and recommended the equi-angular cube ("ASC is probably the best choice") [SA].
  Lambers (2020) recommended an algebraic-sigmoid warp (area within ±12 %) and called QSC "the method with the highest
  quality by a wide margin" but costly [SA]. Dimitrijević & Strobl (2025) rank seventeen cube maps: only the conformal
  and "relaxed conformal" (Adcroft et al. 2004) cubes are C¹ across edges; QSC and HEALPix have "abrupt transitions
  on both edges and diagonals"; MSC and ALG rank highest overall [SA].
- **Smooth cubed spheres in climate models.** Purser & Rančić (1998) relax conformality for uniformity; Rančić et al.
  (2017) describe a "uniform Jacobian" cube smooth to any order across edges, without published formulas [SA,
  abstract only]. All keep the eight vertex singularities.
- **Nothing I or the sub-agents found is exactly equal-area and C¹ across cube edges.** Zucker & Higashi (2018)
  refit the COBE polynomial to about 1 % area error, still kinked [SA]. Optimal-transport area-preserving
  parameterizations (Zhao et al. 2013; Nadeem et al. 2017) are mesh methods, not projections [SA, abstracts]. The
  Moser construction of §4 fills that gap for this project; it is the same flow that diffusion cartograms use (Gastner
  & Newman 2004 [M]).
- **Square equal-area maps:** Petroff 2021 is the newest; nothing newer for the square was found.

---

## 3. Measurements

All numbers are [C]: each map's inverse (block → planet) sampled on 300 × 300 cells per face or hemisphere tile, its
Jacobian by central differences, statistics weighted by planet area (method in §9). "Blocks per planet area" is
relative to the planet mean, so 1 is equal-area; "a/b" is the shape ratio; "2ω" the largest angle error. "Slope jump"
ε is the relative jump of the derivative across a line (the terrain's slope changes by up to ε × 100 % there; 0 for a
smooth map); "kink" is the angle by which a straight planet feature crossing the line at right angles bends on the map.
"Drift" is how fast a straight walk turns on the globe, in degrees per 1,000 blocks at the 16k scale.

### 3.1 Area and shape

| Map (class) | Blocks per planet area: min / p1 / median / p99 / p99.9 / max | Within ×1.25 / ×2 of mean | a/b median / p90 / p99 / max | 2ω mean / max | Drift median / p90 |
|---|---|---|---|---|---|
| **Moser conformal cube** (cube) | 1.000 everywhere (max 1.006, the table's error) | 100 % / 100 % | **1.26 / 1.39 / 1.41 / 1.42** | **12.1° / 19.9°** | 4.4° / 25° |
| **QSC = Roşca–Plonka** (cube) | 1 exactly | 100 % / 100 % | 1.32 / 1.48 / 1.53 / 1.55 | 16.0° / 24.8° | 3.2° / 19° |
| HEALPix / rHEALPix (cube) | 1 exactly | 100 % / 100 % | 1.18 / 1.95 / 2.36 / 2.41 | 15.7° / 48.9° | 7.5° / 22° |
| Conformal cube (cube) | 0.79 / 0.79 / 0.92 / 2.17 / 3.86 / ∞ | 87 % / 98.6 % | 1 | 0 | 3.6° / 14° |
| COBE CSC (cube) | 0.88 / 0.99 / 1.00 / 1.03 / 1.05 / 1.05 | 100 % / 100 % | 1.23 / 1.50 / 1.57 / 1.72 | 12.3° / 30.8° | 5.0° / 17° |
| Nowell (cube) | 0.91 / 0.92 / 1.00 / 1.08 / 1.16 / 1.20 | 100 % / 100 % | 1.20 / 1.51 / 1.67 / 1.72 | 11.7° / 30.8° | 5.4° / 18° |
| Equi-angular = S2 tangent (cube) | 0.85 / 0.85 / 1.00 / 1.18 / 1.20 / 1.20 | 100 % / 100 % | 1.20 / 1.43 / 1.61 / 1.73 | 11.0° / 30.9° | 4.8° / 12° |
| S2 quadratic (cube) | 0.80 / 0.80 / 0.96 / 1.46 / 1.59 / 1.65 | 89 % / 100 % | 1.22 / 1.40 / 1.61 / 1.73 | 12.2° / 30.9° | 4.3° / 13° |
| Gnomonic (cube) | 0.52 / 0.53 / 0.91 / 2.19 / 2.53 / 2.70 | 36 % / 98 % | 1.20 / 1.43 / 1.61 / 1.73 | 11.0° / 31.0° | 0 (great circles) |
| **Petroff 2021** (pillowcase) | 1 exactly | 100 % / 100 % | 1.78 / 2.45 / 2.63 / 2.67 | 30.9° / 54° | 5.8° / 40° |
| Moser Peirce (pillowcase) | 1.000 (max 1.05) | 100 % / 100 % | 1.69 / 2.60 / 2.82 / 2.85 | 29.5° / 57° | 8.1° / 41° |
| Gringorten 1972 (pillowcase) | 1 exactly | 100 % / 100 % | 1.79 / 2.58 / 2.80 / 2.88 | 29.7° / 57.9° | 10° / 42° |
| Collignon quincuncial (pillowcase) | 1 exactly | 100 % / 100 % | 2.05 / 2.52 / 2.82 / 2.98 | 38.8° / 59.7° | 5.8° / 19° |
| Guyou / Peirce (pillowcase, prototype) | 0.46 / 0.46 / 0.79 / 4.6 / 14.5 / ∞ | 34 % / 86 % | 1 | 0 | 7.3° / 19° |
| Lee tetrahedral (pillowcase) | 0.62 / 0.62 / 0.75 / 4.4 / 13.7 / ∞ | 27 % / 95 % | 1 | 0 | not measured |
| *Lambert cylindrical, cap 80° (shipped)* | 1 exactly | 100 % / 100 % | 1.33 / 4.8 / 20 / 27 | 29.0° / 137° | 11° / 34° |

Checks that the tools are right [C]:
- Peirce's figures (0.457, 0.79, 4.6, 14.5) match the prototype crate's `stats` (SCL-075), by two independent routes
  (Jacobi cn, and Schwarz triangle functions).
- Petroff's mean 2ω (30.9° = 0.539 rad) and max a/b (2.67, i.e. ω = 0.95 rad) match his Table 1 (0.54 and 0.95 rad,
  [SA]).
- QSC's max a/b 1.55 matches Dimitrijević & Strobl's 1.555 and HEALPix's 2.41 their 2.430 [SA].
- The gnomonic cube's area ratio 2.70/0.52 = 5.2 matches S2's "5.200" [SA].
- The gnomonic cube's drift is 0, as it must be: it maps straight lines to great circles.
- Collignon, Gringorten, HEALPix, QSC: area constant to 10⁻⁷ or better; Petroff too, apart from the cells below.

Two edge cases:
- **Petroff's minimum and maximum cells.** A handful of cells at points where Petroff's inverse divides 0 by 0 are
  excluded from min and max; its p0.1–p99.9 are clean.
- **The conformal cube near its corners.** It stays within ×2.5 of the mean everywhere farther than 5° from a corner,
  ×1.8 beyond 10°, ×1.3 beyond 20°. Peirce: 11.5, 5.8, 3.1.

### 3.2 Smoothness across seams and inside faces

| Map | Across seams and face edges: slope jump ε max (median); kink | Inside faces: where; ε; kink |
|---|---|---|
| **Moser conformal cube** | **0.02 (0.002); 0.6°**, the table's numerical floor (the conformal map through the same table shows 0.02) | none |
| Conformal cube, Guyou/Peirce | 0 (analytic across every seam) | none |
| **Petroff 2021** | **0** across the seams (the equator and the half-turn edges) | the hexadecant lines (tile midlines and diagonals): ε ≤ 0.087, kink ≤ 5° |
| Moser Peirce | 0.36 at the half-turn seams, only next to the four singular points (median 0.002) | none |
| QSC = Roşca–Plonka | edges: 0.23 (0.15); ≤ 13° | **face diagonals: 0.87 everywhere; 41°** |
| HEALPix | the polar faces' edges (the 41.8° parallel): 0.96 (0.48); ≤ 44° | **polar faces' diagonals: 2.0 everywhere; 63°** |
| Gnomonic, equi-angular, S2, Nowell, COBE | every edge: 0.96–1.06; 44–47°. For the gnomonic cube ε = \|v\| exactly at position v ∈ [−1, 1] along the edge: 0 at the midpoint, 1 at a corner [I] | none (warps) |
| Gringorten | **the seams (the equator): 1.9 (0.32); 62°** | diagonals 0.36 (median 0); 20° |
| Collignon quincuncial | 2.0 at every seam; 63° | diagonals 2.0; 63° |
| Lambert cylindrical | 0 (the x wrap) | none; the pole band is a wall |

What this means in the world [I]:
- **A crease shows wherever terrain has a slope across it.** Deep Time's terrain is the simulation's rasters, sampled
  bilinearly, plus the planet's own detail noise sampled at the planet point. Both pass through the projection, so a
  crease bends every ridge, valley, river and coastline that crosses it. At ε = 0.87 (QSC's diagonals) a uniform slope
  changes by up to 87 % at the line: a hillside visibly folds. A circular volcano centred on a QSC face diagonal comes
  out as a teardrop with a corner.
- **For comparison, bilinear interpolation itself has slope kinks** at every raster cell edge (16 blocks on L1, 2 on
  L3). Those are small and random in direction. A projection crease is a straight line hundreds of kilometres long
  through every feature, so it would read as an artefact on maps and from the air.
- **A crease is not a seam.** Creases sit where the formula changes, seams where portals are; QSC has creases on its
  diagonals, where nothing else happens. The recommended map has neither kind of discontinuity in its derivative.

### 3.3 Singular points, seams and walks

| Layout | Cone points | Seam pairs and transforms | Seam length at 16k (identified / portal faces) | Straight walks |
|---|---|---|---|---|
| **Cube, cross net** (band on the equator, poles at face centres) | 8 × 270°, at ±35.3° latitude, longitudes λ₀ + 45° + k·90° | 7: 1 translation, 4 quarter-turns, 2 half-turns | 7s = 26,432 / 52,864 (s = 3,776) | The equator is one straight east–west line of 4s that wraps. A meridian walk from band face 0 crosses the pole face and comes out on face 2 heading south, as over a real pole; from faces 1 and 3 it enters the polar face through a quarter-turn portal (invisible to the player). Two walkers passing either side of a corner end up at 90° to each other. |
| Cube, 3 × 2 rectangle | 8 × 270° | 8 (one of them an interior line of 3 faces) | 8s = 30,208 / 60,416 | The same, but one more seam inside the world. |
| HEALPix 2:1 (band + teeth) | 8 × 270° at ±41.8° | 1 translation + 8 quarter-turns along 45° lines | 6.66s = 25,137 / 50,274 | Seams at 45° to the block grid: IP can place them, a block-level wrap engine handles them badly. |
| **Guyou 2:1** (prototype) | 4 × 180°, at ±45° latitude | 3: 1 translation, 2 half-turns (each long edge onto itself) | 3b = 19,632 / 39,264 (b = 6,544) | The equator is straight; over a pole the walk comes out heading back south. Two walkers passing either side of a cone point end up going opposite ways. |
| Peirce square | 4 × 180°, on the equator | 4 half-turns (each edge onto itself) | 2a = 18,496 / 36,992 (a = 9,248) | The equator is four legs with a 90° turn at each singular point (prototype §2). |
| Lambert cylindrical (shipped) | none; two pole caps | 1 translation (Toroidal World) | 5,136 (the x wrap to 80°) | East–west is straight; north ends at the pole band. |

In every layout every seam is an isometry of the block grid, so **a straight walk stays straight across every seam**;
what changes is only the walker's heading in raw coordinates, which the portal hides. Where the singular points fall
matters less for the cube: on an equal-area cube a corner magnifies nothing (only a quarter turn is missing), so it
need not be put at sea as the Peirce points must. The cube still has one free rotation about the polar axis (λ₀),
and the bake may use it to keep corners off steep land.

### 3.4 Fit to the block grid

Same planet area per block as today's 16k and 32k worlds (blocks² = 4πR², R = C/2π):

| Layout | 16k (C = 16,384, R = 2,608) | 32k (C = 32,768) | Power-of-two option |
|---|---|---|---|
| Cube, face s = √(2π/3)·R | s = 3,774 → **3,776 = 236 chunks**; cross 15,104 × 11,328 (6 faces of 14.3 M blocks) | s = 7,547 → **7,552 = 472 chunks** | s = 4,096 / 8,192: the planet drawn 1.18× larger in area |
| Guyou, hemisphere b = √(2π)·R | b = 6,536 → 6,544 = 409 chunks; world 13,088 × 6,544 | b = 13,072 = 817 chunks | b = 8,192 / 16,384 (the prototype's): 1.57× larger |
| Peirce square a = √2·b | 9,248 | 18,480 | |
| HEALPix 2:1 | 15,104 × 7,552 (25 % of the rectangle unreachable between the teeth) | 30,208 × 15,104 | |

Every face edge and every turn centre then lies on chunk boundaries or chunk centres, so a wrap engine can alias whole
chunks (a quarter-turn about a chunk corner and a half-turn about a chunk centre both map chunks onto chunks) [I].
Today's rasters are 2:1 strips; the cross net needs either six face rasters or a 4s × 3s box with half of it unused.

### 3.5 Cost of the inverse per block [C]

Single thread on the test VM (AMD EPYC, `rustc -C opt-level=3 -C target-cpu=native`), 4.2 million blocks per map,
`tools/research/projection-survey/bench.rs`:

| Inverse | ns per block | Notes |
|---|---|---|
| Gnomonic cube | 4.3 | normalise only |
| Bilinear table, 513 × 513 per face (the Moser cube) | 13.4 | plus the face lookup; bicubic would be about 2× |
| Roşca–Plonka (= QSC) | 16.4 | one sin/cos pair, a sqrt |
| Equi-angular cube | 16.7 | two tan |
| Lambert cylindrical (shipped) | 31.0 | asin, sin/cos |
| HEALPix | 45.5 | |
| QSC in PROJ's form | 108 | same map as Roşca–Plonka, six times slower |
| Guyou/Peirce (cn by the AGM) | 201 | two real sncndn |
| Petroff 2021 | 369 | many trig calls |
| Gringorten (bisection, 50 steps) | 2,022 | |

**For Deep Time none of these costs matters.** Ben chose to project only at the bake (decisions log 2026-10-03, "Bake
only"). A 16k cube world has about 21 million L3 cells (2 blocks), 1.3 million L2 and 0.3 million L1 cells. Even
Gringorten takes 46 s on one thread; the table map takes 0.3 s. The forward map (planet → block, for rivers,
structures and the rest of the vector data) costs about the same as the inverse for the closed forms; for the table
map it is a second table.

---

## 4. The smooth equal-area cube: how it is built [C, I]

1. **Start from the conformal cube.** On one 48th of the cube, the triangle from a face centre C to an edge midpoint
   E to a corner V, the flat triangle has angles 45°, 90°, 45° and the planet's triangle 45°, 90°, 60°. Both map to the
   upper half plane: the flat one by Schwarz–Christoffel,
   z(t) = A·t^{1/4}/(1/4)·₂F₁(1/4, 1/2; 5/4; t), and the planet's by Schwarz's triangle function,
   ζ(t) = B·t^{1/4}·₂F₁(13/24, 5/24; 5/4; t) / ₂F₁(7/24, −1/24; 3/4; t), with ζ the stereographic coordinate from the
   face centre. A and B come from Gauss's sum at t = 1 (E is 45° from C on the planet). Symmetry gives the rest of the
   cube. The same code with angles (45°, 90°, 45°) and (45°, 90°, 90°) reproduces Peirce to four digits, which checks
   it.
2. **Measure its area.** The conformal face has planet area per block ρ, smooth, largest at the face centre
   (1.26 × the mean) and falling to 0 at the corners.
3. **Flow it to equal area (Moser 1965 [M]).** Solve ∇²u = ρ − 1 on the face with zero normal derivative (exact,
   because face edges and diagonals are mirror lines of the whole cube; a cosine transform on 512 × 512 cells), then
   move every world point along dx/dt = −∇u / ((1 − t) + t·ρ) from t = 0 to 1 (RK4, 400 steps). The end map φ sends
   the uniform measure to ρ, so G = conformal ∘ φ covers equal planet area with every block.
4. **Why it is smooth.** φ keeps every edge and diagonal in place and is mirror-symmetric across them, so its
   Jacobian there is diagonal in edge coordinates, and G's derivative is continuous across every edge exactly when the
   conformal map's is, which it is (analytic continuation across a face edge). Measured: slope jumps ≤ 0.02, the same
   as the conformal map pushed through the same table, so they are interpolation error, not creases.
5. **What it costs to ship.** A table per face, built once (80 s in Python on one core; a Rust port would take
   seconds) or committed as data: 513² vectors per face, or one 48th of that with the symmetry (about 0.4 MB as f32).
   The inverse is a lookup; the forward map is a second table on the face's gnomonic grid. The table's accuracy here
   is about 10⁻⁵ rad (0.03 blocks at 16k) at the seams, which the production version should beat by using bicubic
   interpolation or a finer table and by snapping seam samples to the exact edge.

Applied to Peirce, the same flow gives an equal-area pillowcase smooth everywhere except at its four singular points,
with shapes slightly worse than Petroff's at the worst point (2.85 against 2.67) and slightly better at the median
(1.69 against 1.78). Petroff's closed form is the better pillowcase.

---

## 5. Ranking

Scored for a production Deep Time world (equal area is Ben's standing choice, 2026-10-02: "Use an equal area projects
FFS"; seams through Immersive Portals now and a wrap engine later).

| Rank | Projection and layout | Equal area | a/b median / max | Corners | Seams (16k) | Creases | Inverse | Licence of reference code |
|---|---|---|---|---|---|---|---|---|
| **1** | **Moser-equalized conformal cube, cross net** | yes (table, 10⁻⁴) | **1.26 / 1.42** | 8 × 270° | 7: T + 4Q + 2H; 26,432 | **none** | 13 ns (table) | none needed: built here from textbook pieces (Schwarz–Christoffel, Schwarz triangle functions, Moser's flow) |
| 2 | QSC = Roşca–Plonka (= Snyder cube), cross net | exactly | 1.32 / 1.55 | 8 × 270° | same | diagonals ε 0.87 (41°), edges ε ≤ 0.23 (13°) | 16 ns | paper formulas; PROJ `qsc.cpp` MIT |
| **3 (runner-up)** | **Petroff 2021, Guyou 2:1** | exactly | 1.78 / 2.67 | 4 × 180° | 3: T + 2H; 19,632 | inside hemispheres only, ε ≤ 0.09 (5°) | 369 ns | CC0 (Zenodo 4661351) |
| 4 | Conformal cube, cross net | no: 0.79× to ∞, 87 % within ×1.25 | 1 | 8 × 270° | as 1 | none | table or series | Schwarz evaluation here; CCAM's code is GPL-3 (not used) |
| 5 | HEALPix / rHEALPix, cross net or 2:1 with 45° seams | exactly | 1.18 / 2.41 | 8 × 270° at ±41.8° | 7, or 9 at 45° | polar diagonals ε 2.0 (63°), cap edges ε ≤ 0.96 (44°) | 46 ns | PROJ `healpix.cpp` MIT (HEALPix C++ and healpy are GPL: not used) |
| 6 | Moser Peirce, Guyou 2:1 | yes (table) | 1.69 / 2.85 | 4 × 180° | as 3 | none (0.36 next to the cone points) | table | built here |
| 7 | Gringorten, Guyou 2:1 | exactly | 1.79 / 2.88 | 4 × 180° | as 3 | **the seams** ε 1.9 (62°) | 2,022 ns | d3-geo-projection ISC |
| 8 | Guyou / Peirce conformal (prototype) | no: 0.46× to ∞ | 1 | 4 × 180° | as 3 | none | 201 ns | own (crate) |
| 9 | Near-equal-area cube warps (COBE, Nowell, equi-angular/S2, S2 quadratic) | within 5–65 % | ≈1.2 / 1.73 | 8 × 270° | as 1 | every edge ε ≈ 1 (44–47°) | 4–20 ns | S2 Apache-2.0; formulas |
| 10 | Collignon quincuncial (= Lambert azimuthal + Shirley–Chiu) | exactly | 2.05 / 2.98 | 4 × 180° | as 3 | ε 2.0 everywhere on seams and diagonals | cheap | formulas |
| — | Lee tetrahedral | no | 1 | 4 × 180° | rectangle 4 : √3, not exact on blocks | none | series | d3-geo-polygon ISC |
| — | Octahedral, icosahedral, Dymaxion, rhombic | — | — | 240°/300°/… | need 60°/72°/120° turns | — | — | — |
| ref | Lambert cylindrical (shipped) | exactly | 1.33 / 27 at 80° | pole caps | 1 slide | none | 31 ns | own |

Why this order [I]:
- **1 over 2:** the same seams, cube and cost class, and the smooth map is better on every distortion number as well
  as having no creases. It loses only on provenance: QSC is published and in PROJ, the smooth map is new and must be
  tested here.
- **2 over 3:** QSC's shapes are half as distorted as Petroff's (median 1.32 against 1.78, max 1.55 against 2.67) and
  its corners are milder; but QSC is creased and needs the cube layout, while Petroff is smooth across its seams and
  keeps the prototype's 2:1 world. If the 2:1 world is worth more than the distortion, 3 moves to the top of the
  alternatives.
- **4:** shapes are perfect and the map is smooth, but area spreads by ×1.25 over 13 % of the planet and without
  bound at the corners. Ben asked for equal area.
- **5:** HEALPix has the best median shape of all (its band is cylindrical) but the worst creases of the equal-area
  cubes, and its tail (2.41) is worse than QSC's.
- **7, 8:** Gringorten's crease lies exactly on the seams, where the prototype's continuity checks look; Peirce is not
  equal-area.

---

## 6. The recommendation, the runner-up, and what each costs in `deeptime-quincuncial`

### 6.1 Recommended: the smooth equal-area cube in the cross net

- **Projection:** the conformal cube made equal-area by Moser's flow (§4), as a table per face; QSC by Roşca and
  Plonka's formulas as the fallback if the table does not pass its tests.
- **Layout:** the cross net. Four faces in a band centred on the equator at longitudes λ₀, λ₀ + 90°, λ₀ + 180°,
  λ₀ + 270° (x wraps every 4s, as today's x wraps every C); the north face above band face 0, the south face below it;
  the poles at the polar faces' centres; the eight corners at ±35.26° latitude.
- **Seam set** (faces numbered 0–3 in the band, N, S; `tools/research/projection-survey/survey.py` `CUBE_SEAMS`
  holds the exact table, checked to 10⁻¹⁵ rad):
  - band east edge ↔ band west edge: translation by 4s;
  - N east edge ↔ face 1 top edge: quarter-turn about their common corner;
  - N west edge ↔ face 3 top edge: quarter-turn the other way (through the x wrap);
  - N top edge ↔ face 2 top edge: half-turn about the point midway between them;
  - S east, west and bottom edges ↔ the bottoms of faces 1, 3 and 2: the same three, mirrored.
- **Immersive Portals:** fourteen one-way portals (seven pairs). The prototype's half-turn rotation NBT is
  `rotationA..D = (0, 0, 1, 0)`; by analogy a quarter-turn about +y would be (cos 45°, 0, ±sin 45°, 0), but that is my
  reading of the prototype's convention, not tested [I].
- **Deep Time's own wrap engine:** its design (`../../design/wrap-engine.md`, landed on master the same night) already
  takes any layout of translations, half-turns and quarter-turns, with a quarter-turn corner among its spike's test
  layouts, so the cube needs nothing beyond what it plans [V: its title and the lines on quarter-turns, not the whole document].

**Cost in the crate** (agent days):

| Piece | Days |
|---|---|
| `cube.rs`: face frames, the cross net, block ↔ (face, u, v) ↔ planet, the free longitude λ₀ | 1–2 |
| The Moser table: a Rust port of `moser.py` (Schwarz evaluation, DCT Poisson solve, RK4) or the table committed as data with its generator; bicubic lookup; the forward table; tests (equal area to 10⁻³, slope jumps < 0.01 across every seam, round trip) | 3–4 |
| `glue.rs`: quarter-turns in the seam table and the portal NBT; the seven-pair table and its `/summon` lines | 1 |
| `resample.rs`: six face rasters (or the 4s × 3s box) and footprint filtering; the polar caps sampled from the L0 sphere rather than Lambert rows (prototype §5) | 2–3 |
| `stats.rs`: the measurements of §3 for the cube | 0.5 |
| **Crate total** | **about 8–11 days** |

Outside the crate the production work of the design note's §8 (26–40 days for Guyou) grows: every reader that folds
the world must handle quarter-turns and a cross-shaped world instead of a 2:1 strip with two folding edges
(`FarTerrain`, Distant Horizons, the column rule, maps, the climate API's latitude). Estimate **+5 to +10 days** over
the Guyou plan [I].

### 6.2 Runner-up: Petroff's square in the Guyou 2:1 layout

- Same world as the prototype: two hemisphere squares side by side, b = 6,544 at 16k (409 chunks) or 13,072 at 32k,
  one slide and two half-turn seams, the equator straight. Only the map inside each hemisphere changes.
- **Cost in the crate:** `petroff.rs`, a port of Petroff's CC0 forward and inverse (about 300 lines; the inverse is
  already ported to numpy in `survey.py` and to Rust in `bench.rs`), swapped in for `peirce.rs` in `world.rs`; tests
  (equal area, round trip, the seams); the glue and the resampler unchanged. **1–2 days**, as the design note estimated.
- Accept: shapes up to 2.67× (half the planet worse than 1.78×), and four strong singular points (a cone of 180°).

---

## 7. Licences

| Source | Licence | What may be ported |
|---|---|---|
| Petroff, `new_projection.py` (Zenodo 4661351) | CC0 1.0 (file header) [SA]; the Zenodo record says CC-BY-4.0 for the archive | the code (ported here) |
| d3-geo-projection (Gringorten), d3-geo-polygon (Lee) | ISC, Mike Bostock [SA] | with the notice |
| PROJ (`qsc.cpp`, `healpix.cpp`, `s2.cpp`, `isea.cpp`) | MIT [SA] | with the notice |
| Google s2geometry (`s2coords.h`) | Apache-2.0 [SA] | compatible with LGPL-3.0; formulas are trivial |
| CSIRO CCAM `jimcc_m.f90` (the conformal cube's Taylor series) | GPL-3 [SA] | **nothing**; not used |
| WCSLIB | GPL or LGPL, the two papers disagree [SA] | not used; the papers' equations instead |
| HEALPix C++, healpy | GPL [M] | not used |
| Papers (Roşca & Plonka, Calabretta & Greisen, Petroff, Thurston) | formulas | re-implemented |
| This note's tools (`survey.py`, `conformal.py`, `moser.py`, `bench.rs`) | LGPL-3.0-or-later | ports of Petroff (CC0), d3 Gringorten (ISC) and PROJ QSC (MIT) inside; nothing GPL |

---

## 8. Open questions for Ben (recommendation first)

1. **The world's shape: a cube (recommended), or keep the 2:1 Guyou world?** The cube has eight mild corners and the
   least distortion quarter-turns allow (shapes ≤ 1.42×); the Guyou world keeps today's 2:1 grid and the prototype's
   seams but has four strong singular points and shapes up to 2.67× with Petroff's map.
2. **The cube's map: the smooth equal-area table (recommended), or QSC's closed form?** Same seams; the table map has
   no creases and slightly less distortion but is new and must be tested here; QSC is published and in PROJ but folds
   every ridge 41° on each face diagonal.
3. **The layout: the cross net (recommended), or a 3 × 2 rectangle?** The cross has seven seam pairs and a straight
   wrapped equator; the rectangle fits a rectangular raster but adds a seam inside the world.
4. **The scale: today's blocks per km² (recommended: faces of 3,776 at 16k, 7,552 at 32k), or power-of-two faces
   (4,096 / 8,192, the planet drawn 1.18× larger in area)?**

---

## 9. Method

- **Tools** (committed, LGPL-3.0-or-later): `tools/research/projection-survey/`.
  - `survey.py`: every map as tiles in a plane plus a seam table. It measures area, shape, drift and creases, and
    checks the seam table by continuity (every seam closes to 10⁻¹³ rad or better; Gringorten 10⁻⁹, the bisection's
    tolerance; the table maps 10⁻⁵, their interpolation error).
    - Distortion: 300 × 300 cells per tile, central differences with step 10⁻⁶ tile, singular values of the
      3 × 2 Jacobian.
    - Creases: 300 points along every seam, face edge, midline and diagonal. One-sided Jacobians are taken from
      samples at h, 2h, 3h off the line (h = 10⁻⁵), so a formula's branch exactly on the line does not matter.
      ε = ‖J₋⁺J₊ − I‖; the kink is the angle between the two drawings of the planet direction that crosses at right
      angles. The ends of each line (2 %) are skipped because the corners are cone points.
  - `conformal.py`: Schwarz triangle maps for the conformal cube, Peirce, Lee's tetrahedron and the octahedron, with
    scipy's hypergeometric function and Newton inversion, checked against mpmath at 200 points (10⁻¹⁵). The planet's
    area integrates to 1.0000.
  - `moser.py`: §4.
  - `bench.rs`: §3.5.
  - `run.sh`: runs them on the test VM through the admission tool.
- **Runs:** test VM, 2026-10-03/04, `tools/remote/mac.sh -- tools/remote/detach.sh start <name> --kind shell:python
  tools/research/projection-survey/run.sh all`. Results in `review/out/projection-survey/` (not committed). Each
  Python job took 17–80 s on one core.
- **Ports in the tools:** Petroff's inverse from his CC0 code; Gringorten's inverse from d3-geo-projection (ISC);
  QSC's inverse from PROJ (MIT); Roşca–Plonka, COBE and HEALPix from the equations; the gnomonic family from S2's
  documented formulas.
- **Not done:**
  - no game was launched;
  - nothing was resampled from a real bake onto a cube;
  - the Moser table was built at 512² only (a convergence study is part of the production tests);
  - Lee's tetrahedral and the octahedral maps were measured for area only;
  - Adams' squares were not measured (§2.1 argues they are Peirce in other aspects).

## 10. Sources and how they were read

FT = full text, AB = abstract only, CODE = source code, 2nd = through another source. [SA] = read by a Sonnet
sub-agent this session, which wrote transcriptions to the session scratchpad (not committed).

- Petroff, M. A. (2021). A square equal-area map projection with low angular distortion, minimal cusps, and
  closed-form solutions. ACM Trans. Spatial Algorithms and Systems 7(4):18. arXiv:2008.13670v3; code Zenodo
  10.5281/zenodo.4661351. FT + CODE [SA]; Table 1 reproduced by the sub-agent and by `survey.py`.
- Roşca, D. & Plonka, G. (2011). Uniform spherical grids via equal area projection from the cube to the sphere.
  J. Comput. Appl. Math. 236:1033–1041. Author preprint, FT [SA] (Eqs 12–13, 18–20 read from page images).
- Calabretta, M. R. & Greisen, E. W. (2002). Representations of celestial coordinates in FITS. A&A 395:1077.
  arXiv:astro-ph/0207413. §5.6 (TSC, CSC, QSC) FT [SA].
- O'Neill, E. M. & Laubscher, R. E. (1976), and Chan, F. K. & O'Neill, E. M. (1975): the QSC and COBE reports.
  2nd (Calabretta & Greisen; PROJ `qsc.cpp`) [SA]; not fetched.
- Calabretta, M. R. & Roukema, B. F. (2007). Mapping on the HEALPix grid. MNRAS 381:865. Draft FT [SA]; Calabretta
  (2004) arXiv:astro-ph/0412607 FT [SA].
- Górski, K. M. et al. (2005). HEALPix. ApJ 622:759. [M], not opened.
- Lee, L. P. (1976). Conformal Projections Based on Elliptic Functions. Cartographica Monograph 16. Not fetched (2nd).
- Rančić, M., Purser, R. J. & Mesinger, F. (1996). A global shallow-water model using an expanded spherical cube:
  gnomonic versus conformal coordinates. QJRMS 122:959–982. Paywalled; not read [SA tried].
- Putman, W. M. & Lin, S.-J. (2007). Finite-volume transport on various cubed-sphere grids. J. Comput. Phys.
  227:55–78. FT [SA].
- Dimitrijević, A., Lambers, M. & Rančić, D. (2016). Comparison of spherical cube map projections used in
  planet-sized terrain rendering. Facta Universitatis Ser. Math. Inform. 31(2):259–297. FT [SA].
- Lambers, M. (2020). Survey of cube mapping methods in interactive computer graphics. The Visual Computer
  36:1043–1051. Author preprint FT [SA].
- Dimitrijević, A. & Strobl, P. (2025). Hexahedral projections: a comprehensive review and ranking. ISPRS Int. J.
  Geo-Inf. 14(3):122. FT of the ranking sections [SA].
- Zucker, M. & Higashi, Y. (2018). Cube-to-sphere projections for procedural texturing and beyond. JCGT 7(2). FT
  [SA].
- McGlynn, T. et al. (2019). Octahedron-based projections as intermediate representations for computer imaging:
  TOAST, TEA, and more. ApJS 240:22. FT [SA].
- Thurston, W. P. (1998). Shapes of polyhedra and triangulations of the sphere. Geometry & Topology Monographs
  1:511–549. arXiv:math/9801088. FT of the introduction and §8 [SA].
- Adcroft, A. et al. (2004), MWR 132:2845; Purser, R. J. & Rančić, M. (1998), QJRMS 124:637; Rančić, M. et al.
  (2017), MWR 145:1083; Nadeem, S. et al. (2017), IEEE TVCG 23:1663; Zhao, X. et al. (2013), IEEE TVCG 19:2838.
  AB [SA].
- Moser, J. (1965). On the volume elements on a manifold. Trans. AMS 120:286–294. [M].
- Gastner, M. T. & Newman, M. E. J. (2004). Diffusion-based method for producing density-equalizing maps. PNAS
  101:7499. [M].
- Software: PROJ (`qsc.cpp`, `healpix.cpp`, `s2.cpp`, `isea.cpp`, MIT); d3-geo-projection (`gringorten.js`,
  `quincuncial/`, ISC); d3-geo-polygon (`tetrahedralLee.js`, ISC); Google s2geometry (`s2coords.h`, Apache-2.0);
  Nowell, P. (2005), "Mapping a cube to a sphere" (blog); Google VR blog (2017), the equi-angular cubemap. CODE /
  page text [SA].
- Earlier notes: `world-projection-options.md` (2026-10-02) for Lambert, Gringorten and Collignon figures, which
  §3 reproduces; `../../design/seamless-wrapping.md` for the prototype and the effort estimate.
