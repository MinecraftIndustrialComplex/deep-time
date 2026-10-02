# Extraction notes: plates

Written by the stage-1 extraction agent for this subsystem (2026-10-01/02) while it built the ledger. It holds the measured Earth-against-mod numbers found in the build log, the simplifications and game choices the code or documents state, and flags: admissions of unchecked values, contradictions between comments, code and documents, and things that looked wrong. **Nothing here is verified.** A line marked "suspicion" or "my arithmetic" is the extraction agent's own reading, to be checked in stage 2. See [claims.md](../claims.md).
Abbreviations as in the ledger. "Cortial" = Cortial, Peytavie, Galin & Guerin 2019. Dates are those of the build-log entries. All mod figures are from runs the entries describe; nothing here was re-run.

## Earth against the mod

### Plate-check baseline (the plate model's own targets), 2026-10-01

Run: `earthlike_quick_16k`, supercontinent start, 1000 Myr, 2 Myr steps, seeds 1-4, `deeptime stage volcanoes`, master fa09e8b plus the recording of task 1, 2 threads on the test VM; command `tools/remote/mac.sh -- tools/remote/detach.sh start pc tools/review/plate-check.sh 1000 1,2,3,4 supercontinent base` (docs/design/plate-check-baseline.md; build log 2026-10-01 "deeptime plate-check"). Result: 3 of 19 ranged targets met. Earth figures and their stated sources are the collisions design's (section 2.2 and 3).

| Quantity | Earth (stated source) | Mod, mean of seeds 1-4 [range] | Target |
|---|---|---|---|
| Largest plate, median share | 0.20 today (Bird 2003; Wikipedia list of plates) | 0.58 [0.50-0.66] | 0.15-0.35 |
| Steps with one plate above 50 % | "none today" (no source) | 58.7 % [49.6-68.8] | at most 10 % |
| Steps with one plate above 30 % | no Earth figure | 83.0 % | reported |
| Largest plate, worst step | no Earth figure | 0.90 [0.84-0.93] | reported |
| Plates above 5 % of the planet, median | 7 (Bird 2003; the run review says the 5 % reading is from memory) | 3.75 | 5-9 |
| Plates alive, median | 52 in Bird 2003; about 7 large | 12.5 | reported |
| Median plate lifetime (plates that died) | large plates 100-200 Myr or more; "No published statistic was found" | 45 Myr [38-56] | at least 80 |
| Plates dead within 20 Myr | no Earth figure | 33.4 % | at most 15 |
| Rift-born plates alive after 50 Myr | an ocean opens for 100-200 Myr (Wilson 1966; Wikipedia Iapetus) | 18.7 % [14.0-21.6] | at least 60 |
| Rift-born plates welded back within 50 Myr | none | 41.9 % | at most 10 |
| Most land raised by one collision in one step | none | 4.47 % of planet | at most 0.5 |
| Most land gained in any step after 10 Myr | none | 3.75 % | at most 1.5 |
| Collision belt width, median | Himalaya 150-350 km, Alps 250 km, Tibet 1,000 km (Wikipedia pages) | 2,783 km | at most 800 |
| Collision belt length, median | Himalaya 2,400 km, Alps 1,200, Urals 2,500 | 7,062 km | reported |
| Collision belt elongation, median | young belts 5-15 (Wikipedia pages) | 2.19 | at least 4 |
| Collision length, median | India-Asia 50+ Myr and continuing, Alleghanian 65, Uralian 70-90 (Wikipedia) | 2 Myr (one step) | 15-50 |
| Peak uplift spread (90th / 10th) | "varies enormously" | 1.00 (every collision 1,400 m) | at least 2 |
| Collisions per 100 Myr | "about 3-4 (P1-27's figure)", no source | 10.0 [9.2-11.6] | 2-6 |
| Continental crust, lowest / highest step | 0.41 (Cogley 1984); "about 40 % (from memory)" in the run review | 0.40 / 0.48 | at least 0.37 / at most 0.45 |
| Continental crust at the end | same | 0.42 [0.38-0.45] | reported |
| Active share of the continental margin | 0.2-0.3 (0.48 at most, counting island arcs; Bradley 2008 and others) | 0.30 [0.28-0.32] | 0.25-0.40 |
| Ocean floor 90th percentile age, end | oldest large areas 180-200 Myr (run review) | 235 Myr [208-269] | at most 230 |
| Ocean floor 90th percentile age, mean after the first 100 Myr | same | 200 Myr | reported |
| Slab-attached over other plate speed | 3-4 (Forsyth & Uyeda 1975, abstract) | 1.41 | reported |
| Plate speed, median over plates and steps | RMS about 40 (run review, no source), fastest group about 85, slowest 26-28 (Zahirovic et al. 2015) | 72.6 mm/yr | reported |
| Speed: mostly oceanic / mostly continental plates | about 85 / 26-28 (Zahirovic et al. 2015) | 70.6 / 47.1 mm/yr | reported |
| Speed correlation with slab share | the Forsyth & Uyeda result (no number) | 0.48 | reported |
| Real shortening at a collision, median (seed 1) | no Earth figure | 87.4 km against a counter reading 350 km | reported |
| Contact age at the collision, median / 90th | none | 7.5 / 25.1 Myr | reported |
| Mega-plate regrowth after a rift | none | 17 Myr [14-22] | reported |

