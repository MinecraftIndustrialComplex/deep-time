# The surface's runoff from the bake's water budget: design

Status: **on by default since 2026-10-31** (the coordinator's call, decisions log 2026-10-31, after its review's fixes and the arm rb1 on base35-s4)
(2026-10-31, the runoff lane, branch `surface-runoff-budget` on `plateau-drainage` 7df49df8). The switch is
`surface.runoff_budget` (`crates/deeptime-core/src/surface/climate.rs`, ERO-221). The checks against cold basins are
ERO-222. The arm plat-rb0 ran on the closed basins ("The arm's results"): the flat high ground wears a third less, the
land above 4 km doubles, K's regions at 500 m of relief and more stay inside Montgomery & Brandon's lines (those under
it lie above both, as on the base), and the export to the sea does not move.
Its verdict was default-on. The review (an Opus reviewer at a38ab3e9) held that: the young stage and the amplifier
kept the old rule (the pool's runs stop after the volcanoes, so the arm never read them), the months were centred on
the mean, and no test went through the switch ("The review and its fixes"). The branch carries origin/master 0a3ada8c since 242cd673 (the closed basins on by default since PR #52, the crust
budget of PR #53, the hotspot cone budget of PR #54). With `surface.runoff_budget` off it is master's physics but for
one correction: the climate stage's year is taken as it gives it (its warmest month as written, and its twelve months
where it wrote them; "The two rules"), which changes master's closed basins, the flexure's lakes and the bake's water
where the year reaches below freezing. The re-review (an Opus reviewer at 5e04e9f4) found the thaw dropped the mean
on a lopsided year; fixed with the arms again ("The re-review and its fixes").

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
| a mean above 0 °C, a frosty spell | P less Fu's ET at PET 60 T | the biotemperature counts the frozen time as 0 °C, so it is never under the mean and PET is higher: less runs off (P 600 mm at 5 ± 15 °C: 336 against 245 mm, -27 %; a test) |
| no frost | P less Fu's ET at PET 60 T | the same, to 1e-12 (a test) |
| land of either kind below cold headwaters | its drainage area counts the headwaters' weights | less area where the headwaters run off less, so a frost-free valley below cold ground erodes less too |

The months are the climate stage's warmest and coldest month as it wrote them (`climate/t_warm_c`, `climate/t_cold_c`).
The stage writes the mean and both months from the same twelve months with the same lapse rate, and the surface's
temperature is that mean, so they are HYD-033's departures about the surface's mean exactly. Until the review they
were centred on the mean instead (the half-range on either side), which a polar year does not keep: at a mean of
-17 °C with months of -5 and -40 °C it put the warmest month at +0.5 °C ("The review and its fixes"). Without the stage
they are the mean ± 8 °C (HYD-032).

The year's warmth above freezing (the biotemperature, HYD-030) and its thawed share (HYD-031) come from those months
in one of two ways (`hydro::Climate::thaw`, since the re-review):
- **the twelve months**, where the stage wrote them (its last step: the history's last surface step, the young
  stage, and the bake's monthly layers): Holdridge's own method, the mean of the twelve monthly means with those
  below freezing counted as 0 °C, and the thawed share on the year drawn straight between the months' middles;
- **else a sinusoidal year about the mean**, its amplitude the warmest month's departure from it: frosty where
  2 x mean - warmest < 0, no thaw where the warmest month is at or below 0 °C.

The coldest month does not enter the second. A sinusoid through both months, centred between them, drops the mean
when the year is lopsided, and the stage's years are (its winter inversion): the biotemperature then fell under the
mean and jumped where the coldest month crossed 0 °C. Either way it is now continuous in the months and never under
max(mean, 0). Verkhoyansk's biotemperature is 3.05 °C by the twelve months, Holdridge's own value, and 2.70 by the
sinusoid. The sinusoid's cost: a year whose warmest month stands further above its mean than the mean stands above
0 °C is given a frosty spell even when its coldest month is above freezing (mean 4 °C, warmest month 10: a
biotemperature of 4.35 °C whatever its coldest month).

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
    non-glaciated sites and 2.7 to 11.6 on glaciers. The budget's 4.0 lies inside the first range. No source gives
    4.0 itself, so HYD-037 is a `game-choice` inside the range read (as ERO-219's arid loss is), not `verified`.
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
weight is runoff / 0.5 m, floored at 0.05 (ERO-011). The step's wiring is two functions, `surface::step_water` (the
budget, when the lakes or the switch read it) and `surface::runoff_weight`, and a test goes through both
(`the_runoff_budget_reaches_the_stream_power_s_weight`). The young landscape takes the same weight on L1
(`young::runoff_weights`, YNG-011), and with it the amplifier's drainage (`amplify.rs` reads `young/runoff_weight`).
Nothing else reads the change: the aridity index, the facies and the soils keep the snapshot's PET (MOI-067), and the
climate stage's weathering kernel keeps a law of its own (`climate/mod.rs` and `deeptime-cli/src/climate_dem.rs`:
PET = 60 mm/°C x T, 0 below 0 °C, calibrated on `KERNEL_EARTH`), so the one budget does not cover the carbon
cycle's runoff. The year's thaw ("The two rules") is the bake's too: its lakes' water and balance, its playas, and the
lakes' aridity index (`hydro::Climate::aridity`, in the water's stats) take it.

