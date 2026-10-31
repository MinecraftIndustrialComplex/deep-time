# Sea level

Status of this page: `stage 2 (constants checked against sources on 2026-10-21)`, written with the
change that made the sea level follow the ocean's volume (Ben, decisions log 2026-10-21: "Its own
task now", then "Don't follow that old idea" for the 2026-09-30 row "Keep calibrated coastlines").

Ledger ids on this page: `ISO-085` to `ISO-087`, `PLT-350` to `PLT-356`, `CLM-210`, and the rows
they replace or amend: `ISO-033` to `ISO-039`, `CLM-091`, `CLM-099`, `CLM-106` to `CLM-108`,
`PLT-129`, `HNT-005`, `HNT-041`, `HNT-056` (see [claims.md](claims.md)).

## What you see in the world

The sea stands where the planet's water, less what its ice sheets hold, fills the ocean basins. A
planet whose ocean floor is young and shallow (after a supercontinent breaks up and fast ridges
spread) floods its continents' low interiors with shallow seas; one whose floor is old and deep
(a supercontinent with slow ridges) drains them. Ice sheets that grow on land lower the sea and lay
the shelves dry; sediment washed into the sea takes room and raises it. The world is baked with the
final sea at y 63, as before, so a player meets the same sea level in blocks whatever the planet's
water did; what changes is which ground lies above it, and the strata the sea's advances and
retreats left on the continents.

## Where sea level came from until 2026-10-21

Every consumer reads one number per step, the step's eustatic sea level (m above the plate
model's datum). Until this change it was set in one place and read in many.

- **The curve** (`surface/params.rs`, `SeaLevel::at`, ISO-033 to ISO-035): a base (0 m) plus two
  sines, 80 m with a 250-Myr period and 25 m with a 36-Myr period (every shipped preset), plus an
  optional piecewise-linear curve. No source; the sines stood for ridge volume and the
  supercontinent cycle.
- **Glacio-eustasy** (`climate/mod.rs`, CLM-091, CLM-106 to CLM-108): the climate stage added
  `glacio_eustasy` (1.0) times −(ρ_i/ρ_w)(V − V_ref)/A_ocean, with V the land ice at the step's
  start and V_ref `ice_reference_km3` (1.0·10⁷ km³), the present ice of the calibrated Earth-like
  runs, so that their present-day coastlines stayed where the hypsometry calibration (P1-17) had
  put them (decisions log 2026-09-30, "Keep calibrated coastlines", ISO-037).
- **The steering nudge** (`steer`, STE-006): up to ±200 m, end-weighted, added on top.
- The sum went into `WorldState::globals["climate/sea_level_m"]` (`climate::SEA_LEVEL_GLOBAL`).
  Without the climate stage the surface took its curve alone.

The readers, unchanged by this change:

- the **plate model** (`plates/mod.rs`, `sea_m`): the previous step's global, for the columns that
  settle under water (`settle.rs`, ISO-078) and, now, the dry share of the floor's subsidence
  (PLT-353);
