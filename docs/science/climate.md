# Climate

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md). The per-cell climate, the default since P1-26 (2026-10-03), has its own page: [per-cell-climate.md](per-cell-climate.md).

Climate decides temperature, rain, ice and sea level through the planet's history, and with them erosion, glaciers, coal and the biomes of the finished world. Today's code solves an energy balance (sunlight in, heat out, heat carried poleward) in latitude bands, adds wind and rain belts, rain shadows behind mountains, a carbon cycle with a slowly brightening sun, orbital cycles that pace ice ages, land ice and sea ice. Because it works in bands, climate follows latitude far too closely (the design measured latitude explaining 99.6 % of land temperature variance, against 96.8 % on Earth). **This banded model is no longer the default.** Since P1-26 (built 2026-10-02 to 2026-10-03; design `docs/design/p1-26-per-cell-climate.md`, section 11 "As built") a snapshot runs a per-cell, seasonal, two-dimensional energy balance with a slab ocean, wind-driven gyres, an overturning conveyor and, optionally, a moisture budget, and `climate.model` defaults to `per_cell`. **See [per-cell-climate.md](per-cell-climate.md)** for what it does, its constants and how it compares with Earth. The banded model is what this page's older rows describe: it stays as `climate.model = banded`, it is the zonal model that gives the per-cell model its first state, and the fine clock (every 200 kyr) is still one-dimensional, corrected at each snapshot to the 2-D zonal means. Its rows are marked `pending-rewrite`; the per-cell rows (CLM-119 onward) describe code and carry no such mark, except CLM-126 (mountain blocking), which is not built.

The constants of the per-cell design were checked against their sources before the build (WP-14a, 2026-10-02) and the built model's constants are in [per-cell-climate.md](per-cell-climate.md).

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [CLM](ledger/CLM.md) | energy balance | CLM-001 to CLM-026, CLM-119 to CLM-134, CLM-138, CLM-163 to CLM-165, CLM-172, CLM-175 to CLM-178 (51) |
| [CLM](ledger/CLM.md) | insolation and orbit | CLM-027 to CLM-044 (18) |
| [CLM](ledger/CLM.md) | carbon cycle (CLM-069 to CLM-073 are the CO₂ forcing's old law and its anchors; the law is CLM-167) | CLM-045 to CLM-080, CLM-167, CLM-168, CLM-170 (39) |
| [CLM](ledger/CLM.md) | deep-time clock | CLM-081 to CLM-091, CLM-136, CLM-137, CLM-166, CLM-179 (15) |
| [CLM](ledger/CLM.md) | land ice and sea ice | CLM-092 to CLM-112, CLM-135 (22) |
| [CLM](ledger/CLM.md) | mountain glaciers | CLM-113 to CLM-118 (6) |
| [CLM](ledger/CLM.md) | moisture budget (built behind `rain_model = budget`, not the default) | CLM-139 to CLM-146 (8) |
| [CLM](ledger/CLM.md) | the belts' rain amount (`belts_rain_scale`); the upwind rain is on [upwind-rain.md](upwind-rain.md) | CLM-201 (1) |
| [CLM](ledger/CLM.md) | ice branch: the flowing-sheet switch, present-day climate, supply-limited and seafloor weathering (merged 2026-10-02) | CLM-147 to CLM-162 (16) |
| [CLM](ledger/CLM.md) | the present-day climate since 2026-10-18: the target counts ice sheets (CLM-203) and the temperate present's cap on its CO₂ factor (CLM-206, 2026-10-19) | CLM-203, CLM-206 (2) |
| [GLA](ledger/GLA.md) | the flowing ice sheet (`climate/sheet.rs`, off by default) and the ice mass balance for the hydrology; on the glacial page | GLA-067 to GLA-076 (10) |
| [MOI](ledger/MOI.md) | wind and rain belts | MOI-001 to MOI-033 (33) |
| [MOI](ledger/MOI.md) | orographic rain | MOI-034 to MOI-050 (17) |
| [MOI](ledger/MOI.md) | lapse rate | MOI-051 to MOI-053 (3) |
| [MOI](ledger/MOI.md) | seasons | MOI-054 to MOI-060 (7) |
| [MOI](ledger/MOI.md) | climate snapshot | MOI-061 to MOI-076 (16) |

Rows by status (CLM and MOI, 2026-10-03, counted by hand from the two ledger files; `merged` rows not counted): `verified` 23, `cited-unverified` 50, `from-memory` 3, `calibrated` 16, `game-choice` 41, `uncited` 98; `pending-rewrite` 182 (the status mark only); `not-yet-coded` 1 (CLM-126).

## The ice branch, merged 2026-10-02

What the merge added to the banded model (build log 2026-10-01 and 2026-10-02, "Climate ice branch"):

- **Flowing ice sheets** (`climate/sheet.rs`, GLA-067 to GLA-076): shallow-ice flow with Glen's law on a coarse grid, mass balance on the ice surface against the equilibrium line (the elevation feedback), the bed sinking under the ice, calving where the pressed-down bed lies below the sea. On the Lambert grid (master, 2026-10-03) the coarse rows carry their true, uneven sizes (cos φ wide, 1/cos φ high, the gradients taken between the rows' true centres), and a coarse row wholly past a pole holds no ice and passes no flux. **Off by default and in every built-in preset** (`climate.ice_dynamics`, CLM-147; Ben, decisions log 2026-10-02): driven by the banded climate they overgrow. Off, the land ice is the model from before the branch (Nye's equilibrium profile with Imbrie's lag in the ice-age window), and the flexure's ice load (`climate/ice_thickness_m`, ISO rows) is that ice.
- **Supply-limited weathering** (CLM-152 to CLM-157) and **seafloor weathering** (CLM-158 to CLM-162): on by default.
- **The present-day climate** (`climate.present_climate`, CLM-148 to CLM-151): temperate by default.
- **The fine clock's step** is 200 kyr (CLM-082).
- **For the hydrology:** `climate/ice_balance_m_yr`, the land ice's surface mass balance (GLA-076), beside the ice thickness and extent.

