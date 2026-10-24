# Meteor impacts

Status of this page: `stage 2 (constants checked against sources on 2026-10-02)`.

Ledger ids on this page: `IMP-001` to `IMP-141` ([ledger/IMP.md](ledger/IMP.md)). Of the 135 active
rows (6 more were merged into rows of other ledgers by WP-0), 43 are `verified`, 6 `abstract-only`,
9 `cited-unverified`, 6 `from-memory`, 56 `game-choice` and 15 `uncited`. At the start of stage 2 none
was `verified`.

**How far this has been checked.** Every crater-shape equation in the code was read in its source,
Collins, Melosh & Marcus 2005 (called CMM05 below): the transient crater, the simple and complex
final diameters, the breccia lens, the rim, the ejecta and the melt. They match, with two things
that did not. The depth of complex craters was coded as 0.294 D^0.301 km against the paper's
0.4 D^0.3 km (27 % too shallow); it is now the paper's. Three equation numbers in comments were wrong.
The ocean rule was a straight line fitted to three numbers from an abstract; it is now the relation
in the full paper (Davison & Collins 2007). The rate was the largest finding: the code carried the
size slope of large craters (1.8) down to 2 km, and the published crater record (Hughes 2000) shows
that small craters are far less common than that; the production function below 20 km now follows
the record, which makes the 2 to 20 km craters 17 times rarer. What is **not** checked: the
diffusivity that decides how long craters last (no source could be read), the gravity scaling of
the peak-ring onset (not supported by what was read), the whole of the impact-ore invention (grades,
tonnages and densities are placeholders), and the rate constant itself, which was read only as
quoted in a later paper because its own source is a book chapter.

Scale used throughout: one block is 300 m across and 20 m up.

## What you see in the world

Craters are rare. The default rate is 3 times Earth's (Ben, decisions log 2026-10-02, "Go for a 3x
default": a game choice, Earth's own rate is 1). On the default planet (1.5 % of Earth's area) a
500 Myr history then makes about 145 craters (one run: 145 formed, 25 still existing at the end, 8
showing as landforms, 7 of them on continents), most of the rest lost to the sea (an impactor
smaller than about a kilometre leaves no crater in deep water), erosion, burial and subduction.
At Earth's own rate the same run made 49 (7 existing, 2 showing). The impact rate is a setting
(Terrain: "Meteor impacts"; 1 is Earth's rate, 3 the default, 10 "Heavy").

The smallest crater is 2 km across (7 blocks). Up to 3.2 km (11 blocks) a crater is a simple bowl
with a raised rim. Larger craters are "complex": a flat floor, slumped walls and a central peak
about a fifth of the crater's width across. From about 36 km (120 blocks) the central peak gives way
to a ring of peaks halfway to the rim. The largest crater a run may make is 300 km (1,000 blocks).

Craters are drawn at half their true depth by default, because at 15 times vertical stretch a true
2 km bowl would be 7 blocks across and 21 deep. At the default, a 2 km crater is about 11 blocks
deep when fresh, a 5 km crater about 16 and a 20 km crater about 25 (the complex-crater depth law
was corrected from the first coding, in which a 20 km crater was 18 blocks deep).

Under the floor lies impact breccia (rock shattered and mixed by the impact; cobblestone until a
pack supplies a block), and in larger craters a sheet of impact melt rock on the floor (smooth
basalt). A thin blanket of breccia lies around the rim.

Craters age. Soil creep rounds the rim and shallows the bowl, rivers cut the rim, sediment fills
the floor, and a lake may stand in it. An old crater may show nothing at the surface and survive
only as a lens of breccia in the rock column under later layers. Craters on ocean floor are carried
into trenches with their plate.

Four kinds of ore go with craters: nickel, copper and platinum-group metals under very thick melt
sheets (as at Sudbury, Canada); zinc, copper and gold veins from hot water circulating in the
shattered rock; diamonds formed by shock where the target held carbon; and uranium in the sediment
that fills a crater basin. All four are too rare to be a world's only source of those materials,
and their sizes and grades are invented placeholders (see "Game choices").

## The model in plain words

The impact stage runs once per plate step, after the volcano stage.

1. **How many.** The expected number of impacts in a step is the rate of craters of at least the
   smallest simulated size (2 km) per square kilometre per year (IMP-001, IMP-004), times the
   planet's own area (IMP-007), the step length and the rate setting (IMP-006). The rate is
   5.6 × 10⁻¹⁵ craters of 20 km or more per km² per year, and 3.7 times that for craters of 2 km or
   more. The actual number is drawn from a Poisson distribution, the standard distribution for
   counts of rare independent events (IMP-009).
2. **Where and how large.** Each impact lands at a point chosen evenly over the sphere (IMP-010).
   Its size is drawn from the production function, between 2 km and 300 km, and is read as the
   crater the body would make on Earth's land (IMP-011, IMP-012). Above 20 km the number of craters
   wider than D falls as D^−1.8 (IMP-003); below 20 km it rises much more slowly with falling size
   (about D^−0.3 at 2 to 5 km), following the measured record (IMP-004).
3. **From crater to impactor.** That Earth crater is turned back into an impactor diameter by
   inverting the scaling law at Earth's gravity, for a rocky body of 3,000 kg/m³ arriving at 20 km/s
   at 45 degrees (IMP-020, IMP-021, IMP-024, IMP-029). Every impactor has that same speed, density
   and angle.
4. **The sea.** If the point is under water, the crater is made smaller or not made. Let R be the
   water depth divided by the impactor's diameter. The final crater is 1 − 0.14 R of the dry one;
   none at all if R is 8 or more; and a body that would dig a complex crater on land (over 3.2 km)
   leaves only a crater-less scour, a "resurge feature", if R is over 3.5 (IMP-065, IMP-066). In
   4 km of water that means only bodies wider than about a kilometre leave a crater (IMP-071).
5. **From impactor to crater on this planet.** The transient crater, the bowl just after
   excavation and before its walls collapse, comes from the scaling law at this planet's gravity
   (IMP-030). The final crater is 1.25 times wider if simple, and wider still by a power law if
   complex (IMP-031, IMP-032). The change from simple to complex is at 3.2 km on Earth and moves
   inversely with gravity (IMP-034, IMP-036).
6. **Depth, rim, breccia, melt.** Rim height, the volume and thickness of the breccia lens, the
   depth and the melt volume follow from further laws of the same paper (IMP-037 to IMP-042,
   IMP-062, IMP-063). Everything vertical is then multiplied by the depth setting (IMP-130).
7. **Into the world.** A crater at least two coarse grid cells across (9.6 km on the default
   planet) is cut into the terrain the deep-time simulation erodes, and its breccia and melt are
   written into the rock record. A smaller one is drawn only at block scale, with its breccia as a
   separate body (IMP-132, IMP-093, IMP-094), which the bake seats under the floor the game
   draws at the crater's centre (the amplifier changes the ground after the impact stage placed
   the body; the first placement from the deep-time surface was off by up to nine blocks). A new
   crater destroys smaller ones inside it (IMP-060).