What follows from the budget:
- land with no month below freezing is unchanged (`without_frost_the_budget_is_the_surface_s_runoff`);
- land with a mean above freezing and a frosty month runs off less (`a_warm_mean_with_a_frosty_month_runs_off_less`),
  and so does the land below cold headwaters, whose drainage area counts their smaller weights. K's calibration of
  2026-10-18 was made with the snapshot's rule on all of it, so the regions' table is read again on the arm;
- ground that never thaws runs off nothing, and its weight is the floor. The floor (ERO-011, uncited) is then
  load-bearing: it gives that ground 25 mm a year of runoff in the stream power where the lakes' balance counts 0;
- a cold land with a summer evaporates in it;
- the lakes and the rivers they are fed by share one budget.

## The checks (ERO-222)

Unit tests in `surface/climate.rs` and `surface/plateau_check.rs` (mm/yr):

| site | source | the budget | the snapshot's rule |
|---|---|---|---|
| Mackenzie basin, P 410, months 15 and -25 °C | runoff 169, ET 237 (measured) | runoff 204, ET 206: within Fu's 54 mm | runoff 410, ET 0 |
| Dry Valleys, P 7 to 11, mean -17, coldest -40 °C | "most of which is lost to sublimation" | 0 with the warmest month below 0 °C; under half of P at 2 or 3 °C; 0.29 to 0.87 of P at 0.5 to 1.5 °C (the gap) | all of P |
| Tibetan lakes, P 230 to 420, MAAT 0 to -3 °C (months ± 8 °C, HYD-032) | ET 170 to 290, runoff ratio 0.26 to 0.31 (modelled) | ET 69 to 141, runoff ratio 0.46 to 0.83 (the shortfall printed, not asserted) | all of P |
| Verkhoyansk, Holdridge's twelve months (13.3 and -50.0 °C at the ends) | biotemperature 3.05 °C (Holdridge, rough) | 3.05 °C by the twelve months; 2.70 by the year about the mean from its warmest month | (none) |
| a lopsided year: P 600, mean 4, warmest month 10 °C, coldest 0, +0.01, -0.01 or -6 °C | (none: the method's own property) | biotemperature 4.35 °C, runoff 364 mm, whatever the coldest month (before the re-review: 4.00 and 5.00 °C either side of 0 °C, 381 and 336 mm; months 10 and -6: 3.63 °C, under the mean) | runoff 381 |
| a mean above 0 °C with a frosty month: P 600 at 5 ± 15, 900 at 8 ± 12, 400 at 2 ± 10 °C | (none: the model's own arithmetic) | runoff 245, 456, 189 | runoff 336, 484, 287 (the budget -27, -6 and -34 %) |
| the Dry Valleys' year as the stage gives it: P 7 and 11, mean -17, months -5 and -40 °C | (as above) | 0 | all of P; centred on the mean (+0.5 and -34.5 °C) the budget ran off 5.5 and 9.5 mm, 0.79 and 0.86 of P |

The cold controlled plateau (`a_cold_plateau_runs_off_what_its_summer_leaves`) uses ERO-219's rimmed plateau with Tangra
Yumco's interior climate (P 300, MAAT -1.5 °C) and spills in every case. Its interior weight is 0.600 under the
snapshot's rule and 0.396 under the budget, and its interior net lowering 118.0 and 88.0 m/Myr. An interior that never
thaws wears 3.8 m/Myr.

Gate (VM, `tools/review/plateau/gate.sh full` on da27fa0d): fmt and clippy (workspace, all targets) clean, deeptime-core
1,158 passed and 12 ignored, its integration tests passed. After the review's fixes, the quick gate (`gate.sh quick`):
fmt and clippy (core and cli, all targets) clean, the lane's 80 tests passed; the cold plateau's weights are unchanged
(0.600, 0.396 and 0.050; interior net 118.0, 88.0 and 3.8 m/Myr). The full gates: `cargo test --workspace` 1,474
passed at b20d0e3d (job rb-fix-full), 1,483 at 5dd7ae88 with the flip (job rb-on-full).

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
- **The hydro section's PET and degree-day factor are not pinned.** `hydro.pet_mm_per_c` (60) and
  `hydro.snow_melt_mm_per_degree_day` (4) are the editor's knobs, and the budget reads them, so with the switch they
  steer the history's erosion, as they already steer its lakes (the closed basins, the flexure's water). The snapshot's
  own rule keeps 60 in code (MOI-067). They are documented as such (`HydroParams`, the editor's hints, HYD-029,
  HYD-037), not pinned: one budget for the history's lakes, its rivers and the bake. K's calibration holds at the
  defaults; another PET also parts the budget from the snapshot on frost-free land.
- **No knob, cap or forcing term.** The runoff is the bake's existing, sourced budget.

## The review and its fixes (2026-10-31)

An Opus reviewer read the branch at a38ab3e9. With the switch off nothing changed (the alias was identical), and the
units, the frost-free identity, the NaN cases, the determinism, the lakes and the ice checked out. It asked for fixes
before landing, and no default until the law is armed again:

1. **The young stage and the amplifier kept the old rule** (blocks default-on). `young.rs` built `young/runoff_weight`
   with PET = 60 T, 0 below 0 °C, never reading the switch, and that weight drives L2's stream power and the
   amplifier's drainage (`amplify.rs`). The young lakes already took the budget. A plateau that never thaws at P 300 mm
   weighed 0.05 on L1 and 0.6 on L2: 12 times the stream power the balance uplift (L1's erosion rate) replaces. **Fix:**
   `young::runoff_weights` takes the deep-time surface's rule, the budget on L1's months with the switch on; YNG-011
   and the stage's header say so; `the_young_weights_follow_the_surface_s_runoff` checks it against the surface's own
   weight, cell by cell, on and off.
2. **The months were centred on the mean** (a code path, already on master in the lakes). `cell_water` built
   t ± (warmest - coldest) / 2, but the climate stage writes the mean, the warmest and the coldest of twelve months of
   its energy balance, and the bake keeps their asymmetry (HYD-033). A cell at a mean of -17 °C with months of -5 and
   -40 got +0.5 and -34.5, and some of its precipitation ran off where the bake gives none. **Fix:** `cell_water` takes
   the months; `water_budget` and the young lakes pass the stage's own (`climate::months`); without the stage, the mean
   ± 8 °C (HYD-032) as before. **This changes master's default:** the closed basins (ERO-217, on since PR #52) and the
   flexure's lakes (ISO-126) read the same budget, so lakes on cold ground now get no melt from a year that never
   thaws. Test: `a_year_s_months_are_taken_as_given`.
3. **No test went through the switch.** The ERO-222 tests call the budget directly, and dropping `|| p.runoff_budget`
   from the step failed nothing. **Fix:** the step's wiring is two functions, `surface::step_water` and
   `surface::runoff_weight`; `the_runoff_budget_reaches_the_stream_power_s_weight` runs four climates through them,
   the switch on and off and the lakes' budget alone, and the cold plateau's weights now come from
   `runoff_weight` too. The Tibet test pinned a known shortfall (the budget's ET under Biskop's): it now asserts only
   that the budget evaporates, at most P, and prints the shortfall (`the_tibetan_lakes_evaporate_under_the_budget`).
4. **Docs the data contradicted.** "Below 500 m of relief they are unchanged" and "15 to 19 % slower" are corrected
   from the regions' table ("The arm's results"); "frost-free land is unchanged" now says land with no month below
   freezing, and the table above gains rows for a warm mean with a frosty month (a test:
   `a_warm_mean_with_a_frosty_month_runs_off_less`) and for land below cold headwaters.
5. **Smaller.**
   - The hydro knobs: documented, not pinned ("What it does not do").
   - ERO-011's floor is load-bearing for ground that never thaws (25 mm a year in the stream power where the lakes
     count 0): said so, and marked `game-choice` (no source sets it).
   - ERO-220's `flatf` is a temperature class, not a runoff one.
   - HYD-037 is a `game-choice` inside Hock's range, not `verified`.
   - The bibliography: the orogen lane's Armijo1986 and Blisniuk2001 had landed after this lane's section with no
     blank line; they are back in the collision orogen's section.
   - STATUS's "all 2,996" is gone.
   - The summary's runoff-weight sums run in cell order (they were a parallel f64 reduce).
   - `// ledger: MOI-067` on the snapshot's two 60s.

## The arm

One cumulative 8-run arm, stage 2 on the plateau lane's closed basins. The orogen switches stay on, as in orog-cum
and plat-cb0. With both new switches off, the build is orog-cum's physics; check that first:

    tools/review/pool/pool.sh alias HEAD f2d6412d801f --check-seed 1
    tools/review/pool/pool.sh run plat-rb0 --seeds 1,2,3,4 --set tectonics.collision_two_wedges=true --set tectonics.collision_retro_plateau=true --set surface.closed_basins=true --set surface.runoff_budget=true +DEEPTIME_CRUST_BUDGET=1 +DEEPTIME_CRUST_BUDGET_MAPS=@RUN@/maps
    tools/review/pool/pool.sh table --base orog-cum --arm plat-cb0 --arm plat-rb0

On base35-s4, after the review's fixes (the closed basins and the orogen switches are master's defaults by then).
The months' correction changes the branch's default on cold ground, so the alias check is not expected to be
identical, and the correction gets its own arm beside the law's:

    tools/review/pool/pool.sh run rb-months --seeds 1,2,3,4 +DEEPTIME_CRUST_BUDGET=1 +DEEPTIME_CRUST_BUDGET_MAPS=@RUN@/maps
    tools/review/pool/pool.sh run rb1 --seeds 1,2,3,4 --set surface.runoff_budget=true +DEEPTIME_CRUST_BUDGET=1 +DEEPTIME_CRUST_BUDGET_MAPS=@RUN@/maps
    tools/review/pool/pool.sh table --base base35-s4 --arm rb-months --arm rb1

