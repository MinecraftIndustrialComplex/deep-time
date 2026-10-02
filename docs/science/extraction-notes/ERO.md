# Extraction notes: erosion, isostasy and hypsometry

Written by the stage-1 extraction agent for this subsystem (2026-10-01/02) while it built the ledger. It holds the measured Earth-against-mod numbers found in the build log, the simplifications and game choices the code or documents state, and flags: admissions of unchecked values, contradictions between comments, code and documents, and things that looked wrong. **Nothing here is verified.** A line marked "suspicion" or "my arithmetic" is the extraction agent's own reading, to be checked in stage 2. See [claims.md](../claims.md).
Combined file for the three ledgers `ledger/ERO.md`, `ledger/ISO.md`, `ledger/HYP.md`. W = . Everything below was read from `STATUS.md`, the code and the docs; nothing was re-run or re-measured.

Abbreviations used by the build log (and here): land = true-area land share of the planet; z50 / z90 = median / 90th percentile of land elevation (m); >2 = share of land above 2 km; sea = median depth of the sea (m); crust = continental crust's share of the planet; dry = share of continental crust above sea; T = mean surface temperature (degC); ice = glaciated share of land; E, B = Koppen groups; active = share of margins that are active; arc = median ground height under continental arc cones (m).

## Earth against the mod

### E1. Earth reference figures used by the calibration

| quantity | Earth figure | stated source | where stated |
|---|---|---|---|
| land share of surface | 0.289 | GMT earth_relief 30 arc-minute (SRTM15+, Tozer et al. 2019), cos(latitude)-weighted like the CLI's line | STATUS.md build log 2026-10-01 "P1-17 hypsometry recalibration"; tools/review/p1-17-hyps.py docstring |
| land median elevation | 427 m | same | same |
| land 90th percentile | 2,022 m | same | same |
| land above 2 km | 0.102 | same | same |
| land mean elevation | 784 m | GMT earth_relief_01d (SRTM15+, "720 x 360") | build log 2026-09-29 "P1-17 partial: calibration with the real climate" |
| land above 1 km / 3 km | 0.253 / 0.048 | same | same |
| sea median depth | 4,062 m | same | both entries |
| sea mean depth | 3,683 m (Charette & Smith: 3,682 m, ocean 361.84 M km2 = 70.9 %) | Charette & Smith 2010, Oceanography 23(2):112-114, on SRTM30_PLUS | 2026-10-01 entry |
| shelf (0 to 200 m) | 5.4 % of the sea | GMT earth_relief_01d | 2026-09-29 entry |
| 1-km bands, below -6 km to above +5 km | 0.003/0.137/0.229/0.170/0.060/0.036/0.076/0.216/0.044/0.016/0.009/0.004/0.001 | GMT earth_relief | both entries |
| land median and p90 without Antarctica and Greenland | 388 m and 1,408 m | GMT earth_relief_01d | 2026-09-29 entry |
| continental crust / its dry share | 0.41 / 0.71 | Cogley 1984 | 2026-09-29 entry; params.rs |
| land glaciated today | about 10 % (Antarctica and Greenland, about 15.8 of about 149 M km2) | none given | 2026-09-29 entry |
| Koppen land shares | B 30.2, D 24.6, A 19.0, C 13.4, E 12.8 % | Peel, Finlayson & McMahon 2007 (HESS 11:1633) | 2026-09-29 entry |
| model's Koppen shares on Earth's relief (P1-05 check) | A 0.17, B 0.26, C 0.10, D 0.25, E 0.23 | model output on GMT relief | 2026-09-29 entry |
| mean surface temperature | 14 to 15 degC | no source given | 2026-10-01 per-run table, Earth row |
| land temperature | about 8.5 degC | no source given | 2026-09-29 entry |
| Earth's mountain ceiling | about 9 km | no source given | decisions log 2026-10-01 |
| Tibet's plateau | near 5 km | no source given | params.rs doc of crust_strength_mpa |

### E2. P1-17 hypsometry recalibration (build log 2026-10-01, agent aee888e01d29a40db)

Run: test VM, x86, `earthlike_quick_16k` (L0 100,000 samples, L1 1024 x 512), 500 Myr, `deeptime stage volcanoes`, `tools/review/p1-17-hyps.sh` (4 runs of 2 threads at once, niced), area weighted; "super" = supercontinent start, "scattered" = scattered continents. The presets measured here are the ones before the height-limit stage 2 landed (see E4: stage 2 re-set uplift x0.7 and these are not the final numbers).

**Which merge moved it** (old commits built from `git archive`, seeds 1-2 x both starts):

| build | land | z50 | z90 | >2 | sea | T | ice | E |
|---|---|---|---|---|---|---|---|---|
| 6070e9e (calibration + carbon cycle + runoff fix) | 0.283 | 625 | 2,172 | 0.118 | 3,312 | 13.8 | 0.090 | 0.26 |
| bd1cc0f (+ volcanoes, glacial landforms) | 0.383 | 1,385 | 4,281 | 0.343 | 3,130 | 10.3 | 0.220 | 0.42 |
| master (+ rivers, P1-27) | 0.381 | 1,211 | 3,933 | 0.307 | 3,076 | 8.9 | 0.226 | 0.39 |

**Drivers on master** (one change each; 4 runs, seeds 1-2 x both starts):

