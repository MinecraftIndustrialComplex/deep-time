# Erosion, sediment and sedimentary rock

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

Through the whole history, rivers cut into rising land (the stream power law: erosion grows with the water a river carries and with its slope), hillsides creep downslope, rock weathers to soil, and the eroded material is carried off and laid down as sediment on land, in lakes and on the sea floor, where reefs also build limestone. Each deposit is recorded as a layer with a rock type chosen from where and how it was laid (its facies: for example sandstone near shore, shale offshore, coal from peat swamps). This is the stage that makes valleys, plains, deltas, continental shelves and the layered rock under them.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [ERO](ledger/ERO.md) | stream power erosion | ERO-001 to ERO-021 (21) |
| [ERO](ledger/ERO.md) | hillslope diffusion | ERO-022 to ERO-025, ERO-164 to ERO-167 (8) |
| [ERO](ledger/ERO.md) | sediment | ERO-026 to ERO-044 (19) |
| [ERO](ledger/ERO.md) | marine deposition | ERO-045 to ERO-056 (12) |
| [ERO](ledger/ERO.md) | carbonate | ERO-057 to ERO-067 (11) |
| [ERO](ledger/ERO.md) | weathering | ERO-068 to ERO-077 (10) |
| [ERO](ledger/ERO.md) | flow routing | ERO-078 to ERO-085 (8) |
| [ERO](ledger/ERO.md) | facies and lithology | ERO-086 to ERO-163 (78) |
| [ERO](ledger/ERO.md) | submarine slope failure | ERO-168 to ERO-170 (3) |
| [ERO](ledger/ERO.md) | erosion audit, plateaus and the measured diffusivity | ERO-171 to ERO-178 (8) |
| [ERO](ledger/ERO.md) | the erosion law against measured rates | ERO-179 to ERO-182 (4) |
| [ERO](ledger/ERO.md) | surface numerics: the diffusion solve, the parallel flood | ERO-183 to ERO-184 (2) |
| [ERO](ledger/ERO.md) | pelagic and periplatform carbonate, photic diagnostics, euxinic veto | ERO-191 to ERO-197 (7) |
| [ERO](ledger/ERO.md) | chemical denudation: the dissolved load of the carbon cycle's silicate weathering | ERO-205 to ERO-207 (3) |
| [ERO](ledger/ERO.md) | the pelagic rain: dust, the dissolved load back as opal and carbonate, its test, its facies | ERO-208 to ERO-216 (9) |
| [ERO](ledger/ERO.md) | closed basins: what a lake passes on by its water balance, the internally drained plateau test, the flat high ground split by its water | ERO-217 to ERO-220 (4) |
| [ERO](ledger/ERO.md) | the stream power's runoff from the bake's water budget (cold lands evaporate in their summer, frozen ground runs off nothing) and its checks against cold basins | ERO-221 to ERO-222 (2) |

Rows by status (2026-10-18, `tools/review/claims_counts.py`): `verified` 10, `cited-unverified` 82, `from-memory` 2, `abstract-only` 4, `calibrated` 8, `game-choice` 11, `uncited` 56.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/ERO.md](extraction-notes/ERO.md).
- Research note: `docs/research/notes/surface_processes_climate.md`.
- The implementation follows Braun & Willett 2013 for the implicit stream-power solve; the research note read that paper's abstract only.
- Work package in the verification plan: WP-3 ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- The erodibility `k_spl` was 2e-9 in the Rust default and 3e-9 in all eight presets, the comment describing the older value (extraction notes ERO, flag 1); since 2026-10-18 both are 1.5e-9 and the comment describes it ("The erosion law against measured rates").
- The weathering constants (50 m/Myr, depth scale 5 m) have no source; the research note says the rate constants "were not retrieved" (flag 2).
- The facies rules are, in the research note's words, "my proposal, not a published scheme that was checked".
- Stage 1 spot checks: the soil-production depth scale of 5 m is 10 to 20 times the 0.25 to 0.5 m that secondary sources give for Heimsath's law (ERO-070, MISMATCH at secondary level); the hillslope diffusivity of 2 m2/yr (ERO-023) is 550 times the only primary value read (Roering et al. 1999, read on 2026-10-16: 0.0036 m2/yr measured on hillslopes, ERO-177; kept at the map scale, see "Plateaus and the hillslope diffusivity", and checked against Montgomery & Brandon's quiet-land rate in "The erosion law against measured rates"); the Landlab carbonate and marine constants match Landlab's source.

## Continental margins and submarine slope failure (margins and spires task, 2026-10-03)

**What was wrong.** The vertical-scale study found coasts and continental margins steep in metres, not only in the
drawing. Measured with shore-normal transects from every seventh open-sea coast cell (the shore is where the
elevation crosses sea level; land masses under 750 km² are skipped, about 50,000 km² on Earth by the planet's area),
the planets' slope between the 500 m and 2,000 m isobaths was 15 to 21° on the finished terrain (L3, master
00ef214c; 9.5 to 10° on master f545c0bc, after the continental crust fix), against a median of 2.1° on Earth
(ETOPO 2022, 60 arc-second surface, the same code: 3,777 transects of land masses over 50,000 km² within 70° of the
equator). Pratson and Haxby (1996, abstract read) give regional slopes of 0.5 to 4.4° for five U.S. margins
(ERO-168).

**Where it came from.** The plate model's own elevation (`plates/elevation_m`) already holds the wall: its
ocean–continent step is one plate sample wide, and the Earth-normalized plate geometry is drawn 8.1 times narrower
on the map (the decision "Tectonics scaling"). Three things kept it: marine diffusion vanishes below the 60 m wave
base (ERO-045), so nothing relaxed a slope under the sea; the mouth fill raises every sea cell it reaches to the fill
level, so a prograding shelf ended in a one-cell cliff; and the plate model's oceanic dampening relaxes any load on
old sea floor back to the age–depth curve in about 47 Myr (PLT-129), so the sediment piled at a margin's foot
subsides away. The young landscape and the amplifier then steepened what L1 had (finer cells, the same drop).

**What the code does now.** Submarine slope failure (`marine::slope_failure`, ERO-168 to ERO-170): sediment on the
sea floor steeper than `surface.submarine_slope_deg` (2.5°, Pratson and Haxby's steepest clastic margin) slides down
to the slope's toe and the rise, in the deep-time surface stage after each step's diffusion and in the young stage.
Only sediment fails (the strata log and loose sediment), only between open-sea cells; the plate model's crystalline
margin does not slump. Changing the dampening to subside oceanic crust at its age's rate was tried and withdrawn: it
kept every load and the oceans filled (land 0.23 to 0.62 on one planet); it needs the sediment budget recalibrated
with it. A narrower change, a floor raised by the sediment fill's own isostatic rise (Steckler and Watts 1978, from
memory), was measured on six runs on 2026-10-21 and also taken back out: it lifted the sea-floor pockets that hold a
third of the plate era's lakes but did not cut the lakes, and raised the land 0.242 to 0.297, because the sea level
is a prescribed curve and not the ocean's water over its basins (rivers-and-lakes.md, "Basins that outlast their
sediment"; PLT-129). The surface's lake rule, which built a causeway across every lake and left the rest of it empty,
was the law that kept the game's lakes (HYD-110). Since 2026-10-21 the sea level is the ocean's water over its basins
(sea-level.md) and the floor sinks at GDH1's rate whatever lies on it (PLT-352), so the floor keeps its sediment and
the sea rises over it: on the six-run ensemble the land went from 0.267 to 0.321 of the planet (map p50) and from
0.266 to 0.273 in the bake, and the age-rate trial's filled oceans did not recur.

**Measured** (quick 16k, seed 2; planet A supercontinent 1,000 Myr, planet B scattered 500 Myr; before = master
f545c0bc, after = this change on it; medians of the transects on the baked terrain, metres):

| | Earth (ETOPO 2022) | A before | A after | B before | B after |
|---|---|---|---|---|---|
| Slope 500 to 2,000 m, degrees | 2.1 | 9.5 | 4.2 | 10.0 | 5.9 |
| Slope, shelf break to foot, degrees | 2.2 | 6.8 | 3.6 | 7.5 | 4.3 |
| Steepest 4 km, degrees | 5.5 | 15.2 | 8.0 | 17.3 | 11.8 |
| Shore to the 200 m isobath, km | 25 | 14.5 | 8.5 | 13.0 | 7.0 |
| Shelf-break depth, m | 108 | 84 | 56 | 76 | 47 |
| Share of the sea under 200 m (L1) | | 0.049 | 0.039 | 0.060 | 0.041 |
| Sea median depth, m (L1; Earth 4,062) | | 4,179 | 3,797 | 4,170 | 3,686 |
| Land, share of the planet (L1) | | 0.180 | 0.188 | 0.266 | 0.277 |

Over four 500 Myr planets (seeds 1 and 2, both starts; `tools/review/p1-17-hyps.sh`, the deep-time surface) the
failure moves the land share from 0.291 to 0.266 (Earth 0.289), the sea's median depth from 4,143 to 4,250 m (Earth
4,062), the share of the sea under 200 m from 0.057 to 0.037, and leaves the continental crust (0.386, 0.383) where it
was; the same pair on the code before the continental crust fix moved the land the other way (0.271 to 0.283), so the
hypsometry shifts by a few hundredths with the planet, while the shelf loss is consistent (build log 2026-10-03). The shelves get
narrower: the failure takes the fill front down the slope instead of letting it stand, so less of each river's load
stays on the shelf. Shelf-hosted deposits fell with them on both planets (coal seams 49 → 34 and 23 → 14,
Mississippi Valley-type zinc–lead 98 → 81 and 145 → 46, petroleum traps 6 → 0 and 12 → 4; placers 18 → 29 and
30 → 18), though one planet each is not a sample.

**What is left.** The margins are still about twice Earth's slope: where no sediment covers the plate model's
crystalline step, nothing relaxes it, and the dampening keeps eating the rise. Earth-like margins need either an
ocean–continent transition of Earth width in map metres in the plate model or a sediment budget that keeps loads on
old sea floor (the withdrawn change, recalibrated). The land side is unchanged: the continents stand 400 to 800 m
high 10 to 30 km from the coast (Earth: 61 and 144 m), the same Earth-normalized compression on land.


## The erosion and burial audit (2026-10-05)

**What was reported.** On a baked `earthlike_quick_16k` planet a porphyry formed at 1.5 km (its
`formation_depth_m`, DEP-017) sat 560 blocks (19.6 km) under the ground, and the porphyries that
were eroded away had lost 19 to 35 km of rock. Porphyries are emplaced 1 to 4 km deep and exhumed
by a few kilometres; Ben asked for the cause to be found and fixed (decisions log 2026-10-05).

**How it was measured.** `deeptime erosion-audit` (`tools/review/erosion-audit.sh`) reads a run's
recorded history as the deposit ledger reads it and reports, per region type (ERO-173: orogen,
basin, shield, other land, shelf, ocean), the area-weighted erosion and deposition rates of every
frame; per crust parcel, followed with the plate, the totals over the run; and every porphyry's
ledger. Earth's rates are printed beside them as reference lines, never as targets (decisions log
2026-10-05): cosmogenic outcrops erode at 12 m/Myr on average (median 5.4), drainage basins at 218
(median 54), seismically active basins at 367 and quiet ones at 182 (Portenga and Bierman 2011,
ERO-171; 10Be samples favour steep, eroding places); quiet land erodes at about 0.2 mm/yr per km of
local relief and active ranges faster (Montgomery and Brandon 2002, ERO-172).

**The causes: bookkeeping, not the rates.** Three things, none of them the stream-power or
isostasy physics:

1. *The frames were summed at fixed cells.* The surface stage averages its deposit fields over a
   10-Myr frame, and the ledger reads frame k where each deposit's crust is at the frame's end. The
   sums were kept per L1 cell, but the crust moves 7 to 18 cells in a frame on the 16k map (30 to 70
   mm/yr; the plate model's 200 km arc-to-trench distance is 5 cells there), so an arc's frames
   carried the trench's fill and the forearc's stripping that other crust went through. Now
   `FrameAcc::compose_back` carries the sums with the crust each step: a frame's value at a cell is
   what the crust now there went through. Over all porphyries of the 1000-Myr planet the median
   burial fell from 9.5 to 2.5 km and the 90th percentile from 30.5 to 15.6 km.
2. *The fate ignored the order of erosion and burial.* The ledger and the survival integral used
   the net z_f + B − E, so a body the surface went through and that was buried again counted as
   whole: on the planet above, porphyries with 16 to 22 km of erosion and as much burial survived
   at 1 to 6 km. The fate now follows the deepest the surface ever cut towards the body (DEP-168):
   eroded through means gone; cut and then covered is placed under its old erosion surface.
3. *"Lost 19 to 35 km" is the erosion to today, not to destruction.* A destroyed porphyry's
   `deposit_exhumation` event carries all the erosion since formation, hundreds of Myr of it; the
   body itself was gone once the net erosion passed its base (about 1.5 km plus its half-height).
   EVENTS.md now says so.

**The rates themselves** sit inside Earth's ranges per region type and are not changed: on the
500-Myr planet (frames, after fix 1) orogens erode at 135 m/Myr on average (median 73, 90th
percentile 365; 4.5 % of orogen area-time above 500 m/Myr), shields at 17 (median 7), basins at 1
with 8.5 m/Myr of deposition, shelves take 44 m/Myr of deposition. Over a crust parcel's whole
history the totals are large because the time is long: parcels now in orogens lost 20 km over 500
Myr (40 m/Myr on average) and 28 km over 1000 Myr; parcels now in shields lost 11 and 19 km, most
of it while they were orogens (their peak frame rates have a median of 100 to 160 m/Myr).

**Before and after** (`earthlike_quick_16k`, scattered continents, the preset's seed; `tools/review/erosion-audit.sh
bake|report`; "before" is master 2e07d954 with the ledger recorded on the events, fix 1 the frames carried with the
crust, fix 2 also the fate by the deepest cut). The terrain is the same in all three: the fixes change the bookkeeping
the deposit engine reads, not the surface processes.

| | 500 Myr before | fix 1 | fix 2 | 1000 Myr before | fix 1 | fix 2 |
|---|---|---|---|---|---|---|
| Porphyries formed | 79 | 352 | 484 | 249 | 973 | 975 |
| Porphyry sites (counted / target N\*) | 7 (5 / 6.2) | 25 (21 / 25.0) | 27 (21 / 25.0) | 25 (7 / 15.5) | 68 (19 / 61.5) | 28 (17 / 61.5) |
| Site centre depth under the ground, blocks: median, 90th pct, max | 16, 81, 318 | 28, 175, 356 | 23, 51, 166 | 81, 475, 507 | 45, 221, 537 | 26, 133, 369 |
| Sites deeper than 100 blocks (3.5 km) | 1 | 4 | 2 | 11 | 19 | 5 |
| Burial since formation, all porphyries, km: median, 90th pct | 0.4, 7.6 | 0.5, 5.0 | 0.5, 5.3 | 9.5, 30.5 | 2.5, 15.6 | 2.5, 15.8 |

The formation count and N\* change with fix 1 because the favourability the engine integrates is read along the
crust too: an arc's fields stay with the arc, its tracts are larger and the target follows the area (USGS density
times permissive area, DEP-166). At 1000 Myr the formation hits the cap (20 formed per survivor), so the counted
survivors stay under N\*; that is reported, not tuned.

The deepest sites left are old porphyries under thick sediment: at 1000 Myr site 91 formed 337 Ma, was cut 1.9 km
down and then buried under 15 km of sediment (45 m/Myr), now 369 blocks (12.9 km) down; at 500 Myr the deepest is a
porphyry formed 85.5 Ma, cut, then covered by 9.7 km (166 blocks). Shelves and basins take 42 to 44 and 8 m/Myr of
deposition on average in the frames; Earth's thickest basin fills are 15 to 20 km (from memory, not checked), so these
are deep burials, not impossible ones.

**Hypsometry, a reference line** (the deep-time surface at the end, the same in every run above): 500 Myr: land 0.150
of the planet, land median 613 m, 90th percentile 2,054 m, 11.2 % of land above 2 km; 1000 Myr: land 0.128, median
410 m, 90th percentile 1,830 m, 7.8 % above 2 km (Earth, HYP rows: about 0.29 land, median about 430 m, about 10 %
above 2 km). Rivers and lakes come from the same terrain and are unchanged (lakes 503 and 292, river chains 1,189
and 1,049 in every bake).

**High ground, a reference line for plateaus** (land erosion by the frame's elevation band, after the fixes, the
same in both runs within a few m/Myr): land 0 to 1 km erodes at 36 m/Myr on average (median 6), 1 to 2 km at 146 to
155 (median 77 to 92), and every band above 2 km at 116 to 144 (median 49 to 69, 90th percentile 325 to 410).
These bands mix ranges and plateaus; the model has no plateau that wears more slowly than a range of the same height.
Earth's reference: southern Tibet's plateau surface has been exhumed by less than 0.01 km/Myr (under 750 m in all)
since 12 to 11 Ma, and its rivers' cosmogenic rates are mostly under 100 m/Myr (Tremblay et al. 2015, ERO-174). So
the surface stage wears land above 2 km five to ten times faster than a plateau interior on Earth (medians 50 to 70
against under 10 m/Myr); a plateau the plate model holds up is being worn at range rates. The section "Plateaus and
the hillslope diffusivity" below measured why.

**Left as it is.** The rates are the stream-power and isostasy physics as calibrated before (ERO-001 to ERO-021,
ISO rows); with the bookkeeping fixed they sit inside Earth's ranges per setting, and the decisions log of
2026-10-05 rules out tuning them towards a depth or hypsometry. A parcel's total over hundreds of Myr is large
because it spends part of that time in orogens; how much a stable craton is exhumed over 1 Gyr on Earth (a few km in
the thermochronology literature, from memory, not checked) is the next reference line to source.

## Plateaus and the hillslope diffusivity (2026-10-16)

**What was asked.** The audit above found land above 2 km wearing at 116 to 144 m/Myr (median 49 to 69), five to
ten times a plateau interior's rate on Earth (ERO-174). The candidate cause: the map is drawn 8.15 times narrower
than Earth (40,075 km against the 16k planet's 4,915 km), so linear diffusion, which goes with the curvature, acts
about 66 times stronger on the same relief, while stream power with m = 1, n = 2 does not change. Tibet and the
cosmogenic rates stay reference lines, never targets (decisions log 2026-10-05).

**Where the cell size enters.** The surface stage works in real metres at map scale (decisions file, "Tectonics
scaling": "surface physics (erosion, rivers, slopes) stays in real metres at map scale"): one L1 cell of the 16k
quick planet is 4.8 km (16 blocks of 300 m, ERO-084), the drainage area is summed in m² of those cells and slopes are metres
of drop over metres of cell. All three laws use that one length, so there is no unit or scaling error. What the
narrower drawing does to each law:

- stream power, E = K A^m S^n: with m = 1 and n = 2, A grows as L² and S² falls as 1/L², so the same landform drawn
  at any width erodes at the same rate in metres of height. The knickpoint celerity C = n K A^m S^(n-1) grows as L,
  so a knickpoint crosses the same number of cells per step at either width;
- linear diffusion, dh/dt = κ ∇²h: κ Δt / Δx² is 66 times larger on the map for the same relief;
- the critical slope S_c (ERO-012, ERO-013): a fixed gradient, so ground drawn 8.15 times steeper reaches it far more
  often.

**The controlled plateau** (ERO-176, `surface/plateau_check.rs`, `cargo test -p deeptime-core plateau --
--nocapture`): a plateau 4 km above the sea (0 to 30 m of noise) behind an escarpment to a 200-m plain, worn for
10 Myr by the surface stage's own priority flood, routing, stream power (K 3e-9, m 1, n 2, G 1) and diffusion, on
4.8-km cells and on cells 8.15 times wider. Gross lowering by process, m/Myr, of the interior (two cells and more
inside the escarpment's top) and of the escarpment:

| Case | Interior: stream power, diffusion, net | Escarpment: stream power, diffusion, net |
|---|---|---|
| Map, κ 2, S_c 0.35 (the presets) | 118.4, 21.6, 124.6 | 227.1, 13.3, 222.9 |
| Earth's width, κ 2, S_c 0.35 | 308.7, 0.4, 308.5 | 271.9, 0.2, 271.0 |
| No κ, no S_c (map and Earth's width alike) | 305.1, 0, 304.9 | 271.7, 0, 270.9 |
| Map, κ 0.0036 (ERO-177) | 302.0, 0.1, 301.7 | 271.6, 0.0, 270.9 |
| Map, κ 2, dry (runoff weight 0.1) | 8.2, 4.7, 10.1 | 24.8, 29.0, 42.7 |
| Map, κ 2, terminal lake at the centre | 126.2, 22.0, 132.4 | 227.4, 13.4, 223.3 |
| Map, κ 0.0036, terminal lake at the centre | 298.1, 0.1, 297.9 | 271.5, 0.0, 270.9 |
| 1-cell escarpment, no κ, S_c 0.35 (map and Earth's width alike) | 299.3, 0, 299.0 | 307.0, 0, 306.3 |
| Map, κ 2, a rim range 500 m higher; the closed interior spills over it (the deep-time stage's routing) | 142.6, 18.5, 145.7 | 275.5, 17.6, 276.8 |
| Map, κ 2, the same rim; the closed interior ends in a terminal lake | 116.6, 18.6, 121.1 | 275.2, 14.4, 274.5 |

Without diffusion the two widths give the same numbers to the decimal (stream power is width-invariant); diffusion
is about 50 times stronger on the map's escarpment (13.3 against 0.2 m/Myr) and lowers the interior's stream power
from 305 to 118 m/Myr, because it rounds the escarpment's lip and the knickpoint behind it slows. The critical
slope cuts nothing in any case, the 1-cell escarpment included: the escarpment's rivers carry enough area that
stream power keeps it below 0.35 (on the planet the cut works on peaks and ridges, where the area is small). A
terminal lake does not save the plateau: at the centre of a plateau without a rim it changes nothing (132 against
125 m/Myr), and behind a rim range it lowers the interior's wear only from 146 to 121 m/Myr. The interior's own
rivers can take no more than its 30 m of noise down to the lake (the lake holds its base level at 4,000 m), so the
rest, over a kilometre in 10 Myr, must come from the outer rivers cutting through the rim and capturing the basin.
Only less runoff slows the plateau.

**The planet** (`earthlike_quick_16k`, scattered continents, the preset's seed, 500 Myr, master 4e6b0022 with the
diagnostic of ERO-175: every surface step, each land cell's gross lowering by process, area-time weighted in its 1-km
band of height above the sea; high ground split into flat, under 500 m of relief in its 3 x 3 neighbourhood, and
steep). m/Myr; the critical-slope cut is part of the stream power:

| Band | Area, 10⁶ km² Myr | Stream power (of it, S_c cut) | Glacial | Diffusion | Net lowering |
|---|---|---|---|---|---|
| 0 to 1 km | 650 | 24.6 (0.0) | 0.2 | 13.0 | 32.2 |
| 1 to 2 km | 130 | 132.7 (0.3) | 2.6 | 28.0 | 152.6 |
| 2 to 3 km | 54.6 | 128.0 (9.6) | 4.8 | 35.2 | 159.0 |
| 3 to 4 km | 16.7 | 129.8 (42.0) | 6.3 | 43.4 | 170.6 |
| above 4 km | 9.8 | 123.0 (109.5) | 6.4 | 64.5 | 190.0 |
| above 2 km, flat | 20.0 | 37.0 (0.8) | 1.5 | 26.1 | 62.8 |
| above 2 km, steep | 61.1 | 157.5 (37.3) | 6.6 | 45.1 | 198.6 |

Above 2 km stream power is 63 to 76 % of the gross lowering and diffusion 21 to 33 %. A quarter of the high ground
is flat, and it wears at 63 m/Myr (37 of it stream power, 26 diffusion): six times southern Tibet's plateau surface
(under 10 m/Myr, ERO-174), while the steep three quarters wear at 199. The critical-slope cut grows with height, from
9.6 m/Myr in the 2-km band to 110 of the 123 m/Myr of stream power above 4 km: there it is the process that wears
the ranges, and with diffusion it is one of the two laws here that change with the drawn width.

**κ at a measured value.** The only primary diffusivity read, Roering, Kirchner and Dietrich 1999 (ERO-177), is
0.0036 m²/yr (0.0031 to 0.0045 per hillslope) for soil creep on Oregon Coast Range hillslopes tens of metres long,
in real metres; the shipped 2 m²/yr (ERO-023) has no source and is 550 times larger. eSCAPE, a global landscape
evolution model, takes D = 0.01 m²/yr on a 200-m mesh and D = 0.1 on its 16-km global mesh (Salles 2019, ERO-178): a
model's choice, not a measurement, and the paper does not say it is set by the resolution. A bake of the same
planet with κ = 0.0036 (every other setting the same; `tools/review/erosion-audit.sh`, scratch tag `kd0036`):

| | κ 2 (shipped) | κ 0.0036 (Roering) | Earth (HYP rows) |
|---|---|---|---|
| Net lowering 2 to 3 / 3 to 4 / above 4 km, m/Myr | 159 / 171 / 190 | 145 / 163 / 141 | |
| Flat high ground: net (stream power, diffusion) | 63 (37, 26) | 46 (45, 0.1) | under 10 (ERO-174, reference) |
| Steep high ground: net (stream power, diffusion) | 199 (158, 45) | 178 (172, 3) | |
| Land 0 to 1 km, net | 32 | 23 | |
| Land share | 0.158 | 0.198 | 0.29 (HYP-001) |
| Land median, m (surface; climate line) | 583; 613 | 284; 305 | 427, range 430 to 640 (HYP-002) |
| Land above 2 km (climate line) | 0.124 | 0.066 | 0.102, range 0.10 to 0.13 (HYP-004) |
| Highest point, m | 4,123 | 5,476 | |
| Köppen land A / B / C / D / E | 0.17 / 0.30 / 0.25 / 0.08 / 0.20 | 0.16 / 0.43 / 0.17 / 0.10 / 0.14 | |
| Land rain, mm/yr; glaciated land | 697; 0.054 | 619; 0.039 | |
| Snowballs; mass residual, km³ | 0; -7.2e-4 | 0; 1.7e-4 | |
| Frames (audit): land erosion 2 to 3 / 3 to 4 / above 4 km, mean (median), m/Myr | 131 (55) / 144 (55) / 134 (58) | 115 (29) / 127 (26) / 94 (15) | |
| Porphyry sites (counted / target) | 22 (14 / 25.0, capped) | 27 (13 / 14.5) | |
| Site centre depth, blocks: median, p90, max; sites deeper than 100 blocks (3.5 km) | 24.7, 136.1, 388.5; 3 | 38.6, 222.0, 434.8; 10 | emplaced 1 to 4 km deep |
| Burial since formation, all porphyries, km: median, p90 | 0.5, 4.9 | 0.5, 8.5 | |

With the measured κ diffusion all but stops at 4.8-km cells (a cell's relief relaxes in L² / κ, about 6 Gyr), and
stream power takes over the slopes diffusion rounded: on flat high ground stream power rises from 37 to 45 m/Myr, so
the plateau's net falls only from 63 to 46, still about five times Tibet's interior. Diffusion was not what wears the
plateaus: in the controlled case it protects them, rounding the escarpment's lip so the knickpoint behind it is
slower.

**The cause.** The plateaus wear at range rates because of the stream power as calibrated, not because of a
length scale. K = 3e-9 with n = 2 (ERO-004, ERO-007) was fitted so the planet's hypsometry comes out right, not to
measured erosion rates, and it wears any plateau that sheds water at a hundred metres or more per Myr at either width:
with n = 2 a knickpoint's celerity C = 2 K A S is large for the large drainage area an escarpment's rivers collect,
and on the map the escarpment's knickpoint crosses the controlled plateau within one 2-Myr step, at Earth's width
too (the law is the same at both widths). The drainage does not change that: the priority flood routes every closed
basin over its rim (ERO-080; only land no sea cell can reach drains to a sink of its own, ERO-083), but a terminal
lake in the same place saves little (121 against 146 m/Myr behind a rim range, 132 against 125 without one).
Diffusion slows the interior rather than wearing it (125 against 305 m/Myr without κ or S_c), and the critical
slope cuts nothing on the controlled plateau; on the planet it works on peaks and ridges (110 of the 123 m/Myr of
stream power above 4 km), and S_c = 0.35 (ERO-013) has no source at a 4.8-km cell. In the controlled cases only less
runoff slows the plateau (10 m/Myr at a tenth of the runoff). The planet's plateaus are wet enough that the
calibrated law wears them as ranges.

**Left as it is, and why.** κ stays 2 m²/yr and S_c 0.35. The length scale is the decided one and consistent, so
there is no scaling error to fix; the measured κ does not fix the plateaus (46 against under 10 m/Myr) and moves the
world's shape away from two calibrated targets (land median 284 m and 6.6 % above 2 km, against the 430 to 640 m and
0.10 to 0.13 the 3e-9 erodibility, ERO-004, was calibrated for with κ = 2). Changing κ alone would break that joint
calibration, and the decisions log of 2026-10-05 rules out tuning K back towards the hypsometry. Keeping both was the
coordinator's call (STATUS.md, follow-ups from decided questions). What would make plateaus wear like plateaus is
calibrating K, n, κ and S_c together against measured erosion rates (ERO-171, ERO-172) instead of the hypsometry;
that is a question for Ben (STATUS.md, pending questions). Taken up on 2026-10-18 ("The erosion law against
measured rates", below): the law now erodes at the measured rates, and the plateaus still wear at range rates.

## The erosion law against measured rates (2026-10-18)

**What was asked.** Recalibrate the surface erosion law (K, n with m = n / 2, κ, S_c) against
measured erosion rates, Portenga & Bierman 2011 (ERO-171) and Montgomery & Brandon 2002 (ERO-172),
instead of the hypsometry, which becomes a reference line (decisions log 2026-10-17; STATUS.md
follow-ups). It went on top of plate task 8 (crust at its Airy height), which had raised the land
above 2 km from 0.18 to 0.27 of the land.

**The sources.**

- Montgomery & Brandon 2002, read in full (ERO-179 to ERO-181). Their local relief Rz is the range
  of height within a 10-km circle, averaged over a range or a basin; against it erosion follows
  their eq. 2, E = 0.01 + 2.5·10⁻⁴ Rz / (1 − (Rz / 1500)²) mm/yr (Rz in m), with Rc = 1,500 m a
  limiting relief where landsliding takes over, and over all their regions the power law
  E = 1.4·10⁻⁶ Rz^1.8, which in their words underestimates the rates at high relief. Tectonically
  quiet land erodes at about 0.2 mm/yr per km of relief (ERO-172). At most 2 % of the land has Rz
  above 1,500 m and at most 5 % above 1,000 m.
- Portenga & Bierman 2011 (ERO-171): ¹⁰Be basins erode at a median 54 m/Myr (mean 218), outcrops at
  5.4 (mean 12); the basins are sampled where rivers cut, so they are not a continental average.
- Ruetenik et al. 2023 (ERO-182), read in full: fitted to 3,640 catchments' cosmogenic rates with
  linear hillslope diffusion the slope exponent is n ≈ 2.3 (1 % quantiles 1.59 to 2.70), with m/n
  held at 0.45; without diffusion n ≈ 1.28. They give no absolute K. Harel et al. 2016 (the source
  ERO-006 named) could not be fetched (403).

**How the planets are read against them** (`erosion_bands`, ERO-175; `tools/review/erosion-recal.py`).
Every surface step each land cell's relief is the range over it and its four edge neighbours at the
step's end, after its uplift and erosion, the heights the implicit stream power acts on (at 4.8-km
cells the stencil spans 9.6 km, about Montgomery & Brandon's 10-km circle). Their points are
regions, so the planet is read in regions too: 8 x 8-cell windows (38 km) at least half land,
classed by their land's mean relief, with the net lowering rate of their land. The fit is the RMS
of the log ratio between the regions' rate and Montgomery & Brandon in six classes from 100 m of
relief up: eq. 2 below 1 km, the span between eq. 2 and the power law at 1 to 1.5 km, the power law
above. The share of land with relief above 1 and 1.5 km is read against ERO-181, the regions'
median rate against Portenga & Bierman.

**The critical slope from Rc.** The cut acts on each receiver link (no cell stands more than S_c
times the link's length above the cell it drains to, ERO-012), so S_c caps the relief over a span
of any length at S_c times that length, whatever the cell size. Montgomery & Brandon's Rc over their
10-km circle gives S_c = 1,500 / 10,000 = 0.15 (ERO-013), at every preset's cell size alike
(2.4 km full, 4.8 km quick). 0.35 let 10 km of span stand 3.5 km high.

**κ from the quiet-land line.** With linear diffusion a parabolic hill of half-width L and relief
Rz lowers at 2 κ Rz / L². For Montgomery & Brandon's quiet-land rate, 0.2 mm/yr per km of relief,
κ = 10⁻⁷ L² m²/yr: 2 m²/yr for hills 9 km across, the size their circle and the map's two-cell
stencil see. So κ = 2 at the map's scale is consistent with the measured rates (a check, not a
fit; the hillslope-scale 0.0036 of ERO-177 is for slopes tens of metres long). On the planets
diffusion carries 9 to 30 m/Myr of the regions' 51 to 201 m/Myr at 100 to 1,000 m of relief; the
rest is stream power. κ stays 2 (ERO-023).

**K and n.** Candidates on plate task 8's model (`earthlike_quick_16k`, 500 Myr, seeds 1 and 2,
both starts, the temperate present; `tools/review/force-balance-ensemble.sh` with `--set`). A
different n is set at the same erosion where sqrt(A) S is 130 m (about 50 m/Myr): K' = K 130^(2−n).
Regions' net lowering, m/Myr, by their land's mean relief:

| Run | K, n, S_c | Land | Land above 2 km | Crust | T, °C | CO₂, ppm | Land with relief above 1 / 1.5 km | 100–250 m | 250–500 | 500 m–1 km | 1–1.5 km | 1.5–2 km | 2–3 km | RMS log error | Regions' median, m/Myr |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Montgomery & Brandon | | | | | | | at most 0.05 / 0.02 | 54 | 110 | 210–260 | 525–1,033 | 963 | 1,830 | | |
| Master | 3e-9, 2, 0.35 | 0.175 | 0.196 | 0.307 | 16.2 | 785 | 0.061 / 0.030 | 75 | 143 | 175 | 204 | 261 | 483 | 0.87 | 10 |
| Task 8 | 3e-9, 2, 0.35 | 0.187 | 0.282 | 0.378 | 22.4 | 6,609 | 0.080 / 0.045 | 65 | 116 | 163 | 243 | 341 | 599 | 0.71 | 13 |
| S_c 0.15 | 3e-9, 2, 0.15 | 0.195 | 0.258 | 0.380 | 17.7 | 2,105 | 0.067 / 0.017 | 80 | 150 | 241 | 454 | 815 | 1,122 | 0.30 | 17 |
| n 2.3 | 6.97e-10, 2.3, 0.15 | 0.179 | 0.269 | 0.361 | 22.0 | 5,297 | 0.063 / 0.018 | 70 | 141 | 246 | 460 | 790 | 883 | 0.35 | 13 |
| n 2.3, S_c 0.2 | 6.97e-10, 2.3, 0.2 | 0.198 | 0.261 | 0.374 | 21.2 | 5,012 | 0.070 / 0.031 | 73 | 138 | 206 | 342 | 545 | 956 | 0.42 | 12 |
| n 2.5 | 2.63e-10, 2.5, 0.15 | 0.203 | 0.198 | 0.406 | 17.4 | 1,672 | 0.062 / 0.016 | 76 | 156 | 254 | 463 | 793 | 1,229 | 0.27 | 14 |
| **K 1.5e-9 (chosen)** | 1.5e-9, 2, 0.15 | 0.198 | 0.249 | 0.386 | 18.9 | 2,835 | 0.060 / 0.016 | 51 | 116 | 201 | 420 | 786 | 1,095 | 0.25 | 9 |

(Means of four runs; CO₂ a geometric mean. m = n / 2 in every row. Half the land lies in regions
under 100 m of relief, which every row erodes at 7 to 12 m/Myr, about eq. 2's floor of 10.)

What the table says: S_c is what held the high ground. At 0.35 the regions above 1 km of relief
eroded at 0.3 to 0.5 of the power law and 4.5 % of the land had more than 1.5 km of relief, twice
Earth's most; at 0.15 they erode at 0.6 to 0.9 of it and 1.6 to 1.8 % of the land is that rough.
K then sets the low-relief land, where most of the land is: at 3e-9 it eroded at 1.3 to 1.5 times
eq. 2, at 1.5e-9 within 6 %. n 2.3 and 2.5 at the same erosion fitted no better than n 2 (0.35 and
0.27 against 0.25), and n 2 lies inside Ruetenik et al.'s range, so n stays 2 (ERO-006, ERO-007).
The regions' median, 9 to 17 m/Myr in every row, is below Portenga & Bierman's basins (54) and
above their outcrops (5.4): a planet's regions are mostly quiet land, the basins mostly are not.
The spread between seeds is large (CO₂ from 1,565 to 5,850 ppm in the chosen row), so the rows'
land, high ground and climate differ within it; the erosion against relief is what separates them.

**The law chosen:** K = 1.5·10⁻⁹ m⁻¹ yr⁻¹, n = 2, m = 1, S_c = 0.15, κ = 2 m²/yr, in all eight
presets and the Rust default (ERO-004 to ERO-007, ERO-013, ERO-023; the editor hint, HNT-036).

**Plateau wear, still a gap.** The controlled plateau (ERO-176) wears its interior at 98 to 145
m/Myr under every law tried (K 1e-9 to 6e-9, n 2 or 2.5, S_c 0.15 or 0.2; `DEEPTIME_PLATEAU_LAWS`)
and at 131 under the chosen one (stream power 126, diffusion 20, the critical slope 0.6), against 125 before; the critical slope cuts nothing there.
Only less runoff slows it (10 m/Myr at a tenth of the runoff). Tibet's interior wears at under 10
m/Myr (ERO-174), so the plateau gap of 2026-10-16 stands at about twelve times: it is the knickpoint
of a wet plateau's escarpment, not the law's constants. On the planets the high ground's flat part
(3 x 3 relief under 500 m at the step's end) wears at 105 m/Myr with the chosen law, half of it the
critical-slope cut: measured at the step's end, ground the cut has just flattened counts as flat,
so this is not the 2026-10-16 measure (63 m/Myr, read before the step's uplift and erosion).

**Measured: master, plate task 8 and the recalibration.** `earthlike_quick_16k`, the temperate
present, `DEEPTIME_GAP_BUDGET=1`, both starts; master 6ee43e53, task 8 at 70dfb2a3, the
recalibration on top of task 8 (`tools/review/force-balance-ensemble.sh`, `tools/review/erosion-recal.py`).
The recalibration's rows are seeds 1 to 3 at 500 Myr (six runs) and 1 to 4 at 1000 Myr (eight);
master's and task 8's are the same seeds for the world and the climate, seeds 1 and 2 (four runs)
for the erosion columns. Means; CO₂ a geometric mean; brackets the runs' range.

| | Land | Land median height, m | Land 90th percentile, m | Land above 2 km | Continental crust | 40–55 km crust below its height, m | T, °C | CO₂, ppm |
|---|---|---|---|---|---|---|---|---|
| Earth | 0.289 | 427 | 2,022 | 0.102 | about 0.41 | | about 15 | 280 |
| Master, 500 Myr | 0.178 | 736 | 2,744 | 0.179 | 0.305 | 2,326 | 15.9 | 730 |
| Task 8, 500 Myr | 0.192 | 836 | 4,287 | 0.274 | 0.380 | 1,623 | 21.7 | 6,100 |
| Recalibrated, 500 Myr | 0.194 [0.174, 0.211] | 796 | 3,741 | 0.251 [0.203, 0.301] | 0.377 | 1,766 [1,620, 1,869] | 19.6 [16.8, 21.6] | 3,377 [1,565, 5,850] |
| Master, 1000 Myr | 0.180 | 851 | 3,092 | 0.220 | 0.324 | | 16.6 | 1,110 |
| Task 8, 1000 Myr | 0.195 | 1,142 | 5,210 | 0.343 | 0.391 | | 25.8 | 18,900 |
| Recalibrated, 1000 Myr | 0.212 [0.166, 0.260] | 980 | 4,055 | 0.278 [0.195, 0.367] | 0.414 [0.321, 0.479] | 1,821 [1,650, 2,253] | 18.8 [13.7, 23.7] | 3,245 [802, 18,112] |

| | RMS log error against Montgomery & Brandon | Land with relief above 1 / 1.5 km | Regions' median rate, m/Myr | High ground's flat part, m/Myr |
|---|---|---|---|---|
| The sources | 0 | at most 0.05 / 0.02 (ERO-181) | basins 54, outcrops 5.4 (ERO-171) | Tibet's interior under 10 (ERO-174) |
| Master, 500 Myr | 0.87 | 0.061 / 0.030 | 10 | 85 |
| Task 8, 500 Myr | 0.71 | 0.080 / 0.045 | 13 | 41 |
| Recalibrated, 500 Myr | 0.21 | 0.064 / 0.016 | 11 | 107 |
| Master, 1000 Myr | 0.92 | 0.090 / 0.043 | 20 | 75 |
| Task 8, 1000 Myr | 0.76 | 0.142 / 0.079 | 36 | 45 |
| Recalibrated, 1000 Myr | 0.19 | 0.103 / 0.027 | 29 | 75 |

The recalibrated regions' net rate, m/Myr, by their land's mean relief (under 100 m, then the six
classes of the table above): 7.7 / 54 / 122 / 208 / 425 / 807 / 1,199 at 500 Myr and 10 / 58 / 108 /
192 / 405 / 799 / 1,329 at 1000 Myr, against Montgomery & Brandon's 54 / 110 / 210 to 260 / 525 to
1,033 / 963 / 1,830: eq. 2 within 10 % and the power law at 0.66 to 0.84.

What the measurements say:

- *The erosion now matches the measured rates* at both lengths (RMS 0.19 to 0.21 against 0.71 to
  0.92 before), and the roughest land is rare again at 500 Myr (1.6 % above 1.5 km of relief). At
  1000 Myr it is 2.7 %, above Montgomery & Brandon's 2 %, and the land with relief above 1 km is
  6 to 10 % in every configuration, above their 5 %.
- *The high ground fell, but not to Earth's.* Land above 2 km is 0.25 to 0.28 of the land (task 8
  0.27 to 0.34, master 0.18 to 0.22, Earth 0.10), the 90th percentile 3.7 to 4.1 km (Earth 2.0).
  With the erosion at the measured rates, what is left is the uplift's and the crust's, not the
  erosion law's (inferred; the model's subducting boundary is about five times Earth's, plates.md).
- *The gap is about where task 8 left it.* The 40–55 km crust stands 1,766 m below its height at
  500 Myr (task 8 1,623, master 2,326), of it 1,376 m made on the ocean floor. Erosion and its
  rebound together take 512 m of it (erosion 15,784 m, rebound 15,272), against task 8's 344 and
  master's 401: faster erosion of the thick crust deepens it a little.
- *CO₂ is lower but still high,* 3,200 to 3,400 ppm against task 8's 6,100 to 18,900. The rise is
  the present-day nudge holding the land-ice share at 0.15 on high ground (climate.md, "The present-
  day nudge on high ground"): every run still ends at 0.145 to 0.160 of the land under ice, and the
  runs with the most land above 2 km need the most CO₂ (18,112 ppm at 0.35 of the land above 2 km,
  802 ppm at 0.22).
- *A snowball in one run.* The supercontinent start, seed 1, at 1000 Myr went through five snowball
  episodes, with the land below 0 °C from about −700 to −450 Myr; none of the other 13 recalibrated
  runs did, nor master's or task 8's four 1000-Myr runs. One run in eight; not chased.
- *The plate checks* (`plate-check`, 1000 Myr, seeds 1 to 4, both starts): 25 of 45 targets met,
  against task 8's 26 and master's 29. Moved to fail: collision length, 90th percentile (107 →
  125 Myr, at most 120); most land gained in any step (passed before; 1.53 % of the planet, at most
  1.50, two runs at 2.5). Moved to pass: steps on which the speed guard binds (0.12 → 0). Still
  failing and moved: supercontinent tenure 21 → 86 Myr and period 82 → 168 Myr (100 to 300, 400 to
  800), slab ÷ free plates 2.17 → 2.36 (at least 2.5), continental arc median height 1,095 →
  1,163 m (1,200 to 4,500), the wedge 59 → 48 km, ocean floor older than 250 Myr 4.6 → 5.0 %. The
  others are close to task 8's. The erosion law reaches the plates through the
  loads and the crust's height; the moves are of the size that seeds make (inferred).
- *The structure stage's arcs* (DEF-026, 500 Myr, seeds 1 to 3): mean lift factor 0.259, strain
  0.107, 180 fold trains and 608 thrusts a run at the Andean margin, 235 collision fold trains
  (task 8 with the old law, natural climate, seeds 1 and 2: 0.257, 0.105, 180 and 564, 249). The scale stays
  0.1 mm/yr.

**Left open.** The plateaus' interior wear (131 against under 10 m/Myr) does not move with the law's
constants. The high ground's excess (0.25 to 0.28 above 2 km against 0.10) and the land's roughness
at 1000 Myr (2.7 % above 1.5 km of relief) remain with the erosion at the measured rates. The temperate present's target counted mountain ice,
so on these planets it asked for thousands of ppm of CO₂ (CLM-150). Decided the same day (decisions
log 2026-10-18): it counts ice sheets only (CLM-203; climate.md, "The target counts ice sheets").

## Surface numerics: the diffusion solve, the flood, and the slope failure's stopping (P1-19, 2026-10-18)

**What was asked.** Three of the surface stage's costs in the full 16k gate run (P1-19 exit item
1: 1,242.8 s against 900): the implicit diffusion solve (109.3 s, about 111 conjugate-gradient
iterations a solve), the submarine slope failure (83.8 s, 40 iterations a step) and the serial
Priority-Flood+ε (32.7 s). Each change is a setting under the preset's `surface.numerics`, so the
old and the new method can run side by side. Determinism is not asked for (Ben, 2026-10-18: "I am
not worried about determinism, performance is far more important"). None of the three changes a
law; they change how closely, and how fast, the laws are solved.

**The diffusion solve (ERO-183).** The equation is the same, (I + Δt·L)h¹ = h⁰ with L the
Laplacian of the cells' diffusivities (ERO-022 to ERO-024, ERO-046), solved by conjugate gradients
from the same split first guess and with the same mass put-back. Only the preconditioner changed:
one geometric multigrid V-cycle instead of the diagonal. Jacobi preconditioning leaves the
condition number growing with D·Δt/Δx², which the sea's 100 m²/yr over a 2-Myr step on 2.4-km
cells makes about 35, so its iterations grow with it; a V-cycle damps every wavelength at once
(Briggs, Henson & McCormick 2000), and as a preconditioner of conjugate gradients it need only be
symmetric and positive definite (Tatebe 1993; the test `the_cycle_is_a_symmetric_positive_definite_operator`).
The cycle aggregates 2 × 2 cells, moves values between levels piecewise-constant, takes half the
summed fine couplings as the coarse ones (Braess 1995: the plain Galerkin operator of
piecewise-constant aggregation is about twice too stiff for smooth error), smooths with damped
Jacobi (ω 0.8, two sweeps before and after) and ends at a level of at most 64 cells with 16
sweeps. The cycle runs in f32 and the conjugate gradients in f64; its sums are taken per strip
of rows and added in order, so the result is the same at any thread count. Since the merge with
the byte-identical speed-ups (2026-10-19) the solve keeps its f64 vectors between steps in the
stage's workspace, as the Jacobi solve does; the cycle's levels are still built for each solve.

On a 2048 × 1024 synthetic shelf (the test `tune_the_cycle`, 8 threads) the Jacobi solve took
110 iterations and 0.400 s; the cycle took 8 iterations and 0.087 s to the same 10⁻⁶ m, and 7
iterations and 0.069 s to 10⁻⁵ m. Two sweeps took 6 to 8 iterations against one sweep's 10 to 13,
10 to 20 % faster; coarse couplings of 0.5 to 0.6 of the Galerkin sum were the fastest.

On the planet the solve is checked against both the old solve and a tight one
(`DEEPTIME_DIFFUSION_AUDIT=1`, `earthlike_full_16k`, scattered continents, 100 Myr, 8 threads,
54 solves; each multigrid solve also run by the Jacobi solve and to 10⁻⁹ m):

| Solve | Iterations | Against the Jacobi solve, largest | RMS | Error, largest | RMS | Worst solve's RMS error |
|---|---|---|---|---|---|---|
| Jacobi, 10⁻⁶ m (before) | 127.2 | | | 0.024 mm | 3.0 × 10⁻⁷ m | 3.7 × 10⁻⁷ m |
| Multigrid, 10⁻⁶ m, one sweep | 27.4 | 0.038 mm | 3.1 × 10⁻⁷ m | 0.032 mm | 1.0 × 10⁻⁷ m | 1.7 × 10⁻⁷ m |
| Multigrid, 10⁻⁵ m, two sweeps (the default) | 17.2 | 0.68 mm | 1.5 × 10⁻⁶ m | 0.67 mm | 1.5 × 10⁻⁶ m | 2.5 × 10⁻⁶ m |

The default's error is five times the old solve's in RMS and 28 times at its largest, 0.67 mm in
one solve. The stage stores its elevations as f32, whose step is 0.49 mm from 4 to 8 km: the largest error is about one stored step and the RMS about a hundredth
of one. The planet takes 17 iterations against the synthetic shelf's 7 (the shelf's diffusivity
jumps fifty-fold at the shore); the solve's time in the stage fell by 58 % in the timing pairs
(8 threads) and 42 % in the ensemble's runs (2 threads) (below), less than the iterations, since
the cycle's setup and the first guess are paid every solve.

**The flood (ERO-184).** The serial flood (ERO-080, Barnes, Lehman & Mulla 2014, their improved
variant with a FIFO queue for pits) depends on the order it runs in: a cell in a flat or a pit
takes `next_up` of whichever neighbour the pit queue popped first. The parallel flood computes
the unique least fixed point of the ε relaxation (proof in `pflood/parallel.rs`), which is the
plain Priority-Flood+ε's result in any order. First the ε = 0 fill by Barnes 2016's parallel
Priority-Flood (strips of rows flooded on their own with watershed labels, the strips' edges
joined into a graph of watersheds, a Priority-Flood over the graph for each watershed's spill
level); then the ε fill, exact, by a plain Priority-Flood+ε over each connected group of cells
the first phase cannot settle, the groups in parallel. Barnes 2016 treats ε = 0 only; the second
phase is the project's own. On the 500-Myr `pipeline_quick_16k` run the two floods differ in 168
of 250 steps, in about 170 of the ~13,000 raised cells a step, by ulps. The tests compare the
parallel flood with a plain Priority-Flood+ε bit for bit over several strip heights and thread
counts (`the_parallel_flood_is_the_plain_priority_flood_bit_for_bit`), and check that every land
cell of the filled surface has a strictly lower neighbour (`every_land_cell_drains_on_the_filled_surface`).

**The slope failure's stopping rule (not landed).** The failure (ERO-168, ERO-169) moves 5 % of
each pair's excess over S_c·d per iteration, 40 iterations a step. The asked change was to stop
once what is left is within a physical tolerance of the critical slope. A pair's residual was
taken as the drop its giver's sediment can still remove, min(h_a − h_b − S_c·d, 2·avail_a), in
units of S_c·d (105 m on 2.4-km cells at 2.5°), with a tolerance of 0.1 (0.25° on 2.5°, a tenth of
the spread of measured S_c, ERO-168). A trace of every call (`earthlike_full_16k`, 100 Myr,
54 calls) found that no tolerance from 0.005 to 0.5 ever stops a call early: all ran 40
iterations. The largest residual on the grid, averaged over the calls, fell from 65.8 critical
drops before the first iteration to 8.9 before the last (the largest in any call from 74.9 to
15.4), the 0.95³⁹ = 0.135 of a relaxation limited by its rate; about 200,000 cells moved in every
iteration. The failure is far from the critical slope when the budget runs out, so a stopping rule
cannot save time; the residual carries to the next step. A second form, which also left each pair
alone once its own residual was within the tolerance, cut the cells that moved per pass by a
quarter (251,000 to 188,000) but did not run faster (slope failure 22.8 s against master's 18.8 s
in one 150-Myr timing pair), and its ensemble moved the planet (below). Neither form landed; the
failure is as before. Its cost is set by the 40 iterations at 5 % (ERO-169, a game-choice
"the slope relaxes over a few steps whatever they are"): the same relaxation per step in fewer,
larger iterations, up to the stability limit 1/16 (32 iterations, 0.9375³² = 0.127 against
0.95⁴⁰ = 0.129), would save about a fifth of it, but that changes the relaxation's path and needs
its own ensemble.

**Time.** Two timing pairs (`tools/review/bench-quiet.py`, `earthlike_full_16k`, scattered
continents, 150 Myr, 8 threads each), two runs side by side on the CPU sets A (0, 16, 2, 18, 4,
20, 6, 22) and B (8, 24, 10, 26, 12, 28, 14, 30): master on A and the branch on B, then swapped.
The VM was shared and loaded (other jobs held 19 to 23 CPUs on average, load up to 93), so the
times are slower than the gate's and each pair is one sample. Seconds:

| Timer | Pair 1: master (A) | branch (B) | Pair 2: branch (A) | master (B) | Saved, mean of the pairs |
|---|---|---|---|---|---|
| `diffusion_solve` | 56.4 | 20.2 | 20.8 | 40.3 | 27.9 (58 %) |
| `pflood` | 7.9 | 3.3 | 3.4 | 7.7 | 4.5 (57 %) |
| `slope_failure` (the same code) | 33.6 | 26.3 | 28.3 | 30.1 | (4.6, load) |
| `diffusion` (the whole term) | 90.7 | 47.0 | 49.8 | 71.0 | 32.5 |
| young stage's `diffusion` | 10.5 | 7.5 | 5.8 | 6.9 | 2.1 |
| Surface stage | 194.3 | 126.9 | 136.8 | 166.4 | 48.5 |
| Run | 546.2 | 466.6 | 464.2 | 485.5 | 50.5 (10 %) |

Applied to the full 16k gate run's timers (1,242.8 s: `diffusion_solve` 109.3 s, `pflood`
32.7 s), the pairs' fractions would save about 63 s and 19 s, 82 s of the 343 s the gate needs
(inferred; the gate itself was not rerun). The slope failure's code is unchanged; its 4.6 s is the
load's.

**The planet.** Six runs a binary (`tools/review/force-balance-ensemble.sh`, `earthlike_quick_16k`,
seeds 1 to 3, scattered continents and supercontinent, 500 Myr, `DEEPTIME_GAP_BUDGET=1`;
`tools/review/p1-17-hyps.py`, `tools/review/erosion-recal.py`): master (584b1241, the code of the
branch's base f221b3d3); the branch as landed (multigrid, parallel flood, the old failure); the
branch with the multigrid solve only (`--set surface.numerics.flood=serial`); the branch with the
second stopping form; and the branch with all three settings at their old values
(`--set surface.numerics.diffusion_solver=jacobi --set surface.numerics.flood=serial`, the
stopping form off), which gave master's six worlds figure for figure. Means; brackets the runs'
range.

| | Master | Landed | Multigrid only (serial flood) | With the stopping form |
|---|---|---|---|---|
| Land | 0.191 [0.152, 0.219] | 0.190 [0.143, 0.223] | 0.189 [0.149, 0.222] | 0.161 [0.151, 0.169] |
| Land above 2 km | 0.206 [0.154, 0.239] | 0.177 [0.082, 0.239] | 0.217 [0.145, 0.304] | 0.241 [0.168, 0.282] |
| Highest cell, m | 6,689 [5,655, 7,499] | 6,875 [5,961, 7,575] | 6,683 [6,074, 7,242] | 6,717 |
| Land median height, m | 662 [446, 820] | 588 [396, 804] | 766 [597, 1,033] | 838 [555, 963] |
| Shelf, share of the sea | 0.064 [0.059, 0.070] | 0.061 [0.048, 0.076] | 0.064 [0.036, 0.080] | 0.057 [0.043, 0.087] |
| Continental crust | 0.396 [0.368, 0.418] | 0.423 [0.371, 0.507] | 0.412 [0.378, 0.438] | 0.376 [0.358, 0.426] |
| CO₂, ppm | 1,043 [477, 2,862] | 942 [282, 2,078] | 1,209 [361, 2,431] | 3,237 [335, 14,177] |
| T, °C | 15.7 [13.2, 19.3] | 15.6 [14.0, 19.2] | 16.0 [13.9, 19.1] | 17.7 |
| High ground's flat part, m/Myr | 139.0 | 145.2 | 130.7 | 140.3 |
| All land, m/Myr | 76.8 | 66.6 | 78.8 | |
| Land with relief above 1 / 1.5 km | 0.059 / 0.014 | 0.052 / 0.013 | 0.068 / 0.018 | |
| RMS log error against Montgomery & Brandon | 0.19 | 0.21 | 0.20 | 0.24 |
| Plate checks met (of 45) | 29 | 28 | 27 | 27 |
| Conjugate-gradient iterations a step | 62.2 | 9.5 | 9.6 | 9.5 |

The landed branch's runs stay inside master's range on every surface figure but the continental
crust, whose mean is 0.423 against master's largest 0.418, from one run (supercontinent seed 3,
0.507, which also has the least high ground, 0.082 of its land above 2 km). Its plate checks moved
both ways, every branch mean inside master's runs: three to fail (rift-born plates welded back
9.2 → 14.9 %, at most 10; speed against area −0.36 → −0.42, −0.40 to 0.20; ocean floor older than
180 Myr 8.7 → 10.1 %, at most 10) and two to pass (collision length median 54.5 → 37.3 Myr, 15 to
50; slab ÷ free plate speed 2.29 → 2.57, at least 2.5). Eleven plate figures of
`p1-17-hyps.py` had a branch mean outside master's six runs, all inside their targets where they
have one: the largest plate's median share 0.18 → 0.25 (0.15 to 0.35), steps with one plate above
half the planet 0 → 1.5 % (at most 10), the collision length's 90th percentile 119 → 83 Myr (at
most 120), plate-steps above 150 mm/yr 0 → 0.003 % (at most 0.5; also counted as plate-steps,
0 → 0.17); and, with no target, the largest continental piece 0.16 → 0.24 of the crust, steps
with a supercontinent 28.6 → 49.9 %, a large plate's in-plane stress 2.07 → 0.62 MPa and the
steps it is in tension 100 → 75 %, the balance's scaled speed median 41.3 → 43.9 and 90th
percentile 94.8 → 101 mm/yr.

The planets diverge from master's from the first step that differs (a chaotic system: the flood
alone differs in ulps). The multigrid-only ensemble, whose solve differs from master's by at most
0.68 mm a solve, tests what such a difference does: its surface figures also stay inside master's
runs, it meets 27 plate checks, and eleven plate figures again fall outside master's runs, but
mostly others (plate speed median 33.7 → 38.3 mm/yr, foredeep width 29 → 83 km, collision
length's 90th percentile 119 → 173 Myr, now failing). Only the largest plate's share (0.21) and
the balance's scaled speed median (43.9) moved outside in both. Moves of this size come with any
perturbation, so the landed branch's are read as the spread a different first step makes, not an
effect of the flood (inferred from the two ensembles). The stopping form moved the surface outside
master's runs (land above 2 km, median height, shelf, CO₂ with one run at 14,177 ppm) and failed
two plate checks (rift-born welds 12.7 %; the lowest continental crust 0.36, at least 0.37);
it was not landed.

**The carry's blur and the spreading at long plate steps (ERO-185, ERO-186; P1-19 #27 unlock,
2026-10-19).** The surface rides the plates by one bilinear semi-Lagrangian carry a plate step
(`surface/advect.rs`). A bilinear sample at a fractional offset f is a two-point average of
variance f(1 − f) cells² along each axis, so every carry blurs the carried surface by about a
sixth of a cell² per axis whatever the plates' speed: on the 16k planets a diffusion of 0.25 to
1 m²/yr, beside the hillslope's 2 m²/yr (`kd_land_m2_yr`). A 4-Myr plate step carries half as
often and so blurs half as much per Myr. A step longer than `surface.numerics.carry_blur_ref_myr`
(2 Myr) now spreads, after its carry and before its uplift, the blur that k = step ÷ reference
carries of a k-th of the displacement D would have left: k·v(D/k) − v(D) per axis, v(x) the
variance of x's fraction (at k = 2, 2·min(f, 1 − f)² for f the fraction of D/2, at most half a
cell²), as a [s/2, 1 − s, s/2] stencil over cells of the same crust only (a plate's edge stops it,
as it stops the carry's own sample), in passes of at most half a cell². The regolith's carry and
the climate's ice-sheet and ELA carries take the same top-up. The crust's spreading (`spread.rs`),
whose diffusivity is the cube of the excess height, is solved in equal sub-steps of at most
`surface.numerics.spread_step_myr` (2 Myr), since one solve over a long step holds the diffusivity
at its start value for the whole step. At a 2-Myr plate step neither adds anything and the run is
unchanged. Both are game-choices that keep the numerics of the 2-Myr step per Myr. Their effect on
4-Myr worlds, with what still differs, is measured in plates.md, "The 4-Myr plate step (P1-19 #27
unlock)". Tests: `a_long_step_owes_the_short_steps_blur` (one carry of a long step with the blur it
owes spreads a spike as far as two carries of half the step) and
`the_spreading_steps_at_most_the_setting` (`surface::params`).

**On a GPU (gpu-phase3, 2026-10-19).** Two of these solvers run on a GPU when a player's machine
has one (`docs/design/gpu-compute.md` §15 and §16); no law, setting or ledger value changed. The
diffusion solve's V-cycle (ERO-183) runs on the device in f32 as it ran on the CPU in f32, with
the conjugate gradients, their stopping rule and the mass put-back still in f64 on the CPU, so the
solve meets the same 1e-5 m residual; on the test VM's software GPU it gave the CPU's heights to
the bit in all 50 solves of a 100 Myr history. The slope failure (ERO-168, ERO-169) runs its 40
iterations at 5 % on the device in f32, every iteration over every cell (the CPU stops when an
iteration moves nothing, which gives the same result); matter is kept to the f32 round-off of
the changes, not f64's. On the same states it moved the CPU's metres to 1.5e-8, left heights
within 1.5 mm (RMS 2.5e-5 m at most) and put no cell on the other side of the shoreline. The
stopping rule above was not taken up for the GPU either: a pass costs one dispatch over the grid
whatever moves, and a stop would need a read-back every pass. Over whole 100 Myr histories, worlds
with any ported kernel on the GPU end with slightly more land than the CPU's (+0.007 of the
planet's area on average over 15 seeds), from no single kernel and from none on the same state
(gpu-compute.md §10). CPU runs with changes that do nothing physical (one threshold 1e-10 off,
or four rates 1e-7 off on every cell) end with a little more land too (+0.002 over three such
controls), so part of it is the model's sensitivity to any perturbation; the GPU's worlds sit
+0.005 above those controls, a fifth of the spread between seeds, and no other field leans.

## Carbonate on the sea floor (carbonate and ore-deposit task, 2026-10-19)

**What was wrong.** The review of run 1a156f22b66 found carbonate on 0.009 % of the land and
about 1 % of the column tops. Six quick planets of master b438d47a (`earthlike_quick_16k`,
1,000 Myr, supercontinent and scattered continents, seeds 1 to 3, `strata-check`) agree: the
median planet has carbonate in 0.38 % of the land's top blocks (0 to 2.2 %), 2.0 % of the sea
floor's (chalk 1.8 %) and 0.34 % of all its sedimentary blocks. On Earth carbonate rocks crop out
on 15.2 % of the ice-free land (Goldscheider et al. 2020), a third of the land's sediments are
carbonate (Hartmann and Moosdorf 2012), and calcareous sediment covers 36.8 % of the seabed below
500 m (Diesing 2020; 30 to 45 % of each ocean's floor counting fine calcareous mud, Dutkiewicz et
al. 2015). Three causes, measured:

1. **No pelagic carbonate.** The surface stage's carbonate loop (`surface/mod.rs:1303-1340` on
   master) had only the neritic factory, which needs the photic water, and the climate's cap
   carbonate, laid above 4,000 m only when weathering outruns degassing; otherwise the deep sea
   floor gained only what the marine diffusion and the slope failure brought, recorded as
   clastic. The one deep-sea chalk was the column builder's: the basement's
   sediment pool drawn as chalk on oceanic crust above the CCD (`structure/columns.rs:368`),
   which shows only where no strata log covers it.
2. **The platforms' export was recorded as shale.** Every sea gain of the diffusion solve and the
   slope failure went to the marine clastic part (`surface/mod.rs:1414-1415` on master), so
   carbonate swept off a platform top was laid on its slope as shale or mudstone. On Great Bahama
   Bank the slope wedge holds 40 to 80 % as much sediment as the bank top kept (Wilber, Milliman
   and Halley 1990).
3. **The factory has little room.** Where it works the factory lays about 20 m/Myr averaged over
   the warm and clear photic floor, a long-term platform rate, and the photic floor (0 to 100 m,
   0.38 x 10^6 km2, about 3 % of the planet) is close to Earth's share, but only 0.22 x 10^6 km2
   of it is warm enough and the clarity term (`surface/carbonate.rs:61-62` on master,
   exp(-clastic / clastic_m), ERO-065) takes 64 % of that. Raising its scale did not help: P1-17's
   calibration found five times `carbonate_clastic_m` "gave no more carbonate platform" (build
   log), because the delivered clastics fill the accommodation whatever the factory does. Its
   carbonate is about 1 % of the volume the land delivers to the sea (1.5 against 141 x 10^6 km3
   at the median); Earth's carbonate accumulation, 32 x 10^12 mol/yr (Milliman and Droxler 1995)
   or about 3.2 Gt/yr, is about a seventh of the sediment the rivers bring (some 20 Gt/yr, from
   memory). The planets have no wide epeiric seas: a flooded craton, where most of Earth's
   Palaeozoic platform carbonate formed (Ridgwell and Zeebe 2005: "the mean area of platform
   carbonates during the Mesozoic and Cenozoic is much reduced compared to the Paleozoic"), is
   rare when the continents stand high.

**What changed.**

- **Pelagic chalk** (ERO-191, ERO-192, ORE-270; replaced 2026-10-25 by the chalk of "The pelagic
  rain" below, which keeps the calcifiers' clock and the CCD and drops the fixed rate). Once the planktic calcifiers are in full, the
  open sea floor gains calcareous ooze at 4.0 m/Myr of chalk (from Milliman and Droxler's deep-sea
  accumulation over the calcareous share of the floor, the arithmetic in the ledger row), times
  the share that survives dissolution, all of it down to 1,000 m above the CCD and none at the
  CCD (4,500 m, RCK-015). The calcifiers' clock is 0 before 0.20 Ga and 1 from 0.10 Ga (Ridgwell
  and Zeebe 2005: the Mesozoic proliferation of coccolithophores and planktic foraminifera; before
  it no "substantive deep-sea sedimentary carbonate sink"). A step's deposit that is mostly ooze
  is chalk (ERO-194); chalk is not platform carbonate for the deposit engine (`Kind::Pelagic`).
- **Periplatform carbonate** (ERO-195, ERO-197). A tracer follows the diffusion solve's own change
  (the solved heights less those before the solve, so the slope failure's debris stays clastic)
  through its exchanges: the flux across each face of the five-point operator follows from the
  solved heights, (kd_i + kd_j) / 2 x dt / dx^2 x (h_i - h_j). Taken from the highest sea cell
  down, each cell passes on a well-mixed blend of what came to it and what it lost, its loss
  counted from the top of the column as the substep leaves it (this substep's carbonate and
  chalk, the rest it laid, the step's earlier substeps' deposit, then the column the step began
  with: its regolith and strata log, carbonate where a layer is limestone, dolostone or chalk).
  What a cell kept of its gain takes that blend's carbonate share, out of the marine clastic part
  and into a periplatform part that votes with the factory's carbonate for the class (so a deposit
  mostly of carbonate is limestone, or dolostone where ERO-091 holds). Laid below the photic zone
  where it outweighs the factory's own, it is no platform for the deposit engine
  (`Kind::Periplatform`, ERO-197), and in euxinic water it is black shale (ERO-196). What fails off
  a cell takes its share of the carbonate the diffusion laid there. The carbonate laid is at most
  what the diffusion took off sea cells (`periplatform_source_km3`; the implicit step's exchanges
  conserve it, and the tests check it). The tracer stops where the share falls below 1 % or the
  flux below 1 mm per 2 Myr (a cost bound, not physics, scaled with the substep's length). It is
  always on: a preset switch would have changed the share-code fixture for a tool, and nothing is
  released.
- **Photic diagnostics** (ERO-193). `surface_final` prints the photic floor's area over the run
  and its warm and warm-and-clear parts, with the pelagic and periplatform volumes;
  `tools/review/carbonate-table.py` reads them.
- **Dolostone unchanged.** The facies rule is the sabkha path alone: carbonate laid in arid water
  shallower than 10 m (ERO-091). Warren (2000) lists syndepositional, burial and hydrothermal
  dolomite and finds "up to 50% of the world's carbonate reservoirs are dolomites"; reflux and
  burial need a brine and burial-fluid history the surface stage does not keep, so the rule was
  left as it is and its share is measured below.

**Measured.** Six quick planets each (`tools/review/carbonate-ensemble.sh` on the VM,
`earthlike_quick_16k`, 1,000 Myr, 4 threads, supercontinent and scattered continents, seeds 1 to
3; master b438d47a's binary against the branch at 686a01cf; `tools/review/carbonate-table.py
review/out/carb base branch3`), medians with the range, shares from `strata-check`:

| measure | master | branch | Earth |
|---|---|---|---|
| carbonate in the land's top blocks | 0.38 % [0, 2.2] | 0.71 % [0.18, 2.4] | 15.2 % of the ice-free land |
| carbonate in the sea floor's top blocks | 2.0 % [1.8, 2.9] | 5.9 % [4.6, 6.3] | 36.8 % calcareous below 500 m |
| chalk in the sea floor's top blocks | 1.8 % [0.7, 2.7] | 4.5 % [4.0, 5.6] | |
| carbonate in the sedimentary blocks | 0.34 % [0.27, 0.42] | 0.43 % [0.31, 0.85] | about a third of the land's sediments |
| dolostone's share of the carbonate blocks | 0 [0, 0.19 %] | 0 [0, 0.03 %] | up to half of carbonate reservoirs |
| neritic carbonate laid (10^6 km3) | 1.46 [0.98, 1.90] | 1.52 [1.16, 2.21] | |
| pelagic chalk laid (10^6 km3) | none | 2.36 [2.18, 2.72] | |
| periplatform carbonate laid (10^6 km3) | none | 1.02 [0.88, 1.30] | |
| photic floor, warm, warm and clear (10^6 km2) | | 0.37, 0.21, 0.08 | |

The full 16k planet (`earthlike_full_16k`, scattered continents, seed 1, 8 threads): carbonate in
the land's top blocks 0.28 % on both, in the sea floor's 3.0 % on master and 10.0 % on the branch
(chalk 2.8 % and 9.8 %), in the sedimentary blocks 0.29 % and 0.75 %. The sea floor's share
tripled and the deep floor gained its chalk; the land's moved little, because the factory's room
is unchanged (cause 3) and only the land the sea once covered can show it. Dolostone stays near
zero (the rule is the sabkha path alone, see above).

**After the review (2026-10-20).** An Opus review of the branch raised three points on this page,
each checked against the code before it was fixed; the list above describes the fixed code, the
table above was measured before them. The tracer read the diffusion's and the slope failure's
change together, so the failure's debris took the carbonate share, and every substep read the
column as the step began (A2, A3; ERO-195). A deposit led by chalk or periplatform ooze in
euxinic water was chalk or limestone, so the deposit engine's euxinic-basin and source-rock fields
lost the basin, and the periplatform limestone on a platform's slope counted as platform (A8;
ERO-196, ERO-197).

A8 alone, on the six quick planets (the preset, starts, seeds, threads and content bundle as
above; b2f5c381 against f546d9bc, which differ by A8 alone; `tools/review/carbonate-ensemble.sh`
on the VM, the surface totals in `surface_final` and the strata check; medians with the range):

| measure | before A8 | A8 |
|---|---|---|
| black shale laid over the run (10^6 km3) | 2.68 [1.64, 4.56] | 2.42 [2.04, 2.96] |
| euxinic-basin field, mean area (10^6 km2) | 0.025 [0.010, 0.040] | 0.025 [0.010, 0.032] |
| carbonate-platform field, mean area (10^6 km2) | 1.24 [1.14, 1.31] | 1.20 [1.01, 1.34] |
| source-rock TOC over the area (10^6 km2 x wt %) | 4.17 [4.03, 4.45] | 4.19 [4.00, 4.44] |
| black shale in the sea floor's top blocks | 0.13 % [0.02, 0.24] | 0.07 % [0.01, 0.15] |
| chalk in the sea floor's top blocks | 2.2 % [0.8, 3.5] | 2.0 % [1.6, 2.5] |
| natural MVT sites a planet | 0 [0, 4] | 0 [0, 1] |

Every change is inside the planets' spread. Planet by planet the black shale laid went to 0.51 to
1.54 times what it was, both ways, and the euxinic setting's area with it (0.78 to 1.46 times),
which A8 cannot move: the setting is the 150 km open-water test, the depth and the temperature
(ERO-088), read from the heights and the climate. A8 changes a deposit's class, the class changes
the strata log, and from there the two builds' planets part (their sea-level events and plate
counters differ by the end). What A8 acts on is small, the euxinic field's area being 2 % of the
platform field's; its rules are checked by two unit tests in `surface/facies.rs` (euxinic water
makes black shale of chalk and periplatform ooze; periplatform limestone below the photic zone is
no platform), not by the planets.

Two later changes moved the sea floor's numbers, and neither is separated from the other. Between
686a01cf and b2f5c381 the branch took master 5dc06974 and the tracer fixes: on the six planets the
neritic factory now lays 0.77 x 10^6 km3 [0.70, 1.07] (1.52 above), the tracer 0.45 [0.39, 0.59]
(1.02), the pelagic rain 2.09 [1.94, 2.21] (2.36), and the sea floor's top blocks hold 2.4 %
carbonate [1.1, 3.6], 2.2 % of it chalk (5.9 and 4.5 % above). The ocean-crust merge (master
c24ff793, merged at 45673a37) then changed what the world draws on top: on the same seeds the sea
floor's top blocks hold 0.30 % carbonate (scattered continents 1, 869e65e7) and 0.38 %
(supercontinent 2, 14e4ae7d) against 2.4 and 2.2 % on f546d9bc, chalk 0.23 and 0.34 % against 2.4
and 2.1 %, while the stage lays as much chalk (2.18 and 2.05 x 10^6 km3 against 2.09 and 1.99)
and the columns hold as much (0.03 % of supercontinent 2's sea-floor blocks on both). That merge
draws the rock rising with the amplified relief instead of stretching the top unit over it
(strata.md, STR-052, STR-055 to STR-057): the sea floor's loose mud fell from 36 % of its top
blocks to 0.3 % and shale rose from 46 to 83 %, so the chalk lies under a shale top (inferred from
the census, not traced column by column; open).

**The chalk that P1-07 saw.** P1-07 counted 33 % of the sea floor as chalk; the review read about
2 %. They are two metrics. P1-07's is the `strata/outcrop` view layer of `deeptime stage
structure --preset pipeline_quick_16k --start scattered_continents --length-myr 500 --threads 8`
(500 Myr, scattered continents, seed 1), counted over the sea floor; the review's is the strata
check's drawn top block, which reads 5 to 10 times lower on the planets measured here. On P1-07's
own metric, each commit built from its own tree with its own class list, the sea floor's chalk
went:

| commit | what it is | chalk on the sea floor |
|---|---|---|
| 2fd6f154 | P1-07 merge | 47.4 % |
| 6d8171f0 | before the P1-05 climate merge | 47.4 % |
| f70e20b5 | P1-05 climate merge | 36.1 % |
| ae27fb71 | before the P1-17 calibration | 36.1 % |
| a3c63fed | P1-17 partial: calibration with the real climate | 32.4 % |
| ba8019af | | 32.4 % |
| efafc88c | | 30.7 % |
| 8600e031 | P1-17 hypsometry merge | 20.6 % |
| 3e4c1967 | | 26.5 % |
| 1eba8e09 | continental margins and spires | 20.3 % |
| b438d47a | master | 21.9 % |
| this branch | | 25.1 % |

One run of one seed moves by about 6 points with nothing relevant changed (3e4c1967 rose from
20.6 to 26.5 %; P1-07's own run read 33 % where its merge commit reads 47.4 % here), so two steps
stand out. The first is f70e20b5, the P1-05 climate merge (47.4 to 36.1 %), which gave the
preset its climate section (`pipeline_quick_16k.json:168` at f70e20b5) in place of the zonal
stand-in: the land grew (88,053 to 106,788 cells) and the sea floor's shale, mudstone and mud
rose from 46.7 to 56.5 % as its chalk fell, so more of the floor was covered by a strata log, and
the column builder's pool chalk shows only where none covers it (inferred from the census). The
second is 8600e031, the P1-17 hypsometry merge (30.7 to 20.6 %), which gave the ocean floor an
asymptote of -5,600 m with damping 0.12 mm/yr after GDH1
(`pipeline_quick_16k.json:71-72` at 8600e031, read in place of the -10,000 m trench at
`plates/init.rs:328, 359`). By the age-depth law (`plates/pools.rs` `ocean_elevation`) the floor
then reaches the CCD, below which the column builder draws no pool chalk
(`structure/columns.rs:368`, 4,500 m), at an age of about 48 Myr instead of 62 (inferred from the
law, not measured on the planets). a3c63fed, which moved the ridge from -1,000 to -2,500 m, moved
the chalk only from 36.1 to 32.4 %, inside the noise. The hypsometry merge is a calibration toward
Earth's depths (a median sea depth of 4.0 km against Earth's 4.06, the parameter's note) and the
climate is the real one, so both were kept: the chalk they removed was the basement pool's
stand-in, and the deep floor's chalk now comes from the surface stage's own pelagic rule above.

**The carbon cycle.** The climate's carbon budget is degassing against silicate weathering, and
its weathering supply is blind to rock type: the surface stage's erosion x 2,700 kg/m3 x
1.26 mol/kg (`climate/carbon.rs:54`, CLM-153, CLM-154). Carbonate enters it in one place: the
weathering's drawdown beyond the degassing is laid as cap carbonate over the shallow sea floor
(`climate/mod.rs:1730-1741`). The neritic, pelagic and periplatform carbonate the surface stage
lays is a record; it takes no carbon from the ocean, and limestone weathers like any rock. So the
carbonate changes reach CO2 only through the relief and the land area they change. Measured on
the same planets: the natural CO2 (the end CO2 over the present-day factor) has a median of
795 ppm on master [388, 2,434] and 820 ppm on the branch [414, 1,949], the end temperature
17.95 and 17.55 °C. The planets move both ways (supercontinent seed 1 760 to 414 ppm, scattered
seed 1 850 to 1,949 ppm), as a changed first step moves them, and the full 16k planet went from
1,461 to 739 ppm (19.3 to 16.6 °C). No shift stands out of the planets' spread, as the coupling
predicts.

**Limits.** The factory's room (no epeiric seas on high-standing continents) is the larger part of
the land's shortfall and is a plate and sea-level matter, outside this task. The sea floor's top
is still mostly shale and mud (57 % and 26 % on the branch's supercontinent seed 1, chalk 4 %):
where the marine diffusion's clastics reach, a step's deposit is mostly clastic (ERO-194), and the
diffusion spreads them over the deep floor, which on Earth gets mostly pelagic sediment beyond the
fans (inferred from the shares, not measured cell by cell). Carbonate burial is
not tied to the carbon budget (the ocean does not lose the carbon the surface stage buries). The
periplatform tracer follows the diffusion solve's change; the slope failure's runout, which moves
material further than one face, carries carbonate only as the share of what fails off a cell.

## Chemical denudation (M5 lane, 2026-10-25)

**Why.** The surface stage took rock off the land only mechanically: rivers, hillslopes, slope
failure. The rock the silicate weathering dissolved stayed in place, although the carbon cycle
counted it as weathered (CLM-152 to CLM-157). On base13 the low-relief land (window relief under
100 m, 57 % of the land's window-time) eroded at 5.6 m/Myr net, against Montgomery & Brandon's
eq. 2 at 22.5, whose intercept E0 of 10 m/Myr is their mean chemical denudation of the largest
rivers (ERO-004's note; build log 2026-10-25, M5 lane). The term is approved as a missing sourced
process, not as a lever on any V1 number (the coordinator's call, 2026-10-25).

**The law (ERO-205).** The dissolved load is the mass the carbon cycle already weathers, and no
new rate:

- the clock adds up each EBM band's land weathering over the climate step, mol of Ca + Mg, at the
  implicit carbon step's own CO2 (so the sum is the carbon the reservoirs lost beyond the
  degassing);
- at the step's end, each band's mol goes on its land cells that are not under ice, by each
  cell's own supply-limited flux S (1 - e^(-K/S)) (West, Galy & Bickle 2005's two regimes: the
  supply limits slow-eroding ground, the kinetics fast-eroding ground);
- a cell loses dissolved_kg_per_mol (ERO-206) of rock per mol, over rho_rock and its true area;
- the surface stage takes that thickness off the ground before the step's net change: the
  regolith first, then the strata log, with the isostatic response of any denudation;
- the erosion field the surface hands back to the climate stays mechanical. Counting the
  dissolved part in it would raise the next step's supply, and so the dissolved load, on itself.

**The constant (ERO-206).** 0.26 kg of rock per mol of Ca + Mg: the upper crust's Ca + Mg + Na +
K weight fraction (0.1, West et al.'s figure) times the cratons' dissolved silicate load over its
cations, (SCDR + SiO2) / SCDR, whose median over West et al.'s five cratons is 3.31 (1.72 to
5.24), over the rock's 1.26 mol of Ca + Mg per kg (CLM-153). Dixon & von Blanckenburg 2012's
"<= 50%" mass loss of fully weathered granite gives 0.40, the top of the cratons' span.

**Against measured cratons (ERO-207).** West et al.'s Table 2 gives each craton's silicate
chemical denudation, (SCDR + SiO2) / 2.7 t/m3, with its temperature, runoff and total
denudation. The model, run on one cell per craton at 280 ppm with the default parameters:

| Craton | T (degC) | runoff (mm/yr) | total denudation (m/Myr) | West, chemical (m/Myr) | model, eroding at the total (m/Myr) | model, flat (m/Myr) |
|---|---|---|---|---|---|---|
| Slave | 4.0 | 100 | 1.2 | 0.16 | 0.32 | 0.32 |
| Siberia | 2.0 | 270 | 5.6 | 0.88 | 0.75 | 0.62 |
| Africa | 24.1 | 250 | 3.7 | 1.39 | 2.19 | 1.29 |
| Grenville | 4.5 | 580 | 4.8 | 1.40 | 1.61 | 1.07 |
| Guyana | 25.0 | 1,680 | 13.0 | 4.66 | 8.43 | 1.31 |

Each is within a factor of 2.5 of its craton (the unit test's bound), and flat ground that does
not erode (the supply's least denudation of 2 m/Myr, CLM-155) sinks at 0.3 to 1.3 m/Myr, inside
the cratons' 0.16 to 4.66. The task's "2 to 10 m/Myr" for a craton (CLM-155's comment) is total
denudation, not its chemical part: West et al.'s cratons denude 1.2 to 13 m/Myr in all, and
neither paper states 2 to 10 m/Myr. A flat craton thus sinks by about 0.3 to 1.3 km per Gyr from
the dissolved load alone, less its Airy rebound (about 0.14 of the crust lost, build log
2026-10-25, M5 lane).

**Tests.** The weathering by band adds up to the carbon cycle's flux to the bit; the placed load
is the band's weathered mass to 1e-12, on land only and not under ice while the band has free
land; the clock's dissolved mol equals the carbon the reservoirs lost beyond the degassing to
1e-9, also when a stretch runs in two parts; the switch off leaves the clock unchanged to the
bit and writes no field; the surface stage's mass residual stays under 1e-6 of its erosion with
the dissolved load in it.

**Limits.**

- The supply's x2 (CLM-156) was set when the grid carried 9.5 Gt/yr; on base13 it carries about
  23 Gt/yr. With it, a fully supply-limited cell can lose rock in solution at up to 0.66 of its
  erosion, above the largest depletion fraction Riebe, Kirchner & Finkel 2004 measured (0.61).
- (Closed 2026-10-25 by "The pelagic rain" below: the dissolved load comes back to the floor as
  opal and carbonate, mol for mol. The text that follows is the limit as it stood.) The dissolved
  load leaves the planet's mass budget. On Earth it returns to the sea floor as
  carbonate and silica; here the carbon cycle lays only its drawdown beyond the degassing as cap
  carbonate, and the surface stage's carbonate rules are not tied to the carbon budget (above).
  Checked for a sink to send it to (the coordinator's question, 2026-10-25): none carries it.
  The cap carbonate (`climate/mod.rs`, CAP_CARBONATE_GLOBAL) is the carbon the reservoirs lost
  beyond the degassing, so at the balance (weathering equal to degassing) it lays nothing, while
  the weathering's own carbonate (Urey's reaction) is not laid at all; the neritic, pelagic and
  periplatform rules are production laws (light, warmth, clarity, a fixed pelagic rain over a
  fixed CCD) that take nothing from the ocean. Routing the dissolved mass into either would lay
  the cations' carbonate a second time beside the production laws, which stand for Earth's
  measured accumulation, so it is not done. On `m5-chemden` supercontinent seed 1 the last step
  dissolved 9.65e18 mol of Ca + Mg (4.8e12 mol/yr, 1.26 Gt/yr of rock, 0.91 m/Myr spread over
  the planet, 2.85 m/Myr over its land), while the surface stage laid carbonate, chalk and
  evaporite at 0.46 m/Myr spread over the planet over the run (3.56e6 km3 on the bake's
  7.74e6 km2) and the last step's cap carbonate was 0.02 m over the floor above the CCD. By
  ERO-206's median, about 70 % of the dissolved mass is silica and 30 % cations. A
  mass-conserving return is a supply-limited ocean (the cations' input buried as carbonate, the
  pelagic share taking the rest by the CCD's depth, the silica as biogenic silica): a sourced
  mechanism, but a change to the surface stage's carbonate rules, not a redirect, and not built.
- Carbonate rock's own dissolution is not modelled: the carbon cycle weathers silicate only.
- At Earth's balance the land weathers about 5.3e12 mol/yr (degassing 6.65e12 less the seafloor's
  0.2), so the dissolved silicate load is about 1.4 Gt/yr. Earth's rivers carry about 0.37 Gt/yr
  of dissolved silica (6.2 Tmol/yr, Tréguer et al. 2021) plus the silicate cations; a secondary
  source puts the silicate Ca + Mg at 1.6 to 2.8e12 mol/yr (a 2023 GMD preprint citing Gaillardet
  et al. 1999, whose primary was not read), half the model's. Not checked further.

**Measured.** Pool arm `m5-chemden` (`earthlike_quick_16k`, 1000 Myr, both starts, seeds 1 to 4,
the switch on) against base14 (24 runs, the switch off), medians with the two-sided Mann-Whitney p:

| measure | base14 | chemical denudation | p |
|---|---|---|---|
| land | 0.319 | 0.310 | 0.14 |
| largest landmass | 0.205 | 0.228 | 0.81 |
| continental crust under the sea | 0.202 | 0.193 | 0.69 |
| land above 2 km | 0.085 | 0.095 | 0.81 |
| coast with high ground | 0.201 | 0.224 | 0.22 |
| land under ice | 0.099 | 0.075 | 0.27 |
| collisions per 100 Myr | 1.70 | 1.60 | 0.68 |
| continental crust's net change | -0.043 | -0.053 | 0.094 |
| criteria met per run (median, mean) | 5, 5.42 | 5, 5.12 | 0.21 |
| sea over the datum after 100 Myr / at the end, m | 277 / 409 | 235 / 278 | |
| the surface guard's worst, m from the sea | 10,109 | 9,887 | 0.24 |

No measure moved at p < 0.05, every run reached the end, and the guard stayed where base14 has
it, so the term is on by default since 2026-10-25 (the rule for built physics). The off switch
is bit-identical to base14 (a one-run arm, `m5-chemden-off`, supercontinent seed 1: the same
state hash). The sea's lower stand at the end (278 m against 409, untested) goes with the
dissolved load leaving the mass budget (Limits) and the slightly larger loss of continental
crust.

## The pelagic rain (pelagic lane, 2026-10-25)

**Why.** The sediment diagnosis (build log 2026-10-25, sed-excess-diag, merged 05db3452) found
the sea's sediment 3.0 times GlobSed's solid volume, the whole excess on oceanic floor, and about
a tenth of that excess (in log shares) in the plates' pelagic rain: an unsourced 5 m/Myr on every
oceanic cell, also laid on the initial floor as 5 m/Myr times its age (PLT-103). The surface stage
laid a fixed 4 m/Myr of chalk besides (ERO-191), and the dissolved load the land's weathering took
away left the planet's mass budget ("Chemical denudation", Limits). The coordinator's call
(decisions log 2026-10-26): remove PLT-103 and lay one sourced pelagic rain in its place, from
aeolian dust, the dissolved load returned as opal and carbonate, and chalk where a sourced CCD lets
it survive.

**The rain** (`surface/pelagic.rs`). Three parts, each from a measured flux and none fitted to a
sediment thickness, laid once per plate step, after its last substep, on the floor as the substeps leave it:

- **Aeolian dust** (ERO-208 to ERO-210). Earth's arid land sends 450 Tg of mineral dust a year to
  the oceans (Jickells et al. 2005, Fig. 2). A planet sends that flux per km2 of Earth's 25.5
  million km2 of arid and hyper-arid land (aridity index below 0.2: Safriel et al. 2005, Table
  22.1, after Middleton & Thomas 1997) times its own: the land cells whose climate aridity (P/PET,
  BIO-038) is below 0.2. The grains are silt's 2,661 kg/m3 (Kominz et al. 2011). The dust is laid
  evenly on the sea, and exactly the volume laid is taken evenly off the arid cells at the step's
  end, so the move conserves material. Jickells gives no rate by latitude; the basins' shares he
  gives (North Atlantic 43 %, Indian 25 %, ...) follow Earth's deserts and winds, no law a planet
  can use, so the dust falls evenly. At Earth's arid area it is 0.17 km3/yr, 0.47 m/Myr over the
  ocean.
- **Biogenic opal** (ERO-211 to ERO-213). The dissolved load's silica, 0.70 of its mass (West et
  al. 2005, Table 2: the Grenville craton, the one ERO-206 takes), comes back as opal (SiO2 with
  10 % water, 2,000 kg/m3) the way the modern ocean buries silica, in steady state (Tréguer et al.
  2021: inputs 14.8, outputs 15.6 Tmol Si/yr). The floor deeper than 1 km takes the open ocean's
  5.5 of the 15.6 (Hayes et al. 2021's burial below 1 km); the shallower floor takes the rest (the
  coastal and continental-margin zone's 3.7, reverse weathering 4.7 and sponges 1.7, laid as one
  siliceous part).
- **Carbonate** (ERO-214, ERO-191, ERO-192). Each mol of the dissolved Ca + Mg comes back as a mol
  of calcite, less the share the climate's drawdown already laid as cap carbonate
  (`climate/cap_carbonate_mol`). The planktic calcifiers' share (the deposit engine's clock,
  ORE-270, at the step's middle: 0 before 0.20 Ga, 1 from 0.10 Ga) rains as chalk on the floor
  above the carbonate compensation depth, each cell's share in proportion to what survives
  dissolution at its depth: all of it down to the lysocline 1,000 m above the CCD, none at the CCD
  (Ridgwell & Zeebe 2005, sec. 1.1). What dissolves below the lysocline goes back into the sea and
  is laid where carbonate survives, so the whole supply is laid. The rest (before the calcifiers,
  all of it) is laid as the cap carbonate is, evenly on the floor 0 to 4,000 m deep (ERO-067).

A step's deposit that is mostly opal is chert, mostly dust is shale, and both are black shale in
euxinic water (ERO-216, a game-choice by the facies scheme's majority rule).

**The CCD** (RCK-015, ERO-192). 4,500 m, the column builder's value, now sourced: "At present,
this depth is approximately 4,500 m" (Coxall et al. 2005, abstract); 4.6 km in the equatorial
Pacific (Pälike et al. 2012); a median of 4,439 m over Sulpis et al. 2018's 17 basins (3,895 to
5,926 m). It is fixed. Pälike et al. find it at 3.0 to 3.5 km about 55 Ma and more than 1 km
shallower before the Eocene-Oligocene transition, "consistent with an overall Cenozoic increase in
weathering". A CCD that moves with age or chemistry needs the ocean's alkalinity budget (the
carbonate the ocean gains against what it buries), which the model does not keep, so it is not
built.

**The fill.** A cell takes no more than the water above it. Each part in turn (the chalk, the
shelf carbonate, the margin opal, the deep opal, the dust) is laid as s times its weight on the
cell (the surviving share for the chalk; 1 on the part's floor and 0 elsewhere for the rest),
capped by the room the cell has left, with the s that lays the part whole. Newton's method finds s
from below: the laid volume is concave in s, so each step stays at or under the root. What a
part's floor cannot hold at all goes to the other floor of its kind (the chalk's to the shelf
carbonate, the margin opal's to the deep opal), or is reported unlaid (`rain_unlaid_km3` in the
surface's totals). The sums run in fixed chunks of 4,096 cells, so the thread count does not
change them. The pile is solid (no pore water), as every pool and layer of the model.

**At Earth's balance.** The land weathers 5.32 x 10^12 mol of Ca + Mg a year (the degassing less
the sea floor's share, CLM-158), so the dissolved load is 1.38 Gt/yr (0.51 km3/yr of rock). It
returns 16.1 Tmol Si/yr as 0.54 km3/yr of opal and 5.32 Tmol/yr as 0.20 km3/yr of calcite, and
the dust adds 0.17 km3/yr. On the unit test's synthetic ocean (below), the deep floor gets 0.57 to
0.65 m/Myr of opal, the margin floor 5 to 11 m/Myr, and the floor above the lysocline 1.7 to
2.5 m/Myr of chalk at the calcifiers' full share (each a range by where the test puts the ocean
floor the age grid leaves out). Two findings, noted in the ledger and not acted on:

- The model's silica is about twice the rivers': Tréguer et al. give the rivers 8.1 Tmol Si/yr
  (dissolved 6.2 plus amorphous 1.9) and all inputs to the ocean 14.8, against the model's 16.1.
  ERO-206's mass per mol sets it, through the Grenville craton's SiO2 / SCDR of 2.3 (ERO-211).
- The returned carbonate is under half of Hayes et al.'s 11.3 Tmol/yr of CaCO3 buried below 1 km.
  The rest of Earth's deep-sea carbonate is carbonate rock's weathering recycled, which the model
  does not track (ERO-214; "Chemical denudation", Limits).

**Tests.**

- *Against GlobSed* (ERO-215). The rain on young floor away from the margins against GlobSed's eq.
  2a (Straume et al. 2019: Z = sqrt(tau) (52 - 2.46 |phi| + 0.045 phi^2) m in place, fitted on
  floor younger than 82 Myr, below 72 deg and more than 200 km from the margins, average STD
  140 m). A synthetic Earth at its weathering balance, with Jickells's dust and the calcifiers in
  full; Earth's floor by age (Seton et al. 2020, in 10-Myr bins) at GDH1's depth (PLT-352); the
  rest of the ocean (361.9 x 10^6 km2, Eakins & Sharman 2010) as margin floor, 32.8 x 10^6 km2 of
  it shallower than 500 m (Diesing 2020) at 200 m and the ungridded floor deeper than 500 m at
  either bound of its depth (500 or 5,000 m). The pile a floor gathers at the depths it passes
  through is decompacted with Kominz et al. 2011's average ocean-floor sediment (phi0 0.66,
  lambda 1,333 m) and set against eq. 2a's area-weighted mean over |phi| < 72 deg. No constant in
  the rain was set from GlobSed.

  | Floor age | The rain, in place (m) | Eq. 2a (m) |
  |---|---|---|
  | 10 Myr | 76 to 94 | 113 |
  | 30 Myr | 158 to 182 | 196 |
  | 80 Myr | 284 to 298 | 320 |

  The rain lies 14 to 42 m under eq. 2a, inside its average STD of 140 m (the test's bound).
  Eq. 2a less its pore water (`open_floor_solid_m`) is also where the margin rule PLT-661 (off by
  default) draws its line between floor with the rain alone and floor holding the continents' rock,
  in place of PLT-103's 5 m/Myr times the age (plates.md).
- *Mass, unit.* On 5,000 cells of random depth, 500 of them 1 to 50 mm deep, the dust, the
  silica and the calcite come back whole: each part's laid volume equals its supply to 1e-9, the
  part arrays add up to the laid total to 1e-6, and no cell takes more than its water. Full cells
  pass the rest to the cells with room; the chalk follows the surviving share (none below the
  CCD) and goes to the shelf carbonate when no floor lets it survive.
- *Mass, coupled* (the surface stage's chemical denudation test, the pipeline at 3,000 cells for
  30 Myr). The dissolved silica comes back as opal to 1e-5, and the unlaid rain is under 1e-6 of
  the supply: 129,819 km3 dissolved, 49,383 km3 of carbonate and 135,845 km3 of opal returned,
  308 km3 of dust, -4.2e-10 km3 unlaid. With chemical denudation off, no opal and no returned
  carbonate.

**Limits.**

- The dust falls evenly, not downwind of the deserts, and glacial dust (2 to 20 times today's,
  Jickells) comes only through the arid land's area.
- The model's aridity is Thornthwaite's P/PET; that the atlas behind Earth's arid area used the
  same PET is from memory (ERO-209).
- All the dissolved cations come back as calcite, also the Mg that Earth removes as dolomite and
  by hydrothermal exchange (Shalev et al. 2019).
- Carbonate rock's weathering is not tracked, so the returned carbonate is under half of Earth's
  deep-sea burial (above).
- The CCD is fixed at its present depth (above).
- The margin opal stands for reverse weathering and sponges too, laid as opal.
- The silica is about twice the rivers' (above).
- The chalk and the opal come only from the dissolved load: with chemical denudation off
  (`climate.chemical_denudation`, on by default since 2026-10-25), only the dust rains.
- The initial floor, new floor at a spreading gap and resampled floor start bare and gather the
  rain from then on. GlobSed's eq. 2a could give the initial floor a starting pile by its age; not
  built.
- The rain falls once per plate step, after the last substep, on the floor the substeps leave. So
  the marine diffusion first moves it in the next step, and the periplatform tracer (ERO-195)
  never counts the opal or the dust. The step's end is where its arrays fit the memory budget:
  laid in the first substep, they would be held through all the substeps, beside the tracer's.

**Measured** (2026-10-25). The arm pel-rain (key 4c55ab83c7a1: master 1c2946f2 with this lane, merged
at ef274941) against base17-s4 (base17's seeds 1 to 4 on 1c2946f2, key 8aaddf52958a):
earthlike_quick_16k, 1,000 Myr, both starts, 8 runs a side, argkey cd665cef, the crust budget's maps.
Medians and the two-sided Mann-Whitney p, from `tools/review/seddiag/sedab.py` on the present map
(sedstock.py's stock and sedflow.py's flows run by run; the base's pelagic allowance 5 m/Myr, the arm's 0).
GlobSed is Straume et al. 2019 in place, as in the sediment diagnosis (plates.md's loss budget).

| | base17-s4 | pel-rain | p | GlobSed |
|---|---|---|---|---|
| the sea's sediment, solid | 1.05e9 km3 | 7.19e8 km3 | 0.0002 | 3.37e8 km3 |
| on oceanic crust, solid (in place, Kominz) | 9.00e8 (1.07e9) | 6.45e8 (7.71e8) | 0.0002 | 1.89e8 |
| that over GlobSed, solid (in place) | 4.76 (5.65) | 3.41 (4.08) | 0.0002 | 1 |
| ocean floor within 200 km of continental crust | 5.13e8 km3, 5,834 m | 3.71e8 km3, 4,639 m | 0.0003 | 7.6e7 km3, 2,049 m |
| ocean floor beyond 200 km: mean pile, solid (in place) | 1,786 m (2,342) | 1,138 m (1,491) | 0.0002 | 404 m |
| young floor (eq. 2a's domain), in place over eq. 2a | 4.64 | 3.06 | 0.0003 | 1 |
| wet continental crust (GlobSed's margins) | 1.49e8 km3, 6.86e7 km2, 2,073 m | 8.86e7 km3, 4.76e7 km2, 1,796 m | 0.05 | 1.43e8 km3, 4.69e7 km2, 3,044 m |

- *By share.* The arm takes 0.21 of the ocean floor's log excess over GlobSed away (ln 1.561 to
  1.228; 0.19 in place) and 0.33 of the whole sea's (0.28 in place). The diagnosis gave PLT-103
  about a tenth of the floor's excess (x1.2, from its allowance of 5 m/Myr times the age); the
  arm removes x1.39, 2.55e8 km3 against the base's 1.74e8 km3 allowance. The rest is the plates'
  rain's own fate downstream: it was accreted onto oceanic tops at resamplings (6.58 to 5.19 km3 a
  year, p 0.0002), buried into metamorphic rock (on wet oceanic crust 2.03e8 to 1.30e8 km3, p
  0.0002) and subducted (sediment subduction 1.17 to 0.87 km3 a year, p 0.0002) like the rest.
- *By floor age* (mean pile, 0-20, 20-60, 60-120, 120-180, 180-300, over 300 Myr): 156, 1,052,
  2,314, 4,145, 6,401, 9,242 m to 128, 774, 1,823, 3,078, 5,100, 8,105 m (p 0.33, 0.03, 0.003,
  0.0006, 0.02, 0.0006).
- *Flows* (km3 a year over the run). The plates' rain 1.39 to 0. The surface stage's net
  material onto oceanic samples 4.74 to 4.81 (p 0.72); over all samples 0.43 to 1.03 (p 0.0002),
  since the dissolved load is laid again instead of leaving. Carbonate 0.26 to 0.39, chalk 0.092
  to 0.026 (both p 0.0002: the old fixed 4 m/Myr chalk is gone and the chalk now survives only
  above the CCD). The dissolved load 1.05 to 1.20 Gt a year (p 0.1).
- *What is left.* The ocean floor still holds 3.4 times GlobSed (4.1 in place). Floor in eq.
  2a's domain holds 3.1 times eq. 2a in place, where the rain alone lies 14 to 42 m under eq. 2a
  (the unit test above), so what is left there came from the land. The floor within 200 km
  of continental crust holds 4.9 times GlobSed's. The diagnosis's other shares (the land's supply
  about 0.4, the floor's age about a quarter, the trenches keeping what reaches them about a fifth)
  are not touched by this lane. The margins now hold less than GlobSed's: their wet area is
  GlobSed's (the change, -2.1e7 km2, has p 0.28 and goes with the sea 160 m lower, 579 to 419 m
  over the datum, p 0.007), and their mean pile fell 277 m (p 0.015) to 1,796 m of rock against
  GlobSed's 3,044 m in place.
- *Census and guard.* v1census moves no measure at p < 0.05 but the continental crust's net change
  over the run, -0.049 to -0.064 (p 0.015; the oceanic share 0.626 to 0.642). That change is
  spread over the crust budget's collapse (-0.010, p 0.2), rate (-0.009, p 0.1), founder (-0.003,
  p 0.07) and collision (-0.002, p 0.03) parts; no one route carries it. Land 0.236 to 0.254 (p
  0.38), the largest landmass 0.067 to 0.122 (p 0.13), continental crust under the sea 0.428 to
  0.334 (p 0.19), coast with high ground 0.103 to 0.096 (p 0.96), criteria met 6 to 6.5 of 7 (p
  0.47). The guard is clean: worst 10,046 to 9,962 m from the sea (p 0.72), no step beyond two
  Earth reliefs, the plates' largest excess over Airy 2,408 to 2,591 m (p 0.10).

## The mouth fill's order (mouth-fill lane, 2026-10-28)

**What was wrong.** The mouth fill (`marine::fill`, ERO-050) was written in P1-04 to raise the sea
cells round a river mouth to the clinoform level "shallowest first", so that a delta and its shelf
build out from the mouth. Its heap item was ordered the way the Priority-Flood's is (`pflood.rs`,
the lowest bed popped first), so the fill took the deepest cell it had reached first. A mouth's
load ran down the slope and filled the deepest room within reach, up to a level only 0.5 m per km
below the sea, before it touched the shallow room beside the mouth. Found on the way, in the
speed work of 2026-10-28 (docs/design/p1-17-benchmark-gate.md, "Not speed: the mouth fill's
order").

**What the sources do.** No published scheme read fills the deepest room first:
- goSPL (v2021.08.03, read): the marine cells are taken highest bed first
  (`np.argsort(smthZ)[::-1]`), each filled to its room below the clinoform height, the rest
  passed to its downslope receivers (`distocean`). The current goSPL routes the load downstream
  the same way, cell by cell, filling each to its room before it passes the rest on.
- FastScape's marine scheme (fastscapelib-fortran `Marine.f90`, read; Yuan et al. 2019, EPSL,
  abstract only): the shelf surface is set from the shore down the flow paths at a slope of 1e-4,
  and the river flux less each cell's room is passed downstream, donors first.
- Badlands' `marine_distribution` (read by the research pass, not here) starts at the mouth and
  passes what a cell cannot take to its lowest neighbour.
- Clinoforms: Pirmez, Pratson & Steckler (1998, abstract) find the greatest accumulation on the
  upper foreset and little on the topset and bottomset; the deep bottomset takes least.

Each sends the load down from the mouth and fills the room nearest the mouth before it sends the
rest on; a closed hollow is filled from its floor (goSPL's pits, to their spill level). The
spreading along the shelf comes from the diffusion after the fill in all of them, as here.

**What the code does now.** The load goes down from the mouth, nearest room first, and climbs
over a rise only when there is no room left below it. Each cell the fill reaches is keyed by the
highest bed on the fill's way to it (its spill, as in the Priority-Flood), lowest first; among
equal spills, which is all the ground the load reaches going down, the nearest first, then the
deeper (`Item::cmp`). So a delta builds out from its mouth, the room beside the mouth before the
deep floor beyond it, and a hollow the mouth lies in fills from its floor. A plain "shallowest
first" was tried first and dropped: from a mouth at the floor of a hollow it climbs the hollow's
side along whichever way the grid's rounding favours, and on the cube the area it covered then
depended on how the planet was turned (`stream_power_and_mouth_fill_are_the_same_turned`); the
sources' loads only go down. The level (ERO-050), the fill depth (ERO-051) and the shelf slope
(ERO-052) are unchanged. Our shelf slope, 5e-4, is five times FastScape's 1e-4; that is a separate
question and is left as it is. The tests `a_mouth_fills_the_near_room_before_the_deep_floor_beyond`
and `a_mouth_in_a_hollow_fills_it_from_its_floor` pin the order.

**On the planet.** Arm `mouth28` (8 runs, earthlike quick 16k, 1000 Myr, both starts) against
master's base28 (`pool.sh table --base base28 --arm mouth28`, medians): land 0.307 to 0.328
(Mann-Whitney p 0.033), the largest landmass 0.146 to 0.200 (p 0.33), continental crust under the
sea 0.198 to 0.140 (p 0.094), coast with high ground 0.167 to 0.143 (p 0.85); every V1 target is
met in both. The load that stays on the shelf beside the mouth instead of running to the deep floor
builds land, which is the sources' order and needs no knob. The cost is in the walk: at 300 Myr the
fills raise 6.25 against 2.84 million cells and pop 32.0 against 11.2 million (a topset aggrading
thinly over more cells is most of it), at about 84 ns a pop either way; on the full 16k gate run
marine_fill takes 19.2 against 17.3 s (408 million cells popped, the most one fill popped 20,632 of
its 200,000 cap). Planning the mouths' fills in parallel and applying them in turn (bit for bit,
each plan checked against the cells it popped) re-planned 41 % of them and ran slower, so the fill
stays serial; its heap item is packed into one u128 key (16 bytes, compared as one number;
`the_packed_item_orders_as_its_fields`).


## Internally drained basins (the plateau lane, 2026-10-31)

**What was asked.** Why Tibet-class plateaus wear away (flat high ground at 109 to 129 m/Myr though arid or polar by its
Köppen class, against the Altiplano's 0.02 mm/yr), which terms wear a plateau's flat top, and a sourced law for what
Earth's plateaus do instead (the orogen lane's O4, docs/design/collision-orogen.md; the follow-up "High plateaus still
wear too fast"). The measurements, the sources and the arm are in docs/design/plateau-drainage.md; this is the summary.

**What wears them** (cached runs only: base34, 24 runs, and orog-cum, 8; `tools/review/plateau/`). On the flat high
ground (above 2 km, 3 x 3 relief under 500 m at the step's start) the surface's gross lowering is stream power without
the critical slope's cut 0.37, the cut 0.32 to 0.34, diffusion 0.21 to 0.24, ice 0.06 to 0.07 (net 134 to 138 m/Myr).
The cut on ground that was flat at the step's start follows the receiver's lowering in the same solve: it is the
threshold hillslope behind an incising river, so stream power through the plateau's drainage, with the cut it drives, is
0.69 to 0.71 of the wear. Almost nothing the flat high ground loses is laid back on it. At the runs' end the
frozen-ground runoff of `surface/climate.rs` (all of P below 0 °C) reaches only 0.004 to 0.029 of the flat high
ground, but that is one frame: over the whole run 0.67 of it is below 0 °C (the arm, below). A third to a half of the
flat high ground drains into a
depression at a run's end, and every depression spills (ERO-080). After its collision a plateau's crust goes 0.57 to
0.69 to the plates' collapse and flow (PLT-254 to PLT-259, PLT-300, PLT-412 to PLT-415, sourced) and 0.31 to 0.43 to
the surface, read from the rock pools of the crust-budget maps. The plateaus' own wear is 1 to 2 % of the land's mean
lowering (40.1 and 37.1 m/Myr); the land under 2 km is 0.72 to 0.75 of it.

**What Earth's do.** Their interiors are internally drained and barely erode (southern Tibet 6 to 16 m/Myr, Hetzel et
al. 2011, p. 983; 8.1 and 11.8 m/Myr, Strobl et al. 2012; the Altiplano-Puna's internally drained plateau 0.02 mm/yr
against 0.50 on its wet eastern flank, Bookhagen & Strecker 2012, p. 103). A closed lake's level is its water balance
(Langbein 1961, p. 4: "I + P A_L = E' A_L"; Garcia-Castellanos 2007, p. 387, eq. 12), a lake passes on its runoff less
its evaporation (Garcia-Castellanos 2006, eq. 2: "Qw(x = 0) = P . lb - E . ll", and "for a closed lake Qw(x = 0) = 0"),
and closure keeps the orogen's rock ("its effect in terms of mass removal from the orogen is null", Garcia-Castellanos
2007, p. 380). The basin fills and its base level rises (Sobel et al. 2003, [60]); it opens when it fills to its outlet,
the climate wets, or a river captures it (Garcia-Castellanos 2006, pp. 284-285; Sobel et al. 2003, [58]).

**The law (ERO-217, ERO-218; `surface.closed_basins`, on by default since 2026-10-31, decisions log).** The flexure's lake balance (ISO-126, the bake's budget,
HYD-050 and HYD-051) now also sets what each lake passes on: the stream power's drainage area leaves a lake's exit at
the share of its inflow its balance lets out, and none from a lake that cannot fill to its spill. The depression stays
the solve's lake (HYD-110), so what a closed basin's ranges shed stays on its crust up to the lake's room; a basin
its sediment fills to the spill opens and passes its water ("the sediment overfill of the lake ... can trigger drainage
opening", Garcia-Castellanos 2006, pp. 284-285; that source alone), the step solved again until no lake opens (opening
one sends its water and sediment to the lake below it), and a lake cell is an exit only where its water leaves the lake
for good. The young landscape (L2) takes the same law. The flat high ground is also split
by its water in the erosion bands (`flatc`, `flatd`, `flatf`; ERO-220).

**The controlled plateau (ERO-219).** ERO-176's rimmed plateau on the presets' law, a wet outer flank: the arid interior
(runoff weight 0.05) wears 3.8 m/Myr spilling and 0.5 closed, the edge 121.2 and 111.7; a humid interior 129.9 spilling
and 129.8 with its balance (its lake losing 20 mm a year, passing 0.69 of its inflow). The arid interior's own runoff already slows it far below the planets' flat high ground, so
on the planets its rivers carry water from upstream and over the spills of the depressions; the law takes that water
away where the balance closes them.

**Left as it is.** The plates' collapse and flow; the frozen ground's runoff (ERO-010); the 5 % floor of the runoff
weight (ERO-011, which K was calibrated with); the lakes' evaporation at the bake's 200 + 60 T+ mm a year (HYD-040,
HYD-041), about half Tibet's 650 to 1,300 (Wang et al. 2020) and Titicaca's 1,100 to 1,270; ranges above the plateau
(the orogen's edge peaks).

**Measured on the pool** (the arm plat-cb0 against orog-cum, 8 and 8 runs; the design's "The arm"):
- **Internally drained land.** 0.13 of the land over a run (0.09 to 0.15) drains to basins that pass nothing, against
  Earth's 20 % (Garcia-Castellanos et al. 2003, p. 8-1).
- **K's calibration holds.** The land's erosion by relief and by region is unchanged (regions of 500 m relief 220
  against 218 m/Myr).
- **The guard holds.** No divergence: the worst cell stood 10.5 km from the sea against 9.9.
- **The plateaus are not kept.** The flat high ground wears at 141 against 134 m/Myr. The part of it draining to a closed
  basin (0.30) wears at 53 m/Myr, half by diffusion. That is 3 to 5 times southern Tibet's catchment rates (Hetzel et
  al. 2011). The part below 0 °C, 0.67 of the flat high ground over the run, wears at 141 m/Myr.
- **The next lead is frozen ground's runoff.** The surface still runs off all precipitation below 0 °C (ERO-010). The
  bake's budget has not done so since 2026-10-01: rain share HYD-036, degree-day melt HYD-037, biotemperature PET
  HYD-029, HYD-030 and HYD-035.

## Cold lands' runoff (the runoff lane, 2026-10-31)

**What was asked.** The plateau lane's arm found the flat high ground's wear on frozen ground. Over the run, 0.67 of it
is below 0 °C on the year and it wears at 141 m/Myr. There the surface's runoff is all of the precipitation (PET 60
mm/°C x the mean temperature, 0 below 0 °C, MOI-067). The bake's water budget stopped doing that on 2026-10-01. The
question: give the stream power the bake's budget, and test it against cold basins. The measurements and the arm are in
docs/design/surface-runoff-budget.md; this is the summary.

**What cold lands do.**
- **Permafrost basins** run off part of their precipitation. The Mackenzie: P 410 mm, ET 237, runoff 169 (MAGS Water
  and Energy Budget Synthesis, Table 2.1; Woo & Thorne 2003, Table 2). The Lena's runoff ratio is "about 0.50", the
  Ob's "around 0.30" (Yang et al. 2002, p. 14-3).
- **A polar desert** loses its snow: in the Dry Valleys 7 to 11 mm of snowfall, "most of which is lost to sublimation"
  (Fountain et al.).
- **The Tibetan Plateau's lakes** run off 0.26 to 0.31 of P at a MAAT of 0 to -3 °C (Biskop et al. 2016, Table 4,
  modelled).
- **Sublimation** takes 10 to 90 % of winter snowfall in the Alps (Strasser et al. 2008) and 20 to 32 % in the Rockies
  (MacDonald et al. 2010).

**The law (ERO-221; `surface.runoff_budget`, on by default since 2026-10-31).** The stream power's runoff weight (ERO-010) takes each cell's runoff
from `HydroParams::budget`, the same budget the lakes' balance uses. Precipitation is rain in the thawed share of the
year (HYD-036) and the snow melts by its degree-days (HYD-037: 4 mm per °C per day, a choice inside Hock 2003's 2.5 to
5.5 for snow at non-glaciated sites). PET is Holdridge's 58.93, here 60, x the biotemperature (HYD-029, HYD-030), and
Fu's curve acts on the liquid water (HYD-034, HYD-035). The months are the climate stage's twelve months, which it
writes every step (between snapshots each row's month plus the month's departure carried with the plates, MOI-052),
and the year's thaw is Holdridge's monthly method without his 30 °C cut on them, one law through the history, the
young stage and the bake; where the months are missing, a year of two half-sines that keeps the mean (HYD-030,
HYD-031; since the re-reviews of 2026-10-31: the sinusoid centred between the two months dropped the mean on a
lopsided year, and the history took the twelve months at its last step only). With no month below freezing it is the
old runoff to 1e-12, on lopsided years too. Land with a mean above freezing and a frosty month runs off less (its
biotemperature is never under its mean: P 600 mm at 5 ± 15 °C, 336 against 245 mm), and so does land below cold
headwaters. Ground that never thaws runs off nothing, so its weight sits at ERO-011's floor, which is then
load-bearing (25 mm a year where the lakes count 0). The young landscape and the amplifier take the same weight
(YNG-011).

**The checks (ERO-222).**
- **The Mackenzie:** budget runoff 204 mm and ET 206, against 169 and 237 measured: within the 54 mm mean absolute
  error of Fu's curve (Zhang et al. 2004). The old rule gives 410 and 0.
- **The Dry Valleys:** nothing runs off when the warmest month stays below freezing. But a warmest month just above
  0 °C melts all the snow and evaporates little, so 0.24 to 0.85 of P runs off: the budget has no sublimation.
- **The Tibetan lakes:** the budget evaporates 69 to 141 mm against their modelled 170 to 290. It still runs off 0.46
  to 0.83 of P, against all of it under the old rule. The shortfall is printed, not asserted.
- **A polar year's months as given:** a mean of -17 °C with months of -5 and -40 °C runs off nothing. Centred on the
  mean, the same range put the warmest month at +0.5 °C and ran some of it off (corrected 2026-10-31; this changes the
  closed basins' and the flexure's lakes on cold ground too).
- **Verkhoyansk's biotemperature:** 3.05 °C by its twelve months, Holdridge's monthly method and his value; 2.80 by
  the two half-sines from its mean, warmest and coldest month.
- **A lopsided year** (P 600, mean 4, warmest month 10 °C): with a coldest month at or above 0 °C, the mean itself and
  the old runoff (380.7 mm); at a coldest month of -6 °C, 5.08 °C and 332.9 mm. The sinusoid between the two months gave 4.00 and 5.00 °C either side of a
  coldest month of 0 °C, and the sinusoid about the mean 4.35 °C (364.5 mm) whatever the coldest month.
- **The cold controlled plateau:** the interior wears 88.0 against 118.0 m/Myr; never thawing, 3.8.

**Left as it is.**
- Sublimation of the cold season's snow (its shares vary 10 to 90 % with no law read).
- The plateau's higher evaporation (Holdridge's PET has no radiation term).
- Land ice (HYD-038).
- Holdridge's 30 °C cut (HYD-030).
- The aridity index and the facies (they keep the snapshot's PET).
- K and ERO-011's floor.

Measured on the pool (the arm plat-rb0, 8 runs, on the closed basins; the design's "The arm's results"): the flat high
ground wears 92 against 141 m/Myr and the land above 4 km doubles (0.0022 against 0.0010 of the land, Earth 0.0195);
K's regions wear 11 % slower at 250 to 500 m of relief, 19 and 16 % at 500 and 1,000 m and more, 8 % at 1,500 m and
more, inside Montgomery & Brandon's lines at 500 m and more and above both under it, as on the base; the export to the sea does not move (3.34 against 3.36 km³ a year). The
verdict then was default-on. The runoff law's review (2026-10-31) held it: the young stage and the amplifier kept the
old rule (the pool's runs never reach them), the months were centred on the mean, and no test went through the switch.
All three are fixed. Armed again on base35-s4 (rb1, 8 runs): the guard the base's, the flat high ground 110 against
146.5 m/Myr (the months' correction alone, rb-months), the land above 2 km a sixth more over the run, K's regions 8 to
18 % slower (inside Montgomery & Brandon's lines at 500 m of relief and more, above both under it, as on the base), the criteria a median of 6 against 5. The months' correction alone
moves no column. On by default since 2026-10-31 (the coordinator's call, decisions log), the months' correction with it.
Armed again on the head after its third re-review (rb3 and rb3-off on base35-s4, 8 runs each, 2026-11-01; the design's
"The arms on the fixed head"): the corrections alone move no column (p 0.29 or more); with the law the guard is the
base's, the flat high ground wears 106 against 139 m/Myr, K's regions 4 to 14 % slower (inside Montgomery & Brandon's
lines at 500 m of relief and more, above both under it, as on the base), no V1 column moves at p under 0.16 and the
criteria are a median of 5.5 against 5. The default held (decisions log 2026-11-01).