The young stage is not in the pool's runs (they stop after the volcanoes), so its weight is read by its unit check
(`the_young_weights_follow_the_surface_s_runoff`: L2's weight on L1 equals the deep-time surface's to 1e-6, the
switch on and off).

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

## The arm's results (plat-rb0, 2026-10-31)

Queued by the coordinator against orog-cum. `pool.sh alias HEAD f2d6412d801f --check-seed 1` was identical (`rec.json`
and the world state), so with both switches off the build is orog-cum's physics. Then plat-rb0 ran: key a3b047bb6ddb,
argkey 7591e701, 8 runs, the orogen switches, `surface.closed_basins=true` and `surface.runoff_budget=true`.

The reads (VM): `tools/review/plateau/arm_rb.sh` runs `wear.sh` on plat-rb0, `recal.sh` on the three arms,
`closedshare.py`, and `measures.py`, the cached measures side by side. Before that, `pool.sh measure --arm plat-rb0
--measures guard,coastset,collcoast --registry tools/review/pool/measures_gap4.json --registry
tools/review/pool/measures_boundary.json`. Outputs are in `/data/ban/deeptime-runs/plateau/` (`wear-platrb0.txt`,
`recal.txt`, `arm-rb.txt`). Medians of 8 runs; plat-rb0 against plat-cb0 (the closed basins alone) and orog-cum.