Variants the collisions design ran on seed 1 (master e8668ca, same preset, 1000 Myr): no welding (`collision_suture_frac=1e9`): one plate above half in 19 % of steps, plate lifetime 180 Myr, active margins 0.67, land 0.46, continental crust 0.52; threshold 1,500 km: above half in 4 % of steps, rift-born lifetime 85 Myr (68 % survive 50 Myr), active margins 0.37, continental crust falls from 0.44 to 0.29 and land to 0.16; `collision_distance_km=500`: at most 3.2 % of the planet lifted per collision, elongation 3.2, world ends with land 0.15 and median land height 288 m.

### Run review of workbench run 1a0f969d358 (2026-10-01, `earthlike_quick_16k`, supercontinent, 1000 Myr, seed 1, master about 044c112)

From `docs/research/notes/run-review-1a0f969d358.md`. Earth figures are the note's own, several marked "from memory".

| Quantity | Earth (as stated) | Mod |
|---|---|---|
| Plate speed, new plate's own, median / 90th / max | RMS about 40; fastest about 85 | 65 / 95 / 100 (cap) mm/yr |
| Convergence at new subduction zones, median | full rates 20-150, limit about 200 | 52 mm/yr |
| Spreading at new ridges | East Pacific Rise about 150 | 52 mm/yr |
| Convergence at collisions | India-Eurasia 50 now, up to 190 at 55 Ma | 51 mm/yr |
| Hotspot track speed (35 tracks, median 4,750 km in 78 Myr) | no Earth figure | 63 mm/yr |
| Largest plate | Pacific 20 %; seven plates above 5 % ("5 % reading is from memory") | above 30 % in 80 % of frames, above 50 % in 51 %, worst 98 %; median 3 plates above 5 % |
| Plate turnover | large plates last hundreds of Myr; "no sourced figure for Earth's turnover rate" | 165 born and 157 died in 1000 Myr; median lifetime 32 Myr; 39 % die within 20 Myr |
| Supercontinent cycle period | "quoted from about 400 to 800 Myr" | about 650 Myr |
| Rift-born plates | ocean opens 100-200 Myr or more; the Atlantic about 180 Myr | 65 born, 59 died all by welding; median lifetime 18 Myr; 56 % welded back within 20 Myr, 83 % within 50 Myr |
| Continental crust share | about 40 % (from memory) | 42.3 % start, 39.6 % end, 37.3-45.4 % between |
| Land fraction | 29 % | 33 % start, 21-39 % between, 24 % end |
| Continental crust under water | about 27 % (derived: 29 % land on 40 % crust) | 14-25 % early, 44 % at the end, peak 52 % |
| Sea floor median age | mean 64 Myr | 68 Myr |
| Sea floor 90th percentile / share over 180 Myr / over 250 Myr | oldest large areas 180-200 Myr / close to none / one small basin about 340 Myr | 189 Myr / 12 % (up to 22 %) / 3 % (up to 10 %); `ocean_younger_250` 0.936 |
| Median land height | "Earth's mean is about 800 m (from memory)" | 610 m |
| Land above 2 km | "about the same (from memory)" | 11 % |
| Highest point | Everest 8.8 km | 6.4 km at the end |
| Collisions | Himalaya about 2,400 km long and a few hundred km wide | 107 in 1000 Myr, all 2,000 m, lifting up to 15 % of the planet in one step |

