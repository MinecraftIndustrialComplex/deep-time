# Which projection should the Minecraft world use? Peirce quincuncial, its relatives, and Lambert

Researched 2026-10-02 for Deep Time, after Ben linked the Wikipedia page on the Peirce quincuncial projection and asked
whether there are newer or improved versions of it. This note is the research behind the "square or torus" branch of
[`../../design/equal-area-projection.md`](../../design/equal-area-projection.md) (§5.3 there). It changes no code.

**Tags.** **[V]** verified from the cited source, read as stated in the source list. **[C]** computed by me (method in
§7); the numbers are reproducible from the scripts described there. **[I]** my inference or a proof I wrote, with the
reasoning given. **[M]** from memory, not checked against a source.

---

## 0. The answer in plain words

- **What Peirce quincuncial is.** It is a conformal map of the whole planet onto a square: the north pole in the
  centre, the equator a diamond touching the middle of each side, and the southern hemisphere split into the four
  corners. Copies of the square tile the plane, but each neighbour is turned 180° about the midpoint of the shared edge.
  It does not tile by plain translation.
- **What it would mean for the world.** Toroidal World (TW) can loop x and z together, as a torus, but only by plain
  translation. The smallest Peirce tile that repeats by translation alone is a square holding **two copies of the
  planet**, the second turned 180°. It is not mirrored. The tile must be turned 45° (Guyou's arrangement) to line up
  with x and z. So in a Peirce world:
  - every continent appears twice, as two independent sets of blocks, so a base built in one copy is not in the other;
  - "north" points toward the nearest pole image, not up the z axis;
  - at four points on the equator the projection is singular, and its scale there goes to infinity;
  - it is not equal-area: blocks per km² vary from 0.46× the planet average at the poles to unbounded at those four
    points.
- **Is there a newer, equal-area version? Yes, three.**
  - **Gringorten (1972)**: needs iteration.
  - **The Collignon quincuncial**: an old, closed-form map built from four Collignon triangles per hemisphere. It is the
    same map as the Shirley–Chiu/Clarberg equal-area square used in graphics.
  - **Petroff (2021)**: closed form, and the lowest maximum shape distortion of the three.

  All three have Peirce's layout and tile the same way. So they inherit the same price: two copies of the planet,
  rotated north, and four branch points. In exchange their worst shape distortion is bounded: a block of land is drawn
  at most about 3× too wide (Petroff about 2.7×). Lambert draws it 33× too wide at the 80° stop line.
- **Nothing equal-area beats Lambert for a world that wraps in x only.** I prove in §3.1 that any equal-area world
  wrapping in x at a fixed circumference draws each parallel at least as badly as Lambert does. Lambert's single free
  setting is its standard parallel.
- **What the torus maps cost.** They trade Lambert's polar squash (12% of the planet lies above 60°) for:
  - every continent twice;
  - no "north is up";
  - about twice the chunks for the same detail;
  - a worse middle: Gringorten's median shape ratio is 1.79 against Lambert's 1.32, and it is worst in the tropics.
- **Recommendation: keep Lambert**, wrapping in x with the pole cap (§5). The only real alternative is an equal-area
  torus (Petroff or Gringorten), and only if Ben prefers bounded distortion and walkable poles to a single,
  north-up copy of the planet.

---

## 1. What Minecraft and Toroidal World allow

### 1.1 Toroidal World 0.19.3: what it can loop [V]

- **Project and version.** Modrinth project `toroidal-world`, id `CdjG7l5z`. The id `8YTSYi9y` is the **version**
  0.19.3+mc1.21.1 (Fabric and NeoForge, published 2026-09-27), not the project (Modrinth API `/v2/version/8YTSYi9y`).
- **Shapes.** The Modrinth description offers "Toroidal — both horizontal axes loop" and "Cylinder — one horizontal
  axis loops, the other goes on forever like vanilla; you pick which axis loops".
- **Widths.** The two axes may have different periods. `docs/modding.md` on branch `mc/1.21`:
  - "`LoopSpans.ofWidths(24, 48)` allows differing widths: X spans 24 chunks, Z spans 48";
  - "Which flat surface the world ends up being is read off the spans, never declared: both axes give a torus, one
    gives a cylinder, neither gives an ordinary world."
- **Size limits.** Periods are whole chunks, from 16 to 3,749,998 chunks per axis (`core/WorldLoopSizes`, decompiled).
  For a 1:8 Nether each period must also divide by 8 chunks (`wrapped_planet_worlds.md` §1.2).
- **Rotation or reflection at a seam: neither works today.** I decompiled `core/FlatShape`, `core/WorldFolds`,
  `core/FoldOrientation` and `core/DeckGroupFold` from the 0.19.3 NeoForge jar.
  - `FlatShape` names six identifications: RECTANGLE, CYLINDER, MOBIUS, KLEIN, TORUS and LATTICE_TORUS (a torus whose z
    seam also shifts x, `skewChunks`).
  - `WorldFolds.verifyFoldable` refuses every shape with a mirror and every skewed lattice, with "The wrap engine has no
    fold for … yet: its axes do not decompose" or "…its seam reverses the local indices inside a chunk".
  - There is no identification that turns at a seam (90° or 180°) at all.
  - A general `DeckGroupFold` class with a lattice ships in the jar, but no other class references it.
  - So what runs is rectangle, cylinder, or a torus with an axis-aligned rectangular lattice: **every seam is a pure
    translation along x or z.**
  - [I] The skewed lattice torus and the unused deck-group fold suggest the author plans more. A skewed lattice would
    let the triangle maps of §2.4 tile with two copies instead of four. Nothing is promised.

### 1.2 What follows for any projection [I]

1. **Wrap in x only, cap the rest.** The image of the planet is a band of fixed width C that repeats in x, and its top
   and bottom edges are hidden behind the pole cap. Every row of blocks therefore wraps at the same C. That rules out
   pseudocylindrical maps such as Equal Earth or Mollweide, whose parallels shorten toward the poles; at 80° an Equal
   Earth parallel is 0.62 of the equator [C].
2. **Wrap in x and z (a torus).** The image must tile the plane by translations along x and z alone, with the
   neighbour across every seam matching geographically.
   - A torus cannot cover the sphere once. Any continuous map from a torus onto the sphere that is one-to-one near
     almost every point is a branched covering. A degree-1 branched cover would be a homeomorphism, and a torus is not
     a sphere.
   - Riemann–Hurwitz (0 = 2d − Σ(e − 1)) makes the smallest cover d = 2, with exactly **four** simple branch points. At
     each, a small circle walked in the world goes twice around a point of the planet.
   - The orientation-preserving wallpaper groups whose quotient is a sphere are p2, p3, p4 and p6. They give 2, 3, 4 and
     6 copies per translation cell; p3 and p6 need a 60° lattice. With TW's axis-aligned lattice the counts become 2
     (p2), 4 (p4), 6 (p3) and 12 (p6).
   - **So the best a seamless torus can do is two copies of the planet with four branch points.** That is exactly the
     Peirce family.
3. **Anything whose natural neighbours are turned 90° or 180° across a straight edge cannot be walked seamlessly.**
   This covers cube maps, HEALPix/rHEALPix, Dymaxion and Snyder polyhedral maps. Such maps need teleports or portals at
   those edges (`wrapped_planet_worlds.md` §5.1).

---

## 2. The projections, one by one

### 2.1 Peirce quincuncial (1879), and Guyou (1887) and Adams (1925)

- **Construction.** Stereographic projection of a hemisphere onto a disk, then Schwarz's conformal map from disk to
  square. The southern hemisphere's four triangles sit around the northern square, a quincunx (Wikipedia [V]; PROJ
  `peirce_q` docs [V]). The inverse is the Jacobi elliptic function cn with modulus 1/√2: the stereographic coordinate
  is ζ = cn(w), with the north pole at w = K [I, checked numerically, §7].