8. **Ageing.** Each later step, the crater turns with its plate. Sediment laid inside the rim adds
   to its cover; erosion strips the cover and then wears the crater (IMP-078). Soil creep is
   modelled as a blur that widens with time: the profile is smoothed by a width of √(12 κ t), with
   κ = 0.01 m²/yr times a factor for the rock (IMP-072 to IMP-075). A crater worn through its depth
   and its breccia is removed (IMP-079).
9. **Deposits.** For each crater the stage records melt thickness, a measure of hot-water
   circulation, a measure of shock, and whether a basin remains; the four impact deposit types read
   these (IMP-097 to IMP-102).
10. **Block scale.** The amplifier draws each surviving crater's aged profile, times the share of
    its relief that is left, after the last flooding step, so that the bake's own water decides
    whether a bowl holds a lake (IMP-128 to IMP-135).

## Equations and constants

Symbols: `L` impactor diameter (m), `ρi` and `ρt` impactor and target density (kg/m³), `v` speed
(m/s), `g` surface gravity (m/s²), `θ` angle from the horizontal, `D_tc` transient crater diameter,
`D` final rim-to-rim diameter, `D_c` the simple-to-complex transition diameter, `E` impact energy
(J), `R` water depth over impactor diameter.

"Read" means the stage 2 agent read the value in the source named, on 2026-10-02: CMM05 and
Davison & Collins 2007 and Hughes 2000 in full text, Grieve 1987 through its OCR layer, and Pike
1980 and 1985, Bland & Artemieva 2006 and Moore et al. 1974 as scans through OCR. The ledger gives
the page and the passage for each.