**The pool's table (the coordinator's, against orog-cum):**
- land 0.226 (p 0.19), largest landmass 0.102 (p 0.23), crust under the sea 0.567 (p 0.38);
- land above 2 km 0.017 (p 0.38), coast with high ground 0.107 (p 0.19), land under ice 0.045;
- collisions 5.00 per 100 Myr; criteria met a median 6 (mean 5.88);
- sea at the end 153 m (orog-cum 203, plat-cb0 259).

| read | plat-rb0 | plat-cb0 | orog-cum |
|---|---|---|---|
| runoff weight on land, used / the snapshot's own | 0.731 / 0.833 | (own) | (own) |
| runoff weight on land above 2 km, used / own | 0.438 / 0.902 | (own) | (own) |
| flat high ground: land share | 0.0043 | 0.0025 | 0.0027 |
| flat high ground: net m/Myr (per run) | 91.9 (82 to 123) | 141.4 (114 to 196) | 134.3 (114 to 228) |
| of it frozen (`flatf`): part, net | 0.68, 91.5 | 0.67, 140.8 | |
| of it dry (`flatd`): part, net | 0.74, 63.5 | 0.22, 37.6 | |
| of it draining to a closed basin (`flatc`): part, net | 0.39, 34.0 | 0.30, 52.8 | |
| land above 2 / 3 / 4 km, each run's mean over its maps | 0.0270 / 0.0105 / 0.0022 | 0.0232 / 0.0086 / 0.0010 | 0.0245 / 0.0091 / 0.0012 |
| land above 4 km, the largest snapshot | 0.0228 | 0.0101 | 0.0085 |
| plateau cohorts above 3 km: plates, top (m/Myr of crust) | 68.8, 44.8 | 47.6, 57.7 | 58.9, 53.6 |
| land with relief above 1 / 1.5 km | 0.0155 / 0.0035 | 0.0123 / 0.0025 | 0.0126 / 0.0026 |
| regions of 500 / 1,000 / 1,500 m relief or more, m/Myr | 178.5 / 332.4 / 502.7 | 219.9 / 394.2 / 548.4 | 218.2 / 400.9 / 580.3 |
| regions under 100 / 100 to 250 / 250 to 500 m of relief, m/Myr | 9.0 / 46.2 / 93.5 | 8.3 / 48.0 / 105.3 | 8.9 / 47.9 / 105.2 |
| the land's mean net lowering, m/Myr | 38.9 | 36.8 | 37.1 |
| to the sea / eroded gross, km³ a year (Earth-normalized) | 3.34 / 11.03 | 3.36 / 10.65 | 3.03 / 10.20 |
| land draining to closed basins | 0.119 | 0.127 | |
| guard worst, m; steps beyond one Earth relief | 10,518; 0 | 10,482; 0 | 9,945; 0 |
| largest one-step rise, m; plate peak, m | 7,068; 4,352 | 7,562; 4,463 | 7,106; 4,162 |
| coast with high ground (coastset); collision belt not elongated (collcoast) | 0.099; 0.188 | 0.107; 0.256 | 0.092; 0.075 |

