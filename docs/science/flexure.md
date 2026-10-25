# Lithospheric flexure

Status of this page: written with the feature on 2026-10-02. Three constants were checked against
their sources (ISO-041, ISO-042, ISO-048); the rest are marked in the ledger as read from abstracts,
uncited, or game choices. The measurements are from small ensembles of four runs; two of them
were made before the last change of one constant (see "Earth against the mod"). Updated 2026-10-18
for the polar holes (ISO-056 rewritten, ISO-074 to ISO-076 added; six runs), and 2026-10-22 for
the settling's feedback (ISO-061 rewritten, ISO-091 to ISO-093 added; twelve runs a side), and
2026-10-23 for the trench rows (ISO-096 and ISO-097 added, ISO-088 merged into ISO-096) and the
release's pace (ISO-099 added, ISO-056 amended), and the same day for the flexure faults: the
plate's yield (ISO-120) on by default, the held offsets (ISO-125) and water by basin (ISO-126)
added and on by default, the solve on the sphere's metric (ISO-127) added and off (it diverges);
and 2026-10-24 for the foreland basins (ISO-131 and ISO-133 added, ISO-068 amended, with PLT-546
in the plate model).

Ledger ids on this page: `ISO-040` to `ISO-077`, `ISO-088` (merged), `ISO-091` to `ISO-093`,
`ISO-096`, `ISO-097`, `ISO-099`, `ISO-120`, `ISO-125` to `ISO-127`, `ISO-131`, `ISO-133` (see
[ledger/ISO.md](ledger/ISO.md)).

## What you see in the world

The outer shell of the planet is stiff. A heavy thing standing on it (a mountain belt, a volcanic
island, an ice sheet, a thick pile of sediment) pushes down not only the ground under itself but
the ground around it, and the shell holds part of the weight up.

In the world this shows as low ground beside high ground. Along coasts with volcanic arcs and
deep trenches there are paired bands a few cells wide (a cell is 4.8 km, 16 blocks) where the
ground stands some hundreds of metres above or below where it would float on its own. A volcanic
island on old ocean floor stands a few hundred metres higher than it otherwise would and has a
shallow ring of deeper water round it. In the one planet looked at closely, 14 % of the surface
stood more than 100 m (5 blocks) away from its floating level and 1.6 % more than 500 m.

Inside wide mountain belts nothing is different: their crust is young and weak and floats locally,
as before. Since 2026-10-24 a range in an active collision stands on the end of the stiff plate
that goes under it, which bends down into a basin in front of it (the Ganges plain): see "Foreland
basins".

## The model in plain words

It runs in the surface stage, at the end of each 2-million-year step, after erosion, deposition
and the spreading of over-thick crust.

1. **The loads.** For every cell the stage keeps, moving with the crust, how much weight has been
   added or removed since the start of the run. Each step it adds: the crust the plate model
   added or took away (the change of each column's own crust, not its whole rise: a height the
   plate model moves alone is no load, see "The settling's feedback"), crust moved by spreading, rock eroded or deposited, rock the volcano
   stage laid on top, the change of the sea's depth over the cell, and the land ice the climate
   stage reports. On ocean floor the sea over the plate model's own moves of the ground is left
   out (its cooling under GDH1, a trench's pull, a plate covering a gap): the plate model's depths
   are those of floor under water, so that water's weight is in them already (ISO-096; ISO-088
   for the cooling from 2026-10-21, before which old floor sank about 1.45 times GDH1; the rest
   from 2026-10-23, "The trench rows").
2. **The stiffness.** The effective elastic thickness `Te` says how thick the stiff part of the
   shell is. Ocean floor stiffens with age. A continent's stiffness is that of its strong upper
   crust and its strong mantle, on its lithosphere's geotherm (since 2026-10-25, "The foreland's
   stiffness"): weak where the lithosphere is young and hot, 70 to 84 km on an old craton.
   Volcanic arcs and rifts are weak. A
   plate that the law weakens (an orogeny, an arc, a rift reaching it) has to warm first and
   weakens over tens of millions of years ("The release's pace"). The
   boundary between two plates is a break: one plate does not hold the other up.