| Ledger id | Quantity | As coded | Units | Source (read) | Status |
|---|---|---|---|---|---|
| IMP-001 | Earth's cratering rate, craters ≥ 20 km | 5.6 × 10⁻¹⁵ | per km² per year | Grieve & Shoemaker 1994, as quoted by Hughes 2000 (his own: 3.46 ± 0.30; range of estimates 2.6 to 5.6) | `abstract-only` |
| IMP-003 | Size slope above 20 km | N(≥ D) ∝ D^−1.8 | – | Grieve 1987 (large diameters) | `verified` |
| IMP-004 | Rate below 20 km, relative to 20 km | 3.47 at 2.4 km, 2.75 at 5 km, 1.88 at 12 km (3.7 at 2 km) | – | Hughes 2000, abstract and Table 2 | `verified` |
| IMP-005 | Constant over time | no change over a run | – | Hughes 2000 reports the lunar rate constant for ~2 Gyr | `abstract-only` |
| IMP-020 | Impactor speed | 20 | km/s | CMM05: 12 to 20; Bland & Artemieva, Davison & Collins: 18 | `verified` (inside the range) |
| IMP-021 | Impactor density | 3,000 | kg/m³ | CMM05: stones 2,000 to 3,000 | `verified` |
| IMP-023 | Target density | 2,700 | kg/m³ | CMM05: 2,500 and 2,750 on land, 2,700 under water and in the melt law | `verified` |
| IMP-024 | Impact angle | 45° | – | CMM05, Bland & Artemieva (Shoemaker 1962) | `verified` |
| IMP-030 | Transient crater | D_tc = 1.161 (ρi/ρt)^⅓ L^0.78 v^0.44 g^−0.22 sin^⅓ θ | m | CMM05 eq. 21 | `verified` |
| IMP-031 | Simple crater | D = 1.25 D_tc | m | CMM05 eq. 22 | `verified` |
| IMP-032 | Complex crater | D = 1.17 D_tc^1.13 / D_c^0.13 | m | CMM05 eq. 27 | `verified` |
| IMP-033 | Which applies | the larger of the two | – | the paper switches at D_tc = 2.56 km, with a step down | `game-choice` |
| IMP-034 | Transition on Earth | D_c = 3,200 | m | CMM05 p. 823; Pike 1980: 3.1 km | `verified` |
| IMP-036 | Transition against gravity | D_c = 3,200 × 9.81 / g | m | Pike 1980: D_t ∝ 1/g, Moon 18.7 km (model 19.4), Mars 6 km (model 8.4), Mercury 16 km (model 8.5) | `verified` |
| IMP-037 | Transient depth | d_tc = D_tc / (2√2) | m | CMM05 eq. 25 | `verified` |
| IMP-038 | Rim height | h = 0.07 D_tc⁴ / D³ | m | CMM05 eq. 48 (derived for simple craters) | `verified` |
| IMP-039, IMP-040 | Breccia lens | V = 0.032 D³; t = 2.8 V (d_tc + h) / (d_tc D²) | m³, m | CMM05 eqs. 23, 24 | `verified` |
| IMP-041 | Depth of a simple crater | d_tc + h − t_br (about D / 4.7) | m | CMM05 eq. 26 | `verified` |
| **IMP-042** | **Depth of a complex crater** | **0.4 D^0.3** (km); was 0.294 D^0.301 | km | CMM05 eq. 28 (Herrick et al. 1997, Venus) | `verified`, **code changed** |
| IMP-045 | Peak rings begin | 40 km × 8.87 / g (36 km on Earth) | m | Alexopoulos & McKinnon 1994, not readable | `from-memory` |
| IMP-046 | Peak ring radius | 0.5 of the rim radius | – | Pike 1985 | `verified` |
| IMP-048 | Central peak width | 0.22 of the rim | – | Pike 1985: 20 to 25 % | `verified` |
| IMP-054, IMP-055 | Ejecta thickness | D_tc⁴ / (112 r³) | m | CMM05 eq. 47 (power −3: McGetchin et al. 1973, via CMM05) | `verified` |
| IMP-057 | Continuous blanket on the Moon | 2.3 radii | – | Moore et al. 1974: 2.348 | `verified` |
| IMP-062 | Melt volume | 8.9 × 10⁻¹² E sin θ | m³ | CMM05 eq. 30 | `verified` |
| IMP-063 | Melt sheet thickness | 4 V / (π D_tc²) | m | CMM05 eq. 31 | `verified` |
| **IMP-065** | **Crater under water** | **D_wet = D_dry (1 − 0.14 R); none for R ≥ 8; none for a dry-complex crater at R > 3.5** | m | Davison & Collins 2007, eq. 3 and p. 1922 | `verified`, **code changed** |
| IMP-066 | Water depth with no crater | 8 impactor diameters (was 6.3) | – | Davison & Collins 2007 | `verified`, **default changed** |
| IMP-072, IMP-073 | Ageing by soil creep | a blur of variance 2 κ t | – | the diffusion equation; Fassett & Thomson 2014 (lunar, abstract) | `abstract-only`, `verified` (arithmetic) |
| IMP-074 | Diffusivity κ | 0.01 m²/yr × rock factor | m²/yr | none could be read | `uncited` |
| IMP-130 | Depth setting | 0.5 | – | Ben, decisions log 2026-09-30 (the volcano precedent) | `game-choice` |
| IMP-013, IMP-015 | Size limits | 2 km to 300 km | km | agent, build log 2026-10-01 | `game-choice` |