- **Singular points.** Exactly four, all on the equator, 90° of longitude apart, at the midpoints of the square's
  sides. Wikipedia: "conformal everywhere except at the four corners of the inner hemisphere (thus the midpoints of
  edges of the projection), where the equator and four meridians change direction abruptly" [V].
  - At these points an angle of 360° on the planet becomes 180° on the map, and the map's linear scale goes to infinity
    like r^−½ [I]. The Guyou page says the same: "the deformation is infinite" there [V, Wikipedia].
  - Which longitudes they fall on is a free choice of aspect (§4.3).
- **Area distortion [C].**
  - Over the sphere, the ratio of largest to smallest area scale is **infinite**.
  - With the planet average set to 1, blocks per unit planet area are smallest at the poles, 0.457. Quantiles weighted
    by planet area are:

    | Quantile | 1 % | 50 % | 95 % | 99 % | 99.9 % |
    |---|---|---|---|---|---|
    | Area scale | 0.46 | 0.79 | 2.08 | 4.6 | 14.5 |

  - Excluding discs around the four singular points, the spread is:

    | Disc radius excluded | Share of planet excluded | Max/min area scale over the rest |
    |---|---|---|
    | 5° | 0.8 % | 11.5× |
    | 10° | 3.0 % | 5.9× |
    | 20° | 12 % | 3.1× |

  - Peirce's own claim, "the part where the exaggeration of scale amounts to double that at the centre is only 9% of the
    area of the sphere" (quoted on Wikipedia [V]), is consistent with these numbers.
