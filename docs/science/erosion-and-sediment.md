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

