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

Rows by status: `cited-unverified` 86, `from-memory` 2, `abstract-only` 3, `calibrated` 6, `game-choice` 6, `uncited` 64.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/ERO.md](extraction-notes/ERO.md).
- Research note: `docs/research/notes/surface_processes_climate.md`.
- The implementation follows Braun & Willett 2013 for the implicit stream-power solve; the research note read that paper's abstract only.
- Work package in the verification plan: WP-3 ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- The erodibility `k_spl` is 2e-9 in the Rust default and 3e-9 in all eight presets; the comment describes the older value (extraction notes ERO, flag 1).
- The weathering constants (50 m/Myr, depth scale 5 m) have no source; the research note says the rate constants "were not retrieved" (flag 2).
- The facies rules are, in the research note's words, "my proposal, not a published scheme that was checked".
- Stage 1 spot checks: the soil-production depth scale of 5 m is 10 to 20 times the 0.25 to 0.5 m that secondary sources give for Heimsath's law (ERO-070, MISMATCH at secondary level); the hillslope diffusivity of 2 m2/yr is 20 times the only primary value read (ERO-023; Roering et al. 1999, read on 2026-10-16, measured 0.0036 m2/yr on hillslopes, 550 times smaller, ERO-177; see "Plateaus and the hillslope diffusivity"); the Landlab carbonate and marine constants match Landlab's source.

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
RIM_ROWS

Without diffusion the two widths give the same numbers to the decimal (stream power is width-invariant); diffusion
is about 50 times stronger on the map's escarpment (13.3 against 0.2 m/Myr) and lowers the interior's stream power
from 305 to 118 m/Myr, because it rounds the escarpment's lip and the knickpoint behind it slows. The critical
slope cuts nothing in any case, the 1-cell escarpment included: the escarpment's rivers carry enough area that
stream power keeps it below 0.35 (on the planet the cut works on peaks and ridges, where the area is small). Only
less runoff slows the plateau.

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

Above 2 km stream power is 75 to 80 % of the gross lowering and diffusion 21 to 33 %. A quarter of the high ground
is flat, and it wears at 63 m/Myr (37 of it stream power, 26 diffusion): six times southern Tibet's plateau surface
(under 10 m/Myr, ERO-174), while the steep three quarters wear at 199. The critical-slope cut grows with height, from
9.6 m/Myr in the 2-km band to 110 of the 123 m/Myr of stream power above 4 km: there it is the process that wears
the ranges, the one law here that is not the same at every drawn width.

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
| Porphyry sites (counted / target) | 22 (14 / 25.0, capped) | 27 (13 / 14.5) | |

With the measured κ diffusion all but stops at 4.8-km cells (a cell's relief relaxes in L² / κ, about 6 Gyr), and
stream power takes over the slopes diffusion rounded: on flat high ground stream power rises from 37 to 45 m/Myr, so
the plateau's net falls only from 63 to 46, still about five times Tibet's interior. Diffusion was not what wears the
plateaus: in the controlled case it protects them, rounding the escarpment's lip so the knickpoint behind it is
slower.

**The cause.** A plateau drained to the sea wears at stream-power rates whatever κ is. The deep-time stage's
priority flood fills every closed basin from the sea and routes it over its rim (ERO-080; only land no sea cell
can reach drains to a sink of its own, ERO-083), so the rivers that cut
the escarpment carry the whole plateau's drainage area, and with n = 2 a knickpoint's celerity C = 2 K A S is large
for large A: on the map the escarpment's knickpoint crosses the controlled plateau within one 2-Myr step, and it does
so at Earth's width too (the law is the same at both widths). On Earth the interior of the Tibetan plateau drains to
closed lakes (from memory, not checked here), so the escarpment's rivers drain the escarpment only. The controlled
cases above show the two levers: a terminal lake at the plateau's centre and a dry plateau. The stage-1 hydrology
already balances terminal lakes (`hydro/network.rs`: a basin whose evaporation takes its inflow does not spill), but
only in the bake's water stage, not in the deep-time surface. On steep ground above 3 km the critical slope does the
work, and S_c = 0.35 (ERO-013) has no source at a 4.8-km cell.

**Left as it is, and why.** κ stays 2 m²/yr and S_c 0.35. The length scale is the decided one and consistent, so
there is no scaling error to fix; the measured κ does not fix the plateaus (46 against under 10 m/Myr) and moves the
world's shape away from two calibrated targets (land median 284 m and 6.6 % above 2 km, against the 430 to 640 m and
0.10 to 0.13 the 3e-9 erodibility, ERO-004, was calibrated for with κ = 2). Changing κ alone would break that joint
calibration, and the decisions log of 2026-10-05 rules out tuning K back towards the hypsometry. What would make
plateaus wear like plateaus is internally drained basins in the deep-time stage, a change to the stage's water
routing; that, the map-scale S_c and the κ are questions for Ben (STATUS.md, build log 2026-10-16).
