# Sea level

Status of this page: `stage 2 (constants checked against sources on 2026-10-21)`, written with the
change that made the sea level follow the ocean's volume (Ben, decisions log 2026-10-21: "Its own
task now", then "Don't follow that old idea" for the 2026-09-30 row "Keep calibrated coastlines").

Ledger ids on this page: `ISO-085` to `ISO-087`, `PLT-350` to `PLT-355`, `CLM-210`, and the rows
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
   ρ_i/ρ_w of its volume out of that water (CLM-210).
3. The rest stands in the sea: the level S at which the open sea holds it over the ground the
   surface stage starts the step from. The open sea at S is what the surface itself calls sea, a
   connected body of ground below S at least a 500th of the grid (ERO-055); a closed hollow below S
   that no channel joins to the ocean stays dry until the sea tops its rim (ISO-086).
4. The steering nudge, if any, is added (STE-006), and the result goes into the same global as
   before; every reader above takes it unchanged.

Without the climate stage (the surface-only pipelines) the surface stage fills its own ground the
same way with the whole water (no ice).

Because the level follows the ground, everything that changes the ocean basins' room moves it:

- **The ocean floor's age** (PLT-350 to PLT-353): the shipped Earth-like presets now sink the floor
  by Stein & Stein's GDH1 age–depth law instead of Cortial's dampening (PLT-129). Young floor is
  shallow, so a planet with more young floor has less room and a higher sea.
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
given; one priority flood and two sorts per step.

The floor (`plates/pools.rs`, `surface_step`, `gdh1_depth_m`):

  d(t) = 2,600 + 365 √t (t < 20 Myr),  d(t) = 5,651 − 2,473 e^(−0.0278 t) (t ≥ 20 Myr),
  Δz = −[d(t) − d(t − Δt)]            below the sea,
  Δz = −[d(t) − d(t − Δt)] / W        above it, W = ρ_m / (ρ_m − ρ_w),

with t the sample's age at the step's end (Myr) and Δt the step. New ocean floor (ridges, gaps,
the first ocean) is laid at −d(t) (PLT-351). With a ridge crest z_r and an old floor z_f other
than GDH1's own −2,600 and −5,651 m (the editor's ridge and abyssal depths), the curve is
stretched between them, z = z_r − s·(d(t) − 2,600) with s = (z_r − z_f)/3,051, and each step
sinks s times as much (PLT-355); every shipped preset has s = 1.

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| ISO-085 | The planet's surface water, a layer over the whole planet | `planet.surface_water_m` 2,664 (default and every shipped preset) | m | Charette & Smith 2010 (ocean 1.33238·10⁹ km³), Fretwell et al. 2013 (Antarctica 26.54·10⁶ km³ grounded), Morlighem et al. 2017 (Greenland 2.99·10⁶ km³), IPCC AR5 Table 4.1 (glaciers 0.41 m); Earth's area from memory | `verified` |
| ISO-086 | The sea at S is the open sea: a connected body of ground below S of at least n/500 cells; closed hollows take no water until the sea tops their rim | `fill_level` with the surface's `open_sea` rule (ERO-055) | m | none needed (mass balance); the size rule is ERO-055's | `game-choice` |
| ISO-087 | The sea level series' check against Earth's Phanerozoic range | −100 to +250 m against today | m | Müller et al. 2008 (Late Cretaceous 170 m, 85 to 270), Miller et al. 2005 (Cretaceous peak 100 ± 50 m), both abstracts; Haq et al. 1987's 250 m at second hand (Miller et al. 2011) | `abstract-only` |
| CLM-210 | Land ice holds (ρ_i/ρ_w) of its volume of the planet's water out of the sea | `sea_level::ice_layer_m`, 917/1028 (CLM-099) | m | IPCC AR5 Table 4.1 note b (917 and 1028 kg/m³, read) | `verified` |
| PLT-350 | The ocean floor's law | `tectonics.ocean_depth`: `gdh1` in every shipped Earth-like preset, `dampening` (PLT-129) the default and the plate-only presets' | - | the brief: PLT-129's dampening fixed at an age–depth law | `game-choice` |
| PLT-351 | New ocean floor at the law's depth for its age | `pools::floor_at` (init, ridges, L1 gaps) | m | as PLT-352 | `cited-unverified` |
| PLT-352 | GDH1 | 2,600, 365, 5,651, 2,473, 0.0278, knee 20 Myr | m, Myr | Stein & Stein 1992 (not read); 2,600 m and the 0.0278 decay read in Stein 1995; the rest on a secondary page | `cited-unverified` |
| PLT-353 | Ground above the sea sinks by 1/W of the water-loaded subsidence | W = ρ_m/(ρ_m − ρ_w) (`settle::WATER_LOADED`) | dimensionless | Kominz et al. 1998 (the factor, read) | `verified` |
| PLT-354 | The floor in a gap between plates is held in place, deepens with its age, keeps what lies on it; a cell entering a gap starts bare | `advect::hold_gaps`, `newborn`, `carried_elevation`; `plates/gap`, `surface/crust_gap` | - | none needed (the plate model's kinematics) | `game-choice` |
| PLT-355 | GDH1's shape between the preset's ridge crest and old floor, so the editor's ridge and abyssal depths keep working; GDH1 exactly at its own depths | `pools::gdh1_scale`, `floor_at`, `surface_step`; s = (z_r − z_f)/3,051, 1 in every shipped preset | - | none (a stretch of PLT-352's curve) | `game-choice` |

