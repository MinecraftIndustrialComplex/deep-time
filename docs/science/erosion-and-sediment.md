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
with it.

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
sweeps. The cycle runs in f32 and the conjugate gradients in f64; its reductions are unordered,
so the result depends on the thread count in the last bits.

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
