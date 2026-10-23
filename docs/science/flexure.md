# Lithospheric flexure

Status of this page: written with the feature on 2026-10-02. Three constants were checked against
their sources (ISO-041, ISO-042, ISO-048); the rest are marked in the ledger as read from abstracts,
uncited, or game choices. The measurements are from small ensembles of four runs; two of them
were made before the last change of one constant (see "Earth against the mod"). Updated 2026-10-18
for the polar holes (ISO-056 rewritten, ISO-074 to ISO-076 added; six runs).

Ledger ids on this page: `ISO-040` to `ISO-077` (see [ledger/ISO.md](ledger/ISO.md)).

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
as before. Deep basins in front of mountain ranges (the Ganges plain) do not appear yet. They need
the plate model to build a range as a wedge on the edge of a stiff plate, which is later work.

## The model in plain words

It runs in the surface stage, at the end of each 2-million-year step, after erosion, deposition
and the spreading of over-thick crust.

1. **The loads.** For every cell the stage keeps, moving with the crust, how much weight has been
   added or removed since the start of the run. Each step it adds: the crust the plate model
   thickened (its uplift), crust moved by spreading, rock eroded or deposited, rock the volcano
   stage laid on top, the change of the sea's depth over the cell, and the land ice the climate
   stage reports. On ocean floor that follows GDH1 (the shipped presets) the sea that fills the
   floor's own cooling is left out: GDH1's depths are those of floor under water, so that water's
   weight is in them already (ISO-088; before 2026-10-21 it was loaded again, and old floor sank
   about 1.45 times GDH1).
2. **The stiffness.** The effective elastic thickness `Te` says how thick the stiff part of the
   shell is. Ocean floor stiffens with age. Continental crust is weak just after mountain
   building and stiffens over a few hundred million years. Volcanic arcs and rifts are weak. The
   boundary between two plates is a break: one plate does not hold the other up.
3. **The solve.** The shell is treated as a thin elastic plate floating on a fluid. The plate
   carried from the last step is in balance with the load it carries; the stage solves only for
   how far it sinks or rises under this step's change of load. The plate model's uplift counts
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
| ISO-046 | Crust thickened behind an uplift `u` | `u / (1 − k)`, `k` 0.818 on land, 0.736 under water | – | Airy isostasy, none cited | `uncited` |
| ISO-047 | Density of eroded and deposited rock | 2,640 on land, 2,301 under water | kg/m³ | none (keeps ISO-001, ISO-003) | `uncited` |
| ISO-048 | Oceanic `Te` | 2.7 × √(age in Myr) | km | Calmant et al. 1990 | `verified` |
| ISO-049 | Oceanic `Te`, at most | 40 | km | Watts & ten Brink 1989 | `abstract-only` |
| ISO-050 | Continental `Te` long after an orogeny | 80 | km | Burov & Diament 1995 | `abstract-only` |
| ISO-051 | Continental recovery | `80 × (1 − e^(−t/300 Myr))` | km | form after Lu et al. 2024; the numbers are the agent's | `uncited` |
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
| ISO-088 | The sea over GDH1's own deepening | no load: GDH1 is depth under water | m | Stein & Stein 1992; Kominz et al. 1998 (PLT-353) | `cited-unverified` |

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

- The plate is purely elastic. Real lithosphere also depends on whether crust and mantle are
  coupled and on how sharply the plate is bent (Burov & Diament 1995); neither is modelled.
- `Te` depends only on the crust's type and thermal age.
- The plate equation's extra terms for varying rigidity are left out (ISO-040).
- A step is 2 million years; the ground answers in thousands of years on Earth. The solve gives
  the equilibrium. Ice-age cycles inside a step are not followed: the load is the ice standing at
  the step's end.
- Every plate boundary is a break, also where two plates slide past each other.
- On the quick presets a flexural feature is 2 to 8 cells wide.
- Trenches and the ocean floor's deepening with age are not flexure in the model; they are
  prescribed by the plate model, and under GDH1 the water that fills the deepening is no load
  either (ISO-088).
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
(scattered continents, seed 5, at plate step 449), and in three more the guard's worst passed
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

MEASURED

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
- The rule for a plate that weakens lets go at once, within the step. Relaxation takes about a
  million years (Beaumont 1978, abstract); Watts & Zhong 2000 (abstract) find oceanic plates
  elastic on long timescales, which is the stiffening side. That the share let go is `1 − r` of
  the load held is the agent's rule, from the two limits; no source states it.
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

The sources from `TurcotteSchubert_Geodynamics` to `DeCellesGiles1996_flexure` were read by a sub-agent on 2026-10-02, not by the agent that wrote the code.