## Earth against the mod

Measured 2026-10-21 on the six-run ensemble: `earthlike_quick_16k`, 1,000 Myr, 8 threads, seeds 1
to 3, supercontinent (sup) and scattered (sca) starts. Before is master f1632cf3 (the curve with
glacio-eustasy, Cortial's dampening at 0.12 mm/yr towards −5,600 m from a −2,500 m ridge); after
is this change at 08f68dc0 (PLT-355 changes nothing at s = 1). Each cell is before → after. The map
is the keyframes (p50 over the plate frames, or the last frame where it says final); the bake is the
baked world (`tools/review/water_census.py --bake`); lake basins are `tools/review/lake_basins.py`
on four epoch checkpoints per run (Earth-equivalent areas, the planet's times 66.3); the sea, ice,
continental crust and ocean age are the runs' own records (`notes.climate.series`,
`notes.plates.dynamics`). "Under the sea" is the share of the continental crust (the `crust_age`
layer's continental colour) that the `rivers` layer draws as sea; a landmass is an 8-connected body
of land on the keyframe.

| Measure | Earth | sup 1 | sup 2 | sup 3 | sca 1 | sca 2 | sca 3 | mean |
|---|---|---|---|---|---|---|---|---|
| Sea at the start, m | | 30 → 328 | 30 → 366 | 30 → 374 | 29 → 450 | 26 → 367 | 28 → 442 | 29 → 388 |
| Sea at the end, m | | 0 → −140 | −13 → 191 | −26 → 287 | −55 → 78 | −23 → 36 | −19 → −26 | −23 → 71 |
| Sea against the end after the first 100 Myr, min / p50 / max, m | −100 to +250 (ISO-087) | −147/−20/101 → −896/−314/39 | −174/−33/103 → −668/−326/25 | −157/−19/122 → −1014/−705/36 | −118/6/155 → −1037/−554/75 | −157/−30/102 → −1088/−492/54 | −163/−10/127 → −574/−317/65 | −153/−18/118 → −880/−451/49 |
| Land ice at the end, 10⁶ km³ | 29.7 (ISO-085's sources) | 21 → 20 | 26 → 18 | 30 → 27 | 44 → 22 | 22 → 29 | 29 → 24 | 28 → 23 |
| Continental crust, share of the planet, p50 over the steps | 0.41 (HYP-012) | 0.403 → 0.317 | 0.395 → 0.343 | 0.385 → 0.346 | 0.341 → 0.332 | 0.347 → 0.310 | 0.343 → 0.344 | 0.369 → 0.332 |
| Continental crust under the sea, final | about 0.19 (from memory) | 0.384 → 0.346 | 0.344 → 0.409 | 0.314 → 0.368 | 0.317 → 0.388 | 0.391 → 0.314 | 0.369 → 0.287 | 0.353 → 0.352 |
| Continental crust under the sea, p50 of the plate frames | about 0.19 | 0.358 → 0.203 | 0.297 → 0.312 | 0.311 → 0.216 | 0.319 → 0.213 | 0.319 → 0.251 | 0.313 → 0.216 | 0.320 → 0.235 |
| Land, share of the planet (map p50) | 0.29 | 0.274 → 0.332 | 0.287 → 0.296 | 0.292 → 0.333 | 0.246 → 0.316 | 0.252 → 0.311 | 0.254 → 0.337 | 0.267 → 0.321 |
| Land (bake) | 0.29 | 0.257 → 0.262 | 0.280 → 0.249 | 0.318 → 0.286 | 0.261 → 0.267 | 0.236 → 0.297 | 0.244 → 0.279 | 0.266 → 0.273 |
| Land above 2 km, % of the land (bake) | 10.2 (HYP-004; rock 4.7, HYP-041) | 14.0 → 21.6 | 14.8 → 13.9 | 18.5 → 20.0 | 22.7 → 25.8 | 19.3 → 18.2 | 18.6 → 22.2 | 18.0 → 20.3 |
| Largest landmass, final, share of the planet | 0.167, Afro-Eurasia (from memory) | 0.052 → 0.134 | 0.165 → 0.063 | 0.267 → 0.132 | 0.109 → 0.187 | 0.089 → 0.193 | 0.041 → 0.060 | 0.120 → 0.128 |
| Second landmass, final | 0.083, the Americas (from memory) | 0.052 → 0.038 | 0.050 → 0.057 | 0.022 → 0.079 | 0.041 → 0.040 | 0.060 → 0.090 | 0.036 → 0.055 | 0.043 → 0.060 |
| Land in landmasses of 0.01 of the planet or more, final | about 0.95 (from memory) | 0.845 → 0.823 | 0.921 → 0.785 | 0.878 → 0.885 | 0.925 → 0.874 | 0.877 → 0.912 | 0.897 → 0.796 | 0.891 → 0.846 |
| Coast p50, km (map; final) | 433 (HYP-035) | 327 → 377; 197 → 322 | 448 → 333; 349 → 230 | 285 → 449; 290 → 273 | 297 → 354; 251 → 423 | 262 → 376; 303 → 421 | 249 → 440; 225 → 296 | 311 → 388; 269 → 328 |
| Lakes, % of the land (map p50) | under 1.8 (HYD-090) | 3.07 → 4.83 | 3.86 → 4.14 | 4.47 → 4.07 | 3.15 → 4.05 | 4.79 → 4.13 | 3.73 → 3.73 | 3.85 → 4.16 |
| Lakes, % of the land (bake) | under 1.8 | 0.88 → 2.00 | 1.16 → 1.90 | 2.05 → 2.45 | 1.47 → 2.13 | 3.00 → 2.33 | 1.54 → 1.13 | 1.69 → 1.99 |
| Lake area on oceanic crust, % (epochs) | | 33.9 → 57.0 | 28.3 → 33.0 | 29.3 → 38.9 | 32.2 → 34.9 | 41.6 → 45.2 | 37.3 → 33.5 | 33.8 → 40.4 |
| Lakes of the Caspian's size or more per snapshot, all (on sea floor) | 1 (1) | 3.0 (0.5) → 3.75 (1.75) | 4.75 (1.75) → 4.5 (0.75) | 4.0 (0.25) → 7.75 (2.5) | 3.75 (0.75) → 4.5 (1.0) | 3.75 (1.0) → 3.5 (2.0) | 3.0 (1.0) → 7.0 (1.75) | 3.71 (0.88) → 5.17 (1.62) |
| Closed ground below sea level, % of the land (bake) | | 0.17 → 0.44 | 0.14 → 0.57 | 0.66 → 0.37 | 0.13 → 0.05 | 0.20 → 0.33 | 0.22 → 0.24 | 0.25 → 0.33 |

Closed basins on sea floor per snapshot (the 24 epoch checkpoints), by Earth-equivalent area:
under 10⁴ km² 71.9 → 103.0, 10⁴ to 10⁵ 45.4 → 64.7, 10⁵ to 4·10⁵ 6.2 → 10.5, 4·10⁵ to 10⁶ 0.75 →
1.25, 10⁶ and more 0.12 → 0.38; in all 124 → 180 basins, 3.4 → 5.9 million km² (Earth's natural
lakes 2.67 million, HYD-090). The plate checks (`deeptime plate-check`, per start over the three
seeds) meet 29 → 29 (supercontinent) and 31 → 27 (scattered) of 45 targets; the scattered start's
four lost targets are kinematic ones near their bounds (largest plate 0.15 against 0.15 to 0.35,
the speed-against-area correlation −0.40 against −0.40 to 0.20, the slab-speed ratio, the
collision length).

The same runs in the continents census's terms (`tools/review/continents_census.py keyframes`,
master's tool from the diagnosis of 2026-10-21, plates.md), with a third set: the volume sea over
the old dampening floor (this change with `tectonics.ocean_depth=dampening` at master's 0.12 mm/yr
from −2,500 to −5,600 m; supercontinent seeds 1 and 3, scattered seed 1), to tell the volume law's part from GDH1's. Means
over the runs, the final keyframe / the median over the plate frames (more than 100 Myr in):

| Measure | Earth | before (6 runs) | after (6 runs) | volume sea, old floor (3 runs) |
|---|---|---|---|---|
| Land | 0.29 | 0.264 / 0.251 | 0.275 / 0.305 | 0.219 / 0.230 |
| Continental crust, share of the planet | 0.41 (HYP-012) | 0.380 / 0.360 | 0.341 / 0.327 | 0.317 / 0.352 |
| Its share under the sea: shelf (under 200 m), deeper | 0.195 in all, 0.115 on the shelf (CRUST1.0, crust of 25 km or more, PLT-361) | 0.368: 0.103, 0.265 / 0.338: 0.084, 0.254 | 0.366: 0.098, 0.268 / 0.264: 0.082, 0.183 | 0.376: 0.083, 0.293 / 0.367: 0.090, 0.279 |
| The ocean's water under the sea, 10⁹ km³ | 1.33 (ISO-085) | 1.102 / 1.276 | 1.341 / 1.324 | 1.319 / 1.332 |
| The sea that would hold Earth's water and ice (1.359·10⁹ km³), m against the run's | | +574 / +161 | −10 / +8 | +54 / +9 |
| Land at that sea | 0.29 | 0.165 / 0.205 | 0.278 / 0.302 | 0.209 / 0.227 |

The after runs hold Earth's water within one ramp step of the keyframes (the census's sea
differs from the run's by −44 to +25 m at the end). At the end the curve held 0.81 of Earth's water
(1.10·10⁹ km³), the diagnosis's 0.80 on Ben's run.

The same change in the continent bisect's terms (`tools/review/out-free/continent-bisect.md`: the
final frame at 1,000 Myr, seeds 1 and 2, both starts, 2 threads, the crust budget's env on, read
after `deeptime stage volcanoes`), so it reads as one more row of that table. Before is the
bisect's own 30c4d664 row (master's simulation is byte-identical from 30c4d664 to c790d1f3, this
branch's base); after is this branch at dd2f3f9c (`continent-bisect.sh ship dd2f3f9c`, then
`all dd2f3f9c 1,2 supercontinent,scattered_continents 1000` and `continent-bisect.py ... --runs`).

| Run | land | continental crust | its share under the sea | largest landmass | second |
|---|---|---|---|---|---|
| Earth | 0.29 | 0.41 | about 0.19 | 0.167 | 0.083 |
| sup 1 | 0.252 → 0.264 | 0.389 → 0.325 | 0.408 → 0.356 | 0.101 → 0.129 | 0.036 → 0.037 |
| sup 2 | 0.278 → 0.255 | 0.399 → 0.343 | 0.354 → 0.424 | 0.159 → 0.060 | 0.049 → 0.055 |
| sca 1 | 0.267 → 0.260 | 0.358 → 0.333 | 0.326 → 0.400 | 0.105 → 0.173 | 0.039 → 0.038 |
| sca 2 | 0.229 → 0.300 | 0.350 → 0.351 | 0.408 → 0.328 | 0.083 → 0.184 | 0.057 → 0.055 |
| mean of the four | 0.257 → 0.270 | 0.374 → 0.338 | 0.374 → 0.377 | 0.112 → 0.137 | 0.045 → 0.046 |

The rest of the bisect's pooled row, before → after: land above 2 km 0.270 → 0.275 of the land,
landmasses of 0.01 or more 5.0 → 3.8 holding 0.889 → 0.844 of the land, island arcs 0.412 →
0.448 of the land, pieces 276 → 354; run-to-run sd of land 0.021 → 0.021 and of the largest
landmass 0.033 → 0.057. By the bisect's own rule (a land step under 0.03 is not a finding) land and
the largest landmass do not move, and the drowned share does not either: the volume sea leaves the
end's flooding where the crust put it. The continental crust falls by 0.036, in three runs of four
(the ensemble's p50, 0.369 → 0.332, the same).

**The water amount.** Earth's, 2,664 m (ISO-085), is kept: the coordinator's call to this task
(2026-10-21), made on Earth's numbers after the continents diagnosis found that a sea holding
Earth's water would have drowned more land on master (0.12 to 0.22 on Ben's run). That holds over
master's floor: the volume sea over the old dampening floor ends at land 0.219, as the census
predicts from the before runs (0.165 to 0.205). Over GDH1's floor, deeper under old ocean, the
basins hold Earth's ocean with land 0.275 at the end (0.256 to 0.299 by run; seeds 1, 2 and 3, the
mean of both starts, 0.262, 0.278 and 0.285) and 0.305 over the plate frames, against Earth's 0.29.
A planet with less water than Earth's would be a calibrated knob, and it would hide the drowned
crust that the census measures; none is needed.