3. **The solve.** The shell is treated as a thin elastic plate floating on a fluid. The plate
   carried from the last step is in balance with the load it carries; the stage solves only for
   how far it sinks or rises under this step's change of load. One place is not carried with
   its crust: beside a collision's belt the plate thrust under it slides through a deflection that
   stays with the belt, and there the state stays with the belt ("Foreland basins"). The plate
   model's uplift counts
   only as far as the plate model resolves it: its samples are about 71 km apart (in the plate
   model's Earth-sized kilometres), and what changes from cell to cell below that is left to float
   on its own, as the plate model made it. Crust moved by the spreading floats on its own
   entirely: the spreading's law already gives the floating answer, and the plate does not hold
   any of it up (since 2026-10-18, see "The surface guard").
4. **The result.** The ground is moved by the difference between that sinking and the sinking
   already built into it. The plate model raises ground as if every load floated on its own, so
   that part is already there; erosion, deposition, cones, water and ice have nothing built in.

When the stiffness is set to nothing, step 4 gives exactly the local floating the model had
before. When crust cools and stiffens, the plate keeps the shape it has; only loads added later
meet the stiffer plate. When it weakens (reheated, or becoming a plate's broken edge) it stops
holding up the same share of what it held: a plate that loses nine tenths of its stiffness lets
go of nine tenths of the load it kept from floating on its own, and the ground moves that far
towards floating, never past it (see "The polar holes" below).

The plate model then reads the surface's new heights, as it does for erosion.

## Equations and constants

The plate equation, as coded (ISO-040):

    L(D' L w) + C w = q

- `w`: sinking, m, down positive. `q`: load, kg/m². `L`: the 5-point Laplacian on the L1 raster
  (east and west edges joined, the first and last row closed).
- `D' = E Te³ / (12 (1 − ν²)) / g`: flexural rigidity over gravity.
- `C`: the density that pushes back, ρ_m on land and ρ_m − ρ_w under the sea.
- The width of every flexural feature follows `α = (4D' / ρ_m)^¼`: 18 km for `Te` 5 km, 62 km for
  27 km, 140 km for 80 km, at Earth's gravity. Lower gravity gives wider features (`α ∝ g^−¼`).

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| ISO-041 | Young's modulus `E` | 70e9 | Pa | Turcotte & Schubert, ch. 3 | `verified` |
| ISO-042 | Poisson's ratio `ν` | 0.25 | – | Turcotte & Schubert, ch. 3 | `verified` |
| ISO-043 | Mantle density ρ_m | 3,300 | kg/m³ | none | `uncited` |
| ISO-044 | Sea water density ρ_w | 1,030 | kg/m³ | none | `uncited` |
| ISO-045 | Ice density ρ_i | 917 | kg/m³ | none | `uncited` |
| ISO-046 | Crust thickened behind an uplift `u` | `u / (1 − k)`, `k` 0.858 on land, 0.793 under water (crust 2,830 kg/m³ since 2026-10-06) | – | Airy isostasy, none cited | `uncited` |
| ISO-047 | Density of eroded and deposited rock | 2,640 on land, 2,301 under water | kg/m³ | none (keeps ISO-001, ISO-003) | `uncited` |
| ISO-048 | Oceanic `Te` | 2.7 × √(age in Myr) | km | Calmant et al. 1990 | `verified` |
| ISO-049 | Oceanic `Te`, at most | 40 | km | Watts & ten Brink 1989 | `abstract-only` |
| ISO-050 | Continental `Te` long after an orogeny | 80 | km | Burov & Diament 1995 | `abstract-only` |
| ISO-051 | Continental recovery | `80 × (1 − e^(−t/300 Myr))`; since 2026-10-25 only where a continental cell has no thermal age (ISO-137 takes the rest) | km | form after Lu et al. 2024; the numbers are the agent's | `uncited` |
| ISO-137 | Continental `Te` from the geotherm | `(h₁³ + (h₂ − h_c)³)^⅓`, h₁ the 300 °C isotherm or the Moho, h₂ the 750 °C isotherm, on by default since 2026-10-25 | km | Burov & Diament 1995, eq. 14; Watts & Burov 2003, eq. 3 | `verified` |
| ISO-138 | The geotherm it reads | steady, the crust's heat from sources over h_r 10 km, the mantle's B / L, 1,350 °C at the lithosphere's base | °C, km | Burov & Diament 1995, appendix | `verified` |
| ISO-052 | Lowest `Te` | 5 | km | Lu et al. 2024 | `abstract-only` |
| ISO-053 | Arcs and rifts | at the lowest `Te` | km | none | `uncited` |
| ISO-055 | Plate boundaries | at the lowest `Te`; the boundary cells blurred, the share of stiffness kept `1 − 2.5 × blur` (0, then 0.17 and 0.72 on a straight boundary) | km | none | `uncited` |
| ISO-056 | Stiffening and weakening | gained rigidity joins stress-free; where `D` falls to `r·D` the share `1 − r` of the load held from local isostasy, `ρ_m (Q + ice − w − unsolved)`, is a force | kg/m² | principle: Watts 1978; the Airy limit: Eshagh & Tenzer 2021, Buoninfante et al. 2023 | `abstract-only` (the share rule is the agent's) |
| ISO-075 | Carrying the rigidity | the raster's interpolation blends `ln D`, not `D` | – | none | `game-choice` (numerical) |
| ISO-068 | Increments | each step solves for the step's loads only; the carried plate is in balance; a solve's unfinished part is carried | – | none | `game-choice` (numerical) |
| ISO-069 | The plate model's resolution | its root bends the plate only low-passed at its sample spacing (two binomial passes on the quick presets) | – | none | `game-choice` (numerical) |
| ISO-070 | The spreading's root | bends nothing: the spreading's change stays locally compensated | – | none (its physics from memory of Clark & Royden 2000) | `game-choice` (numerical) |
| ISO-071 | Earth's relief | Everest 8,848.86 m above the sea, the Challenger Deep 10,925 m below | m | Dangol et al. 2020; Stewart & Jamieson 2019 | `verified` |
| ISO-072 | The surface guard | a cell farther from the sea than 12 × 19,774 m × max(1, g_E/g) stops the run | m | none (a margin) | `game-choice` |
| ISO-074 | The step records' polar band | rows at 60° of latitude or more | ° | none (a reporting band) | `game-choice` |
| ISO-076 | Earth's fastest vertical rates | exhumation 9 to 13 mm/yr, rebound 11, subsidence 9 to 12; sustained over millions of years, below 1 mm/yr | mm/yr | Guevara et al. 2022, Govin et al. 2020, Lidberg et al. 2010, Brothers et al. 2009, Garzione et al. 2008, Xie & Heller 2009 and others (the ledger) | `verified` |
| ISO-077 | Two more fast rates | denudation 8 to more than 30 mm/yr (Taiwan), rebound about 10 (Hudson Bay) | mm/yr | Deng et al. 2021, Sella et al. 2007 | `abstract-only` |
| ISO-061 to ISO-064 | What counts as a load | see the ledger | | none | `uncited`, one `game-choice` |
| ISO-091 | Only crust is load | the plate model's crust uplift `u_c`, either sign: load `k_a u_c / (1 − k_a)`; the rest of the uplift and the sea it moves, none | m | none (the plate equation's load is mass, ISO-040) | `uncited` |
| ISO-092 | The crust uplift | the change of each sample's pools' Airy height over the plate step, with the resampling's accretion and the rate limits' cut | mm/yr | none (the plate model's Airy height, PLT-278 to PLT-282) | `uncited` |
| ISO-093 | Earth's departure from local isostasy | at most about 2 to 4 km of rock (free-air −380 to +300 mGal at 0.112 mGal/m) | km | Watts et al. 2025, ten Brink et al. 2004, Stern, Cattin et al. 2001, Hunter & Watts 2016, Watts & Burov 2003 | `verified` (measurement only) |
| ISO-096 | The sea over the plate model's own moves | no load: the move as the carry made it (`h_start` less the source's elevation before the step); on ocean floor that builds nothing all of it, on building columns all but the crust's change (ISO-091). Took in ISO-088 (GDH1's cooling) | m | Turcotte & Schubert §3.13; Levitt & Sandwell 1995; Zhang et al. 2014 | `cited-unverified` |
| ISO-097 | The carry at a trench | the plate model's elevation plus the carried offset, no uplift added | m | none (numerics; Earth's trench depths for the check: Zhang et al. 2014, 2018) | `game-choice` |
| ISO-120 | The plate's yield | where the load a cell's plate holds from local isostasy, ρ_m (Q + ice − w), passes Δg_max / 2πG, the excess sinks under the cell alone; Δg_max 380 mGal, on by default | mGal | ten Brink et al. 2004, Watts et al. 2025, Watts & Burov 2003, Hunter & Watts 2016; G from CODATA 2022 | `game-choice` (the form is the agent's) |
| ISO-125 | The held offsets | cells whose offset the surface holds (gaps between plates, trench rows) hand the plate model's feedback no offset; they count as 0 at their full weight; on by default | – | none (numerics) | `game-choice` |
| ISO-126 | Water by basin | the open sea at the sea level, a lake at its water balance's level, other ground dry, for the water load and `C`; on by default | – | none (the balance: HYD-050, HYD-057, HYD-100, HYD-101) | `game-choice` |
| ISO-127 | The solve on the sphere's metric | the Laplacian's east-west coupling 1 / cos²φ at a row's middle, north-south cos²φ at a row boundary; off (the step diverges at the poles) | – | Snyder 1987, eqs. 10-1 to 10-2b | `verified` |
| ISO-133 | A belt's foreland in the belt's frame | on the carrying plate's crust within `2πα` of a belt it carries, the state is the place's a step ago in the belt's frame; the crust keeps its load and local roots, its load over the place's waits in the unsolved residual | m | the thin plate's equilibrium (ISO-040 to ISO-042): the deflection is set by the load and the plate, not by the plate's past (Beaumont 1981; Flemings & Jordan 1989) | `game-choice` (numerical) |
| ISO-131 | A belt on the incoming plate | where a collision's footprint carries the incoming plate (PLT-546) the break (ISO-055) takes the carrier as the cell's plate; `Te` = `(Te_own³ + Te_carrier³)^⅓` on the overriding plate's cells, the larger on the wedge, the carrier's from ISO-051 at its lithosphere's thermal age | km | Lyon-Caen & Molnar 1983 (abstract), Burov & Diament 1995 p. 3915, Beaumont 1981, DeCelles & Giles 1996, Watts & Burov 2003, Angevine et al. 1990 eq. 5.8 | `abstract-only` (the composite read in full; the setting from an abstract) |

Numerical choices (smoothing of `Te`, the solve's cell size, tolerance, the preconditioner) are
ISO-054 and ISO-057 to ISO-060, all `game-choice`.

## Earth against the mod

| Quantity | Earth | Source of the Earth figure | The mod | Run |
|---|---|---|---|---|
| Sinking under a line load, width to the first zero, bulge height | `V α³ / 8D`, `3πα/4`, 4.3 % of the depth | Turcotte & Schubert (lecture notes on ch. 3) | within 1 %, 2 % and 0.3 % of the depth | unit test `a_line_load_gives_the_analytic_profile`, 5 km cells |
| Sinking under a point load | `P l² / 8D` | Brotchie & Silvester 1969, as quoted by Wickert 2016 | within 3 % | unit test `a_point_load_sinks_by_the_analytic_depth` |
| Volcanic island on old ocean floor | Hawaii: about 5 km of sinking under Oahu (Bianco et al. 2005, quoting Watts & ten Brink 1989; secondary) | secondary | an island held up by 600 m with a ring up to 480 m low round it | `earthlike_quick_16k`, scattered start, 500 Myr, seed 1, 2026-10-02, oceanic law 3.0 km/√Myr |
| Foredeep in front of a collision belt | 100–300 km wide, 2–8 km of fill | DeCelles & Giles 1996, full text | bands of about 100 m beside the largest belt: none of Earth's size | the same run |
| Median sea depth | 4,062 m | GMT earth_relief (build log, P1-17) | 3,408 m without flexure, 4,053 m with (4 runs each) | `earthlike_quick_16k`, 500 Myr, seeds 1–2, both starts, 2026-10-02, `tools/review/p1-17-hyps.sh`, oceanic law 2.7, after the artifact fixes |

The island and foredeep rows are from a run of the first build (oceanic law 3.0 km/√Myr, before
the artifact fixes); the sea-depth row is from the final code (law 2.7, after the fixes).

## Simplifications and limits

- The plate is elastic up to its strength, which is a bound on the load a cell's plate holds away
  from local isostasy (since 2026-10-23, on by default: "The plate's yield", ISO-120), not on its
  bending stress. Real lithosphere also depends on whether crust and mantle are coupled and on how
  sharply the plate is bent (Burov & Diament 1995): the continents' `Te` takes the two as always
  decoupled (ISO-137), and the bending's weakening is not modelled.
- The solve works in the raster's block metric: every cell is a square of the equator's width
  (`norm_dx_m`). On the equal-area raster a cell at latitude φ is cos φ as wide and 1 / cos φ as
  tall on the sphere, so near the poles the plate bends as if it were about 1 / cos⁴ φ too stiff
  north to south and cos⁴ φ too soft east to west (about 300 times each way at 76°). A solve on
  the sphere's metric exists since 2026-10-23 (ISO-127) but is off: its step diverges at the poles
  ("The solve on the sphere's metric").
- `Te` depends only on the crust's type, its thermal age and, on continents, the crust's thickness
  (ISO-137).
- The plate equation's extra terms for varying rigidity are left out (ISO-040).
- A step is 2 million years; the ground answers in thousands of years on Earth. The solve gives
  the equilibrium. Ice-age cycles inside a step are not followed: the load is the ice standing at
  the step's end.
- Every plate boundary is a break, also where two plates slide past each other.
- On the quick presets a flexural feature is 2 to 8 cells wide.
- Trenches and the ocean floor's deepening with age are not flexure in the model; they are
  prescribed by the plate model, the water that fills them is no load either (ISO-096), and at a
  trench the surface follows the plate model's height and keeps only its own offset (ISO-097).
- Along active margins the ground still zigzags from cell to cell somewhat more than without
  flexure: narrow loads that are real at this resolution (marine fill of kilometres in trench
  cells) and the load a plate lets go where a plate boundary moves through cells (since
  2026-10-18 never more than the load it held, "The polar holes"). The first
  build zigzagged far more, for three numerical reasons now fixed (build log 2026-10-02, the
  flexure follow-up; design note section 13): the carried state acting as a force, a stair-stepped
  plate boundary, and the plate model's sample lattice treated as a load.

## The surface guard

**Why it exists.** Until 2026-10-18 the crust moved by the plateau spreading (`spread.rs`) was a
load like the plate model's uplift: its root bent the plate. The spreading is solved first, as an
implicit diffusion of the surface, and the plate after it. On a stiff plate a change at the scale
of a cell cannot sink, so the flexure turned the spreading's floating change into the whole
thickening behind it, up to 1 / (1 − 0.858) = 7 times larger. Per wave of the surface the two steps
together multiply its height by 1 − F·λ/(1 + λ), with F up to 7 and λ = D·Δt·k² the strength of the
spreading's step for that wave: a wave grows once λ passes about 0.4. The spreading works on the
map's metres, so on a large planet its cells are wide (39 km on the Earth-sized planet at
1,222.5 m per block, 3.2 km at 1:100) and nothing holds cells near the strength height. Cells far
above it spread strongly enough (λ about 2.5 for a checkerboard) for the split to grow: peaks
became pits and pits peaks every step, by 40 to 70 km, and the Earth-sized planet reached
3.3 × 10²⁹ m in 500 Myr. A milder form showed at 1:300: over a traced 500 Myr run the median
step's highest cell after the flexure was 11.6 km, and 6.0 km after the fix. 1:100 did not grow
(6.7 and 5.8 km), because its spreading holds plateaus at the strength height, where the
spreading barely moves.

**The fix** (ISO-070). The spreading's law is the floating answer of the lower crust's flow, so
its change now stays locally compensated and the plate answers none of it. The coupling is then
stable at every cell size. The plate model's uplift still bends the plate as before (ISO-069).

**The guard** (ISO-071, ISO-072). After the flexure, every step, the surface stage stops the run
with an error when a cell is not finite or stands farther from the sea than twelve times Earth's
relief (Everest's summit 8,848.86 m to the Challenger Deep 10,925 m: 237 km), times g_E/g below
Earth's gravity. Nothing is clamped. The error names the time, the cell and the bound. With the
old coupling put back, the Earth-sized planet's 500 Myr bake stops after 92 s, at −388 Myr with a
cell 263.8 km above the sea, where it had run 3.56 h to 3.3 × 10²⁹ m.

**What is left, and why the margin is wide.** The plate model now and then lifts a patch by tens
of km in one step (up to 16.5 mm/yr, 33 km a step, against its 30 km ceiling, ISO-025); a stiff
plate holds part of the root behind that uplift up (ISO-069), and erosion takes it down within a
step or two. These transients are there with and without the fix and at every scale: up to
112 km after the flexure in four traced 500 Myr runs at 1:300 and 1:100. A bound of five Earth
reliefs (98.9 km), tried first, stopped a shipped bake (`earthlike_quick_16k`, scattered, seed 2,
at −272 Myr), so the bound sits above them and above the about 210 km a sound step could reach
(the 30 km ceiling with the whole thickening held up, 7 times). The transients themselves were the
plate model's; since 2026-10-18 its rate limits bound them (an open question below).

## The polar holes

**What was seen.** With the plate step's rate limits in (2026-10-18), the cells that came closest
to the guard were on L1's polar rows: a cell there rose or fell by tens of km within a few steps
while the plates under it rose by 2 km or less. The worst, over six 500 Myr runs of
`earthlike_quick_16k`, was a hole two cells wide on the supercontinent planet of seed 2 that fell
to 59.6 km below the sea in 16 Myr, more than five times the Challenger Deep's depth.

**The cause: a weakened plate let go of its bending moment, not of its load** (ISO-056 until
2026-10-18). The plate carried its bending moment `m = D ∇²w`. Where a cell's rigidity fell to `r`
times what it was, the plate kept `r·m`, and the rest, `L((1 − r)·m)`, was a force. For a plate in
balance `L(m)` is exactly the load the plate holds away from local isostasy, so where `r` is the
same over a region the force is that share of the load. Along a plate boundary `r` changes from
cell to cell: a cell the boundary reaches drops to the lowest `Te` (5 km, ISO-052, ISO-055), so a
craton cell keeps (5/80)³ of its rigidity, a four-thousandth, while its neighbours keep theirs.
The curvature of a released moment that steps from cell to cell is a force several times the
load the plate held, up in one cell and down in the next. Traced on the worst run at −290 Myr: a
craton cell at 70° of latitude that a boundary reached released 17.2 km of ground in one step. In a
controlled test (a boundary stepping between rows across a bent plate) the old rule moved the
ground by up to 1.46 times the load held and pushed it 4.2 km past local isostasy.

**Why near the poles.** The same rule ran everywhere; the polar rows have more of what sets it
off. The raster is equal-area, so poleward of 60° a cell is more than twice as tall on the planet
as on the equator (78 km and more on the quick presets) and less than half as wide. The plate
model's fields come from samples about 71 km apart, so there its boundaries step from row to row
at the cell scale, and a plate's rotation carries crust about ten cells a step east to west:
boundaries sweep across many cells each step, and each cell they reach weakens. The carried
rigidity was also blended linearly on the raster. Beside a stiff cell, a weak cell carried a
fraction of a cell read as nearly as stiff, the next step found it weak again, and the rule
released load that nothing had changed, most where crust moves farthest.

**The fix.** A weakened plate lets go of its share of the load it held (ISO-056): where `D` falls
to `r·D`, the force is `(1 − r) ρ_m (Q + ice − w − unsolved)`, the cell's own departure from local
isostasy. Where `r` is uniform this is the old force. Where it is not, the force moves a cell
towards local isostasy, where a plate with no strength puts it, and no further, whatever the
cell's size or latitude. Local isostasy is the limit a viscoelastic plate relaxes to: its flexure
lies between the elastic and the Airy-Heiskanen answers, and reaches the latter as time goes to
infinity (Eshagh & Tenzer 2021); as `D` goes to 0 the compensation is Airy's (Buoninfante et al.
2023). That happens in about a million years (Beaumont 1978, abstract), less than a step. The
bending moment is no longer carried. The rigidity is now carried as its logarithm (ISO-075), so a
weak cell beside a stiff one no longer seems to weaken when it moves. Unit test
`a_plate_that_weakens_settles_towards_local_isostasy` (a 64 × 48 plate bent by a disc load, a
boundary stepping between rows across it for ten steps): no step moves the ground by more than
1.05 times the most the plate held, and nothing goes more than a quarter of that past local
isostasy. The old rule fails it on the third step (11.2 km against a bound of 8.0 km, 4.2 km past
against 1.9 km).

**Measured.** Six runs of `earthlike_quick_16k`, 500 Myr, scattered continents and
supercontinent, seeds 1 to 3. Mean [min, max] over the runs. Before: master 9091c163 with the
step records. After: the fix, merged with master 19cb11e0. Command:
`tools/review/force-balance-ensemble.sh <out> <binary> 500 1,2,3 scattered_continents,supercontinent ck <prefix>`,
run through `tools/remote/detach.sh` on the test machine.

| | Before | After |
|---|---|---|
| Guard's worst (m from the sea) | 30,814 [19,599, 59,631] | 15,471 [13,187, 18,101] |
| Steps beyond 1, 2, 3 and 6 Earth reliefs (the six runs together) | 25, 8, 1, 0 | 0, 0, 0, 0 |
| Largest rise in a step, polar rows (m) | 16,148 [6,359, 39,147] | 5,016 [4,828, 5,338] |
| Largest fall in a step, polar rows (m) | 12,543 [6,367, 24,165] | 5,535 [4,925, 6,992] |
| Largest rise in a step, the rest (m) | 15,012 [10,974, 25,545] | 6,593 [4,899, 9,747] |
| Largest fall in a step, the rest (m) | 15,652 [13,198, 18,228] | 5,628 [4,898, 6,350] |
| L1's highest cell at the end (m) | 7,288 [5,860, 9,958] | 7,031 [6,555, 7,648] |
| Land share at the end | 0.199 [0.160, 0.222] | 0.207 [0.155, 0.237] |

The step records (ISO-074) are the change of a cell's elevation over the surface stage of one
step, with the plate model's uplift taken out. After the fix the polar rows move as the rest do.
Away from the poles (latitude below 60°) the planets come out different cell by cell, by 2.6 km
on average, and the plate model's own elevation differs by as much: a changed surface feeds back
into the plates, and the runs diverge. The surface is not different in kind. Its land share was
0.201 [0.175, 0.229] before and 0.211 [0.158, 0.256] after. Land's median, 90th and 99th
percentiles were 726, 2,965 and 5,053 m before and 715, 2,959 and 5,065 m after. The sea's median
depth was 3,142 m before and 3,147 m after.

**Against Earth's rates** (ISO-076, ISO-077). A step is 2 Myr, so 1 mm/yr is 2 km a step. The fastest
vertical rates measured on Earth run from a few mm/yr to a few tens:

- exhumation: 9 to 13 mm/yr at Nanga Parbat over the last million years, "the fastest known
  intracontinental exhumation rates" (Guevara et al. 2022), and 8.6 km/Myr at Namche Barwa (Govin
  et al. 2020);
- denudation: 8 to more than 30 mm/yr in Taiwan (Deng et al. 2021, abstract only, ISO-077);
- glacial rebound: up to 11 mm/yr (Lidberg et al. 2010);
- subsidence: 9 to 12 mm/yr at the Salton Sea over 1,200 years (Brothers et al. 2009).

Over millions of years the surface itself moves more slowly. The Altiplano rose 1.5 to 3.5 km in
about 4 Myr (Garzione et al. 2008). A basin's tectonic subsidence reaches about 3 km, 4 km in
exceptional cases, mostly within 10 Myr (Xie & Heller 2009).

Before the fix, single cells rose by 39 km in a step (20 mm/yr held for 2 Myr, beyond any
measured rate held that long) and sank to 60 km below the sea. After it, no cell changed by more
than 9.7 km in a step (4.9 mm/yr), or 7.0 km on the polar rows (3.5 mm/yr). That is within the
range of the fastest rates measured on Earth. Nothing stood farther from the sea than 18.1 km,
less than Earth's relief (19.8 km).

**What is left.** Per-step changes of 5 to 10 km at single cells are within Earth's fastest rates,
but above what Earth's surface keeps up over 2 Myr across a region (under 1 mm/yr). Traces of the
first 110 Myr of two runs, taken while the fix was being tried (the trace is not kept in the
code), show which processes make them:

- the fluvial step, stream power with its deposition: up to 9 to 10 km either way;
- marine fill: up to 9 to 12 km into a cell;
- the glacial step: up to 11 km;
- the flexure: up to 8.5 km.

They partly cancel within the step. Whether each is right at a single 39 km cell over 2 Myr was
not checked here.

## The settling's feedback

**What was seen.** Under the collapse arms of the high-ground lane (the sea column and the sheet's
viscosity: `--set tectonics.collapse_sea_column=true --set tectonics.collapse_sea_column_max_km=41.1
--set tectonics.collapse_sheet_viscosity_pa_s=1e22 --set tectonics.collapse_reach_km=5000`), the
surface guard (ISO-072) stopped one run in twelve of `earthlike_quick_16k` over 1000 Myr
(scattered continents, seed 5, at plate step 449), and in four more the guard's worst passed
35 km. Plate columns stood up to 30 km over their Airy height, at the plate model's ceiling
(`max_elevation_m`), with the surface's flexure under them at tens of km. On master's base the
same measure was 3.6 to 10.3 km over the Airy height and the guard's worst 14.5 to 32.3 km.

**The trace.** The stopped run, traced step by step (a scratch patch that printed every clamp,
plate column and flexure cell, not kept in the code). The flexure `F` (`surface/flexure_m`, how far
the plate holds the ground from local isostasy) under continental samples passed 5 km at plate
step 19 and 10 km at step 49; the plate columns' height over their Airy height passed 5 km at
step 39; the 30 km ceiling first clamped at step 305. One column, a rift sample, over steps 290 to
299: its crust 36 to 45 km thick and its Airy height 0.7 to 1.9 km, the plate rules left it near
10 km each step, the settling (ISO-078) moved it to its Airy height plus `F`, the rise guard
(PLT-288) cut that to 7 km in the step, and the surface took the 7 km as crust thickened:
7 / (1 − 0.858) = 49 km of crust, its root held up by the plate. `F` grew from 10.5 to 32 km
over the ten steps. From step 305 polar craton cells (`Te` 80 km) did the same with roots of up
to 42 km a step, held over 148 to 788 km, and `F` reached 95 km by step 350.

**The cause.** The flexure booked every rise of a building column (continental crust, arcs,
hotspots) as crust thickened, 1 / (1 − k) times the rise, whatever raised it (ISO-061 until
2026-10-22), and never booked a fall. The settling moves a height alone, with no crust: to the
column's Airy height plus `F`. The flexure took the move as new crust, a stiff plate held most of
its root up, `F` grew, and the next step's settling target with it: a loop whose gain passes one
on the stiffest plates, ended only by the ceiling's clamp. A column the collapse thinned also kept
the load of the crust it had lost (only rises were booked), a ratchet in the same direction.

**What Earth does.** A plate holds the ground at most a few km from local isostasy. The largest
gravity anomalies over flexurally held loads are about +300 mGal over Hawaii (Watts et al. 2025)
and −380 mGal at the Puerto Rico Trench, the most negative on Earth (ten Brink et al. 2004);
trenches typically sit about 200 mGal under the isostatic expectation and outer rises about 50
over it (Stern, "Ocean Trenches"). At 2πGρ = 0.112 mGal per m of rock (ρ 2,670 kg/m³) these are
1.8 to 3.4 km of rock, and a free-air anomaly is an upper bound on the departure (the moat already
compensates part of the load). Under the Himalaya the foreland is overcompensated and the high
range undercompensated (Cattin et al. 2001). What limits it is the plate's strength: an elastic
plate bent as hard as the trench walls would carry up to 900 MPa (Hunter & Watts 2016), real
plates yield, and `Te` halves at a curvature of 10⁻⁶ m⁻¹ (Watts & Burov 2003). ISO-093.

**The fix** (ISO-091, ISO-092). Only crust is load. The plate model records for each sample the
change of its own crust's Airy height over the step (`plates/crust_uplift_mm_yr`: the pools'
`iso_height` after the step's rules against before, plus the resampling's accretion and the rate
limits' cut, ISO-092), and the flexure books that, either sign, as crust thickened or thinned. The
rest of the uplift (the settling, the height rules, the rise guard's hold) moves the ground with
no load, and the sea it lets in or pushes out is no load either. A thinned column now gives up the
load of the crust it lost. The plate model keeps its heights; only what the surface's plate is told
it carries changes. Unit tests `a_height_moved_alone_is_no_load` (on land and under the sea, a 2 km
rise of a belt 312 km wide on a craton moves the ground 2 km and changes no load) and
`a_belt_thinned_on_a_stiff_plate_stands_below_its_airy_height` (the mirror of
`a_belt_on_a_stiff_plate_gets_a_foreland_basin`).

**Measured** (the shared pool, `earthlike_quick_16k`, 1000 Myr, both starts, seeds 1 to 6, 2
threads, `+DEEPTIME_CRUST_BUDGET=1`; before: master 883a8ef6, source key 54658b9718e7, arms `base`
and `fg-sheet-master`; after: this fix at 1496d688, key adf36da1f017, arms `fg-base` and
`fg-sheet`; the sheet arm is the `--set` keys above; `pool.sh table` and the per-run
`measures/guard.json`). Per run, the guard's worst (km from the sea) and the plate columns' largest
height over their Airy height (km):

| run | base, before | base, after | sheet, before | sheet, after |
|---|---|---|---|---|
| scattered 1 | 15.5, 6.7 | 16.9, 2.8 | 40.6, 29.8 (at the 30 km cap) | 17.1, 6.1 |
| scattered 2 | 16.9, 3.6 | 19.7, 4.5 | 46.5, 5.3 | 15.7, 5.0 |
| scattered 3 | 31.3, 6.1 | 15.9, 3.7 | 16.3, 6.3 | 20.2, 3.9 |
| scattered 4 | 18.5, 4.7 | 15.9, 3.4 | 19.3, 5.2 | 16.2, 5.0 |
| scattered 5 | 14.5, 4.4 | 15.9, 4.7 | stopped by the guard at step 449 | 14.8, 3.7 |
| scattered 6 | 32.3, 10.3 | 15.9, 4.3 | 15.9, 8.8 | 21.9, 2.8 |
| super 1 | 18.0, 4.0 | 15.8, 3.8 | 22.9, 5.0 | 17.3, 3.4 |
| super 2 | 19.8, 4.6 | 15.9, 3.5 | 25.3, 5.0 | 15.3, 3.1 |
| super 3 | 19.5, 3.9 | 15.3, 3.2 | 26.8, 4.5 | 16.1, 3.3 |
| super 4 | 16.5, 5.1 | 18.1, 5.2 | 37.4, 28.0 (28.9 km peak) | 15.7, 3.6 |
| super 5 | 16.6, 4.6 | 15.4, 3.7 | 35.8, 27.0 (28.4 km peak) | 19.5, 3.8 |
| super 6 | 17.5, 5.1 | 16.4, 3.7 | 18.3, 4.0 | 16.1, 2.9 |

Medians (Mann-Whitney p against before): on the base the guard's worst 17.8 to 15.9 km (p 0.046),
the largest height over Airy 4.7 to 3.7 km (p 0.009), the largest rise and fall of a cell in a
step 11.5 and 14.6 km to 8.2 and 10.9 km; on the sheet arm 25.3 to 16.1 km (p 0.004), 5.3 to 3.6
km (p < 0.001), 15.3 and 21.5 km to 6.6 and 11.0 km. No run of either arm leaves two Earth reliefs
or stands a plate column over 8.6 km; the sheet arm's three patches at the cap and its stopped run
are gone.

The V1 measures of the base do not move (`pool.sh table --base base --arm fg-base`, every p 0.19 or
more): land 0.300 to 0.292, the largest landmass 0.124 to 0.144, continental crust under the sea
0.277 to 0.253, land above 2 km 0.271 to 0.245, coast with high ground 0.411 to 0.369, land under
ice 0.170 to 0.168, collisions per 100 Myr 1.80 to 1.75, criteria met per run (median, mean) 2, 2.25
to 2.5, 2.17. The sheet arm against the same base after the fix (`--base fg-base --arm fg-sheet`)
meets 3.5 criteria a run against 2.5 (p < 0.001; 3 against 2 at p 0.063 before the fix): land
above 2 km 0.157 (p 0.001), ice 0.106, the largest landmass 0.230, but land 0.342, over the band.

**What bounds the offset now.** The load is the crust the column gained, so the plate can hold
the ground at most k_a T over its Airy height (the whole root held up), and a belt wider than the
flexural wavelength much less; a height moved without crust cannot raise it again. Over 24 runs the
largest height over Airy is 2.8 to 6.1 km (median 3.7 km), against Earth's 2 to 4 km of rock from
its largest gravity anomalies (ISO-093). A strength limit on the plate (a yield envelope that
lowers `Te` where the plate is bent hard, Watts & Burov 2003) would cut the top of that range; it
was not added, since no run keeps a column more than 5 km over Airy on more than 0.005 % of its
samples. (Added on 2026-10-23 as a switch, off by default, after the polar peaks showed the
transient columns: "The plate's yield", ISO-120.)

**What is left.** The guard's worst is now 14.8 to 21.9 km from the sea in every run of both arms,
above Earth's relief (the Challenger Deep, 10.9 km, ISO-071) but no longer growing. The cells that
make it are traced in "The trench rows".

## The trench rows

**What was seen.** After the settling's fix the guard's worst stood 14.8 to 21.9 km below the sea
in all 24 runs, 4 to 11 km past the Challenger Deep. The base's scattered start, seed 1
(`earthlike_quick_16k`, 1000 Myr), traced step by step with a scratch patch that split each deep
cell's offset from the plate model's height into the surface steps that made it (not kept in the
code): 179 plate steps had a cell more than 11 km down, the run's worst 15.8 km. They are of two
kinds.

- 163 steps: ocean floor where the plate model itself stood at its deepest, the trench depth
  `z_t` = −10 km (median 10.1 km below the sea), and the surface 2.8 km under it. Of that, 2.6 km
  was the flexure's water load. Traced to its start (−724 Myr, 8°S): a cell in a gap of the plate
  model's coverage, its floor GDH1's for its age (3.5 km down), that a plate then covered at
  10.5 km. The carry put the plate's height there, and the flexure loaded the 7 km of new sea over
  it as a new load, 0.45 of it again (ρ_w/(ρ_m − ρ_w)). The plate model reads the surface's offset
  back as the mean over its samples (ISO-078's feedback), clamped at `z_t`, so the samples stayed at
  the clamp and the cells under them deeper still.
- 16 steps, the run's worst among them: rows of cells on a trench (the plate model's TRENCH flag,
  the trench regime), where the plate model stood 6.7 km down and the surface 6.0 km under that at
  the step's start: 4.9 km from the carry, 0.8 km from the flexure's water (nearly all of it the
  sea over the plate model's pull towards `z_t`), 0.3 km from its rock. The worst cell (−806 Myr,
  58.5°S): of its four source corners three were on the other plate, and the one on its own plate
  stood 5.9 km under the plate model's height there (12.7 km down against 6.8). The carry took
  that and added the step's uplift, −1.25 mm/yr or −2.5 km (the trench sample pulled towards
  `z_t`), while the plate model's height at the cell itself rose 0.4 km over the source's: the cell
  began the step 8.8 km under the plate model (15.3 km down against 6.4), and the flexure's loads
  took it to 15.8 km.

**The cause.** Two surface rules counted, a second time, depth that the plate model's heights
already hold.

1. *The sea over the plate model's own moves.* The flexure loads the change of the sea over each
   cell. Since ISO-088 (2026-10-21) it left out the sea over GDH1's cooling, since ISO-091
   (2026-10-22) the sea over a building column's height moved alone; every other move of the
   ocean floor's ground the plate model made, it still loaded: the pull of a trench sample towards
   `z_t` (up to 2.4 km a step), the release from it, and the plate that covers a gap's floor (up to
   7 km in a step). Cortial's `z_t` and GDH1 are depths under the sea, the water's weight in them,
   and so is every height the plate model gives the ocean floor.
