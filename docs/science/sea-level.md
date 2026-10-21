# Sea level

Status of this page: `stage 2 (constants checked against sources on 2026-10-21)`, written with the
change that made the sea level follow the ocean's volume (Ben, decisions log 2026-10-21: "Its own
task now", then "Don't follow that old idea" for the 2026-09-30 row "Keep calibrated coastlines").

Ledger ids on this page: `ISO-085` to `ISO-087`, `PLT-350` to `PLT-354`, `CLM-210`, and the rows
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
the first ocean) is laid at −d(t) (PLT-351).

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

## Earth against the mod

MEASURED_TABLE

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
- OPEN_QUESTIONS

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