What the numbers say:

- **The Phanerozoic check fails.** Past the first 100 Myr the sea stands 570 to 1,090 m below the
  end at its lowest, against Earth's 100 m. The first 100 Myr are a spin-up: the first ocean is
  laid young (median age 19 to 22 Myr at the start, 47 to 70 Myr 100 Myr later), so the sea starts 330 to
  450 m high and falls as that floor ages.
- **What moves it is the plate model's own variability, passed on as the volume law should.** The
  continental crust's share wanders from 0.42 at the start to 0.26 at its lowest and back to 0.33
  to 0.36 (the plate check's 0.37 to 0.45 already failed before, 0.32 and 0.30 at their lowest),
  and the ocean floor's median age by a standard deviation of 9 to 12 Myr. By the fill's arithmetic,
  turning 0.01 of the planet from continent to ocean floor about 4.5 km deep makes room for about
  65 m of sea, and ten Myr more median age about 150 m under GDH1; the land ice, up to 60 to 87·10⁶ km³ in the
  runs' cold spells against 20 to 29 at the end, about 2.6 m per 10⁶ km³ (the land ice's share
  put back, the lowest stands rise by 30 to 160 m). Per run, a fit of the sea against the three
  explains a quarter to two thirds of its variance; the rest is not traced (sediment, the
  continents' heights).
- **The end stands high.** In all six runs the floor is younger at the end (median 45 to 55 Myr,
  against about 60 over the run) and the continental share at or above its median in five, so the
  sea at the end is near its high stand: on Earth the present is near a low stand (ice). The steering
  nudge (STE-006) did nothing in these runs (offsets under 0.02 m).
- **The flooding of the continents is mostly the crust, not the sea level.** Before this change
  the sea stood within 153 m of today all the time, and still 0.32 (p50 of the frames) to 0.35
  (final) of the continental crust lay under it, against Earth's 0.19. To lay it as dry as
  Earth's, the sea would have had to stand at the continental crust's 19th height percentile,
  220 to 1,240 m below the sea on the 24 epoch checkpoints: the continental crust stands too
  low, not the sea too high. After the change, the share under the sea at the end is the same
  (0.352); over the plate frames it falls to 0.235 only because the sea stands 300 to 700 m lower
  for most of the run, which a crust that stood higher would not need.
- **Land and its shape hold.** Land rises (map 0.267 → 0.321, bake 0.266 → 0.273), the land above
  2 km and the coast distance with it; the largest landmass over the plate frames is 0.128 →
  0.167 of the planet (p50; Earth's Afro-Eurasia 0.167) and the second 0.056 → 0.069 (the
  Americas 0.083); at the end both are below Earth's.
- **The sea-floor lakes grow.** The plate era's closed basins on oceanic crust rise from 124 to 180
  per snapshot and from 34 to 40 % of the lake area. They stand on oceanic crust above the sea
  (their level p50 552 m over the datum; island arcs and the stacked crust the plate model keeps
  under the oceanic label, whose arc crests stand 1,360 to 1,410 m high at p90, 1,140 to 1,160 before, against the plate
  check's 1,000), and a sea 300 to 700 m lower lays more of that ground dry. The volume law was expected
  to fill them through the floor's kept sediment (rivers-and-lakes.md, "What keeps the plate
  era's lakes"); it does not, because the pockets are not under the sea.

## Simplifications and limits

- The fill uses the ground at the step's start; the floor's answer to the new water load comes in
  the surface flexure over the following steps, not within the solve.
- Sea water's thermal expansion, groundwater and lakes' water are not in the inventory: the water
  layer is the sea's and the land ice's alone. Lakes on land hold their own water outside it.
- The water is fixed for the whole history: no outgassing of water from the mantle and no return
  of water with subducted slabs (Earth's long-term budget is debated; neither is modelled).
- GDH1 is a fit to Earth's mean floor; the model has no hotspot swells, plateaus or ridge-push
  differences beyond what the plate model builds.
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
- For Ben: the sea swings about a kilometre, past Earth's Phanerozoic range (ISO-087), because
  the plate model's continental area (0.26 to 0.46 of the planet) and ocean age vary much more
  than Earth's. Recommended: keep the volume law with no calibrated damping of the sea, and
  measure it again once the continent check conserves the continental crust; a knob that held the
  sea near a curve would hide the crust's losses, as the curve did.
- For Ben: the first ocean is laid young (median 19 to 22 Myr), so every run starts with the sea
  330 to 450 m high and drains it over 100 Myr. Recommended: lay the first ocean with a
  steady-state spread of ages, as Earth's floor has (from memory), in the plate-geometry task.
- For Ben: the editor's "Ocean water" (`planet.surface_water_m`, 0 to 8,000 m, Earth 2,664) is a
  new outward-facing setting. Recommended: keep it.
- The plate era's closed basins on oceanic crust grew (124 to 180 per snapshot): they stand on
  island arcs and stacked crust above the sea, which the lower sea lays dry; they belong with the
  arcs' rate (the plate-geometry task), not with the sea.

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
- Seton2020: the abstract (mean age 64.2 Myr), 2026-10-21; the grid earlier (WP-13a).