- the **surface stage** (`surface/mod.rs`): its open-sea flood (ERO-055), coast, marine deposition,
  waves, the flexure's water load, the young stage and the amplifier, its `sea_level_m` and its
  `sea_history` (the deposits' lowest stand, DEP-214);
- **volcanoes, impacts and structure** (the surface's `sea_level_m`: submarine features, depth
  below the sea, the horizons' block heights);
- the **climate stage** (its ocean mask, the snapshot's sea, the series column `sea_level_m` in
  `notes.climate.series`);
- **events** (`marine_transgression` and `marine_regression`, ISO-038, EVENTS.md);
- the **bake** (`bake/terrain.rs`: the final sea goes to y 63) and the CLI's run summary.

## The model in plain words

Each plate step the climate stage computes the sea level once, after the plates moved and before
the surface stage runs (`climate/mod.rs`, `sea_from_water`; `sea_level.rs`, `fill_level`):

1. The planet holds a fixed amount of surface water, `planet.surface_water_m`, written as a layer
   over its whole area: Earth's ocean and land ice as water, 2,664 m (ISO-085).
2. The land ice at the step's start (the climate's ice sheets, Earth-normalized km³) takes
   ρ_i/ρ_w of its volume out of that water (CLM-210). Ice that holds more water than the planet
   has (a snowball on a dry preset) leaves the sea empty and is not cut back; the shortfall is
   the global `climate/ice_water_short_m` (m of water over the planet) and a warning in the run's
   log the first time.
3. The rest stands in the sea: the level S at which the open sea holds it over the ground the
   surface stage starts the step from. The open sea at S is what the surface itself calls sea, a
   connected body of ground below S at least a 500th of the grid (ERO-055); a closed hollow below S
   that no channel joins to the ocean stays dry until the sea tops its rim (ISO-086). The open
   sea is taken again at the level found: a hollow that grew to that size on the way up without
   reaching the ocean is sea there, as the surface counts it, and the level is then found by
   halving on the open sea's own water, so the climate's level and the surface's sea agree.
4. The steering nudge, if any, is added (STE-006), and the result goes into the same global as
   before; every reader above takes it unchanged.

Without the climate stage (the surface-only pipelines) the surface stage fills its own ground the
same way with the whole water (no ice), and the plate model takes that level for its land test
(PLT-353), so both stages use one sea.

Because the level follows the ground, everything that changes the ocean basins' room moves it:

- **The ocean floor's age** (PLT-350 to PLT-353): the shipped Earth-like presets now sink the floor
  by Stein & Stein's GDH1 age–depth law instead of Cortial's dampening (PLT-129). Young floor is
  shallow, so a planet with more young floor has less room and a higher sea.
- **The first ocean** (PLT-356): the plate model ages the starting ocean floor by its distance
  from the boundaries that an earlier, invented motion pulled apart (PLT-273), which made it young
  (median 19 to 22 Myr against Earth's 50 to 60). Under GDH1 the distance now only orders the
  floor, young where it diverged, and the ages follow the spread of an ocean in steady state,
  made at a constant rate and consumed evenly at every age (Parsons 1982): a mean of 60 Myr and a
  median of 53. Without it every run started with the sea 330 to 450 m high and drained it over
  100 Myr as that floor aged (with it, the sea starts 280 to 375 m below the datum: "What the
  numbers say"); on the climate test's small planet the sea started 605 m high,
  drowned all the land, and with no weathering the CO₂ rose to a bar and the planet to 76 °C.
- **The floor between diverging plates** (PLT-354): between two resamplings the plate model
  covers the widening gap between diverging plates with no samples; its cells take the floor the
  next resampling will make there, deepening with its age (PLT-351). The surface holds those
  cells in place, keeps what it laid on them, and starts a cell that enters a gap bare. Before
  this, the surface carried a gap cell with the nearest plate and its zero uplift: the gap kept
  the height of the edge it was pulled from (the last resampling's ridge crest, a rifted
  continent's margin), and since GDH1's rate form sinks samples only, that young floor stood 200
  to 450 m too high until it was subducted, with the sea that much higher. The gap's far side is
  found at any distance (the nearest sample of each other plate), as the resampling finds it;
  within a cell's 32 nearest samples alone, a wide gap's cells took the near edge's depth, and
  the sea jumped 40 to 50 m at each resampling.
- **Continents**: their area and height (more continental crust, less ocean; a collision that
  thickens crust frees ocean area).
- **Ice**: step 2.
- **Sediment**: what the rivers carry into the sea fills its floor, and the GDH1 rate form keeps
  it there (the dampening erased it within about 47 Myr, PLT-129's note), so it raises the sea
  until the floor that holds it is subducted.
- **Isostasy of the water**: the surface flexure loads the sea floor with the water over it (ISO-044,
  ISO-046), so a sea that rises presses its floor down and makes some of its own room, the
  hydro-isostatic share Earth's backstripping writes as (ρ_m − ρ_w)/ρ_m (Kominz et al. 1998).
  The water that fills GDH1's own deepening is no new load (ISO-088, since 2026-10-23 part of
  ISO-096 with the plate model's other moves of the floor, flexure.md "The trench rows"): GDH1 is the depth of floor
  under water, that water's weight in it. Until the review of 2026-10-21 the flexure loaded it
  again, and the floor sank about 1.45 times GDH1 ("The flexure's water over GDH1's floor").

## Equations and constants

The fill (`sea_level::fill_level`):

  Σ_{c in the open sea at S} (S − h_c) · a_c = (L − L_ice) · Σ_c a_c,
  L_ice = (ρ_i/ρ_w) · V_ice / A_planet,

with h_c the cell's ground (m), a_c its true area (the grid's row weights, equal-area rows of the
Lambert grid, or the climate's Earth-normalized km² per row), L `planet.surface_water_m` and V_ice
the land ice at the step's start. Over the cells sorted by the level at which the sea reaches them
(their spill height: the lowest pass to the ocean, or their own ground), V(S) = S·A − B is piecewise
linear, A the area and B the area-weighted ground of the cells reached; it jumps where the sea tops
a pass. The solve walks that list and stops where V reaches the water: S = (V + B)/A between two
passes, or the pass itself when the hollow behind it takes the rest. It is exact for the ground
given; one priority flood and two sorts per step. Where the open sea at that level holds a body
the flood from S₀'s sea did not reach (one that grew to n/500 cells between S₀ and S without
touching the ocean), S is found instead by halving on V_open(S) = Σ over the open sea at S of
(S − h)·a to 1 mm: V_open only grows with S, and V_open(S₀) ≤ the water ≤ V_open(S).

The floor (`plates/pools.rs`, `surface_step`, `gdh1_depth_m`):

  d(t) = 2,600 + 365 √t (t < 20 Myr),  d(t) = 5,651 − 2,473 e^(−0.0278 t) (t ≥ 20 Myr),
  Δz = −[d(t) − d(t − Δt)]            below the sea,
  Δz = −[d(t) − d(t − Δt)] / W        above it, W = ρ_m / (ρ_m − ρ_w),

with t the sample's age at the step's end (Myr) and Δt the step. New ocean floor (ridges, gaps,
the first ocean) is laid at −d(t) (PLT-351). With a ridge crest z_r and an old floor z_f other
than GDH1's own −2,600 and −5,651 m (the editor's ridge and abyssal depths), the curve is
stretched between them, z = z_r − s·(d(t) − 2,600) with s = (z_r − z_f)/3,051, and each step
sinks s times as much (PLT-355); every shipped preset has s = 1.

The flexure's water over that floor (`surface/flexure.rs`, `cooled_under_sea` until 2026-10-23;
since then `booked`, which leaves out the sea over the whole of the plate model's move of the
floor, GDH1's sinking among it, ISO-096, flexure.md "The trench rows"): of the step's
change of the sea's depth over an oceanic cell that is not new at the place, the part

  min(max(S − h_start, 0), s·[d(t) − d(t − Δt)])

is no load (ISO-088), with h_start the cell's ground after the plate model's sinking.

The first ocean's ages (`plates/init.rs`, `steady_state_ages`): with ocean floor made at a
constant rate C₀ and consumed evenly at every age up to t_m, the area of each age is
dA/dt = C₀(1 − t/t_m) (Parsons 1982), so the share of the floor younger than t is
F(t) = 1 − (1 − t/t_m)². The L0 samples are equal-area: the sample of rank F among the ocean
floor's samples, ordered by the earlier motion's ages, takes

  t = t_m (1 − √(1 − F)),  t_m = 180 Myr,

a mean of t_m/3 = 60 Myr (Earth's floor today 64.2, Seton et al. 2020) and a median of 52.7.

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| ISO-085 | The planet's surface water, a layer over the whole planet | `planet.surface_water_m` 2,664 (default and every shipped preset) | m | Charette & Smith 2010 (ocean 1.33238·10⁹ km³), Fretwell et al. 2013 (Antarctica 26.54·10⁶ km³ grounded), Morlighem et al. 2017 (Greenland 2.99·10⁶ km³), IPCC AR5 Table 4.1 (glaciers 0.41 m); Earth's area from memory | `verified` |
| ISO-086 | The sea at S is the open sea: a connected body of ground below S of at least n/500 cells; closed hollows take no water until the sea tops their rim | `fill_level` with the surface's `open_sea` rule (ERO-055) | m | none needed (mass balance); the size rule is ERO-055's | `game-choice` |
| ISO-087 | The sea level series' check against Earth's Phanerozoic range | −100 to +250 m against today | m | Müller et al. 2008 (Late Cretaceous 170 m, 85 to 270), Miller et al. 2005 (Cretaceous peak 100 ± 50 m), both abstracts; Haq et al. 1987's 250 m at second hand (Miller et al. 2011) | `abstract-only` |
| ISO-088 | The sea over GDH1's own deepening is no new load on the floor (merged into ISO-096, 2026-10-23) | `flexure::cooled_under_sea` until 2026-10-23, then `flexure::booked` | m | Stein & Stein 1992 (GDH1 is depth below the sea); Kominz et al. 1998's relation (PLT-353) | `cited-unverified` |
| CLM-210 | Land ice holds (ρ_i/ρ_w) of its volume of the planet's water out of the sea | `sea_level::ice_layer_m`, 917/1028 (CLM-099) | m | IPCC AR5 Table 4.1 note b (917 and 1028 kg/m³, read) | `verified` |
| PLT-350 | The ocean floor's law | `tectonics.ocean_depth`: `gdh1` in every shipped Earth-like preset, `dampening` (PLT-129) the default and the plate-only presets' | - | the brief: PLT-129's dampening fixed at an age–depth law | `game-choice` |
| PLT-351 | New ocean floor at the law's depth for its age | `pools::floor_at` (init, ridges, L1 gaps) | m | as PLT-352 | `cited-unverified` |
| PLT-352 | GDH1 | 2,600, 365, 5,651, 2,473, 0.0278, knee 20 Myr | m, Myr | Stein & Stein 1992 (not read); 2,600 m and the 0.0278 decay read in Stein 1995; the rest on a secondary page | `cited-unverified` |
| PLT-353 | Ground above the sea sinks by 1/W of the water-loaded subsidence | W = ρ_m/(ρ_m − ρ_w) (`settle::WATER_LOADED`) | dimensionless | Kominz et al. 1998 (the factor, read) | `verified` |
| PLT-354 | The floor in a gap between plates is held in place, deepens with its age, keeps what lies on it; a cell entering a gap starts bare | `advect::hold_gaps`, `newborn`, `carried_elevation`; `plates/gap`, `surface/crust_gap` | - | none needed (the plate model's kinematics) | `game-choice` |
| PLT-355 | GDH1's shape between the preset's ridge crest and old floor, so the editor's ridge and abyssal depths keep working; GDH1 exactly at its own depths | `pools::gdh1_scale`, `floor_at`, `surface_step`; s = (z_r − z_f)/3,051, 1 in every shipped preset | - | none (a stretch of PLT-352's curve) | `game-choice` |
| PLT-356 | Under GDH1 the first ocean's ages take the steady-state age–area spread in the earlier motion's order | `init::steady_state_ages`, t_m = 180 | Myr | Parsons 1982 (the law, abstract read); t_m from memory (Earth's oldest floor in place) | `abstract-only` |

## Earth against the mod

Measured 2026-10-21 on the six-run ensemble: `earthlike_quick_16k`, 1,000 Myr, 8 threads, seeds 1
to 3, supercontinent (sup) and scattered (sca) starts. Before is master f1632cf3 (the curve with
glacio-eustasy, Cortial's dampening at 0.12 mm/yr towards −5,600 m from a −2,500 m ridge); after
is this change at 95f7ff79 (the simulation is the same at the branch's head, and PLT-355 changes
nothing at s = 1). Each cell is before → after. The map is the keyframes (p50 over the plate
frames, or the last frame where it says final); the bake is the baked world
(`tools/review/water_census.py --bake`); lake basins are `tools/review/lake_basins.py` on four
epoch checkpoints per run (Earth-equivalent areas, the planet's times 66.3; the after runs'
checkpoints come from a second run of the same binary with `--epochs`, whose census matched the first's in all six); the sea, ice,
continental crust and ocean age are the runs' own records (`notes.climate.series`,
`notes.plates.dynamics`). "Under the sea" is the share of the continental crust (the `crust_age`
layer's continental colour) that the `rivers` layer draws as sea; a landmass is an 8-connected body
of land on the keyframe.

| Measure | Earth | sup 1 | sup 2 | sup 3 | sca 1 | sca 2 | sca 3 | mean |
|---|---|---|---|---|---|---|---|---|
| Sea at the start, m | | 30 → −281 | 30 → −335 | 30 → −300 | 29 → −319 | 26 → −375 | 28 → −343 | 29 → −325 |
| Sea at the end, m | | 0 → 608 | −13 → −84 | −26 → −168 | −55 → 1 | −23 → −519 | −19 → −53 | −23 → −36 |
| Sea against the end past the first 100 Myr, min / p50 / max, m | −100 to +250 (ISO-087) | −147/−20/101 → −1378/−561/14 | −174/−33/103 → −563/−288/142 | −157/−19/122 → −613/−296/50 | −118/6/155 → −767/−329/123 | −157/−30/102 → −449/19/374 | −163/−10/127 → −1213/−48/734 | −153/−18/118 → −831/−250/239 |
| Land ice at the end, 10⁶ km³ | 29.7 (ISO-085's sources) | 21 → 17 | 26 → 27 | 30 → 49 | 44 → 25 | 22 → 56 | 29 → 28 | 28 → 34 |
| Continental crust, share of the planet, p50 over the steps | 0.41 (HYP-012) | 0.403 → 0.423 | 0.395 → 0.325 | 0.385 → 0.368 | 0.341 → 0.350 | 0.347 → 0.330 | 0.343 → 0.356 | 0.369 → 0.359 |
| Continental crust under the sea, final | about 0.19 (from memory) | 0.384 → 0.582 | 0.344 → 0.262 | 0.314 → 0.338 | 0.317 → 0.236 | 0.391 → 0.237 | 0.369 → 0.198 | 0.353 → 0.309 |
| Continental crust under the sea, p50 of the plate frames | about 0.19 | 0.358 → 0.363 | 0.297 → 0.223 | 0.311 → 0.204 | 0.319 → 0.236 | 0.319 → 0.217 | 0.313 → 0.255 | 0.320 → 0.250 |
| Land, share of the planet (map p50) | 0.29 | 0.274 → 0.300 | 0.287 → 0.322 | 0.292 → 0.324 | 0.246 → 0.322 | 0.252 → 0.325 | 0.254 → 0.307 | 0.267 → 0.317 |
| Land (bake) | 0.29 | 0.257 → 0.203 | 0.280 → 0.311 | 0.318 → 0.258 | 0.261 → 0.333 | 0.236 → 0.288 | 0.244 → 0.357 | 0.266 → 0.292 |
| Land above 2 km, % of the land (bake) | 10.2 (HYP-004; rock 4.7, HYP-041) | 14.0 → 8.8 | 14.8 → 21.5 | 18.5 → 27.3 | 22.7 → 22.6 | 19.3 → 29.7 | 18.6 → 25.6 | 18.0 → 22.6 |
| Largest landmass, final, share of the planet | 0.167, Afro-Eurasia (from memory) | 0.052 → 0.072 | 0.165 → 0.274 | 0.267 → 0.136 | 0.109 → 0.329 | 0.089 → 0.094 | 0.041 → 0.346 | 0.120 → 0.209 |
| Second landmass, final | 0.083, the Americas (from memory) | 0.052 → 0.034 | 0.050 → 0.016 | 0.022 → 0.069 | 0.041 → 0.002 | 0.060 → 0.088 | 0.036 → 0.014 | 0.043 → 0.037 |
| Land in landmasses of 0.01 of the planet or more, final | about 0.95 (from memory) | 0.845 → 0.764 | 0.921 → 0.896 | 0.878 → 0.872 | 0.925 → 0.958 | 0.877 → 0.888 | 0.897 → 0.981 | 0.891 → 0.893 |
| Coast p50, km (map; final) | 433 (HYP-035) | 327 → 373; 197 → 191 | 448 → 387; 349 → 428 | 285 → 426; 290 → 327 | 297 → 366; 251 → 422 | 262 → 368; 303 → 419 | 249 → 444; 225 → 617 | 311 → 394; 269 → 401 |
| Lakes, % of the land (map p50) | under 1.8 (HYD-090) | 3.07 → 3.79 | 3.86 → 4.50 | 4.47 → 4.31 | 3.15 → 4.07 | 4.79 → 4.15 | 3.73 → 4.23 | 3.85 → 4.18 |
| Lakes, % of the land (bake) | under 1.8 | 0.88 → 2.08 | 1.16 → 2.00 | 2.05 → 1.94 | 1.47 → 2.39 | 3.00 → 1.33 | 1.54 → 1.76 | 1.69 → 1.92 |
| Lake area on oceanic crust, % (epochs) | | 33.9 → 27.5 | 28.3 → 45.3 | 29.3 → 41.5 | 32.2 → 34.4 | 41.6 → 41.8 | 37.3 → 37.4 | 33.8 → 38.0 |
| Lakes of the Caspian's size or more per snapshot, all (on sea floor) | 1 (1) | 3.0 (0.5) → 6.25 (1.0) | 4.75 (1.75) → 5.5 (2.5) | 4.0 (0.25) → 5.25 (2.0) | 3.75 (0.75) → 5.5 (1.5) | 3.75 (1.0) → 4.5 (1.25) | 3.0 (1.0) → 5.25 (2.25) | 3.71 (0.88) → 5.38 (1.75) |
| Closed ground below sea level, % of the land (bake) | | 0.17 → 0.24 | 0.14 → 0.99 | 0.66 → 0.22 | 0.13 → 0.77 | 0.20 → 0.13 | 0.22 → 0.06 | 0.25 → 0.40 |

Closed basins on sea floor per snapshot (the 24 epoch checkpoints), by Earth-equivalent area:
under 10⁴ km² 71.9 → 89.8, 10⁴ to 10⁵ 45.4 → 54.8, 10⁵ to 3.7·10⁵ (the Caspian) 6.2 → 9.7,
3.7·10⁵ to 10⁶ 0.75 → 1.46, 10⁶ and more 0.12 → 0.29; in all 124 → 156 basins, 3.4 → 5.2 million
km² (Earth's natural lakes 2.67 million, HYD-090), their depth p50 305 → 316 m and their level p50
617 → 559 m over the datum.

The plate checks (`deeptime plate-check`, per start over the three seeds) meet 29 → 27
(supercontinent) and 31 → 26 (scattered) of 45 targets. Gained on the supercontinent start: the
rift-born plates welded back to their rift family (12.4 → 9.8 %, at most 10) and the collision
length's 90th percentile (147 → 107 Myr, at most 120). Lost there: the slab-speed ratio (2.72 →
2.43, at least 2.5), the floor older than 180 Myr (9.9 → 10.8 %, at most 10), the supercontinent's
tenure (231 → 21 Myr, 100 to 300) and the assembly-to-assembly period (537 → 249 Myr, 400 to 800);
on the scattered start: the most land gained in a step (1.28 → 1.52 %, at most 1.5), the collision
length's median (36 → 57 Myr, 15 to 50) and 90th percentile (109 → 129 Myr), the slab-speed ratio
(2.58 → 2.12) and the convergence 20 Myr into a collision (0.43 → 0.66, 0.2 to 0.6). All but the
supercontinent's two sit near their bounds. Those two do not, and they go with the moving sea
rather than with GDH1: on supercontinent seeds 1 and 3 the tenure is 316 Myr before, 49 with the
volume sea over the old dampening floor, 36 at dd2f3f9c (GDH1, the first ocean young) and 20 after
(two seeds only; open questions).

The same runs in the continents census's terms (`tools/review/continents_census.py keyframes`,
master's tool from the diagnosis of 2026-10-21, plates.md), with two more sets to tell the parts
apart: dd2f3f9c, this change before the steady-state first ocean (PLT-356); and the volume sea over
the old dampening floor (`tectonics.ocean_depth=dampening` at master's 0.12 mm/yr from −2,500 to
−5,600 m; supercontinent seeds 1 and 3, scattered seed 1). Means over the runs, the final keyframe
/ the median over the plate frames (more than 100 Myr in):

| Measure | Earth | before (6 runs) | dd2f3f9c (6 runs) | after (6 runs) | volume sea, old floor (3 runs) |
|---|---|---|---|---|---|
| Land | 0.29 | 0.264 / 0.251 | 0.275 / 0.305 | 0.292 / 0.301 | 0.219 / 0.230 |
| Continental crust, share of the planet | 0.41 (HYP-012) | 0.380 / 0.360 | 0.341 / 0.327 | 0.344 / 0.342 | 0.317 / 0.352 |
| Its share under the sea: shelf (under 200 m), deeper | 0.195 in all, 0.115 on the shelf (CRUST1.0, crust of 25 km or more, PLT-361) | 0.368: 0.103, 0.265 / 0.338: 0.084, 0.254 | 0.366: 0.098, 0.268 / 0.264: 0.082, 0.183 | 0.323: 0.087, 0.237 / 0.283: 0.082, 0.202 | 0.376: 0.083, 0.293 / 0.367: 0.090, 0.279 |
| The ocean's water under the sea, 10⁹ km³ | 1.33 (ISO-085) | 1.102 / 1.276 | 1.341 / 1.324 | 1.331 / 1.326 | 1.319 / 1.332 |
| The sea that would hold Earth's water and ice (1.359·10⁹ km³), m against the run's | | +574 / +161 | −10 / +8 | −8 / +8 | +54 / +9 |
| Land at that sea | 0.29 | 0.165 / 0.205 | 0.278 / 0.302 | 0.293 / 0.296 | 0.209 / 0.227 |

The after runs hold Earth's water within one ramp step of the keyframes (the census's sea
differs from the run's by −35 to +25 m at the end). At the end the curve held 0.81 of Earth's water
(1.10·10⁹ km³), the diagnosis's 0.80 on Ben's run.

The same change in the continent bisect's terms (`tools/review/out-free/continent-bisect.md`: the
final frame at 1,000 Myr, seeds 1 and 2, both starts, 2 threads, the crust budget's env on, read
after `deeptime stage volcanoes`), so it reads as more rows of that table. Before is the bisect's
own 30c4d664 row (master's simulation is byte-identical from 30c4d664 to c790d1f3, this branch's
base); after is this branch at 95f7ff79 (`continent-bisect.sh ship 95f7ff79`, then
`all 95f7ff79 1,2 supercontinent,scattered_continents 1000` and `continent-bisect.py ... --runs`),
and dd2f3f9c the same way.

| Run | land | continental crust | its share under the sea | largest landmass | second |
|---|---|---|---|---|---|
| Earth | 0.29 | 0.41 | about 0.19 | 0.167 | 0.083 |
| sup 1 | 0.252 → 0.200 | 0.389 → 0.406 | 0.408 → 0.600 | 0.101 → 0.068 | 0.036 → 0.032 |
| sup 2 | 0.278 → 0.313 | 0.399 → 0.337 | 0.354 → 0.269 | 0.159 → 0.265 | 0.049 → 0.015 |
| sca 1 | 0.267 → 0.328 | 0.358 → 0.349 | 0.326 → 0.256 | 0.105 → 0.294 | 0.039 → 0.018 |
| sca 2 | 0.229 → 0.290 | 0.350 → 0.301 | 0.408 → 0.251 | 0.083 → 0.090 | 0.057 → 0.085 |
| mean of the four | 0.257 → 0.283 | 0.374 → 0.348 | 0.374 → 0.344 | 0.112 → 0.179 | 0.045 → 0.038 |

The bisect's pooled row for each commit (the means of the four runs):

| Commit | land | continental crust | under the sea | land above 2 km | largest | second | landmasses of 0.01 or more | their land | island arcs, of the land | pieces |
|---|---|---|---|---|---|---|---|---|---|---|
| 30c4d664 (master) | 0.257 | 0.374 | 0.374 | 0.270 | 0.112 | 0.045 | 5.0 | 0.889 | 0.412 | 276 |
| dd2f3f9c (volume sea, GDH1, first ocean young) | 0.270 | 0.338 | 0.377 | 0.275 | 0.137 | 0.046 | 3.8 | 0.844 | 0.448 | 354 |
| 95f7ff79 (and the first ocean in steady state) | 0.283 | 0.348 | 0.344 | 0.270 | 0.179 | 0.038 | 3.5 | 0.854 | 0.453 | 263 |

By the bisect's own rule (a land step under 0.03 is not a finding) the land's +0.026 is not one.
The drowned share falls by 0.030 and the largest landmass rises by 0.067, but two runs of four
carry both (sup 2 and sca 1 end with one landmass of 0.27 to 0.29 of the planet and the second
under 0.02; the run-to-run sd of the largest landmass is 0.033 → 0.117, of land 0.021 → 0.057), and
sup 1 goes the other way: its sea ends 608 m high with 0.60 of its continental crust under it. The
continental crust falls by 0.026, in three runs of four (the ensemble's p50, 0.369 → 0.359).

**The water amount.** Earth's, 2,664 m (ISO-085), is kept: the coordinator's call to this task
(2026-10-21), made on Earth's numbers after the continents diagnosis found that a sea holding
Earth's water would have drowned more land on master (0.12 to 0.22 on Ben's run). That holds over
master's floor: the volume sea over the old dampening floor ends at land 0.219, as the census
predicts from the before runs (0.165 to 0.205). Over GDH1's floor with the first ocean in steady
state, the basins hold Earth's ocean with land 0.292 at the end (0.200 to 0.358 by run; seeds 1, 2
and 3, the mean of both starts, 0.264, 0.302 and 0.310) and 0.301 over the plate frames, against
Earth's 0.29. A planet with less water than Earth's would be a calibrated knob, and it would hide
the drowned crust that the census measures; none is needed.

What the numbers say:

- **The Phanerozoic check fails.** Past the first 100 Myr the sea stands 450 to 1,380 m below the
  end at its lowest, against Earth's 100 m, and up to 14 to 734 m above it, against 250. With the
  first ocean in steady state there is no spin-up any more (over the whole run the range is the
  same within 25 m). The sea starts 280 to 375 m below the datum, and the laws say why: GDH1's
  floor over the steady-state spread is 4,844 m deep on average, so an ocean of 0.577 of the
  planet (the presets' 0.423 of continental crust, all of it dry at the start) holds 2,795 m of
  water at the datum, 131 m more than Earth's, and the sea stands about 230 m low before the
  margins' slopes are counted. Earth holds the same water at its datum with sediment, plateaus and
  arcs in its basins (from memory); a new first ocean has none of them, and the plate model's
  floor gains them as it ages.
- **What moves it is the plate model's own variability, passed on as the volume law should.** The
  continental crust's share wanders from 0.42 at the start to 0.27 to 0.38 at its lowest and up to
  0.48 (the plate check's 0.37 to 0.45 already failed before, 0.32 and 0.30 at their lowest; 0.31
  and 0.28 now), and the ocean floor's median age by a standard deviation of 7 to 13 Myr (41 to
  105 Myr within sup 1). By the fill's arithmetic, turning 0.01 of the planet from continent to
  ocean floor about 4.5 km deep makes room for about 65 m of sea, and ten Myr more median age about
  150 m under GDH1; the land ice, 41 to 87·10⁶ km³ at its most against 17 to 56 at the end, about
  2.6 m per 10⁶ km³. Per run, a linear fit of the sea against the three explains 0.09 to 0.82 of
  its variance past the first 100 Myr (0.49 on average); the rest is not traced (sediment, the
  continents' heights, the oceanic label's arcs). Sup 1's last 150 Myr show the scale: the floor's
  median age falls from 77 to 56 Myr, the continental share rises from 0.39 to 0.41 and the ice
  shrinks from 32 to 17·10⁶ km³, about 300, 110 and 40 m by that arithmetic of the 670 m the sea
  rises.
- **The end stands high.** In five runs of six the end stands 48 to 560 m above the run's median
  sea, and in all six the floor is younger at the end (median age 44 to 56 Myr, against 57 to 66
  over the run), as it was in the six before runs: the plate model's, not traced here. On Earth the
  present is near a low stand (ice).
- **The flooding of the continents is mostly the crust, not the sea level.** Before this change
  the sea stood within 153 m of today all the time, and still 0.32 (p50 of the frames) to 0.35
  (final) of the continental crust lay under it, against Earth's 0.19. To lay it as dry as
  Earth's, the sea would have had to stand at the continental crust's 19th height percentile,
  220 to 1,240 m below the sea on the 24 epoch checkpoints: the continental crust stands too
  low, not the sea too high. After the change that height still stands below the sea on 22 of the
  24 checkpoints (a median of 218 m below it, 676 m before; −1,055 to +97 m): against the datum
  the crust's 19th percentile stands about 170 m higher than before (median −523 m against −695)
  and the sea about 160 m lower (−228 against −66). The drowned share falls to 0.309 at the
  end and 0.250 over the plate frames only because the sea stands about 250 m below the end for
  half the run, which a crust that stood higher would not need.
- **Land and its shape hold.** Land rises (map 0.267 → 0.317, bake 0.266 → 0.292, Earth 0.29), the
  land above 2 km (18.0 → 22.6 % of the land, Earth 10.2) and the coast distance with it (map p50
  311 → 394 km, Earth 433). The largest landmass over the plate frames is 0.128 → 0.151 of the
  planet (p50; Earth's Afro-Eurasia 0.167) and the second 0.056 → 0.071 (the Americas 0.083); at
  the end three runs hold one landmass of 0.27 to 0.35 with the second under 0.02, a
  supercontinent's moment.
- **The sea-floor lakes grow.** The plate era's closed basins on oceanic crust rise from 124 to 156
  per snapshot and from 34 to 38 % of the lake area. They stand on oceanic crust above the sea
  (their level p50 559 m over the datum; island arcs and the stacked crust the plate model keeps
  under the oceanic label, whose arc crests stand 1,330 to 1,360 m high at p90, 1,135 to 1,156
  before, against the plate check's 1,000), and a sea about 250 m lower for half the run lays
  more of that ground dry. The volume law was expected to fill them through the floor's kept
  sediment (rivers-and-lakes.md, "What keeps the plate era's lakes"); it does not, because the
  pockets are not under the sea.

### The flexure's water over GDH1's floor (review, 2026-10-21)

The review of this branch found the surface flexure loading a second time the sea that fills
GDH1's own deepening. GDH1 is the depth of ocean floor under water (Stein & Stein fit the depth
below the sea), so that water's weight is in its depths already; the flexure loads the change of
the sea's depth over each cell (ISO-061 to ISO-064), and on floor that follows GDH1 most of that
change is GDH1's own sinking. At strength 1 (the Earth-like presets) the floor then sank about
ρ_w/(ρ_m − ρ_w) = 0.45 of GDH1's subsidence more than GDH1, 1.45 times GDH1 in all. Since then the
flexure leaves the sea over each step's GDH1 sinking out of the water load on oceanic crust that
is not new at the place (ISO-088); the sea's own change, the water that erosion and deposition
displace and the water that thickened crust pushes aside still load the floor.

Measured on the continent bisect's four runs (`earthlike_quick_16k`, seeds 1 and 2, both starts,
1,000 Myr, 2 threads, the final frame) at the merged head bcf8cd8e (before) and at 975dd57e (after),
with `build/sl/floor.py` (scratch): area-weighted medians over the ocean-basin and ridge floor under
the sea of the depth below the datum less GDH1's for the cell's age, in m, positive deeper.

| Floor age | 0.45·(d − 2,600) | sca 1 | sca 2 | sup 1 | sup 2 |
|---|---|---|---|---|---|
| 0 to 20 Myr | about 490 (mean over the bin) | 370 → 47 | 337 → −141 | 308 → −118 | 146 → −168 |
| 20 to 50 Myr | about 950 (at 35 Myr) | 955 → −113 | 860 → −377 | 983 → −378 | 6 → −632 |

Before the fix three runs of four stood as deep as the double count predicts (sup 2's 6 m is not
traced). After it the floor older than 20 Myr stands above GDH1, and the rest is the floor's
sediment: on seed 1's two runs rerun with the sediment written out (`surface/sediment_m`;
`build/sl/basement.py`), the basement under the sediment stands deeper than GDH1 by

| Floor age | sca 1 | sup 1 | sediment after, mean (median), sca 1 and sup 1 |
|---|---|---|---|
| 0 to 20 Myr | 699 → 240 | 625 → 206 | 790 (19), 1,026 (17) |
| 20 to 50 Myr | 1,438 → 744 | 1,373 → 342 | 1,741 (138), 1,705 (142) |
| 50 to 100 Myr | 1,762 → 713 | 1,699 → 226 | 2,355 (502), 2,488 (785) |
| 100 to 150 Myr | 2,651 → 429 | 2,241 → 222 | 2,375 (972), 4,313 (1,991) |

What is left, 200 to 750 m, is what the flexure is there to carry (the sediment's weight, the
sea's change, the margins' loads beside the floor); it is not split further here. The ocean at the
end (`build/sl/odepth.py`): before, the sea −68 to 382 m over the datum, the ocean 0.667 to 0.753 of
the planet and 3,490 to 3,939 m deep on average; after, the sea 65 to 819 m, the ocean 0.698 to
0.776 and 3,350 to 3,742 m deep. Earth's is 3,682 m deep over 0.709 of the planet (Charette &
Smith 2010).

The basins hold less, so the same water stands higher, by a mean of 125 → 475 m at the end. In
the bisect's terms (the pooled rows, the means of the four runs):

| Commit | land | continental crust | under the sea | land above 2 km | largest | second | landmasses of 0.01 or more | their land | island arcs, of the land | pieces |
|---|---|---|---|---|---|---|---|---|---|---|
| bcf8cd8e (the merged head) | 0.294 | 0.334 | 0.296 | 0.308 | 0.165 | 0.062 | 3.8 | 0.882 | 0.399 | 272 |
| 975dd57e (ISO-088) | 0.268 | 0.345 | 0.373 | 0.255 | 0.163 | 0.038 | 3.0 | 0.858 | 0.431 | 311 |

By run, land and the continental crust under the sea: sup 1 0.307 and 0.292 → 0.224 and 0.506,
sup 2 0.247 and 0.384 → 0.258 and 0.401, sca 1 0.333 and 0.222 → 0.287 and 0.305, sca 2 0.288 and
0.288 → 0.302 and 0.281. By the bisect's rule (a land step under 0.03 is not a finding) the land's
−0.026 is not one (the run-to-run sd of land is 0.036 → 0.035); the drowned share's +0.077 is, and
sup 1 carries most of it (its sea ends 819 m over the datum). Earth: land 0.29, about 0.19 of the
continental crust under the sea.

The GameTests' 4k planet (`tests/presets/planet_world_4k.json`) has no flexure section, so its
flexure is off (ISO-065) and its bake is the same at bcf8cd8e and 975dd57e. Against master
593f3c58 (one bake each, `deeptime run`, the last checkpoint read by `build/sl/k4.py`): the sea
42.8 → 535.1 m, land 0.076 → 0.072 of the planet, the continental crust 0.383 → 0.400 and 0.848 →
0.857 of it under the sea, the largest landmass 0.061 → 0.027, coast land (land cells with a sea
neighbour) 0.0144 → 0.0231, sea within 100 m of the level 0.0574 → 0.0539, and land within a
sixteenth of the planet's width of the seam 0.026 → 0.053 of that band.

### The continents under the sea and the sea's height at the end (2026-10-22)

V1 item 1's third criterion asks that at most 0.25 of the continental crust lie under the sea in
the final frame; master df40ce80 has a median of 0.414 on twelve runs (the plate model's own
count, averaged over the last 500 Myr, 0.348). This section asks whether the measure
and the target are Earth's, why the sea ends about 450 to 540 m over the datum, and whether the
continents' hypsometry is Earth's. Nothing in the code changed in the measurement; the arms below
are scratch switches and presets set from the command line, on master df40ce80, where W (PLT-262)
was still kilometres on the planet. Its result went into the code the same day (Ben, D6): the last
subsection, "W on the model sphere, in the code", measures the shipped change.

**Runs.** `earthlike_quick_16k`, 1,000 Myr, seeds 1 to 6, both starts (twelve runs an arm), 2
threads, the crust budget on (`DEEPTIME_CRUST_BUDGET=1`), on the test VM through admission
(`tools/review/force-balance-ensemble.sh` and the continents census on the final frame, a
scratch driver in `build/dr`, not committed). The base binary is master df40ce80 built from a
`git archive`. Arms:

- *master*: the base.
- *nosteer*: `--set steering.enabled=false` (no end-weighted nudges; the present climate natural).
- *sub80*: a scratch patch to `plates/ridges.rs` (not committed): at a resampling, 80 % of the
  conserved pools (sediment, sedimentary, metamorphic and felsic rock) of an overridden oceanic
  column go to the mantle instead of accreting onto the column on top, when the top is oceanic
  and the overridden column is hidden or on another plate (ocean-ocean subduction); continental
  tops still take everything. After von Huene & Scholl 1991 (abstract): most sediment that
  reaches a trench is subducted.
- *wsph*: `--set tectonics.transition_width_km=24.5`, the transition's width W at 200 km on the
  Earth-sized model sphere instead of on the planet (PLT-262; the quick planet is 8.16 times
  smaller). This is the shipped meaning since 2026-10-22, where `transition_width_km=200` does
  the same; on df40ce80's code the key was km on the planet, so 24.5 stood for it.
- *w70*: `--set tectonics.transition_width_km=70`, between the two.

The analysis is a scratch script (`build/dr/an.py`) on the census, the dumped fields and the
runs' records: medians of twelve runs and a two-sided Mann-Whitney U over the runs. Many measures
were compared, so a single p near 0.03 to 0.05 is expected by chance.

#### Is the measure Earth's?

The census counts a cell as continental where `plates/crust_type` on the L1 map is over 0.5 and
as under the sea where its height is below the climate's final sea. The kind flag is the plate
model's, and at a rifted margin the model puts the boundary where the transition's target
thickness is 20 km (T* = T_o + (T_n − T_o)/2, `plates/transition.rs`, PLT-261): the taper is
centred on the boundary. Earth's like-for-like share is therefore the crust at least 20 km thick
(sediment to Moho) under the sea. On CRUST1.0 (PLT-361) that crust covers 0.384 of the planet and
0.248 of it is under the sea; at 25 km 0.195 (the "about 0.19" of the decisions log), at 30 km
0.101. A second pass that reads the rock's height instead of the water (`build/dr/earth.py`)
gives 0.237, 0.183 and 0.090. Cogley 1984 (abstract read) puts the continents at 210.4 million
km², 41 % of the surface, with 30.6 % of them submerged; Rudnick & Gao 2003 read him as 31 % to
the shelf break; Harrison et al. 1983 count the continents to the 200 m isobath, 33.5 % of the
planet.

So the target is consistent with Earth only at the model's own boundary, and with no margin:
an Earth drawn with the model's boundary scores 0.237 to 0.248, and by Cogley's
continent-ocean contacts Earth itself would fail it (0.306). The target is not changed (the
coordinator's and Ben's). Recommended beside it: the share of crust at least 25 km thick that
stands above the sea (`t25_dry` here), which does not depend on where the kind flag sits: Earth
0.805 to 0.817, master 0.572.

#### Why the sea stands about 500 m over the datum at the end

The water is Earth's (2,664 m as a layer, less the land ice: ISO-085, ISO-086), and the datum is
calibrated: Earth's water over a bare GDH1 floor stands 280 to 375 m below it (the first ocean's
start, "Earth against the mod"), and Earth's ocean sediment alone lifts its sea by 166 m (Conrad
2013) to 210 m (Dutkiewicz et al. 2017), with its plateaus, seamounts and margins on top, so an
Earth-like ocean puts the sea within a couple of hundred metres of the datum. The model's sea is
high because its ocean basins are shallow.

| Master, final frame, median of 12 | Value | Earth |
|---|---|---|
| The census's sea over the datum, m | 538 | 0 |
| The plates' step sea, median over the run, m | 210 | |
| Oceanic-labelled crust at least 10 km thick, share of the planet | 0.325 | crust 10 to 20 km thick of any kind: 0.074 (CRUST1.0) |
| The sea with that crust at GDH1's floor for its age, change, m | −1,445 | |
| The sea with the normal floor (under 10 km) at GDH1, change, m | −62 | |
| The sea without the surface's loose ocean sediment, change, m | −63 | |
| The sea with the floor's ages scaled to a 64 Myr mean, change, m | −167 | mean age 64.2 Myr (Seton et al. 2020) |
| Conserved pools in the ocean (felsic, sedimentary and sediment), Earth-normalized km³ | 1.6 to 2.1 × 10⁹ (steady after −600 Myr) | 3.37 × 10⁸ (GlobSed, Straume et al. 2019) |
| Sediment delivered to the sea, km³ a year (Earth-normalized) | 12.5 | about 5 of rock (14 Gt a year, Syvitski et al. 2005 via Walling 2006) |
| Sediment subducted at trenches | none | 1.0 to 1.5 km³ a year (von Huene & Scholl 1991, abstract) |

The thick oceanic-labelled crust is mostly sedimentary rock: on supercontinent seed 1, columns 15
to 20 km thick hold 4.9 km of sedimentary rock, 0.8 km of metamorphic and 1.1 km of felsic
plutonic rock on 10 km of mafic crust, columns 20 to 25 km 6.6, 1.9 and 2.0 km; they are 93 to 163
Myr old, within about 100 to 300 km (on the model sphere) of a continent, 0.6 of them in the
ocean-basin regime and 0.2 to 0.3 back-arc. The ocean holds five to six times GlobSed's
sediment because two to three times Earth's inflow arrives and none of it leaves: at every
resampling the conserved pools of every overridden column accrete onto the column on top
(`plates/ridges.rs`), so sediment rides the floor into a trench and comes back on the overriding
plate, and stays in the ocean when that plate is oceanic.

In the datum's frame the continents stand where Earth's do: the median surface crosses the sea
at 35.1 km of crust, 137 m per km, so it would cross the datum at 35.1 − 538/137 ≈ 31.2 km,
against Earth's 31.0 km (CRUST1.0, the median rock height against the crust's thickness).

**The sea at the end against the run.** The final sea stands over the run's median in 11 of 12
runs (sign test p about 0.006); the medians of the 100 Myr windows run 180 to 290 m from −900 to
−200 Myr and 456 to 506 m over the last 100 Myr, and the per-run trend over the last 800 Myr is
+28 m per 100 Myr at the median (positive in 9 of 12, weak). Part of the end height is the
frame the census reads: the plate model resamples "once at the end so the final crust has no gaps
or hidden slabs" (`plates/mod.rs`), and the climate computes the final sea after it. In all 12
runs that sea stands 48 to 118 m (about 80 at the median) over the plates' last-step sea, and the
floor's median age drops at the last step in 12 of 12 (to 49 Myr, against about 61 over the
run). Between resamplings the step sea is a saw: about 90 m up the step after a resampling, about
10 m down a step after it. With 80 % of the ocean-ocean accretion gone (sub80) the jump stays
(the medians of the final sea and the last step's 74 m apart), so it is not that accretion; which state
of the saw is the truer one is not settled, and the criterion is the final frame's, so nothing
was changed.

**Not the steering.** Without the end-weighted nudges (nosteer) the share under the sea is 0.402
against 0.414 (p 0.89) and the census's sea 598 m against 538 (p 0.75); nothing in V1 item 1
moves at p < 0.6.

**Sediment subduction (sub80).** Taking 80 % of the ocean-ocean overrides' pools to the mantle
lowers the sea by about 720 m and leaves the continents as drowned:

| Median of 12 | master | sub80 | p |
|---|---|---|---|
| The census's sea over the datum, m | 538 | −181 | < 0.001 |
| The plates' step sea, median over the run, m | 210 | −156 | < 0.001 |
| Continental crust under the sea | 0.414 | 0.412 | 0.56 |
| Land | 0.247 | 0.216 | 0.001 |
| Continental crust's net change over 1,000 Myr, share of the planet | −0.085 | −0.132 | < 0.001 |
| Continental crust's area at the end | 0.338 | 0.290 | < 0.001 |
| Continental crust's mean thickness, km | 39.5 | 34.6 | < 0.001 |
| Where the median surface crosses the sea, km of crust | 35.1 | 30.0 | < 0.001 |

The continents lose a quarter of their volume and sink about as far as the sea falls (5 km of
crust at 137 m per km is about 680 m). The sediment that erosion takes off the continents comes
back to them only through the ocean: by accretion at resamplings and by stacked ocean crust
turning continental (PLT-289). Take most of it to the mantle and nothing in the model makes it
back; on Earth arc magmatism adds about as much crust as subduction takes, about 3.2 km³ a year
(Stern & Scholl 2010). So the high sea and the drowned continents are one budget, and sediment
subduction alone is not a fix: it needs the continents' growth from arcs first (the plate and
crust-budget tasks). The variant with continental tops subducting too was not run, since it can
only lose more crust.