Reading:
- **Where the weight fell.** The budget cut the stream power's runoff weight by half on the high ground and by an
  eighth on the land as a whole.
- **The flat high ground.** It wears a third less (92 against 141 m/Myr, the runs apart), and there is 1.7 times as
  much of it.
- **The high ground stands.** The land above 4 km doubles over the run (0.0022 against 0.0010; Earth's is 0.0195),
  and above 3 km rises a fifth.
- **What wears the dry flat top now.** It has no runoff to speak of: stream power without the cut is 2.7 m/Myr of its
  63.5. The cut at its edge is 28.2 and diffusion 26.1: the escarpment's retreat and κ 2 at 4.8-km cells.
- **K's regions** (against plat-cb0; corrected after the review, which read the table: the first reading said the
  regions below 500 m were unchanged and those above wore 15 to 19 % slower):
  - under 100 m of relief +8 % (9.0 against 8.3 m/Myr), 100 to 250 m -4 % (46.2 against 48.0), 250 to 500 m -11 %
    (93.5 against 105.3);
  - 500 m and more -19 % (178.5 against 219.9), 1,000 m and more -16 % (332.4 against 394.2), 1,500 m and more -8 %
    (502.7 against 548.4).

  They stay between Montgomery & Brandon's quiet-land line and their power law (at 500 m and more 150 and 210
  against 178.5; at 1,000 m and more 250 and 525 against 332.4). K was not touched.
