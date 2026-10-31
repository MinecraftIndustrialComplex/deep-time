# The surface's runoff from the bake's water budget: design

Status: **built behind a switch, off** (2026-10-31, the runoff lane, branch `surface-runoff-budget` on
`plateau-drainage` 7df49df8). The switch is `surface.runoff_budget` (`crates/deeptime-core/src/surface/climate.rs`,
ERO-221). The checks against cold basins are ERO-222. One cumulative 8-run arm is ready to queue ("The arm"); no
simulation was run for this design.

## Why

The plateau lane's arm (docs/design/plateau-drainage.md, "The arm") found where the flat tops of the plateaus wear:
- over the whole run, 0.67 of the flat high ground has a mean annual temperature below 0 °C;
- that part wears at 141 m/Myr, 0.65 of it by stream power and the critical slope's cut it drives;
- the closed basins did not change it.

On that ground the surface's runoff is all of the precipitation. The climate snapshot (`surface/climate.rs`) takes PET =
60 mm/°C x the mean annual temperature, 0 below 0 °C (MOI-067), and Fu's curve (HYD-034). So a land whose mean is below
freezing evaporates nothing, and its whole P weights the stream power's drainage area (ERO-010).

The bake's water budget dropped that rule on 2026-10-01 (STATUS build log, "P1-22b follow-up: cold-climate runoff
reviewed"). In that entry's words, "the land's evapotranspiration used PET = 60 mm/°C × the mean annual temperature,
zero below freezing, so a land with a mean below 0 °C sent all its precipitation to the rivers". Its fix was "PET =
`hydro.pet_mm_per_c` (60) × the year's mean temperature above freezing, on the liquid water. That is Holdridge's rule
... and equals the old law where there is no frost".

The plateau lane's lake balance already uses that budget (ISO-126, ERO-217). The rivers do not. The plateau design
listed it as a simplification: "two runoffs that differ on frozen ground".

## The two rules

| | the climate snapshot (today's stream power) | the bake's budget (`HydroParams::budget`) |
|---|---|---|
| liquid water | all of P | rain in the thawed share of the year, plus the snow its degree-days melt, at most P (HYD-036, HYD-037) |
| PET | 60 mm/°C x the mean annual temperature, 0 below 0 °C (MOI-067) | 60 mm/°C x the biotemperature, the year's mean of the temperature above 0 °C (HYD-029, HYD-030) |
| evapotranspiration | Fu's curve on P (HYD-034) | Fu's curve on the liquid water (HYD-035) |
| a mean below 0 °C, a thawed summer | all of P runs off | the summer evaporates |
| a warmest month below 0 °C | all of P runs off | nothing runs off: the snow stays, the glaciers' to move (HYD-038) |
| no month below 0 °C | P less Fu's ET at PET 60 T | the same, to 1e-12 (a test) |

The months come from the climate stage's seasons, taken as departures about the surface's own mean temperature
(HYD-033). Without the stage they are the mean ± 8 °C (HYD-032).

## What cold lands do (sources read)

A Sonnet research sub-agent read these on 2026-10-31. The quotes are as it printed them (bibliography, "Cold lands'
runoff").

- **Permafrost basins run off part of their precipitation, not all of it.**
  - The Mackenzie basin: "~1.8x10^6 km2", "Mean annual precipitation: 410mm", "Mean annual evapotranspiration:
    237mm", "Average monthly temperature: 15 C in the summer and -25 C in the winter" (MAGS Water and Energy Budget
    Synthesis, Table 2.1). Its mean runoff is 169 mm at Arctic Red River (Woo & Thorne 2003, Table 2). That is a ratio
    of 0.41 (computed).
  - The Lena and the Ob: "The runoff ratios (runoff/precipitation) are high (about 0.50) for the Lena basin compared
    with the Ob river (around 0.30) with only 30% permafrost" (Yang et al. 2002, p. 14-3).
  - The pan-Arctic land: evapotranspiration "approximately 310 mm yr −1" and P − ET "of 180 mm yr −1" (Serreze et al.
    2006 as cited by Rawlins et al. 2010), a ratio of 0.37 (computed).
- **A polar desert loses its snow to the air.** "Air temperatures average about –17oC with the winter minimum about
  -40oC and the summer maximum a few degrees above freezing"; "In Wright Valley, annual values of snowfall have ranged
  7 – 11 mm water equivalent ..., most of which is lost to sublimation" (Fountain et al., "Snow in the McMurdo Dry
  Valleys, Antarctica"). Also: "in the valley bottom sublimation (not shown) far exceeds precipitation" (the same).
- **The Tibetan Plateau's lakes, modelled.**
  - The climate: "The mean annual air temperature (MAAT) lies between 0 and −3 ◦C and the mean annual precipitation
    ranges between 150 and 500 mm, with 60–80 % of this total occurring between June and September" (Biskop et al.
    2016, §2.1).
  - Their J2000g model on the HAR reanalysis (Table 4: P, AET, land runoff, mm/yr) gives ratios of 0.26 to 0.31:
    Mapam Yumco 230, 170, 60; Paiku Co 250, 180, 70; Tangra Yumco 300, 210, 90; Nam Co 420, 290, 130.
