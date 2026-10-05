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
- Stage 1 spot checks: the soil-production depth scale of 5 m is 10 to 20 times the 0.25 to 0.5 m that secondary sources give for Heimsath's law (ERO-070, MISMATCH at secondary level); the hillslope diffusivity of 2 m2/yr is 20 times the only primary value read (ERO-023); the Landlab carbonate and marine constants match Landlab's source.

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

**Left as it is.** The rates are the stream-power and isostasy physics as calibrated before (ERO-001 to ERO-021,
ISO rows); with the bookkeeping fixed they sit inside Earth's ranges per setting, and the decisions log of
2026-10-05 rules out tuning them towards a depth or hypsometry. A parcel's total over hundreds of Myr is large
because it spends part of that time in orogens; how much a stable craton is exhumed over 1 Gyr on Earth (a few km in
the thermochronology literature, from memory, not checked) is the next reference line to source.