#### The continents' hypsometry

Master's continental crust against Earth's crust at least 20 km thick (CRUST1.0; the census's
final frame, medians of 12, heights over the sea):

| Measure | master | Earth |
|---|---|---|
| Surface over the sea, mean / median, m | 381 / 260 | 311 / 220 |
| Land's mean height, m | 1,284 | 797 (the 2012 ETOPO1 estimate, via Simpson & Simpson 2018) |
| Share 0 to 200 m under the sea (the shelf) | 0.104 | 0.112 |
| Share 200 to 2,000 m under (the slope) | 0.200 | 0.148 |
| Share deeper than 2,000 m | 0.089 | 0.010 |
| Passive margins: the slope between 500 and 2,000 m, median | 2.5° | 2.1° (ETOPO 2022, PLT-265) |
| Passive margins: shelf width, median | 0 km | 57 km, passive 84 to 88 km (Harris et al. 2014) |
| Passive margins: shore to the 200 m isobath, median | 5.0 km | 25 km (ETOPO 2022, PLT-265) |
| Passive margins: the shelf break | 187 m | about 130 to 140 m (secondary sources; Harris et al. 2014 give none) |

The shelf is about Earth's; what is extra is the crust on the slope and below it, 0.29 of the
continental crust against Earth's 0.16, and nearly none of it is far from the sea: 97 to 100 % of
the continental crust lies within 100 km (on the planet) of oceanic crust. That is the
transition's continental half. W was 200 km on the planet (PLT-262, Ben's answer of 2026-10-05),
which is 1,632 km on the Earth-sized model sphere the plate model runs on, eight times Earth's
margins (Harris et al. 2014: passive shelf and slope 134 km). Over the continental half of the
taper the target thickness runs from 20 km at the boundary to 33 km at W/2 (100 km on the
planet), all of it under the 35.1 km at which the surface crosses the sea, so the taper's
continental half is under the sea wherever nothing thickened it. With W at 200 km on the model
sphere (wsph):

