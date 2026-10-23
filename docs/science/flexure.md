# Lithospheric flexure

Status of this page: written with the feature on 2026-10-02. Three constants were checked against
their sources (ISO-041, ISO-042, ISO-048); the rest are marked in the ledger as read from abstracts,
uncited, or game choices. The measurements are from small ensembles of four runs; two of them
were made before the last change of one constant (see "Earth against the mod"). Updated 2026-10-18
for the polar holes (ISO-056 rewritten, ISO-074 to ISO-076 added; six runs), and 2026-10-22 for
the settling's feedback (ISO-061 rewritten, ISO-091 to ISO-093 added; twelve runs a side), and
2026-10-23 for the trench rows (ISO-096 and ISO-097 added, ISO-088 merged into ISO-096).

Ledger ids on this page: `ISO-040` to `ISO-077`, `ISO-088` (merged), `ISO-091` to `ISO-093`,
`ISO-096`, `ISO-097` (see
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
as before. Deep basins in front of mountain ranges (the Ganges plain) do not appear yet. They need
the plate model to build a range as a wedge on the edge of a stiff plate, which is later work.

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
| ISO-046 | Crust thickened behind an uplift `u` | `u / (1 − k)`, `k` 0.858 on land, 0.793 under water (crust 2,830 kg/m³ since 2026-10-06) | – | Airy isostasy, none cited | `uncited` |
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
| ISO-091 | Only crust is load | the plate model's crust uplift `u_c`, either sign: load `k_a u_c / (1 − k_a)`; the rest of the uplift and the sea it moves, none | m | none (the plate equation's load is mass, ISO-040) | `uncited` |
| ISO-092 | The crust uplift | the change of each sample's pools' Airy height over the plate step, with the resampling's accretion and the rate limits' cut | mm/yr | none (the plate model's Airy height, PLT-278 to PLT-282) | `uncited` |
| ISO-093 | Earth's departure from local isostasy | at most about 2 to 4 km of rock (free-air −380 to +300 mGal at 0.112 mGal/m) | km | Watts et al. 2025, ten Brink et al. 2004, Stern, Cattin et al. 2001, Hunter & Watts 2016, Watts & Burov 2003 | `verified` (measurement only) |
| ISO-096 | The sea over the plate model's own moves | no load: the move as the carry made it (`h_start` less the source's elevation before the step); on ocean floor that builds nothing all of it, on building columns all but the crust's change (ISO-091). Took in ISO-088 (GDH1's cooling) | m | Turcotte & Schubert §3.13; Levitt & Sandwell 1995; Zhang et al. 2014 | `cited-unverified` |
| ISO-097 | The carry at a trench | the plate model's elevation plus the carried offset, no uplift added | m | none (numerics; Earth's trench depths for the check: Zhang et al. 2014, 2018) | `game-choice` |

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
samples.

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
`collapse_*` keys), the seven that become the default. Arms `base`/`fg-trench`,
`trench-m-rs12e`/`fg-trcomb-rs12e`, `trench-m-rs12e-sheet`/`fg-trench-sheet`.

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
  −8.2 km. What builds that offset under a trench now was not traced.
- The trench keys' supercontinent 2, before the fix (traced on the trench budget's head, the
  pool's 28,548 m to the metre): the same kind of peak. From −46 to −14 Myr a collision, rift and
  back-arc column on the polar rows (67 to 73°S) stood 14.5 to 29.0 km above the sea in the plate
  model, its uplift at 1.3 to 3.5 mm/yr, and the surface's offset over it reached +10 to +13 km at
  the worst steps. All 13 steps past 19.8 km are that peak. The same run had 341 steps with a floor
  more than 11 km down, the deepest 15.8 km; the two fixed runs above had 25 and 22.

Those heights are the plate model's own (plates/), outside this lane: a column whose uplift runs
at the rate limit's ceiling (PLT-288, 3.5 mm/yr) step after step, up to the 30 km cap, on the
polar rows twice in three runs.

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
- `Watts2025_Hawaii`, `tenBrink2004`, `Stern_OceanTrenches` (proof), `WattsBurov2003`: full text;
  `HunterWatts2016` (about 200,000 of 279,000 characters, section 8 and Appendix C), `Cattin2001`
  (about 100,000 of 130,000 characters): most of the text; `McNuttMenard1982`, `GoetzeEvans1979`:
  abstracts; `Watts1978`, `KarnerWatts1983` and chapter 3 of `TurcotteSchubert_Geodynamics`: full
  text this time. Read 2026-10-22 by a Sonnet sub-agent of the flexure-guard agent, the
  quotations in the bibliography (ISO-093).

The sources from `TurcotteSchubert_Geodynamics` to `DeCellesGiles1996_flexure` were read by a sub-agent on 2026-10-02, not by the agent that wrote the code.