| variant | land | z50 | z90 | >2 | sea | crust | T | ice | arc |
|---|---|---|---|---|---|---|---|---|---|
| master | 0.381 | 1,211 | 3,933 | 0.307 | 3,076 | 0.385 | 8.9 | 0.226 | 753 |
| volcanoes.density 0 | 0.316 | 766 | 2,781 | 0.170 | 3,045 | 0.387 | 11.7 | 0.160 | - |
| surface.k_spl 4e-9 | 0.355 | 803 | 2,628 | 0.162 | 3,218 | 0.385 | 10.7 | 0.128 | 597 |
| tectonics.subduction_uplift_mm_yr 0.3 | 0.354 | 835 | 3,003 | 0.189 | 3,276 | 0.393 | 10.5 | 0.174 | 731 |
| tectonics.collision_uplift_max_m 2,000 | 0.324 | 915 | 2,750 | 0.198 | 3,001 | 0.382 | 11.5 | 0.142 | 710 |
| tectonics.cordillera_uplift_mm_yr 0 | 0.359 | 1,130 | 3,916 | 0.300 | 3,068 | 0.387 | 10.0 | 0.260 | 388 |
| tectonics.continent_elevation_m [50, 600] | 0.421 | 1,154 | 3,858 | 0.302 | 2,788 | 0.414 | 8.6 | 0.212 | 844 |
| climate.glacio_eustasy 0 | 0.394 | 1,173 | 3,982 | 0.319 | 2,771 | 0.409 | 7.7 | 0.263 | 654 |
| surface.sea_level.base_m 50 | 0.382 | 1,176 | 4,069 | 0.306 | 3,047 | 0.407 | 8.7 | 0.248 | 712 |
| surface.marine_fill_depth_m 40 | 0.406 | 1,367 | 4,695 | 0.370 | 2,657 | 0.405 | 7.5 | 0.299 | 778 |
| GDH1 floor -5,600 m at 0.215 mm/yr | 0.204 | 848 | 3,182 | 0.213 | 4,785 | 0.291 | 14.4 | 0.140 | 750 |

Also on the test machine before the pause (6 runs, own baseline land 0.450, z50 1,323): no glacial model z50 1,046, >2 0.254; fixed CO2 z50 1,383, T -0.3 with one frozen planet. Reading in the log: volcano cones (2e7 km3 built per run, a fifth of everything eroded) and the ice's protection of high ground raised the land; erosion efficiency, the collision cap and the Andean uplift bring it down; start elevation, sea-level datum, glacio-eustasy and shelf fill depth do not (noise of 4 chaotic runs about +-0.03 land and +-150 m).

**Calibration rounds** (4 runs each; K = k_spl, cap = collision cap, G = floor -5,600 m at 0.12 mm/yr; S = K 3e-9, cap 2,000, G):

| variant | land | z50 | z90 | >2 | sea | crust | T | ice | arc |
|---|---|---|---|---|---|---|---|---|---|
| K 4e-9, cap 2,000 | 0.343 | 720 | 2,218 | 0.131 | 2,921 | 0.414 | 11.0 | 0.088 | 548 |
| K 3e-9, cap 2,000 | 0.318 | 707 | 2,511 | 0.165 | 2,975 | 0.373 | 11.7 | 0.133 | 546 |
| K 4e-9, cap 2,500 | 0.354 | 749 | 2,522 | 0.155 | 2,901 | 0.389 | 11.6 | 0.131 | 534 |
| K 6e-9, cap 2,500 | 0.326 | 710 | 2,686 | 0.170 | 2,820 | 0.381 | 11.2 | 0.137 | 500 |
| K 3e-9, cap 1,500 | 0.311 | 684 | 2,134 | 0.114 | 3,254 | 0.379 | 11.7 | 0.092 | 525 |
| G alone | 0.239 | 805 | 2,979 | 0.194 | 4,179 | 0.312 | 12.1 | 0.170 | 766 |
| K 4e-9, cap 2,000, G | 0.219 | 543 | 2,080 | 0.109 | 3,959 | 0.322 | 14.3 | 0.092 | 592 |
| S; arc_continental_m 50 | 0.213 | 557 | 2,026 | 0.103 | 4,040 | 0.325 | 13.0 | 0.085 | 546 |
| S, 50, cordillera 0.3 to 3,000 m | 0.242 | 656 | 2,530 | 0.155 | 3,950 | 0.329 | 13.3 | 0.128 | 849 |
| S, 50, cordillera 0.4 to 3,000 m | 0.242 | 696 | 2,471 | 0.146 | 4,142 | 0.316 | 13.3 | 0.133 | 951 |
| S; arc_continental_m 30 | 0.203 | 482 | 1,920 | 0.093 | 3,934 | 0.325 | 12.8 | 0.083 | 612 |
| S, 50, volcanoes.density 0.5 | 0.233 | 582 | 1,710 | 0.070 | 4,019 | 0.332 | 13.1 | 0.068 | 506 |
| S, cordillera 0.3, arc_uplift 0.45 | 0.246 | 604 | 2,440 | 0.142 | 3,646 | 0.356 | 12.9 | 0.122 | 584 |
| **S, cordillera 0.3, arc 0.6 (chosen)** | 0.270 | 585 | 2,072 | 0.110 | 3,488 | 0.400 | 10.9 | 0.127 | 749 |
| S, cordillera 0.4, arc 0.6 | 0.290 | 758 | 2,670 | 0.170 | 3,308 | 0.422 | 11.0 | 0.184 | 825 |
| S, arc 0.6 (cordillera 0.2) | 0.264 | 585 | 2,219 | 0.123 | 3,408 | 0.416 | 11.8 | 0.184 | 497 |