Checks against the source's own worked examples (the test `the_papers_worked_examples_come_out`):
CMM05 Table 6 gives final craters of 1.2, 23.7 and 186 km for a 40 m iron body (after the air has
slowed it from 20 to 10 km/s), and stony bodies of 1.75 km and 18 km. The coded laws give 1.18,
23.8 and 185.6 km. The complex-depth law is tested as an identity with eq. 28 (a crater of 23.8 km
is 1,035 m deep). The sea rule is tested against the runs in Davison & Collins's Table 2.

The rows for impact rock (IMP-084 to IMP-096), the four deposit types (IMP-097 to IMP-127) and the
block-scale shape (IMP-128 to IMP-136) are in the ledger file. The deposit sizes, grades and
densities there are placeholders: the data file says "design values: no USGS model exists".

## Earth against the mod

### The production function against the record

| Craters larger than D, per km² per year (× 10⁻¹⁵) | 2.4 km | 5 km | 12 km | 20 km | 22 km |
|---|---|---|---|---|---|
| Hughes 2000 (craters surviving on stable continents, last 125 Myr) | 12.0 | 9.5 | 6.5 | 3.46 | 3.0 |
| Grieve & Shoemaker 1994 with a D^−1.8 slope all the way down (the first coding) | 258 | 98 | 17 | 5.6 | 4.6 |
| The code now (their level at 20 km, Hughes's shape below) | 19.4 | 15.4 | 10.6 | 5.6 | 4.9 |

The first coding gave twenty times Hughes's rate for 2.4 km craters and ten times for 5 km craters.
The new one is 1.6 times Hughes's at every size, the ratio between the two 20 km rates.

### Known impact structures against the production function

The Earth Impact Database (not readable from a script; its list was taken from the Wikipedia table
"List of impact structures on Earth", a secondary source, 2024 revision, parsed 2026-10-02) has 71
confirmed structures of 2 km or more younger than 130 Myr. The production function gives how many
should have formed on all of Earth's land (148 million km²) in 130 Myr; the rest were never found,
or eroded, buried or were not on land.

| Diameter (km) | 2–5 | 5–10 | 10–20 | 20–50 | 50–100 | 100–300 | all |
|---|---|---|---|---|---|---|---|
| Known structures, age ≤ 130 Myr | 13 | 18 | 18 | 16 | 4 | 2 | 71 |
| Formed on land, production function now | 100 | 77 | 111 | 87 | 15 | 5 | 395 |
| known / formed | 0.13 | 0.23 | 0.16 | 0.18 | 0.27 | 0.39 | 0.18 |
| Formed on land, first coding (D^−1.8 down to 2 km) | 5,489 | 931 | 267 | 87 | 15 | 5 | 6,794 |
| known / formed | 0.002 | 0.019 | 0.067 | 0.18 | 0.27 | 0.39 | 0.01 |

The known fraction is about a fifth at every size under the new function, a plausible survival and
discovery fraction that does not depend strongly on size. Under the first coding it fell by a
factor of 90 from 20 km to 2 km, which no process of erosion or discovery explains better than a
wrong production function does (Hughes 2000 and Bland & Artemieva 2006 reach the same conclusion).

### The craters of one run

Both runs: `deeptime run --preset earthlike_quick_16k --start scattered_continents --length-myr 500
--threads 4 --set structure.deformation.density=0` (the structure stage panics on seed 1, see "Open
questions"; the impact stage runs before it), seed 1, 2026-10-02, on master 1eac09f (general flexure
on, which sinks the land round a crater slightly). "Before" is that master as it was; "after" is the
same master with this branch's impact changes. Script: `tools/review/impacts-census.sh`. One run of
one seed each: the sea, the plates and the Poisson draws differ between them, so the two are
comparisons of kind and size, not of single numbers.

| | before | after |
|---|---|---|
| impactors drawn (expected) | 1,302 (1,359) | 75 (79) |
| craters formed | 548 | 45 |
| by size, under 5 / 5-10 / 10-20 / 20-50 / 50-100 / over 100 km | 425 / 80 / 33 / 5 / 4 / 1 | 13 / 11 / 12 / 8 / 1 / 0 |
| of them in the sea floor | 144 | 15 |
| impactors that left no crater in the sea | 754 | 30 |
| eroded through / subducted | 416 / 27 | 33 / 7 |
| in the catalogue at the end | 105 | 5 |
| at the present: fresh / degraded / buried / eroded to roots | 10 / 9 / 69 / 17 | 2 / 0 / 3 / 0 |
| showing as landforms (fresh or degraded) | 19 (18 on continental crust) | 2 (1) |
| oldest that shows, Myr | 118 | 87 |
| largest formed, km | 112 | 91 |
| impact deposit sites (nickel / hydrothermal / diamond / basin uranium) | 0 / 18 / 0 / 16 | 0 / 14 / 0 / 15 |
| wall time | 14.5 min | 14.6 min |

### The default rate of 3 (2026-10-02)

The same run (`tools/review/impacts-census.sh`, quick 16k, 500 Myr from scattered continents, seed 1,
`structure.deformation.density=0`), once at `impacts.flux` 1 and once at the new default 3, on one
tree. One run each; the histories differ (the stage changes what the later stages see), so the
columns compare kind and size.

| | flux 1 (Earth's) | flux 3 (default) |
|---|---|---|
| impactors drawn | 75 | 228 |
| craters formed | 49 | 145 |
| by size, under 5 / 5-10 / 10-20 / 20-50 / 50-100 / over 100 km | 14 / 13 / 13 / 8 / 1 / 0 | 28 / 21 / 47 / 35 / 12 / 2 |
| formed in the sea floor / impactors stopped by the sea | 18 / 26 | 64 / 83 |
| in the catalogue at the end (surviving) | 7 | 25 |
| at the present: fresh / degraded / buried / eroded to roots | 0 / 2 / 4 / 1 | 2 / 6 / 12 / 5 |
| showing as landforms (fresh or degraded; on continents) | 2 (2) | 8 (7) |
| largest formed, km | 91 | 209 |
| wall time | 19.9 min | 16.4 min (a loaded machine; not a cost of the rate) |

The count of craters fell 12 times (the 2 to 20 km craters 17 times, as the new production
function says; the craters over 20 km are unchanged in rate and came out as 9 against 10).
The old numbers were not the "right" ones for the quick planet either: they carried a small-crater
population that Earth's record does not show. With 45 craters formed the planet has few to show.
The share that survives to the end is similar (11 % against 19 %, a few craters decide it).
Deposit sites hardly changed, because the deposit engine's targets come from its own densities
and the sites formed come from the few large craters; the hydrothermal sites that "counted" fell
from 14 to 5.

Earlier figures of the first P2-08 run (before flexure, 567 craters, 19 showing) are in the
build log of 2026-10-01; flexure alone changed the buried count from 33 to 69.

### Other comparisons

| Quantity | Earth | Source of the Earth figure | The mod |
|---|---|---|---|
| Simple craters' apparent depth | 0.14 D^1.02 km (D/7 at 1 km) | Grieve 1987; Pike 1980 (read) | D/4.7 (CMM05's lens model); about 1.5 times deeper |
| Complex craters' apparent depth | 0.27 D^0.16 km (Grieve 1987), 0.23 D^0.24 km (Pike 1980): 0.44 and 0.47 km at 20 km | read | 0.4 D^0.3: 0.98 km at 20 km (CMM05, from Venus: "more reliable than … erosion-affected data for terrestrial complex craters") |
| Largest structure | Vredefort, 150 ± 10 km (Pike 1985); other estimates up to 300 km | read; Wikipedia | 300 km cap |
| Rim-to-ejecta, ejecta at 200 km from a 1.75 km and an 18 km body | 0.09 m and 137 m | CMM05 Table 6 (the program's own result) | 0.08 m and 117 m |
| Ocean's share of craters removed | two thirds under 1 km, a third near 30 km, little over 100 km | Davison & Collins 2007 abstract | not reproduced; the planet's own sea floor decides |
| Gravity: transition to complex craters | Earth 3.1 km, Moon 18.7 km, Mars 6 km, Mercury 16 km | Pike 1980 | 3.2, 19.4, 8.4, 8.5 km |

## Impact deposits need a source (carbonate and ore-deposit task, 2026-10-19)

**What was wrong.** On master b438d47a a quick 16k planet carried a median 153 sites of basin
uranium (62 to 359) and 75 of hydrothermal Cu-Zn-Au (43 to 541; six planets, supercontinent and
scattered continents, seeds 1 to 3), most of both in shale. Basin
uranium had no uranium source: its source term was the crater basin itself, its driver the basin's
filling and its trap the fill's porosity, so a crater in oceanic basalt made uranium as readily as
one in granite. The hydrothermal type had the hydrothermal strength as its only source. Their
densities, 50 and 100 per 100,000 km² of permissive crater at p50, were placeholders.

**Source, fluid, trap** (IMP-142; ledger ORE-039, ORE-040, ORE-042). Each metal type now asks
for all three, as an ore system does:

| Type | Source (the metal) | Fluid or driver | Trap |
|---|---|---|---|
| Basin uranium | a felsic continental target: crust 15 km (0) to 30 km (1) thick (upper continental crust holds about 2.7 ppm U, mid-ocean-ridge basalt about 0.07, from memory) | the basin filling with sediment (0 to 5 m/Myr), and oxidised groundwater through porous fill (reservoir quality 0.3 at 0 to 1 at 0.5) | a closed basin (inside the rim of a structure at least 5 km across) with a reductant to fix the uranium (source-rock organic carbon, 0.3 at 0 to 1 at 1 wt %) |
| Hydrothermal Cu-Zn-Au | the melt sheet, its heat and the base metals it leaches (10 m of melt 0, 200 m 1) | the hydrothermal system (strength 0.1 to 0.6, craters of about 8 to 25 km) | the fractured central uplift and the rim (trap structure 0.3 at 0 to 1 at 0.5) |
| Diamond | shock on a carbon-bearing target (unchanged, IMP-119) | none | none |
| Ni-Cu-PGE | a thick melt sheet that differentiates (unchanged, IMP-105) | none | continental crust (unchanged) |

Grieve and Masaitis (1994, abstract) name the impact uranium of Ternovka and Carswell as
progenetic: ore in the target, exploitable "because of uplift in the center of complex impact
structures". The model's basin uranium is the epigenetic case, uranium leached from a granitic
target and fixed in the basin's fill; it needs the granitic target either way.

**Densities from Earth's inventory** (IMP-143). The Earth Impact Database (through the Wikipedia
list, read 2026-10-19) holds 199 structures with a diameter, 135,890 km² of crater in all (the 105
at least 8 km across hold 134,649 km²). Grieve and Masaitis (1994) find about 25 % of known
structures associated with an economic resource; James et al. (2022) count 60 of 208, of which
hydrocarbons 19 and major minerals 10 (their LPSC 2023 abstract). Ten metal-bearing structures
spread over three metal types, two or three structures each with about three deposits, is about
4 deposits per 100,000 km² of crater: basin uranium and hydrothermal Cu-Zn-Au are now 1.5 / 4 / 10
per 100,000 km² (were 15 / 50 / 150 and 30 / 100 / 300), diamond 1 / 3 / 8 (was 6 / 20 / 60:
Popigai's two fields and Kara, from memory). Ni-Cu-PGE keeps 30 / 100 / 300: its whole
permissive area is the melt sheet of the largest craters, and Sudbury alone holds some ninety
deposits in one structure of 200 to 250 km. The step from ten structures to a density per metal
type is the project's estimate, so the rows are `calibrated`, not `verified`.

**Measured.** Six quick planets (`earthlike_quick_16k`, 1,000 Myr, supercontinent and scattered
continents, seeds 1 to 3; master b438d47a against the branch at 686a01cf; `deeptime
deposit-audit`, `tools/review/carbonate-table.py`), sites per planet, median [range]:

| type | master | branch |
|---|---|---|
| basin uranium | 153 [62, 359] | 7 [1, 13] |
| hydrothermal Cu-Zn-Au | 74.5 [43, 541] | 1 [0, 2] |
| Ni-Cu-PGE | 5.5 [0, 193] | 6 [0, 51] |
| diamond | 0 [0, 0] | 0 [0, 1] |

The branch's basin uranium sits in the basin's fill (shale 25 and sandstone 9 of its 40 sites) over
granitic or felsic targets. On the full 16k planet (`earthlike_full_16k`, scattered continents,
seed 1) basin uranium went from 31 sites to 4 and hydrothermal Cu-Zn-Au from 33 to 3; the
uranium in deposits from 7,800 ore blocks to 583. Ni-Cu-PGE, unchanged, swings with whether a
crater large enough for a thick melt sheet survives on the planet (0 to 193 on master; 0 and 62
on the full 16k planet's two runs).

## Simplifications and limits

- **One kind of impactor.** Every body is rocky, arrives at 20 km/s and strikes at 45 degrees.
  There are no iron bodies and no comets, and no spread of speeds or angles. Bland & Artemieva 2006
  find that almost all craters under 1.5 km on Earth are made by iron bodies; the 2 km minimum
  sits just above that.
- **One kind of target.** The same crust density and the same 3.2 km transition are used for all
  rock, though the sources give 2 km for sedimentary and 4 km for crystalline targets.
- **A constant rate.** Earth's present-era rate is used for the whole history, with no early heavy
  bombardment. Hughes 2000 reports the lunar rate constant for about 2 Gyr. A 4.5 Gyr history
  would need the early bombardment.
- **Earth's rate on a small planet.** The rate per area is Earth's; the planet has 1.5 % of
  Earth's area, so it collects 1.5 % of Earth's impacts. Crater sizes are not scaled down.
- **The rate below 20 km combines two sources.** The level at 20 km is Grieve & Shoemaker's (as
  quoted); the shape below is Hughes's, whose own rates are 0.62 times lower at all sizes. Both
  are rates of surviving craters on stable continents; if the record is incomplete the true
  production is higher.
- **The sea rule is Davison & Collins's for vertical impacts at 15 km/s into granite.** Their paper
  says oblique impacts, which cross more water, would remove more craters; weak seabed sediments
  and the resurge features themselves are not modelled. Water depth is read from a grid with
  4.8 km cells.
- **Rim heights of complex craters are overstated.** CMM05's rim law is for simple craters.
- **Craters are round.** No oblique shapes, no secondary craters, no rays. Round in the
  simulation's Lambert blocks, not on the planet: the amplifier's crater shape, the impact bodies
  and their footprints all work in Lambert blocks, so on the planet a crater is long north–south and
  narrow east–west by 1 / cos² of its latitude (3.4 to 1 at 57°), the same limit as the
  simulation's other features near the poles (seamless-world.md, "Simplifications and limits"), and
  on the Petroff–Guyou world it is a further ellipse in world blocks. The GameTest
  `aCraterIsABowlOverBreccia` measures its floor, rim and the ground around it round in that frame.
- **Small craters far from the equator are smoothed away.** The projection's footprint is round on
  the planet and at least the source grid's north–south spacing across (SCL-090), so along the
  parallel it averages over a cell · tan² φ or more of Lambert blocks; a crater narrower than that
  east–west loses its bowl into its walls. On the 4,096-block test planet of 2026-10-23, crater 572
  (2.6 km across at 57°, averaged over 0.56 of its radius) showed 0.31 of its relief. A crater shows
  whole only where its diameter is at least about 2 · cell · tan² φ (with that planet's 600 m cells,
  1.2 km at 45° and 2.9 km at 57°). Drawing craters in the world's own frame, after the projection, would remove this
  and the stretch above.
- **The soil stays over the breccia.** A body never replaces what the surface rules laid (one block
  of gravel under the sea, up to five of dirt under a pond, plan §7.5), so a lens thinner than its
  soil shows no breccia at the surface: 572's lens of 3.7 blocks lay wholly in its 5 blocks of
  dirt.
- **Ageing is a blur plus a budget.** Soil creep is a smoothing filter with an unsourced
  diffusivity; river erosion and sedimentation enter only as their average inside the rim.
- **No effect on anything else.** An impact does nothing to the climate or to life, and craters are
  not deformed by later folding.
- **Small craters make no ore.** Impact deposits form only where the breccia is in the coarse rock
  record, so not in craters under 9.6 km on the default planet.
- **Impact ore is sized by hand.** No published grade and tonnage model exists for impact
  deposits, so the sizes and grades are placeholders. The numbers are not: since 2026-10-19 the
  metal types need a source, a fluid and a trap, and their densities follow Earth's inventory of
  impact structures (IMP-142, IMP-143; "Impact deposits need a source").

## Game choices

| What | Why | Who and when |
|---|---|---|
| Impacts exist, at a rate that is a setting with Earth's as 1 (IMP-006) | wanted "in full", before phase 3 | Ben, decisions log 2026-09-29; started 2026-10-01 |
| Craters drawn at half depth (IMP-130) | a true bowl is a shaft at 15 times vertical stretch | agent, following Ben's 2026-09-30 answer for volcano flanks; **an open question for Ben** |
| Smallest crater 2 km, largest 300 km (IMP-013, IMP-015) | 2 km is 7 blocks; above 300 km the planet-wide effects are not modelled | agent, build log 2026-10-01; the cap is **an open question for Ben** |
| Default rate is 3 times Earth's (IMP-006, IMP-140) | Earth's rate leaves about 2 craters showing on a default planet; 3 times shows about 8 | Ben, decisions log 2026-10-02 ("Go for a 3x default") |
| Applying Earth's rate per area to the small planet (IMP-007) | the simplest assumption | agent, build log 2026-10-01 |
| The larger of the simple and complex diameter laws (IMP-033) | continuous and increasing; the paper's switch steps down | agent, build log 2026-10-01 |
| Peak heights (IMP-049), flat-floor radius 0.55 (IMP-051), ring width (IMP-047), ejecta drawn to 3 radii (IMP-056) | shape values chosen for looks within observed ranges | agent, build log 2026-10-01 |
| "Fresh" means 75 % of depth left; "shows" means 40 m of relief, two blocks (IMP-080, IMP-081) | thresholds for the catalogue and the map | agent |
| Stand-in blocks: cobblestone and smooth basalt (IMP-092) | no impact rock blocks exist yet | agent; **an open question for Ben** |
| Impact deposits do not replace the pack's scattered diamond, nickel, platinum and uranium ore (IMP-126) | impact deposits are too rare to be the only source | agent; **an open question for Ben** |
| All tonnages and grades of impact deposits (IMP-105 to IMP-125); the step from Earth's ten metal-bearing structures to a density per type (IMP-143) | no model exists | agent, marked as placeholders in the data; the densities calibrated 2026-10-19 |
| The GameTest planet's rate: 700 times Earth's, craters up to 40 km (IMP-139) | so a 4,096-block planet has craters to test; was 40 times until the rate correction made craters 17 times rarer | agent, build log 2026-10-01 and 2026-10-02 |

## Open questions

1. **Settled (Ben, 2026-10-02, "Go for a 3x default"): the default rate is 3 times Earth's.** With
   the corrected production function Earth's rate made about 49 craters in 500 Myr and showed 2; 3
   times gives about 145 and 8 (measured, below). The default is a game choice (IMP-006); 1 is
   Earth's rate and stays one click away in the editor.
2. **The diffusivity of 0.01 m²/yr** (IMP-074) and the fade times that follow (a 2 km bowl in about
   10 Myr, a 20 km one in about a billion years) rest on no source read. Fernandes & Dietrich 1997
   and Roering et al. 1999 are open at the publisher but were refused to a script.
3. **Peak rings from 36 km on Earth** is scaled from Venus (Alexopoulos & McKinnon 1994, not
   readable) with a 1/g law that the data on basins (Wood & Head 1976: onset 145 km on Mars, 175 km
   on the Moon) do not support (IMP-045).
4. **Is the rate right?** The level at 20 km (5.6 × 10⁻¹⁵) is the highest of the published estimates
   (2.6 to 5.6); Hughes's 3.46 is 0.62 times that. A modern estimate (Mazrouei et al. 2019;
   Hergarten & Kenkmann 2015) could not be fetched.
5. **The scaling law is used below its stated range**: the smallest craters come from impactors of
   about 84 m, where CMM05 says "a couple of hundred meters" (IMP-030), and where the atmosphere
   matters (Bland & Artemieva 2006).
6. **Possible double counting of erosion** for craters cut into the coarse terrain (extraction
   notes, flag 17): a suspicion from reading the code, not tested.
7. **The structure stage panics on seed 1** ("a history holds at most 128 events",
   `deeptime-noddy/src/history.rs`), so a default run of the quick 16k planet fails after the
   impacts. Not in this feature; the evidence runs set `structure.deformation.density=0`, as a
   previous task did.

## Sources

Keys refer to [bibliography.md](bibliography.md).

| Key | Used for | How it has been read |
|---|---|---|
| Collins2005 | crater scaling, melt, ejecta (IMP-020 to IMP-063) | full text, every equation used, stage 2 |
| DavisonCollins2007 | craters under water (IMP-065 to IMP-067) | full text, stage 2 |
| Hughes2000 | the rate and the shape of the production function (IMP-001 to IMP-005) | full text, stage 2 |
| Grieve1987 | transition diameters; depth laws; slope; an older rate | full text (scan, OCR layer), stage 2 |
| Pike1980, Pike1985 | transition against gravity; central peaks and rings | scans read by OCR, stage 2 |
| Bland2006 | atmospheric break-up; the departure of the record below 10 to 20 km; speeds | scan read by OCR, stage 2 |
| Moore1974 | continuous ejecta of lunar craters | scan read by OCR (the one equation), stage 2 |
| WoodHead1976, GrieveCintala1992 | onset of peak rings on other bodies; melt scaling | scans read by OCR (one table; an abstract), stage 2 |
| GrieveShoemaker1994 | the rate and the size slope | not read (a book); the rate read as quoted by Hughes 2000 |
| Neukum2001, LeFeuvre2011, AlexopoulosMcKinnon1994, Herrick1997 | constancy of the rate; speeds; peak rings; the complex depth law | not read |
| Culling1960, Soderblom1970, Fernandes1997, Roering1999 | hillslope diffusion | not read |
| FassettThomson2014 | crater ageing by diffusion (on the Moon) | abstract read in stage 1 |
| Naldrett2004, Masaitis1998, GrieveMasaitis1994, Abramov2004, Osinski2013 and others | impact deposits | not read |
| Wikipedia list of impact structures (from the Earth Impact Database) | known craters by size and age | secondary; parsed 2026-10-02 |
| NASA planetary fact sheets | surface gravity of Earth and Venus; Earth's radius | read, 2026-10-02 |

Extraction notes, with 22 flags: [extraction-notes/IMP.md](extraction-notes/IMP.md). Build log:
STATUS.md, "Meteor impacts (P2-08)" (2026-10-01) and "Science documentation stage 2, WP-1" (2026-10-02).