- **The export did not move:** 3.34 against 3.36 km³ a year, the runs' pairs off in both directions. The land kept its
  mean lowering (38.9 against 36.8 m/Myr) by standing higher: the export is set by what the plates lift, not by the erosion law.
- **The guard** is plat-cb0's (10.5 km, under the Challenger Deep), and the largest one-step rise is back to orog-cum's.
- **The belt-shape read** moves halfway back (0.188).

**Verdict then: default-on** (held by the review: the young stage and the amplifier, which the pool's runs do not
reach, kept the old rule, and the months were centred on the mean, so it is to be armed again; "The review and its
fixes"). It met the project's rule:
- unit tests against its sources (ERO-222);
- no divergence or guard failure on the pool;
- K's regions at 500 m of relief and more inside Montgomery & Brandon's lines (those under it above both, as on the
  base);
- the criteria at a median 6.

It replaces the snapshot's rule that a land below freezing evaporates nothing (MOI-067's PET for the runoff), which the
bake dropped on 2026-10-01, so the rivers and the lakes then share one budget. It does not reach Tibet's 6 to 16 m/Myr.
The plateaus' flat tops now wear by the edge's cut and by diffusion, not by runoff.

## The arms after the review (rb-months and rb1 on base35-s4, 2026-10-31)

The head after the review's fixes (b20d0e3d's code, key c9be4b8a512c), 8 runs each on base35-s4's starts and seeds, the
crust budget's words as the base's. rb-months is the branch's default (the months' correction alone, the switch off);
rb1 has `surface.runoff_budget=true`. No alias: the correction changes the default on cold ground.

**The table** (medians of 8, Mann-Whitney p against base35-s4):

| | land | largest landmass | continental crust under the sea | land above 2 km | coast with high ground | land under ice | collisions per 100 Myr | crust's net change | criteria median, mean | sea after 100 Myr / end, m |
|---|---|---|---|---|---|---|---|---|---|---|
| base35-s4 | 0.216 | 0.048 | 0.594 | 0.009 | 0.071 | 0.062 | 4.85 | +0.113 | 5, 5.38 | 280 / 339 |
| rb-months | 0.215 (0.51) | 0.053 (0.38) | 0.605 (0.65) | 0.015 (0.38) | 0.103 (0.51) | 0.061 (0.72) | 4.70 (0.96) | +0.117 (0.96) | 5, 4.75 (0.21) | 332 / 386 |
| rb1 | 0.220 (0.96) | 0.083 (0.083) | 0.609 (0.72) | 0.015 (0.38) | 0.078 (0.72) | 0.066 (0.65) | 4.65 (0.92) | +0.127 (0.23) | 6, 5.62 (0.36) | 365 / 462 |

**The guard** (`measure guard`): worst 10,256 m (rb-months) and 10,554 (rb1) against 10,304; no step beyond two Earth
reliefs; the largest one-step rise 7,486 and 6,959 against 7,144 m; the highest plate 4,461 and 4,490 against 4,413.