**Before to after, per run** (master efafc88's presets to the recalibrated branch; seeds 1-5; same run recipe; "before to after"):

| start, seed | land | z50 | z90 | >2 | sea | T | ice | E | B | active | arc |
|---|---|---|---|---|---|---|---|---|---|---|---|
| super 1 | 0.459 to 0.300 | 1,059 to 711 | 3,845 to 2,499 | 0.267 to 0.158 | 3,059 to 3,071 | 6.8 to 11.2 | 0.223 to 0.227 | 0.38 to 0.41 | 0.20 to 0.18 | 0.29 to 0.31 | 806 to 801 |
| super 2 | 0.383 to 0.249 | 1,169 to 473 | 4,065 to 1,737 | 0.312 to 0.073 | 3,004 to 3,578 | 9.4 to 11.1 | 0.300 to 0.086 | 0.47 to 0.25 | 0.19 to 0.24 | 0.31 to 0.33 | 774 to 709 |
| super 3 | 0.425 to 0.232 | 1,149 to 338 | 3,538 to 1,791 | 0.287 to 0.082 | 2,870 to 3,412 | 9.5 to 10.9 | 0.183 to 0.170 | 0.38 to 0.34 | 0.18 to 0.15 | 0.35 to 0.29 | 872 to 618 |
| super 4 | 0.376 to 0.292 | 1,177 to 736 | 3,443 to 2,490 | 0.283 to 0.154 | 3,302 to 3,401 | 9.2 to 14.0 | 0.109 to 0.154 | 0.31 to 0.31 | 0.24 to 0.23 | 0.33 to 0.28 | 775 to 550 |
| super 5 | 0.421 to 0.333 | 1,124 to 722 | 3,400 to 2,866 | 0.254 to 0.181 | 3,070 to 3,306 | 9.0 to 10.2 | 0.177 to 0.196 | 0.34 to 0.38 | 0.22 to 0.16 | 0.34 to 0.33 | 747 to 777 |
| scattered 1 | 0.331 to 0.297 | 1,364 to 661 | 3,792 to 2,224 | 0.345 to 0.123 | 3,057 to 3,754 | 9.0 to 8.9 | 0.213 to 0.090 | 0.36 to 0.22 | 0.25 to 0.27 | 0.30 to 0.35 | 671 to 972 |
| scattered 2 | 0.351 to 0.233 | 1,251 to 494 | 4,030 to 1,829 | 0.306 to 0.085 | 3,184 to 3,548 | 10.6 to 12.5 | 0.168 to 0.104 | 0.35 to 0.24 | 0.22 to 0.28 | 0.34 to 0.31 | 762 to 514 |
| scattered 3 | 0.384 to 0.351 | 1,190 to 775 | 3,556 to 2,433 | 0.297 to 0.151 | 3,100 to 3,371 | 10.5 to 10.1 | 0.159 to 0.212 | 0.34 to 0.37 | 0.24 to 0.21 | 0.36 to 0.33 | 779 to 730 |
| scattered 4 | 0.659 to 0.350 | 2,505 to 845 | 5,441 to 2,563 | 0.616 to 0.165 | 1,676 to 3,224 | -28.1 to 10.4 | 0.908 to 0.179 | 1.00 to 0.32 | 0.00 to 0.23 | 0.28 to 0.36 | 1,006 to 745 |
| scattered 5 | 0.354 to 0.289 | 1,249 to 645 | 3,495 to 1,978 | 0.318 to 0.097 | 2,777 to 3,275 | 11.3 to 10.4 | 0.165 to 0.132 | 0.34 to 0.31 | 0.25 to 0.15 | 0.33 to 0.36 | 744 to 616 |
| **mean** | 0.414 to **0.293** | 1,324 to **640** | 3,860 to **2,241** | 0.329 to **0.127** | 2,910 to **3,394** | 5.7 to **11.0** | 0.261 to **0.155** | 0.43 to **0.32** | 0.20 to **0.21** | 0.33 to 0.33 | 794 to **703** |
| **Earth** | 0.289 | 427 | 2,022 | 0.102 | 4,062 | 14-15 | 0.10 | 0.13 (model on Earth 0.23) | 0.30 (0.26) | about 0.3 | 1,200-4,000 |

Master's scattered seed 4 froze (snowball to the present); none of the calibrated runs did. Continental crust 0.401 to 0.419 (dry 0.86 to 0.67; Earth 0.41, 0.71). Mean 1-km bands after: 0.020/0.117/0.139/0.130/0.093/0.053/0.155/0.191/0.063/0.023/0.009/0.004/0.002; before: 0.061/0.044/0.072/0.114/0.117/0.057/0.121/0.161/0.110/0.063/0.036/0.021/0.023 (Earth: 0.003/0.137/0.229/0.170/0.060/0.036/0.076/0.216/0.044/0.016/0.009/0.004/0.001).

**Other checks** (one run each unless stated): 1,000 Myr supercontinent seed 1: land 0.249, z50 430, z90 1,967, >2 0.097, T 11.5, ice 0.103 (master: land 0.375, z50 2,175, >2 0.521, T 4.9, ice 0.493); scattered seed 2: land 0.270, z50 585, >2 0.103, T 12.7, ice 0.096. `earthlike_quick_32k` seed 1, both starts: land 0.358 to 0.323, z50 1,181 to 696, >2 0.306 to 0.155, sea 3,330 to 3,847, T 11.2 to 9.2, ice 0.234 to 0.197. `earthlike_full_16k` scattered seed 1: land 0.604 to 0.378, z50 2,043 to 1,013, z90 6,032 to 2,896, >2 0.507 to 0.218, sea 1,662 to 2,858, T 0.1 to 8.9, ice 0.418 to 0.180 ("the full preset stays higher than the quick one", one run, needs its own ensemble). Land above 10 km: no calibrated run has any (maxima 6.5-9.3 km over ten runs); master's runs reached 10.7-13.0 km on five of ten planets.

**Still off, as the log states:** (1) climate on the cold side (T 11.0, ice 0.155, E 0.32 against 14-15, 0.10, model-on-Earth 0.23); (2) too much shallow sea, 0.155 of the planet between -1 and 0 km (Earth 0.076), so sea median 3.4 km not 4.06; (3) ground under arc cones about 0.7 km, not Andean 2-4 km (raising to 0.4 gave 825 m and cost 0.06 above 2 km and 0.06 ice); (4) the full preset.

### E3. Height limit, stage 1 (build log 2026-10-01 "Terrain above the build limit, stage 1")

Run: VM x86_64, 2 threads, `deeptime stage volcanoes`, 500 Myr `earthlike_quick_16k`; "before" = the same binary with the three bookkeeping changes switched off (reproduces master's 20,082 m on the test planet).

| 500 Myr earthlike_quick_16k | max, km | p99 land, km | median land, km | land fraction | cap bound |
|---|---|---|---|---|---|
| before: scattered seeds 1, 2, 3 | 9.2, 10.7, 9.2 | 6.7, 6.7, 6.4 | 1.45, 1.27, 1.05 | 0.28, 0.29, 0.37 | 0, 6e-5, 2e-5 |
| before: supercontinent 1, 2, 3 | 12.4, 9.1, 13.0 | 7.1, 6.4, 7.4 | 1.08, 1.31, 1.17 | 0.43, 0.42, 0.43 | 5e-5, 2e-5, 6e-5 |
| after: scattered 1, 2, 3 | 10.4, 9.1, 11.1 | 5.7, 7.1, 7.1 | 1.20, 2.03, 1.53 | 0.38, 0.50, 0.39 | 1e-5, 0, 1e-5 |
| after: supercontinent 1, 2, 3 | 11.2, 10.2, 9.0 | 7.5, 6.9, 7.4 | 2.07, 1.08, 1.52 | 0.47, 0.26, 0.38 | 5e-5, 3e-5, 0 |

Means over the six: max 10.6 to 10.1 km (highest 13.0 to 11.2), p99 6.8 to 7.0 km, median 1.22 to 1.57 km, land fraction 0.37 to 0.40. Test planet (`planet_world_4k`, 150 Myr, seeds 11-16): max 20.1, 7.5, 16.2, 2.5, 3.3, 4.6 km to 4.4, 4.2, 4.5, 4.4, 3.0, 3.7 km; p99 15.7, 6.1, 13.1, 2.3, 2.1, 3.6 to 3.1, 3.6, 3.9, 3.4, 2.1, 3.0 km; median 0.49, 0.69, 0.73, 0.18, 0.59, 0.30 to 0.33, 0.58, 0.43, 0.24, 0.59, 0.34 km; land fraction 0.19, 0.13, 0.14, 0.06, 0.19, 0.15 to 0.18, 0.10, 0.08, 0.06, 0.18, 0.15; cap bound 0.0024, 0.0016, 0.0024, 0, 0, 0.0005 to 0 on all six. Defect measured: the plate cap bound 0.24 % of continental sample-steps on the worst planet and under 0.01 % on the 16k worlds; 5 to 500 cells a step were lifted more than 1 km by the offset bookkeeping; erosion on cells above 10 km 0.3 to 1.3 km/Myr against 0.02-0.1 km/Myr on the land's average, about a quarter returned by isostasy. Bake after stage 1: 0 cells above `max_y` and 0 compressed on all six seeds (highest y 274-287); seeds 11 and 13 had 2.5 % and 1.0 % of their land above it before.

### E4. Height limit, stage 2 (build log 2026-10-01 "The height limit, stage 2")

Runs: VM the test machine, `tools/review/p1-17-hyps.sh`, 500 Myr, `earthlike_quick_16k`, area weighted, mean [min, max], uplift ("0.7") meaning `subduction_uplift_mm_yr` 0.6 to 0.42 and `collision_uplift_max_m` 2000 to 1400.

| | n | land | median, m | p90, m | above 2 km |
|---|---|---|---|---|---|
| Earth | | 0.289 | 427 | 2,022 | 0.102 |
| targets (recalibration 8600e03) | | about 0.29 | 430-640 | | 0.10-0.13 |
| before: stage 1 + recalibrated presets | 6 | 0.292 [0.283, 0.307] | 695 [604, 800] | 2,444 | 0.151 [0.115, 0.198] |
| no ceiling, D0 3,000 | 4 | 0.277 [0.243, 0.318] | 601 [481, 803] | 2,423 | 0.139 [0.095, 0.187] |
| no ceiling, D0 30,000 | 4 | 0.317 [0.248, 0.396] | 768 [513, 1001] | 2,496 | 0.163 [0.101, 0.225] |
| **the presets now: D0 30,000, uplift x0.7** | 6 | 0.273 [0.234, 0.310] | 547 [436, 701] | 2,102 [1,789, 2,392] | 0.114 [0.085, 0.152] |

Heights (same runs; seeds 1-3 x both starts): before max 6.4-7.8 km, p99 land 4.3-5.1 km; now max 5.7-6.8 km, p99 4.2-5.5 km; cap bound 0 in all. At 0.38 g (n = 4) max 9.5-11.0 km, p99 3.4-5.7 km, spread 0 (no cell ever reached the 13.4 km strength height; the 9.5-11 km summits are volcanoes under their higher ceiling); at 2 g (n = 4) max 3.42-3.43 km, p99 2.9-3.1 km, 2-3 million km3 moved per run against 0.2-0.7 million at 1 g. Plateau profile (seed 1, supercontinent, km per L1 cell of 4.8 km): before a dome 4.2 5.1 5.6 5.9 6.1 6.3 6.3 6.4 6.3 6.2 ...; now a table 0.8 1.0 2.3 3.2 4.5 5.4 5.6 5.6 5.6 5.7 ...; land above 5 km 0.30 % to 0.42 %, nothing above 6 km. Finished world (`deeptime run`, amplifier and bake, six default runs): highest terrain y 365-409 = 6.0-6.9 km (6.92, 6.17, 6.14, 6.03, 6.43, 6.45), p99.9 5.2-6.2 km, p99 4.0-5.6 km, 0 cells above the compression knee (y 575). Full preset (`earthlike_full_16k`, 2 seeds x 2 starts): land 0.327 [0.266, 0.359], median 746 m [652, 861], p90 2,491 m, above 2 km 0.148 [0.099, 0.168], L1 max 6.6-6.8 km; with uplift x0.8 (0.34 mm/yr, 1100 m; n = 4): land 0.292 [0.257, 0.319], median 630 m [577, 675], p90 2,049 m, above 2 km 0.106 [0.082, 0.131], max 5.7-6.8 km. Sea floor: deepest finished floor y -412 to -472 (9.5-10.7 km under sea) against min_y -352 (8.3 km); 0.01-0.08 % of sea cells below the floor. The spreading solve costs 22-35 s of a 190-240 s surface stage.

### E5. P1-17 partial, calibration with the real climate (build log 2026-09-29, agent ae48007457b607716)

Run: the test machine, `deeptime stage deposits`, 500 Myr, `pipeline_quick_16k` variants (throw-away scripts, not committed), seed 1 unless stated, 3 runs at a time. Baseline master 85b5671: land median 2.35 km (scattered) / 1.74 km (super), 56 % / 45 % of land above 2 km, Koppen E 53 % / 64 %, 31-32 % of land glaciated with the feedback off.

| run | land | z50 | z90 | >2 | sea | Koppen A/B/C/D/E | ice |
|---|---|---|---|---|---|---|---|
| base master, scattered | 0.233 | 2,352 | 5,587 | 0.56 | 1,710 | .10/.12/.21/.04/.53 | .307 |
| base master, super | 0.215 | 1,743 | 4,602 | 0.45 | 1,714 | .03/.08/.09/.16/.64 | .317 |
| K 1e-9 alone, scattered | 0.205 | 1,624 | 6,106 | 0.44 | 1,705 | .13/.16/.22/.06/.43 | .249 |
| band 800 km alone | 0.194 | 1,752 | 4,152 | 0.44 | 2,685 | .04/.22/.16/.18/.40 | .130 |
| + collision 2,000 km | 0.153 | 1,026 | 2,298 | 0.15 | 2,262 | .13/.21/.26/.09/.31 | .109 |
| i8 (isostasy 0.8), scattered | 0.144 | 1,025 | 2,661 | 0.18 | 2,373 | .16/.21/.16/.19/.29 | .062 |
| i7 (isostasy 0.7), scattered | 0.158 | 631 | 2,339 | 0.13 | 3,626 | .11/.42/.18/.15/.14 | .020 |
| km100 (marine D0 100, wave base 60), scattered / super | 0.160 / 0.164 | 750 / 591 | 2,391 / 1,539 | 0.14 / 0.06 | 3,427 / 3,933 | | .079 / .063 |
| km30 (D0 30), scattered / super | 0.179 / 0.210 | 762 / 969 | 2,405 / 2,516 | 0.15 / 0.19 | 3,707 / 3,588 | | .124 / .026 |
| sh29 (+ shelf share 0.29), scattered / super | 0.324 / 0.331 | 888 / 983 | 2,538 / 3,100 | 0.17 / 0.24 | 2,346 / 2,316 | .19/.22/.19/.16/.24 and .20/.24/.24/.10/.23 | .047 / .053 |
| cc100 (carbonate_clastic_m 100) scattered / super | 0.250 / 0.317 | 586 / 668 | 2,123 / 2,234 | 0.11 / 0.12 | 3,490 / 3,250 | carbonate platform 2.6 / 3.5 % vs 3.8 / 4.4 %: rejected | .082 / .030 |
| **ocK2 = final**, scattered / super | **0.270 / 0.282** | **601 / 624** | **2,095 / 1,679** | **0.11 / 0.07** | **3,229 / 3,334** | .18/.21/.24/.19/.18 and .19/.25/.16/.16/.23 | **.058 / .067** |
| final seed 2, scattered / super | 0.273 / 0.327 | 634 / 862 | 1,941 / 2,803 | 0.09 / 0.20 | 3,549 / 3,472 | .17/.26/.22/.19/.16 and .23/.27/.19/.11/.19 | .037 / .033 |
| final seed 3, scattered / super | 0.287 / 0.314 | 785 / 1,055 | 2,093 / 3,749 | 0.11 / 0.31 | 3,207 / 3,308 | .13/.28/.17/.21/.22 and .09/.17/.20/.16/.38 | .045 / .177 |

Summary in the log: scattered / supercontinent land median 2,352 / 1,743 to 601 / 624 m (Earth 427); p90 5,587 / 4,602 to 2,095 / 1,679 m (Earth 2,022); above 2 km 0.56 / 0.45 to 0.11 / 0.07 (Earth 0.10); land share 0.233 / 0.215 to 0.270 / 0.282 (target 0.30); sea median 1,710 to 3,229 / 3,334 m (Earth 4,062); bands (scattered) 0.127/0.049/0.073/0.158/0.111/0.047/0.164/0.186/0.055/0.017/0.006/0.004/0.003; glaciated land 0.31 / 0.32 to 0.058 / 0.067; land temperature 9.2 / 8.8 degC (Earth land about 8.5); over three seeds x two starts land 0.27-0.33, median 0.60-1.06 km, above 2 km 0.07-0.31, ice 3-18 %, E 16-38 %. Continental crust and its dry share at the end: base 0.305 / 0.71; i8 0.23 / 0.62; final 0.36-0.41 / 0.65-0.79 (Earth 0.41 / 0.71). Land share every 50 Myr from -450 (scattered final): 0.36, 0.34, 0.33, 0.31, 0.29, 0.30, 0.29, 0.29, 0.30, 0.28, median between 0.49 and 0.85 km. Mass residual -4e-4 to -1.9e-3 km3 on 4.9e7-1.05e8 km3 eroded. Earth check (`deeptime climate-dem --dem target/earth/earth_720.f32 --width 720 --preset pipeline_quick_16k`): glaciated land 9.3 to 9.7 % (Earth about 10 %); EBM 15.3 degC, net TOA -0.003 m2, land 7.7 degC and 563 mm/yr, groups A 0.17 / B 0.26 / C 0.10 / D 0.24 / E 0.24, 19 of 24 cities in the right main group and 5 exact; the climate was not tuned. Ocean law before: Cortial's ridge at -1 km and damping 0.04 mm/yr put 57-59 % of the planet between -3 and 0 km (Earth 17 %), median sea depth 1.7 km. With the chosen ridge -2.5 km and damping 0.05: depths 3.2 km at 20 Myr, 4.2 at 50, 5.5 at 100 against GDH1 4.2 / 5.0 / 5.5 (no source given for the GDH1 numbers).

### E6. P1-04 first build (build log 2026-09-28)

Acceptance run `deeptime stage surface --preset pipeline_quick_16k --start scattered_continents --length-myr 500` (the test machine): `land_frac=0.215 max_elevation_m=10048 sediment_km3=9.6e6 eroded_km3=1.20e8 marine_km3=1.06e8 fluvial_km3=1.1e7 lake_km3=3.1e6 carbonate_km3=6.9e5 evaporite_km3=1.4e5 mass_residual_km3=-6.2e-4`; 88 % of eroded rock reaches the sea. Tuning: n = 1, m = 0.5 flattened continents in about 20 Myr; defaults then n = 2, m = 1, K = 3e-10, isostasy 0.6, kappa 2, land fraction 0.30 to 0.26 to 0.215 over 300-500 Myr "instead of 0.15"; interior highlands 3-4 km; no Earth figure is compared there. Cost: 1.05 microsecond per cell per step single-threaded against the plan's 1 microsecond.

### E7. Oracle against fastscapelib (build log 2026-09-28 P1-04; tools/validation/fastscape_compare.py)

`nix shell nixpkgs#python312 -c python3 tools/validation/fastscape_compare.py --grid 256` (fastscapelib 0.3.0): 256 x 256, dx 4.8 km, K 2e-6, m 0.5, n 1, U 1 mm/yr, 500 x 0.1 Myr, same initial noise: `hypsometry_ks=0.0209 chi_slope_ratio=1.0000 chi_fit_ratio=0.9997 area_log10_ks=0.0038`, PASS. Seeds 1-5 hypsometry KS 0.0209, 0.0127, 0.0100, 0.0169, 0.0395; area KS 0.0038, 0.0022, 0.0024, 0.0055, 0.0029; chi ratio 1.0000 on all. From the same mature surface one step agrees cell for cell (99.98-100 % identical areas, mean abs dz 2 mm). Networks diverge from the initial noise (a third keep the same receiver at 50 Myr) because Deep Time fills lakes with sediment at G = 0.

### E8. Claims in code comments and docs that compare mod to Earth

- params.rs (k_spl): "the land's median ends 0.6 km high and its 90th percentile ~2 km, Earth's 0.43 and 2.0 km" for K = 2e-9; the build log's chosen run ocK2 shows 601 / 624 m and 2,095 / 1,679 m; with K 3e-9 and the later changes the mean is 547 m and 2,102 m (E4).
- params.rs (kd_marine): "at 300 the shores retreated ~250 km in 500 Myr and drowned a third of the continental crust once the continents were Earth-low".
- params.rs (oceanic_floor_elevation_m): "ε_o 0.215 mm/yr from a -2,500 m ridge ... follows GDH1 within 0.12 km from 20 Myr on"; pipeline presets -5,600 m with 0.12 mm/yr: "3.6 km at 20 Myr, 4.5 at 50, 5.2 at 100, 5.6 beyond"; at GDH1's full rate "the continental crust shrinks from 0.39 to 0.29 of the planet in 500 Myr (median sea depth 4.0 km, Earth 4.06)".
- plan section 8.2 (as built): "500-Myr pipeline_quick_16k runs now end with Earth-like hypsometry (land median 0.6-1.1 km, Earth 0.43; above 2 km 7-30 %, Earth 10 %), land 27-33 %".

## Simplifications and game choices

Stated in the code or docs:

- **Local isostasy, no flexure.** Compensation is a smoothed local share (isostasy 0.8, 0.7 under water, box-blur radius 50 km) and not an elastic-plate solve. A general flexure solve for every load (belts, volcanic islands, ice, sediment) was chosen by Ben on 2026-10-02 (decisions log, "General flexure everywhere"); the surface's blurred local isostasy is what it replaces (docs/design/plate-force-balance.md 5.6 (b)). Today's surface code is unchanged.
- **D8 single-direction routing** (not multiple flow): the implicit stream-power solve needs one receiver; "grid artefacts of D8 are below what the keyframes resolve" at 4.8 km cells (routing.rs).
- **Flood kept serial** on purpose (result depends on its own order; every checkpoint is compared bit for bit).
- **Marine sediment** is delivered by a Deep Time rule (clinoform fill from the mouth) and then diffused; there is no grain size, no turbidity current, no longshore drift, no porosity or compaction in the surface stage. Facies depend only on process share and the setting thresholds, not on grain size (the research note's "grain-size fraction" input is not implemented).
- **Carbonate** differs from Landlab: temperature factor and clastic suppression added, growth capped at sea level; no pelagic carbonate or reef-specific rate beyond Bosscher-Schlager.
- **Weathering** differs from Landlab: climate (precipitation, temperature) scales w0; regolith kind rule and its thresholds are "design values".
- **Facies rules** are "a proposal, not a published scheme" (research note, facies.rs); "every number below is a design value".
- **Crust spreading** is a diffusion of the surface (conservative) that moves crust, not surface rock; "nothing is clamped"; the plate model's cap is a guard (30 km) and its uplift ceiling is switched off in the pipeline presets.
- **Gravity enters the height limit only through rho_c g h**; the build log notes that fold belts on a light planet are therefore not markedly taller because erosion and uplift do not depend on gravity in the model (Ben was told; "For Ben" item).
- **Build limit squeeze** (BuildFit) is a game safety net: nothing above a knee 10.24 km above sea survives, smoothly compressed; chosen by Ben (decisions log 2026-10-01).
- **Surface physics stays in real metres** at map scale (decisions "Tectonics scaling"), so K keeps its meaning at 4.8 km cells; the 32k presets are twice the real length scale and "erode more" with the same K (build log 2026-09-29).
- **Sea level**: the preset's two sinusoids stand for tectonic sea level; glacio-eustasy comes from the climate stage; "calibrated coastlines" kept over the Earth reference ice volume (Ben, 2026-09-30).
- **Events** (pulses, basins, transgression) are thresholds on frame means, tracked by centroid distance.
- **Defaults in Rust differ from the shipped presets** for k_spl (2e-9 vs 3e-9), the sea-level cycles (none vs two) and several tectonics keys (`SurfaceParams::default` was deliberately left at K 2e-9; build log 2026-10-01).

## Flags

Admissions of "from memory" or "not retrieved" and unchecked values (quoted):

1. **Weathering constants.** research note `docs/research/notes/surface_processes_climate.md`, section 3 Gaps: "The rate constants for Landlab `w0` and `w_star` or Heimsath's soil production were not retrieved. The Heimsath abstract was unavailable." Code values `weathering_max_m_myr` 50 m/Myr and `weathering_decay_depth_m` 5 m (ERO-069, ERO-070) have no recorded source; marked from-memory on that basis (the note admits the constants were not retrieved; it does not say where the code's numbers came from).
2. **Braun & Willett 2013 read as abstract only.** research note section 1: "[Braun & Willett 2013 (Geomorphology 180-181) abstract via ResearchGate]"; code says "Implemented from the papers" (spl.rs, routing.rs). ERO-001, ERO-078, ERO-079, HYP-030 marked abstract-only. The Yuan et al. 2019 reference (DOI 10.1029/2018JF004867) is described only in one sentence in the note; no record of the full text being read.
3. **Braun 2016 and laterite.** note section 3 Gaps: "the Braun 2016 full text (coefficients) was not accessible"; the laterite conditions come from Wikipedia: Laterite ("high temperatures and heavy rainfall alternating with dry periods"), yet weathering.rs cites `surface_processes_climate.md` section 3-4 for the rule. Thresholds are admitted "design values (the literature gives conditions, not numbers; P1-07 tunes them)".
4. **Facies rules.** research note section 4 Gaps: "No retrieved tool outputs evaporite or coal facies directly. Those facies rules are my proposal, not a published scheme that was checked." facies.rs and weathering.rs say numbers are "tuned in P1-07"/"P1-07 tunes them"; in the parts of the P1-07 build-log entry (2026-09-29) that were read for this ledger (its metamorphism, pluton and "doubtful" paragraphs) no tuning of the facies or regolith thresholds is recorded; it mentions only that little evaporite/coal reaches the surface (0.3 %, "P1-04's facies volumes"). Not every line of that entry was read.
5. **Landlab defaults not shown as retrieved.** marine.rs and carbonate.rs state Landlab defaults (D0 100, wave base 60, tidal range 2; G_m 10 mm/yr, k 0.1, I0 2,000, I_k 400) "the equations and defaults"; the research note cites the Landlab source files but lists none of these numbers, and no build-log entry says they were read. ERO-046 to ERO-048 and ERO-058 to ERO-061 are cited-unverified.
6. **spread.rs sources "checked" means bibliographic.** "Each checked against its publisher's record on 2026-10-01 (title, year, pages)": the claims drawn from them (diffusion form with the cube of channel thickness; threshold weight S) are not said to have been read. Beaumont et al. 2001 (Nature 414:738) appears in the build log's design but not in spread.rs. The D0 range "10^2 for a strong, 10^5 for a weak lower crust" is attributed to Clark & Royden with no page.
7. **Peel et al. 2007, Cogley 1984, Parsons & Sclater 1977, Shepard (shelf break), Charette & Smith 2010, Tozer et al. 2019, Stein & Stein 1992** are cited in build-log text or comments only; nothing says how they were read.
8. **Izu-Bonin-Mariana arc crust production "60-120 km3/km/Myr"** and "Central Andes 400-900 km" (HYP-015, HYP-017) have no reference given.

Contradictions and inconsistencies:

9. **k_spl: comment and default say 2e-9, every preset says 3e-9.** `SurfaceParams::default().k_spl` is 2.0e-9 and its doc comment, and the params.rs example ("k_spl": 2e-9), describe the 2e-9 calibration ("land's median ends 0.6 km high ... Earth's 0.43"). All eight presets carry 3e-09 (build log 2026-10-01 changed presets only: "`SurfaceParams::default` (K 2e-9)" left alone). The plan section 8.2 still says "use K = 2e-9"; the editor list in STATUS says "erosion strength k_spl 1e-10...1e-8 (2e-9)" (hint updated later per the 2026-10-01 entry).
10. **Preset sea level vs Rust default.** `SeaLevel::default` has no cycles; every preset has two (80 m / 250 Myr and 25 m / 36 Myr at phase 90); the params.rs example shows one. None of the three is sourced.
11. **Ocean floor is shallower than the GDH1 numbers quoted in the log.** The 2026-09-29 entry gives GDH1 as 4.2 / 5.0 / 5.5 km at 20 / 50 / 100 Myr; the shipped presets (-5,600 m, 0.12 mm/yr) give 3.6 / 4.5 / 5.2 km, i.e. 0.3-0.6 km shallower, while the decisions log 2026-10-01 accepts "the age-based (GDH1) ocean floor as the Earth-like default". The 0.12 rate was chosen below GDH1's 0.215 so island arcs keep emerging; the quoted GDH1 depths themselves have no source.
12. **params.rs comment on the GDH1 full-rate variant** says continental crust "shrinks from 0.39 to 0.29 ... median sea depth 4.0 km, Earth 4.06", but the build-log table row "GDH1 floor -5,600 m at 0.215 mm/yr" has sea 4,785 m and crust 0.291 (and "G alone" at 0.12 has sea 4,179, crust 0.312); no row has 4.0 km.
13. **Earth DEM named three ways.** run.rs hypsometry_line doc: "Earth on GMT's 1 degree relief"; build log 2026-09-29: `earth_relief_01d` with "720 x 360" (a 30 arc-minute grid is 720 x 360, a 1 degree grid is 360 x 180); build log 2026-10-01 and p1-17-hyps-plot.py: "GMT earth_relief 30'". The land fraction 0.289 and the other Earth numbers are the same in all of them.
14. **Target ranges are not Earth's values.** The recalibration "targets" row gives land median 430-640 m and >2 km 0.10-0.13 against Earth 427 m and 0.102; the final presets (E4) sit at 547 m / 0.114 (quick) and, in the full-preset ensemble, 630 m / 0.106. The "target" is therefore a band whose upper end is 50 % above Earth's median; and the figures in the build log E2 table (640 m mean, 0.127) are from before stage 2's uplift re-set.
15. **Shelf depth in three places:** continental_shelf_share initialises shelves "0 to -130 m, Shepard's mean shelf break"; the deposit engine's `shelf` field is sea shallower than 200 m (ERO-044); the hypsometry line's shelf share is 0 to 200 m (HYP-008).
16. **Crust densities differ:** spread.rs / params.rs use 2,800 kg/m3 (ERO/ISO-009); plates `pools.rs` uses crust 2,700 and mantle 3,300 for the Airy factor 5.5 and the isostasy 0.8 (2.7/3.3); granite.json density 2,650. None is cited. The strength height therefore depends on the 2,800 choice (5.1 km at 140 MPa), while the isostasy ratio uses 2,700.
17. **Thermal critical slope:** thermal.rs "noisifiedAngle x0.9 to x1.4" but plan 8.2 says amplifier noise is plus or minus 25 percent; the critical slope values in play are 0.35 (surface, ERO-013), 0.6 (amplifier, plan) and Tzathas 2024's atan 30 degrees (0.577) quoted in the research note.
18. **Elevation of arc ground target inconsistent:** the per-run table's Earth row says arc ground "1,200-4,000" m; the "Still off" list says "an Andean 2-4 km".
19. **Test comment:** spread.rs test says "at 0.38 g (13.2 km)" while the doc says 13.4 km (the test uses S/(rho g) = 13,200 as a stand-in; the formula gives 13.4 km at 0.38 g with 2,800 kg/m3 and 140 MPa).
20. **Facies comment:** the `Setting` doc says aridity "below 0.2 hyper-arid, above 1 humid" while the thresholds used are 0.3 (evaporite), 0.15 (halite), 0.5 (red beds), 1.2 (coal).

Suspicions (labelled as the extractor's, not established):

21. SUSPICION: `isostasy_radius_km` 50 is called "a flexural wavelength" but is the half-width of a 3-pass box blur of the load (local compensation spread out); the effective Gaussian sigma is about 50 km, while real flexural wavelengths of continental lithosphere are usually quoted as hundreds of km. No elastic thickness is stated anywhere. (ISO-004)
22. SUSPICION: `0.7` for water-loaded compensation (ISO-003) is unexplained; it resembles (rho_c - rho_w) / (rho_m - rho_w) with 2,700 / 3,300 / 1,000 kg/m3 (about 0.74), but nothing says so.
23. SUSPICION: source-rock TOC of 3 wt % for coal (ERO-036) is below what coal contains (coal is mostly organic carbon); likely a stand-in for the petroleum "source" ranking, not a physical TOC.
24. SUSPICION: `carbonate_clastic_m` = 20 m is described as "m per step" and compared with the clastic thickness a cell received in the plate step (2 Myr) and, with substeps > 1, accumulated so far; the e-folding therefore depends on the step length, not on a rate.
25. SUSPICION: runoff weight uses runoff over `runoff_ref_m_yr` 0.5 m/yr with m = 1; the log says K and the reference are "degenerate with K for m = 1", so changing the reference without changing K has no effect, but the reference is documented as if it had physical meaning (full area at 0.5 m/yr).
26. SUSPICION: the K comment says "P1-17 calibration ... Earth's 0.43 and 2.0 km" for the land median and p90, while the shipped value (3e-9) is described as compensating "the cones' added rock and the ice's protection" (build log 2026-10-01): the calibration target of K moved from "match Earth with the real climate" to "offset volcano cones and glaciers", so K is partly a patch for two other stages.
27. SUSPICION: hypsometry means in the 2026-10-01 per-run table pool two different starts and chaotic histories (n = 10 runs, histories "chaotic: ... a mean moves by about +-0.02 in land and +-100 m in the median between batches"), so differences between presets smaller than that are within noise; calibration to "430-640 m" is inside that noise.