Measured on `earthlike_quick_16k` (seeds 1 and 2; scattered start 500 Myr, supercontinent 1000 Myr; final state; build log 2026-10-02): runs on one build are reproducible to every printed digit. With the branch's carbon-cycle settings set back to master's, the merged build with the sheets off printed master's figures exactly. With the defaults (sheets off) the new weathering keeps more CO2 (scattered 113 and 271 ppm against master's 88 and 119; supercontinent 471 and 475 against 237 and 374), so the land ice falls from 0.14 to 0.10 (scattered) and from 0.11 to 0.07 (supercontinent) of the land. Earth today is about 0.10. With the sheets on, the land ice is 0.15 in all four runs, but only because the temperate present raises CO2 to 650 to 1,726 ppm (15 to 18 degC). The sea ice is then 0.03 to 0.07 of the ocean, against 0.07 to 0.14 with them off. Without the present-day nudge, three 500-Myr planets ended with 47 to 52 % of the land under ice (build log 2026-10-01).

Re-measured on master 35dacc0e (the Lambert world, 35 m per block, plate tasks 3–4) against the merged branch, sheets off, scattered start, 500 Myr, seeds 1–3 (build log 2026-10-02, "Readied for landing"): land 0.181 → 0.174 and continental crust 0.298 → 0.309 of the planet (seed-to-seed scatter larger than either change), mean surface temperature 15.3 → 16.1 degC, land under ice 0.116 → 0.064, CO2 444 → 519 ppm (means).

## The present-day nudge on high ground (2026-10-18)

Plate task 8 (continental crust at its Airy height) raised the temperate worlds' CO2 from 730 to
6,100 ppm and their temperature from 15.9 to 21.7 degC at 500 Myr (6 runs each, `plates.md`). The
erosion recalibration traced it (build log 2026-10-18; `earthlike_quick_16k`, 500 Myr, seeds 1 and
2, both starts; the `carbon_series` line). With the present-day setting off (`natural`) the two
models end at the same temperature, 15.1 and 15.05 degC, with CO2 at 579 and 1,009 ppm: task 8's
higher ground holds more ice and so needs more CO2 for the same warmth, but the carbon cycle's
thermostat is unchanged. The rise is the present-day nudge (CLM-148 to CLM-151): it raises CO2
until the land ice is at most CLM-150's 0.15 of the land, and that share counts mountain ice. On
task 8's supercontinent seed 1 the temperate and natural runs are the same to −50 Myr (565 ppm);
at the end the natural run has 0.315 of its land under ice (916 ppm, 13.7 degC) and 0.371 of its
land above 2 km, and the temperate run reaches the bracket's top, 30,250 ppm and 28.3 degC, with
0.156 still under ice. On master the natural end had 0.144 under ice and the nudge did nothing.
Earth's 0.10 is almost all ice sheet, so a target that counts mountain glaciers bites on a
mountainous planet; that is a design question for the present-day setting, not a change to the
deep-time carbon cycle. The recalibrated erosion law (ERO-004 to ERO-007, ERO-013) lowers the high
ground and with it the nudge's CO2 (erosion-and-sediment.md, "The erosion law against measured
rates").

### The target counts ice sheets (decided 2026-10-18)

The coordinator decided it the same day (decisions log 2026-10-18): the temperate present's share
counts ice sheets only, as Earth's ~0.10 does, and the mountain ice is left out; the shares stay
0.15 (temperate, at most) and 0.25 (ice age, at least) (CLM-150). Earth's land is 0.10 under
glacial ice (NSIDC, glaciers, ice caps and the two sheets); Antarctica's almost 14 million km2 and
Greenland's 1.7 million are about 0.105 of it (the land area, 149 million km2, from memory), and the
Randolph Glacier Inventory, which "excludes the ice sheets of Greenland and Antarctica", holds about
706,000 km2, about 0.5 % of the land.

What an ice sheet is comes from the glossary of glacier terms (Cogley et al. 2011, p. 53): "An ice
body that covers an area of continental size, generally defined as covering 50 000 km2 or more"; an
ice cap is the same dome under 50,000 km2, and an icefield covers mountainous terrain without
burying it (NSIDC gives the same 50,000 km2). The code takes it as written (CLM-203): every
continuous body of land ice on the climate's grid (cells with at least 1 m of ice sharing an edge,
the rows wrapping east to west) with an Earth-normalized area of 50,000 km2 or more is an ice sheet;
the rest, glaciers, ice caps and icefields, is mountain ice. It is not a height cut: a high plateau
under one body of ice of that size counts as a sheet, and a long range glaciated along its crest
counts once its ice is continuous over 50,000 km2 (on `earthlike_quick_16k` an L1 cell is about
1,520 km2, so about 33 cells). The present's trials read the sheets from the clock's own ice (the
window's thickness, or the land above the shifted line outside the window), so the sheet share is a
part of the share the clock counted. Only the target changed: the clock's land-ice share, all of the
ice, still sets the albedo and the weathering, and nothing before the last 100 kyr moves. The run
reports both parts (`ice_sheet_land_frac`, `mountain_ice_land_frac` beside `glaciated_land_frac`
in the climate note and the CLI's `climate_final` line; the present's `land_ice_sheet_frac`,
`land_ice_mountain_frac` and `natural_land_ice_sheet_frac`).

**Measured** (build log 2026-10-18, "The temperate present counts ice sheets"; `earthlike_quick_16k`
on the erosion recalibration, the temperate present, `DEEPTIME_GAP_BUDGET=1`, both starts;
`tools/review/force-balance-ensemble.sh ... hyps`; means, CO₂ a geometric mean, brackets the runs'
range; "before" is the same runs with the target counting all of the land ice, erosion-and-sediment.md):

| | Land | Land above 2 km | T, °C | CO₂, ppm | Land under ice | Under ice sheets | Under mountain ice |
|---|---|---|---|---|---|---|---|
| 500 Myr, seeds 1 to 3, before | 0.194 | 0.251 | 19.6 [16.8, 21.6] | 3,377 [1,565, 5,850] | 0.145 to 0.160 | | |
| 500 Myr, seeds 1 to 3 | 0.195 [0.174, 0.212] | 0.249 [0.201, 0.300] | 17.9 [15.5, 19.9] | 2,110 [1,033, 3,782] | 0.173 [0.166, 0.178] | 0.153 [0.149, 0.160] | 0.019 [0.015, 0.025] |
| 1000 Myr, seeds 1 to 4, before | 0.212 | 0.278 | 18.8 [13.7, 23.7] | 3,245 [802, 18,112] | 0.145 to 0.160 | | |
| 1000 Myr, seeds 1 to 4 | 0.213 [0.166, 0.261] | 0.277 [0.193, 0.366] | 17.5 [12.8, 22.1] | 2,318 [623, 12,763] | 0.167 [0.145, 0.177] | 0.147 [0.120, 0.156] | 0.019 [0.015, 0.025] |

- *Mountain ice is a small part of the land ice here,* 0.015 to 0.025 of the land at the end, and
  0.014 to 0.028 in the natural climate the nudge starts from (500 Myr: 0.182 to 0.239 under ice,
  0.166 to 0.225 under sheets). The land ice on these planets lies mostly in bodies of 50,000 km2 or
  more, so dropping the mountain ice lowers CO₂ by about a third (geometric means 3,377 to 2,110 and
  3,245 to 2,318 ppm) and the temperature by 1.7 and 1.3 degC, and no more.
- *The high-ground planets still need the most CO₂.* The 1000 Myr supercontinent seed 2 (0.35 of its
  land above 2 km) ends at 12,763 ppm and 22.0 degC, 11.6 times its natural 1,101 ppm; the
  scattered seed 4 (0.34) at 9,899 ppm, 6.3 times. Two runs (scattered seed 3, supercontinent seed 4)
  need almost no nudge (factor 1.02 and 1.00). The natural CO₂ is 525 to 1,576 ppm in all 14 runs.
- *The target is met as the trials count it:* the present's own sheet share (land at the window's
  start, band-weighted) is 0.149 to 0.150 in the six 500 Myr runs. The step's end share
  (`ice_sheet_land_frac`, the final land by area) is 0.149 to 0.160; the two count different land,
  as the all-ice target's 0.145 to 0.160 did.
