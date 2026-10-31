# The surface's runoff from the bake's water budget: design

Status: **built behind a switch, off; its arm ran, verdict default-on** (2026-10-31, the runoff lane, branch
`surface-runoff-budget` on `plateau-drainage` 7df49df8). The switch is `surface.runoff_budget`
(`crates/deeptime-core/src/surface/climate.rs`, ERO-221). The checks against cold basins are ERO-222. The arm plat-rb0
ran on the closed basins ("The arm's results"): the flat high ground wears a third less, the land above 4 km doubles,
K's regions stay inside Montgomery & Brandon's lines, and the export to the sea does not move.

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

Gate (VM, `tools/review/plateau/gate.sh full` on da27fa0d): fmt and clippy (workspace, all targets) clean, deeptime-core
1,158 passed and 12 ignored, its integration tests passed.

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
- **K's regions.** Below 500 m of relief they are unchanged. At 500 m and more they wear 15 to 19 % slower, still
  between Montgomery & Brandon's quiet-land line and their power law (at 500 m and more 150 and 210 against 178.5; at
  1,000 m and more 250 and 525 against 332.4). K was not touched.
- **The export did not move:** 3.34 against 3.36 km³ a year, the runs' pairs off in both directions. The land kept its
  mean lowering (38.9 against 36.8 m/Myr) by standing higher: the export is set by what the plates lift, not by the erosion law.
- **The guard** is plat-cb0's (10.5 km, under the Challenger Deep), and the largest one-step rise is back to orog-cum's.
- **The belt-shape read** moves halfway back (0.188).

**Verdict: default-on.** It meets the project's rule:
- unit tests against its sources (ERO-222);
- no divergence or guard failure on the pool;
- K's regions inside Montgomery & Brandon's lines;
- the criteria at a median 6.

It replaces the snapshot's rule that a land below freezing evaporates nothing (MOI-067's PET for the runoff), which the
bake dropped on 2026-10-01, so the rivers and the lakes then share one budget. It does not reach Tibet's 6 to 16 m/Myr.
The plateaus' flat tops now wear by the edge's cut and by diffusion, not by runoff.

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

**K's calibration.** The regions' erosion at each relief lies between Montgomery & Brandon's lines in every arm
(`recal.sh`). Montgomery & Brandon's rates are of the present-day kind, like the modern loads. The model's land is
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