| Median of 12 | master | wsph | p | w70 | p |
|---|---|---|---|---|---|
| Land (0.26 to 0.32) | 0.247 | 0.274 | 0.002 | 0.253 | 0.51 |
| Largest landmass (≥ 0.12) | 0.104 | 0.125 | 0.073 | 0.080 | 0.84 |
| Continental crust under the sea (≤ 0.25) | 0.414 | 0.279 | 0.002 | 0.384 | 0.40 |
| Land above 2 km, of the land (≤ 0.12) | 0.248 | 0.277 | 0.40 | 0.222 | 0.80 |
| Coast with high ground (≤ 0.15) | 0.369 | 0.403 | 0.39 | 0.328 | 0.77 |
| Land under ice (≤ 0.15) | 0.170 | 0.159 | 0.89 | 0.143 | 0.93 |
| Collisions per 100 Myr (2 to 6) | 1.65 | 1.85 | 0.43 | 1.70 | 1.0 |
| Continental crust's net change over 1,000 Myr | −0.085 | −0.064 | 0.13 | −0.060 | 0.11 |
| Criteria met per run | 1 | 2 | 0.035 | 2 | 0.25 |
| Under the sea, the plate model's mean over the last 500 Myr | 0.348 | 0.236 | < 0.001 | 0.308 | 0.14 |
| Under the sea, the plate model's mean over the frames after the first 100 Myr | 0.362 | 0.243 | < 0.001 | 0.281 | 0.001 |
| Land, the plate model's mean over the same frames | 0.282 | 0.320 | < 0.001 | 0.306 | < 0.001 |
| The plates' step sea, median over the run, m | 210 | 413 | 0.24 | 210 | 0.98 |
| Crust at least 25 km thick above the sea (Earth 0.81) | 0.572 | 0.683 | 0.014 | 0.578 | 0.98 |
| The census's sea over the datum, m | 538 | 826 | 0.069 | 900 | 0.053 |
| Continental crust's mean thickness, km | 39.5 | 42.9 | < 0.001 | 41.3 | 0.002 |
| Slope share (200 to 2,000 m under) | 0.200 | 0.126 | < 0.001 | 0.183 | 0.089 |
| Deeper than 2,000 m | 0.089 | 0.073 | 0.017 | 0.080 | 0.24 |
| Passive margins: slope between 500 and 2,000 m, median | 2.5° | 3.2° | | 2.8° |  |
| Passive margins: shelf break, m | 187 | 213 | | 189 |  |