### P1-27 margins (build log 2026-09-30), `tools/review/p1-27-ensemble.sh 400 "1 2 3" "supercontinent scattered_continents"` (6 runs of 400 Myr, coupled pipeline) and plates-only runs (4 x 750 Myr)

| Quantity | Earth (stated source) | Mod before / after |
|---|---|---|
| Passive share of continental margin | about 0.7 (passive 105,000 km against 53,000 km convergent: Bradley 2008) | 0.30 / 0.63 [0.59-0.66] |
| Active share | about 0.2-0.3, 0.48 at most (Bradley 2008; Lallemand; Schellart et al. 2008 "tallied"; Mercier de Lepinay et al. 2016) | 0.67 [0.59-0.74] / 0.31 [0.29-0.35]; plates only 0.61-0.70 before, 0.29-0.36 after |
| Floor beside passive margins, median age | modern passive margins 104 Myr (oldest 180); ancient 181 Myr (Bradley 2008) | 50 / 102 Myr; plates only 64-72 before, 103-141 after |
| Ocean 90th percentile age | none stated | 108 / 194 Myr [162-228]; younger than 250 Myr 0.996 / 0.955 |
| Plates at the end | none stated | 17.3 / 10.5 |
| Collisions per run | none stated | 43 / 40 |
| Subduction initiations | none stated | 1.2 / 25.5 |

### P1-17 hypsometry recalibration (build log 2026-10-01), `tools/review/p1-17-hyps.sh`, 500 Myr, `earthlike_quick_16k`, `deeptime stage volcanoes`, VM, 10 runs (seeds 1-5 x both starts), mean before -> after

Earth reference: GMT earth_relief 30 arc-minute (SRTM15+, Tozer et al. 2019), cos(latitude)-weighted; Charette & Smith 2010 for the sea mean.