- **Tiling [V + I].**
  - Wikipedia [V]: it "tessellates the plane … each copy's features exactly matching those of its neighbors".
  - Petroff 2021 [V] states the quincuncial maps tile "with adjacent tiles rotated by 180° (p2 wallpaper group)". Praun &
    Hoppe 2003 [V] extend their octahedral square "by rotating it 180° around the midpoints of boundaries".
  - [I] The analytic continuation across a side is the Schwarz reflection, which in the square is a half-turn about that
    side's midpoint. Two half-turns about adjacent side midpoints compose to a translation along the square's diagonal.
  - **The translation lattice** is generated by (s, s) and (s, −s) for a square of side s. This is a square lattice
    turned 45°, with cell side s√2 and area 2s², so **two copies of the planet per cell**. The second copy is the first
    turned 180° about a singular point, not mirrored.
  - In Peirce's own orientation the lattice is diagonal, so TW cannot use it as drawn. Turned 45°, the cell lines up
    with x and z.
  - In that frame the world is a 2×2 checkerboard of hemisphere squares of side L/2 (N S / S′ N′), and TW loops both
    axes with the same period L. This is Guyou's arrangement: Wikipedia says the four southern triangles "can be
    rearranged as another square … equivalent to the transverse aspect of the Guyou hemisphere-in-a-square projection"
    [V].