- **How cold snow leaves.**
  - Sublimation takes "between 10 and 90%" of winter snowfall in the Alps (Strasser et al. 2008, abstract), and 20 to
    32 % in the Canadian Rockies (MacDonald et al. 2010, abstract).
  - Blowing snow alone takes "28% of annual snowfall over western Canadian Arctic tundra (Pomeroy et al., 1997)" and
    "18–25% of winter precipitation over the Alaskan arctic (Liston and Sturm, 2002)" (as cited by MacDonald et al.
    2010).
- **The budget's own constants.**
  - **PET and biotemperature.** Holdridge: "the mean annual potential evapotranspiration in mm. at any site may be
    determined by multiplying the mean annual biotemperature by the factor 58.93" (HYD-029, now `verified`), and "All
    temperatures lower than O0 Celsius were considered as O0" (HYD-030). He also cuts at 30 °C: "biotemperature is an
    average of the temperature between 0. and 300C". The code does not cut there.
  - **The degree-day factor.** Hock 2003's Table 1 gives snow factors of 2.5 to 5.5 mm per °C per day at
    non-glaciated sites and 2.7 to 11.6 on glaciers. The budget's 4.0 lies inside the first range (HYD-037, now
    `verified`).
  - **Fu's ω of 2.6** is read only at second hand: "The original formulation introduced by Budyko (1956, 1974) is best
    represented by setting ω = 2.6 (Zhang et al., 2004)" (Greve et al. 2015, author response). Zhang, Hickel & Shao
    (MODSIM 2005) used "a single value of 2.63".
  - **Fu's error.** Zhang et al. 2004's abstract gives the curve's error over 470 catchments: "The mean absolute error
    between modeled and observed evapotranspiration was 54 mm".
- **Not a test.** Svalbard's catchments run off more than their precipitation, since their glaciers lose mass
  (Killingtveit et al. 2003). Boot Creek (Inuvik) and Imnavait Creek have budgets in the sources but no temperatures
  the agent could print.

## The rule

**R1 (ERO-221).** With `surface.runoff_budget`, the stream power's runoff weight (ERO-010) takes each cell's runoff from
the bake's budget: `climate::water_budget` per cell, the same `climate::cell_water` the lakes' balance takes. The
weight is runoff / 0.5 m, floored at 0.05 (ERO-011). Nothing else reads the change: the aridity index, the facies and
the soils keep the snapshot's PET (MOI-067).

What follows from the budget:
- frost-free land is unchanged (`without_frost_the_budget_is_the_surface_s_runoff`), so K's calibration of 2026-10-18 on
  that land is unchanged;
- ground that never thaws runs off nothing, and its weight is the floor;
- a cold land with a summer evaporates in it;
- the lakes and the rivers they are fed by share one budget.

## The checks (ERO-222)

Unit tests in `surface/climate.rs` and `surface/plateau_check.rs` (mm/yr):