- *Nothing earlier moved:* the 1000 Myr supercontinent seed 1 still has its five snowball episodes,
  the land below 0 degC from −700 to −450 Myr (−2.1 degC at −700, −6.3 at −650, 4.2 at −450), as
  before; the change only touches the last 100 kyr. No 500 Myr run had a snowball.

## The climate snapshot every 20 Myr (P1-19 #27, 2026-10-18)

A cost choice for the 900 s gate (plates.md, "The coupled group's cadence"): the pipeline presets
(`earthlike_quick_16k`, `earthlike_full_16k`, `earthlike_quick_32k`, `earthlike_full_32k`) and the
built-in earthlike presets take a full climate snapshot every 20 Myr instead of 10 (SCL-038,
`climate.snapshot_every_myr`; the parameter's default and `planet_world_4k` stay at 10). Nothing
else changed: the first and the last plate step still take a full snapshot, between snapshots the
climate's geographic part still moves with the plates, and the present-day nudge still works from
the last step's natural climate. The plate step stays 2 Myr, so the fine clock (its EBM, the carbon
cycle and the ice) and the erosion still step every 2 Myr; only the per-cell climate fields are
recomputed half as often.

**Measured** (`earthlike_quick_16k`, seeds 1 to 3, both starts, 2 threads, the temperate present;
`tools/review/force-balance-ensemble.sh` and `tools/review/p1-17-hyps.py`; build log 2026-10-18,
"The coupled group's cadence"; means, [min, max], CO₂ a geometric mean; "natural" is the run's own
climate at the last step before the present-day nudge):

| | T, °C | CO₂, ppm | natural CO₂, ppm | land under ice | land above 2 km | climate stage, s |
|---|---|---|---|---|---|---|
| 500 Myr, master | 15.7 [13.2, 19.3] | 837 [477, 2,862] | 550 | 0.142 [0.068, 0.169] | 0.206 | 158.9 |
| 500 Myr, 20 Myr snapshots | 17.7 [15.8, 21.6] | 1,810 [929, 8,208] | 830 | 0.158 [0.146, 0.167] | 0.238 | 124.6 |
| 1000 Myr, master | 17.2 [15.2, 21.4] | 1,209 [425, 4,668] | 630 | 0.139 [0.034, 0.173] | 0.215 | 249.6 |
| 1000 Myr, 20 Myr snapshots | 19.0 [13.8, 27.6] | 1,704 [331, 23,563] | 640 | 0.143 [0.094, 0.168] | 0.205 | 149.8 |

- *The present's CO₂ is higher on average, inside the nudge's spread.* A Mann–Whitney test on the
  six runs each gives p ≈ 0.1 (500 Myr) and 0.8 (1000 Myr). The high means come from three runs
  (one at 500 Myr, 8,208 ppm; two at 1000, 23,563 and 8,409 ppm) whose natural end had 0.28 to
  0.31 of the land under ice sheets, more than any master run (0.24 at most), so the nudge raised
  CO₂ the most there (CLM-150; "The present-day nudge on high ground" above). The natural CO₂ the
  nudge starts from is 830 against 550 ppm at 500 Myr and 640 against 630 at 1000: the carbon
  cycle's thermostat holds at the coarser snapshot.
- *The temperature follows the CO₂,* 2.0 and 1.9 degC warmer on average, with the same three runs
  at the top of the range.
- *The plates do not notice:* one of the 45 plate checks left master's seed range at each length
  (plates.md).
- *The climate stage takes 22 % and 40 % less time* on a shared machine (a guide; the two lengths
  ran at different loads). The per-step part (the fine clock, the fields moved with the plates) is
  untouched and the present-day nudge's trials cost the same at any length, so the share saved
  grows with the run's length. On the full 16k gate run (`earthlike_full_16k`, 1000 Myr,
  8 threads, side by side with master on a loaded machine) the climate stage took 247.0 s against
  332.0 s and the whole run 0.961 of master's time, about 40 s of the quiet 1,242.8 s (plates.md,
  "The coupled group's cadence").

**Again on master of 2026-10-19** (fde7e021, after the plates', climate's and surface's speed-ups;
the same ensembles with the shipped x86-64-v3 builds; the "natural" column was not read):