**The plateau reads** (`tools/review/plateau/arm_rb1.sh`: `recal.sh` and `wear.sh`; base35's read covers its 24
runs, the arms' 8):
- runoff weight on rb1: land 0.748 used against the snapshot's own 0.835, above 2 km 0.433 against 0.969;
- the flat high ground: rb1 110.0 m/Myr net on 0.0039 of the land (per run 81 to 137), rb-months 146.5 on 0.0029,
  base35 133.3 on 0.0029; of it below 0 C on the year (`flatf`) 0.72, 0.73 and 0.67;
- land above 2 / 3 / 4 km, each run's mean over its maps: rb1 0.0280 / 0.0098 / 0.00115, rb-months 0.0238 / 0.0083 /
  0.00110, base35 0.0239 / 0.0084 / 0.00115;
- K's regions (8 x 8-cell windows by their mean relief, m/Myr; Montgomery & Brandon's power law and quiet-land line in
  brackets): 100 m and more 47.0 (rb-months 50.9, base35 50.0; 15 and 35), 250 m and more 95.7 (109.8, 104.6; 60 and
  75), 500 m and more 183.0 (224.0, 215.7; 210 and 150), 1,000 m and more 329.5 (399.8, 389.6; 525 and 250), 1,500 m
  and more 498.3 (572.6, 534.8; 963 and 350). Against rb-months the law wears them 8 to 18 % slower: between the
  two lines at 500 m of relief and more; above both at 100 and 250 m, as on the base and rb-months, and 8 and 13 %
  closer to them (corrected after the re-review: the first reading said each lay between them);
- the land's mean net lowering 38.3 m/Myr (rb-months 38.6, base35 38.1).

**Reading.**
- **The months' correction is neutral on the planet:** no column moves (p 0.21 or more). It can land as a correction
  of master's default.
- **The law** behaves as plat-rb0 did, on the new base: the high ground's runoff weight halves, the flat high ground
  wears a quarter less (110 against 146.5 m/Myr) and there is 1.3 times as much of it, the land above 2 km is a sixth
  more over the run, K's regions at 500 m and more stay inside Montgomery & Brandon's lines (under it above both, as on the base), the
  land's mean lowering does not move, and
  the guard is the base's. The largest landmass and the criteria rise (0.083, p 0.083; a median of 6 against 5); the
  sea ends higher (462 against 339 m).
- **The young stage** is outside the pool's runs; its weight is the deep-time surface's on L1, cell for cell, on and
  off (`the_young_weights_follow_the_surface_s_runoff`).

**Verdict: default-on.** Unit tests against its sources (ERO-222), no divergence or guard failure on the pool, K's
regions at 500 m and more inside their lines, the young stage and the amplifier on the same rule. It replaces MOI-067's PET for the
stream power's runoff on frozen ground, as the bake did on 2026-10-01.

## The continents' export: the erosion law or the plates' uplift (2026-10-31, a diagnosis)