2. *The carry on a trench row.* The surface carries its ground with the crust (advect.rs), and on
   a cell that is not in a gap adds the plate model's uplift there, `(z_end − z0)/dt` of the
   samples round it. That is right where the samples move with the crust. On a trench row the crust
   moves into the trench and is consumed: each step the cell's source is a cell of the same row
   that the step before already took down, the uplift of the trench sample (its pull towards `z_t`)
   is added again, and the plate model's height at the cell, interpolated from samples that are
   consumed and replaced, does not follow. The sum of the pulls ran 5 to 9 km under the plate
   model's own trench, and the plate model's feedback (means over its samples, 71 km apart) does not
   take back a row one cell wide.

**What Earth does.** A trench is the ocean floor bent down by the subducting plate, and the sea
fills it: the water "fills in the space vacated" by the mantle the bent plate moves aside, and the
restoring force is (ρ_m − ρ_w) g w (Turcotte & Schubert §3.13, eqs. 3.101 to 3.103); fits of the
trench's outer wall take the restoring force so and fit the depth of the floor under the sea
directly (Levitt & Sandwell 1995; Zhang et al. 2014, eq. 1). The deepest floor is the Challenger
Deep at 10.9 km (ISO-071), with a trench relief, its depth below the far-field floor, of 5.7 km;
along the Mariana trench the relief is 0.9 to 5.7 km (Zhang et al. 2014), and averaged over
sections of the world's trenches 0.9 to 4.7 km (Zhang et al. 2018). The bent plate's elastic
thickness is 20 to 30 km at most trenches (Stern). The plate model's `z_t` of −10 km and its pull
there are its picture of that bend, the water in them.

**The fix** (ISO-096, ISO-097).

1. *The water* (ISO-096). The flexure books the plate model's move of the ground as the carry made
   it, the step's start less the surface's elevation at the crust's source before the step (carried
   with the flexure's state, `FlexState::from_m`; the plate model's uplift where that is unknown).
   On ocean floor that builds nothing (oceanic crust outside arcs and hotspots) the whole move is a
   height moved alone, its sea no load; on building columns the crust's own change is a load as
   before (ISO-091) and the rest of the move is not. This covers GDH1's cooling (ISO-088, whose
   separate rule went), the trench's pull and release, the carry's onto a trench row and a gap's
   floor covered by a plate. The change of the sea level itself, the water that erosion and
   deposition displace and the water thickened crust pushes aside still load the floor.
2. *The trench rows* (ISO-097). On a cell at a trench (the TRENCH flag or the trench regime) the
   carry no longer adds the uplift to the carried ground: the cell takes the plate model's height
   there plus the offset carried from its source (the surface's own departures: its sediment, its
   flexure), as the carry already did in a gap. The plate model's trench sets the trench's depth.

Unit tests `the_sea_over_the_ocean_floors_own_moves_is_no_load` (GDH1's sinking and a trench's
2.4 km pull sink the floor by exactly the move; a gap's floor at −3.5 km covered by a plate at
−10 km sinks no further; without the source's height, 6.5 km × 0.45 more, as before; a sea 100 m
higher sinks the floor by 0.45 of it) and `a_trench_carries_its_offset` (advect.rs).

**Measured** (the shared pool, `earthlike_quick_16k`, 1000 Myr, both starts, seeds 1 to 4, 8 runs
an arm, `+DEEPTIME_CRUST_BUDGET=1`; each argument set against itself without the fix on the same
seeds; medians, two-sided Mann-Whitney p). Three argument sets: the base; the trench budget's three
keys (`trench_release_half_life_myr=2`, `sediment_subduction_share=0.12`,
`subduction_erosion_km3_per_km_myr=90`); and those with the high ground's sheet (the four
`collapse_*` keys), the seven that are master's default since 57558f0a, so that column is the
fix against the pool's new `base` (its seeds 1 to 4). Arms `base`/`fg-trench` (the old base),
`trench-m-rs12e`/`fg-trcomb-rs12e`, `trench-m-rs12e-sheet`/`fg-trench-sheet`. The branch with
57558f0a merged in, at its own default, made `fg-trench-sheet`'s supercontinent 1 again, every
field and the plate table byte for byte.

| measure (Earth) | base | fixed | p | trench keys | fixed | p | seven keys | fixed | p |
|---|---|---|---|---|---|---|---|---|---|
| deepest floor, m (9,242 at 15', 10,920) | 12,943 | 10,169 | < 0.001 | 11,154 | 9,113 | 0.001 | 10,953 | 9,674 | 0.028 |
| the plate model's deepest, m | 10,672 | 9,662 | 0.002 | 9,387 | 9,240 | 0.28 | 9,561 | 9,289 | 0.11 |
| floor deeper than 8 km, share (0.0003) | 0.0318 | 0.0163 | 0.021 | 0.0022 | 0.0007 | < 0.001 | 0.0030 | 0.0007 | 0.001 |
| floor p99, m (5,953) | 8,967 | 8,292 | 0.001 | 6,951 | 6,663 | 0.010 | 7,115 | 6,746 | 0.050 |
| floor deeper than 6 km, share (0.0073) | 0.145 | 0.167 | 0.23 | 0.054 | 0.077 | 0.57 | 0.099 | 0.121 | 0.20 |
| floor p50, m | 3,924 | 4,271 | 0.083 | 3,971 | 4,181 | 0.010 | 4,276 | 4,308 | 0.80 |
| the guard's worst, m from the sea | 15,898 | 14,452 | 0.015 | 16,938 | 13,704 | 0.065 | 15,640 | 13,837 | 0.33 |
| the guard's worst, the arm's largest | 19,684 | 21,175 | | 28,548 | 30,945 | | 20,619 | 25,955 | |
| V1 criteria met per run, mean | 2.50 | 2.62 | 0.91 | 2.00 | 2.12 | 0.91 | 2.88 | 3.00 | 0.78 |

The guard's worst per run, m (scattered continents 1 to 4, then supercontinent 1 to 4):

| arm | sc1 | sc2 | sc3 | sc4 | sp1 | sp2 | sp3 | sp4 |
|---|---|---|---|---|---|---|---|---|
| base | 16,894 | 19,684 | 15,892 | 15,901 | 15,818 | 15,896 | 15,322 | 18,084 |
| fixed | 15,053 | 13,916 | 13,312 | 14,478 | 12,370 | 14,427 | 21,175 | 15,541 |
| trench keys | 17,392 | 15,515 | 18,474 | 16,174 | 14,778 | 28,548 | 16,483 | 19,525 |
| fixed | 15,012 | 17,761 | 12,153 | 12,878 | 12,148 | 14,530 | 30,945 | 11,784 |
| seven keys | 16,309 | 15,550 | 20,619 | 15,730 | 15,225 | 16,608 | 14,991 | 15,165 |
| fixed | 12,420 | 13,084 | 18,833 | 25,955 | 17,481 | 12,395 | 14,590 | 13,017 |

The surface's deepest floor now stands within 0.1 to 0.5 km of the plate model's (median of the
arm against median; it was 2.3, 1.8 and 1.4 km under it), and inside the trench measure's 8 to 11 km
in all three sets. The plate model's own deepest rose 1.0 km on the base set: its samples read the
surface's offset back (ISO-078), and the offset the sea added under its trenches is gone. The V1
measures did not move in any set (every p 0.13 or more over the seven and the criteria; 0.38 or
more in the seven-key set), so the runs stopped at 4 seeds. The median floor came out 0.2 to
0.35 km deeper in two sets (p 0.010 and 0.083, not in the seven-key set), most likely (not traced)
because the sea the plate model's rises of the floor took away (a trench's pull released, for one)
had lifted it by 0.45 of each rise, and that is gone with the rest. The share deeper
than 6 km is the plate model's (its own share moved with it, 0.143 to 0.168 on the base set), the
trench budget's open item.