| | T, °C | CO₂, ppm | land under ice | land above 2 km | climate stage, s |
|---|---|---|---|---|---|
| 500 Myr, master | 18.5 [15.5, 24.3] | 2,237 [1,269, 11,046] | 0.161 [0.155, 0.169] | 0.241 | 91.1 |
| 500 Myr, 20 Myr snapshots | 16.9 [14.2, 21.3] | 1,325 [602, 3,868] | 0.154 [0.143, 0.165] | 0.237 | 68.6 |
| 1000 Myr, master | 17.2 [14.8, 20.3] | 1,664 [634, 3,858] | 0.146 [0.096, 0.159] | 0.275 | 157.5 |
| 1000 Myr, 20 Myr snapshots | 17.5 [13.8, 22.5] | 1,712 [358, 9,305] | 0.159 [0.151, 0.169] | 0.261 | 118.7 |

- *The CO₂ difference of the first measurement does not come back:* the 20-Myr snapshot's present
  is cooler and lower in CO₂ than master's at 500 Myr and level with it at 1000, so both
  measurements sit inside the nudge's spread.
- *The climate stage takes 25 % less time* at both lengths in the ensembles, and 0.81 of master's
  in the full 16k gate run's pairs (both orders, geometric mean; plates.md, "Time on that
  master"). With the stage at about 200 s per 1000 Myr on the quiet gate run after its speed-ups,
  that is roughly 40 to 50 s; the pairs' whole-run ratio (1.008) is below what the loaded machine
  resolves.

What it costs: a climate that changes within 20 Myr is seen in the per-cell fields up to 20 Myr
late instead of 10. A fast change of the clock's climate (a snowball's onset or escape, a hothouse)
still takes a snapshot when it happens: the per-cell model triggers one when the clock's global
temperature or sea ice moves far from the last snapshot's (P1-26 design §4.2), at any interval.
In the runs above the triggered snapshots barely changed: none on the scattered start and two on
every supercontinent run with either interval, except two 1000 Myr supercontinent runs at 20 Myr
with three.

## The present-day nudge's cap (2026-10-19)

The 1000 Myr full 16k gate runs on master (7d60fc4a) ended at 10,011 ppm CO2 (CPU) and 7,981 ppm
(GPU), at 22 to 25 degC. The 100 Myr runs of the same build ended at 385 to 681 ppm. The cause is
the temperate present-day nudge (CLM-148 to CLM-151), not the carbon cycle. It is now capped at
x1.75 (CLM-206). Build log 2026-10-19, "The present-day nudge's cap".

### Where the CO2 comes from

The runs: `earthlike_quick_16k` (500 Myr, 2 threads, seeds 1 to 4, both starts) and
`earthlike_full_16k` (1000 Myr, 8 threads, scattered start, seeds 1 to 4). They used
`tools/review/co2-ensemble.sh` and were read with `tools/review/co2-table.py`. "Natural" is the
run's own CO2 at the last plate step, before the nudge; "factor" is what the nudge multiplied it by.

- *The end CO2 is the nudge's factor times the natural CO2.* On master 26ce664e at 1000 Myr the
  factor was 6.9, 1.6, 9.7 and 12.6 (seeds 1 to 4). The natural CO2 was 1,460, 315, 794 and
  932 ppm. Seed 1 reproduces the gate's 10,011 ppm. The nudge searched up to x10^4, raising CO2
  until at most 0.15 of the land was under ice sheets (CLM-150, CLM-203). It held the carbon reservoirs
  still while it did so. At the end, weathering is therefore 30.91 and 25.97 against a degassing
  of 7.66 and 6.35 x 10^12 mol/yr (seeds 1 and 3): the nudge held the CO2 high, and weathering is
  what that CO2 would draw down.
- *The carbon cycle's thermostat holds.* At every 100 Myr point of every series before the last
  step, weathering equals degassing to within 0.2 %. The natural CO2 shows no drift toward a
  hothouse:
  - Over 1000 Myr it falls from 3,800 to 4,600 ppm at the start to 360 to 960 ppm at -100 Myr,
    as the sun brightens (one run passes 8,600 ppm at -800 Myr and comes back).
  - At -1000 Myr the sun is about 8 % dimmer (from memory), so the planet needs more CO2 for the
    same warmth.
  - The global temperature stays between 9 and 16 degC on the 1000 Myr runs, and between 11 and
    19 degC on the 500 Myr runs, until the last 100 kyr.
  - Over the last 500 Myr the natural CO2 runs from 267 to 3,209 ppm, about 1 to 11 times
    280 ppm, inside the Phanerozoic's 1 to 25 (below). It leaves that range only in the last
    100 kyr, and only through the nudge: master's 1000 Myr ends of 10,011, 7,723 and 11,790 ppm
    are 36, 28 and 42 times 280 ppm.
- *What grows over the history is the high ground.* Land above 2 km goes from 0 at the start to
  0.20 to 0.41 of the land at the end (Earth about 0.1, from memory). The land's share of the
  surface falls from 0.30 to 0.12 to 0.23. Ice sheets form on that ground at any CO2, as
  Antarctica's do.
- *The nudge cannot melt them cheaply.* The present's new report
  (`sheets_by_height`) splits the land at 1, 2 and 3 km. In the four capped 500 Myr runs that
  needed warming (the scattered start, seeds 1 to 4):
  - 0.17 to 0.22 of the land lies above 3 km;
  - 0.68 to 0.88 of that land is under sheets in the natural climate;
  - x1.64 to x1.75 clears 5 to 14 % of it (0.64 to 0.83 stays under sheets);
  - below 1 km at most 0.04 of the land in that class is under sheets, natural or not.

  The trials' rows (`trial_rows`) show the sheet share falling only slowly with the factor. On
  the scattered start, seed 2:
  - x1.0: 0.192 of the land under sheets;
  - x1.42: 0.173;
  - x2.01: 0.157;
  - x2.86: 0.141;
  - x4.06: 0.125.

  The uncapped search stopped at x2.40, 19.2 degC at the end. On three of the 1000 Myr runs,
  with 0.27 to 0.33 of the land under sheets in the natural end, it went to x6.9, x9.7 and
  x12.6.
- *Why 100 Myr looked Earth-like.* In the 1000 Myr series, 100 Myr in, 0.06 to 0.14 of the land
  is above 2 km. A 100 Myr run ends with that little high ground, so (inferred; no 100 Myr run was
  made here) it has few sheets on high ground and the nudge has little to do.