- **Relatives.**
  - Guyou (1887) and Adams (1925, hemisphere-in-a-square, world-in-a-square I and II) are the same elliptic map in other
    aspects: the hemisphere-in-a-square conformal map is unique up to its symmetries [I].
  - Adams world-in-a-square II puts the poles at two corners, which makes them two of the four singular points (ArcGIS
    docs via a search snippet [2nd-hand]).
  - Implementations: PROJ `peirce_q` (since PROJ 9.0; shapes square, diamond, horizontal, vertical and hemispheres; "The
    resulting tile can be infinitely tessellated" [V]), `guyou`, `adams_ws1`, `adams_ws2` and `adams_hemi`; d3
    `geoPeirceQuincuncial` and `geoGuyou` (sub-agent [2nd]).
- **Cost per column.** The inverse (block to planet) is cn of a complex argument. That is closed form in terms of real
  sn, cn and dn, which are cheap through the arithmetic–geometric mean. The forward map needs an incomplete elliptic
  integral of complex argument (Carlson R_F), or Lee's series [I].

### 2.2 Equal-area relatives with the same layout (the answer to "newer versions")

All three below are equal-area, square, quincuncial (north pole in the centre, south pole in the corners), and tile by
the same p2 rule, so two planets per TW cell. Petroff says of all three: "can be seamlessly tiled, with adjacent tiles
rotated by 180° (p2 wallpaper group)" [V].

| | Gringorten 1972 | Collignon quincuncial | Petroff 2021 |
|---|---|---|---|
| Construction | Derived from differential equations; no closed form (Petroff [V]). d3's port inverts by bisection on latitude, then closed form for longitude; forward by Newton or bisection [V, d3 source] | Each octant is a Collignon triangle; parallels are concentric diamonds. Also named the equal-area zenithal orthotriangular, the octahedral equal-area partition, and TEA (McGlynn et al. 2019) [V, via Petroff and McGlynn] | Generalised van Leeuwen–Strebe "slice-and-dice" on the octahedron, with the face split optimised (φ₀ = 3π/8) [V] |
| Forward / inverse | Iterative / iterative | Closed form; a square root and one trig call [V, I] | Closed form both ways; forward about 8 trig evaluations after simplification [V] |
| Mean max-angular-deformation 2ω | 29.7° [C]; 0.52 rad = 29.8° (Petroff [V]) | 39.0° [C]; 0.68 rad = 39.0° (Petroff [V]) | 0.54 rad = 30.9° (Petroff [V]) |
| Worst 2ω (shape ratio a/b) | 59.2° at grid resolution, 60° in the limit (a/b → 3), at the four branch points [C]; 1.05 rad (Petroff [V]) | 60° (a/b → 3) at the branch points [C]; 1.05 rad [V] | 0.95 rad = 54.4° (a/b ≈ 2.69) [V] |
| Shape ratio, planet-weighted median / 90 % / 99 % | 1.79 / 2.59 / 2.80 [C] | 2.05 / 2.52 / 2.82 [C] | not recomputed |
| Share of planet with a/b ≤ 1.5 / ≤ 2 | 36.8 % / 65.3 % [C] | 8.3 % / 45.3 % [C] | not recomputed |
| Where the distortion sits (normal aspect) | Near the equator. Median a/b by latitude: 2.54 (0–5°), 2.29 (5–15°), 1.96 (15–30°), 1.57 (30–45°), 1.21 (45–60°), 1.06 (60–75°), 1.01 (75–90°) [C]. Petroff: Gringorten "concentrates angular distortion near the equator" [V] | Everywhere, about 1.65 near the poles rising to 2.6 at the equator [C]; visible creases along the octant edges (Petroff [V]) | "More spread out than the Gringorten projection"; "no significant visible cusps" [V] |
| Implementations | d3 `geoGringorten` (each hemisphere a square) and `geoGringortenQuincuncial` [V, source]; not in PROJ [2nd] | Graphics code (Clarberg 2008); McGlynn's TEA; no d3 or PROJ name [2nd] | Paper appendices and Zenodo code (DOI 10.5281/zenodo.4661351); no d3 or PROJ [2nd] |

- **Same map, two names [I, C].** The Lambert-azimuthal-plus-Shirley–Chiu "concentric" square, the usual equal-area
  hemisphere square in graphics, is the Collignon quincuncial turned 45°. Both put sin φ = 1 − r² on nested squares and
  run longitude linearly along each square. Every distortion statistic I computed for the two agrees to the last digit.
- **A lower bound for every equal-area torus [I].** Take a world disc of radius ρ around a branch point. Its image
  covers a planet disc of area πρ²/2 twice. By the isoperimetric inequality that disc's rim is at least √2·πρ long,
  and the world circle (length 2πρ) runs around it twice. So the mean stretch along the circle is at least √2. Equal
  area then forces a shape ratio of at least 2 somewhere on every small circle around each branch point. Gringorten and
  the Collignon quincuncial reach 3 there; Petroff stays below 2.7 everywhere. I found no published map that
  approaches 2.
- **Cogley 2002**, "Variations of the Gringorten square equal-area map projection" (CaGIS 29(4)), gives other
  arrangements of Gringorten (citation only, not read).

### 2.3 Equal-area maps that wrap in x: Lambert and its family

- **The family.** Cylindrical equal-area: x = R·λ·cos φs, z = R·sin φ / cos φs. The standard parallel φs is the one
  free setting.
  - φs = 0 is Lambert, today's plan.
  - φs = 30° is Behrmann, 37.5° is Hobo–Dyer, 45° is Gall–Peters [M for the names].
  - Shape is true at ±φs. The shape ratio is (cos φs / cos φ)² poleward of φs, and its inverse equatorward [I].
- **Shape ratio a/b ("drawn N times too wide") [C].**

  | φs \ latitude | 0° | 15° | 30° | 45° | 60° | 70° | 75° | 80° | 85° |
  |---|---|---|---|---|---|---|---|---|---|
  | 0° (Lambert) | 1.00 | 1.07 | 1.33 | 2.00 | 4.00 | 8.55 | 14.9 | 33.2 | 132 |
  | 30° | 1.33 | 1.24 | 1.00 | 1.50 | 3.00 | 6.41 | 11.2 | 24.9 | 99 |
  | 37.5° | 1.59 | 1.48 | 1.19 | 1.26 | 2.52 | 5.38 | 9.40 | 20.9 | 83 |
  | 45° | 2.00 | 1.87 | 1.50 | 1.00 | 2.00 | 4.27 | 7.46 | 16.6 | 66 |

- **Over the walkable planet [C].**

  | Cap at | φs | a/b ≤ 1.5 | a/b ≤ 2 | Median | Max | Mean 2ω | Planet hidden |
  |---|---|---|---|---|---|---|---|
  | 80° | 0° | 58.6 % | 71.8 % | 1.32 | 33.2 | 29.0° | 1.5 % |
  | 80° | 30° | 71.8 % | 80.3 % | 1.30 | 24.9 | 24.9° | 1.5 % |
  | 75° | 0° | 59.8 % | 73.2 % | 1.30 | 14.9 | 27.0° | 3.4 % |
  | 75° | 30° | 73.2 % | 81.8 % | 1.30 | 11.2 | 23.0° | 3.4 % |

  - The z extent of the world, cap to cap, is 0.31·C at φs = 0 and 0.42·C at φs = 30°.
  - At a fixed planet size, φs = 30° shrinks C by cos 30° = 0.87 and lengthens z by 1.15.
- **Cost per column.** One asin, or one sin. The rasters on a regular (x, z) grid are exactly equal-area.

### 2.4 Other square and tiling maps

- **Lee tetrahedral (Lee 1976 monograph; Lee 1965 and 1973 papers) and Cox (1935).**
  - Both are conformal maps of the world into an equilateral triangle, built from elliptic (Dixon) functions [2nd:
    Wikipedia, visionscarto]. Cox's map is conformal, not equal-area; the brief's "Cox equal-area triangle" is wrong
    [2nd].
  - They have four singular points at the tetrahedron's vertices, each with a cone angle of 180°, so they are p2 again,
    on a 60° lattice [I].
  - TW cannot loop a 60° lattice (skew is refused, §1.1). An axis-aligned cell holds **four** planets [I].
  - In d3-geo-polygon as `geoTetrahedralLee` and `geoCox` [2nd].
  - Petroff [V]: an equal-area square from a tetrahedron is possible "but such a projection would have increased angular
    distortion"; "the other Platonic solids cannot be mapped onto the square without discontinuities".
- **HEALPix (Górski et al. 2005 [M]; Calabretta & Roukema 2007).**
  - Equal-area. Lambert cylindrical (z ∝ sin φ) for |sin φ| ≤ 2/3 (φ ≤ 41.8°), with four Collignon "teeth" per
    polar cap [V, Calabretta preprint].
  - Its distortion is the best of anything here [C]:

    | a/b by latitude | 0–30° | 30–41.8° | 41.8–90° |
    |---|---|---|---|
    | Median | 1.11 | 1.29 | about 1.7 |

    Over the whole planet the median is 1.18 and the maximum 2.42. 91 % of the planet has a/b ≤ 2, and the mean 2ω is
    15.7°.
  - **But each polar cap is cut along four meridians from 41.8° to the pole.** Across a cut the true neighbour is the
    next tooth turned 90°. In flat-metric terms the cut ends are cone points of 270°, which no isometric tiling can
    realise [I]. A HEALPix world needs eight walls, or teleports with a 90° turn, running from 41.8° to the poles.
  - rHEALPix (Gibb, Raichev & Speth, Landcare Research draft, DOI 10.7931/J2D21VHM) only rearranges the teeth into one
    north and one south square (a cube-like cross of six squares). It has the same local distortion and the same
    problem at the polar squares' edges [V, sub-agent read the full draft; I].
  - Its real use for Deep Time is as an equal-area grid for the simulation, not as the world.
- **Cube maps: the COBE quadrilateralized spherical cube (Chan & O'Neill 1975), the exact equal-area QSC (O'Neill &
  Laubscher 1976), and Snyder's equal-area polyhedral maps (Snyder 1992, Cartographica 29(1)).**
  - Corners have cone angles of 270° (cube), 240° (octahedron with equilateral faces) or 300° (icosahedron), so none
    tiles by isometries [I].
  - COBE is only nearly equal-area: area errors of +13.7 % and −4.1 % at face edges (Calabretta & Greisen 2002, read by
    sub-agent [V]).
  - Not walkable without rotated portals (`wrapped_planet_worlds.md` §5.1, option 5).
- **Equal Earth (Šavrič, Patterson & Jenny, 2018–19).** Equal-area pseudocylindrical, for reference only.
  - Over |φ| ≤ 80°: median a/b 1.41, maximum 9.9, mean 2ω 27.5° [C, from the published polynomial, coefficients M].
  - Its parallels shorten to 0.62 of the equator at 80°, so it cannot wrap at a fixed period [C].
- **Recent work.** I found no cartography paper (2010–2026) on equal-area maps made to tile a torus.
  - Closest: Petroff 2021, and McGlynn et al. 2019 (ApJS 240:22, octahedral TOAST and TEA: "Adjacent tiles are rotated
    by 180 degrees" [V, sub-agent]).
  - Red Blob Games' game-dev notes on wraparound sphere maps [2nd], and papers using Peirce for periodic physics
    textures (Marco et al. 2024, APL Photonics) and spherical-harmonic plots (Schaefer 2026, arXiv:2601.17045) [abstracts
    only].

---

## 3. Two results that settle most of the question

### 3.1 No equal-area world that wraps in x beats Lambert, latitude by latitude [I]

- **Claim.** Take any equal-area world that wraps in x with period C and hides the poles behind caps. Every parallel in
  the walkable band separates the two caps, so its image is a closed loop that goes once around the cylinder, and its
  length is at least C. On the planet that parallel is 2πR·cos φ long. So the stretch along it averages at least
  C / (2πR cos φ) = cos φs / cos φ, where cos φs := C / 2πR is set by the circumference and the planet's size in blocks.
- **Consequence.** With equal area (ab = 1), the shape ratio is at least the square of that stretch. So poleward of φs:
  - the **largest** a/b on the parallel is ≥ (cos φs / cos φ)²;
  - by Jensen, so is its **mean** along the parallel.
- **Lambert attains both bounds everywhere.**
  - Equal Earth, Mollweide, sheared or sinuous cylinders, and oblique variants cannot do better on any parallel. Oblique
    variants are measured against their own oblique parallels.
  - The only freedoms left are φs, where the caps go (§4.3), and how far the cap reaches.

### 3.2 A seamless torus always shows the planet at least twice (§1.2) [I]

- With TW's axis-aligned lattice, the minimum is two copies. It is reached only by the p2 family (Peirce, Gringorten,
  Collignon quincuncial, Petroff), with four branch points.
- Every equal-area member has a shape ratio of at least 2 next to each branch point (§2.2).

---

## 4. The candidates that fit Minecraft, ranked

### 4.1 The table

"World area" is relative to today's Lambert world with a cap at 80°, at the same detail (blocks per km²). The tori use
a square TW cell of side L = √(8π)·R ≈ 0.80·C. At C = 16,384 that is about 13,070 blocks (816 chunks, a multiple of 8);
at 32,768 it is 1,632 chunks.

| Rank | Projection and topology | Equal area? | Worst / median shape ratio (walkable) | What the player sees | TW setting | Pole cap | Distant Horizons | Simulation rasters | Per-column cost |
|---|---|---|---|---|---|---|---|---|---|
| **1 (baseline, recommended)** | Lambert cylindrical equal-area, φs = 0; wrap in x, cap at 75–80° | Yes | 33.2 at 80° (14.9 at 75°) / 1.32 | North is up and east is +x, matching the sun. True shape at the equator; 2× too wide at 45°, 4× at 60°, 15× at 75°. One copy of everything. Seam is one north–south line. | Cylinder, X = C | Needed; hides 1.5 % (80°) or 3.4 % (75°) | As now (TW handles DH across the x seam, 0.19.3 changelog [V]) | Regular (x, z) grid, every cell equal-area; poles outside the grid | Trivial (asin) |
| 1a | Same, standard parallel 30° (Behrmann-like) | Yes | 24.9 / 1.30 | Equator drawn 1.33× too tall; better from 37° poleward; 80 % of the planet within 2× | Cylinder; C shrinks by 0.87 at fixed planet size | Same | Same | Same | Trivial |
| 2 | **Petroff 2021 torus** (Guyou-style 2×2 checkerboard) | Yes | 2.69 / median not computed; mean 2ω 30.9°, close to Gringorten's 29.7° [V] | Each continent twice, the second turned 180° and independent. Poles are ordinary walkable points at the centres of the hemisphere squares. North points radially to the nearest pole image, so it is not up and does not match the sun's east–west. Four branch points on a great circle where the world is point-symmetric; put them in ocean. Distortion spread evenly, about 2× in the tropics. | Torus, X = Z = L | None | Periodic in both axes. From a pole, the nearest duplicate is L/√2 ≈ 9,200 blocks away; near a branch point the duplicate is close (mirror through the point), so DH can show it | Two square rasters (N and S hemispheres), equal-area cells, folded edges | Closed form, ~8 trig forward |
| 3 | Gringorten torus | Yes | 2.95 / 1.79 | As 2, but the distortion is concentrated in the tropics (median 2.5 at the equator, about 1.0 above 60°) and there is a crease at the equator | Torus, X = Z = L | None | As 2 | As 2 | Iterative (bisection, about 50 steps); precompute per chunk |
| 4 | Lambert, oblique aspect: caps over two antipodal ocean points | Yes | As 1, around the oblique poles | Both real poles walkable, but climate bands run diagonally and north is not up | Cylinder | Over two ocean points | As 1 | Same grid in oblique coordinates | Trivial plus a rotation |
| 5 | Collignon quincuncial torus (= LAEA + Shirley–Chiu) | Yes | 2.99 / 2.05 | As 2 with worse typical distortion and visible creases along meridians and the equator | Torus | None | As 2 | As 2 | Closed form, cheapest torus |
| 6 | Peirce / Guyou torus | **No** (conformal) | Shape 1; area scale 0.46× to ∞ | As 2, but land near the four singular points is magnified without bound (5.9× spread even after cutting 10° discs) | Torus | None | As 2 | Not equal-area; cells cover 0.46× to unbounded planet area | Elliptic functions |
| — | HEALPix / rHEALPix | Yes | 2.42 / 1.18 | Best shapes, but 8 cut walls from 41.8° to the poles (teleport with a 90° turn to cross) | Cylinder plus walls | Teeth replace the cap | Breaks at the cuts | Excellent equal-area grid (a DGGS) | Closed form |
| — | Lee tetrahedral, Cox | No (conformal) | — | Four copies per TW cell until TW runs skewed lattices | Torus | — | — | — | Series or elliptic |
| — | Cube, QSC, Snyder polyhedral, Dymaxion, Equal Earth | — | — | Not tileable by translation, or rows of unequal length | — | — | — | — | — |

### 4.2 What the torus rows mean in play [I]

- **Layout.** One TW cell is a 2×2 checkerboard of hemisphere squares, N S above S′ N′. The primes mark the 180°-turned
  second copy.
  - Walking straight in +z from a north pole crosses the equator, the south pole, the equator again, and returns to
    the same north-pole image after L blocks.
  - Walking along a diagonal from a pole leads into a branch point.
- **Two copies.** They are separate blocks. A player who settles "Africa" in one copy finds an untouched, rotated Africa
  half a cell away. Exploring the whole planet means generating twice the area. On a server, two players at "the same
  place" can be in different copies.
- **Climate and sky.** Climate bands become concentric squares around each pole image, and the vanilla sun moves
  along x everywhere. Compasses, maps and any "north" UI need per-position rules (design note §5.3).
- **No pole cap.** Polar ice sheets and the poles themselves become walkable, which Lambert can never offer.

### 4.3 Where to put the bad spots

- **Lambert.** Its distortion grows with latitude, and the cap hides only the last 10–15°. An oblique aspect (row 4)
  could put the caps over open ocean on the baked planet, at the cost of north-up.
- **Torus maps.** The four branch points lie on one great circle, 90° apart. Keeping the poles at the square centres
  leaves one free rotation, about the polar axis, to put all four equatorial points in ocean. For Earth, longitudes
  −20°, 70°, 160° and −110° are all sea [M, from general geography]. Dropping the poles-at-centres constraint gives
  three free rotations. The bake can search for the best rotation.

---

## 5. Recommendation and the honest trade-off

**Keep Lambert's cylindrical equal-area projection, wrapping in x with the pole cap, as decided on 2026-10-02.**

1. **It is provably the best.** §3.1 shows that no equal-area map that wraps in x draws any parallel better than
   Lambert with the same circumference. Peirce's newer equal-area relatives do not change that, because they need the
   torus.
2. **The torus's price is structural, not a matter of tuning.** Every seamless torus world shows the planet at least
   twice (§3.2), and none keeps north up. TW 0.19.3 can run the torus (equal X and Z periods) but cannot rotate or
   mirror at a seam, so the Peirce-style "single square" with turned neighbours is not walkable.
3. **Lambert is better where most land usually is.** Its median shape ratio is 1.32 against Gringorten's 1.79 and
   Collignon's 2.05. It is perfect at the equator, where the torus maps are at their worst.
4. **It costs half the chunks for the same detail, and it is trivial to compute.**

**The trade-off, plainly.**
- Lambert draws the high latitudes badly: 4× too wide at 60°, 15× at 75°, 33× at the 80° stop line. That band (60° to
  the cap) is about 12 % of the planet.
- The equal-area tori draw nothing worse than about 3× too wide (Petroff 2.7×) and make the poles walkable. They pay
  for it with:
  - two independent, rotated copies of every continent;
  - no north-up;
  - about 2× tropical distortion;
  - twice the chunks.
- If Ben values bounded distortion and walkable poles more than a single north-up planet, the right torus is
  **Petroff 2021** (closed form, lowest worst case), with Gringorten as the fallback.

**Two cheap knobs on Lambert, for Ben to decide.** Neither is a recommendation to reverse his 2026-10-02 answer.
- **Standard parallel 30°.** Same cap; the share of the planet within 2× rises from 72 % to 80 %; the worst case falls
  from 33 to 25. The equator is then drawn 1.33× too tall.
- **Cap line.** Where to draw it: 75° hides 3.4 % and limits the worst case to 15; 80° hides 1.5 % with a worst case
  of 33. The design note's §6 has the band sizes.

---

## 6. What I could not verify

- **Primary sources read only second-hand.**
  - Peirce 1879 (Am. J. Math. 2(4) [M for volume and pages]), Lee 1976, Gringorten 1972 (AMS and DTIC returned 403 to
    the sub-agent), Snyder's Professional Paper 1395 and *Flattening the Earth*, and Górski et al. 2005: all through
    Wikipedia, Petroff, Calabretta, or d3 and PROJ docs.
  - Snyder 1992's distortion figures were not found.
  - The singular-point locations of Adams' world-in-a-square I were not confirmed.
- **Petroff's projection was not reimplemented.** Its numbers are the paper's (Table 1, 10,000 Fibonacci points). Its
  shape-ratio quantiles in §2.2 are therefore missing.
- **Methods agree, not proven equal.** My Gringorten and Collignon means match Petroff's Table 1 to 0.01 rad, which
  suggests "angular distortion" there is the same 2ω I computed. His maxima (1.05 rad) match my limits at the branch
  points.
- **Equal Earth coefficients are from memory.** The map is equal-area for any coefficients, so only the quoted shape
  numbers depend on them.
- **TW behaviour comes from reading code**, not from running a torus world. Behaviour of DH, Sable or Create across a z
  seam was not tested. The TW changelog claims DH and Xaero/JourneyMap work "beyond edges".
- **The ocean longitudes for Earth's equator** in §4.3 are from general knowledge.
- **No game was launched**, and no Deep Time code was read beyond the design note and the decisions log.

---

## 7. Method

- **Distortion.**
  - Python and NumPy on the Deep Time test VM. Each projection's inverse (block to latitude/longitude) was sampled on a
    regular grid of 800² to 1,200² points per hemisphere square, or 2,400 × 4,800 for HEALPix. The 3-D planet position
    was differenced centrally (h = 10⁻⁶).
  - The singular values a ≥ b of the block-to-planet Jacobian give the shape ratio a/b, 2ω = 2·asin((a − b)/(a + b)),
    and the area scale ab. Statistics are weighted by planet area. Lambert's figures are analytic.
  - Equal-area checks: the coefficient of variation of ab was below 10⁻⁷ for Gringorten, Collignon, HEALPix and Equal
    Earth.
- **The four maps.**
  - Peirce: ζ = cn(w | m = ½) through the complex-argument addition formula.
  - Gringorten: a vectorised port of d3-geo-projection's `gringortenRaw.invert`. Its [−1, 1]² square is one hemisphere
    in transverse aspect, with the rim on the boundary and the corners at (±45°, ±90°). It was rotated so that the
    hemisphere's centre is the pole.
  - Collignon quincuncial: sin φ = 1 − r², with longitude linear along each diamond.
  - HEALPix: Calabretta's inverse with H = 4, K = 3.
- **Torus assembly.** A 2×2 checkerboard of one hemisphere-in-a-square map, mirrored across each edge with latitude
  negated. This is valid for any map that sends the equator to the square's boundary (§2.1).
- **Comparison image.** `review/out/projections/projections.png` (not committed).
  - Panels: Lambert with the cap, HEALPix with its cuts, and the Peirce, Gringorten and Collignon tori.
  - Colour is the shape ratio, or the area scale for Peirce.
  - Overlays: 6° circles on a 30°×45° grid (Tissot-style), a 30° graticule, and two test "continents", F and L. These
    show each one twice on the tori, the second copy turned 180°.
  - The scripts were kept only in the session scratchpad; the VM scratch directory was deleted.
- **Toroidal World.** CFR 0.152 decompile of the 0.19.3 NeoForge inner jar (`libs/ref/twx/…/toroidal_world-neoforge-0.19.3+mc1.21.1.jar`)
  and a byte search of every class for references to `DeckGroupFold` and `latticeTorus`; plus `docs/modding.md` on
  GitHub and the Modrinth project and version API.

## Sources

How each was read: **FT** full text, **AB** abstract or landing page, **2nd** through a secondary source, **SA** read by
a Sonnet sub-agent this session (its tags are carried over).

- C. S. Peirce, "A quincuncial projection of the sphere", American Journal of Mathematics 2 (1879). 2nd (Wikipedia
  "Peirce quincuncial projection", read today).
- Wikipedia, "Guyou hemisphere-in-a-square projection". FT.
- PROJ, `peirce_q` operation page (proj.org). FT. PROJ index for `guyou`, `adams_ws1`, `adams_ws2`, `qsc`, `rhealpix`:
  SA, 2nd.
- d3-geo-projection `src/gringorten.js` (GitHub, main). FT (source code). d3 README entries: SA, 2nd.
- I. I. Gringorten, "A square equal-area map of the world", J. Applied Meteorology 11(5):763–767 (1972),
  doi:10.1175/1520-0450(1972)011<0763:ASEAMO>2.0.CO;2. 2nd (via Petroff; original returned 403).
- M. A. Petroff, "A square equal-area map projection with low angular distortion, minimal cusps, and closed-form
  solutions", ACM Trans. Spatial Algorithms and Systems 7(4), art. 18 (2021), doi:10.1145/3460521, arXiv:2008.13670v3.
  FT (arXiv PDF, §§1–3 and Table 1).
- J. G. Cogley, "Variations of the Gringorten square equal-area map projection", CaGIS 29(4):381–390 (2002). Citation
  only.
- E. Praun & H. Hoppe, "Spherical parametrization and remeshing", ACM TOG 22(3) (SIGGRAPH 2003). SA, FT.
- P. Clarberg, "Fast equal-area mapping of the (hemi)sphere using SIMD", J. Graphics Tools 13(3):53–68 (2008). SA, AB.
- T. McGlynn et al., "Octahedron-based projections as intermediate representations for computer imaging: TOAST, TEA,
  and more", ApJS 240:22 (2019), arXiv:1812.03926. SA, FT.
- D. Roşca, "New uniform grids on the sphere", A&A 520:A63 (2010); Holhoş & Roşca, Comput. Math. Appl. 67(5) (2014).
  SA, AB or citation.
- L. P. Lee, *Conformal Projections Based on Elliptic Functions*, Cartographica Monograph 16 (1976). 2nd (catalogue,
  Wikipedia "Lee conformal world in a tetrahedron").
- J. P. Snyder, "An equal-area map projection for polyhedral globes", Cartographica 29(1):10–21 (1992). 2nd; figures not
  found. Snyder, *Map Projections: A Working Manual*, USGS PP 1395 (1987): not opened.
- M. R. Calabretta, "Mapping on the HEALPix grid", arXiv:astro-ph/0412607 (2004). SA, FT. Calabretta & Roukema, MNRAS
  381(2):865–872 (2007). SA, AB.
- K. M. Górski et al., "HEALPix: a framework for high-resolution discretization…", ApJ 622:759 (2005). M (not opened).
- R. Gibb, A. Raichev & M. Speth, "The rHEALPix discrete global grid system", Landcare Research draft,
  doi:10.7931/J2D21VHM. SA, FT.
- M. R. Calabretta & E. W. Greisen, "Representations of celestial coordinates in FITS" (A&A 395:1077, 2002), for COBE
  and QSC area errors. SA, FT.
- B. Šavrič, T. Patterson & B. Jenny, "The Equal Earth map projection", IJGIS 33(3):454–465 (2019). M (formula from
  memory, not opened).
- Toroidal World: Modrinth API `/v2/project/toroidal-world` and `/v2/version/8YTSYi9y` (FT); GitHub
  `anyttng/toroidal_world` `docs/modding.md`, branch `mc/1.21` (FT, via fetch); 0.19.3 jar decompiled (FT).
- Red Blob Games, "Wraparound square tile maps on a sphere" (2019). SA, 2nd. Marco et al., APL Photonics 9:110803
  (2024); Schaefer, arXiv:2601.17045 (2026); K.-T. Chen, arXiv:2209.13251 (2022). SA, AB.