| Quantity | Earth | Mod before -> after |
|---|---|---|
| Land share | 0.289 | 0.414 -> 0.293 |
| Median land height | 427 m | 1,324 -> 640 m |
| 90th percentile of land | 2,022 m | 3,860 -> 2,241 m |
| Land above 2 km | 0.102 | 0.329 -> 0.127 |
| Median sea depth | 4,062 m (sea mean 3,683 m) | 2,910 -> 3,394 m |
| Continental crust | 0.41 | 0.401 -> 0.419 (dry share 0.86 -> 0.67; Earth 0.71) |
| Share of planet below -6 km | 0.003 | 0.061 -> 0.020 (for the GDH1 floor step alone) |
| Active margin share | about 0.3 | 0.33 -> 0.33 |
| Ground under continental arc cones | 1,200-4,000 m (no source) | 794 -> 703 m |
| Mean temperature | 14-15 degC | 5.7 -> 11.0 |
Still off (the entry's own list): too much shallow sea (0.155 of the planet between -1 and 0 km, Earth 0.076); ground under arc cones about 0.7 km, not an Andean 2-4 km; the full preset (`earthlike_full_16k` scattered seed 1: land 0.604 -> 0.378, median 2,043 -> 1,013 m).
Also measured: 1,000 Myr supercontinent seed 1: land 0.249, median 430 m, 90th percentile 1,967 m, above 2 km 0.097 (no upward drift after 500 Myr); `earthlike_quick_32k` seed 1 both starts: land 0.358 -> 0.323, median 1,181 -> 696 m.

### The height limit, stage 2 (build log 2026-10-01), 500 Myr, `earthlike_quick_16k`, seeds 1-3 x both starts

| Quantity | Earth | Mod |
|---|---|---|
| Plateau level where crust spreads | Tibet near 5 km | 5.1 km at 1 g with `crust_strength_mpa` 140; 13.4 km at 0.38 g; 2.55 km at 2 g |
| Land share / median / above 2 km (presets now, n = 6) | 0.289 / 427 m / 0.102 | 0.273 [0.234-0.310] / 547 m [436-701] / 0.114 [0.085-0.152] |
| 90th percentile of land | 2,022 m | 2,102 m [1,789-2,392] |
| Highest L1 cell | Everest 8.8 km (run review) | 5.7-6.8 km; finished world 6.0-6.9 km |
| Full preset after uplift x 0.8 (n = 4) | same | land 0.292, median 630 m, above 2 km 0.106 |
| Plate elevation cap binding | 0 targeted | 0 in every run |

### Cost model against Cortial's Table 2 (build log 2026-09-28 P1-03; the test machine, release, 500 Myr)

Cortial's estimate: 100k samples 76-106 s, 250k samples 330-437 s. Mod: 100k 8.3 s on 12 threads (23 s on 1); 250k 25-31 s on 12 threads, 72 s on 1.

### P1-03 statistics against the plan's targets (build log 2026-09-28; `deeptime stage plates --preset plates_quick_16k --start scattered_continents --length-myr 500`, seed 1, 100k samples)

Target (plan section 8.1): oceanic crust mostly younger than about 250 Myr, a few large plates plus small ones. Mod: ocean_age_p90 202.3 Myr, p50 69.7 Myr, 92.5 percent younger than 250 Myr, 13 plates alive, 8 above 5 percent of the planet, largest plate 0.179; `plates_full_16k` (250k): p90 122.0 Myr, 99.9 percent younger than 250 Myr, 16 plates. 20k-sample statistics test: supercontinent p90 167 Myr, 98 percent younger than 250 Myr, 15 plates (8 at 5 percent or more). Sweep (3 starts x 2 seeds, 100k): p90 141-228 Myr, 92.5-99.4 percent younger than 250 Myr, ocean fraction 0.73-0.76, 10-22 plates. Resolution dependence: ocean fraction 0.74 at 100k, 0.69 at 250k, 0.82-0.85 at 20k. No Earth figure is given for these.

### Force-balance open loop (build log 2026-10-02; four 1000 Myr runs, supercontinent and scattered starts, seeds 1 and 2, extended trace patch; `tools/review/plate-forces.py`)

| Quantity | Earth (stated source) | Present model | Balance variants |
|---|---|---|---|
| Plate speed 10th / 50th / 90th percentile, mm/yr | slowest group 26-28, median 40, fastest 85 (Zahirovic et al. 2015) | 41 / 71 / 83 | drag only 0-5 / 60-72 / 165-209; with slab and boundary resistance 27-31 / 62-68 / 95-109 |
| Oceanic / continental median speed | about 85 / 26-28 | 73 / 42-49 (ratio 1.6) | with both resistances 66-72 / 27-33 |
| Speed against slab share (rank correlation) | the Forsyth & Uyeda result | 0.47-0.59 | drag only 0.76-0.84; with both 0.69-0.77 |
| Change of rotation per step, median / 90th pct | none | 5 % / 9 % | 16-18 % / 62-70 % |
| Plate-steps above 150 mm/yr | rare, short | 0 (capped at 100) | 1.3-3.1 % with both resistances |
The scattered start has the same faults as the supercontinent start: one plate above half the planet in 52-69 % of steps over the four runs.

## Simplifications and game choices

From module docs, params docs, the P1-03 log and the designs. Reason given where one is.

- **Earth-normalised scaling** (decisions file "Tectonics scaling"): the plates run as if on a 6,370 km sphere and results are mapped onto the 782 km (16k) or 1,565 km (32k) planet by latitude and longitude, so plate count, speeds and ages are Earth's; elevations and uplift rates keep physical values. Reason: without it Earth's large-plate size (about 40 million km2, Mallard et al. 2016) would exceed the whole 32k planet (30.8 million km2) (plan section 8.1).
- **Plates are rigid; there is no slab dip, depth or length, no trench roll-back and no transform-fault physics.** The front is the overriding plate's own lattice boundary, fixed in its frame (P1-03 log design choice 2); subducted crust is hidden until the next resampling. The force-balance design plans slab length (section 3.3) but it is not built.
- **Plate motion is rules, not forces** (force-balance design section 2): a random Euler pole, a speed relaxing to 45 mm/yr or to the slab-pull speed, a 0.6 slow-continent factor, a random walk, a cap. The design lists six hand rules to remove. A plate with no slab moves at 45 mm/yr "for no reason". P1-03 log item 6: without the slab-pull speed and the initiation rule the oceans never recycled (p90 age 327 Myr).
- **Collision is a discrete event** with a capped uplift surge in one step (design: "a hand cap doing the work"); the cap is 1,400 m (quick) or 1,100 m (full) in the shipped presets; Cortial's uncapped product is tens of km.
- **A collision counter adds relative speed times dt**, not shortening across the boundary; the collisions design calls this the cause of quick re-welding.
- **Welding of whole plates on a collision** (P1-27): added to keep passive margins passive; it builds a mega-plate (collisions design section 2.3).
- **Spontaneous subduction initiation, polarity reversal, locked sutures, the arc band, the cordillera, the arc-maturity rule, the shelf share, the elevation ceiling, the isostatic ceiling, hotspots as mantle-fixed points, pelagic sediment, rock pools** are all additions beyond Cortial (module docs; P1-03 and P1-27 logs), each introduced to fix a measured fault.
- **Elevation in the plate model is the surface** (Cortial's z), so continental erosion removes 5.5 times the surface lowering from the rock pools (Airy factor, pools.rs).
- **The plate model's elevation cap is a numerical guard** (30 km in pipeline presets) and the uplift ceiling is off in them: the height of a range is meant to emerge from uplift, erosion and the surface stage's crust spreading (decisions log 2026-10-01, Ben: "the limit must emerge, not be a number"; STATUS stage 1 and 2 entries).
- **Resolution dependence** (P1-03 log "Not done"): continental area (ocean fraction 0.74 at 100k, 0.69 at 250k, 0.82-0.85 at 20k) and event counts (610 to 1,218 events from 100k to 250k) depend on sample spacing; "both for P1-17's tuning".
- **Sample-based fronts**: fronts, front lengths and slab shares are counts of boundary samples (71 km at 16k quick), "read changes in it, not its level" (plate-check baseline).
- **Cortial limitations the model inherits** (research note): rifting "progressively tears the crust"; "more forced subduction events than expected"; no passive margins or shallow shelves (the shelf share is an add-on); no drainage networks.
- **The start-up land jump** (4-6 % of the planet in the first 4 Myr with no collision) is known and left to a separate small task (decisions log 2026-10-02).
- **Mantle flow is a stand-in** in the force-balance design (5 mm/yr, "not physics the model computes", section 11 question 1); decided by Ben 2026-10-02.
- **Editor sliders**: every slider end must be valid (`every_knob_is_in_every_pipeline_preset_and_both_ends_are_valid`), so some ranges are wider than physical (for example `collision_suture_frac` 0.01-100).

## Flags

Quotes carry the file and, for the build log, the entry date. "Suspicion" marks my own reading, not a fact.

### The three admissions the task asked for (force-balance design, docs/design/plate-force-balance.md)

1. Gross size of slab pull, section 3.2, last paragraph: "Known sizes, for orientation: slab pull about 1-3 x 10^13 N/m gross, ridge push 2-3.5 x 10^12 N/m, a high plateau 6-7 x 10^12 N/m. Gross slab pull is from textbook examples and was not confirmed in Turcotte & Schubert itself." (ledger PLT-183; status from-memory)
2. The factor of 8 for continental drag, section 3.2 table row "Basal drag": "Continental roots raise the traction 2-5 times (Conrad & Lithgow-Bertelloni 2006, GRL 33:L05312, abstract). A factor of 8 is often attributed to Forsyth & Uyeda; not confirmed." (PLT-188). The design's own `keel_drag` default is 3.
3. The wedge-taper reference, section 3.5: "Source for a fixed taper: Davis, Suppe & Dahlen 1983, JGR 88:1153 (from memory)." and section 12: "From memory, not checked: Davis, Suppe & Dahlen 1983, J. Geophys. Res. 88:1153-1172." (PLT-192; `wedge_taper_deg` 1.5)

### Other "from memory" or "not confirmed" admissions

4. docs/design/plate-collisions-and-welding.md section 3: "No source gives the largest plate's share through a supercontinent's life. From memory: the plate carrying Pangaea was well under half the planet, with several oceanic plates in the surrounding ocean." This sits under targets PLT-143 and PLT-144.
5. Same section: "An orogen-wide mean surface uplift of 0.1-1 mm/yr is from memory." (PLT-180)
6. Same section: "Checked online on 2026-10-01 unless marked 'from memory'. Paywalled papers were confirmed from their abstracts or from secondary pages." Applies to PLT-143 to PLT-162 (abstract-only status).
7. docs/design/plate-force-balance.md section 12: "Checked online on 2026-10-02 by a sub-agent; most papers are paywalled, so figures are from abstracts or secondary pages unless a link is given. Items it could not confirm are marked in the text." Also "Bialas, Buck & Qin 2010, Earth Planet. Sci. Lett. (volume not confirmed)"; "DeCelles & Giles 1996 ... via a secondary source" (section 3.5); build log 2026-10-02: "Earth figures checked online by a Sonnet sub-agent; most papers were read from abstracts only."
8. docs/research/notes/run-review-1a0f969d358.md: "Earth has seven plates above 5 % of its area (the seven-plate count is sourced; the 5 % reading is from memory)"; "Earth is about 40 % (from memory)" (continental crust); "Earth's mean is about 800 m (from memory)" (land height); "11 % of land is above 2 km ... Earth's figures are about the same (from memory)". Note the P1-17 log's measured Earth median land height is 427 m, and the same note's 800 m (mean, not median) is therefore a different quantity from the 427 m the calibration targets. Run review also says "I have no sourced figure for Earth's turnover rate."
9. Crust spreading references (England & McKenzie 1982, Molnar & Lyon-Caen 1988, Bird 1991, Royden et al. 1997, Clark & Royden 2000, Rey et al. 2001, Beaumont et al. 2001) are listed only in the build log entry 2026-10-01 "Terrain above the build limit, stage 1"; none is cited in code. `crust_flow_km2_myr` is described there as "default to be calibrated, of the order of 10^3 km2/Myr: Clark & Royden's range is 10^2 for a strong, 10^5 for a weak lower crust", but the shipped value is 30,000 (nearer the weak end). Suspicion: the value was tuned, not derived; the "height limit, stage 2" entry shows D0 3,000 against 30,000 moving mean land from 0.277 to 0.317.
10. Several "Earth" values in code comments and editor hints have no source named: arc distance "150-300 km from their trench" (stats.rs), "Central Andes plateau stands near 4 km, the Cascades' base 1.2-2 km" (params.rs), "Andes 400-900 km" (editor hint and calib 09-29 log: "Andean orogen width, trench to foreland thrust front: 400-900 km (Central Andes)"), "Earth: about 14" plates (editor hint, see 13), "Earth: about -5,600 m" abyssal depth (editor hint).

### Contradictions and inconsistencies in the code and its comments

11. Two crust densities. pools.rs: `ISOSTATIC_FACTOR = 3300.0 / (3300.0 - 2700.0)` and doc "mantle 3,300 and crust 2,700 kg/m3". params.rs: `CRUST_DENSITY_KG_M3 = 2800.0` for the strength height and build log "rho_c 2,800". The surface stage's isostasy is 0.8 ("Airy rho_c/rho_m about 2.7/3.3" = 0.818; "isostasy 0.8 makes each km cost 5 km of erosion" = factor 5) while the plates use 5.5. The collisions design says the Airy share is 0.18 "already in pools::ISOSTATIC_FACTOR" (1 - 2700/3300 = 0.1818, consistent with 2,700, not 2,800).
12. Cortial's oceanic damping. params.rs default 0.04 and the research note's list of Appendix A constants say 0.04 mm/yr. The pools.rs test comment (not production code) says "Cortial's law (towards the -10 km trench at 0.05 mm/yr)". The 0.05 is the earlier pipeline preset (calib 09-29 log: 0.04 -> 0.05), not Cortial's.
13. Plate count. Editor hint (editor_fields.json, id plate_count): "Earth: about 14." The collisions design says Earth has 52 plates in Bird's model and "about seven large plates and many small ones"; its target for plates above 5 % is 5-9 with Earth 7. The default `initial_plates` 14 and the hint are not supported by either Earth reading in the docs (suspicion: 14 is a tuning value presented as Earth's).
14. The ocean floor claim in params.rs doc of `oceanic_floor_elevation_m`: "The pipeline presets use -5,600 m with eps_o 0.12 mm/yr (3.6 km at 20 Myr, 4.5 at 50, 5.2 at 100, 5.6 beyond) ... median sea depth 4.0 km, Earth 4.06". The P1-17 log's final table gives the chosen configuration a median sea depth of 3,488 m for the chosen run and 3,394 m for the ten-run mean, and its "Still off" list says "the sea median is 3.4 km, not 4.06". The 4.0 km in the comment matches a different variant in that entry's table (3,959 m, "K 4e-9, cap 2,000, G"). Also the same comment says "with eps_o 0.215 mm/yr from a -2,500 m ridge the law follows GDH1 within 0.12 km from 20 Myr on": that is for 0.215, not the shipped 0.12. Suspicion, computed from the test's own GDH1 formula (2,600 + 365 sqrt(t) then 5,651 - 2,473 e^(-0.0278 t)) against the shipped law (tau = 5,600 / 0.12 mm/yr = 46.7 Myr): GDH1 gives 4.23 km at 20 Myr, 5.04 km at 50 Myr and 5.50 km at 100 Myr; the shipped law gives 3.58, 4.54 and 5.24 km, that is 0.3 to 0.65 km shallower than GDH1 at all three ages. The build log accepts this on purpose ("GDH1's own rate (0.215) drowned the arcs").
15. Dead parameter. `abyssal_elevation_m` (-6,000) is validated (`trench < abyssal < ridge`) but no code reads it (rg over crates finds only params.rs). The settings-audit entry (build log 2026-09-29) says "`tectonics.abyssal_elevation_m` and `trench_elevation_m` (validated, never read by the model ...)"; `trench_elevation_m` is in fact read (subduction.rs, pools.rs, mod.rs).
16. Check labels and code disagree on a window. check.rs MEASURES label: "Active share of the continental margin (after the first 100 Myr)" and "Ocean floor, 90th percentile age, mean after the first 100 Myr"; the code comment and filter use 90 Myr ("mean over the records after the first 90 Myr of the record", `m.t_myr - first.t_myr >= 90.0`, `a.t - first_age_t >= 90.0`).
17. Subduction initiation rate. The P1-03 log says "Poisson, rate 0.1 per A0 of old crust per step" and design choice 6 is described with 150 Myr; the code default and every preset is 0.3 (P1-27 changed it, "keeps the floor recycled with sutures"). The calib 09-29 log (2026-09-29) says "Not changed: ... the plate-only presets and `TectonicParams::default` (Cortial's constants; the plate-only presets have no surface to flood their continents)", but the default also now carries P1-03/P1-27 additions (initiation 0.3, clearance 1,000 km, suture 0.1, orogen core 0.5), which are not Cortial's constants.
18. Slab-pull epsilon. params.rs calls `slab_pull_rad_per_1000km_myr` "Cortial section 4.1's epsilon scaled by front length"; the Appendix A constants as listed in the research note include no such epsilon, so the value 0.002 has no stated origin (suspicion: tuned; P1-27 log shows "slab pull (off)" changed active share only 0.56-0.66).
19. The collisions design's Earth reference "about 3-4 (P1-27's figure)" for collisions per 100 Myr: the P1-27 log has no such figure; the only other place is STATUS.md's P1-10 row ("about 10 collisions per 100 Myr vs 3-4; log 2026-10-01"), also unsourced. The target 2-6 rests on it (PLT-157).
20. The collision-length target (15-50 Myr, PLT-154) sits below the design's own Earth examples (India-Asia converging 50+ Myr, Alleghanian 65 Myr, Uralian 70-90 Myr, section 3) and the design says its 20-50 Myr timescale is "Inferred, to be measured" (section 4.2). Suspicion: the range was set by what the speed law's 20 Myr relaxation allows, not by Earth.
21. `collision_orogen_core` comment says "Earth's collision belts are a few hundred to ~1,500 km wide" while target PLT-152 is at most 800 km median and the design's Earth examples are 150-350 km (Himalaya) and 1,000 km (Tibet).
22. Collision counter uses magnitude of relative velocity, not the component across the boundary, and it never decays. The design calls it a bug (collisions design section 2.3 fault 2) and the plate-check entry says "Noticed, left alone. `subduction::resolve` adds relative speed, not the speed across the boundary, to `depth_km` (the design's bug)." Code comment in subduction.rs still says "mm/yr x Myr = km" with no caveat.
23. The run review calls plate speeds "Earth-like: median 52 mm/yr between plates, Earth about 40", while the later plate-check baseline measures a median plate speed of 72.6 mm/yr and the force-balance design says the present model's RMS is "about 65" against Earth "about 40" and slab-attached plates 1.4 times faster than the rest against Earth 3-4. Different quantities (convergence at new zones against plate speed), but the review's headline "Plates move at Earth-like speeds" is contradicted by the later figures.
24. Cortial table location. params.rs: "Appendix A, Table 3"; the research note: "Constants (Appendix A)" and "Table 1" for per-sample parameters, "Table 2" for timings. Cannot be settled without the PDF.
25. The mod.rs cost table and the plan: "Cortial's Table 2 ... crust generation and rifting every 20 to 120 My" is rewritten in mod.rs as "one crust generation and one rift every 20 steps (40 Myr)"; the 40 Myr is the implementation's own choice inside Cortial's range.