The narrower taper keeps the continents' crust in their cores: they are thicker for their area
and stand higher (the continental crust's mean height over the sea 381 to 822 m), the sea rises
with them by less, and the share under it falls to 0.279. The slope share falls to below Earth's;
the slope steepens by 0.7°, not the 1.9° the decision of 2026-10-05 weighed: that 4.5° was the
one-sample step with the transition off (W = 0), not a narrower one, and the surface's slope
failure and sediment now shape the slope too.

W at 70 km sits between on the frames' mean (0.281, p 0.001) but not on the final frame (0.384,
p 0.40): its sea climbs through the run (the medians of the 100 Myr windows from about 0 m at
−900 Myr to 741 m over the last 100 Myr, against master's 180 to 290 m and then 456 to 506 m),
and the final frame catches it at its highest. The higher, thicker continents shed more
sediment, and none of it leaves the ocean (above), so the basins fill faster the narrower the
taper, and the final frame's share rewards a narrower taper less than the run's mean does. Its
passive slopes are 2.8° (the steepest 4.5°), its shelf 0.5 km wide at the median and the 200 m
isobath 6.0 km from the shore.

By run, under the sea, master / wsph / w70: super-1 0.387 / 0.379 / 0.371, super-2 0.445 /
0.210 / 0.275, super-3 0.324 / 0.230 / 0.444, super-4 0.418 / 0.405 / 0.375, super-5 0.322 /
0.409 / 0.393, super-6 0.412 / 0.269 / 0.190, scatt-1 0.428 / 0.261 / 0.424, scatt-2 0.417 /
0.273 / 0.393, scatt-3 0.527 / 0.270 / 0.333, scatt-4 0.394 / 0.324 / 0.448, scatt-5 0.420 /
0.285 / 0.245, scatt-6 0.361 / 0.364 / 0.492.

W was Ben's: he chose 200 km on the planet to keep Earth's slopes in the game's metres. The
measurement said the plate model needs Earth's width in its own frame to keep Earth's share of
continental crust above the sea, and that the game's slopes pay less for it than was thought.
Recommended to him: W at 200 km on the model sphere (24.5 km on the quick planet), or the taper
in the model's frame and the drawn margin in the planet's, if the slope matters more (a larger
change: the L1 map and the surface would draw the margin wider than the crust's taper). He chose
the first the same day (decisions log 2026-10-22, D6: "200 km on sphere"); see the next
subsection.

#### W on the model sphere, in the code (2026-10-22)

`transition_width_km` is now kilometres on the model sphere (`plates/transition.rs`, `width_rad`
= `Norm::rad`; PLT-262), 200 in every preset, and PLT-400 (`collision_restack_whole`) is on in the
default and every built-in preset, the coordinator's call (bookkeeping that moved no criterion).
Measured as above (12 runs an arm; `earthlike_quick_16k`, 1,000 Myr, seeds 1 to 6, both starts, 2
threads, the crust budget on), the base master 10bafdae (W on the planet, PLT-400 off; it
reproduces df40ce80's medians to the digit) against this branch, with a W-only arm
(`--set tectonics.collision_restack_whole=false`) beside them; medians, p from a two-sided
Mann-Whitney U over the runs against the base:

| Median of 12 | base | arm | p | W only | p |
|---|---|---|---|---|---|
| Land (0.26 to 0.32) | 0.247 | 0.300 | 0.001 | 0.300 | 0.001 |
| Largest landmass (≥ 0.12) | 0.104 | 0.124 | 0.34 | 0.158 | 0.069 |
| Continental crust under the sea (≤ 0.25) | 0.414 | 0.278 | < 0.001 | 0.263 | 0.001 |
| Land above 2 km, of the land (≤ 0.12) | 0.248 | 0.271 | 0.80 | 0.267 | 0.40 |
| Coast with high ground (≤ 0.15) | 0.369 | 0.411 | 0.033 | 0.404 | 0.078 |
| Land under ice (≤ 0.15) | 0.170 | 0.170 | 0.93 | 0.174 | 0.47 |
| Collisions per 100 Myr (2 to 6) | 1.65 | 1.80 | 0.84 | 1.60 | 1.0 |
| Criteria of the seven met, per run | 1 | 2 | 0.018 | 2 | 0.010 |
| Under the sea, the plate model's mean over the last 500 Myr | 0.348 | 0.215 | < 0.001 | 0.247 | < 0.001 |
| Crust at least 25 km thick above the sea (Earth 0.81) | 0.572 | 0.677 | 0.001 | 0.693 | 0.002 |
| The census's sea over the datum, m | 538 | 820 | 0.030 | 839 | 0.069 |
| Continental crust's mean height over the sea, m | 381 | 737 | 0.046 | 747 | 0.026 |
| Continental crust's mean thickness, km | 39.5 | 43.4 | < 0.001 | 44.4 | < 0.001 |
| Slope share (200 to 2,000 m under) | 0.200 | 0.115 | < 0.001 | 0.115 | < 0.001 |
| Deeper than 2,000 m | 0.089 | 0.069 | 0.005 | | |
| Passive margins: slope between 500 and 2,000 m, median | 2.5° | 3.1° | | 3.0° | |
| Passive margins: the steepest slope, median | 4.4° | 5.1° | | 4.8° | |
| Passive margins: shelf break, m | 187 | 208 | | 203 | |
| Passive margins: shore to the 200 m isobath, km | 5.0 | 4.0 | | 4.0 | |

By run, under the sea, base / arm: super-1 0.387 / 0.280, super-2 0.445 / 0.347, super-3 0.324 /
0.287, super-4 0.418 / 0.224, super-5 0.322 / 0.197, super-6 0.412 / 0.248, scatt-1 0.428 /
0.412, scatt-2 0.417 / 0.248, scatt-3 0.527 / 0.322, scatt-4 0.394 / 0.288, scatt-5 0.420 /
0.275, scatt-6 0.361 / 0.217: every run falls, and five of the twelve end under 0.25.

The scratch arm's result holds on the shipped code: 0.414 to 0.278 against 0.279, the land and the
largest landmass in their bands (0.300 and 0.124), the slope 0.6° steeper. The width does not
reach the criteria on the high ground (0.271 of the land above 2 km; the coast 0.411, p 0.033
worse), the ice or the collisions. The land's median is 0.026 above the scratch arm's 0.274 at the
same W to 0.3 % (24.5 km on the planet is a taper 0.26 % narrower than 200/6,370 rad); the W-only
arm gives 0.300 too, and any change reshuffles the plates, so medians of twelve carry about 0.02
on the land and the drowned share. PLT-400 changes nothing at this size (against W only, no
measure compared has p under 0.17). The sea ends higher (820 m over the datum against 538, the crust under the
sea's 0.278 still over 0.25): the thicker continents shed more sediment and none leaves the ocean
("Why the sea stands about 500 m over the datum at the end"), so the sea's height, not the
taper, is what remains between the final frame and its target.

### Freeboard: why the normal continent stands under the sea (2026-10-31)

The coast-with-high-ground audit (docs/research/notes/coast-high-ground-audit-2026-10-30.md) found
the sea standing about 150 m over the normal continental column, and Ben chose the order
"Freeboard, then orogen (Recommended)" (decisions log 2026-10-30). This section measures why, on
base33 (master 2abbc567, key 31c7c70bd7c7, 24 runs) and base32, with the M5 lane's tools on the
cached runs (`tools/review/m5diag/m5sea.py`, `tools/review/seddiag/sedstock.py`,
`tools/review/loss-budget.py`), and reads two arms of switches the M5 lane built and left off.

**The sea's budget** (m5sea, final frame, medians): the sea stands 536 m over the datum on base32 and
701 m on base33. With oceanic crust of 10 km and more at the bare floor's height for its age, it
would stand at −253 and −219 m: the thick oceanic-labelled crust raises the sea by about 790 and 920
m. From base32 to base33 (the reversal behind the overriding block, PLT-737) the filled sea rose 154
m, of it 128 m from the oceanic floor's height (oceanic crust 15 to 25 km thick from 0.123 to 0.141
of it, over 25 km from 0.045 to 0.056), while the floor's age alone (older) would have lowered it
by 46 m.

**The sea's sediment** (sedstock, base33, Earth-normalized km³, against GlobSed, Straume et al.
2019):

| region | model | GlobSed |
|---|---|---|
| the sea's sediment, all of it | 6.82 × 10⁸ | 3.37 × 10⁸ |
| wet continental crust (the margins) | 6.38 × 10⁷ (pile 1,774 m) | 1.43 × 10⁸ (3,044 m) |
| ocean floor within 200 km of continental crust | 2.97 × 10⁸ (4,061 m, over 7.1 × 10⁷ km²) | 7.6 × 10⁷ (2,049 m, over 3.7 × 10⁷ km²) |
| ocean floor beyond 200 km | 3.14 × 10⁸ (1,385 m) | 1.13 × 10⁸ (404 m) |
| young open floor (under 82 Myr, beyond 200 km) | 466 m, 3.45 times GlobSed's eq. 2a | |

Of the 6.15 × 10⁸ km³ on oceanic crust, 5.27 × 10⁸ is terrigenous, and 5.2 × 10⁸ lies on floor within
500 km of a passive margin (0.457 of the wet oceanic floor, a 3,591 m pile). With Earth's floor ages
the oceanic part would be 0.82 of itself: the old floor is a small part of the excess.

**The crust cycle** (loss-budget.py, km³ a year over 1,000 Myr, base33's seeds 1 to 4, against
Stern & Scholl 2010's Table 1): arcs 1.85 (Earth 2.5), rifts, hotspots and rifted margins 0.04
(0.5), crust-suturing zones 0 (0.2); sediment subduction 1.27 (1.1, PLT-440's 0.12 calibrated to
it), subduction erosion 0.51 (1.4); the net change of the continental volume −0.66 (Earth about 0).
The surface stage carries 6.6 km³ a year off the continents, and resamplings accrete 6.4 km³ a year
of ocean-floor rock back onto the columns on top, the model's main return path.

**Arm sedsub** (`tectonics.sediment_subduction_sourced`, PLT-660 alone, Clift & Vannucchi 2004's
shares; 8 runs, key 31c7c70bd7c7, argkey 10c53dc8, against base33-s4, base33's seeds 1 to 4): the
sea after 100 Myr and at the end 541 / 687 to 157 / −148 m; the continental crust's net change
−0.033 to −0.116 (p < 0.001); land 0.300 to 0.253 (p 0.007); coast with high ground 0.114 to 0.134
(p 0.72); land under ice 0.101 to 0.053 (p 0.007); the guard clean (worst 10,416 m). The slab takes
4.23 km³ a year (Earth 1.1) and the net change of the continental volume is −3.08 km³ a year: at
Clift & Vannucchi's shares the trenches take four times Earth's flux, because the ocean's stock
reaches them, and the resamplings' return falls from 6.4 to 2.4 km³ a year. As on 2026-10-22
(sub80, above), sediment subduction alone drains the continents.

**Arm sedmargin** (PLT-660 with `tectonics.margin_sediment_continental`, PLT-661 and PLT-662; the
class blend PLT-663 closed since 2026-10-25; argkey 5818ecce, against base33-s4):

| measure, medians of 8 | base33-s4 | sedmargin | p |
|---|---|---|---|
| sea over the datum, after 100 Myr / at the end, m | 541 / 687 | 174 / 169 | |
| continental crust, share of the planet (m5sea) | 0.388 | 0.487 | |
| its mean thickness, km (p10) | 38.1 (33.3) | 29.9 (18.7) | |
| continental crust under the sea (target ≤ 0.56) | 0.267 | 0.527 | < 0.001 |
| land | 0.300 | 0.238 | 0.002 |
| land above 2 km | 0.046 | 0.013 | 0.050 |
| coast with high ground (target ≤ 0.11) | 0.114 | 0.064 | 0.038 |
| collisions per 100 Myr | 3.90 | 5.00 | 0.040 |
| the guard's worst, m from the sea | 10,518 | 9,964 | 0.038 |
| crust 30 to 35 km: share, plate height over the sea p50 | 0.081, −264 m | 0.540, +41 m | |
| crust 35 to 40 km: share, plate height over the sea p50 | 0.602, +136 m | 0.114, +591 m | |
| the open floor beyond 200 km: pile (GlobSed 404 m) | 1,385 m | 416 m | |
| the sea's sediment, km³ (on continental crust) | 6.82 × 10⁸ | 6.92 × 10⁸ (3.8 × 10⁸) | |
| sediment subduction, km³ a year (Earth 1.1) | 1.27 | 2.60 | |
| net change of the continental volume, km³ a year | −0.66 | −0.98 | |

So the two together give the normal continent Earth's freeboard (the shoreline at 30 to 35 km of
crust, as Earth's 31 km) and the open ocean Earth's sediment, but the converted margins spread the
continents' volume over a quarter more area (0.388 × 38.1 against 0.487 × 29.9 km), so the land is
lower and thinner, and the sea's sediment is relabelled onto converted margin crust rather than
removed (1.24 × 10⁸ km² of wet continental crust against GlobSed's margins' 4.69 × 10⁷).

**The high ground with the margins converted** (the advisor's maps script,
`/data/ban/deeptime-cache/advisor/adv_maps.py`, on both arms' maps 50 Myr before the end): the land's
mean height 562 to 373 m, its p99 3.46 to 2.87 km; the land above 2 km a median 253 to 186 km from
the sea, beyond 400 km 0.244 to 0.050 of it (Earth 0.74); the high coasts on landmasses under 1
million km² 0.073 to 0.232. The interiors thin as well as the margins (crust 35 to 40 km thick 0.602
to 0.114 of the continental crust), and the crust cycle's rows move together: the transition adds
0.59 km³ a year to the continents (−0.47 on base33-s4), the thickening limit 0.78 (−0.01), erosion
takes 3.82 (6.61), collapse adds 0.30 (1.43); no one row explains the thinning (traced below,
"Why the continents thin with the margin's sediment continental").

**Neither switch goes on by default** (the coordinator, 2026-10-31): PLT-660 alone drains the
continents, and with PLT-661 the planet loses most of its high ground and its interiors, through a
path not yet traced, while the stock it was meant to clear is relabelled rather than removed. What
the arms do show: the sea's height is the ocean's sediment stock, and Earth's freeboard comes back
when the margin's sediment is continental crust and the trenches take Clift & Vannucchi's share.

#### Why the continents thin with the margin's sediment continental (traced 2026-10-31)

Ben turned both switches on ("Turn both on now", decisions log 2026-10-31) with the thinning
untraced. This traces it on the cached runs alone (no simulation): `tools/review/freeboard/thinning.py`
over the crust-budget maps every 100 Myr of base33-s4 (both off), sedmargin (both on) and sedsub
(PLT-660 alone), 8 runs each, medians. It splits the continental crust into the converted margins
(under 10 km of crystalline crust of its own, PLT-661's line), the edges (the rest within 300 km of a
margin or ocean sample) and the interiors (300 km and more in), and counts every sample's rock
whatever its label.

**The interiors follow the sea.** A continental column 300 km or more from the edge that no orogeny
raised loses crust only to erosion, which works on its height over the sea (the surface's feedback
strips the column from the top; Cortial's ε_c·(z/z_c) on the plate step, PLT-127, PLT-128). The other
movers do not reach it: the collapse takes from columns an orogeny raised, or thicker than 50 km, or,
with the sheet on crust no orogeny raised (PLT-581), standing above the room column's height, 525 m
over the sea, which these interiors' medians do not reach; the lower-crustal flow and the founder
take from crust over 50 and 70 km, the thickening limit from columns past the step's ceiling, the
transition within W/2 of the edge (one ring on the quick planet), and the resampling's class blend
(PLT-663) averages like columns. The interiors start at 35.1 km and 501 m over a sea at the datum:

| interiors no orogeny raised | base33-s4 | sedmargin | sedsub |
|---|---|---|---|
| the sea at the end, m over the datum | 687 | 169 | −148 |
| height over the sea, m: t−900, −700, −500, −300, end | 296, 162, 149, 137, 158 | 401, 154, 115, 66, 90 | 181 (t−800), 220, 317 (t−200), 372 |
| thickness at the end, km | 38.2 | 33.1 | 32.9 |
| T − 7 (z − S), the column at the sea (PLT-281), km | 36.0 | 32.4 | 30.1 |
| T_n + 7 (S − z_0), z_0 = 525 m | 36.2 | 32.6 | 30.4 |
| all interiors: thickness, km (height over the sea, m) | 39.2 (+356) | 34.0 (+101) | 33.3 (+439) |

On base33-s4 the sea rose to 687 m and the interiors rose with it: deposits, and the collapse's room
column T_n + 7 S (PLT-412), 39.9 km under that sea, the column the collapse fills its room up to and
at which it turned ocean floor continental. The interiors made from ocean floor (an ocean basement of
3.4 km on average) end at 40.0 km and +467 m, at that column's +525 m. On sedmargin the sea stays
under 210 m, the room column is 36.3 km, and the interiors erode down to the sea: 35-40 km becomes
30-35 km because base level moved 518 m (3.6 km of crust at 7.0 m a metre), and the interiors
stand 90 to 160 m over the sea on both arms. It holds run by run: those interiors end at 37.6 to 38.8
km on every base33-s4 run and 32.7 to 34.0 km on every sedmargin run, 10 to 380 m over their own sea,
and the column at the sea follows each run's sea (sedmargin's scattered_continents-1, a sea 20 m
over the datum: 31.4 km against T_n + 7 (S − z_0) = 31.6). On sedsub the sea falls through the run and the interiors
lag it: as thin as sedmargin's but 372 m over their sea. This is Airy isostasy with erosion and
deposition to base level, laws the model had: base33's thick interiors were its high sea's.

**The edges** (sedmargin against base33-s4, the end; height over the sea p50):

| | base33-s4 | sedmargin |
|---|---|---|
| converted margins: share of the planet, thickness | 0.003, 15.2 km | 0.076, 17.1 km (own crystalline 3.4, sediment 6.3, basement 7.4), −2,902 m |
| edge under 100 km in | 35.9 km, −28 m, land 0.45 | 28.3 km, −1,079 m, land 0.15 |
| edge 100 to 200 km | 39.2 km, +241 m | 32.4 km, −79 m |
| edge 200 to 300 km | 40.1 km, +331 m | 33.8 km, +27 m |
| share of the edges on an ocean basement | 0.90 | 0.89 |

The ring beside the margin is the taper's continental side (PLT-261, PLT-475), whose target at 35 km
in is 26.5 km of crystalline crust. On sedmargin it holds 22.0 km of its own over 4.7 km of basement,
26.6 km: the profile. On base33-s4 it held 33.2 km. There the collapse refilled it towards the room
column faster than the taper thinned it: the collapse moved 5.7 × 10⁹ km³ over the run against 2.2 ×
10⁹, and the transition passed 0.43 km³ a year of the continents' rock to the ocean side as ocean
crust (PLT-294) against 0.28. So on sedmargin the continent ends at the sea in a slope 1 km down
where base33's ended at the shore.

**The rock, whatever its label** (the felsic family, metamorphic, felsic plutonic and volcanic, and
the sediment and sedimentary rock, planet-mean km): 12.93 at the start; 13.56 on base33-s4 at the
end, 11.54 on sedmargin, 8.76 on sedsub. The crust budget's parts give the same change (the felsic
family's parts sum to +0.40 and −0.71 km³ a year), so no rock is counted twice or lost by a label
flip, PLT-667's turnbacks included (1.18 × 10⁹ km³ of margins' own crust turned back to ocean floor
over a sedmargin run). The 2.0 km between the two arms is the slab: sediment subduction takes 2.58
× 10⁹ km³ over a run against 1.27 × 10⁹. The trenches' floor carries a pile of 1.1 to 1.3 km (p90 4.0
to 4.3) against base33-s4's 1.9 to 2.0, a quarter to a third of it on floor that holds continental
crystalline rock (sediment margins turned back, PLT-667, or a spread's receivers under the ocean
label).

**The area.** The continental label covers 0.487 of the planet against 0.389: the converted margins
(0.076) and ocean floor that crust spread onto, continental at once under PLT-662 (supercontinent-1's
ocean floor made continental over the run, as shares of the planet: the thickening limit's releases
0.267 with 12.7 km of crust over the basement, 2,776 m down, the collapse's 0.055 with 14.6 km,
against base33's 0.123 with 28.2 km and 0.227 with 31.6 km, filled to the normal or room column
first).

**The mean thickness, 38.5 to 30.0 km**: the converted margins in the count take 2.4 km of it
(the rest, 0.413 of the planet, is 32.5 km); the interiors give 39.2 to 34.0 km and the edges 38.0 to
31.0 km.

**The high ground.** Collisions take the edges first, and on sedmargin those are the slope at 28 km,
the shelf at 32 km and the converted margins at 17 km (all continental crust, so they collide): the
crust crossing the fronts is 24.1 km against 32.9 (crossed 9.9 against 11.8 × 10⁹ km³ over a run, over
4.1 against 3.6 × 10⁸ km²), the belts stack thinner crust, and the collapse, fed by the belts, moves
2.6 times less.

**Reading.** No code path moves crust from the interiors to the margins, and no label flip counts
rock twice or drops it. The interiors thin by erosion to a sea 518 m lower, and lose the collapse's
fill that base33's high sea set (its room column 3.6 km thicker). The edges sit at the taper's
profile, a slope, with no refill. Both are the model's sourced laws once the sea is Earth's; they
are not an interaction of the margin switch, so nothing was built to undo them. What the arm does
lose for real is rock: 2.0 km of it planet-mean down the slab over the run (the continents' net change
−0.98 km³ a year against −0.66, arcs 1.65 against Earth's 2.5 and sediment subduction 2.6 against 1.1
to 1.65, Stern & Scholl 2010's Table 1 with Clift et al. 2009's figures as it quotes them). That drain and the belts built
from the slope's crust are the two leads it leaves (the build log of 2026-10-31).

### Why base35's continents stand low, against Earth (2026-10-31)

base35 (master 9d9e4c94, pool key 4dd28019a295, argkey cd665cef, 24 runs) against base34 (122e4803cded, 24 runs): land
0.227 to 0.209, continental crust under the sea 0.540 to 0.602 (the check: at most 0.56, RLC-032), the sea at the end
174 to 319 m over the datum (the final frames' climate sea, medians, the sea every number below uses; it carries the
steering's nudge, "The sea" below; the record's last step, which the crust budget's cb_t lines print, reads 154 to 316
m). This section measures the continents' crust and heights against Earth's, splits a continental column's height over
the sea into its terms, and splits the continental area into its sources through the run, on the cached runs only (no
simulation, nothing built). Tools: `tools/review/freeboard/lowcont.py` (modes `series`, `area`, `earth`, `model`) and
its VM driver `lowcont.sh`, beside `thinning.py`, `m5sea.py`, `m4cold.py` and `m5earthbands.py`; outputs in
`/data/ban/deeptime-runs/lowcont2/` on the VM. Earth: CRUST1.0 (Laske et al. 2013, 1 x 1 degree, the solid surface,
ice or rock, over the sea; cells by cos latitude) and SRTM15+ at 15' (Tozer et al. 2019) for the hypsometry. "Crust at
least 20 km thick" is the census's like-for-like continental crust (RLC-030: the model's kind flag sits where the
margin's target thickness is 20 km), read without labels on both sides. Revised after PR #56's review: the first
version read the area at -950 Myr as a share of the visible samples (below), read the line on the loaded surface, kept
Earth's ice sheets in the line, left the steering out of the fill and printed medians that do not add.

**The continents and the land against Earth** (final frames, medians; Earth's ice sheets out in brackets):

| measure | base35 | base34 | Earth |
|---|---|---|---|
| crust at least 20 km thick: share of the planet | 0.483 | 0.442 | 0.384 |
| its thickness, mean (p10, p50, p90), km | 31.8 (24.2, 33.2, 36.4) | 31.7 (24.8, 32.6, 36.5) | 36.4 (26.6, 36.9, 44.5) |
| its volume, planet-mean km | 15.4 | 14.2 | 14.0 |
| the planet's share at 10-30 km / 30-35 / 35-40 / 40-50 km | 0.274 / 0.272 / 0.059 / 0.012 | 0.245 / 0.252 / 0.063 / 0.008 | 0.149 / 0.077 / 0.115 / 0.104 |
| its share above the sea (under it) | 0.446 (0.554) | 0.505 (0.495) | 0.752 (0.248, RLC-030) |
| its height over the sea, mean / p50, m | -408 / -21 | -216 / +3 | +457 / +270 (+351 / +240) |
| continental label under the sea (the check) | 0.602 | 0.540 | 0.248 to 0.306 (RLC-030) |
| land, share of the planet | 0.209 | 0.227 | 0.291 (0.281) |
| land's mean / p50 height, m | 279 / 118 | 357 / 162 | 793 / 424 (655 / 383) |
| land above 1 km / 2 km, share of the land | 0.054 / 0.009 | 0.067 / 0.022 | 0.257 / 0.106 (0.199 / 0.055) |
| sea floor 0-200 m deep / 200-2,000 m, share of the planet | 0.086 / 0.145 | 0.080 / 0.124 | 0.047 / 0.064 |
| the sea over the datum, m | 319 | 174 | about 0 (the datum's calibration, above) |

base35 holds 10 % more crust at least 20 km thick than Earth by volume (15.4 against 14.0 km planet-mean; the ocean
basement under crust made continental counts) over a quarter more of the planet. Its mean thickness is 13 % under
Earth's and its p90 18 %; half of it lies at 30 to 35 km (Earth a fifth), and the planet's share at 35 to 50 km is a
third of Earth's (0.071 against 0.219), at 40 km and more a ninth (0.013 against 0.117). The land stands a third of
Earth's height; the shelf and slope cover twice Earth's sea floor.

**Where the area comes from.** Every run starts with 0.4225 of the planet continental at 35.1 km (the preset: (1 -
0.7) / (1 - 0.29), with the continents' flooded-shelf share of Cogley 1984; PLT-131's column), already 10 % over
Earth's 0.384, at 14.8 km planet-mean against Earth's 14.0 (6 % more volume) in a uniform column under Earth's mean
(36.4 km) and median (36.9), with none of Earth's 0.117 of the planet at 40 km and more. On Earth's own histograms and
water the start's area alone puts the sea 213 m over Earth's fill (lowcont.py `earth`). Through the run (`area`: the
crust budget's maps, every sample's thickness whatever its label, shares of the planet, medians over the runs):

| t, Myr | -1000 | -950 | -900 | -700 | -500 | -300 | 0 |
|---|---|---|---|---|---|---|---|
| base35, crust at least 20 km thick, share of the planet | 0.4225 | 0.431 | 0.437 | 0.452 | 0.463 | 0.470 | 0.481 |
| of it the start's continents (no ocean basement) | 0.4225 | 0.421 | 0.407 | 0.310 | 0.240 | 0.185 | 0.127 |
| of it crust on ocean floor with island-arc magma | 0 | 0.011 | 0.024 | 0.126 | 0.220 | 0.277 | 0.347 |
| of it other crust on ocean floor (belts, margins, the collapse's fill) | 0 | 0.001 | 0.005 | 0.015 | 0.007 | 0.004 | 0.001 |
| its mean thickness, km | 35.1 | 34.5 | 34.0 | 33.3 | 32.8 | 32.3 | 32.0 |
| its volume, planet-mean km | 14.8 | 14.9 | 14.9 | 15.1 | 15.2 | 15.2 | 15.4 |
| the planet at 35 to 50 km | 0.4225 | 0.167 | 0.106 | 0.086 | 0.086 | 0.078 | 0.078 |
| base34, crust at least 20 km thick | 0.4225 | 0.428 | 0.432 | 0.443 | 0.447 | 0.447 | 0.441 |

(Medians of each row, the "other" row the sum of two medians, so the rows need not add. Ocean-labelled crust at least
20 km thick is at most 0.008 of the planet. "No ocean basement": continental, with under 1 km of ocean crust under it
(Sample::basement); a resampling's blend can give a ring of the start's crust a basement. The maps at the end agree
with the final frames to 0.002.) The first 50 Myr add 0.008. The 0.48 to 0.50 that cb_t and `thinning.py` print at
-950 Myr, which the first version (and the review, from it) read as the start's outline, are shares of the visible
samples: between resamplings about a tenth of the planet is new floor the plates' samples do not yet cover (0.89 of
the planet visible at -950). The start's continents are turned over: the subduction step hides continental crust over
0.77 of the planet in 1,000 Myr (cb_parts, the label's area, shares of the visible samples, medians at the end), its
volume kept on the upper plate by the crust budget's restacking (cb_t's own-crust volume falls 5.6 %, 7.56 to 7.14 ×
10⁹ km³), while new area grows on ocean floor: matured island arcs 0.26 (PLT-047, PLT-760), the thickening's spread
onto the floor 0.25 (`rate`, PLT-287), the transition's margins 0.32 (most under 20 km), the collapse's fill 0.04
(PLT-254), the foundering 0.01, collisions 0.01. The crust on ocean floor stands at 31.6 km with its 7 km ocean
basement; the start's crust thins to 33.5 km by the end.

Since the area of that crust is its volume over its mean thickness, base35's end area over Earth's, 0.481 / 0.384 =
1.25, splits into factors (exact per run; our arithmetic here from the medians, which multiply to 1.25 only roughly):
the start's area 1.10 (its volume 1.06, its thinner column 1.04), the run's volume 1.04 and the run's thinning 1.095
(35.1 to 32.0 km). In logarithms the start carries 0.42 of the excess, the run's thinning 0.40 and the run's volume
(the crust budget's kept crust) 0.18. On base34 the start carries about 0.7, the thinning about 0.7 and the volume
about -0.3 (it loses 4 % by the end; from the medians, so the shares add only roughly): the same start and the same
thinning, with base35's budget keeping the volume base34 lost. On the fill: Earth's histograms at the model's volume
spread at Earth's mean thickness (0.418 of the planet, the mean over base35's runs) put the sea 185 m over Earth's
fill (means; base34 0.386 and 12 m), so the thinness is about 0.6 of base35's area term.

**The cratons' standing height.** The interiors no orogeny raised (300 km or more from an edge; `thinning.py`'s
"never" class) start at 35.1 km and +496 m over a sea at the datum; on base35 they stand +312 m at -900 Myr, +92 m at
-700 and +9 to +68 m over the sea from then to the end, at 33.8 to 34.1 km, their own crystalline crust 33.0 km at the
start and 31.0 at the end. At the end (`area`) they cover 0.083 of the planet, 0.86 of their samples dry, +68 m over
the sea p50 over all of them and +82 m over their land only, on 33.8 km, +363 m over the datum. Their column at the
sea, T - 7 (z - S), is 33.1 to 33.9 km, T_n + 7 (S - 525 m) as the thinning trace found. Earth's cratons (CRUST1.0,
m4cold.py's boxes, p50; the land only, then in brackets every cell of crust at least 20 km thick, wet ones too, with
that set's dry share): the East European platform 42 km at 150 m (42 km at 140 m, 0.88 dry), the Siberian platform 42
km at 360 m (42 km at 350 m, 0.98), the Canadian shield 39 km at 280 m (38 km at 190 m, 0.74: Hudson Bay), western and
central Australia 40 km at 290 m (39 km at 240 m, 0.83), the Brazilian shield 39 km at 304 m (39 km at 270 m, 0.89);
the Africa box, which holds the Ethiopian plateau and the rift shoulders as well as the cratons, 39 km at 620 m (39 km
at 550 m, 0.90). Read either way, Earth's cratons stand 140 to 360 m over the sea on 38 to 42 km of crust, the model's
interiors 68 to 82 m on 34 km. Earth's freeboard changed little through the Phanerozoic: "the freeboard must have been
close to zero at least during the Phanerozoic", which the next sentence reads as "the constancy of the freeboard", the
sea "less than 200 m" from today's, and cratons "such as Laurentia, were episodically flooded and exposed repeatedly
through the last 0.5 Gyr" (Korenaga, Planavsky & Evans 2017, full text, citing Wise 1974, Algeo & Seslavinsky 1995 and
Sloss 1963). Earth's drowned share of continental crust reached 0.528 at 80 Ma and 0.600 at 510 Ma on Marcilly et
al.'s polygons (RLC-031), which stand 0.044 over the crust-at-least-20-km measure today; less that offset its most
flooded Phanerozoic stand is about 0.556 (RLC-032), and base35's 0.602 is over it. From -700 Myr on base35's
continental crust is 0.42 to 0.63 under the sea (cb_t, medians).

**The split.** For the crust at least 20 km thick in 1-km bands b, with s_b the band's share of that crust, d_b its
dry share and m_b its mean height over the sea, the dry share D = Σ s_b d_b and the mean height H = Σ s_b m_b split
exactly per run into a thickness term Σ (s_M - s_E) x_E (the model's thicknesses on Earth's heights), a line term Σ
s_M (x_M0 - x_E) (the model's band with its sea at the datum against Earth's over its sea, at equal thickness: a
drowned column's height unloaded to its Airy height S + (z - S) / 1.454, ISO-078's water load in settle.rs, and
reloaded under a sea at 0) and a sea term Σ s_M (x_M - x_M0). Means over the runs, which add (the medians do not; on
base35 and base34 their residual is under 0.01 of the dry share and 45 m of the height):

| | model | Earth | thickness | line | sea |
|---|---|---|---|---|---|
| base35, dry share | 0.438 | 0.752 | -0.128 | +0.039 | -0.226 |
| base35, dry share, Earth's ice sheets out | 0.438 | 0.750 | -0.152 | +0.066 | -0.226 |
| base35, mean height, m | -415 | +457 | -188 | -336 | -348 |
| base35, mean height, Earth's ice sheets out | -415 | +351 | -311 | -106 | -348 |
| base34, dry share | 0.512 | 0.752 | -0.123 | +0.053 | -0.170 |
| base34, mean height, m | -209 | +457 | -191 | -302 | -173 |
| base34, mean height, Earth's ice sheets out | -209 | +351 | -287 | -99 | -173 |

Read on the loaded surface (the first version) the line term took about 55 m of the sea's (-390 against -336 m on
base35) and 0.027 of the dry share. Earth's ice tops stand over the continental crust of Antarctica and Greenland and
the model's surface carries no ice sheets: with the sheets out the line term of the mean height falls to about -100 m
on both arms, and the thickness term grows. The land on that crust, A x D, is 0.209 against Earth's 0.289 (means): its
larger area gives +0.071, its dry share -0.151.

The line at equal thickness (p50 of each 1-km band [T, T+1), m; base35 over its sea, over its datum as it stands, and
with its sea at the datum; PLT-281's line z_0 + (T - 35.1 km)/7.02 at the band's centre over the datum, derived; Earth
over its sea, ice sheets in and out):

| crust, km | 24 | 28 | 30 | 32 | 34 | 36 | 38 | 40 | 44 | 48 |
|---|---|---|---|---|---|---|---|---|---|---|
| base35 over the sea | -1,897 | -1,127 | -654 | -85 | 98 | 405 | 672 | 916 | 1,429 | 1,957 |
| base35 over the datum | -1,571 | -812 | -337 | 207 | 416 | 718 | 983 | 1,241 | 1,752 | 2,259 |
| base35, the sea at the datum | -1,441 | -666 | -203 | 232 | 416 | 718 | 983 | 1,241 | 1,752 | 2,259 |
| PLT-281's line (derived) | -985 | -415 | -130 | 155 | 440 | 724 | 1,009 | 1,294 | 1,864 | 2,433 |
| Earth over its sea | -780 | -70 | 10 | 198 | 410 | 430 | 340 | 380 | 437 | 820 |
| Earth, ice sheets out | -749 | -80 | 0 | 150 | 250 | 320 | 300 | 370 | 430 | 820 |

From 34 km up the model's columns stand on or under PLT-281's line (-24, -6, -26, -53, -112 and -174 m at 34, 36, 38,
40, 44 and 48 km; the first version read the line at the bands' lower edges, 71 m low, and put them above it). With
its sea at the datum the model's crust stands near Earth's at 32 km and 166 m over Earth's ice-free crust at 34 km. It
is not Earth's on either side: thin crust (20 to 30 km, mostly margins under the sea) stands 200 to 690 m deeper than
Earth's ice-free crust at equal thickness, which is most of the line term of the mean height but moves only how deep
drowned crust lies; and thick crust stands too high, 400 m at 36 km, 680 m at 38 km and 870 to 1,320 m at 40 to 44 km,
where Earth's height is flat with thickness (250 to 560 m from 34 to 44 km, the ice sheets out).

The terms of a column's height, against Earth:

- **The crust's thickness and density.** PLT-281's line is Airy for one crust density (2,830 kg/m³) from z_0 = 525 m
  at the normal 35.1 km; the pools enter it only through the eclogite root (PLT-278 to PLT-280), so sediment, felsic
  and mafic pools raise a column alike per metre. Earth's crust does not separate its high and low columns by density
  either: in CRUST1.0 (m4cold.py) the Basin and Range and Cordillera crust (mean 2,777 to 2,802 kg/m³) and the
  cratons' (2,776 to 2,832) are alike, so the crust's density is not the term (as the M4 trace found).
- **The mantle lithosphere's buoyancy.** Not in the model's height: ISO-110 (thermal isostasy) is built and off, "an
  open fault" (lithosphere-thermal-state.md); the keel K and the lithosphere's depth L (PLT-460 to PLT-463) set
  strength and heat flow only. The start's continents are cratons by the state (L at the frame, K 200 km) and stand on
  the same line as any column. On Earth the same crust stands 1.2 to 1.4 km higher over a hot lithosphere: the Basin
  and Range and Cordillera's land stands 1,407 m on 30 to 35 km of crust, 1,606 m on 35 to 40 and 1,834 m on 40 to 45,
  the cratons' 140 to 660 m on 35 to 45 km (m4cold.py, CRUST1.0, rock p50), the uppermost mantle 3,250 to 3,310 kg/m³
  under the Cordillera against 3,340 to 3,420 under the cratons; Hyndman & Currie 2011 (read in full earlier, PLT-281)
  put a hot back-arc at the sea on about 27 km of crust and a craton on about 38 km, about 1,600 m apart at equal
  thickness. PLT-281's one line, set through the global median, is therefore a mean of the two: on the model's datum
  it lies about 940 m over Hyndman & Currie's craton line at 38 km and about 630 m under their back-arc line at 27 km
  (lithosphere-thermal-state.md, "Why these frames", with the sea at the datum). On a craton line the start's 35.1 km
  continents would stand about 400 m under a sea at the datum ((35.1 - 38) km x 141 m/km, our arithmetic), so ISO-110
  alone, with the start as it is, would drown them further: the height law and the start's column go together. The
  keel does not cancel the cold: depletion lowers the cratonic roots' density by 1.1 to 1.5 % on average, and "the
  density variations due to temperature are partly (about 40%) compensated by density variations due to compositional
  differences. This conclusion contradicts the classical isopycnic hypothesis of Jordan" (Kaban et al. 2003, the
  abstract and the conclusions, pp. 66-67); Eaton & Perry 2013 (abstract) find the near-isopycnic state "inherently
  ephemeral".
- **Dynamic topography.** None in the model (physics-gap-audit.md lists it as a second-order gap). On Earth the Africa
  box stands 620 m (land only) against the other cratons' 150 to 360 m on the same 39 to 42 km of crust, with the
  Ethiopian plateau and the rift shoulders in it; Gurnis et al. 2000 (abstract) put southern Africa's residual
  topography at 300 to 600 m, and Hoggard, White & Al-Attar 2016 (abstract) observe "about ±500 m" at 10^4 km
  wavelengths. It explains part of Africa's extra height, not the other cratons' low stand.
- **The sea.** Filled on Earth's own CRUST1.0 surface with its own water (2,613 m over the planet), the fill gives +2
  m: a check of the binning (the water is read from the same surface), not a calibration. Swapping in the model's
  factors one at a time (Shapley over the water, the share of crust at least 20 km thick, that crust's height
  histogram and the thinner crust's; lowcont.py `model`), with the run's sea less its own fill as a fifth term (the
  residual: the steering's nudge, on in the preset, earthlike_quick_16k's band of 0.25 to 0.35 of land lowering the
  sea while the land is under it, with the open-sea rule, ISO-086, against it), base35's sea stands 292 m over Earth's
  fill (means, which add): the area of the thick crust +492 m, its lower heights -161, the water +54 (the planet's
  2,664 m less its ice against Earth's present ocean), the ocean floor and margins under 20 km -25, the residual -68
  (every run's own fill stands 18 to 109 m over its sea). base34: 148 m = +309, -116, +57, -67 and -36 (its runs 4 to
  92 m). The end seas and the dry shares are a steered sea's; without the residual the seas would stand 68 and 36 m
  higher on the means. The ocean-floor term is a net, not split here: the model's crust under 10 km stands at -4,606 m
  over the datum on average (0.375 of the planet; Earth's -4,363 m over 0.542), on a floor older than Earth's (mean
  106 Myr on base35, 88.5 on base34, m5sea.py; Seton et al. 2020's 64.2), and its crust of 10 to 20 km (converted
  margins and arcs) covers 0.141 of the planet at -3,154 m against Earth's 0.074. From base34 to base35 the filled sea
  rose 179 m, of it +204 m the continental area (m5sea.py's Shapley, which reads the continental label on the plate
  model's elevation where lowcont.py reads the crust at least 20 km thick on the surface's), from the crust budget's
  kept crust: the own-crust volume at the end 6.56 to 7.14 × 10⁹ km³ (cb_t lines), at the same mean thickness (26.0
  and 26.1 km own; 31.7 and 31.8 km for the crust at least 20 km thick); the crust at least 20 km thick 0.441 to 0.481
  of the planet at the end (`area`).

So the sea and the thin crust put base35's continents low, and both come from the continental area: 10 % more volume
than Earth's crust at least 20 km thick, 13 % thinner, over 0.48 of the planet instead of 0.38. The start sets about
0.4 of the excess area (0.4225 of the planet in a uniform 35.1 km column), the run's thinning about 0.4 (35.1 to 32.0
km: the start's crust taken under and kept as volume while the area regrows as island-arc crust on ocean floor at 31.6
km, and the start's own crust thinning to 33.5 km) and the crust budget's kept volume about 0.2; the sea follows the
area (0 to 157 m by -900 Myr, 316 m at the record's last step).

**Why the crust is thin: a hypothesis.** The model never holds Earth's thick crust: the start has none at 40 km and
more (Earth 0.117 of the planet), the run makes 0.013, and the start's 35.1 km, just over the 35 km edge, falls under
it within 100 Myr (0.106 of the planet at 35 to 50 km by -900 Myr). What thins it is not measured here. A hypothesis:
on PLT-281's line a column thicker than the column at the sea (31.4 km + 7.02 S, 33.6 km under base35's sea) stands
above the sea in proportion and erosion wears it toward that column, while thinner crust stands deep and fills; the
interiors reach the sea within 300 Myr and stay there, their column at the sea 33.1 to 33.9 km. The collapse is not
this mechanism: it fills its room up to T_n + 7.02 S (PLT-412, 37.3 km under a 319 m sea), not to the column at the
sea, and turns filled ocean floor continental (PLT-254, 0.04 of the planet over the run). On Earth thick cold crust
stands low: a 40 km craton stands about 280 m on Hyndman & Currie's craton line (141 m/km from 38 km, our arithmetic),
where CRUST1.0's cratons are, against 1,223 m on the model's line. If the hypothesis holds, a height law that holds
cold thick crust low keeps it, and the same volume covers less area.

**Erosion at low relief.** The interiors do not test K: from -700 Myr they sit at the sea, their base level, so their
slow loss of own crust (2.0 km in 1,000 Myr) is set by the base level, not by the erodibility, and it is basement
lost, not Clift's whole-basin rock export. The low-relief land does: the land in windows under 100 m of relief (0.68
of the land's window-time) lowers at 8.4 m/Myr net (stream power 4.9, diffusion 1.8; `erosion_bands` wz0, base35
median), 2.1 times the median of the ten basins that drain stable continent in Clift et al. 2009's Table 4 (the
library text, p. 93; 0.98 m/Myr for the Orange to 12.8 for the Mississippi over the Cenozoic, a median of 4.0, ours
from the table's cm/m.y.; the four orogenic systems 6.7 for the Alps to 169 for the Andes), and the land below 1 km of
height (0.94 of the land) erodes at 22 m/Myr (`erosion_bands` z0), 5.5 times that median and 1.7 times the fastest
stable basin; the windows with 100 to 250 m of relief erode at 48 m/Myr, Montgomery & Brandon's present-day eq. 2 (54
m/Myr there; ERO-004, ERO-179). Clift et al. 2009: "erosion will rapidly reduce mountains to altitudes of <1 km and
then more slowly remove the remainder as the land surface is reduced to the stable freeboard value above the ocean
surface (Harrison, 1994)", and erosion will "thin the crust back to the equilibrium values of ca. 38 km, close to
sealevel (Christensen and Mooney, 1995)" (p. 89); their long-term export, 1.77 km³ a year, "is much less than the
modern 6.5 km3/a estimated by Milliman (1997)" (p. 93), with "low lying, stable" land 83 % of the exposed continents
(p. 92) and "modern rivers are not in equilibrium" (p. 91). The cratons' long-run rates, from the sources the research
sub-agent read: the western Canadian shield "≤2.5 μm/yr since ca. 1.7 Ga" (Flowers et al. 2006, abstract); West Africa
"~ 2–20 m Ma–1" over three Cenozoic intervals, "weathering‐limited" (Beauvais & Chardon 2013, abstract); Namibian
outcrops 3.2 to 3.6 m/Myr, basins 5 to 16, large rivers 3 to 9, fission track 2 to 14 (Bierman & Caffee 2001, full
text); Australian granite 0.3 to 5.7 m/Myr (Bierman & Caffee 2002, abstract). The whole continents' Phanerozoic mean
is 16 m/Myr (SD 11; 4 to 53), against about 62 from present river loads, and the lowest present rates, near sea level,
7.21 m/Myr (Wilkinson & McElroy 2007, full text). Cosmogenic catchment rates barely sample flat land ("only 1 % of the
available 10Be measurements" from basins under 22 m/km, half the land; Willenbring et al. 2014, full text). So the
model's low-relief land wears 2 to 5.5 times faster than Earth's stable basins over the Cenozoic and about at the
Phanerozoic mean of all land: K's present-day calibration over-erodes the lowland (the edges and the belts' decaying
ground; the export is 2.2 times Clift's, the uplift trace). Whether that matters for the drowning goes through the
thick crust: under the hypothesis above, slower erosion would keep thick crust longer and, with the volume held, cover
less area.

**The batch's first change** (the coordinator's pointer: orog-cum, the two wedges and the retro plateau on base34,
already read 0.605 under the sea). orog-cum (f2d6412d801f, d3261b0a, 8 runs) against base34-s4 (base34's seeds 1 to
4), medians unless said: continental label under the sea 0.527 to 0.605 (the orogen lane's p 0.23 on 8 runs); crust at
least 20 km thick 0.444 to 0.464 of the planet, its mean thickness 31.7 to 31.4 km, p90 36.3 to 35.9 km; the planet's
crust under 10 km 0.428 to 0.392, 10 to 30 km 0.239 to 0.273, 35 to 40 km 0.056 to 0.042. By class (`thinning.py`, the
end): the converted margins 0.076 to 0.086 of the planet, the edges 0.207 to 0.223 at -200 to -357 m over the sea
(land 0.33 to 0.25), the interiors 0.206 to 0.201, the raised interiors 34.4 to 33.9 km. The pro-wedge fills the
incoming plate's drowned margin first (docs/design/collision-orogen.md) and the crest stands lower: the two wedges
turn about 0.035 of the planet from ocean floor into 10 to 35 km crust under the sea and take crust out of the 35 to
40 km band. The dry share of the crust at least 20 km thick falls 0.512 to 0.467 (means), across all three terms:
thickness -0.118 to -0.136, line +0.065 to +0.054, sea -0.188 to -0.203. The sea against Earth's fill (means; medians
in brackets): the water +57 to +60 (+59 to +58), the area +320 to +390 (+316 to +418), the heights -113 to -144 (-117
to -153), the floor -77 to -62 (-93 to -62), the residual -33 to -49 (-29 to -46): the run's sea 156 to 198 m (187 to
201). The area's rise is offset in part by lower heights, the floor and a larger nudge, and the drowning's rise is
mostly the wedges' new thin crust under the sea; the first change does not act through the area alone. base34 to
base35 (24 runs each) does: +0.041 of the planet at 20 km and more at the same thickness (the budget's kept volume),
the dry share 0.512 to 0.438 (means) of which the sea term -0.056, thickness -0.005, line -0.014.

**Reading.** The continents stand low because the sea stands high (-0.226 of the dry share of the crust at least 20 km
thick, means; -348 m of its mean height) and their crust is thin (-0.128; -188 m); at equal thickness the model's
crust stands near Earth's at 32 to 34 km with its sea at the datum (+0.039, +0.066 against Earth's ice-free crust).
Both terms are the continental area: +492 m of the sea's 292 over Earth's fill (means, with the steering's -68), and
15.4 km of planet-mean crust at 31.8 km mean thickness instead of 14.0 at 36.4. The area has two sources of about
equal weight: the start (0.4225 of the planet in a uniform 35.1 km column, 6 % more volume than Earth's and none of
its thick crust; about 0.4 of the excess, 0.7 on base34) and the run's thinning to 32 km (about 0.4), with base35's
kept volume the rest. The first is the preset's continents; the second's cause is the hypothesis above (thick crust
standing high on PLT-281's one line and wearing to the column at the sea), not tested here. What Earth does that the
model does not: it holds its thick crust low on cold, keeled lithosphere and its thin crust high on hot lithosphere
(the Cordillera 1.4 to 1.8 km on 30 to 45 km), about 1,600 m apart at equal crust (Hyndman & Currie 2011); and Earth's
continents carry 14.0 km of planet-mean crust over 0.384 of the planet at a mean 36.4 km, a tenth of the planet at 40
km and more. The lever is therefore one design with two parts: the continental column's height from its lithosphere
(ISO-110, or the whole column's buoyancy against the ridge's, lithosphere-thermal-state.md's open question) in place
of one line, and the start's continents (their normal column and their area) set from Earth's hypsometry and volume
under that law, since a start column on the new law's craton line at 35.1 km would drown. The erosion law is not the
first lever: it moves the thinning only if the hypothesis holds, and the height law decides how high the crust it
wears stands.

**K's calibration** (the plateau lane's question, 2026-10-31; Ben's decision of 2026-10-17 set the erosion law against
measured rates). Kept as it is for now (the coordinator's call, decisions log 2026-10-31): the drowning's terms are
the area, the line and the sea, and K enters them only through the thick crust's wear, which the height law sets first
(a column's height sets how fast it wears); a recalibration would revise Ben's decision, so it goes to him once the
column's height law is on, if the question still stands. K (ERO-004), κ (ERO-023) and the chemical floor were set on
Montgomery & Brandon's eq. 2, a present-day relation, whose low-relief end (E0 10 m/Myr and the quiet-land 0.2 mm/yr
per km of relief) is 2 to 5 times Clift et al. 2009's Cenozoic rates for stable basins (median 4.0 m/Myr), and Clift
et al. call the modern export unrepresentative of long periods (p. 90 to 91); Wilkinson & McElroy 2007 put the
Phanerozoic mean of all land at 16 m/Myr against about 62 from present river loads, a factor of about 4. Baldwin,
Whipple & Tucker 2003 (abstract) find that a critical shear stress for erosion, with isostasy and transport-limited
conditions, keeps residual topography "for hundreds of million years" where detachment-limited stream power decays it
in 1 to 10 Myr; the model has the isostasy and Yuan et al.'s deposition (`g_deposition` 1) but no erosion threshold
(S_c caps a slope, ERO-013). The options:

1. *Keep the present-day calibration* (the recommendation until the column's height law is built). The lowland keeps
   eroding at 22 m/Myr; the export stays 2.2 times Clift's, set by the plates' supply (the uplift trace: erosion-law
   changes leave it and make relief instead); the drowning's area and line stay as they are unless the thinning
   hypothesis holds.
2. *Calibrate to long-run rates* (Clift et al.'s Table 4, with thermochronology and outcrop rates). K and κ fall about
   2 to 5 times at low relief; by the plateau lane's finding the export would stay and the land would stand higher
   until its erosion met the supply (with n = 2, slopes as (E/K)^(1/2): relief 1.4 to 2.2 times on eroding ground, our
   arithmetic), more land above 1 and 2 km and the thick crust lasting longer; with the volume held, longer-lasting
   thick crust means less area, if the thinning hypothesis holds (not measured here), and it moves the high coasts
   with the relief.

## Simplifications and limits

- The fill uses the ground at the step's start; the floor's answer to the new water load comes in
  the surface flexure over the following steps, not within the solve.
- Sea water's thermal expansion, groundwater and lakes' water are not in the inventory: the water
  layer is the sea's and the land ice's alone. Lakes on land hold their own water outside it.
- The water is fixed for the whole history: no outgassing of water from the mantle and no return
  of water with subducted slabs (Earth's long-term budget is debated; neither is modelled).
- GDH1 is a fit to Earth's mean floor; the model has no plateaus or ridge-push differences beyond
  what the plate model builds. Hotspot swells are local ones the plate model builds (PLT-111,
  PLT-112), not part of the GDH1 floor.
- The first ocean starts bare: no sediment, plateaus or arcs take room in it, so the sea starts
  lower than an older ocean of the same ages would hold it (about 230 m by the laws, "What the
  numbers say").
- A closed hollow below the sea joined to the ocean by a channel narrower than one cell counts as
  closed: the open-sea rule works on cells.

## Game choices

- The open-sea size (ERO-055) and the closed-hollow rule (ISO-086): the agent's, build log
  2026-10-21.
- The plate-only presets keep Cortial's dampening (PLT-350), so the plate model's own checks and
  reproductions are unchanged: the agent's, build log 2026-10-21.
- The steering nudge (STE-006) stays on top of the volume, as Ben's steering decisions have it.

## Open questions

- GDH1's coefficients 365, 5,651 and 2,473 and its 20-Myr knee are not yet read in Stein & Stein
  1992 itself (paywalled); 2,600 m and the 0.0278 decay are read in Stein 1995.
- Decided (the coordinator's call, decisions log 2026-10-21: the volume law stays undamped and
  the swing is measured again once the continents keep their crust): the sea swings about a
  kilometre (450 to 1,380 m below the end at its lowest), past
  Earth's Phanerozoic range (ISO-087), because the plate model's continental area (0.27 to 0.48
  of the planet within a run) and ocean age (a median of 41 to 105 Myr within one run) vary much
  more than Earth's. Recommended: keep the volume law with no calibrated damping of the sea, and
  measure it again once the continent check conserves the continental crust; a knob that held the
  sea near a curve would hide the crust's losses, as the curve did.
- For the plate and continent task: the plates answer the moving sea. On supercontinent seeds 1
  and 3 the supercontinent's tenure falls from 316 Myr to 20 to 49 Myr with the volume sea over
  either floor, and the assembly-to-assembly period from 671 to 166 to 285 Myr. Under GDH1 the
  continental crust goes faster (the crust budget over 1,000 Myr on the bisect's four runs: net
  −0.049 of the planet on master, −0.084 at dd2f3f9c, −0.074 now; subduction takes 0.608 → 0.703
  and the resampling's blend 0.264 → 0.280, the rest made back), with more collisions (14.5 →
  17.5 per run) and subductions (860 → 948). The plate model reads the sea in its settling
  (`settle.rs`, ISO-078) and in GDH1's dry share (PLT-353), and gets it back through the surface's
  erosion and deposition on its samples; the force balance's column energy reads the samples'
  height, deeper at every age under GDH1 (about 600 m at 5 to 20 Myr, 250 m at 100 against the
  dampening). Which of these moves the plates is not traced. Decided (the coordinator's call,
  decisions log 2026-10-21): the plate-geometry task traces it; the sea stays where the volume
  puts it. Traced (plates.md, "The plates under the sea from the ocean's volume", twelve runs per
  arm and more): neither the motion nor GDH1. A sea pinned off 0, at −250 m or +300 m, loses as
  much crust as the moving one, and a sea pinned at 0 loses what master does with GDH1 on or off:
  about 0.026 of the planet more over 1,000 Myr off the datum, in either direction, which points
  at code that still reads 0 as the sea. The tenure's fall is inside master's own spread between
  seeds (126 Myr on seeds 1 to 3, 46 on seeds 4 to 6). Followed up (plates.md, "The sea off the
  datum", twelve runs an arm on seeds 1 to 6): the plate model's readers of 0 as the sea now read
  the sea (the column energy, the wedge floor, the Andean mark, the measurement), and none moves
  the crust; a sea held at the datum loses as much as the moving one (net −0.099 of the planet
  against −0.095, p 0.69), so the 0.026 was seeds 1 to 3's draw. What the sea's height does is
  drown the continents: held at the datum, 0.235 of the continental crust is under it, against
  0.348 under the moving sea.
- For Ben: with the flexure's GDH1 water left out (ISO-088) the Earth-like planets hold Earth's
  water with land 0.268 of the planet and 0.373 of the continental crust under the sea, against
  Earth's 0.29 and about 0.19 ("The flexure's water over GDH1's floor"). The water amount stays
  the coordinator's call (Earth's, 2,664 m); the drowned crust is the crust's, which stands too
  low ("What the numbers say"), not a reason for less water.
- The editor's "Ocean water" (`planet.surface_water_m`, 0 to 8,000 m, Earth 2,664) is a new
  outward-facing setting. Decided (the coordinator's call, decisions log 2026-10-21): it stays;
  Ben was told it is player-visible and his to veto.
- The plate era's closed basins on oceanic crust grew (124 to 156 per snapshot): they stand on
  island arcs and stacked crust above the sea, which the lower sea lays dry; they belong with the
  arcs' rate (the plate-geometry task), not with the sea.
- Decided (Ben, 2026-10-22, D6 "200 km on sphere"; "The continents under the sea and the sea's
  height at the end"): the transition's width W was 200 km on the planet, eight times Earth's
  margins in the plate model's own frame, and it is what put most of the drowned crust under the
  sea. It is now 200 km on the model sphere (24.6 km on the quick planet), in the code and every
  preset: the share under the sea falls 0.414 to 0.278, and land and the largest landmass come
  into their bands, for 0.6° of passive slope. Left: the high-ground, ice and collision
  criteria, which the width does not move.
- For the coordinator: the 0.25 target equals Earth's share at the model's boundary (0.237 to
  0.248, CRUST1.0 at 20 km) with no margin, and Earth fails it by Cogley's continent-ocean
  contacts (0.306). Recommended: report the share of crust at least 25 km thick above the sea
  beside it (Earth 0.81).
- For the plate and crust-budget tasks: the ocean holds five to six times Earth's sediment, since
  none subducts, and that is what holds the sea 500 m over the datum; but the continents get
  their eroded crust back only through it (sub80), so sediment subduction waits on the arcs'
  growth of continental crust.
- Not settled: the final frame is always read right after the plate model's forced last
  resampling, 48 to 118 m of sea over the last step's; whether the saw's top or its run is the
  truer state.

## Sources

- CharetteSmith2010: full text (Table 2), 2026-10-21.
- EakinsSharman2010: a mirror of the report (the total row), 2026-10-21; not the NOAA page.
- IPCC_AR5_WG1_Ch4: Table 4.1 and its note b, 2026-10-21.
- Fretwell2013: Tables 7 and 8, 2026-10-21.
- Morlighem2017: key points, read earlier (spot-checks, CLM-108).
- SteinStein1992: not read (paywalled); Stein1995 read for the ridge depth and the decay constant.
- ParsonsSclater1977: the abstract (the scanned first page), 2026-10-21.
- Muller2008: the abstract, 2026-10-21.
- Miller2005: the abstract, 2026-10-21.
- Miller2011: the Fig. 5 caption (Haq et al. 1987's 250 m), 2026-10-21.
- Haq1987: the abstract (no numbers), 2026-10-21.
- Kominz1998: the backstripping relation, 2026-10-21.
- Parsons1982: the abstract on OSTI (the age–area law, the consumption rate C₀/t_m), 2026-10-21.
- Seton2020: the abstract (mean age 64.2 Myr), 2026-10-21; the grid earlier (WP-13a).
- For "The continents under the sea and the sea's height at the end" (2026-10-22): Cogley1984
  (the abstract), RudnickGao2003 (full text, the 31 %), Harrison1983 (full text, Table 5),
  Laske2013 (the CRUST1.0 data, measured), Straume2019 (the accepted manuscript, Table 3),
  Conrad2013 (the "Marine Sedimentation" section), Dutkiewicz2017 (sections 4 and 5),
  Syvitski2005 (at second hand, Walling 2006), vonHueneScholl1991 (the abstract), SternScholl2010
  (read earlier), Harris2014 (read earlier) and SimpsonSimpson2018 (full text, the land's mean
  heights).
- For "Why base35's continents stand low, against Earth" (2026-10-31; bibliography.md, "The sources of the
  low-continents lane"): Laske2013 and Tozer2019 (measured), Clift2009 (the library text, pp. 89 to 93 and Table
  4), HyndmanCurrie2011 (read earlier), Flowers2006, BeauvaisChardon2013, BiermanCaffee2002, EatonPerry2013,
  HoggardWhiteAlAttar2016, Gurnis2000 and BaldwinWhippleTucker2003 (abstracts), BiermanCaffee2001,
  WilkinsonMcElroy2007, Willenbring2014, Kaban2003 and Korenaga2017 (full texts, Kaban in its main parts);
  Cogley1984, Marcilly2022 and Seton2020 (read earlier: the start's shelf share, RLC-031, the floor's ages).