- *Why the runs spread so far.* The planet is chaotic: in the GPU agent's runs, one f32 ulp moved
  a 100 Myr end from 546 to 633 ppm. The nudge then multiplied whatever natural state a run
  ended in by whatever its high ground needed. The end CO2 inherited both spreads. With the cap,
  a last-bit change in the natural state moves the end by at most x1.75, not up to x10^4.

**Which merge?** A bisect on the quick preset (scattered start, seeds 1 and 2, 500 Myr) over the
merges from 25ae6723 to master 26ce664e found three commits that change the worlds. Each moves the
nudge's factor; none moves the natural CO2 outside the spread between seeds. The runs at 5cf2f9bb
(after-group), e2166684 (climate), c767da2c (plates part 2), fde7e021 (surface) and 64982ccd (GPU)
are the same to every printed digit as the run before them.

| commit | end CO2, ppm | factor | natural CO2, ppm | land under sheets, natural | land above 2 km |
|---|---|---|---|---|---|
| 25ae6723 (old master) | 477, 739 | 1.00, 1.00 | 477, 739 | 0.136, 0.134 | 0.243, 0.221 |
| 32589362 (natives and plates part 1) | 1,751, 1,283 | 2.51, 1.97 | 699, 651 | 0.190, 0.177 | 0.199, 0.214 |
| fb38204f (plate cadence: per-Myr rules, 20 Myr snapshots) | 3,868, 1,165 | 2.99, 1.00 | 1,296, 1,165 | 0.187, 0.144 | 0.217, 0.305 |
| 7d60fc4a (surface solvers) = 26ce664e (master) | 2,140, 1,930 | 1.65, 2.40 | 1,294, 805 | 0.178, 0.192 | 0.315, 0.298 |

The old master had the same hothouse on other seeds. In its 500 Myr ensemble (8 runs):
- scattered seed 4 ended at 7,832 ppm (x5.27 on 1,486 ppm);
- supercontinent seed 1 ended at 2,862 ppm (x3.72).

### The carbon cycle against WHAK, GEOCARB and COPSE

These are checks of order of magnitude. The sources' figures are from memory: none of the three
was read for this.

- *WHAK* (Walker, Hays & Kasting 1981): weathering rises with temperature and runoff, so the CO2
  settles where weathering balances degassing. The model does this at every point of every
  series (above).
- *GEOCARB* (Berner): at fixed degassing, the steady CO2 rises as the weathering land shrinks.
  - Weathering goes roughly as RCO2^0.5: the temperature's effect on the kinetics, with about
    4 K per e-fold of CO2, and the runoff's.
  - The 500 Myr planets have 0.65 of Earth's land (0.19 of the surface against 0.29). About a
    sixth of it is under sheets in the natural end, against Earth's tenth, and ice-covered land
    does not weather (CLM-066). That leaves about 0.61 of Earth's weathering land, which needs
    about (1 / 0.61)^2 = 2.7 times the pre-industrial CO2.
  - Their degassing is about 1.2 times Earth's 6.65 x 10^12 mol/yr (`DEGASSING_EARTH_MOL_YR`),
    which squares to 1.44. Together: about 3.9 times, near 1,100 ppm.
  - The natural CO2 at 500 Myr has a median of 653 ppm on the old master and 1,019 ppm on master
    and the fixed branch, the size this estimate gives.
- *The Phanerozoic range:* GEOCARB III's RCO2 runs from about 1 to 25 (Berner & Kothavala 2001),
  and COPSE's from about 1 to 20 (Bergman, Lenton & Watson 2004). Every natural end here lies
  between 1.1 and 6.2 times 280 ppm.
- *Degassing* follows the ridge rate:
  - at the start, 13 to 15 x 10^12 mol/yr, on 6.7 to 7.8 km2/yr of new seafloor;
  - at the end, 4 to 9 x 10^12 mol/yr, on 2.1 to 4.5 km2/yr;
  - Earth makes about 3 km2/yr (from memory).
- *The land at 0.12 to 0.23 of the surface* is the plates' output, not the climate's:
  - the crust thickens into high ground, so the same volume covers less area;
  - collisions take crust under, and the resamplings consume it (PLT-290 to PLT-292).

  The climate takes the land as given. A planet with less land and more of it high is warmer and
  higher in CO2 than Earth for a physical reason.

### The fix

A temperate present raises the CO2 by at most x1.75 (`climate.present_co2_factor_max`, CLM-206).
That is what the ocean gives back from a glacial to an interglacial: Lüthi et al. 2008 extend the
late Quaternary's range to 172 to 300 ppm, and 300 / 172 = 1.74.

- A planet whose sheets that warming does not clear keeps them, as Antarctica and Greenland stay
  through Earth's interglacials. The note marks such a present not reached.
- `ice_age` and `snowball` keep the unlimited search: the user forces them.
- CLM-151's bracket is unchanged below the cap, and the cap itself is the bracket's last step
  before the bisection (code: `present_search`).
- Nothing before the last 100 kyr moves, and the carbon cycle and CLM-150's shares are as they
  were.
- With the cap set to 10,000 the branch reproduces master's runs to every printed digit (the
  quick scattered seeds 1 and 2: 2,140 and 1,930 ppm, x1.654 and x2.399).

**Measured** (median [min, max]; 500 Myr is `earthlike_quick_16k`, 2 threads, seeds 1 to 4,
both starts; 1000 Myr is `earthlike_full_16k`, 8 threads, the scattered start, seeds 1 to 4; "land
under sheets" is at the end, after the nudge; "reached" counts the runs whose present met the
target):