### Preset differences the task asked to note

26. `earthlike_full_16k.json`, `earthlike_full_32k.json`, `pipeline_full_16k.json` and `pipeline_full_32k.json` differ from the quick presets in two tectonics values only: `subduction_uplift_mm_yr` 0.34 (quick 0.42) and `collision_uplift_max_m` 1100 (quick 1400). Reason in the build log 2026-10-01 "height limit, stage 2": the full preset (250,000 samples) stayed higher than the quick one, and "uplift x 0.8" brought it inside the targets (n = 4). `earthlike_quick_32k.json` and `pipeline_quick_*` equal `earthlike_quick_16k.json`. The calibration was made on the 16k quick preset; the 32k presets inherit its values (the P1-17 log measured one 32k quick run: land 0.323, median 696 m).
27. The four `presets/plates/plates_*.json` carry exactly the Rust defaults (Cortial constants plus later additions) and differ from the quick pipeline preset in 16 keys: `ridge_elevation_m` -1,000 (pipeline -2,500), `max_elevation_m` 10,000 (30,000), `uplift_ceiling_m` 10,000 (0), `crust_flow_km2_myr` 0 (30,000), `subduction_distance_km` 1,800 (800), `collision_distance_km` 4,200 (2,000), `oceanic_damping_mm_yr` 0.04 (0.12), `oceanic_floor_elevation_m` -10,000 (-5,600), `continental_erosion_mm_yr` 0.03 (0), `subduction_uplift_mm_yr` 0.6 (0.42), `collision_uplift_max_m` 4,000 (1,400), `continental_shelf_share` 0 (0.29), `arc_uplift_mm_yr` 0.1 (0.6), `arc_continental_m` 0 (80), `cordillera_uplift_mm_yr` 0 (0.3), `cordillera_height_m` 2,500 (3,000); `l1_every_steps` 0 against 1 (software). So the plates-only presets are an uncalibrated plate model (their continents never flood, no spreading, Cortial 10 km ceiling), not "the same plate model".
28. The game's `tests/presets/planet_world_4k.json` (not read here) is said to keep older values ("the GameTest planet keeps its values; coordinator's call", P1-17 log; "4000 -> 2800" in the stage 2 entry), so its plate tuning differs from the presets above.

### Smaller observations (suspicions)

29. `ISOSTATIC_FACTOR` 5.5 means erosion of 1 km of surface strips 5.5 km of rock, so a 35 km column can be eroded by about 6 km of surface lowering; the surface stage uses 5 (0.8). Inconsistent stages (see 11), not necessarily wrong.
30. Hotspot basalt added is 3 times the rise and the LIP surface rise is one third of the pile (hotspots.rs): both are the same 1/3 ratio, written twice with no source.
31. The oxidation proxy (PLT-054) and heat-flow proxy (PLT-137) are numeric recipes with no source, yet they feed the deposit engine (plan section 8.1).
32. The editor hint for `subduction_initiation_age_myr` says "passive margins last 180 million years on average" while the P1-27 log (Bradley 2008) says modern passive margins average 104 Myr and ancient ones 181 Myr. The hint quotes only the ancient figure.
33. `time_step_myr` is not what the stage steps by: `PlatesStage::dt_myr()` is `length_myr / plate_steps`, so any code that treats "10-60 steps" as "20-120 Myr" is only true at 2 Myr per step.