The crust-budget lane (branch `trench-input`, docs/design/crust-budget.md, plates.md "The trenches' input split by
source and by delivery term") found the continents exporting 3.82 km³ of rock a year to the ocean floor on base34-s4.
That is 2.16 times Clift et al. 2009's long-run 1.77, and the floor's sediment stock agrees. It asked which law sets
the rate. From cached runs only: plat-rb0, plat-cb0 and orog-cum, 8 runs each, and base34, 24 runs.
`tools/review/plateau/export_corr.sh` gives each arm's export and its correlations within the arms;
`/data/ban/deeptime-runs/plateau/export-corr.txt` holds them.

**Earth's numbers, per km² of land.** Earth's land is 0.289 of 5.10 x 10^8 km², about 1.47 x 10^8 km² (this page's
arithmetic, the project's hypsometry figure). Rock lowering per land area, all computed:
- **Clift et al. 2009's long-run rivers,** 1.77 km³ a year of rock (a Cenozoic mean): 12 m/Myr.
- **Modern loads:**
  - Milliman 1997's 6.5 km³ a year, as Clift et al. quote it: 44 m/Myr;
  - Milliman & Syvitski 1992's 20 Gt a year, about 7.5 km³ of rock (the crust-budget lane's arithmetic): 51 m/Myr.
- **Cosmogenic rates** (ERO-171, Portenga & Bierman 2011): basins a median of 54 m/Myr (mean 218), outcrops 5.4
  (mean 12).
- **Montgomery & Brandon 2002** (ERO-172): quiet land at 0.2 mm/yr per km of relief, over a background of 0.01 mm/yr.

**The model.** Its export is 3.03 to 3.86 km³ a year by arm. Over its land, 0.30 of the visible samples (the
crust-budget lane), that is 20 to 25 m/Myr. Its land's mean net lowering, over the surface's own smaller land, is 37 to
39 m/Myr. So the model sheds about twice Clift's long run per km² and about half the modern loads.

**K's calibration.** The regions' erosion at 500 m of relief and more lies between Montgomery & Brandon's lines in
every arm (`recal.sh`); under it, above both, as on the bases read. Montgomery & Brandon's rates are of the present-day kind, like the modern loads. The model's land is
flatter than Earth's:
- relief above 1 km: 0.012 to 0.016 of the land, against Earth's at most 0.05;
- land above 2 km: 0.013 to 0.019, against Earth's 0.102.

Against Clift's long run, then, the model's erosion per relief is several times Earth's: it sheds twice the rock off
lower ground.

**Which one sets the export.** On a 1,000-Myr run the land is near a steady state, where the erosion equals the rock
the plates bring above the sea. A change in the erosion law moves the relief that carries that supply, not the supply.
The cached runs test this in both directions:

| change | what it changed | export, km³ a year (8 runs, median) |
|---|---|---|
| the closed basins (plat-cb0 against orog-cum) | the water below closed lakes, 0.13 of the land draining to them | 3.36 against 3.03 (the runs' pairs off in both directions) |
| the runoff budget (plat-rb0 against plat-cb0) | the runoff weight: -12 % on land, -51 % above 2 km | 3.34 against 3.36; land above 4 km doubled, relief above 1 km +25 % |
| the two orogen switches (orog-cum against base34-s4, same erosion law) | the collisions' wedges and plateau, a plate law | 3.03 against 3.74 (5 of the 8 seed pairs lower, 1 equal) |

- **The erosion law's two changes left the export where it was.** The land answered with relief, and the export over
  gross erosion stayed at 0.31.
- **The plate law's change moved the export by a fifth.**
- **Within the arms,** after each arm's median is taken off (48 runs), the export follows the land's median height
  (Spearman +0.50), the sea's level at the end (-0.40) and the collisions that parted (+0.50). It runs against the
  continental crust left at the end (-0.48): the supply of land above the sea, and an export that drains the
  continents.

**The split.**
- The export's 2.2 times Clift's is the plates' supply: rock lifted above the sea at about twice Earth's long-run rate.
- The erosion law's efficiency, calibrated on present-day rates, shows as the land's low relief, not as the export.
- Retuning K would not move the export. A lower K would raise the relief at the same export; a higher one would flatten
  the land further.
- The runoff law does the first of these, for the cold high ground.

**The lead, on the plates' side.** Measure the rock the plates lift above the sea, by process, on one traced seed. That
needs measurement-only globals, bit for bit with the base, as the crust-budget lane's ti-src: the gap budget
(`DEEPTIME_GAP_BUDGET`, PLT-283's 27 parts) with the crust budget's steps. Set each process against Earth's long-run
references:
- collision thickening and the flow and collapse that spread it (the plateau lane: the plates take 0.57 to 0.69 of a
  plateau's crust after its collision);
- the arcs' additions;
- the flows back to the continents: accretion at resamplings 1.15 and the transition 0.59 km³ a year, on base34-s4;
- the continents' own drain: a net -0.98 km³ a year, so about a quarter of the export is not resupplied.

The flows back come first. They return 0.45 of the export to the continents, where it can be lifted and eroded again,
so a recycling loop would count the same rock twice. The land's lowlands carry the most of the lowering (land below 1
km: 0.53 of it at 21 to 23 m/Myr), which fits rock being returned and re-lifted near the coasts.

The relief shortfall is a separate question: present-day rates against a long-run supply. It is the erosion law's
calibration target (K 1.5e-9 against Montgomery & Brandon, set 2026-10-18), and so not this lane's to retune. If the
plates' supply comes down to Earth's, the relief at today's K would fall further.