**What is left.** The guard's worst median fell 1.4 to 3.2 km, to 13.7 to 14.5 km, and single runs
still reach 19 to 31 km for a few steps (the arms' largest, above; 23 steps beyond 19.8 km in the
worst). Those runs differ from one arm to the next, with the fix and without it (28.5 km in the
trench keys' supercontinent 2 before it, 30.9 km in their supercontinent 3 after). The guard's
worst is the cell farthest from the sea either way, and these are peaks, not floors. Traced (a
scratch patch, not kept, printed the guard's cell at every step past 11 km; the reruns gave the
pool's 30,945 and 25,955 m to the metre):

- The trench keys' supercontinent 3, fixed: from −262 to −216 Myr a continental column on the
  polar rows (79 to 81°N) stood 18.6 to 29.4 km above the sea in the plate model itself (the
  preset's cap, `max_elevation_m`, is 30 km). It was a collision, back-arc and rift column whose
  plate uplift ran 1.3 to 3.5 mm/yr each step while the peak moved 90 cells west along the row.
  The surface stood 1.6 km over the plate model at the worst step.
- The seven keys' scattered 4, fixed: from −222 to −212 Myr a back-arc and rift column at 37°S
  stood 15.5 to 29.2 km above the sea in the plate model, beside trench cells 7 km down. The surface
  stood 3.3 km under the plate model at the worst step.
- Every step past 19.8 km in the two runs is such a peak (23 and 6 steps). The deepest floors in
  them were 13.8 and 14.4 km down, 3 to 3.5 km past the Challenger Deep, on trench cells where the
  plate model stood 8.2 and 10.4 km down and the surface's carried offset was still −6.1 and
  −8.2 km. What builds that offset under a trench now was not traced here; it was traced on
  2026-10-23 to the plate model reading the held offset back every step ("The held offsets").
- The trench keys' supercontinent 2, before the fix (traced on the trench budget's head, the
  pool's 28,548 m to the metre): the same kind of peak. From −46 to −14 Myr a collision, rift and
  back-arc column on the polar rows (67 to 73°S) stood 14.5 to 29.0 km above the sea in the plate
  model, its uplift at 1.3 to 3.5 mm/yr, and the surface's offset over it reached +10 to +13 km at
  the worst steps. All 13 steps past 19.8 km are that peak. The same run had 341 steps with a floor
  more than 11 km down, the deepest 15.8 km; the two fixed runs above had 25 and 22.

Those heights are the plate model's own (plates/), outside this lane: a column whose uplift runs
at the rate limit's ceiling (PLT-288, 3.5 mm/yr) step after step, up to the 30 km cap, on the
polar rows twice in three runs.

## The release's pace

Added 2026-10-23 (ISO-099), after the savanna pit (build log 2026-10-23, "Savanna pit, part B").

**What was seen.** On Ben's seed (`earthlike_quick_16k`, supercontinent, 1,000 Myr) a pit opened
in the last three steps at the L1 cell (396, 231): its regime went from craton to back-arc, the
plate let go of 2,047, 1,216 and 1,025 m of held load (ISO-056; the sea's share of it −43 m), and
the floor fell 4.3 km in 6 Myr, with `plates/thickness_m` going from 38.6 to 27.6 km over the last
40 Myr. On the pool's base runs (12, both starts, 1,000 Myr) the `basins` measure counts 61 to 246
closed basins deeper than 10 blocks (350 m) a run, median 116, and 33 to 145 of them, median 81,
with their floor under sea level. Earth's lowest dry land, the Dead Sea's shore, was 433 m below
the sea in October 2018 (Vey et al. 2021, full text), and it is low because the lake is drying, not
because the plate let go.

**The cause.** The release itself is the right kind of thing: a plate that weakens stops holding
up what it held, and the ground goes towards local isostasy (ISO-056). Its pace was not. The law's
`Te` (ISO-048 to ISO-053) is a thermal state: the depth of an isotherm, set by the floor's age,
the time since the last orogeny, or an arc's or a rift's heat. When the Andean arc's uplift
reaches a craton the plate model stamps an orogeny on it (`plates/subduction.rs`, PLT-060), the
law's clock restarts, and `Te` went from the craton's 80 km to the floor's 5 km in the step:
`D` fell to (5/80)³ = 0.0002 of itself, and nearly all the load the craton held came loose at once
(over three steps here, as the smoothed `Te` of the neighbouring cells followed).

**What Earth does.** A plate loses strength when its interior warms (Te follows the depth of
the 300 to 600 °C isotherms, Watts & Zhong 2000, full text), and heat crosses the lithosphere by
conduction. The time that sets the pace is the lithosphere's thermal time constant,
`τ = a² / (π² κ)`: 62.8 Myr for a plate a = 125 km thick (McKenzie 1978, Table 1 and eq. 6, read
in full text; Parsons & Sclater's sea-floor cooling constant, the same 62.8; the same `τ` the
plate model's thin sheet uses for an orogen's strength, PLT-415). "The thermal effect of
extension decays with a time constant of ~50 m.y." (Hyndman, Currie & Mazzotti 2005, full text).
The viscous relaxation of the elastic layer is faster: most of it within 1 to 2 Myr of loading,
with small changes out to 100 Myr (Watts & Zhong 2000), a fitted 10^6 yr in Beaumont 1978
(abstract). So it does not set the pace: the plate cannot let go of a load before it is warm
enough to flow. The plate model itself lets an orogen's heat flow decay over 60 Myr
(`plates/mod.rs`). Faster routes exist where the mantle does the work: `τ` goes as the plate's
thickness squared, so a back-arc's 50 to 60 km lithosphere (Currie & Hyndman 2006) re-equilibrates
in 10 to 15 Myr, and models remove a back-arc's lower lithosphere by convection in 5 to 10 Myr,
most of the thinning "within 30 Ma of subduction initiation" (Currie, Huismans & Beaumont 2008,
full text). One `τ` for every plate is the model's simplification (ISO-099).

**The fix.** The plate holds a law's `Te` of its own, carried with its crust (`FlexState::te_law_km`,
as a logarithm like the rigidity, ISO-075). Each step, where the law gives a lower `Te` than the
plate held, the plate's `Te` relaxes towards it as `e^(−dt/τ)` (`FlexureParams::te_weakening_myr`,
62.8 Myr; `weakened_te_km`): a 2 Myr step closes 3.1 % of the gap, so the craton above goes from
80 to 77.7 km and lets go of about 9 % of its held load, not nearly all of it. Where the law gives more, the plate
takes it at once, stress-free, as before (ISO-056). The break at a plate boundary (ISO-055) is
mechanical, not thermal, and stays at once. `te_weakening_myr = 0` is the old rule, and it is the
default: on the pool's runs (below) the pace left the criteria where they were and gave more closed
basins and larger one-step falls, so it is off until that is understood (the agent's call,
2026-10-23); `--set surface.flexure.te_weakening_myr=62.8` turns it on. Unit test
`a_plate_the_law_weakens_lets_go_as_it_warms`: a belt on a craton the law weakens lets go of a
few per cent of the instant release in the first step and stands where the instant rule put it
after eight time constants.

**Does the surface's sediment fill it?** As far as sediment reaches it. A closed basin is a
lake in the surface stage (`pflood`: ground under sea level that the open sea does not reach is
land) and a lake keeps all the sediment that reaches it up to its room, settling it over its whole
floor (HYD-110, `spl.rs`); land nothing drains keeps everything. What it gets is what its
catchment erodes, so the fill is limited by supply, not by the rules. On Earth a basin that opens
inside a continent with big rivers does fill as it goes: in the Pannonian basin "Sediment
accumulation rate was significantly higher than the subsidence rate during Late Miocene time",
and its plain now stands 90 to 120 m above the sea on up to 700 m of Plio-Quaternary strata
(Horváth & Cloetingh 1996, full text), although by simple isostasy its thin crust "should be well
below the sea level" (Horváth et al. 2015, full text: the basin is "over-elevated").

Measured with the pool's `fill` measure (`tools/review/pool/measure_fill.py`: the `basins`
measure's closed basins deeper than 10 blocks, their net deposition and strata at the last step)
over the 24 runs with the pace on, base arguments: the median basin takes 271 m of sediment a Myr
(the runs' medians 120 to 698) and already holds 4.0 km of strata (3.3 to 4.9). Their room over
the last step's deposition is 2.5 Myr (1.8 to 3.6), the median basin's own fill time 2.7 Myr (1.5
to 4.2), and 84 % of the basins (71 to 94 %) would fill within 10 Myr. The 24 runs on the
trench-m-rs12e-sheet arguments are alike: 268 m/Myr, 2.7 Myr, 85 %. The one run of the old rule
(supercontinent, seed 1): 499 m/Myr, 2.0 Myr, 95 %; the base runs were measured before this field
was dumped, so there is no wider comparison. So the surface fills a basin as it opens, in a few
Myr, much faster than the plate weakens: the basins left in the final surface are ones that sink
about as fast as sediment arrives, or opened in the last few Myr (which of the two, not traced).

**Measured.** Through the pool, 12 seeds by two starts, 1,000 Myr, `earthlike_quick_16k`: the
branch with the pace on (source key 76b4db9c66ff, τ = 62.8 Myr) on the old base's arguments
(`fg-release`) and on the trench-m-rs12e-sheet set (`fg-release-sheet`: the sheet's four keys
and the trench budget's three, master's default since 57558f0a), each against the pool's 24 runs
of the same arguments without the pace (the old `base`; `trench-m-rs12e-sheet`, the pool's `base`
since that merge). One run with `te_weakening_myr = 0` (supercontinent, seed 1) is bit-identical
to the old base's. With 57558f0a merged in, the branch at its default made the new base's
supercontinent 1 again and with τ = 62.8 made `fg-release-sheet`'s again, every field byte for
byte, so the right-hand columns are the pace against the new base. Medians (the criteria: the mean
a run), Mann-Whitney p:

| measure | old base | pace on | p | the default | pace on | p |
|---|---|---|---|---|---|---|
| V1 criteria met (of 7) | 2.33 | 2.54 | 0.60 | 3.42 | 3.17 | 0.45 |
| land share | 0.292 | 0.302 | 0.63 | 0.306 | 0.309 | 0.67 |
| largest landmass | 0.139 | 0.124 | 0.53 | 0.189 | 0.157 | 0.16 |
| closed basins deeper than 10 blocks | 125 | 144 | 0.27 | 102 | 130 | 0.018 |
| of them under sea level | 86 | 97 | 0.58 | 69 | 86 | 0.022 |
| deeper than 30 blocks | 75 | 74 | 0.88 | 54 | 70 | 0.12 |
| deepest basin, blocks | 303 | 315 | 0.42 | 292 | 291 | 0.58 |
| closed ground under sea level, kb² | 716 | 675 | 0.29 | 441 | 563 | 0.34 |
| their room, km³ | 69,855 | 74,193 | 0.98 | 60,527 | 66,374 | 0.26 |
| the guard's worst, m | 16,029 | 16,394 | 0.92 | 16,351 | 15,930 | 0.84 |
| largest one-step fall of a cell, m | 11,404 | 13,984 | 0.003 | 10,649 | 14,363 | < 0.001 |
| largest one-step rise of a cell, m | 7,993 | 8,189 | 0.065 | 6,692 | 7,981 | < 0.001 |

- **V1 does not move** in either set. On the first four seeds the base set's largest landmass
  fell from 0.191 to 0.104 (p 0.028); at twelve it is 0.139 against 0.124 (p 0.53), so that was
  the seeds' draw.
- **The closed basins do not fall: they rise**, significantly on the sheet set (102 to 130, and
  69 to 86 of them under sea level). The pace was meant to stop a pit opening in a few steps; the
  final surface has more closed basins, not fewer.
- **The largest one-step fall of a cell grows** by 2.6 and 3.7 km, on the polar rows and away from
  them alike (base set: polar rows 8.4 to 10.3 km, p 0.003; elsewhere 10.9 to 13.9 km, p 0.001),
  and on the sheet set so does the largest rise.

Why, a hypothesis that was not traced: a plate that weakens slowly keeps its strength, and the
load it holds away from local isostasy, for longer, so the break at a plate boundary (ISO-055),
still at once, lets go of more in its one step; and a plate kept stiff makes broader moats and
bulges, which the priority flood finds as closed basins. The water the flexure puts on closed dry
ground under sea level (it calls a cell wet by `h <= sea`, not by the open sea) would deepen the
ones under the sea. The savanna pit's own cell cannot be compared: a changed rule makes the
planets part after its first change. With the pace on as the default, the bake's unit test
`a_planet_run_bakes_every_chunk_time_layer` also fails: one cell of its small planet bakes a
continental heat flow of 33.7 mW/m², under the plate model's floor of 55, which is the sea floor's
law at 229 Myr; most likely a continental sample still holding an oceanic value (a plate-model
edge case the changed history reaches, not traced).

## The plate's yield

Added 2026-10-23 (ISO-120), from the polar peaks ([plates.md](plates.md), "The polar peaks"). On
by default since the flexure-faults lane the same day, at 380 mGal;
`--set surface.flexure.yield_anomaly_mgal=0` turns it off (a purely elastic plate).

**What was seen.** On the pool's base (master bf8477b2, 24 runs) the guard's worst passed 24 km in
four runs (24.7 to 30.0 km). Traced on seeds 1 to 4 (scattered 4, at 26.0 km, the worst of them),
those steps are plate columns of 25 to 40 km of crust standing 10 to 30 km high: the settling (ISO-078) holds each at its Airy height plus `F`, the flexure's offset,
and `F` under them was 10 to 47 km. The plate held the roots of the crust the collapse fed them step
after step, and nothing bounded what it held. "What bounds the offset now" (in "The settling's
feedback", above) reckoned that bound as at most k_a T of the column's own crust; a column the
collapse keeps feeding has no such limit, since every step's crust is new load.

**What Earth does** (ISO-093). No plate holds more than about 2 to 4 km of rock away from local
isostasy: the largest free-air anomalies over held loads are −380 mGal at the Puerto Rico Trench,
"the most negative free-air gravity anomaly on Earth" (ten Brink et al. 2004), and up to +300 mGal
over Hawaii (Watts et al. 2025). What sets the limit is the plate's strength: "Ideal elastic
materials support any stress level. In the case of real materials, however, stress levels are
limited to the rock strength at the corresponding depth", and `Te` falls to about half at a
curvature of 10⁻⁶ m⁻¹, "50% yielding" (Watts & Burov 2003); an elastic plate bent as hard as the
trench walls would carry "unreasonably high bending stresses (up to 900 MPa)" (Hunter & Watts 2016).

**The law** (ISO-120). The plate's strength is the largest free-air anomaly it holds, Δg_max. A
sheet of surplus mass σ per square metre pulls 2πGσ (the Bouguer slab), so the most load a plate
holds away from local isostasy is σ_max = Δg_max / 2πG, with G = 6.674 30 × 10⁻¹¹ m³/(kg s²)
(CODATA 2022). At 380 mGal, σ_max = 9.06 × 10⁶ kg/m²: 2.75 km of mantle, the ground held 2.75 km
from local isostasy on land and 3.99 km under the sea (ρ_m − ρ_w). After each solve, where a cell's
held load ρ_m (Q + ice − w) passes σ_max either way, the plate there has yielded: the excess sinks
(or rises) under the cell alone, w gains (held − σ_max) / ρ_m with the held load's sign, and the
elevation, the carried state (ISO-068) and the step's release record (`released_m`, which the gap
budget reads) take it. That is the answer of a plate with no strength left (local isostasy, as `D`
goes to 0: "The polar holes"), for the part of the load past the strength. Where the plate holds
less, nothing changes: the solve is the elastic one, bit for bit. The count of cells that yielded is
kept in `surface/flexure_yielded_cells`. Unit test `a_plate_that_yields_holds_no_more_than_its_strength`:
a column two cells wide on a 64 × 64 craton, fed 1 mm/yr of Airy height for 12 steps, stands held
more than four times the bound by the elastic plate; with the yield no cell holds more than the
bound at any step, the column stands within the bound of its Airy height, and a load the plate can
hold (0.05 mm/yr) runs the same with and without the switch.

**Why after the solve, and not a weaker plate.** The first form lowered the cell's rigidity to
D σ_max / |held| for the next solve and let ISO-056 release the rest. It failed the test: after one
step the column held 20.8 km against a bound of 2.7 km, because the force ISO-056 released was
spread again by the stiffness the neighbours kept.

**Simplifications.** The bound is on the load a cell's plate holds, not on its bending stress: a
yield envelope (Goetze & Evans 1979, abstract; Watts & Burov 2003) would need the stresses through
the plate, which the solve does not keep. Over a load narrower than its depth of compensation the
anomaly is less than 2πGσ, so for one cell of the quick presets (39 km) the bound is stricter than
Earth's anomalies require. The bound does not depend on `Te`, the plate's age or the latitude.

**Measured** (pool arm `pp-yield380`, key 08c9d7d679c0, seeds 1 to 12 by both starts, 1000 Myr,
`+DEEPTIME_CRUST_BUDGET=1`, against the 24 runs of master bf8477b2; per-run tables in
[plates.md](plates.md), "The polar peaks"). The guard's worst fell from 13.7 to 13.1 km at the
median (Mann-Whitney p 0.015) and from 30.0 to 15.5 km at the largest; 9 base runs passed 15 km and
4 passed 20 km, against 1 and 0 with the yield. The largest one-step fall went from 9.45 to 7.42 km
(p < 0.001) and the largest rise from 6.99 to 6.52 km (p 0.012). No V1 criterion moved (every p
0.24 or more; criteria met per run 3.33 to 3.04, p 0.31).

**On by default** (2026-10-23). Under the rule of the decisions log of that day (Ben: "Always, fix
V1 elsewhere"; physics goes on by default once it is built and correct, the V1 criteria are realism
checks and no gate), the yield is on: it is unit-tested against its sources' bound, and its pool
arm had no divergence and no guard failure while it cut the guard's worst and the largest one-step
fall. No forcing term goes off with it: nothing else in the model bounded the load a plate holds.

**300 or 380 mGal.** The bound is the largest anomaly a plate holds. 380 mGal is the Puerto Rico
Trench's, Earth's most negative free-air anomaly (ten Brink et al. 2004), but a trench's anomaly is
not all a load held by the plate's strength: the slab's pull and the mantle's flow under it hold
part of it down. Hawaii's +300 mGal (Watts et al. 2025) is a volcanic load on old ocean floor, the
textbook case of a load the plate's strength alone holds, so 300 mGal is the better-sourced bound
for what this law bounds, and 380 the larger of the two measured. 380 stays: with the held offsets
on (below) the bound is passed rarely (13,000 cell-steps at the median of a 1000 Myr run of 500
steps, about 27 cells a step), so the choice between them moves little, and no 300 mGal arm was
run. An open question.

## The held offsets

Added 2026-10-23 (ISO-125), the flexure-faults lane, for the last two faults the yield left: the
oceanic carry peaks ([plates.md](plates.md), "The polar peaks", "What is left") and the trench
offset ("The trench rows", "What is left").

**What was seen.** With the yield on, the largest excursions left in the plate model were ridge
and basin samples raised 10 to 16 km, the carry's part of their gap 9 to 23 km; one stood 14.7 km
high at 78°S in the last step of the scattered start, seed 1. Under the trenches some cells still
carried an offset of −6 to −8 km under the plate model's height, their floors down to 14.4 km.

**The trace.** That run (`earthlike_quick_16k`, 1000 Myr, scattered start, seed 1, the yield at
380 mGal, `+DEEPTIME_CRUST_BUDGET=1`) ran again with the peak trace (`DEEPTIME_PEAK_TRACE=10000`,
plates/peaks.rs, which now also prints the share of a sample's cells whose offset the surface holds
and what those cells handed its elevation in the step) and a floor trace of the surface
(`DEEPTIME_FLOOR_TRACE=11000`, surface/floors.rs: every cell more than 11 km under the sea, the
plate model's height under it, its offset over that at the step's start and end, and whether the
surface holds that offset; measurement only, the world is the same).

- 17 peak lines, every one an ocean sample (Airy height −1.8 to −3.7 km) at a ridge, 10.0 to
  14.7 km high, 9 of them on the polar rows. 0.2 to 0.99 of their cells (0.75 or more in 11 of
  the 14 with a mask) lay in a gap between plates, and those cells handed the sample's elevation
  0.7 to 2.8 km in the step, which is what the sample rose by: at −2 Myr, 77.9°S, it went from
  9.5 to 12.3 km in one step, 2.8 km of it from its gap cells.
- 34 floor lines, every one on a trench row (the trench regime): 11.0 to 15.5 km under the sea,
  the plate model's height under them −6.8 to −9.4 km, their offset at the step's start −1.2 to
  −7.9 km. The deepest (−46 Myr, 62°N): 15.5 km down, the plate model at −9.4 km and the offset
  −5.3 km. The guard's worst of the run was this cell.

**The cause.** Two rules each kept the same offset. The surface holds its offset over the plate
model's height on two kinds of cell: in a gap between plates the carry takes the plate model's
height there plus the offset carried from the cell's source (the gap budget's carry), and on a
trench row it does the same (ISO-097, "The trench rows"). The plate model's feedback (ISO-078)
read that offset back every step as well, as the mean of `surface h − plate elevation` over each
sample's cells, and added it to the sample's elevation. The next step the surface again stood the
same offset over the sample's new height, and the feedback added it again: an integrator. Beside a
gap the offset is positive (the floor the gap carries stands over the plate model's young ridge)
and the ridge samples rose by it step after step; under a trench it is negative (the trench row's
sediment and flexure) and the trench samples sank by it, and the cells over them stood that much
deeper again.

**The fix** (ISO-125, `surface.held_offsets`). The surface marks the cells whose offset it holds
(`surface/offset_held`: a gap between plates or a trench row) and hands the plate model no offset
there; the plate model counts such a cell as an offset of 0 at its full weight (a sample half on
held cells takes half the offset of the rest). The carry onto a trench row then keeps only the
offset the surface held, and the climate stage's carry reads it the same way. The first form
renormalised the mean over the cells not held: in the traced run a collision column whose cells
were 92 % held took 13 times its few other cells' offset and rose 7.6 km in a step (2.7 to
10.3 km), so the held cells now keep their weight.

Unit tests `the_surface_hands_the_plate_model_nothing_of_a_held_offset` (surface/tests.rs),
`the_held_cells_are_the_gaps_and_the_trench_rows` and `a_trench_carries_its_offset` (advect.rs; a
trench row whose source was half held takes half the offset, one whose offset the plate model took
the plate's floor alone).

**Traced with the fix** (the first form, renormalised): one peak line (the collision column above)
and no floor past 11 km; the guard's worst 10.3 km against 15.5 km, the highest ground 7.0 km
against 8.2 km, land 0.301 against 0.322, collisions 15 against 18.

**Measured** (pool arm `ff-held`, key 9bdf6a15219c at 9a46f3c2, the yield at 380 mGal and the held
offsets, `earthlike_quick_16k`, 1000 Myr, seeds 1 to 4 by both starts, `+DEEPTIME_CRUST_BUDGET=1`;
medians against the same eight runs of the base, master bf8477b2, key 7f9d8f115049, and of the yield
alone, `pp-yield380`, key 08c9d7d679c0; Mann-Whitney p against the base):

| measure | base | yield | yield + held offsets |
|---|---|---|---|
| the guard's worst (km) | 13.84 | 12.92 (p 0.20) | 10.17 (p < 0.001; every run 9.7 to 10.5) |
| largest one-step fall (km) | 9.44 | 7.53 (p 0.015) | 7.08 (p 0.002) |
| largest one-step rise (km) | 6.90 | 6.68 (p 0.65) | 6.50 (p 0.074) |
| most over the Airy height (km) | 3.85 | 3.29 (p 0.16) | 3.21 (p 0.083) |
| deepest sea floor (km) | 9.67 | 9.25 (p 0.16) | 8.94 (p 0.050) |
| share deeper than 6 km | 0.121 | 0.114 | 0.099 (p 0.44) |
| V1 criteria met per run (mean of 7) | 3.00 | 3.00 (p 0.87) | 3.75 (p 0.13) |
| land | 0.292 | 0.296 | 0.315 (p 0.16) |
| cells yielded (whole run) | | | 13,340 |

No measure is worse: the worst spike is down by a quarter, and the spread of the guard's worst
between runs, 9.7 to 10.5 km, is a third of the base's. The plate model's own deepest floor
(9.41 km against 9.29) did not move (p 0.65): the trench floors under 11 km were the surface's.

**On by default** (2026-10-23): it removes an integrator, a numerical fault and no physics, and the
pool shows no divergence and no guard failure. Nothing goes off with it.

## Water by basin

Added 2026-10-23 (ISO-126), `surface.flexure.water_by_basin`.

**The fault.** The flexure loads the change of the water over each cell, and it took a cell to be
under water when its ground stood at or under the sea level, `h ≤ sea`. A closed basin below the
sea level with no outlet to the ocean (the Dead Sea's, the Caspian's, the Qattara Depression's) was
loaded as if the sea filled it, and a lake above the sea level carried no water.

**The law.** The water over a cell stands at the level of the body it is in:

- the open sea (the surface's open-sea mask, `pflood::open_sea`, taken again on the ground the
  flexure starts from: every body at or under the sea level of at least 1/500 of the cells,
  joined to the ocean or not, the surface's own rule for what is sea) at the sea level;
- a lake (a closed basin the routing's flood fills, `filled − h` over 1 cm) at the level its water
  balance holds: the basin's inflow (the runoff routed into it and what the lakes above it spill)
  against the evaporation less the rain over its surface (`HydroParams::budget`, HYD-050, HYD-057,
  HYD-100, HYD-101; the bake's lakes take the same balance). If the inflow
  meets the loss over the whole flooded area, the lake fills to its spill level; otherwise it fills
  from its lowest cell up while the loss over the cells it covers is less than the inflow, and the
  level is the height of the first cell it cannot cover. A lake whose level is at or under the sea
  level stands at the sea level (it is the sea's);
- everything else is dry: no water, however low.

The flexure's water load and its choice of ρ_m or ρ_m − ρ_w per cell take that level instead of
the sea level. Unit tests (surface/lakes.rs): `the_sea_is_the_sea_and_land_is_dry`,
`a_wet_climate_fills_a_closed_bowl_to_its_rim`, `a_dry_climate_leaves_a_closed_bowl_dry`,
`a_terminal_lake_stands_where_its_losses_take_its_inflow`, `a_lake_at_the_sea_level_is_the_sea`,
`ground_the_sea_reaches_after_the_flood_is_under_the_sea`. The stage summary counts the cells under
the sea level that are dry and the cells over it that are wet
(`surface/flexure_dry_below_sea_cells`, `surface/flexure_wet_above_sea_cells`), and the lakes.

**Measured** (pool arm `ff-basin`, key 9bdf6a15219c, the yield at 380 mGal and water by basin, the
same eight runs as `ff-held` above). Over a run, at the median, 86,900 cell-steps under the sea level
were dry (about 174 cells a step) and 1.34 million over it were wet (about 2,700 a step: the lakes'
water, which the old rule never loaded). Against the base: the guard's worst 13.11 km (p 0.38), the
largest one-step fall 7.30 km (p 0.004, the yield's), the deepest floor 9.88 km (p 0.88), criteria
met per run 3.12 (p 0.96); no measure moved past the yield alone's. The law changes which cells
carry water and so where the plate bends under lakes and dry basins, not the planet's large
numbers.

**Simplifications** (game-choice, numerical, the agent's; surface/lakes.rs). The lakes' levels are
those of the step's last flood, on the elevation it flooded; the open sea is taken again on the
elevation the flexure starts from, so ground the step's later processes put under the sea level
carries the sea. Neither is taken again on the ground the flexure moves in its own step. One level
stands over each flooded depression, so a basin whose balance level falls under a sill inside it
is still one body of water (its water's mass is right, its share between the sub-basins is not),
and a lake with two outlets spills over the one its last exit drains to.

**On by default** (2026-10-23): it is the load the physics asks for (water where water stands),
unit-tested against the balance it takes from the hydrology, and the pool shows no divergence and
no guard failure. The rule it replaces, `h ≤ sea`, goes off with it.

## The solve on the sphere's metric

Added 2026-10-23 (ISO-127), `surface.flexure.row_metric`.

**The fault.** The L1 raster is Lambert's cylindrical equal-area projection: x = R λ, y = R sin φ
(Snyder 1987, eqs. 10-1, 10-2 with the equator as the standard parallel), so the scale along a
parallel is 1 / cos φ and along a meridian cos φ (eqs. 10-2a, 10-2b), and a cell at latitude φ is
cos φ as wide and 1 / cos φ as tall on the sphere as a cell at the equator, its area the same. The
solve treated every cell as a square of the equator's width (the block metric). The Laplacian in
the raster's coordinates is ∂x(cos⁻²φ ∂x) + ∂y(cos²φ ∂y) (the area element is uniform, so the
metric's factors stand inside the derivatives), and the plate equation's operator is its square:
the block metric is about 1 / cos⁴ φ too stiff north to south and cos⁴ φ too soft east to west.

**The law.** In finite volumes on the raster, each row's east-west coupling is a_j = 1 / cos²φ at
its middle and each boundary between rows has the north-south coupling b = cos²φ there (0 at the
raster's first and last edge, closed as before; a row wholly past a pole takes cos²φ as half a
row's height in sin φ, so the operator stays finite: a numerical choice). The operator stays
symmetric, and at the equator it is the block metric's. The preconditioner, which solved the block
metric's operator exactly by a Fourier transform along both axes, keeps the transform along x and
solves each wavenumber's column exactly as a symmetric pentadiagonal system (an L D Lᵀ
factorisation), so it is still the exact inverse of the uniform-rigidity operator; its diagonal
scaling takes the metric's diagonal.

Unit tests `the_row_metric_is_the_block_metric_at_the_equator`,
`the_solve_on_the_sphere_inverts_its_operator` and
`a_load_along_a_row_bends_the_plate_as_on_the_sphere` (flexure.rs). The last puts a strip load
along a row at latitudes 0 to 80° and compares the sinking across it with the analytic profile of
a beam on an elastic foundation under a uniform load over a finite width (Hetényi 1946, §5,
eqs. 7a and 8a), with the strip's width and the flexural length in the sphere's kilometres (a
uniform plate, α = 400 km; the worst error over the rows within 2α of the strip, as a share of
the analytic peak):

| latitude | sphere's metric | block metric |
|---|---|---|
| 0° | 0.004 | 0.003 |
| 45° | 0.016 | 0.292 |
| 60° | 0.028 | 0.509 |
| 70° | 0.043 | 0.643 |
| 80° | 0.086 | 0.697 |

A single solve on the sphere's metric is right where the block metric is not.

**Measured: the step diverges** (pool arm `ff-metric`, key 9bdf6a15219c, the yield at 380 mGal and
the metric, seeds 1 to 4 by both starts, 1000 Myr). Six of the eight runs were stopped by the
surface guard, 202 to 942 Myr into the run, every one at a cell of the row that reaches the north
pole (row 1) or of the last row, wholly past the south pole (row 327), 0.4 to 3.6 thousand km under
the sea; the two that finished had the guard's worst at 25.7 and 234 km. The metric trace
(`DEEPTIME_FLEX_METRIC_TRACE=1`, `metric_trace` in flexure.rs: each step solves the same right-hand
side again in the other metric and prints, per band of latitude, the rms of the sphere's deflection
and the rms and largest difference) on the first of them (arm `ff-metric-trace2`, scattered 1,
stopped at the same step and cell) shows the growth: over the last three steps the rms of the
step's deflection over 80 to 90° went 0.94, 1.79 and 3.20 thousand km (1.8 to 1.9 times a step),
while under 50° it stayed at 230 to 340 m. One bad solve would not do that; the step itself is
unstable at the poles.

**Why it diverges.** The probe `probe_the_sphere_solve_where_the_rigidity_varies` (flexure.rs,
`--ignored --nocapture`) takes the raster of `earthlike_quick_16k` (1024 by 328 cells, its first
and last row past the poles), a load of ±50 m of mantle in noise, and cratons (`Te` 80 km) against
60 Myr ocean in blocks of 64 cells by 41 rows, and solves once to 0.01 m or 200 iterations:

| case | iterations | converged | largest residual, m of mantle (row) |
|---|---|---|---|
| block metric, rigidity by x and y | 29 | yes | 0.3 (124) |
| sphere's metric, uniform rigidity | 1 | yes | 0.0 |
| sphere's metric, rigidity changing from row to row only | 26 | yes | 0.1 (286) |
| sphere's metric, rigidity changing along the rows only | 200 | no (14.8 m) | 1,062 (327); 727 on row 0, 758 on row 1 |
| sphere's metric, rigidity by x and y | 200 | no (15.1 m) | 1,145 (0) |
| the same, the rows past the poles given the polar rows' couplings | 200 | no (8.8 m) | 401 (320); 395 on row 2 |
| the same, the couplings capped at their 60° values (a ≤ 4, b ≥ ¼) | 67 | yes | 0.4 (299) |

The solve fails where the rigidity changes along a row near a pole, and only there. The cause is
the preconditioner. Its bank solves the operator of a uniform plate exactly and blends those
answers cell by cell by the cell's own rigidity, which is right while a load's response stays on
plate of about the cell's rigidity; the damped Jacobi sweeps around it reach only error at the
cell scale. On the sphere's metric a row's east-west coupling is a = 1 / cos²φ, so the plate's
flexural length along the row, counted in cells, is √a times the equator's: the craton's 3.6
cells (α = 141 km) become 27 on row 2 (a = 55.5), 47 on row 1 (a = 167) and 65 on the row past
the pole (a = 326), the blocks' width, and the ocean's 1.3 become 10 to 24. A load's response
there spans plate 56 times stiffer or softer than its own cell, which the blend does not see.
Rigidity that changes from row to row does no harm, since the north-south coupling cos²φ is
small near the poles; capping the couplings at 60° brings the solve back (67 iterations, the
craton's flexural length 7 cells); giving the rows past the poles the polar rows' couplings moves
the worst residual one row towards the equator and cures nothing. The block metric never meets
it: its flexural length is the same in every row.

In the step, a solve's unfinished part waits as a load in the unsolved residual (ISO-059) for the
next solve. The operator at the polar rows is a² ≈ 3 × 10³ to 10⁵ times stiffer east to west than
at the equator, so a centimetre's error in the deflection there is kilometres of load. In the
probe's 24 steps (the default 60 iterations and 2 m; the ocean's age alternating between 60 and
15 Myr, so the weakening plate lets go, ISO-056) every solve on the sphere's metric stopped at 60
iterations unconverged (error 11 to 21 m), and the residual carried at rows 0, 1 and 327 stood at
1.5 to 8 km of mantle, against 5 to 37 m on the block metric. In those steps it did not grow, and
the ground at the poles moved as much as at the equator. What turns it into the runs' growth of
1.8 to 1.9 times a step was not isolated: the probe has no plate model, no surface processes, and
its yield never acted. The candidates are the yield (ISO-120), which acts on the deflection of an
unconverged solve, the load a weakening plate lets go (ISO-056), which counts the carried residual
as held, and the surface's answer to a pole that sinks. Each feeds on the unconverged solve, so a
solve that converges at the poles is the cure; the growth is its symptom.

## Foreland basins

Added 2026-10-24 (ISO-131, PLT-546), V1 gap 6 ([physics-gap-audit.md](physics-gap-audit.md),
"Gap 6"). The switch is `tectonics.collision_underthrust`, off: on the pool gap 6's form brought no
foredeep beyond the wedge ("On the pool", below), and gap 6b's brought one with twice the step
noise ("The superposition and the foreland's stiffness (gap 6b)").

**What Earth does.** A thrust belt is a load on the edge of the stiff plate that goes under it, and
that plate bends down into a foredeep in front of the belt, with a forebulge beyond: foredeeps are
100 to 300 km wide and hold 2 to 8 km of fill, forebulges 60 to 470 km wide (DeCelles & Giles
1996, p. 110), for flexural parameters of 26 to 150 km (rigidities 10²¹ to 10²⁴ N m, p. 111). The
plate under the Ganges basin and the Lesser Himalaya has a rigidity of about 0.7 × 10²⁵ N m, a
hundred times or more that under the Greater Himalaya, and Lyon-Caen & Molnar 1983 (abstract) fit
the gravity with the Indian plate bent under the mountains and the Ganga sediments by the range's
weight and a moment on its end. Its `Te` is 80 to 90 km, the Appalachians' 50 to 60 km (Watts &
Burov 2003, pp. 117 and 118); forelands are wide where the loads stand on old cratons and narrow on
young plates such as the Apennines and Taiwan (p. 127). Beaumont 1981 models the Alberta basin on
a plate continuous under the orogen, with D = 10²⁵ N m (his Fig. 9: `Te` about 83 km), and calls a
broken plate a "worst case" that would "increase the subsidence immediately beneath the fold-thrust
belt" (p. 327); Flemings & Jordan 1990 (the full text; their 1989 paper's abstract only) use an
unbroken plate of `Te` 25 km, Jordan 1981 (abstract) a rigidity of about 10²³ N m. In all of them
the basin's fill is a load too: the range's sediment fills the basin it bends and bends it deeper
(DeCelles & Giles 1996: topographic, sediment and subduction loads, p. 111), and Molnar & Lyon-Caen
1988 (abstract) find some ranges too light for their foredeeps.

**What the model did.** In the four runs of base5 (seeds 1 to 4) the plate check's foredeep in
front of belts above 3 km was 0 to 36 km wide and 17 to 134 m deep. Two causes:

1. The belt's load stood on the overriding plate, across the plate boundary's break (ISO-055) from
   the stiff incoming plate, so only the wedge (the incoming plate's own shortened edge) bent the
   foreland.
2. Belt and wedge are young orogenic crust: their `Te` is at the floor, 5 km (ISO-051 counts
   from the orogeny, ISO-055 floors the boundary's cells), so their load floated where it stood.

**The law** (PLT-546, ISO-131). In an active collision the incoming plate's lithosphere goes on
under the front. The plate model marks the footprint's continental samples that have it under them:
the wedge, and the belt as far behind the front as the collision has shortened (what crossed the
front went under it; the belt beyond is the overriding plate's own). Each takes the incoming
plate's id and the thickness of its foreland's lithosphere (the median over the foreland's
continental samples, else over the wedge's); where two collisions mark a sample, the faster one's.
L1 carries them as `plates/carrier_plate_id` and `plates/carrier_thermal_age_myr` (the thermal age
of that lithosphere, ISO-110 to ISO-112, which the `Te` law reads). The flexure then:

- **moves the break behind the belt.** For the plate boundary's break (ISO-055) a carried cell
  belongs to its carrying plate, so the break runs between the carried belt and the overriding
  plate behind it, and the suture itself is no break: the belt and the wedge stand on the end of
  the incoming plate, a broken plate loaded at its end (Lyon-Caen & Molnar 1983).
- **gives a carried cell both plates' stiffness.** On the belt's cells, the overriding plate's own
  lithosphere over the underthrust one, two layers that bend together without a bond:
  `Te = (Te_own³ + Te_carrier³)^⅓` (decoupled layers, Burov & Diament 1995, p. 3915; welded layers
  would add, their Eq. 15). On the wedge, the incoming plate's own crust on its own lithosphere, the
  larger of the two. The carrier's `Te` is the continental law (ISO-051) at its lithosphere's
  thermal age, not floored as orogenic crust: the plate under the belt is not reheated by the
  shortening above it.
- **keeps the foreland in the belt's frame** (ISO-133; since gap 6b only the belt's own part of
  the deflection, "The superposition and the foreland's stiffness (gap 6b)" below; what follows is
  gap 6's column swap, which that replaced). The flexure carries the plate's state with
  its crust and takes it to be in balance ("Increments, not the whole load", in the code's module
  notes), which holds where the loads move with the plate. Beside a belt they do not: the belt
  moves with the overriding plate, and the incoming plate it bends slides under it. Carried with
  its crust, the foreland's deflection went into the front, so the basin held only what the last
  few steps' loads had bent, and crust from the forebulge arrived beside the front. Now, on the
  incoming plate's crust within one flexural wavelength of the belt (`2πα` of the stiffest plate,
  885 km), the state is the place's a step ago in the belt's frame: the place keeps its deflection
  and the load the plate held. The crust that slides through keeps its own load and the roots
  local isostasy put under it, and its load over what the place answered is solved with the
  step's loads. A plate sliding under a load that stands still then bends as one at rest, as an
  elastic plate must (the deflection is set by the load and the plate, not by the plate's past),
  and the foreland's crust subsides as it nears the front (Beaumont 1981; Flemings & Jordan
  1989). Two other forms were tried and dropped: carrying the plate's bending force with the crust
  diverged on every pool run (the rigidity jumps of the floor bands), and the footprint carried
  with the incoming plate moved the break behind the belt through the plate and the belts' local
  roots with it (14 to 56 km a step).

The basin's fill needed nothing new: what the surface stage lays in the basin is a load as before
(ISO-061 to ISO-064), and with the stiff plate under the belt it bends the foreland, not only the
cell it lies on.

**Unit tests.**

- `a_line_load_on_a_broken_plate_gives_the_analytic_profile`: a line load on the end of a plate
  whose `Te` drops from 30 to 0.5 km at the load. The broken plate bends as
  `w = w₀ e^(−x/α) cos(x/α)` with `w₀ = 2V / (ρ_m g α) = V α³ / 2D` when the whole load is on its
  end (the textbooks' `V α³ / 4D`, Angevine et al. 1990 eq. 5.8, shares `V` between the two broken
  ends). The end's depth is within 6 % of it on 2.5 km cells and converges at first order (the
  error halves with the cell), the first zero within 1.5 % of `πα/2` (the continuous plate's is at
  `3πα/4`), the bulge within two cells of `3πα/4` at −6.7 % of the depth (within 0.2 %).
- `a_belt_carried_by_the_incoming_plate_bends_a_wide_foreland_basin`: a belt on the overriding
  plate, its wedge and a foreland of `Te` 80 km. Carried, the basin is 2.07 times as deep; its edge
  is between `πα/2` and `πα` of the front either way, in the same place within a row (the
  incoming plate's `α` sets the width); the overriding plate behind the belt is left alone.
- `a_foreland_sliding_under_its_belt_bends_as_one_at_rest`: the belt stands still and the
  foreland slides under it at 0.5 and 1.5 rows a step (10 and 30 km/Myr) for 20 steps of a belt
  rising by up to 200 m a step. The flexure and the elevation are those of the case at rest to
  the metre over the belt and the basin (measured the same after 10, 40 and 80 steps), and
  within 0.5 % of the basin's depth beyond the reach. Carried with the crust, at 10 km/Myr, the
  basin by the front was 2.5 km deep against 10.3 and ended about 80 km nearer the front, with
  the forebulge's crust beside it.
- `a_belts_foreland_is_the_carrying_plates_crust_within_reach`: the reach takes the carrying
  plate's crust and the wedge within its cells of a belt, across the wrap, and leaves the belt,
  the overriding plate behind it and a third plate's crust out.
- `the_footprint_carries_the_incoming_plates_lithosphere` (plates): the wedge and the belt within
  the shortening of the front carry the incoming plate and its foreland's lithosphere; the belt
  beyond the shortening, the foreland and the overriding plate carry nothing; off, nothing does.

**On the pool** (2026-10-24; arms `g6-frame1` and `g6-frame2`: both starts x seeds 1 to 4,
1,000 Myr, the switch on; the first on master 1b56529d against base7's 24 runs and its same 8,
`g6-base7-s1to4`, the second on master 4dd85fe2, the outline cover, against base8's same 8,
`g6-base8-s1to4`, which are the front lane's `fl-outline`; Mann-Whitney p against base7's 24,
then against the same 8). Nothing diverged and no step passed one Earth relief (19.8 km) in any
of the 16 runs, but the foreland basin did not come:

| | base7, 24 runs | base7, the same 8 | switch on, 8 | base8, the same 8 | switch on, 8 |
|---|---|---|---|---|---|
| land above 2 km | 0.141 | 0.174 | 0.107 (p 0.12; 0.005) | 0.118 | 0.102 (p 0.083) |
| coast with high ground | 0.266 | 0.282 | 0.261 (p 0.95; 0.28) | 0.254 | 0.260 (p 0.88) |
| criteria met per run, mean | 5.04 | 4.88 | 5.12 | 5.12 | 5.00 |
| record: belts above 3 km with a foredeep | | 0.42 | 0.21 | 0.36 | 0.18 |
| record: foredeep width, median, km | | 18 | 0 (p 0.20) | 0 | 0 (p 0.076) |
| the guard's worst, km from the sea | | 10.2 | 13.9 (p < 0.001) | 10.3 | 15.0 (p < 0.001) |
| largest rise / fall of a cell in a step, km | | 7.8 / 7.3 | 15.4 / 12.4 | 7.9 / 7.4 | 14.2 / 13.3 |

In the final frames the flexure just beyond the footprint's toe stands above local isostasy at
the median (+100 to +200 m in the first 40 km: the forebulge), with the switch on, off, and with
ISO-131 alone (the earlier arm `gap6-ut1`), so the measure finds no foredeep beyond the toe in any
of them. Two causes, found on the runs (`build/gap6/profile1.py` over the final frames, and a
traced run of scattered continents seed 2):

1. **The foreland is not stiff enough for its wedge.** Near the front `Te` is 20 to 35 km (the
   footprint's cells within 40 km of the toe and the foreland's first 40 km, three runs), so α is
   50 to 76 km and the broken plate's deflection crosses zero (`πα/2`) 78 to 119 km from the
   load, about the wedge's width (96 km at the median in the record). The basin lies under the
   wedge and the toe stands near the forebulge. Earth's wide foredeeps lie on plates of `Te` 50
   to 90 km (the Ganges 80 to 90, the Appalachians 50 to 60, Watts & Burov 2003, pp. 117 and 118).
2. **The frame's split of the deflection does not hold on the runs.** The local roots a crust
   carries (`FlexState::local_m`: the plate model's root less the part the plate bends, summed
   step by step as the crust moves over the grid) wander at the cell scale: the cell whose frame
   change was the largest carried −15 km of them under 31 km of load. The frame swaps a place's
   local part for its crust's, so a coast or a thick block of the incoming plate that moves
   through the belt's frame brings its whole column into the unsolved residual each step, and
   the solve gives most of it back: frame changes up to 46 km in a step, net changes up to 12 km.
   With a source's corners on the bending plate's own crust only (not the belt's footprint) the
   frame's largest change halved (18 to 24 km), but over the whole run (scattered continents
   seed 2) the largest rise of a cell in a step was 12.4 km, against 15.9 with the switch as it
   is and 8.0 on base7, and the guard's worst 12.6 km against 16.2 and 10.3.

So the switch stays off, and the fall in land above 2 km is not taken as its effect: it comes
with the frame's noise, not with a basin. What would bring the basin: the belt's own deflection
kept apart (the plate is linear: the deflection is the sum of that under the belt's loads, which
moves with the belt, and that under the rest, which moves with the crust; the belt's increments
solved apart on steps with a collision and that part carried in the belt's frame would swap a
field as smooth as α, with no load to solve again), and a foreland as stiff as its lithosphere's
age makes it (the carrier's `Te` from its foreland's thermal age, ISO-051 and PLT-546).

### The superposition and the foreland's stiffness (gap 6b)

Added 2026-10-24 (ISO-136 to ISO-138), both of what the paragraph above asked for. The
superposition replaced gap 6's column swap behind `tectonics.collision_underthrust`; the stiffness
is `surface.flexure.te_geotherm`. The stiffness is on by default since 2026-10-25: on the pool it
neither diverged nor moved the guard or the steps. The superposition stays off: with it the
foredeep came, but the largest change of a cell in a step doubled and the guard's worst rose by 2
to 5 km, and the traces show why ("On the pool", below).

**The superposition** (ISO-136). The thin plate is linear: with the step's rigidity and restoring
density, the deflection under two loads is the sum of the deflections under each. So beside the
deflection it carries with the crust the flexure keeps the belt's part of it (`FlexState::belt_m`),
the deflection the belt's own increments bent, carried with the crust like the rest of the state.
On a solve step the belt's increments are solved apart with the step's operator: on its footprint
cells only (the over plate's crust that a collision's incoming plate carries; the wedge is the
incoming plate's own crust and goes with it), its new loads and roots, the residual it carried, the
bending of its plate root and the load it let go. The step's own solve is the whole one, as
before, so the step's change of the deflection is unchanged; the belt's is one more solve on
collision steps, never a load solved again. Over the belt's foreland (ISO-133's cells) a place then
takes the belt's part where the place was a step ago in the belt's frame, and the crust that slid
there keeps the rest of its own. What moves is a field as smooth as α, not a column's whole state:
the local roots, the load and the ice stay with their crust, and nothing goes into the unsolved
residual. Under the sea the water the move lets in or out is booked as answered, since the belt's
solve took that water's restoring density; without that booking the flexure of a foreland under the
sea moved 3.4 m off the case at rest already behind the belt (the bound is 1 m). Crust new at a
place takes the belt's part where it is and moves nothing.

Four rules came from the first arm's traces and the singles after it (g6b-both2, "On the pool"
below):

- **The yield is the belt's too.** Where the belt bends its foreland past what the plate holds
  (ISO-120), the plate yields there, and the belt's part yields with the whole: off the belt it
  holds `−ρ_m × belt_m`, and where that pushes the same way as the whole it gives up its share of
  the sinking, all of it where the rest pushes the other way. Left with the rest, the belt's part
  over a foreland kept the elastic answer, 18 to 38 km on the traced run where the plate holds about
  2.75 km of mantle, and the frame moved that over the crust each step for the yield to take back.
- **Crust coming into a foreland moves nothing.** The flexure names, for each cell, the belt whose
  frame it is in or that it stood under a belt (`FlexState::belt_of`, carried with the crust). Crust
  that comes into a belt's foreland from under a belt or from another belt's foreland carries a belt
  part that is not this belt's far field, so it takes this belt's part where it is, without a move.
  Crust from beyond any foreland moves as before: it carries the belt's far field, and the sliding
  tests need that move (blocking it put the crust that had just entered 1.1 to 1.5 m off the case at
  rest).
- **A foreland ends where another belt is nearer.** With the reach of 1,180 km, belts' forelands
  overlapped and a place's frame could change from one step to the next. A cell nearer, measured
  over any crust, to another belt than to its own is that belt's to bend, and is in no other's
  frame.
- **A foreland takes the belt's part from its own crust only.** The place a step ago in the belt's
  frame counts where it was the bending plate's own crust, not where it was the belt the plate
  carried: there the belt's part is the belt's root under its own load. Counted, a cell by the
  front took 16 to 28 km from such corners against the few hundred metres of the foreland's crust
  beside them, and the yield took most of it back in the same step.

**The foreland's stiffness** (ISO-137, ISO-138). A continental plate's strength is in two layers:
the upper crust, strong down to about 300 °C (quartz), and the mantle under the Moho, strong down to
about 750 °C (olivine) (Burov & Diament 1995, Figure 2's caption). Between them the weak lower
crust lets them bend without a bond, so `Te = (h₁³ + (h₂ − h_c)³)^⅓` (their eq. 14, after McNutt
et al. 1988; Watts & Burov 2003's eq. 3, the leaf spring), with h₁ the depth of the 300 °C
isotherm or the Moho if that is shallower, h₂ that of the 750 °C isotherm and h_c the crust. For
crust thicker than 20 to 25 km "decoupling should be permanent" (p. 3909), and the decoupled model
"fits most of the continental data" (Figure 7b), so the model is always decoupled. The isotherms
are those of the plate model's own steady geotherm: its lithosphere's depth L from the thermal age
(PLT-460, ISO-112), its heat flow (PLT-467: the crust's 26 mW/m² from sources falling off over
h_r = 10 km, the mantle's B / L), and T_m = 1,350 °C at L (both from B&D's appendix, ISO-138). The
conductivity that implies is 2.69 W/m/K, within B&D's 2 to 3.5. Against ISO-051's recovery law
at the same thermal age (τ_c 203.5 Myr from L_hot to L_cold):

| thermal age, Myr | L, km | 300 °C, km | 750 °C, km | `Te`, 35 km crust | 40 km | 50 km | ISO-051 |
|---|---|---|---|---|---|---|---|
| 0 | 60 | 11.2 | 31.5 | 11.2 | 11.2 | 11.2 | 5 (floor) |
| 50 | 95.9 | 16.9 | 50.1 | 20.2 | 18.0 | 16.9 | 12.3 |
| 100 | 124.1 | 21.3 | 64.7 | 33.0 | 29.1 | 23.4 | 22.7 |
| 142 | 142.9 | 24.2 | 74.5 | 42.3 | 38.1 | 30.7 | 30.2 |
| 250 | 176.7 | 29.4 | 92.1 | 59.6 | 55.1 | 46.4 | 45.2 |
| 600 | 216.4 | 35.6 | 112.8 | 80.1 | 75.5 | 66.4 | 69.2 |
| a craton | 225 | 37.0 | 117.3 | 84.4 | 80.0 | 70.8 | 80 |

An old craton under 40 to 50 km of crust gets 71 to 80 km, at the top of B&D's "about 60-75 km" for
old decoupled lithosphere under 35 to 50 km of crust (p. 3916) and of Watts & Burov 2003's 80 to 90
km under the Ganges (p. 117). The law is stiffer than ISO-051 where the crust is thin or the
lithosphere young (a 100 Myr foreland under 35 km of crust: 33 km against 23, α 82 km against 64),
and the same under 50 km. A belt's footprint is its own crust over the colder of its own and the
carrier's lithosphere (a craton under 60 km of belt: 62 km), never under its own `Te`; an oceanic
footprint keeps ISO-131's sum. The belt's reach is now 2πα of a craton's mantle with no crust, `Te`
117.3 km: 1,180 km. The brief's suggestion, the depth of the 450 to 600 °C isotherm, is the
oceanic relation (Watts 1978, as B&D cite it, p. 3906); a continent's `Te` is not one isotherm's
depth (B&D's point).

**Unit tests.**

- `a_belts_increments_solved_apart_add_up_to_the_whole_solve`: on a 192 by 96 plate with patches
  of `Te` 6 to 70 km (a rigidity varying more than a thousandfold) and the sea over a third of it,
  a belt's loads and the rest's solved apart add up to the whole solve within 10⁻³ of the peak
  deflection (over 100 m), and the belt's part reaches beyond the belt (over 1 m within 14 rows).
- `a_belts_part_of_the_deflection_is_what_its_own_loads_bend`: a belt rising on its footprint and
  a fill on its foreland, in one step; the belt's part the step keeps is the deflection the belt's
  uplift alone bends (to 5 cm), the fill alone adds none to it, the two steps' deflections add up
  to the step with both (to 10 cm), and over the foreland the belt's part is the belt's whole
  deflection.
- `a_foreland_sliding_under_its_belt_bends_as_one_at_rest` (gap 6's, now on the superposition) and
  `a_foreland_under_the_sea_sliding_under_its_belt_bends_as_one_at_rest`: the belt stands still and
  the foreland slides under it at 10 and 30 km/Myr; the flexure and the elevation are those of the
  case at rest to the metre within the reach, on land and under the sea.
- `where_a_belt_bends_its_foreland_past_its_strength_the_yield_is_the_belts`: a belt rising 3 mm/yr
  bends its foreland past the yield (380 mGal); the foreland yields, and there the belt's part is
  still the whole deflection, to 5 cm.
- `crust_coming_into_a_belts_frame_moves_nothing`: a 3 km deflection on crust entering a foreland
  moves with the frame when the crust came from beyond any foreland or was in this belt's frame
  already, and does not when it stood under a belt or in another belt's frame; the frame's cells
  are named for the belt and the crust under the belt is marked.
- `a_belts_foreland_ends_where_another_belt_is_nearer`: two belts on three plates; each foreland is
  its carrier's crust within reach and nearer to its own belt than to the other.
- `a_place_that_held_the_belt_gives_its_foreland_no_belt_part`: a belt's root of 16 km beside a
  foreland of 1 km; a source between them takes the foreland's 1 km (8.5 km with the belt's
  corners), a source on the belt alone none.
- `the_continental_te_follows_its_lithospheres_geotherm`: the isotherms against the geotherm's
  closed form, the table's values (84.4, 80.0, 70.8, 42.4, 21.9 and 11.2 km), `Te` rising with the
  thermal age and falling with the crust, and the belt's reach.

**On the pool** (2026-10-24). The cumulative arm `g6b-both2`, both switches on, both starts x seeds
1 to 4, 1,000 Myr, on master 19fa4077 (the superposition as first built, before the four rules),
against base9's same 8 (`fl-pull`) and base9's 24 (Mann-Whitney p against the same 8, then the 24):

| | base9, 24 | base9, the same 8 | both on, 8 |
|---|---|---|---|
| land | 0.296 | 0.288 | 0.316 (p 0.10; 0.22) |
| land above 2 km | 0.114 | 0.086 | 0.141 (p 0.050; 0.10) |
| coast with high ground | 0.249 | 0.198 | 0.319 (p < 0.001; < 0.001) |
| continental crust under the sea | 0.249 | 0.318 | 0.283 (p 0.38; 0.88) |
| land under ice | 0.113 | 0.083 | 0.122 (p 0.10; 0.48) |
| collisions per 100 Myr | 1.90 | 1.95 | 2.20 (p 0.67; 0.16) |
| criteria met per run, median and mean | 5, 5.08 | 5, 5.12 | 5, 5.12 |
| the guard's worst, km from the sea | 10.2 | 10.1 | 15.3 (p < 0.001; < 0.001) |
| largest rise / fall of a cell in a step, km | 7.3 / 7.2 | 7.2 / 7.4 | 14.4 / 11.5 |
| record: belts above 3 km / share with a foredeep | 9 / 0.41 | 7 / 0.38 | 8 / 0.71 |
| record: foredeep width, km (plate check 150 to 450) | 0 | 0 | 179 (p 0.001; < 0.001) |
| record: foredeep flexure below local isostasy, m | 76 | 80 | 452 (p < 0.001; < 0.001) |
| record: wedge width, km | 96 | 90 | 92 (p 0.96; 0.36) |
| final frame: foredeep width to the zero crossing, all / belts above 3 km, km | | | 149 / 155 |
| final frame: `Te`, footprint / foreland, km | | | 16.2 / 33.4 |

No run diverged and no step passed two Earth reliefs. The foredeep came: beyond the wedge, 179 km
wide at the median in the records, inside the plate check's 150 to 450 km, with 452 m of flexure
below local isostasy where there were 80. But a cell's largest change in a step doubled, and the
guard's worst rose by 5 km. So the arm meets the first of the brief's three conditions and fails
the second.

**The traces.** A run with a trace of every flexure step (`build/gap6b/trace2.py`, scattered
continents seed 4, the arm's noisiest) put 321 of the 334 rises over 6 km in a step and 215 of the
222 falls on cells in a belt's frame. The four rules above came from there; each took away a path
the traces showed. With all four, the singles on that run against the base's same run:

| scattered continents seed 4 | the guard's worst, km | largest rise / fall, km (beyond the polar rows) | polar rows, km | cells that yielded |
|---|---|---|---|---|
| base9 (fl-pull's run) | 10.6 | 6.7 / 7.3 | | |
| as first built, the superposition alone | 12.1 | 11.8 / 10.5 | | 866,123 |
| the first three rules, the superposition alone | 12.7 | 11.2 / 10.7 | 10.7 / 8.8 | 564,721 |
| the first three rules, both on | 14.4 | 10.9 / 10.1 | 10.9 / 7.7 | 853,460 |
| as first built, both on | 15.6 | 16.6 / 12.1 | | 1,197,045 |
| `te_geotherm` alone | 10.5 | 6.7 / 6.6 | | 10,036 |
| base10 (master 2dd50093) | 10.1 | 6.9 / 7.2 | 6.2 / 6.5 | 6,096 |
| the first three rules, the superposition alone, traced | 16.1 | 14.5 / 11.3 | 9.8 / 10.3 | 588,000 |
| the four rules, the superposition alone, traced | 12.7 | 12.0 / 8.4 | 10.2 / 7.1 | 728,000 |
| the first three rules, both on, traced | 12.4 | 9.5 / 8.6 | 10.6 / 7.2 | 631,000 |

The first five rows ran on master 19fa4077 (base9's planet), the rest on master 2dd50093
(base10's). With both on, the first three rules took the largest rise from 16.6 to 10.9 km and the
guard's worst from 15.6 to 14.4 km; the superposition alone hardly moved with them (12.1 to 12.7 km,
11.8 to 11.2 km). The fourth took the superposition alone on base10's planet from 16.1 to 12.7 km
and 14.5 to 12.0 km. None came near the base's 10 and 7 km. On the traced run
with the first three rules, the frame's move was half the size of the rises over 6 km and 1.34
times that of the falls; the yield took back 0.53 of the falls. The largest frame moves of a step
came three ways: from source corners on the belt (bf 27.8 km against the 2.1 km the crust
carried; the fourth rule ends this), from crust whose mark was lost (it came from beyond any
foreland, NaN, and carried 26.8 km of the belt's part against 5 m at the place; the crust's carry
and the belt's part's take their sources apart, and a cell can come from the belt for one and
from beyond for the other), and inside the frame (26.6 km carried onto 168 m). With the fourth
rule the last two remain: 22 km carried onto a place with none, and jumps of 21.6 km inside the
frame.

**Why it does not work.** The superposition moves a field that should be as smooth as α. On the
runs it is not. Where a belt stands, its `Te` is low: 5 to 15 km on the traced cells, 16.2 km at
the median over the footprints of the arm's final frames (orogenic crust; the foreland's is 33.4
km). With E 70 GPa (ISO-041) that is α of 18 to 42 km, about one cell of the quick preset's 39 km.
The belt's part there is the belt's own root, 30 to 49 km at the largest of a step (35 km at the
median over the traced steps), and it falls to the foreland's few hundred metres within a cell or
two. And the two parts are not each small: at a traced cell the belt's part was 22 km and the rest
−19.7 km. The split books an increment to the belt by where it fell at the time; a cell that
leaves the footprint keeps the part it gathered, and what the plate model, the surface and the
yield take off it later is the rest's. So the parts can drift into large and opposite fields with
the cell-scale shape of the loads (the traces measured the parts, not their history: that the
drift comes this way is the agent's reading). Any change of the frame's geometry (a
front that moves, a cell that enters or leaves a foreland, a mark the carry loses) then moves a
cell-scale step of 20 to 27 km over the crust, and the yield, which holds about 2.75 km of mantle,
takes most of it back in the same step. That is the step noise.

What would work, not tried: keep the belt's part smooth by construction, the deflection under the
belt's present load and not the sum of past increments. That is a solve of the belt's whole load
each collision step, which the brief rules out ("Nothing is solved again"), and its cost is one
more full solve. A cut-off on the belt's part or a smoothing of it would be a forcing term, so
neither was tried.

**The foreland's stiffness alone** (arm `g6b-te3`, 2026-10-25: `te_geotherm` on, the superposition
off, both starts x seeds 1 to 4, 1,000 Myr, on master 2dd50093, against base10's same 8, `fl-ends`,
and base10's 24; Mann-Whitney p against the same 8, then the 24). It is the per-switch arm that
says which part of `g6b-both2`'s noise is whose:

| | base10, 24 | base10, the same 8 | `te_geotherm` on, 8 |
|---|---|---|---|
| land | 0.294 | 0.294 | 0.299 (p 0.87; 1.0) |
| land above 2 km | 0.112 | 0.112 | 0.137 (p 0.28; 0.15) |
| coast with high ground | 0.226 | 0.233 | 0.250 (p 0.65; 0.24) |
| continental crust under the sea | 0.246 | 0.205 | 0.212 (p 0.96; 0.69) |
| land under ice | 0.095 | 0.095 | 0.119 (p 0.96; 0.59) |
| collisions per 100 Myr | 1.80 | 1.65 | 1.55 (p 1.0; 0.73) |
| criteria met per run, median and mean | 5, 5.00 | 5, 5.00 | 5, 5.12 |
| the guard's worst, km from the sea | 10.1 | 10.2 | 10.1 (p 0.96; 0.98) |
| largest rise / fall of a cell in a step, km | 7.7 / 7.2 | 7.4 / 7.2 | 7.5 / 7.2 |
| record: belts above 3 km / share with a foredeep | 7 / 0.37 | 6 / 0.45 | 7.5 / 0.38 |
| record: foredeep width, km | 0 | 36 | 0 (p 0.18; 0.68) |
| final frame: `Te`, footprint / foreland, km | | | 11.4 / 34.7 |

No run diverged, no step passed two Earth reliefs, and the guard and the largest steps are the
base's. No V1 number moved beyond the seeds' spread. The noise of `g6b-both2` is the
superposition's. Alone, the stiffness brings no foredeep: with the superposition off the belt is not
carried (ISO-131 is under `tectonics.collision_underthrust`), so it stands on the overriding plate
across the break, and the foreland's stiffness does not reach it.

**The four rules, both on** (arm `g6b-both3`, 2026-10-25: both switches on, the superposition with
the four rules, both starts x seeds 1 to 4, 1,000 Myr, on master 2dd50093, against base10's same 8,
`fl-ends`, and base10's 24; Mann-Whitney p against the same 8, then the 24):

| | base10, 24 | base10, the same 8 | both on, the four rules, 8 |
|---|---|---|---|
| land | 0.294 | 0.294 | 0.313 (p 0.038; 0.037) |
| land above 2 km | 0.112 | 0.112 | 0.147 (p 0.13; 0.026) |
| coast with high ground | 0.226 | 0.233 | 0.274 (p 0.050; 0.007) |
| continental crust under the sea | 0.246 | 0.205 | 0.183 (p 0.33; 0.10) |
| land under ice | 0.095 | 0.095 | 0.115 (p 0.51; 0.45) |
| collisions per 100 Myr | 1.80 | 1.65 | 1.80 (p 0.63; 0.91) |
| criteria met per run, median and mean | 5, 5.00 | 5, 5.00 | 5, 5.00 |
| the guard's worst, km from the sea | 10.1 | 10.2 | 12.1 (p < 0.001; < 0.001) |
| largest rise / fall of a cell in a step, km | 7.7 / 7.2 | 7.4 / 7.2 | 10.3 / 9.0 |
| plate cells' largest excess over Airy, km | 2.6 | 2.3 | 3.8 (p 0.002; 0.003) |
| record: belts above 3 km / share with a foredeep | 7 / 0.37 | 6 / 0.45 | 7 / 0.67 |
| record: foredeep width, km (plate check 150 to 450) | 0 | 36 | 143 (p 0.037; < 0.001) |
| record: foredeep flexure below local isostasy, m | 75 | 120 | 291 (p 0.038; 0.001) |
| record: wedge width, km | 99 | 117 | 95 (p 0.33; 0.38) |
| final frame: foredeep width to the zero crossing, all / belts above 3 km, km | | | 0 / 0 |
| final frame: `Te`, footprint / foreland, km | | | 16.2 / 33.3 |

No run diverged and no step passed two Earth reliefs. The foredeep stays, beyond the wedge: 143 km
wide at the median in the records, just short of the plate check's 150 km, with 291 m of flexure
below local isostasy where the base had 75 to 120, and two thirds of the belts above 3 km with
one. The noise stays too. The guard's worst is 12.1 km against the base's 10.1 (p < 0.001), a
cell's largest rise in a step 10.3 km against 7.4 to 7.7 and its largest fall 9.0 km against 7.2,
and the plate cells' largest excess over Airy 3.8 km against 2.3 to 2.6. Against `g6b-both2`'s
excess over its own base (5.2 km of guard, 7.2 km of rise) the four rules took away about 60 per
cent (1.9 to 2.0 km, 2.6 to 2.9 km), on a different base (base9 against base10), so that is a
rough reading. The land, the land above 2 km and the coast with high ground rose (p 0.007 to 0.05),
the last two away from their targets. In the final frames, at 1,000 Myr, the median run held no
foredeep (`g6b-both2`'s 149 km); that was not traced. So the arm meets the brief's first and third
conditions and fails the second again: the superposition stays off, for the reasons above.

**Simplifications.**

- The carried lithosphere lasts as long as the collision: when the front stops converging the
  carrier goes, the belt's `Te` falls back to its own and the plate lets go of what the carrier held
  (ISO-056, at ISO-099's pace). On Earth the underthrust plate stays under the belt (India under
  southern Tibet).
- Its reach is the collision's shortening, not a slab's geometry: the dip and the end of the
  underthrust plate are not followed, and a long collision carries the whole belt.
- One lithosphere thickness per front (its foreland's median).
- Behind a carried belt the break separates the belt from the overriding plate, so no retro-foreland
  basin forms there (the Himalaya's case; the Andes' retroarc basins are a subduction setting).

## Game choices

- **Flexural lengths are Earth-normalized** (ISO-057): the plate's kilometres are the plate
  model's, not the map's. A basin is then as wide against its belt as on Earth, and a cone drawn
  in real metres is, to the plate, eight times wider than it looks. Chosen by the agent (build log
  2026-10-02); a question for Ben.
- **On by default in the four built-in presets** (ISO-065): Ben, decisions log 2026-10-02,
  "General flexure everywhere". The hypsometry was not recalibrated.
- **The ice load is the ice at the step's end** (ISO-064). A question for Ben.

## Open questions

- The plate model's one-step uplift spikes (up to 33 km in a step over hundreds of cells) and the
  tens of km the flexure then holds up for a step: traced on 2026-10-18 to a resampling's
  accretion (up to 36 km of rise and 566 km of crust in one step) and the collision belts (13 km),
  and bounded by the rate limits of [plates.md](plates.md), "The fastest a column thickens and
  rises" (PLT-287, PLT-288): no plate step now raises a column more than 7 km. The guard's worst
  over six 500 Myr runs of `earthlike_quick_16k` fell from 66.7 to 30.8 km, over six 1000 Myr runs from 82.3 to 32.1 km (ISO-073). What was
  left was the surface stage's own: a polar row of L1 that rose 20 to 40 km over a few steps under
  plate uplift of 2.2 km or less, and a hole two cells wide on a row near the pole that fell to
  59.6 km below the sea in 16 Myr while the plates raised it. Traced on 2026-10-18 to the flexure's
  rule for a plate that weakens and fixed ("The polar holes"): the guard's worst over the same six
  500 Myr runs is now 15.5 km [13.2, 18.1].
- Single cells still change by 5 to 10 km in a step (2.5 to 5 mm/yr), by the fluvial step, marine
  fill, the glacial step and the flexure: within the fastest rates measured on Earth, above what a
  region sustains over 2 Myr ("The polar holes", "What is left"). Not checked process by process.
- A plate that weakens lets go of what it held as its `Te` falls (ISO-056), and since 2026-10-23
  the law's `Te` falls over the lithosphere's thermal time constant (ISO-099, "The release's
  pace"), not in the step. That a plate's `Te` follows the law's value as a single exponential at
  McKenzie's 62.8 Myr is the agent's simplification of a warming geotherm (a real one warms from
  below first, and its `Te` would fall slowly, then faster); no source gives the curve. The
  viscous relaxation once the plate is warm takes about a million years (Beaumont 1978,
  abstract); Watts & Zhong 2000 (abstract) find oceanic plates elastic on long timescales, which
  is the stiffening side. That the share let go is `1 − r` of the load held is the agent's rule,
  from the two limits; no source states it. The break at a plate boundary (ISO-055) still lets go
  at once.
- The solve's block metric ("Simplifications and limits"): the solve on the sphere's metric
  (ISO-127) is built and right for a single solve on a uniform plate, but its step diverges at the
  poles ("The solve on the sphere's metric", "Why it diverges"), so it is off. Its preconditioner
  does not converge where the rigidity changes along a row near a pole. The recommendation is a
  line solve for those rows: on every row where the east-west coupling passes a few times the
  equator's (a > 4, past 60°), an exact solve of the row's own operator along x with its own
  rigidity (a cyclic five-band system per row) in place of the point Jacobi sweeps, around the
  same bank. Rows there are coupled mostly along themselves, so the row solve holds the part the
  bank misses. Line relaxation along the strongly coupled direction is the usual cure for an
  operator this anisotropic (from memory, no source read). The probe's capped case says the bank
  alone is enough below 60°. Then the probe, the strip test and one 8-run arm; it goes on by
  default if that arm neither diverges nor fails the guard. Capping the couplings at 60° also
  converges, but it is the block metric's error again past 60°, so it is not recommended.
- The yield's bound (ISO-120) is on the load a cell holds, not on the plate's stress or curvature;
  whether a curvature form (Watts & Burov 2003) gives different planets was not tried.
- The yield's value: 380 mGal (the Puerto Rico Trench, where the slab's pull holds part of the
  anomaly) or 300 mGal (Hawaii, a load the plate's strength alone holds), "The plate's yield". The
  recommendation is to keep 380 until a 300 mGal arm is run on the next base; the bound is passed
  on about 27 cells a step.
- Water by basin (ISO-126) takes the lakes' levels from the step's last flood, before the flexure
  moves the ground, and one level per flooded depression; whether a basin the flexure deepens in
  the step should take its water again was not measured.
- The recovery law of continental `Te` (ISO-051) has no source for its form or its 300 Myr.
- Densities (ISO-043 to ISO-047) are uncited, and the model uses three different crust densities
  elsewhere (see [isostasy-and-height-limit.md](isostasy-and-height-limit.md)).
- Whether transform boundaries should be breaks (ISO-055).
- The cap of oceanic `Te` at 40 km (ISO-049) is one measurement at Hawaii that its authors call
  higher than expected.
- Why the hypsometry moves as it does (deeper sea, narrower shelves, less land above 3 km) was
  not isolated by experiment.

## Sources

By their keys in [bibliography.md](bibliography.md), section "Lithospheric flexure":

- `TurcotteSchubert_Geodynamics`: chapter 3 as a PDF (constants), lecture notes on it (formulas).
- `Calmant1990`: full text.
- `Watts1978`, `WattsTenBrink1989`, `Wessel1993`, `BurovDiament1995`, `PerezGussinyeWatts2005`,
  `Lu2024`, `KarnerWatts1983`, `LyonCaenMolnar1983`, `WieczerkowskiMitrovicaWolf1999`: abstracts.
- `BrotchieSilvester1969`: not read; its formula as quoted in `Wickert2016` (full text).
- `DeCellesGiles1996_flexure`: full text.
- `StewartJamieson2019`, `Dangol2020`: full text, read 2026-10-18 by the agent that wrote the
  guard (ISO-071).
- `EshaghTenzer2021` (Summary and Section 2.2), `Buoninfante2023`: full text; `Beaumont1978`,
  `WattsZhong2000`: abstracts; read 2026-10-18 by a sub-agent of the polar-holes agent (ISO-056).
- `Guevara2022`, `Govin2020`, `Lidberg2010` (author preprint), `Brothers2009`, `Garzione2008`,
  `XieHeller2009`, `Portenga2011`: full text; `Deng2021`, `Sella2007`: abstracts; fetched
  2026-10-18 by a sub-agent of the polar-holes agent, the values read by the agent (ISO-076).
- `Watts2025_Hawaii`, `tenBrink2004`, `Stern_OceanTrenches` (proof), `WattsBurov2003`: full text;
  `HunterWatts2016` (about 200,000 of 279,000 characters, section 8 and Appendix C), `Cattin2001`
  (about 100,000 of 130,000 characters): most of the text; `McNuttMenard1982`, `GoetzeEvans1979`:
  abstracts; `Watts1978`, `KarnerWatts1983` and chapter 3 of `TurcotteSchubert_Geodynamics`: full
  text this time. Read 2026-10-22 by a Sonnet sub-agent of the flexure-guard agent, the
  quotations in the bibliography (ISO-093). The yield (ISO-120) reads the same entries.
- `CODATA2022_G`: NIST's value page for G, read 2026-10-23 by the polar-peaks agent (ISO-120).
- `Snyder1987` (chapter 10, chapter 4), `Hetenyi1946` (chapter 1): full text, read 2026-10-23 by a
  Sonnet sub-agent of the flexure-faults agent, the equations checked on page images (ISO-127).
- `Beaumont1981`, `DeCellesGiles1996_flexure`, `BurovDiament1995`, `WattsBurov2003`,
  `FlemingsJordan1990`, `Angevine1990` (chapter 5): full text; `Jordan1981`, `FlemingsJordan1989`,
  `LyonCaenMolnar1983`, `MolnarLyonCaen1988`: abstracts. Read 2026-10-24 by a Sonnet sub-agent of
  the gap 6 agent (ISO-131, PLT-546); the broken plate's end depth (Angevine et al. eq. 5.8) checked
  by the agent. Watts 2001 (the book) was not obtained; Watts & Burov 2003 stands in for its
  forelands' `Te`.

The sources from `TurcotteSchubert_Geodynamics` to `DeCellesGiles1996_flexure` were read by a sub-agent on 2026-10-02, not by the agent that wrote the code.