| | runs | end CO2, ppm | factor | natural CO2, ppm | T global, degC | land | land above 2 km | land under sheets | reached |
|---|---|---|---|---|---|---|---|---|---|
| old master 25ae6723, 500 Myr | 8 | 881 [477, 7,832] | 1.46 [1.00, 5.27] | 653 [342, 1,682] | 17.2 [14.8, 23.7] | 0.189 [0.151, 0.219] | 0.254 [0.165, 0.329] | 0.148 [0.062, 0.150] | 8 of 8 |
| master 26ce664e, 500 Myr | 8 | 2,030 [474, 15,463] | 1.67 [1.00, 8.91] | 1,019 [474, 1,735] | 19.2 [15.6, 28.2] | 0.180 [0.124, 0.222] | 0.274 [0.195, 0.353] | 0.148 [0.117, 0.150] | 8 of 8 |
| fixed, 500 Myr | 8 | 1,764 [474, 3,036] | 1.66 [1.00, 1.75] | 1,019 [474, 1,735] | 18.7 [15.6, 21.4] | 0.180 [0.126, 0.223] | 0.274 [0.195, 0.351] | 0.150 [0.117, 0.257] | 5 of 8 |
| master 26ce664e, 1000 Myr | 4 | 8,867 [498, 11,790] | 8.29 [1.58, 12.65] | 863 [315, 1,460] | 24.7 [16.0, 25.9] | 0.206 [0.169, 0.218] | 0.333 [0.203, 0.406] | 0.148 [0.143, 0.150] | 4 of 4 |
| fixed, 1000 Myr | 4 | 1,510 [502, 2,556] | 1.75 [1.60, 1.75] | 863 [314, 1,461] | 18.5 [16.0, 19.3] | 0.212 [0.169, 0.225] | 0.329 [0.203, 0.403] | 0.248 [0.150, 0.288] | 1 of 4 |

- *The fix takes out the multiplication.* At 1000 Myr the median end falls from 8,867 to
  1,510 ppm, the highest from 11,790 to 2,556 ppm, and the temperature from 24.7 to 18.5 degC.
  The four runs' clock series are the same as master's at every 100 Myr point to the printed
  digit, so only the last 100 kyr changed. At 500 Myr master's worst end, 15,463 ppm (x8.9,
  28.2 degC, the scattered start, seed 3), becomes 3,036 ppm (x1.75, 21.4 degC).
- *Where a run needed less than x1.75 the end barely moves.* The supercontinent seeds 1 to 3 are
  the same as master's. The supercontinent seed 4 (2,119 against 2,129 ppm) and the scattered
  seed 1 (2,121 against 2,140) differ a little, because the cap is now the bracket's second step
  and the bisection starts from a different pair.
- *Against the expected range.* The fixed 1000 Myr ends are 1.8 to 9.1 times 280 ppm, inside the
  Phanerozoic's 1 to 25. Master's were 1.8 to 42, three of the four above 25. One of the four
  is Earth-like (1 to 2 PAL): the scattered seed 2 (502 ppm, 16.0 degC). The other three are
  warmer because their natural planets are: 794 to 1,461 ppm, with 0.28 to 0.40 of their land
  above 2 km and 0.27 to 0.33 of it under sheets before the nudge.
- *The cost is ice.* Three of the four 1000 Myr runs and three of the eight 500 Myr runs end at
  the cap with more of their land under sheets than the target's 0.15: 0.244 to 0.288 at
  1000 Myr (Earth 0.10). The ice is on the high ground:
  - in the four 1000 Myr runs 0.10 to 0.26 of the land lies above 3 km;
  - 0.75 to 0.83 of that land is under sheets in the natural climate, and 0.72 to 0.80 after
    the nudge.
- *Threads.* The quick scattered seed 1 at 4 threads ends the same as at 2 threads to every
  printed digit (2,121 ppm, x1.639).

**An independent review of a laptop run** (`earthlike_quick_16k`, supercontinent, 1000 Myr,
seed 1, built from 025676c8, 2026-10-19) found two things:
- The CO2 jumps from 577 to 3,347 ppm at the last step. Its climate note shows the same cause:
  `co2_factor` 5.75, 0.230 of the land under sheets in the natural end and 0.149 after. With the
  cap it ends at no more than 1.75 x 577 = 1,010 ppm, with more of its land under sheets.