| site | source | the budget | the snapshot's rule |
|---|---|---|---|
| Mackenzie basin, P 410, months 15 and -25 °C | runoff 169, ET 237 (measured) | runoff 204, ET 206: within Fu's 54 mm | runoff 410, ET 0 |
| Dry Valleys, P 7 to 11, mean -17, coldest -40 °C | "most of which is lost to sublimation" | 0 with the warmest month below 0 °C; under half of P at 2 or 3 °C; 0.29 to 0.87 of P at 0.5 to 1.5 °C (the gap) | all of P |
| Tibetan lakes, P 230 to 420, MAAT 0 to -3 °C (months ± 8 °C, HYD-032) | ET 170 to 290, runoff ratio 0.26 to 0.31 (modelled) | ET 69 to 141, runoff ratio 0.46 to 0.83 | all of P |
| Verkhoyansk, months 13.3 and -50.0 °C | biotemperature 3.05 °C (Holdridge, rough) | 2.65 °C | (none) |

The cold controlled plateau (`a_cold_plateau_runs_off_what_its_summer_leaves`) uses ERO-219's rimmed plateau with Tangra
Yumco's interior climate (P 300, MAAT -1.5 °C) and spills in every case. Its interior weight is 0.600 under the
snapshot's rule and 0.396 under the budget, and its interior net lowering 118.0 and 88.0 m/Myr. An interior that never
thaws wears 3.8 m/Myr.

Reading: the budget gets a measured permafrost basin right within the curve's own error, and closes off ground that
never thaws, where the old rule erred most. It still runs off too much where the plateau's sun and wind evaporate more
than Holdridge's biotemperature allows, and where cold-season snow sublimates.

## What it does not do

- **Sublimation of the cold season's snow.** The sources give shares of 10 to 90 % that vary with wind and exposure. No
  law was read that sets the share without a fit, so none is built. The Dry Valleys' gap is printed by its test.
- **The plateau's evaporation.** Holdridge's PET has no term for radiation, wind or pressure. Biskop et al.'s modelled
  AET is 1.4 to 4 times the budget's. Left: changing the PET would change all land, and K with it.
- **Land ice.** The budget does not read the climate's ice (HYD-038; open since 2026-10-01). Under thick ice the
  rivers do not erode anyway (`surface/mod.rs`).
- **Holdridge's 30 °C cut** is not applied (HYD-030). It would matter only on hot land, where the two rules agree today.
- **K, ERO-011's floor and the reference runoff** are untouched. The arm reads the regions' table instead (`recal.sh`).
- **No knob, cap or forcing term.** The runoff is the bake's existing, sourced budget.

## The arm

One cumulative 8-run arm, stage 2 on the plateau lane's closed basins. The orogen switches stay on, as in orog-cum
and plat-cb0. With both new switches off, the build is orog-cum's physics; check that first:

    tools/review/pool/pool.sh alias HEAD f2d6412d801f --check-seed 1
    tools/review/pool/pool.sh run plat-rb0 --seeds 1,2,3,4 --set tectonics.collision_two_wedges=true --set tectonics.collision_retro_plateau=true --set surface.closed_basins=true --set surface.runoff_budget=true +DEEPTIME_CRUST_BUDGET=1 +DEEPTIME_CRUST_BUDGET_MAPS=@RUN@/maps
    tools/review/pool/pool.sh table --base orog-cum --arm plat-cb0 --arm plat-rb0

On base35-s4 instead, once it exists: merge `origin/master` into this
branch, then `pool.sh run plat-rb --seeds 1,2,3,4 --set surface.runoff_budget=true` (and `--set surface.closed_basins=true` unless it is on by default by then) with the same two environment
words, and `pool.sh table --base base35-s4 --arm plat-rb`.

The reads, on the VM:

    tools/review/plateau/wear.sh platrb0=<folder> platcb0=<folder> orogcum=<folder>
    tools/review/plateau/recal.sh platrb0=<folder> platcb0=<folder> orogcum=<folder>
    tools/review/plateau/closedshare.py <folder>

What they show:
- `wear.sh` runs the erosion bands, with `flatf` for the frozen flat high ground and the summary's `runoff_weight`
  words, used against the snapshot's own. It also runs the cohorts, the high ground through time and the extremes.
- `recal.sh` gives K's regions' table against Montgomery & Brandon.
- `closedshare.py` gives the closed basins' share of the land: the balance is unchanged, but the rivers now carry what it
  leaves.

Plat-cb0 against plat-rb0 is the runoff alone. No V1 number is a gate (decisions log 2026-10-23).
