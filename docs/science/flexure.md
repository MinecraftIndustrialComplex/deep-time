# Lithospheric flexure

Status of this page: written with the feature on 2026-10-02. Three constants were checked against
their sources (ISO-041, ISO-042, ISO-048); the rest are marked in the ledger as read from abstracts,
uncited, or game choices. The measurements are from small ensembles of four runs; two of them
were made before the last change of one constant (see "Earth against the mod").

Ledger ids on this page: `ISO-040` to `ISO-072` (see [ledger/ISO.md](ledger/ISO.md)).

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
   stage reports.
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
meet the stiffer plate. When it weakens (reheated, or becoming a plate's broken edge) its bending
stress is released and the ground moves to the new balance.

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
| ISO-056 | Stiffening and weakening | the plate carries its bending moment; gained rigidity joins stress-free, lost rigidity releases the moment in proportion | – | principle: Watts 1978 | `abstract-only` |
| ISO-068 | Increments | each step solves for the step's loads only; the carried plate is in balance; a solve's unfinished part is carried | – | none | `game-choice` (numerical) |
| ISO-069 | The plate model's resolution | its root bends the plate only low-passed at its sample spacing (two binomial passes on the quick presets) | – | none | `game-choice` (numerical) |
| ISO-070 | The spreading's root | bends nothing: the spreading's change stays locally compensated | – | none (its physics from memory of Clark & Royden 2000) | `game-choice` (numerical) |
| ISO-071 | Earth's relief | Everest 8,848.86 m above the sea, the Challenger Deep 10,925 m below | m | Dangol et al. 2020; Stewart & Jamieson 2019 | `verified` |
| ISO-072 | The surface guard | a cell farther from the sea than 12 × 19,774 m × max(1, g_E/g) stops the run | m | none (a margin) | `game-choice` |
| ISO-061 to ISO-064 | What counts as a load | see the ledger | | none | `uncited`, one `game-choice` |

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
  prescribed by the plate model.
- Along active margins the ground still zigzags from cell to cell somewhat more than without
  flexure: narrow loads that are real at this resolution (marine fill of kilometres in trench
  cells) and the release of bending stress where a plate boundary moves through cells. The first
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
(the 30 km ceiling with the whole thickening held up, 7 times). The transients themselves are the
plate model's to fix (an open question below).

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
  tens of km the flexure then holds up for a step: which plate process makes them was not traced
  (build log 2026-10-18, the surface divergence).
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

All were read by a sub-agent on 2026-10-02, not by the agent that wrote the code.