- Land ice holds without a break from -760 Myr to the present. At the end, 17 % of it lies within
  30 degrees of the equator, at a median 4.1 km and -9 degC.

  The cap does not touch this: it is the natural climate on high ground. Two things to check,
  neither done here:
  - The glaciers' extent rule (`glaciers.rs`, Ohmura et al. 1992's T_ELA) still puts ice where
    the summer is below -2.4 degC at zero precipitation, so a dry tropical plateau glaciates.
    Neither the extent nor the default thickness (Nye's profile) depends on the snowfall. Only
    the mass balance read by the hydrology and the flowing sheet (off by default) does
    (`ice.rs`, `surface_balance`).
  - The hypsometry (plates.md) puts 0.2 to 0.4 of the land above 2 km, where Earth has about
    0.1.

The present's report in the climate note now carries:
- `co2_factor_max` (temperate only);
- `trial_rows`: every trial's factor, land under sheets, land under ice, sea ice and global
  temperature, the natural end first;
- `sheets_by_height` and `sheets_by_latitude`: the share of the land in each class, and the share
  of each class under sheets, in the natural climate and the chosen one.

The geometry's sums are taken row by row in parallel and added in row order, and the bracket's
trials run side by side and are read in order. The choice therefore does not depend on the
thread count, and the 4-thread run above bears that out.

**On the test planet** (`planet_world_4k`, seeds 11 to 14) the nudge does nothing: the factor is
1.00 on all four, with or without the cap, and 0.000 to 0.010 of the land is under sheets. The
MIC GameTest's `climateConsistent` failure on master ("C 0.621 < 0.630 (132 columns)") is
therefore not a climate fault. It is the census's sampling noise, and the floors now allow for
it (biomes.md, "The floors against the census's noise").

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/CLM.md](extraction-notes/CLM.md), [extraction-notes/MOI.md](extraction-notes/MOI.md).
- Design: `docs/design/p1-26-per-cell-climate.md` (its section 2 measures the faults of the banded model; its section 9 lists references, six of them marked "cited from memory").
- Session notes: `docs/research/notes/session-polar-climate-bands.md`.
- Work package in the verification plan: WP-14 (deferred until the rewrite lands) ([claims.md](claims.md#verification-plan-for-stage-2)).

## The CO₂ forcing

One law at every concentration and temperature, since P1-26 T6 (2026-10-03; `climate/co2.rs`, ledger CLM-167, CLM-168, CLM-170, CLM-172; the sources are in the research note `docs/research/notes/high-co2-forcing-and-snowball-escape.md`):

```text
F(C, T) = m · [OLR_RC(T, 280 ppm) − OLR_RC(T, C)]        m = 0.903
OLR(T)  = A + B T − F(C, T) − c_snow · frozen share       A = 203.3 W/m², B = 2.09 W/m²/K
```

- OLR_RC is the clear-sky outgoing longwave of a radiative-convective model (Kopparapu et al. 2013: 1 bar of N₂ plus CO₂, moist adiabat, saturated, CO₂ collision-induced absorption), as HEXTOR (Haqq-Misra & Hayworth 2022, MIT licence, commit f800e25f) tabulates it: 19 temperatures (190 to 370 K) by 91 CO₂ partial pressures (10⁻⁴ to 10 bar), interpolated linearly in T and in log₁₀ pCO₂ with pCO₂ = C · 10⁻⁶ bar. It is each tile's own surface temperature that is looked up, so the greenhouse of a snowball's cold tropics and of a hothouse differ.
- m scales the clear-sky table so that doubling CO₂ at 288.15 K gives Myhre et al. 1998's all-sky 5.35 ln 2 = 3.71 W/m². F is zero at 280 ppm at every temperature, so Earth's calibration is unchanged. Below the table (about 100 ppm) the forcing continues as a logarithm with the table's own slope between about 100 and 200 ppm.
- It replaced the Myhre law below 1,000 ppm blended into Byrne & Goldblatt 2014's quadratic fit above 2,000 ppm (CLM-069 to CLM-071), which was stated only to 10,000 ppm (computed to 50,000) and was used to 1,000,000 ppm without temperature dependence. Byrne & Goldblatt's values are the cross-check: at 288 K the table (× m) gives 22.7 W/m² at 10⁴ ppm against their 24.1, and 40.4 against 38.1 at 5·10⁴ ppm.
- A + B T already carries about 20 W/m² of longwave cloud greenhouse at snowball temperatures. Extra cloud forcing on frozen surfaces is a setting (`climate.snowball_cloud_w_m2`, default 0, range 0 to 20 W/m²; Ben, decisions log 2026-10-02).
- Not modelled: CO₂ condensation, Rayleigh scattering of a dense CO₂ atmosphere, clouds' state dependence. The source states no error for the table; Kopparapu et al. say the method tends to overestimate the greenhouse effect of dense CO₂ atmospheres. The research note puts it at about ±10 % in forcing at 0.1 to 1 bar from the radiation alone.
- Expected effect: the escape CO₂ of a snowball falls (the note's 1-D stand-in: about 1.9·10⁵ ppm at today's Sun against 3.3·10⁵ with the old law; the real clock's escapes are listed in each run's notes beside Hoffman et al. 2017's 10⁴ to 10⁵ ppm, CLM-166).

## The belts' rain amount, 2026-10-06

On Earth's own relief and CO₂ (`deeptime climate-dem`, scored by `tools/validation/climate_vs_obs.py --obs`
against Beck et al. 2023) the belts rained 532 mm a year over land, against Earth's 773 mm. Ben chose to
keep the belts as the default rain and scale their amount to Earth's (decisions log 2026-10-06, "Belts,
scaled up to Earth's amount (Recommended)"): `climate.belts_rain_scale` multiplies every L1 cell's
circulation rain before the mountains' rain is added, so the pattern stays the belts' (CLM-201, a
calibration on Earth's own inputs, which the 2026-10-05 validation policy allows). Its value, 1.42, gives
773 mm; 1.40 gave 762 and 1.45 790.

| Earth's relief, 280 ppm | Earth (Beck) | Factor 1.0 | Factor 1.42 |
|---|---|---|---|
| Land rain, mm | 773 | 532 | 773 |
| ln annual rain over land, RMSE; bias | – | 1.135; −0.05 | 1.173; +0.33 |
| Köppen exact class; main group | – | 33.8 %; 63.7 % | 30.9 %; 61.8 % |
| Köppen land shares A / B / C / D / E | 0.19 / 0.32 / 0.11 / 0.23 / 0.15 | 0.14 / 0.29 / 0.14 / 0.22 / 0.21 | 0.17 / 0.16 / 0.24 / 0.23 / 0.20 |
| Reference cities in the right main group (target 21 of 24) | – | 19 | 21 |
| Global mean temperature (target 14.1 ± 0.4 °C) | – | 13.82 °C | 14.41 °C |
| The planet's mean rain, land and sea (reference line) | about 990 mm (GPCP, from memory) | 764 mm | 1,095 mm |
| Mumbai; Sahara; Atacama, mm | 3,096; 11; 16 | 623; 244; 179 | 904; 359; 252 |

What it does. Earth's amount is now right, and the reference cities gain two: Mumbai, Buenos Aires and
Anchorage reach their main group, Cairo (now Csa, 436 mm) leaves its desert. The belts' errors are spread differently, not removed: their deserts were already too wet
(the Sahara 280 mm against 25), and a uniform factor wets them further, so the desert share of the land
falls from 0.29 to 0.16 (Earth 0.32) and the Köppen agreement drops by two points. The global mean warms by
0.6 K because fewer bright deserts reflect sunlight (`desert_albedo`, CLM-123). The pattern's errors
(no monsoons, wet deserts) are the follow-up Ben named for both rain models.

**On generated planets (reference lines, 2026-10-16).** Six quick planets (`earthlike_quick_16k`, 500 Myr
to the deposits stage, seeds 1 to 3 of the scattered-continents and the supercontinent starts;
`tools/review/p1-17-hyps.sh`, each run admitted as a 2-thread simulation), each run three ways: the factor
at 1.0 with the weathering kernel measured at 1.0 (`KERNEL_EARTH` 2.475·10⁷, CLM-061); 1.42 with the same
kernel; and 1.42 with the kernel re-measured on Earth's relief under the scaled rain (5.039·10⁷, the default
since 2026-10-16). Mean [min, max] of six:

| | Factor 1.0 | 1.42, old kernel | 1.42, re-measured kernel (default) | Earth (reference line) |
|---|---|---|---|---|
| Land share | 0.197 [0.159, 0.215] | 0.175 [0.150, 0.193] | 0.183 [0.154, 0.235] | 0.289 today (HYP-001) |
| Land rain, mm | 770 [658, 1,209] | 999 [883, 1,078] | 997 [879, 1,101] | 773 (Beck et al. 2023) |
| Land median height, m | 418 [322, 583] | 304 [239, 394] | 256 [204, 352] | 427 (HYP-001's source) |
| Land 90th percentile, m | 1,816 [1,519, 2,082] | 1,513 [1,337, 1,672] | 1,492 [1,253, 1,729] | 2,022 |
| Land above 2 km | 0.085 [0.066, 0.112] | 0.053 [0.040, 0.071] | 0.049 [0.030, 0.071] | 0.102 |
| Global mean temperature, °C | 14.9 [14.0, 16.2] | 12.8 [11.2, 15.4] | 14.9 [12.3, 16.3] | 14.1, preindustrial (CLM-138) |
| CO₂, ppm | 459 [338, 653] | 198 [111, 423] | 384 [120, 632] | 280, preindustrial |
| Glaciated share of land | 0.058 [0.025, 0.084] | 0.107 [0.035, 0.167] | 0.058 [0.029, 0.075] | about 0.10 |
| Köppen B share of land | 0.27 [0.25, 0.30] | 0.15 [0.11, 0.17] | 0.16 [0.08, 0.30] | 0.32 (Beck et al. 2018) |
| Snowballs; hothouses | 0; 0 | 0; 0 | 0; 0 | |

Per planet (land share, land rain in mm, global mean in °C, CO₂ in ppm; factor 1.0 → 1.42 with the
re-measured kernel):

| Start, seed | Land | Land rain | Temperature | CO₂ |
|---|---|---|---|---|
| scattered, 1 | 0.159 → 0.167 | 697 → 942 | 16.2 → 12.3 | 653 → 120 |
| scattered, 2 | 0.182 → 0.154 | 659 → 879 | 15.5 → 16.3 | 540 → 632 |
| scattered, 3 | 0.212 → 0.184 | 726 → 1,032 | 14.8 → 15.0 | 456 → 339 |
| supercontinent, 1 | 0.212 → 0.235 | 1,209 → 935 | 14.0 → 14.4 | 338 → 283 |
| supercontinent, 2 | 0.201 → 0.185 | 658 → 1,095 | 14.3 → 15.2 | 387 → 396 |
| supercontinent, 3 | 0.215 → 0.174 | 673 → 1,101 | 14.5 → 16.1 | 381 → 533 |

What held and what did not:

- The amount scales across seeds and land shares: the land rain rose ×1.33 to ×1.66 on five planets. On the
  sixth (supercontinent, seed 1) the factor-1.0 run ended unusually wet (1,209 mm), and both 1.42 runs ended
  at 883 and 935 mm. The planets diverge (the climate feeds the erosion and the erosion the crust's loads), so
  a run's end at 1.42 is another world, not the same world wetter. Planets rain more over land than Earth's
  relief at the same factor (about 1,000 against 773 mm): their land is smaller and nearer the sea. Earth's
  own relief still rains 773 mm at 1.42 (re-run on 2026-10-16 with the climate code unchanged).
- No planet ran away: no snowball and no hothouse in 18 runs.
- **The weathering normalisation did not hold** and was re-measured. The scaled rain more than doubles the
  weathering of Earth's own land (the kernel on Earth's relief goes from 2.327·10⁷ to 5.039·10⁷: under Fu's
  curve the runoff grows faster than the rain), so with the old kernel every planet weathered as if it had
  twice Earth's land and drew its CO₂ down to 111 to 423 ppm (four of six below 150), 2.1 K colder on
  average with nearly twice the land ice. CLM-061 is re-measured whenever the climate's defaults change; with the new
  value the mean temperature and land ice are back to the factor-1.0 runs'. One planet (scattered, seed 1)
  still ends cold, at 120 ppm and 12.3 °C.
- **Wetter land erodes more**, as expected: the land's median height falls from 418 to 256 m, its 90th
  percentile from 1,816 to 1,492 m and the land above 2 km from 0.085 to 0.049 of the land, further from
  Earth's 427 m, 2,022 m and 0.102 (the P1-17 hypsometry was calibrated at the unscaled rain). The land share
  loses 0.014 (0.197 to 0.183), against Earth's 0.289 today. Over the Phanerozoic Earth's sea level rose and
  fell by some 100 to 250 m (STE-013, from memory), flooding and draining part of the continents; a land
  share for those high stands has not been read. These are reference lines, not targets (validation policy,
  2026-10-05).
- The desert share falls on planets as on Earth's relief (0.27 to 0.16 of the land, Earth 0.32): the belts'
  deserts are too wet, and a uniform factor wets them further.

## Known before the page is written

- Land ice thickness used a hard-coded gravity of 9.81 m/s2; since P1-26 T6 it takes the planet's gravity (CLM-100).
- Earth-like runs end near 210 to 220 ppm CO2, not the 280 ppm the carbon cycle is balanced for (flag 7).
- Ice sheets overgrow on the banded climate (47 to 52 % of the land under ice on three 500-Myr Earth-like planets, build log 2026-10-01 "Climate ice branch"), so the flowing sheets (`climate.ice_dynamics`, CLM-147, GLA-067 to GLA-076) are off by default and in every built-in preset (decisions log 2026-10-02, "Merge with ice sheets off"); they are turned on once the per-cell climate (P1-26) fixes the cold bias. See "The ice branch, merged 2026-10-02" below.
- Stage 1 spot checks (before P1-26): the CO2 forcing fit (Byrne & Goldblatt 2014) was stated valid for 200 to 10,000 ppm and used up to 1,000,000 ppm (replaced, see "The CO₂ forcing"); a snowball state ended at 0.55 bar of CO2 where Hoffman and others 2017 give 0.01 to 0.1 bar (the new law lowers it, not yet to that range); the banded model's Earth mean temperature of 15.6 degC is above any long-term mean (Berkeley Earth: 14.1 degC for 1951 to 1980; the per-cell model is calibrated to 14.1 ± 0.4 degC, CLM-138).
- The energy-balance constants are a mix: A, B and D attributed to North, Cahalan & Coakley 1981 (not readable: a scan), albedos from two different climlab classes that climlab does not combine.
