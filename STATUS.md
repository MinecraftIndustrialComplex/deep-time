# Deep Time — status

Phase: 1 (Phase 0 exit passed 2026-09-28) · Landed 2026-09-29: P1-05 climate, P1-06 young landscape + amplification, P1-07 rock/strata, P1-14 deposits on the recorded history, P1-17 partials (parallel SPL, calibration with real climate), P1-20 workbench stages 1–2 + dark UI, settings audit (world-shaping knobs as editor settings) · In flight: diffusion loop fusion · P1-22 rivers and lakes done on its branch (worktree-agent-ad204cf8d55ef26f5) · Coal capped to the sedimentary cover + rock-layer settings (σ 10 default) done on its branch (strata-coal-settings) · Continuous strata (format 0.2) done on its branch (worktree-agent-aacd864d2709d52dd) · P1-08 bake + format 0.1 landed (built-ins are real planets) · History spill to the SSD done on its branch (worktree-agent-a33c2dc374c0ccd2f) · P1-09 climate mapping done on its branch (worktree-agent-a116e3d557a130679) · P1-21 volcanoes done on its branch (worktree-agent-a348bd3bf6e50dd72) · Next: deposit formula (survivors) + MVT regional driver · Sub-agents: Sonnet 5.5 for well-specified work, Opus for hard work (Ben, 2026-09-29) · MIC GameTest heap stall fixed on its branch (worktree-agent-a11ef8b35a6ebb8ee; `-PwithMic` 5 of 5) · EF ice cap vs barren frozen ground (placement `glaciated` block, `weirdness`) done on its branch (worktree-agent-adda9513719f24370) · Satellite view (true colour, the default layer of the workbench, viewer and timelapse) done on its branch (worktree-agent-a7ca2a0e875263a86) · P1-27 plate margins (sutures, initiation clear of trenches, arc cordilleras) + coherent provinces done on its branch (worktree-agent-ad3b417fd428c1ac2) · Rivers & lakes on the workbench maps (view-only; satellite overlay + a `rivers` layer) done on its branch (worktree-agent-a3d4fadfe3e053b77) · P1-28 stone and deepslate for the common rocks done on its branch (worktree-agent-a72907749021691b5) · Updated: 2026-10-01 (session f981c0d3-5144-49ec-8686-2946d755872c)
Plan revision: see `git log -1 --format=%h -- docs/deeptime-implementation-plan.md`
MIC pack tested against: 303fd89 (snapshot of HEAD, re-indexed; see known issues)

## Tasks
| ID | Title | State | Date | Commit | Evidence |
|---|---|---|---|---|---|
| P0-01 | Repository bootstrap and the docs move | done | 2026-09-28 | 2725107 | gh repo view → PRIVATE; 14 docs copied, sha256 equal; nix develop: rustc 1.98.1, JDK 21.0.12.1, zig-0.13.0, cargo-xwin 0.23.1; mic/Docs pointers written |
| P0-02 | Cargo workspace and CI | done | 2026-09-28 | 677b6e6 | cargo test --workspace ok (6 crates); clippy -D warnings ok; cargo deny: bans ok, licenses ok. CI run: see build log |
| P0-03 | Gradle NeoForge module skeleton | done | 2026-09-28 | 774a3ef | gradlew build → BUILD SUCCESSFUL; junit-summary tests=1 failures=0; ./gametest.sh → All 1 required tests passed; checkVersions fails on 0.0.2 |
| P0-04 | Deep Time's server sets and smoke harness | done | 2026-09-28 | — | standalone + mic smoke PASS (laptop run); Mac: ./gametest.sh -PwithMic → All 7 required tests passed with 160 MIC mods (dev-run exclusions: Connector, antique-atlas, surveyor, Distant Horizons; see build log) |
| P0-05 | Generator skeleton on the analytic test terrain | done | 2026-09-28 | 33f1cd9 | ./gametest.sh → all pass (heightMatchesReference cornerMax 0, maxAbsDiff 0; periodicAcrossSeam 0 mismatches; veins off); planet server (Mac) boots with level-type deeptime:planet, probe column 0 0 → surfaceY 14 for H 14.99. -PwithMic run: with P0-04 |
| P0-06 | Toroidal World spike (server-side items) | done | 2026-09-28 | — | Mac planet server → PASS: columnscontinuous, entitycrossesseam, structurestraddlesseam, treestraddlesseam pass; shaftlineacrossseam (informational) rotation propagates; jetcrossesseam 20/20 attempts (one PASS); prototypepushback pass; `sable_straddle: OK` (19 blocks, same id). MIC smoke PASS at 32k (tw.widthChunks 2048) and 16k (1024): stamped=true loopsZ=false climateCompression=off, `regions outside span: 0`. No TW failure (no Q6). Review page not published: files in build log |
| P0-07 | JNI hello world, native loader and the five failure classes | done | 2026-09-28 | c813177 | Mac: JUnit tests=30 failures=0; NativeSmoke OK + 5 forced classes; ./gametest.sh NativeGameTests pass: native version=0.0.1 format=0.0 target=aarch64-apple-darwin selftest=1997318690096807936. Linux load path first runs in CI |
| P0-08 | Cross-builds and the OS load matrix | done | 2026-09-28 | b20fbb8 | Mac: `nix run .#natives` → 8 libraries, gnu GLIBC_2.17, musl NEEDED libc.so only; `gradlew build -Prelease` → 8 in the jar, tests=30 failures=0; [natives.yml run 36492319610](https://github.com/MinecraftIndustrialComplex/deep-time/actions/runs/36492319610): all 9 jobs success, 8/8 load jobs `status=OK version=0.0.1` (Windows arm64 included) |
| P0-09 | World-data format v0, synthetic planets, Java reader | done | 2026-09-28 | 75d90d3 | Mac: cargo test ok (format 15 tests); synth wrap-test 16k → format_version 0.0, viewer bundle; TilePackParityTest + Lz4RoundTripTest pass; wrap-test equals AnalyticTestTerrain within 1/32 block |
| P0-10 | The generator on world data | done | 2026-09-28 | — | Mac ./gametest.sh → All 10 required tests passed: heightMatchesRaster cornerMax 0 maxAbsDiff 1; strataClassesResolve checked=32 wrong=0; spawnOnLand y=456 > 63. StagingMover deferred to P1-11 (no staging before the editor exists) |
| P0-11 | `deeptime-inspect` | done | 2026-09-28 | 86140f2 | Mac: cargo test -p deeptime-inspect ok (19 unit + 7 fixture); `cargo run -p deeptime-inspect -- compare --world smoke/standalone/server/planet --world-data …/deeptime` → p99_abs_diff_blocks 1 (p50 0, max 5, 99.6 % within 1 block; 109,184 of 128,000 columns compared); seam 144 rows, 0 mismatches; clippy, fmt, deny ok |
| P0-12 | Terralith, TerraBlender and Biomes O' Plenty checks | done | 2026-09-28 | 00db177 | Mac: ./gametest.sh default/-PwithMic/-PwithTerraBlender → All 13 passed; terralith=3008 of 4096 land samples (MIC), biomesoplenty=1325 of 4096 (TB), surface painted 24/24; smoke mic --synth wrap-test (production server) PASS: probe tw stamped/loopsX/!loopsZ/1024, locate village found, no C2ME/DH errors, chunky done, regions outside span 0 |
| P0-13 | Water above sea level holds | done | 2026-09-28 | — | Mac ./gametest.sh → All 14 required tests passed: lakeHolds before=65304 after=65304 expected=65304 outside=0 (every source woken by a fluid tick, 400 ticks forceloaded), water_top_y=120 at the lake centre, generatedFluidTicks=0. The inspect section PNG (review picture) comes with P0-11; -PwithMic (Mac): All 14 required tests passed, lakeHolds identical (outside=0) |
| P0-14 | BlueMap renders on MIC blocks | done | 2026-09-28 | — | https://spacesheep.dev/@benmac/deeptime-p0-bluemap (private): MIC smoke world with synthetic world data, box 1952..2143 × 416..607 around the test mountain, surface + below-y-0 maps; gzip tiles unzipped in the browser (BlueMap clientDecompression), 29 MB/174 files; loads in headless Chromium over plain HTTP; the hosted page itself is private, so its first real load is Ben's. No service worker needed |
| P0-15 | Phase 0 exit | done | 2026-09-28 | — | tools/review/phase0.py → 11 of 11 pass (Rust, JUnit, GameTests default/MIC/TerraBlender, planet server seam+Sable+pole, smoke standalone + MIC 16k/32k, inspect compare, natives.yml); https://spacesheep.dev/@benmac/deeptime-phase0 |
| P1-01 | Session, checkpoints, keyframes, event log and site formats | done | 2026-09-28 | — | Mac: cargo test -p deeptime-format 27 ok (segments + sites round-trip, two writers merge with no id collision); run --preset dummy.json → manifest.json, events/1-0.dtev, keyframes/index.json (13 events, 9 keyframes, 3 sites); test --workspace, fmt, clippy -D warnings, deny licenses bans clean; gradlew build test tests=34 failures=0 and ./gametest.sh all 14 pass (after merging master 2f81ae5) |
| P1-02 | Fibonacci sphere, triangulation and raster coupling | done | 2026-09-28 | 0ea8a1b | Mac: cargo test -p deeptime-core sphere:: → 23 passed (V−E+F=2; every 16k-quick L1 cell: weights sum 1 ±1e-6, ≥ −1e-9, containing triangle; k-d nearest = brute force on 10,000 points); sphere_bench 250k: hull 255 ms (12 threads; 280 ms on 1), k-d tree 8 ms, L1 2048×1024 coupling 41 ms, 0 bad cells on all four L1 grids |
| P1-03 | Plates and crust | done | 2026-09-28 | 0867a61 | Mac: cargo test --workspace ok (plates::/params::/preset:: 20 + 1 ignored: rift area conserved exactly, partition, ages ≥ 0, hull invariants after every resampling, byte-identical resume); 500 Myr 20k: p90 ocean age 167/169/245 Myr, 98/97/90.5 % < 250 Myr; `stage plates --preset plates_quick_16k --start scattered_continents --length-myr 500` → plates_final=13 ocean_age_p90_myr=202.3 steps=250 seconds=8.3; 100k 8.3 s (12 thr) / 23 s (1 thr), 250k 25–31 s / 72 s vs Cortial 76–106 / 330–437 s; clippy, fmt, deny clean |
| P1-04 | Deep-time surface processes on L1 | done | 2026-09-28 | 50dd95f | Mac: cargo test --workspace ok (core 139 incl. surface:: 20: receivers acyclic, depressions resolved, diffusion Σh conserved, SPL ramp = analytic within 1e-6 (n 1) / 1e-3 (n 2), strata-log invariants, coupled pipeline resume identical; cli 11); `stage surface --preset pipeline_quick_16k --start scattered_continents --length-myr 500 --viewer review/out/p1-surface` → 250 steps 96 s (plates 22 s, surface 74 s), land_frac 0.215, mass residual −6e-4 km³ of 1.2e8 eroded, 50 frames, 70 erosion pulses / 265 basins / 3 sea events; bench 1 thread surface 137 s = 1.05 µs/cell/step vs the 131 s cost model; fastscapelib oracle hypsometry_ks=0.0209 chi_slope_ratio=1.0000 PASS; fmt, clippy -D warnings, deny clean; gradlew build test 106/0 |
| P1-05 | Climate | done | 2026-09-29 | — | Mac: cargo test --workspace ok (core 162 + 1 ignored incl. climate:: 14: EBM balance |ASR−OLR| < 0.2 W/m², energy conserved by diffusion/ice to 1e-9, ridge windward > 3× lee with a rain shadow, Köppen Af/BWh/Cfb/Dfc/ET/EF/Aw/Csa/Dwd hand-built cases, idealised planet has all five groups, coupled pipeline resumes identically on 1 vs 3 threads); `stage climate --preset pipeline_quick_16k --start scattered_continents --length-myr 500 --viewer review/out/p1-climate` → 51 snapshots, climate 11.7 s of 191 s, t_surface 11.3 °C, EBM 15.6 °C, net TOA 0.006 W/m², land Köppen A 0.10/B 0.12/C 0.21/D 0.04/E 0.53; bench 12 vs 1 thread identical; Earth relief (`climate-dem`): 19/24 cities in the right Köppen group; clippy, fmt, deny clean; gradlew build test 118/0 (after merging master 6d8171f) |
| P1-06 | Young landscape, amplification and the detail layer | done | 2026-09-29 | — | On `pipeline_quick_16k` (built-ins stay placeholders until P1-08). Mac: `cargo test -p deeptime-core surface::amplify` → 25 passed (no new sink outside the lakes on a wrapped planet with a lake, the flood repairs a pit nobody declared, streamed = whole-raster to the bit, tiled middle level = whole, 1 = 5 threads, the seam, drama, pack round trip, coupled pipeline on 1 and 3 threads, resume); workspace tests ok (core 204 + 2 ignored, cli 23, format 33, inspect 37, jni 7, viewer 5), clippy -D warnings, fmt, deny clean; 500 Myr planet: `stage young` 7.6 s, `stage amplify` 8192×4096 → sinks=0, 18.7 s on 12 threads / 53 s on 1 (erode 7.6/24.1, thermal 3.3/14.1, deposit 3.3/8.9, flood 3.2/2.9), streamed (8 tiles, halo 196) and 1 thread give all 512 of 512 tiles of both L3 layers identical to the in-memory run; 134 M-cell L3 (32k full's size) streams in 58.5 s; views: review/out/p1-amplify (16 MB, not published). Ben reviews: the before/after views and the drama figure |
| P1-07 | Rock classes, column strata and metamorphism | done | 2026-09-29 | — | On the coupled pipeline preset (built-ins stay placeholders until P1-08). Mac: cargo test --workspace ok (core 149 + 1 ignored incl. structure:: 11: stacks ordered top-down, ≥ 1 block thick, classes valid, ≤ K; resume identical; every built-in deposit type finds all its fields incl. trap_structure and host/*), fmt, clippy -D warnings, deny clean; gradlew build test 106/0; ./gametest.sh All 45 passed (everyClassInTheWorld 59/59, 240 samples 0 wrong); `stage structure --preset pipeline_quick_16k --start scattered_continents --length-myr 500 --viewer review/out/p1-structure` → horizon_check=ok, 3.88 horizons/cell, 22,000 plutons (8,138 bodies), 50 frames published; structure 4.8 s of 122 s |
| P1-08 | Bake, world data v1 and parity | done | 2026-09-29 | — | Format 0.1; `deeptime run --preset earthlike_quick_16k --start scattered_continents --length-myr 500 --out … --viewer review/out/p1-q16` → format_version 0.1 (752 s on a loaded Mac, bake 1.1 s, folder 156 MB); `./gradlew test --tests '*ColumnParityTest'` → tests=2 failures=0 errors=0 (500 of 500 columns equal, 77 through bodies, JNI sampleColumn identical); JUnit 124/0; real-preset first start PASS (generation 492 s, 441 chunks in 19 s); built-ins are the pipeline presets; see build log |
| P1-09 | Climate mapping and biomes | done | 2026-09-29 | — | Real planet (earthlike_quick_16k, 500 Myr), `tools/review/p1-09-smoke.sh standalone mic` → both smokes PASS; whole-planet census (1024 × 512 samples) standalone / MIC: BW desert or badlands 75.1 / 78.0 % (any dry tag 90.4 / 92.3 %), Af jungle 84.3 / 82.2 %, Dfc taiga 88.9 / 66.5 %, ET/EF snowy or icy 97.7 / 99.3 %; Chunky areas rendered with `deeptime-inspect biomes` agree. cargo fmt/clippy -D warnings/test (core 238 + 3 ignored)/deny clean; JUnit 140/0; `./gametest.sh` All 50 passed (climateConsistent, bandsAreTheBuilders); `-PwithMic`: every finished test passes (48 of 50) but lakeHolds stalls and the JVM dies of heap, 4 of 4 runs (known flake, see build log). Review bundle review/out/p1-09 (3.4 MB, not published). See build log |
| P1-10 | TerraBlender regions follow provinces | done | 2026-10-01 | | Branch worktree-agent-a7e713f9fdd326fb4 (session f981c0d3-5144-49ec-8686-2946d755872c), not merged. The bake's `meta/province` is now the USGS classes (decisions log 2026-09-29) cut into contiguous provinces; with TerraBlender every province gets one region (stored by name in `climate_mapping.json`), and one mixin on `MultiNoiseBiomeSource.getNoiseBiome` makes a Deep Time level's biome query use it. Test VM: `./gametest.sh -PwithTerraBlender -PgametestOnly=regionsFollowProvinces` passes on `test_planet_real` (34 provinces, 4 regions, 4,608 samples, 0 biomes outside their province's region, 223 land samples under mod regions); with the mixin removed the same test fails (245 biomes outside); the real 16k planet with BOP (smoke): 40 provinces, review/out/p1-10. Orogen share: classified 0.42 of the continents (Earth ~0.17), thresholds are settings, defaults unchanged, see Pending questions. cargo fmt/clippy -D warnings/deny clean, test (core 384 + 3 ignored, cli 54), JUnit 208/0, `./gametest.sh` 66 passed; `-PwithTerraBlender` 67 of 67 after the merges with master 5e601d9 and 2e53ad2 (`climateConsistent` had failed in that profile on master itself, see the build log follow-up); `-PwithMic` 67 of 67. See build log |
| P1-11 | Customize editor, timelapse and staging | doing | 2026-09-28 | d42375e | Screens, models and staging built on the placeholder presets (real stages pending). Mac, merged with master f14d258 (P1-03, P1-18): JUnit tests=101 failures=0 (new: EditorModelTest, TimelapseModelTest, ModelHelpersTest, StagingMoverTest, TimelineAcrossJniTest, ScreenSourcesTest); ./gametest.sh → All 31 required tests passed (EditorModelGameTests acceptBuildsGenerator, stagingMovedOnStart, editorRunsAcceptsAndBinds, shareCodeThroughModel, realPresetFieldsLightUp); cargo fmt/clippy -D warnings/test --locked (193 passed)/deny clean; gradlew build clean. Client (laptop, HeadlessMC + Xvfb): whole walk Create World → editor → run → review → Accept → world created from the moved staging, no crash; guidance panel with natives forced off; screenshots in review/out/p1-11-client. Rerun with the real presets after P1-08; biome names on the review page with P1-09 |
| P1-12 | Strata pass with rock tags and `rock_states` | done | 2026-09-28 | — | Mac: ./gametest.sh and -PwithMic (+ MIC rock tags via -Ddeeptime.extraDatapack) → All 21 required tests passed each; coal stone / black argillite; Create stones in their classes; loess = projectatmosphere:dust[layers=8]; everyClassInTheWorld 59/59 classes, 240 samples 0 wrong; MIC smoke probe palette 59/59 (member 24, fallback 35); JUnit 46/0; bodies deferred (P1-08/P1-14) |
| P1-13 | Dedicated server: generate on first start | doing | 2026-09-28 | 743b69d | Mechanism verified on placeholder presets (dummy pipeline + wrap-test bake): standalone and MIC first starts generate in ~12 s with progress lines ≤ 5 s apart, restart does not regenerate; no-config / circumference-mismatch / native-failure all stop cleanly before loadLevel. Rerun with the real presets after P1-08 |
| P1-14 | Deposit engine: the four families | done | 2026-09-29 | — | On the real recorded history (plates → surface → structure → deposits, `pipeline_{quick,full}_{16k,32k}` have a `deposits` section). Mac: cargo test --workspace ok (core 155 + 1 ignored incl. deposits::recorded_tests 6: 1 vs 3 threads give identical sites/bodies/events/census, a checkpoint inside the deposit stage resumes on another thread count to the same sites, favourability of a rigidly drifting crust = the fixed-crust map moved cell for cell, missing fields skip exactly the types that read them and change nothing else, every family on recorded frames; cli 21 incl. `stage deposits` on the real history; format 32, inspect 28, jni 7, viewer 5); clippy -D warnings, deny licenses bans clean; gradlew build test 118/0; fmt clean for every file this task touches (watch/editor.rs on master is not); `stage deposits --preset pipeline_quick_16k --start scattered_continents --length-myr 500 --viewer review/out/p1-deposits-real` → 154 s (plates 30.0, surface 111.6, structure 5.0, deposits 4.4), 5 sites at count_mult 1 (porphyry 1, coal 4; MVT, SEDEX, petroleum expected 0.00 / 0.00 / 0.11), 42 at count_mult 10; 17 MB bundle. Ben reviews: ore provinces, site list, three chains; see the build log for what is empty and why |
| P1-15 | Deposits in the world | doing | 2026-09-28 | — | On the synthetic deposit world (real fields wait for P1-03…P1-08). Mac: ./gametest.sh, -PwithEv, -PwithMic → All 30 required tests passed each (DepositGameTests body 1222/1222 ore exact, 0 missed, 0 unexplained, regeneration 0 of 44,880 differ; ClaimGameTests; SpawnGameTests); ./gametest-planet.sh claimoninplanetworld PASS standalone (19 removed) and -PwithMic (38 removed); JUnit 62/0 incl. DepositDataParityTest (19,649 body samples, worst Δ 3.3e-16); smoke standalone first-start deposit world PASS + `deeptime-inspect ore` render |
| P1-16 | COE and CDG hooks | done | 2026-09-28 | — | Mac: `./gametest.sh` and `-PwithMic` → All 45 required tests passed each; CompatGameTests with COE 1.6.8 + CDG 1.3.15: 7 of 7 site chunks = the site's vein in COE, 30 of 30 claimed noise veins dropped (COE picks them in the overworld), 34 unclaimed kept, 33 stored randomMul = the site's quantile (24 = Rust's drawn quantile, worst Δ 2.8e-8), 16 trap chunks = charge × cdg_scale × share, 111 chunks outside traps 0. MIC smoke (16k deposit world) PASS: `probe coe` at 9 site chunks = the site's vein (agrees), `probe cdg` over trap 27 > 0 (infinite / 553,979 mB), 0 elsewhere; `/coe locate` copper not found, redstone found. JUnit 106/0; cargo fmt/clippy/test/deny clean. Driven by the synthetic deposit world (see build log) |
| P1-17 | Benchmark gate and preset tuning | doing | 2026-09-29 | | Partial (surface parallelism, branch worktree-agent-a9bc0ab99e68b4a07): stream-power solve 34.0 → 6.5 s at 12 threads, bit-identical; flood left serial on purpose; diffusion is now the largest surface cost; see the build log entry "P1-17 partial". Partial (calibration with the real climate, branch worktree-agent-ae48007457b607716, Ben 2026-09-29): pipeline presets + surface/climate/structure defaults + four deposit balance multipliers; Mac: `stage deposits --preset pipeline_quick_16k --length-myr 500` scattered / supercontinent → land median 2,352 / 1,743 → 601 / 624 m (Earth 427), above 2 km 0.56 / 0.45 → 0.11 / 0.07 (Earth 0.10), land 0.233 / 0.215 → 0.270 / 0.282, Köppen E 0.53 / 0.64 → 0.18 / 0.23, glaciated 0.31 (feedback off) → 0.06 / 0.07 (on), sites porphyry 1 / 1 → 5 / 34 (median 8 over 3 seeds × 2 starts), SEDEX, MVT, oil on 4, 3, 5 of 6 planets, plutons 7,830 / 5,374 → 1,192 / 885 bodies, mass residual ≤ 1.9e-3 km³; Earth check 19/24 (unchanged), 9.7 % glaciated with the feedback; cargo test/clippy/fmt/deny clean, gradlew build test 118/0; bundles review/out/p1-calib-{scattered_continents,supercontinent} (20 / 15 MB, not published); see the build log entry "P1-17 partial: calibration" Partial (history spill, branch worktree-agent-a33c2dc374c0ccd2f, Ben 2026-09-29): the recorded history spills to the run folder over the memory cap; `pipeline_full_32k` 1000 Myr finishes under 4 GiB (2.3 GiB on disk); spilled = in-memory byte for byte; see the build log entry "the recorded history spills to the SSD" Hypsometry recalibration 2026-10-01 (branch worktree-agent-aee888e01d29a40db): 10 runs × 500 Myr land 0.414 → 0.293, median 1,324 → 640 m, above 2 km 0.329 → 0.127, sea median 2,910 → 3,394 m, T 5.7 → 11.0 °C, glaciated 0.261 → 0.155; see the build log entry "P1-17 hypsometry recalibration 2026-10-01" |
| P1-18 | Pole cap and client info | done | 2026-09-28 | — | Mac: JUnit 65/0; ./gametest.sh All 26 passed; planet PASS: entitypushedback, hardstopat80, sablehullpushedback, worldinfochannelrequired + seam set + sable_straddle; refuse_client OK (NeoForge client without Deep Time refused naming deeptime:world_info + pole_storm; with it accepted; vanilla refused). Client code compiled only (see build log) |
| P1-19 | Phase 1 exit | todo | | | |
| P1-20 | Browser workbench on the Mac (log 2026-09-28): stage 1 editor controls + live runs + compare; stage 2 click-to-inspect and 3D block preview | doing | 2026-09-28 | | Both stages in parallel (Ben, 2026-09-28): stage 1 grows out of the live viewer; stage 2 runs the real generator headless on the Mac and exports the area (`deeptime-inspect export-area`), so it does not wait for P1-08; a fast Rust column path joins after P1-08. **Stage 1 built** (branch worktree-agent-a3abc4a868f0c3b81): `tools/viewer/mac-watch.sh` → http://mac-mini:8642/ (tailnet only); Mac: cargo test --workspace, clippy -D warnings, fmt, deny clean, JUnit 107/0; two real 500-Myr pipeline_quick_16k runs watched live (the scattered run equals P1-04's batch numbers: 1,238 events, land_frac 0.215), compared side by side, screenshots looked at; stage 2 hooks are 501 stubs (see build log). **Stage 2 built** (branch worktree-agent-a63e3a8af506069de): `deeptime-inspect export-area` + `tools/workbench/preview.py` (real generator headless on the Mac) + `tools/review/pages/area3d/` viewer, wired as `deeptime watch --preview`; 256 × 256 × 992 boxes in 44 s (standalone) / 90 s (MIC), verify 0 wrong, 6/6 probe site agree; until P1-08 a preview is the synthetic deposit world at the box's lat/lon and says so (see build log). **Follow-up** (branch worktree-agent-afd5791e0f5ea82cd): a run with a bake is previewed and inspected as its own terrain (`/api/inspect` from `deeptime inspect-column --why` in 60-90 ms, previews boot the mod on the run's bake; height p50 0 vs the bake); runs without a bake keep the test world and say so (see build log). **UI overhaul** (branch worktree-agent-a95571be05fdd89d6, Ben: "should be dark themed"): dark map-first layout, run cards, sectioned editor with the estimate by Run, split/side-by-side compare, timeline dock, stage-2 job cards, `?tv` HUD, 390 px; placeholders hidden; Mac gates clean, JUnit 118/0; before/after in review/out/workbench-ui/ (see build log) |
| P1-21 | Volcanoes as landforms (log 2026-09-29) | done | 2026-09-30 | | Branch worktree-agent-a348bd3bf6e50dd72 (session f981c0d3-5144-49ec-8686-2946d755872c): coupled `volcanoes` stage (arc stratovolcanoes on the volcanic front ~60 km apart, hotspot shields and island chains that age and subside, calderas, flood-basalt plateaus from LIPs), volcanic rock in the strata log, `deeptime:volcano_*` events with causes (workbench, `inspect-column --why` landforms, porphyry pathway), block-scale cones/craters/calderas/terraces in amplify, every knob a setting. Mac: earthlike_quick_16k 500 Myr 418 s (volcanoes 14.1 s), 203 live arc cones, 8 standing shields, 295 calderas, 9 flood basalts, 1,468 events; cargo test/clippy -D warnings/fmt/deny clean; gradlew build test 146/0; `mac.sh --server -- ./gametest.sh` All 50 required tests passed; bundle review/out/p1-21/bundle (11 MB, not published); follow-ups (log 2026-09-30): flank steepness setting (default 0.5), one event per arc segment / hotspot chain, segments under 3 cones counted on their subduction onset, young volcanoes exempt from the mountain pass's unsharp mask; see build log |
| P1-22 | Rivers and lakes (moved from P2-03, log 2026-09-29) | done | 2026-09-30 | — | On branch worktree-agent-ad204cf8d55ef26f5, not yet merged. core::hydro in the bake (lakes full to their spill level; river network `rivers/rivers.dtrv`, format 0.2), the column rule on both sides, the WaterBodies pass, river biomes, `hydro` settings. <br>**Real 16k 500 Myr planet:** 27 lakes (68,139 km²), 4,489 rivers, 119,813 km, widest 36.7 blocks; water part 4.9 s. <br>**Parity:** 500/500 and lake1024 300/300. <br>**GameTests:** `./gametest.sh` and `-PwithMic` 53/54 each on master 0905793 (riversHold 0 floating, 0 leaks; the one failure was master's climateConsistent, fixed on master since); `./gametest-planet.sh` PASS; the last merge (5d4147b) got cargo and Gradle only (157/0), per the new rule. <br>**Previews:** 0 floating and 0 river/lake leaks in 3 of 4 boxes, 1 on a boulder in the 4th; the whole lake's rim − surface = 0. <br>**Chunk time:** the pass costs 0.3 ms a chunk. <br>Review bundle review/out/p1-22 (not published); see the build log |
| P1-22b | Climate-sized and frozen lakes (log 2026-10-01; the lake part of P2-03 pulled forward) | done | 2026-10-01 | — | On branch worktree-agent-a2195c5c858c73075, not yet merged. Lake levels from a per-basin water balance (full, below the rim, playa or dry), frozen lakes, pointer text, 5 `hydro` settings; no format change. <br>Bake B: lakes 4.51 % → 2.46 % of land, largest 74,262 → 9,995 km². <br>Gates: cargo, Gradle 160/0, GameTests 57/57, browser check 11/11. See the build log. |
| P1-23 | Grid-artifact audit (log 2026-09-29) | done | 2026-10-01 | — | Branch worktree-agent-a851cf286456a3901 (sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c), not merged. Audit: docs/research/notes/grid-artifact-audit.md. <br>**Clean:** the seam, the mesh, slope directions (within 2° of an axis 4.8 % against 4.4 %), cones, deposits. <br>**Shows, worst first:** straight sediment dams and lake shores on plains (3–8 blocks high, up to 1,000 blocks long); rivers in dead-straight reaches (10.2 % of their length within 0.5° of an eight-direction heading against 2.2 %); biome borders in 16-block steps (24 % of the temperature input's change faces along an axis against 4.4 %). <br>**Fixed, each with a property test:** rock erodibility interpolated between L1 cells (its steps were 7 × larger at cell edges, now 1.02 ×), the strata smoothing's lattice (thickness phase 15 % → 7 %), the distance to the sea (inland a8 −0.39 → 0.00). <br>**Gates:** cargo fmt/clippy/deny clean, cargo test 546 passed, JUnit 199/0, `./gametest.sh` and `-PwithMic` (168 mods) All 65 required tests passed each. <br>Four design questions for Ben in the note and the build log. |
| P1-24 | Faster Create World (log 2026-09-29) | todo | | | After P1-17's benchmarks; options go to Ben. |
| P1-25 | Climate API for mic-climate and Project Atmosphere (log 2026-09-29) | done | 2026-09-30 | — | On branch worktree-agent-a02882c5e6567370e, not yet merged. Format 0.2 bakes twelve monthly layers. `DeepTimeClimate` is at API version 1.<br>**Parity:** `ClimateParityTest` 500/500 blocks bit for bit, and the GameTest `climateApiOnTheRealPlanet` matches too.<br>**mic-climate `deep-time` branch** (pushed): Deep Time's climate plus Project Atmosphere's weather anomaly. Its GameTests pass 19/19, with and without Project Atmosphere, and its pack smoke passes 20/20.<br>**MIC planet probe** (earthlike_quick_16k): Af 24.5 °C, BWk 8.8/19.6 °C, Dfc −12.9/15.3 °C, ET −24.7/0.6 °C, EF plateau (4,860 m) −22.7/−46.7 °C, each pair midwinter/midsummer. Project Atmosphere read 8.6, 8.9, −0.6, 4.0 and 10.5 °C at the same places. Its simulated weather adds −2.4…+2.5 °C.<br>JUnit 150/0 and `./gametest.sh` All 52 passed after merging master 8872c91; cargo gates clean. See build log. |
| P1-26 | Per-cell climate: 2-D energy balance, sea ice, moisture (log 2026-09-30) | doing | 2026-09-30 | | Design approved 2026-10-01 (docs/design/p1-26-per-cell-climate.md, recommended answers); tasks T1–T11 start after the climate agent's ice-sheet branch (worktree-agent-adc924860a859865c-ice) lands. |
| P1-27 | Plate margins and coherent continents (logs 2026-09-29, 2026-09-30) | done | 2026-09-30 | | Branch worktree-agent-ad3b417fd428c1ac2 (session f981c0d3-5144-49ec-8686-2946d755872c), not merged. Cause: a collision moved the continent alone to the overriding plate, leaving its passive margins facing an ocean that kept converging (collisions cut 295,000–455,000 km of margin off its ocean per 750-Myr run; rifts ~2,000, initiations < 300). Fixed by sutures, initiation anywhere clear of trenches (rate 0.3), arc cordilleras, collision orogen cores, arc maturity; provinces view generalised to the USGS map's 150 km. Mac, 6 pipeline runs × 400 Myr before → after: active margin 0.67 → 0.31 (Earth ~0.3, Bradley 2008), passive 0.30 → 0.63, floor beside passive margins 50 → 102 Myr, ground under arc cones 307 → 812 m, provinces orogen 0.51 → 0.40, shield+platform 0.17 → 0.29, edge density 0.078 → 0.060; land 0.398 → 0.391, median 1,350 → 1,030 m, ocean p90 108 → 194 Myr. cargo test/clippy/fmt/deny clean, JUnit 155/0; evidence review/out/p1-27/ (not committed); see build log |
| P1-28 | Stone and deepslate for the common rocks (log 2026-10-01; first the hardness tiers, replaced the same day) | done | 2026-10-01 | | Branch worktree-agent-a72907749021691b5 (session f981c0d3-5144-49ec-8686-2946d755872c), not merged. `minecraft:stone` is the `vanilla_fallback` of shale and mudstone, `minecraft:deepslate` of slate, phyllite, schist and gneiss (`CRUST_STAND_INS` in `tools/dev/gen_rock_classes.py`); no recipe, item tag or drop touched. Stone or deepslate share of the placed rock, top 64 blocks / whole column: 0.0 % → 47.8 % / 55.3 % (12 generated boxes, standalone), census of the whole planet 0.1 % → 56.2 % / 0.0 % → 56.0 %. Gates: cargo fmt/clippy/test/deny clean, gradlew build test 161/0, `./gametest.sh` and `-PwithMic` All 58 passed. MIC rows are in the build log; see the entry. |
| P2-08 | Meteor impacts (log 2026-09-29; started on a branch, log 2026-10-01) | done | 2026-10-01 | | Branch worktree-agent-a24e90e955a5a64d3 (sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c), master 38ca6b0 merged in, not merged to master, not pushed. Coupled `impacts` stage, craters in the amplifier, `impact_breccia` and `impact_melt` (62 classes), four impact deposit types, events, workbench layers, eleven settings. Quick 16k, 500 Myr: 1,302 impactors, 567 craters formed (448/81/28/5/4/1 by size), 735 stopped by the sea, 439 eroded, 57 subducted, 71 in the catalogue, 19 showing (17 on continental crust). Build log 2026-10-01, "Meteor impacts". |
| P3-04 | Distant Horizons far terrain (asked for ahead of Phase 3, Ben 2026-10-01) | done | 2026-10-01 | — | Branch worktree-agent-a6e8592da3ce000d8, not merged. DH LODs from the world data through a world-generator override bound by name (DH 3.3.0's merged `_neoforge` API and 3.3.1+'s published one). VM: JUnit 180/0 (`FarColumnTest` 14, `DhApiBindingTest` 4 on the real 3.3.0 and 3.3.3 jars); `FarTerrainGameTests` 3 of 3 (ground within a block 117 of 119, water and biome 119 of 119); smoke `dh` (`run.py --dh`): registered, DH's server-side pregen fed 257 sections with no failure, what DH stores has far terrain's surface in 400 of 400 columns, far terrain 9.6–11.1 ms a section; smoke `dh-next` (DH 3.3.3): Toroidal World refuses the override (pending question). Looks unverified (no client). See build log |
| P2-14 | Geothermal features (log 2026-10-01; started ahead of Phase 2 on its own branch, Ben approved) | done | 2026-10-01 | | Branch worktree-agent-abbb227c962473b98 (sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c), master e8668ca merged in, not merged to master. Hot springs and sinter/travertine terraces where groundwater is available, fumaroles with native sulfur and altered ground where it is dry, around the active volcanoes of `volcanoes/active.json` (the only heat the bake exports). 16k quick planet: 260 sites, 23 pools, 142 vents, 95 patches around 40 of 67 volcanoes; JUnit `GeothermalTest` 14; `./gametest.sh` and `-PwithMic` 66 of 66; block census on the real generator: 0 leaks, 0 water on lava, standalone = MIC site for site. Build-log entry "P2-14 Geothermal features". Four `underground` settings, density 0 = none; `DeepTimeClimate.geothermalWarmthC` (API 3). |

| P2-01 | Fold-and-fault engine (Noddy evaluator; pulled forward, log 2026-10-01) | doing | 2026-10-01 | | Branch worktree-agent-a49703993341aa701 (sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c), **not merged: waits for the coordinator's merge and GameTest batch**. Stage A (the port, bit-exact against the original C) and **stage B** are built: format 0.3 with `structure/events.dtst`, the deformed column rule in Rust and Java (`ColumnParityTest` over deformed columns, 173 JUnit tests, 0 failures), unconformities by age key, stocks, dykes and plugs as intrusion events, traps from the world's own structures, fault scarps and rift valleys in the amplifier, grouped `fold_belt` / `fault_system` events, `deeptime structure-sections` and the structure census of `strata-check`. `earthlike_quick_16k`, 500 Myr: 12,657 events (1,081 fold trains, 3,848 thrusts, 1,256 normal and 261 strike-slip faults, 2,760 stocks, 2,647 dykes, 804 plugs); they change 52 % of land columns, none of 20,000 malformed; the column rule costs 0.60 ms a chunk through them against 0.44 without (Java, one thread); land share 0.2770 with the relief and without, land median 661.1 m against 661.0. cargo fmt, clippy -D warnings, test --workspace, deny and `gradlew build test` clean after merging master ea40d3b (200 JUnit tests, GameTests 65 of 65 default and MIC). GameTests on the branch (allowed for this task): `./gametest.sh` and `./gametest.sh -PwithMic` (168 MIC mods): all 60 required tests passed in each. See the build log. |
| SCI-1 | Science documentation, stage 1: the claims ledger, bibliography, feature pages (log 2026-10-02) | done | 2026-10-02 | | Branch worktree-agent-a33b33f9ecaf3af1a (sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c), not merged, not pushed. Docs only: `docs/science/` (README, claims.md with 2,069 ledger rows in `ledger/`, bibliography with 475 entries, 403 spot checks, two full pages, 17 stubs), `docs/research/README.md`, seven session notes, the standing rule in CLAUDE.md. No row is `verified`. One coded constant found wrong against its own citation (IMP-042, complex crater depth). Stage 2 work packages are at the end of `docs/science/claims.md`. |
| SCI-2 / WP-0 | Science documentation, stage 2, work package 0: source library, de-duplicated bibliography and ledger, coverage gaps (log 2026-10-02) | done | 2026-10-02 | | Branch worktree-agent-a024f62a028c56ca4 (sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c), master 1eac09f merged in, not merged to master, not pushed. Docs only. 475 bibliography entries (+13 from flexure) became 385 works; 193 duplicate ledger rows retired (`merged`); 301 rows added in five new ledgers (GEO 51, PRS 74, HNT 84, CRE 35, CHK 57); the source library holds 104 readable works, 6 scans and 35 landing pages (not distributed). No row changed status except by retirement. |
## Review pages
| Name | URL | Version label | Date |
|---|---|---|---|
| hub (every review page in one place; update it when a page is added) | https://spacesheep.dev/@benmac/deeptime-hub | hub 2026-09-28 | 2026-09-28 |
| seams (where the wrap is, what crossing does, checked against TW 0.19.3 code; links the wrap graphics page) | https://spacesheep.dev/@benmac/deeptime-seams | wrap explainer 2026-09-28 | 2026-09-28 |
| wrap (how the east–west wrap works: 3D planet, server vs client views of a crossing, two players, a lap, Nether 1:8) | https://spacesheep.dev/@benmac/deeptime-wrap | how the wrap works 2026-09-28 | 2026-09-28 |
| phase0 (Phase 0 exit: every check, evidence, links) | https://spacesheep.dev/@benmac/deeptime-phase0 | Phase 0 exit 2026-09-28 | 2026-09-28 |
| p1-editor (real-client screenshots of the Create World editor, timelapse, review, world) | https://spacesheep.dev/@benmac/deeptime-p1-editor | P1-11 client screenshots 2026-09-28 | 2026-09-28 |
| p1-plates (500 Myr of plates, 100k samples, keyframes + events) | https://spacesheep.dev/@benmac/deeptime-p1-plates | P1-03 plates 100k 500 Myr 2026-09-28 | 2026-09-28 |
| p1-surface (500 Myr of plates + erosion, deposition, strata; 16k quick pipeline; PNGs losslessly recompressed to fit the 25 MB Free-plan deploy cap) | https://spacesheep.dev/@benmac/deeptime-p1-surface | P1-04 surface processes, 500 Myr 2026-09-28 | 2026-09-28 |
| p1-structure (P1-07: surface rock, bedrock, metamorphic grade, trap structure, plutons, six pole-to-pole cross-sections; trimmed to 23 MB) | https://spacesheep.dev/@benmac/deeptime-p1-structure | P1-07 rock classes and strata, 500 Myr 2026-09-29 | 2026-09-29 |
| p1-calib, p1-calib-super (P1-17 partial: 500 Myr on pipeline_quick_16k after recalibration with real climate; scattered continents / supercontinent) | https://spacesheep.dev/@benmac/deeptime-p1-calib · https://spacesheep.dev/@benmac/deeptime-p1-calib-super | calibrated 500 Myr 2026-09-29 | 2026-09-29 |
| p1-amplify (P1-06: before/after amplification of mountains, highlands, coast, river, lake, a tile edge and the seam; drama figure) | https://spacesheep.dev/@benmac/deeptime-p1-amplify | P1-06 before/after amplification 2026-09-29 | 2026-09-29 |
| p1-biomes (P1-09: Köppen beside biomes on earthlike_quick_16k 500 Myr, standalone and MIC; tag shares, top biomes per class, Chunky areas) | https://spacesheep.dev/@benmac/deeptime-p1-biomes | P1-09 biomes vs climate 2026-09-29 | 2026-09-29 |
| mountains (block-scale mountain character before/after: arc, young collision, old range; standalone + MIC; vanilla and Terralith comparisons; ruggedness sweep) | https://spacesheep.dev/@benmac/deeptime-mountains | mountain character before/after 2026-09-30 | 2026-09-30 |
| p1-deposits (deposit engine on synthetic fields, Why-here chains) | https://spacesheep.dev/@benmac/deeptime-p1-deposits | P1-14 first pass 2026-09-28 | 2026-09-28 |
| p1-deposits-real (P1-14: deposits on the real 500 Myr history; not published, review/out/p1-deposits-real on the laptop and the Mac, 17 MB) | — | P1-14 real history 2026-09-29 | 2026-09-29 |
| p0-bluemap (3D render on MIC blocks) | https://spacesheep.dev/@benmac/deeptime-p0-bluemap | P0-14 mountain render 2026-09-28 | 2026-09-28 |
| q10-ev (Q10 evidence: the same deposits in MIC without and with Excavated Variants; cut screenshots, per-host ore table, costs, trade-offs) | https://spacesheep.dev/@benmac/deeptime-q10-ev | Q10 evidence 2026-09-28 | 2026-09-28 |
| q10-map-plain (BlueMap, MIC without EV: 3 sites, surface + cut maps; linked from q10-ev) | https://spacesheep.dev/@benmac/deeptime-q10-map-plain | Q10 map without EV 2026-09-28 | 2026-09-28 |
| q10-map-ev (BlueMap, MIC with EV + the EV bridge fix; linked from q10-ev) | https://spacesheep.dev/@benmac/deeptime-q10-map-ev | Q10 map with EV, EV bridge fix (smooth basalt) 2026-09-28 | 2026-09-28 |

## Pending questions
Questions asked of Ben and not yet answered. Answers go into the decisions file's log
(docs/deeptime-design-decisions.md, last section), never here.

| Date | Task | Question | Asked how |
|---|---|---|---|
| 2026-10-02 | plate-motion task 2 | **Turn real shortening on before tasks 3–4?** `tectonics.collision_on_shortening` is off in every preset, so the planets are master's (build log 2026-10-02, "Plate-motion task 2"). On, on master with flexure (2 seeds): continental crust 0.29 → 0.36 of the planet, collisions per 100 Myr 11.3 → 7.4, plate lifetime 41 → 63 Myr, but one plate above half the planet in 45 % of steps against 41 %, and every collision still ends in the one-step surge. (a, recommended) Keep it off until tasks 3 and 4 land together and remove it; (b) turn it on now as a light interim gain on the crust and collision counts. Not asked through AskUserQuestion (sub-agent); for the coordinator to put to Ben. | coordinator |
| 2026-10-02 | plate-motion task 2 (found) | **Continental crust on master with flexure:** 1000 Myr `earthlike_quick_16k` runs on 1eac09f end with 0.29 of the planet continental (lowest step 0.28, plate-check target ≥ 0.37); on 38ca6b0 the same runs gave 0.42 (0.40). Not isolated between those commits (flexure, impacts, plate-check 1b). Recommended: bisect with 1–2 seeds before the belt calibration of task 3. | coordinator |
| 2026-10-02 | science WP-13a | **Slab suction share** (PLT-186; docs/design/plate-force-balance.md section 6 note). The design's `slab_suction_share` 0.3 pulls only the overriding plate; Conrad & Lithgow-Bertelloni 2002 and 2004 (abstracts) have suction act on both plates, at 40 % of the forces against pull's 60 % (30 % with a weak asthenosphere), so their suction ÷ pull is 0.43–0.67. (a) Keep 0.3 as the open-loop starting value and calibrate it in task 8, with 0.43–0.67 noted as the sources' range (recommended: task 5 records forces read-only and the term's meaning differs anyway); (b) set 0.5 now, before task 7; (c) change the design so suction acts on both plates, as in the sources. | not asked (overnight; recorded by the WP-13a agent) |
| 2026-10-02 | science WP-13a | **The overall speed scale** (PLT-181). `plate_speed_scale` 45 mm/yr was set to "Earth about 40", which is the median of plate-samples. Earth's area-weighted RMS is about 47 mm/yr today and 50–100 over the last 200 Myr (Forsyth & Uyeda 1975 Table 1; Zahirovic et al. 2015); the plate-check target is now 40–100. (a) Keep 45 for task 5 and calibrate it in task 8 against the new target (recommended); (b) raise it to 60 now. | not asked (overnight; recorded by the WP-13a agent) |
| 2026-10-02 | science WP-13a | **Sea-floor age ceiling** (CHK-019, PLT-161). Today's ocean floor has a 90th percentile age of 133 Myr (Seton et al. 2020 grid, area-weighted); the plate-check ceiling is 230 Myr (master measures 208–269). (a) Keep 230 until the rewrite is measured, then tighten to about 180 in task 8 (recommended); (b) tighten to 180 now (master fails it); (c) keep 230 for good, as a cycle-long allowance. | not asked (overnight; recorded by the WP-13a agent) |
| 2026-10-02 | science WP-13a | **Collision length target** (CHK-012, PLT-154). The target median is 15–50 Myr; Earth's examples are longer: India–Asia 50 Myr and continuing, the Alleghanian 65 Myr, the Uralian 70–140 Myr (Wikipedia; no statistic exists). (a) Keep 15–50 until the rewrite's collisions are measured, then decide with numbers (recommended: the model's collisions include small terranes); (b) raise the top to 100 Myr now. | not asked (overnight; recorded by the WP-13a agent) |
| 2026-10-01 | P2-14 | **Where geothermal features come from beyond the volcanoes.** Today only the active volcanoes (the bake's one export of heat); on the 16k quick planet that is 260 sites around 40 of 67 volcanoes, 23 of them hot-spring pools (median 1,422 blocks from a land point to the nearest pool, 837 to any site, 766 to a volcano). Options: **(1, recommended)** export a heat-flow layer (the plate model already computes `heat_flow_mw_m2` per sample: 55 + 35 e^(-t/60) on continents, +40 arc, +40 hotspot, +30 divergent) as an L1 `meta` layer and put fields on continental crust above a threshold (rifts, arcs, young orogens), same site machinery; **(2)** export ridge and rift segments; **(3)** volcanoes only. Rift provinces are in `meta/province` but absorbed at 50,000 km², so none exist on a typical planet. | Not asked yet: for the coordinator to put to Ben |
| 2026-10-01 | P2-14 | **Default density and the warm-ground signal.** (a) Keep `geothermal_density` 1.0 (recommended; the default is a few fields per volcano, at 2.0 the sites roughly double, not measured) or raise it? (b) `DeepTimeClimate.geothermalWarmthC` (API 3, up to 12 K at a vent) exists but mic-climate does not read it: wire it as a warm refuge only, after the mic-climate connector (recommended), as a heat source for machines too, or leave it unread? (c) Native sulfur and cinnabar are vanilla stand-ins (yellow concrete, red terracotta); MIC has no such blocks, so the pack needs nothing now (recommended), or add blocks to `deeptime:geothermal/sulfur` / `cinnabar` in the pack later. | Not asked yet (sub-agent report) |
| 2026-10-01 | deposit audit | **How rich should deposits be, against scatter and vanilla?** Three options with settings and numbers per family (A deposits clearly beat scatter, recommended; B deposits are the only real source; C realistic grades with strong multipliers), in the sub-agent's report and the build log entry "Deposit audit". Also: **a gold-bearing gravel block of Deep Time's own** for packs without Excavated Variants (it would be the mod's first block), or keep `minecraft:gold_ore` in the gravel bed there? No default changes until answered. | to be asked by the coordinator (the sub-agent has no AskUserQuestion) |
| 2026-10-01 | P1-23 | **Four grid artifacts that need a design change** (build log 2026-10-01, "P1-23 grid-artifact audit"; docs/research/notes/grid-artifact-audit.md, "What is left"). **(1) Lake sediment** builds straight dams across lake plains: fill each lake level, deepest first (recommended, 2–3 days), a delta from each inlet, or hide it in the amplifier? **(2) Rivers and valleys run along eight directions**: route by a direction not bound to eight in the amplifier and the river network (recommended, about 3 days, changes every valley), meander the river lines only (a day), or leave for Phase 2? **(3) Biome borders step with the L1 cells**: decide the Köppen class on L2 inside P1-26 (recommended), jitter the lookup now (half a day), or leave? **(4) The detail noise's octaves share one lattice** (5 % of the block-scale relief): offset each octave when the format next changes (recommended), gradient noise, or leave? | Not asked yet: for the coordinator to put to Ben |
| 2026-09-28 | P1-16 | **COE vein size:** finite, tonnage-sized veins (`defaultInfinite = false` on MIC's deep-time pack branch) or keep COE's infinite default? Ben (log 2026-09-28): decide later, after P1-08, with numbers from the real presets. | AskUserQuestion, answered "Decide later" |
| 2026-10-01 | world height / mountains | After stage 2 (build log 2026-10-01, "The height limit, stage 2"): **(1) the bottom:** default trenches reach y −412 to −472 against `min_y` −352: lower `min_y` to −512 (+16 % sections; recommended), squeeze the bottom band only, or leave flat trench floors? **(2) summits:** Earth-like worlds top out at 6.0–6.9 km (plateaus at 5–5.7 km, little summit relief above them): give the amplifier summit relief over plateaus (recommended) or raise `crust_strength_mpa` to ~190? **(3) light planets** get taller volcanoes but not taller fold belts, since erosion and uplift do not know gravity: leave, or make them? **(4) the top** stays at y 639 (default worlds end 166 blocks under the compression knee) unless Mars-like worlds matter now (then `max_y` 1023, +39 % sections). | Not asked yet: for the coordinator to put to Ben |
| 2026-10-01 | lava | **How far a player is from lava** (build log 2026-10-01, "Magma under active volcanoes"). With lava only at active volcanoes and the world's floor, on the 16k quick planet measured (123 active volcanoes, 100 on land, 16 crater lakes) a land point is a median 658 blocks (p90 1,309) from the nearest active volcano on land, where lava lies six blocks under the crater's magma plug, a median 1,178 blocks (p90 2,251) from the nearest open crater lake, and 438 blocks (p90 510) straight down from the lava floor. Vanilla has lava within about 100 blocks of anywhere. Options: **(1, recommended)** keep it: lava is a place to travel to, which is what the decision asked for (what MIC's early recipes need lava for was not checked); **(2)** a crater lake at every active volcano on land (`crater_lake_share` 1: about 62 of the 100 can hold one, four times today's 16); **(3)** a second, shallower source: lava in the deep cave aquifers again (`lava_lake_blocks` raised, e.g. 150: lava lakes in caves below about y -200); **(4)** keep the biomes' random lava (`remove_lava_features` off): vanilla's amount, geologically arbitrary. Also open: **the lava floor** at the bottom of the world (`lava_floor_blocks` 10; Ben has not decided whether to keep it), and **spreading ridges and rifts**, which the bake does not export (no lava there yet). | Not asked yet: for the coordinator to put to Ben |
| 2026-10-01 | P3-04 | **Toroidal World refuses Deep Time's Distant Horizons generator from DH 3.3.1 on** (it works with MIC's 3.3.0 only because TW's DH mixins miss that jar's renamed classes). (a) Keep MIC on DH 3.3.0 for now and ask TW's author for a way to declare a seam-aware generator before any DH update (recommended); (b) a Deep Time mixin into TW's `DhGenerators.requireOwn` that lets our generator through (works today, breaks when TW changes, and overrides a guard TW put there on purpose); (c) accept that a DH update turns far terrain off and DH generates from real chunks. | Not asked yet (sub-agent report) |
| 2026-10-01 | P3-04 | **Far terrain's top blocks.** A table by biome gives 85–96 % of ground blocks right on vanilla biomes and 55 % in MIC (Terralith). (a) A MIC table for Terralith's biomes on the pack's deep-time branch, measured with `probe dh check` (recommended: cheap, pack-side); (b) run the pack's real surface rules over a throwaway chunk per far chunk (exact soil, est. 5–15 ms more a section, touches vanilla chunk classes other mods mix into); (c) leave it. | Not asked yet (sub-agent report) |
| 2026-10-01 | P3-04 | **Trees in far terrain.** Far terrain has none, so a forest reads as grass until real chunks replace the LODs. (a) Leave it until a client can show how it looks (recommended); (b) add a canopy layer by biome (leaf block, cover share and height in the surface table). | Not asked yet (sub-agent report) |
| 2026-10-02 | force-balance plate motion (design) | Design: docs/design/plate-force-balance.md (build log 2026-10-02, "Force-balance plate motion: design"); the four questions of the collisions design were answered on 2026-10-02 (decisions log). **(1) What moves a plate nothing pulls, and a planet that starts with no slabs?** (a, recommended) ridge push, suction and neighbours plus a weak mantle flow (5 mm/yr, a setting, a stand-in) and slabs planted by subduction initiation; (b) no mantle flow: such plates stand still; (c) keep a base speed. **(2) The speed guard:** (a, recommended) keep `max_plate_speed_mm_yr` at 250 as a reported guard that must never bind; (b) remove it. **(3) How the work lands:** (a, recommended) two stages: collisions and belts on the present speed law first (a usable state and the fallback), then the switch to the balance; (b) one stage, no fallback, two days saved. **(4) Flexure:** (a, recommended) the line-load profile in front of collision wedges only; (b) a general flexure solve in the surface stage now (5–10 % of run time, 3–4 more days). **(5) Calibration:** (a, recommended) the built-in planets change twice, with a light calibration at the first; (b) full calibration at both points (3 more days); (c) hold everything on a branch until calibrated. Nothing is built beyond task 1 (`plate-check`) until answered. | Not asked yet: for the coordinator to put to Ben |
| 2026-10-01 | P3-04 | **The switch.** Far terrain is on whenever Distant Horizons is installed; off with `-Ddeeptime.dh=false`. Make it a setting (server config or the editor), and on or off by default? Recommended: on by default, a server config key when the config file next changes. | Not asked yet (sub-agent report) |
| 2026-10-01 | P2-08 impacts | **How often should meteors strike by default?** (build log 2026-10-01, "Meteor impacts"). At Earth's rate on the planet's own area (`impacts.flux` 1), the quick 16k planet forms 567 craters in 500 Myr and shows 19 at the present, 7 of them dry bowls on land, on 8,192 × 16,384 blocks: a player may never meet one. (a) Keep Earth's rate, which is what the decision asked for (recommended; the editor's Terrain section has the slider, "Heavy" is 10); (b) default to about 5 so every continent has a few craters. | Not asked yet (sub-agent report) |
| 2026-10-01 | P2-08 impacts | **How deep should craters be drawn?** Blocks are 15 times taller than wide, so a real 2-km bowl is 7 blocks across and 21 deep. `impacts.depth_scale` defaults to 0.5 (11 blocks), following "Setting, gentler default" for volcano flanks (log 2026-09-30). (a) Keep 0.5 (recommended); (b) physical depth (1); (c) gentler still (0.35). | Not asked yet |
| 2026-10-01 | P2-08 impacts | **May a run make a giant crater?** `impacts.max_crater_km` is 300 (the largest known on Earth, Vredefort): a crater of 200–300 km, 700–1,000 blocks across, has about a 1 in 6 chance in a 500-Myr run at 16k; anything larger would melt and resurface much of so small a planet and is not simulated. (a) Keep 300 (recommended); (b) cap lower on the 16k presets (for example 150 km); (c) no cap, with the planet-wide effects modelled as a later task. | Not asked yet |
| 2026-10-01 | P2-08 impacts | **Two new rock classes and their stand-ins.** `impact_breccia` (stand-in cobblestone) and `impact_melt` (stand-in smooth basalt) are the 61st and 62nd classes; the MIC rock tags need their rows (listed in the build log). (a) Keep both with these stand-ins (recommended); (b) other blocks from the pack; (c) fold melt rock into an existing class and keep only the breccia. | Not asked yet |
| 2026-10-01 | P2-08 impacts | **Impact deposits do not claim their materials.** Diamonds, nickel, platinum and uranium are products of the impact deposit types, and a claimed material loses its scattered ore in a Deep Time world; impact deposits are far too rare to be the only source, so the four types carry `minecraft.claims_scatter: false` and vanilla diamonds and the pack's nickel, platinum and uranium ores keep generating. (a) Keep it so (recommended); (b) claim them and add background scatter for each. Their tonnages, grades and densities are design values (no USGS model exists) and belong to the deposit audit's richness question. | Not asked yet |
| 2026-10-01 | P1-10 | **The orogen share (Ben: "revisit the orogen share in P1-10").** On `earthlike_quick_16k` (500 Myr, scattered start, seed 1, this tree) the USGS rules class 0.42 of the continents as orogen (Earth ~0.17: orogens 12 % + arcs 5 %). 0.125 of it is the plate model's own active collision and arc regime (its 800-km back-arc halo), which no province threshold moves; the rest is collision belts touched in the last 500 Myr (0.166) and Andean ranges (0.134). Options: (a) keep the thresholds (500 / 50 Myr, 1,500 m) and treat the share as the plate model's, to fix with collision frequency and the regime halo in a P1-27 follow-up (recommended: the provinces then show what the model produced, and nothing changes for existing presets); (b) set `provinces.orogen_max_age_myr` 200, `andean_max_age_myr` 25, `andean_min_elevation_m` 2,500 in the built-in presets (share 0.33, a quarter fewer orogen provinces, no plate-model change); (c) 50 Myr / 10 Myr (0.24). Table: `tools/review/p1-10-orogen.sh`, build log P1-10.  **Update 2026-10-02 (master 1eac09f, flexure on, same preset and start, seed 1):** the classified orogen share is now 0.245 of the continents (active regime 0.118, belts 0.042, Andean 0.060); thresholds 200 / 25 / 2,500 take it to 0.185; option (a) is now even more clearly right. Large igneous provinces are 0.10 of the continents, probably high. See `docs/science/provinces.md`. | Not asked yet (sub-agent report) |
| 2026-10-01 | P1-10 | **Region size and when a world gets its regions.** (1) Provinces are at least 50,000 km² (the earlier bake minimum, kept; `provinces.min_area_km2`): the 16k planet has 40 provinces of 68,000 to 2,200,000 km² (about 870 to 4,900 blocks across), TerraBlender's own regions being about 1,000 to 3,000 blocks; smaller values give more, smaller regions (recommended: keep). (2) A world's province → region table is dealt once, at the first start with TerraBlender installed, and kept: a biome mod added later brings its regions to new worlds only, and one removed falls back to the vanilla region for its provinces (recommended; the alternative re-deals for unexplored chunks and shifts biomes at the border of explored ones). | Not asked yet (sub-agent report) |

## Build log
### 2026-10-02 — Plate-motion task 2: real shortening and the collision state (agent worktree agent-aaf46227fde85f19a, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c; not merged)
- **What changed in behaviour: nothing in the built-in planets.** Every built-in preset gets
  `tectonics.collision_on_shortening: false`, which keeps the trigger used before this task, so the worlds are master's.
  Measured: plate-check on seeds 1–4 gives the same value for every measure that does not depend on when a collision
  starts, to the last digit, seed by seed. The hypsometry run checked against master is bit-identical too (below).
  The collision state, the real-shortening count and their records run alongside and decide nothing. Setting the
  switch to `true` makes real shortening fire transfers and welds.
- **Real shortening** (`plates/shortening.rs`). A contact's **collision front** is the stretch of the overriding plate's
  edge where continental crust of the subducting plate stands now (the nearest sample of another plate within 1.75
  spacings is that plate's, continental, not freshly sutured). The **convergence across it** is the flux of the relative
  velocity through the edge's Voronoi edges (cotangent lengths), divided by the front's length (the summed length ×
  outward-normal vector, and at least half the summed edge). On a straight front this is the speed across the boundary.
  Pure strike-slip gives under 2 % of the speed, an oblique contact its normal part, and a parting one a negative
  value. A first version used each edge sample's mean normal and read 19 % of a strike-slip speed as convergence,
  because the Fibonacci lattice's normals lean one way along a boundary. **Shortening** is that convergence integrated
  over the collision (parting steps add nothing), and the crust crossed is shortening × front length (km²).
- **The collision state** (`state::Collision`, on `Contact::collision`). It begins on the first step a live contact's
  front has 3 or more samples and converges at 10 mm/yr or more. That is `subduction::CONFIRM_MM_YR`; counted from
  above zero, the onset convergence was a few mm/yr and the 20-Myr slowdown ratio a median of 3–8. The state carries
  its onset, last step, shortening, crust crossed, convergence now, at onset, peak and 20 Myr after onset, the front's
  length and peak length, its centroid and outward normal, and the share of the front that is continental on the
  overriding side. It survives 3 steps with no front, since a resampling consumes the overlap and it comes back. It
  ends in one of five ways. A transfer or weld writes a `CollisionRec`, whose `start_myr` is now the state's onset and
  `end_myr` the transfer, plus a `BeltRec` with the convergence at onset and 20 Myr later. The other four write a new
  `PartingRec`: parted (no front for over 3 steps), reversed, closed (the contact timed out or a plate died), emptied
  (the terrane was gone) or open (the run ended). The `collision` event gains `onset_myr` and `shortening_km` (EVENTS.md).
  `Counters::collision_onsets` counts states, and the run test checks that every state ends exactly once.
- **The `depth_km` bug** that the plate-check agent noted is real: it added the *magnitude* of the relative velocity at
  the overlap's centroid, so sliding plates counted in full. `depth_km` is gone. The real count is above, and the old
  count is kept on purpose, unchanged, as `Contact::speed_count_km` (it fires while the switch is off). In the default
  runs it reads 351 km at the transfer or weld while the front had shortened by a median of 167 km (p90 320).
- **Interim weld and uplift: the old trigger still fires, by measurement.** The new setting is
  `tectonics.collision_on_shortening` (bool): an editor toggle `collision_on_shortening` and lang key
  `deeptime.editor.field.collision_on_shortening`, in all 13 presets and the test planet. Task 4 removes it together
  with the old count. Off is what master did. On fires at 300 km of real shortening: collisions then last 10 Myr
  instead of one step (p90 21), but the one-step surge still fires at the end. Measured, on master 38ca6b0 (before
  flexure, 4 seeds): one plate above half the planet in 32 % of steps (59 % today), welded back 26 % (42 %), and
  continental crust at the end 0.38 (0.42), lowest step 0.37 (0.40). Thresholds of 100–250 km (1–2 seeds each) do not
  help the mega-plate (53–72 % of steps). On master with flexure (2 seeds) the picture changes; see the table and the
  pending question.
- **plate-check and baseline** (`earthlike_quick_16k`, supercontinent, 1000 Myr, `stage volcanoes`, 2 threads; before =
  master 1eac09f, after = this branch on 1eac09f, on = after with `collision_on_shortening=true`; seeds 1–4, on: 1–2):

  The full table (every measure, per seed) is in docs/design/plate-check-baseline.md, "Task 2". The measures that
  move, and the main ones (mean of seeds; before = after unless shown):

  | Measure | Before (= after, switch off) | After, where it differs | Switch on (2 seeds) | Target |
  |---|---|---|---|---|
  | Largest plate, median share | 0.45 | | 0.46 | 0.15–0.35 |
  | Steps with one plate above 50 % | 40.8 % | | 44.7 % | ≤ 10 % |
  | Plates above 5 %, median | 4.25 | | 3.5 | 5–9 |
  | Median plate lifetime | 41.5 Myr | | 63 Myr | ≥ 80 |
  | Plates dead within 20 Myr | 34.6 % | | 21.2 % | ≤ 15 % |
  | Rift-born alive after 50 Myr / welded back within 50 Myr | 24.1 % / 31.8 % | | 23.6 % / 29.6 % | ≥ 60 / ≤ 10 |
  | Land raised by one collision in a step, most / most land gained in a step | 2.37 % / 2.34 % | | 1.29 % / 1.52 % | ≤ 0.5 / ≤ 1.5 |
  | Collision belt width / elongation, median | 2,604 km / 2.05 | | 2,519 km / 2.30 | ≤ 800 / ≥ 4 |
  | Collision length, median / 90th pct | 2 / 2 Myr | 6 / 10.9 Myr | 10 / 20.8 Myr | 15–50 / ≤ 120 |
  | Convergence 20 Myr after onset ÷ at onset | no data | 1.30 (few collisions last 20 Myr) | 0.69 | 0.2–0.6 |
  | Collisions per 100 Myr | 11.3 | | 7.4 | 2–6 |
  | Continental crust, lowest / highest step | 0.28 / 0.45 | | 0.35 / 0.46 | ≥ 0.37 / ≤ 0.45 |
  | Continental crust at the end | 0.29 | | 0.36 | – |
  | Active margin share | 0.31 | | 0.34 | 0.25–0.40 |
  | Sea floor, 90th pct age (mean after 100 Myr) | 194 Myr | | 182 Myr | ≤ 230 |
  | Plate speed, RMS / median | 59.6 / 72.3 mm/yr | | 57.6 / 72.8 | 35–55 / 30–60 |
  | Contact age when the collision began, median | 7 Myr | 3.25 Myr | 2 Myr | – |
  | What fires (old count) at the transfer or weld, median | 351 km | | 568 km | – |
  | Shortening across the front at the transfer or weld, median / p90 | – | 167 / 320 km | 334 / 386 km | – |
  | Collisions with no state (no converging continental front) | – | 13.3 % | 0 % | – |
  | States ending with no transfer, per 100 Myr (parted / reversed / closed) | – | 26.4 (7.3 / 2.7 / 15.9) | 37.6 (17.5 / 3.2 / 15.9) | – |

- **Hypsometry at 500 Myr** (`p1-17-hyps` lines, seeds 1–3 × both starts, mean [min, max]; the same builds):

  | Measure | Before | After (switch off) | Switch on | Target |
  |---|---|---|---|---|
  | Land share | 0.295 [0.252, 0.327] | same | 0.315 [0.286, 0.365] | 0.27–0.30 |
  | Median land height | 687 m [569, 832] | same | 732 m [603, 933] | 430–640 m |
  | Land above 2 km | 0.124 [0.092, 0.149] | same | 0.119 [0.079, 0.178] | 0.10–0.13 |
  | Land, 90th percentile | 2,181 m | same | 2,130 m | – |
  | Continental crust | 0.407 | same | 0.418 | – |
  | Sea depth, median | 3,903 m | same | 3,814 m | – |

  "same": the after run of seed 1, supercontinent, on the branch merged with master 412d517 printed the same
  hypsometry, climate and plate lines as master 1eac09f's. The other five were not re-run; plate-check
  identity on four 1000 Myr seeds covers them. Master's own median land height (687 m) is above the target; that
  is not this task's. Highest L1 cell and the 99th percentile are not in the summary line and were not taken.

- **Found on the way, not mine:** master with flexure on (1eac09f) ends 1000 Myr runs with continental crust at 0.29
  of the planet (lowest step 0.28; plate-check fails `continental_min` at 0.37). On 38ca6b0 the same four runs gave
  0.42 (lowest 0.40). The cause is not isolated between those commits (flexure, impacts, plate-check 1b); flexure
  lowering young arcs below the sea before they turn continental is a guess. The switch at `true` keeps 0.36.
  Separately, the structure stage's Noddy-check panic on seed 1 is fixed on master (my own guard was dropped).
- **Ledger**: PLT-068 rewritten; PLT-222 to PLT-227 new (flux convergence, front rule, state, grace steps, 20-Myr
  measure, the switch); ids beside the constants; claims.md and plates.md counts.
- **Gates** (VM, branch at 826b912 with master 412d517): cargo fmt --check clean, clippy -D warnings clean,
  `cargo test --workspace` 689 passed / 0 failed / 3 ignored, `cargo deny check licenses bans` ok, `./gradlew build
  test` BUILD SUCCESSFUL, JUnit tests=226 failures=0 skipped=1. No GameTests: the game reads nothing new except the
  two collision event params and the larger `notes.plates.dynamics`.
- **Tests added**: `plates::shortening::tests` — normal component; head-on accumulates speed × time (within 2 %);
  strike-slip accumulates none (under 2 %); oblique counts its normal part; the front rule; the life cycle (onset,
  persistence through steps with no front, parting steps add nothing, the 10 mm/yr start, the 20-Myr value, parting
  after the grace steps); the threshold fires on real shortening and a parting is recorded; the old count fires while
  the switch is off; Voronoi cells close and tile the sphere. `plates::tests`: the life-cycle checks on the 300 Myr
  recording run (every state ends once, shortening never falls, a belt record per collision with a state), and
  `collisions_on_real_shortening_fire_at_the_threshold`.
- **Not verified**: the full presets (250,000 samples); the 1–2-seed sweeps are indicative only. With the switch off
  the 20-Myr slowdown ratio rests on a handful of collisions per run (most fire within 6–10 Myr), so its value
  (1.3, spread −0.35 to 2.8) means nothing yet.
- **Next**: tasks 3 and 4, landed together. Task 3 stacks each step's `area_km2` growth behind the front's `centroid`
  along its `normal`, on both plates, and fills `BeltRec`'s wedge and foredeep. Task 4 replaces `shortening::advance`'s
  trigger with welding on stalled convergence (`Collision::convergence_mm_yr`) and removes `collision_on_shortening`
  and `speed_count_km`. Task 5 starts break-off at `onset_myr`.
### 2026-10-02 — Flexure follow-up: the margin zigzag and the polar stripes, causes and fixes (agent worktree agent-afae960b6db068f94, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c; not merged)
- **Asked.** The coordinator, after flexure landed (5ce5af4): find the cause of the cell-to-cell alternation along
  active margins and the rise in land roughness (99th percentile 294 → 391 m in one pair), and of the stripes near
  the poles, show each with a controlled test, fix it there, re-measure on the merged tree. Ben's answers to the
  five flexure questions are in the decisions log (2026-10-02, logged in 4918af3).
- **Measured how.** A temporary diagnostic split each step's flexure response linearly by source (solving the same
  operator for each part of the residual) and counted cell-scale zigzags (a cell and its next neighbour both strict
  extrema of opposite kind, by more than 5 m a step) at margins (within three cells of a plate boundary) and
  elsewhere; `tools/review/flexure_artifacts.py` does the same on a run's final fields (20 m threshold, margins,
  interiors, poleward of 70°). Design note `docs/design/flexure.md` section 13 has the whole account.
- **Cause 1, the main one: the carried state acted as a force.** Every step re-solved the whole load against the
  deflection and frozen moment carried from the last step. Carrying them on the raster interpolates them (near the
  pole a plate's rotation shifts rows by many cells); new crust and arriving crust restart at zero beside cells that
  carry kilometres. In a whole-load solve that mismatch is a residual, and the solve moved the ground to remove it:
  **up to 4.6–6.2 km in a step from the carried state alone** (200 Myr, scattered seed 1), split about equally
  between restarted crust, cells whose stiffness jumped fourfold, and interpolation. Controlled test
  `carrying_a_bent_plate_moves_nothing`: the old formulation moves a carried, unloaded bent plate by 1.15 m a step,
  the new one by under 1 cm. **Fix:** each step solves only for the step's load increments; the carried plate is in
  equilibrium; the solve's unfinished residual is carried and solved with the next step; the plate carries its
  bending moment (kept when it stiffens, released as a force when it weakens). Ledger ISO-068, ISO-056 changed.
- **Cause 2: the plate-boundary break was a staircase.** Applied after the stiffness smoothing, it stepped with the
  raster. Controlled test `a_moving_margin_bends_smoothly` (an ocean plate moving 0.37 cells a step under a craton
  advancing 0.23 cells a step along a wavy boundary, a sediment fan, an arc): with the staircase the deflection
  zigzags beside the boundary by 3–70 m; fixed, nowhere by more than 10 m. It was the polar artifact: polar-band
  zigzag 3.5 % [1.4, 5.0] with the staircase, 0.87 % [0.49, 1.27] fixed, 1.48 % with flexure off (four planets).
  **Fix:** the boundary cells are blurred and the stiffness share taken from that (0 on the boundary rows, then 0.17,
  0.72). Ledger ISO-055 changed.
- **Cause 3: the plate model's sample lattice reached the ground.** Its uplift is interpolated from samples 71
  Earth-normalized km apart onto 39 km cells; the uplift field itself zigzags in 4.8 % of margin cells by more than
  5 m a step (aliasing). Local isostasy passed that at its own height; a stiff plate passed the thickening behind
  it, 5.5 times higher. Controlled test `the_plate_models_sample_lattice_is_not_a_load`: a 500 m column-by-column
  uplift on a craton now steps the ground by 500 m (over 1,500 m unfiltered). **Fix:** only the part of the plate
  model's root that it resolves bends the plate (binomial (1,2,1)/4 passes set by the sample spacing, two on the
  quick presets); erosion, deposition and spreading are not filtered. Per step, the uplift part's margin zigzag fell
  from 4.2–6.0 % to 1.3–3.6 %. Ledger ISO-069 (new), `StepIn::plate_spacing_m`.
- **Ruled out:** the preconditioner's leftover residual (its response zigzags in under 0.1 % of cells); latitude
  terms in the operator or preconditioner (none: block metric throughout; a ring load bends the plate the same at
  every longitude, tested). The regime field is populated on resampling steps, unlike `plates/flags`.
- **What remains at margins** (per step, final code): erosion and deposition 1.6–4.1 % of margin cells (marine
  fill of kilometres in trench cells: real at this resolution), uplift with water and ice 1.3–3.6 %, moment released
  where a moving boundary weakens the plate 1.0–3.1 %. On the final fields the worst margin window of each planet,
  with and without flexure, is old ocean floor whose own local-isostatic level wiggles by ±50 m.
- **Re-measured on the merged tree** (master 412d517; quick preset, test VM):
  - Artifacts, 200 Myr, seeds 1–2 × both starts: elevation zigzag at margins 11.1 % off, 13.1 % final (first build,
    one planet, 19.3 % against 10.8 % off); polar band 1.48 % off, 0.87 % final; land roughness p99 237 m off,
    274 m final.
  - **Roughness:** on the pair of the first complaint (500 Myr, supercontinent seed 1) land p99 289 → 372 m, but land
    away from coasts gets smoother (271 → 225 m; at margins 563 → 451 m). The rise is coast: coast cells at plate
    boundaries double (1,378 → 2,830) because arcs held up by the stiff plate edge emerge as islands; 85 % of the
    roughest 1 % of land cells lie at margins (19 % without), a tenth of them zigzag.
  - **Run time** (that pair, side by side): 557 → 576 s, **+3.3 %**; flexure step 56 s, 9.3 iterations a solve (was
    85–107 s, 20–24), none at the limit.
  - **Hypsometry** (500 Myr, seeds 1–2 × both starts, `tools/review/p1-17-hyps.sh`): without / with: land 0.299 /
    0.290, median 578 / 636 m, p90 1,927 / 1,997 m, above 2 km 0.097 / 0.102, sea median 3,408 / 4,053 m (Earth
    4,062), shelf share of the sea 0.115 / 0.059, highest 1 % of land 4.0–4.8 / 3.0–3.4 km, highest cell 6.8 /
    5.0–5.3 km. Not recalibrated.
- **Which oceanic law:** every measurement in this entry used 2.7 km/√Myr (ISO-048, verified). The first build's
  timing pairs, the 1,000 Myr pair and its island and foredeep pictures used 3.0; its final hypsometry ensemble 2.7.
- **Not explained:** a few cells per planet stand kilometres below their local-isostatic level (−11.8 km at one
  polar trench in the final build; −6.2 and −8.8 km on the first build's planets); share of the planet more than
  500 m below: 0.55 % final, 0.71 % first build.
- **Pictures** (`review/out/flexure/`, not committed; `tools/review/flexure-views.sh`): `scattered-200-old` (first
  build) and `scattered-200-final`, `super-500-on` and `super-500-off`, each with `planet`, `pole`, `margin`
  (the window with most zigzag cells along a plate boundary), `moat`, `belt` and `numbers.json`. The first build's
  pole shows swept arcs of ±50–100 m across the polar continent (the carried state rotating about the pole); the
  final one has none, only bands along coasts and trenches.
- **Tools:** `tools/review/flexure_artifacts.py`, `flexure-compare.sh`, `flexure-ensemble.sh`; `flexure_views.py` gains
  `pole.png` and `margin.png`.
- **Gates** (VM, the merged tree): `cargo fmt --all -- --check` (after formatting), `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo deny check licenses bans` clean; `cargo test --workspace --no-fail-fast` all
  pass (core 481 + 3 ignored, cli 54); `./gradlew build test` tests=226 failures=0 skipped=1. No GameTests.
- **Next.** The calibration task after the plate work (hypsometry moved as above). The deep negative cells at
  trenches, if they matter in the game.

### 2026-10-02 — `plate-check`: the boundary mix and the arc heights (agent worktree agent-a8f59d60569b27473, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c; not merged)
- **Why.** Ben saw, 28 Myr after a supercontinent start, nearly every boundary between ocean plates as subduction with a
  continuous high arc; chose "Measure now, fix in the plate track" (decisions log 2026-10-02). Measurement only: no plate,
  collision, surface or volcano rule changed; `subduction.rs` and `collision.rs` not touched.
- **What.** `plates/dynamics.rs` (record version 3): per step `StepRec::mix_km` (boundary length by kind, km:
  ocean-ocean and ocean-continent subduction, collision, ridge, rift, other; `classify_boundary`, `boundary_mix`) and
  `arc_oceanic`/`arc_continental` (`ArcStat` `[cells, emergent, p50, p90, max]` of the elevation of the `ARC`-flag cells,
  `arc_stats`). The mix repeats `fronts_and_uplift`'s relative-motion test, called right after it (the sample flags are
  cleared by a resampling, so they cannot be read at the end of the step); arcs are read after `finish_samples`.
  `record_step` now takes an `Observed` (fronts, mix, arcs) so `mod.rs` passes three more values. `plates/check.rs`: 5 judged
  rows (`waits`-style, so a run without the data says "no data" and does not fail): subducting share 15-25 %, oceanic arc
  cells above sea level 10-35 %, oceanic arc crest -1,700 to 1,000 m, continental arc median 1,200-4,500 m and crest
  2,000-6,000 m; 23 reported rows (the rest of the mix, the first 50 Myr, lengths in km against Bird's, cells per step);
  new definition `early_myr` (50). Ledger PLT-213 to PLT-221, bibliography section "Plate-check boundary mix and arc
  heights", page `docs/science/plates.md`, baseline `docs/design/plate-check-baseline.md`.
- **Earth sources, as read.** Bird 2003 Table 3 (full text PDF, read): subduction 51,310 of 260,487 km = 19.7 %, spreading
  36.4 %, transform 28.4 %, continental convergent 8.8 %, oceanic convergent 6.7 % (PLT-213 `verified`). Hildreth 2007 p. 3
  and Gregory-Wodzicki 2000 (full texts, read): Cascade basement 1.2-2 km, a dozen cones above 3 km; Andean arc on a 4.5 km
  plateau (PLT-218 `verified`). Baker et al. 2008, de Ronde et al. 2003, Baker et al. 2003: abstracts only (PLT-215, -216
  `abstract-only`). `from-memory`: the 1 km crest of island arcs (PLT-217) and the 6 km Andean crest (PLT-219). Range widths
  are the agent's (`game-choice`, PLT-214).
- **Measured** (test VM, master 1eac09f + this branch, `earthlike_quick_16k`, supercontinent, 1000 Myr, seeds 1-4; seed 1 with
  `structure.deformation.density=0` for the structure-stage panic): subducting share **55.9 %** (55.5 54.9 56.9 56.4) against
  Earth's 19.7 %: FAIL; 191,000 km a step against 51,310 (total boundary 341,000 against 260,487 km); ocean-ocean 50 % of it
  (63 % in the first 50 Myr, when the subducting share is already 50 %); spreading 39.5 % (Earth 36.4), collision 3.2 %
  (8.8), other 1.3 % (transform 28.4 + oceanic convergent 6.7: the model's sign-only tests leave no strike-slip class).
  Oceanic arc cells above sea level 14.4 % (pass), crest 312 m (pass), median -2,146 m (reported); continental arc median
  1,670 m and crest 2,874 m (pass). So in the plate model's own elevation the arcs are inside Earth's ranges; the snow-high
  arcs Ben saw are not in this record (perhaps the volcano stage's cones or the surface stage's amplification; not
  checked). The check now meets 13 of 41 ranged targets. The old rows moved because master now has general flexure on
  (largest plate median 0.58 to 0.45, continental crust lowest step 0.40 to 0.29 so that target now fails, tenure 45 to 34
  Myr, collisions per 100 Myr 10.0 to 11.3, ...; list in the baseline doc): not this branch's doing.
- **Cost** (same 300 Myr plates run with and without the whole record, at once): +1.6 % (seed 1: 80.1 vs 78.8 s) and +2.6 %
  (seed 2: 91.5 vs 89.2 s) of the plates stage; task 1b's record alone was within 0.5 %, so the new measures are about 1-2 %
  of the plates stage (10-15 ms a step), about 0.5 % of a run. Record +16 numbers a step (about 50 KB of JSON; seed 1 1.05 MB).
- **Gates** (VM): cargo fmt --check clean, clippy --workspace --all-targets -D warnings clean, `cargo test --workspace`
  678 passed / 0 failed / 3 ignored (deeptime-core 478; new: boundary classification, arc statistics, record round trip, the
  mix and arcs in `plate-check` and the mix compared with the recorded fronts on a 300 Myr run, plus the existing test that
  recording does not change the world), `cargo deny check licenses bans` ok, `./gradlew build test` BUILD SUCCESSFUL, JUnit
  tests=217 failures=0 skipped=1. No GameTests (nothing the game reads changed except the size of `notes.plates.dynamics`).
- **Not verified.** The boundary mix is a count of lattice samples, not a surveyed length, and it repeats rather than shares
  the fronts' code (a test ties its fronts to the recorded `front_km` except in steps where a plate died). How much of the
  subduction surplus is the missing strike-slip tolerance (Bird: +-20 degrees) and how much real was not separated. The
  oceanic/continental arc split by the arc accumulators (PLT-220) was not compared with a split by the overriding plate's
  crust. Earth's island-arc emergent share rests on abstracts and on volcano counts, not on arc area. Full presets were not
  run (the mix is one more pass over the lattice edges, like the fronts').
- **Next.** The force-balance and belt tasks are judged on `boundary_subduction_pct` (and the arc rows). A strike-slip
  tolerance for the mix (the +-20 degree rule) would make the transform row comparable with Bird's.
### 2026-10-02 — Volcano decisions: stratovolcano heights, island subsidence, continental hotspot calderas (agent worktree agent-a131f01c22943a1ff, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c; not merged)
- **What.** Ben's four answers of 2026-10-02 (decisions log, last four rows): (1) stratovolcanoes `arc_height_km`
  [1.2, 3.0] → **[0.6, 2.0]** and `arc_radius_km` 14 → **10** (VOL-019, VOL-020); (2) `island_subsidence_m_myr` 25 → **0**,
  setting kept (VOL-048); (3) continental hotspot centres collapse at about **1 per Myr**: a new setting
  `volcanoes.hotspot_caldera_rate_per_myr` = 1.0 replaces the factor 3 on `caldera_rate_per_myr` (0.04, unchanged for arc
  cones and tall shields; VOL-045, new row VOL-074); (4) geothermal rules (400 mm rain test, no pools under 3 degC,
  pools of 4 to 10 blocks) kept, no code change, GEO-007, -019, -020, -027, -051 marked owner-confirmed. The four
  questions are out of "Pending questions" (GEO-044 and the other geothermal questions stay open).
- **Code.** Defaults in `volcanoes/params.rs` and in all eight built-in presets and the test preset
  `planet_world_4k.json`; the stage's decisions factored into functions the tests call (`arc_height_km`, `caldera_rate`,
  `caldera_chance`, `caldera_erupts`, `sink_dead_islands`; draws unchanged); the summary line gained
  `hotspot_calderas=`. **How the caldera rate is expressed:** a hazard per active centre, `1 - exp(-rate * dt)` a step
  (so 0.86 per 2-Myr step at rate 1); a collapse ends the centre (as before), and the hotspot builds a new one only
  once the old has drifted 70 km from the plume, so a track makes about one caldera per centre-life and the rate
  holds per Myr of an active centre, not per Myr of the hotspot. Editor: new field `hotspot_caldera_rate` (label in
  the lang file), hints for `arc_height_high`, `arc_radius`, `island_subsidence`, `caldera_rate` and `volcano_size`
  rewritten, `arc_height_high` minimum 1.2 → 0.6 and `arc_height_low` maximum 3 → 2 (the editor's extremes test sets a
  knob to its end with the others at their defaults, and the pair must stay ordered).
- **Tests** (new, `volcanoes/tests.rs`): `stratovolcano_heights_fall_in_earths_range` (a synthetic arc with flux rising
  from the threshold to 1: the cones' aimed median is inside Earth's mean ± s.d., 0.58-1.68 km, every cone inside
  0.17-4.57 km, height to basal width inside 0.02-0.30 and within 10 % of the old proportion);
  `a_continental_hotspot_makes_about_one_caldera_per_myr` (300 Myr of an active centre renewed after each collapse,
  through the stage's own `caldera_erupts`: 240 to 360 calderas, expected 293; arc rate 0.04 unchanged; a shield under
  4 km never collapses); `island_subsidence_defaults_to_nothing_and_the_setting_still_works` (default 0 leaves the
  surface unchanged; 25 m/Myr sinks a dead island to its 1.5 km maximum and spares an active one); the pipeline test
  also asserts that no volcano of its 120 Myr run has sunk.
- **Science docs.** `docs/science/volcanoes-and-magma.md` (decisions section replaces open questions 1-3, tables,
  numbers, new runs table), ledger VOL (VOL-019 `calibrated`, VOL-020 and VOL-048 `game-choice`, VOL-045 reworded,
  VOL-074 new `game-choice`: VOL 74 rows, calibrated 2, game-choice 47, uncited 24), HNT (HNT-016, -061 revised;
  HNT-085 to -088 new: 88 rows, verified 13, game-choice 38), GEO rows, `claims.md` totals (2,214 rows, 19 `verified`),
  bibliography, `preset-coverage.md`. Sources read again on the test VM: Christiansen 2001 (three cycles, each ending
  in a caldera-forming ash-flow eruption, 2.1, 1.3 and 0.64 Ma: `verified`) and Grosse et al. 2014, table 1
  (stratovolcanoes, n = 512: height median 1.02 km, mean 1.13, s.d. 0.55, range 0.17-4.57; height/width median 0.13).
- **Measured** (test VM; `earthlike_quick_16k`, 500 Myr, seeds 1-3, scattered continents and supercontinent, six runs
  per variant, `deeptime stage volcanoes`, 2 threads, 4 at a time; `tools/review/p1-17-hyps.sh 500 "1 2 3"
  "scattered_continents supercontinent" new "old:volcanoes.arc_height_km=[1.2,3.0];volcanoes.arc_radius_km=14;
  volcanoes.island_subsidence_m_myr=25;volcanoes.hotspot_caldera_rate_per_myr=0.12"`; the old variant is master's
  behaviour and reproduces the WP-2 seed-1 numbers exactly). Means, ranges in brackets. **Standing cones' median height
  1,484 m (1,389-1,627) → 869 m (764-994)** (Earth 1,020 m); active cones 1,875 → 1,044 m. **Land above 2 km 0.124
  (0.092-0.149) → 0.077 (0.062-0.097)**, lower in all six pairs (Earth 0.102); median land height 687 → 559 m (Earth
  427 m); land share 0.295 → 0.266 (Earth 0.289). Calderas formed 512 → 496; at continental hotspot centres 42 (25-68)
  → 62 (18-102); share of silicic hotspot centres that collapsed 66 % (253 of 382) → 98 % (374 of 380) (WP-2's
  "10-30 % chance" was low: a centre stays active several Myr); arc calderas 469 → 433; summit calderas 22 → 16;
  relief built by volcanoes 2.4 × 10⁷ → 1.2 × 10⁷ km³. **Contradicts nothing in the plan, but note:** the lower cones
  move the land above 2 km from above Earth's to below it; the hypsometry recalibration (decisions log 2026-10-01,
  "later with the plate work") should expect that. Scratch runs in `/Volumes/Graphide/ban/deeptime-runs/vol-decisions`,
  deleted.
- **Gates** (test VM): `cargo fmt --check` clean; `clippy --workspace --all-targets -D warnings` clean; `cargo test
  --workspace` 682 passed, 0 failed, 3 ignored; `cargo deny check licenses bans` ok; `./gradlew build test` 226 tests,
  0 failures, 1 skipped; `./gametest.sh` All 68 required tests passed; MIC: `./gametest.sh -PwithMic` (168 mods) All 68 required tests passed; its first run died in mod loading before any test (Registrate's "Found unused register callbacks" in Create's register event, the flake recorded in the P2-08 entry), the same jars loaded on the second.
- **Next step.** The coordinator merges the branch; the hypsometry recalibration (decisions log 2026-10-01) now starts
  from land above 2 km 0.077 and median land height 559 m.

### 2026-10-02 — Science documentation, stage 2, WP-2: volcanoes and magma checked against sources (agent worktree agent-a44828926d2eaecb7, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c; not merged)
- **What.** Every VOL and MAG row's source and status revised (`docs/science/ledger/VOL.md`, `MAG.md`); the page
  `docs/science/volcanoes-and-magma.md` rewritten for stage 2; ten volcano and magma hints corrected
  (`editor_fields.json`; HNT-015 to -017, -058 to -060, -062, -063, -081, -082); code comments in
  `volcanoes/{params,shape,mod}.rs`, `underground.rs` and `Magma.java` rewritten from the sources read, with
  `// ledger:` ids beside the constants; nine bibliography entries added, four updated. Fixes 11, 12, 30 and flag 27
  (HNT-017 and the params comment) settled. **No value in the code changed.** The volcano summary line now reports
  cone height percentiles (`height_p10_p50_p90_m`).
- **Rows.** VOL: `cited-unverified` 1, `calibrated` 1, `game-choice` 16, `uncited` 55 → `abstract-only` 1,
  `calibrated` 1, `game-choice` 44, `uncited` 27. MAG: `cited-unverified` 3, `game-choice` 31, `uncited` 8 →
  `verified` 2 (vanilla's lava rules, read in the Minecraft source), `abstract-only` 1, `cited-unverified` 1,
  `game-choice` 37, `uncited` 1. Most new `game-choice` rows are implementation and display rules labelled as such.
- **Sources read in full** (library, test VM): Grosse et al. 2014 (global composite-volcano morphometry, CONICET
  repository copy), Hildreth 2007, Clague & Sherrod 2014, Poland et al. 2014, Christiansen 2001, Courtillot & Renne
  2003; USGS Mauna Loa and FAQ pages. Abstracts only: de Bremond d'Ars et al. 1995 (the publisher refuses scripted
  fetches of its free PDF), Rasmussen et al. 2022, Chaussard & Amelung 2014, ten Brink 1991. Not obtained: Lev et
  al. 2019 (lava lakes), the Global Volcanism Program page, Geyer & Martí 2008 (repository behind a bot check).
- **Findings.** The arc spacing citation was wrong (the paper finds no characteristic spacing; mean spacings of
  large centres are 27–87 km, Hildreth 2007): 60 km is now a labelled game choice. Stratovolcanoes are taller than
  Earth's (above). Dead ocean volcanoes are sunk twice since flexure landed (above). Continental hotspot calderas
  are about ten times rarer than Yellowstone's (above). Comments corrected: flank slopes (the strato profile is
  18° at the summit for 3 km over 14 km, not 20° at 2.5 km), crater sizes, caldera radii (the comment's
  Crater Lake and Long Valley numbers were radii; extraction flag 4 misread them), chamber depths, the lava-lake
  count (USGS: 1,350 potentially active volcanoes, not 1,400). Three questions under "Pending questions".
- **Measured** (test VM, `earthlike_quick_16k`, 500 Myr, seed 1, both starts, `deeptime stage volcanoes`,
  2 threads; `tools/review/p1-17-hyps.sh 500 "1" "scattered_continents supercontinent" base
  "h:volcanoes.arc_height_km=[0.6,2.0]" "sub0:volcanoes.island_subsidence_m_myr=0"`, built from 360b2ed = master
  29d493f + the height report; scattered / supercontinent): defaults land 0.252 / 0.284, median 569 / 650 m, above
  2 km 0.092 / 0.149, standing cones' median height 1,471 / 1,524 m, live 1,869 / 1,831 m; cones 0.6–2.0 km
  (radius unchanged): land 0.246 / 0.266, above 2 km 0.079 / 0.156, standing cones 929 / 914 m; island subsidence 0:
  land 0.285 / 0.280, above 2 km 0.091 / 0.134, standing shields 7 / 11 (defaults 5 / 14). Every difference is
  inside the run-to-run scatter (0.092–0.149 above 2 km over four runs, flexure build log).
- **Gates** (test VM): `cargo fmt --check` clean; clippy `-D warnings` clean; `cargo test --workspace` 673 passed,
  0 failed, 3 ignored; `cargo deny check licenses bans` ok; `./gradlew build test` 217 tests, 0 failures, 1
  skipped. GameTests not run: nothing the game reads changed (comments, hints and a summary line only).
- **Next step.** Done: Ben answered the three questions on 2026-10-02 and the value changes landed in the entry "Volcano decisions" above.
  Left for others: the arc-flux law is coded three times (VOL-003, RCK-059, the deposit engine).
- **Merged with master a7ea05d** (WP-15/16, P1-10): where both packages corrected the same text, the version that
  agrees with the primary sources read was kept (volcano comments and the flanks, spacing and largest-caldera hints
  from WP-2; Crater Lake, Mauna Loa and the impact hints from WP-15); the HNT rows describe the merged hints;
  `claims.md` counts recomputed from the ledger files (17 `verified` in all). Re-gated after the merge: fmt and clippy
  clean, `cargo test --workspace` 679 passed, 0 failed, 3 ignored, deny ok, `./gradlew build test` 226 tests, 0 failures.

### 2026-10-02 — Science stage 2, WP-15 and WP-16: editor hints and geothermal features (agent worktree agent-a800766534f3bdb2b, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Hints.** `editor_fields.json` (the only copy of the hint texts; the lang file holds labels) corrected: impact rate (a factor of a million), "15 times taller than wide" (heights are stretched 15 times against widths; comments in five source files reworded), ice yield stress (32 kPa Antarctica, 80 kPa Greenland, 50 chosen), caldera and shield figures (Crater Lake, Yellowstone, Mauna Loa read in USGS and NPS pages), ejecta taper, river units, PET factor 58.93 (Holdridge read), dyke width, thrust slip, lake freezing, day length, star age, the arc-range width scale, and the four geothermal hints (fields per volcano none to seven, pool size, reach, 3 degC). `editor.rs`'s hint test (length over 20) is the only test that reads them. No behaviour changed.
- **Geothermal.** GEO ledger: 1 `verified` (the sinter and travertine rule), 47 `game-choice`, 3 `uncited`; page written; FORMAT.md and comments agree with the code; ledger ids in comments. The P2-14 entry's heading and first bullets (lost in a merge) are restored above, reconstructed, because no commit holds the original text.
- **Gates.** See the report; no GameTest (no geothermal code changed).

### 2026-10-02 — Science documentation, stage 2, WP-0: source library, de-duplicated bibliography and ledger, coverage gaps (agent worktree agent-a024f62a028c56ca4, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c; docs only)
- **Ask.** Prepare the ground for the verification packages: readable sources, one entry per work, one row per claim, the stage 1
  coverage gaps filled. No simulation or game code, presets or tests changed; no claim status changed except by retiring duplicates.
  Master 1eac09f (flexure, ISO-040 to ISO-067) merged in; the flexure rows duplicate nothing.
- **Source library** (a folder on the test machine, one folder per bibliography key, with the PDF or page and a `.txt`
  beside it; never in the repo, not distributed). Fetched by direct request only (DOIs, publisher and agency pages, repositories,
  Crossref, Europe PMC, Semantic Scholar, NASA ADS, the Internet Archive); no shadow library; the 350 MB of stage 1 working copies
  were copied in, not fetched again. Result: of 385 works, 104 are readable as text, 6 are page scans with machine OCR, 35 have a
  landing page only, 240 were not fetched (146 paywalled with no lawful substitute found, 31 books, 22 open-access or open-copy
  works whose publisher refuses scripted access, the rest software, web pages and unresolved citations). Seven paywalled works have
  a repository copy. `bibliography.md` records, per entry, fetched or not, from where and the text file name.
- **Bibliography.** 488 entries (475 + 13 flexure) merged into 385 works (103 entries folded in; two keys that named two different works
  each were split (`Courtillot2003`, `USGS2000`), four joint entries divided). Stage 1's 7 unidentified: `SAIMM-coalfields` identified
  and read (Jeffrey 2005), `Shepard`, `ICAO1993`, `Lallemand_EncMarineGeosciences` identified as probable, `Software` read as three
  works, the rest unresolved. Of the 15 `not found`: `Gilbert1893` read (Internet Archive), five NASA ADS scans retrieved,
  `GrieveCintala1992` identified (DOI), the rest unresolved. 171 of the 676 research-note mentions are the same work as an entry
  (`[= Key]`); 505 stay `[unresolved]`.
- **Ledger.** 216 candidate pairs (shared code symbols, text similarity) judged by four Sonnet agents; 193 rows retired, none renumbered;
  six DEP/ORE and RCK/ERO disagreements between agents were settled by the lead. Owners absorb the retired row's extra citation; the
  status disagreements that merging exposed are listed in `claims.md` for the verifying package. Rows now: 2,205 active (+193 merged).
- **Rows added** (Sonnet agents, code read as on master): GEO 51 (the geothermal features), PRS 74 (presets, test presets, world
  presets), HNT 84 (editor hints, incl. the impact-rate hint, factor of a million, not fixed), CRE 35 (plate geometry, noise, plate
  state, deformation lattice), CHK 57 (plate-check target ranges). Flags 27 to 48 in `claims.md` list what they found.
- **Against the task or the rules.** Ledger prefixes PRS, HNT, CRE, CHK were added besides GEO. A web fetch of ADS returned two
  anti-robot pages saved as small PDFs; a sweep (`pdfinfo`, size per page) caught and removed them. OpenAlex's free allowance ran
  out after about 300 lookups. `plates/dynamics.rs` is still not covered. A peer's `_incoming` folder in the library was not touched.
- **Next step.** The verification packages read the text files named in `claims.md` ("Sources in the library, by work package").
### 2026-10-02 — P2-01 fix: the structure stage panicked on long-lived belts (agent worktree agent-a49703993341aa701, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Bug.** `earthlike_quick_16k`, supercontinent, 1000 Myr, seed 1 panicked: "a history holds at most 128 events"
  (`deeptime-noddy/src/history.rs`). **Cause:** only the end-of-run check (`deform/lattice.rs` `noddy_check`, which
  compares the column rule with the Noddy port on every 53rd L2 column) built a Noddy history, one per sampled column
  from all its candidate structures; the port keeps the original's limit of 127 events a history (its 4-bit-per-event
  sequence code), and a belt active for most of 1,000 Myr puts more structures over one place. The column rule the
  game runs (Rust and Java) never had a limit, so nothing the game reads changed. Master's interim guard (5ce5af4)
  skipped such columns silently.
- **Fix.** `noddy::Chain`: any number of events undone as a run of histories of at most 127 each, youngest first; the
  same restored position exactly (unit test against undoing event by event). The check uses it, skips no column, and
  the note carries `noddy_check.max_history` and `noddy_check.long_histories` (sampled columns that needed a chain).
  `History::is_full` lets a caller ask before `push`. The density itself (option c) was left alone: `max_overlap` 3 per
  kind with overprinting already bounds each kind, and 218–340 candidates over a column is what three kinds over
  1,000 Myr of belts give; nothing showed it to be wrong in the rock.
- **Tests.** `a_place_with_more_structures_than_one_noddy_history_holds` (420 structures over one place: panicked on
  the old code, skipped every column with the guard), `the_stage_never_panics_however_dense_the_structures` (seeds 1–4,
  both starts, dense settings; asserts the longest history met passes 127), `a_full_history_says_so_and_a_chain_takes_any_number`.
- **Measured** (`tools/review/p2-01-planet.sh world sN earthlike_quick_16k supercontinent 1000 --set seed=N`, then
  `tools/review/p2-01-dense.py`): s1–s4 on fd66131 (before the flexure merge), h1 = seed 1 on a62824d (with flexure).
  All five ran through the bake, strata-check, sections and parity with exit 0.

  | run | events | max candidates | max history | chained columns | changed land | malformed | bed offset p90 | grade inversions |
  |---|---|---|---|---|---|---|---|---|
  | s1 | 27,461 | 266 | 120 | 0 | 66.5 % | 0 | 8 | 0.145 % |
  | s2 | 30,976 | 340 | 175 | 94 | 62.8 % | 0 | 8 | 0.245 % |
  | s3 | 20,261 | 126 | 75 | 0 | 72.9 % | 0 | 8 | 0.115 % |
  | s4 | 24,187 | 243 | 135 | 3 | 68.0 % | 0 | 8 | 0.235 % |
  | h1 | 21,442 | 218 | 111 | 0 | 69.3 % | 0 | 9 | 0.095 % |

  Noddy against the column rule: worst 1.5e-5 block, 0 side flips, in every run. Cost on the densest (s2, Java
  `ColumnRuleBenchTest` with `DEEPTIME_BENCH_WORLD`, one thread): 1.117 ms a chunk through the structures against
  0.558 plain; slowest chunk 91 ms.
- **Gates** (a62824d): fmt, clippy -D warnings, `cargo test --workspace` 0 failed (core 475), deny ok, `gradlew build
  test` 217 tests 0 failures. GameTests not re-run: nothing the game reads changed.
### 2026-10-01 — P1-10 TerraBlender regions follow provinces (agent worktree agent-a7e713f9fdd326fb4, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
Branch `worktree-agent-a7e713f9fdd326fb4`, master e8668ca merged in (then 5e601d9, folds and faults stage B, format 0.3), not merged back, not pushed. Plan §10 P1-10 "As built" and §7.3 carry the design; FORMAT.md, "The geologic provinces and TerraBlender's regions", the contract.

- **The plan's mechanism was not needed.** TerraBlender 4.1.0.8 (decompiled on the VM) has no `terrablender:uniqueness` density function and no uniqueness on `Climate.TargetPoint`/`Sampler`; it picks a region in `MixinMultiNoiseBiomeSource.getNoiseBiome` (HEAD, cancellable) from `IExtendedParameterList.getUniqueness(x, y, z)`, a layered noise of (x, z) alone, on a parameter list that every level on the overworld preset shares. So ReTerraForged's four mixins and `RegionIndexDensity` were not ported (no ReTerraForged code used; NOTICE.md unchanged). One mixin, `mixin/compat/terrablender/RegionBiomeSourceMixin` (priority 900 so its handler runs before TerraBlender's), answers first **only for a Deep Time level's sampler** and runs the same tree search with the column's province region (`compat/terrablender/RegionTrees`). How it knows the level: `GeologicChunkGenerator.bindSettings` wraps the noise settings' `spawnTarget` list in `worldgen/RegionSpawnTarget` (carries the level's `RegionSource`: `meta/province` at the column → region index), and `RandomState` hands that list to `Climate.Sampler` unchanged. Anything else (other levels, no table, TerraBlender not initialised) falls through. Compile-only dependency `maven.modrinth:terrablender:6e8GCrLb`, plus five access-transformer lines (the ones TerraBlender's own AT has) so Deep Time compiles without TerraBlender applied.
- **No fallback depth** (the plan's "points below the fallback depth use TerraBlender's own index"): TerraBlender's uniqueness ignores y, so a province keeps its region at every depth and cave biomes come from the region's tree as before.
- **The bake's provinces are the USGS classes** (log 2026-09-29, "Yes, for P1-10"). `bake/meta.rs` runs `provinces::derive_with` (the view layer's classifier, now with the preset's thresholds) and cuts the classes into connected regions, small ones absorbed as before. `meta/province` keeps its name, type and contiguity; `notes.bake.meta.provinces` lists `classes`, `params`, per-province `class`/`cells`/`km2` and `shares` (`classified` and `provinces`, planet and continental, plus `absorbed_cells`). Settings added: preset section `provinces` (`min_area_km2` 50,000, `orogen_max_age_myr` 500, `andean_max_age_myr` 50, `andean_min_elevation_m` 1,500), four editor fields in Advanced · Biome placement and regions (`region_size`, `orogen_age`, `range_age`, `range_height`), lang keys, all eight built-in presets. The view layer keeps the default thresholds. `planet_world_4k.json` (test planet) sets `min_area_km2` 1,500: it has 1/66 of the Earth-scale planet's area and with 50,000 it baked 2 provinces (sea floor and one extended-crust continent); with 1,500, 34 to 35. This changes no simulation figure, only `meta/province`.
- **Province → region** (`climate/ProvinceRegions`): stratified by weight (rank by a hash of province id and the preset seed, ranks dealt over the regions by cumulative weight), stored by region *name* in `climate_mapping.json` `regions` the first time a start has TerraBlender and the world has provinces; kept afterwards; unknown names fall back to the default region with a warning. Without TerraBlender nothing is written and nothing changes (MIC).
- **The orogen share, measured** (`earthlike_quick_16k`, scattered start, 500 Myr, preset seed, this tree; checkpoint `7-amplify`; `tools/review/p1-10-orogen.sh`). Classified shares of the continents, default thresholds: shield 0.157, platform 0.123, **orogen 0.421**, basin 0.085, LIP 0.045, extended 0.168 (matches P1-27's 0.40). Orogen by reason: active collision/arc regime 0.125, collision belt within 500 Myr 0.166, Andean range 0.134. Thresholds (orogen share): `orogen_max_age_myr` 400 0.421, 300 0.389, 200 0.366, 100 0.355, 50 0.238; `andean_min_elevation_m` 2,000 0.402, 2,500 0.385, 3,000 0.375; `andean_max_age_myr` 25 0.414, 10 0.396; 200 + 25 + 2,500 together 0.327. The active-regime part (0.125) is the plate model's, so thresholds cannot reach Earth's 0.17. Defaults unchanged (question in Pending). **After absorption** (the bake: 40 provinces on this planet, 68,000 to 2,200,000 km², `absorbed_cells` 412,247 of 524,288) the share of the continents is orogen 0.09, extended 0.44, basin 0.19, LIP 0.12, platform 0.11, shield 0.04: thin belts lose to the long coastal extended-crust ring. The notes record both.
- **Tests.** Rust: `bake::meta::tests` (3: absorption, two regions of one class, shares classified vs provinces); `provinces::ProvinceParams` through the preset and editor tests. JUnit: `ProvinceRegionsTest` (7: weights, small planet reaches every region, determinism, runs, JSON round trip through `climate_mapping.json`), `ProvinceBakeTest` (2: the real test planet's classes, ids = layer, shares, parameters). GameTest `TerraBlenderGameTests.regionsFollowProvinces` (+ `TerraBlenderChecks`) on `test_planet_real`: every column of a province has the one region the world's table gives it, the table's names resolve to TerraBlender's indices, every sampled biome is offered by its province's region or vanilla's, at least 3 provinces, 2 regions, some land under a mod region. Without TerraBlender it checks that no source is attached. **Negative control:** with the mixin removed the same test fails (245 of 4,608 samples are biomes their province's region does not offer, e.g. `biomesoplenty:tundra` in a vanilla province). The mixin priority works as intended (positive run: 0 outside).
- **Measured.** `./gametest.sh -PwithTerraBlender -PgametestOnly=regionsFollowProvinces`: samples=4608 provinces=34 regions=4 provincesByRegion={0=11, 1=11, 2=10, 3=2} landUnderModRegions=223 landByNamespace={biomesoplenty=199, minecraft=427} outside=0 mismatch=0. Real 16k planet (`tools/review/p1-10-smoke.sh`, terrablender set, 500 Myr, generated at first start): smoke PASS, `/deeptime probe regions 16` has_regions true, 40 provinces, land biomes biomesoplenty 95,548 / minecraft 41,096 (BOP biomes fill whole provinces of BOP regions, only where the region has a biome for the climate; elsewhere its tree defers to vanilla's). Review page: `tools/review/region_map.py` over the probe's grid → review/out/p1-10/index.html (regions.png: provinces coloured by region; namespaces.png: biome namespaces with province borders). Not published.
- **Gates** (test VM). `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check licenses bans` clean; `cargo test --workspace`: core 384 + 3 ignored (the last new test: `provinces::tests::province_params_are_the_rules_and_the_builtins_carry_the_defaults`), cli 54, format 53, inspect 31 + 2 + 7, jni 7, viewer 6, history_cap 1, all pass; `./gradlew build test` BUILD SUCCESSFUL, tests=208 failures=0 errors=0 skipped=1. `./gametest.sh` All 66 required tests passed, all mixins bound. `./gametest.sh -PwithTerraBlender`: 65 of 66; the one failure is `climateConsistent` (C 0.681 < 0.750, BW:desert 0.284 < 0.370 with the mixin; C 0.558 < 0.750 with the mixin removed): the floors were calibrated for standalone and MIC, and BOP's biomes lack the `c:` tags the check counts, so it fails in this profile with or without this change (not a P1-10 regression; whether it passed here on master was not measured). `-PwithMic` (168 mods, MIC rock tags, `install.sh mic` at the pack's current pin): All 66 required tests passed, `regionsFollowProvinces` logs loaded=false (no TerraBlender: no source attached).
- **Not verified.** Real client (nobody looked at BOP regions in a running game; the probe reads the same biome source the chunk generator uses, and the Chunky smoke generated chunks without error). Other region mods than BOP (Regions Unexplored uses Lithostitched's own registry: out of scope, plan §7.3). TerraBlender versions other than 4.1.0.8 (the mixin relies on `IExtendedParameterList` and `Region.DEFERRED_PLACEHOLDER`). Worlds baked before this change have no province raster in the new meaning (no migration, as ruled). Behaviour of `RegionBiomeSourceMixin` with C2ME's parallel biome fill beyond the smoke's Chunky run.
- **Contradicts the plan:** the four mixins and the density function (above); the fallback depth (above); `compat/terrablender/…Mixin` lives in `mixin/compat/terrablender/` (the plugin's convention); the test is in `TerraBlenderGameTests` (plan) with `bopBiomesPresent` still in `BiomeGameTests` (P0-12; it runs on the synthetic `test_planet_data`, which has no provinces, so it still exercises TerraBlender's own noise).
- **Follow-up, 2026-10-02: master 5e601d9 merged; `climateConsistent` under `-PwithTerraBlender` settled.** The merge (STATUS rows and log entries of both sides kept; `planet_world_4k.json` unchanged by master, keeps `provinces.min_area_km2` 1,500; the built-in presets carry both master's folds sections and `provinces`) needed no code change: `meta/province` reads plate fields and `structure/rock`/`surface/class` as before, and FORMAT.md's province section is independent of 0.3.
  - *Does it fail on master without this branch?* Yes. A clean checkout of master 5e601d9 (a scratch worktree, its own VM copy) under `-PwithTerraBlender`: `climateConsistent` fails on the default planet (C 0.558 < 0.75) and on planets 11-15 the shares over all columns are C 0.27-0.69, E:snow 0.28-0.84, D 0.50-0.97, BW:desert 0.26-0.67 (floors 0.75, 0.72, 0.88, 0.37). The floors were read from the standalone and MIC sets; Biomes O' Plenty's biomes carry few of the `c:` tags the families count, so the measure in this profile is BOP's tagging, not Deep Time's mapping.
  - *Why 0.681 with the mixin against 0.558 without:* chance. With the province regions the BOP biomes fill whole provinces, and the C columns sit in a few provinces, so the share is which provinces drew a BOP region: planet 11 went 0.558 to 0.681, planet 12 went 0.644 to 0.27, planets 14 and 15 to 0.18 and 0.14 (planets 11-15 with the mixin: C 0.14-0.68, E:snow 0.13-0.95). The mixin neither caused nor cured it.
  - *Decision (not a floor lowered until it passed):* the census (`BiomeCensus.run(..., untaggedNamespaces)`) leaves the columns of an untagged mod namespace out of the tag checks, counted as `skipped_untagged`; `climateConsistent` passes `biomesoplenty` when it is loaded and logs the all-columns census beside it (not asserted). What remains is vanilla biomes through the same mapping: on planets 11-15 the shares sit in the standalone ranges (C 0.85-0.97, D 0.99-1.0, E 0.96-1.0, E:snow 0.78-0.99, Dsub 0.91-0.95, BW:desert 0.57-0.62). That leaves 40-60 % of the columns, so the TerraBlender profile asserts a family from `MIN_COLUMNS_TB` = 100 columns (standalone and MIC: 40, unchanged), and two floors are lower there (`FLOORS_TB`): BS 0.39 (lowest 0.496) and E:snow 0.67 (lowest 0.776), 0.1 under the lowest measured; the others keep theirs. Measurement table in the comment above `FLOORS_TB`; `tools/review/climate-floors.sh "<seeds>" -PwithTerraBlender` repeats it.
  - *Checked on more planets:* seeds 12-17 with the new test: all pass (planets 14 and 17 have fewer than 100 columns of most families and assert little; that is the price of leaving BOP out).
  - *Gates on the merged tree* (test VM): cargo fmt, clippy `-D warnings`, deny clean; `cargo test --workspace` core 401 + 3 ignored, cli 54, format 63, inspect 31 + 2 + 7, jni 7, viewer 6 and the rest, all pass; `./gradlew build test` tests=209 failures=0 skipped=1; `./gametest.sh` 66 of 66, `-PwithMic` 66 of 66, `-PwithTerraBlender` 66 of 66, all mixins bound.
- **Follow-up 2, 2026-10-02: master 2e53ad2 merged (P2-08 meteor impacts, plate-check, grid audit); chunks did not follow the provinces until now.**
  - *Merge:* conflicts only in STATUS.md (rows and entries of both sides kept), the eight built-in presets and `planet_world_4k.json` (both `provinces` and `impacts` sections, valid JSON), and `params.rs` (`PLANET_KEYS` is 17 now, both sections parsed and written). `editor_fields.json`, `EditorFields.java`, `AdvancedSettingsTest` and the lang file merged cleanly (ten advanced sections, my four fields in Biome placement and regions).
  - *A bug of mine, found by the merge:* `farTerrainMatchesGeneratedChunks` failed under `-PwithTerraBlender` (biome agreement 107 of 119, e.g. `biomesoplenty:cold_desert` against `tundra`). Cause: `NoiseBasedChunkGenerator.doCreateBiomes` builds a generated chunk's climate sampler with `this.settings.value().spawnTarget()`, the *registry* settings' list, not the per-level copy `GeologicChunkGenerator.bindSettings` makes, so `RegionBiomeSourceMixin` never saw a `RegionSpawnTarget` on chunk generation: generated chunks still took TerraBlender's own regions, while `RandomState`'s sampler (queries, `/locate`, far terrain, my GameTest and probe) followed the provinces. It passed before only because that planet had less BOP land. The earlier statements here that BOP biomes fill whole provinces held for the sampler, not for stored chunk biomes.
  - *Fix:* `mixin/compat/terrablender/RegionChunkBiomesMixin` (`@ModifyArg` on the `cachedClimateSampler` call in `doCreateBiomes`) passes the generator's bound `spawnTarget`. `TerraBlenderChecks` now also reads the stored biomes of 64 generated chunks at `ChunkStatus.BIOMES` and requires each to be offered by its column's region. **Negative control:** without the new mixin that check finds 3 of 64 chunk biomes outside their region (`biomesoplenty:cold_desert` in a region-2 column) and `farTerrainMatchesGeneratedChunks` fails; with it both pass.
  - *Re-measured `climateConsistent` (TerraBlender profile, BOP columns out, planets 11-17, families of 100+ columns):* A 0.717-0.98, Af 0.985-1.0, BW 0.756-0.758, BS 0.527-0.733, C 0.848-0.97, D 1.0, Dsub 0.797-0.934, E 0.953-1.0, BW:desert 0.569-0.605, Af:jungle 0.489, Dfc:taiga 0.65, E:snow 0.751-0.957. Two came within 0.1 of their floors on the new planets, so with the first set taken in: E:snow 0.67 to 0.65, new Af:jungle 0.39 (`FLOORS_TB`); the rest hold with margin. `regionsFollowProvinces` on the new test planet: 33 provinces, 4 regions, 525 land samples under mod regions, 0 outside.
  - *Gates on the merged tree* (test VM): cargo fmt, clippy `-D warnings`, deny clean; `cargo test --workspace` core 445 + 3 ignored, cli 54, format 63, inspect 31 + 2 + 7, jni 7, viewer 6, history_cap 1, all pass; `./gradlew build test` tests=212 failures=0 skipped=1; `./gametest.sh` 67 of 67, `-PwithMic` 67 of 67, `-PwithTerraBlender` 67 of 67, all mixins bound.
  - *Not done:* `docs/science` (the science-documentation rule is in master's CLAUDE.md after this merge's base, 2e53ad2 has no `docs/science/`); the province thresholds (orogen 500/50 Myr, 1,500 m, 50,000 km²) are ledger rows to add when it lands.
- **Follow-up 3, 2026-10-02: master 1eac09f merged (geothermal pass, general flexure, `docs/science/`); science documentation added.**
  - *Merge:* conflicts in STATUS.md only, after the first merge of 5ce5af4 (also `GeologicChunkGenerator`: both the geothermal wiring and the region note kept). The P2-14 build-log entry on master had lost its header and first bullets to a copy of my P1-10 entry in an earlier STATUS.md merge on master; I kept its surviving tail under a header that says so. Master's answered flexure question row is dropped.
  - *Science documentation:* `docs/science/provinces.md` written from the template; `ledger/PRV.md` re-marked (the USGS map's class names, age bands and 150 km read in the Wikimedia Commons description fetched today: PRV-001 to PRV-004 and PRV-021 are now `abstract-only`; the original USGS page is dead and the Internet Archive and the other USGS pages could not be fetched; the map's own assignment rules were not found, so the mod's rules stay `uncited`/`calibrated`/`game-choice`) and PRV-028 to PRV-031 added (bake scheme, minimum area, region dealing, unclassified code, all `game-choice`); BAK-010 to BAK-012 marked superseded or moved; `claims.md` counts (2,101 rows), `bibliography.md` USGSprov entry updated; `// ledger:` ids beside the `Rules` and `ProvinceParams` defaults.
  - *Orogen share with flexure on* (`earthlike_quick_16k`, scattered, 500 Myr, seed 1, this tree; `tools/review/p1-10-orogen.sh`): classified shield 0.157, platform 0.139, orogen 0.245 (active regime 0.118, belts 0.042, Andean 0.060), basin 0.136, LIP 0.100, extended 0.224; thresholds 200 / 25 / 2,500 give 0.185; after absorption 43 provinces, orogen 0.285. The share fell from 0.421 (earlier plate model) to 0.245; the pending question row carries the update.
  - *Re-measured with flexure on:* the `-PwithTerraBlender` `climateConsistent` shares on planets 11-17 and `regionsFollowProvinces` (33 provinces, 4 regions, 525 land samples under mod regions, 0 outside, 64 generated chunks 0 outside) are identical to the previous measurement: the test planets come from `tests/presets/planet_world_4k.json`, not from the built-in presets that now turn flexure on, so nothing moved and no floor changed.
  - *Gates* (test VM): fmt, clippy `-D warnings`, deny clean; `cargo test --workspace` core 476 + 3 ignored, cli 54, format 63, inspect 31 + 2 + 7, jni 7, viewer 6, history_cap 1; `./gradlew build test` tests=226 failures=0 skipped=1; `./gametest.sh` 68 of 68, `-PwithMic` 68 of 68, `-PwithTerraBlender` 68 of 68, all mixins bound. The Distant Horizons 3.3.1 caution did not show: the MIC profile's far-terrain tests passed (the mod set is the one `install.sh mic` installed from the pack's pin at the time of the run).
- **Next.** Ben's answers on the orogen thresholds and region size (Pending); P1-11's review page could show province names; the plate model's collision frequency and back-arc halo are the real orogen-share lever (P1-27 follow-up).

### 2026-10-02 — General lithospheric flexure in the surface stage (agent worktree agent-afae960b6db068f94, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c; not merged)
- **Why.** Ben, decisions log 2026-10-02: "General flexure everywhere": every load bends the crust (belts, volcanic
  islands, ice, sediment), not only collision forelands. Design and measurements: `docs/design/flexure.md` (written
  and committed before the code; its section 12 is what was measured). Science page `docs/science/flexure.md`,
  ledger ISO-040 to ISO-067, 13 bibliography entries.
- **The model** (`surface/flexure.rs`, step 5c of the surface stage, after the plateau spreading). A thin elastic
  plate over a fluid mantle, L(D L w) + C w = load, on the L1 raster in block metric. The stage keeps the load since
  the start of the run and the plate's sinking with the crust (`SurfaceState::flexure`, carried by the same
  same-crust rule as the elevation, `advect::carried_fields`). Loads each step: crust the plate model thickened
  (positive uplift on continental crust, or on oceanic crust in an arc or hotspot regime, turned back into the
  thickening behind its Airy elevation), crust moved by spreading, erosion and deposition, rock later stages laid on
  the surface (the volcano stage's cones), the sea over the cell, and land ice (`climate/ice_thickness_m`). The
  elevation changes by the solved sinking less the Airy root already in it, so nothing is compensated twice; with no
  stiffness the result is the old local isostasy without its blur.
- **Stiffness.** Oceanic Te = 2.7 km × √age (Calmant et al. 1990, full text read), at most 40 km; continental Te =
  80 km × (1 − e^(−t/300 Myr)) with t the time since the last orogeny; arcs, rifts and everything else at least 5 km;
  smoothed over 80 plate km. Rigidity gained as crust cools joins stress-free (a frozen bending moment per cell), so
  an old belt's local root does not become relief. **A plate boundary is a break** (found by the first runs: the
  subducting plate held arcs 10 km above their floating level across the trench).
- **Lengths are Earth-normalized** (the solve's cell is the L1 cell's share of the model sphere: 39 km on the quick
  presets), so a basin is as wide against its belt as on Earth. Gravity enters through the flexural parameter
  (`α ∝ g^−¼`). Question 1 below.
- **Solver.** Conjugate gradients; preconditioner: exact spectral solves (Fourier in x, cosine in y, `rustfft`) at
  three uniform stiffnesses, blended, between two damped Jacobi sweeps. Finer L1 rasters (the full presets) solve on
  L1 coarsened to about 39 km and interpolate. Options tried on prototypes (diagonal preconditioner, one spectral
  solve, the blend alone, more levels) are in the design note's section 5 with their iteration counts.
- **Settings.** `surface.flexure.{strength, elastic_thickness_scale}` in the four pipeline presets and built-ins
  (1.0, 1.0; the struct's default strength is 0, so plate-only presets and the GameTest planet are unchanged);
  editor fields `flexure` ("Plate bending", 0–1) and `plate_stiffness` ("Plate stiffness", 0.25–3) with lang keys;
  preset-only: the Te law's five numbers, `every_steps`, `tolerance_m` (2), `max_iterations` (60). New L1 fields each
  step: `surface/flexure_m` (how far the ground stands from local isostasy), `surface/deflection_m`,
  `surface/elastic_thickness_km`. `surface_final` prints `flexure_solves`, `flexure_iterations_per_solve`,
  `flexure_unconverged`; the surface timings have `flexure`.
- **Tests** (`surface::flexure::tests`, 10, and `surface::tests::flexure_off_changes_nothing_and_on_bends_the_plate`):
  a point load sinks by P l²/8D within 3 % and the load is compensated to 1e-9; a line load gives T&S's depth within
  1 %, its zero at 3πα/4 within 2 % and its bulge at πα with e^(−π) of the depth; the width goes as Te^¾ and as
  g^(−¼); a thousandfold patchy stiffness converges and compensates to 1e-3, under water too; the editor's extremes
  and beyond (Te 0.1 to 500 km) converge and compensate; no stiffness is local isostasy (erosion rebounds 0.8, 0.56
  under water, uplift untouched, 3 km of ice sinks 834 m and rebounds); a belt on a craton stands above its Airy
  height with a basin and a bulge, and rebounds when eroded; a plate that stiffens does not move; a plate boundary
  is a break; a coarsened solve agrees with the direct one; strength 0 is the same planet as a preset without the
  section, with no state and no fields.
- **Measured** (test VM, `earthlike_quick_16k`, 500 Myr, `deeptime stage volcanoes`, 2 threads;
  `BIN=… R=… J=4 T=2 tools/review/p1-17-hyps.sh 500 "1 2" "scattered_continents supercontinent"
  on:surface.flexure.strength=1 off:surface.flexure.strength=0`; mean [min, max] of 4):

  | | land | median, m | p90, m | above 2 km | above 3 km | sea median, m | shelf share of sea | highest cell, km |
  |---|---|---|---|---|---|---|---|---|
  | Earth | 0.289 | 427 | 2,022 | 0.102 | | 4,062 | | |
  | without | 0.299 [0.271, 0.340] | 578 [409, 724] | 1,927 [1,637, 2,377] | 0.097 [0.068, 0.142] | 0.035 | 3,408 [3,217, 3,925] | 0.115 | 6.8 (all four) |
  | with | 0.287 [0.252, 0.315] | 676 [569, 753] | 2,219 [1,925, 2,392] | 0.130 [0.092, 0.149] | 0.030 | 3,843 [3,654, 4,175] | 0.061 | 5.3–6.8 |

  Two earlier ensembles of the day (before the merge with master 1aa0579, oceanic law 3.0) gave, with flexure, land
  0.266 and 0.297, median 774 and 638 m, sea 4,096 and 3,798 m, against 0.274, 578 m and 3,437 m without. So the land
  share stays inside the noise (±0.02, ±100 m for 4 runs), the median rises 60–200 m, the sea is 360–690 m deeper,
  shelves are about half as wide, the highest 1 % of land cells is at 3.1–3.8 km instead of 4.0–5.3 km, and the
  plateau spreading moves 5–10 thousand km³ a run instead of 160–400 thousand. 1,000 Myr supercontinent seed 1 (one
  pair, law 3.0): land 0.225 → 0.305, median 502 → 774 m, sea 2,998 → 3,751 m. **Not recalibrated**, as instructed.
- **Run time** (same seed with and without, side by side, 2 threads): scattered 1: 547 → 584 s (+6.7 %);
  supercontinent 1: 534 → 579 s (+8.4 %); 1,000 Myr supercontinent 1: 1,177 → 1,295 s (+10.0 %). The flexure step
  itself is 85–107 s a 500 Myr run (15–18 % of the run without it), 20–24 iterations a solve, 0–2 solves of 250 at
  the limit; the run grows by less because the old isostasy (6 s) and most of the spreading solve (30 → 4 s) go.
  **So the gross cost is above the 5–10 % promised; the net is inside it on 500 Myr and at its edge on 1,000 Myr.**
  One run each: tolerance 5 m saves one iteration; at most 12 iterations cuts the step by a third (177 of 250
  solves then finish in later steps); every second step halves it. Question 2 below.
- **Pictures** (`review/out/flexure/{scattered-1-on, supercontinent-1-on, supercontinent-1-off}/{planet,moat,belt}.png`,
  `numbers.json`; `tools/review/flexure-views.sh <checkpoint> <name> "<label>"`): the ground is more than 100 m off
  local isostasy on 14 % of the planet and more than 500 m on 1.6 % (extremes −3.3 and +2.4 km), nearly all of it
  as paired bands a few cells wide along active margins and arcs, and round volcanic loads on old ocean floor (an
  island held up by 600 m with a ring up to 480 m low). Belt interiors are at local isostasy. In front of the
  largest collision belt there are bands of about 100 m: **no foredeep of the acceptance table's 2–7 km**, as
  expected from today's one-step, 2,000 km wide uplift. Along margins the result alternates from cell to cell; the
  land's cell-to-cell roughness is higher (median 8.7 → 10.2 m, 99th percentile 294 → 391 m, one pair). Stripes
  near the poles, outside the pole cap. The supercontinent pictures are from the build before the regime change
  below and carry an older legend wording.
- **For the belt work** (task 3 of the force-balance design): raise the ground (the wedge on the incoming plate, the
  stack on the overriding one) as uplift; the basin and bulge follow from the incoming plate's Te. Until the two
  plates weld their boundary is a break, so the wedge must sit on the incoming plate's own cells. The foreland for
  the structure stage is `surface/flexure_m` below about −100 m; `foreland_elastic_km` is not needed.
- **For the ice branch:** `flexure::StepIn::ice_m` takes any thickness field; the surface stage passes
  `climate/ice_thickness_m` (the ice at the step's end) when it exists. `climate/ice_balance_m_yr` is not needed.
  Ice is not kept in the carried load, so the ground comes back at the first solve after it melts.
- **Found on the way, outside this task.** (1) `structure/deform/lattice.rs` `noddy_check` panicked the whole run
  when a sampled column lay under more than 126 structures (the reference engine's limit): hit on a 500 Myr quick
  planet; such a column is now left out of the comparison (five lines; the only change outside the surface stage).
  (2) **`plates/flags` is all zero on every step where the plate model resamples its crust** (`ridges.rs` sets
  `flags = 0`, and the resampling runs after `finish_samples` and before the L1 sampling): about a fifth of the
  steps and always the last one. The deposit engine's arc flux and the volcano stage read that field. Not touched
  (plate code); flexure reads `regime` instead.
- **Gates** (VM, the branch merged with master 1aa0579): `cargo fmt --all -- --check`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo deny check licenses bans` clean; `cargo test --workspace --no-fail-fast` all
  pass (673: core 473 + 3 ignored, cli 54); `./gradlew build test` tests=203 failures=0 skipped=1.
  `tools/dev/knob_extremes.py --only flexure,plate_stiffness --length-myr 100 --stage volcanoes`: four runs ok. No
  GameTests (coordinator).
- **Not verified.** That strength 0 is bit-identical to the build before this work (same code path, and a test
  holds it equal to a preset without the section; the old binary was not run). The full presets on a planet (the
  coarsened solve is unit-tested only). Other gravities and strength between 0 and 1 on a planet. Which load causes
  which hypsometry change. The merged tree's with/without run time (the pairs above are from before the merge).
- **Housekeeping.** Scratch folder `/Volumes/Graphide/ban/deeptime-runs/flexure/` deleted. The memory estimate of
  the surface stage grows by 120 B per L1 cell with flexure on.
- **Next.** Ben's answers; the hypsometry in the calibration task after the plate work; the belt work's wedge, then
  the foredeep measured against 150–450 km and 2–7 km; if the cost matters, single-precision transforms or a
  multigrid preconditioner.

### 2026-10-02 — Science documentation, stage 1: every claim in the code ledgered, sources resolved, two pages written (agent worktree agent-a33b33f9ecaf3af1a, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c; docs only)
- **Ask.** Ben, decisions log 2026-10-02: the mod "as backed by science as possible", Markdown in the repo, "Check everything,
  fix the code". Stage 1 changes no simulation or game code, presets or tests: it makes the unchecked visible. Stage 2
  verifies each row and fixes the code. Branch `worktree-agent-a33b33f9ecaf3af1a`, master 60fc341 merged in, not merged,
  not pushed.
- **What is new** (all under `docs/`, plus CLAUDE.md and this file):
  - `docs/science/README.md`: what the mod simulates, the pipeline, the page list, the status marks, the standing limits.
  - `docs/science/claims.md`: the ledger's front (counts, column and status definitions, known gaps, the verification
    plan). The rows are in `docs/science/ledger/<PREFIX>.md`, 24 files, because one file would be about 940 KB.
  - `docs/science/bibliography.md` (sources the code relies on, full references, access, how read),
    `bibliography-research-notes.md` (the further sources the report, notes and designs cite, as they write them) and
    `spot-checks.md` (values the bibliography pass tried to read in their sources, with the passage quoted).
  - `docs/science/_template.md`, two full pages (`impacts.md`, `volcanoes-and-magma.md`) and 17 stub pages.
  - `docs/science/extraction-notes/`: per subsystem, the measured Earth-against-mod numbers found in this build log, the
    stated simplifications, and flags; and `preset-coverage.md` (all 342 leaves of `earthlike_quick_16k` with their owner).
  - `docs/research/README.md` (index of every note and design with its date) and seven `docs/research/notes/session-*.md`
    notes recovered from the coordinator's conversation (Rust against Java, GPU against CPU, lava realism, the mountain
    height limit, polar climate bands, supercontinent heating, the Distant Horizons version).
  - CLAUDE.md: the rule "Science documentation is part of done", and `docs/science/` in the authority list.
- **Counts.** 2,069 ledger rows: `game-choice` 723, `uncited` 635, `cited-unverified` 522, `calibrated` 76,
  `abstract-only` 60, `from-memory` 53, `verified` 0; 377 of them `pending-rewrite` (today's plate motion and banded
  climate) and 57 `not-yet-coded` (constants of the two approved designs). By ledger: PLT 212, CLM 146, MOI 76, BIO 99,
  ERO 167, ISO 39, HYP 34, HYD 76, YNG 21, AMP 71, GLA 66, VOL 73, MAG 42, IMP 141, RCK 84, STR 48, DEF 74, DEP 204,
  ORE 257, PRV 27, SCL 51, BAK 40, POL 21, GEO 0 (geothermal features are not on master). Bibliography: 475 entries
  (a work cited in two subsystems counted twice; 94 known duplicate keys): 206 paywalled, 108 open access, 70 open
  copy, 40 books, 15 not found, 7 unidentified; read in this pass: 74 in full text, 119 abstract, 29 secondary or
  snippet, 228 not fetched. Spot checks: 403 (176 match, 134 partial, 19 mismatch, 74 could not read).
- **How it was made.** Fourteen Sonnet sub-agents, one per subsystem, read the code, the presets, the data files and this
  build log and wrote ledger rows, source lists and flags; ten more resolved the references by web lookup and tried to
  read the most important values in their sources; one recovered the session notes from the transcript. The lead (Opus)
  assembled, normalised the statuses, removed machine names and paths, read the impacts and volcano material in full and
  wrote the pages, the README, the ledger front and the plan. No cargo, Gradle or test run: nothing executable changed.
- **Found (nothing fixed; all in `docs/science/claims.md`, "Found in stage 1: fix first").** (1) IMP-042: complex crater
  depth is coded 0.294 D^0.301 km citing Collins et al. 2005 eq. 28, which reads 0.4 D^0.3 km (read in the paper's text by
  the lead): craters over 3.2 km are about 27 % too shallow, as the impacts agent suspected. (2) The deposit time windows
  are not applied on real histories (the BIF, oxygen and land-plant clocks are hard-coded), so no BIF ever forms. (3) The
  richness setting does not reach the blocks. (4) Köppen tests the polar group before the dry group, against Peel et al.
  2007. (5) Holdridge biotemperature lacks its 30 degC upper limit. (6) Soil-production depth scale 5 m against about
  0.25 to 0.5 m in secondary sources. (7) `k_spl` 2e-9 in the Rust default, 3e-9 in the presets. (8) An editor hint for the
  impact rate is wrong by a factor of a million. (9) The one paper the volcano code cites says, in its abstract, that arc
  volcano spacing has no characteristic value. (10) The "3 to 4 times faster" plate-speed ratio is attributed to Forsyth
  & Uyeda 1975 and found in Conrad & Lithgow-Bertelloni 2004. Twenty-six such items are listed. Confirmed correct, from
  the sources: every Table 3 constant of Cortial et al. 2019; the USGS grade and tonnage tables of the metal deposit
  families; every Köppen threshold of Peel et al. 2007; the Landlab carbonate and marine constants; the other Collins et al.
  2005 crater laws.
- **Against the task or the rules.** The ledger is split into `ledger/*.md` files with `claims.md` as its front, not one
  file. `spot-checks.md`, `extraction-notes/` and `bibliography-research-notes.md` were not asked for. No row is marked
  `verified`: the spot checks were made by sub-agents, and only IMP-042 was re-read by the lead. The CLAUDE.md step
  "set your task to doing and push" was not done (the task forbids pushing).
- **Not covered.** Test modules; `plates/dynamics.rs`, `geometry.rs`, `noise.rs`, `state.rs`; `structure/deform/lattice.rs`;
  solver internals; the Java twins of Rust code; the editor hint texts as a whole; presets other than
  `earthlike_quick_16k` beyond a diff; the test presets. Duplicate rows between ledgers (ocean age-depth in PLT and ISO,
  rock erodibility in ERO and RCK, coal and petroleum in DEP and ORE, the equilibrium-line rule in CLM, GLA and HYD) are
  not merged. Bibliography keys are not de-duplicated across sections. The 676 source mentions of the research notes are
  listed as written, not resolved. The stub pages hold a paragraph, ids and known problems, no model text.
- **For the coordinator.** The older documents hold private details that should go before the repository is public
  (machine and host names, home-directory paths, a full name, account handles, prices): the research-index sub-agent
  listed them by file and line; the list is in this agent's report, not in the repo. The new files were scanned for them.
- **Next step.** Ben reads `docs/science/README.md`, `impacts.md` and the fix list. Then stage 2 by work package
  (`claims.md`, "Work packages"): WP-0 first (a way to read PDF text on the test machine: stage 1 lost most of its
  time to PDFs the fetch tool cannot read and a web-search budget that ran out), WP-13a and WP-14a before the force
  balance and the per-cell climate are built, the rest in parallel.

### 2026-10-01 — P2-14 Geothermal features (agent worktree agent-abbb227c962473b98, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c; heading and first bullets restored 2026-10-02 in science stage 2, WP-15/16)
This entry's heading and first bullets were overwritten by a copy of the P1-10 entry when STATUS.md was merged. No commit of the geothermal branch holds them (the file's whole history and the unreachable objects of the repository were searched), so they could not be recovered verbatim. The heading is restored from the status-table row, and the bullets below up to "Gates" are written back from the code, FORMAT.md ("Geothermal features (P2-14) and their settings") and the plan's "As built" paragraph (section 10, P2-14). They are a reconstruction, not the original text. Branch `worktree-agent-abbb227c962473b98`, master e8668ca merged in at first.

- **Ask.** Ben, decisions log 2026-10-01 (the answer "Geothermal features" among four picked ideas, planned as P2-12 to P2-15) and 2026-10-01 ("Feel free to start now", on its own branch).
- **What was built.** Hot springs with sinter or travertine terraces where the baked rain and temperature allow standing water; fumaroles (magma-block vents ringed with native sulfur) and bleached or stained altered ground where it is dry, frozen or near the summit; all around the active volcanoes of `volcanoes/active.json`, the bake's only export of heat. Code in `worldgen/`: `Geothermal` (the sites, drawn from each volcano's id and position, the terrain height and the water columns), `GeothermalEnv` (rain, temperature, ice and carbonate under a site), `GeothermalRules` (the four settings), `GeothermalPalette` (vanilla blocks behind `deeptime:geothermal/<role>` tags with `geothermal_fallback` tags; travertine through the rock class), `GeothermalPass` (writes a chunk's blocks), called from `GeologicChunkGenerator`. Four settings in the `underground` section: `geothermal_density` (0 = none), `geothermal_reach`, `geothermal_size`, `geothermal_spring_min_precip_mm`; editor section "Advanced · Hot springs and fumaroles"; all eight built-in presets carry the defaults.
- **Warm-ground signal.** `DeepTimeClimate.geothermalWarmthC` (climate API 3): up to 12 K over the climate's temperature at the heart of a vent, 10 K at a pool, 6 K at a patch of altered ground. mic-climate does not read it yet (Pending question).
- **Measured** (16k quick planet): 260 sites, 23 pools, 142 vents, 95 patches around 40 of 67 volcanoes; JUnit `GeothermalTest` 14; the block census on the real generator found 0 leaks and 0 water on lava, and the standalone and MIC worlds agree site for site.
- **Gates** (VM, master fa09e8b merged in; after the second merge of master the cargo and Gradle gates again: cargo test 622 passed, 0 failed, 3 ignored, JUnit tests=214 failures=0; merged again with master 2e53ad2 (P2-08 impacts, the craters are in `elevation`, so the terrain height the sites read already holds them; no new Java pass): fmt, clippy, deny ok, cargo test 642 passed, 0 failed, 3 ignored, JUnit tests=217 failures=0, `./gametest.sh` and `-PwithMic` All 67 required tests passed (the test planet then had 35 sites, 4 pools; the earlier GameTest runs are before that merge, which brought docs, tools and the P2-01 work, nothing of this task's files): `cargo fmt --check`, `clippy --workspace --all-targets -D warnings` ok; `cargo test --workspace` 542
  passed, 0 failed, 3 ignored; `./gradlew build test` -> tests=214 failures=0 skipped=2; `./gametest.sh` -> All 66 required tests passed (518 s) and `./gametest.sh -PwithMic` -> All 66 required tests passed (MIC set of pack 790e116); in both, `geothermalAroundActiveVolcanoes` read 4 of the test planet's 53 sites (4 oceanic shields, 2 pools, 34 vents, 17 patches): the pool 46/46 water blocks, 30/30 beds, 35/35 rim columns, 0 leaks, 0 lava near, 28/28 ground columns; the vent and two patches 80/80, 71/71 and 172/172 ground columns.
- **Also seen, not changed.**
  - A volcano's field lands mostly in a forest or a snowfield; vegetation features then put trees on grass beside altered ground and
    their canopy overhangs it. Cold fields (mean under 3 C) have vents only, because vanilla's `freeze_top_layer` would put ice on
    a hot pool; hot-spring water is plain water (no steam or heat, vanilla has none).
  - Distant Horizons far terrain (P3-04) draws none of these: they are a few blocks wide.
  - Travertine is rare near volcanoes (1 of 23 pools): limestone seldom lies under the arcs; sinter is the usual terrace.
- **Contradicts the plan:** "from the heat flow already modelled": the heat flow is not exported (only volcanoes), so it is not used.
  Cinnabar is a stain at silicic volcanoes, not an epithermal deposit (P2-10 does not exist).
- **Next step.** Ben's answers to the questions below; then export a heat-flow layer and a rift/ridge source if wanted.
### 2026-10-01 — Meteor impacts (P2-08): craters that form, age and are buried; breccia, melt and impact deposits (agent worktree agent-a24e90e955a5a64d3, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Ask.** Ben, decisions log 2026-09-29 ("Yes, in full … not in v1, but before phase 3") and 2026-10-01 ("Feel free to start
  now, it can always just stay on a separate branch"). Branch `worktree-agent-a24e90e955a5a64d3`, master 38ca6b0 (with
  P2-01's folds and faults) merged in, not merged to master, not pushed.
- **The stage** (`crates/deeptime-core/src/surface/impacts/`, coupled to the plates after the volcanoes:
  `[plates, climate, surface, volcanoes, impacts, structure]`; the catalogue is `WorldState::impacts`, checkpoint schema 9).
  Each step: craters turn with their plates (oceanic crust carried under another plate takes its craters with it); each is
  worn by what the surface stage just did inside its rim (`surface/material_m`: deposition lays a cover over the floor,
  erosion strips the cover and then wears the crater, which keeps 1 − (cover + worn) / depth of its relief) and, between
  the cells, by hillslope diffusion (Σ κ·t on land and above the wave base); a crater worn through its depth and its
  breccia lens leaves the catalogue. Then the step's impacts: a Poisson draw with mean rate × `flux` × the planet's area ×
  Δt, each at a point uniform on the sphere with a size from the production function, turned into an impactor at Earth's
  gravity and back into a crater at this planet's gravity and the depth of the sea there. A new crater destroys smaller ones
  inside it. Nothing is stamped at the end: the amplifier draws a crater's aged profile times the relief it has left.
- **Scaling laws** (`scaling.rs`, cited there):
  - rate 5.6 × 10⁻¹⁵ craters ≥ 20 km per km² per year and N(≥ D) ∝ D⁻¹·⁸ (Grieve & Shoemaker 1994), constant over a run
    (Neukum, Ivanov & Hartmann 2001), on the planet's own area (7.69 million km² at 16k: 2.7 craters ≥ 2 km per Myr, one
    ≥ 20 km per 23 Myr);
  - transient diameter 1.161 (ρi/ρt)^⅓ L^0.78 v^0.44 g^−0.22 sin^⅓θ; final diameter 1.25 D_tc (simple) or
    1.17 D_tc^1.13 / D_c^0.13 (complex); rim 0.07 D_tc⁴ / D³; breccia lens 2.8 · 0.032 D³ (d_tc + h) / (d_tc D²), so a
    simple crater is D / 4.7 deep; complex depth 0.294 D^0.301 km; melt 8.9 × 10⁻¹² E sin θ spread over the transient
    crater; ejecta D_tc⁴ / (112 r³) (Collins, Melosh & Marcus 2005, after Schmidt & Housen 1987, McKinnon & Schenk 1985,
    Grieve & Cintala 1992, McGetchin et al. 1973);
  - simple-to-complex transition 3.2 km × 9.81 / g (Pike 1980); peak rings from 40 km × 8.87 / g (Venus: Alexopoulos &
    McKinnon 1994), the ring at half the rim's diameter; central peak 0.22 D across (Pike 1985), half the floor's depth
    high (a design value);
  - the sea: the transient crater keeps 1 − (H / L) / 6.3 of its diameter in water H deep for an impactor L across, none
    beyond 6.3 diameters: a linear fit to Davison & Collins 2007's three numbers (the sea removes two thirds of craters
    under 1 km, a third at 30 km, hardly any over 100 km), not their curves;
  - ageing: diffusion for a time t is a blur of variance 2 κ t (Culling 1960; Fassett & Thomson 2014), κ = 0.01 m²/yr ×
    the rock's `diffusivity`: a 2-km bowl fades in about 10 Myr, a 20-km crater in about a billion years.
- **Landform.** A crater at least two L1 cells across (9.6 km at 16k) is cut into the deep-time surface (offset,
  elevation and the plate feedback, as the volcanoes), so rivers, lakes and sediment see it and the surface stage fills
  and erodes it; the amplifier adds what L1 cannot hold of its aged profile (`amplify/impact.rs`: the profile less the
  profile blurred by 1.5 L1 cells). A smaller crater is drawn whole by the amplifier. Both on the last grid after the last
  flood, like the volcanoes' craters, so the bake's lakes decide whether a bowl holds water. Shapes (`shape.rs`): a
  parabolic bowl; a flat floor to 0.55 R with walls and a central peak; a peak ring at 0.5 R; rim and ejecta falling as
  r⁻³ out to `ejecta_radii`.
- **Rock.** Two classes, `impact_breccia` and `impact_melt` (after `greisen`; 62 in all, family `special_host`; stand-ins
  cobblestone and smooth basalt; breccia is a reservoir rock). A crater cut into L1 has its target removed in the strata
  log down to the base of the lens, the lens laid as breccia, the melt sheet (10 m and more) on the floor, and half the
  rise outside the rim as ejecta breccia. A smaller crater's lens becomes a body at the end (`impact_breccia`, an
  ellipsoid about the original floor, with a `melt_sheet` zone when it has one), under its cover when buried, cut when
  eroded to its roots. Shocked-target deposits are not modelled (the plan does not ask).
  **MIC rock-tag rows for the coordinator** (the pack and its datapack were not touched): Classes table, after `greisen`:
  `| Special hosts | impact_breccia | suevite, polymict lithic impact breccia (crater fill and proximal ejecta) | — | minecraft:cobblestone |`
  and `| Special hosts | impact_melt | impact melt rock, melt sheet (Manicouagan, Sudbury Igneous Complex) | — | minecraft:smooth_basalt |`;
  Fallbacks table: `| impact_breccia | minecraft:cobblestone | angular fragments of mixed rock in a matrix |` and
  `| impact_melt | minecraft:smooth_basalt | dark, fine-grained crystalline to glassy rock |`; Groups: `deeptime:rock/special`
  gains both, `deeptime:rock/igneous` gains `impact_melt`; tag files `rock/impact_breccia.json` and `rock/impact_melt.json`
  (`{"replace": false, "values": []}`), `rock_fallback/impact_breccia.json` (`["minecraft:cobblestone"]`),
  `rock_fallback/impact_melt.json` (`["minecraft:smooth_basalt"]`), `#deeptime:rock/impact_breccia` and
  `#deeptime:rock/impact_melt` appended to `rock/special.json`, the latter to `rock/igneous.json`; the README's counts
  become 62 classes, 46 fallback tags, 38 stand-in-only classes. Until then the game uses the stand-ins
  (`StrataGameTests` accepts either).
- **Deposits.** Four history fields added to every frame (`impact_melt_m`, `impact_hydrothermal`, `impact_shock`,
  `impact_basin`; `deposits/fields.rs`) and four types on them, each citing `deeptime:impact_crater` as its cause:
  `impact_ni_cu_pge` (Sudbury-type: a melt sheet of 500–1,500 m on continental crust), `impact_hydrothermal_cu_zn_au`
  (craters from about 8 km), `impact_diamond` (craters from about 30 km over gneiss, schist, black shale or coal),
  `impact_basin_u` (the fill of structures of 5 km and more). Crater-basin oil is not a type: the stage's closure field
  (`impacts/trap`, 0.7 inside complex craters) joins `trap_structure`, breccia is a reservoir, and `petroleum_trap` names
  the impact among its trap causes. No USGS model covers impact deposits: tonnages, grades and densities are design
  values, marked as placeholders (`data/usgs/impact_structures.json`). The types carry `minecraft.claims_scatter: false`
  (new key, default true), so vanilla diamonds and a pack's nickel, platinum and uranium keep their scatter.
- **Events and the workbench.** `deeptime:impact_crater` (time, place, diameter, morphology, impactor, energy, target,
  melt, fate, relief left; the event of a standing crater is where its plate has carried it) for every crater of 5 km and
  more and every smaller one that still shows, and `deeptime:impact_ocean` for a body the sea stopped that would have made
  20 km on land; written at the end of the coupled group in time order, like the volcanoes' (EVENTS.md). Layers
  `impacts/craters` (fresh, degraded, buried, roots) and `impacts/trap`; event colours, the four deposit families and the
  crater's facts in "Why here" (`inspect-column --why` lists craters as landforms). `impacts/craters.json` in the bake
  (FORMAT.md, "Impact craters"), read by Java's `worlddata/Craters`.
- **Settings** (the `impacts` section in all eight presets; editor fields and lang keys): Terrain: `flux` (1; 0 = no
  impacts), `depth_scale` (0.5). Advanced · Meteor impacts: `size_slope` (1.8), `min_crater_km` (2), `max_crater_km`
  (300), `speed_km_s` (20), `impactor_density_kg_m3` (3,000), `ejecta_radii` (3), `sea_cutoff_ratio` (6.3),
  `diffusivity_m2_yr` (0.01), `degradation` (1).
- **Measured** (test VM; `deeptime run --preset earthlike_quick_16k --start scattered_continents --length-myr 500
  --threads 4`, 506 s wall, peak 3.7 GiB; the impact stage 1.3 s of it; `tools/review/p2-08-impacts.sh`, pictures in
  `review/out/p2-08/q16/`):

  | | under 5 km | 5–10 | 10–20 | 20–50 | 50–100 | over 100 km | all |
  |---|---|---|---|---|---|---|---|
  | craters formed | 448 | 81 | 28 | 5 | 4 | 1 | 567 |
  | of them in the sea floor | 134 | 22 | 16 | 0 | 3 | 1 | 176 |
  | impactors the sea stopped | 637 | 89 | 9 | 0 | 0 | 0 | 735 |
  | eroded through | 354 | 62 | 16 | 5 | 1 | 1 | 439 |
  | subducted | 38 | 9 | 8 | 0 | 2 | 0 | 57 |
  | at the present: fresh | 6 | 1 | 2 | 0 | 0 | 0 | 9 |
  | degraded | 7 | 2 | 0 | 0 | 1 | 0 | 10 |
  | buried | 28 | 5 | 0 | 0 | 0 | 0 | 33 |
  | eroded to their roots | 15 | 2 | 2 | 0 | 0 | 0 | 19 |

  1,302 impactors (1,359 expected). 71 craters remain, 19 show (40 m of relief or more; 17 on continental crust; 7 with a
  dry floor above the sea), the oldest that shows 288 Myr old, the largest formed 112 km (eroded). 40 were cut into L1, 61
  left bodies, 132 events. Deposits: 10 impact-hydrothermal sites and 3 crater-basin uranium sites survive (72 and 81
  formed); 3 impact-diamond deposits formed and all eroded; no Sudbury-type (no melt sheet reached 500 m on land).
  `inspect-column --why` at a hydrothermal site: its chain is the 73-km crater, and the column holds `impact_breccia`
  under later shale and mud. Views: a 2.1-km simple crater 5 Myr old (7 blocks across, 6 deep in the terrain, fresh); a
  4.9-km crater 40 Myr old (degraded: 3.6 blocks deep, a small lake on its floor); an 11.9-km complex crater with its
  central peak; a 73-km peak-ring crater 5.5 Myr old in an active arc, a third of its relief left and hard to read among
  the ranges and cones (a flaw of looks, not of the model: the crater is shallower than the relief around it).
- **Tests.** Rust: `surface::impacts` (scaling 8: a 1-km asteroid makes ~15 km, Barringer ~1.2 km, simple craters a fifth
  as deep as wide, complex ones shallower, the transition and peak rings ∝ 1/g, melt sheets, ejecta ∝ r⁻³, the sea, the
  production function; shape 3; the stage 6: rate and size classes, state by cover, wear and age, fields, map, bodies,
  the pipeline's counts, fading with age, breccia in the log, events, no flux = no impacts and the same world as without
  the stage, threads and resume), `surface::amplify::impact` 3, deposit types and tables. Java: `CratersTest`,
  `RockDataTest` (62), `AdvancedSettingsTest`. GameTest `aCraterIsABowlOverBreccia` on the real test planet
  (`planet_world_4k.json` now runs impacts at 40 times Earth's rate, craters up to 40 km).
- **Gates** (test VM, on the branch with master 38ca6b0 merged): cargo fmt, clippy `-D warnings`, `cargo test
  --workspace --no-fail-fast` all pass (core 442 passed, 3 ignored; cli 54; format 63), `cargo deny check licenses bans`
  ok, `./gradlew build test` tests=203 failures=0. `./gametest.sh`: all 66 required tests passed, all mixins bound;
  `./gametest.sh -PwithMic` (after `tools/smoke/install.sh mic`, 168 mods installed, 160 linked, MIC rock tags): all 66
  passed. `aCraterIsABowlOverBreccia` in both: 194 craters on the test planet, 80 showing; crater 444 (2.1 km, 3 Myr) is
  5.5 blocks deep in the terrain with breccia in 5 of 5 columns under its floor. Before that merge, on master fa09e8b,
  the same gates passed (core 401, tests=202, 66 and 66), except that the first of two MIC runs died in mod loading
  before any test (Registrate's "Found unused register callbacks" in Create's register event, then a registry
  rollback); the same jars loaded on the next two runs, so it was not chased. The 16k numbers above were measured before
  the merge with P2-01.
- **Found on the way.** (1) The claim rules take every product of every deposit type, so adding `c:diamond` removed
  vanilla's diamond ore from Deep Time worlds (`claimRulesSortFeatures` caught it); hence `claims_scatter`. (2)
  `realPlanetHeightMatchesTheBake` reads a heightmap of chunks another test (`farTerrainMatchesGeneratedChunks`, at
  FEATURES) may already have generated further, where a tree stands 13 blocks over the terrain; which columns coincide
  depends on the planet, and the new planet hit one. The test now skips and counts such columns (`generatedFurther`).
- **Against the plan or the rules.** Nothing against the plan's P2-08. The GameTests were run on the branch because the
  task asked for them, against CLAUDE.md's "GameTests run once per merge batch". Oil in crater basins is closure and
  reservoir rock for the existing petroleum type, not a type of its own.
- **Not verified.** The constants of Collins et al. 2005 and the other papers were written from memory and checked for
  consistency by the unit tests, not against the papers (the paper could not be fetched); the one I am least sure of is
  the complex depth law (0.294 D^0.301 km; 0.4 D^0.3 is the other form in the literature). The rate was checked against
  the literature by a search, and the ocean rule against the paper's abstract only. No real client, no BlueMap, no
  browser look at the workbench (the scripts pass `node --check`). The 32k presets and other gravities were not run. Impact
  deposits form only where the crater's breccia is in the strata log, so not in craters under two L1 cells. Craters are
  not deformed by later folding, and an impact does nothing to the climate.
- **Scratch.** `/Volumes/Graphide/ban/deeptime-runs/p2-08/` (the run, 199 MB) deleted; the pictures are in `review/out/p2-08/q16/` on the laptop (not committed).
- **Next step.** Ben's answers to the five questions above; then the coordinator merges the branch after v1 lands, applies
  the MIC rock-tag rows, and runs both GameTest suites on master.

### 2026-10-02 — `plate-check`: the additions that need no new physics (task 1b of the force-balance design; agent worktree agent-ae6e5f954f5374acc, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c; not merged)
- **What.** Measurement only; no plate, collision or surface rule changed. `plates/dynamics.rs` (record version 2) now keeps,
  per plate and step: the change of the rotation vector since the previous step (`|Ω−Ω_prev|/|Ω_prev|`), whether the pole
  lies on the plate (the visible sample nearest to either end of the axis is the plate's), whether it lies within 30° of
  the plate's centroid (spinning), and whether the speed guard holds the plate (rotation at the cap; `PlateStep` is now the
  array `[id, visible, continental, speed, front, boundary, rot_change, flags]`). Per step: the sea floor's median and
  90th-percentile age and the counts older than 180 and 250 Myr (**every step**: one sort of the oceanic ages, no
  measurable cost; the histogram stays at 10 Myr), and the sizes of the 16 largest connected pieces of continental crust
  (lattice edges whose ends are still within 2 sample spacings, so a rift's fragments stop being one piece within a few
  steps). `record_step` takes the samples' positions (the only change outside the record, the check and the CLI: `mod.rs` passes `&pos`).
  `plates/check.rs`: 17 new ranged measures with the force-balance design's section 7 targets (area-weighted RMS speed;
  speed 10th/50th/90th percentile; oceanic ÷ continental and on-slab ÷ free median speed; Spearman rank correlation of speed
  with slab share and with area; plate-steps above 150 mm/yr; steps on which the guard binds (target 0, so the check fails
  whenever it binds); rotation change median and 90th percentile; sea-floor median age and share older than 180 and 250 Myr;
  supercontinent tenure and assembly-to-assembly period), 15 reported ones (spinning and pole-inside shares, continental
  pieces and largest share, assemblies and time assembled, rifts of a plate above 40 %, how long it stayed so, and whether the
  fragments are still apart 50 Myr later, strict and lenient, the continental extremes over all steps, the guard's plate-steps)
  and 8 placeholder rows for physics not built (`MeasureDef::pending`, `Status::Pending`, "no data", never fails): convergence
  20 Myr after onset ÷ at onset (0.2-0.6), wedge width (100-350 km), foredeep width (150-450 km) and depth (2-7 km), pull share
  of the driving torque, mean in-plane stress and tension share of a plate above 40 %, slab length per contact. They read empty
  record fields that the later tasks fill (`StepRec::forces: Vec<PlateForces>`, `StepRec::slabs_km`, `Dynamics::belts:
  Vec<BeltRec>`; a test fills them and the rows light up and are judged). The definitions are `check::Params` (assembled 70 %,
  dispersed under 50 %, mega-plate 40 %, plates of 30+ samples, 150 mm/yr, warm-up 100 Myr, start-up 10 Myr, piece 1 %, rift
  window 50 Myr), set by `plate-check --param name=value` (repeatable), `PARAMS="…"` in `plate-check.sh`, or a `"params"` object
  in the targets file; the table prints them. The checkpoint's postcard cannot skip fields, so the empty vectors are written.
- **The three open readings, settled** (docs/design/plate-check-baseline.md): continental crust judged on the lowest and
  highest step after the first 10 Myr (extremes over all steps reported; the start-up is not the cause, highest step is 0.48
  either way); ocean-floor age judged as the mean over the run after 100 Myr of the per-step 90th percentile (the last step's
  value reported; the snapshot is noisy, 208-269 across seeds); collision length first step to last step inclusive, not from
  contact onset, and `CollisionRec::start_myr`/`end_myr` documented as first and last step (`collision.rs`/`subduction.rs`
  not touched).
- **Baseline** (master 38ca6b0 + 1 + 1b; `earthlike_quick_16k`, supercontinent, 1000 Myr, seeds 1-4, `stage volcanoes`):
  docs/design/plate-check-baseline.md. 10 of 36 ranged targets met (collisions design 4 of 19, force-balance additions 6 of
  17), 8 rows wait for physics. Every figure of the first baseline reproduces to the digit; the new rows agree with the
  force-balance design's "Now" for speed percentiles (40/73/85), the oceanic/continental and slab/free speed ratios (1.64,
  1.61), rotation change (4.9 %/9.0 %), sea-floor age and the old-floor shares. New findings: the speed guard (100 mm/yr on a
  plate's fastest point) binds in 369 of 500 steps; the supercontinent (70 %/50 %) lasts a median 45 Myr and comes back after
  76 Myr, ten assemblies a run; a plate above 40 % of the planet is split by a rift after 18 Myr, 19 times a run, and 94 % of
  those rifts see a fragment rejoin within 50 Myr.
- **Cost.** The same 300 Myr run with and without the record (`DEEPTIME_NO_PLATE_DYNAMICS=1`), run at the same time on the
  test VM: wall 341.9 vs 343.0 s and CPU 539.3 vs 542.2 s (seed 1), 310.4 vs 309.3 s and 488.3 vs 488.8 s (seed 2): within
  0.5 %. (A first attempt with different seeds side by side differed by 15 %: the machine is shared.) Record 0.95-1.09 MB of
  JSON per 1000 Myr run (task 1: 0.9), kept in `manifest.json`.
- **Seed 1 and the structure stage.** On master `stage volcanoes` seed 1, supercontinent, 1000 Myr panics in the structure
  stage ("a history holds at most 128 events, the base stratigraphy included", `deeptime-noddy/src/history.rs:108`; P2-01 stage
  B). Not fixed here. The baseline's seed 1 ran with `structure.deformation.density=0`, which cannot change the plate record
  (nothing the plates read comes from the structure stage; seed 2 gives an identical `plate-check --json` with and without it).
- **Tooling.** `tools/remote/detach.sh start` no longer keeps the calling `mac.sh` waiting for the job: the job inherited the
  ssh session's descriptors (a nix lock file and a socket); the job's shell now closes descriptors 3-9 first (a 45 s job: the
  call returns in 6 s instead of 49 s).
- **Gates** (VM): cargo fmt --check clean, clippy --workspace --all-targets -D warnings clean, `cargo test --workspace`
  642 passed / 0 failed / 3 ignored, `cargo deny check licenses bans` ok, `./gradlew build test` BUILD SUCCESSFUL, JUnit tests=200 failures=0 skipped=1. No GameTests (nothing the game reads
  changed except the size of `notes.plates.dynamics` in the manifest).
- **Not verified.** The pole test is a nearest-sample test, checked on synthetic inputs and for consistency with the state, not
  against an independent trace; the design's "0.5-2 % turning about a pole near themselves" has no definition, so the
  spinning share (6.3 %) is not compared with it. The review's 360 and 650 Myr supercontinent figures have no definition
  either. Full presets (250,000 samples) were not run: the pole scan is 2 × plates dot products per sample and the pieces are one
  pass over the lattice edges, a few ms a step by estimate.
- **Next.** Task 2 (real shortening and the collision state) keeps `CollisionRec`'s `start_myr`/`end_myr` as first and last
  step and may fill `Dynamics::belts`; task 5 fills `StepRec::forces` and `slabs_km`. Someone owns the structure-stage panic.
### 2026-10-02 — Force-balance plate motion: design (agent worktree agent-a4560e6e131ce6cbd, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c; design only, nothing built)
- **Why.** Ben's decisions of 2026-10-02 on the collisions design: full force-balance motion; nothing designed for the
  mega-plate yet; both sides of a suture rise, with a foreland basin; the start-up land jump is a separate task.
- **Document.** docs/design/plate-force-balance.md (the collisions design now points to it and says which of its
  sections it supersedes): how plates move today, the force model with sources, an open-loop test, options, what is
  replaced, acceptance targets, the mega-plate physics, a nine-task plan, risks and fallback, five questions.
- **Measured** (test VM; `earthlike_quick_16k`, 1000 Myr, supercontinent and scattered starts, seeds 1 and 2;
  `R=… BIN=… tools/review/plate-dynamics.sh 1000 1,2 supercontinent,scattered_continents base` on a binary with the
  extended `tools/review/plate-dynamics-trace.patch`, then `tools/review/plate-forces.py <runs>`). The balance is
  solved on the present model's geometry each step, without changing the run (open loop); range over the four runs:
  - present model: plate speed 41 / 71 / 83 mm/yr (10th / 50th / 90th percentile), oceanic 73 against continental
    42–49, rank correlation of speed with the slab share of the edge 0.47–0.59, rotation change per step 5 % / 9 %
    (median / 90th percentile);
  - balance with basal drag only: correlation 0.76–0.84, but 13–19 % of plate-steps above 150 mm/yr (up to 2,500)
    and continental plates at 0–5 mm/yr;
  - with slab resistance and boundary resistance: 27–31 / 62–68 / 95–109 mm/yr, 1.3–3.1 % above 150, oceanic 66–72
    against continental 27–33, correlation 0.69–0.77; rotation change per step 16–18 % / 62–70 %; 13–17 % of
    plate-steps turn about a pole near themselves.
  So the pattern Earth shows comes out of a balance, small plates are the stability problem, boundary resistance is
  needed, and the raw answer needs smoothing (slab-length tracking, a relaxation time).
- **The scattered start has the same faults as the supercontinent start** (one plate above half the planet in 52–69 %
  of steps over the four runs).
- **Not tested:** the closed loop (the balance driving the geometry). The plan keeps a fallback: collisions and belts
  land first on the present speed law, the balance replaces the speed law last.
- **Estimate.** 19–21 agent-days beyond task 1, 13.5–15.5 on the critical path; a quarter is recalibration.
- **Housekeeping.** No simulation code changed. Scratch folder `/Volumes/Graphide/ban/deeptime-runs/plate-force-design/`
  deleted. Earth figures checked online by a Sonnet sub-agent; most papers were read from abstracts only.
- **Next.** Ben's answers (pending questions); `plate-check` additions (design section 7) for the agent on task 1.
### 2026-10-01 — `deeptime plate-check`: the plate dynamics record and its baseline (task 1 of the plate collisions and welding design; agent worktree agent-aa612c9a7664feba7, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c; not merged)
- **What.** Measurement only. The plates stage records `PlateState::dynamics` (`plates/dynamics.rs`), copied as JSON
  into `notes.plates.dynamics` beside `margins` every 10 Myr and at the end (format in the module docs and
  FORMAT.md): per step the plates (id, visible samples, continental samples, speed, slab front length, boundary
  length), land, continental crust and collisions; every plate's parent, origin, death and fate (`sutured` = weld,
  `absorbed`, `consumed` = subducted away) read from the plate events; one record per terrane transfer (contact
  onset, counter, real shortening, peak uplift, footprints of ground lifted by 10 m / 100 m / from below the sea
  to above it with length and width from second moments); the sea-floor age histogram every 10 Myr. `Contact`
  gains `cont_in` (continental samples that went under; counted only while recording; no rule reads it).
  `deeptime plate-check --run <checkpoint | world-data folder | manifest.json | note.json> [--run …] [--json out]
  [--targets file]` prints measure, value, target, pass/fail, exits 1 when a ranged measure fails or has no data.
  Ranges are `MEASURES` in `plates/check.rs` (the design's section 6 table), moved by a `{"key": [lo, hi]}` file.
  `tools/review/plate-check.sh` runs an ensemble. `plate-dynamics-trace.patch`, `plate-dynamics.sh` and
  `plate-dynamics.py` are removed.
- **Not changed.** No plate, collision or surface rule. The test `recording_does_not_change_the_world_and_matches_the_run`
  runs 300 Myr at 8,000 samples with and without the record and compares crust, elevation, plate ids, pools,
  contacts, rng, events and fields bit for bit; it also checks the record against the run's counters and events.
- **Baseline** (master fa09e8b + this; `earthlike_quick_16k`, supercontinent, 1000 Myr, seeds 1-4, `stage
  volcanoes`, 2 threads, ~15 min each at VM load 13-21): docs/design/plate-check-baseline.md. 3 of 19 targets met
  (FAILS as intended). It reproduces the design's "Now" column inside its stated ranges for every figure (largest
  plate 0.58, 59 % of steps above half, life 45 Myr, rift-born alive 19 %, welded back 42 %, land gain 3.75 %,
  collision belt 7,062 × 2,783 km, elongation 2.19, 10 collisions per 100 Myr, megaplate regrows in 17 Myr, active
  margins 0.30, ocean p90 208-269); seed 1's real shortening 87.4 / 269 km is the design's 87 / 269.
  Three readings to settle: continental crust "every step" (end values 0.38-0.45 agree with the design, the
  highest step is 0.45-0.49), ocean age read at the end of the run, collision length (one step today).
- **Cost.** Same 300 Myr run, with and without the record (`DEEPTIME_NO_PLATE_DYNAMICS=1`), two at a time on the VM
  and again with the seeds swapped: CPU 413.7 vs 413.0 s and 471.2 vs 470.5 s, wall 255.3 vs 254.8 s and 294.0 vs
  293.6 s: +0.15 %. The record is ~0.9 MB of JSON per 1000 Myr on the quick 16k planet (500 steps); it also goes
  into the game's manifest.json, which could be trimmed there if loading it ever matters.
- **Not cheaply available / approximations.** The active margin share is not per step: `margins` (every 10 Myr,
  `stats::margins`) has it and `plate-check` reads it. Slab share of a plate's boundary is an estimate (front samples
  × spacing over boundary samples × spacing). Passing the full speed-vs-slab judgement needs the force-balance
  model; today slab-attached plates are 1.4× faster than the rest (Earth 3-4×).
- **Noticed, left alone.** `subduction::resolve` adds relative speed, not the speed across the boundary, to
  `depth_km` (the design's bug). Checkpoints of the 16k planet are 1.1 GB each. `tools/remote/detach.sh start` kept
  the calling `mac.sh` waiting for ~10 minutes while the detached job ran fine (killed the local call, not the job).
- **Gates** (VM): cargo fmt --check clean, clippy -D warnings clean, `cargo test --workspace` 562 passed / 0 failed /
  3 ignored, `cargo deny check licenses bans` ok, `./gradlew build test` BUILD SUCCESSFUL, JUnit tests=199 failures=0
  skipped=1. No GameTests (nothing the game reads changed except `notes.plates` in the manifest).
- **Next.** Task 2 (real shortening and the collision state): keep `CollisionRec`'s fields' meanings (`start_myr` /
  `end_myr`, `shortening_km`, `peak_uplift_m`, the footprints).
### 2026-10-01 — Plate collisions and welding: design (agent worktree agent-a4560e6e131ce6cbd, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c; design only, nothing built)
- **Why.** The run review (docs/research/notes/run-review-1a0f969d358.md) found three plate-stage faults: land
  popping up at collisions, no Wilson cycle, a mega-plate. Ben: "Design, then fix".
- **Document.** docs/design/plate-collisions-and-welding.md: diagnosis, what Earth does (with sources), options,
  settings, acceptance targets, a seven-task build plan, four questions (under "Pending questions").
- **Measured** (test VM, master e8668ca, `earthlike_quick_16k`, supercontinent, 1000 Myr, `deeptime stage volcanoes`,
  2 threads, 1,030–1,120 s a run; `tools/review/plate-dynamics.sh 1000 1,2,3,4 supercontinent base` on a binary built
  with `tools/review/plate-dynamics-trace.patch`; mean of seeds 1–4 [range]):
  - one plate above half the planet in 59 % of steps [50–69], median largest share 0.58, worst 0.90; 3.75 plates
    above 5 %;
  - median plate lifetime 45 Myr [38–56]; rift-born plates 16–22 Myr, 19 % alive after 50 Myr; 100 welds per run, in
    half of them the dying plate is the larger one; about 40 % of rift-born plates are welded back into the plate
    family they left; after a rift splits a plate above half the planet, one plate is above half again in 17 Myr;
  - most land gained in a step 3.8 % of the planet [2.6–4.6]; one collision lifts up to 23 % of the planet [15–28] by
    more than 10 m and raises up to 4.5 % as land; every collision 1,400 m; lifted area 7,100 × 2,800 km, elongation 2.2;
  - a collision fires after a median 8 Myr of contact, when the counter reads 350 km and the real shortening is a
    median 87 km (seed 1).
- **Crust spreading did not change the three findings** (it and the ×0.7 uplift lowered the surge from 2,000 to
  1,400 m; areas, plate sizes and lifetimes are as the review found them).
- **Causes** (code). The collision counter in `plates/subduction.rs` `resolve()` adds relative speed × time, not
  shortening; `plates/collision.rs` then welds whole plates at once and applies a capped, 2,000-km-wide surge in that
  step.
- **Variants** (seed 1, one run each, `--set`): no welding (`collision_suture_frac=1e9`): one plate above half in 19 %
  of steps, plate lifetime 180 Myr, but P1-27's fault returns (active margins 0.67, land 0.46). Threshold 1,500 km:
  above half in 4 % of steps, rift-born lifetime 85 Myr, active margins 0.37, but continental crust falls to 0.29 of
  the planet (nothing resists a collision). Surge radius 500 km: land per collision at most 0.9 %, but the world ends
  with land 0.15: the wide surges build much of today's land, so the fix needs a hypsometry recalibration.
- **Recommended.** Uplift from the crust shortened each step, in a belt behind the front; a resistance that grows with
  the belt's height; plates weld once convergence has stalled after real shortening; the larger plate keeps its id. A
  supercontinent-heating rift term only if the mega-plate target is still missed.
- **Also found.** The first 4 Myr of a run add 4–6 % land with no collision (start-up). At 1000 Myr master's land
  share is 0.19–0.26 (the hypsometry targets are checked at 500 Myr).
- **Housekeeping.** No code changed. The trace is a patch file, applied only on the VM copy. Scratch folder
  `/Volumes/Graphide/ban/deeptime-runs/plate-collisions-design/` deleted. One mistake: two duplicate runs started
  with `STAGE=plates` (which runs the whole coupled group on this preset) were killed by PID after three minutes.
- **Next.** Ben's answers; then task 1 of the plan (`deeptime plate-check`, Sonnet).
### 2026-10-01 — P2-01 stage B: folds, faults and intrusions in the game's columns, and on the ground (agent worktree agent-a49703993341aa701, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
Ben (decisions log 2026-10-01): approach "Store structures, bend per column", defaults "Accept, judge from sections",
terrain "Terrain too". Not merged: the branch waits for the coordinator.
- **World data 0.3: `structure/events.dtst`** (`deeptime_format::structure`, FORMAT.md "Structural events"): the events
  youngest first, 136 bytes each (kind fold, fault or intrusion; window; age as a stratigraphic key), the fold profile
  table, and the candidate margin the bake measured. No compatibility with 0.2 (the standing rule).
- **The deformed column rule** (`Structure::deformed_stack`, Java `worlddata/Structure.java`): a column is cut into
  pieces as its candidate events are undone, youngest first. Per event and piece: the rock above the first horizon whose
  unit key is at least the event's key is *finished* (younger than the event: the unconformity, with no event of its
  own); a fold shifts the rest; a fault cuts it at its plane and the hanging wall reads the column it came from; an
  intrusion takes the part inside its ellipsoid. Writing the pieces out, a block that reads rock younger than the
  event that moved its piece shows the cover's lowest bed, so that bed fills the troughs of the folded surface it was
  laid on. (First version: the first bed *under* the cover stood in. Sections showed thin evaporite beds blown up into
  striped walls a hundred blocks tall; changed after looking.) Arithmetic `+ − × / √ floor ceil` only;
  `ColumnParityTest` compares 500 columns of the test planet, a quarter of them inside event windows, and the count
  of columns the events change. Readers switched: the strata pass, `Columns.sample`, `inspect-column`, the parity
  fixture, the workbench's sections, `strata-check`, the viewer's sections, `deeptime-inspect`.
- **Intrusions** (`deform/intrude.rs`): stocks in collision belts (granite) and arcs (granodiorite, diorite), basalt
  dykes along continental rifts, and a plug under every extinct volcano on land (rhyolite, andesite or basalt by the
  volcano). Each is an ellipsoid rooted on the world's floor (first version: a free ellipsoid; `strata-check` then
  found 1.1 % of columns with "plutonic rock with a floor", so they were rooted), dated like a fold: it cuts older
  folds, younger faults offset it, younger cover seals it. Stocks and dykes ride their plates; plugs are made at the
  end from the volcano catalogue. The batholiths of P1-07 stay bodies.
- **Traps** (`traps::structural`): closure comes from the world's own structures (a tight fold train up to 1, a
  thrust's hanging wall 0.8, a tilted fault block 0.6, a strike-slip fault 0.3, fading to the structure's edge), where
  there is the minimum sediment. **`fold_belt_km` was mis-scaled**: it is documented in Earth km (the Zagros) but was
  compared with a distance in km on the block metric, eight times too wide on a 16k planet. It is now compared on the
  plate model's sphere; the rule it belongs to is used only when deformation is off. Measured on the planet below, the
  share of continental cells with closure ≥ 0.5 (`notes.structure.deformation.traps`): 28.9 % from the world's
  structures, 29.2 % by Phase 1's rule as it was, 26.1 % by that rule with the width corrected. `petroleum_trap`: 421
  tracts, 29,938 km² permissive, 4 sites.
- **Terrain** (`surface/amplify/structure.rs`; the module comment says why there): on L2 before the amplifier's
  erosion passes, each young structure moves the ground as it moved the rock (a fault's hanging wall by the vertical
  slip, its foot wall the other way by `footwall_share`; a fold train by `fold_relief` of its shift), times what is
  left of its relief: 1 − age / `relief_age_myr` (10), times 1 − wear / (`relief_wear` (5) × its own height), wear
  being the |erosion or deposition| the deep-time surface did at its centre since it formed. Only land outside lakes,
  never below 1 m above the sea. One switch: `structure.deformation.relief` (0 = off). Rivers and lakes are the
  bake's, routed on the amplifier's output as before (sinks 0 with and without).
- **Event log**: one `deeptime:fold_belt` (folds, thrusts and stocks of one orogeny) or `deeptime:fault_system` (a
  rift's faults and dykes, or an oblique margin's strike-slip faults) per tectonic cause, with counts, median sizes,
  and its strongest structure's place and azimuth (EVENTS.md).
- **Settings**: `structure.deformation.relief`, `relief_age_myr`, `relief_wear`, `fold_relief`, `footwall_share`,
  `intrusions`, `stock_rate_km2_myr`, `stock_radius_km`, `stock_top_depth_blocks`, `dyke_rate_km2_myr`,
  `dyke_length_km`, `dyke_half_width_blocks`, `plug_radius_share`, `plug_radius_blocks`; ten of them in the editor
  ("Fault scarps and rift valleys" and "Stocks, dykes and plugs" among the multipliers), the lang file and the presets.
- **Review tools**: `deeptime structure-sections --world-data W --out D` (the largest collision belt, arc and rift of
  the event log, each cut across its strongest structure, drawn as laid down and with the structures, fault planes
  in red); `strata-check` gains `structure` (share of columns changed, malformed columns, bed offsets, repeated beds,
  intruded columns, and the same columns' findings without the structures); `tools/review/p2-01-planet.sh world`,
  `p2-01-report.sh` (numbers, hypsometry with and without the relief, a hillshade pair) and `p2-01-page.py`.
- **A cache fault found on the way** (not P2-01's, but it decided the cost): `TileCache` split its 256 MiB over one
  Guava segment per processor, so on a 32-processor machine a segment held 8 MiB and a decoded tile of horizon stacks
  (14.7 MiB) was dropped as soon as it was loaded; every chunk on a tile's edge decoded tiles again and again
  (`RasterSampler` kept one tile per thread). Now at most one segment per 64 MiB, and two tiles per thread. Plain
  column rule on the 16k planet: 1.52 → 0.44 ms a chunk on the test machine.
- **Measured** (test machine; `tools/remote/mac.sh -- tools/review/p2-01-planet.sh world on earthlike_quick_16k
  scattered_continents 500`, the same with `--set structure.deformation.relief=0` as `off`, then
  `tools/review/p2-01-report.sh on off`):
  - run: 9 min 13 s on 3 threads, 4.7 GB peak; the structure stage 72.7 s of it, the deformation steps 9.9 s and the
    end-of-run lattice and Noddy check 1.4 s;
  - kept: 1,081 fold trains, 3,848 thrusts, 1,256 normal faults, 261 strike-slip faults, 2,760 stocks, 2,647 dykes,
    804 plugs (12,657 events, 1.7 MB); 796 grouped events; Noddy port against the column rule on 7.8 M points: worst
    9.8e-6 block, 0 side flips;
  - lattice (L2): a structure acts on 51.9 % of columns, a fault plane crosses 37.1 %; 9.1 candidates a column on
    average, 106 at most; largest shift 47 blocks, largest sideways move 49 (margin 64);
  - where: 89 % of collision cells, 87 % of arc and rift cells, 70 % of craton cells and 73 % of passive-margin
    cells lie under a structure (52 %, 54 %, 51 %, 23 %, 28 % under one younger than 50 Myr);
  - `strata-check`, 20,000 columns: 32.0 % changed (51.9 % of land columns), 0 malformed; a bed's offset median 2
    blocks, p90 7; 70 columns meet a bed twice; 545 hold an intrusion. Findings with the structures against the
    same columns without: grade inversions 0.095 % against 0 (gneiss or schist over basalt and andesite: dykes and
    plugs under metamorphic rock, and thrust sheets), metamorphic slivers 0.175 % against 0, plutonic sheets 0.025 %
    against 0, plutonic rock with a floor 0.055 % against 0.015 %;
  - **cost in the game** (`ColumnRuleBenchTest`, Java, one thread, the 256 columns of a chunk; `DEEPTIME_BENCH_WORLD`
    points it at a planet): 16k planet, 12,657 events: 0.437 ms a chunk without the structures, 0.603 ms through them
    (+0.17 ms), slowest chunk 22 ms; 4k test planet, 328 events: 0.486 and 0.561 ms;
  - **terrain**: 268 structures have relief left. On L2 before the erosion passes 81,722 of 585,704 land cells move,
    by 12.8 m on average, at most 314 m down and 224 m up (`notes.amplify.structure_relief`). Baked terrain with the
    relief against without (cos-latitude weighted, 8 blocks a sample): land share 0.27698 against 0.27697; land
    median 661.1 m against 661.0; p10 90.7 both; p90 2,277 against 2,276; p99 4,368 both; mean 959.2 against 958.8;
    3.6 % of land differs by a block or more (56 m on average there); 104 cells of 583,476 became sea and 127 became
    land (the erosion passes after the term). Lakes 1,350 against 1,344, river chains 4,263 against 4,250, sinks 0.
- **Gates**: cargo fmt, clippy -D warnings, `cargo test --workspace` (0 failed in every crate; core 390, format 63),
  `cargo deny check licenses bans` ok, `./gradlew build test` 173 tests, 0 failures. GameTests on the branch (allowed for this task): `./gametest.sh` and `./gametest.sh -PwithMic` (168 MIC mods): all 60 required tests passed in each.
- **Left open** (none blocks the merge): bodies (batholiths, ore bodies) are painted where they are, so a fault
  younger than a body does not offset it, and the deposit engine's cover depth (`structure::Cover`) reads the
  undeformed stack; 70 % of craton cells lie under some old structure (stage A's number, accepted with "judge from
  sections"); stocks are slender spires in section where the world is deep (13 blocks of radius under 400 of height);
  the review page is built (`review/out/p2-01-page/`) but not published: `tools/review/publish.sh` was refused to
  this sub-agent.
- **Re-merged on master ea40d3b** (lava at volcanoes, height stage 2, the deposit audit, far terrain). Conflicts:
  STATUS.md and `main.rs` keep both sides; `TileCache.java` is master's (P3-04 made the same segment cap,
  `MIN_SEGMENT_BYTES`; this branch's copy is dropped, its two tiles per thread in `RasterSampler` stay). The new
  readers: the **magma pass** reads terrain height and the volcano list only, no rock columns; **`deposit-audit`**
  reads `ColumnWorld::sample`, which is the deformed rule; **far terrain** (`FarColumn.rock`) read the plain stacks
  and now goes through the structure rule: narrow columns by `Columns.deformedStack` exactly as the strata pass, wide
  ones by the same events undone over nearest-cell stacks (`Structure.deformedStack(…, cell = true)`).
  `FarColumnTest` follows. FORMAT.md's examples say 0.3 throughout; master added no format bump. Gates on the merged
  tree: fmt, clippy, `cargo test --workspace` 0 failed (core 398, cli 54, format 63), deny ok, `gradlew build test`
  200 tests 0 failures, `./gametest.sh` and `-PwithMic` 65 of 65 each. Column rule on the 4k test planet: 0.452 ms
  a chunk plain, 0.516 through the structures.
- **Next step**: the coordinator merges the branch and runs the GameTest batch; then Ben judges the sections.

### 2026-10-01 — P2-01 stage A: the fold-and-fault engine, standalone (agent worktree agent-a49703993341aa701, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
Ben (decisions log 2026-10-01): "can you start folds & faults rn (only do if it can be parellized with everything else)".
Stage A is the part with no dependence on other branches; stage B puts it into the baked columns.
- **The port: `crates/deeptime-noddy`** (no dependencies; LGPL crate, MIT source attributed in NOTICE.md and every
  file header). From Loop3D/noddyverse 6a21b2374516f8506c19cabcbe3ead00be9cc45d (MIT, checked in its LICENSE):
  `unEvents.c` (`unfold`, `unfault` with all five geometries and the shear-zone and kink forms, `unplan`, `undyke`,
  `unplug` with its four shapes, `unrot` for tilt and strain, `reverseEvents`), `taste.c` (the sequence codes,
  `taste`, `coincide`, `lineCoincide`, `cypherDiff`, `lastdiff`), `matrix.c`, and what those three call:
  `utils.c` (`convrt`, `rotset`, `irotate`, `matmlt`, `paxis`, `plane`), `ellFault.c`, `proflib.c` `calcProfile`,
  `which.c` `whichRock`/`whichLayer`, and each event's orientation set-up from `getEvent.c`. Not ported: `unbury`,
  `unloft` and `unbiot` (dead code in the original: commented out or never called), DXF surfaces, imported blocks,
  alteration zones, the geophysics half of `taste.c`. Two deliberate differences: `rotset`'s out-of-range `acos`
  is clamped (the original tests the wrong variable and can give NaN), and `matrix.c`'s `rotateX` is the rotation
  it meant (the original writes `[1][1]` twice).
- **Against the original.** `tools/validation/noddy/reference.sh` clones noddyverse at that commit on the test
  machine, compiles the six C files the harness reaches (`harness.c` supplies the event list the original walks)
  and writes `crates/deeptime-noddy/tests/data/reference.txt`: 159 cases (every event kind alone three times, 60
  random histories of 2 to 7 events), 40 random points each, with where the original restored each point, whether
  it stopped, its sequence code and what `taste` said. `tests/reference.rs`: all 6,360 points agree, worst
  difference 0 (bit for bit, x86_64 glibc; the tolerance is 1e-3 for other libms, because `rotset` takes `acos`
  next to 1). Three things the comparison found: `utils.c` has its own TOLERANCE (0.01, not `unEvents.c`'s 0.001),
  `rotset` needs the original's π literal to match to the last bit, and a harness that leaves
  `initSineFourierCoeff` unlinked computes shear zones from garbage (the script now refuses to run if a reached
  symbol is unresolved). `tests/analytic.rs` (11 tests): a cylindrical fold has its wavelength and amplitude and
  keeps bed thickness along the axial plane, a fold with a cycle halves in a cycle, a planar fault offsets a
  horizon by slip·sin dip and slip·cos dip (positive slip is normal, negative reverse, a reverse fault repeats the
  section), strike-slip, the order of composition, an angular unconformity, a plug and a dyke cutting folded
  layers, merged intrusions, a tilt, determinism.
- **Finite events (`deeptime_noddy::windowed`, Deep Time's addition).** Noddy's folds are endless trains and its
  planar faults endless planes (it models a 4 km cube). A windowed event is a ported event whose displacement is
  scaled by (1 − t²)² along and across a rectangle on the map: exactly zero outside, no table, no libm.
- **The column rule (`deeptime_format::structure`).** Not the plan's "lattice at bake, fitted back into the
  horizon stacks": the stacks are on L1 (16 blocks) and blended over 4 × 4 cells, so a fitted fault is a flexure
  three cells wide. Every event used moves a whole column at once (the window depends on the map position only;
  an upright fold shifts the column vertically by Noddy's sine profile; a fault moves what is above its plane by
  one vector), so a deformed column is a few `Segment`s, each a shifted copy of part of an undeformed column.
  `Structure::column` returns them; `StructIndex` says which events a column asks (those whose window reach plus
  the structure's `margin` holds it: a rule, not only a shortcut, so both sides will agree; the margin is at
  least 64 blocks and is raised at the end of a run until it holds the furthest sideways shift any chain of
  faults makes, with a quarter to spare, `deform::settle_margin`). Arithmetic: `+ − × /`, `floor`, and
  a stored 629-sample profile table. `structure/deform/tests.rs`: 120 random structures of every kind, 397,600
  lattice points with 22.7 events each: the column rule and the Noddy `History` of windowed events agree to
  2.3e-7 block.
- **Plate events → structures (`core::structure::deform`, in the structure stage each plate step).** Per
  continental L1 cell: the collision regime shortens at full strength; an arc or back-arc while the plate model
  lifts it (uplift over `arc_uplift_mm_yr`, 0.1) at half strength at 80 mm/yr; the foreland within `foreland_km`
  (250 km Earth-normalized = 102 blocks at 16k) of an active orogen, fading; the rift regime extends; a lifted
  margin converging more than 30° obliquely shears. Shortening starts **fold trains** (wavelength lognormal about
  16 km = 53 blocks, 2 to 5 wavelengths across, 5 to 12 long, amplitude λ·√e/π with e from the crust's thickening
  between 0.08 and 0.30) and **thrusts** (dip 20 to 40°, slip 0.08 × length, dipping under the high ground:
  towards the orogen in a foreland, up the regional slope inside it), both along `plates/fold_trend_deg`;
  extension starts **normal faults** (dip 55 to 70°, slip 0.05 × length) along the rift mask's long axis,
  dipping towards its axis, one in four antithetic; shear starts **strike-slip faults**. A Poisson process per
  kind (rate × strain × block-metric area × Δt), at most `max_overlap` (3) of a kind over a place, the oldest
  giving way to a new one 30 Myr younger (`overprint_myr`). Structures ride their plates with the plutons'
  rotation (azimuth turned by a second point), hold to their continent's edge when a boundary cell reads
  oceanic, and are dated; at the end they become `WorldState::structure` (youngest first, faults' traces at the
  present surface). Each names the nearest collision, orogeny phase, accretion, subduction or rift onset of the
  150 Myr before it as its cause (for stage B's events).
- **Settings** (`structure.deformation`, all in the four pipeline and four built-in presets): `strength` and
  `density` in the editor's Multipliers ("Folding and faulting", "Fold and fault count"); under Advanced · Granite,
  rock layers, folds and faults: size, fold wavelength, tightest folding, fold, thrust, normal and strike-slip
  frequency, fault length, block width, the three slip ratios, foreland width, overlap. 16 lang keys.
  `strength` 0 turns it off (no structures, the P1-07 layers).
- **Measured** (`tools/review/p2-01-planet.sh start q16h 500`: `stage structure --preset earthlike_quick_16k
  --start scattered_continents --length-myr 500`, release, 6 threads, test VM, after merging master): 310 s for
  the run, of which riding and translating the structures took 10.1 s over the 250 steps (40 ms a step: the
  strain of every cell, the orogen distance, the overlap scans; not optimised) and the end-of-run lattice and
  Noddy check 1.2 s; 8,843 structures started,
  1,303 overprinted, 80 refused (full), 1,094 lost at sea, **6,446 kept**: 1,081 fold trains (median wavelength
  52 blocks, amplitude 5.2, length 428), 3,848 thrusts (median slip 8.0 blocks), 1,256 normal faults (6.7),
  261 strike-slip; fault length median 128 blocks. Share of continental L1 cells under at least one structure,
  by what the crust is now (and with one younger than 50 Myr): collision 0.89 (0.52), arc 0.87 (0.54), back-arc
  0.83 (0.45), rift 0.87 (0.51), passive margin 0.73 (0.28), craton 0.70 (0.23). **Lattice**: the column rule
  at all 2,097,152 L2 columns (the whole height of each) in **0.05 s**; 52 % of the planet's columns hold a
  structure, 37 % a fault plane; 7.7 candidate events a column on average, 70 at most; at most 20 segments;
  largest shift 47 blocks, largest sideways 49 (inside the 64-block margin). The Noddy port on every 53rd L2
  column at every 4th block (7.7 M points, 9.8 events each): **124 ns a point** on one thread (13 ns an event);
  the whole L2 × 4-block lattice (520 M points) would be about a minute on one thread, ten seconds on six. The
  column rule on the same points: 6.6 ns a point. Agreement: worst 9.8e-6 block, no side flips. Memory: the
  structures and events are 1.4 MB; nothing is stored per lattice point. Peak RSS of the run 8.2 GB (7.9 to
  9.7 GB over seven runs of this planet today; a run without structures was not measured).
- **A second planet** (`start q16s2 500 supercontinent`): 1,101 fold trains, 3,988 thrusts, 1,311 normal and 173
  strike-slip faults; collision cells 0.90 covered, craton 0.72, rift 0.83, passive margin 0.76; largest shift
  75 blocks, largest sideways 72, so the margin settled at 109 blocks (11.5 candidates a column on average, 119
  at most, at most 25 segments); lattice 0.11 s for its two passes; Noddy agreement 9.5e-6 block, 154 ns a
  point; 10.7 s of translation in a 341 s run.
- **Gates** (test VM, after merging master 224f00c): `cargo fmt --all --check`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo deny check licenses bans` clean; `cargo test --workspace --no-fail-fast`
  all ok (core 384 + 3 ignored, cli 52 + 1, format 57, inspect 31, jni 7 + 7, noddy 16 + 11 + 2, viewer 6);
  `cd neoforge && ./gradlew build test` BUILD SUCCESSFUL, JUnit 168/0 (1 skipped). No GameTests: no baked data
  the game reads has changed yet.
- **Scratch**: /Volumes/Graphide/ban/deeptime-runs/p2-01/ (nine runs, 7.7 GB) deleted after the last one;
  `tools/review/p2-01-planet.sh start <name> 500` remakes a run in five minutes.
- **What stage A leaves out.** Rock younger than a structure is not deformed by it (unconformities): the rule
  needs the stacks' key at the event's age, stage B. Dykes, plugs and discordant plutons: bodies, stage B. The
  events are in `WorldState::structure` and the run summary only; the bake does not write them and the game
  does not read them yet. The event log has no fold or fault events yet (they will be grouped by belt, as the
  volcanoes were).
- **Found on the way.** (1) The plate model's back-arc regime reaches the whole subduction distance inland
  (1,800 km Earth-normalized = 737 blocks at 16k), so "back-arc" alone is no sign of deformation: the translation
  asks for uplift too. (2) `structure.fold_belt_km` (300, read by the trap structure field) is on the block
  metric: 1,000 blocks at 300 m a block, a tenth of the planet's width; the translation uses an Earth-normalized
  width of its own. The trap field was left alone (the deposit audit is in flight). (3) After 500 Myr most
  continental crust has been in a strain zone at some time (craton cells: 70 % under a structure), which follows
  from the plate model's ten collisions per 100 Myr (known: P1-10, P1-27); what a player sees there will be
  decided by the age rule, not by this number.
- **Decisions for Ben**: in the hand-back report (how structure is evaluated; default strength, count and size;
  whether old belts keep their structures; whether the ground shows fault scarps).
- **Next step (stage B)**: (1) `structure/events` in the world data (a sectioned file: the events and the
  profile table and the margin) and FORMAT.md, format 0.3; (2) `ColumnWorld::sample` and Java `Columns`/`StrataPass` restore
  the column first, reading each segment's rock from the stack at its source column, with the unconformity rule
  (the height of the stack at the key of the event's age: above it the rock is younger and is not moved); Java
  twin `worlddata/Structure.java`; `ColumnParityTest` over deformed columns; (3) the per-chunk cost of the
  rule in the game (the strata pass asks for a stack at every segment's source column); (4) the basement's
  granite as discordant bodies, dykes and plugs
  as bodies; (5) trap structure from the real folds and faults, deposits otherwise untouched; (6) `deeptime
  strata-check` and the workbench sections before and after through a collision belt, an arc and a rift; grouped
  `deeptime:fold_belt` and `deeptime:fault_system` events.

### 2026-10-01 — P1-23 grid-artifact audit (agent worktree agent-a851cf286456a3901, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Ask.** Ben, decisions log 2026-09-29: the finished world must not show the grid it was computed on. Branch
  `worktree-agent-a851cf286456a3901`, master e8668ca merged in, not merged to master, not pushed. The audit is
  `docs/research/notes/grid-artifact-audit.md` (method, every number, the pictures by path, options).
- **How.** One planet: `earthlike_quick_16k`, scattered continents, 500 Myr, seed 1 (333 s on the VM). New tooling:
  `tools/review/grid-audit.sh` (bake, dump, boxes, analyse, gates, clean), `tools/review/grid_audit.py` (the tests),
  `deeptime heightmap --columns` (rock at four depths, horizon count, second top, water kind per column; each thread
  opens the folder itself, because eight threads through one `LazyLayer` cache ran at 129 % CPU). Tests: gradient
  orientation (a4, a8, share within 2° of an axis), the phase of steps and curvature inside coarser cells with
  no-grid periods as control, the directional spectrum, a column/row outlier scan for seams, straight runs of class
  boundaries against a random map, the same on the terrain of each stage. Pictures in `review/out/grid-audit/`.
- **Clean (measured).** Seam: elevation step across it 0.95 block against 1.07 between interior columns. Slopes on land:
  a4 +0.017, within 2° of an axis 4.8 % (isotropic 4.4 %). Land/sea map: 12.6 % of edges in straight runs of 8+
  (random map 14.8 %, nearest-copied map 72 %). Rock every 8th block: 0.8–2.8 % (random 9–13 %). Cones: harmonics 4 and
  8 hold 2.0 % and 0.3 % of the variance round a cone (even share 8.3 %). Deposit sites and bodies are on no grid.
  Tile edges: none exist at 16k (`mode=in_memory tiles=1`); the streamed 32k amplifier was **not** tested.
- **Shows, worst first.**
  1. **Dams and straight lake shores on plains.** Ridges 3–8 blocks high and 150 to over 1,000 blocks long along the
     axes and diagonals, with lakes behind them. Already in the deep-time surface (L1). Read from `surface/spl.rs`: a
     lake cell traps sediment only from the flow routed through it, so the trunk path across a lake fills first. Open.
  2. **Straight rivers.** 10.2 % of the river length within 0.5° of an eight-direction heading (isotropic 2.2 %); 6.5 %
     in dead-straight reaches of 32+ blocks, longest 274; median sinuosity 1.035. D8 in the amplifier and
     `hydro::network`. Open.
  3. **Biome borders in 16-block steps.** Temperature input: 24 % of its change faces within 2° of an axis (4.4 %),
     N–S lines alone 9.3 % (2.2 %); vegetation 31 %. The band coordinates are set per L1 cell from its Köppen class.
     The east–west share on top is P1-26's latitude banding. Open, belongs with P1-26.
  4. **Square pits and benches where the rock changes.** Fixed (below).
  5. Coast pieces of 30–60 blocks along the axes twice as often as chance (4.3–4.8 % per axis against 2.2 % below 30°).
  6. Rock: the smoothing lattice (fixed); steps in the top unit's base along axes and diagonals (20.6 % within 2° of an
     axis where it drops 30+ blocks in 16; cause not found); boundaries on L1 lines in one flat box (cause not found).
  7. Octagonal inland bands of continentalness. Fixed.
  8. Curvature 19–24 % higher on every L2 and L1 line of the terrain (controls under 1.3 %); spectrum a8 +0.12 to
     +0.14 ± 0.03 at 4–32 blocks. Low.
  9. The detail noise: its three octaves share one lattice, so it changes 3.2 × less across lines 32 blocks apart; it is
     4.6 % of the block-scale relief. Low.
  10. East–west stretch towards the poles (slope energy x over z 0.54 at 60–75°): the projection, not a grid.
- **Fixed** (each with a property test that fails on the old code; both strata tests were run against the old weights
  and failed).
  - `surface/young.rs` `rock_property`: erodibility and diffusivity interpolated linearly from the L1 cells (they were
    looked up through `young/class`, L1's class copied into every L2 cell). `young/erodibility` steps across an L1 cell
    edge against inside the cell: 7.0 × → 1.02 ×. The crisp rectangles in `boxes/seam-terrain.png` become faint.
  - `structure/bake_horizons.rs` `smooth_axis_weight`: per axis a Gaussian of 1.5 σ less its value at the window's edge
    (it was a Gaussian of σ cut off at 2σ, 0.135 of its peak: the samples' mean position swung ±0.23 cells with the
    64-block lattice). Top unit's thickness, phase excess at 64 blocks: 15 % → 7 % (controls 2–6 %). Effective width
    0.85 σ (was 0.89 σ). FORMAT.md says so.
  - `bake/biomes.rs` `distance_cells`: exact Euclidean distance to the sea (was a 3 × 3 chamfer, 8 % out at 22.5°).
    Continentalness beyond 96 blocks from the sea: a8 −0.39 → 0.00.
  - Same planet before → after: land 27.7 % → 27.7 %, median land 715 → 715 m, highest 6,609 → 6,614 m, lakes
    997 → 1,025, river chains 2,369 → 2,345.
- **Gates** (VM, `tools/review/grid-audit.sh gates`): `cargo fmt --check`, `clippy -D warnings`, `deny` clean;
  `cargo test --workspace` 546 passed, 3 ignored; `./gradlew build test` tests=199 failures=0 skipped=1;
  `mac.sh --server -- ./gametest.sh` All 65 required tests passed; `tools/smoke/install.sh mic` (168 mods) and
  `./gametest.sh -PwithMic` All 65 required tests passed on the second run (the first never reached the tests: the
  known mod-loading race in Artifacts, "unbound value artifacts:generic.attack_burning_duration", outside Deep Time).
- **For Ben** (options and reasons in the note; not asked, the sub-agent has no AskUserQuestion).
  1. **Lake sediment**: fill each lake level, deepest first (recommended, 2–3 days); a delta from each inlet; or hide it
     in the amplifier.
  2. **Rivers and valleys**: route by a direction not bound to eight, in the amplifier and the river network
     (recommended, about 3 days, changes every valley); meander the river lines only (a day); or leave for Phase 2.
  3. **Köppen class**: decide it on L2 inside P1-26 (recommended); jitter the L1 lookup now (half a day); or leave.
  4. **Detail noise**: offset each octave's lattice when the format next changes (recommended); gradient noise; or leave.
- **Not verified.** Nothing was seen in the game (no client). One planet and seed. The 32k streamed amplifier, sea ice,
  MIC's biome mapping and Distant Horizons were not looked at. The cause of finding 1 is read from the code and the
  stage pictures, not proven by a run with the rule changed.
- **Housekeeping.** VM scratch `/Volumes/Graphide/ban/deeptime-runs/p1-23-grid` deleted. The workbench was not touched.
- **Next step.** The coordinator puts the four questions to Ben; finding 6's two open causes want an hour with
  `inspect-column` on the plain box (x 5632…6144, z −1536…−1024).
### 2026-10-01 — Magma under active volcanoes; the biomes' random lava removed (agent worktree agent-a04fab824846efbfc, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Ask.** Ben, decisions log 2026-10-01, "Volcanoes is a go ahead": lava where the simulation has it. Branch
  `worktree-agent-a04fab824846efbfc`, master merged in, not merged to master, not pushed.
- **The bake** writes `volcanoes/active.json`: the volcanoes the volcano stage leaves active at the present (id, kind,
  summit x and z, basal radius, height, activity 0–1, oceanic, silicic, age, crater radius and depth), in id order
  (`crate::underground::active_volcanoes`; FORMAT.md, "Lava"). Dead, eroded and subducted ones are not listed.
  **Spreading ridges and rifts are not exported**: they are plate-boundary geometry, not cheap, and nothing draws them yet.
- **In the world** (`worldgen/Magma` decides every block from the list, the settings and the terrain height, once per
  level; `worldgen/MagmaPass` places it in the surface step after the water pass).
  - **Chamber**: an ellipsoid of lava whose top is `magma_chamber_depth_km` (3 km = 150 blocks) under the crater floor;
    radius a tenth of the volcano's basal radius (1.2 to 4.5 km), half-height a fifth of that. In blocks it is a tall lens,
    because the world is 15 times taller than wide: 19 across and 57 high (9,500 blocks) under a 28-km arc cone, 31 across
    and 91 high (42,400 blocks) under the shield. Raised to stay 8 blocks over the lava floor, squeezed under thin ground.
  - **Conduit**: a plus-shaped column of lava from the chamber's centre to six blocks under the crater floor, then a plug
    of magma blocks: to the surface on land, buried three blocks under the bed under water.
  - **Crater lava lake** at the `crater_lake_share` (0.25) most active volcanoes that can hold one: radius half the
    crater's (2 to 10 blocks), lava up to one block under the lowest ground around it, the ground above opened to the sky.
    Not under or beside water, not beyond the pole cap's line, not where the summit is too sharp (a cut over 12 blocks).
  - **Sealed.** Open space within a block or two of magma becomes smooth basalt, and every block of a body and its
    shell is in the carving masks, so lava never touches the sea, a lake, a river, an aquifer or a cave. A volcano
    **under the sea or at the coast** keeps its chamber and conduit under at least 8 blocks of rock, gets no lava lake,
    and its plug stays buried. Dead volcanoes get nothing.
- **Random lava removed** (`underground.remove_lava_features`, on by default). Traced first, in the 168-mod MIC set and
  vanilla: `minecraft:lake_lava_surface`, `lake_lava_underground`, `spring_lava`, `spring_lava_frozen` (in nearly every
  overworld biome, Terralith's included) and Terralith's `volcano/lava_lakes`, `volcano/lava_flow`, `volcano/spring_lava`,
  `cave/mantle/lava_drip`. `worldgen/LavaFeatures` refuses any placed feature whose configuration holds a lava block or
  fluid state, where it is placed (`PlacedFeature.placeWithContext`, one small mixin), for Deep Time generators only.
  Not a biome modifier: those run once per server, before a new world's data is in place, and for every dimension.
  - **Terralith's `cave/frostfire/lava_remover` is refused too.** It turns lava sources into deepslate, 300 blobs of
    radius 10 to 12 per chunk below y 30 in frostfire caves, and ate the chamber of a submarine volcano under such a
    cave (4,226 of 4,521 blocks became deepslate in MIC; standalone kept all). It also explains the deepslate seen in
    the flooded zone in the first lava entry.
- **Settings** (preset section `underground`, all eight presets; editor section "Advanced · Lava and magma"):
  `magma_chamber_depth_km` (3.0), `magma_chamber_size` (1.0; 0 = no magma), `crater_lake_share` (0.25),
  `remove_lava_features` (true). `lava_floor_blocks` and `lava_lake_blocks` are unchanged (10, 54).
- **Evidence** (a fresh 16k quick planet, supercontinent, 500 Myr, baked by this branch: 123 active volcanoes, 119 arc
  stratovolcanoes, 3 caldera complexes, 1 shield; `tools/workbench/preview.py run`, seed 20261001, three 256 × 256 × 992
  boxes per profile; screenshots, censuses and the probes in `review/out/lava-volcano/`).

  | Box | MIC | Standalone | What the lava is |
  |---|---|---|---|
  | arc stratovolcano 12529 (crater lake, summit y 190) | 10,300 | 10,300 | one body of 10,239 from y −16 to 182 (chamber, conduit, lake); 29 faces open to the sky, the lake's surface; 61 blocks at the lava floor |
  | shield 12523 (summit 81 blocks under the sea) | 43,188 | 45,677 | one body of 43,051 from y −258 to −24, 0 faces on water; the rest lava floor and lava-lake aquifers below y −300 |
  | oceanic arc volcano 12565 (summit at y −91, under frostfire caves in MIC) | 5,709 | 6,747 | one body of 5,181 from y −285 to −97, 0 faces on water; the rest below y −300 |

  Lava above y −288 that is not a volcano's: 0 blocks in all six boxes. Lava touching water: 0 faces. The bodies are
  block-for-block the same in both profiles.
  - **What it means for a player** (`/deeptime probe magma`: 4,000 land points drawn evenly inside the pole cap). On
    this planet 100 of the 123 active volcanoes stand on land and 16 hold a crater lake.

    | From a random land point | p10 | median | p90 |
    |---|---|---|---|
    | to the nearest active volcano on land (lava six blocks under the plug in its crater) | 169 | 658 | 1,309 blocks |
    | to the nearest open crater lava lake | 380 | 1,178 | 2,251 blocks |
    | straight down to the lava floor (y −342) | 411 | 438 | 510 blocks |

    Vanilla has lava within about 100 blocks of anywhere. That gap is the pending question above.
- **Tests.**
  - JUnit `MagmaTest` (8, cones in closed form, no planet): chamber, conduit and plug where the rule puts them; the most
    active volcanoes get the lakes and the ring holds them; **every molten block's six neighbours belong to the body**
    (only a lake's surface faces air); under or beside water no lake and a buried plug; chambers stay between the ground
    and the lava floor; bodies across the seam; the settings and the list; what makes a lava feature.
  - Rust `underground::tests` (3): only the volcanoes active at the present are listed.
  - GameTest `magmaUnderActiveVolcanoes` (the level with caves and aquifers): up to four of the planet's bodies read
    back block by block at the surface step: the molten blocks are lava, no lava outside a body above the lava-lake
    zone, no lava face on water or on air except a lake's surface, the lake open to the sky. A planet without an active
    volcano passes with a line saying so.
  - GameTest `lavaFeaturesStayOut`: every placed feature is asked; vanilla's four are refused for the Deep Time level and
    placed for any other; water springs, ores and trees are untouched; with Terralith, its four and the lava remover.
  - `lavaStaysAtTheFloor`, `riversHold`, `carversSpareWater`: in the suites below.
- **Gates** (VM, master merged in, tree 2e49b4d): `cargo fmt --check`, `clippy -D warnings`, `cargo deny` ok; `cargo test
  --workspace` 533 passed, 0 failed, 3 ignored; `./gradlew build test` → tests=180 failures=0; `./gametest.sh` → All 62
  required tests passed (544 s); `./gametest.sh -PwithMic` → All 62 required tests passed (the MIC set is a copy of the
  workbench's, pack 790e116). In both, `magmaUnderActiveVolcanoes` read three shields of the test planet (a crater lake,
  one under water, one on land): 43,075 / 42,881 / 43,051 molten blocks, all lava, `stray=0 wetFaces=0 airFaces=0`; with
  MIC `lavaFeaturesStayOut` refused 14 of 932 placed features (vanilla's and Terralith's, the lava remover included).
  After the carving-mask change the lake volcano was generated again (MIC): the only lava faces on air are the 29 of
  the lake's surface (before it, a carver left 41 more at the chamber's side).
- **Also seen, not changed.**
  - **Baked summits are not bowls.** At every lake volcano the ground three to five blocks from the listed summit lies
    1 to 11 blocks below it, and in the section through 12529 the cone's highest point stands beside the listed column.
    So the lake sits in a pit the pass cuts, beside the peak, and a summit too sharp for that gets no lake (about 62 of
    the 100 land volcanoes can hold one). The amplifier's crater and the catalogue's position should be compared at L3.
  - Activity saturates at 1.0 for most arc volcanoes, so the lakes go to the lowest ids among them.
  - 13 of the first 20 lakes stood within 160 blocks of the pole edge (an arc along the edge of the map) and 2 more
    beyond the pole cap, which is why lakes now stop at the cap's line.
- **Contradicts the plan:** nothing in the plan places lava; §7.1 keeps vanilla's aquifer and features.
- **Next step.** Ben's answer to the lava question above.
### 2026-10-01 — The height limit, stage 2: crust over its strength spreads; the plate model's cap is a guard (agent worktree agent-ac2a33b7f51fa4bf8, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Why.** Ben (decisions log 2026-10-01): "Build it out": the height limit and its dependence on gravity come from
  the simulation, not from a number in the plate model; the world's height may rise if the model needs it.
- **The mechanism** (`surface/spread.rs`, step 5b of the surface stage, after isostasy). Where the weight of the land
  above the sea, ρ_c·g·h (ρ_c 2,800 kg/m³), passes the crust's strength S, the surface relaxes sideways as the lower
  crust flows out from under it: ∂h/∂t = ∇·(D ∇h), D = D₀·min(x, 2)³, x = max(0, ρ_c·g·h/S − 1). Conservative
  and implicit (the surface's own diffusion solver), solved only when some cell is over the threshold. It moves
  crust, not surface rock: strata log and mass residual untouched; the plate model takes it through the feedback.
  Nothing is clamped. Settings: `tectonics.crust_strength_mpa` (140: 5.1 km at 1 g) and `tectonics.crust_flow_km2_myr`
  (D₀ at twice the strength height; 30,000), both in the editor (Advanced · Plates, `crust_strength`, `crust_flow`)
  with lang keys.
- **The plate model's cap is demoted.** `tectonics.max_elevation_m` is 30 km in the pipeline presets, a guard that
  bound 0 continental sample-steps in every run below. The uplift ceiling (uplift × (1 − z/10 km), which did the
  work before) is its own setting, `uplift_ceiling_m`, and 0 (none) in the pipeline presets; Cortial's z_c as the
  scale of the subduction transfer function is `uplift_reference_m` (10 km, a scale only). The struct's defaults and
  the plate-only presets keep the old behaviour (10 km, ceiling on, no spreading): they have no surface stage.
- **Volcanoes.** The ceiling (`volcanoes.max_elevation_km`, 6.8) is now that height times the planet's strength
  height over Earth's with the default crust: unchanged at 1 g and 140 MPa, 17.9 km at 0.38 g, 3.4 km at 2 g.
- **Recalibration.** Without the uplift ceiling the land ran high with strong spreading alone (land 0.317, median
  768 m, above 2 km 0.163, n = 4), so `subduction_uplift_mm_yr` 0.6 → 0.42 and `collision_uplift_max_m` 2000 → 1400
  (×0.7) in the four pipeline presets and the built-ins (and the test planet, 4000 → 2800). `tools/review/p1-17-hyps.sh`,
  500 Myr, `earthlike_quick_16k`, area-weighted, mean [min, max]:

  | | n | land | median, m | p90, m | above 2 km |
  |---|---|---|---|---|---|
  | Earth | | 0.289 | 427 | 2,022 | 0.102 |
  | targets (recalibration 8600e03) | | ≈ 0.29 | 430–640 | | 0.10–0.13 |
  | before: stage 1 + recalibrated presets | 6 | 0.292 [0.283, 0.307] | 695 [604, 800] | 2,444 | 0.151 [0.115, 0.198] |
  | no ceiling, D₀ 3,000 | 4 | 0.277 [0.243, 0.318] | 601 [481, 803] | 2,423 | 0.139 [0.095, 0.187] |
  | no ceiling, D₀ 30,000 | 4 | 0.317 [0.248, 0.396] | 768 [513, 1001] | 2,496 | 0.163 [0.101, 0.225] |
  | **the presets now: D₀ 30,000, uplift × 0.7** | 6 | 0.273 [0.234, 0.310] | 547 [436, 701] | 2,102 [1,789, 2,392] | 0.114 [0.085, 0.152] |

  Histories are chaotic: with n = 4–6 a mean moves by about ±0.02 in land and ±100 m in the median between batches.
- **Heights, deep-time surface on L1** (same runs; seeds 1–3 × both starts, "before" has seeds 1, 2 from the first
  batch and seed 3 with the old settings on the new binary):

  | | max, km | p99 land (cells), km | cap bound |
  |---|---|---|---|
  | before | 6.4–7.8 | 4.3–5.1 | 0 |
  | now | 5.7–6.8 | 4.2–5.5 | 0 |
  | now, 0.38 g (n = 4) | 9.5–11.0 | 3.4–5.7 | 0 |
  | now, 2 g (n = 4) | 3.42–3.43 | 2.9–3.1 | 0 |

  At 2 g everything above 2.55 km spreads (2–3 million km³ moved per run against 0.2–0.7 million at 1 g) and the
  highest ground is the volcanoes at their 3.4 km ceiling. At 0.38 g the strength height is 13.4 km and **no cell
  ever reached it** (spread 0): ranges there are limited by how fast uplift feeds them against erosion, which does
  not depend on gravity in this model, so their p99 is the Earth-like one; the 9.5–11 km summits are volcanoes under
  their higher ceiling. So gravity lowers mountains on heavy planets and frees volcanoes on light ones, but a light
  planet's fold belts are not markedly taller: that would need erosion (or uplift) to know gravity. For Ben.
- **Plateaus** (`deeptime stage volcanoes … --out`, seed 1, supercontinent; `review/out/height-stage2/map-{old,new,g038}.png`,
  sections through the highest cell, km per L1 cell of 4.8 km). Before: a dome, `… 4.2 5.1 5.6 5.9 6.1 6.3 6.3 6.4 6.3
  6.2 6.1 6.0 5.9 …`. Now: a table with steep edges, `0.8 1.0 2.3 3.2 4.5 5.4 5.6 5.6 5.6 5.7 5.7 5.7 5.7 5.6 5.6 5.6 5.5
  5.3 5.1 4.8 …` (1 km to 5.4 km in five cells, then 13 cells within 0.2 km). Land above 5 km: 0.30 % → 0.42 %,
  nothing above 6 km. At 0.38 g the highest ground is a cone, `… 5.9 7.1 8.2 9.1 9.4 9.2 8.6 7.7 …`.
- **The finished world** (`deeptime run`, amplifier and bake included, the six default runs; H every 8th block):
  highest terrain y 365–409 = **6.0–6.9 km** (6.92, 6.17, 6.14, 6.03, 6.43, 6.45), p99.9 5.2–6.2 km, p99 4.0–5.6 km;
  0 cells above the compression knee (y 575), 0 compressed, in all six. The amplifier adds at most a few hundred
  metres to the L1 maxima. **So the compression band does not shape Earth-like worlds, and the world height does not
  need to rise for them**: the highest block is 166 blocks under the knee.
- **Run time.** The spreading solve is 22–35 s of a 190–240 s surface stage over 250 steps (about 13 %, 4–5 % of
  the 500-Myr run of ~580 s at 2 threads); more than the few per cent I estimated, because the solve runs over the
  whole grid. Restricting it to the cells over the threshold and their surroundings is the obvious saving.
- **World height, the numbers asked for.** Minecraft allows a dimension from y −2032 to 2031 (`min_y` and `height`
  multiples of 16, height ≤ 4064); NeoForge adds no limit. Deep Time's `planet_overworld` is −352..639 (62
  sections). Not needed for default worlds (above). It is needed for what low gravity can build: at 0.38 g volcanoes
  reach 11 km on L1 (ceiling 17.9 km = y 958), which today is squeezed into the 48-block band under y 623.
  Options: **(A, recommended) leave 639** and let the band take low-gravity summits until someone wants Mars-like
  worlds; (B) `max_y` 1023 (86 sections, +39 %): holds 19.2 km, costs every world's noise fill, lighting and
  heightmaps in proportion to the extra 24 empty-air sections per chunk (not measured; the dimension type and noise
  settings are one JSON each, the format already carries `max_y`, existing test worlds would need regenerating);
  (C) a per-preset height (the dimension type chosen from the preset's gravity): only tall planets pay, but the
  dimension type becomes dynamic, which Toroidal World and the client-info channel would have to follow.
- **The bottom** (the lava work found seabeds under the world's floor). In the six default runs the deepest finished
  sea floor is y −412 to −472, 9.5–10.7 km under the sea (`tectonics.trench_elevation_m` is −10 km), against `min_y`
  −352 (8.3 km): 0.01–0.08 % of the sea's cells are below the floor and 0.14–0.49 % in its lowest 32 blocks, the
  router's bottom slide. Nothing squeezes the bottom as the band does the top. Options: **(A, recommended) `min_y`
  −512** (72 sections instead of 62, +16 %: 11.5 km of sea, real trenches fit with the slide), with the same
  smooth squeeze above the floor as a net for deeper settings; (B) the squeeze only, at −352: no cost, trenches
  flatten gently at 8 km; (C) leave it: flat trench floors at bedrock in under 0.1 % of the sea. Either way existing
  test worlds are regenerated; the format already carries `min_y`.
- **The full preset** (`earthlike_full_16k`: 250,000 samples, L1 r 8; the ensemble Ben approved, 2 seeds × 2 starts,
  500 Myr, 2,600 s a run at 2 threads): land 0.327 [0.266, 0.359], median 746 m [652, 861], p90 2,491 m, above 2 km
  0.148 [0.099, 0.168]; L1 max 6.6–6.8 km, cap bound 0. Still high (one run before all of today's changes had land
  0.38, median 1,013 m). With uplift × 0.8 on top (`subduction_uplift_mm_yr` 0.34, `collision_uplift_max_m` 1100; n = 4): land 0.292 [0.257, 0.319], median 630 m [577, 675], p90 2,049 m, above 2 km 0.106 [0.082, 0.131], L1 max 5.7–6.8 km: inside the targets, so the two full presets carry these values (the quick ones keep 0.42 and 1400).
- **Not in these runs:** the folds-and-faults term (worktree-agent-a49703993341aa701), which is on its own branch
  and switchable; the combined effect is to be re-checked after both land.
- **Tests.** `surface::spread` (a 9 km block on a plain sinks to between the strength height and 7.5 km and widens,
  volume kept, nothing moves at 0.38 g or with the flow off), the editor's catalogue test covers the two new
  settings in every pipeline preset. `riverBiomes` failed on seed 16's new planet (12 river of 37 wide-channel
  samples, 21 of them swamp): vanilla's own valley slice is swamp on the flattest, wettest ground, so swamp and
  mangrove swamp count with river.
- **Measured** (VM `deeptime-test`, x86_64 Linux, `mac.sh` default caps): on the tree merged with master e6a7af0 (load average 13–25): `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo deny check licenses bans` clean; `cargo test --workspace --locked` all pass (533: cli 52, core 374 + 3 ignored); `./gradlew build test` tests=168 failures=0 errors=0 skipped=1; `./gametest.sh` All 59 required tests passed (wall 455 s, bake `above_max_y=0cells compressed=0cells highest_y=280`); `tools/smoke/install.sh mic`, `./gametest.sh -PwithMic` All 59 passed (89 s). The nine real-planet tests on other planets (`DEEPTIME_TEST_PLANET_SET="seed=N"`), one merge earlier: seeds 12–15 standalone All 9 passed, seed 16 failed `riverBiomes` (above) and passed after the fix, seed 13 again All 9. A MIC run one merge earlier failed `firstInstalledFallback` (shale and gneiss): master's own fix 13b8657, in the final tree. After the gates only the two full presets' uplift values changed; `cargo test` and `./gradlew build test` were run again on that (all pass, 168/0). The laptop's address changed twice during the runs and the VM was unreachable for minutes at a time; the detached jobs were not affected. x86_64 Linux only.
- **For Ben.** (1) Earth-like summits are 6.0–6.9 km in the finished world, under the 8–10 km he expected: the
  plateau level is where Tibet's is and the highest ground is plateau, not peaks, because neither the surface stage
  nor the amplifier puts 3 km of summit relief on a plateau. Options: **leave it and give the amplifier summit
  relief over high plateaus (recommended; the mountain-character work)**, or raise `crust_strength_mpa` to about
  190 (plateaus near 7 km, summits 8–9 km, less like Earth below the summits). (2) Light planets: taller volcanoes
  but not taller fold belts (above); making erosion or uplift depend on gravity is its own piece of work.
  (3) The world's bottom, options above. (4) The top stays at 639 unless Mars-like worlds matter now.
- **Next.** The full preset's own uplift factor (one more ensemble); the spreading solve restricted to the cells
  that need it; the hypsometry and folds branches re-checked on this base.

### 2026-10-01 — Rock columns: grade rises downwards, no metamorphic or plutonic slivers; `deeptime strata-check` (agent worktree agent-a4c7bf65d55bc4810, branch worktree-agent-a4c7bf65d55bc4810, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c; not merged)
- **Ask.** Ben, 2026-10-01 ("Investigate and fix"), on the workbench run `1a0f6e918c9` (`earthlike_quick_16k`, supercontinent,
  1000 Myr): at x −6901, z −128 schist lies on unmetamorphosed shale (and again deeper), gneiss is a 3-block sheet between
  greenstone and granodiorite, and granodiorite and gabbro lie in the pile like beds. A second column from the coordinator
  (x −7697, z 46, a 3.7 km mountain): slate over unaltered sandstone, grade falling under the gneiss, many 1–5 block slivers.
- **The measurement** (`deeptime strata-check`, new; `structure::realism`). Three findings a column can have:
  a **grade inversion** — a horizon whose class was never as hot as a horizon above it formed at (each regional-metamorphic class
  has its formation temperature, each class with a metamorphic path the temperature of its first step: schist ≥ 450 °C over shale
  < 250 °C; cover on basement is the other way round and fine; a plutonic horizon is never the lower member, steps under 200 °C
  are diagenesis); a **metamorphic sliver** — a regional-metamorphic horizon under 8 blocks with other rock over and under it
  (one cut by the ground or the floor is not one); **plutonic rock as a bed** — an intrusive horizon with rock of another
  family under it ("with a floor"), a *sheet* when it is under 16 blocks. `--world-data <folder>` checks the columns the game
  builds (the column rule under the terrain, 20,000 random blocks, seed 1) and the stored stacks cell by cell;
  `--checkpoint <file>` the stacks before and after the bake's smoothing; `--column X,Z` prints stacks (with `--checkpoint`:
  the cell's log, pools, units and keys). The bake writes the stored stacks' shares into `notes.bake.strata.realism`.
- **Before** (VM; the workbench bake, and the same preset re-run on master efafc88 as `big-base`: identical shares and column):

  | share of columns | grade inversion | metamorphic sliver | high-grade sliver | plutonic sheet | plutonic with a floor | horizons / column |
  |---|---|---|---|---|---|---|
  | 1000 Myr supercontinent, all (20,000) | **57.1 %** | **56.3 %** | 35.5 % | 10.7 % | 28.6 % | 15.4 |
  | … land (8,384) | 71.5 % | 73.9 % | 46.6 % | 15.3 % | 45.0 % | 23.0 |
  | … stored stacks (524,288 cells) | 56.7 % | 56.0 % | 33.9 % | 8.7 % | 27.0 % | 13.9 |
  | 300 Myr scattered, seed 7, all | 26.2 % | 36.8 % | 24.8 % | 7.7 % | 13.1 % | — |
  | … its stacks **before the smoothing** | 0.73 % | 3.97 % | 3.65 % | 0.09 % | 0.71 % | 4.5 |
  | … its stacks after the smoothing | 26.6 % | 37.0 % | 24.3 % | 6.4 % | 12.1 % | 8.2 |

- **Causes** (read from the code, each shown by a measurement or a dumped stack).
  1. **The key space ordered the strata log by age alone, and the bake's smoothing (σ 10 cells) and the column rule blend
     neighbouring stacks by key.** Metamorphic grade is not a function of age: a belt's schist (buried deep, then exhumed) got
     the same keys as the shale its neighbours laid down at that time and never buried, so the blend put them in one sequence
     — schist over shale. One cell's own log cannot do it (a layer's class steps by the temperature at its depth, deeper is
     hotter, classes never go back): 0.7 % of the unsmoothed stacks have an inversion, 26.6 % of the smoothed ones.
  2. **Merges took the union of key spans** (`settle`, `bake_cell`, `columns::push`, the smoothing's join): a metre of soil
     merged into kilometres of basement schist gave the schist the keys 0 … 49,663 — the regolith's, the whole log's and four
     pools' — and its height became a straight line over keys that neighbours give to other rocks. Ben's top horizon is exactly
     that: `schist`, unit key 0 (`inspect-column`), 63 blocks of it over the neighbours' shale.
  3. **The pools know only today's geotherm; the log keeps its peak.** A pool unit under a metamorphosed log layer stayed
     unmetamorphosed (schist log over `sandstone` pool), and the mafic plutonic pool under the gneiss pool turned to greenstone
     at 300 °C (gneiss over greenstone: 2,968 horizons, the most frequent pair in the unsmoothed stacks of the small run). A buried soil without a path
     of its own showed as mudstone under metamorphic rock.
  4. **Slivers**: every pool thicker than 1 m is a unit (the 3-block gneiss is a 60-m leftover of the metamorphic pool); thin
     metamorphic log layers; the tongues of the column rule between classes of one age; and every unit that ends sideways thins
     through 1 … 7 blocks.
  5. **Plutonic rock as beds**: the basement is the plate model's rock pools stacked in erosion order (`TOP_ORDER`: sediment,
     sedimentary, felsic and mafic volcanic, metamorphic, felsic plutonic, mafic plutonic), each a unit. That is a composition
     ledger, not a structure. The discordant plutons P1-07 has are the bodies (ellipsoids with aureoles, painted over the
     horizons); the pools are something else and are still layers (see the decisions).
- **Fixes.**
  - *Grade-ordered keys* (`columns::key`; FORMAT.md, "The bake's key space"): regolith 0–4095; band 0 (unmetamorphosed) log
    4096–32767 by age as before, then its pools 32768–34559; six metamorphic bands of 4,480 keys from 34,560 (from 250, 300, 350,
    450, 500, 650 °C: slate; quartzite, marble, greenstone; phyllite; schist; amphibolite; gneiss), each a log by age (2,688
    keys) and the pools (7 × 4 slots of 64 keys); mantle 61440–65535. A unit's band is that of the hottest formation temperature
    at or above it in its own column; plutonic pool units are always in band 6 with the gneiss (the crystalline basement).
    Any blend of stacks by key therefore has the grade rising downwards: a belt's rocks lie under its neighbours'
    unmetamorphosed cover and over everyone's crystalline basement.
  - *No unit cooler than the rock above it* (`columns::build`): a sedimentary or volcanic pool steps along its path at least
    as far as the hottest formation temperature above it; a plutonic pool stays what it is (gabbro under gneiss is gabbro;
    only granite → gneiss at the 650 °C isotherm is taken); a log layer shown as another class (buried soil → mudstone)
    follows the rock above it.
  - *Key spans stay in one key domain* (regolith, each band's log, each band's pools, mantle): a merge across domains keeps
    the kept class's span, the other's keys become a hiatus; units of one class in different domains stay separate horizons.
  - *No slivers in the stored stacks* (`settle`): inside a stack a regional-metamorphic horizon is ≥ 8 blocks and a plutonic
    one ≥ 16; a thinner one gives its blocks to the horizon under it (of its domain; else the one over it), which keeps its
    class and keys.
  - *No slivers in the columns* (**the column rule's new step 5**, Rust `strata::drop_slivers`, Java `Columns.Stack.dropSlivers`):
    the layer descriptor's optional `min_blocks` (per class; the bake writes 8 / 16 / 1) names least thicknesses; a thinner
    horizon of such a class under the first and over the last is removed and its blocks go to the horizon under it. A unit that
    ends sideways ends at a step of that many blocks instead of a one-block sheet.
  - *The smoothing's top* (two regressions the key fix would otherwise have exposed, both from "the top unit fills the height
    of a cell standing over the mean surface"): the regolith and loose sediment now lie on the ground with their mean thickness
    (first pictures had 44–65 blocks of saprolite, then of clay, on ridges), and the unit that fills a ridge is the thickest
    one beginning within 16 blocks of the mean rock surface, the thin beds over it riding up with the ground (first pictures had
    25-block coal caps on two peaks).
- **After** (the same history: `stage amplify --in big-base/checkpoints/6-young.dtcp --bake`; the small run from scratch; a
  from-scratch run of the 1000 Myr preset with the final code wrote a byte-identical `strata/horizons.dtr`, `cmp`):

  | share of columns | grade inversion | metamorphic sliver | high-grade sliver | plutonic sheet | plutonic with a floor | horizons / column |
  |---|---|---|---|---|---|---|
  | 1000 Myr supercontinent, all | **0** | **0** | 0 | 0 | 0.50 % | 12.1 |
  | … land | 0 | 0 | 0 | 0 | 1.04 % | 17.3 |
  | … stored stacks | 0 | 0 | 0 | 0 | 0.57 % | 12.1 |
  | 300 Myr scattered, seed 7, all / land / stored | 0 / 0 / 0 | 0 / 0 / 0 | 0 | 0 | 0.17 / 0.46 / 0.15 % | 7.3 |
  | the JUnit test planet (`planet4k`), all / stored | 0 / 0 | 0 / 0 | 0 | 0 | 0 / 0 | — |

  **x −6901, z −128** (class, top y, blocks), before: schist 56 (56) · shale 0 (57) · andesite −57 (3) · tuff −60 (4) · sandstone
  −64 (2) · tuff −66 (2) · shale −68 (16) · basalt −84 (22) · phyllite −106 (5) · schist −111 (5) · shale −116 (41) · sandstone
  −157 (17) · basalt −174 (45) · greenstone −219 (3) · gneiss −222 (30) · granodiorite −252 (60) · gabbro −312 (39) · greenstone
  −351 (2): 9 inversions, 3 slivers. After: mud 56 (5) · shale 51 (112) · andesite −61 (4) · tuff −65 (3) · sandstone −68 (2) ·
  tuff −70 (1) · shale −71 (5) · sandstone −76 (1) · shale −77 (10) · basalt −87 (22) · sandstone −109 (2) · shale −111 (28) ·
  basalt −139 (58) · phyllite −197 (9) · schist −206 (15) · gneiss −221 (31) · granodiorite −252 (59) · gabbro −311 (42): none.
  **x −7697, z 46**, before: 36 horizons, 9 inversions, 13 slivers (slate 5, quartzite 4, slate 3, slate 4, schist 1, slate 2,
  schist 6, gneiss 6, schist 1, phyllite 1, schist 1, phyllite 4, greenstone 6). After: mudstone 210 (1) · shale 209 (75) · 19
  thin sandstone, mudstone, shale and tuff beds 134 … 57 · sandstone 56 (86) · basalt −30 (56) · slate −86 (25) · sandstone −111
  (8) · basalt −119 (1) · quartzite −120 (13) · greenstone −133 (18) · phyllite −151 (12) · greenstone −163 (11) · schist −174
  (48) · gneiss −222 (117) · gabbro −339 (14): none. Sections through both (`deeptime-inspect wd-section`, before, after, and
  the rejected "all thin beds ride up" variant): `review/out/rock-order/{big-base,big-after,ride-all}/` (not published).
- **Format** (stays 0.2, FORMAT.md updated): the key layout is the baker's choice and the column rule reads any keys, so
  **old bakes load and read exactly as before** — and keep their inversions until the world data is baked again. `min_blocks`
  is an optional descriptor field; a layer without it (every older bake, the synthetic worlds) skips step 5. A jar from before
  this change reading a new bake would ignore `min_blocks` and paint the slivers the Rust evaluator drops (no version bump was
  made for it, as for the tongue parameters; say so if a bump is wanted). `StrataPass.horizonKey` (the block-variant era, the
  key's top four bits) works unchanged; its eras over 32768 now are grade bands instead of pools.
- **Tests** (planet-independent): `structure::tests::no_column_holds_rock_that_was_never_as_hot_as_the_rock_above_it` (4,000
  random cells — logs with deposits, burial metamorphism and erosion, random pools, crust, regime, geotherm: keys ordered, every
  span in one domain, no inversion); `…::baked_stacks_and_blended_columns_have_no_grade_inversion_and_no_sliver` (four random
  64 × 32 planets of provinces, σ 0 / 2.5 / 10: `check`, `check_bake`, zero inversions, slivers and sheets in the stored stacks
  and in 6,000 columns by the column rule each); the pipeline test asserts the same on its real small planet;
  `realism::tests` (Ben's column is flagged; cover on basement is not); `bake_horizons` (sliver merge and domains, a sliver
  joins the horizon under it, the regolith follows the ground, a thin top bed does not fill a ridge);
  `strata::tests::slivers_of_the_named_classes_are_dropped_inside_a_column`; Java↔Rust: `ColumnParityTest` on the re-baked
  `planet4k` (its layer carries `min_blocks`), 500 of 500 columns equal.
- **Gates** (VM, `tools/dev/gates.sh`, new: the branch gates detached with one log; HEAD d92b6c4): `GATES fmt=0 clippy=0 test=0
  deny=0 gradle=0`; cargo test: cli 52, core 359 + 3 ignored (+ 1), format 53, inspect 29 + 2 + 7, jni 7, viewer 6; JUnit
  tests=160 failures=0 errors=0 skipped=1 (`MacMemoryTest`, not this change); `ColumnRuleBenchTest` 0.61 ms/chunk at 13.3
  horizons per column. No GameTests (the coordinator's, per merge batch): `StrataGameTests` and the deposit tests read the
  strata and may need their numbers looked at. Strata bake 36.9 s → 33.2 s on the small run, 61.7 s → 56.2 s on the 1000 Myr one.
- **What changes in a world.** Metamorphic rock is no longer at the surface on its neighbours' cover; belts read as
  cover → slate/quartzite → phyllite → schist → gneiss → granite/granodiorite → gabbro downwards. Fewer horizons per column
  (15.4 → 12.1). The host fractions the deposit stage reads change a little (pool units under a metamorphosed log are now
  metamorphic; gabbro no longer turns to greenstone), so deposit sites differ slightly from a master run of the same seed.
- **Observed, not changed.** The mountain at x −7697 still has one thick unit holding its top (shale, 75 blocks, before 84 of
  shale): the smoothing has no information about a ridge above the mean surface (σ 10 cells = 160 blocks) and fills it with
  one formation (see the decisions).
- **A mistake to report.** While restarting my own gates I ran `pkill -x cargo` on the shared VM (18:27 VM time): it would
  have killed any other agent's cargo then running. None was running a minute later; an agent whose build died at that time
  should simply re-run it.
- **Decisions for Ben, answered 2026-10-01** (relayed by the coordinator): granite-type rock stays as flat basement layers
  until folds and faults ("Leave it until folds and faults"); a ridge is filled by the thickest nearby formation, as built; no
  format bump (his standing rule: no backward compatibility before a public release, so no "old bakes still load" concerns).
- **Merged with master 94c54c2** (mountain-height stage 1, hypsometry, lakes by climate, P1-28; merge commit 6599c81; conflicts
  in STATUS.md and `bake/strata.rs`, both sides kept: the realism note beside the build-limit counters). FORMAT.md no longer
  speaks of older bakes (Ben's rule: no backward compatibility before a public release). VM, on the merge: `tools/dev/gates.sh`
  → `GATES fmt=0 clippy=0 test=0 deny=0 gradle=0` (cargo: cli 52, core 371 + 3 ignored, format 53, inspect 31 + 2 + 7, jni 7,
  viewer 6; JUnit tests=161 failures=0, `ColumnParityTest columns=500 equal=500`, lake world 300 of 300). A fresh bake
  (`run --preset earthlike_quick_16k --start scattered_continents --length-myr 300 --set seed=7`) + `strata-check`: grade
  inversion, metamorphic sliver, plutonic sheet and plutonic with a floor all 0 in 20,000 columns, 6,278 land columns and
  524,288 stored stacks (`notes.bake.strata.realism.stack_rules` ok). `mac.sh --server -- ./gametest.sh` → **All 58 required
  tests passed** (58 result lines, 0 not PASS). `tools/smoke/install.sh mic` (`--pack`) + `./gametest.sh -PwithMic` → 57 of
  58: `firstinstalledfallback` fails with "shale: expected fallback born_in_chaos_v1:black_argillite, got minecraft:stone;
  gneiss: expected minecraft:granite, got minecraft:deepslate[axis=y]". Not this branch: the MIC rock-tag datapack
  (`mic/Docs/datapacks/deeptime-mic-rock-tags`, outside the repo, synced by `--pack`) had its `rock_fallback` rows for shale,
  mudstone, slate, phyllite, schist and gneiss changed to stone and deepslate on 2026-10-01 (P1-28's MIC rows), and
  `StrataGameTests.firstInstalledFallback` on master still expects the old MIC stand-ins under `rockTags`. The fix is two
  expectation lines in that test, left to P1-28's owner or the coordinator.
- **Scratch.** `/Volumes/Graphide/ban/deeptime-runs/rock-order/` (runs, checkpoints, censuses, binaries) deleted at the end.
- **Next.** Merge (the coordinator), run both GameTest suites on the batch, re-bake the workbench run; then Ben's answers.

### 2026-10-01 — Lava floods: lava is measured from the world's floor, not from y 0 (agent worktree agent-a04fab824846efbfc, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Report.** Ben, 2026-10-01, from a workbench 3D preview of run `1a0f6e918c9` (earthlike_quick_16k, MIC profile) around
  x −6901, z −128: a lava-filled wedge more than 100 blocks deep under a coastal valley, lava bodies all through the rock,
  "a bunch of overlap between lava pools & the ocean". A second box (x −7801..−7546, z −209..46, a mountain) had 311,110
  lava blocks.
- **Cause.** Vanilla hard-codes two lava heights for a world whose floor is y −64, and Deep Time's floor is y −352.
  - `NoiseBasedChunkGenerator.createFluidPicker` fills **every open block below y −54** with lava (floor + 10 in vanilla;
    floor + 298 here). `Aquifer.computeSubstance` asks that picker first, before any aquifer, so it holds for caves and for
    open sea alike.
  - `Aquifer.computeFluidType` lets a cave lake whose surface is at **y −10 or lower** be lava (floor + 54 in vanilla).
  - The plan (§7.1, the router table) keeps the aquifer functions "as vanilla, by reference" and never mentions the
    picker. Tectonic patches the same constant for its own deeper worlds; the plan notes that patch does not fire for us.
- **Evidence** (`tools/workbench/lava_stats.py` on the exports of Ben's own two preview jobs, MIC):
  - Coast box: 2,152,868 lava blocks, 51,040 of 65,536 columns. Below y −54 the open space is 100 % lava (0 cave air);
    above it 337 blocks. 4,020 sea columns have their water standing directly on lava, down to y −267: the "wedge" is a
    submarine canyon whose water turns to lava at y −55 (screenshot `before-coast-section-z72.png`). 5,810 lava faces
    touch water.
  - Mountain box: 311,110 lava blocks (the workbench's own counter says the same), 100 % of the open space below y −54.
  - Neither `WaterBodies` (it only fills air above the ground with water) nor the pack is involved: the pack's share is
    a few lava-lake features (below).
- **Fix** (the default; both rules are vanilla's, measured from the floor).
  - `worldgen/LavaRules` (plain Java): the lava floor is `min_y + lava_floor_blocks` (10 → y −342); lava lakes only in
    aquifer level cells centred at or below `min_y + lava_lake_blocks` (54 → cells centred at y −300 and −340, surfaces
    up to about y −288); under a sea bed that reaches the lava floor the lava stays 8 blocks below the bed.
  - `worldgen/LavaFluids`: Deep Time's global fluid picker. `GeologicChunkGenerator` puts it in the superclass's
    `globalFluidPicker` field (one access-transformer line, `public-f`).
  - `worldgen/density/LavaLakesDensity` (`deeptime:lava_lakes {min_y}`): the router's `lava` is now this 0/1 gate times
    vanilla's `aquifer_lava` noise (`planet.json`, `planet_nocaves.json`, `tools/dev/gen_worldgen.py`, which reproduces
    both files). The aquifer samples that function at cell coordinates (⌊x/64⌋, ⌊y/40⌋, ⌊z/64⌋), so the gate reads the
    cell's y. A density function, not a mixin on the aquifer, because C2ME rewrites the aquifer.
  - **Settings** (preset section `underground`, in all eight presets; editor section "Advanced · Caves and lava"):
    `lava_floor_blocks` (10, "Lava floor height") and `lava_lake_blocks` (54, "Lava lakes up to"); 0 turns either off.
    The bake records them in `notes.bake.underground.params`; the game reads them per level (`WorldDataTerrain.lava`).
    FORMAT.md, "Lava".
- **Measured after** (VM, `tools/workbench/preview.py run --profile … --world-data <the run's bake> --box … --box …`,
  then `lava_stats.py`; screenshots and the censuses in `review/out/lava-diag/`).

  | Box | Before (Ben's jobs, MIC) | After (MIC, another seed) |
  |---|---|---|
  | coast | 2,152,868 lava; 4,020 sea columns on lava; 5,810 lava–water faces | 8,267 lava (8,163 of them below y −300, the lava floor); 0; 0 |
  | mountain | 311,110 lava | 568 lava (60 at the floor, the rest feature lakes) |

  A preview's world seed is random, so the cave layout differs between those two. The same seed (`--seed 20261001`), with
  the old levels brought back through the settings (a copy of the bake with `lava_floor_blocks` 298 and
  `lava_lake_blocks` 342, i.e. y −54 and y −10; the sea-bed rule stays on):

  | Box | Old levels, MIC | Default, MIC | Default, standalone |
  |---|---|---|---|
  | coast | 732,008 | 395 | 3,719 |
  | mountain | 930,637 | 1,421 | 3,249 |

  Lava by depth after the fix: nothing between y −250 and y −54 except single-block lava springs; lava-lake aquifers at
  y −324..−307 (standalone: 3,586 and 1,017 blocks); the lava floor at y ≤ −343. Above y −54 only lava-lake
  **features** remain, 80 to 140 blocks each: 3 in the coast box (y −24, 25, 35, under the sea bed), 7 to 9 in the
  mountain box (y 51 to 156), in standalone and MIC alike. They are the open question above.
- **Tests.**
  - JUnit `LavaRulesTest` (7): the defaults are vanilla's heights from the floor, the sea-bed rule, the lake cells, the
    reader, a baked world carries the settings. Rust `underground::tests` (2). `AdvancedSettingsTest` and the workbench
    catalogue test know the ninth advanced section.
  - GameTest `LavaGameTests.lavaStaysAtTheFloor` on the new level `deeptime:test_planet_caves` (the real test planet
    with the full router, `deeptime:planet`; every other test level uses `planet_nocaves`, which is why nothing saw
    this): chunks at the deepest sea, a sea about 150 blocks deep, a coast and high ground; no lava above y −288, no sea
    column standing on lava, open space below the floor level is lava.
    With the fix: `lava=0 lavaAboveLakes=0 seaOnLava=0 openDeep=42275` (4,096 columns, 2,048 of them over a sea bed
    below y −60). **With vanilla's picker put back** (the one assignment removed on the VM's copy, then restored): FAIL,
    `lava=256029 topLavaY=-55 lavaAboveLakes=238767 seaOnLava=418`: the test catches the bug.
- **Gates** (VM, after merging master 83aaa51 into the branch, tree 4c928b7): `cargo fmt --check`, `clippy -D warnings`,
  `cargo deny` ok; `cargo test --workspace` 523 passed, 0 failed, 3 ignored; `./gradlew build test` → tests=168
  failures=0; `./gametest.sh` → All 59 required tests passed (458 s); `./gametest.sh -PwithMic
  -PgametestOnly=lavaStaysAtTheFloor` → passed (`lava=0 lavaAboveLakes=0 seaOnLava=0 openDeep=78373`; the MIC set is a
  copy of the workbench's, pack 790e116). A first MIC attempt died in mod loading before any test (Cobblemon's
  `PrioritizedList.add` raced during parallel mod construction, reported as tim_core failing to load); the next start
  loaded. The full MIC suite was not run here. Before the merge the default suite's heap peaked at 6,134 of 6,144 MB;
  the new level held 800 of the 24,263 chunks at that peak.
- **Also seen, not changed.**
  - The test planet's deepest sea bed is at y −399 (y −389 after the merge), below the world's floor (y −352): the sea there is water down to
    bedrock level. It is the mirror of terrain above the build limit (the height-limit question above).
  - In the old worlds the flooded zone also held a few hundred thousand blocks of plain deepslate and blackstone per box
    where the fixed worlds have open cave (crust 296,087 → 12,819 in the coast box): something reacting with the lava.
    Not investigated; it is gone with the lava.
- **Contradicts the plan:** §7.1's router table says `lava` is vanilla's function by reference; it is now gated, and the
  fluid picker is Deep Time's.
- **Next step.** Ben's answer to the lava question above. The MIC full GameTest suite is the coordinator's per-batch run;
  the new level adds one more loaded dimension to it.
### 2026-10-01 — P1-22b follow-up: cold-climate runoff reviewed, lake ice and salt flats in the game (agent worktree agent-a2195c5c858c73075, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
Ben's answers of 2026-10-01 to the first report ("Are the physics correct, if so then it's fine as a frozen lake"; lake ice and playas in the game: do them; dry land below sea level: deferred). Same branch, master merged in (b9300cb), not merged back, not pushed. This entry supersedes the one below where they differ (that one says the bake marks nothing for ice and that no Java changed).

- **The polar lake: it still fills, with sound inflow, and stays a frozen lake.**
  - *What was wrong:* the land's evapotranspiration used PET = 60 mm/°C × the mean annual temperature, zero below freezing, so a land with a mean below 0 °C sent all its precipitation to the rivers.
  - *Fix:* PET = `hydro.pet_mm_per_c` (60) × the year's mean temperature above freezing, on the liquid water. That is Holdridge's rule (PET = 58.93 × biotemperature; Holdridge 1967, "Life Zone Ecology"), and equals the old law where there is no frost.
  - *Ben's frame* (1000-Myr random run at 676 Myr ago, rebuilt on today's master, whose terrain differs from the morning's: the largest lake is now 90,109 km² at 72°S, mean −16.6 °C, warmest month +1.5 °C): inflow 792 → 768 m³/s against a loss of 596 m³/s. It fills with 29 % to spare, so it is full to its rim and frozen. Lake share of land 5.88 % under the old rule, 5.68 % with the balance and the old runoff, 5.64 % with the new runoff.
  - *What the loss is:* nearly all of it is the 200 mm a year an ice-covered lake loses (`lake_evaporation_base_mm`). The lake would stop filling at about 260 mm. Polar lakes lose 150–350 mm, so this basin is a close call that the default decides for "full".
  - *Checked, sound:* snow that never melts feeds nothing (the degree-day rule; at 100 mm of precipitation it cuts in at a warmest month of about −1 °C, close to the summer temperature at the equilibrium line that Ohmura et al. 1992 give for so little accumulation, about −2 °C; cited from memory, not re-read). A lake under ice loses the base amount, not zero.
  - *Checked, not modelled:* land ice. The hydrology does not read the climate's ice thickness, so precipitation on an ice sheet runs off as if on bare ground where the summer thaws, and a basin the ice sheet would flow into is still a lake. The views hide a lake under 60 m of ice; the bake and the statistics do not.
- **Rivers lose about a tenth** to the new runoff law (cold lands evaporate in summer): bake A 3,404 → 2,985 chains and 68,212 → 61,366 km, largest discharge 622 → 528 m³/s; bake B 3,620 → 3,159 chains and 72,233 → 65,908 km; Ben's frame 5,486 → 4,678 chains and 126,329 → 112,330 km.
- **Lakes barely move with it** (`deeptime hydro` on the same two 500-Myr bakes of today's master, old rule → balance with the new runoff): A 1,224 lakes, 1.79 % of land → 1,192 lakes, 1.49 % (230 below their rim, 10 playas, 28 dry; 37 % of lake area frozen); B 1,105 lakes, 1.67 % → 1,004 lakes, 1.42 % (199 below rim, 11 playas, 97 dry; 83 % frozen). Largest lake unchanged in both (11,947 and 6,293 km²). Bake water part 5.1–6.3 s either way; view 89–118 ms a frame.
- **Lake ice in the game** (FORMAT.md "Lake ice"). No new layer: a lake column is frozen where `climate/t_mean_c`, bilinear at the column, is below `notes.bake.water.params.lake_freeze_mean_c` (Java `worlddata/LakeIce`, the rule of the maps and the statistics).
  - `WaterBodies` sets a frozen lake's top water block to ice, whatever the biome (5 lines in the lake case).
  - `mixin/BiomeFreezeMixin` → `worldgen/LakeFreeze`: `Biome.shouldFreeze` answers false for the surface block of a lake the rule keeps open, so neither `freeze_top_layer` at generation nor snowfall later ices it. Only "does not freeze" is ever forced; rivers and the sea are left to the biome.
  - Before this, on the two bakes, 2–13 % of the frozen lake area lay outside the frozen biome band (no ice) and 3–10 % of the open lake area inside it (iced).
- **Salt flats in the game** (FORMAT.md "Playas"). The bake writes the layer `playa` (u8, blocks of salt, on the `water_level` grid; `hydro.playa_blocks`, 2) and `notes.bake.water.playa`. `worldgen/PlayaPass` (after the water pass, before the ice cover) replaces that many of the ground's top blocks with the block the rock palette resolves for the `halite` class, so the pack's rock tags decide the block.
- **Settings added:** `hydro.pet_mm_per_c`, `hydro.playa_blocks` (editor fields, lang keys, presets).
- **Java touched:** new `worlddata/LakeIce`, `worlddata/Playa`, `worldgen/LakeFreeze`, `worldgen/PlayaPass`, `mixin/BiomeFreezeMixin`, `gametest/LakeGameTests`, `LakeIceTest`; small edits in `WaterBodies` (lake case), `GeologicChunkGenerator.buildSurface` (one line), `RiverGameTests` (iced lake tops), `WaterPass` (comment), `deeptime.mixins.json`. `Water.java` (the column rule) is untouched.
- **Gates** (test VM): cargo fmt, clippy `-D warnings`, `cargo test --workspace` (core 363 passed), deny ok; `./gradlew build test` tests=164 failures=0. GameTests (after merging master d4e876c; `./gradlew build test` again: BUILD SUCCESSFUL, tests=165 failures=0, `LakeGameTests` compiled): `./gametest.sh` all 59 required tests passed, all mixins bound (the default 4k planet has no lakes, so `lakeIceAndSaltFollowTheClimate` checked nothing there). `DEEPTIME_TEST_PLANET_SET=seed=21` with `-PgametestOnly=lakeIceAndSaltFollowTheClimate,riversHold`: 2 of 2 (24 frozen lake columns, ice read on 22; 9 open, water on 9; wrong 0). `seed=12`: 2 of 2 (10 frozen, ice on 5, 4 of them in a biome too warm to freeze; 10 open, water on 10; 1 playa column, salt read as `minecraft:calcite`, the halite class's vanilla block; wrong 0). `-PwithMic` with seed 21: 2 of 2 (ice on 22, 6 of them in a warm biome; wrong 0). **The full `-PwithMic` suite has not passed**: its one run died in mod loading before any test ("Found unused register callbacks", unbound `artifacts:generic.attack_burning_duration`: another mod's registration, and the MIC seed-21 run minutes later loaded the same set), and the re-run was still waiting for the server lock when SSH to the VM was lost again. No sampled open lake lay in a cold biome, so the mixin's "stays open" answer was never the deciding one in a test; it is bound and did not break generation. Scratch is still on the VM under /Volumes/Graphide/ban/deeptime-runs/lakes/ (SSH lost before it could be deleted).
- **Deferred by Ben:** dry land below sea level (a terminal basin below sea level stands at sea level).
- **Open for Ben:** whether the hydrology should read the climate's land ice (no runoff from under an ice sheet, no lake where ice stands); the 200 mm base loss decides close polar cases.
### 2026-10-01 — Deposit audit: every type measured in blocks; placers moved into the rivers; richness options for Ben (agent worktree agent-a797e24e4a8e91f36, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
Ben, 2026-10-01, on a 3D preview: "This gold placer feels pathetic"; decision "Audit all types, then tune". On branch `worktree-agent-a797e24e4a8e91f36` (local master 94c54c2 with P1-28 merged in), not merged, not pushed. **No default richness changed.** The three options are a pending question.

**The tool (re-run after any tuning).** `deeptime deposit-audit --world-data <dir> [--box X0 Z0 X1 Z1] [--only <type>] [--chunks N] [--scatter-dir <dir>] [--json <file>]` (`crates/deeptime-cli/src/deposit_audit.rs`); on the VM `tools/review/deposit-audit.sh audit <world-data>`. No server: it ports `DepositFeature.materialAt` and `BackgroundScatterFeature.plan` bit for bit and models the host check from the column evaluator. Checked against a real MIC box (256 chunks): the placer 75 modelled vs 71 real ore blocks, scatter 3.5–7 % high (caves, lava and aquifers are not modelled). Per type: sites, body blocks, ore per site p10/p50/p90, ore per chunk in the body, scatter per chunk, vanilla per chunk, the median deposit in scatter chunks, rock dug for 640 ore; per material the planet's ore in deposits against its scatter. Real blocks: `tools/review/deposit-audit-detached.sh <world-data> <box>` (takes the server lock itself, survives SSH drops; runs `deposit-audit.sh vanilla`, then the box on the standalone and mic-ev sets; results in review/out/deposit-audit/).

**Measured** (two planets of today's `earthlike_quick_16k`, 1000 Myr, seed 1: workbench runs 1a0f88578b3 "random" and 1a0f6e918c9 "supercontinent"; values are the two planets'; ore = blocks of every product; placers after the fix below).

| type | sites | body blocks p50 | ore per site p10 / p50 / p90 | ore per chunk in body | median deposit = chunks of the same scatter | of the planet's ore blocks |
|---|---|---|---|---|---|---|
| porphyry Cu-Mo-Au | 11 / 6 | 9,462 / 16,683 | 192 / 453 / 2,531 and 93 / 836 / 5,870 | 76 / 104 | 60 / 110 | copper 1 % |
| placer Au | 67 / 43 | 810 / 1,368 | 5 / 32 / 122 and 4 / 55 / 434 | 7 / 11 | 29 / 52 | gold 3–6 % |
| MVT Zn-Pb | 167 / 87 | 986 / 796 | 6 / 166 / 2,485 and 4 / 135 / 943 | 47 / 40 | 78 / 64 | zinc 11–35 % (with SEDEX) |
| SEDEX Zn-Pb | 2 / 2 | 10,426 / 8,195 | 2,628–3,939 and 2,217–3,123 | 303 / 240 | 1,857 / 1,486 | |
| coal seam field | 84 / 47 | 187,493 / 184,099 | 21,300 / 99,899 / 429,767 and 23,210 / 85,655 / 451,634 | 1,773 / 1,518 | 11,444 / 11,264 | coal 85–94 % |
| petroleum trap | 29 / 8 | 1,278 / 2,986 | no blocks (CDG) | | | |
| BIF, supergene blanket | 0 / 0 | | none formed on either planet (computed from the type: about 1,050 iron and 1,070 copper ore at the median) | | | iron 0 % |

Scatter, real blocks per chunk (box of 256 chunks, master with P1-28): coal 9.8, copper 6.7, gold 1.06, iron 9.5, zinc 1.7 on the standalone set; 9.1, 6.6, 0.77, 7.8, 1.7 on MIC + EV. Per km² of blocks ×3,906 (iron 37,000). 640 ore from scatter = every block of 67 chunks (iron), 95 (copper), 65 (coal), 375 (zinc), 600 (gold). Inside a median body 640 ore take 10,000–16,500 blocks of rock (porphyry; one whole median deposit holds 400–450 copper ore), 12,000 (placer; a median placer holds 30–55), 3,200 (MVT), 1,700 (SEDEX), 1,070 (coal).
**Vanilla reference** (measured: 1,024 chunks around 0,0, seed 20261001, standalone set with level-type `minecraft:normal`): coal 101 per chunk, iron 77, gold 25.6, copper 293 (one sample; 92 % of it above y 0, so likely `ore_copper_large` under dripstone caves; treat as an upper figure), Create's zinc 76, redstone 36, lapis 25, diamond 24. Deep Time's scatter is 10 % of vanilla for coal, 12 % for iron, 4 % for gold, 2 % for zinc.
**P1-28 and who places ore in a Deep Time dimension** (the same box, real blocks): Deep Time's `claim_ores` removes the features of claimed materials (vanilla coal, iron, gold, copper; Create zinc; Create Nuclear lead; Ice and Fire silver; Terralith's extra coal) and its own scatter replaces them; P1-28 leaves that scatter where it was (MIC before: coal 2,599, copper 1,830, gold 268, iron 2,496, zinc 488 in the box; after, standalone: 2,514 / 1,714 / 271 / 2,435 / 438; MIC + EV: 2,323 / 1,696 / 198 / 1,984 / 432 — the MIC + EV iron and gold are 20–26 % lower and that is not explained), because the scatter hosts in any rock block and never depended on stone. Everything unclaimed is the pack's own features, which target stone and deepslate and so gained host rock: per chunk on MIC + EV uranium 35 (was 2.1 before P1-28), nickel 26 (was 2.0), fluorite 15 (was 1.2), platinum 13.6 (13.8), lapis 13.1 (6.8), redstone 17.7 (20.2), diamond 8.3 (7.6), infected diamond 2.4, emerald 0.2, Cobblemon stones 1.6 together; standalone: redstone 19.6, diamond 14.9, lapis 13.6. In ore blocks per chunk MIC went from 84 to 159. Excavated Variants changes looks only: 4,732 of the box's 40,813 ore blocks are EV ids; amounts follow the same draws.

**Where the richness goes** (tonnage and grade to blocks). (1) Volume = tonnage / density; at 300 m × 20 m a block is 1.8 million m³, so a median porphyry (240 Mt) is 49 blocks; `balance.size_mult_h²·size_mult_v` enlarges it (porphyry 144×, placer 1,440×, MVT 432×, SEDEX 576×, BIF 144×, coal 20×). (2) Only part of the body is ore zone (a porphyry's outer 58 % is barren halo; a placer's upper half). (3) In a zone a host block becomes ore with the zone's `density`, a constant of the type (porphyry 0.35–0.45 in the core, 0.2 phyllic; placer 0.08 + 0.01; MVT 0.35–0.47; coal 0.6). **The grade never reaches the blocks**: `Zone.grade` is stored and nothing reads it, so `deposits.richness_mult` and a type's `balance.richness_mult` change the site's listed grade and no block; the editor's "Deposit richness" is a no-op in the world. So block counts follow volume, not metal: a porphyry holding 1.2 Mt of copper and a placer holding 0.2 t of gold differ by 13× in ore blocks. `deposits.size_mult` scales tonnage, so ore scales with it; `scale.feature_scale.deposits` scales the three axes, so ore scales with its cube; `deposits.count_mult` scales the number of sites.
**COE and CDG.** COE: every chunk an ore body reaches is one vein of the type's recipe; the site's tonnage quantile only sets `randomMul`, which matters only for finite veins (gold 2,000–4,000 items, copper and iron 10,000–30,000, coal 15,000–40,000, zinc 8,000–24,000 per chunk at `finiteAmountBase` 1000). MIC ships `defaultInfinite = true`, so each of a median porphyry's 6 chunks, a placer's 4–5 and a coal field's 53–61 is an infinite vein, against 76–104, 7–11 and 1,500–1,800 placed ore blocks per chunk. The drilling value is not consistent with what is placed: one drill out-yields the whole placed deposit, for every family; the blocks are a signpost and a pre-drill supply. The parked question (finite veins?) decides whether deposit size matters to COE at all. CDG: per chunk `charge × 0.02 × the chunk's share`: 2.5 / 4.2 / 19 million mB (p10 / p50 / p90) and 3.7 / 7.1 / 10.8; 6 of 29 and 3 of 8 traps average above CDG's 10 million mB infinite threshold. Consistent with CDG's own scale (its high chunks start at 4 million).

**The placer bug** (Ben's site 56; preview job 20261001-180014-jz2m is of run 1a0f88578b3, not 1a0f6e918c9). Its box holds 71 gold ore in the placer and 268 gold scatter blocks (not 960). The deposit stage lays a placer on the coarse grid's ground before the amplifier and the bake's rivers exist, as one flat ribbon: its centre was a median 0–7 blocks from the real ground but 77–121 under it at p10 and 37–66 over it at p90, 39 of 67 and 22 of 43 placers placed no ore, and site 56 lay 13 blocks inside a hillside: gravel painted into the shale, `minecraft:gold_ore` in it (no gravel ore block exists without EV), no water anywhere near.
**The fix** (`deposits/placer_channels.rs`, called once the folder is complete, and by `deeptime hydro`): each placer is laid again from its site record, into the nearest baked river channel within `placer.river_reach_km` (new type key, 30 km = 100 blocks) as gravel bars along the channel (water's top block down to 3 blocks under the bed, the channel's width + 2, downstream through confluences; one `channel` body per straight piece), or with no channel in reach as a bench in pieces on the finished ground. Java: `OreBlockResolver` takes `#c:ores_in_ground/<host>` for hosts other than stone and deepslate; EV stone configs for gravel and sand; in a loose body `DepositFeature` resolves the ore against the body's own rock (so soil over the gravel also gives gravel ore) — that last line is compiled and unit-tested by the build only, not yet seen in a world.
Before / after: placers with ore 28 → 66 of 67 and 21 → 43 of 43; median ore 0 → 32 and 0 → 55; all placer gold on the planet 3,128 → 4,928 and 6,776 → 11,267 blocks; in channels 28 and 25, benches 39 and 18 (most benches of the first planet are old placers now under the sea). Site 56 in real blocks: before, 71 `gold_ore` under soil in a hillside; after (standalone), 93 ore in 5 bars along the river 60 blocks east, 34 of its 84 ore columns under river water and the rest under the gravel and soil of the banks, 1,122 gravel + 156 sand + 159 water blocks in the bodies; after (MIC + EV), 78 ore, 59 of them `excavated_variants:…minecraft_gold_ore__deeptime__gravel` and 19 `gold_ore` where the host was soil (what the DepositFeature line above addresses).
Still true: without EV (pack master, the workbench's `mic` profile) the ore block is `minecraft:gold_ore`, now in a gravel bed under a river. A gold-bearing gravel block of Deep Time's own would be the mod's first block: asked below.

**Gates** (VM, tree merged with master 94c54c2): `cargo fmt --check` ok; clippy `-D warnings` clean; `cargo test --workspace` ok (core 369, format 54, cli 52, inspect 31, …; 8 new tests); `cargo deny` ok; `./gradlew build test` 161 tests, 0 failures. GameTests not run (batch rule); `./gametest.sh -PwithEv` should run with the batch (EV now has gravel and sand stones). After the last full run only the audit's vanilla constants and the `DepositFeature` line changed: fmt, clippy and the CLI crate's tests were re-run for the first, `./gradlew build test` for the second.
**Next step:** Ben picks a richness option (pending question); then make `richness_mult` act on zone density, add `deposits.scatter_mult` (preset field, editor field, lang key; Java reads it from the manifest), set the chosen values, re-run `deposit-audit.sh audit` on two planets.
### 2026-10-01 — P3-04 Distant Horizons far terrain: LODs from the world data (agent worktree agent-a6e8592da3ce000d8, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
Ben asked for it ahead of the rest of Phase 3. On branch `worktree-agent-a6e8592da3ce000d8` (master 83aaa51 merged in), not merged, not pushed.

- **What it does.** With Distant Horizons installed, every Deep Time level that generates from world data registers a world generator with DH, and DH's level-of-detail data comes from the baked rasters instead of from real chunks generated far away. Without DH nothing runs; `-Ddeeptime.dh=false` switches it off.
- **Verified API surface** (`javap` and a decompile of the real jars on the VM; `DhApiBindingTest` opens 3.3.0 and 3.3.3):
  - Versions: 3.3.0 and 3.3.1 are API 7.1.0, 3.3.3 is API 7.2.0. The plan's "API 4.0.0 renamed methods" is stale. The members used are identical in the three.
  - `DhApi.worldGenOverrides.registerWorldGeneratorOverride(IDhApiLevelWrapper, IDhApiWorldGenerator)` → `DhApiResult` (`success`, `message`).
  - `IDhApiWorldGenerator`: `getReturnType()` = `API_DATA_SOURCES`; `generateLod(chunkPosMinX, chunkPosMinZ, lodPosX, lodPosZ, byte detailLevel, IDhApiFullDataSource, EDhApiDistantGeneratorMode, ExecutorService, Consumer)`; `getSmallest/LargestDataDetailLevel()`; `runApiValidation()`; `preGeneratorTaskStart()`; `close()`; `getPriority()` (10).
  - `IDhApiFullDataSource.setApiDataPointColumn(relX, relZ, EDhApiWorldGenerationStep, List<DhApiTerrainDataPoint>)`, 64 × 64 columns of 2^detail blocks; `DhApiTerrainDataPoint.create(detail 0, blockLight, skyLight, bottomY, topY, block, biome)`.
  - **Three traps found in the class files.**
    1. *3.3.0's jar (the MIC pin) renames every NeoForge class `<Class>_neoforge`, the API included.* The unsuffixed `DhApi` in that jar is the Fabric copy, which nothing on NeoForge reads: a mod compiled against the published API registers into a dead object. 3.3.1 fixed the packaging (only the Minecraft-facing `common` classes keep the suffix). So `compat/dh` binds by name at run time and takes whichever naming the jar has; nothing is `compileOnly`.
    2. *Data-source y counts from the level's bottom* (`setApiDataPointColumn` packs with offset 0; DH's own generator subtracts `getMinHeight()`), and a release build does not validate the range.
    3. *Columns must be written with step `LIGHT`*, or DH queues the section again for its own chunk generation.
- **Toroidal World: a conflict, not worked around.** TW 0.19.3's `WorldGeneratorInjectorMixin` wraps `WorldGeneratorInjector.bind` and throws for any generator whose class is not in `com.seibel.distanthorizons.` on a level it wraps.
  - With DH 3.3.0 TW's DH mixins (all but `ChunkWrapperMixin`) target the unsuffixed classes, which NeoForge never loads: they are inert, TW does not fold DH's keys at all, and the override registers. **This is what MIC runs today.**
  - With DH 3.3.1+ they apply and the override is refused: `smoke dh-next` (3.3.3) logs TW's message and DH generates from real chunks. So a pack update to DH ≥ 3.3.1 turns far terrain off until TW lets a seam-aware generator through. Options are under "Pending questions".
  - Wrapping itself needs nothing from TW: every sample folds x into the canonical lap, so a section on any lap or across the seam is the planet's terrain (GameTest and JUnit). Past the pole edge the rasters clamp, as the real generator reads them (`deeptime:beyond_pole` exists but the shipped router does not use it; plan §7.11's "ice-sheet ocean" is not what real chunks do, and far terrain follows the chunks).
- **What a far column is** (`worlddata/FarColumn`, `farterrain/FarTerrain`).
  - Ground: the bilinear height between noise-cell corners (`Water.cellSurface`, detail term included), i.e. what the generator draws. Water: `Water.column` — lake, sea, river with its cut bed, banks raised and lowered. Ice cover on the top blocks (`IceCover`). All shared code, called, not copied.
  - Biome: the level's biome source and sampler at the quart cell a chunk stores. Frozen: `Biome.coldEnoughToSnow` → ice on water (so frozen lakes, as vanilla's `freeze_top_layer` makes them), a snow layer on ground.
  - Soil: a datapack table by biome (`data/deeptime/deeptime/far_terrain/surface.json`, pack-overridable). The pack's surface rules run over a generated chunk, which far terrain does not have.
  - Rock: the strata column rule bed by bed down to the drop to the lowest neighbour (+8, capped at 256), so cliffs show their strata; one run below. Blocks through the rock palette with the strata pass's member choice (so P1-28's stone and deepslate come through).
  - **Level of detail chosen.** One- and two-block columns: every rule in full, the strata rule only where rock shows (bare biome, or a side dropping below the soil), because it is 85 % of such a column's cost. Wider columns (detail ≥ 2): sampled at the centre, no bank rule (a bank is one block), nearest strata cell's stack as stored. That path reads the horizons itself, so a rule added to `Columns.stack` (folds, faults) must be added there too.
  - Not in far terrain: trees and every other feature, caves, bodies, icebergs, the pack's exact surface rules.
- **A bug found on the way: the tile cache never kept strata tiles on a 32-core machine.** `TileCache` gave Guava one segment per processor; at 32 that is 8 MiB a segment, a `strata/horizons` tile is 14.7 MiB, and Guava drops an entry heavier than its segment. Every sample leaving a thread's last tile decoded it again. Real chunks at a strata tile's edge paid it too. Now segments hold at least 64 MiB (4 for the default 256 MiB). On a 16k planet a far section at a tile edge went from 566 ms to 34 ms; coarse sections in the game from 161 ms (detail 3) and 1,248 ms (detail 6) to 7–10 ms.
- **Measured** (VM, load average 20–50, so upper bounds).
  - `tools/review/p3-04-dh.sh`, smoke `dh` (standalone + DH 3.3.0, `earthlike_quick_16k`, scattered continents, 200 Myr, 98 % land beside the spawn): DH's own server-side `/dh pregen start minecraft:overworld <x z> 32` asked the override for 257 sections (4,112 chunks of area) and finished within the harness's first 5 s poll; no failed section; DH's validation on. Inside DH, cold and several threads at once: 37–44 ms a section, 4.92 data points a column.
  - `/deeptime probe far bench`, one thread, warm, no DH: **9.6–11.1 ms a section at detail 0, 6.9–7.9 ms at detail 3, 8.5–10.7 ms at detail 6** (a detail-6 section is 4,096 blocks wide). For scale: P1-08 measured 441 real chunks in 19 s on a first start, 0.7 s for a section's 16 chunks.
  - JUnit `FarColumnTest.sectionTime` (4k planet, no biome): 13.7 ms at step 1 (geometry 1.8, strata rule 11.6), 4.9–6.2 ms at steps 4–64.
  - GameTest on the real test planet (pack surface rules and features, no caves), 119 columns over the planet: ground exact 112, within one block 117, water 119 of 119 (top exact 97 of 97), biome 119 of 119, snow layer 119 of 119, ground block 101 (85 %); 7 iceberg columns set aside. Sections there: 4.0 ms (detail 0), 9.2 ms (detail 3); four threads agree with one.
  - Smoke `dh`, 400 columns, far terrain against the chunks generated afterwards (vanilla caves and carvers on): water 400 of 400, biome 399–400, ground block 85–99 %, ground within one block 75–94 % and within two 78–96 % over four planets (the area is a mountain range; worst cases are cave mouths and steep faces, 26–128 blocks).
  - Smoke `mic` (the whole pack, DH 3.3.0, Terralith, C2ME): registers, 257 sections, no failure, 52 ms a section in DH; ground within one block 96 %, water and biome 400 of 400, snow layer 95 %; **ground block 55 %** — the built-in table knows vanilla's biomes, not Terralith's (51× grass where Terralith's gravel desert has gravel, 25× coarse dirt in its birch taiga, podzol either way).
  - What DH stores: all 400 columns held; its default world compression (`VISUALLY_EQUAL`) merges blocks no side shows into the block above when it saves (45 of 400 came back layer for layer; the dirt under grass becomes grass, the soil under a snow layer snow), so the check compares the top block, its height and the water top: 400 of 400 in the final run (`smoke/dh/report.md`, PASS).
- **Tests.** JUnit 180 / 0 failures (`./gradlew build test`): `FarColumnTest` 14 (the plan's exit test: geometry = `Water.column`, rock = `Columns.stack`, gap-free layers, the seam on three laps, past the pole edge, coarse columns, hand-made columns for ice cover, frozen water, river beds, bare rock, six threads) and `DhApiBindingTest` 4 (binds under each jar's naming; `generateLod` called through each jar's own interface fills 2 × 4,096 columns DH's rules accept). GameTests `FarTerrainGameTests` 3 of 3 (`./gametest.sh -PgametestOnly=…`); the whole suite and `-PwithMic` are the coordinator's batch run. cargo fmt, clippy `-D warnings`, test, deny: clean (no Rust changed).
- **Verified / not verified.**
  - Verified: the binding against the real jars; registration on a dedicated server with DH 3.3.0 (standalone and MIC) and the refusal with 3.3.3; DH's server-side generation path end to end (its queue, its threads, its validation, its store) without a player; the data against real chunks; the seam and the pole; thread safety; time per section.
  - **Not verified: how it looks.** No client ran (no GPU on the VM, the Mac is off limits): colours, the light on cliff sides (buried rock is sent dark, exposed runs lit; DH lights a side from the air beside it, read in `ColumnBox` but not seen), the hand-over when real chunks replace LODs, trees popping in, pop-in at the seam. Also not exercised: DH sending LODs to a connected client (its network path), singleplayer (the same server code inside the client), and DH's quad tree across the seam on a client.
- **New tools.** `tools/smoke/profiles/dh.txt` and `dh-next.txt` (the latter without Chunky: DH 3.3.3's Chunky hook crashes the boot, "Chunky is not loaded"); `run.py --dh`; `/deeptime probe far <x> <z>`, `probe far land`, `probe far bench`, `probe dh`, `probe dh check`; `tools/review/p3-04-dh.sh`; `tools/remote/detach.sh` (jobs that outlive the ssh session; the laptop's address changed twice today and both times the detached jobs survived).
- **Next step.** Ben's answers below; then either the pack-side table for Terralith's biomes on the MIC deep-time branch, or the exact-surface option. When P2-01's folds and faults land, give `FarColumn.rock`'s wide-column path the same rule.


### 2026-10-01 — P1-22b lakes sized by the climate, and frozen (agent worktree agent-a2195c5c858c73075, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
Ben's decision of 2026-10-01, "Both: freeze them and size them by climate". On branch `worktree-agent-a2195c5c858c73075`, not merged, not pushed.

- **What changed.**
  - **Lake levels from a water balance** (`hydro/network.rs`, `Net::resolve`). Each basin (a depression over the lake thresholds) is solved when the discharge pass has met its last cell, so everything upstream has given.
    - Wet to its rim and still gaining (inflow ≥ the losses of its cells): a lake full to the rim that spills the surplus, as before.
    - Otherwise terminal: its lowest cell becomes a sink, a Priority-Flood of the basin alone drains its floor to it (the exposed floor's streams follow the real ground; basins inside it are solved first, to 8 levels), and the lake stands at the highest 1/16-block step of the basin's hypsometry (cells binned by their spill level towards the sink) whose losses the inflow at the sink covers. A lake inside that this level covers becomes part of it.
    - A balance lake under the lake thresholds is a playa (drawn as a salt flat, not stored); a basin no water reaches is dry.
    - A terminal basin whose floor lies a block or more below sea level stands at sea level at least: the column rule and the aquifer flood such ground in the game whatever the bake says.
    - Chains: basins are solved in drainage order, and a lake that does not spill passes nothing downstream.
  - **The budget** (`HydroParams::budget`). Liquid water = rain in the share of the year above freezing plus the snow the year's degree-days melt (`snow_melt_mm_per_degree_day`, 4); ground whose warmest month is below freezing gives no runoff. Runoff = liquid − ET (Fu, PET 60 mm/°C·T, as before). Open water loses `lake_evaporation_factor` × (`lake_evaporation_base_mm` 200 + 60 mm/°C × the year's mean temperature above freezing). The base is calibrated on Earth's lakes: 1,500 km³ a year from 2.3 M km² without the Caspian, about 650 mm, mostly in cold climates (Zhao et al. 2022, Nat. Commun. 13:3686).
  - **Frozen lakes.** Criterion: mean annual temperature below `lake_freeze_mean_c` (0 °C), which with a yearly cycle means ice for more than half the year. Chosen over the coldest month (would call every mid-latitude lake with a frosty January frozen) and over the warmest month (perennial ice only, which almost no lake has). The views draw it per cell from the frame's `climate/t_mean_c`. **The bake marks nothing**: `WaterPass` places lake water before features, and vanilla's `freeze_top_layer` ices it in the frozen biomes; the bake's note records how much of the frozen lake area lies in the biome temperature layer's frozen band (91–96 % on the two planets below), so a second mechanism would double up.
  - **View** (`water_view.rs`, `ramps.json`, `workbench.js`): frozen lakes and playas on both views, "Frozen lake" and "Salt flat" in the Rivers & lakes legend, and the pointer on the satellite view reads the frame's Rivers & lakes layer, so a lake says "lake" or "frozen lake" (was "looks like shallow sea").
  - **Settings** (`hydro`): `lake_balance` (true; false is the old rule), `lake_evaporation_base_mm`, `lake_evaporation_factor`, `snow_melt_mm_per_degree_day`, `lake_freeze_mean_c`; editor fields, lang keys, the 8 preset files.
  - **CLI**: `deeptime satellite --set PATH=VALUE` and a `lakes {…}` statistics line; `deeptime hydro` prints the share of land, the largest lake and what became of the basins. `tools/workbench/lakes-check.sh` is the browser check.
- **Data and parity.** No format change (0.2). `water_level` still holds each lake's surface; it may now be a level below the rim. The rivers file, the column rule (Rust `deeptime_format::water`, Java `worlddata/Water`) and the parity fixture are untouched, and no Java source changed. **Old bakes load and play exactly as they did** (their lakes stay full). `notes.bake.water.method` is "p1-22b" (the game does not read it). FORMAT.md "Rivers" has the method.
- **Measured** (test VM; `deeptime run --preset earthlike_quick_16k --length-myr 500 --checkpoints` with master's binary, then `deeptime hydro --world-data` and `deeptime satellite --in …/7-amplify.dtcp [--on elevation]` with this branch's, `--set hydro.lake_balance=false` for the old rule on the same planet). Areas are at the preset's scale (300 m a block).

  | | lakes | lake area, km² | share of land | largest, km² | frozen (share of lake area) | arid, P/PET < 0.2 | polar, warmest month < 10 °C | below rim / playa / dry |
  |---|---|---|---|---|---|---|---|---|
  | Bake A (start random), old rule | 1,056 | 66,229 | 2.37 % | 17,433 | 540 (26 %) | 8 (0.1 %) | 625 (29 %) | — |
  | Bake A, balance | 1,035 | 62,398 | 2.23 % | 17,433 | 520 (27 %) | 7 (0.1 %) | 30 % of area | 159 / 5 / 26 |
  | Bake B (start supercontinent), old rule | 931 | 192,375 | 4.51 % | 74,262 (32°N, P/PET 0.40) | 618 (29 %) | 0.01 % | 29 % | — |
  | Bake B, balance | 896 | 105,090 | 2.46 % | 9,995 | 581 (50 %) | 0.01 % | 48 % | 167 / 13 / 49 |
  | View frame B (L1 terrain), old rule | 158 | 152,571 | 3.44 % | 28,040 | 108 (31 %) | 3 (1.2 %) | 31 % | — |
  | View frame B, balance | 116 | 92,252 | 2.08 % | 11,497 | 66 (39 %) | 0 | 39 % | 25 / 8 / 54 |
  | View, Ben's case (1000-Myr random run at 676 Myr ago), old rule | 173 | 574,894 | 11.29 % | 374,907 (the polar lake) | 125 (79 %) | 1 | 79 % | — |
  | View, Ben's case, balance | 118 | 528,261 | 10.37 % | 374,907, now drawn frozen | 72 (78 %) | 0 | 79 % | 40 / 11 / 89 |

  - Earth for reference: lakes cover 1.8 % of the land (HydroLAKES, lakes of 10 ha and more: 1.42 million, 2.67 M km²; Messager et al. 2016, Nat. Commun. 7:13603) to 3.7 % of the non-glaciated land (117 million lakes over 0.002 km², 5.0 M km²; Verpoorter et al. 2014, GRL 41:6396). The Caspian is about 371,000–390,000 km².
  - **The polar lake of Ben's case stays full.** It is frozen now, but the balance fills it: mean −13 °C, warmest month +3.6 °C, 141 mm of precipitation against a loss of 230 mm, and a thawing catchment larger than the lake. It only starts to shrink at `lake_evaporation_base_mm` 400 (330,785 km²) and halves at 600 (195,218 km²), which is more than polar lakes lose. See the decisions below.
  - Rivers: the frozen-ground rule removes rivers from ground that never thaws (bake A: 4,228 chains and 77,175 km with master's binary, 3,490 and 68,186 km now, under either lake rule).
- **Cost.** Bake's water part: A 4.5 s → 4.1 s, B 7.2 s (inside the run) → 5.0 s (`deeptime hydro`); no measurable change. View, water derived per frame at 1024 × 512: A 63 → 77 ms, B 75 → 78 ms, Ben's case 127 → 90–112 ms (the satellite picture itself is 225–290 ms).
- **Gates** (test VM): `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (core 361 passed), `cargo deny check licenses bans` ok; `./gradlew build test` tests=160 failures=0. Browser check `tools/workbench/lakes-check.sh` on a private port: 11 of 11 (legend, pointer text on a lake and a frozen lake on both layers). GameTests: `tools/remote/mac.sh --server -- ./gametest.sh` all 57 required tests passed (`riversHold` floating 0/0, leaks 0/0, 749 lake columns; `carversSpareWater` drained 0, holes 0; `lakeHolds` 65,040 before and after); `-PwithMic` not run on the branch (left to the merge batch).
- **Tests** assert properties, not one planet's numbers: no lake leaks (no dry cell beside a lake lies below its level) on rough ground in four climates; drier climates hold less lake; a lake grows with the rain until it spills; an upstream terminal lake feeds nothing downstream; frozen and never-thawing cases; a basin below sea level stands at sea level; thread-count independence.
- **Pictures**: review/out/lakes/ (not committed): `ben676-{before,full,after}[-rivers].png`, `{a,b}-frame-…`, `{a,b}-final-…`, `workbench-{satellite,rivers-layer}.png`.
- **Scratch** under /Volumes/Graphide/ban/deeptime-runs/lakes/ deleted.
- **Decisions for Ben** (in the hand-back report): the polar lake that still fills; dry land below sea level; a salt-flat surface in the game; whether the bake should force lake ice; the runoff law of cold climates.
- **Next step:** merge this branch in the next batch; the coordinator's GameTest run covers it.

### 2026-10-01 — Workbench: the Inspector pops out to a large overlay (agent worktree agent-a2a973aae56130698, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
Ben: "Inspector view on the workbench is a bit small, can you add a 'fullscreen' button so I can pop it out".
- **What**: an expand button in the Inspector card's header (`#inspbig`, `aria-label`, `aria-expanded`, tooltip with **P**).
  It adds a class to the same card (`#inspector.big`, `position: fixed`, nearly the whole window, over a dimmed scrim), so
  nothing is moved or rebuilt: the tab, scrolling, the selected event, the jobs and the 3D iframes stay as they are. A
  stand-in keeps the side panel's height and scroll. The same button (icon and label swap), **P**, **Esc** or a click on the
  scrim put it back. In-page overlay; the browser Fullscreen API adds nothing here (F still does the whole page).
- **Layout** (all CSS under `#inspector.big`, plus an invisible `.ipanes` wrapper): Events shows the list and the details
  side by side (picking an event stays on Events; the Details tab is still there); Details and Run get wider columns
  (key/value rows, 190 px stat tiles); 3D job cards become a grid with the preview iframe `clamp(380px, 62dvh, 860px)` high
  (was 210 px); the Section tab draws the cross-section large in the tab itself (`buildView` from sections.js, via the new
  `dt:inspector-big` event and `DTWB.inspectorBig()`), so its "Enlarge" button is hidden there; the dock's and the side
  panel's Enlarge dialog is unchanged.
- **Accessibility**: the card is `role=dialog aria-modal` while open, Tab and Shift+Tab stay inside it, Esc closes (a
  dialog opened over it, e.g. Enlarge or help, closes first), focus goes to the button on open and close.
- **Not touched**: `?tv` (button and scrim are `display: none`, P does nothing). Phones (<= 760 px): the button is hidden
  and P does nothing, since the inspector already has the page; narrowing the window while expanded closes it. The
  mobile tab bar was checked (screenshots 30, 31).
- **Shortcut**: **P** (free: the list had space, arrows, Home, End/L, G, M, D, C, E, V, I, B, S, [ ], ?, F, 1-9); listed in
  the help ("Pop the inspector out to a large overlay (Esc closes)").
- **Checks** (test VM, private `deeptime watch` on a spare port, a 60 Myr `earthlike_quick_16k` run, Chromium via nix):
  new `tools/workbench/inspector-check.sh` + `inspector_shots.py` (about 55 checks: open/close paths, tab/scroll/selection kept,
  side-by-side events, iframe fills the embed, large section, focus trap, dialog stacking, phone, ?tv, no page errors) all
  pass; screenshots in review/out/wb-inspector/ (not committed). Existing checks re-run: `deposits-check.js`, the deposits
  shot driver (`deposits-shots.sh --reuse`) and `sections-shots.sh` all pass; `cargo fmt --check`, `clippy --workspace
  --all-targets -D warnings`, `cargo test -p deeptime-cli -p deeptime-viewer` (52 + 6) clean.
- **Not covered**: a real 3D preview iframe (needs the Minecraft preview server, off on this workbench): the check uses an
  iframe stand-in inside a real `.embed` card, and the iframe fills its box. Not merged or pushed (asked to leave it).
### 2026-10-01 — Terrain above the build limit, stage 1: the surface moves with its own crust; nothing is cut flat (agent worktree agent-ac2a33b7f51fa4bf8, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Why.** Ben (decisions log 2026-10-01): no hard cap in the surface stage, the height limit should emerge; first
  fix the bookkeeping that built 13–20 km "mountains", add the game-side safety net, measure. Stage 2 (the emergent
  limit) is designed below and not built.
- **The flaw, measured.** A trace of the surface step on the test planet (per step: plate elevation, offset, the
  plate-side change, erosion, deposition, isostasy over the cells above 8 and 10 km). The plates refusing uplift at
  their cap was not it (the cap bound 0.24 % of continental sample-steps on the worst planet, under 0.01 % on the
  16k worlds). Three things were:
  1. **The offset was added to whatever crust a cell read next.** The surface's state was the offset from the plate
     model's elevation, advected by position. The plate model's elevation under a cell is sampled anew every step
     from another mix of L0 samples and changed by 5–8 km in one step with its own uplift there at 0 (the cell now
     reads the high side of a margin, or another plate's edge over a trench): a trench with −6 km of plate and +4 km
     of fill became a 9 km margin plus the 4 km. 5 to 500 cells a step were lifted more than 1 km this way.
  2. **The plates were told only each step's change.** What a sample did not take (the spread onto coarser samples)
     stayed in the surface, so crust the plate model held at 2.8 km stood at 14 km and still took a full collision
     surge (+3.7 km in a step): the uplift ceiling looked at the wrong height.
  3. **Volcanoes rebuilt what isostasy had already given back.** A cone's height was debited the whole erosion of
     the step and rebuilt, while about a quarter of that erosion came back as rebound; its ceiling was measured from the
     ring round the cone, which did not rise. Cones stood on 15–25 km of their own rebuilding (seed 13).
- **The fix** (no clamp anywhere in the surface stage):
  1. `advect::carried_elevation`, used by the surface and the climate stage: a cell starts a step at the last
     elevation of the same crust at its source (corner cells that held the same rigid group, the same plate, or the
     parent of a plate born this step; new L1 field `plates/group_id`, the surface keeps `surface/crust_plate` and
     `surface/crust_group`) plus the plate model's uplift of that crust (`plates/uplift_mm_yr`); where no corner held
     the same crust, the plate model's elevation. The offset is an output.
  2. `surface/feedback_m` is the whole offset, so a sample's elevation is the surface's at its scale.
  3. A cone grows only while every cell it stands on is under the volcano ceiling (the existing
     `volcanoes.max_elevation_km`, measured at the ground now).
  Mass: `mass_residual_km3` stays at 1e-5 to 1e-3 km³ (it was the same before). New counters on the `surface_final`
  line: `land_z_p99_m`, `plate_cap_bound_share` (continental sample-steps at `max_elevation_m`),
  `foreign_crust_cell_steps` (cells that started again from the plate model's elevation).
- **Before and after, deep-time surface on L1** (`deeptime stage volcanoes`, x86_64 VM, 2 threads; "before" is the
  same binary with the three changes switched off, and reproduces master's 20,082 m on the test planet):

  | 500 Myr `earthlike_quick_16k` | max, km | p99 land, km | median land, km | land fraction | cap bound |
  |---|---|---|---|---|---|
  | before: scattered seeds 1, 2, 3 | 9.2, 10.7, 9.2 | 6.7, 6.7, 6.4 | 1.45, 1.27, 1.05 | 0.28, 0.29, 0.37 | 0, 6e-5, 2e-5 |
  | before: supercontinent 1, 2, 3 | 12.4, 9.1, 13.0 | 7.1, 6.4, 7.4 | 1.08, 1.31, 1.17 | 0.43, 0.42, 0.43 | 5e-5, 2e-5, 6e-5 |
  | after: scattered 1, 2, 3 | 10.4, 9.1, 11.1 | 5.7, 7.1, 7.1 | 1.20, 2.03, 1.53 | 0.38, 0.50, 0.39 | 1e-5, 0, 1e-5 |
  | after: supercontinent 1, 2, 3 | 11.2, 10.2, 9.0 | 7.5, 6.9, 7.4 | 2.07, 1.08, 1.52 | 0.47, 0.26, 0.38 | 5e-5, 3e-5, 0 |

  Means over the six: max 10.6 → 10.1 km (highest 13.0 → 11.2), p99 6.8 → 7.0 km, median 1.22 → 1.57 km, land
  fraction 0.37 → 0.40. Test planet (`planet_world_4k`, 150 Myr, seeds 11–16): max 20.1, 7.5, 16.2, 2.5, 3.3, 4.6 km →
  4.4, 4.2, 4.5, 4.4, 3.0, 3.7 km (the 4.4 km are shield volcanoes at their ceiling); p99 15.7, 6.1, 13.1, 2.3, 2.1,
  3.6 → 3.1, 3.6, 3.9, 3.4, 2.1, 3.0 km; median 0.49, 0.69, 0.73, 0.18, 0.59, 0.30 → 0.33, 0.58, 0.43, 0.24, 0.59,
  0.34 km; land fraction 0.19, 0.13, 0.14, 0.06, 0.19, 0.15 → 0.18, 0.10, 0.08, 0.06, 0.18, 0.15; cap bound 0.0024,
  0.0016, 0.0024, 0, 0, 0.0005 → 0 on all six.
  **For the hypsometry agent:** the 16k land median rises by about 0.35 km (six runs each side, chaotic histories, so
  ±0.3) and the p90 from 3.9 to 4.6 km: recalibrate on this base. The parameters themselves did not move.
- **The game-side safety net.**
  - `scale.build_limit_band_blocks` (default 48; pipeline and built-in presets, editor field `build_limit_band` in
    Scale, lang key): the stored elevation is as simulated up to a knee at `max_y` − 16 − band (y 575, 10.24 km above
    the sea by default) and above it y ↦ knee + B·(1 − e^(−(y − knee)/B)), which never reaches `max_y` − 16, so with
    the detail term nothing reaches `max_y` (`params::BuildFit`; in the amplifier's sink and the bake's own
    upsampling). Increasing, slope 1 at the knee: ridges stay ridges and drainage keeps its direction.
  - **Preset check: not a rejection.** I first refused presets whose `tectonics.max_elevation_m` does not fit under
    the knee; the editor's contract is that both ends of every slider are valid
    (`every_knob_is_in_every_pipeline_preset_and_both_ends_are_valid`), and `m_per_block_v` goes down to 1. So a
    world too short for its mountains is squeezed, and the bake says how much
    (`notes.bake.terrain.build_limit`: `knee_y`, `top_y`, `cells_compressed`, `cells_above`; `compressed=` on the
    `bake` line). Validation refuses only a band under 8 blocks or a knee less than 64 blocks above the sea. The
    other choice (raise the slider's minimum to 19.6 m and refuse) is Ben's.
  - `realPlanetHeightMatchesTheBake` fails on any sampled column above the limit and on `cells_above` > 0 or a
    highest elevation within 16 blocks of `max_y` in the bake's note.
- **Two more one-planet tests** (the planets changed again, as they will with every change to the simulation):
  `realPlanetRockMatchesTheParityFixture` failed on seed 14, whose fixture columns cross coal seams: coal's stand-in
  without a coal rock is plain stone, which `classesOf` reads as unclassified crust (now the class is asked for its
  blocks). `climateConsistent` floors re-measured, see the test's comment.
- **Does stage 1 alone keep Earth-like worlds near 8–10 km? Nearly, and not for the right reason.** The 16k worlds
  peak at 9.0–11.2 km on L1 (the amplifier adds up to about a kilometre), so half of them reach into the compression
  band above 10.24 km and none is cut. But what holds them there is still the plate model's uplift ceiling
  (uplift × (1 − z/`max_elevation_m`)), which now sees the true height: a number, not a mechanism, and it does not
  know gravity. The hard clamp itself almost never binds (at most 0.005 % of continental sample-steps, before and
  after).
- **Stage 2, designed, not built: crust too thick for its strength spreads.**
  - *Mechanism.* Lower-crustal channel flow / gravitational collapse: where the weight of elevated crust exceeds what
    the crust can hold, the lower crust flows out from under it and the surface relaxes sideways into a plateau
    (Tibet, the Altiplano). As topographic diffusion with a threshold: ∂h/∂t = ∇·(D ∇h), D = D₀ · max(0, σ/S − 1)³,
    σ = ρ_c · g · h (h above sea level, ρ_c 2,800 kg/m³, g the planet's `gravity_m_s2`). The cube is the channel
    law (flux ∝ channel thickness³, Clark & Royden 2000); below the threshold nothing moves.
  - *Settings.* `tectonics.crust_strength_mpa` (S, default about 140: ρ_c·g·5.1 km on Earth, the level of the high
    plateaus) and `tectonics.crust_flow_km2_myr` (D₀ at twice the threshold, default to be calibrated, of the order
    of 10³ km²/Myr: Clark & Royden's range is 10² for a strong, 10⁵ for a weak lower crust). Editor: Advanced ·
    Plates.
  - *Where.* In the surface step, on L1, after the plate model's uplift is added and before erosion; conservative
    (Σ Δh · area = 0), so it moves crust, not surface rock: the strata log is untouched and `mass_residual` is not
    affected; the change reaches the plate model through the feedback. Implicit, with the existing diffusion
    solver on the cells over the threshold and a ring round them: a few per cent of the surface step.
  - *What it changes visibly.* Collision zones stop growing upward at about S/(ρ_c·g) and grow outward: broad high
    plateaus with steep edges instead of ever higher cores; summits are the plateau plus what erosion carves and the
    amplifier adds (Earth: 5 km + 3–4 km). Ranges under the threshold (most of them) do not change.
  - *Gravity.* Through σ only: the plateau level is S/(ρ_c·g): 5.1 km at 1 g, 13.4 km at 0.38 g, 2.5 km at 2 g. The
    volcano ceiling (`volcanoes.max_elevation_km`, 6.8) should become the same comparison (an edifice's load against
    the same strength), or Mars-gravity shields stay at 4.4 km.
  - *The plate model's cap.* The uplift ceiling's `max_elevation_m` then has to move out of the way (30 km, a guard),
    or the plate model keeps doing the work at 1 g and blocks taller ranges at low g. That changes how fast ranges
    rise below the threshold, so the hypsometry calibration has to follow it: this is the part with a cost.
  - *Erosion already rises with relief:* on the cells above 10 km the trace shows 0.3 to 1.3 km/Myr of erosion
    (stream power on the steep flanks, the glacial pass above the snowline) against 0.02–0.1 km/Myr on the land's
    average, and about a quarter of it returned by isostasy.
  - *References.* England & McKenzie 1982 (thin viscous sheet; GJRAS 70:295); Molnar & Lyon-Caen 1988 (the weight of
    a range against the forces that build it; GSA Spec. Pap. 218); Bird 1991 (lower-crustal flow under plateaus;
    JGR 96:10275); Royden et al. 1997 and Clark & Royden 2000 (channel flow, topographic diffusion with D ∝ h³;
    Geology 28:703); Rey, Vanderhaeghe & Teyssier 2001 (gravitational collapse; Tectonophysics 342:435); Beaumont
    et al. 2001 (channel flow coupled to erosion; Nature 414:738).
  - *A choice for Ben inside stage 2:* whether the plate model's uplift ceiling goes (recommended: yes, with a
    recalibration of uplift rates by the hypsometry work) or stays at 10 km with the spreading under it (smaller
    change, but then Earth's limit is still the number and only low-gravity planets differ).
- **Measured** (VM `deeptime-test`, x86_64 Linux, `mac.sh` default caps; load average 10–30): `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo deny check licenses bans` clean; `cargo test --workspace --locked` all pass (509: cli 52, core 353 + 3 ignored); `./gradlew build test` tests=160 failures=0 errors=0 skipped=1; `./gametest.sh` All 57 required tests passed (wall 485 s) and, after `tools/smoke/install.sh mic`, `./gametest.sh -PwithMic` All 57 passed (81 s), both with `above_max_y=0cells compressed=0cells highest_y=281` on the default planet. The nine real-planet tests on other planets (`DEEPTIME_TEST_PLANET_SET="seed=N"`): seeds 12, 13, 15, 16 standalone All 9 passed; seed 14 failed two (the rock and Dsub items above), and with those fixed seed 14 standalone and seeds 13 and 14 with MIC All 9 passed. On all six seeds the bake has 0 cells above `max_y` and 0 compressed (highest y 274–287); seeds 11 and 13 had 2.5 % and 1.0 % of their land above it before. The two full `./gametest.sh` runs were made one commit before the rock-test and Dsub-floor changes (test code only); cargo, Gradle and the seed-14 and MIC seed runs are on the final code. x86_64 Linux only.
- **Next.** Ben's go for stage 2 and his choice on the uplift ceiling; the hypsometry agent
  (worktree-agent-aee888e01d29a40db) takes this base first.
### 2026-10-01 — P1-28 stone and deepslate for the common rocks (agent worktree agent-a72907749021691b5, branch worktree-agent-a72907749021691b5, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Ask.** Ben, 2026-10-01 (decisions log): "I'm seeing very little minecraft:stone, will things like stone tools still be easily
  craftable?" The first answer (hardness tiers for `minecraft:stone_tool_materials` / `stone_crafting_materials`, tool-grade rock near
  spawn, river/beach/till cobbles) was **replaced** the same day: no recipe or item tag is widened ("How can we fix this w/o just
  having a cheeky lil 'yeah, your crafting recipes can now use some extra materials'"), and "Use stone for common rocks and we
  should be chilling". That work was built and then reverted (commits bb7f2f5 and e601959, reverted by 678a0e7 and ef4272e; the
  history keeps it: the per-class `stone_use` tiers with reasons, the generated item tags and the `ground` settings draft). Nothing of it
  ships.
- **What changed.**
  - `tools/dev/gen_rock_classes.py`: `CRUST_STAND_INS`, class → (block, why). `minecraft:stone` is now the `vanilla_fallback` of
    `shale` and `mudstone` (coal already had it); `minecraft:deepslate` of `slate`, `phyllite`, `schist`, `gneiss`. Each class file
    says why in `sources.vanilla_fallback`. The generator's rule that stone and deepslate are never a class block is gone (its
    validation now demands exactly the table's classes).
  - Why these: on the 500 Myr `earthlike_quick_16k` planet the top-64-block share of the placed classes is shale 34.7 %, gneiss 17.6 %,
    mudstone 1.3 %, schist 1.0 %, slate 0.9 %, phyllite 0.7 % (the rest: sandstone 25.8 %, rhyolite 7.7 %, basalt 1.8 %, gypsum 1.7 %,
    granite 1.1 %, black shale 0.8 %, …). Shale and mudstone are the fine-grained grey sedimentary rocks nothing in vanilla looks
    like; slate, phyllite, schist and gneiss are one dark-grey foliated pelite-to-gneiss series. Kept on their own blocks: granite,
    diorite, andesite, basalt, tuff, sandstone, red sandstone, calcite (limestone, marble, the evaporites, chalk), dripstone, obsidian,
    cobblestone for conglomerate; and, as rare (each under 0.3 % of the surface blocks) with a look worth more than their share,
    hornfels, amphibolite, greenstone, blueschist, eclogite, quartzite, serpentinite; black shale (0.8 %) keeps smooth basalt as
    its vanilla-only stand-in (Born in Chaos' argillite in MIC) so the source rock stays recognisable.
  - `PaletteRules`: only air is rejected now. `RockPalette.classesOf` gives, for stone and deepslate, every class that resolves to
    them plus `unclassified_crust` (`RockPalette.UNCLASSIFIED_CRUST`). `BackgroundScatterFeature.hostClasses/bias`: where the host
    block is stone or deepslate the horizon at the position (not the block) decides the host bias, so the coal scatter still prefers
    shale and mudstone over the unclassified crust; only rules whose bias depends on the class pay for the lookup
    (`OreData.ScatterRule.classBlind`). The strata pass is unchanged: a class that resolves to stone is painted stone even where the
    crust was deepslate (and the reverse).
  - `deeptime-inspect rock-census` (new, `crates/deeptime-inspect/src/census.rs`): per-class shares of a baked planet's dry land (top
    rock, 4 blocks down, top N blocks, whole column), review-box examples, and with `--stone-classes` the share of land with no
    stone-class rock within a reach. `tools/review/p1-28-stone-share.py` maps a census to the pure-vanilla blocks of two revisions;
    `tools/review/p1-28-block-share.py` counts stone/deepslate in generated boxes (`.dtva`).
- **Measured.** Planet: `deeptime run --preset earthlike_quick_16k --start scattered_continents --length-myr 500 --threads 4` on the VM
  (land 21 %, 49,642 land columns at step 24). Top rock under the soil: shale 36.3 %, sandstone 26.2 %, gneiss 18.6 % (Ben's run: shale 39 %).
  Stone or deepslate share of the placed rock blocks, **before → after**:

  | | top 64 blocks | whole column |
  |---|---|---|
  | census, whole planet, pure-vanilla pack | 0.1 % → 56.2 % | 0.0 % → 56.0 % |
  | generated, 12 boxes of 96 × 96 (the real generator, standalone) | 0.0 % → 47.8 % | 0.0 % → 55.3 % |
  | generated, same boxes, MIC, rows applied as an overlay (before: pack as is) | 0.0 % → 47.6 % | 1.5 % → 43.8 % (deepslate 34.1, stone 9.7, and blackstone 10.2) |

  The 12 boxes are evenly spread dry-land columns (`census.examples["*land*"]`), not picked by rock. Most of the 56 % is
  stone (shale, 35 %) in the top blocks and deepslate (gneiss) in depth. **MIC finding:** in the MIC after run blackstone is 10 % of the
  whole column (y −320…−160 in five boxes, 0.2 % before): a Terralith cave feature
  (`terralith:cave/…/blackstone`) turns deepslate into blackstone, and deep gneiss was granite before, so it had nothing to work on.
  Not part of this change; Ben may want it off or accepted.
- **A start in granite or basalt country.** Only this change, no cobbles and no spawn rule: stone and deepslate come from the six classes
  above, so a start on pure granite/basalt/andesite/sandstone/gypsum ground has no stone or deepslate at the surface. From the census (the
  classes shale, mudstone, coal, slate, phyllite, schist, gneiss; `--stone-classes`, step 16): no such surface rock within 32 / 64 / 128
  blocks of 31 % / 24 % / 13 % of the dry land; none even within 32 blocks of digging down (of which any of them counts) of 17 % / 11 % /
  5.7 %. Those players have granite, diorite, andesite, smooth basalt, sandstone to build with but no cobblestone for stone tools or a
  furnace until they walk to the next stone or deepslate (under 130 blocks for 94 % of land); a pickaxe of wood or iron mines
  granite and drops granite, which crafts nothing stone. That is the cost of dropping the spawn guarantee; see Ben's options in the report.
- **Mining.** `miningTheCommonRocksDropsVanillaCobbles`: stone → cobblestone, deepslate → cobbled deepslate, and 12 other placed
  blocks drop their own item, unchanged; no loot table, recipe or item tag comes from Deep Time's jar (checked through the resource
  manager's pack ids), and a JUnit test asserts `data/minecraft` holds only the world preset tag. Side effect worth knowing: vanilla
  ore features target `#stone_ore_replaceables` and `#deepslate_ore_replaceables` (stone, granite, diorite, andesite / deepslate,
  tuff), so ore that vanilla features place now has far more stone and deepslate to sit in (polished tuff and packed mud, the old
  shale and mudstone blocks, were in neither tag).
- **MIC rows** (not applied; MIC's datapack and README were not edited). The MIC pack's fallback tags are read before
  `vanilla_fallback`, so MIC worlds stay as they were until these change. In `deeptime-mic-rock-tags` (and the pack's copy under
  `pack/kubejs/data/deeptime/`), replace the fallback tag files `data/deeptime/tags/block/rock_fallback/<class>.json` with
  `{"replace": true, "values": ["minecraft:stone"]}` for `shale` and `mudstone` and `{"replace": true, "values": ["minecraft:deepslate"]}`
  for `slate`, `phyllite`, `schist`, `gneiss` (or simply edit their value lists to that single entry). README rows to change:
  Classes table, Stand-in column: `mudstone` `minecraft:packed_mud` → `minecraft:stone`; `shale` `born_in_chaos_v1:black_argillite` →
  `minecraft:stone`; `slate` and `phyllite` `born_in_chaos_v1:black_argillite` → `minecraft:deepslate`; `schist` `minecraft:tuff` →
  `minecraft:deepslate`; `gneiss` `minecraft:granite` → `minecraft:deepslate`. Fallbacks table, rows `mudstone`, `shale`, `slate`,
  `phyllite`, `schist`, `gneiss`: stand-ins "`minecraft:stone`" / "`minecraft:deepslate`", why "the commonest sedimentary rock, fine, grey,
  fissile; vanilla has nothing like it" (shale), "shale's unlaminated twin" (mudstone), "metamorphosed shale: dark grey, foliated" (slate,
  phyllite, schist), "banded grey crystalline basement" (gneiss). Text: the Vocabulary row of `deeptime:rock/unclassified_crust` ("Never
  used as a stand-in, with one documented exception") and the paragraph "Stand-ins follow Ben's rules: … never `stone` or `deepslate`
  (reserved for `unclassified_crust`), except `stone` as the last `coal` fallback" become "stone and deepslate are also the stand-ins
  of the six common classes above (decisions log 2026-10-01); `unclassified_crust` stays the filler no class owns". `black_argillite`
  stays the member of `black_shale` and the first fallback of `coal` and `hornfels`'s fired variant. Checked as an overlay datapack
  (`replace: true` tags for the six classes) on the MIC server set: stone 35.8 % and deepslate 11.8 % of the top 64 blocks (table above).
- **Gates** (VM `deeptime-test`): `cargo fmt --check`, `clippy --workspace --all-targets -D warnings`, `cargo test --workspace`,
  `cargo deny check licenses bans` clean; `./gradlew build test` 161 tests, 0 failures; `./gametest.sh` and `./gametest.sh -PwithMic`
  (after `tools/smoke/install.sh mic`, `--pack`): All 58 required tests passed each (the first default run had 3 failures, all the
  `deepslate[axis=y]` state string in my new assertions; fixed).
- **Next.** Ben decides the options in the report (spawn cobbles for stoneless starts; MIC rows; the Terralith blackstone). The coordinator
  merges and runs the batch GameTests.

### 2026-10-01 — Workbench deposits overlay: coal fields and ore sites by family, hover, "why here" (agent worktree agent-aae4250c131f0cf7a, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Ask.** Ben, 2026-10-01, "Deposits layer": draw coal fields and ore deposits by type on the map at every frame where the
  deposit stage has run (and the baked deposits at the end); one symbol and colour per family, coal as footprint areas; a legend
  with counts; hover for family, size, age; click opens the inspector with the site's event and cause chain; works over any
  base layer; view-only (nothing added to the simulation or the bake); run time unchanged.
- **Overlay, not a layer (why).** Keyframe layers are RGBA images the core draws on the stepping thread at keyframe time. They
  cannot carry symbols to hover or click, a new one would be extra work on that thread for every frame, and it would only
  exist where a stage names it. An overlay drawn on the pane's canvas from the site list works over satellite, elevation and
  any layer the other workbench changes add, like the event markers do (same `mapXY` projection). Toggle: the Deposits button
  in the map overlays group (key `D`, remembered); the legend sits in the Layer card.
- **Data (nothing new is computed).** `GET /api/runs/<id>/deposits` (`watch/deposits.rs`, new): one JSON object with each site's
  type, lat/lon, block position, depth range, footprint ellipse (the widest of its bodies, else its box), products with
  tonnage and the forming event id. While a run goes on, the driver thread takes it once from `world.sites`/`world.bodies`
  between two steps, when the first published frame names `deposits/sites` (`Run::capture_deposits`, 3 lines in
  `Workbench::publish`); a done run answers from `sites/sites.dtsr` + `bodies/bodies.dtb` of its world data (the baked deposits;
  also what a restarted workbench shows). Age and the cause chain come from the events the page already holds. `git diff
  --stat` touches nothing in deeptime-core or deeptime-format. The overlay shows from the first frame that names the sites layer
  on (the deposit stage and the young/amplify frames after it); earlier frames say when the sites appear.
- **Look.** Eight families, colours = the data-viz palette's dark-mode categorical steps in slot order (blue MVT, orange
  porphyry, aqua supergene, yellow placer, magenta SEDEX, violet petroleum, red BIF; green left out, it is the land), checked
  with its `validate_palette.py` (all checks pass), plus shape as the second cue (hexagon, up/down triangle, star, square, drop,
  diamond); coal is a charcoal footprint with a light outline, drawn at its real size (widest body's ellipse) but never under 7 px
  across. A symbol's size follows its rank by tonnage inside its family. Sites that share a spot (a blanket under its porphyry)
  are drawn side by side. Hover: card with family, size (tonnes of the primary product), footprint km2, age (formed X Myr ago),
  grade and by-products; the site's footprint and the link to its parent or children light up. Click (Select tool): the Details
  tab gets the site card (facts, yields), "Why here" (the forming event and its causes four links deep, breadth first as
  `inspect-column --why` lists them, with levels), links to the deposit it came from or led to, then the forming event's own
  detail (the Events tab row is selected). Legend rows toggle a family (Alt-click: only it), counts include zeros.
- **Pieces.** `crates/deeptime-viewer/assets/deposits.js` (new, served as `/deposits.js`, pure functions in `DTDeposits`: families,
  model, draw, hit-test, legend/hover/details text, cause chain), `watch/deposits.rs` (new), small hooks in `workbench.js` (one
  section plus one-line hooks in `RunView`, `drawPane`, `legend`, `buttons`, the key handler), `workbench.html` (button, legend slot,
  script tag), `http.rs` (two routes), `runs.rs` (cache, capture), viewer `lib.rs` (`DEPOSITS_JS`). Checks: `tools/viewer/deposits-check.js`
  (node: the module, run on the Mac), `tools/viewer/deposits-shots.sh` + `deposits_shots.py` (private `deeptime watch` on 127.0.0.1 and
  headless Chrome for Testing from the playwright cache, real mouse input; never port 8642).
- **Measured / checked (Mac, before the stop).** `cargo clippy --workspace --all-targets -D warnings` and `cargo fmt --check` clean;
  `cargo test -p deeptime-cli -p deeptime-viewer` watch tests pass, incl. new `the_deposits_overlay_reads_the_sites_live_and_baked`
  (synthetic deposit world: the live capture and the bake agree site for site, every forming event is in the stream, a restart
  reads the same sites) and 5 `watch::deposits` unit tests; `node tools/viewer/deposits-check.js` passes (hit-testing across the
  seam, coal areas, co-located sites, draw counts, text, cause chain); the browser driver on the synthetic 16k deposit world
  (135 sites, 8 families): legend counts add up, hover cards, click opens Why here, kin links, overlay over another layer, family
  and overlay toggles, no script errors (two checks failed only on upper-case headings in the driver, fixed, not rerun).
  Screenshots of that run (before the last legend/nudge fixes) are in `review/out/wb-deposits/` (not committed).
- **Gates (Azure VM `deeptime-test`, branch at master 5c45b04's mac.sh).** `cargo fmt --check` and `cargo clippy --workspace
  --all-targets -- -D warnings` clean; `cargo test --workspace` all ok (core 335 + 3 ignored, cli 37, format 51, inspect 29, viewer 7, jni 6,
  ...; includes `watch::deposits::*` and `the_deposits_overlay_reads_the_sites_live_and_baked`); `cargo deny check licenses bans` ok;
  `node tools/viewer/deposits-check.js` ok; `neoforge/gradlew build test` BUILD SUCCESSFUL (3 m 22 s; nothing in neoforge changed).
  No GameTests (per the rule).
- **Run time before/after (the gate "run time unchanged").** `earthlike_quick_16k`, scattered_continents, 500 Myr through `deeptime
  watch --exit-when-done --threads 6 --mem-cap 8 GiB`, release binaries built on the VM, one run at a time, `/usr/bin/time -v`,
  box shared (load 33 to 57 from other agents, so compare CPU, not wall): **before** (8dae3f2): 858 s wall, **2194 s user** +
  38 s sys, 5.99 GB max RSS; **after** (this branch): 814 s wall, **2249 s user** + 35 s sys, 5.87 GB. Same world both times
  (2,767 events, 253 frames, identical `surface_final` line to the last digit), so the overlay changes nothing in the run; the
  +2.5 % user CPU is inside the noise of a loaded box (the per-stage seconds swing the other way: plates 118 vs 103 s, deposits
  27 vs 36 s). The only addition on a run's thread is one clone of the site list between two steps (about 55 sites here).
  (The earlier M6 baseline, 1354 s wall / 21.39e12 instructions, is not comparable: different machine, 12 threads, load 60 to 90.)
- **Browser checks on a real run** (`tools/viewer/deposits-shots.sh`, private server on 127.0.0.1 with the page open while the run
  went on, Chromium from nixpkgs, headless; 0 failed): the legend appears when the first frame names `deposits/sites`; while the run
  is going the legend says the sites are as the stage placed them (live), and when it ends it switches to the baked sites with no
  reload; 55 sites (coal 28, petroleum 5, porphyry 4, placer 9, MVT 7, SEDEX 2; supergene and BIF 0 on this planet) and the legend
  counts add up to what the server sent; hover on a coal field and a porphyry shows family, size, footprint, age (and grade and
  by-products); clicking opens the Details tab with the site card, "Why here" (the cause chain) and the forming event; the overlay
  stays over the elevation layer; a legend row hides its family; the toggle hides and restores the overlay and legend; the
  legend says when the sites appear on steps before the deposit stage; no script errors. **Draw cost** (2048 x 1024 canvas, in the
  page): 55 sites draw in 0.2 to 0.3 ms and hit-test in 0.08 ms; ten times as many sites (550) draw in 12.9 ms (prepare 7.5 ms,
  once per fetch). Screenshots in `review/out/wb-deposits/` (not committed): `00-live-overlay` (mid-run), `01-satellite-overlay`,
  `02-hover-coal`, `03-click-coal-why-here`, `04-hover-porphyry_cu_mo_au`, `05-click-porphyry_cu_mo_au`, `07-elevation-overlay`,
  `08-family-hidden`, `09-overlay-off`, `10-first-step`. Linux Chromium needs a font: `~/.config/fontconfig/fonts.conf` on the VM now
  points at nixpkgs `dejavu_fonts` (without one the page draws no text); `deposits-shots.sh` finds Chromium on PATH or builds
  `nixpkgs#chromium`.
- **Merged with the other three workbench features (master e4094d6: rivers & lakes, Final world, rock at depth and sections).**
  Conflicts in STATUS.md, `watch/http.rs` (both new routes and docs kept), `workbench.js` (the help list: S and D keys) and viewer
  `lib.rs` (all three script constants); the script tags and `DTWB` needed nothing. Re-run on the VM: fmt, clippy `-D warnings`,
  `cargo test --workspace` (all ok; core 351), deny, `deposits-check.js`, `gradlew build test` (ok, 2 m 5 s); the browser driver on a
  fresh 500 Myr run (live, then baked; 0 failed) now also checks, with the overlay on: the Rivers & lakes layer shows
  (`11-rivers-overlay`), the section tool opens and two clicks fill the Section panel (`12-section-overlay`), the Final world view
  opens and closes with the overlay still there (`13-final-world`). Draw cost on the merged page: 55 sites 0.5 ms, 550 sites 17.8 ms.
- **Left / not done on purpose.** Deposits on the globe, and sites at their position before the end of the run (the record holds
  only the present position; the engine's per-frame origin maps are not kept). The supergene/BIF rows are empty on this seed, so the
  kin links (placer to porphyry, blanket to porphyry) were checked on the synthetic 16k deposit world, not on the 500 Myr planet.
### 2026-10-01 — Rivers & lakes on the workbench maps: view-only, on every frame, the bake's own on the last (agent worktree agent-a3d4fadfe3e053b77, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **What (Ben, 2026-10-01: "Rivers & lakes on the maps").** Rivers (width and colour by flow) and lakes are drawn over the satellite view of every terrain frame, and a new derived layer `rivers` ("Rivers & lakes") shows them on a quiet map. Nothing is added to the simulation or the bake (no new field, no stage work): like `satellite` and `provinces` it is derived when a frame is drawn.
  1. **`core::water_view`** (`water_view.rs`, tests in `water_view/tests.rs`)
     - **Derivation.** `derive` runs the bake's own hydrology, `hydro::network::run`, on the frame's elevation (`amplify/elevation`, else `young/elevation`, else `elevation`; a grid over 600,000 cells is averaged 2 × 2 down first, so a 32k planet's L1 and the 2048 × 1024 young/amplify frames cost what an L1 frame costs). That is: open sea (≥ 5,000 km²), Priority-Flood of the frame grid, lakes from `hydro.lake_min_area_km2` and a block deep, the climate's runoff (`climate/precip_mm`, `t_mean_c` bilinear, as the bake reads them; the zonal stand-in without a climate stage) accumulated down the steepest descent, rivers from `hydro.river_min_discharge_m3s` (2 m³/s) and `river_min_area_km2` (50 km²). The preset's `hydro` section drives it (new `ViewScale` fields: `hydro`, `m_per_block_h`, `rivers_scale`; `hydro.enabled` false = no water and no layer). The thresholds are physical, so they scale with the grid by themselves (a 4.8 km L1 cell: a stream starts where about nine cells drain into it). I used the bake's code rather than a new Priority-Flood plus `surface/drainage`: it is the same rule the bake applies (so the last frame and the others agree), already tested, and costs little (the flood is 13–26 ms of the 104–237 ms the whole derivation took on the loaded Mac, below).
     - **Lake cells under a block deep are not drawn** on derived frames (a shallow margin of a 4.8 km flat read as streaks); the bake's `water_level` is drawn as it is.
     - **Drawing.** A `WaterView` holds the water in the picture's pixels: river coverage (anti-aliased capsules along the network's segments, the larger coverage and discharge kept where segments meet, so a network sampled finer than the picture does not thicken), the discharge under each pixel, the lake coverage (the mean over the cells a pixel covers), and a hide factor from `climate/ice_thickness_m` (satellite only: none show under 60 m of glacier ice). Width is at least 1 px at the stream threshold, +0.5 px per tenfold of flow, and never under the real width (`w` blocks); no water on pixels a block or more below sea level. Colour is the new `rivers` ramp (`ramps.json`, log10 m³/s: `#4aa0f2` at 2 → `#0f3f95` at 3,162; its `keys` hold land `#ddd9cd`, sea `#2c4056`, lake `#5cc0ea`, checked against the constants by a test).
     - **On the satellite view:** `overlay` blends rivers at 60 % (a stream) to 92 % (the largest), lakes at 95 %, after the hill shade. **As the layer:** `render` draws land (soft hill shade), sea, lakes and rivers hard-edged so the page reads the colour back.
  2. **Frames.** `keyframes::render` derives the water once per frame for both layers; `DERIVED_LAYERS` and `add_derived` gain `rivers` (last layer of every frame that has the satellite view; the `add_derived` test updated). `satellite::heights_m`, `elevation_field` and `nearest` were pulled out for sharing (no behaviour change). `Session::take_keyframe` renders inside the session's pool and logs `drawn in N ms` per frame (debug level).
  3. **The last frame is the bake's.** `Session::finalize_write` runs `bake_water_into_last_frame` right after the bake and before the frame files are listed in the manifest: `water_view::from_bake` reads `rivers/rivers.dtrv` and the `water_level` layer (any grid: L3 8192 × 4096 is averaged per pixel), `keyframes::redraw_water` redraws the last frame that has `satellite`/`rivers` (the other layers are kept) and `KeyframeStore::replace` rewrites the ring slot, `kf-N.dtk` and the index entry. A run without a bake (or `hydro.enabled` false) keeps the derived water; a failure is a WARN, not the run's.
  4. **The workbench holds the last frame back until then** (`watch/runs.rs`: `publish` takes `held_from`; the frames of the last advance are announced after `finalize_write`), because a frame's PNG is cached for good once fetched. The page therefore shows the final frame a bake's length (30–70 s) after the last step instead of at it.
  5. **Page** (`workbench.js`, local to the layer list, legend and hover): `rivers` is "Rivers & lakes"; its legend is Lake / Land / Sea swatches and the river colour bar with ticks 2, 10, 100, 1,000 m³/s; hover reads the pixel (and the 3 × 3 around it, rivers are 1–3 px wide) back: "river (~110 m³/s)" or "lake". `deeptime.layer.rivers` in the game's lang file. `deeptime satellite --in <checkpoint> --out x.png [--on elevation]` now writes the water too (`x-rivers.png`) and prints what the satellite picture, the derivation (per phase) and the drawing cost.
- **Per-frame cost** (`deeptime satellite` on a 120-Myr `earthlike_quick_16k` checkpoint, L1 `elevation` frame = 1024 × 512, release build, Mac at load 50–60 from other sessions so every figure is inflated several times over an idle machine):
  - water derived 104–237 ms (phases, ms: sea 4–8, flood 13–26, flats 8–123, receivers 8–26, lakes 1–2, discharge 18–59, rivers 7–14); drawn (overlay + layer) 4–13 ms; the satellite picture alone 92–270 ms in the same conditions. So the water costs about what the satellite picture does, within the ~0.1–0.2 s asked for.
  - In the 500-Myr workbench run, all 253 frames' drawing (satellite + provinces + the water, load 60–100 and my own other jobs): mean 658 ms, median 530 ms (frames: `review/out/wb-rivers/log-after.txt`).
- **Before / after, one `earthlike_quick_16k` 500-Myr run** (redone 2026-10-01 on the Azure test VM `deeptime-test`, after the M6 stop; one `deeptime watch --preset earthlike_quick_16k --start scattered_continents --length-myr 500 --exit-when-done --mem-cap 8589934592 --verbose` per binary under `/usr/bin/time -v`, a keyframe every step, 253 frames, seed 1, baseline = master 8dae3f2 built on the VM, run back to back, VM load 40–100 from other agents so wall time and stage seconds are noisy; logs `review/out/wb-rivers-vm/log-{before,after}.txt`):

  | | before (8dae3f2) | after (this branch) |
  |---|---|---|
  | user CPU | 2,606.7 s | 2,574.7 s (−1.2 %) |
  | system CPU | 165.3 s | 171.2 s |
  | wall | 781 s | 784 s |
  | structure stage (includes drawing the 250 frames) | 251.5 s | 273.4 s (+22 s, 0.09 s a frame) |
  | other stages (plates / climate / surface / volcanoes) | 93.2 / 86.1 / 166.2 / 36.2 s | 103.0 / 86.2 / 181.3 / 39.1 s |
  | deposits / young / amplify | 47.9 / 8.4 / 59.6 s | 29.2 / 6.6 / 41.0 s |
  | bake total (water part) | 31.4 s (6.6 s) | 23.3 s (5.2 s) |

  The user CPU and wall times agree within 1.2 %, well inside the run-to-run noise on this shared box (the stage seconds move ±10–40 % either way, deposits and amplify faster after, structure slower): the water adds no measurable run time. Only the structure stage, which draws the frames, is slower, by about 0.09 s a frame, in line with the per-frame cost above. The after run's own log gives the whole frame draw (satellite + provinces + water) on that loaded box: mean 801 ms, median 830, p90 910, max 977 ms over 253 frames. Chromium on the VM (`nix shell nixpkgs#chromium` + a fonts.conf for DejaVu) took the screenshots.
- **End to end** (a 60-Myr `deeptime watch` run, private port 18653, final binary): `keyframe 32 drawn again with the bake's water (5103 river chains, 16714 px of lakes) in 560 ms`; the bake wrote 672 lakes and 5,103 chains (water part 7.0 s). Screenshots of the workbench on that run, `review/out/wb-rivers/` (not committed; the final ones, of the 500-Myr after run, are in `review/out/wb-rivers-vm/`: `wb-satellite.png`, `wb-rivers.png`, `wb-rivers-hover-river.png` "river (~110 m³/s)", `wb-rivers-hover-lake.png` "lake"): `wb-satellite.png` (final frame, satellite with the water), `wb-rivers.png` (the layer with its legend), `wb-rivers-hover-river.png` ("river (~110 m³/s)"), `wb-rivers-hover-lake.png` ("lake"), `frame{5,15,28,31,32}-{satellite,rivers}.png` (derived L1 frames, the young frame derived on L2, and the bake's final frame), `ck-l1*.png`/`ck-l2*.png`/`f-l1.png` (the 120-Myr checkpoint on `elevation`, on `young/elevation`, and with the glacier-ice hiding), the logs.
- **Gates** (all on the Mac before the stop, on this branch; nothing after the last small edit but a local rustfmt):
  - `cargo test --workspace --no-fail-fast`: core 347 + 3 ignored, cli 31, format 51, inspect 29 + 2 + 7, jni 7, viewer 6 — all pass. 13 new core tests in `water_view` (a valley's river grows to the sea and a bowl holds a lake; thresholds follow the preset's hydrology and `enabled`; width grows with flow; overlay after the shading; the layer's colours read back from the ramp; ramps.json `keys` equal the constants; the bake's file draws the same rivers as the derivation; a river across the seam; lake coverage; a fine grid is averaged down; 1 thread = 3 threads; ice hides the water on the satellite only; one is a bare default check) and 2 in `keyframes` (the frame's layers and `redraw_water`; `KeyframeStore::replace` in ring, file and index).
  - `cargo clippy --workspace --all-targets -- -D warnings`: clean on the code before the ice-hiding and shallow-lake edits (the Mac stopped before a re-run). `cargo fmt`: `rustfmt --check --edition 2024` on every touched file is clean (run locally, one-off). `cargo deny check licenses bans`: ok.
  - `./gradlew build test`: 157 tests, 0 failures. No GameTests (rule).
- **Gates on the VM, 2026-10-01** (after merging master): `cargo clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean, `cargo test -p deeptime-core --lib -- water_view keyframes` 18/0. The full-workspace tests and `./gradlew build test` (157/0) last ran on the M6 before the ice-hiding and shallow-lake edits and were not re-run.
- **Next.** Merge-time: `tools/remote/mac-gc.sh` (this worktree's VM copy `/Volumes/Graphide/ban/mic/mods/agent-a3d4fadfe3e053b77`). Conflicts to expect with the Deposits-layer agent: `keyframes.rs` (`DERIVED_LAYERS`, `add_derived`, `derived_field`), `workbench.js` (`LAYER_WORDS`, `legendHTML`, `valueText`), this file's header line.
### 2026-10-01 — First gates on the x86_64 test VM: five failures traced; real-planet tests assert properties, not one planet's numbers (agent worktree agent-ac2a33b7f51fa4bf8, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Symptom.** Master bd6250e/5c45b04 on the new VM (x86_64 Linux): `cargo test --workspace` failed one CLI test
  (`deposit_stage_reads_the_recorded_history`, main.rs:1676) and `./gametest.sh`, both profiles, failed
  `climateConsistent`, `realPlanetHeightMatchesTheBake`, `riversHold` and `carversSpareWater`. The Mac (aarch64 macOS)
  had passed all of it on the same simulation code.
- **The planet differs by platform, not by thread count or run.** `deeptime run --preset planet_world_4k.json` seven
  times on the VM (`--threads` 1, 2, 3, 6, 6, 12, and master's own GameTest bake at 6): the CRC32 of all 42 layers is
  identical in every run (land 0.187, 56 lakes, 286 river chains). Against the Mac there is no bake to compare (the M6
  is off limits), only its log on the same code: `carversSpareWater columns=695 highColumns=106 carvedBelow=1589` there,
  `columns=915 highColumns=271 carvedBelow=1072` here, from the same box rule. So one preset is one planet per
  platform. **Where it comes from:** the simulation calls the platform's libm about 280 times in non-test code (`exp`
  63, `ln` 45, `sin`/`cos`/`sin_cos` 98, `powf` 29, `hypot` 17, `atan2` 7, …), and those functions are not correctly
  rounded: Apple's libm, glibc, musl and the Windows CRT differ in the last bit, and 75 coupled plate steps with
  thresholds (collisions, rifts, subduction starts) turn a last bit into another history. Rust never fuses a
  multiply-add on its own, and the thread count changes nothing (measured above). The Linux natives link the
  system's libm, so two Linux distributions can differ too. Ben
  (2026-10-01, relayed by the coordinator): identical runs across machines are not a goal; speed is. So tests must
  hold on any planet. Stored data stays exact: a baked world plays the same everywhere, and the Rust/Java parity tests
  on baked data are unchanged.
- **A way to test on other planets.** `DEEPTIME_TEST_PLANET_SET="seed=12" ./gametest.sh [-PwithMic]` bakes the
  real-planet level from the same preset with `--set` overrides. Every real-planet test below was run on seeds 11–16
  standalone and 11–14 with MIC.
- **`realPlanetHeightMatchesTheBake` (a real defect, reported, not fixed).** Column (−892, 324) of the x86 planet has
  H = 726.8 and the world ends at y 639. **Nothing bounds the terrain to the build range.** The plate model caps its
  own elevation at `tectonics.max_elevation_m` (10 km = y 563), but the surface stage's elevation is the plate
  elevation plus an offset that carries deposition and isostatic rebound, and when the plates refuse a rise at their
  cap the remainder stays in the offset (`plates::apply_surface_feedback` clamps, `surface` subtracts only what was
  applied). On this planet 237 L1 cells stand above 10 km at the end (offset up to +10.5 km over a plate at 9.6 km,
  under 15–38 km of logged sediment and the 3 km ice cap), 16 km already after 10 steps; the young stage and the
  amplifier pass it on, the bake writes it, and the game cuts it flat at y 639 (the column evaluator's `top_y` is
  clamped to `max_y`, so strata and parity stay consistent). Baked: 8,977 L3 cells above y 639, 2.5 % of the land,
  highest y 969. Of the six seeds two do it (seed 13: 1.0 % of land, y 801); of the four distinct 500-Myr 16k
  runs in other agents' logs on the VM, two ended with surface peaks of 12.4 and 13.0 km (the others 9.1 and
  9.2 km) against a build limit of 11.5 km above the sea, before the amplifier adds its own relief. The
  bake now counts it (`notes.bake.terrain.build_limit`: cells above, share of land, highest y; `above_max_y=` on the
  `bake` line) and the GameTest compares with the clamped top and logs `aboveBuildLimit`. What to do about it is a
  world-shaping choice: pending question below.
  The test's other bound changed too: "non-corner columns within 12 blocks of their own H" is a planet's number (1
  to 10 on the seeds: it grows with the slope, the corners do not see the raster inside a cell); the property is that
  a column between corners tops out between its cell's four corner tops ± 1 (`outsideCellCorners=0` on all ten runs).
- **`riversHold` (the test, not the water).** The six columns are lake columns one or two blocks deep by the
  column rule (bilinear surface 61.7–63.0, lake top 63) whose generated ground is y 63: grass at 63, dirt under it,
  nothing drained.
  The lake pass fills the air down to the generated ground (FORMAT.md says so), and between noise-cell corners that
  ground can stand a block or two above the rule's bilinear `ground`; where it reaches the lake's top the column is
  dry shore, flush with the water beside it. Banks around it are raised as if it held water, so nothing leaks
  (`leaks=0`, `outside=0` before and after 300 ticks). The test now counts such columns apart (`dryShore`, at most 1 in
  20 water columns; measured 6/915, 1/429 and 0 on four planets) and still fails a lake column whose top block is
  open without water and any river column below its top. FORMAT.md's "Water columns" names the case.
- **`carversSpareWater` (the test, not the carvers).** The same columns: the carver cut the shore's ground block at
  y 63, and the test called air in a lake column's water range "drained". It now compares with the uncarved twin
  level (`test_rivers`, same world data): a block that is water without carvers and air with them is drained
  (0 on every run, over 929 to 9,458 water blocks); carved ground in that range is counted apart (`carvedShore`,
  0 to 5). Its premise (water above sea level with cuts under it) failed on seed 14, whose lakes and mouths all sit at
  the coast (`highColumns=28 carvedBelowHigh=0`): the test adds up to six boxes on rivers between sea level + 4 and
  y 170 (`highSites`), and then holds everywhere (567 to 2,605 high columns).
- **`climateConsistent` (floors read from one planet).** A 0.553 < 0.6 and Dsub 0.812 < 0.85 with no change to the
  mapping. Ten censuses (shares, lowest–highest): A 0.553–0.944, Af 0.642–0.963, BW 0.675–0.925, BS 0.623–0.744,
  C 0.849–0.968, D 0.984–1.0, Dsub 0.659–0.882, E 0.922–1.0, BW:desert 0.475–0.827, Af:jungle 0.515–0.774,
  Dfc:taiga 0.563–0.822, E:snow 0.849–0.998. Floors are now 0.1 under the lowest share on any planet, the Mac's
  earlier ones included: A 0.45, Af 0.54, BW 0.57, D 0.88, Dsub 0.55, E 0.82, BW:desert 0.37, Af:jungle 0.41; BS 0.42,
  C 0.75, sea 0.95, Dfc:taiga 0.33, E:snow 0.72 unchanged. `MIN_COLUMNS` 20 → 40. One thing the census shows that
  is the mapping's, not the test's: on seed 11 standalone the Aw and As columns are 63–71 % `minecraft:forest`
  (savanna climates drawn as forest); with MIC the same planet gives A 0.761. For P1-09's owner.
- **The CLI test: not reproduced, cause not established.** `cargo test -p deeptime-cli` 12 more times on the VM (load
  30–65; three of them pinned to two cores with 32 test threads): 0 failures. The failed run left its folder
  behind; replaying its last step (`deeptime stage deposits --in group.dtcp`) from that very checkpoint succeeds, so
  the checkpoint was good and the failure was in the process at that moment (load ~100, minutes after the VM
  booted). Ruled out by measurement: the memory cap (the test process's counted allocations peak at 1.06 GiB of the
  4 GiB cap), a spilled history (resuming from a checkpoint whose frames are on disk works), two calls sharing the
  temporary work folder (only this call uses it in the process; the folder is per call now anyway). If it recurs,
  the panic's `Err(…)` text is what is needed.
- **Also fixed.** `gametest.sh` reported `Peak server-JVM RSS: 0 MB` on the VM: `$PWD` was the symlinked path
  while Gradle's command line has the physical one, and inside the dev shell the system `ps` fails on the shell's
  `LD_LIBRARY_PATH`. Now 6.3–7.1 GB.
- **Measured** (VM `deeptime-test`, x86_64 Linux, through `tools/remote/mac.sh` with its default caps,
  `CARGO_BUILD_JOBS=8` and `RAYON_NUM_THREADS=6`; load average 27–35 during cargo and Gradle, 15–17 during the
  GameTests): `cargo fmt --all -- --check` clean; `cargo clippy --workspace --all-targets --locked -- -D warnings`
  clean; `cargo test --workspace --locked` all pass (469: cli 31, core 335 + 3 ignored); `cargo deny check licenses
  bans` ok; `./gradlew build test` tests=160 failures=0 errors=0 skipped=1; `./gametest.sh` All 57 required tests
  passed (wall 541 s, server JVM peak 7.1 GB); `tools/smoke/install.sh mic` (168 mods), `./gametest.sh -PwithMic` All
  57 required tests passed (wall 91 s, 9.3 GB). The nine real-planet tests alone on seeds 12–16 standalone with the
  final code: All 9 passed, five times. The MIC runs of seeds 11–14 were made before the new floors and the
  corner-bracket check (their river tests are the final ones and passed; their censuses are in the ranges above).
  **Verified on:** x86_64 Linux only; bakes at 1, 2, 3, 6 and 12 threads, everything else at 6. Not run on aarch64
  macOS (the M6 is off limits): the Mac's planet is one more planet, like another seed.
- **Scratch.** `/Volumes/Graphide/ban/deeptime-runs/x86-diag/` on the VM, deleted.
- **Next.** Ben's choice on terrain above the build limit (pending question); then the fix and a GameTest that
  fails on any column above `max_y` (`aboveBuildLimit` must be 0 on every seed).

### 2026-10-01 — M6 paused; batch 4 pushed with one GameTest re-run pending (session f981c0d3-5144-49ec-8686-2946d755872c)
- **Pause.** Ben: our test load on the M6 made graphide's ms-precision timing tests flaky; "please cease
  testing on the m6, I will be provisioning a new machine for your tests". All Deep Time jobs there were
  killed (six P1-17 calibration runs, the climate agent's run, two workbench timing runs, our Gradle
  daemon); the idle workbench server (port 8642) stays up. `mac.sh` now refuses while
  `/Volumes/Graphide/ban/deeptime-cache/PAUSED` exists on the M6 (4051cbb); CLAUDE.md says so.
- **Batch 4** (amplify memory fix 80cd51b, water protection ce8a4fa, test fix 05a3073): cargo (fmt, clippy,
  test, deny) and `./gradlew build test` (160/0) green on ce8a4fa; GameTests 55/56 on both profiles, the
  one failure the new `carversSpareWater` (its premise counted only air under the beds, but the test
  level has no aquifer, so carved cells below sea level fill with water). The fix (8b48bce) passed
  `carversSpareWater` alone (`-PgametestOnly`: columns=695 drained=0 holes=0 carvedBelow=1589
  entrances=246); the full `./gametest.sh` and `-PwithMic` re-run waits for the new test machine.
- **Next.** Wire the new machine into `mac.sh` (`DEEPTIME_MAC`, SSD path), re-run the batch-4 GameTests,
  then resume the paused agents' work (P1-17 hypsometry, the climate ice branch, the four workbench
  features).
### 2026-10-01 — P1-17 hypsometry recalibration 2026-10-01 (agent worktree agent-aee888e01d29a40db, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Why.** Decisions log 2026-10-01 ("Now, after this batch"): master had drifted from the 2026-09-29 calibration.
  All numbers below are from the Azure test VM (x86; planets differ from the M6's by libm), `earthlike_quick_16k`,
  500 Myr, `deeptime stage volcanoes`, `tools/review/p1-17-hyps.sh` (4 runs of 2 threads at once, niced), true-area
  weighted. z50/z90 = land median/p90, >2 = land above 2 km, sea = median sea depth, T = mean surface temperature,
  arc = ground under continental arc cones (mean of the 10-Myr medians).
- **Earth reference.** GMT earth_relief 30′ (SRTM15+, Tozer et al. 2019), cos φ-weighted like the CLI's line
  (`p1-17-hyps-plot.py` recomputes it): land 0.289, median 427 m, p90 2,022 m, above 2 km 0.102, sea median 4,062 m,
  sea mean 3,683 m (Charette & Smith 2010, Oceanography 23(2):112–114, on SRTM30_PLUS: 3,682 m, ocean 361.84 M km² =
  70.9 %), 1-km bands from below −6 to above +5 km `0.003/0.137/0.229/0.170/0.060/0.036/0.076/0.216/0.044/0.016/0.009/0.004/0.001`.
- **Which merge moved it** (old commits built from `git archive`, seeds 1–2 × both starts):

  | build | land | z50 | z90 | >2 | sea | T | ice | E |
  |---|---|---|---|---|---|---|---|---|
  | 6070e9e (calibration + carbon cycle + runoff fix) | 0.283 | 625 | 2,172 | 0.118 | 3,312 | 13.8 | 0.090 | 0.26 |
  | bd1cc0f (+ volcanoes, glacial landforms) | 0.383 | 1,385 | 4,281 | 0.343 | 3,130 | 10.3 | 0.220 | 0.42 |
  | master (+ rivers, P1-27) | 0.381 | 1,211 | 3,933 | 0.307 | 3,076 | 8.9 | 0.226 | 0.39 |

  The drift came with the volcano and glacial merges; the deep-time climate did not cause it and P1-27 lowered it a
  little. (a3c63fe's CLI rejected the run's arguments; 6070e9e carries the same calibration.)
- **Drivers on master** (one change each; 4 runs, seeds 1–2 × both starts; baseline of the same 4 runs first):

  | variant | land | z50 | z90 | >2 | sea | crust | T | ice | arc |
  |---|---|---|---|---|---|---|---|---|---|
  | master | 0.381 | 1,211 | 3,933 | 0.307 | 3,076 | 0.385 | 8.9 | 0.226 | 753 |
  | `volcanoes.density` 0 | 0.316 | 766 | 2,781 | 0.170 | 3,045 | 0.387 | 11.7 | 0.160 | – |
  | `surface.k_spl` 4e-9 | 0.355 | 803 | 2,628 | 0.162 | 3,218 | 0.385 | 10.7 | 0.128 | 597 |
  | `tectonics.subduction_uplift_mm_yr` 0.3 | 0.354 | 835 | 3,003 | 0.189 | 3,276 | 0.393 | 10.5 | 0.174 | 731 |
  | `tectonics.collision_uplift_max_m` 2,000 | 0.324 | 915 | 2,750 | 0.198 | 3,001 | 0.382 | 11.5 | 0.142 | 710 |
  | `tectonics.cordillera_uplift_mm_yr` 0 | 0.359 | 1,130 | 3,916 | 0.300 | 3,068 | 0.387 | 10.0 | 0.260 | 388 |
  | `tectonics.continent_elevation_m` [50, 600] | 0.421 | 1,154 | 3,858 | 0.302 | 2,788 | 0.414 | 8.6 | 0.212 | 844 |
  | `climate.glacio_eustasy` 0 | 0.394 | 1,173 | 3,982 | 0.319 | 2,771 | 0.409 | 7.7 | 0.263 | 654 |
  | `surface.sea_level.base_m` 50 | 0.382 | 1,176 | 4,069 | 0.306 | 3,047 | 0.407 | 8.7 | 0.248 | 712 |
  | `surface.marine_fill_depth_m` 40 | 0.406 | 1,367 | 4,695 | 0.370 | 2,657 | 0.405 | 7.5 | 0.299 | 778 |
  | GDH1 floor −5,600 m at 0.215 mm/yr | 0.204 | 848 | 3,182 | 0.213 | 4,785 | 0.291 | 14.4 | 0.140 | 750 |

  On the M6 before the pause (6 runs, its own baseline land 0.450, z50 1,323): no glacial model (no K = 0 under ice)
  z50 1,046, >2 0.254; fixed CO₂ z50 1,383, T −0.3 with one frozen planet (the carbon cycle is the thermostat, not
  the cause). Reading: the cones (2·10⁷ km³ built per run, a fifth of everything eroded) and the ice's protection of
  high ground raised the land; high land is cold, ice-covered and, through the ice's sea-level fall (−8 … −400 m),
  larger. Erosion efficiency, the collision cap and the Andean uplift are the levers that bring it down; the start
  elevation, the sea-level datum, glacio-eustasy and the shelf fill depth are not (within the noise of 4 chaotic runs,
  about ±0.03 land and ±150 m). The cordillera costs little height overall. The ocean law is the lever for sea depth,
  but at GDH1's full rate island arcs stop emerging and the continental crust shrinks.
- **Calibration rounds** (4 runs each; K = `k_spl`, cap = collision cap, G = floor −5,600 m at 0.12 mm/yr):

  | variant | land | z50 | z90 | >2 | sea | crust | T | ice | arc |
  |---|---|---|---|---|---|---|---|---|---|
  | K 4e-9, cap 2,000 | 0.343 | 720 | 2,218 | 0.131 | 2,921 | 0.414 | 11.0 | 0.088 | 548 |
  | K 3e-9, cap 2,000 | 0.318 | 707 | 2,511 | 0.165 | 2,975 | 0.373 | 11.7 | 0.133 | 546 |
  | K 4e-9, cap 2,500 | 0.354 | 749 | 2,522 | 0.155 | 2,901 | 0.389 | 11.6 | 0.131 | 534 |
  | K 6e-9, cap 2,500 | 0.326 | 710 | 2,686 | 0.170 | 2,820 | 0.381 | 11.2 | 0.137 | 500 |
  | K 3e-9, cap 1,500 | 0.311 | 684 | 2,134 | 0.114 | 3,254 | 0.379 | 11.7 | 0.092 | 525 |
  | G alone | 0.239 | 805 | 2,979 | 0.194 | 4,179 | 0.312 | 12.1 | 0.170 | 766 |
  | K 4e-9, cap 2,000, G | 0.219 | 543 | 2,080 | 0.109 | 3,959 | 0.322 | 14.3 | 0.092 | 592 |
  | S = K 3e-9, cap 2,000, G; `arc_continental_m` 50 | 0.213 | 557 | 2,026 | 0.103 | 4,040 | 0.325 | 13.0 | 0.085 | 546 |
  | S, 50, cordillera 0.3 to 3,000 m | 0.242 | 656 | 2,530 | 0.155 | 3,950 | 0.329 | 13.3 | 0.128 | 849 |
  | S, 50, cordillera 0.4 to 3,000 m | 0.242 | 696 | 2,471 | 0.146 | 4,142 | 0.316 | 13.3 | 0.133 | 951 |
  | S, `arc_continental_m` 30 | 0.203 | 482 | 1,920 | 0.093 | 3,934 | 0.325 | 12.8 | 0.083 | 612 |
  | S, 50, `volcanoes.density` 0.5 | 0.233 | 582 | 1,710 | 0.070 | 4,019 | 0.332 | 13.1 | 0.068 | 506 |
  | S, cordillera 0.3, `arc_uplift_mm_yr` 0.45 | 0.246 | 604 | 2,440 | 0.142 | 3,646 | 0.356 | 12.9 | 0.122 | 584 |
  | **S, cordillera 0.3, arc 0.6 (chosen)** | 0.270 | 585 | 2,072 | 0.110 | 3,488 | 0.400 | 10.9 | 0.127 | 749 |
  | S, cordillera 0.4, arc 0.6 | 0.290 | 758 | 2,670 | 0.170 | 3,308 | 0.422 | 11.0 | 0.184 | 825 |
  | S, arc 0.6 (cordillera 0.2) | 0.264 | 585 | 2,219 | 0.123 | 3,408 | 0.416 | 11.8 | 0.184 | 497 |

  With the deeper young floor the continental crust falls to 0.32 whatever `arc_continental_m` is (arcs do not
  emerge); raising the island-arc build-up restores it (0.356 at 0.45, 0.40 at 0.6) at the cost of some sea depth.
- **Changes** (the four `presets/pipeline/*.json`, built-ins re-synced; nothing else in the presets; for the merge with
  the ice branch: six lines in `tectonics`, one new, and one in `surface`):

  | key | before → after | why |
  |---|---|---|
  | `surface.k_spl` | 2e-9 → 3e-9 | compensates the cones' added rock and the ice's protection; editor hint updated |
  | `tectonics.collision_uplift_max_m` | 4,000 → 2,000 | a surge lifts ⅓ of its peak over up to 2,000 km, 30+ times per run, and isostasy 0.8 makes each km cost 5 km of erosion |
  | `tectonics.oceanic_floor_elevation_m` (new) | −10,000 (Cortial) → −5,600 | GDH1's old-floor depth (Stein & Stein 1992); floor below −6 km 0.061 → 0.020 of the planet (Earth 0.003) |
  | `tectonics.oceanic_damping_mm_yr` | 0.05 → 0.12 | 3.6 km at 20 Myr, 4.5 at 50, 5.2 at 100; GDH1's own rate (0.215) drowned the arcs |
  | `tectonics.arc_uplift_mm_yr` | 0.3 → 0.6 | island arcs build from a floor 1 km deeper; keeps the continental crust at 0.40–0.42 |
  | `tectonics.cordillera_uplift_mm_yr` | 0.2 → 0.3 | Ben's revisit: ground under the cones 497 → 749 m at the same land share |
  | `tectonics.cordillera_height_m` | 2,500 → 3,000 | with it |

  Not changed: isostasy (Airy), `glacial.strength` (Ben: physical), volcano density (Ben's call, below), the climate,
  `planet_world_4k.json` (the GameTest planet keeps its values; coordinator's call), `SurfaceParams::default` (K 2e-9).
- **Code.** `TectonicParams::oceanic_floor_elevation_m` (default = Cortial's trench, plate-only presets and old
  checkpoints unchanged: a 60-Myr run of master and of this build with the key unset print the same hypsometry line);
  `pools::{surface_step, ocean_elevation}` subside towards it; validation `[trench, ridge)`; tests
  `plates::pools::tests::{dampening_matches_its_closed_form, the_gdh1_floor_follows_stein_and_stein}`. Editor field
  `abyssal_depth` (adv_plates, −8,000…−4,600 m, lang key); `tools/dev/knob_extremes.py --only abyssal_depth --length-myr
  100`: both ends ok (sea median 4,367 / 3,723 m). `deeptime stage` prints `hypsometry_curve`. Tools:
  `tools/review/p1-17-hyps.{sh,py}`, `p1-17-hyps-plot.py`.
- **Before → after, per run** (master efafc88's presets → this branch; seeds 1–5):

  | start, seed | land | z50 | z90 | >2 | sea | T | ice | E | B | active | arc |
  |---|---|---|---|---|---|---|---|---|---|---|---|
  | super 1 | 0.459 → 0.300 | 1,059 → 711 | 3,845 → 2,499 | 0.267 → 0.158 | 3,059 → 3,071 | 6.8 → 11.2 | 0.223 → 0.227 | 0.38 → 0.41 | 0.20 → 0.18 | 0.29 → 0.31 | 806 → 801 |
  | super 2 | 0.383 → 0.249 | 1,169 → 473 | 4,065 → 1,737 | 0.312 → 0.073 | 3,004 → 3,578 | 9.4 → 11.1 | 0.300 → 0.086 | 0.47 → 0.25 | 0.19 → 0.24 | 0.31 → 0.33 | 774 → 709 |
  | super 3 | 0.425 → 0.232 | 1,149 → 338 | 3,538 → 1,791 | 0.287 → 0.082 | 2,870 → 3,412 | 9.5 → 10.9 | 0.183 → 0.170 | 0.38 → 0.34 | 0.18 → 0.15 | 0.35 → 0.29 | 872 → 618 |
  | super 4 | 0.376 → 0.292 | 1,177 → 736 | 3,443 → 2,490 | 0.283 → 0.154 | 3,302 → 3,401 | 9.2 → 14.0 | 0.109 → 0.154 | 0.31 → 0.31 | 0.24 → 0.23 | 0.33 → 0.28 | 775 → 550 |
  | super 5 | 0.421 → 0.333 | 1,124 → 722 | 3,400 → 2,866 | 0.254 → 0.181 | 3,070 → 3,306 | 9.0 → 10.2 | 0.177 → 0.196 | 0.34 → 0.38 | 0.22 → 0.16 | 0.34 → 0.33 | 747 → 777 |
  | scattered 1 | 0.331 → 0.297 | 1,364 → 661 | 3,792 → 2,224 | 0.345 → 0.123 | 3,057 → 3,754 | 9.0 → 8.9 | 0.213 → 0.090 | 0.36 → 0.22 | 0.25 → 0.27 | 0.30 → 0.35 | 671 → 972 |
  | scattered 2 | 0.351 → 0.233 | 1,251 → 494 | 4,030 → 1,829 | 0.306 → 0.085 | 3,184 → 3,548 | 10.6 → 12.5 | 0.168 → 0.104 | 0.35 → 0.24 | 0.22 → 0.28 | 0.34 → 0.31 | 762 → 514 |
  | scattered 3 | 0.384 → 0.351 | 1,190 → 775 | 3,556 → 2,433 | 0.297 → 0.151 | 3,100 → 3,371 | 10.5 → 10.1 | 0.159 → 0.212 | 0.34 → 0.37 | 0.24 → 0.21 | 0.36 → 0.33 | 779 → 730 |
  | scattered 4 | 0.659 → 0.350 | 2,505 → 845 | 5,441 → 2,563 | 0.616 → 0.165 | 1,676 → 3,224 | −28.1 → 10.4 | 0.908 → 0.179 | 1.00 → 0.32 | 0.00 → 0.23 | 0.28 → 0.36 | 1,006 → 745 |
  | scattered 5 | 0.354 → 0.289 | 1,249 → 645 | 3,495 → 1,978 | 0.318 → 0.097 | 2,777 → 3,275 | 11.3 → 10.4 | 0.165 → 0.132 | 0.34 → 0.31 | 0.25 → 0.15 | 0.33 → 0.36 | 744 → 616 |
  | **mean** | 0.414 → **0.293** | 1,324 → **640** | 3,860 → **2,241** | 0.329 → **0.127** | 2,910 → **3,394** | 5.7 → **11.0** | 0.261 → **0.155** | 0.43 → **0.32** | 0.20 → **0.21** | 0.33 → 0.33 | 794 → 703 |
  | Earth | 0.289 | 427 | 2,022 | 0.102 | 4,062 | 14–15 | 0.10 | 0.13 (model on Earth 0.23) | 0.30 (0.26) | ≈ 0.3 | 1,200–4,000 |

  Continental crust 0.401 → 0.419 (dry share 0.86 → 0.67; Earth 0.41, 0.71). Mean 1-km bands after:
  `0.020/0.117/0.139/0.130/0.093/0.053/0.155/0.191/0.063/0.023/0.009/0.004/0.002` (before
  `0.061/0.044/0.072/0.114/0.117/0.057/0.121/0.161/0.110/0.063/0.036/0.021/0.023`). Master's scattered seed 4 froze
  (snowball to the present); none of the calibrated runs did.
- **Other checks** (one run each unless stated):
  - 1,000 Myr: supercontinent seed 1 land 0.249, z50 430, z90 1,967, >2 0.097, T 11.5, ice 0.103 (master: land 0.375,
    z50 2,175, >2 0.521, T 4.9, ice 0.493); scattered seed 2 land 0.270, z50 585, >2 0.103, T 12.7, ice 0.096. No
    upward drift after 500 Myr.
  - `earthlike_quick_32k` (seed 1, both starts): land 0.358 → 0.323, z50 1,181 → 696, >2 0.306 → 0.155, sea 3,330 →
    3,847, T 11.2 → 9.2, ice 0.234 → 0.197.
  - `earthlike_full_16k` (scattered seed 1): land 0.604 → 0.378, z50 2,043 → 1,013, z90 6,032 → 2,896, >2 0.507 →
    0.218, sea 1,662 → 2,858, T 0.1 → 8.9, ice 0.418 → 0.180. **The full preset stays higher than the quick one** (one
    run; its finer grid and 250,000 samples behave differently); it needs its own ensemble.
  - Run time (same seed, side by side, 2 threads, `/usr/bin/time -v`): user CPU 716 → 692 s (−3 %), wall 450 → 424 s,
    peak RSS 2.40 → 2.38 GB; stage seconds plates 102 → 98, climate 69 → 68, surface 153 → 139, volcanoes 26 → 28,
    structure 99 → 91.
- **Land above 10 km** (the build-limit fix on worktree-agent-ac2a33b7f51fa4bf8): no calibrated run has any (maxima
  6.5–9.3 km over the ten runs), so none of the after figures depends on it. Master's runs reach 10.7–13.0 km on five
  of ten planets, in the top 1 % of the surface: the before maxima depend on it, the before medians, p90 and >2 km
  shares do not measurably.
- **Still off.** (1) The climate is on the cold side: T 11.0 °C, ice 0.155, E 0.32 against 14–15, 0.10 and the
  model's own 0.23 on Earth's relief; not tuned (ice branch, P1-26). (2) Too much shallow sea: 0.155 of the planet
  between −1 and 0 km (Earth 0.076), so the sea median is 3.4 km, not 4.06. (3) Ground under arc cones is ~0.7 km, not
  an Andean 2–4 km: raising the rate to 0.4 gave 825 m and cost 0.06 above 2 km and 0.06 ice. (4) The full preset.
- **Evidence** (pulled, not committed): review/out/p1-17-hyps/index.html (mean and per-run hypsographic curves against
  Earth's, land-only curve, numbers; `_shot.png` is the page as rendered; master's seeds 4–5 have no curve, its
  build lacks the line), the `summary-*.json` of every variant and the stage logs beside it.
- **Gates** (VM, tree d76c491 = this branch merged with master efafc88): `cargo fmt --all -- --check`, `cargo clippy
  --workspace --all-targets -- -D warnings`, `cargo deny check licenses bans` clean; `cargo test --workspace
  --no-fail-fast` core 352 + 3 ignored, cli 52, all pass; `./gradlew build test` tests=160 failures=0. No GameTests
  (coordinator); the built-in planets change, so property floors measured on them may move.
- **Decisions for Ben** (not asked): cordillera strength; volcano volume; the full presets; see the final report.
- **Next.** Re-check after the build-limit fix, the ice branch and P1-26; an ensemble on `earthlike_full_16k`.

### 2026-10-01 — Zoomable final world: a deep-zoom map of a run's bake (WIP, stopped at Ben's request; agent worktree agent-ad5fc9991593ce7ef, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Status: built and working, not finished.** Ben stopped all Mac testing mid-task (the load broke graphide's timing tests), so the gates, the "after" run time and the final screenshot set were not done. Not merged, not pushed.
- **What it is.** The workbench's new "Final world" view (stage bar button, key `w`; `?run=<id>&world=<layer>/<x>/<z>/<blocks per px>` opens it at a place): a canvas slippy map with no library, pan/zoom by mouse and touch (drag, wheel, double click, pinch, arrows, +/-/0), the world wrapping east-west, readout of block (x, z), lat/lon and, after a pause, the bake's terrain y, surface, Koeppen class, T and P (`api/runs/<id>/world/sample`); layers Satellite, Elevation (hill shade), Water, an optional Rivers and lakes overlay, a graticule with the seam, a scale bar, an About card, and "3D preview here", which hands the middle 256 x 256 blocks of the view (the view's own box if smaller) to the existing Area 3D preview (`DTWB.preview` -> `Stage2.preview`; a dashed square shows the box). Server: `GET /api/runs/<id>/world.json`, `/world/sample?x=&z=`, `/tiles/<layer>/<z>/<x>/<y>.png` (XYZ, geodetic 2:1: zoom 0 is two 256-px tiles, `s = C / (512 * 2^z)` blocks a pixel, last native zoom 5 at C = 16,384 and 6 at 32,768).
- **Code (new modules, small edits elsewhere).** `crates/deeptime-cli/src/watch/worldmap.rs` (the bake as a map: tile geometry, L3 windows, mips, climate fields, rivers, the four painters, `sample`), `watch/tiles.rs` (routes, per-run disk cache with a least-recently-asked bound, at most two maps in memory), `crates/deeptime-viewer/assets/worldmap.js` (the view; injects its own CSS), `tools/workbench/tiles_bench.py`. Edits: `watch/http.rs` (one route hook and `/worldmap.js`), `watch/mod.rs` (two `mod` lines), `workbench.html` (button, container, script tag), `workbench.js` (one `window.DTWB` export at the end), `deeptime-viewer/src/lib.rs` (`WORLDMAP_JS`, `encode_png_rgb`), `deeptime-core/src/satellite.rs` (the lighting split out as `Light` / `shaded`, `box_filter`, `rock_tint`, `depth_colour` made `pub`; `hillshade` is the same arithmetic, held by a test against the formula written out). Nothing vendored: no NOTICE.md change. Nothing in the simulation or the bake changed.
- **The true finest resolution.** The bake's finest data is the terrain height `H = E + A*N` evaluated at every block: `E` the L3 `elevation` (2 blocks a cell at 16k, bilinear), `A` the L3 `detail/amplitude` and `N` the game's sub-cell noise (32/16/8-block octaves, used up to 4 blocks a pixel). That *is* block-level amplified terrain and it costs about nothing (a 256 x 256 tile in 3-50 ms); the game then draws it between corners four blocks apart. Colour comes from L1 (16 blocks) climate and L2 (8 blocks) rock and ice grids, interpolated and lapsed to each block's height (monthly sea-level temperatures minus the lapse, the climate API's rule), so they are smooth, not block detail. The view magnifies up to 4 screen px a block past zoom 5.
- **Colours match the frame maps.** Same `satellite::colour`, `cover`, `box_filter`, light and ramp, fed from the bake: P/PET and the equilibrium line rebuilt with the climate stage's own `thornthwaite`, `glaciers::summer` and `ela_temperature` from the baked monthly temperatures; ice from `climate/ice_cover` (blurred, then kept to ground above the equilibrium line so glaciers follow relief), rock from `surface/class_top`. Zoom 1 (16 blocks a pixel = the frame's own pixel) against the last keyframe's satellite layer: RMSE 0.073 of full scale including lakes and rivers the frame does not draw; the difference image is black over land. Sea ice is not in the bake; the proxy (annual mean sea-level temperature, none above -1 C, whole below -7 C) puts the ice edge within a few degrees of the frame's.
- **Measured (Mac, load 55-80, private workbench on 127.0.0.1:8671, run = a 500-Myr `earthlike_quick_16k` bake, `tiles_bench.py`-style, 12 random tiles a zoom).** Cold tile (render + PNG + cache write, over HTTP): satellite 11-26 ms mean (max 48), elevation 3-20 (max 36), water 4-11 (max 17), overlay 3-10; warm 0.3-1.4 ms. First open of a run's map (mips, climate fields): 0.67 s. Tile PNGs 3-110 KB. Full pyramid at 16k (6 zooms, 2,730 tiles a layer), from the mean tile sizes: satellite ~98 MiB, elevation ~204, water ~204, overlay ~9 (about 515 MiB for all four; the hill-shaded layers compress worst, and the zoom 5 level is 85 % of it); 32k adds a zoom (about 4x). The per-run cache is capped at 384 MiB (`DEEPTIME_TILE_CACHE_MIB`), least recently asked dropped, and goes with the run's folder.
- **Run time, before.** `deeptime run --preset earthlike_quick_16k --start scattered_continents --length-myr 500 --threads default` (release, master 8dae3f2, Mac load 29 -> 65): **1066.4 s** real, 1364 s user, peak 4.37 GiB (`/usr/bin/time -l`). The "after" run was not made (it is the same command with `deeptime-after`; nothing in the simulation changed, so it is expected to match within the Mac's load noise).
- **Finished on the Azure test box (2026-10-01).** Branch = master 5c45b04 + this work.
  - **Gates, all green:** `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings` (after three fixes: two `chunks_exact(4)` -> `as_chunks::<4>()`, one `(20 - 0)`); `cargo test --workspace` (core 336 + 3 ignored, cli 39 including the 6 new `watch::worldmap` / `watch::tiles` tests and the hill-shade formula test, format 51, all ok; the three tests that failed on the M6 pass); `cargo deny check licenses bans` ok; `./gradlew build test` BUILD SUCCESSFUL. No GameTests (branch).
  - **Run time, before/after** (`RAYON_NUM_THREADS=6 /usr/bin/time -v deeptime run --preset earthlike_quick_16k --start scattered_continents --length-myr 500`, one after the other on a box at load 40-70, so wall clock is noise): before = master 5c45b04 built from `git archive`, after = this branch. **User CPU 2231 s vs 2266 s (+1.6 %, inside the noise of a shared box), system 154 s vs 197 s**; wall 11:12 vs 8:21 (the box was busier during the first); peak RSS 10.4 vs 11.7 GB; per-stage seconds of the same order (the after run is lower in every stage because the box was quieter, e.g. surface 197 s vs 161 s, amplify 65 s vs 38 s). **The bake is byte-identical**: `rasters/elevation.dtr` and `rasters/water_level.dtr` compare equal between the two runs (as expected: `satellite.rs` only had its lighting split into `Light`, and the keyframes it draws are held by a test). The tiles add nothing to a run.
  - **Tiles, measured** (`tools/workbench/tiles_bench.py 8671 r16 16` against a private workbench on the after bake, box load 35-47; 16 random tiles a layer and zoom, cold = render + PNG + cache write over loopback HTTP): satellite 17-46 ms mean, max 51 (zoom 3 is the slowest: 4 blocks a pixel is a 2 x 2 box over the L3 cells plus the detail noise); elevation 8-34, water 7-34, overlay 5-31, max 37; warm 0.3-0.8 ms. First open of the map 0.9 s. Mean tile 3-98 KB. **Full pyramid at 16k** (2,730 tiles a layer): satellite 132 MiB, elevation 139, water 138, overlay 9, 418 MiB for all four, 75 % of it zoom 5. Cache cap 384 MiB a run.
  - **Shots** (review/out/wb-zoom/ui/, not committed; Chromium 153 from nix on the box, DevTools-driven): 01-12 are the whole world (satellite and elevation), the biggest river mouth of the bake (x -1218, z 1640; 8, 2 and 0.5 blocks a pixel, and the water layer), the fjord (x 6016, z 3032; satellite at 3 and 1, elevation, water), the seam wrap, and the 5901/2892 cone area. **Verified by looking:** the river mouth (a channel that widens as it meets the sea, with the shelf colour round it) and the fjord at (6016, 3032) (a narrow arm between ridges with the river entering at its head and truncated spurs, ice on the high ground). **Not verified: the volcanoes.** The run's events list 20+ live stratovolcanoes, arcs and two hotspot shields (`find_places.py volcanoes` / the events), and a transect of the terrain (peak against the mean of a ring 120 blocks out) puts a cone at the listed block for several of them (x 5901, z 2892: peak 243, ring mean 52, ring max 119; x 5311, z 1061: 292 vs 90; the hotspot shield at x -955, z 1445: 302 vs 195; a seamount at x 2969, z -786: -159 vs -253), but at 1-3 blocks a pixel they read as parts of ridges and plateaus, not as cones (the elevation ramp is flat above 639, and the events' place is probably the segment's, not the cone's centre). I therefore claim no volcano shot. **Known flaw seen in the shots:** sea ice (and the ice cover over shallow water) has square edges, the 8-block cells of the bake's ice cover and the 16-block ones of the sea-ice proxy.
  - **Flaws and left.** (1) The volcano review needs a look at the amplified cones themselves, e.g. through `deeptime volcano-view` with a checkpoint (`run --checkpoints`), which marks them on the same terrain; the map could mark volcano events (a layer from `events.jsonl`) so they can be found. (2) `find_places.py`'s fjord scan found only three arms at a weak threshold; it works but is crude. (3) Touch (pinch) was not exercised on a touch device. (4) The Mac scratch of the earlier attempt was deleted by the coordinator; mine on the box (`/data/ban/deeptime-runs/wb-zoom`) is deleted; the checkout copy `/data/ban/mic/mods/agent-ad5fc9991593ce7ef` (2.1 GB) goes with `mac-gc.sh` after the merge.

### 2026-10-01 — Workbench: rock at depth and cross-sections (agent worktree agent-aa7d40b4ebf642bd7, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Why.** Decisions log 2026-10-01, "Which workbench features?": Ben asked for rivers, lakes, coal, diamictite and volcano
  shapes to be visible in the workbench; "All four", view-only, run time unchanged. This is "Rock at depth + sections".
- **What Ben gets** (a finished run with a bake; any other run says why it cannot):
  - *Depth control.* On a rock layer (any layer with the `rock_class` ramp: `structure/rock`, `surface/class`, …) a "Rock at
    depth" control sits under the legend: chips Surface, −20, −50, −100, −200, Floor and a slider over 0, 10, 20, 35, 50, 75,
    100, 150, 200, 300, 400, 600, 800 blocks and the world floor. Below the surface the map shows the bake's rock class that
    deep under the terrain's top block (the picture sits under the map like a frame's layer, so the globe and the hover
    readout follow it; hovering names the class); a badge on the pane says "Rock 100 blocks down · the finished world
    (bake)", and the control's note says the time slider does not change it. The surface is still the run's own frame.
  - *Section tool* (toolbar, key S): click two points; the line is drawn (the shorter way round the planet, across the
    seam too) and the section opens in a dock over the map (the upper edge when the line runs along the lower one), or
    large in a dialog: rock classes in the `rock_class` colours, the ground, water over it, sea level, an elevation axis in
    blocks (y), distance along the line in blocks and km, the line's length, the deposit/seam/pluton bodies it crosses
    (outlined by role; "coal seam ×8"), a key of the classes present with their share (click one to light it up: diamictite,
    coal), a hover readout (distance, x, z, y, depth below the surface, the rock, the water, the bodies inside), a moving
    dot on the map, "Whole column / Near the surface" and the scales ("stretched 37× vertically"). The Section tab keeps
    the line's numbers, the key and the recent sections.
  - *No bake:* the depth control greys out and says why (a run that has not finished: the bake is written at its end; an
    older format: "world data is format 0.0; this workbench reads 0.2"; a stage run: no bake); a section pick says so in a
    toast. The map keeps showing the run's frames.
- **How.** New `crates/deeptime-cli/src/watch/sections/` (`mod.rs` routes, caching and the page's script; `world.rs` the bake
  opened for rock views; `slice.rs`; `section.rs`; `tests.rs`), hooked in at one place in `http.rs::get`:
  `GET /api/runs/<id>/rock` (availability, scale, depths, class names and colours), `…/rock/slice.png?depth=<blocks|floor>`
  (1024 × 512 RGBA, a keyframe's projection) and `…/rock/section?x0&z0&x1&z1` (JSON, up to 2,048 columns). The bake's
  strata are read whole into memory once (kept 10 minutes after the last request, one world at a time); answers are kept
  as files in `<run>/rock/<bake stamp>/`; one slice or section computes at a time on up to 8 threads. Page: new
  `crates/deeptime-viewer/assets/sections.js` (served at `/sections.js`) plus ~30 lines in `workbench.js` (`window.DTWB`
  for the new file, hooks in `drawPane`, the pointer handler, `setTool`, `setITab`, the tool buttons, the S key, the 64-class
  legend) and 4 lines of `workbench.html` (the tool button, the tab, its pane, the script tag). The rock a block has is what
  the game's strata pass paints: the horizon stack by the column rule, then a body that names a rock class (the first by
  id, as `StrataPass.paintBodies` leaves it).
- **The one change outside the workbench: `strata::stack_at` is faster.** A slice asks for 524,288 columns and the old
  function allocated 17 vectors and searched every horizon from the top for every (cell, key) pair. It now keeps its
  working memory in a `StackScratch` (one per thread; `stack_at` makes a fresh one, so every caller is unchanged) and
  walks each cell's horizons once up the sorted keys: the search for key s is "the first horizon whose successor starts
  after s", true for every smaller key too, so starting from the previous answer finds the same horizon. The arithmetic
  and its order are untouched. `strata::tests::reusing_the_scratch_and_walking_the_keys_changes_no_answer` keeps the old
  code as a reference and compares 2,000+ random columns (random stacks, hiatuses, repeated classes) with both `stack_at`
  and `stack_at_with` for equality; the 500-column Java parity test and the tiny-planet tests below ran on it.
  The deposits stage's `Cover::blocks_at` (coal cover) also calls `stack_at`: same answers, fewer allocations.
- **Measured** (Mac, load average 60–130 all afternoon: other agents' runs and servers; wall times are noisy, CPU times
  are the figure to trust). Run: `earthlike_quick_16k`, 500 Myr, scattered continents, 12.1 horizons a cell on average
  (K = 32 slots), baked 206 MB. *Slice* (1024 × 512): 3.4–4.0 s wall before the `stack_at` change at load 47, then 2.1–3.4 s,
  and 1.1–1.6 s (4.4 s of CPU, 8 µs a column) at load 63; cached: 3 ms. *Section* of 2,048 columns: 25–37 ms (cached 2 ms).
  *Opening the bake:* 0.15–0.21 s. A planet run adds nothing: the page only reads.
  **Run time before and after** (Azure test VM, `/usr/bin/time -v deeptime run --preset earthlike_quick_16k --start
  scattered_continents --length-myr 500 --threads 6`, release builds of master b46f512 (base) and of this branch, one run at a
  time in the order base, branch, branch, base, load average 25–50 from other agents' jobs): **user seconds base 1,934 and
  1,714 (mean 1,824), branch 1,866 and 1,799 (mean 1,832)**; wall 9:53 / 7:22 against 8:50 / 7:48; max RSS 6.1 / 5.1 GB
  against 5.7 / 5.1 GB. The two builds differ by less than the spread between repeats of one build (the second run of each pair
  was faster as the box quieted): no measurable change. The only code a run executes that this branch changed is
  `strata::stack_at` (the deposits stage's coal cover, 42 / 23 s base against 24 / 26 s branch), bit-identical and with
  fewer allocations.
- **Tests.** `watch::sections::tests::rock_at_depth_and_sections_agree_with_the_column_evaluator` (a tiny baked planet,
  through `route()` as the server calls it): 96,932 blocks of 368 sampled section columns equal `ColumnWorld::sample`'s
  rock block for block; the water and ground equal `hydro()`; body spans equal `sample().bodies`, 93 of them through the
  planet's plutons; four slices (surface, 60, 200, floor) equal the evaluator at every sampled pixel; PNG size and cache;
  400, 409 (no bake, with the reason) and 404 answers; a line across the seam takes the short way. Unit tests for the
  pixel projection, depth parsing, the seam rule and the body index (odd circumferences, bodies over the seam), and for
  `sections.js` being served. `tools/workbench/sections_shots.py` drives the page (real mouse events, headless
  Chromium on the Mac): depth chips, three sections, fold/close/show, the surface chip, Esc: no page errors.
- **Pictures** (not committed): `review/out/wb-sections/` in this worktree: `rock-surface/-20/-100/-floor.png` (the map at
  each depth), `dia-dock.png`, `dia-dock-lit.png`, `dia-big.png`, `dia-big-lit.png` (a section through a baked diamictite
  bed: 54.9° S 72.3° E → 73.6° S 89.9° E, 350 km, the basin fill with the plum bed lit up in its key), `coal-dock.png`,
  `coal-big.png` (coal beds and eight coal-seam bodies), `seam-dock.png` (a line across the seam), `*-tab.png`,
  `no-bake/rock-100.png` (a run whose world data is format 0.0).
- **Found on the way.** (1) `ColumnWorld::sample` lets the *last* body that names a rock class win where two overlap; the
  game's strata pass keeps the *first* (a block a body painted is no longer crust or the horizon's block). The slice and
  the section follow the game; `inspect-column` and the Java twin (`Columns.sample`) agree with each other and can
  differ from the game at such a block (rare; none in the test planet). (2) The default 500-Myr run has diamictite in
  0.04 % of surface pixels and 0.09 % at 50 blocks, mostly at 85–87° latitude and in a basin near 67° S; coal 0.07 % at the
  surface, 0.2 % at 50 blocks. (3) The first slice of a run opens the strata (117 MB at 16k, about 470 MB at 32k full):
  they stay for ten minutes. (4) A run that is still running has no bake until it ends; the page says so.
- **Gates** (Azure test VM, this branch merged with master b46f512; `CARGO_BUILD_JOBS=8`): `cargo fmt --check` clean; `cargo clippy
  --workspace --all-targets -D warnings` clean (it first caught a needless borrow in `stack_at_with`, fixed); `cargo test
  --workspace` all pass (core 335 + 3 ignored, cli 38 incl. the five `watch::sections` tests, format 52 incl. the `stack_at`
  equality test, inspect 29 + 9, jni 7, viewer 6, history_cap 1); `cargo deny check licenses bans` ok; `cd neoforge &&
  ./gradlew build test` BUILD SUCCESSFUL, tests=160 failures=0 errors=0 skipped=1, **ColumnParityTest `columns=500 equal=500`**
  (71 through bodies, 31 with water; lake1024 300/300) against the new `stack_at`. No GameTests (branch rule).
- **On a real baked planet** (`tools/workbench/check-sections.sh`, a fresh 500-Myr `earthlike_quick_16k`,
  merged master): the slice takes 1.4–2.0 s cold (load ~40, 12.1 horizons a cell), the section of 2,048 columns 0.03 s; opening the
  bake 0.07 s; `sections_shots.py` in Chromium 153: depth chips, three sections, fold, unfold, close, show from the tab, the
  surface chip and Esc all ok, no page errors. The pictures were first made without fonts (nixpkgs' Chromium has none);
  `tools/workbench/sections-shots.sh` now gives it DejaVu. A section through a diamictite bed (68.1° S 207.2° E → 80.2° S,
  165 km, `dia-big-lit.png`): a 1.8 % bed the key lights up in plum, thinning up a slope and cropping out in a fjord-like
  trough. New diamictite coordinates differ from the first run's (master changed the planet).
- **Merged with the rivers & lakes and final-world branches** (master 386bd14; conflicts in STATUS.md, `http.rs`, `workbench.html`,
  `workbench.js`, `lib.rs`, all keep-both): one `window.DTWB` now serves `sections.js` and `worldmap.js`; the rock legend limit
  (64 classes) sits beside the rivers legend branch. Re-run on the Azure VM: fmt, clippy `-D warnings`, `cargo test --workspace`
  (core 351, cli 46, format 52), `./gradlew build test` (160/0, `ColumnParityTest columns=500 equal=500`) all green, and
  `check-sections.sh --other-views` on a fresh baked run: the sections checks pass, the Rivers & lakes layer shows its own
  legend (no depth control), the Final world view opens, paints and closes, the rock layer's depth control comes back, no
  page errors (a slice 0.8–1.2 s cold with the box quieter).
- **Next.** The coordinator merges this with the rivers, deposits and zoomable-map branches (the conflicts will be the
  `workbench.js`/`workbench.html` hunks and this file's top); restart the live workbench on the test machine to get it
  (`tools/viewer/mac-watch.sh`). Runs made before format 0.2 show "needs a bake" until they are run again. Scratch deleted;
  this worktree's checkout copy `/data/ban/mic/mods/agent-aa7d40b4ebf642bd7` stays until `tools/remote/mac-gc.sh` after the merge.

### 2026-09-30 — P1-27 plate margins and coherent continents (agent worktree agent-ad3b417fd428c1ac2, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Why.** Ben (decisions log 2026-09-29, "Provinces view shows speckled, patchy continents"; 2026-09-30, "most coast lines
  are getting speckled with mountains"): "Investigate + fix". Workbench run 1a0f5089640 (earthlike_quick_16k,
  supercontinent, 750 Myr) at step 101 had an arc + back-arc band on nearly every continental coast and arc cones
  about 60 km apart on all of them, on low coastal plains.
- **Measuring margins** (new; `plates::stats::margins`, recorded every 10 Myr into `notes.plates.margins`; `stage`/`run`
  print a `margins` line with the means after the first 100 Myr and the series; `deeptime margins --in <checkpoint>
  --out map.png` draws the classes). The margin is the continent–ocean boundary: visible continental samples with
  oceanic crust within 1.5 spacings, one spacing of coast each, on landmasses of at least 200,000 km² Earth-normalized
  (smaller island-arc specks are counted apart, `fragment_coast`). Classes, first that applies: collision (regime) >
  subduction (a confirmed, converging front within 400 km on either plate; "Andean" when the continent overrides) >
  other plate boundary > passive. Also the regime layer's share (arc, back-arc, trench, collision at the coast) and
  the age of the floor beside passive margins.
- **Earth.** Present-day passive margins total 105,000 km against 53,000 km of convergent plate boundaries (Bradley
  2008, Earth-Sci. Rev. 91:1–26); active continental margins ~45,000 km counting arcs like Japan (Lallemand, Encycl.
  Marine Geosciences), ~28,000–32,000 km with continental overriding plates only (tallied from Schellart et al. 2008,
  Earth-Sci. Rev. 88:118, Table 1); convergent ≈ 48 % of continental margin length counting island arcs (Mercier de
  Lépinay et al. 2016, Tectonophysics 693). So **passive ≈ 0.7, active ≈ 0.2–0.3** (0.48 at most). Modern passive
  margins average 104 Myr old (oldest 180); ancient ones lasted 181 Myr on average (Bradley 2008), who calls a direct
  passive-to-subduction conversion "rare indeed"; Stern 2004 (EPSL 226:275) and Stern & Gerya 2018 (Tectonophysics
  746:173): no Cenozoic passive-margin collapse, subduction starts at transforms/fracture zones or by polarity reversal.
- **Before** (master's plate model with the instruments, commit 4cf2cda): plates only, 4 runs × 750 Myr (2 starts × 2
  seeds): active 0.61–0.70 of the margin, passive 0.29–0.37, regime arc/back-arc 0.60–0.68, floor beside passive
  margins (median) 64–72 Myr. Coupled pipeline, 6 runs × 400 Myr (`tools/review/p1-27-ensemble.sh`): active 0.67
  [0.59–0.74], passive 0.30, regime 0.69, floor 50 Myr.
- **The cause, measured.** Not the spontaneous initiation: it fired 0–2 times per 750-Myr run (only plates that subducted
  nowhere qualified), and switching it off left the active share at 0.59–0.71. Nor rifting (half the rate: 0.57–0.69),
  plate-motion noise (none: 0.59–0.68), slab pull (off: 0.56–0.66), slow slab-free plates (0.65–0.69), slow continents
  (0.60–0.67) or fewer plates (≤ 20: 0.56–0.68). Every rift, initiation and collision now reports the continental margin
  it cut off its own ocean (`coast_split_km`): per 750-Myr run (planet km; the whole coast is ~61,000) collisions
  295,000–455,000, terrane accretions 89,000–130,000, rifts 1,700–2,400, initiations 40–270. **Cortial's collision
  moved the colliding continent alone to the overriding plate and left its ocean on the old plate, which kept
  converging: every passive margin of that continent became a plate boundary and then an Andean margin** (~10
  collisions per 100 Myr). The regime layer adds an 800-km back-arc halo (`subduction_distance_km`), so its map showed
  more active coast than there was. Rifted margins only become plate boundaries transiently.
- **Changes** (all settings; pipeline/earthlike presets, editor fields with lang keys, plate-only presets at P1-03's
  behaviour where noted):

  | setting | value | what |
  |---|---|---|
  | `tectonics.collision_suture_frac` (new) | 0.1 | a colliding continent of at least this share of an average plate sutures its whole plate to the receiving one (plate_death `sutured`), keeping the ocean on its other coasts; the crust is resampled at once so the welded slab is consumed. Smaller terranes still transfer alone |
  | `tectonics.subduction_initiation_clearance_km` (new) | 1,000 | old floor may start subducting wherever it lies this far from every trench (before: only on plates that subducted nowhere, so welded plates' old floor aged for ever); the split never tears a continent (slivers stranded in the old floor ride along) |
  | `tectonics.subduction_initiation_rate` | 0.1 → 0.3 | keeps the floor recycled with sutures (ocean p90 ~190 Myr) |
  | `tectonics.cordillera_uplift_mm_yr` / `_width_km` / `_height_m` (new) | 0.2 / 120 / 2,500 (plate-only 0) | continental crust over an arc above an oceanic slab rises in a Gaussian band round the arc axis towards the height (thickened crust spreads), with its Airy root added to the crust's felsic plutonic (0.6) and metamorphic (0.4) pools |
  | `tectonics.collision_orogen_core` (new) | 0.5 | a collision stamps its Himalayan orogeny (folds, metamorphism) only where its surge reached half its peak; before, everything lifted by 10 m: belts up to 4,000 km wide |
  | `tectonics.arc_continental_m` (new) | 80 (plate-only 0) | an emerged island arc becomes continental only after this much arc magma (~8 Myr of arc): volcano cones lifted young intra-oceanic arcs above the sea, and with initiation in the open ocean the continental crust grew from 0.41 to 0.56 of the planet in 750 Myr (0: 51,000 conversions; 300: crust fell to 0.20; 60: 0.44–0.48; 120: 0.30–0.37; 80 keeps it at ~0.40) |

- **Cone heights do not double-count.** The volcano stage's cone relief is the height it built (`height_m`, target from
  the arc flux), not measured from the ground; the ground only enters the summit ceiling (6.8 km above the sea). The
  amplifier's measured relief is clamped to the built cone. A range under a cone raises the summit, not the cone.
  The volcano stage now records the ground under active cones on continental landmasses (surface at the summit less
  the cone's own relief) every 10 Myr (`notes.volcanoes.arc_bases`). Scale note: at 16k a cone's footprint (radius ~28
  km with the default flank steepness 0.5) is about as wide as an Andes-wide range shrunk by the Earth normalization
  (×0.123), so the range shows as the cones' common base more than as a separate ridge.
- **Provinces (Part 2).** Causes of the patchwork, measured with a new coherence line (`provinces …`: class shares by
  true area, edge density = share of neighbouring continental cells of different classes, speck share = cells in
  same-class patches under 25 cells, and why cells are orogens): (1) the active-margin rings (arc regime and Andean
  ranges) on every coast, gone with the margins fix; (2) collision stamps covering most of every continent (the 10-m
  threshold over surges up to 2,000 km), fixed by `collision_orogen_core`; (3) the one-sample passive-margin regime
  drawn as dashed "extended crust" along every coast; (4) island-arc specks in the oceans. View fixes (`provinces.rs`,
  view-only): rifted margins are a continuous band 150 km (1.35°) wide along every quiet coast (`rifted_margin_deg`),
  the generalisation looks half the USGS map's 150-km feature size (radius 2 cells at 1024) and continental patches
  smaller than that feature squared (15 cells) take the sea floor's class (`feature_deg`, `drop_specks`).
  Earth reference (CRUST 2.0 crust types, land without Antarctica, derived by a sub-agent): Precambrian basement 45 %
  (exposed ~27 %) + Phanerozoic platform 25.5 % ≈ 71 % cratons, orogens 12 % + arcs 5 %, extended + rift 7 %.
- **After** (this branch; `tools/review/p1-27-ensemble.sh 400 "1 2 3" "supercontinent scattered_continents"`, 6 runs
  × 400 Myr, mean [min–max], before → after):
  - margins: active 0.67 → **0.31** [0.29–0.35], passive 0.30 → **0.63** [0.59–0.66], regime layer 0.69 → 0.45, floor beside
    passive margins 50 → **102** Myr; plates only (4 runs × 750 Myr): active 0.29–0.36, passive 0.59–0.65, floor 103–141 Myr;
  - arcs: ground under continental arc cones 307 → **812** m (median of the 10-Myr records), cones on ground below 500 m
    62 % → 41 %;
  - provinces: shield 0.065 → 0.148, platform 0.106 → 0.143, orogen 0.513 → 0.401, basin 0.124 → 0.085, LIP 0.032 →
    0.058, extended 0.161 → 0.165; edge density 0.078 → 0.060, specks 0.062 → 0.033;
  - hypsometry: land 0.398 → 0.391, land median 1,350 → 1,030 m, p90 4,120 → 3,560 m, above 2 km 0.336 → 0.267,
    continental crust 0.461 → 0.420 (dry 0.82 → 0.81); glaciated land 0.235 → 0.204; Köppen A/B/C/D/E 0.09/0.17/0.18/0.12/0.44
    → 0.08/0.22/0.21/0.10/0.40;
  - plates: 17.3 → 10.5 at the end, ocean p90 108 → 194 Myr [162–228], younger than 250 Myr 0.996 → 0.955, collisions 43
    → 40, initiations 1.2 → 25.5.
  - 750 Myr, seed 1 (`tools/review/p1-27-evidence.sh`): supercontinent active 0.67 → 0.32, floor 58 → 124 Myr, land 0.33 →
    0.32, crust 0.40 → 0.40, ocean p90 116 → 224 Myr; scattered active 0.70 → 0.32, floor 54 → 128 Myr, land 0.38 → 0.41,
    crust 0.44 → 0.40, ocean p90 104 → 209 Myr.
- **Against P1-17.** Master itself has drifted from P1-17's calibration (land 0.27–0.28, median ~0.6 km at 500 Myr then;
  0.40 and 1.35 km in the before ensemble at 400 Myr; Earth 0.29 and 0.43 km): this branch does not widen the gap
  (median and high ground come down), but P1-17 proper needs a recalibration pass.
- **Evidence** (pulled, not committed): review/out/p1-27/evidence/index.html (margin series chart, run summaries, the
  same keyframes before/after of regime, elevation, satellite, provinces and volcanoes for both starts, 750 Myr, seed 1;
  each run is also a viewer bundle), review/out/p1-27/maps/ (margin-class maps, plates only, supercontinent seed 1),
  review/out/p1-27/diag/ and seq/ (the investigation's maps).
- **Tests.** `plates::tests::margins_are_measured_and_add_up`, `sutures_and_initiations_keep_continents_whole` (sutured
  plates own nothing; initiations never tear a continent; a suture leaves no coast behind), `arcs_build_a_cordillera`;
  `geometry::tests::coast_split_counts_continental_edges_cut_off_their_ocean`; `provinces::tests::
  rifted_margins_are_a_band_on_quiet_coasts_only`, `specks_below_the_feature_size_go_to_the_sea`; the volcano pipeline
  test checks `arc_bases`. Fixed on the way: a subtraction overflow when an initiation split a plate whose count was
  stale after a suture.
- **Gates** (Mac): `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check
  licenses bans` clean; `cargo test --workspace` core 323 + 3 ignored, cli 30, …, all pass; `cargo test --release -p
  deeptime-core plates::tests::statistics -- --ignored` passes (p90 165–200 Myr, 95.7–97.1 % < 250 Myr, 27–30 plates);
  `./gradlew build test` tests=155 failures=0 errors=0. Re-run after merging master 4105de7 (P1-22 rivers and
  lakes, volcano follow-ups, diamictite, coal placement): core 332 + 3 ignored, cli 31, fmt, clippy, deny clean, JUnit
  157/0; one 400-Myr scattered run on the merged tree matches the ensemble (active 0.31, passive 0.63, floor 103 Myr,
  orogen 0.33, cones' ground 745 m). No GameTests (coordinator).
- **For the coordinator's GameTests.** The built-in planets change (plate model, cordillera, presets, the test planet
  `planet_world_4k`): expect different land/biome counts in planet-world tests; climate shares on the 400-Myr ensemble
  stay close (E 0.44 → 0.40, B 0.17 → 0.22), so `climateConsistent` floors re-based on the test planet may need the
  same treatment as after the EF change. Checkpoints from before this branch do not resume (new state fields).
- **Next.** Ben's decisions (final report); P1-17 recalibration of master's hypsometry; P1-10 uses the provinces.
### 2026-10-01 — Protect water blocks: carvers never replace Deep Time's lake and river water (agent worktree agent-af1249582ad3d3fa3, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **What.** Ben's decision of 2026-10-01 ("Protect water blocks"): caves still run beneath rivers and lakes and may open into a bed, but carvers never replace water blocks Deep Time placed. Vanilla's `#minecraft:overworld_carver_replaceables` lists water, so since 2026-09-30 ("Allow caves under water", no carving mask) carvers cut air pockets through lakes and rivers and now and then a hole in a lake's surface.
- **Mechanism: the carving mask, water cells only.**
  - Vanilla's `WorldCarver.carveEllipsoid` tests the chunk's `CarvingMask` before `carveBlock`, so a masked position is never replaced. `WaterBodies` already runs before the carvers (in `buildSurface`), so it marks each water block it places or keeps in a river, lake or topped-up sea column, for the air and liquid steps. Cells: a river's bed + 1 to top; a lake's first ground + 1 to top; the sea cells it filled; and the flowing and falling blocks of a step. Nothing else is masked: bed, banks, plugs and ground under and beside the water stay carvable.
  - The first build (2026-09-29, b3d4750) masked a shell two blocks under the beds and a block over the tops, which kept caves from the ground under the water; Ben's 2026-09-30 decision removed it. This one is the middle option the P1-22 entry asked about.
  - **Why not re-assert after the carvers:** a post-carve pass would need the whole column region again (the pass's 0.3 ms a chunk) and a hook, and it would still let the carver write air first. The mask costs a bit per water block, no hook for the protection itself, and uses vanilla's own contract.
  - **Why not edit the replaceables tag:** it is global (every pack's water, the aquifer's sea too), cannot be switched per world, and another pack's tag can add water back.
- **Where a cave opens into a lake or river bed.**
  - The lowest water block of that column now sits over air, and the game does not tick anything placed at generation, so left alone it would hang (the water-check counts it as water floating over a cave).
  - `GeologicChunkGenerator.applyCarvers` (new override, air step only) calls `WaterBodies.wake`: for each column whose lowest water block has air under it, that block is marked for post-processing (`markPosForPostprocessing`). Vanilla runs `postProcessGeneration` when a chunk first ticks and ticks the fluid: the source spreads down as falling water into the cave, as for any fluid a carver exposes. The lake's own sources never drain; a stream spreads on the cave floor up to 7 blocks.
  - Chunky only generates to FULL, which does not tick, so a pregenerated region holds the marks and the flow starts at first ticking.
  - `wake` runs only in chunks a river or wet `water_level` cell reaches, and walks each column from its top down through water: about 256 block reads.
- **Setting:** `hydro.protect_water_from_carvers` (bool, default true, all eight presets, section "Advanced · Rivers and lakes", toggle "Caves spare water"). It lands in `notes.bake.water.params`, which the game reads (`Water.protectsFromCarvers`; absent means true, so a world baked earlier is protected). Off, the pass masks nothing and `wake` does nothing: exactly the behaviour of 2026-09-30.
- **Tests.**
  - JUnit `WaterProtectionTest` (3): the reader's default and parsing, and that a baked world (planet4k) records the setting as true.
  - GameTest `RiverGameTests.carversSpareWater` on a new level `deeptime:test_rivers_carved` (`test_rivers`' world data under biome `deeptime:test_carved`, whose only carver `deeptime:test_wide_caves` is a cave carver that starts in every chunk, 1.5 to 2.5 times vanilla's width, y -32 to 180). It reads back every river and lake column in the four `riversHold` boxes: no air between the bed and the top, no top block of air, and asserts the carvers did cut the ground under the beds and opened some beds. **Written, compiled, not run** (no GameTests on a branch): the coordinator's batch run will be its first.
  - `deeptime-inspect water-check` gained, with the world data: `water_columns`, `drained_water_blocks`, `drained_water_columns`, `surface_holes` (air above a river or lake column's lowest water block, and air for its top block), `drained_samples`, and `floating_over_caves_sources`.
- **Measured** (Mac, standalone profile, `preview.py run --keep-world`, the synthetic lake world `deeptime synth --kind lake-test --size 16384` (a basin and channel of 7,298 columns, floor y 110, water surface at y 120), box -64,-96,192,160, y 0..260, seed 7; before = the same jar with `hydro.protect_water_from_carvers` false in the manifest, after = true; `deeptime-inspect water-check --min-lake 50`):

| | carvers | drained columns | drained blocks | surface holes | water over caves | leaks into caves | lakes found |
|---|---|---|---|---|---|---|---|
| before | vanilla | 601 | 1,281 | 305 | 341 | 553 | 1 (6,993 columns) |
| after | vanilla | 0 | 0 | 0 | 22 | 42 | 1 (7,298 columns, rim − surface 0) |
| before | `tools/dev/datapacks/wide-caves` | 1,700 | 5,568 | 1,189 | 924 | 1,717 | 5 pieces |
| after | wide caves | 0 | 0 | 0 | 182 | 578 | 1 (7,298, rim − surface 0) |
| after, settled | wide caves | 0 | 0 | 0 | 0 | 48 | 1 (7,298, rim − surface 0) |

  - "Water over caves" is water over air in a river or lake column, in the saved region files. "Settled" is the same world booted again with the box force-loaded for 120 s (`forceload add`, so the chunks tick and post-processing runs): the 182 sources hanging over carved beds became falling water (water blocks 66,755 to 67,905; sources 66,293 to 66,429), 0 left floating, the lake intact. The commands are below.
  - The lake census still lists 1 to 3 "drained" lake columns beside the lake in the after rows (3 and 1 vanilla, 1 wide): shore columns whose generated ground stands above the lake's level, so there is no water to place there. They are not carved water (the new counters read 0).
  - Cost not re-measured: the pass adds a bit-set write per water block (two masks); `wake` reads about 256 columns' water in a chunk that has any.
- **Commands.** `tools/remote/mac.sh --server -- 'python3 tools/workbench/preview.py run --lock-held --job J --profile standalone --size 16384 --box=-64,-96,192,160 --y=0,260 --world-data=smoke/wp/lake-on --datapack=tools/dev/datapacks/wide-caves --seed=7 --keep-world'` (lake-off has the setting false in its manifest); then `deeptime-inspect water-check --area=-64,-96,192,160 --world-data J/world/deeptime --min-lake 50 J/world`. The settle run was a scratch script over preview.py's `Server` (boot, `forceload add -64 -96 191 159`, wait, `save-all flush`, stop), not committed.
- **Gates** (Mac, on this branch, base bb6672b): `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo deny check licenses bans` clean; `cargo test --workspace --no-fail-fast` all green (core 326 + 3 ignored, the others as before); `./gradlew build test` BUILD SUCCESSFUL, `tests=160 failures=0 errors=0` (157 + the 3 of `WaterProtectionTest`; parity and the editor catalogue tests unchanged). No GameTests (branch rule).
- **Follow-up: `carversSpareWater` premise (batch run failed "the carvers cut only 0 blocks under the beds").**
  - **Cause:** the test's metric, not the carver. `test_rivers_carved` uses `deeptime:planet_nocaves` (no aquifer), whose fluid picker fills a carver's cut below sea level (63) with water, so cuts under beds near the sea were water, not air, and the test counted only air. The carver was on the biome (`[deeptime:test_wide_caves]` read back) and cut 42 % of the ground cells sampled in the boxes (23,193 of 56,696 water, 653 air).
  - **Fix (commit 8b48bce):** `carvedBelow` and `entrances` count air or fluid under the beds (the pass leaves solid ground there); a new premise needs 20+ columns with water above sea level (the only place a carver leaves air) and 20+ cuts under them; the test carver's `replaceable` is `neoforge:or` of the vanilla tag and `mineable/pickaxe` and `mineable/shovel` (Deep Time's rock, e.g. polished tuff, is outside the vanilla tag); premise diagnostics are logged.
  - **Run:** `./gametest.sh -PgametestOnly=carversSpareWater` on the Mac, standalone: PASS, `columns=695 drainedBlocks=0 holes=0 carvedBelow=1589 entrances=246 highColumns=106 carvedBelowHigh=434`. With `hydro.protect_water_from_carvers` false in the test planet's preset the test fails at its first check (`protectsWater`), not on drained water; the drained-water side has not been shown to fail in-game with protection off (the water-check run above shows it on the lake world).
  - **Not run** (Ben stopped Mac testing, 2026-10-01): the full `./gametest.sh` and `-PwithMic`.
- **Not done / for the coordinator.** `carversSpareWater` has not run. The measurement jar was built before the periodic log line gained its two counters (water blocks kept from the carvers, columns woken); nothing else changed in it. With vanilla carvers on a real planet the effect is as above but rarer: a cave has to reach a lake. Scratch on the Mac (`smoke/wp`, the `wp-*` jobs) is deleted.

### 2026-09-30 — P1-26 per-cell climate: design doc (agent worktree agent-a2104321849041969, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **What.** The design Ben asked for ("Design doc first", decisions log 2026-09-30) is
  `docs/design/p1-26-per-cell-climate.md`, awaiting his approval. No code changed. It covers:
  - the current banding measured;
  - a seasonal 2-D EBM on a 128 × 64 climate grid: land/sea tiles, per-cell sea ice and albedo,
    surface-dependent diffusion, zonal heat advection by a low-level wind profile, Stommel gyres
    from the coastlines, and an overturning conveyor in the narrowest polar-connected basin;
  - a monthly steady moisture budget (belt + ITCZ + thermal winds, bulk evaporation, soil bucket,
    Bretherton precipitation, Smith & Barstad kept on L1 as a sink) behind `climate.rain_model`,
    with today's belts, fed by the per-cell temperatures, as the first default;
  - the 1-D clock, flux-corrected per band at each snapshot, with pattern scaling between snapshots
    and triggered snapshots at snowball and hothouse crossings;
  - unchanged baked layers and climate API, so mic-climate needs nothing;
  - costs, validation targets, tasks T1–T11, licences, and open questions Q1–Q11.

  The plan's P1-26 entry points at the doc. It fits the ice branch (`458c15c`): its `sheet::Grid`
  becomes the shared coarse grid, and the sheet's ELA and precipitation forcing come from the 2-D
  climate.
- **Measured** on the Mac (load 70–95; it rebooted once; scratch under
  `/Volumes/Graphide/ban/deeptime-runs/p1-26-design/`, deleted at the end).
  - **The real run.** `deeptime stage climate --preset earthlike_quick_16k --start
    scattered_continents --length-myr 500` took 864 s. The climate stage was 186 s: clock 76.5,
    snapshots 38.5 (51 × 0.75 s), ice 31.4. Fields were taken with `dump-fields`.
  - **Earth's relief.** A scratch copy of `climate-dem` ran on GMT `earth_relief_30m_p` and dumped
    the months.
  - **Observations.** Beck et al. 2018 Köppen and Beck et al. 2023 monthly T/P, 0.5°, CC BY 4.0,
    validation only.
  - **How banded today's climate is.** Latitude explains 99.6 % of land sea-level temperature
    variance (Earth 96.8 %). Winter along-row std is 3.1 K on the real run and 3.4 K on Earth's
    relief, against Earth's 6.1 K. Sea ice has η² 0.998, and the southern ice edge has a std over
    longitudes of 0.23° (0.00° on Earth's relief). The polar continent is all ET/EF.
  - **Today's skill on Earth's relief.** Köppen agreement with Beck is 34.6 % exact and 64.9 % by
    main group. Monthly T RMSE is 6.6 K and ln P RMSE 1.11. Yakutsk in January is −16.5 °C (observed
    −39), and Mumbai gets 531 mm (observed 3,096).
- **Prototype** (scratch Rust, not committed; Appendix B of the doc). It is the 2-D EBM + ocean +
  moisture budget at 2.5° on Earth's relief.
  - **Cost.** 0.10–0.13 s CPU per model year on one core, 0.3–0.8 s for the twelve monthly moisture
    solves, 26 MB.
  - **Closure.** Global means 14.9–15.5 °C against the 1-D model's 15.2 °C, net TOA ≤ 0.01 W/m²,
    water to 3·10⁻⁶.
  - **Contradicts the plan's premise:** a plain diffusive 2-D EBM keeps winter temperatures as
    banded as today (3.2 K).
  - **What fixed it.** The conveyor (January sea off Brest 4.3 → 9.2 °C, northern ocean heat
    transport 0.7 → 1.6 PW), a low-level westerly profile for heat advection and a winter inversion
    term. With them the best run beat today's model on the same grid: winter RMSE 4.9 against
    7.2 K, along-row std 5.4 against 3.3 K (Earth 6.0), η² 0.949 against 0.975 (Earth 0.944).
  - **Rain is not yet better than today's belts.** ln P RMSE is 1.7–2.0 against 1.05, even though
    the bucket gives the Sahara 32 mm and the Kazakh steppe 170 mm. Hence the staged rain plan
    (Q2).
- **Next.** Ben answers Q1–Q11 (doc §10). The build starts after the ice branch lands: T1, the
  shared coarse grid, first, then T2/T3/T4 in parallel.
### 2026-09-30 — Coal placement knows the sedimentary cover; eroded fields free their space (agent worktree agent-aacd864d2709d52dd, branch coal-cover-placement, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Why.** After coal was capped to the cover (entry below), counted coal fell to 73 % of the USGS target: fields
  were drawn where there is no cover to hold seams. Follow-up asked for: fields form only where there is cover, in the
  formation weights, not a post-hoc fix; then Ben (decisions log 2026-09-30): "Yes, eroded fields free their space
  (Recommended)".
- **Cover in the weights** (`deposits::mod`, the counts loop). For a stacked type with a cover, each tract cell's room
  is `(Cover::blocks_at − COVER_MARGIN_BLOCKS) × m_per_block_v`; a cell with less than one block of room holds no
  coal. Such cells get zero preservation (`p_cell`), the ledger's depth limit is cut to the room, the tract weight
  φ = A·F̄·E uses the favourability of the holding cells only, and the placement weights and `allowed` skip cells that
  hold nothing. The survivor formula then rescales λ so the target is met from the cells that can hold coal.
- **Eroded fields free their space.** The weights alone did not bring the count back (counted 46, sites 62, 93 of
  352 drawn fields found no room): coal is the one adjoining type (non-overlapping plan ellipses), and a field that
  would later be eroded or find no cover still took its footprint. `cluster::place` now takes a `claims(k, point)`
  callback and only a field that claims keeps its ellipse in `placed`; the stage answers with `survives`, which decides
  the field's fate exactly as `materialise` does — the same formation draw (`formation_draw`, shared), the preservation
  ledger, the stack in the cover (`stack_plan`, shared: seam plan, `fit_stack`, capped re-plan), the build-range check
  and the seams left below the ground (`stack_bodies`) — from the same random streams, without touching the world.
  Surviving fields still never overlap; eroded, cover-less and too-deep ones do not crowd them out. Deterministic:
  `materialise`'s draws are unchanged by the refactor, `claims` draws nothing from the placement stream (a second
  `stage deposits` run gives the same census for every type). Types formed at their causes skip the check (none is
  adjoining). Unit test `only_claiming_fields_reserve_space` (a one-cell tract: six fields land when none claims, one
  when all do, four when the fourth is the first to claim).
- **Measured** (Mac, release; the six landscapes: `stage deposits --in` the `stage structure --preset
  pipeline_quick_16k --start {scattered_continents,supercontinent} --length-myr 500 --set seed={1,2,3}` checkpoints;
  master 52b6626's binary, the cover-weights commit c51f775's, and this branch's):

  | coal, six landscapes | master | cover in weights | **+ eroded fields free space** |
  |---|---|---|---|
  | counted (target 11.92 / 13.94 / 11.61 / 5.32 / 9.85 / 15.99, Σ 68.6) | 9 / 9 / 6 / 2 / 5 / 19 = 50 (73 %) | 3 / 10 / 6 / 5 / 8 / 14 = 46 | **8 / 20 / 13 / 5 / 7 / 17 = 70 (102 %)** |
  | sites | 12 / 15 / 7 / 3 / 6 / 19 = 62 | 7 / 13 / 6 / 7 / 9 / 20 = 62 | **13 / 25 / 14 / 7 / 10 / 18 = 87** |
  | fields that found no room (of drawn) | 57 of 294 | 93 of 352 | **7 of 320** (0 / 2 / 0 / 0 / 4 / 1) |
  | expected sites at the placed positions vs the formula's | 94.5 vs 125.7 | 65.5 vs 82.6 | **82.8 vs 82.6** |
  | no_cover / eroded | 56 / 119 | 75 / 122 | 86 / 140 |
  | seams (max per landscape) | 364 (8–22) | 411 (13–22) | 560 (13–24) |

  Per landscape the counts scatter around their targets (Poisson draws; scattered 2 has 20 against 13.9), the sum is
  on target. `stage deposits` 51–74 s per landscape.

  **On master's new histories** (the branch after merging master bca3ff3, which brought the weathering-runoff climate fix,
  volcanoes and glacial landforms: the six histories rebuilt with the merged binary, same command). The new climate
  roughly doubles coal's permissive area (85,126 → 145,780 km² on scattered 1), so the USGS target is now
  **20.41 / 25.33 / 27.87 / 10.45 / 22.58 / 26.89 = 133.5**, not 68.6:

  | coal, six new histories | merged branch |
  |---|---|
  | counted (target Σ 133.5) | **17 / 29 / 35 / 12 / 21 / 28 = 142 (106 %)** |
  | sites | 23 / 32 / 39 / 14 / 28 / 31 = 167 |
  | fields that found no room (of drawn) | **28 of 647** (0 / 6 / 10 / 0 / 5 / 7) |
  | expected sites at the placed positions vs the formula's | 165.9 vs 161.4 |
  | no_cover / eroded | 144 / 308 |
  | seams (max per landscape) | 1,137 (17–22) |

  `stage deposits` 25–126 s per landscape (the Mac's load swung between 19 and 99 during the runs).
- **Gates** (Mac, the branch after merging master bca3ff3, final tree): `cargo fmt --check`, `clippy --workspace
  --all-targets --locked -D warnings`, `deny check licenses bans` clean; `cargo test --workspace --locked` 444 passed,
  0 failed, 3 ignored; `cd neoforge && ./gradlew build test` → tests=155 failures=0. GameTests are no longer a
  branch gate (Ben, 2026-09-30: the coordinator runs both profiles once per merge batch on master); a run already
  queued on the merged tree gave `./gametest.sh` 54 of 54 and `install.sh mic` + `./gametest.sh -PwithMic` 54 of
  54. (Before that merge, on master 0905793: 51 PASS / 1 FAIL in both, `climateConsistent` with master's own
  numbers, since fixed on master in 6070e9e.) The Mac copy and the scratch runs are deleted.
- **Next.** Merge into master (the coordinator merges when green).

### 2026-09-30 — Diamictite: lithified till is its own (60th) rock class (agent worktree agent-a9307a6541fa46794, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Why.** Decisions log 2026-09-30, "Add diamictite (Recommended)": buried glacial till turned into
  conglomerate (cobblestone), which reads wrong for the snowball and ice-age deposits the climate now makes.
- **What.** `tools/dev/gen_rock_classes.py` gains a `diamictite` row (clastic sedimentary, after conglomerate in
  the README's family order; groups `deeptime:rock/clastic` + `sedimentary`): density 2,600 kg/m³, erodibility
  1.2, diffusivity 1.1 (a matrix-strong, unbedded rock between conglomerate and mudstone), weathers to soil,
  `loose_equivalent` till, pelitic metamorphic path (slate → phyllite → schist → gneiss: the mud matrix decides),
  stand-in **`minecraft:cobbled_deepslate`**. `till.lithifies_to` conglomerate → diamictite, and till's
  metamorphic path [] → the same pelitic path (gravel and loess already follow their rocks' paths; the empty
  path left hot buried till as till). The generator takes a per-row `tuned` note; its count check is 60.
  Core: `synth::EVERY_CLASS` and `surface/lithology.rs` FILES are 60 long (diamictite at index 14, every later
  index +1; world data maps indices through `strata/classes.json`, so no reader depends on the order), the
  `deeptime:rock/clastic` group, the satellite colour (grey, with the schists), deeptime-inspect's section
  palette, the every-class synthetic rule `(6·i + h) mod 60` (still every class in any ten cell columns at
  h ≥ 2); two tests sized by the list instead of 59. Viewer: the categorical `rock_class` ramp renumbered,
  diamictite **plum #804080** (the most distinct muted colour against the other 59; a dark grey-green sat on
  hornfels and slate). Java: RockDataTest (60 classes; till → diamictite, diamictite → till, the stand-in),
  StrataGameTests (60 throughout; the vanilla view resolves diamictite to cobbled deepslate; the live palette
  keeps that stand-in until a pack names diamictite, then accepts the pack's member or fallback). FORMAT.md
  (the lithification note, `strata/classes.json`, the synthetic rule), the plan's counts (§1.3, glossary,
  §5.4.1), the decisions file's MIC row "(see log 2026-09-30)".
- **Stand-in: cobbled deepslate.** A diamictite is a dark, fine mud matrix with clasts of every size and shape
  floating in it. Cobbled deepslate's texture is that: a near-black matrix with lighter angular and rounded
  fragments of mixed size, irregularly spaced. Conglomerate's cobblestone is clast-supported (the cobbles touch),
  which is the difference between the two rocks. No other class uses it; overworld block; not filler
  (`StrataPass.isCrust` is stone and deepslate only); not terracotta. Rejected: tuff (seven classes, reads
  volcanic), gravel and coarse dirt (loose: till's own look), andesite and polished blocks (no clasts). Like
  cobblestone under stone, cobbled deepslate is on the MIC README's excluded-variants list as a *member*; it is
  a stand-in only.
- **Measured** (Mac, load ~80; `deeptime run --preset earthlike_quick_16k --start supercontinent --length-myr
  1000 --set climate.degassing_scale=X --set amplify.stream=always`, then `deeptime parity --columns 20000` and
  a scratch plot of the baked horizons; pictures in `review/out/diamictite/degassing-{0.3,0.1}/`, unpublished):
  - **degassing 0.3 no longer snowballs** on this master (the climate agent's −807 → −761 Myr snowball predates
    the weathering-runoff fix 970efd1): coldest 4.1 °C at −824 Myr, sea ice at most 27 % (the onset needs 95 %),
    land ice up to 36 %; 1,816 s. Diamictite in 3.2 % of columns (780 beds, median 2 blocks, up to 67),
    all from the last 230 Myr; the thickest a 67-block bed at 36–37 Myr, lat −59°.
  - **degassing 0.1: six snowballs** (Myr ago, onset → escape: 902–814, 806–732, 726–638, 628–548, 116–72,
    68–8), six hothouses; 1,365 s, 4.2 GB peak. Diamictite in **31.8 %** of columns (13,909 beds, 12,614 of them
    snowball-age; median 2 blocks, up to 127); till (young, shallow) in 4. Thickest: x −13963, z 690
    (lat −15°), y −174…−48, 32–45 Myr ago, inside the 68–8 Myr snowball, a marine basin filled with diamictite
    under 70 blocks of younger shale, with five thinner snowball beds above it and one below (`diamictite-column.png`); the
    meridian section shows the basin fill and tongues running off along the bedding (`diamictite-section.png`).
    The four early snowballs left almost nothing in the baked columns (largest bars at 380, 270, 190 Myr):
    probably 500 Myr of ~40 % land ice eroding them, or burial past the pelitic isotherms.
- **Gates** (Mac, branch merged with master ab5259e): `cargo fmt --check`, `cargo clippy --workspace
  --all-targets -D warnings`, `cargo deny check licenses bans` clean; `cargo test --workspace` all pass except
  one structure test with a `[false; 59]` array (now sized by `EVERY_CLASS`; `structure::` re-run 11/11, the
  viewer and inspect crates re-run after the colour change); `./gradlew build test` 155/0. GameTests not run
  (Ben, 2026-09-30: once per merge batch on master): StrataGameTests' new diamictite checks first run there.
- **Found on the way.** (1) The 1000-Myr `earthlike_quick_16k` run fails at the default 4 GiB cap in amplify
  ("needed 4,451,574,045 bytes"): `stream: auto` decides in memory from the last level's buffers alone, then a
  later reservation passes the cap; `--set amplify.stream=always` finishes (76 s). (2) The Mac rebooted at
  ~19:06 while this task's first 16k run (cap 6 GB, structure stage, ~2 GB in use) ran beside other agents'
  servers at load ~80; 16 GB of RAM in all. Later runs kept the 4 GiB default.
- **For MIC.** The rock-tag README (outside this repo) is not edited: rows for it are in the coordinator report
  (Classes row, clastic group, fallback row and tag file, empty member tag, header counts 60 / 44).
- **Next.** MIC adds those rows and files; then the `-PwithMic` palette shows diamictite as `fallback`, which
  StrataGameTests accepts. The coordinator's batch GameTests run the new checks.
### 2026-09-30 — P1-22 rivers and lakes: done on its branch (agent worktree agent-ad204cf8d55ef26f5, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **What.** Rivers and lakes in v1 (Ben, decisions log 2026-09-29, "Rivers & lakes in v1").
  1. **`core::hydro`** (`hydro/{mod, network, tests}.rs`), run by the bake's new water part (`bake/water.rs`)
     - It runs after the climate part; the spawn candidates now come after it (`terrain::spawn`).
     - It works on the L3 terrain itself, 2 blocks a cell. An L3 over 40 M cells (32k full) is averaged 2 × 2 into grid `L3h`.
     - **The open sea** is every cell at most a block below sea level, connected, in a body ≥ 5,000 km².
     - **Priority-Flood** uses the pit queue (Barnes et al. 2014, from the paper, as `surface::pflood`). The map's first and last rows, past the pole caps, drain as outlets, so no river runs along the map's edge.
     - **Receivers** are the steepest strictly lower neighbour on the filled surface. Across a flat, the receiver comes from the flat's drainage mask (Barnes, Lehman & Mulla 2014, from the paper): towards the flat's outlets and away from higher ground. The first build took the flood's breadth-first parent there, and coastal plains carried parallel straight rays.
     - **Lakes** are depressions ≥ `hydro.lake_min_area_km2` (1) and ≥ `lake_min_depth_blocks` (1) deep, always full to their spill level. A lake passes on its inflow less surface evaporation, and is flagged `endorheic` when evaporation takes all of it.
     - **Discharge** accumulates runoff P − ET (Fu's curve, L1 climate).
     - **River cells** need ≥ `river_min_discharge_m3s` (2) and ≥ `river_min_area_km2` (50). They cross lake margins less than a block deep, and the beach.
     - **Chains** are cut at confluences; a chain follows its largest donor.
     - **Geometry.** The water surface is F − `freeboard_blocks` (1), never below the sea or lake downstream. Width is `width_coef`·Q^0.5·`scale.feature_scale.rivers` (1 block per √(m³/s)). Depth is `depth_coef`·Q^0.4 (0.5), at least 1 block. They are capped at 64 and 12. Vertices are Gaussian-smoothed (half-width 8 blocks), with junctions pinned.
     - **It writes** `water_level` (lake surfaces), **`rivers/rivers.dtrv`** and `notes.bake.water`. The `.dtrv` file is a new `DTRV` sectioned file: vertices x, z, w, hw, depth, Q; chains ending in sea, lake or river; a 32-block tile index.
     - `hydro.enabled = false` writes an all-nodata `water_level` and no rivers file.
     - **Format.** `rivers.dtrv` is part of **format 0.2**, which is unreleased. FORMAT.md's 0.2 note now lists three changes: continuous strata, the monthly climate and the rivers file. The file is optional: a folder without it has no rivers.
  2. **The column rule both sides implement** (`deeptime_format::water`, Java `worlddata/Water`; FORMAT.md "Water columns")
     - The generated surface is H bilinear over the 4-block noise-cell corners.
     - The kinds, in order of priority:
       - lake (P0-13's shore rule);
       - sea (ground under the sea's top block);
       - river: a channel covers the column; top ⌈w⌉ − 1, parabolic bed, at least 1 block of water;
       - bank: a 4-neighbour holds water. Ground reaches at least its top (+1 beside a river). Air is sealed from its bed up (only the waterline beside the sea). Within a channel's bank zone, at most a 1:1 slope.
     - The column JSON gains `hydro`.
     - `deeptime parity` adds columns: 10 % on and beside river vertices, 5 % around lake mouths, and 5 % around lake shore cells (wet beside dry).
  3. **`worldgen/WaterBodies`** replaces the lake-only pass (`WaterPass` delegates)
     - Rivers are cut or filled to their bed (gravel under 6 blocks wide, sand wider), with water to their top.
     - Lakes are filled as in P0-13.
     - Sea columns the aquifer left dry are topped up. The aquifer leaves coastal flats a block below sea level dry, and river mouths leaked into them.
     - Banks are raised or lowered, keeping their top block, and sealed.
     - A one-block step in a river gets vanilla's resting flowing block; a larger step gets a falling curtain (the cheap waterfalls).
     - **Caves may cut under the water** (Ben, log 2026-09-30, "Allow caves under water"). The carving mask that kept carvers off water and banks in the first build is gone.
     - It runs only in chunks near a river segment, a wet `water_level` cell, or elevation within 17 blocks of sea level, and logs its own cost every 512 chunks.
  4. **River biomes.** `WorldDataTerrain.raster(ridges)` returns 0 (the valley slice) under channels at least `biome_min_width_blocks` (6) wide.
  5. **Settings** (standing rule)
     - Preset section `hydro` (14 keys) in the four pipeline presets and the built-ins.
     - `scale.feature_scale.rivers` is wired (no longer "planned").
     - Editor: a "Rivers and lakes" toggle (terrain), the river multiplier, and section "Advanced · Rivers and lakes" (lang keys, `EditorFields.Section.ADV_WATER`; workbench and JUnit catalogue tests updated).
  6. **Tools**
     - `deeptime hydro --world-data D [--set hydro.K=V]` re-runs the water part on a baked folder.
     - `deeptime-inspect rivers` draws planet and box maps; `--columns` shows the column rule per block.
     - `deeptime-inspect water-check` counts water that floats (air under it) and leaks (air beside it over ground), resting steps, and lakes against their lowest rim, split by the column's kind. Water over air under river, lake or sea water, and leaks into covered air or into ground the pass had left, are counted apart as caves.
  7. **Tests**
     - Core `hydro::tests`, 6 tests:
       - every river descends to the sea or a lake;
       - every lake is full to its lowest rim (exactly its spill height);
       - 1 thread = 3 threads, and run = run byte for byte;
       - a river crosses the seam;
       - geometry follows Q;
       - a dry climate leaves a lake without an outlet.
     - Format: `rivers` 4 tests, `water` 3 tests.
     - JUnit:
       - `ColumnParityTest` compares hydro, on planet4k and on the lake world lake1024 (which must have lake columns);
       - `WaterRegionTest`: the per-chunk path equals the per-column path.
     - GameTests:
       - `RiverGameTests.riversHold`, on level `deeptime:test_rivers` (the 4k planet with a bare biome, no carvers, features or caves);
       - `riverBiomes`.
- **Real planet.** `earthlike_quick_16k`: scattered continents, 500 Myr, format 0.2, on the branch after the master merge of 2026-09-30. `deeptime hydro` was re-run on its bake after the flats and edge-outlet commits.
  - **Lakes:** 27, covering 68,139 km². The largest is 16,068 km²; 6 are endorheic.
  - **Rivers:** 4,489 chains, 190,265 vertices, 119,813 km. Largest discharge 1,345 m³/s; widest channel 36.7 blocks. `rivers.dtrv` is 6.6 MB.
  - **Time:** the bake's water part took 4.9 s (flood 1.8 s, discharge 0.7 s, flats 0.2 s) of a 30 s bake.
  - **For comparison:** the first build on master e5d98e3's terrain gave 26 lakes and 3,337 chains.
  - **Maps:** `deeptime-inspect rivers`.
- **3D previews and water checks.**
  - **Previews:** `preview.py run --keep-world --y 0,260`, standalone, on the final bake. Three 256 × 256 boxes:
    - the largest river's mouth (−6300, 2130);
    - mountain streams (−700, −1950);
    - streams entering a large lake (2200, −1130).
  - **Checks:** `deeptime-inspect water-check --world-data` on each box.
  - **Results:**
    - 0 floating water blocks in all three.
    - 0 leaks in the river-mouth and lake boxes.
    - 1 leak in the mountain box: a flowing step block on a mossy boulder that vanilla's `forest_rock` dropped into the channel.
    - Caves under or beside water: 357 floating / 435 leaks at the river mouth and 221 / 373 at the lake. The same river-mouth box without rivers has 231 / 201.
    - Resting steps: 79, 2 and 22.
  - **Whole lake.** A fourth box, 300 × 260 at (1936, −1546), holds one complete lake of 40,950 columns (952,927 water blocks).
    - 0 floating, 0 leaks.
    - Surface y 80; lowest rim top 79, at the outlet. **Rim − surface = 0.**
    - Vanilla's carvers also replace water: `overworld_carver_replaceables` lists `minecraft:water`. So with the carving mask gone, they cut air pockets through the lake itself, not only beneath it. In this box: 34 lake columns with no water left at the top, 1 shore column trenched, and 1,959 / 3,844 cave floating / leaks.
    - water-check now reports those as `drained_columns` and `carved_rim_columns` instead of taking them as the rim. Counted as rim, the trench reads 5 blocks under the surface.
  - **Iterations.** Seven, each fixing what it found:
    - rivers ended short of lakes;
    - river mouths stopped at beach flats;
    - carvers cut beds (a carving mask, since removed by Ben's decision);
    - step sources spread over ice and boulders;
    - coastal flats left dry;
    - parallel rays across flats;
    - rivers along the polar edge.
- **Chunk time.** Measured with Chunky over the same three 361-chunk boxes: same bake with and without rivers, same jar, Mac at load 30–100 from other agents.
  - Before (no rivers): 21.5, 21.4 and 14.3 s, so 40–60 ms a chunk.
  - After (rivers): 20.4, 20.3 and 19.4 s, so 54–57 ms a chunk. The difference is within the load noise.
  - The pass's own logged cost is 0.27 ms a chunk on average and 0.32 ms where it works (1,298 of 1,536 chunks). Without rivers it still tops up coasts in 692 chunks at 0.29 ms.
  - `WaterRegionTest`'s per-column path costs 0.69 ms a chunk where rivers are (JIT warm).
- **Gates on the last merge** (master 5d4147b: volcanoes, glacial landforms, the climateConsistent fix, EF ice cover). Branch agents no longer run GameTests (Ben, 2026-09-30, through the coordinator), so this merge got cargo and Gradle only:
  - cargo fmt, clippy `-D warnings` and deny are clean; `cargo test --workspace --no-fail-fast` all green (core 323 + 3 ignored).
  - `./gradlew build test`: 157/0. Parity: 500 of 500 (23 river, 3 lake and 25 bank columns: the volcanic and glacial terrain gives the 4k test planet lakes); lake1024 300 of 300.
  - Two fixes in the merge:
    - the editor catalogue test now expects 8 advanced sections (Rivers and lakes plus Volcanoes);
    - a FORMAT.md layer-table hunk from the format-0.2 merge be67401 had been committed with its conflict markers; it now holds this branch's `water_level` row and master's `strata/horizons` row.
- **Gates on the earlier merges** (master 0905793, 6f52b57 and 18706c7, the same results each time):
  - `cargo fmt`, clippy `-D warnings` and deny (bans, licenses) are clean. `cargo test --workspace` all green (core 294 + 3 ignored).
  - `./gradlew build test`: 153/0. Parity: 500 of 500 equal (19 river and 14 bank columns); the lake world lake1024 300 of 300 (8 lake, 1 bank).
  - `./gametest.sh` (`--server`, three times): **53 of 54 pass**.
    - `riversHold`: 3 sites, 40 chunks, 368 water columns, 946 of 946 blocks; 0 floating, 0 leaks, 0 top mismatches, 0 water outside.
    - `riverBiomes`: 101 of 107 = 0.944.
    - `lakeHolds` is unchanged.
    - The one failure is `climateConsistent`: BW 0.472 < 0.650, Dsub 0.822 < 0.850, BW:desert 0.423 < 0.450. It comes from master's carbon-cycle merge 27e88f7, not this branch. The glacial-erosion branch fails with exactly the same numbers without any P1-22 code, and the census samples only the biome at the terrain height.
  - `./gametest.sh -PwithMic` (after `install.sh mic`, 160 mods linked):
    - One run (on 6f52b57) never reached the tests: a mod-loading race in Artifacts ("unbound value … artifacts:generic.attack_burning_duration" while registries reverted to vanilla), outside Deep Time.
    - The retry and the run on 0905793: **53 of 54 pass**. `riversHold` and `riverBiomes` give the same numbers as standalone.
    - The failure is again `climateConsistent`, with MIC's biomes: BW 0.485, BS 0.509, BW:desert 0.446, Dfc:taiga 0.430. The carbon-cycle branch shows the same kind of BS and Dfc:taiga failure.
  - `./gametest-planet.sh`: PASS, twice (wrap-test planet, TW stamped, loopsX).
- **Review bundle.** review/out/p1-22 (index.html, maps, 3D shots, the area3d viewer with the four boxes, water-check JSONs), 16 MB. Not published.
- **For P2-03:**
  - evaporation-limited lake levels below the rim (Fill–Spill–Merge's partial fills);
  - HAND floodplains;
  - deltas;
  - meanders beyond the smoothing;
  - knickpoint waterfalls from erodibility;
  - incision burned into `elevation`.
- **For Ben:**
  1. Defaults to judge from the pictures:
     - a stream starts at 2 m³/s and 50 km² (4,489 rivers on a 16k planet);
     - 1 block of width per √(m³/s);
     - river biomes from 6 blocks wide.
  2. **Caves under water.** Vanilla's carver tag includes water, so with no carving mask the carvers cut through the water itself: air pockets inside lakes and rivers, and now and then a hole in a lake's surface. The game refills these with flowing water. This goes beyond cutting beneath the water. The middle option is to mask only the water blocks, so carvers still cut the beds and banks under the water but skip the water. It is a small change in `WaterBodies` (the removed mask, limited to the water); it waits for your call.
  3. A polar river near z ≈ −4020 lies inside the unplayable pole cap (cosmetic).
  4. Straight parallel streams on even slopes (D8 on a square grid) outside flats are left for P1-23.
### 2026-10-01 — Amplify's `auto` decides on the whole path's full peak, so a long run no longer fails at the 4 GiB cap (agent worktree agent-af5b7754a329c4418, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Bug.** `deeptime run --preset earthlike_quick_16k --start supercontinent --length-myr 1000` failed in amplify at the
  default 4 GiB cap ("needed … bytes"); `--set amplify.stream=always` finished. Reproduced on the Mac before the fix
  (`stage amplify` log: "1565 MiB in use of the 4096 MiB cap; the last level of 8192 x 4096 cells needs 2528 MiB whole
  and runs in memory", then `needed 4449249634 bytes, cap 4294967296 bytes`; the first report had 4,451,574,045).
- **Cause.** `AmplifyStage::advance` asked `ctx.reserve` for the last level's buffers alone (8192 x 4096 x 79 B =
  2,650,800,128 B) when amplify starts, with the L2 level, 2,048 x 1,024 cells, as the only level held. By the time the
  last level reserves those buffers (`run_tiles`) the ladder holds the 4096 x 2048 level (176 MB) in place of L2 (44 MB)
  and the preview arrays (25 MB, three f32 a cell on 2048 x 1024) are allocated: 157 MB more than at the decision.
  `reserve` is a check against the gauge, not a hold, so nothing kept that room. The 1000 Myr history (100 frames,
  783 MiB) is in memory at that point, `reserve` spilled 724 MiB of it so the buffers fit, with about 3 MB to spare, and
  the later reservation went over. (With every frame spilled the full path is 4,185 MiB there, so memory was never
  possible; streaming was.)
- **Fix** (`surface/amplify.rs`). `whole_last_level_peak(l2_bytes, doublings, gw, gh, preview_width)` = the last level's
  buffers + (the level before it − L2, when there are levels before the last) + the preview, and
  `streams(mode, peak_bytes, reserve)` is the one decision: `always` true, `never` false, `auto` when `reserve(peak)` is
  refused (so the history spill still makes room first when that is enough). `advance` asks it with the actual
  `Level::bytes()` (new, `resample.rs`) and the preview width; the levels before the last use it with their own
  `whole_bytes` as before (their decision is made with the gauge as it is when they start, so it was already complete).
  The log line now also prints the full peak. `always` and `never` are unchanged.
- **Estimate left as it was.** `amplify::estimate` still picks whole or streamed from the buffers alone under the default
  cap. Adding the kept level and preview to it (+157 MB at 16k) pushed `pipeline_quick_16k`'s estimate over the
  workbench's cap-less-512-MiB limit and broke four `deeptime-cli` tests (`a_run_over_its_memory_cap…`,
  `a_tuned_code_from_the_game…`, `edits_land_where_the_core_reads_them`, `workbench_queue_editor_and_kept_runs`), because
  `Stage::estimate` has no cap parameter and the workbench's lower cap is not seen by the pick. Not changed; a
  follow-up is to give `Stage::estimate` the run's cap (comment at the pick in `estimate`). Consequence now: a last
  level the estimate lists whole may run in tiles (it overstates), and one that does run whole needs ~0.15 GiB (16k)
  more than listed.
- **Tests** (`surface/amplify/tests.rs`). `auto_streams_when_the_whole_path_peak_would_not_fit_though_the_last_level_alone_does`
  (the 16k numbers: held 1,643,487,517 B, the last level alone fits, the full peak needs the reported
  4,451,574,045 and does not; `auto` streams, `never` does not, `always` does; every cap from "last level alone" to one
  byte under the peak streams, the peak exactly keeps memory); `the_whole_path_peak_is_what_the_last_reservation_sees`
  (runs the ladder in memory with a recording reserve and checks the formula against the last request, the level
  the run kept and the preview it made); `the_stage_streams_under_auto_when_only_the_whole_path_peak_is_over_the_cap`
  (the small coupled pipeline, two ×2 levels, a gauge holding a fixed amount: a cap 1 MB over the buffers alone gives
  `mode: streamed`, a roomy cap gives `in_memory`).
- **Measured** (Mac, release `deeptime run --preset earthlike_quick_16k --start supercontinent --length-myr N`, default
  4 GiB cap, loaded Mac, load ~12):

  | Run | Amplify decision | Amplify | Run peak (counting allocator) | Total |
  |---|---|---|---|---|
  | 1000 Myr, before | in memory, then failed (`needed 4449249634`) | failed | — | 481 s |
  | 1000 Myr, after (two runs) | 2289 MiB in use, full peak 2678 MiB: tiles (`mode=streamed`, 8 tiles) | 56.7 s, then 26.6 s | 3089.3 MiB | 453 s, 376 s |
  | 500 Myr, after | 1412 MiB in use (history spilled 249 MiB for the reservation): in memory (`mode=in_memory`) | 23.1 s | 3802.5 MiB | 239 s |

  The 1000 Myr run keeps its whole history in memory now (the old rule spilled 724 MiB of it for nothing). The
  session's `reserved_peak_mib` (4967) still counts the refused probe of the full peak; it is a probe, not an allocation.
- **Gates** (Mac): `cargo test --workspace --locked --no-fail-fast` all ok (core 320 + 3 ignored, cli 30, format 44,
  inspect 29 + 2 + 7, jni 7, viewer 6); `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all
  -- --check`, `cargo deny check licenses bans` clean; `cd neoforge && ./gradlew build test` BUILD SUCCESSFUL, 155 tests,
  0 failures. No GameTests (per the batch rule). Mac scratch (`deeptime-runs/amplify-cap`) deleted.
- **Next.** The coordinator merges with the batch and runs `mac-gc.sh`. Optional follow-up: cap-aware `Stage::estimate`.

### 2026-09-30 — Satellite view: a true-colour keyframe layer, the default everywhere (agent worktree agent-a7ca2a0e875263a86, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **What.** Ben: "Default view on the viewer is currently an elevation map, can you turn it into an actual map. (I.E.
  Approximately what you would see on google earth zoomed out all the way / satellite imagery.) So I can see where
  glaciars / snow is". A derived, view-only keyframe layer `satellite` (`core::satellite`), composited when a frame is
  drawn from fields the stages already wrote, like `provinces`: nothing goes into `WorldState`. `keyframes::add_derived`
  puts it **first** in every frame of the plate model's group (`elevation` + `regime`, with or without a climate) and in
  the young landscape's and amplification's frames (drawn on `young/elevation`, `amplify/elevation` box-averaged to the
  frame); a dummy stage's `elevation` and the deposit frame get none. `keyframes::render` takes a `ViewScale`
  (`Preset::view_scale`: `sea_level_y`, `m_per_block_v`, `model_radius_km`) to turn block y into metres. Also:
  `deeptime satellite --in <checkpoint> --out x.png [--width W]` (for tuning without a rerun); `climate-dem --viewer`
  now draws the satellite view of the relief it is given (Earth's, when there is a DEM).
- **Colour rules** (the module docs have them in full; constants `satellite::Look`). The anchor colours are the 18
  classes of the new `satellite` ramp in `ramps.json` (labels + `groups` Land / Snow and ice / Sea), so the legend is a
  key to exactly the colours drawn. Sea by depth: turquoise under 8 m, shelf blue at 120 m, ocean at 1.5 km, deep navy
  from 5 km; annual `climate/sea_ice` whitens it (fully from ~90 %). Land with a climate: herb cover from the aridity
  index (0.05 → 0.45) and tree cover (0.5 → 1.0), herbs needing a warmest month over ~0 °C and trees over ~10 °C; the
  Köppen class holds the picture on its map (BW ≤ 15 % herbs, no trees; BS 35–75 % herbs, < 8 % trees; Af/Am forest;
  Aw/As 20–55 % trees; ET/EF no trees). Ground: sand where hot, grey-tan where cold, 45 % tinted by the surface rock
  (`surface/class`: sandstone pale, red beds reddish, basalt dark, evaporites salt-white). Herbs: khaki steppe → olive
  savanna with moisture, brown-grey tundra under cold summers; forest: boreal / temperate / rainforest by the coldest
  month. Bare grey-brown rock from 2.5 km up. Snow: perennial (85 %) where the warmest month is below freezing, above
  the snow line (`climate/ela_m`) or EF; seasonal tint up to 35 % as the coldest month goes −2 → −22 °C (half under
  forest). Glacier ice (`climate/ice_thickness_m` ≥ 1 m, else `climate/glacier`) white. Hill shade: NW light at 45°,
  heights ×14 on the model sphere, 60 % strength on land, 45 % on ice, 35 % on the sea floor, none under sea ice.
  Smoothing (found in the first pictures): the energy-balance model's 2° bands drew ruled lines in the sea-ice edge and
  the snow, the orographic rain's cell streaks drew stripes in the Sahel-like belts, Köppen borders drew steps and the
  per-cell rock tint speckled deserts; so temperatures are box-filtered over ±3 rows, moisture over 3 × 3 (geometric),
  sea ice over 5 × 17 sea cells, and the herb/tree cover and rock colour over 3 × 3 land cells before compositing.
- **Frames without a climate** (plate-only presets, pipelines without the climate stage): bare ground by geology —
  the surface rock's colour (`surface/class`, else `structure/rock`), else desert tan in the lowlands turning to grey-brown
  rock with height, darker on hotspots; no plants, snow or sea ice (nothing says where they would be). In an
  earthlike run every group frame has a climate (the climate steps before the first frame).
- **Storage and the game.** No new layer kind: keyframe layers are RGBA8 already, so FORMAT.md's Keyframes section only
  documents the two derived layers (the satellite ramp is a legend, not applied). The in-game timelapse builds its
  buttons from the frames' layer names and takes the first as its default, so it opens on the satellite view with no
  Java change; `deeptime.layer.satellite` = "Satellite" added to en_us.json. The review page's final map is still the
  world data's elevation (not changed).
- **Default.** Workbench: `DEFAULT_LAYERS = ['satellite', 'elevation']` (a `layer=` in the URL hash still wins); the Planet
  group lists Satellite, then Elevation (keys 1, 2); the legend shows a key (swatches in the ramp's groups, wide columns
  for long labels — provinces' labels benefit too); the pointer readout says "looks like temperate forest". `?tv`: the
  same default, a three-column key without headings. Static bundle: the timelapse lists every layer any frame names and
  opens on `satellite`; the world-data map gets a `satellite` image layer first (the last satellite keyframe, drawn on a
  keyframe grid r = C/w; no values, no water overlay) with a swatch legend.
- **Measured** (Mac, release). One 1024 × 512 frame < 0.3 s (the CLI from a checkpoint, reading included). Size: the
  satellite layer is 0.58–1.17 MB LZ4 per frame (+16 % on a group frame: 7.0 → 8.2 MB at the present); the default run's
  keyframes 68 → 80 MB; a workbench run with a frame every step (250) grows ~250 MB.
  **World files vs master** (`run --preset earthlike_quick_16k --start scattered_continents --length-myr 500 --threads 3`,
  master 18706c7 binary against this branch): `diff -rq` differs only in `keyframes/index.json`, 13 of 14 `kf-*.dtk` (the
  deposit frame is identical) and `manifest.json` (created_utc, stage timings, the keyframe files' bytes/crc and
  `reserved_peak_bytes`/`reserved_base_peak_bytes` +25.2 MB: the plates estimate counts one more keyframe layer).
- **Screenshots** (headless Chromium on the Mac — Playwright's chrome-headless-shell through a `chromium` wrapper on
  PATH for `tools/viewer/shot.py`; private `deeptime watch --port 8653` with its own runs folder on the SSD; 8642 untouched),
  1600 × 900 in `review/out/sat/shots/`: `wb-present.png` and `wb-present-clean.png` (events off) — earthlike_quick_16k
  scattered, 500 Myr, the present; `wb-mid.png` — 268 Myr ago; `wb-snowball.png`, `wb-snowball-clean.png` — the snowball
  scenario (supercontinent, `degassing_scale` 0.3, 1,000 Myr; `deeptime:snowball` −807.3 → −760.6 Myr) at 784 Myr ago,
  the whole planet white; `wb-tv.png` — `?tv`; `static-viewer.png` — the run's static bundle opening on the satellite
  world map. Raw frames in `review/out/sat/frames/` (present, 244 and 498 Myr ago; 840, 784, 720 Myr ago in the snowball
  run); `ck-present2.png`, `crop3x2.png` from `deeptime satellite`. Looked at: glaciers and ice caps stand out white on
  mountains and poles, sea ice fades from the poles, deserts are tan in continental interiors and rain shadows, forests
  green on wet margins, tundra brown-grey at high latitudes, shelves lighter than the deep sea with ridges visible.
- **Gates.** `cargo test --workspace` ok (core 287 + 3 ignored, incl. satellite:: 6 — synthetic frame: deep ocean dark
  navy, a shallow bank turquoise, hot desert tan, rainforest deep green, glacier and frozen sea white, steppe khaki,
  tundra brown-grey; snowball all white; no-climate frame bare rock and ice-free; ramp labels = classes; hill shade lit
  NW; box filter bands and mask — keyframes:: the satellite leads terrain frames only, viewer: the last satellite frame
  leads the world layers); clippy `--workspace --all-targets -D warnings` ok; `cargo fmt --check` ok; `cargo deny check
  licenses bans` ok; `./gradlew build test` BUILD SUCCESSFUL, tests=151 failures=0 (the lang string; `ScreenSourcesTest` reads en_us.json). All of them re-run after merging master 52b6626 (the climate API's latitude; coal in the sedimentary cover and the strata settings): all pass (core 288 + 3 ignored; Gradle tests=151 failures=0).
- **Next.** Merge when reviewed (the caller lands it). Not done: the in-game review page's final map could show the last
  satellite keyframe instead of the elevation ramp (a ReviewModel change); the workbench readout could give the
  elevation under the pointer on the satellite layer too.

### 2026-09-30 — Coal held in the sedimentary cover; rock-layer settings with a flatter default; readable deposit-world strata (agent worktree agent-aacd864d2709d52dd, branch strata-coal-settings, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Why.** Decisions log 2026-09-30: "Cap coal to the sediment (Recommended)" and "Settings + flatter default
  (Recommended)", plus readable strata for the synthetic deposit test world.
- **Coal capped to the cover.** The deposit stage bakes the world's horizon stacks the way the bake will
  (`structure::Cover`: `bake_horizons::bake`, smoothed, before the top raise) and reads the sedimentary cover at a
  site off them with the column rule (`blocks_at`: down to the first horizon of a non-sedimentary class ≥ 3 blocks
  thick; sedimentary = chemical/clastic sedimentary and unconsolidated families). `geometry::fit_stack` keeps the
  preservation ledger's depth for the top seam and caps the stack from below: seams that would lie under the cover
  (within one block of its base) are not there — thin cover leaves fewer, shallower seams (`seam_plan` on the capped
  measures, same random stream); a top seam under the cover's base or < 2 blocks of cover leaves no coal (census
  `no_cover`, `cover_capped`). The guaranteed coal site follows the same fit and keeps one seam at the surface where
  there is no cover. A first version raised deep stacks into the cover instead; that turned `too_deep` sites into
  counted ones and overshot the USGS targets by half (counted 101 against 68.6 on the six landscapes), so the cap
  cuts from below. Unit test `a_stack_is_held_in_its_sedimentary_cover`.
- **Settings** (preset `structure`, `editor_fields.json` under "Advanced · Granite and rock layers", lang keys, the
  pipeline presets and the built-ins synced with `tools/dev/sync_builtin_presets.py`): `strata_smoothing_cells`
  (σ, 0–16, **default 10**, stops Cell by cell 0 / Wavy 2.5 / Rolling 6 / Gentle 10 / Flat 16),
  `strata_tongue_scale` (0.25–16 × the 64-key band) and `strata_minor_share` (0–0.45, default 0.15). The two tongue
  settings go into the horizons layer's descriptor (`tongue_band_keys`, `tongue_minor_share`, optional, defaults
  64 / 0.15) and both column rules (Rust `strata::stack_at`, Java `Columns.Stack`, `LayerDesc`) read them. The
  smoothing samples an 11 × 11 lattice of step ⌈σ / 2.5⌉ anchored to the grid, so any σ costs the same.
  `notes.bake.strata.smoothing` records σ, step, window and tongues. FORMAT.md updated (it still said 256 keys per
  band from before; the rule is written with B and m now).
- **Default σ from dips.** `dips.py` (Mac): 1,500 random columns of `earthlike_quick_16k` (scattered, 500 Myr, seed 1),
  horizons matched by (unit key, class) 16 blocks east and south, beds ≥ 6 blocks thick below the ground, slope in
  block space (the world's vertical exaggeration is 15×: 20 m per block up, 300 m across, so a real dip is
  atan(tan(block dip) / 15)):

  | σ (cells) | median dip, all | lowland (ground 23–103) | sea floor | upland | real-world median, lowland |
  |---|---|---|---|---|---|
  | 2.5 (master) | 38.3° | 43.2° | 28.3° | 44.4° | 3.6° |
  | 6 | 23.8° | 26.0° | 17.4° | 30.0° | 1.9° |
  | **10** | **15.6°** | **18.6°** | **12.7°** | **20.6°** | **1.3°** |

  σ 10 reads as undisturbed basin fill in the sections (gently tilted beds, onlap onto basement highs, the coal
  stack lying in shale), σ 6 still arches the layers under mountains. Strata bake 7.7 s at σ 10 (mean horizons per
  cell 5.97 → 8.59).
- **Basin strata** (`synth::Strata::Basin`, CLI `--strata basin`): sandstone, shale, sandstone, mudstone, limestone,
  shale, sandstone, shale beds 0/8/22/36/55/75/100/130 blocks down, granite from 165 blocks, the same in every cell;
  `deposits_world_16k` uses it (`deposits_world_4k` stays placeholder; the every-class pattern stays only in the
  GameTest world `gametest.sh` writes).
- **Coal measured** (Mac). Host rock at coal-body centres by the column rule (`hosts.py` over `deeptime
  inspect-column`, the planet above, master vs branch): **sedimentary 60.5 % → 97.2 %** (152 → 72 seams: shale 41,
  sandstone 28, slate 1, schist 1, black shale 1; before: sandstone 56, shale 35, gneiss 14, basalt 13, slate 13,
  granodiorite 6). The six landscapes (`stage deposits --in` the `stage structure --preset pipeline_quick_16k --start
  {scattered_continents,supercontinent} --length-myr 500 --set seed={1,2,3}` checkpoints, master's and the branch's
  binary):

  | coal | master | branch |
  |---|---|---|
  | sites, median (range) | 16.5 (5–25) | **9.5 (3–19)** |
  | counted, six summed (target 68.6) | 58 | **50** |
  | seams, six summed (per site) | 1,013 (10.6) | **364 (5.9)** |
  | seams_max per landscape | 17–37 | 8–22 |
  | no_cover / cover_capped | — | 56 / 29 |
  | too_deep | 22 | 0 |

- **Before/after** (`tools/workbench/preview.py run --keep-world`, standalone, master's jar on master's bake and the
  branch's on its own, boxes 1800,−1260 (coal basin), −480,−1968 (a 430-block mountain), −1087,1754 (a basin coal
  field); `deeptime-inspect section` along x and z; `wd-section` wider): before: layers folded in waves one to three cells long under the basin and arched under the mountain, the coal stack cutting across them, coal-field seams in gneiss and basalt in places; after: gently tilted basin fill, onlap onto basement highs, the coal stacks inside the shale and sandstone. Painted share: master
  98.16 / 99.17 / 98.92 % (93,601,451 of 94,746,729, **98.79 %**), branch 98.21 / 99.10 / 98.62 % (91,617,663 of 92,822,166, **98.70 %**; the third box's "other" rock, surface-rule and feature blocks, rose from 286,611 to 371,492). Chunky (loaded Mac, not comparable run to run): master 39/48/42 ms/chunk, branch 51/73/73 ms/chunk.
  Column-rule benchmark 0.34 ms/chunk (`ColumnRuleBenchTest`, mean 8.4 horizons per column). Review bundle (not
  published): `review/out/strata-coal-settings/`.
- **Gates** (Mac, the branch after merging master 18706c7): `cargo fmt --check`, `clippy --workspace --all-targets --locked
  -D warnings`, `deny check licenses bans` clean; `cargo test --workspace --locked` all pass (cli 30, core 281 + 3
  ignored, format 44, inspect 29 + 2 + 7, jni 7, viewer 5, …); `cd neoforge && ./gradlew build test` → tests=151
  failures=0 (`ColumnParityTest` 500 columns, now with the layer's tongue parameters); `./gametest.sh` → 49 of 50
  (`climateConsistent` fails: Köppen BW 0.472 < 0.650, Dsub 0.822 < 0.850, BW:desert 0.423 < 0.450 — **the same
  numbers on master's own code** (control run in a copy of master 51c6c18), its floors predate the climate merges
  that gave the test planet 466 BW columns instead of 17); `tools/smoke/install.sh mic` + `./gametest.sh -PwithMic` →
  52 result lines, 1 not PASS (the same `climateConsistent`, BW 0.485, BS 0.513, BW:desert 0.446, Dfc:taiga
  0.430).
- **Decisions for Ben** (not asked): (1) With the cap, coal counted survivors fall to 73 % of the USGS target (50 of
  68.6 over six landscapes; master 85 %): favourability does not know the cover, so fields drawn on thin cover are
  dropped (`no_cover` 56). Folding the cover into coal's placement (a host term) would bring the count back.
  (2) σ 10 is a look choice; the real-world dips of every σ are already a few degrees because the world's vertical
  scale is 15× exaggerated. (3) Seams now sit in the drawn rock, so a player following a seam stays in shale or
  sandstone.
- **Next.** Merge into master (the parent); then, if Ben wants it, the coal cover in placement.
### 2026-09-30 — Ice cover over glaciated ground; no chasm on barren EF; ice caps off cherry grove (agent worktree agent-adda9513719f24370, branch `ef-ice-cover`, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Why.** Decisions log 2026-09-30: "Force snow/ice cover (Recommended)" (Terralith has no flat inland ice biome, so 31 % of glaciated EF
  had come out as gravel desert) and "Fix both (Recommended)" (glacial chasm only on glaciated ground; ice caps steered off snowy cherry
  grove; via the placement file, pack-overridable).
- **Ice cover.** The bake writes **`climate/ice_cover`** (L2, u8, blocks; `bake/ice.rs`): for ground above the sea and above its equilibrium
  line (the climate stage's `climate/anomaly/ela_m`, L1 every cell, resampled to L2; the young landscape's elevation), `1 + ⌊height above
  the line / step_m⌋` blocks up to `max_blocks`. The table's new **`ice_cover`** section (`placement.json`; defaults: step 160 m, at
  most 12 blocks, 3 snow blocks on top, blue ice for the last 2 of a cover of 9 or more, blocks `snow_block` / `packed_ice` / `blue_ice`)
  is validated by `bake/placement.rs` and recorded in `notes.bake.climate.ice_cover`. Java: `worlddata/IceCover` (the small interface:
  `blocks(x, z)` and the block mix `Spec.kind(index, total)`; `WorldData.iceCover()`) and `worldgen/IcePass`, called last in
  `GeologicChunkGenerator.buildSurface` (after the pack's surface rules, the strata pass and the water pass): the cover **replaces** the
  top `n` solid blocks of the ground (the terrain's height does not change), reading the layer bilinearly and rounding, so a cover's
  edge is a slope of cells, not 8-block steps; fluid-topped columns, air and bedrock are left alone. It reads the climate only through
  `IceCover`: a climate stage that carries a real ice thickness (`climate/ice_thickness_m`) only has to be turned into the layer by the
  bake. Not in the column evaluator (Java alone reads the layer; `inspect-column` prints it with the other layers), so no Rust/Java
  parity obligation arises; FORMAT.md has "The ice cover" and the layer table row.
- **Fix both, through the placement file** (schema additions, no version bump: all optional): a class may carry **`erosion: {min, max}`**
  (clamp, vanilla's units; a `glaciated` block may give its own) and **`weirdness`** as `"positive"` or **`{"positive_below_erosion": e}`**
  (positive branch below `e`, negative above). EF: `erosion.max` 0.3 (barren), `glaciated.erosion.max` 1.0 (ice cap keeps the chasm),
  `weirdness.positive_below_erosion` −0.27. Why these (evidence, offline on the planet's glaciated and unglaciated EF cells with
  Terralith's parameter list under MIC's mapping and under the identity mapping): barren EF at erosion ≥ 0.4 gets `glacial_chasm`
  (21 % in MIC) and holding it below 0.3 gives 0 (gravel desert 53 → 74 %, cold shrubland 16 %); the cherry grove that stayed on ice caps
  after the first change (2 %) is the positive-branch variant at erosion −0.22 … 0.05 and weirdness ≥ 0.4, mid-inland, which no window on
  erosion or weirdness in vanilla units removes (MIC's router squashes both), but negative-branch flats cannot have it; the peaks stay
  positive so scarlet mountains stay off them, and the threshold −0.27 (not vanilla's −0.375) is where MIC's stretched erosion leaves no
  negative cell in scarlet's band (at −0.375 the box kept 11 % scarlet; −0.27 gives 0). Planet-wide glaciated EF, emulated: MIC ice 40 /
  snow 56 / barren 4 %, scarlet 0.2 %, cherry 0; identity mapping ice 48 / snow 36–47 / barren 6–12 %, cherry ≤ 0.6 %. Barren EF: cold
  shrubland 79–81 %, snowy plains 9–13 %, glacial chasm 1 %. Side effect (undone below): with all the flats on the negative branch vanilla's
  ice spikes (a positive-branch variant) disappeared from ice caps (Terralith's stay).
- **Tests.** Rust (`bake::placement`, `bake::biomes`, `bake::ice`, `bake::tests`): erosion limits and their errors, the weirdness forms,
  the ice-cover section (defaults, bounds, cover thickness), the cover layer from an ELA, EF's peaks positive / flats negative / barren held
  off the chasm. Java: `IceCoverTest` (record, block mix, the baked 4k planet's layer read back), GameTest `icePassLaysTheCover` (a chunk
  built for it: covers of 0, 1, 5 and 12 blocks, a lake column, a two-block column), `iceCoverLaidOnGlaciatedGround` (real planet,
  if it has cover; the 150 Myr 4k planet has none above its line), `BiomeCensus.FORBIDDEN` gains `EF:no_chasm_on_barren` and
  `EF:no_cherry` for `climateConsistent`.
- **Ice spikes back (Ben, 2026-09-30, "Bring ice spikes back").** The flats' branch is now `weirdness.flats`: `{positive_noise_above,
  max_magnitude}` (new, optional; default never positive and no cap, which is what the entry above did). EF: the flats are positive
  where the variant noise is above 0.2 (about a fifth of them), at |weirdness| at most 0.25 (vanilla's units), the rest negative;
  peaks positive. Vanilla's ice spikes (a positive-branch variant at humidity band 0) come back on a share of the flats, and the cherry
  groves (positive branch, |weirdness| 0.4 and up at mid erosion, where MIC's router puts reference 0.3 … 0.57) stay off: without the
  cap a half of the flats positive gave cherry on 0.9 % of the ice cap, with it 0.1 % (emulated, MIC). The bake carries the two numbers
  per class as nearest-cell L2 layers like the rest of the class rules. Tests: `bake::placement` (forms and errors),
  `bake::biomes::ef_flats_let_some_noise_through_below_a_magnitude` (share between 2 % and 60 %, |w| at most the cap, noise above 0 gives
  more, above 1 none, peaks stay positive).
- **Ice cover from the climate's ice** (the climate agent's `climate/ice_thickness_m`, merged with master): `bake/ice.rs`
  `cover_from_thickness`: ice of at least 1 m on ground above the sea gives `1 + ⌊t / step_m⌋` blocks (same table: 160 m per block, at
  most 12), the L1 field resampled bilinearly; a run without ice fields falls back to the height above the equilibrium line as before.
  `notes.bake.climate.ice_cover.source` names which. Cheap, so done. The 160 m step is unchanged: an ice sheet of 2 km or more is 12.
- **The box moved.** Master's climate (carbon cycle, ice-age cycles, ice-linked sea level, continuous strata) makes `earthlike_quick_16k`
  a different planet (9,544 EF L1 cells against 53,837; the old box, x 3675…3931, z −3779…−3523, is sea now), so the old before/after
  cannot be redone. Control: the merged tree baked with master's table (`deeptime run --content` with the old `climate_placement`) and
  its `climate/ice_cover` layer deleted (so no cover), against the new table on the same planet (`smoke/glaciercheck-ctl`, `-after6`).
  On the new planet 99 % of EF land carries ice (the ice model and the Köppen class agree; 112 L2 cells are barren EF). Two boxes of the
  real generator (`preview-area.sh --keep-world`, `deeptime-inspect biomes`, and the ground under the snow layer read from the
  `.dtva` export): A x −3840…−3584, z 2112…2368 (91 % flats), B x −4160…−3904, z 1984…2240 (43 % flats), both ice cap at 12 blocks.
  | box A (flats) / B (mixed), 65,536 columns each | before | after |
  |---|---|---|
  | MIC: gravel desert | 73.0 % / 29.9 % | 0 / 0 |
  | MIC: snowy plains | 0 / 0 | 73.0 % / 29.9 % |
  | MIC: snowy slopes, ice spikes, frozen peaks | 20.2, 6.8, 0 % / 68.3, 0, 1.8 % | the same |
  | MIC: ground block snow or ice | 27.0 % / 69.1 % (granite 35 %, gravel 23 %, dripstone 12 % / gravel 22 %) | 99.6 % / 99.0 % |
  | vanilla: ice spikes | 99.7 % / 75.9 % | 8.7 % / 32.2 % |
  | vanilla: snowy plains, slopes, frozen peaks | 0, 0.3, 0 % / 0, 20.9, 3.2 % | 91.0, 0.3, 0 % / 43.8, 20.9, 3.2 % |
  | vanilla: ground block snow or ice | 99.9 % / 98.9 % | 98.0 % / 99.7 % (the rest: a village's planks, grass, paths) |
  (MIC's ice spikes are Terralith's own, at humidity band 0, for both signs, so they do not change; vanilla's were on all of the flats
  when they were all positive.) Planet-wide, emulated on the Terralith list and MIC's mapping, ice cap: snowy slopes 54 %, frozen peaks
  27 %, gravel/shrubland 7 % (12 % before), ice spikes 6.6 %, plains 5.7 % (0 before); cherry grove, scarlet mountains, snowy badlands,
  glacial chasm 0.
- **Verified (Mac, on the tree merged with master 39f7ad4 and these changes).** `cargo fmt --check`, `clippy --workspace --all-targets
  --locked -D warnings`, `cargo deny check licenses bans` clean; `cargo test --workspace --locked` (core 287 + 3 ignored, cli 30, format
  44, inspect 29, …) all pass; `./gradlew build test` 155/0. `./gametest.sh` and `-PwithMic`: 53 of 54 pass, including `icePassLaysTheCover`
  (1,028 blocks placed as expected, none wrong) and `iceCoverLaidOnGlaciatedGround` (the 4k test planet has no ice cap: 0 columns, logged);
  the one failure is `climateConsistent`: BW 0.47–0.49 < 0.65, Dsub 0.82 < 0.85, BW:desert 0.42–0.45 < 0.45 (standalone), and under MIC BW,
  BS 0.51 < 0.60, Dfc:taiga 0.43 < 0.50. **It fails the same way on master without these changes** (same numbers, run on master 39f7ad4):
  the 4k test planet is a different planet since the climate merge (466 BW columns where there were 17) and its floors were calibrated on
  the old one; not EF. The planet has no polar land (E family 0 columns), so `EF:no_red`, `EF:no_chasm_on_barren` and `EF:no_cherry`
  never get columns there (and sit after the floor check in the test); the floors need a recalibration by whoever owns the climate merge.
- **Next.** The floors of `climateConsistent` on the new test planet (not this work); Ben's call on the ice-cap humidity (caveat (a) above:
  the gravel desert it once fed is 0 in the boxes now); merge (this branch is not merged).

### 2026-09-30 — `climateConsistent` after the deep-time climate: a weathering-runoff bug fixed, then the test planet's floors re-set (agent worktree agent-adc924860a859865c, branch worktree-agent-adc924860a859865c-gtfix, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Symptom.** Master (after the deep-time climate, continuous strata and P1-25) failed `climateconsistent`
  standalone (BW 0.472 < 0.65 on 466 columns, Dsub 0.822, BW:desert 0.423) and under `-PwithMic` (BS 0.513,
  Dfc:taiga 0.430). The strata agent reproduced it without the strata changes.
- **Cause 1, a bug (970efd1).** `deeptime stage climate` on `planet_world_4k.json` with each change toggled
  (`build/dtc/gt1.sh`, scratch): only the carbon cycle mattered — 8,475 ppm, 26.9 °C at the present against
  280 ppm, 15.6 °C with it off. New series columns (land share, kernel, land runoff) showed the weathering
  kernel falling as the planet warmed: the kernel's runoff used the stand-in PET (60 mm/°C, the surface
  stage's) at the warmed temperature, which grows ~6 %/K while precipitation grows 2 %/K, so warming dried the
  land and weakened the thermostat. Fixed: the weathering's runoff takes PET at the cell's temperature less
  the global warming against `hydro_ref_c` (the spatial pattern kept, the global drying removed; runoff now
  rises with warming as GEOCARB's does). The test planet then ends at 2,949 ppm and 23.1 °C.
- **Cause 2, the sample (adda36d).** The planet is land-poor (8.4 % land area at the present in every
  variant, before and after the climate merge), so under the carbon cycle it is genuinely warmer, and its
  Köppen columns moved (census, standalone): E 1,765 → 319, D 1,296 → 1,964, BS 274 → 792, C 279 → 521,
  BW 25 → 87. The mapping is right: with the old climate pinned in the preset (carbon cycle, sun, ice
  window, glacio-eustasy, hydrological factor off; uncommitted experiment) the same master code passes both
  profiles (standalone BS 0.741, Dfc:taiga 0.94, E:snow 0.956; MIC BS 0.825, Dfc:taiga 0.689, E:snow 0.999).
  The lower shares sit where the new columns are: cold coasts whose shore biome is stony shore (ET 17 %, BWh
  24 % standalone), Terralith's alpine highlands (26 %) and yellowstone (23 %) on the Dfc uplands that were ET,
  moonlight valley (20 %) and gravel desert (14 %) on BSk. New shares (standalone / MIC): BW 0.793 / 0.931,
  BS 0.610 / 0.519, C 0.881 / 0.891, D 1.0 / 1.0, Dsub 0.961 / 0.970, E 1.0 / 0.991, BW:desert 0.690 / 0.816,
  Dfc:taiga 0.952 / 0.435, E:snow 0.824 / 0.991. BS, Dfc:taiga and E:snow take the file's rule (0.1 under the
  lower): 0.42, 0.33, 0.72; the others keep their floors.
- **Measured** (Mac): `./gametest.sh` and `./gametest.sh -PwithMic` (after `tools/smoke/install.sh mic`), both
  with `--server`: All 52 required tests passed, both. `cargo test --workspace --release` all pass (core 280 + 3
  ignored), clippy `-D warnings`, fmt, `cargo deny` clean; `./gradlew build test` tests=151 failures=0.
  Earth-like 500-Myr runs after the fix (`stage climate --preset pipeline_quick_16k`): scattered 14.5 °C,
  220 ppm, glaciated land 14.6 %, Köppen A .20/B .20/C .13/D .20/E .26; supercontinent 13.8 °C, 364 ppm,
  25.4 % (land median 1,109 m against 671 m in the run before the fix: a different surface history — one run
  per start; the calibration's range was 3–18 %). `climate-dem` does not run the carbon cycle: unchanged.
- **Next.** The ice-sheet dynamics and supply-limited weathering (decisions log 2026-09-30) continue on
  `worktree-agent-adc924860a859865c-ice`, which takes this branch.

### 2026-09-30 — P1-21 volcanoes as landforms (agent worktree agent-a348bd3bf6e50dd72, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **What.** A new coupled stage `volcanoes` (`crates/deeptime-core/src/surface/volcanoes/{mod, shape, params, tests}.rs`)
  between the surface and the structure stage (`[plates, climate, surface, volcanoes, structure]`, one step per plate
  step), the amplifier's shaping (`surface/amplify/volcano.rs`), the preset section `volcanoes` (every key a setting,
  `VolcanoParams`, in all four `pipeline_*` presets, the synced `earthlike_*` built-ins and `tests/presets/planet_world_4k.json`),
  `WorldState::volcanoes` (the catalogue; checkpoint schema 7 → 8 after merging the deep-time climate's 7), the `deeptime:volcano_*` events (EVENTS.md, "The
  volcanoes' events"), the keyframe layer `volcanoes/map` and the end layer `volcanoes/flood_basalt` (ramps.json
  `volcanoes`), editor fields (Terrain: `volcano_density`, `volcano_size`; new section Advanced · Volcanoes with the 24
  other keys; Java `EditorFields.Section.ADV_VOLCANOES`, lang keys, `AdvancedSettingsTest`), workbench event colours and
  layer names, `inspect-column --why` → `landforms` (the volcano and flood-basalt events whose footprint holds the column,
  with their cause chains; the workbench's inspect panel shows them as "Why here: …"), porphyry's `causes.pathway`
  (`volcano_stratovolcano`, `volcano_caldera`: a porphyry's chain can now cite its arc volcano), `deeptime volcano-view`
  (`crates/deeptime-cli/src/volcano_view.rs`: global map, settings map, before/after close views with profiles and
  relief numbers at L1/L2/L3), the CLI summary line `volcanoes_final=…`.
- **Model.** A catalogue of discrete edifices (block position, kind, basal radius, height) that ride their plates like the
  plutons. Each step: (1) motion with the plate's Ω (oceanic ones subduct with their crust); (2) erosion: the surface
  stage's net change of the step (`surface/material_m`) fitted over the footprint as dH × the body's rendered weight +
  a uniform part (erosion of the cone lowers it, burial by sediment round it too, a change of the whole area neither),
  and the height is the smaller of that account and the relief the L1 surface shows over the foot's lower-quartile
  level (so blur and the plate model's resampling also age it) — each kept separately, so a one-step dip does not stay;
  (3) arcs: the plate model's arc flag covers ~12 % of the planet at 100k samples (bands a few samples wide plus broad
  hatched patches where intra-oceanic arcs overlap), so volcanoes form only on the **volcanic front**, the medial line
  of the closed flux ≥ `arc_min_flux` mask (not deeper than 4 km: trenches excluded), highest flux first, Poisson-disk
  spaced at `arc_spacing_km / density` × U(0.8, 1.2) from live, just-dead and still-standing cones; target height from
  the flux (`arc_height_km` 1.2–3 km), R ∝ H^0.8 (14 km at the top), life U(2, 8) Myr; silicic on crust ≥ 42 km;
  vents plate motion brings within half the spacing merge (the lower dies); steps whose arc flags the plate model
  cleared (every resampling and the last step) are "blind": no births, no deaths; (4) hotspots: a shield (oceanic) or
  silicic centre (continental) at the plume once the chain's last volcano has drifted `hotspot_spacing_km / density`
  away; it builds while within that distance, more the closer it is; dead oceanic ones subside 25 m/Myr (≤ 1.5 km);
  (5) growth (target − height)·(1 − e^(−Δt/1 Myr)), capped at 6.8 km above the sea (shields 4.4 km), laid on the strata
  log (strato: tuff under andesite, rhyolite when silicic; shield: basalt; silicic: tuff + rhyolite; regolith buried as a
  palaeosol), and added to the surface's offset, elevation and the plate feedback (`surface/feedback_m`, so L0 carries
  the edifice); (6) calderas: silicic cones at 0.04/Myr (continental hotspots 0.12), shields over 4 km a summit caldera;
  the summit inside the rim is removed, half the volume becomes an ignimbrite (tuff) sheet to 4 rim radii, half a tuff
  fill; (7) each new `large_igneous_province` becomes a flood-basalt province: 8 basalt flows each under a thin tuff bed
  over its radius, the plate model's dome of a third of the pile flattened to a plateau; (8) dead cones worn to a tenth of
  their peak for 5 Myr leave the catalogue. Events at the end of the group (all causes exist then): every hotspot volcano,
  live stratovolcanoes and dead ones still at half their peak and ≥ 800 m, calderas, flood basalts (causes: the nearest
  subduction onset on its plate / the hotspot track, else its LIP / the volcano / the LIP).
- **Block-scale shape (amplify).** The young landscape holds a blurred, weaker bump of each edifice (measured: 50–70 % of
  the nominal relief at L1, less after the young stage). Before the erosion passes, on L2, that bump (the body blurred by
  a tent 1.5 L1 cells wide, scaled to the relief measured over the foot, never more than the volcano built) is replaced
  by the true profile — stratovolcano (1 − ρ)^1.5 (Fuji's), shield (1 − ρ²)^1.5, silicic (1 − ρ²)² — scaled by the relief
  kept, so the amplifier's erosion cuts radial gullies into a real cone; after the last flood (in a `TileSink` wrapper,
  tile-independent) the part L2 cannot hold (the sharp summit), the crater (radius max(1 km, 0.07 R), 6 % deep), a
  caldera's flat floor inside a slightly irregular steep rim (floor level from the young landscape's rim), and the
  flood-basalt terraces (80 m treads and risers where a pile crops out within 150 m). Dead volcanoes fade: crater with
  `fade_myr` (5), cone and caldera with 3 × that. **Mountain character (merged from master, 50464ac):** the volcano
  footprints suppress its fold ridges (`CharacterFields::suppress_folds`, fold strength × 0 inside the body rising to 1 at
  1.3 R); its gullies, young-peak sharpening and cliff bands still act on the cones (the L3 residual comes after it;
  since the second follow-ups young volcanoes are exempt from the unsharp mask, below).
- **Measured.** `deeptime run --preset earthlike_quick_16k --start scattered_continents --length-myr 500 --checkpoints`
  (Mac M6, shared, load 60–110): 418 s wall, peak 3.6 GiB (unchanged); **volcanoes 14.1 s** (births 6.0, erosion
  accounting 6.0), amplify's volcano sink 0.6 s. `volcanoes_final=250 strato_born=24017 shield_born=181
  caldera_complex_born=91 standing_strato=1275 standing_shield=8 standing_caldera_complex=5 active_strato=203
  active_shield=2 calderas=295 summit_calderas=27 flood_basalts=9 tallest_m=4643 flood_basalt_outcrop_cells=389
  events=1468` (of 3,498). Close views (`deeptime volcano-view`): the highest live continental-arc cone relief 2.1 km
  (young L2) → 3.1 km (L3), summit 3.6 km a.s.l.; an island-arc cone 2.5 → 3.2 km, an island 418 m high; the youngest
  shield of the chain with most islands 4.8 → 5.0 km (756 m island) with two older islands and islets down-chain; a 2-Myr
  caldera 14.6 km across, flat floor 764 m deep. `cargo test --workspace --locked` (core 272 + 3 ignored, cli 30, format
  40, inspect 29 + 2 + 7, jni 7, viewer 5), `clippy --workspace --all-targets --locked -D warnings`, `fmt --check`, `deny
  check licenses bans` clean; `./gradlew build test` tests=146 failures=0; `tools/remote/mac.sh --server -- ./gametest.sh` → All 50 required tests
  passed (realPlanetHeightMatchesTheBake and realPlanetRockMatchesTheParityFixture on the 4k planet, which now runs the
  volcano stage).
- **Tests.** `the_front_is_the_middle_of_the_arc_band` (band → its centre row, striped patch → a skeleton, no front in a
  trench), `arcs_line_up_along_the_front_at_the_spacing` (2,400 km arc: all on flux cells, no pair nearer than 0.8 ×
  60 km, mean neighbour 48–80 km, 28–52 cones, saturation, density 2 → ≥ 1.6×), `the_pipeline_puts_volcanoes_where_the_plates_say`
  (20k samples, 120 Myr: every stratovolcano born on a flux ≥ threshold cell of its own step; live ones ≥ 0.5 × spacing
  apart; hotspot chains born in order, only the newest two alive, the live ones within the spacing of their plume, ≥ 70 %
  of consecutive same-plate pairs older-farther; every LIP → one flood basalt caused by it; every event's causes of the
  right kinds; andesite, basalt and tuff in the log; log invariants), `threads_and_a_resume_leave_the_volcanoes_unchanged`
  (1 thread straight = 3 threads resumed from a mid-group checkpoint: catalogue, events, elevation, map bit for bit),
  shape tests (profiles, caldera truncation, blur keeps volume), amplifier tests (a cone sharpened at L2 with the volume
  kept, its crater cut at L3; a 30-Myr-dead crater gone; terraces monotone with treads and risers).
- **Pictures.** `review/out/p1-21/bundle/` (11 MB, not published): `index.html` with the global map (volcano chains along
  every active margin and island arc, hotspot islands, flood-basalt rings), the settings map (the last frame's arc flux
  and live plumes under the markers), before/after close views with E–W profiles, and nine 3D previews from the real
  generator on this run's bake (`tools/workbench/preview-area.sh --world-data review/out/p1-21/q16f`, standalone,
  256 × 256 blocks; screenshots by Playwright's headless Chromium on the Mac — nixpkgs' chromium does not build for
  aarch64-darwin): a mid-latitude coastal arc cone, the highest arc cone, a caldera with flat floor and tuff/andesite walls,
  an island-arc volcano, the youngest shield of a chain and an older member.
- **Findings (not changed here).** (1) The plate model clears its `ARC`… flags on resampled crust, so every resampling step
  and the last step have no arc anywhere (`arc_magma_flux` of those steps is 0 for plutons and the deposit frames too; the
  frames average it away). (2) Its arc band is ~12 % of the planet at 100k samples. (3) Under deep ocean the 3D previews
  show caves below y −54 filled with lava (4.5 M lava blocks in a 256 × 256 box of sea floor): vanilla's lava level in
  Deep Time's cavities, not the volcanoes'. (4) One run (q16c) showed a ~5 km deep pit on an Andean plateau already in the
  young landscape (not at a volcano); not investigated. (5) At 20 m vs 300 m per block the world is 15× taller than wide,
  so true-proportion cones are steep peaks in blocks.
- **Follow-ups (Ben, decisions log 2026-09-30).** (1) **"Setting, gentler default"**: `volcanoes.flank_steepness`
  (0.1–1, default **0.5**; the editor's Terrain field "Volcano flank steepness", stops Gentle 0.35 / Readable 0.5 /
  Physical 1) divides every edifice's physical basal radius, the height unchanged: at 0.5 a 3-km arc cone is 26–28 km in
  radius (56 blocks of relief per 90 blocks of run instead of per 45), shields up to 71 km; 1 is the physical profile.
  (2) **"Group by arc/chain"**: one `deeptime:volcano_arc` per arc segment (the cones one subduction contact fed, keyed by
  its onset event, else by plate; `ArcSegment` in the catalogue: first/last activity, cones born, where, peak, calderas)
  and one `deeptime:volcano_hotspot_chain` per hotspot, each with its standing cones as a detail (`cones`:
  `lat,lon,radius_km,height_m;…`, tallest first, ≤ 80); on their own only each group's tallest standing cone (a
  stratovolcano ≥ 1.5 km × size), the calderas still standing or ≥ 6 km across, and the flood basalts. Notable cones are
  caused by their group, groups by the subduction onset / hotspot track. `inspect-column --why` finds a column's cone in
  a group's list (the notable cone's own event first); porphyry's `causes.pathway` now also lists `volcano_arc`, so a
  porphyry of any age on an arc cites its segment (an arc event spans the segment's whole life) or a notable cone nearer.
  **Event counts** on the same preset (`earthlike_quick_16k`, 500 Myr, run q16g): **611** volcano events of 2,931
  (before: 1,468 of 3,498): `volcano_arc` 341, `volcano_hotspot_chain` 20, `volcano_stratovolcano` 73, `volcano_shield` 4,
  `volcano_caldera_complex` 3, `volcano_caldera` 162, `volcano_flood_basalt` 8 (`notes.volcanoes.events_by_kind`).
  Run: 420 s wall on the loaded Mac, volcanoes 22.7 s (the wider footprints: the erosion fit 13.5 s), amplify's volcano
  sink 1.1 s; 282 live arc cones, 1 live shield, 137 calderas, 8 flood basalts.
- **Second follow-ups (Ben, decisions log 2026-09-30, commit 86d66d6; branch merged with master 86d66d6 first:
  EF ice cover, the climateConsistent runoff fix, glacial landforms).** (1) **"Count small ones"**: an arc segment that
  built fewer than `SMALL_SEGMENT` = 3 cones and has a subduction onset writes no `volcano_arc`; the onset event gains
  `volcano_cones_built`, `volcano_cones_standing` and `volcano_cones` (the same `lat,lon,radius_km,height_m;…` list,
  several small segments of one contact add up), set at the end of the coupled group (the events file is written from
  the world's events at finalize; a live watcher keeps the onset as first published); a notable cone of such a segment is
  caused by the onset; plate-keyed segments (no onset) keep their event. `inspect-column --why` reads `volcano_cones` on
  any event, wherever the event itself is placed (`why_here_finds_a_small_arc_segments_cone_on_its_subduction_onset`).
  Notes: `counts.small_arc_segments`, `counts.small_arc_cones` (`Counts` now `#[serde(default)]`, so older schema-8
  checkpoints load). (2) **"Smooth young, rugged old"**: `Shaper::youth_keep` (1 − the volcano's youth inside its body,
  back to 1 at 1.3 R; youth = 1 while active, e^(−age/`fade_myr`) once dead, the crater's clock) →
  `CharacterFields::calm_volcanoes`: there the mountain character's unsharp mask is faded out both ways (no young-peak
  sharpening, and no old-ground rounding, which would blunt the cone towards its 24-block blur) and its gullies take old
  ground's soft shape; folds stay suppressed as before. The sea is untouched by the character anyway, so a submerged
  shield does not change. Tests: `young_volcanoes_get_no_young_peak_sharpening_and_old_ones_regain_it` (active 0,
  5-Myr-dead 1 − e⁻¹, 30-Myr-dead ≈ 1), `a_young_volcano_keeps_its_profile_on_young_ground` (the cone fixture: young
  ground lifts the summit, a calmed one barely moves; keep 1 changes nothing), the pipeline test's event rules.
  **Run** (`deeptime run --preset earthlike_quick_16k --start scattered_continents --length-myr 500 --checkpoints`,
  `/Volumes/Graphide/ban/deeptime-runs/p1-21/after`, Mac at load ~80): 1,269 s wall, peak 4.0 GiB, volcanoes 31.0 s;
  **585 volcano events of 2,499**: `volcano_arc` 296, `volcano_caldera` 164, `volcano_stratovolcano` 87,
  `volcano_hotspot_chain` 21, `volcano_shield` 8, `volcano_caldera_complex` 2, `volcano_flood_basalt` 7; **15 small
  segments (20 cones) counted on their onsets** (most segments build ≥ 3 cones: arc events fall by 5 %, not more).
  **A/B of the calm mask** on that run's young landscape (`deeptime stage amplify --in 6-young.dtcp --bake` with the
  mask line removed = before; the run's own bake = after; `deeptime heightmap` over the boxes): the active continental-arc
  cone 19773 (2,976 m, x 371 z −2765) summit y 452 → 442 (the sharpened spike gone), mean |∇²| on its land 0.656 → 0.613
  blocks, mean |Δ| 0.16 blocks; the active shield 19366 (4,673 m, x −3668 z 658, a 38-block island in a 70-km body)
  mean |∇²| 0.919 → 0.826, the highest ground in its body (a peak 110 blocks off the vent) y 415 → 380, mean |Δ| 0.74
  blocks; the whole planet's character change 1.86 → 1.78 blocks mean. 3D previews (`preview.py run`, standalone,
  same boxes, before/after) and shots: `review/out/p1-21/shots3/` (arc cone 256²: the craggy, notched summit becomes a
  clean cone; shield 384²: its vent is a low island and the two peaks inside its body, older ground, go from spiky to
  rounded; the submerged active shield 19757 for contrast: no change), the arc and the emergent shield pairs copied
  into `review/out/p1-21/bundle/` (now 19 MB, not published). This run has no emergent young shield with a clean dome
  of its own: of the 8 standing young shields and caldera complexes, the tallest above the sea is a 38-block island.
  Master bd1cc0f's `deeptime-core` lib tests did not compile (the volcano shaper test's `Level` literal lacked the
  glacial work's new `zs` field); fixed here and on master the same way, so the merge of master (9d7fb4b) was clean.
  **Gates** (the follow-ups merged with master 9d7fb4b): `cargo fmt --check`, `clippy --workspace --all-targets
  --locked -D warnings`, `deny check licenses bans` clean; `cargo test --workspace --locked` ok (cli 31, core 319 + 3
  ignored, history_cap 1, format 44, inspect 29 + 2 + 7, jni 7, viewer 6); `./gradlew build test` tests=155
  failures=0 (on the tree before the master merge, which only brought docs, a probe script and the -O1 test profile).
  GameTests not run on the branch (Ben, 2026-09-30: once per merge batch on master). The Mac rebooted mid-way
  (~19:06); its server lock from before the reboot (owner `deep-time`, no process left) was removed so the previews
  could run. Scratch runs under `/Volumes/Graphide/ban/deeptime-runs/p1-21/` and the old q16f/q16g runs deleted.
- **Gates on the final tree** (master 84e2b1d and the decisions-log commits after it merged, schema 8 on the climate's
  7, with the follow-ups): `cargo fmt --check`, `clippy --workspace --all-targets --locked -D warnings`, `deny check
  licenses bans` clean; `cargo test --workspace --locked` ok (cli 30, core 290 + 3 ignored, history_cap 1, format 44,
  inspect 29 + 2 + 7, jni 7, viewer 5); `./gradlew build test` tests=151 failures=0; `mac.sh --server -- ./gametest.sh`:
  **All 52 required tests passed**, after re-basing `climateConsistent`'s narrow `E:snow` floor 0.9 → 0.8 (it measured
  0.899 on `test_planet_real`, E overall 0.98: master's EF barren frozen ground is not snowy-tagged, and the test planet
  now runs the volcano stage). (The merged tree before the follow-ups: the same cargo and Gradle results.) 3D previews of the gentler
  cones: `review/out/p1-21/bundle/shots/` (arc cone at 0.5 beside the physical-profile shots of run q16f); the bundle is
  15.7 MB. Old scratch runs on the Mac SSD deleted (disk hygiene rule).
- **Deviations.** Stratovolcano "centres" live 2–8 Myr (real cones 0.1–1 Myr) because a step is 2 Myr. Stratovolcanoes
  that erode away are counted, not written as events (~24,000 per run). No lava or obsidian blocks, no volcanic ore
  (P2-10's epithermal), no guyot planing beyond the surface's wave-base diffusion (P2-11).
- **Next.** Merge into master (the parent). Ben: the volcano event volume (1,468 of 3,498 events) and whether cones should
  be less steep in blocks than real proportions (a `volcanoes.size`-independent flank setting).
  Answered and done (follow-ups above); next: the parent merges commit 899d797 and its STATUS commit into master and
  runs the GameTest batch there.
### 2026-09-30 — Glacial erosion landforms: troughs, fjords, till, U-shaped valleys at block scale (agent worktree agent-a9307a6541fa46794, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Why.** Decisions log 2026-09-29, "Which glacial features should Deep Time add?" → "Glacial erosion
  landforms": U-valleys, cirques, fjords, moraines/till where ice sat, now or in past glaciations. Until now
  P1-05's glaciers were only diagnosed.
- **What.** (1) `surface/glacial.rs` (+ `glacial/tests.rs`): one glacier model per LEM step, `substeps` (4)
  ice solves each, used by the deep-time surface (L1) and the young landscape (L2). Mass balance b = β(z − ELA),
  β 0.005/yr (Oerlemans 2011), capped by precipitation and 2 m/yr, ablation ≤ 10 m/yr; ice flux routed down the
  step's D8 tree (Hergarten 2021; goSPL's GPL docs read for comparison only); each cell's ice surface solved from
  its receiver's by bisection on the shallow-ice flux q = (2A/5)(ρgS)³H⁵ + u_s·H, A = 2.4·10⁻²⁴ Pa⁻³s⁻¹,
  Weertman sliding u_s = B τ³ (30 m/yr at 100 kPa) on thawed beds (mean annual T −14…−4 °C, or 800–2,500 m of
  ice insulating), glacier width H/0.3 up to 7 cells; erosion E = K_g·u_s, K_g 10⁻⁵ (Humphrey & Raymond 1994's
  law, the coefficient from Hallet et al. 1996's rates), spread parabolically across a glacier wider than its
  cell, none under floating ice or where the bed rises towards the outlet more than 1.5× the ice-surface slope
  (Alley et al. 1998/2003); debris melts out with the ice, 30 % as till (≤ 100 m per substep), the rest as
  meltwater load into lakes it crosses and to the sea at its mouth (glaciomarine: `marine::fill` into the till
  part). Mass kept exactly (tested). Fjords follow: ice grounded below sea level at a coast keeps cutting until it
  floats. (2) Surface stage: fluvial K × (1 − the glaciers' cover); till is its own facies part (`Setting.glacial`
  → class `till`, provenance in situ); the existing `till` class (P1-12; family unconsolidated, lithifies to
  conglomerate) is used — no new class. New fields `surface/ice_thickness_m` (a keyframe layer, ramp `ice`) and
  `surface/glacial_erosion_m_myr`; totals in `notes.glacial` (so checkpoints carry them; no schema change) and the
  CLI's `glacial_final` line. (3) Young stage: the glacial state's ELA (the present one − `young_ela_drop_m`, 500 m,
  Porter 1989's average glacial snowline; or the climate's own fields when published), sea-level temperature,
  precipitation on L2; glaciers erode, leave till (≥ 5 m makes `young/class` till, so the bake's `class_top` shows
  it) and hand `young/ice_surface_m` (over each glacier's whole width) to the amplifier; rivers keep their K
  there (see findings). (4) Amplifier: `amplify/glacial.rs` on the **last level only**: from the ice depth D and
  the valley's smoothed centre-line depth D_max, cells lowered by D_max·t(1 − t) (t = D/D_max: a V's walls cut to
  the parabola of Svensson 1959/Harbor 1992), only where the ice is confined (ridges within reach stand above it:
  Sugden 1978's selective linear erosion vs areal scouring) and the margin lateral (not a terminus or a tidewater
  front); half the carved rock goes back onto crests; fluvial incision × (1 − 0.8·ice) on every level; its radius
  (3·20 + 1 cells) is in the halo; the mountain character's fade (`character::apply`, new `glacier` argument) also
  skips the troughs the pass cut (the cut is handed over in `cap`/`zs`), so gullies and fold ridges don't fight
  the U while crests above the ice keep their character. (5) One place names the climate's fields:
  `glacial::source`, now the deep-time climate's (FORMAT.md "The climate's ice fields", merged from master
  27e88f7): the ELA (`climate/anomaly/ela_m`); deep-time glaciers only where `climate/ice_max_thickness_m` > 0
  (ice at some time in the step), eroding × `climate/ice_time_share`, and fluvial K also × (1 − ramp of
  `climate/ice_thickness_m` 10–100 m) (the climate agent asked for no stream power under thick ice); the young
  landscape's glaciers only inside `climate/ice_max_extent` (the LGM at the present). (6) Settings
  (presets + `editor_fields.json` + lang): `surface.glacial.strength` ("Glacial erosion", terrain), 
  `amplify.glacial.strength` ("Glacial valleys", terrain), `surface.glacial.young_ela_drop_m` ("Ice-age snowline
  drop", adv_climate), `amplify.glacial.ridge_share` ("Arêtes from troughs", adv_surface); every constant above is
  a preset key. (7) Tools: `deeptime dump-fields --in ckpt --out dir [--fields a,b]` (raw f32 + index.json),
  `tools/review/glacial-compare.sh` (young/amplify on/off from a deposits checkpoint → `glacial_views.py`: the most
  glaciated confined range and fjord coast, maps, zooms and 1:1 block cross-sections), `glacial_planet.py`,
  `area3d_shots.py` (headless shots of `.dtva` exports on the Mac).
- **Tests** (`cargo test --workspace --locked` on the Mac after merging master 8872c91: cli 30, core 273 + 3
  ignored, format 40, inspect 29 + 2 + 7, jni 7, viewer 5, all pass; clippy `-D warnings`, fmt, deny clean):
  `glacial::tests` — a glacier reaching the sea cuts its valley below the sea floor (a fjord behind a sill),
  eroded = till + lake + sink + sea to 1e-9 and the grid's change = −(sea loads), a thin polar cap erodes nothing
  and a thick one thaws its bed, no ice below the ELA or where the climate forbids it, the same bits on 1 and 4
  threads, the SIA flux law; `a_glaciated_planet_keeps_its_mass_logs_its_till_and_resumes` (plates + climate +
  surface + structure, snowline pushed down, 40 Myr: totals partition, surface mass residual < 1e-6 of erosion,
  the strata log holds till, a checkpoint resumed on 3 threads from 1 gives identical elevation, log, notes and ice
  fields); `amplify::glacial::tests` — a glaciated V valley becomes y ∝ |x|^b with b > 1.6 (from 1.0; the axis and
  the trimline stay), crests get some rock back, a flat bed under an ice sheet is untouched, a window with the halo
  gives the whole raster's bits; `amplify::tests::a_glaciated_level_amplifies_the_same_in_tiles_and_on_any_thread_count`.
- **Runs** (Mac M6, 12 threads, shared: load 20–96; `deeptime run --preset earthlike_quick_16k --start
  scattered_continents --length-myr 500 --out target/glacial/after4 --checkpoints --viewer …`): 478 s wall (master's
  baseline 236 s at lower load); surface 195 s of which glacial 19.7 s; young 15.6 s (glacial 2.2 s); amplify 61.7 s
  (glacial pass 5.9 s); peak footprint 4.6 GB. `glacial_final eroded_km3=2.14e6 till_km3=4.6e5 lake_km3=3.1e5
  sea_km3=1.37e6 steps_with_ice=250 ice_cells_max=45038 ice_cells_last=29771 cut_below_sea_cells=8654
  max_rate_m_myr=1351` (2.4 % of the planet's 8.8e7 km³ of erosion; the mass residual stays −8.9e-4 km³);
  young: 174,894 L2 cells under the glacial state's ice, 5.9e4 km³ eroded, 29,204 cells of till; amplify: 4.4e4 km³
  carved, 6.2e3 raised, 5.97 M cells shaped, the final flood raises 22,330 cells, at most 121 m (overdeepened
  basins filled flat: lake floors until water lands in Phase 2) vs 1,869 without glaciers. The glacial deep time
  diverges from master's planet (the surface feeds the plates), so before/after compare statistics: land 0.271 →
  0.332, median 601 → 665 m, p90 2,093 → 1,918, **p99 5,092 → 3,826 m**, max 8,246 → 7,757 (the high tail is cut
  towards the ELA: the glacial buzzsaw), glaciated land 5.9 → 4.8 %.
- **What the pictures show** (`review/out/glacial-review/`, not published; L3 in blocks at 1:1): planet maps and
  hypsometry; the fjord coasts (long narrow drowned inlets on both sides of a glaciated island range, cut 10–20
  blocks below sea level where the glaciers met the sea); an outlet trough off an ice-capped plateau with a flat
  floor and steep walls (U) where the orange L2-only profile is a rounded V; ice-sheet interiors left smooth;
  fold/gully texture absent under the ice. Iterations that the pictures forced: one-cell D8 slots and lakes
  (glaciers now spread over their width), ridge-raise spikes (the ridge share is capped and needs a quarter of the
  neighbourhood to be crest), carving on the coarse level (24-km radius) that printed grooves on ice caps (last
  level only), max-filter streaks (smoothed), frontal margins cut like walls (the lateral factor), lowlands lifted
  200 m under the young stage's ice (next bullet).
- **Findings; Ben's answers (decisions log 2026-09-30).** (1) The glacial deep time lowers the highest ground (p99
  −1.3 km on the first seed): **strength stays 1 (physical)**, heights are rechecked in P1-17. (2) At 300 m/block and
  20 m/block the vertical exaggeration is 15×: a physical 2-km trough is 7 blocks wide and reads as a gorge →
  **"Glacial valley width" setting, default 3, 1 = physical** (`surface.glacial.valley_width`, editor field
  `glacial_width`): the young landscape draws each glacier's footprint and the spread of its erosion that many times
  wider (the flow law, thickness and eroded volume are unchanged), so the amplifier's U spans the wider footprint;
  the deep-time surface always uses 1. (3) Till lithifies to conglomerate (P1-12's class); a `diamictite`/tillite
  class would be more correct — a 60th class touches Java, GameTests and the MIC rock-tag README (not asked yet).
  (4) The young stage keeps fluvial K under the glacial-state ice: taking it away with the `balance` uplift lifted
  ice-covered lowlands by ~200 m. (5) Eskers and drumlins not done; moraines are the till humps at termini (and the
  lakes they dam), no crisp crest line; hanging valleys and cirques come only from the physics and the parabola.
- **With the deep-time climate** (after merging local master 27e88f7; `…--out target/glacial/after5`, 1,130 s wall
  on a loaded Mac): `glacial_final eroded_km3=1.85e6 till_km3=3.6e5 lake_km3=4.8e5 sea_km3=1.0e6
  ice_cells_max=66350 cut_below_sea_cells=2330 max_rate_m_myr=1121`, glaciated land 15.1 %, 3 glaciations, land
  median 768 m, p90 2,180, max 9.6 km (this climate's ice lies mostly on low polar land under cold beds, so the
  buzzsaw of the earlier runs is weaker); young 232,124 L2 cells under the LGM-masked glacial ice, 9.7e4 km³
  eroded, 34,591 till cells; amplify 4.3e4 km³ carved, the flood raises 27,777 cells (max 267 m). The views
  (`review/out/glacial-width/`: young + amplify from after5's deposits checkpoint without ice-age glaciers, at real
  width and at width 3) show fjord networks where the ice met the sea (drowned U-troughs 20–40 blocks wide at width
  3, narrower and more ragged at 1) and ice-sheet interiors left smooth.
- **3D** (`tools/workbench/preview-area.sh --profile standalone --y 20,330 --box -2544,-3208,-2288,-2952 --box
  2976,-3248,3232,-2992 --world-data target/glacial/after4`, and the same boxes of the same planet run with
  `--set surface.glacial.young_ela_drop_m=-2000 --set amplify.glacial.strength=0`, i.e. without the ice-age
  glaciers; shots by `area3d_shots.py` in `review/out/glacial-3d/`): the valley box is a V-shaped, gullied valley
  without them and a deeper trough with a flat floor (sand, grass and trees, a small lake) and steep smooth snowy
  walls with them; the coastal box shows a smoother, scoured lowland. Width comparison on after5 (young + `stage
  amplify --bake` from its deposits checkpoint at width 3, width 1 and without ice-age glaciers; the same two boxes
  in each, `review/out/glacial-3dw/`, bundle `review/out/glacial-review/`, 16 MB, not published): at x −6608..−6353,
  z 2944..3199 a snowy plateau without them, a narrow fjord at real width, and at width 3 a wider fjord with a side
  arm, steep rock walls and a flat, partly flooded floor; the other box (the thickest ice of the window, on an
  ice-capped plateau) barely changes. Scratch runs deleted from the Mac afterwards.
- **Gates** (branch merged with local master 51c6c18). c2ec70f (the character's fade limited to the carved
  troughs) compiled and passed, not reverted. cargo fmt, clippy `-D warnings`, deny, `cargo test --workspace
  --locked` (cli 30, core 291 + 3 ignored, format 44, inspect 29 + 2 + 7, jni 7, viewer 5) pass; after the width
  setting: `glacial` 12/12, `preset` 14/14, cli 30/30, clippy clean; `./gradlew build test` tests=151 failures=0;
  `mac.sh --server -- ./gametest.sh`: every test passes but **`climateconsistent`** (BW 0.504 < 0.650, 409 columns).
  Not the glaciers: the same run with the test planet baked at `surface.glacial.strength=0` and
  `amplify.glacial.strength=0` fails it worse (BW 0.472, Dsub 0.822, BW:desert 0.423; 466 BW columns where P1-09
  calibrated on 17): the merged deep-time climate made the 4k test planet much drier, and its BWk columns map to
  plains 64 %. The floors (or the BWk mapping) need the climate agent's or P1-09's look.
- **Deviations.** None from the plan's §8.2 structure; the plan has an "As built: glacial landforms" bullet.
- **Next.** Merge into master (the coordinator); the `climateconsistent` floors after the deep-time climate;
  a `diamictite` class if Ben wants it.

### 2026-09-30 — P1-25: a climate API for mic-climate and Project Atmosphere (agent worktree agent-a02882c5e6567370e, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **The bake (format 0.1 → 0.2).**
  - At its last snapshot the climate stage keeps every cell's twelve months:
    - the sea-level temperature `t_sl`;
    - the precipitation rate.
  - The switch is `snapshot::Input::monthly`. The fields are `climate::field::{t_sl_month, p_month}`, written only at the last step. The stage estimate grows by 96 B per L1 cell.
  - The bake stores the months on L1:
    - `climate/month/t_sl_c_01..12`, i16 at 0.01 °C, at sea level so a reader lapses at its own height;
    - `climate/month/p_share_01..12`, u8 at 1/40, each month's rate over the annual.
  - `notes.bake.climate.monthly` records the layer names, `lapse_c_per_km` and the month convention (month 01 starts at solar longitude 280°, about 1 January). This adds 24 layers, about 12 MB raw on a 16k planet.
  - The version is pinned in: Cargo.toml, `FORMAT_MINOR`, Java `FormatVersion` (CURRENT and MINOR), and the tests' pins, including the JNI version string.
  - FORMAT.md has the "0.2 over 0.1" paragraph and "The climate at a block": the evaluator, in order. It does:
    - bilinear months, less the lapse over `max(0, y − sea)·m_per_block_v`;
    - shares normalised to 12;
    - the Köppen class as the nearest cell.
  - Rust `deeptime_format::ClimateWorld` and Java `worlddata.ClimateLayers` implement it operation for operation.
  - `deeptime parity` also writes `parity/climate.json`: 500 blocks, a third each at the terrain top, at sea level, and at a random height. The Gradle `testWorldPlanet4k` task lists it as an output.
- **The API** (package `com.minecraftindustrialcomplex.deeptime.api.climate`).
  - Public surface:
    - `DeepTimeClimate`: `of(Level)`, `API_VERSION = 1`, `isDeepTimeLevel()`, `hasClimate()`, `at(BlockPos)`, `at(x, y, z)`, `temperatureC(pos, yearFraction)`, `lapseRateCPerKm()`, `seaLevelY()`, and the year convention `yearFractionAtSolarLongitude` / `NORTHERN_SPRING_EQUINOX`;
    - `ClimateSample`: twelve months at the block's height, mean / warmest / coldest (with month indices), `temperatureC(f)` (periodic linear between month midpoints), annual and monthly precipitation, `precipitationRateMmPerYear(f)`, wettest and driest month, `precipitationSeasonality()` (BIO15 CV %), Köppen code and group, `seaLevelMonthC`.
  - Internals, package-private:
    - `LevelClimate`, one per WorldData;
    - `ClimateColumns`, which caches the height-independent column: 16,384 columns, dropped when full.
  - Behaviour outside a Deep Time climate:
    - it is server-side only;
    - a client or non-Deep-Time level answers `isDeepTimeLevel` false and every query empty;
    - a synthetic level has `hasClimate` false.
  - `/deeptime climate [pos]` logs a `[deeptime-probe]` JSON line; its own listener merges into `/deeptime`.
  - Inputs are baked fields only, as agreed with the climate-forcing agent. The only change outside the new package and the bake is those 24 fields.
- **What Project Atmosphere 0.9.1.2 allows** (a Sonnet lookup over the jar, decompiles on the Mac):
  - **No base-temperature hook.** It has:
    - no provider, registry or event for the base;
    - `RegionAtmosphereState.baseTemperature` as `private final`, set once from a persisted 7-day forecast that is sampled from biome **base** temperatures (no height) through a per-biome table (`BiomeTempConfig`);
    - no Terralith biomes in that table, so they get `Range(0,0)` and count as about 0 °C;
    - regions 2000 blocks square;
    - region keys with no dimension.
  - **Written temperatures do not hold.** `setTemperature` and `adjustTemperature` are public, but the scheduler blends back toward its own target (0.6 every 20 ticks when active), so a pushed base erodes within a few updates.
  - **Seasons are global.** The season offset is one value for the whole server (+6 summer, −8 winter, from Serene Seasons) and applies to both hemispheres.
  - **It simulates only while a player is in the overworld** (`serverLevel.players().isEmpty()`). Regions are created at boot within 10,000 blocks of spawn, and hold `temperature == base` exactly until the first update.
  - **What it consumes from its own regions:**
    - its snow and freeze mixin blends the region with the local biome's range, plus a height lapse;
    - clouds, crop stress and Serene Seasons Plus snow read the region directly.
  - **No command sets a region's temperature.** `/weather` is hijacked to spawn a cloud.
  - Readable through public classes (no mixins): `AtmoApi.getCurrentWeather`, `AtmosphericStateRegistry.getState(RegionInstanceKey.from(pos))`, `getTemperature()` and `getEffectiveBaseTemperature()`. mic-climate already used these for pollution.
- **The coupling chosen: (a), and not (b).**
  - (b), pushing Deep Time's base into Project Atmosphere's regions, is not possible through its API. It would erode, and one value per 2000-block region is about a quarter of the planet's pole-to-pole height.
  - mic-climate's `mic_climate:unified` provider, in a Deep Time world with a climate:

    ```
    T = DeepTime(pos, SS date) + clamp(PA live − PA effective base, ±20) + pollution
    ```

    - Deep Time's monthly mean at the block's height, at the Serene Seasons date. Early Spring is read as the start of March, so the cycle maps linearly onto the year from month 2. Without Serene Seasons it uses Thermoo's season middle month; without a season source, the annual mean.
    - Project Atmosphere's anomaly carries its day/night swing, rain, the forecast and advection. Its season offset cancels, and a never-simulated region gives no anomaly.
    - Pollution arrives through the anomaly in ATMOSPHERE mode and is added directly otherwise.
  - Everything else takes the old path: worlds Deep Time did not generate, packs without Deep Time, and the client.
  - Other mic-climate changes:
    - `Climate` caches per chunk section, because height now matters;
    - `/mic_climate probe` gains `deeptime` and `dt+weather` lines;
    - config `deepTime.{enabled, weatherAnomaly, maxAnomaly}`;
    - `/mic_climate atmosphere drive <ticks>`, only with `-Dmic_climate.driveAtmosphere=true`, calls Project Atmosphere's own public `AtmosphericUpdateScheduler.tick` and `SeasonalAtmosphericDrift.tick` on a headless server;
    - `gametest.sh` runs Gradle directly inside an existing dev shell (its flake's Linux-only libraries do not build on the Mac);
    - `DeepTimeGameTests` covers the calendar, the cap, and other worlds unchanged;
    - `plans/phase-07-deep-time.md`.
  - The branch is `deep-time` on MinecraftIndustrialComplex/mic-climate (default branch `master`), pushed. Master, releases and the pack are untouched.
  - The branch compiles against `libs/deeptime-1.21.1.jar`, gitignored like the Project Atmosphere jar.
- **Serene Seasons vs Deep Time.**
  - Serene Seasons' seasons are global. Its own temperature effect only shifts cool, non-tropical biomes by −0.8 biome units in winter. Its visuals (leaves, crops, its snowfall shift) stay northern everywhere.
  - Deep Time's amplitude is per place, from the simulation: tropics about 1.4 °C, desert 12 °C, Dfc 30 °C, ET 28 °C, EF 26 °C.
  - mic-climate follows Deep Time's amplitude and hemisphere. The temperature machines read is right; Serene Seasons' visuals disagree in the south.
- **Measured.**
  - Mac, after merging master 00c02ce:
    - `cargo fmt --check`, clippy `--locked -D warnings`, and `deny licenses bans` are clean;
    - `cargo test --workspace --release` passes (core 256 + 3 ignored, including the bake test's monthly checks, 1,520 sea cells equal within 0.02 °C, share sums within ±0.1);
    - `./gradlew build test` gives tests=147 failures=0;
    - `ClimateParityTest` is 500 of 500 equal (112 on land, 143 above the sea);
    - `DeepTimeClimateTest`: warmest month in the local summer at 126 of 126 seasonal samples.
  - `./gametest.sh --server`:
    - `climateApiOnTheRealPlanet` fixture 500/500 equal;
    - the first run failed its "tropical land" check because the 4k test planet has none; the check is now over land and sea;
    - rerun after the fix: **All 52 required tests passed** (tropics 26.6 °C vs high latitudes −4.8 °C; 50 blocks up colder at 400 of 400).
  - mic-climate on the Mac (`tools/remote/mic-climate.sh`):
    - `./gametest.sh` passes 19/19;
    - `-PwithAtmosphere` passes 19/19, with Project Atmosphere present;
    - smoke (`smoke/install.sh` + `run.py`: the real pack, 166 mods, flat world, no Deep Time, this branch's jar) passes **20/20**;
    - the smoke ran on the pack snapshot Deep Time's `install.sh mic` makes, because pack HEAD 4d0e685 carries a stale index hash for `kubejs/server_scripts/destroy_metallurgy_integration.js` that packwiz refuses (a pack finding, not this task's).
  - `mac.sh --pack --server -- tools/review/p1-25-probe.sh` (MIC, earthlike_quick_16k scattered 500 Myr, first start 191 s, smoke PASS). All values °C at the surface; D = midwinter (`season set mid_winter`), S = midsummer:

    | Site | Height | Deep Time mean / warmest / coldest | Unified, D / S | PA's own reading |
    |---|---|---|---|---|
    | Af (-3968, 32) | 440 m | 25.2 / 25.8 / 24.4 | 24.5 / 24.6 | 8.6 |
    | BWk (-4544, -1120) | 1,060 m | 14.8 / 20.0 / 8.1 | 8.8 / 19.6 | 13.7 → 8.9 |
    | Dfc (7504, -2416) | 260 m | 0.3 / 16.1 / −14.2 | −12.9 / 15.3 | −0.6 |
    | ET (2800, -3184) | 1,100 m | −14.1 / 1.6 / −26.7 | −24.7 / 0.6 | 4.0 |
    | EF plateau (1744, 2912, south) | 4,860 m | −35.5 / −22.3 / −48.0 | −22.7 / −46.7 | 10.5 |

    - The unified value is what Power Grid, Destroy, LSO and Thermoo all read.
    - Before this change the unified value was PA's own reading.
    - The seasons invert across the equator.
  - With PA's simulation driven:
    - anomaly at noon: +2.2, −0.2, +1.4, +2.5, +1.6;
    - anomaly at midnight: −1.2, −1.0, −2.4, −1.3, −0.9;
    - so the unified value swings by 1–4 °C day to night on top of Deep Time's climate.
  - `/weather rain` spawned its cloud at the spawn region, not at the sites, so no rain term showed.
  - Data: review/out/p1-25/ (`summary.txt`, `rcon.jsonl`, `report.md`), not published.
- **Tools.**
  - `tools/remote/mic-climate.sh` runs a mic-climate checkout on the Mac inside Deep Time's dev shell. It syncs its gitignored libs and the Destroy / petrolpark / mic-destroy-electric jars its build reads, and takes `--server` for Deep Time's lock.
  - `tools/review/p1-25-probe.sh` and `tools/review/p1-25-gates.sh`.
  - `tools/smoke/run.py` accepts `--rcon 'sleep N'`.
  - Resource note (Ben, 2026-09-29): the Project Atmosphere decompiles and the Serene Seasons lookups moved to the Mac. The first CFR runs had finished on the laptop before the order; nothing heavy has run locally since.
- **For Ben.**
  1. **Format 0.2 invalidates 0.1 worlds and workbench runs** in the new mod. A peer branch that bumps too (rivers?) must bump again at its merge.
  2. **Project Atmosphere's biome table has no Terralith entries**, so in the whole MIC pack (not only Deep Time worlds) Terralith-dominated regions sit near 0 °C in PA's own model. A pack-side `config/projectatmosphere/biome_temps.json` would fix PA's own visuals. That is a pack change, so it needs your decision.
  3. **Serene Seasons' global seasons disagree with Deep Time in the southern hemisphere**, in SS's own visuals and crops. Accept, or look for a per-hemisphere option later.
  4. **Asking PA's author (xGabou) for a base-temperature provider hook** would allow coupling (b), so PA's snow, clouds and crops use Deep Time's numbers.
  5. **mic-climate `deep-time`**: merge, release and a pack pin are yours to call. It compiles against Deep Time's jar, which is private until the MVP.
- **Gates after merging master 8872c91** (the mountain-character merge), on the Mac:
  - cargo fmt, clippy `--locked -D warnings` and deny are clean;
  - `cargo test --workspace --release` passes (core 262 + 3 ignored);
  - `./gradlew build test` gives tests=150 failures=0;
  - `./gametest.sh` (`--server`): **All 52 required tests passed**.
- **Next.** Merge this branch into master (the parent). Ben decides the "For Ben" items. Then mic-climate `deep-time` can be merged and released and pinned in the pack's deep-time branch.

### 2026-09-29 — Continuous strata across cells: stratigraphic keys, a correlating column rule, smoothed stacks; format 0.2 (agent worktree agent-aacd864d2709d52dd, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Why.** Decisions log 2026-09-29, "Fix now (Recommended)": cross-sections showed the host rock as vertical
  stripes one L1 cell wide. **Cause** (read from the code and the stacks): the column rule (`raster::stack_at`,
  Java `Columns.stack`) matched horizons by their *index* in each cell's stack and took every class from the
  *nearest* cell. Index h is a different unit in neighbouring cells — the stacks hold different units, and the
  bake merges thin ones and caps K differently per cell — so no bed could continue: classes switched at the
  nearest-cell line (the walls) and tops of unrelated units were interpolated into each other (the triangles
  under the P1-08 mountain). Bilinear tops were applied to every horizon; they could not help. Under that, the
  strata log itself differs a lot from one 16-block cell to the next (it is carried cell by cell through plate
  motion): neighbouring cells 4.8 km apart hold 60–340-block units of different classes. **The coal-stack
  picture** (`deeptime-inspect section`, site 44) is a different case: it is the synthetic deposit world
  (`deposits_world_16k.json`, `"strata": "every-class"`), whose horizon h of cell column i is class
  (6·i + h) mod 59 by design (P1-12's test pattern), so its stripes are the pattern, not the pipeline.
- **Design.** (1) **Stratigraphic keys** (`structure::columns::key`): every unit carries a u16 span that orders
  units the same way in every cell — regolith 0–4095; the strata log 4096 (now) … 32767 (the run's start),
  linear in age (a layer spans from the base of the layer above to its own base; the top layer to the present
  where the cell and its neighbours still accumulate at the last step, else only its base age; the young
  loose part of a layer the upper half of its span); the basement pools by erosion order, 4096 keys each, 512
  per metamorphic step; the mantle 61440–65535. Every merge in the bake keeps the union. (2) **The column
  rule** (`deeptime_format::strata`, Java `worlddata/Columns.Stack`): the 4 × 4 cells around a column with
  uniform cubic B-spline weights; each key's height is the weighted mean of the cells' key→height functions
  (linear inside a span, flat across a hiatus), so a shared bed is interpolated smoothly and a bed some cells
  lack thins out towards them; where cells disagree about the rock of the same keys (a facies change) the
  classes **intertongue** on a 64-key band grid every column shares (shares less a 0.15 soft threshold,
  stacked by key-span midpoint), so the answer is independent of how intervals are cut: a zero-weight cell
  changes nothing and columns are continuous across cell edges (a first per-run wedge version was not, and
  showed rectangles at cell lines). (3) **Smoothed stacks** (`Horizons::smoothed`, in `bake_with`): each
  cell's stack becomes the Gaussian mean (σ 2.5 cells, 11 × 11) of its neighbours' height functions cut at
  every key any of them has, each interval the class with the largest weighted thickness; the cell's own
  surface stays its first top (the P1-08 raise follows, unchanged). σ 1.5 still read as tight folds.
- **Format 0.2** (`Cargo.toml`, `FORMAT_MINOR`, Java `FormatVersion`): `horizon_stack` cells are 7K bytes (K tops,
  K classes, K key tops, K key bases; padding keys 65535); column JSON horizons are `[top, class, unit]`; the
  synthetic worlds give horizon h the span 4096·h … in every cell. FORMAT.md: the 0.2 note, the cell layout,
  the key space, the column rule step by step (the parity contract), "The 0.2 bake"; `notes.bake.strata.smoothing`.
  **Java:** `Columns.Stack` caches the 16 stacks of the L1 square it was last asked about (one read per chunk),
  grows as needed; `StrataPass` sizes its runs per chunk and picks a class's member by `horizonKey(unit, class)`
  = class + the key's era (top four bits: a bed keeps one block type across columns; a synthetic world's era is
  its horizon index, as before). `StrataGameTests.everyClassInTheWorld` samples only horizons with blocks to
  spare (the rule mixes neighbouring columns' classes in as thin tongues). **Tools:** `deeptime-inspect
  wd-section` (the world data's own cross-section by rock class, legend; no world needed), its `section`
  overlays use the column rule and skip tops above the ground; the viewer's sections pass `min_y`.
- **Before/after** (Mac; `deeptime run --preset earthlike_quick_16k --start scattered_continents --length-myr 500`
  on master dc8528e and on the branch — the same history; then `tools/workbench/preview.py run --profile
  standalone --world-data <bake> --box 2500,2202,2756,2458 --box -320,2312,-64,2568 --box -2900,-1988,-2644,-1732
  --keep-world` in each checkout and `deeptime-inspect section` along x and z through a coal basin, a 430-block
  mountain and a northern basin with coal fields). Before: walls one cell wide from the surface to the floor and
  wedges between them. After: beds run across the sections, pinch out, onlap a basement high, dip gently over
  the mountain's granite core; coal-seam bodies lie in the shale and sandstone. Review bundle (not published,
  584 kB): `review/out/strata-continuous/` (index.html; before, after and world-data-only sections of all six
  planes; hosts JSON).
- **Measured.** Strata pass painted share (export-area `strata`, rock below the first top that is its horizon's or
  body's class): before 98.57 / 98.93 / 98.97 % (86,585,695 of 87,609,999 blocks, **98.83 %**), after 98.49 /
  98.97 / 99.06 % (86,062,219 of 87,053,558, **98.86 %**). Column rule, the strata pass's per-chunk evaluation
  (`ColumnRuleBenchTest`, planet4k, 936 chunks, single thread, JIT warm): **0.048 → 0.744 ms/chunk** (mean
  horizons per column 3.6 → 7.6). Chunky on the loaded Mac (other agents' servers; wall, not comparable run to
  run): before 59 / 71 / 45 ms/chunk, after 104 / 95 / 56 ms/chunk for the three 361-chunk boxes — the rule's
  0.7 ms cannot explain the difference; the controlled number is the benchmark. Bake strata 0.1–0.4 s → 6.0 s
  (the smoothing), run 138 s → 149–170 s; mean horizons per cell 3.71 → 5.99; the bake's bytes 78.6 → 86.2 MiB.
  Deposit bodies are unchanged (their depths are the ledger's); the horizon class at ore-body centres
  (`deeptime inspect-column`, 354 bodies): coal seams in sedimentary rock 48.7 % → 56.2 %, placer 61 → 72 %,
  MVT 29 → 43 %, SEDEX 50 → 75 %, porphyry 50 → 44 %.
- **Gates** (Mac; run after merging master 00c02ce and again, all green, after merging master 5a70511 — cargo core 268, JUnit 147, both GameTest runs 50 of 50): `cargo fmt --check`, `clippy --workspace --all-targets
  --locked -D warnings`, `deny check licenses bans` clean; `cargo test --workspace --locked` all pass (cli 30, core
  256 + 3 ignored, format 44 incl. `strata::tests` — the same stack everywhere is that stack, B-spline weights, a
  shared bed smooth and a missing one pinching out with every column within a block of its neighbour, a facies
  change intertongues, a zero-weight cell changes nothing —, inspect 29 + 2 + 7, jni 7, viewer 5); `cd neoforge &&
  ./gradlew build test` → tests=144 failures=0 (`ColumnParityTest` 500 columns equal, `TilePackParityTest` checks
  the keys, `ColumnRuleBenchTest` 0.44–0.74 ms/chunk); `mac.sh --server -- ./gametest.sh` → **All 50 required tests
  passed**; `tools/smoke/install.sh mic` + `./gametest.sh -PwithMic` → **All 50 required tests passed**, 50
  `gametest-result` lines, 0 not PASS. `everyClassInTheWorld` first failed (59 wrong): it sampled horizons by
  index, which now reach the surface-rule layer and the bedrock floor in the every-class world; it samples by depth
  now (20+ blocks down, clear of the floor).
- **Decisions for Ben** (not asked): (1) ~44 % of coal-seam bodies sit in basement rock (granodiorite, granite,
  gneiss) before and after — the stack's depth comes from the ledger, not from the sediment thickness at the site;
  capping a coal stack at the sedimentary cover would fix it (deposit engine, not strata). (2) `deposits_world_16k`
  (the coal pictures, workbench previews of runs without a bake) uses the every-class test strata, which will
  always look striped (now as tongues); switching it to the placeholder strata would make its sections readable.
  (3) Taste levers, all constants for now: smoothing σ 2.5 cells, band 64 keys, minor share 0.15 — larger σ
  flattens the undulating beds further (the log's thickness really varies; Phase 1 has no folds, but thickness
  changes give dips up to ~30° over short distances). (4) The B-spline rule approximates: a cell point shows a
  blend of its neighbours, not its own stored stack.
- **Next.** Merge into master (the parent); then the coal-stack depth cap if Ben wants it.

### 2026-09-29 — Deep-time climate: carbon cycle, brightening sun, emergent snowball Earths, ice-age cycles, ice-linked sea level (agent worktree agent-adc924860a859865c, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Why.** Decisions log 2026-09-29 (the last two entries: "Greenhouse & sun over time", "Recent ice-age
  cycles", "Ice-linked sea level"; snowball Earth "emergent … not scripted"). Plan §8.3 has an "As built for the
  deep-time climate" paragraph (items 8–12).
- **The ice-field contract (committed first, 83b731d; FORMAT.md "The climate's ice fields").** World fields every
  plate step, before the surface step: `climate/ice_thickness_m` (land ice at the step's end), `climate/
  ice_max_thickness_m` (thickest during the step's fine clock; inside the ice-age window the maximum since it
  began, so at the present the **LGM**), `climate/ice_max_extent` (0/1 of the former), `climate/ice_time_share`
  (share of the step under ice), `climate/sea_ice` (sea-ice cover of the sea), and `climate/glacier` now means
  ice ≥ 1 m. History frames (surface's accumulator): `deeptime:ice_thickness_m`, `deeptime:ice_cover` (frame
  means; not deposit-engine fields, so 16-bit packed). Globals: `WorldState::globals["climate/sea_level_m"]` (the
  surface's sea level) and `["climate/cap_carbonate_m"]`.
- **What.** New `climate/{carbon,clock,orbit,ice}.rs`; `ebm.rs` (eccentric orbit with Kepler timing, forcing via
  A, sea-ice conductance tapering to zero between 5 and 10 m so the EBM's latent store melts in model years —
  the clock runs few EBM years per 100 kyr — early stop also waits for the ice, annual sea-ice cover out);
  `insolation.rs` (`Orbit`: (a/r)² and Kepler λ); `snapshot.rs` (forcing, sun, orbit, hydrological factor
  e^(0.02·(T_g − 15)) on precipitation, zonal refs from any EBM year); `mod.rs` rewritten around the clock.
  `WorldState::globals` (checkpoint **schema 7**). Surface: sea level from the globals; the drawdown carbonate
  laid on sea floor above 4 km; the ice fields into the frames. `deeptime stage --climate-json` writes the
  climate note (summary + curves); `climate-dem` prints the Earth weathering kernel and ice volume.
  - **Carbon** (`carbon.rs`): COPSE's 3.193·10¹⁸ mol reservoir split into air (4.96·10¹⁶ mol at 280 ppm) and a
    buffered ocean (A/A₀ = (O/O₀)^5; Archer et al. 2009); the air–sea exchange shuts under sea ice, so a
    snowball's degassing piles up in the air (350× in ~3 Myr at D₀, as Hoffman et al. 1998). Degassing D₀ =
    6.65·10¹² mol/yr × `degassing_scale` × (½ sea-floor production / 3.4 km²/yr (Rowley 2002) + ½ ocean floor
    consumed / 3.4 km²/yr — production minus the ocean floor's growth, smoothed 10 Myr; resolution-independent;
    a first try with Σ arc-cell area × convergence varied 13× with the L0 resolution) + LIPs (0.5 wt % CO₂ of
    their basalt over 1 Myr). Silicate weathering per land cell runoff × e^(0.09(T − 15)) (GEOCARB/WHAK), band
    changes × e^(0.09ΔT)(1 + 0.038ΔT)^0.65 × (1 − ice share), × (CO₂/280)^0.3, normalized so Earth's relief under
    the model's present climate at 280 ppm weathers D₀ (`KERNEL_EARTH` 2.845·10⁷, from `climate-dem`).
    Forcing 5.35 x below 1,000 ppm, Byrne & Goldblatt 2014's 5.32 x + 0.39 x² above 2,000 ppm (checked in their
    Table 2 by a Sonnet lookup: it gives their 38.1 W/m² at 50,000 ppm), blended between. Gough 1981 sun on the
    timeline's age (`present_age_gyr` 4.57). Implicit carbon step (bisection), halved down to 1 kyr where CO₂
    would move by more than half (a first version overshot to 33 ppm after an escape and refroze).
  - **Clock** (`clock.rs`): per plate step, 100-kyr steps (2 kyr in the window): degassing, carbon, EBM (≤ 10
    years, warm), each band's ELA shift from its summer land temperature, the ice (a callback), weathering,
    trackers. Snowball = annual sea-ice cover ≥ 0.95 of the ocean, ends < 0.5; hothouse = global mean > 25 °C;
    events `deeptime:snowball` (onset/escape CO₂, drawdown factor, tropical land and weathering shares,
    degassing, sun, causes `weathering_drawdown`/`slow_cooling` and `degassing_buildup`, the escape step's cap
    carbonate) and `deeptime:hothouse` (caused by its snowball) — EVENTS.md. First step: a starting snapshot,
    then CO₂ bisected to weathering = degassing on the warm branch (`initial_co2_ppm` 0) or the preset's value.
    Between snapshots the zonal refs follow the clock's EBM and the published ELA = snapshot ELA + band shift.
  - **Orbit** (`orbit.rs`): last `orbital_window_myr` (3): obliquity 41 (+53.6) kyr ±1.2°, eccentricity 405/95/
    124 kyr (0–0.055), perihelion 23 kyr, phases so t = 0 is today's (ε at the preset's tilt, e 0.018, ϖ 283°);
    `orbital_strength` scales the swing and e (0 = circular: no cycles).
  - **Ice** (`ice.rs`): Nye's plastic profile √(2τL/ρg) (τ 50 kPa) on the ice mask via an exact separable EDT
    (Felzenszwalb & Huttenlocher, from the paper), Earth-normalized lengths, capped at 3,000 m (4,000 gave ~2 km
    snowball sea-level falls against Hoffman et al. 2017's ~1 km); Imbrie & Imbrie lag in the window (growth
    42.5 kyr, decay 10.6 kyr); glacio-eustasy −(ρ_i/ρ_w)ΔV/A_ocean (2.47 m per 10⁶ km³) against
    `ice_reference_km3` 1.0·10⁷ (the calibrated Earth-like runs' present ice: 0.74–1.28·10⁷; the model puts 2.5·10⁷
    on Earth's relief). **Sea level = the preset's curve (its 250- and 36-Myr cycles kept as the tectonic part)
    + the ice's glacio-eustasy** ("alongside").
  - **Cap carbonate**: the carbon that weathering removes beyond degassing (the drawdown's alkalinity) is laid as
    carbonate on sea floor above 4 km (calcite 3.69·10⁻⁵ m³/mol); outside drawdowns it is millimetres.
  - Removed after trying: snowball dust darkening (Abbot & Pierrehumbert 2010) — with Byrne & Goldblatt's
    forcing the ice melts on CO₂ alone; `climate.snowball_dust_*` never shipped.
- **Settings** (editor `adv_climate`, lang keys, all four pipeline presets + built-ins synced): `carbon_cycle`,
  `initial_co2_ppm`, `degassing_scale`, `weathering_strength`, `solar_brightening`, `present_age_gyr`,
  `orbital_window_myr`, `orbital_strength`, `glacio_eustasy`, `ice_yield_stress_kpa`. Preset-only (physics,
  numerics, thresholds, per the catalogue's rule): `weathering_temp_per_k`, `runoff_temp_per_k`,
  `weathering_co2_exponent`, `hydro_sensitivity_per_k`, `hydro_ref_c`, `carbon_step_kyr`, `ebm_clock_years`,
  `snowball_on/off_frac`, `hothouse_c`, `orbital_step_kyr`, `ice_max_thickness_m`, `ice_response_kyr`,
  `ice_reference_km3`. `tools/dev/knob_extremes.py --stage climate --length-myr 100` on the 8 numeric knobs: 16 of
  16 ok (degassing 0 freezes the planet for good: land −63 °C, all ice — as it must without volcanoes).
- **Measured** (Mac, heavily shared: load 60–110; gates re-run on the tree merged with master 8bdbedc: core 280 + 3 ignored, Gradle 146/0):
  `cargo test --workspace --release` all pass (core 274 + 3 ignored; new: carbon ODE balance and mass
  conservation, sealed build-up 350× in 2–4 Myr, Gough, forcing forms, orbit ranges, Kepler annual mean, EDT
  exact, Nye/eustasy numbers, Imbrie asymmetry, band margins; clock: Earth-like 40 Myr stays 5–25 °C with no
  snowball, a tropical supercontinent snowballs under a weathering ramp and escapes by degassing (CO₂ ×20+) into a
  hothouse caused by it, orbital cycles ≥ 20 swings in 3 Myr on a cold planet and none at strength 0 or when warm;
  coupled pipeline: every ice field, glacier = ice ≥ 1 m, the surface's sea level = the climate's, the curve and
  window rows, weathering ≈ degassing at the end, frames carry the ice, resume on 3 threads from 1 identical
  incl. ice fields, curve and globals), clippy `-D warnings`, fmt, `cargo deny` clean; `./gradlew build test`
  tests=146 failures=0 (`AdvancedSettingsTest` now round-trips toggles). **Earth check** (`climate-dem`, Earth
  1° relief): EBM 15.28 °C, glaciated 9.9 %, 19 of 24 cities in the right group (unchanged), Earth ice 2.47·10⁷ km³.
  **Earth-like 500-Myr runs** (`stage climate --preset pipeline_quick_16k`, scattered seed 1 / supercontinent /
  scattered seed 3): present T 14.5 / 14.3 / 14.0 °C, CO₂ 217 / 210 / 219 ppm, glaciated land 8.1 / 6.6 / 7.7 %,
  Köppen A .17/.14/.17 B .27/.26/.24 C .21/.16/.18 D .17/.24/.16 E .18/.20/.25, sea level 27 / 29 / 32 m (curve 25),
  **no snowball**; over the run CO₂ 157–2,665 ppm, T 10.4–17.8 °C, glaciated land 0–18 %, sea level −174…+132 m,
  degassing 0.66–2.2× Earth; the window's land ice swings 4.9–8.9 / 4.2–7.3 / 4.4–8.5 %. **Snowball scenario**
  (supercontinent, 1,000 Myr, `degassing_scale` 0.3): one snowball −807.3 → −760.6 Myr (46.7 Myr), onset after a
  3.7× CO₂ drawdown to 321 ppm with 87 % of the weathering in the tropics (`weathering_drawdown`), T −38 °C,
  sea level −1.48 km, escape at 5.5·10⁵ ppm (`degassing_buildup`, ×1,720), a 200-kyr hothouse at 36.6 °C, 16.8 m
  of cap carbonate; present 12.1 °C, 116 ppm, 22 % glaciated, window swings 13.6–22.6 %. The same scenario on the
  code one commit earlier (cap carbonate on ≤ 200 m shelves: 180–570 m, too thick) gave 6 snowballs (74, 68,
  40, 35, 31, 26 Myr), each refreezing ~1 Myr after its hothouse; degassing 0.3 from scattered continents: 3;
  supercontinent at 0.4: none — near the threshold the history is chaotic. Stage cost: climate 108 s vs plates
  114 s on 3 loaded threads (P1-05: 12 s vs 37 s): the clock's ~6,500 EBM runs (serial) ≈ 45 %, ice ≈ 17 %.
- **Review bundle** `review/out/deeptime-climate/` (1.8 MB, not published; `_shot.png` is the page as rendered):
  CO₂, T, sea ice, land ice, ice volume, sea level, degassing, weathering, sun over the run for the snowball
  scenario and two Earth-like runs; the window's ice, 65° N summer, obliquity, eccentricity; maps (elevation,
  ice, sea ice, temperature) before, at, during, at the end of and after the snowball. Looked at: the runaway
  and the escape are clean; the snowball's continents grow by their exposed shelves.
- **For Ben** (not asked; the report lists them): (1) glacial cycles are ELA-driven (no ice–elevation feedback
  or ablation-zone flow), so Earth-like swings are ~4 % of the land and ~10–15 m of sea level, not Earth's
  10 → 25 % and 120 m; (2) post-snowball drawdown has no supply limit, so low-degassing planets can refreeze
  within ~1 Myr (Earth's Cryogenian interlude was 9–19 Myr); (3) snowball sea-level falls reach ~1.5 km and the
  surface keeps eroding under the ice and on exposed shelves (glacial erosion should stop fluvial erosion under
  ice — the glacial-erosion agent's surface work); (4) escape needs ~0.5 bar CO₂ (the land ice keeps its albedo;
  Pierrehumbert's GCM needs > 0.2 bar, Hoffman et al. 2017 quote 10⁴–10⁵ ppm); (5) `ice_reference_km3` is the
  Earth-like runs' present ice (1.0·10⁷), not Earth's own (2.5·10⁷) — Earth's would raise present seas ~35 m;
  (6) the climate stage now costs ~3× what P1-05 did (`carbon_step_kyr` 200 would halve the clock).
- **Next.** Merge (the caller lands it). The glacial-erosion agent reads the FORMAT.md ice fields; P1-25's
  climate API could expose CO₂ and the sea level.

### 2026-09-29 — Block-scale mountain character, ruggedness setting, the striped-snow fix (agent worktree agent-a2d4bbb2f592ce1d7, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Why.** Decisions log 2026-09-29 ("Do it now, in Phase 1"): P1-08's 6.8 km peak was a smooth dome, and its
  snowfield had vertical stripes.
- **The striped snow, found and fixed.** Not a resampling artefact of the rasters: the noise router. The final
  density takes `deeptime:terrain` only within 1.5625·k blocks (k = 8: 12.5 blocks) of the surface and the cave
  field below (vanilla's `range_choice` around `sloped_cheese`), and vanilla interpolates between corners 4 blocks
  apart. On a face steeper than ~3 blocks per block the lower corner's cell mixed the upper corner's cave value
  (≈ 0.5 instead of the terrain's 2–4) into the interpolation, so the surface jumped up-slope: 3-block benches and
  20-block steps every 4 blocks. P1-08's world, x 2976..3000 at z −960 (`deeptime-inspect column`): tops 344, 337,
  308, 308, 307, 287, 284, 283, 282, 275, 256, 254, 254, 233, … — each bench snow-capped, one N–S stripe per 4
  blocks on the east flank. **Fix** (`worldgen/TerrainScale`, `TerrainDensity`): the density's scale doubles from
  k until it covers 0.8 × the largest height difference to the 8 neighbouring corners (powers of two keep the
  interpolation exact at whole-number heights; a 9.6 broke `lakeHolds`' floor by one block), and a new mode
  `ratio` (k/scale) multiplies the router's cave-entrance term, so cave mouths stay where they were
  (`planet.json`, and `tools/dev/gen_worldgen.py` regenerates it byte for byte). Corner columns are H exactly as
  before; parity unchanged. `-Ddeeptime.steepScale=0` restores P1-08's router (the "before" renders use it).
  Measured on steep noise cells (drop ≥ 16 over the cell, `stairs.py` over real exports): one step taking ≥ 70 % of
  the drop on **43 % / 28 % of cells before, 11 % / 4 % after** (boxes A / C); median max-step/mean-step 2.6 → 1.2.
- **The mountain character** (`crates/deeptime-core/src/surface/amplify/character.rs`, run on the amplifier's last
  level after its passes, `tiles.rs`): all baked into `elevation`, so the Java height, the column evaluator and
  the world-data contract are unchanged (format 0.1; FORMAT.md "The mountain character").
  - *Drainage-aligned gullies:* 4 octaves of stripe noise (128 → 16 blocks) whose crests follow the slope of the
    smoothed terrain (so the amplifier's multiple-flow valleys), each octave aligned with what the coarser left
    (tributaries). Depth = 0.2 × slope × wavelength, capped by 0.35 × local relief, 0.25 × wavelength (hard rock;
    60 % on soft) and the height above the local base level; faded on rivers (5–100 km² drainage) and at the coast.
  - *Fold ridges:* ridged stripes along `plates/fold_trend_deg` where `plates/orogeny_type` records an orogen
    (Himalayan 1.0, Andean 0.4, × youth), 0.18 × local relief; gaps where streams (1–10 km²) cross.
  - *Young vs old:* youth = exp(−`plates/orogeny_age_myr`/60) or uplift; young terrain gets the convex part of an
    unsharp mask (sharper, higher summits and ridgelines; one-sided, so no moats at mountain feet), old terrain is
    pulled towards its smoothed self (rounded); gully profiles are triangles when young/hard, cosines when old/soft.
  - *Cliffs:* a smooth terrace (8-block benches) where the rock is hard (`young/erodibility` ≤ ~1.3: granite,
    gneiss, andesite, basalt, quartzite) and the slope over 0.6.
  - *Drains:* a pit the character closes deeper than 1.5 blocks is softened back (radius 6 cells), then the flood
    runs again: 0 new sinks; on the 16k planet the last flood raises 10k land cells by more than a block (8 by more
    than 10), and softens 11.7 % of land cells somewhat.
  - `amplify.ruggedness` (0..2, default 1; editor field `ruggedness`, section Terrain, stops Smooth 0 / Gentle 0.5 /
    Natural 1 / Wild 1.6, lang key; all four `pipeline_*` / `earthlike_*` presets carry 1.0) scales everything;
    0 is the P1-06 terrain bit for bit. The tuning knobs are `amplify.character.*` (every key optional).
    `surface.drama` (thermal slope + the chunk-time A·N texture) and `amplify.incision_cap` (how deep the amplifier's
    rivers cut) stay beside it, unchanged: they shape the simulated terrain, ruggedness shapes the block-scale
    character on top of it.
  - Keyframes/workbench: `amplify/youth`, `amplify/fold` (L1, ramp `unit`). Notes: `notes.amplify.character`
    (cells changed, mean/max change, flood, softened, ms); the CLI's `amplify_final=` line prints them.
  - Tiles: halo + the character's radius (≈ 36 cells) + two flood margins + 13; streamed = in memory to the bit
    (the existing tests, now with the character on). Memory: +4 B a cell (71 B/cell whole-raster).
- **Where a glacial pass plugs in** (the glacial-erosion agent, `amplify/glacial.rs`): in `tiles::run_tiles`, after
  `run_passes` and before `character::apply` — the character then puts spurs and gullies on the glacial valleys'
  walls and its `fade` (character.rs) is the place to take a glacial mask (ice-smoothed rock: less ruggedness, no
  folds). Its halo adds to `Config::halo`.
- **Numbers (Mac M6, shared).** `deeptime run --preset earthlike_quick_16k --start scattered_continents
  --length-myr 500 --out …/mountains/after --checkpoints` → 109 s wall; amplify 13.1 s of which the character
  3.8 s (12 threads); mean |change| on land 2.6 blocks, max +53/−50; sinks 0. Chunk generation (standalone, the
  same three 384 × 384 boxes = 2,187 chunks, Chunky wall time, interleaved runs `t-after-1, t-before-1, t-after-2, t-before-2`): **after 30.9 / 31.0 ms per chunk, before 41.2 / 54.6** — no slowdown (the grown scale adds 8 memoised height lookups per corner column; the rest is load on the shared Mac, 8–22). P1-08 measured ≈ 43 ms (441 chunks, loaded Mac).
- **Evidence** (`review/out/mountains/index.html`, not published, 12 MB): before/after of three areas of that planet through the
  real generator (preview jobs, exports rendered isometrically and in the area3d viewer): A arc volcano (andesite,
  active; 2752..3136 × −1152..−768), C young collision belt (−512..−128 × 2320..2704), O old collision belt (422 Myr;
  5648..6032 × −2384..−2000); the same with the MIC set; vanilla (standalone `minecraft:normal`, seed 20260929:
  jagged and stony peaks) and MIC's Terralith + Tectonic (`terralith:rocky_mountains`, `yosemite_cliffs`); before/after
  sections; the ruggedness sweep 0/0.5/1/1.6; the youth and fold maps. BlueMap renders were made
  (`review/out/mtn-bm/bm-after`, 139 MB) but its web camera distorts 450-block-tall boxes; the bundle uses the
  exports instead.
- **Tools.** `deeptime heightmap` (H, E, A·N and layers of a box as .npy), `deeptime stage … --bake` (bake after a
  stage that finishes the run, for trying a later stage's settings on the game's terrain), preview.py
  `--level-type` (a comparison world in the pack's own generator) with `--locate BIOME[@x,z]`, `--jvm-prop`,
  `--max-side` passed to the export; ScreenshotDriver `.start/.length/.vista` properties and
  `mac-screenshots.sh --prop`.
- **Gates** (tree merged with master 00c02ce): cargo fmt, clippy `--locked -D warnings`, deny licenses bans
  clean; `cargo test --workspace --locked` (cli 30, core 262 + 3 ignored incl. character:: 6 and amplify 25 with the
  character on — streamed = in memory, 1 = N threads, no new sinks —, format 40, inspect 29 + 2 + 7, jni 7, viewer 5);
  `./gradlew build test` 146/0 (`ColumnParityTest` 500 of 500 columns equal on the regenerated planet4k fixture,
  `TerrainScaleTest` 3); `./gametest.sh` 50/50, `-PwithMic` 50/50 (MIC set installed today), `./gametest-planet.sh`
  PASS. **lakeHolds** needed two changes on the way: the scale must be a power of two times k (a 9.6 put the lake
  floor one block high at 442 corners), and under MIC a short plant at the waterline, which the new wall shape put
  by the water, washed away into a 65,124th source — the test now allows such washable blocks (it counts and logs
  them, `washable=1`) instead of calling them leaks.
- **Real client** (`tools/client/mac-screenshots.sh --prop …`, the ScreenshotDriver's new `.start/.length/.vista`):
  one run on the Mac's desktop, 27 min of in-client simulation on the loaded Mac; its four vistas came out empty
  (sky only): the timeline properties were applied on only one of the driver's two paths (the planet was the default
  supercontinent/1000 Myr one, so the coordinates missed the mountains) and 60 s did not generate terrain 200 blocks
  from spawn. Both fixed after the run (the Advanced path reads them too; a vista waits until the chunk 160 blocks
  ahead exists, then 15 s, at most 170 s); not re-run. `review/out/mtn-client/` has the walk's shots.
- **Deferred / for Ben.** Talus and scree at cliff feet (a surface-block rule the column evaluator would have to
  mirror for parity); overhangs/3D crags (Phase 2). The softening touches 11.7 % of land cells a little.
- **Next.** Merge into master (the parent).

### 2026-09-29 — EF is ice cap where the glacier model says ice, barren frozen ground elsewhere (agent worktree agent-adda9513719f24370, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Why.** A workbench preview of run 1a0ef97aaca (`earthlike_quick_16k`, MIC, box x 3675…3931, z −3779…−3523, 77–83°N, EF
  almost everywhere) came out 58 % `terralith:scarlet_mountains`, snowy slopes, `terralith:snowy_badlands`, frozen cliffs. Ben first
  answered "Yes, EF means ice" and then, when told EF only means "warmest month below 0 °C" and includes ice-free cold desert
  (Dry Valleys), **"Ice only where there's ice (Recommended)"**: where the glacier/ice-sheet model says ice accumulates, ice biomes;
  EF too dry for ice, a frozen but barren biome (bare rock, gravel, thin snow; no ice spikes or packed ice; not Scarlet or a warm
  or red biome); ET unchanged; all in the placement file (decisions log 2026-09-29; the coordinator writes that entry).
- **Why the plateau was not "flagged glaciated": it was, and the flag is not the problem.** Reproduced the run (same preset, seed 1,
  supercontinent, 500 Myr; 53,837 EF L1 cells, as `notes.bake.climate.koppen_bands`) and read `climate/glacier`, `ela_m`,
  `precip_mm`, `t_warm_c` from the climate checkpoint: **46,026 of 53,837 EF cells (85.5 %) have glacier = 1, every cell of the
  plateau in the box included** (L1 rows 19–34, `glacier` = 1; ELA 660–810 m, summer sea-level temperature about −2 … 0 °C, the
  ground 1–4 km). Precipitation there is the `precip_min_mm` floor (10.0 mm/yr on the plateau: 2,744 EF cells at ≤ 10.5 mm, 6,432
  more under 30 mm), and **every one of those 9,176 cells is glaciated**: the ELA rule (Ohmura et al. 1992 as P1-05 solves it,
  T_ELA(P) = −2.3 °C at P → 0) has no cutoff for dryness, so any polar land whose summer stays under about −2.3 °C is ice
  wherever it lies above its ELA. Not a bug: East Antarctica is ice at 20–50 mm/yr water equivalent; the fit is extrapolated
  below its data (its lowest ELAs sit at a few hundred mm) but nothing better is wanted. The 7,811 unglaciated EF cells (14.5 %)
  are the other kind of EF: median t_warm −0.3 °C (barely EF), median ground 738 m under an ELA of 784 m, 81–89°N, precipitation
  44–120 mm (p10–p90): low-lying, marginal polar land, the Dry Valleys' kind. ET: 433 of 53,111 cells glaciated. **What the
  flag did in P1-09:** only capped the temperature band at 0.35, which is a no-op where the class already sits at band 0 (EF's
  polar ramp gives 0 at t_warm ≤ −20 °C: temperature −1.0, the bottom of the frozen band, glaciated or not). So the biome the pack
  drew there was decided by the other inputs, not by the flag. **Found on the way:** P/PET is **pinned at 10** for every EF cell
  (Thornthwaite PET is 0 below freezing), so ET/EF humidity was always the rule's cap (1.95) and never followed the climate.
- **What the pack does with the frozen band** (probed, not assumed): vanilla `OverworldBiomeBuilder` on `test_planet_real`
  (GameTest probe, standalone) and Terralith 2.6.2's parameter list (`data/minecraft/dimension/overworld.json`, 1,713 entries;
  a Python emulation of `Climate.ParameterList` that reproduces the real MIC run to 0.1 %). Tectonic adds no biomes (router
  only). Vanilla in the frozen band: ice spikes only at humidity band 0 with positive weirdness, snowy slopes at humidity ≤ 1,
  grove above it, snowy taiga at humidity 3, jagged peaks (weirdness < 0) / frozen peaks (> 0), snowy plains between; the
  temperature within the band does not matter; **vanilla has no dry frozen biome** (closest: snowy plains, `windswept_gravelly_hills`
  at erosion 0.45–0.55). Terralith splits the band at temperature −0.727 (coordinate 0.496): **below it every dry cell
  (humidity < −0.35) is `snowy_badlands`** (white and orange terracotta, red sand: what EF had drawn: 58 % of glaciated EF on
  the planet), above it the dry cells are `gravel_desert`, `rocky_shrubland`, `cold_shrubland` (gravel, granite, coarse dirt, tuff:
  the barren frozen biomes); `scarlet_mountains` takes every peak with weirdness −0.93 … −0.4 and humidity above −0.35 (no
  temperature or humidity within the frozen band avoids it); `frozen_cliffs` only on the coast band; `glacial_chasm` (packed
  ice) only at erosion ≥ 0.4 (≥ 0.565 whatever the humidity); `ice_spikes` at humidity < −0.35 near the coast (continentalness ≤ 0.03,
  else nearest fit). Ice biomes Terralith lacks: any inland ice-sheet biome on low or mid erosion.
- **Design (candidates swept offline over the planet's glaciated and unglaciated EF cells on the Terralith list; the
  scripts are in the session scratchpad, not committed).** The levers the placement table has are the temperature and humidity
  band coordinates; the result: temperature 0.55 … 0.9 all alike (upper frozen band, above Terralith's split) and humidity
  fixed low for ice, fixed 1.5 for barren:
  | planet-wide EF, Terralith list | ice | snow | barren | other |
  |---|---|---|---|---|
  | glaciated, before (t 0, h 1.95) | 5 % | 32 % | 0 % | 63 % (snowy badlands 59 %, scarlet 5 %) |
  | glaciated, now (t 0.6–0.8, h 0.5, weirdness +) | 41 % | 26 % | 31 % | 2 % (cherry grove) |
  | not glaciated, before | 2 % | 15 % | 2 % | 81 % (snowy badlands 80 %) |
  | not glaciated, now (h 1.5) | 26 % | 5 % | 69 % | 0 % |
  (ice = frozen/jagged peaks, ice spikes, glacial chasm, frozen cliffs; snow = snowy plains and slopes, beaches, frozen ocean;
  barren = gravel desert, cold/rocky shrubland). Humidity band 2 and up turns the ice sheet into wintry forest, grove and snowy
  taiga (trees), so ice cap is band 0.5, barren 1.5 (below the band whose variants are taiga and grove, as P1-09's 1.95 cap
  was). The scarlet peaks are not reachable from temperature or humidity at all, hence the third lever.
- **Change.** (1) `placement.json`: ramp `polar_ice` (0.6 … 0.8 by the warmest month, −20 → 0 °C), fixed humidity rules
  `polar_desert` (1.5) and `ice_cap` (0.5), and EF = `{temperature: polar_ice, humidity: polar_desert, glaciated: {humidity:
  ice_cap}, weirdness: positive}`; ET untouched. (2) Schema (`bake/placement.rs`, FORMAT.md "The climate mapping", no schema
  version bump: both keys optional): a class may have a **`glaciated` block** (its own temperature and/or humidity where the glacier
  flag is set; it replaces `glacier_cap` for that class, each key falling back to the class's ordinary one) and
  **`weirdness: "positive"`** (the class's land keeps the positive branch of the variant noise: vanilla's frozen peaks, never the
  negative branch's jagged peaks / Terralith's scarlet mountains). `humidity_land` takes the glacier flag; the bake resamples the
  class's weirdness flag L1 → L2 like the humidity (`Coords.w_pos`). Every other class, ET included, is bit-identical
  (the "default reproduces the compiled constants" test now covers all classes but EF, and asserts a glaciated `humidity_land` equals
  the plain one for them). (3) GameTest `climateConsistent` gains a negative check (`BiomeCensus.FORBIDDEN`, `EF:no_red`: EF columns
  on `snowy_badlands`, `scarlet_mountains`, vanilla badlands ≤ 10 %).
- **Result on the workbench box** (`preview-area.sh --profile mic --world-data <rebake> --keep-world`, then `deeptime-inspect biomes`
  on the kept world; 65,121 EF columns of the chunks' stored biomes; the pre-change kept world is jobs/20260929-195758-82h8):
  | biome | before | after |
  |---|---|---|
  | scarlet mountains | 58.3 % | 0 |
  | snowy badlands | 18.4 % | 0 |
  | frozen peaks | 0 | 58.3 % |
  | snowy slopes | 21.3 % | 21.3 % |
  | gravel desert | 0 | 18.4 % |
  | frozen cliffs | 2.0 % | 2.0 % |
  The emulation predicted 60.8 / 20.7 / 17.8 / 0.7 → 60.8 (frozen peaks) / 20.7 / 17.8 (gravel desert) / 0.7. The glacier flag did not
  change anywhere (the climate stage is untouched; continentalness and erosion layers are bit-identical, the weirdness magnitude is
  identical and 131,822 L2 cells, all under EF or filled from it, changed sign). The box is one sign region of the 1,024-block variant noise (all negative), so
  the whole scarlet share becomes frozen peaks; elsewhere half of the EF peaks were already frozen peaks.
- **Caveats (for Ben).** (a) 18 % of this box, and 31 % of glaciated EF planet-wide, is `gravel_desert` (bare gravel on ice
  cap): the flat, far-inland glaciated cells, where Terralith has no ice biome at humidity band 0 and, at humidity ≥ 2, only trees;
  raising the ice-cap humidity to ≥ 2.05 trades it for wintry forest and grove. (b) 21 % of unglaciated EF gets Terralith's
  `glacial_chasm` (packed ice; erosion-driven, unreachable from temperature or humidity); vanilla gives snowy plains and slopes
  there (no dry frozen biome exists). (c) `snowy_cherry_grove` (pink, snowy) on about 2 % of glaciated EF at weirdness > 0.93 /
  0.27–0.4. (d) EF temperature is now 0.6–0.8, warmer in band coordinate than ET's coldest (0.5): only Terralith's split at 0.496
  reads it (vanilla's band is one). (e) EF humidity no longer follows P/PET (pinned at 10 below freezing); ET's still does.
- **Gates** (Mac, this branch): `cargo fmt --check`, `clippy --workspace --all-targets --locked -D warnings`, `cargo test
  --workspace --locked` (core 258 + 3 ignored, cli 30, format 40, inspect 29, …), `cargo deny check licenses bans` clean;
  `./gradlew build test` 143/0; `./gametest.sh` all 50 passed (`climateConsistent`: E 1.0, E:snow 0.941 against its 0.9 floor: ET's
  1,765 columns are unchanged code, EF has 29 columns on the small planet, `EF:no_red` 0 of 29); `-PwithMic` all 50 passed
  (E 1.0, E:snow 1.0, `EF:no_red` 0 of 29; EF there: frozen peaks 59 %, frozen cliffs 31 %, snowy slopes 10 %). **Re-run after
  merging master 00c02ce** (USGS provinces, coherent-continents decisions): fmt, clippy `-D warnings`, `cargo test --workspace`
  (core 262 + 3 ignored, cli 30, format 40, inspect 29), deny, `./gradlew build test` 143/0, `./gametest.sh` 50 of 50 and
  `./gametest.sh -PwithMic` 50 of 50.
- **Next.** Ben's call on caveat (a) (ice-cap humidity band 0.5 vs ≥ 2.05); the coordinator merges (this branch is not merged).
  ET is untouched (Scarlet stays on 3 % of ET, as Ben's "keep it" allows outside EF).

### 2026-09-30 — Geologic provinces: a USGS-style view layer on every keyframe (agent worktree agent-a03fc35a652ae22e4, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **What.** Ben's decision (decisions log 2026-09-29, "Just want the view, don't change world generation"): a
  `provinces` keyframe layer, drawn like `regime` or `crust_age`, so the workbench (layer picker, legend, globe, compare)
  and the static viewer show it at every time step. View only: `core::provinces` derives it when a frame is drawn
  (`keyframes::render` takes derived layers; `keyframes::add_derived` adds `provinces` to a group whose layers hold
  `regime` and `crust_age`) into a scratch field; nothing is written to `WorldState`. Also: `deeptime provinces --in
  <checkpoint> --out x.png` (prints each class's share; `DEEPTIME_PROVINCE_STATS=1` prints field quantiles,
  `DEEPTIME_PROVINCE_DUMP=<dir>` dumps the fields as raw f32), the `provinces` ramp in `ramps.json` (9 classes, `labels`,
  the rules as its `_comment`, which the workbench legend shows and the static viewer shows as a tooltip; existing ramps
  untouched), the workbench's layer word "Geologic provinces", the plan's §8.1 note. The plates estimate counts one more
  keyframe layer (16 frames × 2 MiB), so `notes.run.history.reserved_peak_bytes` grew by 23 MB.
- **Rules** (first that applies wins; thresholds `provinces::Rules`, the plan's §8.1 note has the full text):
  LIP = mafic volcanic pile ≥ 0.2 × the recorded LIP thickness above the column's own basalt (2 km where the gabbro
  lower crust is 5 km, as on oceanic crust; none before the first LIP event); oceanic crust by `crust_age` 0–20 / 20–65 /
  over 65 Ma, an island arc (regime arc, crust over 9 km) being an orogen; continental: orogen = collision or arc regime,
  a Himalayan-type orogeny within 500 Myr, or an Andean one within 50 Myr on ground over 1.5 km; extended = rift or
  passive-margin regime or crust under 20 km; basin = over 3 km of sediment + sedimentary rock (strata log or pools,
  the thicker) or subsiding over 0.03 mm/yr under ≥ 1 km; shield = under 0.2 km of cover or a crystalline surface rock
  (`structure/rock`, else `surface/class`, by the class table's family); platform = the rest. Then a 3 × 3 majority
  filter over the continental classes.
- **What calibration found.** `earthlike_quick_16k` scattered_continents, 500 Myr, final: continental crust is 35 % of
  the planet and on it: `plates/orogeny_type` is Andean on 87 % of the cells (age median 20 Myr, also under regime
  craton), so "orogeny within 500 Myr" made whole continents orogens (24 % of the planet at first); the collision type
  is 11 % and is the one that persists. Continental columns carry 2 km of mafic volcanic rock wherever the crust was
  oceanic or an emerged arc (the 4.5 km gabbro of a continent against 5 km of an ocean tells them apart), so a plain
  "mafic pile over 300 m" flagged 8–14 % of the planet as LIP; with the baseline it is 1.1–2.1 %, about the area of
  the run's LIP blobs. Oceanic arc-regime cells sit at a median −2.3 km (a thin rim around every continent); only the
  ones with crust over 9 km (23 %) are island arcs. Continental crust is 28 km thick at the median, so the 28 km
  "thinned" guess made half of it extended; 20 km is the 10th percentile of cratons.
- **Measured** (Mac, release; the checkpoints from `stage plates … --max-steps 125` and the full stage):
  class shares of the planet, mid (step 125) / final: shield 5.6 / 4.3 %, platform 8.8 / 4.5 %, orogen 22.7 / 11.9 %,
  basin 5.8 / 6.3 %, LIP 1.1 / 2.1 %, extended 4.3 / 7.2 %, ocean 0–20 Ma 16.4 / 18.9 %, 20–65 Ma 17.9 / 26.6 %, over
  65 Ma 17.6 / 18.4 %. By eye against the USGS map: shields and platforms fill continental interiors, orogens follow the
  collisions and active margins, basins sit beside them, extended crust fringes margins and rifts, LIPs are round blobs,
  the sea floor ages away from the ridges. Differences from Earth: too little platform against shield (the sim's
  craton cover is patchy: median 0.8 km, 25 % over 3.9 km), and orogens are over-large at mid-run because the six
  scattered continents all collide.
- **Gates.** `cargo test --workspace` ok (core 256 + 3 ignored, incl. 5 provinces:: tests: every rule, LIP with and
  without the event, the derivation leaves the world equal, the generalisation, ramp labels and colours); clippy
  `--workspace --all-targets -D warnings` ok; `cargo fmt --check` ok; `cargo deny check licenses bans` ok; all four
  re-run after merging master 553ecda (docs only): the same results.
  **World files vs master** (`run --preset earthlike_quick_16k --start scattered_continents --length-myr 500 --threads
  6`, master 032c94c binary against this branch): `diff -rq` differs only in `keyframes/index.json`, the 11 plate-group
  `kf-*.dtk` and `manifest.json` (created_utc, the keyframe files' bytes/crc, timings, `reserved_peak_bytes`); every
  raster, strata, event, site, body, spawn file is byte-identical.
- **Screenshots** (private `deeptime watch --port 8653` on the Mac, own runs folder, reached through an ssh tunnel;
  the live 8642 was not touched): `review/out/prov/wb-mid.png` (234 Myr ago, step 133) and `wb-final.png` (the present),
  the workbench with the Geologic provinces layer, its legend and note; `mid.png`, `fin.png` are the bare maps.
  (review/out is not committed.) The tool: `tools/viewer/shot.py` with `--eval` clicking `#lpickbtn`, then
  `[data-layer=provinces]`, then Shift+← ten frames at a time.
- **Would the world's provinces be better with these classes?** Probably yes, for TerraBlender: `meta/province` is the
  regime setting (ocean, island, arc, orogen, rift, hotspot, craton, highland, margin) with connected regions of at least
  50,000 km²; it has no sedimentary/crystalline distinction, so a craton's shield, platform and basin all become one
  province (one biome region), and it makes every hotspot and LIP a small one. Provinces of shield/platform/basin/LIP
  would give regions that follow the rock the player sees and the ore the deposit stages place (basins hold the coal,
  petroleum and MVT, shields the BIF and orogens the porphyry). Not changed here (Ben: view only); if wanted, it is a
  change to `bake/meta.rs` plus P1-10's region table, and it would change `meta/province` and the biome regions of
  every world.
- **Next.** Nothing blocks: merge to master when reviewed. Not checked in the game: the in-game timelapse builds its
  layer list from the frames' layer names (`TimelapseModel`) and the frames are already coloured, so `provinces` should
  appear in its picker (under its raw name) with the right colours; no Java changed.

### 2026-09-29 — Coal fields adjoin and stack: adjoining footprints, several seams per field from the sediment history (agent worktree agent-a3e41a57dc812f499, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **What.** Ben's follow-up (decisions log 2026-09-29, "Yes, adjoining + stacked (Recommended)"): coal fields in a basin
  may adjoin and fill its minable area (no spacing), and each field holds a stack of seams at different depths. Changes:
  `deposits/cluster.rs` (`clustering.model: adjoining`, `ellipses_overlap`, `Point.ellipse`), `deposits/geometry.rs`
  (`measures_thickness_m`, `seam_plan`, `stack_bodies`, `footprint_blocks`), `deposits/types.rs` (`Stack`,
  `Geometry.stack`, validation), `deposits/mod.rs` (placement passes footprints; `materialise` builds the stack; the
  playability-guaranteed coal site is a six-seam stack; census `seams`, `seams_max`, `measures_m`; forming events carry
  `seams` and `coal_measures_m`), the schema (`clustering.model` enum, `geometry.stack`), `coal_seam.json`, FORMAT.md,
  Java `compat/SiteChunks.java`, the plan's §8.6 note. Other types are untouched (their tests and counts are unchanged).
- **Adjoining.** No camps, `min_separation_m` 0. Each field, largest first, is drawn anywhere in the tract (∝ F·E, up to
  400 tries) where its plan ellipse crosses none placed before; the yaw is drawn at placement and the body uses it. Two
  ellipses are disjoint when a direction u exists with u·(pb − pa) > h_a(u) + h_b(u) (support functions); 360 directions
  and a 0.1 %-of-the-smaller-axis margin make "not overlapping" exact for the placed bodies (checked independently in
  the test by sampling each boundary against the other ellipse).
- **Stacked seams.** One body per seam (role `ore`, consecutive ids from the top, one footprint, yaw and dip), so the
  world-data layout is unchanged (a site already owned several bodies; the Java deposit feature, strata pass and
  `LevelSites` already loop over all of them) and the format version stays. The count comes from the recorded history:
  coal measures under the site S = Σ_k F_k · deposition_k · Δt_k over the frames its crust existed in (F recovered from the
  favourability hazard the placement was drawn from), n = round(S / 25 m) clamped to 1–40 and to cumulative thickness ÷
  0.7 m (Circular 891's thinnest minable bed); gap mean S/n, `gap_sigma` 0.6; tonnage split by thickness (`share_sigma`
  0.7). `spacing_mult_v` 6 enlarges the gaps in blocks (25 m is 1.25 blocks at 20 m per block). The top seam is half the
  measures above the ledger's centre depth; seams above the ground are dropped from the top, the one crossing it is
  clipped, and the site's tonnage is what is left. Ledger and survival stay per field (one thin equivalent body).
- **Research** (Sonnet sub-agent, explicit model; a second one after the first, for stacking). Real numbers with URLs
  are in the type's notes; the ones the agent marked (calc) are its own arithmetic and the paywalled ones (Heckel 1986,
  Hampson 1999, the Silesia MDPI paper, the Ruhr counts) come from search snippets. Measures per seam 12–60 m (Allegheny
  ~15, Illinois 11–29, Ruhr 12 all coals / ~60 economic, Silesia ~35, Jharia ~19, Illawarra ~27, Highveld 16–60); up to
  ~40 mineable seams in a section (Llewellyn 40, Donets 30–40 workable, Illinois ~20 mined of 75 named); cyclothems
  160–400 kyr, so about one minable seam in two to four cycles; cumulative minable coal per field 4.6–20 m in most
  basins (Illinois, Allegheny, Highveld, Illawarra, Witbank), ~30–46 m in the Powder River's merged zone, ~100–140 m
  in Jharia and Silesia.
- **Tonnage rebased (contradiction with the previous entry, and why).** The 540 / 2,500 / 10,000 Mt used *one* bed's
  thickness (Circular 891's classes); a field of ~11 seams outgrows that, and a stack needs the *cumulative* thickness so
  that seams stay minable (≥ 0.7 m). Now cumulative 5 / 14 / 60 m × area 300 / 1,000 / 3,000 km² × 1.32 = **3,800 /
  19,800 / 115,000 Mt** (about 15 Mt per km² at the median, between Silesia ~13 and Jharia/Raniganj ~110–130); the aspect
  (1, 5.15e-4, 1.5) keeps the median body at 1,000 km² and 15 m of coal (29 × 44 km, 49 × 73 blocks semi-axis), P90 334
  km² × 8.7 m, P10 3,230 km² × 27 m. COE's vein amount follows the tonnage quantile, so amounts do not change.
- **Counts** (Mac, the same six `stage structure` checkpoints as the previous entry, `stage deposits --in`):

  | coal | first entry (before) | seam fields, camps + 26 km spacing | adjoining + stacked |
  |---|---|---|---|
  | sites, median (range) | 47.5 (25–56) | 17.5 (10–30) | **20 (13–31)** |
  | counted, median (range) | 23.5 (17–35) | 12.5 (4–22) | **14 (6–19)** |
  | counted, six landscapes summed | 148 | 75 | **80** |
  | target N\* (counted), summed | 178.3 | 83.1 | 83.1 |
  | formed / dropped for overlap | 681 / 34 | 344 / 76 | 330 / 44 |

  Per landscape (counted / target, sites): scattered 1–3 16 / 18.1 (26), 15 / 14.4 (20), 6 / 7.1 (13); supercontinent
  1–3 13 / 11.9 (20), 11 / 11.2 (17), 19 / 20.4 (31). Counted survivors are now 96 % of the target (75 before: 90 %);
  sites still run 17 % under `expected_sites` (127 of 153): 44 of 330 formed fields find no room (the formed ones outnumber
  the survivors 3–6× and cannot overlap in plan, while in a real basin an old field would be buried under a later one:
  the same limit as before, smaller). Stacks: 1,460 seams over 127 sites, 11.5 per field on average, the most 38; the
  coal measures under a site 287–861 m on average per landscape (many are held to 15–20 seams by the 0.7 m minimum:
  cumulative thickness ÷ 0.7).
- **Checks.** Tests: `geometry::tests` (measures = Σ F·deposition·Δt by hand; the count follows the measures, the type's
  clamps and the minable thickness, shares sum to 1, gaps average S/n, deterministic per seed; a stack's bodies are
  disjoint, ids consecutive, volumes follow the shares, erosion removes seams from the top and clips the crossing one,
  nothing is left far above the ground; the median field is ~1,000 km² and tens of km across), `cluster::tests`
  (`ellipses_overlap` on circles, aligned and quarter-turned ellipses and across the seam; 40 seeds of adjoining fields
  in a tract with no overlap by independent boundary sampling), `recorded_tests::coal_fields_adjoin_without_overlap_and_hold_stacked_seams`
  (a shrunken coal on the synthetic planet: consecutive `ore` bodies, one footprint, apart, no two fields overlap). The
  existing 1-vs-3-thread test (sites, bodies, events, census identical) and the resume tests cover determinism.
- **Java.** `SiteChunks.compute` used to sample one box over all ore bodies' union (a stack's box would be a tall mostly
  empty volume and the lattice would coarsen past the 1-block seams); it now samples each ore body over its own bounds and
  adds (ore bodies of a site are disjoint), the lattice set by the bounds' total volume. The deposit feature, strata pass
  (`Columns`), `LevelSites.at` and the GameTests' block-by-block counts already loop over all of a site's bodies. Codec:
  Java reads only `products`, `minecraft` and `sizing.tonnage_mt` of a deposit type, so `geometry.stack` and
  `clustering.model` need nothing there (like `hosts.full_at`).
- **Render.** A cross-section of a real generated coal field, headless: `tools/workbench/preview.py run` on the Mac
  (`--profile standalone`, the synthetic deposit world `deposits_world_16k.json`, box 1930,−1790..2186,−1534, real mod and
  generator, `--keep-world`) then `deeptime-inspect section --axis z --at -1661 --from 1935 --to 2180 --y-from -140
  --y-to 20 --scale 4`: site 44, 15 seams in 130 blocks, a thick top seam and thin ones below with the speckle of
  60 % ore density, clipped at the top by the surface (the PNG is in `review/out/coalsection/`, gitignored).
- **Gates** (Mac, on the branch after merging master c35f56f): `cargo test --workspace --locked` all pass (core 252 + 3
  ignored, cli 30, format 40, …), clippy `-D warnings` clean, fmt clean, `cargo deny check licenses bans` ok;
  `cd neoforge && ./gradlew build test` → tests=143 failures=0; `tools/remote/mac.sh --server -- ./gametest.sh` → All 50
  required tests passed (`coePicksSiteVein` counts the coal stack's ore blocks per chunk block by block against
  `SiteChunks`: 11,557 blocks in one chunk of site 51).
  `tools/smoke/install.sh mic` + `./gametest.sh -PwithMic` (168 MIC mods, COE and CDG live) → Gradle BUILD SUCCESSFUL,
  48 `gametest-result` lines, 0 not PASS (`coepickssitevein`, `coedropsclaimednoise`, `coeamountscaleswithtonnage`,
  `cdgoilfromtrap`, `cdgzeroelsewhere` all PASS; the coal site's 11,557 ore blocks per chunk match `SiteChunks`); the
  "All N required tests passed" line did not print in that run (the MIC run's known end-of-run behaviour), so the count is
  from the result lines.
- **Decisions for Ben** (not asked): (1) the tonnage rebase to cumulative thickness above (7.9× the median of the previous
  entry). (2) A coal site is now ~11 bodies with ~1.4 blocks of coal each, spaced ~7 blocks apart over up to ~150
  blocks of depth, under a 29 × 44 km footprint; `spacing_mult_v` 6 is a playability lever (1 keeps the real 25 m).
  (3) Formed-but-eroded fields still cannot overlap in plan (the 13 % of formed fields dropped); letting them would
  raise sites to the target's `expected_sites`.
- **Next.** A look at the coal in the workbench 3D view (`tools/workbench/view.sh` on the export) if Ben wants one; the
  overlap of formed-but-eroded fields above.

### 2026-09-29 — MIC GameTest heap stall: the suite's scattered samples, not lakeHolds (agent worktree agent-a11ef8b35a6ebb8ee, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Ben's decision** (decisions log 2026-09-29, "Chase the stall (Recommended)"). Resolves the P1-12 known issue
  (`-PwithMic` stalls in `lakeHolds` and runs out of heap; 4 of 4 runs at P1-09).
- **Reproduced** (Mac, `tools/smoke/install.sh mic` → 168 mods, pack 4d0e685-dirty, then `./gametest.sh -PwithMic`):
  4 of 4 runs failed as before. In three, 48 tests passed within ~20 s of the batch start, `lakeHolds` sat in
  its 400-tick wait, the heap climbed to 8 GB and C2ME's workers threw `OutOfMemoryError` 40–80 s later; the
  fourth ran out of heap during the batch start itself. `jcmd` could no longer attach once the heap was full. A watcher on the Mac (`jcmd GC.heap_info` / `Thread.print` every 2 s, `GC.class_histogram`
  above 3 GB) caught it on the way up.
- **Evidence.** (1) Class histogram at 5.1 GB live: 68,488 `LevelChunkSection[]` (one per chunk), 4.1 M
  `LevelChunkSection`, 8.0 M `SingleValuePalette`/`ZeroBitStorage` (empty sections, i.e. proto-chunks), 12.4 M
  `DataResult$Success` and 4.5 M `CompoundTag` (chunk serialization): ~68,000 chunks resident. (2) Thread
  dumps: the server thread inside test start functions — `GeneratorGameTests.surfaceAtNoise` ←
  `heightMatchesReference`, `RealPlanetGameTests.realPlanetHeightMatchesTheBake`, `BiomeGameTests.surfaceRulesApplied`
  — each in `ServerChunkCache.getChunk` → `managedBlock`; after the batch, C2ME's `ReadFromDiskAsync.downgradeFromThis`
  → `asyncSave` serializing the chunks as they unload. No frame in `lakeHolds`, no fluid ticking. (3) A new
  chunk census (`ChunkCensus`, below) on the standalone run: peak **51,793** chunks; `heightMatchesReference`
  alone added 41,408 (C = 16,384 `test_planet`), `realPlanetHeightMatchesTheBake` 21,463 (`test_planet_real`),
  `periodicAcrossSeam` 9,614, `realPlanetRockMatchesTheParityFixture` 5,356; `lakeHolds` holds 1,428
  (`test_lake`, forceloaded) — small. (4) `lakeHolds` alone under MIC (`./gametest.sh -PwithMic
  -PgametestOnly=lakeHolds`): passes in 6.2 s, heap 2.3 GB, before=after=expected=65304, outside=0.
- **Root cause.** A chunk requested at NOISE or later needs STRUCTURE_STARTS within radius 8
  (`ChunkPyramid.GENERATION_PYRAMID`), so a sample far from the others loads up to 289 proto-chunks. A
  `getChunk` ticket lasts one tick and nothing unloads inside a tick, and the sampling tests took all their
  columns synchronously in their start ticks (the batch starts its 49 tests within ~40 ticks), so the suite's
  whole footprint was resident at once and then unloaded (and serialized) at once. Standalone fits its 6 GB
  (peak ~5.6 GB); the MIC server (C2ME's async chunk system and serialization, the pack's structure sets)
  carries more per chunk and runs 8 GB out. `lakeHolds` is the one test still running (its 400-tick wait) when
  that happens, and `worldInfoFollowsThePlayer` is in the next batch: that is why exactly those two never
  reported. P1-08's real-planet level (~27,000 more chunks in the census) is consistent with the step from 3 of 7
  runs to every run.
- **Fix (test design, bounded per tick).** `gametest/ChunkSampling`: `forEach(helper, n, step, finish)` runs a
  test's samples at most `STEPS_PER_TICK` = 8 per server tick, under one budget shared by the whole GameTest
  server (a vanilla `GameTestSequence`: wait until sampled, then assert); a step that loads no chunk costs
  nothing; an assertion inside a step fails the test after the rest are dropped. Each tick's chunks unload
  before the next ones load, so the resident set is bounded by the budget (~8 × 289), not by the suite.
  Every sample is still taken at the same status; only the timing changed. Converted: `heightMatchesReference`,
  `periodicAcrossSeam`, `realPlanetHeightMatchesTheBake`, `realPlanetRockMatchesTheParityFixture` (a shared
  `HeightTally` for the three height comparisons). The C = 1,024 samplers (`heightMatchesRaster`,
  `strataClassesResolve`, `surfaceRulesApplied`, `everyClassInTheWorld`) stay synchronous: that planet is
  small enough (≤ 3,540 chunks in the census) that they are bounded by it. Neither the heap nor the test list
  changed.
- **Also.** `gametest/ChunkCensus` logs `[gametest] chunks peak=… heapPeakMB=… liveHeapPeakMB=…` at the end of every
  GameTest run (and `[gametest-chunks]` lines per passing test / new peak). Under C2ME `getLoadedChunksCount`
  counts only promoted chunks, so for MIC the heap numbers are the measure. `-PgametestOnly=<name>[,…]`
  (`GameTestServerMixin`, `-Ddeeptime.gametestOnly`) runs only the named tests. `gametest.sh`'s peak-RSS
  sampler matched nothing (the game directory sits in ModDevGradle's @argfile) and printed 0 MB; it now
  matches the JVM by its `-Dfml.modFolders=deeptime%%<checkout>/neoforge/…` argument (`ps -ww`). Class names:
  `ChunkSampling`, `ChunkCensus`.
- **Measured** (branch merged with master 7f907ce; Mac; one `tools/remote/mac.sh --server` loop of five
  `./gametest.sh -PwithMic` then `./gametest.sh`):

  | run | result | wall | heap peak (in use) | live heap peak | JVM RSS peak |
  |---|---|---|---|---|---|
  | MIC 1 | All 50 required tests passed | 81 s | 4,949 MB | 3,326 MB | 6,365 MB |
  | MIC 2 | All 50 required tests passed | 62 s | 5,874 MB | 4,368 MB | 7,691 MB |
  | MIC 3 | All 50 required tests passed | 52 s | 5,459 MB | 3,144 MB | 7,161 MB |
  | MIC 4 | All 50 required tests passed | 58 s | 5,788 MB | 4,179 MB | 7,431 MB |
  | MIC 5 | All 50 required tests passed | 51 s | 5,623 MB | 3,519 MB | 6,487 MB |
  | standalone | All 50 required tests passed | 36 s | 3,578 MB | 1,344 MB | 4,866 MB |

  (heap max 8,192 MB MIC / 6,144 MB standalone; "live" = G1 old generation after a collection.) Standalone chunk
  peak 51,793 → 15,094. `lakeHolds` identical in every run: before=after=expected=65304, outside=0,
  water_top_y=120. Before the fix: 4 of 4 MIC runs out of heap (8 GB) here, plus P1-09's 4 of 4.
- **Gates** (merged tree): cargo fmt, clippy `--locked -D warnings`, deny licenses bans clean; cargo test --workspace
  (core 239 + 3 ignored, …) ok; `./gradlew build test` tests=140 failures=0. No Rust changed on this branch.
- **Found on the way.** A `tools/remote/mac.sh --server` command that dies of a zsh expansion error (an
  argument starting with `=`) skips the `EXIT` trap and leaves `server.lock` behind (mine did once; I removed
  my own stale lock after checking no process of this checkout held it). The stuck JVMs of the failed runs
  were killed with `kill -9` on their verified PIDs (cwd = this checkout's `run-gametest`).
- **Next.** New GameTests that sample columns across a planet should go through `ChunkSampling.forEach`; the
  `[gametest] chunks peak` line shows a test's footprint.

### 2026-09-29 — P1-09 follow-up: every biome-mapping constant is configurable (agent worktree agent-a506f027d5a6b70e1, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Why.** Ben, 2026-09-29 (decisions log): "All of them" — the inland continentalness distances, the wetland thresholds and
  the band placement per Köppen class that P1-09's "For Ben (2)" listed as constants.
- **Settings** (preset section `biomes`, optional keys with the old values as defaults; the pipeline and built-in presets carry
  them, resynced with `sync_builtin_presets.py`; range checks name the key; one new **Advanced · Biome placement** section
  `adv_biomes`, after the other advanced ones, in `editor_fields.json`, the game's `EditorFields.Section.ADV_BIOMES` and lang):

  | id | key | default | range |
  |---|---|---|---|
  | Mid-inland distance (`inland_mid`) | `biomes.inland_mid_blocks` | 200 blocks | 160–800 |
  | Far-inland distance (`inland_far`) | `biomes.inland_far_blocks` | 1,000 | 900–3,000 |
  | Continental interior distance (`inland_interior`) | `biomes.inland_interior_blocks` | 5,000 | 3,500–20,000 |
  | Swamp humidity threshold (`wetland_humidity`) | `biomes.wetland_humidity` | 2.5 bands (full at +1.5) | 1–4.5 |
  | Swamp flat-land share (`wetland_flat_share`) | `biomes.wetland_flat_share` | 0.35 | 0.05–0.9 |
  | Swamp height limit (`wetland_lowland`) | `biomes.wetland_low_m` | 150 m | 20–600 |

  The three inland ranges do not overlap (and the beach, ≤ 150, stays below the mid-inland edge), so the edges stay in order
  whatever the player sets. The wetland gate `s > 0.15`, the thaw condition and the sea's depth anchors stay in the core.
- **The placement table** is data: `crates/deeptime-core/data/climate_mapping/placement.json`, beside `default.json`; the jar
  ships it as `data/deeptime/deeptime/climate_mapping/placement.json` (one added `include` in `neoforge/build.gradle`'s
  climate-mapping copy: the only edit to a Gradle file) and the core compiles it in (`include_str!`, `bake/placement.rs`). Schema
  (FORMAT.md, "The climate mapping"): `glacier_cap`; named `temperature_ramps` (`base + span · Σ weight · clamp01((t − from)/(to − from))`
  over `t_mean` / `t_warm` / `t_cold`, weights adding to 1); named `humidity_rules` (`squeeze` [lo, hi] over band 1…4, then `min`,
  `max`); `classes`: each of the 31 land classes a ramp by name (or a `{when: {of, above}, then, else}` switch, as BSk needs) and a
  humidity rule. Ramps use `base`/`span`, not two end points, because 2.95 − 2.0 ≠ 0.95 in f64 and the default must reproduce the old
  constants bit for bit. The default has six ramps (polar, boreal, temperate, tropical, hot_steppe, desert) and nine rules
  (rainforest, monsoon, savanna, desert, steppe, mediterranean, humid, taiga, tundra).
- **How a pack's table reaches the bake.** The bake is Rust and runs before the game can read a datapack, so the table travels
  in the **content bundle** (the JSON passed with the preset since P1-15, key `climate_placement`). At a dedicated server's first
  start `FirstStartGenerator` reads `deeptime:deeptime/climate_mapping/placement.json` from the server's resource manager (the
  highest-priority pack's copy; the jar's own when nothing overrides it; a file that is not a JSON object stops the start with the
  pack named) and adds it to the bundle (`climate/ClimatePlacement`, pure part `PlacementBundle`, JUnit `PlacementBundleTest`);
  `Preset::with_content` parses and checks it (`Placement::from_value`, errors carry the path, e.g.
  `$.climate_placement.classes.Af.temperature`) and keeps it on `PlanetPreset.placement` (not in the preset JSON, so share codes
  and `preset.json` are unchanged); the bake places the climate by it and records the whole table in the world data,
  `notes.bake.climate.placement = {source: "default" | "content bundle", table}`. Java never interprets the table again (the world's
  values are already baked with it). `deeptime run --content <file>` takes the same key. **Not covered:** the Create World editor's
  runs and the workbench pass no content bundle (as with the essentials), so they use the core's default; a pack override applies to
  dedicated-server first starts. A singleplayer world made through the editor keeps the default table until the editor learns to pass a bundle (it
  needs the datapack registries of the world being created).
- **Bit-identical.** `planet_world_4k` (C = 4096, 150 Myr, `deeptime run --threads 6`, release), master 8cf16f9 vs this branch:
  `diff -rq` of the two world folders reports only `manifest.json`; every raster, event, keyframe, strata, sites, bodies and
  `preset.json` are identical. The manifest differs in timings and `created_utc` and in the record: `notes.bake.climate.params` has the six new
  keys and `notes.bake.climate.placement` is new. Also `placement::tests::the_default_table_reproduces_the_compiled_in_constants_bit_for_bit`
  (the old formulas kept in the test as an oracle; every class, 35 temperatures × 3 × 7 month sets × glacier, 400 aridities: identical bits).
- **Extremes** (`tools/dev/knob_extremes.py --stage run`, new: `deeptime run` with the bake, reads `notes.bake.climate` back and checks
  the biome inputs' land shares are finite and add up to 1; Mac, `--jobs 3 --threads 4`). Small planet (`planet_world_4k` with a `biomes` section, 150 Myr),
  all 7 biome knobs at both ends + the default: 15 of 15 ok. Wetland cells (default 53): humidity 1.0 → 921, 4.5 → 0; flat share 0.05 → 0, 0.9 → 843;
  height 20 m → 20, 600 m → 202. The small planet has no land beyond ~1,000 blocks from the sea, so the inland knobs ran on
  `pipeline_quick_16k` (100 Myr, scattered continents, 9 runs, 0 failed; land shares of the seven continentalness bands, near / mid / far-inland =
  the last three): default 0.303 / 0.628 / 0.037 (the coast band holds 0.032); mid-inland edge 160 → 0.239 / 0.692 / 0.037, 800 → 0.875 / 0.056 / 0.037;
  far edge 900 → 0.303 / 0.607 / 0.059, 3,000 → 0.303 / 0.666 / 0.000; interior edge 3,500 and 20,000 → unchanged shares (that edge only sets the
  top of the last band's scale: it changes the values inside it, not the band shares); height 20 m → 6 wetland cells, 600 m → 9,207 (default 842).
- **Tests.** Rust: `bake::placement` (bit-for-bit default, a changed table changes the answers, every bad table names its path), `bake::biomes`
  (`a_changed_placement_table_changes_the_biome_inputs`: Af moved to the desert ramp and the tundra rule changes the temperature and humidity layers
  of that quarter only and nothing else, and the note records the table; `the_inland_distances_and_wetland_thresholds_are_settings`; range checks).
  Java: `PlacementBundleTest` (the jar's copy is the core's file; the bundle carries a pack's table), the catalogue tests count six advanced
  sections, the tuned share-code case now carries a `biomes` section with all twelve keys non-default (`share-codes.json` regenerated on the Mac with
  `share_code_fixture.py`: Rust and Java write the same code; `AdvancedSettingsTest.aTunedPipelineCodeLoadsUnchanged` and the workbench's
  `a_tuned_code_from_the_game_loads_unchanged` read every biome field back and the JSON unchanged).
- **Gates** (Mac, run on the branch, then again after merging master 7f907ce, coal seam fields: same numbers): `cargo fmt --check`, `clippy --workspace
  --all-targets --locked -D warnings`, `deny check licenses bans` clean; `cargo test --workspace --locked --no-fail-fast`: cli 30, core 245 (+3 ignored),
  history_cap 1, format 40, inspect 29 + 2 + 7, jni 7, viewer 5, 0 failed; `cd neoforge && ./gradlew build test`: tests=143 failures=0 (+3);
  `./gametest.sh --server` (both times): **All 50 required tests passed** (`climateConsistent`, `bandsAreTheBuilders` among them). Not run: `-PwithMic` (the MIC stall is another agent's).
- **Deviations.** The plan put "anchors in a datapack file"; P1-09 had made them the preset's section, and the placement per class is now that file. The
  ramp form is `base`/`span` (above). `mapping` in the note stays `"p1-09"` (Java checks it).
- **Left as constants:** the sea's temperature curve, the aridity → humidity anchors, the depth anchors of the sea, the wetland gate and the E6 band's
  internal steps.
- **Next.** The editor could pass the content bundle (essentials and this table) once it can read the datapack registries of the world being created.

### 2026-09-29 — Coal seam fields: a deposit is one minable seam field, density 4 / 14 / 40 per 100,000 km² (agent worktree agent-a3e41a57dc812f499, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **What.** Ben's decision (decisions log 2026-09-29, "OK; define coal as seams (Recommended)"): coal's survivor-based
  counts stay; a deposit is one minable seam field, tens of km across, with its density sized from coal-basin area
  statistics and labelled an estimate. Only `deposit_type/coal_seam.json` changed (plus the plan's §8.6 as-built note and
  two notes in `data/usgs/coal.json`; no schema change, no code): `density_per_1e5km2` 10 / 30 / 80 → **4 / 14 / 40**
  (`density_basis: estimate`, so the P50 14 is used), `tonnage_mt` 4.2 / 13 / 25 → **540 / 2,500 / 10,000 Mt**, `aspect`
  (1, 0.001, 1.5) → **(1, 6.5e-5, 1.5)**, `balance.size_mult_h` 6 → **1**, `min_separation_m` 2,400 → **26,000**,
  `clustering` camps 20 / 8 km → **100 / 40 km**. The derivation and every source are in the type's `notes` and the plan.
- **Research** (Sonnet sub-agent, explicit model; web search plus the USGS PDFs it downloaded). No source counts minable
  seam fields; every count is a judgement, and the type's notes list what the research did not verify. Density = fields ÷
  tract area × 10⁵: Powder River (Fort Union, 57,000 km², 6–9 named coalfields, PP 1625-A chap. PS/PG) 10.5–15.8; Sydney
  Basin onshore (36,000 km², five coalfields) 14–17; Upper Silesia (5,600 km², three coal areas) 54; Illinois Basin
  (~95,000 km², three fields ≥ 30,000 km²) ~3; India (~113 coalfields on 3.29 M km² of land) 3.4 (a floor); geometric check
  10⁵ · f / A (f minable share, A field area 300–3,000 km²) 3–167. P50 14 = Powder River and Sydney; P90 4 = Illinois-style
  large fields; P10 40 = Silesia-like. Field size 300 / 1,000 / 3,000 km² (Gillette study area ~4,150 km², Gondwana fields
  280–2,200 km², Pennsylvania anthracite fields mean 313 km²; Averitt, USGS Bull. 1412; Circular 891 glossary "coal field").
- **Body sizes changed, because they contradicted the definition.** The old block was 1.2 km radius (Circular 891's
  indicated radius) with tonnage to match, drawn 6× wider (`size_mult_h` 6): 12 × 18 km across at the median for a
  13 Mt "deposit", i.e. neither a mine block nor a field. Now the tonnage is field area × thickness (0.71 / 2.13 / 4.27 m,
  Circular 891's thickness classes) × 1.32 t/m³, combined by simulation, and the body is the field with no enlargement:
  median 1,000 km² × 1.9 m = 29 × 44 km across = 97 × 146 blocks at `m_per_block_h` 300 (semi-axes 49 × 73); P90 360 km² ×
  1.1 m, P10 2,500 km² × 3.0 m (the engine scales all axes by the cube root of volume, so the area spread is 7× against the
  sourced 10×). `size_mult_v` 20 stays (a 2 m seam is 2 blocks). COE's vein amount follows the site's tonnage *quantile*
  within the type's distribution, so the 100× tonnage does not change amounts by itself.
- **Counts** (Mac, `stage deposits --in` the six `stage structure --preset pipeline_quick_16k --start
  {scattered_continents,supercontinent} --length-myr 500 --set seed={1,2,3}` checkpoints made on this base; the "before"
  numbers are master 8cf16f9's binary on the same checkpoints and reproduce the earlier STATUS table):

  | coal | before | after |
  |---|---|---|
  | sites, median (range) | 47.5 (25–56) | **17.5 (10–30)** |
  | counted (within 1,800 m), median (range) | 23.5 (17–35) | **12.5 (4–22)** |
  | target N\* (counted) | 15.2–43.8 | 7.1–20.4 |
  | expected sites (all) | 30.5–75.0 | 14.2–35.0 |
  | drawn (formed), total of six | 681 | 344 |
  | dropped for separation, total of six | 34 | 76 |
  | permissive tract | 50,598–145,979 km² (unchanged) | same |

  Per landscape (sites / counted): scattered 1–3 56/35 → 25/15, 50/31 → 17/12, 25/18 → 10/4; supercontinent 1–3 41/20 →
  18/13, 45/17 → 17/9, 56/27 → 30/22. Sites now come in 7–39 % (median 25 %) under `expected_sites`: fields tens of km across cannot
  overlap, and the formed fields (target ÷ preservation, 17–37 %) crowd small tracts (scattered 3: 35 formed into 50,600 km²,
  8 dropped by the sum-of-radii rule of `cluster::place`, one of six landscapes with counted 4 against target 7.1). That is
  the size and the 100 km camp spacing at work, not a bias in the survivor formula; if Ben wants sites to sit closer to N\*,
  the lever is `clustering.camp_spacing_km` or letting formed (not surviving) fields overlap in the engine.
- **Gates** (Mac, on the branch after merging master b1cbc97): `cargo test --workspace --locked` all pass (core 239 + 3
  ignored, cli 30, format 40, …), clippy `-D warnings` clean, fmt clean, `cargo deny check licenses bans` ok;
  `cd neoforge && ./gradlew build test` → tests=140 failures=0; `tools/remote/mac.sh --server -- ./gametest.sh` → All 50
  required tests passed. Scratch scripts `coal-gen.sh` / `coal-dep.sh` (untracked, not committed) made the checkpoints and ran
  the before/after counts; the checkpoints are in `/Volumes/Graphide/ban/deeptime-runs/coal/` on the Mac.
- **Decisions for Ben** (not asked): (1) the size_mult_h 6 → 1 reading of "tens of km across" (physical field size, no
  enlargement); the other types keep their 6–12. (2) Coal is now the type with the biggest bodies: about 100 × 150 blocks and
  2 blocks thick per site, 10–30 per planet. (3) P50 14 sits below the geometric median of 25; using the geometric case
  would double the counts.
- **Next.** Ben reviews the coal fields in a viewer bundle if he wants the look; the survivor/overlap question above.

### 2026-09-29 — P1-09 climate mapping and biomes (agent worktree agent-a116e3d557a130679, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **What.** (1) **The mapping, in the bake** (`deeptime-core/src/bake/biomes.rs`; replaces P1-08's provisional
  anchors). Physical climate → the five biome inputs in *band coordinates* against a reference,
  `crates/deeptime-core/data/climate_mapping/default.json`: OverworldBiomeBuilder's band edges and vanilla's
  land distributions of erosion and peaks-and-valleys (measured by `ClimateMappingBuilder` on the vanilla
  router: seed 0, 65,536 points with continents ≥ −0.19 of 91,405). **Temperature** from the Köppen class
  and the warmest/coldest month: E 0–0.75 by the warmest month, D 0.75–2 (Dfc's cold edge on snowy taiga),
  C 2–2.95 by the coldest month, A and BSh 3–3.9 (never the hot band: jungle stays jungle), BW 4.05–4.95
  (desert/badlands, cold deserts included), BSk as C or D; glaciated land ≤ 0.35; the sea by its own
  temperature. **Humidity** from the aridity index P/PET (0.03, 0.2, 0.5, 1, 2, 5 → 0…5, log scale) held in
  each class's range (BW ≤ 0.95, BS 1.05–1.95, Af ≥ 4.1, Am ≥ 3.2, Aw/As/Cs 1.05–2.5, subarctic D ≥ 3.05,
  other C/D ≥ 2.05, ET/EF ≤ 1.95). Both on L1, filled across the other medium by nearest value, resampled
  to L2, chosen by the L2 land mask. **Continentalness** from distance to the sea (0, beach, 200, 1,000,
  5,000 blocks) and depth (0, 200, 2,500, 6,000 m). **Erosion**: relief (range over the terrain window,
  box-smoothed) ranked over land → the reference's land quantiles below E6; wet flat lowlands into E6 by
  `wetlands`; deep sea ≥ E4 (away from Terralith's sky-island band). **Weirdness**: ridge index ranked →
  peaks-and-valleys quantiles above the valley slice → |w| on the rising branch, signed by a value noise;
  no river biomes (Phase 2). `notes.bake.climate`: mapping `p1-09`, params, reference, per-Köppen mean band
  coordinates, land shares per band. (2) **Knobs as settings** (standing rule, log 2026-09-29): preset
  section `biomes` (`temperature_shift`, `humidity_shift`, `beach_blocks`, `terrain_window_blocks`,
  `variant_blocks`, `wetlands`; optional keys, range checks naming the key); the four pipeline presets carry
  it, built-ins resynced (`sync_builtin_presets.py`); `editor_fields.json` gains a `biomes` section with six
  fields (lang keys; `EditorFields.Section.BIOMES`, `SetupPage`; the workbench test lists them).
  (3) **Per world in the game** (`climate/{OverworldBands, ClimateMapping, ClimateMappingBuilder}`): at a
  P1-09 world's first start `WorldDataBinder` (with the server's registries) builds the mapping from the
  pack's bands (its copy of `data/deeptime/deeptime/climate_mapping/default.json`, copied into the jar by
  Gradle; a datapack may override it) and its own `minecraft:overworld` router (65,536 land points), five
  monotone tables, written as `<world data>/climate_mapping.json` and read on every later start.
  `WorldDataTerrain.raster` applies them; in P1-09 worlds the weirdness is sampled as an odd field
  (`RasterSampler.bilinearOdd`: |w| bilinear, sign of the nearest cell), because a bilinear ramp through 0
  where the variant noise changes sign drew straight lines of river biome (found in the first 16k render:
  1,184 river columns in the BW area, 0 after). Synthetic worlds keep their values. FORMAT.md "The climate
  mapping" is the contract; plan §10 P1-09 has an "As built". (4) **Checks and tools:** `climate/BiomeCensus`
  (surface biome from the biome source vs the Köppen class; per family the share whose biome carries a
  matching `c:` tag, plus the narrow checks Ben asked for); `BiomeGameTests.climateConsistent` (floors) and
  `bandsAreTheBuilders` (the reference edges = OverworldBiomeBuilder's, by reflection); `/deeptime probe
  biomes [step]` (the whole planet, grid to `<server>/deeptime-probe/biomemap.json`); `deeptime-inspect
  biomes` (chunk biome palettes parsed; the biome at each column's terrain top beside the world data's
  Köppen map); `tools/review/biome_map.py` (the review page), `tools/review/p1-09-smoke.sh` (both profiles:
  first start, Chunky over Af/BW/Dfc/ET/EF areas, the probe, inspect).
- **Acceptance (real planet, `earthlike_quick_16k`, scattered continents, 500 Myr).**
  `mac.sh --pack --server -- tools/review/p1-09-smoke.sh standalone mic` → both smokes **PASS** (first-start
  generation 100 s / 133 s on a quiet Mac). Whole planet, one sample per 16 blocks (1024 × 512), share of a
  Köppen family's columns whose biome carries the family's tags, standalone / MIC:
  | check | columns | standalone | MIC |
  |---|---|---|---|
  | BW: desert or badlands | 7,798 | 75.1 % | 78.0 % |
  | BW: any dry tag | 7,798 | 90.4 % | 92.3 % |
  | Af: jungle | 4,651 | 84.3 % | 82.2 % |
  | Dfc: taiga or coniferous | 11,847 | 88.9 % | 66.5 % |
  | ET/EF: snowy or icy | 34,526 | 97.7 % | 99.3 % |
  | A (hot/jungle/savanna/wet) | 13,751 | 76.3 % | 95.9 % |
  | BS | 12,562 | 82.0 % | 75.8 % |
  | C (temperate) | 23,232 | 96.0 % | 81.9 % |
  | D | 24,429 | 100 % | 99.6 % |
  | sea (ocean tags) | 367,253 | 100 % | 100 % |
  Standalone: Af → bamboo jungle 50 %, jungle 25 %, sparse jungle 9 %; BWh → desert 57 %, badlands 14 %;
  BWk → desert 48 %, badlands 28 %; Dfc → taiga 50 %, old-growth spruce 19 %, pine 14 %; ET → snowy plains
  80 %; EF → snowy plains 53 %, snowy slopes 34 %, peaks 11 %. MIC (100 biomes placed, 49 Terralith): Af →
  rocky/tropical jungle + vanilla jungles; BW → desert 28–38 %, (eroded) badlands 37–47 %, painted
  mountains; Dfc → taiga 36 %, Terralith forested highlands 15 %, yellowstone 14 % (neither tagged taiga,
  hence 66.5 %); ET → cold shrubland, snowy plains, gravel desert; EF → snowy slopes, snowy badlands,
  scarlet mountains. MIC's mapping moved erosion by up to ±0.45 and weirdness ±0.2 (Tectonic's router:
  stepped erosion −1/−0.5/0/0.3/0.6); standalone's tables are identity. Chunky areas (`deeptime-inspect
  biomes`, the biomes the chunks store) agree with the probe map: the BW area is desert/badlands, Af jungle,
  Dfc taiga, ET/EF snowy.
  **First render, fixed before this one:** ET had 35 % snowy taiga and EF 34 % grove (vanilla's variant at
  the frozen band's third humidity band, and its slope biome) → ET/EF humidity capped at 1.95; the river lines
  above → odd sampling.
- **GameTest floors** (`climateConsistent` on `test_planet_real`, C 4096, 150 Myr; first runs standalone /
  MIC): BW 0.765 / 0.824 (17 columns: logged only), BS 0.738 / 0.825, C 0.86 / 0.86, D 1.0 / 1.0, Dsub 0.969 /
  0.987, E 1.0 / 1.0, sea 1.0 / 1.0, Dfc:taiga — / 0.657, E:snow 1.0 / 1.0; floors about 0.1 under the lower.
- **Gates** (branch merged with master 2a78b1c, the settings audit; the smoke renders ran on the tree before that merge,
  whose defaults reproduce the same planet): cargo fmt, clippy `--locked -D warnings`, deny licenses bans clean; cargo test
  --workspace (core 238 + 3 ignored incl. `bake::biomes` 8, cli 30, format 40, inspect 29, …); `./gradlew build test`
  tests=140 failures=0; `./gametest.sh` → **All 50 required tests passed**;
  `./gametest.sh -PwithMic` (after `install.sh mic`, 168 mods), four runs (the last on the final merge): every test that finished passed (48 of 50, `climateConsistent` and `bandsAreTheBuilders` among them), but each run stalled in `lakeHolds` and died of `OutOfMemoryError: Java heap space` (8 GB) before `lakeHolds` and `worldInfoFollowsThePlayer` reported — the known P1-12 flake (3 of 7 then), now 4 of 4. No Deep Time frame in the traces (C2ME workers). `-PwithMic` had not been run on master since P1-08 added the real-planet and deposit test levels; no control run of master was made, so whether P1-09 adds to the heap is not measured (it adds no chunk loading: the census samples the biome source).
- **Found on the way (not P1-09).** `CompatGameTests.cdgOilFromTrap` failed under `-PwithMic` on the merged
  master: the survivor-formula deposit world has traps sharing chunks, and the sum check compared the chunks a
  trap fills with its whole charge (the deposits merge ran no GameTests). Fixed: the check leaves out blocks
  in chunks another trap fills (passes since). `-PwithMic` also stalls in `lakeHolds` and dies of heap
  (the known P1-12 flake) — see the MIC line above.
- **Deviations.** The physical → vanilla step runs in Rust's bake, not in Java (the CLI, the workbench and the
  game all bake; only the bake has every physical field); Java only maps baked values onto the pack's bands
  and router distribution, which is the pack-dependent part. The anchors the plan put in a datapack file are
  the preset's `biomes` section (Ben's settings rule); the datapack file holds the bands a pack may override.
  The Create World review page's biome names stay a P1-11 follow-up. Class names as the plan
  (`ClimateMappingBuilder`, `ClimateMapping`, `OverworldBands`) plus `BiomeCensus`.
- **Review bundle** (not published): `review/out/p1-09/` (3.4 MB, laptop): the Köppen map beside the biome map,
  standalone and MIC, hover readout, the family table, top biomes per Köppen class, the legend, and the five
  Chunky areas (chunks' own biomes beside the Köppen map). Data: `review/out/p1-09-data/`; the first render
  (before the ET/EF and river fixes): `review/out/p1-09-v1/`.
- **For Ben.** (1) The `-PwithMic` gate: the `lakeHolds` heap stall now happens every run (above); options are
  a master control run to see whether it predates P1-09, a larger `-PwithMic` heap, or chasing the stall
  (resolved 2026-09-29, build log "MIC GameTest heap stall"). (2)
  Mapping internals that stay constants (candidates for more settings under the standing rule): the inland
  continentalness distances (200 / 1,000 / 5,000 blocks), the wetland thresholds (vegetation 2.5 bands, flattest
  35 %, below 150 m) and the band placement per Köppen class. (3) In MIC, Dfc lands on Terralith's forested
  highlands and yellowstone as well as taiga (66.5 % taiga-tagged); Dsb is 59 % yellowstone.
- **Next.** Ben reviews the bundle; P1-10 (TerraBlender regions follow provinces) can start; P1-11's review
  page could show biome names by running `BiomeCensus` on the staged world.

### 2026-09-29 — Settings audit: the world-shaping knobs become editor settings (agent worktree agent-aebf7d2ff8d444591, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Why.** Ben, 2026-09-29 (decisions log): "Again, shoud be configurable. For now go for Medium" (terrain carving), with the standing rule that
  tuning knobs that shape the world are settings with a sensible default, not constants. Then, on this work: "Wire them up
  (Recommended)" (the planned Multipliers) and "Show only Earth-like (Recommended)" (the workbench's preset list).
- **What.** `editor_fields.json` (shared by the game's `EditorFields` and the workbench) now has, besides the old sections:
  a **Terrain** section (`drama`; **Terrain carving** = `amplify.incision_cap`, subtle 0.5 / **medium 3, the default** /
  dramatic 10, range 0.25–10, named `stops` the workbench shows as buttons; the core accepts (0, 100] now, 10 is "practically
  uncapped": on the 100 Myr planet the amplifier's eroded volume is 0.39 / 0.56 / 0.69 / 0.84 / 0.92 / 0.97 / 0.98 / 1.00 of the
  uncapped run's at 0.25 / 0.5 / 1 / 3 / 5 / 8 / 12 / ∞, and the flood's largest repair is 3 m at 3, 13 m at 12, 82 m uncapped)
  and **five collapsed Advanced sections, one per stage** (`advanced_sections`). Every field has a plain-language `hint`.
  Defaults are the calibrated values, and **the pipeline presets (and so the built-in `earthlike_*`, synced by
  `tools/dev/sync_builtin_presets.py`) now carry every such key**, since a field is editable only where the preset carries its path.
  A path may index an array (`structure.pluton_length_km.0`, `surface.sea_level.cycles.0.amplitude_m`); a `choice` may list
  `options` (`deposits.province`); `Formats.number` writes 2e-9 (six decimals rounded it away).
  Settings added (id → key, range, default; the unit is in the catalogue):
  - Terrain: carving `amplify.incision_cap` 0.25–10 (3); drama `surface.drama` 0–2 (1, existed).
  - Multipliers (wired to the core, all default 1, replacing the eight `planned` fields except three): deposit count
    `deposits.count_mult` 0–10; **deposit size** `deposits.size_mult` 0.1–10 (scales every type's tonnage percentiles, so the body
    volume follows: 8× the ore is a body twice as wide); **deposit richness** `deposits.richness_mult` 0.1–10 (grades and the
    metal each site holds, grades stay ≤ 1); province strength `deposits.province` off/weak/medium/**strong** (a fixed-list choice, was
    a 0–1 proposal); **feature scale, deposits** `scale.feature_scale.deposits` 0.1–10 (ore bodies' size in blocks, tonnage and grade
    unchanged); **feature scale, strata** `scale.feature_scale.strata` 0.1–10 (every layer's thickness in blocks: the bake divides the
    metres per block by it). `params::FeatureScale`, `DepositParams.{size_mult, richness_mult}`, `select_types(params, feature)` folds
    them into each type's own balance, so 1 is exact.
  - Advanced · Plates and crust: plates `tectonics.initial_plates` 4–40 (14, integer); continents at the start
    `tectonics.scattered_continents` 1–12 (6); breakup rate `rift_lambda0` 0–0.04 (0.012); ridge depth `ridge_elevation_m`
    −4500…−500 m (−2500); flooded margin `continental_shelf_share` 0–0.6 (0.29); island-arc build-up `arc_uplift_mm_yr` 0.05–1 (0.3);
    mountain uplift `subduction_uplift_mm_yr` 0.1–2 (0.6); mountain belt width `subduction_distance_km` 400–2000 (800); collision belt
    width `collision_distance_km` 800–4200 (2000).
  - Advanced · Erosion and coasts: erosion strength `surface.k_spl` 1e-10…1e-8 (2e-9); isostatic rebound `isostasy` 0.3–0.9 (0.8);
    hillslope creep `kd_land_m2_yr` 0.1–20 (2); coastal erosion `kd_marine_m2_yr` 0–300 (100); wave base `wave_base_m` 10–200 (60);
    limestone growth `carbonate_max_mm_yr` 0–30 (10); salt deposition `evaporite_m_myr` 0–500 (100); sea-level offset
    `sea_level.base_m` ±200 (0); sea-level swings `sea_level.cycles.0.amplitude_m` 0–200 (80).
  - Advanced · Climate: ice-sheet feedback `climate.ice_sheet_feedback` 0–1 (1); lapse rate 3–9 °C/km (6.5); rain reach
    `moisture_decay_km` 1000–8000 (3500); monsoon shift `itcz_max_shift_deg` 0–35 (22); mountain rain `oro_hours_per_day` 0.1–3 (0.5);
    snowline `ela_offset_c` ±8 °C (0).
  - Advanced · Granite bodies: `structure.pluton_rate_km2_myr` 1e-7…1e-4 (1e-5); size `pluton_length_km.0` 8–80 km (25).
  - Advanced · Ore provinces: province size `deposits.province_km` 300–3000 (800); brine flow reach `deposits.brine_flow_km` 100–2000 (600,
    MVT's regional driver, added by the deposit rework).
  Also the planet ranges moved to what the extremes allow: `planet.ocean_fraction` 0.31–0.95, `scale.m_per_block_h` 50–10000.
  **Left out on purpose** (the catalogue's `_comment` says so): iteration counts, solver tolerances, tile and grid sizes, time steps,
  frame and snapshot intervals, horizon and layer counts, event thresholds, the energy balance's radiation constants, the orographic
  physics constants, the rest of Cortial's plate constants, `deposits.max_formed_per_survivor` (a cap of the count model),
  `tectonics.abyssal_elevation_m` and `trench_elevation_m` (validated, never read by the model: an ocean-floor-depth knob changed
  nothing in a 100 Myr run, so it was dropped again), `amplify` constants other than the cap, `surface.substeps`.
  **Still `planned`, and why:** rivers and canyons scales (`scale.feature_scale.rivers` / `canyons`): the model has no rivers or
  canyons to size before Phase 2's river and cave stages (the detail layer only fades noise along channels); the **essentials** list
  (`playability.essentials`): it is pack-defined (datapack lists the game would offer), the core takes inline `Essential` entries and
  there is no datapack loader for named lists yet.
- **Game.** The setup page keeps the Advanced sections behind one header button ("+ Advanced settings (n)", "· k changed" when edited),
  closed by default, open in the page's own scrolling list with each stage as a sub-header; tooltips show the hint and range; a choice
  cycles its options. `EditorModel.advancedOpen/advancedAvailable/advancedEdited`, `EditorFields.ADVANCED/advancedFields/advancedSections`,
  `Field.hint/options`. The ScreenshotDriver opens and scrolls it (`02b`, `02c`). **Workbench** (no `workbench.js` change was needed for
  the sections; small additions): advanced sections start folded, integer knobs are sliders, choices list their options, `stops` are
  buttons, and the page lists only the **Earth-like family** (`watch/editor.rs` `PresetEntry.hidden`: pipeline and plates presets stay
  resolvable by name for tests and the CLI but are not in `/api/catalog`, and a share code of one loads as an imported preset); the
  default is Earth-like · quick · 16k, and `FAMILY` names `earthlike_*`. The in-game editor lists only `Presets.BUILTIN` (unchanged).
- **Extremes.** `tools/dev/knob_extremes.py` (stdlib) runs every catalogue field the preset carries at both ends (numbers) or each
  option (choices) on a 100 Myr `pipeline_quick_16k` planet through the amplifier: about 110 runs over the session (Mac M6, `--jobs 3 --threads 4`, ~1–5 min each).
  Verdict: process succeeds, no panic or non-finite value, mass residual ≤ 1e-2 km³ (all under 5e-3), no sink. **One found a bug:**
  `planet.ocean_fraction` 0.25 with the presets' shelf share 0.29 asks for continental crust over 100 % of the planet and the surface
  stage's elevations come out NaN everywhere (`body 0: centre is not finite`); `PlanetPreset::validate` now refuses continental crust
  over 98 % (`params::MAX_CONTINENTAL_SHARE`, tested), naming both fields; ocean fraction min raised to 0.31 (land 0.85 there, flagged
  "almost no sea" by the script and kept: a valid desert world) and `m_per_block_h` min to 50 (10 m/block left 0.000 of the planet
  above the sea). Non-obvious extremes: `star_flux` 0.1 / 10 give −95 / +1030 °C land (planet ranges are P1-03's, not touched here);
  `m_per_block_h` 10000 gives 274,262 plutons (the count scales with area). `drama`, `m_per_block_v`, the deposit and feature
  scales change nothing in the amplifier's summary lines (they act in the bake and the deposit records).
- **Default runs are bit-identical.** Master (16e4c90, before this branch) against this branch's merge with it, `pipeline_quick_16k`,
  scattered continents, 100 Myr, 6 threads: `diff -rq` of the whole world folder (keyframes, L3 elevation and detail rasters) reports
  nothing. (Checkpoints differ in size only: the preset they carry now names the keys explicitly.)
- **Round trip.** Share codes: `crates/deeptime-core/tests/share-codes.json` case `pipeline_quick_16k_tuned` (a pipeline preset with 13 knobs
  changed; Rust and Java encode it, both decode both): `AdvancedSettingsTest` (Java: every field of a code shows in its field, the model
  writes back the same JSON, top of every range and last option survive export → import) and `editor.rs`
  `a_tuned_code_from_the_game_loads_unchanged` (the workbench decodes both codes, `effective` returns the very same JSON and each
  field's value). Game to workbench and back is the same two tests on one file.
- **Measured gates** (Mac, branch merged with master b8788a7): `cargo fmt --check`, `clippy --workspace --all-targets -D warnings`,
  `deny check licenses bans` clean; `cargo test --workspace --locked`: cli 30, core 233 (+3 ignored), format 40, history_cap 1, others as before, 0 failed
  (new: `deposits::tests::the_global_multipliers_fold_into_the_types`, `bake_horizons::tests::the_strata_feature_scale_thickens_every_layer`,
  `params::tests::the_pipeline_presets_write_out_the_tuning_knobs_at_their_defaults`, incision-cap range, the ocean-floor refusal, several
  editor tests); `cd neoforge && ./gradlew build test`: tests=134 failures=0 (new `AdvancedSettingsTest` 9, `Formats`, lang keys).
  GameTests not re-run.
- **Screenshots** (gitignored, on the laptop): real client
  (`tools/client/mac-screenshots.sh --name settings --no-guidance --no-create`, the Mac's desktop, exit 0): `02b-setup-advanced.png`,
  `02c-setup-advanced-scrolled.png` (the Advanced header open and the list scrolled to the erosion and climate sections) next to the
  usual `01`…`09` (`review/out/settings/`); workbench (`review/out/settings-workbench/`) from a private instance on port 8653 (`tools/viewer/shot.py`, headless Chromium through an ssh tunnel; the live
  8642 was not touched): `workbench-editor-default.png` (Earth-like · quick default, only the two families), `workbench-terrain-carving.png`
  (the carving with its stops, the Multipliers), `workbench-advanced-open.png`.
- **Next.** Rivers and canyons scales with Phase 2; the essentials list with the datapack loader. P1-17 proper should re-check that
  every range still holds after its final numbers (`python3 tools/dev/knob_extremes.py --bin target/release/deeptime --out build/knobs`).
### 2026-09-29 — P1-20 stage 2 follow-up: previews and click-to-inspect show the run's own terrain (agent worktree agent-afd5791e0f5ea82cd, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **What a finished workbench run leaves.** `deeptime watch` already bakes: `Run::drive` calls
  `Session::finalize_write`, which bakes every planet preset, so `<runs>/<id>/world/` holds manifest `format_version`
  0.1 with `notes.bake`, the rasters, `strata/horizons`, bodies, sites and events (a 150-Myr `earthlike_quick_16k` run:
  433 MB, 355 MB of it `keyframes/`). No change to the run path was needed. Runs from before P1-08 (format 0.0), placeholders
  and runs still running have no bake. The manifest is written last, so its presence means the run finished.
- **Server → runner.** `stage2::Preview` gained `runs` and `deeptime`; `watch::serve` fills them (the `--runs` folder,
  made absolute, and its own `current_exe()`), and `Preview::call` passes them to `preview.py` as
  `DEEPTIME_WORKBENCH_RUNS` and `DEEPTIME_WORKBENCH_DEEPTIME`. The runs folder is also an allowed `world_data` root.
- **`deeptime inspect-column --why`** (new flag): puts `why` on each site of the answer: its forming event and the cause chain
  (4 links) from the world data's event log, as the 3D export's site cards have it. `attach_why` in `main.rs`.
- **`preview.py`.** `run_bake(run)` (format >= 0.1, `notes.bake`, not a placeholder, terrain and strata layers, C in
  16,384/32,768) returns the bake or the reason there is none.
  - `GET /api/inspect` for a run with a bake answers at once (`200`, no job) from `deeptime inspect-column --world-data
    <run>/world --why X Z`: `terrain_height`, `top_y`, `seconds`, `note`, `note_kind: "run"` and the whole column under `inspect`
    (horizons, `rock` from the top block down, `body_details`, `sites` with `why`, every other layer's value). x, z come from the
    page (the run's own C), folded.
  - `POST /api/preview` for a run with a bake builds a job whose `world.world_data` is the run's bake (copied without
    `keyframes/`), `size` = the manifest's C, the box in the run's blocks; the real mod boots on it ("world data present …: not
    regenerating; Bound Deep Time world data (format 0.1, C = 16384)"). `note` says it is the run's terrain and that a bake is the
    planet at the end of the run (`note_kind: "run"`). The job contract (status, `status_url`, `url`, files) is unchanged.
  - Runs without a bake keep the synthetic deposit world at the drawn latitude and longitude, with a note that says why it is not
    the run's terrain (`note_kind: "test-world"`), and the 32 x 32 inspect preview. A finished-preview lookup now only matches
    test-world jobs.
- **`workbench.js` / `workbench.html`.** A direct column answer (`inspect` present) fills the inspector's Details tab
  (`showColumn`: terrain height, elevation + detail, water, the rock rows top block down, ore bodies with their y range and
  role, and per site the yields and the forming event and cause chain, as links into the run's event list when the event is
  known; "Preview 256 x 256 here" works from it) and leaves a small card in the 3D tab. The note callout's lead follows
  `note_kind`; the intro text and tool hints no longer promise a test world. Look unchanged otherwise.
- **Tests.** `tools/workbench/test_preview.py` (8 tests, stdlib: bake detection for baked/old/placeholder/running/odd
  runs, the runs root as an allowed path, a 32k preview's spec and box, the fallback spec and note, inspect through a fake
  binary with the exact arguments, folding, a failing evaluator, the fallback inspect preview); Rust: the stage2 forwarder
  passes the two env vars, the CLI planet test exercises `attach_why` on a column inside a site.
- **Verified on the Mac (private `deeptime watch --preview` on 8654: 8653 was already taken by another session's watch; 8642
  untouched).** `earthlike_quick_16k`, scattered continents, 150 Myr, through `/api/preset` + `/api/runs`: 251 s on a loaded Mac
  (load 20-35), manifest 0.1.
  - *Inspect vs the CLI:* `/api/inspect` answers in 60-90 ms wall (evaluator 8-37 ms) and its `inspect` equals `deeptime
    inspect-column --why` on the same world for 5 of 5 columns (a site column with a 3-event chain from `deposit_formation`, a
    placer site with 8, a 406-block mountain, a body column, a plain one; a lap-shifted x differs only in the echoed x).
  - *Preview:* a 64 x 64 box of the run's highest ground (x 1856..1920, z -2112..-2048; bake 406 blocks) through
    `POST /api/preview` with the page's body: done in 64.8 s (jar refresh 20.9 s, the first Gradle build in this copy; boot 22.1 s;
    Chunky 49 chunks 11.0 s; stop 3.6 s), export 0.18 MB, `verify` 4,063,232 blocks 0 wrong, top at y 414.
  - *Height vs the bake, `deeptime-inspect compare` on a kept world (same job, two boxes; carved columns excluded by its rules):*
    mountain box 3,170 of 4,096 columns, **p50 |diff| 0**, 94.9 % within 1 block, p99 3, max 6, mean -0.30; a second box at the
    coast (x 1856..1920, z -2432..-2368; a body there) 2,078 columns, p50 0, 89.0 % within 1, p99 3, max 4, mean -0.50. The
    same shape as P1-08's comparison (negative tail: caves and carvers).
  - *Screenshots* (headless Chromium over an ssh tunnel to 8654, `tools/viewer/shot.py`): `review/out/workbench-bake/` inspect of
    the site column (Details tab with rock, sites, why chain), the Area 3D card ("This run's own terrain", waiting for the server
    lock behind another session), the workbench-made preview in the viewer.
- **Gates** (branch merged with master b8788a7): `cargo fmt --check`, `clippy --workspace --all-targets --locked -D warnings`,
  `deny check licenses bans`, `cargo test --workspace --locked` all pass (cli 27, core 230 + 3 ignored, format 40, inspect 28 + 2 + 7, jni 7,
  viewer 5, history cap 1); `python3 tools/workbench/test_preview.py` and `tools/smoke/test_jarstate.py` pass. No Java changed.
- **Not done / found.** The MIC default profile and the per-job jar refresh are untouched. A click in the viewer outside every
  exported box still does not ask the column route (README "Still to do"). The run card's tag reads "Placeholder · Earth-like
  (quick)" although `earthlike_*` are real since P1-08 (`familyName` in `workbench.js`); not changed here.
  Deep Time's answer for a column is not the placed blocks (other mods' features, carvers): the 3D export stays the truth for those.
- **Next.** Publish the screenshots if Ben wants them; the viewer's `describe()` could reuse the column route.

### 2026-09-29 — Deposits: USGS mean density, survivors within the assessment depth, researched densities; GameTest cdgOilFromTrap fixed (agent worktree agent-afa559c18eb85c3f6, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **GameTest `cdgOilFromTrap` (failed on master 62c87c9: trap 66, 9,393,132 of 9,600,610 mB).** Cause:
  the test's accounting, not the engine. The survivor formula forms more traps; in the synthetic
  deposit world two traps now reach one chunk, `SiteChunks.covering` hands that chunk to the trap
  that fills it more, and the test skipped the chunk before adding the other trap's share to its
  sum. Fix (810d33b): every chunk a trap reaches adds its share; the per-chunk comparison with what
  CDG is told still runs only on the chunks the trap supplies. After it trap 66 sums to 9,600,611
  mB. Mac `./gametest.sh` → All 48 required tests passed; `tools/smoke/install.sh mic` + `./gametest.sh
  -PwithMic` → All 48 passed (a first MIC run died of `OutOfMemoryError: Java heap space` in C2ME
  workers 10 min after the last test logged, while two other agents' workbenches held the Mac; the
  rerun passed).
- **Ben's answers (decisions log 2026-09-29) as built.** (1) `sizing.density_basis`: `measured` → the
  survivor target uses the mean of the split lognormal through P90/P50/P10 (`Lognormal::mean`),
  `estimate` (default) → P50, never inflated. Only porphyry is measured (Singer, Berger, Menzie &
  Berger 2005: 4 / 15 / 35 → mean 17.8). (2) `preservation.assessment_depth_m` (default 1000): only
  survivors whose top lies within it count toward N\* (USGS Circular 1178 p. 15: assessments count
  deposits to 1 km, a deposit counting if any part is within it); λ_i now uses p̄₁ (survive within
  the depth), deeper survivors are sites on top. Coal 1,800 m (Circular 891: coal deeper than 6,000
  ft is excluded, printed p. 24), petroleum 10 km (USGS petroleum assessments have no fixed cut-off;
  deep gas below 4.5 km, Dyman et al. 2002). Census: `density`, `counted` (sites within the depth),
  `expected` (counted expected), `expected_sites` (all). (3) Research (Sonnet sub-agent): no USGS
  deposit-density model covers sedex (PP 1640-A, SIR 2010-5070-N); the 1998 National Mineral Resource
  Assessment's sedex tracts (Circular 1178, mrdata.usgs.gov/nmra) give undiscovered-deposit
  estimates, 12 numeric tracts over 207,508 km² summing to about 2 / 10 / 26 → SEDEX density 1 / 5 /
  12.5 per 100,000 km² (was the placeholder 0.5 / 2 / 8; an estimate and a lower bound, P50 used).
  No published coal deposit density exists (Circular 891, PP 1809, IEA CCC/32 checked): coal's 30
  stays a placeholder. Sources cited in the type JSONs and the plan's §8.6 note.
- **Counts** (the six landscapes regenerated at checkpoint schema 6 after the master merge; `stage
  deposits --in`, sites median (range), planets with any; counted = sites with top within the
  assessment depth, against the target):

  | type | sites | counted (target) |
  |---|---|---|
  | porphyry | 13.5 (10–19), 6/6 | 5–12 (7.0–14.6) |
  | MVT | 18 (7–22), 6/6 | 1–17 (2.6–11.0) |
  | SEDEX | 4 (0–11), 5/6 | 0–2 (0.33–1.11) |
  | petroleum | 3.5 (1–5), 6/6 | = sites (2.1–4.5) |
  | coal | 47.5 (25–56), 6/6 | 17–35 (15–44) |
  | placer Au | 11.5 (5–34) | children of eroded porphyries |

  The 1 km rule is what moves the numbers: a porphyry formed at 1.5 km, an MVT at 1 km and a SEDEX
  sheet buried in a subsiding rift mostly lie deeper than 1 km, so more form and the deep survivors
  are extra sites (porphyry ~2×, MVT ~4×, SEDEX mostly deep). SEDEX preservation within 1 km is 5–30 %,
  so it is capped at 20 formed per counted survivor on two landscapes. Porphyry forms 28–92 per
  planet, hence more placers.
- **Tests.** `survival::tests` gain the counted window (uniform erosion: counted share = the closed
  form of the age window); `sizing::tests::split_lognormal_mean` (porphyry 17.8, symmetric case,
  numerical integral); `recorded_tests::the_survivor_formula_hits_its_target_in_expectation` now checks
  counted survivors = target (P̄₁ closed form), all sites = target·P̄/P̄₁, formed = target/P̄₁.
- **Gates** (Mac, on 29c4aef = master b8788a7 merged + this work): `cargo test --workspace --locked`
  (core 231 + 3 ignored), clippy `-D warnings`, fmt, deny clean; `./gradlew build test` → tests=125
  failures=0; `./gametest.sh` → All 48 required tests passed; `-PwithMic` → see below.
- **Deposit stage time** (seed 1 scattered, `--in`, 6 threads, `--verbose`): in memory 15.6 s wall,
  `--history-mem 0` (all 50 frames on disk, `50 frames on disk`) 13.9 s wall; pass 2 over 14,522
  tract cells 3.7 / 3.9 s; frames read in 0.04–0.1 s either way. Pass 2 reads frames once each, in
  order (the trace reads origins last-to-first once).
- **Decisions for Ben** (not asked): MVT now has 7–22 sites per planet (formation depth 1 km sits on
  the 1 km line, so most are "deep extras"); coal's density is still a placeholder (what is a coal
  deposit?); SEDEX's density is a lower bound from undiscovered-deposit estimates.

### 2026-09-29 — P1-17 partial: the diffusion solve fused into two passes per CG iteration, bit-identical (agent worktree agent-a0f4f991a0d939f16, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **What.** Only `surface/diffusion.rs` (plus the new `surface/diffusion/parity.rs`). The Jacobi-PCG iteration was seven
  parallel regions over ~22 vector streams of 4 MB; it is now two passes over strips of 8 rows: **pass 1** forms the direction
  (z, then z + βp), applies the operator to it and reduces p·Ap; **pass 2** does x += αp, r −= αAp, z = r/diag and reduces r·r
  and r·z. The operator is applied one term at a time per row (same order per cell, so it vectorises); each strip keeps a
  three-row ring of the direction so neighbouring strips never read what another writes. Also parallel now: `Operator::new`
  (was serial), the final residual correction, and `lod`'s z-sweep scatter (columns are solved in blocks of 8, so one cache
  line serves eight columns; each column is solved exactly as before).
- **Bit-identical, and how the reductions work.** The old `dot` was already thread-count independent: a sum per row in
  ascending column order (`.sum()`), the row sums collected and added in ascending row order on one thread. The fused passes
  keep exactly that: per-row sums, started from the same value `Iterator::sum` starts from (`sum_zero()`: -0.0 today, so a row
  of negative zeros keeps its sign), then the `h` row sums added serially in row order; four rows are summed side by side
  only to overlap latencies, never within a row. Nothing depends on the strip height (verified at 1, 3, 16) or the pool. No FMA
  (Rust never contracts), no reciprocal for a division, A·p's terms in the old order (diag, east, west, south, north).
  **Evidence:** (1) `diffusion/parity.rs` keeps the old `step`/`lod`/`Operator`/`dot`/`cg` verbatim and compares bit patterns
  of the field and the iteration counts on random wrapping grids with stiff diffusivities (0, 1e-20, 5, 300, 1e5), on fractal
  terrain with the surface stage's real diffusivities (`submarine_kd`: open sea of 300 m²/yr next to 2 m²/yr coasts, 5/40/78/100 %
  of cells in the sea, wrapping and not), 30 awkward shapes (1×1, 1×9, 9×1, 2×n, heights around the strip), multi-step runs,
  kd = 0 (loop never starts), a flat field and the iteration cap, each on pools of 1, 2, 4 and 8 threads (and the reference on
  each pool too). Mutation checks: swapping the east/west term order fails 4 of 6 tests; splitting a row sum over two
  accumulators fails 2. (2) Real 500 Myr `pipeline_quick_16k` worlds, old binary (master) at 4 threads against the new binary at
  12 and at 1 thread: every world file (12 keyframes, events, sites, bodies, indices, preset; sha256 over all files but
  `manifest.json`) identical, `surface_final` line identical (108.8 CG iterations per step); `manifest.json` differs only in
  `created_utc` and the `millis` timings. That was on c79b38f; after merging master 455b57f (P1-06 young/amplify stages) I
  repeated it: scattered_continents old@4 = new@12 = new@1, and supercontinent old@4 = new@12, all byte-identical. The
  `run --checkpoints` whole-file compare is not usable now: the checkpoint embeds stage summaries with timing text whose length
  varies, so the files are byte-identical apart from the header CRC/length and those numbers (checked by re-aligning the diff:
  every difference is digits or a length prefix inside a summary JSON).
- **Measured (Mac M6, load average 2–4, so quiet), one 500 Myr-sized step on a 1024×512 L1 raster with earthlike
  diffusivities, min of 12 interleaved old/new pairs (`cargo test --release -p deeptime-core diffusion_timing -- --ignored
  --nocapture`; `DEEPTIME_TIMING_REPS` sets the pairs; 101 CG iterations):**

  ```
  threads  cg old ms  cg new ms  ms/iter old→new   lod old→new   operator old→new   step old→new
  1        193.9      102.3      1.92 → 1.01       14.4 → 13.3   0.8 → 0.7          208.5 → 116.6   (1.79x)
  2        101.2       55.1      1.00 → 0.55        9.0 →  7.1   0.8 → 0.4          111.1 →  63.0   (1.76x)
  4         70.8       41.9      0.70 → 0.42        5.5 →  4.0   0.8 → 0.3           77.6 →  46.0   (1.69x)
  8         55.1       30.9      0.55 → 0.31        3.8 →  2.4   0.8 → 0.2           59.0 →  33.5   (1.76x)
  12        51.7       28.6      0.51 → 0.28        3.5 →  1.9   0.8 → 0.2           62.3 →  33.1   (1.88x)
  ```
  Scaling of the new step: 1 → 12 threads 3.5x (old 3.3x); the solve is ~1.8x faster at every thread count, and at 1 thread
  too, so the gain is mostly memory traffic, not only barriers. The kd fill in `surface/mod.rs` is serial and 1.4–1.6 ms.
  **End to end (`bench --preset pipeline_quick_16k --start scattered_continents --length-myr 500`, old and new binaries
  interleaved, 3 rounds, on c79b38f; the Mac's load average was 2–27 from other agents, min..max):**

  ```
  threads code   total s        surface s      diffusion s    spl s        surface−diffusion s
  12      old    77.0..204.3    40.9..136.2    15.5..82.5     4.1..11.2    25.4..53.7
  12      new    74.4..145.9    36.5..81.3     10.0..29.9     4.3..12.1    26.5..51.4
  4       old    103.5..224.2   52.6..133.0    21.0..70.7     7.8..15.3    31.6..62.3
  4       new    95.8..121.1    44.4..57.1     12.7..18.5     7.8..9.6     31.7..38.6
  1       old    274.2..391.8   133.8..192.8   64.0..93.4     28.5..36.4   69.8..99.4
  1       new    288.4..329.1   124.8..145.1   45.5..54.8     30.8..33.9   79.3..90.3
  ```
  Diffusion per run (s), round 1/2/3: 12 threads old 82.5/16.4/15.5, new 29.9/19.2/10.0; 4 threads old 21.0/70.7/31.1, new
  12.7/14.3/18.5; 1 thread old 71.6/64.0/93.4, new 45.5/54.8/46.7. The whole-run numbers are dominated by the Mac's load (the
  same binary swung 15–82 s at 12 threads); read the minima and the microbench. Diffusion minima: 15.5 → 10.0 s at 12 threads,
  21.0 → 12.7 s at 4, 64.0 → 45.5 s at 1. On the merged tree (master 455b57f, which adds the young/amplify stages; one round, old then new, scattered_continents,
  load average 5 → 40–50 during the runs, other agents' jobs): 12 threads diffusion 252.7 → 70.0 s, surface 428.5 → 214.2 s, total 635 → 397 s;
  4 threads diffusion 73.4 → 30.0 s, surface 166.5 → 99.4 s, total 314 → 231 s. Loaded numbers, so only the direction counts;
  the machine was never quiet for a whole end-to-end run.
  Per-step diffusion in the best 12-thread run: 10.0 s / 250 = 40 ms, against the microbench's 33 ms plus the stage's kd fill and copies.
- **What is left for the preconditioner step.** Diffusion is still 25–35 % of the surface stage: 101 iterations × ~0.28–1.0 ms.
  A coarse-grid correction (111 → ~10 iterations) is now worth ~10× on this phase, but it changes results, so it needs the
  calibration agent's agreement; if it goes in, the two-pass structure stays (pass 1 gets the correction's prolongation in the
  direction formation, pass 2 the restriction sums; the reduction scheme, row sums added in row order, must stay for
  thread-count independence). Without changing results there is little more here: the iteration is at ~0.28 ms per iteration =
  ~14 GB/s of traffic on 12 threads, near what the Mac gives, and the serial remainder of the step (kd fill 1.5 ms, `before`
  clone, the difference loop after `step` in `surface/mod.rs`, ~3–5 ms in all) is now a few percent of it.
- **Gates on this branch (merged with master 455b57f):** `cargo fmt --check`, clippy `-D warnings`, `cargo deny check licenses
  bans` clean; `cargo test --workspace --no-fail-fast` all green (core 211 + 3 ignored, cli 25, format 33, inspect 28+2+7, jni 7,
  viewer 5). One flake seen once: `tests::surface_stage_runs_from_a_plates_checkpoint` failed ("keyframes/index.json: No such
  file" in `$TMPDIR/deeptime-stage-<pid>`, the shared Mac temp dir, while other agents' jobs ran); it passed on the immediate
  re-run with nothing changed. It is a temp-dir race in `stage` with `--work` unset (`deeptime-stage-{pid}`), not diffusion.

### 2026-09-29 — P1-17 partial: the recorded history spills to the SSD instead of failing at the memory cap (agent worktree agent-a33c2dc374c0ccd2f, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Why.** Ben, 2026-09-29, "Spill history to the SSD (Recommended)" (decisions log). The failing run (`pipeline_full_32k`,
  supercontinent, 1000 Myr, 4 GiB) died at the surface stage's first `reserve(estimate / 2)`: the estimate counted the
  whole 2.5 GB history, so that one reservation asked 2.65 GB on ~1.65 GB in use (the workbench entry's reading).
- **What grows with length (measured).** Only the recorded history: 12.7 B per L1 cell per frame at 32k full (2,540.9 MiB
  for 100 frames of 2048 × 1024), 5.3 MiB per frame at 16k quick. The base in use at the end of the surface group,
  without the history, is flat in length: 16k quick 761 MiB in use at 500 Myr, 931 MiB at 2000 Myr (+170 MiB for 4× the length, most likely the keyframe
  ring filling to its 16 frames; not traced). Events (7,391 at 32k × 1000 Myr), sites, the
  pluton list, the keyframe ring (16 frames, the rest on disk) and the log ring are small or bounded; checkpoints are disk
  and are now streamed (below). Documented, not spilled.
- **Design** (`deposits/{recorded, spill}.rs`, `session.rs`, `stage.rs`, `checkpoint.rs`).
  - `RecordedHistory` keeps each frame as `Memory(Arc<PackedFrame>)` or `Disk(head + file + offset + CRC32)`; a frame on
    disk is its fields' LZ4 blobs back to back (the existing encoding, same bytes), its head (names, ranges, bits,
    lengths) stays in memory, so the crust-origin offsets (`origins(k)`, read by `trace`) are two small reads.
  - The session owns a `SpillStore` in `out_dir` (the workbench's run folder, the game's staging folder, the server's
    `<world>/<folder>.generating`); the spill file `history-<pid>-<n>.spill` is created on the first eviction and is
    anonymous: unlinked at once on Unix, delete-on-close on Windows. Nothing is left behind after a crash, and nothing
    can be moved into a world. Its size is reported (`notes.run.history.spilled_bytes`, the CLI's `history` line).
  - Eviction, oldest-written first: (1) `StageContext::reserve` — when a reservation would pass the cap and the history's
    frames in memory can cover the excess, they go to disk first and the reservation is checked again; only a base that
    does not fit fails (`MemoryCap`, as before); (2) after every stage step the history is trimmed while the gauge is over
    `cap − cap/16`. Frames read back are not cached (flat memory; the OS page cache caches); a forward scan (the deposit
    engine's pass 1 and pass 2) reads the next frame ahead on a thread of its own. A frame a later stage adds fields to
    (P1-07's host fractions) comes back into memory and is written again when evicted.
  - Reservations that were missing or counted the history: the surface's first reservation now excludes the history;
    the deposit stage reserves its unpacked frame (ingest, placement), so on a long run its ~0.5 GB of decoded fields at
    32k make the history give way instead of passing the cap unreserved.
  - `--history-mem N` (CLI) / `Session::set_history_budget`: at most N bytes of frames in memory whatever the cap
    (0 = every frame to disk after its step), for tests and measurements. `Session::needed_peak` / `needed_base_peak`:
    the largest reservation with and without the history's frames (`notes.run.history.reserved_*`).
- **Checkpoints, schema 5 → 6.** Fixed fields `u32 schema, u32 history_frames`; section 1 is the body without the
  history's data plus its head; one section per frame holds its record. Written streamed (`<path>.tmp`, fsync, rename;
  new `container::StreamWriter`, same bytes as `encode`; the body straight from the world with `postcard::to_io`; frames
  copied from memory or the spill file) — no copy of the world or the history is made, and the session's own
  checkpoints borrow the world (`CheckpointRef`, `Session::write_checkpoint`). `Checkpoint::read` checks every section's
  CRC but leaves the frames in the file (the history reads them from the checkpoint on demand); schema 5 is refused
  naming both schemas. (Sections stay u32-long: a frame is ≤ ~100 MB; the body of a 32k world is ~2 GB.)
- **Estimate.** `session::estimate_for(preset, content, cap)` (and `estimate` = the default 4 GiB) adds `base_bytes`,
  `history_bytes`, `spill_bytes`, `mem_cap` (run and per stage; `StageEstimate::history_bytes`); `peak_bytes` = base +
  the history that fits, so `peak_bytes > cap` only when the base is over. The workbench (master's cap check) asks for the
  estimate under its cap less the 512 MiB server overhead, so it refuses only runs whose base does not fit, and shows
  "… of history to disk" in the estimate tile and a warning instead of "near the cap". Java: `Estimate` reads the new
  fields (`exceeds(cap)` = base over cap, `peakUnder/spillUnder(cap)`); the Create World editor blocks on the base and
  warns `editor.warning.spill`; the first-start generator refuses on the base and logs the spill. `SimSession` completes
  its future only after the native session is destroyed, so the first-start move of the work folder and the staging
  cleanup never race an open spill file (Windows).
- **Determinism evidence.**
  - `deposits::spill_tests::a_history_spilled_to_disk_gives_the_same_world_and_census`: the coupled pipeline (256 × 128,
    100 Myr) with every frame on disk (1 thread), a two-frame budget (2 threads) and all in memory (3 threads): the same
    world byte for byte (postcard body), notes but for timings, census and history.
  - `…::a_checkpoint_with_a_spilled_history_resumes_to_the_same_result`: checkpoints taken inside the plates–surface
    group and inside the deposit stage from spilled histories, written to files, read back (frames left in the file) and
    resumed on 3 and 2 threads with other budgets: the same world as a straight in-memory run; the checkpoint streamed
    from the spill file equals one written from memory.
  - `tests/history_cap.rs` (own binary, counting allocator): cap = max(unlimited peak − history/2, largest base
    reservation + 2 MiB) → 26 of 50 frames spilled, peak 46.6 MiB against 54.6 MiB unlimited, the world byte-identical; a
    1 MiB cap fails cleanly with `MemoryCap`.
  - Unit tests for the container writer/reader, fsutil, the spill file, evicted frames (origins, add_fields, clone,
    head + records, a flipped byte caught by the CRC), checkpoints with frames, the estimate under caps.
  - **At scale** (Mac, release, `deeptime run --preset pipeline_quick_16k --start supercontinent --length-myr 500
    --threads 4`, in memory vs `--history-mem 0`, run side by side): all 23 world-data files identical (117 MiB: events,
    sites, bodies, keyframes, the L3 tile packs) and every manifest note equal but for its timings (56 sites, same census);
    50 frames, 266.9 MiB of history, 478.8 MiB written (each frame twice: once by the surface, again after the structure
    stage added its fields), 787.8 MiB read back.
  - **At scale, 32k:** the failing run under 4 GiB (91 of 100 frames on disk) vs the same run under 8 GiB (5 frames
    spilled, by the amplifier's whole-raster reservation; its middle level then ran whole, 1 tile instead of 8): all 33
    world-data files identical (368.7 MiB, the L3 tile packs included), the deposit census and every note equal but for
    timings (118 sites, 7,391 events). Wall time 2,421 s (4 GiB) vs 2,534 s (8 GiB), both on the shared Mac under other
    agents' load: no slowdown to measure; 8 GiB run: max RSS 4.76 GB, peak footprint 9.17 GB, gauge peak 8,182 MiB.
- **Timings and memory.**
  - 16k quick 500 Myr side by side on the loaded Mac: 798 s in memory vs 795 s all on disk (noise); deposits 7.4 vs
    6.2 s; spill I/O 0.74 s writing, 0.24 s reading. Gauge peak 3,373 vs 3,106 MiB (the young/amplify stages dominate the
    peak at 16k), max RSS 1.40 GB both.
  - **The run that failed**, `pipeline_full_32k`, supercontinent, 1000 Myr, `--mem-cap 4294967296`, 8 threads (Mac
    load 40–78 from other agents' runs): **finished**, 2,421 s (40 min; plates 483, climate 187, surface 1,571, structure
    55, deposits 37.7, young 9.8, amplify 77.6 s streamed in 32 tiles), 118 sites, 7,391 events. History 2,540.9 MiB in 100
    frames; the first spill at 3,285 of 4,096 MiB in use (a reservation), 69 frames on disk after the surface group (2,970
    MiB in use), 91 at the end; **disk: 2,325.6 MiB written to the spill file** (2.3 GiB for a 1000 Myr 32k full run;
    ~2.5 GiB is the bound, the history's size), gone when the run ended. Gauge peak 4,405.5 MiB (the young and amplify
    stages allocate without reserving; P1-06's own finding), max RSS 4.61 GB, macOS peak footprint 6.69 GB.
  - **The long run** (parked question), `pipeline_quick_16k`, supercontinent, **2000 Myr**, `--mem-cap 4294967296`, 6
    threads: **finished**, 1,459 s (24 min on the loaded Mac; plates 343, climate 132, surface 911, structure 38, deposits
    20, young 2.5, amplify 13.8 s), 21 sites, 5,961 events. History 1,356.3 MiB in 200 frames (13.0 B per cell per frame);
    it all stayed in memory until the amplifier's whole-raster reservation, which sent 116 frames (762.5 MiB) to disk and
    ran in memory. Gauge peak 3,805 MiB, max RSS 3.38 GB. Deposits on 2000 Myr of history at `count_mult` as calibrated:
    porphyry 3, placer 10, coal 7, petroleum 1 (most drawn deposits eroded: 19 of 24 porphyries) — physics, not tuned here.
- **Gates** (Mac, after merging master 64752e6, P1-08 included): `cargo fmt --all -- --check` clean; `cargo clippy
  --workspace --all-targets -- -D warnings` clean; `cargo test --workspace --no-fail-fast` all ok (core 215 + 2 ignored,
  history_cap 1, cli 27, format 40, …); `./gametest.sh --server` All 48 required tests passed (before the merge
  `siteApiAndProbes` failed on master's deposit world too; P1-08 fixed it); `cargo deny check licenses bans` ok; `cargo check --target x86_64-pc-windows-msvc -p
  deeptime-core` ok (the Windows spill-file code); `cd neoforge && ./gradlew build test` → JUnit 125/0.
- **Next.** P1-17 proper: re-measure the workbench's 512 MiB overhead with the flat history (the method in the
  workbench entry); make young and amplify reserve what they allocate (the 32k run passed its 4 GiB gauge cap by
  ~0.3 GiB there, unreserved); the 12 B/cell/frame history constant in `surface::estimate` holds (12.7 measured at 32k).

### 2026-09-29 — P1-08 bake, world data 0.1 and parity (agent worktree agent-a09db47c6d05e1417, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **What.** (1) **Format 0.1** (`Cargo.toml`, `FORMAT_MINOR`, Java `FormatVersion`): every header's minor is 1; FORMAT.md
  gains "The bake" (layer table, `notes.bake`), "The terrain height" (H = E + A·N), "The P1-08 bake" (top horizon)
  and "Columns" (the evaluator and `parity/columns.json`). (2) **`deeptime-core/src/bake/{mod, terrain, strata,
  climate, surface, meta, tests}.rs`**, called by `Session::finalize_write` for every planet preset: keeps the
  amplifier's L3 `elevation` + `detail/amplitude` (or upsamples young L2 / surface L1 when a preset has no amplify
  stage), writes `water_level` (all nodata: lakes are Phase 2), `strata/horizons` + `classes.json` (P1-07's stacks),
  `climate/{t_mean_c, p_mm, koppen}` (L1), the five router inputs `climate/{temperature, vegetation, continents,
  erosion, ridges}` (L2, a **provisional piecewise mapping until P1-09**, anchors on OverworldBiomeBuilder's band
  edges, recorded in `notes.bake.climate`), `surface/{regolith_blocks, class_top}` (L2), `meta/{plate_id,
  crust_age_myr, province}` (L1; provinces = connected regions of one setting (regime + land/sea + highland) ≥ 50,000
  km², smaller ones absorbed), spawn candidates (ranked by essentials when the content bundle has them, else the synth
  rule, both now over any `RasterRead`), the preset's scale in the manifest, and `notes.bake` (incl. `detail`: seed as
  a decimal string, salt 1729, octaves, wavelength). **Top horizon raise:** P1-07 put each cell's first top at the
  L1 surface, so amplified ridges above it stayed unclassified stone; the bake raises the first top (only upwards) to
  ⌈max E + max A⌉ + 1 over the 4 × 4 L1 cells around (every L3 cell the bilinear column rule reaches), so the top unit
  fills ridges; deeper tops stay the history's. (3) **The column evaluator** in the format crate
  (`deeptime_format::column::ColumnWorld`, over the new `LazyLayer`: tiles decode on demand; `raster::stack_at` is now
  one shared function), with the detail noise moved into the contract crate (`deeptime_format::detail`; core
  re-exports it). `deeptime inspect-column --world-data D X Z [--parity]`, `deeptime parity --world-data D [--out]
  [--columns 500]`, JNI `sampleColumn` (cached world) — one evaluator. (4) **Java reader v1:** `worlddata/DetailNoise`,
  `WorldData.height` = E + A·N (`elevation()`, `detail()`, `classes()`), `worlddata/Columns` (the evaluator; StrataPass's
  column rule moved there, `StrataPass.column`/`Column` delegate). (5) **Real built-ins:** `earthlike_*` = `pipeline_*`
  (name/description only differ; `tools/dev/sync_builtin_presets.py` writes them, `builtins_are_the_pipeline_presets`
  fails on drift; mapping quick_16k/full_16k/quick_32k/full_32k one to one). The placeholder mechanism stays (dummy +
  synth + `placeholder: true`; tests use inline placeholders); no placeholder is built in any more, so the workbench's
  dev toggle lists none. (6) **Tests/fixtures:** `tests/presets/planet_world_4k.json` (small real planet, C 4096,
  every stage, 150 Myr); Gradle `testWorldPlanet4k` (run + parity); `ColumnParityTest`, `DetailNoiseTest`,
  `NativeLinkTest.sampleColumnIsTheParityFixture`; GameTest level `deeptime:test_planet_real` + `RealPlanetGameTests`
  (heights, rock vs the parity fixture, spawn; gametest.sh bakes it); core `bake::tests`, CLI
  `a_planet_run_bakes_and_parity_samples_it`. `deeptime-inspect`'s height includes the detail term; ramps for the new
  layers.
- **Acceptance.** `deeptime run --preset earthlike_quick_16k --start scattered_continents --length-myr 500 --out
  $S/q16 --viewer review/out/p1-q16` (a real preset names no timeline, so `--start/--length-myr` are needed: deviation)
  → `jq -r .format_version $S/q16/manifest.json` → **0.1**. `cd neoforge && ./gradlew test --tests '*ColumnParityTest'`
  → junit-summary **tests=2 failures=0 errors=0** (`columns=500 equal=500 throughBodies=77 withDetail=454`); the JNI
  oracle gives the fixture's exact JSON for all 500.
- **Sizes and timings (Mac M6, load 40–57 from other agents: timings noisy).** Run 752 s wall (plates 135, climate 57,
  surface 470, structure 14, deposits 12, young 14, amplify 30 s; **bake 1.1 s**: terrain 0.23, strata 0.42, climate
  0.35, surface 0.04, meta 0.04), alloc peak 3,439 MiB, peak footprint 4.64 GB. World folder **156 MB**: rasters 81 MB
  (`elevation` 49.1 MB and `detail/amplitude` 15.7 MB are the amplifier's; the bake's own layers ~25 MB: strata 6.8,
  L2 biome inputs 16.6, L1 climate 1.3, surface 0.5, meta 0.5, water 0.3), keyframes 69 MB, events/bodies/sites < 1 MB.
  Viewer bundle 108 MB (not published). 20 provinces; top horizon raised on 522,798 of 524,288 cells, largest raise
  449.5 blocks (a trench cell beside a high neighbour: the raise covers every block interpolating from it).
- **Smoke (first start with the real preset).** `mac.sh --server -- 'tools/smoke/install.sh standalone --preset
  earthlike_quick_16k --start scattered_continents --length-myr 500 && python3 tools/smoke/run.py --profile standalone
  --fresh --restart --chunky 160 --chunky-center 2880,-900 --timeout 2400'` → **PASS**: generation 491.6 s in the
  server (progress lines ≤ 5 s apart, native peak 3.1/4.0 GiB), Done in 498 s, RSS 2.76 GB; Chunky **441 chunks in 19 s
  (≈ 43 ms per chunk, wall, parallel, loaded Mac)**; restart does not regenerate (Done 1.1 s). Renders
  (`review/out/p1-08/`, `deeptime-inspect topdown|rock|section|compare` over x 2720..3040, z −1060..−740, 21° N): a
  6.8-km snow-capped peak (EF, −21 °C at the top) dropping to forested lowlands and a bay with sand beaches; the
  z = −930 and x = 2900 sections show the horizon stacks under the terrain to the surface (no unclassified stone at the
  top: the rock render's unmapped blocks are only ores and magma), columnar steps at L1 cell edges (P1-07's design),
  pluton lenses, caves. `compare`: p50 0, 80 % within 1 block, p99 18, max 38 (a negative tail: caves and carvers of
  the standalone router, steep slopes between noise-cell corners).
- **Gates** (branch merged with master 455b57f): cargo fmt, clippy `--locked -D warnings`, deny licenses bans clean;
  cargo test --workspace (cli 26, core 209 + 2 ignored, format 39, inspect 28 + 2 + 7, jni 7, viewer 5); `./gradlew
  build test` tests=124 failures=0; `mac.sh --server -- ./gametest.sh` → **All 48 required tests passed**
  (realPlanetHeightMatchesTheBake 47 corner columns max diff 0, all ≤ 1, 91 of 94 with a detail term;
  realPlanetRockMatchesTheParityFixture 48 columns, 124 samples, 0 wrong; realPlanetSpawnOnLand (112, 66, −494));
  `./gametest-planet.sh` (analytic terrain) seam set + entitypushedback, hardstopat80, worldinfochannelrequired PASS.
- **Found and fixed on the way.** `DepositGameTests.siteApiAndProbes` required a block with d < 0.3 in the target body,
  which the calibrated synthetic deposit world's body 0 (a clipped body) has none of: it now takes the most central
  block (not caused by P1-08; the calibration merge had not rerun GameTests). `NativeGameTests` pinned format "0.0";
  the share-code GameTest asked for 2,500 Myr, whose real estimate would pass the default 4 GiB cap (now 500 Myr).
  `editor_fields.json`: `deposit_count` → `deposits.count_mult`; eight fields whose preset key does not exist yet
  (size/richness multipliers, province strength, feature scales, essentials) are marked `"planned": true`, which
  `ModelHelpersTest.realPresetsCarryEveryField` exempts (it now checks 4 real built-ins).
- **Deviations.** Acceptance command needs a timeline (above). `water_level` is nodata (plan: Phase 2). The biome
  inputs are a provisional mapping (P1-09 replaces it). Horizon keys stay (index, class) — no unit ids yet. The
  evaluator lives in `deeptime-format` (the contract crate), not `core::bake`; `DetailNoise` is `worlddata/`, not
  `worldgen/`. `ColumnParityTest` runs on a small real planet (C 4096), not a synthetic world. The planet GameTest
  harness still runs on the analytic terrain (its seam/pole tests compare against that closed form); the real bake is
  covered by `test_planet_real` and the smoke.
- **Follow-ups the plan names.** Re-run P1-11 (`./gametest.sh`, client check), P1-13 (first-start acceptance: done
  above on earthlike_quick_16k, 500 Myr) and P1-15 (deposit GameTests, `-PwithEv`, `-PwithMic`, planet claim test,
  ore render) on the real presets; point the workbench's 3D preview (`preview.py workbench_preview`) at a run's bake
  and answer `/api/inspect` from `deeptime inspect-column` (tools/workbench/README "Later"); P1-09 replaces the
  provisional biome mapping; P1-10 maps `meta/province` to regions; the workbench catalogue lists `earthlike_*` and
  `pipeline_*` (identical) side by side — drop one family there.
- **Next.** Merge into master (the parent), then P1-09.

### 2026-09-29 — Deposits: the survivor formula and the regional MVT driver (agent worktree agent-afa559c18eb85c3f6, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Why.** Ben, 2026-09-29 (decisions log): "Change the formula (Recommended)" — USGS densities count
  deposits that survive to the present, the engine forms extra so the survivors match, no per-type
  magic multipliers; and MVT "Rework its driver" — a regional (basin-wide flow) driver so most planets
  get some.
- **The formula** (`deposits/survival.rs`; plan §8.6 as-built note). Per type: target survivors
  **N\* = count_mult · ρ_P50/10⁵ · Σ A_i** over the permissive tracts (density × permissive area,
  Singer's three-part method — mean F no longer scales the total). Formation is spread over tracts
  ∝ φ_i = A_i·F̄_i·E_i, within a tract ∝ F·E (unchanged placement), and in time ∝ each cell's
  favourability hazard −(Δt/τ)·ln(1 − F_t) (the terms that integrate to F). p(c) = the chance a deposit
  formed in cell c survives (centre D = z_f + B − E: D > −h, D ≥ h for `destroy`, body still in the
  build range) integrated over that time distribution (exact within a frame, D linear in the instant)
  and 32 tonnage quantiles; p̄_i = p averaged over tract i with the position weights. One formation
  rate per type: **λ_i = N\* · φ_i / Σ_j φ_j p̄_j**, so Σ λ_i p̄_i = N\*. Petroleum forms at its charge,
  `formation_time: cause` types before the history. Cap: planet preservation below
  1/`deposits.max_formed_per_survivor` (new preset key, default 20) stops formation there (census
  `capped`, a WARN). materialise now draws the formation frame ∝ the cell's hazard (was: the best
  frame). Pass 2 replays every frame at **every tract cell** (was: the chosen cells), recording the
  column and the hazard; the chosen cells' ledger series come from it, identical to before.
  Porphyry/MVT/SEDEX/petroleum `balance.count_mult` 10/20/20/3 → 1 (the key stays, a pack lever).
  Census per type: `target, preservation, formation_mult, capped, expected_formed, expected`
  (survivors), `drawn` (formed), `expected_at_positions` (Σ p over where the formed ones landed);
  forming events carry `p_survive`. CLI line and viewer census table updated; FORMAT.md updated.
- **The MVT driver** (`deposits/regional.rs`, `mvt_zn_pb.json`). Two fields derived from every frame
  on reading (not recorded; RecordedHistory untouched): `deeptime:basin_brine` = tapered share, within
  `deposits.brine_flow_km` (new preset key, default 600 km = USGS SIR 2010-5070-A's MVT tract
  criterion, platform carbonates "less than ~600 km inboard of Phanerozoic orogenic belts"), of hot
  (burial temperature 50→100 °C; MVT fluids 75–200 °C, Garven 1985, Leach et al. 2005), aggrading
  (≥ 10 m/Myr), continental basin; `deeptime:regional_head_drop_m_km` = (max within the reach of the
  50-km head of ground rising ≥ 0.1 mm/yr − the cell's head) / reach (full at 3.3 m/km, Garven & Freeze
  1984's base case; "MVT formed mainly during contractional events … platform carbonates with some
  hydrological connection to orogenic belts", Leach et al. 2001; topography the dominant drive,
  Bethke & Marshak 1990). MVT = source(basin_brine, full at 0.035) · driver(head drop) · pathway(faults,
  crust ≥ 30 km) · trap(carbonate_platform 0.02→0.2) · host(#carbonate, dolostone 1.5, new
  `hosts.full_at` 0.05 = ~100 m of carbonate in the 2 km host window). Windows are octagons of the
  disk's area (square + two diagonal passes), integer sums / sliding maxima, exact under east–west
  shifts (the drift test stays bit-exact). Tried first and rejected: brine share with a salinity term
  and head drop from any high ground → MVT on 3.6 % of the planet, 9–31 sites (the drive and the brine
  saturate over most continents at a 600 km reach); the hot-basin and active-orogen terms are what
  discriminate. Viewer ramps `basin_brine`, `head_drop` added to ramps.json.
- **Schema.** `hosts.full_at` in `deposit_type.schema.json` and `types.rs` (default 1, validated
  (0, 1]; Java's codec ignores it). DepositParams gains `max_formed_per_survivor`, `brine_flow_km`
  (serde defaults; the preset JSONs are not edited).
- **Counts** (Mac, `stage deposits --in` the six group checkpoints of `stage structure --preset
  pipeline_quick_16k --start {scattered_continents,supercontinent} --length-myr 500 --set seed={1,2,3}`
  on master b8909e4; baseline = master with the calibration multipliers). Sites (target):

  | type | baseline median (range), present | now median (range), present | targets N\* |
  |---|---|---|---|
  | porphyry | 8 (5–34), 6/6 | 5.5 (3–9), 6/6 | 5.9–12.3 (permissive 40–82 k km², preserved 27–59 %) |
  | MVT | 0.5 (0–5), 3/6 | 4.5 (0–12), 5/6 | 2.6–11.0 (permissive 52–220 k km²) |
  | SEDEX | 1.5 (0–6), 4/6 | 0.5 (0–1), 3/6 | 0.13–0.44 |
  | petroleum | 2.5 (0–4), 5/6 | 3.5 (1–5), 6/6 | 2.1–4.5 |
  | coal | 6.5 (1–9), 6/6 | 21 (14–53), 6/6 | 15–44 |
  | placer Au | 7 (3–10) | 6 (2–13) | children of eroded porphyries |

  Sites track `expected_at_positions` to within a site on every landscape (the ledger agrees with p).
  The one MVT-less landscape (seed 2 scattered) formed 15 whose camps all fell on crust that eroded
  (Σ p = 0.03 at their cells against 2.95 planned): clustering variance, not bias.
- **Tests.** New: `survival::tests` (interval integration, `survives` = ledger + too-deep for every
  built-in type, uniform erosion = closed form, plan hits target and caps), `regional::tests` (windows
  = naive, octagon radius/density/shift-exactness, basin beside orogen), `recorded_tests::
  the_survivor_formula_hits_its_target_in_expectation` (block favourable 100 Myr, 40 m/Myr erosion:
  engine p̄ = closed form 0.4726 within 0.01; 300 planets: mean survivors 2.89 vs 3 ± 0.10, formed
  6.26 vs 6.35), `recorded_tests::mvt_forms_where_a_basin_lies_beside_carbonate` (platform A beside a
  hot foreland basin before a rising orogen: F > 0.5 and all sites there; platform B 1,000 km away,
  no uplift, or a cold basin: F = 0, no sites). Determinism across thread counts: the coupled
  pipeline test (1 vs 3 threads, sites/bodies/events/census identical) and the resume-in-stage test
  pass unchanged. Pipeline tests that list the fields every built-in type reads now include the
  derivable regional names.
- **Gates** (Mac, before the master merge): `cargo test --workspace --locked` all pass (core 214 + 2
  ignored), `cargo clippy --workspace --all-targets --locked -- -D warnings` clean, `cargo fmt --all
  -- --check` clean, `cargo deny check licenses bans` ok, `cd neoforge && ./gradlew build test` →
  tests=118 failures=0 (the synthetic deposit world: 72 sites). GameTests not run (no Java change).
- **Cost.** `stage deposits --in` seed 1 scattered (Mac, 6 threads, `--verbose` phase lines): pass 1
  ≈ 0.1 s a frame (read + regional fields 0.07, fold 0.04), pass 2 over 14,522 tract cells 4.2 s,
  survival + placement 0.02 s, 12 s wall with the 630 MB checkpoint load (the calibration quoted 4.1 s
  for the old stage). The regional fields are computed only when an enabled type reads them, their
  window normalisation is cached per grid and radius.
- **Viewer bundle** review/out/p1-deposits-survivors (seed 1 scattered, deposit layers + inputs incl.
  the two regional fields, 3.8 MB, not published). Looked at: MVT favourability in belts on continental
  margins beside orogens (strongest along one southern margin), porphyry along arcs, the brine field as
  blobs round the hot basins, the head drop as octagonal plateaus round rising orogens (a max-within-
  reach field). Chains: MVT site ← basin_subsidence + orogeny_phase near where the crust was when it
  formed; porphyry ← subduction_onset/orogeny_phase; oil ← petroleum_generation ← basin_subsidence.
- **Decisions for Ben** (not asked): (1) porphyry lands at 5.5 median (targets 6–12), below the
  plan's "about 11": ρ_P50 × our permissive arcs (0.5–1.1 % of the planet). Using the mean of the USGS
  density distribution (split lognormal through P90/P50/P10: porphyry 17.8, ×1.19) would be the
  statistically proper expectation but also multiplies petroleum's placeholder by 3.8. (2) SEDEX
  (placeholder density, no USGS value) is on a minority of planets now. (3) Coal triples (placeholder
  30/10⁵ km² over 50–146 k km² of coal tracts). (4) Survivors count everything still in the build
  range; counting only deposits within ~1 km of the surface (USGS "exposed permissive rock") would add
  deeper extra sites.
- **Next.** Main session merges the branch; Ben's answers on (1)–(4); P1-17 proper.

### 2026-09-29 — Workbench: a run over its memory cap is refused before it starts; the default cap is 8 GiB (agent worktree agent-afd5b3eb4fa7751da, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **The bug.** Ben's `pipeline_full_32k`, supercontinent, 1000 Myr: `POST /api/preset` estimated `peak_bytes`
  5,532,931,904 against the 4 GiB default cap and still answered `can_run: true`; the run started and failed 15
  minutes in ("needed 4300143147 bytes, cap 4294967296 bytes"). Reproduced on a private release workbench (Mac,
  port 8653, before the fix): the same 5,532,931,904 and `can_run: true`. Why 4.30 GB and not 5.53 GB: the cap is
  only checked at `ctx.reserve(..)` calls (in-use bytes + the request), the estimate is the *peak*; the death is
  consistent with the surface stage's first `reserve(estimate / 2)` (2.65 GB) on ~1.65 GB in use (inferred from
  the code, not from a trace). (The client-side
  `overCap()` of the UI overhaul landed after Ben's run, at 03:29; the server never checked.)
- **The rule.** `editor.rs`: `Catalog::effective` holds the estimate to the run's cap (`cap_gib`, the editor's
  Memory field; absent, null or 0 = the workbench's default; a number below 0.25 GiB is a problem of its own, the
  game's minimum) and answers `can_run: false` with the reason among `problems`, plus `needed_bytes` and
  `cap_bytes`. Wording: "needs about 5.7 GiB (the estimate, 5.2 GiB, and 512 MiB for the server itself); the
  memory cap is 4 GiB: raise the cap in Memory, or shorten the run or pick a smaller preset".
  **The margin is measured, not guessed.** What a run needs of the cap is the estimate's peak plus
  `SERVER_OVERHEAD_BYTES` = 512 MiB (a preset that estimates 0 needs 0). The estimate runs low by an almost
  constant amount, not a factor (the cap's gauge counts the whole process): private release workbench, a
  finished run's `peak N MiB` summary line minus its `session::estimate` peak, all `supercontinent`:

  | preset | Myr | estimate | measured peak | difference |
  |---|---|---|---|---|
  | pipeline_quick_16k | 100 | 0.945 GB | 1.347 GB (1284.8 MiB) | +0.40 GB |
  | pipeline_quick_16k | 250 | 1.039 GB | 1.442 GB (1375.0 MiB) | +0.40 GB |
  | pipeline_quick_16k | 500 | 1.196 GB | 1.604 GB (1529.5 MiB) | +0.41 GB |
  | pipeline_full_32k | 30 | 3.092 GB | 3.578 GB (3412.7 MiB) | +0.49 GB |
  | pipeline_full_32k | 250 | 3.645 GB | 4.088 GB (3898.7 MiB) | +0.44 GB |

  (Before the fix it was also plain that the estimate was not the failing number: 5.53 GB estimated against a
  4.30 GB death. The margin only matters within ~0.5 GB of a cap.) The end-of-run `bytes_peak` in the progress
  is higher still (the final checkpoint clones the world: +0.16 to +0.97 GB over the summary peaks above); it is
  not cap-checked and not counted.
- **The request.** `runs.rs` `Workbench::submit` (the `POST /api/runs` path, and `deeptime watch --preset`)
  resolves the cap (0 = the workbench default), calls `Catalog::within_cap` and refuses with the same words
  (HTTP 400) before anything is created (no run, no folder); the run keeps the resolved cap in `run.json`
  (`mem_cap`; runs from before, which stored 0, read back as the core's 4 GiB). An invalid `cap_gib` is a 400
  too (it used to fall back to the default silently). Run cards carry `mem_cap`; `/api/catalog` carries
  `mem_cap_bytes`, `min_cap_gib` and `server_overhead_bytes`.
- **The default cap (Ben, 2026-09-29, "8 GiB (Recommended)", in the decisions log).** `deeptime watch --mem-cap N`
  (bytes; 0 = the core's 4 GiB, unchanged, as is the in-game editor) is the workbench's default:
  `Catalog::with_default_cap`, used by the estimate check, the request check and `create`. `tools/viewer/mac-watch.sh`
  passes 8 GiB (8589934592) and takes `--mem-cap SIZE` (bytes, `8G`, `12GiB`, `6.5G`): checked with a stub `ssh`
  (default 8589934592; 12GiB -> 12884901888; 6.5G -> 6979321856; `lots` -> exit 2), the live 8642 workbench not touched.
- **The page.** `workbench.js`: the Memory field starts at the server's default and says so ("8 GiB · the default", "4 GiB
  for this run · default 8 GiB"); the game's tooltip ("Default 4 GiB") is replaced by the workbench's own text; a new cap
  asks the server again (it decides); the reason is a red problem in the list whose click unfolds and focuses the
  Memory field; the estimate tile shows the needed memory (estimate + server, tooltip says so), the bar against the
  cap and "cap 8 GiB · default"; a run's panel shows its memory cap. Headless shots (`tools/viewer/shot.py` against a
  private debug workbench on the Mac, `--mem-cap 8589934592`, port 8654 through an ssh forward, 8642 untouched;
  in `review/out/workbench-memory-cap/`, not committed): `1-default-8gib-runs.png` (pipeline_full 32k, supercontinent,
  1000 Myr: 7.03 GiB needed after merging master, cap 8 GiB · default, Run on, "Near the memory cap"),
  `2-cap-4gib-refused.png` (cap 4: the red reason, Run off), `3-reason-clicked-memory-field.png` (Memory folded, the
  reason clicked: it unfolds, scrolls to and focuses `f_cap`; asserted in the page), `4-run-panel-memory-cap.png`.
- **Tests.** `editor::tests::a_run_over_its_memory_cap_is_refused_and_a_raised_cap_allows_it` (refuses at the 4 GiB
  default with the words above; `cap_gib` 8 allows it; the edge either side of the needed bytes; `--mem-cap` 8 GiB
  allows it by default, `cap_gib` 0 = default, `cap_gib` 4 refuses again; 0.1, -3 and "lots" are refused) and
  `watch::tests::the_run_request_refuses_a_run_over_its_memory_cap` (POST /api/runs: 400 with the form's words, for no
  cap, 4 and 0; 0.1 refused; nothing created, no run folder; `cap_gib` 8 queues it (queue held), run keeps 8 GiB;
  `--mem-cap` 8 GiB queues it with no field or 0, and `cap_gib` 4 refuses).
- **Gates** (Mac, after merging master 05c772e): `cargo fmt --all -- --check` clean; `cargo clippy --workspace
  --all-targets -- -D warnings` clean; `cargo test --workspace --no-fail-fast` all ok (25 in the cli binary, 205 + 2 ignored in core, …);
  `cargo deny check licenses bans` ok.
- **End to end, the run that failed.** On the private release workbench (before the fix, cap 8 GiB in the request),
  `pipeline_full_32k`, supercontinent, 1000 Myr ran past the 8.8 % where Ben's run died: 11.2 % when cancelled, peak
  4.65 GB, 3.44 GB in use (the 4 GiB run had died at 4.30 GB). It was cancelled there (well over an hour to finish on a shared
  Mac); the finished peak is unmeasured. **Master's new stages raised that preset's estimate** (young, amplify): 5.53 GB
  became 6.6 GiB (+0.5 GiB server = 7.1 GiB needed), which still fits 8 GiB but is "near the cap"; 4600 Myr full 32k
  (14.6 GB before the merge) is refused at 8 GiB.
- **Not done / next.** The live 8642 workbench runs the old binary: restart it from the main checkout after merging
  (`tools/viewer/mac-watch.sh`, which now passes `--mem-cap 8G`). Ben's later decision (log 2026-09-29, spill the
  recorded history to the SSD instead of failing at the cap) makes the deposit estimate flat in length: when it
  lands, `over_cap`'s estimate and the 512 MiB constant should be re-measured the same way (the table above is the
  method: run, read the `peak` line, subtract the estimate).

### 2026-09-29 — P1-17 partial: calibration with the real climate (agent worktree agent-ae48007457b607716, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Why.** Ben, 2026-09-29, "Now, beside P1-06 (Recommended)": recalibrate erosion and uplift with the climate on,
  turn the ice-sheet feedback on once elevations look Earth-like, retune deposit counts; P1-17 proper does the final
  numbers. Baseline (master 85b5671, `pipeline_quick_16k`, 500 Myr, seed 1): land median 2.35 km (scattered) /
  1.74 km (supercontinent), 56 % / 45 % of the land above 2 km, Köppen E 53 % / 64 %, 31–32 % of the land glaciated with
  the feedback *off*, 8–9 sites (porphyry 1, no MVT or SEDEX, oil 0–1), 5,374–7,830 pluton bodies.
- **References.** Earth's hypsometry from GMT's `earth_relief_01d` (SRTM15+, the P1-05 Earth check's DEM, 720 × 360),
  cos φ-weighted like the new CLI line: land 0.289, land mean 784 m, median 427 m, p90 2,022 m, above 1 / 2 / 3 km
  0.253 / 0.102 / 0.048, sea median depth 4,062 m, shelf (0–200 m) 5.4 % of the sea; share of the planet per 1-km band
  −7…+6 km `0.003/0.137/0.229/0.170/0.060/0.036/0.076/0.216/0.044/0.016/0.009/0.004/0.001` (without Antarctica and
  Greenland: median 388 m, p90 1,408 m). Köppen land shares: Peel, Finlayson & McMahon 2007 (HESS 11:1633) B 30.2 %,
  D 24.6 %, A 19.0 %, C 13.4 %, E 12.8 %; this model on Earth's relief (P1-05 check) A 0.17 / B 0.26 / C 0.10 /
  D 0.25 / E 0.23. Glaciated land today ≈ 10 % (Antarctica and Greenland, ~15.8 of ~149 M km²). Continental crust
  41 % of the surface under 29 % land (Cogley 1984).
- **What was wrong (diagnosis from the runs below).** (1) *Domes:* the plate model's Andean uplift acts over
  Cortial's 1,800-km band and its collision surges over up to 4,200 km (Earth-normalised), so the whole of every
  continent rose: `orogenic_uplift_m_myr` was non-zero on 35 % of the planet at 35 m/Myr mean; with the real climate's
  dry interiors, n = 2 stream power (E ∝ S²) cannot wear low-slope plateaus down. (2) *Drowning, once the domes were
  gone:* narrowing both bands took the land from 0.23 to 0.15 of the planet (0.13–0.16 in the variants that followed),
  for four reasons found one at a time: P1-04's isostasy 0.6 lowers the ground 400 m per km eroded (Airy ≈ 180–200 m); the marine diffusion's
  arithmetic face mean erodes a coastal land cell at ≈ D₀/2 = 150 m²/yr at D₀ 300, so shores retreated ~250 km in
  500 Myr (wide light-blue platforms round every continent in the first maps); fewer island arcs emerged, so the
  continental crust shrank from 0.30 to 0.21–0.25 of the planet (arc emergence is the plate model's continental
  crust factory); and with Earth-like flooding (71 % of continental crust dry) a 30 % continental planet can only have
  ~21 % land — P1-03's `ocean_fraction` made the continental crust, not the water. (3) *Shallow oceans:* Cortial's
  ridge at −1 km and damping 0.04 mm/yr towards the −10 km trench put 57–59 % of the planet between −3 and 0 km
  (Earth 17 %); median sea depth 1.7 km. (4) *Deposits:* see the deposit bullet.
- **Changes** (the four `presets/pipeline/*.json`; defaults where the defaults are the calibrated values):

  | parameter | before → after | why |
  |---|---|---|
  | `tectonics.subduction_distance_km` | 1,800 → 800 | Andean orogen width, trench to foreland thrust front: 400–900 km (Central Andes); uplift confined to the orogen instead of doming the continent |
  | `tectonics.collision_distance_km` | 4,200 → 2,000 | half-weight radius ~1,300 km: a Tibet-sized plateau, not a continent-sized one |
  | `tectonics.arc_uplift_mm_yr` | 0.1 → 0.3 | island-arc edifices build from −4 km to the sea in ~10–15 Myr (arc crust production 60–120 km³/km/Myr, e.g. Izu–Bonin–Mariana); keeps the continental crust from shrinking: 0.21 → 0.31 of the planet at the end (i8K1od vs arc3) |
  | `tectonics.continental_shelf_share` (new) | — → 0.29 | Earth's 29 % land on 41 % continental crust (Cogley 1984): continents start with their outer 29 % as a shelf (0 to −130 m, Shepard's mean shelf break), so `ocean_fraction` is the water's share at the end too |
  | `tectonics.ridge_elevation_m` | −1,000 → −2,500 | Earth's mean ridge crest depth (Parsons & Sclater 1977: 2.5 km) |
  | `tectonics.oceanic_damping_mm_yr` | 0.04 → 0.05 | with the deeper ridge: 3.2 km at 20 Myr, 4.2 at 50, 5.5 at 100 (GDH1 4.2 / 5.0 / 5.5); older crust is too deep (Cortial's exponential goes to the −10 km trench, GDH1 flattens at 5.65 km) |
  | `surface.k_spl` (+ default) | 3e-10 → 2e-9 | P1-04's K was set on the stand-in's wetter interiors; 2e-9 gives Earth's hypsometry with the real runoff (runs below; 1e-9 left medians 0.6–1.0 km) |
  | `surface.isostasy` (+ default) | 0.6 → 0.8 | Airy ρc/ρm ≈ 2.7/3.3; 0.6 was chosen against the domes' continent-wide uplift and let cratons erode below the sea once uplift is confined to orogens |
  | `surface.kd_marine_m2_yr`, `wave_base_m` (+ defaults) | 300, 100 → 100, 60 | Landlab's `SimpleSubmarineDiffuser` defaults; at 300 the coasts retreated ~250 km (above) |
  | `climate.ice_sheet_feedback` (+ default) | 0 → 1 | on now that the land is Earth-low: 3–18 % glaciated at the end of 500 Myr (Earth 10 %); on Earth's relief 9.7 % (Earth check below) |
  | `structure.pluton_length_km` (default) | [10, 3, 40] → [25, 8, 80] | a pluton stands for an intrusive suite (Tuolumne: ~1,100 km², ~45 × 25 km; Bateman & Chappell 1979) |
  | `structure.pluton_rate_km2_myr` (default) | 1e-4 → 1e-5 | ×(10/25)^2.6 = 0.09 (V ∝ L²·T ∝ L^2.6): the same emplaced volume in a tenth as many bodies |
  | porphyry `balance.count_mult` | 1 → 10 | see deposits |
  | MVT, SEDEX `balance.count_mult` | 1 → 20 | see deposits |
  | petroleum `balance.count_mult` | 1 → 3 | see deposits |
  | MVT driver `topographic_gradient_m_km` curve | [[0,0],[5,0.5],[15,1]] → [[0,0],[1,0.5],[4,1]] | Garven 1985's topography-driven flow is basin-scale (~1 km over a few hundred km); 5–15 m/km picked mountain fronts, where the deposits then eroded away |

  Not changed: n = 2, m = 1, `runoff_ref_m_yr` (degenerate with K for m = 1), subduction uplift 0.6 mm/yr
  (0.4 moved the median by 50 m), the climate model (the Earth check is unchanged), `carbonate_clastic_m` (×5 gave no
  more carbonate platform), the plate-only presets and `TectonicParams::default` (Cortial's constants; the plate-only
  presets have no surface to flood their continents).
- **Code changes, only where a parameter could not reach the cause.** (1) `tectonics.continental_shelf_share`
  (`params.rs`, `plates/init.rs`): the initial continental area is (1 − ocean_fraction)/(1 − s) and its part nearest the
  ocean (Dijkstra from the coast inside the continents) is born as a shelf; s = 0 (the default) is P1-03's rule
  bit for bit (no extra draws). Test `plates::tests::the_shelf_share_floods_the_continents_margins` (three starts: dry
  share = 1 − ocean_fraction to one sample, continental share = dry/0.71, shelf between 0 and −130 m). (2) CLI
  (`run.rs`): `stage` prints a `hypsometry` line (true-area land share, land mean and p10–p90, above 1/2/3 km, sea
  median depth, shelf share, the continental crust's share and dry share, 1-km bands) and an `evolution` line (land
  share and median every 50 Myr from the recorded frames). (3) `climate-dem` repeats the snapshot five times with
  the previous ice sheets when the preset turns the feedback on, so the Earth check covers it.
- **The runs** (all `deeptime stage deposits`, 500 Myr, `pipeline_quick_16k` variants written by a throw-away script
  (`build/tune/`, not committed), Mac M6 mini, 4–6 threads each, 3 at a time, load 10–48 from other agents: **timings
  are noisy**; scattered = `scattered_continents`, super = `supercontinent`, seed 1 unless stated; land = true-area
  land share; z50/z90 = land elevation median/p90 (m); >2 = land share above 2 km; sea = median sea depth (m); ice =
  glaciated land share; feedback off until "sh29ice"):

  | run | change (cumulative unless "alone") | start | land | z50 | z90 | >2 | sea | Köppen A/B/C/D/E | ice |
  |---|---|---|---|---|---|---|---|---|---|
  | base | master 85b5671 | scattered | 0.233 | 2352 | 5587 | 0.56 | 1710 | .10/.12/.21/.04/.53 | .307 |
  | base | | super | 0.215 | 1743 | 4602 | 0.45 | 1714 | .03/.08/.09/.16/.64 | .317 |
  | K1e9 | K 1e-9 alone | scattered | 0.205 | 1624 | 6106 | 0.44 | 1705 | .13/.16/.22/.06/.43 | .249 |
  | sd800 | band 800 km alone | scattered | 0.194 | 1752 | 4152 | 0.44 | 2685 | .04/.22/.16/.18/.40 | .130 |
  | sd800rc2k | + collision 2,000 km | scattered | 0.153 | 1026 | 2298 | 0.15 | 2262 | .13/.21/.26/.09/.31 | .109 |
  | +K1e9 | + K 1e-9 | scattered | 0.180 | 943 | 3744 | 0.25 | 2182 | .09/.19/.18/.15/.38 | .130 |
  | +od07 | + damping 0.07 | scattered | 0.153 | 735 | 2116 | 0.12 | 3542 | .12/.34/.17/.18/.19 | .022 |
  | i8 | sd800rc2k + isostasy 0.8 | scattered | 0.144 | 1025 | 2661 | 0.18 | 2373 | .16/.21/.16/.19/.29 | .062 |
  | i8K1 | + K 1e-9 | scattered | 0.127 | 620 | 2038 | 0.10 | 1968 | .19/.20/.19/.13/.29 | .079 |
  | i8K1od | + damping 0.07 | scattered | 0.137 | 950 | 2507 | 0.16 | 3817 | .18/.24/.24/.11/.23 | .062 |
  | i8K1od | | super | 0.125 | 694 | 1863 | 0.08 | 3268 | .21/.24/.21/.10/.24 | .046 |
  | i7K1od | isostasy 0.7 instead | scattered | 0.158 | 631 | 2339 | 0.13 | 3626 | .11/.42/.18/.15/.14 | .020 |
  | ct150 | i8K1od + collision threshold 150 km | scattered | 0.144 | 773 | 2958 | 0.20 | 3341 | .21/.18/.17/.16/.29 | .111 |
  | arc2 / arc3 | i8K1od + arc 0.2 / 0.3 mm/yr | scattered | 0.146 / 0.194 | 626 / 722 | 1841 / 2130 | 0.08 / 0.12 | 3462 / 2661 | .17/.25/.18/.20/.20 (arc3) | .045 / .034 |
  | km100 / km30 | i8K1od + marine D₀ 100 (wave base 60) / 30 | scattered | 0.160 / 0.179 | 750 / 762 | 2391 / 2405 | 0.14 / 0.15 | 3427 / 3707 | .20/.22/.15/.11/.32 (km100) | .079 / .124 |
  | km100 / km30 | | super | 0.164 / 0.210 | 591 / 969 | 1539 / 2516 | 0.06 / 0.19 | 3933 / 3588 | | .063 / .026 |
  | km100arc3 | km100 + arc 0.3 | scattered / super | 0.199 / 0.176 | 746 / 602 | 2149 / 1367 | 0.12 / 0.03 | 3376 / 3526 | .17/.22/.20/.18/.23 · .30/.43/.18/.07/.02 | .045 / .002 |
  | sh29 | + shelf share 0.29 | scattered / super | 0.324 / 0.331 | 888 / 983 | 2538 / 3100 | 0.17 / 0.24 | 2346 / 2316 | .19/.22/.19/.16/.24 · .20/.24/.24/.10/.23 | .047 / .053 |
  | sh29km30 | sh29 with D₀ 30 | scattered / super | 0.304 / 0.318 | 757 / 893 | 1965 / 3059 | 0.10 / 0.23 | 2774 / 2817 | .22/.25/.23/.11/.20 · .22/.26/.25/.07/.21 | .041 / .060 |
  | sh29ice | + ice-sheet feedback 1 | scattered / super | 0.321 / 0.296 | 1047 / 758 | 2949 / 2104 | 0.21 / 0.11 | 2343 / 2519 | .11/.20/.19/.13/.36 · .14/.30/.19/.11/.26 | .124 / .091 |
  | plu | + plutons 25 km, 1e-5 | scattered | identical to sh29ice (the structure stage does not feed back); 10,832 → 1,296 pluton bodies | | | | | | |
  | oc | + ridge −2.5 km, damping 0.05 | scattered / super | 0.273 / 0.308 | 723 / 949 | 2171 / 2857 | 0.12 / 0.19 | 3210 / 3421 | .20/.28/.20/.12/.20 · .13/.24/.17/.21/.26 | .045 / .053 |
  | ocU4 | oc + subduction uplift 0.4 | scattered | 0.277 | 676 | 2157 | 0.11 | 3417 | .09/.26/.16/.25/.23 | .052 |
  | cc100 | ocK2 + carbonate_clastic_m 100 | scattered / super | 0.250 / 0.317 | 586 / 668 | 2123 / 2234 | 0.11 / 0.12 | 3490 / 3250 | no more carbonate platform (2.6 / 3.5 % vs 3.8 / 4.4 %): rejected | .082 / .030 |
  | **ocK2 = final** | oc + K 2e-9 | scattered / super | **0.270 / 0.282** | **601 / 624** | **2095 / 1679** | **0.11 / 0.07** | **3229 / 3334** | **.18/.21/.24/.19/.18 · .19/.25/.16/.16/.23** | **.058 / .067** |
  | final seed 2 | | scattered / super | 0.273 / 0.327 | 634 / 862 | 1941 / 2803 | 0.09 / 0.20 | 3549 / 3472 | .17/.26/.22/.19/.16 · .23/.27/.19/.11/.19 | .037 / .033 |
  | final seed 3 | | scattered / super | 0.287 / 0.314 | 785 / 1055 | 2093 / 3749 | 0.11 / 0.31 | 3207 / 3308 | .13/.28/.17/.21/.22 · .09/.17/.20/.16/.38 | .045 / .177 |

  Continental crust share / its dry share at the end: base 0.305 / 0.71; i8 0.23 / 0.62; km100arc3 0.28 / 0.70; final
  0.36–0.41 / 0.65–0.79 (Earth 0.41 / 0.71). Land share over the final run (scattered, every 50 Myr from −450):
  0.36, 0.34, 0.33, 0.31, 0.29, 0.30, 0.29, 0.29, 0.30, 0.28 with the median between 0.49 and 0.85 km throughout.
  Mass balance: `mass_residual_km3` between −4e-4 and −1.9e-3 km³ on every run (baseline −1.1e-3 / −1.9e-3) with
  4.9e7–1.05e8 km³ eroded (baseline 9.9e7 / 5.9e7): still exact to 1e-11 of the eroded volume.
- **Final (the built-in `pipeline_quick_16k` now), before → after, seed 1:** scattered / supercontinent land median
  2,352 / 1,743 → 601 / 624 m (Earth 427); p90 5,587 / 4,602 → 2,095 / 1,679 m (Earth 2,022); land above 2 km 0.56 /
  0.45 → 0.11 / 0.07 (Earth 0.10); land share 0.233 / 0.215 → 0.270 / 0.282 (target 0.30); sea median depth 1,710 →
  3,229 / 3,334 m (Earth 4,062); bands (−7…+6 km) scattered `0.127/0.049/0.073/0.158/0.111/0.047/0.164/0.186/0.055/
  0.017/0.006/0.004/0.003` — bimodal (a land mode at 0–1 km, an ocean mode at −4…−3 km), but with too much ocean
  below −6 km (old crust, Cortial's exponential) and between −1 and 0 km (shelves and epeiric seas); Köppen .10/.12/.21/
  .04/.53 / .03/.08/.09/.16/.64 → .18/.21/.24/.19/.18 / .19/.25/.16/.16/.23; glaciated land (feedback off → on)
  0.31 / 0.32 → 0.058 / 0.067; land 9.2 / 8.8 °C (Earth land ≈ 8.5 °C); pluton bodies 7,830 / 5,374 → 1,192 / 885.
  Over the three seeds × two starts: land 0.27–0.33, median 0.60–1.06 km, above 2 km 0.07–0.31, ice 3–18 %, E 16–38 %
  (the one cold outlier, seed 3 supercontinent, carries a young collision plateau: p90 3.7 km, 31 % above 2 km).
- **Deposits.** Two things kept the counts low besides the landscape. (a) The §8.6 formula ρ·A·F: the permissive arcs
  cover 0.5–1 % of the planet (the plan's 1 %) but at mean F ≈ 0.33 (the plan's "about 11 at 16k" took F ≈ 1); and
  the preservation ledger erodes half to four fifths of the deposits drawn over 500 Myr, while USGS densities count
  what is preserved today. (b) MVT's and SEDEX's densities are placeholders (the type files say so) that give 0.1–0.3
  deposits on a 16k planet. Balance multipliers (`balance.count_mult`, the per-type gameplay lever; each file has a
  note) porphyry 10, MVT 20, SEDEX 20, petroleum 3, and MVT's driver on basin-scale gradients. Sites (expected) on the
  six final landscapes (deposit stage re-run on saved group checkpoints; seed 1 = the bundles):

  | run | porphyry | coal | MVT | SEDEX | petroleum | placer Au |
  |---|---|---|---|---|---|---|
  | base scattered / super | 1 (1.0) / 1 (2.5) | 6 (12.3) / 5 (14.9) | 0 (0.1) / 0 (0.1) | 0 (0.0) / 0 (0.0) | 0 (0.5) / 1 (3.5) | 1 / 2 |
  | seed 1 scattered / super | 5 (21.2) / 34 (42.2) | 5 (20.3) / 9 (13.8) | 0 (0.7) / 0 (4.3) | 1 (3.5) / 0 (2.2) | 4 (4.0) / 3 (2.8) | 8 / 7 |
  | seed 2 scattered / super | 6 (32.7) / 11 (21.0) | 7 (16.1) / 8 (11.7) | 0 (0.3) / 1 (4.3) | 0 (1.5) / 2 (5.2) | 2 (1.5) / 0 (3.4) | 10 / 7 |
  | seed 3 scattered / super | 8 (29.0) / 8 (18.9) | 1 (6.2) / 6 (22.7) | 5 (1.8) / 1 (3.9) | 3 (2.1) / 6 (4.4) | 2 (3.6) / 4 (2.7) | 3 / 7 |

  Porphyry median 8 (5–34; the seed-1 supercontinent's long active margin draws 49); MVT on 3 of 6 planets, SEDEX
  on 4 of 6, petroleum on 5 of 6. Where they sit (bundles, looked at): porphyries in lines along active continental
  margins where the porphyry favourability draws its arcs, placers a few cells downstream of eroded ones; the SEDEX
  site of the scattered run in the brightest patch of the SEDEX map (a rifted euxinic basin in the western continents);
  coal on coastal plains and polar shelves; oil traps on margins and in basins. MVT stays the weakest family: carbonate
  hosts are scarce (carbonate platform facies on 1–5 % of cells; `host/limestone` ~2 %) and many MVT draws erode.
  Facies at the last frame: carbonate platform 1.0–4.4 % of cells, evaporite 1.1–3.3 %, euxinic 1.5–4.1 %, coal
  0.1–0.6 % (baseline 4.2 / 3.1 / 2.0 / 0.3 — the baseline's shallow 1–2 km oceans gave more platform).
  Porphyry `drawn` ran about twice `expected` over the first 29 runs; coal did not. Both types' draws come from a
  fixed per-type stream (`deposits/<id>` of seed 1) and every run used seed 1, so the realisations are correlated; the
  sampler itself has its test (`count::poisson_counts_match_the_mean`). Judge counts by `expected`.
- **Earth check** (`deeptime climate-dem --dem target/earth/earth_720.f32 --width 720 --preset pipeline_quick_16k
  --viewer review/out/p1-calib-earth`): with the feedback on the five repeated snapshots settle at 9.3 → 9.7 %
  glaciated land (Earth ≈ 10 %); EBM 15.3 °C, net TOA −0.003 W/m², land 7.7 °C and 563 mm/yr, groups A 0.17 / B 0.26 /
  C 0.10 / D 0.24 / E 0.24, 19 of 24 cities in the right main group and 5 exact — P1-05's result (19/24, 5 exact; the
  feedback moves D 0.25 → 0.24, E 0.23 → 0.24). The climate was not tuned.
- **What the maps show** (review/out/p1-calib-scattered_continents 20 MB and p1-calib-supercontinent 15 MB, pulled
  back, 11 frames of elevation, Köppen, precipitation, temperature and the surface rock (`structure/rock`), and the
  deposit keyframe: sites, favourability of porphyry, MVT, SEDEX, petroleum and coal, the carbonate platform input):
  at 0 Myr green lowland continents with tan-to-brown mountain belts along their active margins and a few interior
  highs, shelves and inland seas in light blue, a drowned continental block (a Zealandia) in the scattered run, ridges
  and old dark basins in the oceans; at −250 Myr the same style, no domes. Köppen: Af/Am/Aw along the equator, BWh/BSh
  in the subtropical interiors and lee coasts, C on mid-latitude west and east coasts, D inland and poleward, ET/EF
  on the polar continents and a few high ranges. Precipitation: ITCZ belt, dry subtropics, wet westerlies, rain
  shadows behind every range. Rock: chalk and shale sea floors, basalt at the ridges, sandstone and mudstone plains,
  laterite over the tropical lowlands, crystalline and metamorphic belts in the orogens, pluton outcrops now a few
  large bodies. Still wrong or doubtful: the continents are low but broad plains dominate (little hilly relief
  between 0.3 and 1 km; P1-06's young landscape and amplification add that); the old ocean floor is too deep (−6…−8 km)
  and the −1…0 km band too wide; E is 16–38 % against Peel's 13 % (the model's own Earth has 24 %).
- **Deviations.** (1) Acceptance on `pipeline_quick_16k` (placeholders until P1-08), as before. (2) The 32k and full
  presets got the same values without their own runs: K acts on drainage areas in real m², so a 32k planet (twice the
  real length scale) erodes more; P1-17 proper should check its hypsometry. (3) Deposit counts use per-type balance
  multipliers rather than new densities (MVT/SEDEX densities stay the placeholders P1-14 recorded).
- **Measured gates** (Mac, on the branch merged with master c79b38f and again with ae27fb7, same results both times): `cargo test --workspace --locked` → the test
  binaries report 23 / 174 (+ 2 ignored) / 32 / 28 / 2 / 7 / 7 / 5 passed, 0 failed (core's 174 include the new
  `plates::tests::the_shelf_share_floods_the_continents_margins`; the coupled pipeline tests — resume on another
  thread count, mass balance, deposits identical on 1 vs 3 threads — pass on the recalibrated preset); `cargo clippy --workspace --all-targets --locked
  -- -D warnings`, `cargo fmt --all -- --check`, `cargo deny check licenses bans` clean; `cd neoforge && ./gradlew
  build test` → tests=118 failures=0 (the synthetic deposit world the JUnit tests read now has 228 sites: the balance
  multipliers apply to it too). GameTests not re-run. The master merge (P1-17's parallel stream-power solve)
  reproduced every calibrated number bit for bit (hypsometry, Köppen, facies, deposits of both starts).
- **Decisions for Ben** (proposals; none asked yet). (1) Deposit counts: the four balance multipliers (porphyry 10,
  MVT 20, SEDEX 20, petroleum 3) make the families appear on a 16k planet; the alternative is to let USGS densities
  count *preserved* deposits (draw ρ·A·F / preservation) — a change to the engine's §8.6 formula. (2) `ocean_fraction`
  now is the water's share on the pipeline presets (continents born with flooded margins), which is what the
  workbench editor's hint already says ("Share of the surface under the sea. Earth: about 0.71."); the plate-only
  presets keep P1-03's crust share (their continents never flood) — confirm, or make the shelf share the default. (3) MVT is still absent on half the planets: accept (essentials
  guarantee what a pack needs) or redesign its driver as a regional one in P1-17 proper.
- **Next.** P1-17 proper: bench and the 32k/full presets' hypsometry; a GDH1-shaped ocean subsidence (an asymptote
  at −5.65 km instead of the trench) if the old-ocean tail matters; MVT's driver as a regional one (an orogen within a
  few hundred km of the platform) and more carbonate on warm shelves; P1-06 adds the relief the plains lack.

### 2026-09-29 — Workbench previews can no longer boot a stale Deep Time jar (agent worktree agent-a2d1ff97d824f3782, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **The bug.** `preview.py install_profile()` ran `install.sh` only when `smoke/<profile>/install.json` was
  missing, and the Deep Time jar in a server set is a copy taken at install time under a name that never
  changes (`deeptime-1.21.1-0.0.1.jar`; install.json recorded only that name). The main checkout's `smoke/mic`
  on the Mac kept the jar of 2026-09-28, so every preview died on `$.deposits_synthetic: unknown key`.
- **The fix (option B, with a record).** Every job now runs `./gradlew jar` on the checkout (about a second
  when nothing changed), compares the built jar's sha256 with the set's copy and replaces the copy when the
  bytes differ (written aside as a non-`.jar` file and renamed into place, so a server that has the old jar
  open keeps its inode; any other `deeptime-*.jar` is removed). Why Gradle and not a hash of the sources
  kept in install.json: Gradle's `jar` task already tracks exactly what goes into the jar (Java sources,
  resources, the built-in presets and the viewer assets `processResources` packs, and through
  `hostNatives` -> `cargoBuildJni` the Rust natives), so the check cannot miss an input the way a
  hand-kept list of trees can, and it costs one second. A source hash is still recorded (see below) but only
  as a label. The jar is refreshed before the block-colour table and the server lock, so nothing stale is
  ever read; a jar that does not build fails the job with Gradle's output instead of booting older code.
- **Where.** `tools/smoke/jarstate.py` (stdlib, Python 3.9-safe: the Mac's `/usr/bin/python3` runs
  `preview.py`): `refresh`, `built_jar` (named by `mod_version`, never "the last one sorted"),
  `source_hash` (sha256 over the contents of `neoforge/`, `deeptime-{jni,core,format}`, the viewer assets,
  `Cargo.toml/lock`; not mtimes, not build outputs, no git needed), `source_rev`, `record`, `status`.
  `preview.py`: `refresh_deeptime_jar` (called by `run_job` right after `install_profile`),
  `preview.py refresh-jar --profile P` (the same step alone: no server, no job, no lock), and `tool_env()`:
  the live workbench (`deeptime watch`, started by `mac-watch.sh`, not by `mac.sh`) has no
  `GRADLE_USER_HOME` / `CARGO_HOME`, so its jobs would have built into a second `~/.gradle` on the Mac's
  small internal disk; `run_cmd` now points them at `deeptime-cache/{gradle,cargo}` when they are unset.
  Checked from a bare `zsh -lc` ssh session (the workbench's environment): `nix develop <profile> -c
  ./gradlew jar` current in 0.8 s, no `~/.gradle` created.
- **Recorded.** `install.sh` merges into install.json `deeptime_jar_sha256`, `deeptime_src_rev` (the
  laptop's `git describe --always --dirty` at the last `mac.sh` sync; the Mac's copy has no usable git, so
  `mac.sh` writes `.sync-rev` beside the tree after `git add -A`, untracked and invisible to flakes; on the
  laptop it asks git), `deeptime_src_commit`, `deeptime_src_hash`, `deeptime_jar_recorded`. A refresh
  updates them. Job status: `jar` {`refreshed`, `sha256`, `was`, `src_rev`, `src_hash`, `seconds`},
  `timings.jar_s`, `install` carries the sha and source, the message reads "refreshed the Deep Time jar
  in smoke/mic: … -> …" (or "the Deep Time jar is current: …") and a refreshed job ends "exported;
  refreshed the Deep Time jar first"; `preview-area.sh` prints a `Deep Time jar:` line. The smoke report
  (`run.py`) has a "Deep Time build" row: sha, revision, tree hash, and "STALE: the sources changed since
  this jar was installed" when the tree no longer matches (it can only warn; it does not build).
- **Tests.** `tools/smoke/test_jarstate.py`, 13 tests on a throwaway checkout and server set with a fake
  Gradle whose jar bytes are a function of the sources: source hash follows content and not mtime, build
  output or docs; refresh installs / leaves same bytes alone (same inode) / replaces atomically (an open
  handle keeps the old bytes) / drops other `deeptime-*.jar` and leaves other mods and `-sources.jar`;
  `built_jar` picks the `mod_version` name over a higher-sorting stale one; record and `status`;
  `.sync-rev` wins over git; and `refresh_deeptime_jar` end to end: stale -> refreshed, again -> current,
  touch -> current, edit -> refreshed, revert -> refreshed, status file carries `jar`, a build failure
  swaps nothing. `python3 -m unittest discover -s tools/smoke -p 'test_*.py'` passes on the laptop
  (3.13), on the Mac's `/usr/bin/python3` 3.9.6 and in its dev shell.
- **Verified on the Mac** (this worktree's own copy at `/Volumes/Graphide/ban/mic/mods/agent-a2d1ff97d824f3782`;
  `smoke/mic` seeded from the main checkout's with the 2026-09-28 jar `a2e4ff02c0ba` put back in, i.e. the
  bug's state; the live workbench on 8642 and the main checkout's `smoke/` were not touched):
  1. `tools/workbench/preview-area.sh --profile mic --box -5698,-1158,-5666,-1126 --preset
     …/deposits_world_16k.json --start supercontinent --length-myr 500` (32 x 32, the `mic` profile): **done**,
     "exported; refreshed the Deep Time jar first", `jar` refreshed `a2e4ff02c0ba` -> `1703ead73d8b`
     (`jar_s` 0.8, the jar was already built by an earlier Gradle run), the server generated the synthetic
     deposit world (`source run:deposits_synthetic`, 42 sites; zero "unknown key" in server.log), 88.7 s
     total (boot 61.0 s, Chunky 12.5 s).
  2. The same command again: **the jar is current**, `1703ead73d8b` unchanged, `jar_s` 1.2, 60.4 s total.
  3. `preview.py refresh-jar --profile mic` around edits made on the laptop and synced by `mac.sh`:
     `touch` DeepTime.java -> current (1.3 s; same bytes, Gradle up to date); a real change to DeepTime.java
     (a new constant) -> **refreshed** `1703ead73d8b` -> `f6889f351c06` (2.7 s, tree hash and revision
     `74b80daa-dirty` change); reverted -> **refreshed** again (`6fa76e763deb`, 1.3 s; a rebuilt jar is never
     byte-identical, so a revert costs one harmless copy); **natives**: a one-string change in
     `crates/deeptime-jni/src/lib.rs` -> refreshed `b216dc736e51` (2.6 s including the Rust rebuild), and
     the new string is inside `natives/deeptime/macos-aarch64/libdeeptime_jni.dylib` in the installed jar
     (`unzip -p … | strings | grep -c` = 1); reverted -> refreshed, string gone, `jarstate.py status` says
     current.
  4. `tools/smoke/install.sh standalone` on the Mac ran to the end and printed the record line; `run.py`'s
     "Deep Time build" row was exercised on it (current / STALE / not recorded).
- **Not done / notes.** `smoke/run.py` and the `-PwithMic` gates do not build the jar: they only report it
  (run `install.sh <profile>`, or `preview.py refresh-jar --profile P`, after Java or Rust changes).
  Gradle is not reproducible byte for byte, so a `clean` or a branch switch that makes Gradle rebuild the
  jar looks like a change and is refreshed once. The other profiles of the main checkout's Mac
  (`standalone`, `ev`, `terrablender`) also hold jars from 2026-09-28; each is refreshed by the first
  preview that uses it. To make the live 8642 workbench use this: merge, then restart it from the main
  checkout with `tools/viewer/mac-watch.sh` (it syncs that checkout and points `--preview` at its
  `tools/workbench/preview.py` on the Mac; the server binary itself is a copy).

### 2026-09-29 — P1-06 young landscape, amplification and the detail layer (agent worktree agent-af500836f573f424e, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **What.** Two stages after the deposits in every planet preset with the sections `young` and `amplify`
  (all four `pipeline_*` carry them, `{}` = the defaults): `young` (`surface/young.rs`) and `amplify`
  (`surface/amplify.rs` + `amplify/{window, resample, kernels, tiles, tests}.rs`, `surface/thermal.rs`,
  `surface/detail.rs`). New: `surface.drama` (0..2, default 1; the editor's slider, its lang key and field
  existed, the pipeline presets now carry the path so the workbench lights it up),
  `deeptime_format::TilePackWriter` (tiles in any order, index patched at the end, same bytes as
  `write_tile_pack`), `StageContext.out_dir`, `amplify::register` (`finalize_write` lists the L3 layers in the
  manifest), CLI `stage young|amplify` lines (`young_final=…`, `amplify_final=…`), `--set PATH=VALUE`
  (edits the preset before a run or a checkpoint's preset before a later stage), the before/after views
  (`crates/deeptime-cli/src/amplify_view.rs`, `viewer.js`/`index.html` section), `tools/review/amplify_extras.py`
  (the drama figure), ramps for `young/*` and `amplify/*`, NOTICE.md (MultiScaleErosion, MIT, 964a3d5), FORMAT.md
  ("The detail layer": layer, formula and the C-periodic noise the game adds), plan §8.2 "As built" and §10.3
  "As run". Layers written: `rasters/elevation.dtr` (L3, i16, 1/16 block) and `rasters/detail/amplitude.dtr`
  (L3, u8, 1/16 block); keyframes `young/{elevation, drainage}` and `amplify/{elevation, detail, drainage}`
  (the workbench discovers them, no workbench change).
- **Method.** *Young* (L2): L1 surface → L2 by Catmull–Rom clamped between the two nearest samples (no
  coastal rims), depressions under 5 m filled (deeper ones are L1's lakes), 4 LEM steps of 0.5 Myr with the
  deep-time physics (`pflood`, `routing`, `spl` with erosion–deposition and the critical slope, `diffusion`),
  runoff weights from the climate fields, K from the surface rock, uplift = the last deep-time step's erosion
  rate (`balance`), the clock not advanced. *Amplify*: `doublings(l2_r, l3_r)` ×2 levels (2 in every preset but
  16k full). Per level, all on a window: upsample (terrain, the coarse level's exact drainage area as a prior,
  runoff weight, erodibility, lake footprint grown a cell); relief seeding; Priority-Flood+ε (`flood_repair`;
  outlets are the sea, the inherited lakes and open window edges); 40 erosion iterations (×3 on the levels
  before the last) of multiple-flow drainage area (p = 2) refined from its prior and stream-power incision
  (area term capped at 300 km², ≤ 30 % of the drop to the receiver per iteration, ≤ 3 × the initial drop to the
  lowest neighbour in all, sea = base level, lakes and filled depressions untouched); 24 thermal iterations
  (critical slope 0.6 × (0.25 + 0.75 · drama), ±25 % noise, pairwise excess-proportional transfer, mass
  conserved); 12 deposition iterations (the eroded load along the same weights, settling where the capacity
  falls under the load, capped at half the gap to the lowest higher neighbour: it cannot make a pit); the flood
  again. **No new sinks:** the two floods leave every land cell outside the inherited lakes a strictly lower
  neighbour; the stage counts violations over the finished cores (`notes.amplify.sinks`, 0 on every run) and the
  tests count them on the assembled planet. **Tiles:** every pass is a Jacobi iteration on a window (x wraps when
  the window holds every column, else the edge is replicated; a planet edge is absent), so a tile with a halo of
  `radius + 2 · repair_margin + 3` cells (196 with the defaults; 136 in the first tuning) gives the whole-raster
  bits; the flood is the one non-local pass, its reach (32 cells) is in the halo. The levels before the last
  stream too (`LevelSink`) when the gauge says their buffers would not fit. **Detail layer:** amplitude =
  min(drama · 0.12 · relief5, 6 · drama) blocks × (1 − smoothstep(25, 250 km²)); noise per FORMAT.md.
- **How the parameters were chosen (all in `AmplifyParams`, editable with `--set amplify.KEY=V`).** On the real
  500 Myr `pipeline_quick_16k` planet, by looking at the hill-shaded mountains and reading the flood's
  largest raise. Findings that changed the design: erosion of a surface with an undrained pit carved
  1-cell-wide slots 500 m deep ending in the pit (170 m of fill afterwards) → a flood after the seeding and
  again at the end, filled depressions frozen against erosion, a cumulative cap on each cell's incision
  (3 × its initial drop to its lowest neighbour: the largest fill is now 27 m, at a coast) and an area cap;
  flow exponent 2 (1.5 combed the gullies); relief seeding 0.25; incision 8 m per iteration at 100 km² and
  slope 0.1. A stronger setting (no cap) shows dramatic gullies but cuts 250–400 m off small steep
  islands; a weaker one (cap 0.5) is nearly the interpolation. Ben's call, see below.
- **Measured (Mac, M6, 12 threads, shared: load 18–38 during the runs).** `stage young --in deposits.dtcp` (a
  500 Myr `pipeline_quick_16k` checkpoint made with `stage deposits`, 9 min 38 s) 7.6 s: land p50 2,518 → 2,645 m, p90 5,397 → 5,476,
  max 10,578 → 10,594 (balance uplift keeps the relief), 7,558 lake cells, 972 small depressions filled, sinks 0
  (setup 0.1, route 0.4, spl 1.8, diffusion 5.2 s). `stage amplify` 8192 × 4096 (L3 r = 2, 33.6 M cells; levels
  2048 × 1024 → 4096 × 2048 → 8192 × 4096): **in memory 18.7 s** (passes over both levels: erode 7.6, thermal
  3.3, deposit 3.3, flood 3.2, seed 0.1; sample 0.2, detail 0.4, sink 0.4), **one thread 53.2 s** (erode 24.1,
  thermal 14.1, deposit 8.9, flood 2.9 (serial), seed 0.7; sample 1.2, detail 0.7), streamed with 8 tiles
  25.1 s. Gauge 903 MiB in use before the stage; the last level needs 2,272 MiB whole (67 B a cell) and the
  auto rule picks memory; process peak 5.3 GB in memory, 3.05 GB streamed (the checkpoint's world included).
  **Streamed = in memory = one thread: all 512 of 512 tiles of both layers have identical CRCs** (`dtr_cmp.py`
  over the tile indexes), sinks = 0, 15,684 cells raised by the first flood and 934 (≤ 27 m) by the last, mean
  |Δ| over land 15.5 m, 7.1e4 km³ eroded of which 3.6e4 km³ left in transit, mean detail amplitude 0.29 / 1.16 /
  2.32 blocks at drama 0.25 / 1 / 2. **134 M-cell L3** (`--set resolution.l2_r=4 --set resolution.l3_r=1` on the
  same checkpoint: the size of 32k full): young at 4096 × 2048 30.7 s, amplify 16384 × 8192 **58.5 s**, streamed
  (auto: 9,088 MiB whole, 1,221 MiB in use) in 32 tiles, middle level 8192 × 4096 whole (erode 23.9, thermal
  11.2, flood 10.7, deposit 8.1, seed 0.8; sample 1.0, detail 1.0, sink 0.9); peak footprint 4.5 GB with the
  checkpoint's world; L3 layers 164 MB + 45 MB. Gates on the tree merged with master da1827e (see the
  commit): `cargo fmt --check`, `clippy --workspace --all-targets -D warnings`, `deny check licenses bans`,
  `test --workspace --locked` (core 204 + 2 ignored, cli 23, format 33, inspect 28 + 2 + 7, jni 7, viewer 5).
- **What the pictures show** (review/out/p1-amplify; seven 512 × 512-cell views of 307 km at 16k quick (mountains, highlands, a coast, the biggest river's lower course, the deepest lake, a tile boundary, the seam),
  before = L2 interpolated to L3 with no erosion pass, after = the L3 layer as the game reads it, sliding
  comparison, D8 rivers of each terrain (≥ 80 km²), the height change, and the drama figure). Steep faces and
  scarps that were smooth ramps grow gully and ridge networks a few km apart, the network is denser (rivers
  6,288 → 9,403 cells in the mountain view), plateau tops, broad valley floors, plains and the sea floor do not
  change; the change picture is blue (cut) on steep faces and red (a little fill) at valley mouths. The seam
  view (a range and a coast crossing x = ±C/2) and the tile view (a tile boundary of the streamed run over a
  plateau and its gullied escarpment, red ticks on the edges mark the boundary) show no mark. **Doubtful:** the gullies still run mostly along the grid's axes and 45° (multiple
  flow on a square grid); D8 rivers on flat polar plains are drawn as straight lines by the viewer's drainage
  analysis (a viewer artefact, not terrain); the coastlines keep the L2 staircase of the young landscape; the
  drama slider changes the terrain only a little (thermal slumping reaches a few dozen cells) and the game-time
  detail in proportion (0.29 → 2.32 blocks of mean amplitude).
- **Deviations.** (1) Acceptance on `pipeline_quick_16k`, not `earthlike_*` (placeholders until P1-08).
  (2) The stages sit after the deposits (the plan's §8 lists them before structure); nothing reads L2/L3 yet and
  `stage deposits` stays fast. (3) Young adds erosion time without moving the clock (`t` stays 0); the plan's
  "last 2–10 Myr" is redone on L2, not cut from L1's timeline. (4) Halo derived from the iteration counts
  (196), not the plan's fixed 64; tile 2,048 as planned. (5) Working set 46–50 B a cell (67 with a tile's
  copies) against 20; the estimate accounts for it and streams when the default cap says so. (6) Drama maps
  to 0.25 + 0.75 · drama times the base slope (floor so that 0 is smooth, not flat). (7) Fine drainage area
  refines the coarse level's exact area instead of starting from zero (Schott needs hundreds of iterations
  for that). (8) The flood is a serial Priority-Flood per window, not a Jacobi pass; equality across tilings
  rests on every depression the amplifier makes lying within `repair_margin_cells` of its spill (tested, and
  identical on the real planet). (9) `water_level` (lakes) is not written: Phase 2.
- **Findings for Ben / P1-17.** (1) Amplification is cheap: 19 s on 12 threads for 16k quick, 59 s for 32k
  full's L3 size, with the erosion passes 40 % of it; P1-17 has room to raise the iterations if the pictures ask
  for more. (2) The plan's 20 B/cell working set was not reachable with the arrays these passes need; 32k full
  peaks near 4.5 GB with the world on the Mac, so the cap decides. (3) The surface stage keeps its scratch
  after its last step (known issues). (4) The calibration agent's changes to `surface`/`climate` parameters
  do not touch this; when it lands, tests that derive from a `pipeline_*` preset need the two `remove(...)`
  lines. (5) 7.1e4 km³ eroded, half left "in transit": deposition only settles material within its reach
  (12 cells), the rest leaves the amplifier (no mass claim beyond the pass).
- **Next.** P1-08 bakes: it reads `world.notes["amplify"]` (layers, grid, preview) and the tile packs already
  in the run folder (`rasters/elevation.dtr`, `rasters/detail/amplitude.dtr`) and adds `water_level`; the Java
  `deeptime:terrain` term (`worldgen/DetailNoise`, formula in FORMAT.md) comes with it.

### 2026-09-29 — P1-20: the workbench UI overhaul, dark (agent worktree agent-a95571be05fdd89d6, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Why.** Ben opened the workbench: "Sorry, that looks shit. … a complete UI overhaul to look WAYY better.
  (Also, should be dark themed.)" Then, from real use: he ran `earthlike_full_32k` for 4,600 Myr and got
  one green blob, because the placeholder presets sat next to the real ones.
- **What changed (front end; the HTTP/SSE contract, the POST guard and the stage-2 contract are as they
  were).** `crates/deeptime-viewer/assets/workbench.{html,js}` rewritten (still plain HTML/CSS/JS, embedded
  with `include_str!`; Inter and JetBrains Mono from Google Fonts with a system fallback; icons inline SVG).
  One dark palette as CSS custom properties (elevation layers, one indigo accent, live/queued/done/failed
  colours), the map on near-black so ramps.json's colours read as in the game (ramps untouched).
  Layout: top bar (run title and chips, the Mac's slot with progress and time left, connection); left
  sidebar, collapsible (`E`; a drawer below 1180 px): run cards (status, progress bar, simulated share and
  time left, preset/start/length/seed chips, Open / Compare / Settings / Cancel with an inline confirm),
  grouped "On the Mac" and "Finished"; the editor: preset cards (friendly names such as "Earth-like ·
  quick", the raw family as secondary text, the stages as chips), size as a segmented control, start as
  three picture cards, length as a log slider with quick picks, the memory cap, every catalogue section
  with its description and every field with its hint (a section this preset lacks folds away and says
  why), share code copy/paste-to-import, and a sticky footer with the estimate as two tiles (peak memory
  with a bar against the cap; time on the Mac) above the reasons Run is off (click one to jump to its
  field) and warnings for near-cap and long runs; Run says "Queue run" when the Mac is busy. The stage:
  the map as the hero with a graticule, lat/lon axes and the seam marked, a crosshair and a readout
  (lat, lon, block x/z, the layer's value read back from the colour), event markers with glowing fresh
  ones, tools Select / Inspect / Area 3D with a hint pill, Events and Grid toggles, and compare as Single /
  Split (one map, a draggable divider; arrow keys on it) / Side by side (stacked when that gives larger
  maps) with a run-B picker and swap. Right panel: the globe (lit sphere, atmosphere rim, graticule, the
  seam dashed, lat/lon on hover, spin), the layer picker (grouped popover with swatches; ↑/↓ and 1–9) and
  its legend (ticks, units, Köppen and other labelled classes by name), the inspector (Events with a
  kind filter; Details: an event's fields, its causes and effects as clickable chips, "go to this step",
  or a clicked point's values with "inspect this column" / "preview 256 × 256 here"; 3D: stage-2 job
  cards; Run: the summary lines as stat tiles, share code, load settings). The dock: transport, a large
  time readout, speed −/+ (`[` `]`), Live / Jump to end, and a timeline over the run's whole planned span
  (computed part, live edge, playhead, Myr-ago ticks, an event histogram by kind); drag or click to seek.
  Tooltips with shortcuts, toasts, a shortcuts sheet (`?`), focus rings, `prefers-reduced-motion`
  honoured. `?tv`: no chrome; title, the time in 3.6 rem type, a LIVE badge with the share simulated and
  time left, the newest events, the legend and a globe float over a full-screen map; follows new runs and
  replays the last one after 25 s idle. Phones (390 px): one column, map, dock, then a tab bar (Layers,
  Inspect, Globe, Runs, New run); no horizontal scroll. Empty, loading and not-answering states drawn.
- **Placeholders (the coordinator's addition).** Placeholder presets are hidden behind "Show placeholder
  presets (dev)"; the real `pipeline_*` / `plates_*` families come first under friendly names. A
  placeholder run carries a "placeholder" chip on its card and in the top bar, a map badge
  "PLACEHOLDER · NOT THE REAL SIMULATION" and a warning in its Run tab.
- **Stage 2 in the page.** Inspect and Area 3D make job cards: steps (queued → waiting for the lock →
  booting → generating → exporting), elapsed "usually 1–2 min", the runner's message, the note's lead
  ("A test world at this latitude and longitude, until P1-08.") with More; when done the 3D view runs in
  the card (`area3d/?…&embed=1`) and opens full size over the page. A failure shows the error and Try
  again; a 501 says preview is off. `tools/review/pages/area3d/` takes the same look (dark by default,
  light one click away, `?embed=1` shows the scene alone). A direct, non-job answer (P1-08's column
  route) is shown as it is.
- **Server and catalogue (minimal).** `GET /api/catalog` also sends the catalogue's new `sections`
  (a description per section); every field gained a one-line `hint` (`editor_fields.json`; Java's
  `EditorFields` ignores both, JUnit 118/0). `GET /api/runs` rows gained `seed`, `elapsed_ms`, `eta_ms`.
  Progress in the page is the share of the history simulated (`t_years` over `length_myr`) with the
  time left at the pace so far: the core's `total_permille` read 3 % at 20 % of a run.
- **Measured.** Private `deeptime watch --preview` on the Mac, port 8653 (`deeptime-cache/workbench-ui/`,
  standalone profile, runs copied from the live folder plus new ones), reached over an ssh tunnel; shots
  by a batch DevTools driver (headless Chromium, SwiftShader WebGL). End to end through the new UI: a run
  started from the editor (random, 200 Myr: 101 frames, 491 events, 2:24), a running run cancelled from
  its card (kept 210 frames), compare split and side by side, an inspect and an area preview (standalone,
  77 s job, the 3D view in the card and the modal). Playback at 20 Myr/s on a 251-frame run, same
  machine and data: new page 57.6 fps with 1 long task in 8 s, old page 43.8 fps with 37.
  Screenshots: `review/out/workbench-ui/{before,after}/` and `index.html` (9 before/after pairs + 20 new
  states: 1600 × 900, 1920 × 1080, 1280, 1024, 390 px, `?tv` at 1080p and 4K).
- **Gates** (Mac, after merging master c79b38f): cargo fmt --check, clippy --workspace --all-targets
  -D warnings, test --workspace --locked (cli 23, core 173 + 2 ignored, format 32, inspect 28 + 2 + 7,
  jni 7, viewer 5), deny licenses bans clean; `cd neoforge && ./gradlew build test` → tests=118
  failures=0 (the catalogue change).
- **Found, not fixed here.** Every preview on the live 8642 server failed ("the preset …
  deposits_world_16k.json is not valid: $.deposits_synthetic: unknown key"): the main checkout's
  `smoke/mic` on the Mac holds a Deep Time jar from 2026-09-28 19:33, older than P1-15's synthetic deposit
  preset, and `preview.py` installs a profile only when `install.json` is missing. Fix: re-run
  `tools/smoke/install.sh mic` for that checkout (mac.sh --pack); later, have `preview.py` reinstall when
  the checkout's jar is newer.
- **Next.** The parent merges, restarts 8642 with `mac-watch.sh` and puts `?tv` on the TV. Open for Ben:
  the accent colour, whether the TV should replay when idle, whether the placeholder toggle should go.

===
### 2026-09-29 — P1-17 partial: the surface stage's serial solves run in parallel (agent worktree agent-a9bc0ab99e68b4a07, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **What.** `surface/routing/forest.rs` (new): the drainage forest in stack order (cells by position, receivers,
  donors as CSR in ascending cell index, subtree sizes, tops, leaves, river mouths) and two schedulers,
  `Forest::down` (each cell after its receiver: a work-stealing walk; a worker keeps a stack of ready subtrees,
  runs the heaviest child next and gives the oldest, largest entries away once it holds two grains, 256 cells)
  and `Forest::up` (each cell after all its donors: leaves in chunks, and the last donor to finish, an
  acquire-release countdown, carries on to the receiver). `spl::solve`, `spl::families` and
  `routing::accumulate` are rewritten on them; `routing::stack` builds the forest. No `unsafe`: values that
  tasks write and others read live in `SharedF64` (relaxed atomics over `f64` bits; the schedulers supply the
  ordering). `pflood.rs` is unchanged except its doc comment. New tests in `routing/parity.rs`; nothing else
  moved (`surface/mod.rs` untouched).
- **Bit-identical, by construction and by test.** A cell reads only finished values and writes only itself. Every
  sum over donors is a *gather* by the receiver, over the donors in ascending cell index, starting from the
  receiver's own term: exactly the order in which the serial reverse-stack loop's `acc[rc] += x[c]` adds them
  (the stack lists a cell's donors highest index first, so the reverse walk meets the lowest first), so the
  floating-point sums are the serial ones, not close to them. The loop-invariant `F = K A^m Δt / L^n` (two `powf`
  per cell) is computed once per solve instead of once per iteration: the same expression on the same inputs.
  **Evidence:** (1) `parity.rs` keeps P1-04's serial `accumulate`/`solve`/`families` verbatim and compares bit
  patterns of area, elevation, inflow, iteration count, residual and the source mix on random wrapping grids
  (160×96, ties from quantised heights, a grid that does not wrap, land with no sea so the flood makes sinks,
  3- and 5-wide bands where x = 0 and x = w−1 are neighbours and donors sit on both sides of the seam) with
  n = 1/G = 0, n = 2/G = 1 with a critical slope, and n = 1.5, each on pools of 1, 2, 4 and 8 threads; reversing
  the donor order in `Forest::build` makes three of the five tests fail, so they see summation order. (2) The
  500 Myr `pipeline_quick_16k` run (`run --checkpoints`, scattered_continents) before and after, on master
  6d8171f with P1-07's structure stage in the group: the whole 596 MB checkpoint (plates + surface + structure,
  incl. the 220 MiB history) of the old code at 4 threads and of the new code at 12 threads is byte-identical
  except the length/crc bytes at offsets 13–56 and the `"millis"` timing text inside the stage summaries;
  every world file (keyframes, events, sites, bodies) is identical (`cmp`); the same holds on the pre-merge
  master (old code 6 threads vs new 6 threads, checkpoint included) and on the final merge with master
  43f087d (P1-05 climate, P1-07 structure, P1-14 deposits in the pipeline): old code at 4 threads against new
  code at 12 threads and at 1 thread, every world file (12 keyframes, events, sites, bodies) byte-identical and
  the `surface_final` line the same. (3) P1-04's `the_coupled_pipeline_runs_and_resumes` and the
  rest of the suite are green (core 173 + 2 ignored, cli, format).
- **Measured (Mac, pipeline_quick_16k, scattered_continents, 500 Myr, `bench --threads 12,4,1`, old and new
  binaries interleaved, 3 rounds; the Mac's load average was 5–40 from other agents, so min..max is shown).**

  ```
  threads code   total s        surface s      spl s         diffusion s    surface−diffusion s
  12      old    101.9..170.1   74.5..130.7    34.0..39.3    17.8..58.2     56.7..72.5
  12      new     92.3..119.9   60.7..82.8      6.5..8.5     27.3..43.5     33.4..39.3
  4       old    131.3..155.4   89.5..106.6    35.9..39.7    26.4..34.9     63.1..71.7
  4       new    109.8..132.7   66.4..82.2     10.6..12.0     27.3..36.2     39.1..46.0
  1       old    259.0..319.3   146.9..182.0   36.6..42.9     66.6..83.5     80.3..99.0
  1       new    251.9..316.8   142.1..180.2   35.2..38.8     64.3..85.2     77.8..95.0
  ```
  (P1-04's own bench, a quieter machine and before P1-07: surface 65.9 / 79.0 / 137.0 s.) The stream-power
  solve is 5.2× faster at 12 threads by the minima (34.0 → 6.5 s), 3.4× at 4, and 1.04× at one thread (the
  hoisted `powf`s); what the solve costs is now 6.5 s of 60.7. **The surface total moves much less because the
  diffusion, which the change does not touch, is the largest piece and swings 17–58 s with the Mac's load**
  (the same binary, back to back); taking it out, the surface stage went 56.7 → 33.4 s at 12 threads
  (scaling 1.4× → 2.3× against one thread). Whole-run minima at 12 threads: 101.9 → 92.3 s total (plates +
  surface + structure; the plates stage moved 23–34 s between runs of the *same* binary). The min-run stage line
  of the new binary: `plates=27.0s surface=60.7s structure=4.6s surface[advect=3.4s,carbonate=0.6s,
  diffusion=27.3s,fields_events=8.0s,isostasy=1.9s,marine_fill=0.5s,outputs=1.5s,pflood=3.9s,routing=1.6s,
  setup=3.9s,spl=6.5s,strata=1.2s]`. `routing` rose 1.0–1.3 → 1.6–1.8 s (building the forest, ~2 ms per step).
  **Per phase, one LEM step on an L1-sized fractal terrain, best of ten** (`cargo test --release -p
  deeptime-core surface_timing -- --ignored --nocapture`; the Mac's load average was 20 while it ran, and a
  perfectly parallel calibration loop got 5× at 8 and 12 threads there): P1-04's serial loops accumulate 0.33 +
  solve 81.2 + families 1.2 = 82.7 ms; the parallel passes at 1 / 2 / 4 / 8 / 12 threads: solve 78.7 / 42.5 /
  25.8 / 15.5 / 17.8 ms (5.1× at 8, the calibration loop's own scaling), families 1.6 / 1.8 / 1.3 / 1.5 / 1.6,
  accumulate ~0.7 (too small to gain), receivers 2.3 → 0.5; serial: open_sea 4.3, flood 6.4, stack 3.8–4.0.
  **32k quick** (`pipeline_quick_32k`, 500 Myr, 12 threads, one run each, Mac load 18–24 the whole time; its L1
  is the same 1024×512 grid at dx 9.6 km, so the cost is the same): old surface 160.4 s (spl 38.0, diffusion
  67.3) → new 142.0 s (spl 13.6, diffusion 70.1); total 221.4 → 207.2 s; identical `surface_final` line.
- **What is still serial, and why.** `pflood::open_sea` (4–5 ms per step), `pflood::flood` (6–10 ms) and
  `routing::stack` (4 ms; the DFS that fixes the stack order and builds the forest): about 15 ms of the ~34
  ms that routing and the solve cost per step at 12 threads on an L1 raster (98 ms with P1-04's serial
  loops), 3.7 s of the 500 Myr. The flood is not tiled *on purpose*: its result depends
  on its own processing order where a flat is fed from two sides at exactly the same elevation (the FIFO pit
  queue gives the cell `next_up` of whichever neighbour it popped first), and the order-independent flood (the
  least fixed point of the ε relaxation: what a tiled flood merged through a spill graph, or Dijkstra with exact
  ties, computes) equals it on generic heights (a test) but not on flats. Lakes are flats by construction
  (sediment fills them to their level, leaving the cells within ulps of one another). Measured on this very
  run (a throw-away hook computing both floods at every step, not committed): **the least fixed point differs
  from the serial flood in 168 of 250 steps, in ~170 (43,038 over the run) of ~13,000 raised cells per step**,
  and ~12,700 land cells per step are lakes. So a tiled flood changes worlds; it is not the "sound method"
  this task asked for. A radix heap and a parallel seed scan made the serial flood no faster (6–8 ms on a
  fractal L1 terrain), and a 4 ms `stack` is not worth a parallel counting sort.
- **Findings for P1-17 proper.** (1) P1-04's "33 of 66 s serial" was mostly the stream-power solve
  (`spl` 34–44 s at every thread count before; `pflood` 3.6–5 s, `routing` 1–1.6 s); the flood was never the
  bottleneck. (2) **Diffusion (Jacobi-PCG, `diffusion.rs`, ~111 CG iterations per step) is now the largest
  piece of the surface stage and scales poorly**: at 12 threads 17.8–58.2 s over the six old and new runs,
  against 64–85 s at one thread (3.6× at best, hardly any when the Mac is busy), and in one determinism
  run on the merged pipeline under a load average above 20 it took 268 s at 12 threads against 66 s at
  one. Each iteration streams ~20 vectors of 4 MB through 7 parallel regions, so it is memory- and
  barrier-bound, and it is what makes the end-to-end 12-thread number swing with the machine's load. Next
  speed-ups, in order: fuse the CG vector loops into two passes per iteration (same per-element arithmetic,
  same row-ordered sums, so still thread-count independent), then a better preconditioner (a coarse-grid
  correction: 111 iterations become ~10). (3) Nothing here touches the plan's determinism rule; the per-phase
  table above comes from `cargo test --release -p deeptime-core surface_timing -- --ignored --nocapture` (best
  of ten per phase on an L1-sized fractal terrain, the P1-04 serial loops beside the parallel passes).
- **Next step.** P1-17 proper (bench gate, presets) starts from these numbers; the CG speed-up above is the
  next thing to buy. Gates on this branch (merged with master 43f087d): `cargo test --workspace`, clippy
  `-D warnings`, `cargo fmt --check` and `cargo deny check licenses bans` all clean.

### 2026-09-29 — P1-14: the deposit engine on the real recorded history (agent worktree agent-a1861e86fd3f0f213, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **What.** `DepositStage::recorded(&PlanetPreset)` reads `WorldState::history` (P1-04's frames plus P1-07's `trap_structure`
  and `host/<class>`), the synthetic path is unchanged (`Source::{Synthetic, Recorded}`). A planet preset with a `deposits`
  section (`DepositParams`: count_mult, province, province_km, types, essentials; validated, needs `surface`) adds the stage
  after the coupled group; the four `presets/pipeline/pipeline_*.json` got `"deposits": {count_mult 1, strong, 800 km, all
  types, no essentials}` inserted between `resolution` and `tectonics` (a different place from where P1-05 appends, so the
  merge is textual-conflict free). The stage takes the history out of the world for the placement step and puts it back
  (checkpoints carry it); frame count and frame times come from the history, not the preset (`state.frames`, restored on
  resume); scales, grid and seed from the preset. `with_content` now also sets a planet preset's essentials. Field names are
  read from the history's own `names()`, nothing is listed in the stage.
- **Missing fields degrade deterministically.** A type whose rules read a name the history lacks is skipped: no accumulator,
  no sites, a WARN per type (`deposit type X skipped: the history lacks [...]`), an INFO summary (`8 of 8 types run on 50
  frames (55 fields, 3 clocks)`), and the census (`skipped: [{type, missing}]`, printed by the CLI as `deposits SKIPPED`).
  Absent `host/*` fields are a documented fallback, not a skip (host score 1, "no lithology opinion"): logged and in
  `fallbacks`. With P1-07 merged **no type is skipped and no fallback fires** on the real history; the tests exercise both
  by dropping `trap_structure`/`carbonate_platform`/the host fields from a recorded synthetic history (the types that kept
  their fields give bit-identical favourability). Types that are *enabled but empty* are visible instead through the census'
  `best_cell` (where the most favourable cell is, its six component scores, the limiting component; the CLI prints it) and
  `tracts`/`expected`.
- **Found while looking at the first real map: the accumulators did not move with the plates.** The engine folds favourability
  per L1 cell over frames; on the real history a cell's crust changes every 10 Myr (100+ cells), so an arc's history smeared
  along its track and sites landed on old arc positions in the ocean (2 of 5 porphyries and 1 of 11 coal on land, arcs as
  random streaks). Fix, additive to P1-04: the surface stage composes its per-step nearest-cell advection maps into a
  per-frame *origin map* (`FrameAcc::compose_back`, crust born in the step marked new) and records it in every frame as two
  exact-integer fields `deeptime:origin_di/dj` (cell offsets, NaN = new crust; 16-bit packing is exact for them, so
  `RecordedHistory` is unchanged). The deposit engine carries every type's `CellAcc` and the petroleum `CellPet` with the crust
  into each frame (`carry`, a parallel gather) before folding the frame in, so favourability is per crust parcel; pass 2 traces
  each site's cell back through the frames (`trace`) so the ledger's erosion/burial series and the supergene window are the
  column's own, and every forming/exhumation/placer event is looked up at where the deposit *was* at that time (`xz_at`), not
  where its crust is now. Placers stay on the present drainage. A rigid-drift test proves the carry exact. **Checkpoint schema
  4 → 5** (surface `FrameAcc` and the deposit state changed shape); the history grows by two mostly-constant fields per frame.
- **Also fixed.** (1) Petroleum kitchens: the real history has 3,348 connected regions of mature source rock (p50 one cell,
  22 km²; 3,271 of them under 500 km²), which made 3,348 `petroleum_generation` events that buried the viewer and the ticker.
  `PetroleumRules.min_kitchen_km2` (schema + type JSON: 500, about 22 L1 cells; default 0 for datapacks) keeps 77. (2) The
  keyframe dedupe in the session was per instant; a recorded run reaches the deposit stage at t = 0 where the group has just
  drawn its closing keyframe, so the deposit stage's own was skipped. It is now per group (`last_kf_t: (t, group lead)`),
  three lines in `session.rs`. (3) The deposit bundle leaves the strata inspector to the structure bundle (its layers were
  ~20 MB) and drops six bulk surface/structure layers from its keyframes (`surface/{erosion,deposition,sediment,regolith,
  class}`, `structure/log_grade`), so a 500 Myr bundle is 23 MB before oxipng and 17 MB after.
- **Tests that pinned the pipeline.** `surface/tests.rs::pipeline_json`, `structure/tests.rs::pipeline_json`, the CLI's
  surface test and the workbench's `small_pipeline` now remove the `deposits` section (they are about the group); new tests
  cover the stage. `deposits::recorded_tests`: `coupled_pipeline_reads_its_own_history_and_is_deterministic_across_threads`
  (3,000 samples, 100 Myr, 1 vs 3 threads: sites, bodies, events, census and the sites layer bit-identical; no skipped type, no
  fallback; petroleum sites with kitchens in the log; every site's causes are real plates/surface events),
  `a_checkpoint_inside_the_deposit_stage_resumes_to_the_same_sites`, `deposits_ride_a_rigidly_drifting_crust`,
  `missing_fields_skip_the_types_that_need_them_and_nothing_else`, `the_recorded_stage_places_every_family` (recorded frames
  made from the synthetic history: all eight types have sites), `a_world_without_history_is_a_clear_error`; CLI
  `deposit_stage_reads_the_recorded_history` (`stage deposits --preset` and `--in` after `stage structure --out`).
  **A 100 Myr run on 3,000 samples is too young for most families** (only petroleum gets tracts, coal one that erodes), so the
  "every family" assertion is made on the recorded synthetic history and on the 500 Myr acceptance run below, not on the
  small coupled run; longer coupled tests would cost minutes in debug.
- **Measured (Mac, `tools/remote/mac.sh`, M6, 12 threads, shared).** `stage deposits --preset pipeline_quick_16k --start
  scattered_continents --length-myr 500 --viewer review/out/p1-deposits-real --out …` → 154 s wall on a quiet Mac (plates
  30.0 s, surface 111.6, structure 5.0, deposits 4.4 for 52 steps; 224 s when the Mac was busy); `stage deposits --in` a saved
  group checkpoint: 4.1 s wall. 55 fields + 3 clocks in the history. The deposit stage adds the accumulators (8 types × 24 B ×
  524k cells ≈ 100 MB) to the world; history 327 MiB at 500 Myr 16k quick (312.8 MiB before the origin fields).
- **What the 500 Myr map showed (count_mult 1 and, for the placement check, 10; both with strong provinces).**

  | type | ρ·A·F expected | drawn | sites | tracts (km²) | best cell |
  |---|---|---|---|---|---|
  | porphyry_cu_mo_au | 1.88 | 2 | 1 (+0 children) | 641 (37,641) | F 0.91, limited by driver (convergence/obliquity/slab age) |
  | coal_seam | 15.24 | 17 | 4 (9 eroded, 4 too deep) | 1,298 (105,784) | F 1.00 |
  | mvt_zn_pb | 0.00 | 0 | 0 | 2 (45) | F 0.21, limited by host and source (evaporite) |
  | sedex_zn_pb | 0.00 | 0 | 0 | 4 (112) | F 0.80, limited by pathway; 112 km² |
  | petroleum_trap | 0.11 | 0 | 0 | 84 (2,797) | 77 kitchens ≥ 500 km² |
  | bif_fe, placer_au, supergene | 0 | 0 | 0 | 0 | bif: `bif_acme` clock 0 and `inherited_bif` 0 (P1-05/P1-07 stand-ins) |

  At count_mult 10: 42 sites (porphyry 3 with 9 placers, coal 30, petroleum 0 after its one draw was too deep).
  Where they sit: the favourability maps now follow the margins the arcs and mires were on (the arc-flux input shows thin
  lines along every active margin; the porphyry map shows the same lines where the crust ended up); porphyries are on
  continental-margin arcs with placers within a few cells of them downstream (4 of 9 on land, the rest at the river mouths
  the walk ends in); coal is on shelves and coastal plains and in the drowned mires of the polar shelf (19 of 30 above sea
  level); nothing lands in the deep ocean. Site chains checked in the viewer: porphyry #4 (46.2°N, 235.3°E, andesite host,
  buried, weakest component driver, barren nearby for lack of source) ← subduction_onset 1:98, orogeny_phase 1:290,
  terrane_accretion 1:82 ← subduction_onset 1:65, collision 1:81 ← subduction_onset 1:16 — real plates events, not synthetic.
  **The counts are low and MVT/SEDEX/oil are empty because the surface's facies are scarce**, not because of the engine:
  at the last frame the carbonate platform covers 0.5 % of cells, evaporite 0.8 %, euxinic basin 0.4 %, coal facies 0.1 %, and
  the mean temperature is −1.8 °C (P1-05's climate replaces this stand-in; carbonate needs water ≥ 18 °C). Porphyry is 1.9
  against the plan's rough "about 11 at 16k" (1 % arcs at F 1): the arcs cover 0.5 % of the planet at mean F 0.33.
- **Deviations.** (1) The acceptance run is `pipeline_quick_16k` (P1-13's `earthlike_*` stay placeholders until P1-08).
  (2) Scales come from the preset but the surface elevation → block y still uses sea level y = 63. (3) The deposit stage runs
  at t = 0 after the group, so the sites and census come out with a second keyframe at t = 0 labelled `deposits`.
  (4) The essentials list is empty in the pipeline presets (the game passes `deeptime:essentials` in the content bundle;
  `with_content` applies it), so this run has no `guaranteed_occurrence` sites.
- **Not done / next.** P1-05's climate (warm shallow seas → carbonate, evaporite, coal) is what fills MVT/SEDEX/coal; then
  count_mult and the facies rules are tuning (P1-17), and BIF needs P1-07's basement ages to reach `inherited_bif` for a
  timeline that starts after the acme. P1-15/P1-16 rerun on a real deposit world after P1-08. Decisions for Ben: none
  blocking; the count question (`count_mult` default 1 gives 1 porphyry on a 16k planet) waits for P1-05.

### 2026-09-29 — P1-20 stage 2: 3D area preview and click-to-inspect from the real generator (agent worktree agent-a63e3a8af506069de, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Approach.** The Rust column evaluator is a stub until P1-08, so stage 2 runs the real mod headless
  on the Mac and exports what it placed: Deep Time's density, strata pass and deposits and the
  pack's surface rules and ores, as a player's world gets them.
- **`deeptime-inspect export-area`** (`crates/deeptime-inspect/src/area.rs`, format in its docs):
  a box of at most 256 × 256 (`--max-side`), y −352..639 or `--y`, written as `.dtva(.gz)`: a JSON
  header, then u16 run counts per column, u16 run values and lengths (bottom-up), and the strata stack
  per column (K × i16 tops, K × u8 classes, read through P1-07's `LoadedLayer::stack_at`, the same
  column the Java strata pass builds). A run value is a (block, tag) entry; a tag lists the deposit
  bodies and zones whose SDF holds the block centre. x0 is folded into [−C/2, C/2) and the box runs
  east, so a box may straddle the seam (column i = world x fold(x0 + i), unwrapped x0 + i). The header
  carries blocks (colour from `colors`, category, ore material, the rock classes a block belongs to),
  the bodies and sites in the box with their "why here" chains (from `/deeptime why` when the runner
  recorded it, else the world data's event log), the palette and claims probes, and stats: categories,
  ore in bodies / claimed scatter / unclaimed, how much rock is its horizon's (or body's) class, and
  whether the `probe site` answers match the export's blocks and body tags. `--verify` decodes the file
  and compares it with the region files block by block. Tests: 7 unit (round trip, RLE runs maximal
  and summing to the height, runs past the stored sections, a box across the seam at C = 128 given in
  three frames, a body across the seam, probes parsing, categories) + 2 on the real fixture region
  (80 × 64 × 384 blocks equal the world's; the same columns one lap away; proto and missing chunks
  reported).
- **Job runner** `tools/workbench/preview.py` (stdlib; `/usr/bin/python3` 3.9 on the Mac runs it):
  installs the profile's server set if missing, prepares the job's own world
  (`smoke/workbench/jobs/<job>/world`, booted with `--universe <job> --world world`, so smoke worlds
  are untouched; synthetic data, a world-data folder, a first-start preset or a test terrain), takes
  the Mac's server lock (the one `mac.sh --server` holds; waits, never breaks it), Chunky per box (a
  seam box in its two canonical pieces), the probes, stop, `export-area --verify`, deletes the world.
  From the laptop: `tools/workbench/preview-area.sh`. Contract in `tools/workbench/README.md`.
- **Viewer** `tools/review/pages/area3d/` (three.js 0.169 from jsDelivr): greedy-meshed 32³ sections,
  a section re-meshed only when a cut, toggle or colour mode changes what it shows; top cut, x/z
  cross-section, category toggles and presets (everything / hide stone / ores and deposits with bodies
  see-through), colour by block, rock horizon or body, vivid ores (ore textures average to their stone,
  so ores are painted by material by default); the seam drawn in red; click a block for id, folded and
  unwrapped position, chunk, horizon and whether the strata pass painted it, the body and zones, the
  site with products and why-here chain, and the column's strata stack; "cut here" makes a notch through
  a site's ore body. Light and dark, 390 px. URL state (`site`, `pick`, `pickcol`, `cuty`, `sec`, …).
- **Wired into the workbench** (stage 1 at 9e34b45): `deeptime watch --preview <preview.py>` forwards
  `POST /api/preview`, `GET /api/preview/…`, `GET /api/inspect`, `GET /area3d/…` to `preview.py http`
  (new `crates/deeptime-cli/src/watch/stage2.rs`; `http.rs` asks it first, 501 without `--preview`);
  `mac-watch.sh` passes the checkout's preview.py (`--no-preview` to leave it off); `workbench.js`
  `Stage2` passes lat/lon to inspect and shows the answer's `note`. **What a preview is:** runs are
  rasters, not worlds, so until P1-08 a preview is the synthetic deposit world (deposits_world_16k,
  500 Myr, C = 16,384) at the box's latitude and longitude; every answer, the panel and the viewer say
  so. Inspect answers from a finished preview holding the column (the viewer opens on the column's top
  block), else starts a 32 × 32 preview. The P1-08 plug-in points are in the README ("Later").
- **Measured (Mac)**, `tools/workbench/preview-area.sh`, all `verify` 0 wrong:
  | Box | Profile, world | Boot | First start | Chunky | Total | File |
  |---|---|---|---|---|---|---|
  | 256 × 256 × 992 at −5698,−1158 | standalone, deposits_world_16k | 12.7 s | 3.3 s | 361 chunks 9.1 s | 43.8 s | 0.94 MB, 31.3 M blocks, 1.26 M runs |
  | same, after merging P1-07 | standalone | 17.0 s | 4.5 s | 361 chunks 11.1 s | 63.1 s (with the jar rebuild) | 1.02 MB |
  | same | MIC, 168 mods, pack 303fd89 | 50.3 s | 5.8 s | 361 chunks 25.3 s | 89.9 s | 1.09 MB, 173 block types |
  | 128 × 128 across the seam (8128..8256) | standalone, wrap-test every-class | 21.4 s | — | 2 × 55 chunks 9.4 s | 54.5 s | 0.19 MB |
  | 63 × 27 through the workbench | standalone, deposits_world_16k | — | — | 35 chunks | 27.2 s | 34 kB |
  Export itself 0.5 s for the largest box. 6 of 6 `probe site` answers agree with the export's blocks
  and body tags (standalone after the merge, and MIC). Strata: 99.2 % (standalone) and 98.5 % (MIC) of
  rock below the first horizon top is its horizon's or body's class; crust 2,091 / 134,146 blocks
  (MIC: stone that features put back). Seam box: terrain top continuous (largest step between the two
  seam columns 4, mean 0.62; a neighbouring pair: 4, 0.10; the seam's carvers, as P0-15 saw).
  `tools/workbench/check-watch.sh` (a private `deeptime watch --preview` on port 8653): POST with the
  page's body → 202 + note → polled → done in 115 s (90 s of it waiting for another session's server
  lock) → export 34 kB, `/area3d/` 200, inspect of a column inside a finished box → the viewer URL with
  `pickcol`. Screenshots (headless Chromium, SwiftShader; about 15 s to load and mesh 256 × 256 × 992):
  `review/out/area3d-shots/` (overview, notch cutaway through the supergene blanket with a copper ore
  selected, ores-and-deposits, seam, MIC overview and cutaway, 390 px dark, the workbench-made preview).
  Review bundle (not published): `review/out/area3d/` (viewer + 4 exports, 2.3 MB, 7 files).
- **Gates** after merging master 6d8171f: `cargo fmt --check`, `clippy --workspace --all-targets -D
  warnings`, `test --workspace --locked` (cli 22 incl. 2 stage2, core 149, format 32, inspect 28 + 2 +
  7, jni 7, viewer 5), `deny check licenses bans` clean. `fmt` also needed master's b150ea0 watch test
  reformatted (done here). No Java changed, so no JUnit/GameTest rerun.
- **Findings.** In the MIC box the unclaimed ores are many (uranium 5,420, nickel 3,682, fluorite
  2,262, platinum 1,840 blocks in 256 × 256): kept by design, since claiming covers Deep Time's
  materials only. The same blanket holds 337 ore blocks in the standalone world and 367 in MIC (MIC's
  Create stones are hosts where vanilla's stand-ins are not).
- **Next.** Publish `review/out/area3d/` (parent session). After P1-08: point `workbench_preview` at
  the run's bake and answer `/api/inspect` from `deeptime inspect-column` (README "Later").

### 2026-09-29 — macOS free memory in MachineInfo; the Q10 client launcher folded into mac-client's launch path (agent worktree agent-aab6da56089b714ed, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **1. macOS free memory (the twin of P1-11's Linux `MemFree` fix).** The JDK is not right on the Mac:
  `OperatingSystemMXBean.getFreeMemorySize()` was 1,068,990,464 B (0.996 GiB) on the 16 GB M6 with 6+ GiB
  available. It is the kernel's free-page count alone (65,246 pages of 16 KiB at that moment, matching `vm_stat`'s
  "Pages free"; not a 4 KiB page-size slip), so `MachineInfo` now reads `vm_stat`. New `MacMemory` (package-private,
  `screen.model`): parses **free + speculative + inactive + purgeable** pages × the header's page size (vm_stat's
  "Pages free" already excludes the speculative pages, so nothing is counted twice; a purgeable page that also sits
  in the inactive queue is counted twice, tens of MiB). `/usr/bin/vm_stat` costs about 12 ms of process spawn from
  Java, so nothing waits for it: `availableBytes()` returns the last reading and starts the next one on a daemon
  thread when it is older than 5 s; `MachineInfo.prime()` starts one at client setup (`ClientSetup`, off the render
  thread: loads the classes too) and whenever a Create World screen inits (`CreateWorldGuard`, rate-limited by the
  TTL). `current()` on macOS uses the cache; with no reading yet, or no working `vm_stat`, the figure is **unknown
  (−1, "?" and no low-memory warning), never the bean's wrong one**, and is capped at the total. The figure is a
  snapshot of the moment the editor model is created, as on Linux (a follow-up would be re-reading it per frame).
- **Compared on the Mac** (`ssh`, the same `vm_stat` for every figure, awk; three samples 2 s apart while other agents
  were building, then one on the idle machine after the client run). JDK bean's free-only figure 0.15–0.29 GiB
  (idle: 3.67); **this formula 5.58–5.84 GiB (idle: 8.10)**; Activity Monitor's arithmetic, physical memory minus
  (App Memory + Wired + Compressed) 3.40–3.64 GiB (idle: 7.55); `memory_pressure` "free percentage" 70–73 %
  = 11.2–11.7 GiB (idle 76 %), which counts active pages as available (a pressure metric, not headroom). So the
  figure is between Activity Monitor's and `memory_pressure`'s and within 0.55 GiB (7 %) of Activity Monitor's on an
  idle machine; under load Activity Monitor is 2 GiB lower because it counts idle anonymous (inactive) memory as
  used, while macOS compresses or drops it when a new process needs the room. It is the sum graphide's `m6lock.sh
  avail_gb` uses on this Mac (plus purgeable) and psutil/Zabbix's `free + inactive`. Chosen on that basis and Ben's
  brief; if it proves too generous under load, the conservative alternative is free + speculative + file-backed +
  purgeable (2.85–3.09 GiB loaded, 7.19 idle), also from vm_stat.
- **Tests.** `MacMemoryTest` (11): the parser on the real M6 capture (expected 8,443,494,400 B = 7.86 GiB; the tag,
  File-backed and Anonymous lines are not mistaken for the four that count), an Intel-layout 4 KiB capture (written
  from the format), optional lines, CRLF, missing page size / free / inactive → empty, a counter too large for a long;
  the cache with an injected source, queue executor and clock (the first call returns −1 without running the
  source on the caller's thread; one reading in flight at a time; a stale figure is served while the next runs; a
  failing or garbled vm_stat keeps the last good figure and is not retried inside the TTL; a rejecting executor
  does not wedge it); and, `@EnabledOnOs(MAC)`, the real thing: `prime()` took 7.7 ms on the calling thread in a cold
  JUnit JVM (class init; the reading itself is on the daemon thread), `MachineInfo.freeBytes` = 6.10 GiB of 16.00 vs the bean's
  1.02 GiB a moment before. **Real client** (`tools/client/mac-screenshots.sh --name macmem --no-guidance
  --no-create --timeout 240`, exit 0, 32 s): the setup page reads "This machine: **6.6 GiB free of 16.0 GiB**, game
  heap 3.0 GiB, 11 simulation threads" (`review/out/macmem/03-setup-ready.png`, gitignored); it read 1.0 GiB before.
- **2. `ev-assets.sh` on `mac-client.sh`'s launch path.** The lease, the desktop-session checks, the pid-marker kill,
  the `launchctl asuser` launch and the pass loop (timeout, lease renewal every 60 s, notes) moved out of
  `mac-client.sh` into **`tools/client/mac-client-lib.sh`** (Mac side, sourced), which `mac-client.sh` and
  `ev-assets-mac.sh` both use; the laptop side (host and layout, graphide's `m6lock.sh`, copy to the Mac, the
  Ctrl-C handler) is **`tools/client/mac-client-laptop-lib.sh`**, sourced by `mac-screenshots.sh` and `ev-assets.sh`.
  `mac-client.sh`'s command line and output are unchanged (the two `-Ddeeptime.dev.screenshots=true` moved to its
  passes; the library knows nothing of screenshots). `ev-assets-mac.sh` keeps only what is EV's: the DAG config, the
  "ready" condition (the library's `MC_READY_REGEX` = the block atlas line, `MC_READY_GRACE` 20 s, `MC_FAIL_REGEX` =
  a crash report), summary.json and the pack. Gone from it: its own `sudo launchctl asuser` of `gradlew runClient`,
  the `ps | grep` client/wrapper pid patterns, its own trap; gone from `ev-assets.sh`: the lease streamed from the
  laptop over ssh (TTL 2400 s, no renewal).
- **What changed for `ev-assets.sh` (behaviour kept: same outputs in `review/out/q10/ev-assets/`).** The client is
  ModDevGradle's dev launch (`createClientLaunchScript`, direct `java`) with `-Xmx4G` instead of `gradlew runClient`;
  `tools/smoke/install.sh ev-client` and `./gradlew -PrunSet=ev-client … linkRunModsClient` run in the laptop-side
  build step (no `--server` lock needed) and the run checks the set is linked; the lease is taken and renewed on the
  Mac; options.txt is the shared one (guiScale 2, no early window); `--no-build`, `--timeout` (EV_ASSETS_TIMEOUT
  default), `--lease-wait` flags; exit status is nonzero when the client crashes or times out (it used to be 0 with
  `status` in summary.json); `runner.log` replaces `launch.sh`. The local output folder is still not emptied before
  the pull.
- **Verified.** `tools/client/ev-assets.sh` one run on the free, unlocked desktop: exit 0, block atlas 14 s after
  launch, client stopped 20 s later, 35 s in all; DAG wrote 338 textures / 488 models / 190 blockstates (the Q10
  entry's counts), `pack/` 1,018 files vs the earlier agent's 1,019 (the only file missing is
  `assets/minecraft/atlases/blocks.json`, the atlas definition `ev-assets-pack.py` documents leaving out; the lang
  file has the same 190 entries in another order), client RSS 1,527 MB at the stop (2,188 MB in the Q10 entry under
  `runClient`, another heap and moment). `m6lock status` was `free` after every run including the failed ones.
- **Found while folding, fixed:** (a) Nix's stdenv exports `SIZE=size`, which the shared `SIZE` variable picked
  up (`--size is WxH (got 'size')`): renamed `WINDOW_SIZE`. (b) The dev shell's bash is Nix's non-interactive one:
  no `compgen`. (c) The library's "screen is locked" warning tested `/dev/console`'s owner, which is `root` here
  with `ban` logged in and unlocked, so it fired on every run; it now reads ioreg's `IOConsoleLocked` and a locked
  screen stops the run with exit 4 before anything is leased or started (`IOConsoleLocked = No` for every run
  here; the locked case is not exercised, and nothing types a password). (d) An early `exit 4` left a stale
  `runner.pid`, which the laptop's Ctrl-C handler would then TERM; every exit now removes it.
- **Gates** (master 9e34b45 merged into the branch; the same two gates also passed on the branch merged with
  master 2fd6f15, tests=117): `tools/remote/mac.sh -- 'cd neoforge && ./gradlew build test && python3
  ../tools/ci/junit-summary.py build/test-results/test'` → BUILD SUCCESSFUL, tests=118 failures=0 errors=0
  skipped=0; `tools/remote/mac.sh --server -- './gametest.sh'` → **All 45 required tests passed**.
- **Next.** Nothing blocks. The setup page's memory line stays a snapshot from when the editor model is created
  (Create World open); to show the machine live, `SetupPage` could read `MachineInfo.current()` each frame (it
  is one volatile read now). Not merged into master and not pushed, as asked.

### 2026-09-29 — P1-05 climate (agent worktree agent-ac0e58a6cc29f08bf, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **What.** `crates/deeptime-core/src/climate/` (mod, params, insolation, ebm, belts, lapse,
  orographic, koppen, glaciers, snapshot, tests) — plan §8.3. Ported (NOTICE.md, file headers):
  fastscape-lem/orographic-precipitation (MIT, commit baf8a0c: Smith & Barstad's Fourier transfer
  function and defaults) and climlab's EBM equations (MIT, commit fcbde37: `EBM_seasonal`,
  `StepFunctionAlbedo`, `AplusBT`, `MeridionalHeatDiffusion`, daily insolation — equations, not
  code). New dependency: `rustfft` 6.4.1 (MIT OR Apache-2.0; `cargo deny` clean). Also: preset
  section `climate` (every key optional, `deny_unknown_fields`, range checks naming the key; all
  four `presets/pipeline/*.json` carry it), `PlanetPreset.climate`, the `climate_final` CLI line,
  climate per-process timings, EVENTS.md "The climate's events" (`glaciation`), ramps
  `climate_t`, `precip`, `aridity`, `koppen` (categorical, with `labels`) in ramps.json, a keyframe
  legend in the viewer (swatches for Köppen), `deeptime stage --viewer-layers a,b,…` (keeps a review
  page small; without a `structure…` name it also leaves P1-07's structure view out), and a dev
  subcommand `deeptime climate-dem --dem <f32> --width W [--preset P] [--viewer D]` (one snapshot on
  a given relief). Plan §8.3 and §10.3 P1-05 have "As built"/"As run" notes.
- **Wiring.** The climate is a stage coupled to the plates, placed *before* the surface: the group is
  now `plates → climate → surface → structure` (preset.rs pushes climate when the preset has a
  `climate` section). Each plate step it computes the ground the surface is about to erode (the
  plate model's L1 elevation + the surface's offset advected with the plates exactly as the surface
  will) and writes `climate/{t_mean_c, precip_mm}` (read by `surface/climate.rs` for runoff,
  weathering, facies, carbonate water temperature — now the stage's temperature, not the zonal
  stand-in — and by P1-07's metamorphism via `climate/t_mean_c`) plus `climate/{koppen, t_warm_c,
  t_cold_c, aridity, glacier, ela_m}` and the carried anomalies `climate/anomaly/*`. The deposit
  frames' `precip_mm` and `t_mean_c` therefore are the real climate (checked in the coupled test:
  the last frame's mean precipitation within 20 % of the field's). Without a `climate` section the
  surface keeps its stand-in. Checkpoint schema unchanged (the stage keeps its state in its own
  postcard blob: EBM state, ice sheets, row references, glaciation tracker).
- **Snapshots.** Full snapshot at the first step, every 10 Myr and the last step (51 in 500 Myr);
  between them each cell's sea-level departure from its row's reference (EBM ocean temperature;
  ln of the belts' precipitation), ELA, aridity and Köppen move with the plates (semi-Lagrangian,
  `surface::advect`) and the lapse rate is re-applied on the current ground; land risen from the sea
  gets a Köppen from a sinusoidal year. The EBM warm-starts from the previous snapshot (6 years;
  40 at spin-up; early stop at 0.005 K).
- **Model decisions (implementation details, recorded here).** EBM: climlab's equation with North,
  Cahalan & Coakley 1981's A 203.3 W/m², B 2.09, D 0.649 (climlab's 210/2/0.555 ran this model's
  Earth 3 °C cold) and climlab's `EBM_seasonal` albedos (a₀ 0.33, a₂ 0.25, a_i 0.62, T_f −10 °C);
  per band a land surface (3 m of water) and a sea surface layer over a 50-m slab (coupled 30 W/m²/K,
  k/h through sea ice), latent sea ice at −1.8 °C, slab heat diffusion 0.3 W/m²/K, land–sea
  exchange 10 W/m²/K, 72 backward-Euler steps a year, circular orbit. Belts: trades/westerlies/polar
  easterlies by belt latitude, the ITCZ per column and month as the thermal equator (soft max of
  the land/sea-mixed EBM temperature over ±35°, smoothed 1,500 Earth km, ≤ 22°), monsoon westerlies
  where trades cross the equator, the highs shifted ±6° × sin(tilt)/sin 23.4°; gyre SST anomalies
  and drying under the highs' eastern flanks from each ocean cell's basin position; summer onshore
  air on east coasts 15–45° (`onshore` 1.0). Moisture decays inland over 3,500 Earth km (× (1 + 1.5 ×
  belt wetness) for recycling) to a floor of 0.3; rain shadow e^(−Δz/2,500 m) behind barriers;
  plateau drying above 1.5 km. Orographic: one FFT per wind regime over the rows it can reach,
  0.5 rain-hours/day, lee drying proportional with a 0.3 floor. Köppen: Peel et al. 2007 + As/Aw.
  Glaciers: ELA from Ohmura et al. 1992. `ice_sheet_feedback` = 0 by default (see findings).
- **Measured (Mac, M6 mini, 12 threads, shared — other agents' builds and a Gradle daemon were
  running; load 5–10).** `cargo test --workspace --locked` → cli 20, core 162 + 1 ignored (climate::
  14: insolation limits (S₀/π at the equator on an equinox, S₀ sin ε at the pole on the solstice,
  global mean S₀/4 within 0.2 %); EBM converges with |ASR − OLR| < 0.2 W/m², global 5–20 °C, land
  seasons > 2 × sea at 55°N; diffusion + ice conserve Σ C·T − latent to 1e-9, +5 % star warms > 3 K;
  Smith–Barstad ridge across a westerly: windward mean > 0.5 mm/h, lee < a third of it, negative in
  the shadow, easterly mirror-symmetric to 1e-6; Köppen Af, BWh, Cfb, Dfc, ET, Csa (southern), EF,
  Aw, Dwd; belts and basins; the air's march; ELA; one glaciation event; an idealised planet has
  all five groups, a rain shadow (windward > 1.5 × lee), glaciated polar land; the coupled
  pipeline (3,000 samples, 60 Myr, 256 × 128) writes every field, the surface reads it
  (`from_stage`), 7 snapshots, and a checkpoint resumed on 3 threads (from 1) gives identical
  climate fields, surface, history and events), format 32, inspect 21 + 7, jni 7 + 7, viewer 4;
  `cargo clippy --workspace --all-targets -D warnings`, `cargo fmt --check`, `cargo deny check
  licenses bans` clean; `cd neoforge && ./gradlew build test` → tests=118 failures=0 on the tree merged with master 6d8171f (ramps.json in
  the jar still parses).
  **Acceptance:** `deeptime stage climate --preset pipeline_quick_16k --start scattered_continents
  --length-myr 500 --viewer review/out/p1-climate --viewer-layers elevation,climate/koppen,
  climate/t_mean_c,climate/precip_mm,surface/class,surface/drainage` (after merging P1-07) →
  250 steps, 191 s (plates 37.1, climate 11.7, surface 135.8, structure 6.5; climate: snapshot 7.7
  (cells 4.5, belts 1.7, orographic 0.9, EBM 0.1), topography 2.3, advect 1.7), peak footprint
  2.5 GB; `climate_final=51 t_surface_c=11.3 t_land_c=-0.1 precip_land_mm=536
  koppen_land=A0.10/B0.12/C0.21/D0.04/E0.53 glaciated_land_frac=0.307 glaciations=1
  ebm_t_global_c=15.6 ebm_net_toa_w_m2=0.006 itcz_deg=-8.0..9.0 land_z_p50_m=2496
  land_z_p90_m=5407 land_above_2km=0.598`. **Bench** (before the P1-07 merge; `deeptime bench
  --preset pipeline_quick_16k --start scattered_continents --length-myr 500 --threads 12,1`):
  12 threads 192 s (plates 36.7, climate 13.3, surface 142.2), 1 thread 373 s (143.0 / 37.1 /
  193.1), peak 1.20 GB, **identical summaries on both thread counts**. The climate costs ~5–7 % of
  the run (§8.3's "one or two erosion steps per snapshot" holds: 51 snapshots ≈ 8 s). The surface's
  diffusion took 65–73 s here against 21 s in P1-04's run — the Mac was loaded, not a regression
  (the same code; P1-17 should bench on a quiet machine).
  **Earth check:** `deeptime climate-dem --dem build/earth/earth_720.f32 --width 720 --preset
  pipeline_quick_16k --viewer review/out/p1-climate/earth` on GMT's `earth_relief_01d_p`
  (SRTM15+, 1°, doubled to 720 × 360; a local file, not committed) → EBM 15.6 °C, net TOA 0.013,
  land 8.1 °C / 562 mm/yr (Earth ≈ 8.5 °C / 715 mm), groups A 0.17 / B 0.26 / C 0.10 / D 0.25 /
  E 0.23, 19 of 24 reference cities in the right main group (5 exact: London Cfb, Manaus Af,
  Kinshasa Aw, Chicago Dfa, Vostok EF). The map is recognisably Earth's: Sahara–Arabia–Australia
  deserts, Amazon/Congo/Indonesia A, Siberia/Canada D, Mediterranean C, Andes and Tibet cold.
  Misses: Mumbai (the ITCZ reaches only ~12°N, no Indian monsoon), Beijing/Buenos Aires/Sydney too
  dry (east-coast humid subtropics are weak), Denver/Lhasa/Anchorage too cold, land 20 % dry overall.
- **What the maps show** (review/out/p1-climate, pulled back and looked at; 16 MB after oxipng,
  6 keyframe layers × 11 frames plus `earth/`): temperature has clean zonal bands with continents
  hot in the tropics and cold on their highlands (the lapse rate over the domed interiors is the
  dominant land signal), polar continents deep blue; precipitation has the ITCZ wet belt, the
  subtropical dry belts, the westerlies' wet belt, sharp windward/leeward contrasts on every range
  (orographic stripes on the ridges, dry lees) and drier continental interiors; faint east–west
  streaks off some coasts from the row-wise march survive the 300-km smoothing. Köppen: coastal
  rims of A (tropics), B (subtropical west coasts and lees), C and D, with E (ET/EF) over high
  interiors and polar land; at −450 Myr, when the continents were still low, the Köppen map looks
  Earth-like (large BWh/BSh interiors, Af/Am/Aw equatorial belts, C/D mid-latitudes).
- **Findings for Ben / P1-17.** (1) **Land is too high and therefore too cold.** With the real
  climate the continents end 2.5–3.1 km high at the median (land above 2 km: 60–68 %) against
  1.35 km with the stand-in: dry interiors and rain shadows cut runoff, so the stream-power erosion
  calibrated in P1-04 on the stand-in's wetter interiors no longer holds P1-04's domed continents
  down. Half the land ends in Köppen E. The climate is doing what it should with that relief (the
  Earth check shows it); the fix belongs to the surface's calibration with the climate on (K,
  `runoff_ref_m_yr`, the domed uplift inboard of trenches) — P1-17's tuning, or a decision to
  recalibrate now. (2) The ice-sheet albedo feedback is implemented but off
  (`ice_sheet_feedback` 0): at 1 it froze most of that high land (46 % glaciated). Turn it on once
  the hypsometry is Earth-like. (3) No glacial erosion exists (glaciers are only diagnosed).
- **Deviations from the plan.** (1) Acceptance on `pipeline_quick_16k` (placeholders until P1-08),
  as P1-04. (2) The climate is a coupled member of the plates group, not a stage after it (the plan's
  §8 order says "then climate snapshots"; the surface needs runoff every step). (3) The fixed
  §8.3 files exist except `sealevel.rs` (sea level stays in `surface::params::SeaLevel`); `lapse.rs`
  holds the continentality march; `snapshot.rs` assembles. (4) Smith–Barstad FFT per wind regime
  over the whole width (not per band's latitude strip only; a regime is solved over the rows its
  weights reach). (5) Monthly fields are not stored (annual mean, warmest/coldest month); §5.3's
  `climate/p_mm` is named `climate/precip_mm` (P1-04's interface).
- **Next.** P1-17: recalibrate the surface with the climate on (see finding 1), then
  `ice_sheet_feedback`; P1-09 reads `climate/{koppen, t_mean_c, t_warm_c, t_cold_c, precip_mm,
  aridity}`. The review page is review/out/p1-climate (not published; `earth/index.html` inside
  is the Earth check).


### 2026-09-29 — P1-07 rock classes, column strata and metamorphism (agent worktree agent-a660cc7be45a80cf6, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **What.** New stage `structure` (`crates/deeptime-core/src/structure/{mod, params, columns,
  metamorphism, plutons, traps, hosts, bake_horizons, tests}.rs`), the third member of the coupled
  group: every planet preset with a `surface` section now runs `[plates, surface, structure]`, one
  step each per plate step (optional preset section `structure`, `PLANET_KEYS` + `PlanetPreset`,
  refused without `surface`). Additive to the pipeline: `preset.rs` pushes one more stage; the
  pipeline preset JSONs are unchanged (defaults apply). P1-05's files were not touched.
  - **Metamorphism** in the surface's strata log itself (irreversible; metamorphic rock erodes as
    what it became): each layer steps along its class's `metamorphic_path` as far as the temperature
    at its mid-depth allows. Linear geotherm T_s + q/k·z (q = `plates/heat_flow_mw_m2` capped at
    `max_heat_flow_mw_m2` 120, k 2.5, T_s from `climate/t_mean_c` when P1-05 writes it, else 10 °C);
    under an active collision (collision regime, or Himalayan orogeny < 30 Myr) depths scale by
    β = clamp(e/35 km, 1, 2). Continental crust only. Formation temperatures: anhydrite 50, slate 250,
    quartzite/marble/greenstone 300, phyllite 350, schist 450, amphibolite 500, gneiss 650 °C (rounded
    textbook facies boundaries, cited in the module). No blueschist/eclogite (Q8, P2-02).
  - **Plutons:** Poisson under arcs (`arc_magma_flux` intensity) and collision zones (×0.3),
    1e-4 per km² of full-intensity arc per Myr (Coastal Batholith order), L lognormal median 10 km
    [3, 40], T = 0.6·L^0.6 (Cruden 1998), roof 2–6 km; granodiorite/granite/diorite by setting; ride
    their plate with its finite rotation, unroofed by `surface/material_m`, lost when their oceanic
    crust is subducted. At the end: ellipsoid bodies (no site, role `pluton`, `rock_class` the
    intrusion) with an `aureole` zone (d 0.85–1) of skarn (carbonate host) or hornfels
    (non-crystalline host). FORMAT.md documents them.
  - **Columns:** regolith, then the strata log (the young < 20 Myr and shallow < 150 m part of a
    clastic layer shown as its new `loose_equivalent`, buried loose material and palaeosols as their
    new `lithifies_to`), then the plate model's rock pools less the log (new L1 fields
    `plates/pool/<pool>_m`, **interpolated** — nearest-sample pools gave checkerboard columns), each
    pool one unit by the surface's basement rule, split at the isotherms of its metamorphic path by
    the present geotherm (a thick sedimentary pool turns quartzite below 300 °C), oceanic sediment
    as chalk above a 4.5 km CCD, then peridotite.
  - **Horizon bake:** a column → ≤ K (32) horizons in block space, surface first, every horizon
    ≥ 1 block (thin ones merge into the thinner neighbour, then the thinnest adjacent pair while > K;
    the thicker member's class). `Horizons::check` is the acceptance. The bake is on demand
    (`bake_horizons::bake`); P1-08 writes it and the pluton bodies into world data.
  - **Deposit fields (the task's P1-04 hand-off):** every frame the surface closes gets
    `deeptime:trap_structure` (fold-thrust belts ≤ 300 km from an active orogen, salt domes, rift fault
    blocks, growth faults, drape; `traps.rs`) and `deeptime:host/<class>` for every class a built-in
    type names (share of the top 2 km of the column, plutons included; 8-bit) through the new
    `RecordedHistory::add_fields`. The P1-04 test that listed `trap_structure` as missing now asserts
    **no missing field for any built-in type** (`surface::tests::the_coupled_pipeline_runs_and_resumes`,
    and `structure::tests` also checks every host field exists and sums ≤ 1).
- **Rock classes.** The 59 JSONs stay P1-12's numbers (reviewed against the maps and sections, kept;
  sources now say so). `tools/dev/gen_rock_classes.py` adds `lithifies_to` (10 loose classes) and
  `loose_equivalent` (7 clastic rocks), validated; the Java codec (`RockClass`) and `RockDataTest`
  read them; the generator's docstring lists the facies-rule names the core forms. P1-12's strata pass
  and tags are unaffected (GameTests below).
- **Surface change:** `surface::basement_class` made every basement under a young Andean orogeny
  andesite (a third of the land in the first rock map); now only arcs and the felsic-volcanic pool
  there are andesite. This changes P1-04's surface class and erodibility slightly.
- **Viewer / CLI.** `stage` with `--viewer` on a structure run writes the baked stacks (the strata
  inspector, only the horizons used), map layers `elevation`, `strata/outcrop`, `strata/bedrock`
  (under the regolith), keyframes `structure/{rock, log_grade, trap_structure}`, and six meridian
  cross-sections (`deeptime_viewer::section`: each pixel column the stack as the Java strata pass
  reads it — `LoadedLayer::stack_at` mirrors `StrataPass.column`, unit-tested — with pluton bodies
  painted over). CLI prints `structure_final=…` and `structure[…]` timings. `report.js` now carries
  only sited bodies (8,000 plutons made it 3 MB).
- **Measured (Mac; shared, load 10–23).** `cargo fmt --check`, `clippy --workspace --all-targets
  -D warnings`, `test --workspace` (cli 11, core 149 + 1 ignored, format 32, inspect 21 + 7, jni 7,
  viewer 4), `deny check licenses bans` clean; `gradlew build test` → 106/0; `mac.sh --server --
  ./gametest.sh` → All 45 required tests passed (everyClassInTheWorld 59/59, 240 samples, 0 wrong;
  vanillaOnlyResolvesEveryClass 68 checked, 0 wrong). **Acceptance:** `target/release/deeptime stage
  structure --preset pipeline_quick_16k --start scattered_continents --length-myr 500 --out
  build/p107/end.dtcp --viewer review/out/p1-structure` → `structure_final=250 plutons_emplaced=22000
  plutons_alive=10619 plutons_eroded=9291 plutons_subducted=2090 pluton_bodies=8138
  metamorphosed_km3=9.565e6 frames_published=50 horizons_k=32 mean_horizons=3.88 horizon_check=ok`;
  land_frac 0.342, 1,439 events. **Timings (P1-17):** stage 122.2 s = plates 26.5 + surface 90.8 +
  structure 4.8 s (frame_fields 2.7, plutons 0.5, outputs 0.3, metamorphism 0.2); 129 s wall with
  the viewer bake and sections; max RSS 2.13 GB, peak footprint 3.9 GB (`/usr/bin/time -l`, CLI incl.
  viewer). Earlier runs of the same preset under other load: structure 4.5–7.3 s.
- **What the pictures show** (review/out/p1-structure; review bundle review/out/p1-structure-review,
  23 MB after dropping the non-structure keyframe layers, halving the inspector grid and oxipng; not
  published). Rock at the surface on land: sandstone 29 %, mudstone 20 %, sand 8 %, basalt 5.5 %,
  gravel 5.4 %, granodiorite 5 %, saprolite 4.5 %, conglomerate 4 %, andesite 3.6 %, gneiss 3.5 %,
  shale 3.5 %, schist 1.6 %; sea floor chalk 33 %, shale 31 %, mudstone 14 %, mud 11 %, basalt 6 %.
  Right: sediment-covered continents with crystalline (gneiss, granite, granodiorite) and
  metamorphic belts where orogens exhumed them, loose sand/gravel on young basins, pelagic chalk and
  deep-sea mud on ocean floors, basalt at ridges and on young ocean crust, pluton outcrops and hornfels
  aureoles in arcs, horizons thin up and down with relief. Wrong or doubtful: (1) the continents are
  P1-04's domed highlands, so pole-to-pole sections show 6–9 km plateaus and the stacks' deep parts are
  mostly basement — geologically the columns are thin sediment over thick crust; (2) plutons are many
  and small at 16k (8,138 bodies, ~10 km); P1-17 may lower the rate or merge them into batholiths;
  (3) no folds, faults or dipping beds (Phase 2), so sections are layer-cake with columnar jumps
  between L1 cells (the per-cell section shows the raw stacks; the Java bilinear tops smooth them);
  (4) little evaporite/coal reaches the surface (0.3 %) — P1-04's facies volumes.
- **Deviations.** (1) Acceptance on `pipeline_quick_16k`, not `earthlike_quick_16k` (placeholder
  until P1-08). (2) The work item's `rock_class/*.json` already existed (P1-12); P1-07 extended and
  reviewed them rather than creating them. (3) Metamorphism uses the present geotherm for the pool
  basement (no burial history for pools) and a collision burial factor for the log, not full P–T–t.
  (4) The horizon layer is not yet written into world data (P1-08); plutons are in
  `WorldState::bodies` and written by `finalize_write` as bodies without a site. (5) `inherited_bif`
  is still 0 (no pre-run craton ages).
- **Next.** P1-08 bakes `strata/horizons` from `bake_horizons` and the pluton bodies; P1-14 wires
  `RecordedHistory` (now complete for every built-in type) into `DepositStage`; P1-17 tunes pluton
  rate/size and the metamorphic heat-flow cap.

### 2026-09-29 — P1-20 stage 1: the browser workbench (`deeptime watch`) (agent worktree agent-a3abc4a868f0c3b81, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **What.** `deeptime watch` (crates/deeptime-cli/src/watch/: `mod.rs` command and live cadence,
  `editor.rs` the setup page's rules, `runs.rs` queue and kept runs, `live.rs` per-run stream state,
  `http.rs` the server) serves one page (crates/deeptime-viewer/assets/workbench.{html,js}) with the
  Create World editor's controls, the runs, and a live view of each run: map, globe, legend, layer
  buttons, events (fresh ones ringed, older ones fading over 24 Myr, click for the cause chain), with
  play/pause, step, rewind, speed (0.5–100 Myr/s), a scrubber over every finished step and "follow
  live"; **compare** shows two runs side by side at the same time and layer (the focused pane drives
  the clock, the other shows its frame at or before that time). `?tv` = dark, large, spinning globe,
  no editor, follows the newest run. Commits 6eaf013 (live page), d12bc06 (shared catalogue), 97f06a2
  (server), then the page and the wrapper; merged master 4070f62 cleanly.
- **The editor is driven from the game's own definitions** (Ben: no hand-copied list). The field
  catalogue moved out of `EditorFields.ALL` into `crates/deeptime-viewer/assets/editor_fields.json`,
  which Gradle ships as `deeptime/editor_fields.json` (like ramps.json) and `EditorFields` now reads;
  the workbench reads the same file (`include_str!`). It gained the **planet seed** (an integer field:
  written as a JSON integer, the core's u64; `EditorFields.Field.integer`, `number()`), editable where a
  preset carries a top-level `seed` (the planet pipelines; not the placeholders). Labels come from the
  game's lang file (`deeptime.editor.*`, one new key `field.seed`). Presets: `preset::BUILTIN` (the
  game's four, placeholders) plus `params::PIPELINE_BUILTIN` and `PLATES_BUILTIN` (the real simulation
  until P1-08), grouped into families by `_<n>k` as `EditorModel.familyOf` does; the page defaults to
  `pipeline_quick_16k`. Start configurations and the length range are the core's `StartConfig::ALL`,
  `LENGTH_MYR_RANGE`. The server computes the effective preset with the game's rules (a field is
  editable only when the preset carries its path; out-of-range or non-integer values are refused and
  named; the timeline by `preset::with_timeline`; numbers written as the game writes them), and its
  canonical JSON, share code (`share_code`, the Java twin's partner) and the core's `session::estimate`;
  a pasted code selects the built-in it equals, else becomes an imported preset (`EditorModel.
  importShareCode`). A run's own code is kept, so "Load its settings" refills the form. Tests:
  `edits_land_where_the_core_reads_them` (every available field edited → the core accepts the preset;
  the code round-trips to the same JSON), `a_built_in_code_selects_the_built_in…`, JUnit
  `catalogueIsTheSharedFileAndTheSeedIsAnInteger`.
- **Runs.** One at a time (the Mac is shared); up to 3 wait, a 4th is refused with a message (409);
  cancelling a queued run removes it, cancelling the running one stops it after its current step and
  keeps its frames. Each run is a folder under `--runs` (on the Mac `deeptime-cache/workbench/runs/`):
  `run.json` (request, share code, meta, final status), `events.jsonl` (each event's JSON exactly as
  the session wrote it: re-serialising moved float digits, caught by the stream test), `world/` (the
  session's output: a keyframe per step + the world data) and `frames/` (PNG cache). A restart reads
  every run back (an unfinished one as "interrupted"); the newest `--keep` (default 8) finished runs
  stay, older folders are deleted. **Size: ~1 GB per 500-Myr pipeline_quick_16k run** (250 frames × 10
  layers × 1024×512 RGBA `.dtk` ≈ 930 MB; PNGs ~0.7 MB per viewed layer), so 8 kept ≈ 8 GB of the SSD's
  213 GB free.
- **Watching changes nothing.** The run's preset gets `keyframes.every_myr = 1e-6` (a frame after every
  step; `--every-myr` for fewer); keyframes are drawn from the world between steps and do not feed back
  (plates' L1 sampling already runs every step in the pipeline presets). The runner advances with a zero
  budget (exactly one step) and publishes frames (`keyframe_info`), events (`events_since`) and progress;
  images come from `copy_keyframe` while it runs, from the `.dtk` files after. Test
  `workbench_streams_every_step_and_matches_the_batch_run` (plates+surface, 3,000 samples, 20 Myr):
  10 frames, one per step, times increasing to 0; every event in emission order; the last frame's 10
  layers byte-equal (as PNG) the batch run's final keyframe; the watched world equals the batch
  world (postcard bytes, wall-clock `millis` removed). At full size the watched scattered-continents
  run printed exactly P1-04's batch summary (`land_frac=0.215 sediment_km3=9.571e6`, 1,238 events).
- **Serving: tailnet only.** The server binds 127.0.0.1 on the Mac and `tailscale serve --bg --tcp 8642
  tcp://127.0.0.1:8642` forwards the tailnet to it. Measured before choosing: the Mac's application
  firewall is on and lists only known apps (tailscaled among them); a new binary bound to the 100.x
  address would raise the "accept incoming connections?" dialog on the shared screen. The server also
  refuses peers outside 127/8, ::1, 100.64/10, fd7a:115c:a1e0::/48 and any `--bind` outside them;
  POSTs must be `application/json`, same-origin when the browser sends Origin, and addressed to a
  tailnet/local Host (no DNS rebinding). From the laptop: page and SSE over `mac-mini:8642` 200,
  `10.0.0.217:8642` (LAN) connection refused, a cross-origin POST 403. The tailscale CLI matching the
  Mac's tailscaled 1.102.4 is GC-rooted at `deeptime-cache/tailscale` (the daemon ships none).
- **`tools/viewer/mac-watch.sh`** `[--preset P --start S --length-myr L] [--port N] [--keep N]
  [--threads N] [--tv]`, `stop`, `status`, `tv`: release build via mac.sh (no `--server` lock: no
  Minecraft), the binary copied to `deeptime-cache/workbench/bin/`, started with nohup, pid and log in
  `deeptime-cache/workbench/watch-<port>.{pid,log}`; `stop` kills only a `deeptime watch` pid, removes
  the tailnet forward and the TV's caffeinate. `--tv`/`tv`: `vncdrive.sh status` (waits up to 15 min
  while "locked", never types), `open -a Safari http://localhost:<port>/?tv`, `vncdrive.sh key
  ctrl-super-f` (a moment of graphide's drive lease; reports if the M6 is busy), `vncdrive.sh close`,
  and `caffeinate -d -w <server pid>`. Not run on the screen (Ben was using it); the lock parse was
  checked read-only (`status` → "locked").
- **Stage 2 hook (the sibling worktree's 3D preview and inspector).** Server: `POST /api/preview`
  `{run, frame, t_years, box: {x0, z0, x1, z1} (blocks), lat_lon: {north, south, west, east}}` and
  `GET /api/inspect?run=&frame=&x=&z=` answer 501 `{"available": false, "message"}`. Page: the one
  `Stage2` object in workbench.js (a map click inspects the column there; the ▭ area tool drags a box
  and asks for a preview) draws into the "Inspect & 3D preview" panel; its `render` already follows
  a `status_url` and embeds a `url`, so the job contract (box → job id → status → export URL) needs
  the server routes plus at most that object. Block coordinates from the map follow plan §5.2
  (x = 360°-longitude scaled to C from the seam at −C/2, north −z).
- **Measured** (Mac, `tools/remote/mac.sh`): `cargo test --workspace` (cli 20 incl. 3 editor, 2 watch
  server, live, http; viewer 5; core 139; format 31; inspect 21; jni 7+7), `cargo clippy --workspace
  --all-targets -D warnings`, `cargo fmt --check`, `cargo deny check licenses bans` clean; `cd neoforge
  && ./gradlew build test` → tests=107 failures=0. End to end: `tools/viewer/mac-watch.sh --preset
  pipeline_quick_16k --start scattered_continents --length-myr 500` → 250 frames live, 133 s; a second
  run started through `/api/preset` + `/api/runs` from the laptop (supercontinent, seed 7, ocean 0.6)
  → 250 frames, 2,510 events, 345 s (the Mac was loaded). Screenshots (headless Chromium over
  DevTools, `tools/viewer/shot.py`: Chromium's `--screenshot` fires before a streamed page draws)
  mid-run (LIVE at 380 Myr ago, globe, legend, events), compare at 298 and 98 Myr ago on elevation and
  plates, the editor filled (Random, 800 Myr, gravity 3.71 → estimate peak 1.24 GiB, ~5:50, Run on).
- **Next.** Stage 2 plugs into `/api/preview`, `/api/inspect` and `Stage2`. P1-05/P1-07's new
  layers appear as buttons with no change here (checked by construction: layers are whatever the
  frames name). Open for Ben: see the report's decisions (default keep 8 runs ≈ 8 GB; the TV path
  untested on the screen).

### 2026-09-29 — MIC pack branch `deep-time`: Excavated Variants and its gaps (Q10), rock tags (Q16) (agent, pack worktree ../../pack-deeptime)
- **What.** Pack branch `deep-time` pushed at 16dfd80 (on pack master 303fd89; master untouched): EV 4.3.1 + DAG 6.1.2
  (packwiz, Modrinth); EV configs under `globalresources/mic/globaldata/excavated_variants/excavated_variants/` for
  Born in Chaos black and fired black argillite, Create Nuclear lead ore (with `createnuclear:lead_ores`), Ice and Fire
  silver ore; the rock-tag datapack as `kubejs/data/deeptime/` (120 files, byte-identical); an Almost Unified
  `unification/materials.json`; the branch report `modDocs/deeptime-branch.md` (force-added, packwiz-ignored).
  index.toml edited by hand (additions only, no `packwiz refresh`).
- **Measured** (Q10 recipe on the Mac, `deposits_world_16k`, seed 7, Deep Time 84759b0; install via the new
  `DEEPTIME_PACK=<dir> tools/smoke/install.sh mic|mic-ev`): EV variants 252 (Q10 190). Q10 boxes: 2,291 of 2,507
  Deep Time ore blocks are EV variants (Q10 1,350); black argillite 549/549, fired 395/395. Zn–Pb boxes (MVT sites
  23/24, SEDEX 25): lead 262/262, silver 46/46, zinc 3,176/3,176 variants. No EV/DAG/DefaultResources errors; both
  worlds pass every smoke check incl. restart. Still plain: ore in stone (its variant is the original), dirt, obsidian.
- **Found.** MIC's Almost Unified config is never read, on master too: AU 1.4.2 reads only
  `config/almostunified/unification/*.json`; the pack's `unify.json` is the 0.x layout, so its priorities
  (e.g. `c:ingots/lead → destroy`) do not apply. Asked Ben (pack-wide).
- **Not checked.** DAG's generated textures for the new variants in a client.
- **Leftovers on the Mac** (safe to delete later): /Volumes/Graphide/ban/mic/evpack-verify/,
  /Volumes/Graphide/ban/mic/mods/pack-deeptime-verify/.

### 2026-09-28 — wrap page merged; snowball figure settled (main session f981c0d3)
- **Snowball.** entityCrossesSeam starts the gravity-free snowball at x = 8,188.5 with 0.25 blocks/tick and
  reads it after 40 ticks: 0.25 × (1 − 0.99⁴⁰)/0.01 = 8.28 blocks → 8,196.78 → stored −8,187.22, exactly the
  P0-06 log. The seams page's −8,191.79 was wrong; it now says −8,187.22 forty ticks later.
- **Pages.** https://spacesheep.dev/@benmac/deeptime-wrap (graphics) and the rewritten seams page are both up.

### 2026-09-28 — Seams page rewritten from TW's code; correction to the P0-06 teleport note (agent worktree agent-aa125dc28bc6eb2a3)
- **What.** tools/review/pages/seams/ now opens with where the seam is (x = −C/2 ≡ C/2, pole to pole) and a
  step-by-step of a crossing; docs/research/notes/wrapped_planet_worlds.md §7 records the mechanics read from
  TW 0.19.3's decompiled code: the client keeps its own unfolded x (reset only on login, respawn, dimension
  change); a walking crossing sends no position packet; F3's XYZ/Block/Chunk show the folded x and TW adds
  "Unwrapped" lines; commands refuse absolute x outside [−C/2, C/2) and fold `~`; `/fill`, `/fillbiome`,
  `/clone` refuse boxes across the seam; spawn may land on the seam (spawn.rs uses wrapped distances).
- **Correction.** The P0-06 note that "`setPos` is a teleport and stays unfolded" is too strong: a non-player
  entity past the edge is folded at the end of its next tick, and one added past the edge is folded on
  `addEntity`. The seam tests' use of self-motion is still right, since it exercises the crossing itself.
- **Next.** F3 screenshots on each side of the seam with tools/client/mac-screenshots.sh would settle the
  "Needs a client" rows.

### 2026-09-28 — Q10 evidence: Excavated Variants in MIC, side by side (agent worktree agent-a8b970576f8fca34f, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **What.** The renders Ben asked for before deciding Q10 (decisions log 2026-09-28), not a decision:
  https://spacesheep.dev/@benmac/deeptime-q10-ev (page), linking two BlueMap spaces,
  https://spacesheep.dev/@benmac/deeptime-q10-map-plain and https://spacesheep.dev/@benmac/deeptime-q10-map-ev (all
  private; `spacesheep list` confirms). One MIC world per set from the same recipe: `deposits_world_16k.json`
  (every-class strata), the MIC rock-tag datapack, level seed 7, first-start generation, pack 303fd89 (HEAD snapshot;
  the pack's working tree was dirty and was not used). The EV set is the mic set + EV 4.3.1 + DAG 6.1.2 (168 → 170
  jars). Both worlds hold the same 42 sites (`sites/` and `bodies/` byte-identical); 8,947 ore positions compared in
  the three boxes, 39 hold ore in only one world (feature placement jitter).
- **Tooling (committed on the branch).** `install.sh`: profiles `mic-ev` (= `@include mic` + `@include ev`, pins
  listed once) and `ev-client` (ev + Create), `--seed` (level-seed). `run.py`: `--chunky-at x,z,r` (repeatable), boot
  time / the server's Done figure / RSS at Done for the first boot and the restart. `render-bluemap.sh`:
  `BLUEMAP_BOXES` (several boxes, the second map becomes a cut at each box's height, ambient light 1.0),
  `BLUEMAP_MAP_PREFIX/LABEL/SORT`, `BLUEMAP_PACKS` (BlueMap reads `<config>/packs/`), rstate dropped; the old
  single-box behaviour is unchanged. `deeptime-inspect ore-hosts [--other <world>]`: ore blocks by host rock (the most
  common rock among the 8 same-layer neighbours, then the 5×5 layer, then 3D; strata are horizontal, a thin seam's
  rock above/below is another horizon), with `--other` a position-by-position pairing of two worlds and per-y counts;
  `blocks` takes negative areas. `neoforge/build.gradle`: `-PrunSet=<set>` links any installed set into a run.
  `tools/client/ev-assets.sh` (laptop) → `ev-assets-mac.sh` (Mac): drive lease on graphide's m6lock + the server lock,
  the dev client with `-PrunSet=ev-client` in ban's desktop session via `sudo launchctl asuser`, DAG config
  `full_cache`/`time_resources`, stops at the block atlas; `ev-assets-pack.py` turns DAG's cache (blockstates/models
  under `cache/<mod>/assets/`, painted sprites under `cache/dynamic_asset_generator/tex_sources__minecraft__blocks/`)
  into a resource pack (the DAG atlas file left out). Master's newer `tools/client/mac-client.sh` (lease renewal, pid
  marker) covers the same launch; folding ev-assets into it is a follow-up. `tools/review/q10-ev.sh` (Mac steps:
  install, worlds, inspect, tiles, bluemap, boots; `Q10_PROFILES` redoes one world), `q10_ev.py` (merge, shots, page),
  `block-tiles.py` (a block's 16×16 texture from blockstate → model → texture), `shot-page.py` (a page folder in
  headless Chromium over HTTP).
- **Why a client.** EV's textures do not exist on a server: DAG paints them when a client loads resources (EV's
  models are plain `cube_all` with 4 random variants; the sprites come from a DAG `foreground_transfer` of the ore
  over the new stone). With DAG's full cache the 5-mod client wrote 338 sprites, 488 models, 190 blockstates; BlueMap
  and `deeptime-inspect colors` then draw EV's blocks like any other (colour table 4,304 → 4,494 blocks). No other
  MIC mod has an EV ore or stone config, so the 5-mod client's variant set equals MIC's (190).
- **Bug found and fixed: `EvBridgeImpl` kept one EV stone per block.** EV 4.3.1 ships two stones for
  `minecraft:smooth_basalt` (`excavated_variants:minecraft/smooth_basalt`, overworld; `…:create/smooth_basalt`,
  nether), the map kept the last, and ore in smooth basalt (MIC's basalt, gabbro, chert, amphibolite) never became a
  variant. Now every stone of a block is kept in id order and the first with a block for the ore wins;
  `DepositGameTests.evVariantWhenInstalled` also checks copper in smooth basalt. `mac.sh --server --
  './gametest.sh -PwithEv'` (merged with master 8da1d6b) → **All 45 required tests passed**. The EV world was
  regenerated, re-inspected and re-rendered with the fix.
- **Measured.** Deep Time's own ore (claimed materials) in the three boxes: 2,506 blocks; with EV 1,350 are a
  different block (an ore-in-this-rock variant); 935 sit in hosts EV has no stone for. Per host (ore / changed):
  black argillite 483 / 0 (MIC's coal, shale, slate, phyllite rock), fired black argillite 390 / 0, veridium 375 / 366,
  diorite 241 / 240, crimsite 232 / 232, granite 229 / 220, stone 168 / 0 (the variant is the original), dripstone
  70 / 70, scoria 59 / 58, calcite 42 / 27, dirt 40 / 0, ochrum 26 / 19, smooth basalt 24 / 24 (after the fix),
  packed mud 17 / 16, asurine 16 / 16, limestone 16 / 16, tuff 14 / 13, obsidian 10 / 0, cobblestone 9 / 9. The host
  estimate matched EV's own stone for 1,253 of 1,333 variant positions (94%); "smooth sandstone 14 / 0" rows are
  estimate errors (coal-seam edges). Other mods' ores and vanilla's diamond/lapis/redstone (6,451 blocks) are
  identical in both worlds. Cuts chosen from `ore-hosts` per-y counts: porphyry y 128 (156 ore, 73 differ), placer
  y 162 (gold in gravel, none differ), spawn y 66 (213 coal in argillite, 1 differs).
  Costs: `q10-ev.sh boots` + the restarts (existing world): boot to Done medians 17.4 s without / 17.8 s with EV
  (samples 17.1–20.7 / 17.2–19.5), RSS at Done medians 3,300 / 3,398 MB (overlapping); first start with 2 Chunky
  squares 27.4 s / 5,447 MB peak vs 23.9 s / 5,671 MB (one sample each; the three EV boot-only samples ran the jar
  before the bridge fix). EV registers 190 blocks (171 ores, 111 stones known), two generated data packs; MIC's
  Almost Unified hides the variant items in 7 ore tags. Client: DAG's summed generation time 1,024 ms (textures
  892 ms) on the M6; block atlas 6 s after ModLauncher start on the 5-mod client (RSS 2,188 MB; no no-EV client run).
  EV and DAG: LGPL-3.0-or-later, last released 2024-08-13 (still the newest 1.21.1 builds).
- **Publishing limits.** One BlueMap app with all four maps (195 files, 33 MB) failed twice after 24.2 MB uploaded
  ("fetch failed"), consistent with the ~25 MB per-deploy cap the P1-04 agent met; one space per world (103 files,
  17 MB each) went through (one transient "fetch failed" on an update, fine on retry). Headless Chromium repeats the
  page from the top in captures taller than about 8,200 px, so long pages were reviewed in parts.
- **Checks.** cargo test -p deeptime-inspect 21 + 7, clippy -D warnings, fmt clean; `gradlew build test` tests=106 failures=0 (Mac). Every screenshot on the page
  (12 BlueMap views, 8 sections, 6 ore maps) and the page itself were looked at in headless Chromium over plain HTTP
  before publishing; the hosted spaces are private, so their first real load is Ben's.
- **Next.** Ben decides Q10 from the page. Options the evidence points at, not decided: EV stone configs for Born in
  Chaos's argillites (the largest host EV lacks in MIC, like Deep Time's four stand-ins); EV ore configs for Create
  Nuclear lead / Ice and Fire silver. Rerun `q10-ev.sh` on the real presets after P1-08.

### 2026-09-28 — P1-16 COE and CDG hooks (agent worktree agent-a8ca83a89f23c10ef, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Signatures** (javap + cfr on the Mac MIC set's jars, `createoreexcavation-1.21-1.6.8.jar` sha512
  844ae32f…, `createdieselgenerators-1.21.1-1.3.15.jar` 1507ebcd…, both = the pack's pins; both MIT):
  `public static RecipeHolder<VeinRecipe> OreVeinGenerator.pick(LevelChunk)`, `public void
  OreData.populate(LevelChunk)` (sets `recipe` and `randomMul = rngFromChunk.nextFloat()` only when
  pick returns a vein), private `RandomSpreadGenerator.pick(ServerLevel, ChunkPos, RecipeHolder)` (the
  vein finder's far search and `/coe locate`), `public static int OilChunksSavedData.getBaseOilAmount(
  ServerLevel, ChunkPos)` (reached by the pumpjack through `getChunkOilAmount`, which stores what is
  left once pumping starts and reads a stored value above the infinite threshold as infinite). §7.14
  matched; no drift. `grep -rn calculateOilChunks pack/kubejs` → nothing (re-checked).
- **MIC's COE vein ids** = COE 1.6.8's own 15 `createoreexcavation:vein` recipes (scanned every
  recipe JSON of the 168 MIC jars: only COE's jar has vein or drilling recipes; `pack/kubejs`,
  `config`, `defaultconfigs` name none). Deep Time's deposit types already name them, so **no MIC
  override is needed** (none written). Mapping (`minecraft.coe_vein`): porphyry_cu_mo_au and
  supergene_cu_blanket → `ore_vein_type/copper`, placer_au → `gold`, bif_fe → `iron`, coal_seam →
  `coal`, mvt_zn_pb and sedex_zn_pb → `zinc`, petroleum_trap → none (CDG). Left to COE's noise in
  Deep Time levels: diamond, emerald, hardened_diamond, lapis, redstone, water (and the nether
  veins). Override path for a pack: a datapack copy of `data/deeptime/deeptime/deposit_type/<type>.json`
  with another `coe_vein` (read at run time from the registry, so existing worlds follow).
- **Built.** `compat/SiteChunks` (a site covers a chunk when its `ore` bodies have blocks in the
  chunk's column; volume per canonical chunk, computed once per site, coarser lattice above 2 M
  evaluations; the site filling the chunk most wins), `compat/coe/CoeBridge` (vein = the registry
  type's `coe_vein`, else the record's; none without products; claimed = the types' veins + the
  records'; q = the site's present tonnage as a quantile of its type's split lognormal
  `sizing.tonnage_mt`, `OreData.Percentiles.cdf` with the core's erf), `compat/cdg/CdgBridge`
  (chunk amount = round(`cdg_charge_mb` × `cdg_scale` × the chunk's share of the ore body's
  volume), `Integer.MAX_VALUE` above CDG's infinite threshold like CDG's own generator, 0 away from
  traps), `CoeAccess`/`CdgAccess` + `*Impl` (the mods' own calls, reflective like `EvBridge`), the
  mixins `mixin/compat/createoreexcavation/{OreVeinGeneratorMixin (HEAD site vein, RETURN drop
  claimed), OreDataMixin (populate RETURN → setRandomMul(q)), RandomSpreadGeneratorMixin}` and
  `mixin/compat/createdieselgenerators/OilChunksSavedDataMixin` (HEAD), `compat/CompatHooks` (loads
  the four target classes at mod construction so `defaultRequire 1` fails the boot, and so the smoke's
  "every declared mixin applied" check sees them), `server/CompatCommands` (`probe coe|cdg <chunkX>
  <chunkZ>`: Deep Time's decision, the mod's stored state, `agrees`), `sites near` lists each site's
  `ore_chunk`, `tools/smoke/run.py` adds a check per coe/cdg probe that they agree and a "COE and
  CDG" table to the report. Deposit-type key **`minecraft.cdg_scale`** (Rust struct + validation,
  schema, Java codec; default 1; `petroleum_trap` 0.02, judged: a median trap's chunks hold a few
  million mB like CDG's own oil chunks, large traps' cores reach CDG's infinite threshold); the Java
  codec also reads `sizing.tonnage_mt`. compileOnly `maven.modrinth:create-ore-excavation:tivxiPTo`
  and `create-diesel-generators:UoPH8lO1`; `neoforge.mods.toml` declares both optional
  (`[1.6.8,)`, `[1.21.1-1.3.15,)`). Plan §7.14 and §10.3 P1-16 have "As built" notes; FORMAT.md and
  the schema describe the run-time scaling.
- **Tests** (`CompatGameTests`, namespace `deeptime`, on the GameTest server's Deep Time level
  `deeptime:test_deposits`; the flat overworld is the control where COE and CDG decide alone):
  `coePicksSiteVein` (per type the first vein site's best chunk, generated; COE's stored vein via
  `OreDataAttachment.getData` and `pick` = Deep Time's; every type's vein is a COE recipe),
  `coeDropsClaimedNoise` (COE's own placement grid within ±512 chunks, six grid chunks per vein
  outside every site, as detached plains `LevelChunk`s so COE's biome filter passes: the overworld
  gets the claimed vein, test_deposits none; unclaimed veins identical in both),
  `coeAmountScalesWithTonnage` (q against Rust's `tonnage_quantile` for intact deposits, monotone in
  tonnage per type, COE's populate stores it), `cdgOilFromTrap` (block-by-block volume per chunk,
  base and pumpjack amounts), `cdgZeroElsewhere` (vein-site chunks, the ring round each trap, a grid).
  Without the mods each still checks Deep Time's side and skips the mods' calls.
- **Measured (Mac).** `cargo fmt --check`, `clippy -D warnings`, `test --workspace --locked` (core
  118 + 1 ignored, format 31, …), `deny check licenses bans` clean. `gradlew build test` → tests=106
  failures=0. `./gametest.sh` → **All 45 required tests passed** (compat mixins skipped, Deep Time's
  side: 7 vein sites, 24 quantiles = Rust's, worst |Δ| 2.8e-8; 16 trap chunks; 111 zero chunks).
  `tools/smoke/install.sh mic` (pack 303fd89) then `./gametest.sh -PwithMic` → **All 45 passed**:
  "createoreexcavation hooks bound", "createdieselgenerators hooks bound"; coePicksSiteVein 7 of 7
  agree (bif 13 iron mul 0.995, coal 19, mvt 29 zinc 0.976, placer 6 gold 0.556, porphyry 2 copper
  0.654, sedex 36 zinc 0.073, supergene 1 copper 0.001); coeDropsClaimedNoise 888 grid candidates,
  30 claimed tried, COE picks all 30 in the overworld, 30 dropped in Deep Time (6 each of coal,
  copper, gold, iron, zinc), 56 unclaimed tried, 34 kept, 0 differ; coeAmountScalesWithTonnage 33
  stored = q, 6 types ordered; cdgOilFromTrap trap 37 (0.41 Mt, 480,030,524 mB × 0.02) over 6 chunks,
  max 4,275,128 mB, sum 9,600,610; trap 38 (3.95 Mt) over 10 chunks, max 28,254,828 (4 chunks
  infinite); cdgZeroElsewhere 111 chunks 0 in Deep Time, CDG's own noise finds oil in 65 of 72 of
  the control chunks in the overworld. **MIC smoke** (install `--preset deposits_world_16k.json
  --length-myr 500 --datapack deeptime-mic-rock-tags`; `run.py --profile mic --fresh --rcon "deeptime
  sites near 0 0 12000"`, then `run.py --profile mic` with 16 probes) → **PASS** both boots:
  first start 2.3 s, 42 sites, every declared Deep Time mixin applied by Done, and the report's COE/CDG
  table: porphyry 10 (−348,−65) copper q 0.079, porphyry 2 (−348,−63) copper 0.001, supergene 1
  copper, placer 8 gold 0.754, BIF 11 iron 0.791, coal 15 0.991, MVT 23 zinc 0.559, SEDEX 25 zinc
  0.736, the guaranteed coal+iron pair at (−2,−135) → coal 29 (the fuller), COE stores each with
  mul = q (all **infinite**, see findings); no vein at (0,0) or the trap; CDG over trap 27 (62 Mt,
  73,317,183,120 mB): (334,25) and (333,25) infinite, (336,27) 553,979 mB, (−348,−65) and (0,0) 0 —
  CDG's base and pumpjack amounts equal Deep Time's in all 5; `/coe locate …/copper` → "Could not
  find", `/coe locate …/redstone` → 2,073 blocks away. **After merging master 82ca8ea (P1-04)**:
  cargo fmt/clippy/deny clean, test ok (core 139 + 1 ignored), `gradlew build test` 106/0,
  `./gametest.sh` and `-PwithMic` → All 45 passed each (the same compat numbers).
- **Findings for Ben.** (1) **MIC's COE veins are all infinite**: COE's `defaultInfinite` is true
  and the pack ships no COE server config, so the tonnage quantile changes nothing in MIC today (it
  is stored, and the finite amounts it would give are in the table: copper 10,021–11,576, coal
  17,500–39,772 at `finiteAmountBase` 1000). Finite, size-scaled site veins need
  `defaultconfigs/createoreexcavation-server.toml` with `defaultInfinite = false` on MIC's pack
  branch — a gameplay decision (with infinite veins, unlimited drilling is already confined to
  Deep Time's sites, the decisions file's "unlimited-mining sites"). (2) COE's vein finder "far" line
  and `/coe locate` no longer point at claimed veins in Deep Time levels but do not find sites either
  (prospecting is planned separately). (3) P1-15's stacked guarantees (coal and BIF on one cell) give
  one vein per chunk, the fuller body's. (4) The 16k synthetic trap 27 holds 62 Mt (five times the
  type's P10), so most of its chunks are infinite at `cdg_scale` 0.02; retune on the real presets
  (P1-17).
- **Deviations.** (1) The five tests run on the GameTest server's Deep Time test level, not
  `./gametest-planet.sh -PwithMic` (the planet server has no deposit world; the hooks work in any
  Deep Time level) — the MIC smoke covers a real Deep Time + TW overworld. (2) A third COE mixin
  (`RandomSpreadGeneratorMixin`) the plan did not list. (3) `cdg_scale` added to the deposit-type
  schema (the core only accepts and validates it). (4) q from the site's present tonnage (after
  truncation and leaching), not the drawn quantile; equal for intact deposits. (5) No MIC override
  file: MIC's ids are COE's defaults. (6) CDG's config in the GameTest run is CDG's default (normal
  oil on), MIC's (normal ×0.0) only in the smoke; neither matters inside Deep Time levels.
- **Next step.** After P1-08, rerun `./gametest.sh -PwithMic` and the two-phase MIC smoke above on
  the real preset (pick probe chunks from `sites near`'s `ore_chunk`); P1-17 retunes `cdg_scale`;
  Ben decides (1).

### 2026-09-28 — P1-04 deep-time surface processes on L1 (agent worktree agent-a7fbbf1cc57a46cda, branch worktree-agent-a7fbbf1cc57a46cda)
- **What.** `crates/deeptime-core/src/surface/` (mod, params, grid, advect, pflood, routing, spl,
  diffusion, marine, carbonate, weathering, strata_log, facies, lithology, climate, fields, events,
  oracle, tests) — the surface stage of plan §8.2's deep-time half — plus `deposits/recorded.rs`
  (the deposit engine's recorded field history). From papers: Braun & Willett 2013 (implicit stream
  power, stack), Yuan et al. 2019 (erosion–deposition G, iterated), Barnes et al. 2014
  (Priority-Flood+ε with the pit queue; RichDEM not opened), Bosscher & Schlager 1992, Fu 1981
  (Budyko). Ported from Landlab (MIT; NOTICE.md and file headers): `ExponentialWeatherer(Integrated)`,
  `Lithology`/`LithoLayers` (the log's layer model), `ThresholdEroder` (critical slope),
  `SimpleSubmarineDiffuser` (D(z)), `CarbonateProducer` (equations and defaults). fastscapelib
  only as the oracle (`tools/validation/fastscape_compare.py`, venv, never vendored). New dependency:
  `lz4_flex` in the core (already in the tree via the format crate, MIT). Checkpoint schema 3 → 4.
- **Coupling (the design).** `Stage::coupled_to` + session *groups*: a planet preset with a
  `surface` section builds `[plates, surface]`, and the session steps plates once, then surface once,
  per plate step (keyframes draw the union of the group's layers; a checkpoint inside the group
  stores every member's state; `stage_groups`/`group_of` in `session.rs`). The surface keeps
  `h = plates/elevation_m + offset`; the offset, regolith and strata log are advected
  semi-Lagrangianly with each cell's plate (R(−Ω·Δt), Ω stored at the end of the previous step,
  parents' Ω for plates born in the step); crust born this step starts bare. The surface's net
  change (`surface/feedback_m`, material `surface/material_m`) is splatted onto L0 samples with the
  last coupling's barycentric weights at the start of the next plate step (elevation and rock pools),
  which writes back what it applied (`surface/feedback_applied_m`) and the surface takes that off
  its offset — so L0 and L1 never drift apart and uplift is exactly the plate model's. Pipeline
  presets `presets/pipeline/pipeline_{quick,full}_{16k,32k}.json` = the plate presets +
  `l1_every_steps: 1`, `continental_erosion_mm_yr: 0` (the surface erodes) + a full `surface`
  section; a planet preset with `surface` and `l1_every_steps ≠ 1` is refused. `plates/l1.rs`
  publishes 8 more L1 fields per step (`plates/{flags, top_pool, oxidation, convergence_mm_yr,
  obliquity_deg, slab_age_myr, heat_flow_mw_m2, thickness_m}`).
- **One step (2 Myr, `substeps` = 1).** Open sea = below-sea cells in a connected body ≥ W·H/500
  cells (smaller closed basins are lakes); Priority-Flood+ε from the sea; D8 on the filled surface
  (MFD rejected: the O(n) implicit solve needs one receiver; P1-06 draws channels at block scale);
  runoff-weighted areas; Braun–Willett + Yuan G = 1 with Newton for n ≠ 1, lakes trap sediment to
  their level, deposition capped by supply, critical slope 0.35 with the cut rock routed as sediment;
  river loads fill the sea from each mouth outwards (priority by depth, to sea level − 0 m deepening
  0.5 m/km; a full closed sea spreads the rest over the cells it reached — the first version dumped
  it on one cell and made 160-km towers); Bosscher–Schlager carbonates (× water temperature, ×
  e^(−clastic/20 m), capped at sea level); evaporites 100 m/Myr in restricted (ocean fraction within
  ~150 km < 0.4) arid shallow seas and arid lakes; **2D implicit diffusion by Jacobi-PCG** (land κ
  = 2 m²/yr × rock diffusivity, sea Landlab D(z) with D₀ 300, wave base 100 m) — the first version's
  LOD splitting left N–S streaks on the shelves (D·Δt/Δx² ≈ 26), so LOD is now only the initial
  guess; the residual's sum is returned so Σh is exact; then the strata log (facies rules in
  `facies.rs`: the research's proposal + red beds, dolostone, black shale in restricted deep warm
  water, halite/gypsum by aridity, coal in humid flat aggrading mires), regolith (Landlab integrated
  exponential weathering, w₀ 50 m/Myr × climate; laterite/bauxite rule; buried regolith becomes a
  palaeosol layer), local isostasy (net load × 0.6 (× 0.7 at sea), box-blurred over 50 km).
  Erodibility/diffusivity from the surface rock (regolith only when ≥ 10 m, else the log's top, else
  the basement from the plates' crust type/regime/orogeny/top pool), the 59-class table compiled in
  from the P1-12 JSON (`include_str!`). Runoff: `climate/{t_mean_c,precip_mm}` if a climate stage
  wrote them (P1-05), else a documented zonal stand-in (T = 28 − 42 sin²φ − 6.5 °C/km; belts of P;
  orographic and continentality factors; Fu's Budyko curve). Strata log K = 32, 12 B/layer, merge of
  same-class neighbours, cap by merging the thinnest adjacent pair; base ages, provenance = dominant
  source family and its share (`strata_log::provenance`).
- **Calibration (tuning runs, 100–500 Myr on 16k quick; `build/tune/*.json` variants, not
  committed).** n = 1, m = 0.5: at K 2e-6 whole continents eroded at 50–150 m/Myr and flattened in
  ~20 Myr; lower K only raised relief (erosion at steady state = uplift / (1 − isostasy)). Defaults
  now **n = 2, m = 1, K = 3e-10**, isostasy 0.6, κ 2, deltas fill to sea level: land fraction 0.30 →
  0.26 → 0.215 over 300–500 Myr instead of 0.15; orogens erode fast, cratons slowly. Continents
  still end up domed (interior highlands up to ~3–4 km with erosion rims) — the plate model's broad
  uplift inboard of trenches; P1-17 tunes.
- **Deposit engine frames (P1-14's interface).** Every step adds its fields to an accumulator;
  every 10 Myr a time-averaged frame is packed (16-bit elevation, erosion and deposition rates,
  8-bit the rest, LZ4) into `WorldState::history` (`RecordedHistory: FieldHistory`); receivers of
  the last routing at the end. **Published:** P1-03's `arc_magma_flux`, `magma_oxidation`,
  `convergence_rate_km_myr`, `obliquity_deg`, `slab_age_myr`, `crust_thickness_km`,
  `fault_density` (regime proxy), `orogenic_uplift_m_myr`, `rift_subsidence_m_myr`,
  `heat_flow_mw_m2`, `dist_litho_step_km` (chamfer distance to craton edges), `inherited_bif` (0:
  no pre-run crust ages); P1-04's `elevation_m`, `erosion_rate_m_myr`, `deposition_rate_m_myr`,
  `topographic_gradient_m_km`, `shelf`, `carbonate_platform`, `evaporite`, `euxinic_basin`,
  `coal_facies`, `source_rock_toc`, `burial_temp_c`, `reservoir_quality`, `seal`,
  `water_table_depth_m`; stand-ins until P1-05: `precip_mm`, `t_mean_c`; clocks `oxygenated_oceans`
  1, `land_plants` 1, `bif_acme` 0 (a Phanerozoic Earth until the preset has `planet.atmosphere`).
  Definitions in `surface/fields.rs`. **Missing:** `trap_structure` (P1-07), `host/<class>` (P1-07).
  The coupled test checks every built-in deposit type finds all its fields except the petroleum
  trap's `trap_structure`. `DepositStage` still only builds a synthetic history (wiring
  `RecordedHistory` in is P1-14's next step).
- **Events** (`EVENTS.md`, "The surface's events"): `erosion_pulse` (≥ 200 m/Myr, ≥ 3,000 km²),
  `basin_subsidence` (≥ 80 m/Myr, with the regime as `setting`), `marine_transgression`/`regression`
  (zigzag of the flooded share of continental crust, ≥ 0.04), tracked across frames, written when
  they end (`ongoing` at the end of the run), with causes (nearest orogeny/collision/subduction,
  rift/ridge…).
- **Measured (Mac, `tools/remote/mac.sh`, M6 Mac mini, 12 threads, shared).** `cargo test
  --workspace` → core 139 + 1 ignored (surface:: 20: receivers acyclic and every land cell drains to
  the sea on 5 noisy DEMs; depressions resolved (strictly lower neighbour on the filled surface);
  land without a sea drains to one sink; diffusion Σh conserved to 1e-9 on wrapping/closed/2-wide/
  1-wide grids with D from 0 to 1e5; **stream-power steady state on a 4 × 40 ramp equals the
  analytic profile S_j = (U/(K A_j^m))^(1/n)** within 1e-6 (n = 1) and 1e-3 (n = 2) relative;
  erosion–deposition + mouth fill + diffusion conserve mass to 1e-9; strata log invariants over
  4,000 random deposits/erosions with K = 6; the coupled plates + surface pipeline (3,000 samples,
  60 Myr): log invariants on every cell, mass residual < 1e-6 of the eroded volume, 6 frames with
  every field, acyclic present-day receivers, a checkpoint taken inside the group and resumed on
  another thread count gives the identical surface, log, events and history), cli 11 (new: `stage
  plates --max-steps 3 --out plates.dtcp` then `stage surface --in plates.dtcp` finishes the group;
  `stage surface --preset` runs it from the start), format 31, inspect 19, jni 7+7, viewer 4;
  `cargo fmt`, `cargo clippy --workspace --all-targets -D warnings`, `cargo deny check licenses bans`
  clean; `cd neoforge && ./gradlew build test` → BUILD SUCCESSFUL, tests=106 failures=0.
  **Acceptance run:** `deeptime stage surface --preset pipeline_quick_16k --start scattered_continents
  --length-myr 500 --viewer review/out/p1-surface` (L0 100k, L1 1024 × 512) → 250 steps in 96 s
  (plates 22.3 s, surface 74.0 s: spl 29.9, diffusion 21.1 (112 CG iterations/step), fields+events
  7.0, pflood 3.2, setup 3.1, advect 2.9, isostasy 1.7, outputs 1.3, strata 1.0, routing 0.9,
  carbonate 0.6, marine fill 0.5), peak footprint 1.63 GB (counting allocator 1.11 GB);
  `surface_final=250 land_frac=0.215 max_elevation_m=10048 sediment_km3=9.6e6 eroded_km3=1.20e8
  marine_km3=1.06e8 fluvial_km3=1.1e7 lake_km3=3.1e6 carbonate_km3=6.9e5 evaporite_km3=1.4e5
  mass_residual_km3=-6.2e-4 mean_log_layers=1.71 erosion_pulses=70 basins=265 sea_events=3
  frames=50 history_mib=220`; 1,238 events (338 from the surface, 320 with causes). **Bench**
  (`deeptime bench --preset pipeline_quick_16k --start scattered_continents --length-myr 500
  --threads 12,4,1`, identical results on every thread count): total 86.6 / 111.8 / 235.1 s; plates
  20.7 / 32.7 / 98.0 s; surface 65.9 / 79.0 / 137.0 s. **Against §8.2's cost model** (L1 0.52 M
  cells × 250 steps × 1 µs = 131 s single-threaded, 33–44 s parallel): **137 s single-threaded
  (1.05 µs/cell/step)**, 66 s on 12 threads (2.1×: priority-flood and the stream-power solve are
  serial, 33 s of the 66). The plates stage costs 2.5× its P1-03 time in the pipeline (8.3 → 20.7 s)
  because it now samples L1 every step (relink + 19 fields).
- **Oracle** (`tools/remote/mac.sh -- 'nix shell nixpkgs#python312 -c python3
  tools/validation/fastscape_compare.py --grid 256'`, fastscapelib 0.3.0 in a venv under
  review/out/validation/; 256², dx 4.8 km, K 2e-6, m 0.5, n 1, U 1 mm/yr, 500 × 0.1 Myr, same
  initial noise) → `hypsometry_ks=0.0209 chi_slope_ratio=1.0000 chi_fit_ratio=0.9997
  area_log10_ks=0.0038`, PASS (tolerances in the script: KS ≤ 0.06 from seeds 1–5 (0.010–0.040),
  χ ratio 0.98–1.02 (1.0000 on every seed; k_s = U/K = 500 both), area KS ≤ 0.02). From the same
  mature surface one step agrees cell for cell (99.98–100 % identical areas, mean |Δz| 2 mm); the
  networks diverge from the initial noise because Deep Time fills lakes with sediment at G = 0 and
  fastscapelib does not (intended here) and the floods break ties differently. Deep Time 2.5 s vs
  fastscapelib 3.6 s per run.
- **What the keyframes show** (review/out/p1-surface, 11 frames × 10 layers, pulled back and looked
  at): at −498 Myr six low green continents; by −348 Myr they carry interior highlands (tan to brown,
  2–4 km) with erosion concentrated on escarpments and active margins, dendritic river networks
  (drainage layer) running to every coast, and light-blue shelves prograding from the river mouths;
  at −248 Myr a northern supercontinent with deposition strung along its margins (shelf wedges,
  deltas, a restricted interior sea) while the regime layer shows back-arc belts inboard of trenches
  and cratonic interiors; at 0 Myr scattered continents with snow-white peaks (≥ 4.7 km) in collision
  and arc belts, erosion rims around domed interiors, thick sediment (≥ 2 km) on passive margins and
  in basins beside orogens, laterite/saprolite over the tropical lowlands, andesite on arc margins,
  shale veneers on shelves and basalt on bare ocean floor. Mass balance: 88 % of eroded rock reaches
  the sea; the log keeps 9.6e6 km³ (the rest was subducted with its crust or cleared by new crust).
- **Deviations from the plan.** (1) The acceptance command is `stage surface --preset
  pipeline_quick_16k`, not `stage erosion --preset earthlike_quick_16k` (the built-ins stay P1-13's
  placeholders until P1-08; the stage is named `surface`). (2) Interleaving is a session feature
  (coupled groups), so `stage surface --in` takes a checkpoint whose next stage is the plates lead.
  (3) Priority-flood and the stream-power solve are serial (plan: tiled flood, per-basin parallel
  solve) — they are 33 s of 66 s on 12 threads; the obvious speed-up for P1-17. (4) Diffusion is 2D
  PCG, not ADI (ADI/LOD streaks at deep-time steps). (5) Default stream power n = 2, m = 1 (not the
  plan's implied n = 1). (6) Isostasy (not in the plan) is needed for sediment basins and orogen
  persistence.
- **Not done / next.** Wire `RecordedHistory` into `DepositStage` and put `deposits` after the group
  in the pipeline presets (P1-14); `trap_structure` and host fractions (P1-07); P1-05 replaces the
  climate stand-in by writing `climate/{t_mean_c,precip_mm}`; tuning (domed continents, event counts
  — 265 basins in 500 Myr is many; history is 4.4 MiB per frame at 16k quick, 220 MiB per 500 Myr,
  ~4× that at 16k full) and parallel flood/solve in P1-17. A spacesheep review page "Deep Time — P1
  erosion" is not published (the bundle is review/out/p1-surface on the Mac and laptop).

### 2026-09-28 — Real-client screenshots on the Mac's desktop session (agent worktree agent-ab0d4cbba4384f9e5)
- **Why.** Ben (2026-09-28, CLAUDE.md "Real-client runs"): clients boot in the M6 Mac mini's
  logged-in desktop session, never under the laptop's Xvfb. P1-11's `tools/client/screenshots.sh`
  (HeadlessMC + Xvfb, laptop) now refuses to run unless `DEEPTIME_XVFB_CLIENT=1`.
- **Shape.** `tools/client/mac-screenshots.sh [--name N] [--no-build] [--no-guidance] [--no-create]
  [--size WxH] [--timeout S] [--lease-wait S]` on the laptop: (1) `tools/remote/mac.sh` syncs
  and runs `./gradlew classes gametestClasses createClientLaunchScript linkRunModsClient`;
  (2) copies graphide's `utilities/scripts/mactest/m6lock.sh` (read-only source; `M6LOCK_SH`
  overrides the path) to `/Volumes/Graphide/ban/deeptime-cache/m6lock.sh`; (3) `mac.sh --no-sync
  --server --pull review/out/<name>` runs `tools/client/mac-client.sh` on the Mac, so Deep Time's
  server lock is held for the whole client run (the client starts an integrated server); (4) prints
  the driver's lines and the PNG list. Ctrl-C/TERM on the laptop TERMs the Mac runner through its
  pid file (`neoforge/build/mac-client/runner.pid`).
- **Launch.** Not HeadlessMC: offline it stubs LWJGL unless it sees Xvfb, and its dummy assets are
  what made P1-11's backgrounds dark red. Instead ModDevGradle's own dev launch:
  `createClientLaunchScript` writes `neoforge/build/moddev/runClient.sh` (one line, `(cd run; exec
  java @clientRunClasspath.txt @clientRunVmArgs.txt -Dfml.modFolders=… net.neoforged.devlaunch.Main
  @clientRunProgramArgs.txt)`, `-XstartOnFirstThread` already in the VM args on macOS; real assets
  from the Gradle cache). The runner reads its words back, adds `-Ddeeptime.client.run=<id>` (the
  kill marker), `-Xmx3G`, `-Ddeeptime.dev.screenshots=true` and the pass's properties, and starts it
  with `sudo -n launchctl asuser 501 sudo -n -u ban -H env -i … /bin/bash launch-<pass>.sh`
  (graphide's mactest pattern: from sshd a process is in the `Background` launchd manager and gets
  no WindowServer; this puts it in `Aqua`, checked with `launchctl managername`). Deep Time loads
  from the build output (natives `aarch64-apple-darwin` from build/resources), Toroidal World
  0.19.3's bundle from run/mods (linkRunModsClient); the log shows both and TW's 1,024-chunk
  cylinder. `options.txt` as P1-11's plus master/music volume 0 (Ben's speakers) and no pause on
  lost focus; NeoForge's early window off; `caffeinate -d -w <pid>` keeps the display awake. PNGs
  come from the GL framebuffer (ScreenshotDriver), so no macOS Screen Recording permission is
  involved (`screencapture` would need one; graphide notes nothing grants it).
- **Lease.** graphide's M6 lock, kind `drive` (excludes other desktop walks and the perf lane;
  graphide's `tests` runs may continue beside it), holder `deeptime-client:<checkout name>`, TTL
  300 s, renewed every 60 s while the client runs (`acquire` again; if that fails — the lease lapsed
  and someone else took the desktop — the client is stopped), released on every exit path. Waits up
  to `--lease-wait` (600 s) for a busy desktop and never breaks another holder's lease. Order:
  Deep Time's server lock first (mac.sh), then the lease. Every failure, timeout, lost lease or
  signal kills only this run's JVM: its pid from the launch script, checked against the marker.
- **Measured (Mac, `tools/client/mac-screenshots.sh --name p1-11-mac`, about 1 min 15 s end to end
  with an up-to-date build; three full runs, the same every time):** main pass 46–47 s (~680 ticks:
  Create World with "Deep Time Planet" and Create blocked `NOT_SIMULATED` → setup → timeline → run
  live, map then globe → done, 9 keyframes, 13 events → scrubbed 5/9 → review, spawn −8.3° N
  204.4° E, 6 province cells → Accept → `Moved staged … (rename)`, `Bound … C = 16384` → in world,
  last run at (1110, 65, 373)); guidance pass 14–17 s (Create blocked `NATIVES`).
  Twelve 854×480 PNGs in `review/out/p1-11-mac/` (gitignored), the same twelve as P1-11's laptop
  set, same content; differences: the real title panorama (blurred) behind every screen instead of
  the dark fallback, Steve's arm, and the in-world view (random seed: other vegetation, a coast in
  view). Failure paths tried: `--timeout 15` → client stopped at 15 s, lease released, 5 PNGs, exit 1;
  SIGTERM to the laptop's process group during a client run → runner stopped the client, released
  the lease, server lock gone, no process of this checkout left; the same while waiting for the
  server lock (another agent's `q10-ev.sh` held it) → nothing left behind. The drive lease and the
  server lock were free after every run.
- **Found:** the setup page reads "This machine: 1.0 GiB free of 16.0 GiB" on the Mac:
  `MachineInfo` falls back to `OperatingSystemMXBean.getFreeMemorySize()`, which on macOS counts only
  free pages (inactive and purgeable memory, which the system hands out on demand, are left out) —
  the macOS twin of the Linux `MemFree` bug P1-11 fixed. The memory warnings will misfire on Macs.
  Next step for P1-11: a macOS reading (vm_stat free + inactive + speculative pages, as graphide's
  m6lock `avail_gb` does, or `host_statistics64` through the JNI side).
- **Needs from Ben:** nothing while `ban` stays logged in at the Mac (auto-login or a Screen Sharing
  login); with nobody logged in the runner stops with `BLOCKED: … no GUI session` (exit 4). A
  locked screen is only warned about (untested: every run here had `ban` unlocked at the console).
### 2026-09-28 — wrap review page (agent worktree agent-abdcd741641764e1f)
- **New page `tools/review/pages/wrap/`, published private as https://spacesheep.dev/@benmac/deeptime-wrap**
  (`tools/review/publish.sh wrap tools/review/pages/wrap -m "how the wrap works" --emoji 🌀`; space
  87492f6d-8791-4d7d-8f43-d96f19f63913, recorded in `tools/review/spaces.json`). One self-contained HTML file, raw WebGL for
  the rolling planet, canvas for the rest; follows the 16k/32k switch; light and dark; checked with headless Chromium at
  1400 px and in a 390 px iframe (`--use-angle=swiftshader-webgl --enable-unsafe-swiftshader`; plain
  `--use-gl=swiftshader` gives no WebGL), and every control clicked once with no script errors. `#shot` in the URL runs the
  crossing to just past the seam for screenshots.
- **Mechanics read from TW 0.19.3's source** (github.com/anyttng/toroidal_world at 3c1f3a3, the 0.19.3 build stamp; key
  classes and F3 strings confirmed in the jar). They agree point for point with the sibling note's §7 in
  `docs/research/notes/wrapped_planet_worlds.md`. The page labels each claim "seen on a test server", "read in code" or
  "not yet watched in a client".
- **Corrections to other docs, found here:** spawn is not at x = 0 (Deep Time picks it from `spawn/candidates.json`, so it
  can be near the seam); the seams page's snowball figure (came out at x = −8,191.79) differs from P0-06's build log
  (x = −8,187.22).
- **Next:** the seams page can link to this one; everything client-side on it stays unverified until
  `tools/client/mac-screenshots.sh` (or a successor) walks a real client across the seam.

### 2026-09-28 — P1-11 Customize editor, timelapse and staging, on the placeholder presets (agent worktree agent-a61f0e6a8e0a67dfa)
- **Shape.** Client-only screens in `screen/` (reached only through `screen/ClientSetup`, a
  `@Mod(dist = CLIENT)` class: preset editor for `deeptime:planet` and `planet_32k`, the Create
  guard, the staging cleanup, the dev screenshot driver); all rules in plain-Java `screen/model/`
  (no Minecraft imports; `ScreenSourcesTest` enforces both). Plan §7.12 has an "As built in P1-11"
  paragraph; §10.3 P1-11 lists the class names that differ from the plan's (`EditorModel`,
  `TimelapseModel`, `ReviewModel` instead of `PlanetEditorModel`/`RunModel`; share codes stay in
  P1-13's `preset/ShareCode`; `worldgen/PlanetGenerators.withStaging` instead of
  `GeologicChunkGenerator.forStaging`).
- **Setup page** (`EditorModel`, `EditorFields`): preset = built-in family (`earthlike_quick`,
  `earthlike_full`) × size (16k/32k picks the built-in), or an imported code (size fixed); timeline
  start and length with no default, Run disabled until both are chosen (plus: estimate in, peak ≤
  cap, numbers in range, natives linked); the §5.1 parameter catalogue (planet, scale, multipliers,
  essentials, fantasy toggles as Phase 3 placeholders) where **a field is editable only when the
  preset carries its JSON path** — the placeholders carry none and the core rejects unknown keys, so
  they show unavailable, and the real presets light them up without a screen change (the paths are
  my proposal for §5.1's structs: `planet.gravity_m_s2`, `scale.m_per_block_h`,
  `deposits.count_multiplier`, `scale.feature_scale.rivers`, `surface.drama`,
  `playability.essentials`, …; P1-03…P1-17 must use them or change the catalogue line). Memory cap
  default 4 GiB, any value ≥ 0.25 accepted, warnings when above the machine's RAM or free memory;
  the estimate runs off the render thread, stale answers dropped. Share codes: export = the
  effective preset (timeline included); import selects the built-in when the code is one, else a
  custom preset.
- **Run page** (`TimelapseModel`, `SessionRunHandle`): `SimSession.launch` (new) creates the session
  on the `deeptime-sim` worker (plan §6.2; `start` keeps creating on the caller for the server) and,
  with `collectTimeline`, gathers keyframe infos and events after each step and before destroy, so
  the last step's output is never lost; `SimSession.copyKeyframe` while alive, `worlddata/KeyframeFiles`
  (pure-Java `DTKF` reader over the new `worlddata/Container`) after. Frames decode on a loader
  thread into a 64 MiB LRU ring; the view follows the newest frame until scrubbed back; events show
  up to the frame's time, fresh ones (after the previous frame) highlighted and drawn larger; layer
  buttons are data-driven (every layer a frame names); playback 400 ms/frame; the globe is an
  orthographic projection on the CPU (`GlobeProjection`, 256² texture). Closing the editor leaves a
  run going; reopening Customize (one model per Create World screen) shows it. Cancelled and failed
  runs delete their staging folder from the completion future (no polling needed).
- **Review page** (`ReviewModel`): the staged world data's `elevation` sampled 512×256 and coloured
  with `Ramps` (Java port of `deeptime_format::Ramps`; `ramps.json` now copied into the jar as
  `deeptime/ramps.json`; `ModelHelpersTest` checks the Rust test's colours plus a half-way
  rounding case), fallback the last keyframe; spawn marker = first `spawn/candidates.json` entry
  above sea level; ore-province hints (`ProvinceHints`: dominant deposit type per 30° cell, each site
  weighing 1 in its cell and 0.5 in the eight around, shown at ≥ 1; `worlddata/SiteRecords` reads
  `sites.dtsr`). Biome names and the staged §7.2 mapping: P1-09.
- **Guidance panel** (`GuidanceModel`) when the natives did not load and nothing finished to review.
- **Staging**: `server/Staging` (`<gamedir>/deeptime/staging/<uuid>/`, ids validated as lower-case
  UUIDs, `cleanupAbandoned`: untouched 24 h and held by no open editor), `server/StagingMover`
  (target already has a manifest → nothing; rename with `ATOMIC_MOVE`; else copy to
  `<target>.staging-copy` comparing each file's CRC32 with the source, check every manifest entry
  with plain reads — a memory-mapped tile pack would block the rename on Windows —, rename into
  place, then delete the staging folder; any failure removes the partial copy and leaves staging
  untouched). `WorldDataBinder`'s HIGHEST listener now runs `moveStaged` → `FirstStartGenerator.prepare`
  → bind (a failed move stops a dedicated server through `ServerStop`, throws on an integrated one);
  cleanup runs in the background at every server start and at client setup. Accept =
  `PlanetGenerators.withStaging` (current overworld's biome source and settings; `WorldDataRef`
  with the staging id and the preset's C; TW's shape follows at bake). **Create is disabled** for a
  Deep Time world until staged data exists (`PlanetGenerators.createBlock`: no accepted run, natives
  missing, staging folder gone), Enter held back too (`screen/CreateWorldGuard`); a world type
  switched away and back gets the accepted binding back when Customize reopens.
- **Found on master:** `Cargo.lock` lacked `flate2` under `deeptime-core` (a merge slip after
  P1-13), so every `--locked` build refused to run (`build-natives.sh`, `cargo tree --locked`);
  unlocked builds rewrote the lock silently. Fixed in commit 6c6298d.
- **Tests.** JUnit (Mac, `cd neoforge && ./gradlew build`): tests=93 failures=0 — `EditorModelTest`
  (9: families and sizes, Run disabled without a timeline, stale estimates, cap, natives missing,
  fields follow the preset, share-code round trip, staging folders across run/accept/discard/cancel,
  launch failure), `TimelapseModelTest` (8: live following, event cursor and freshness, layers,
  off-thread ring, bounded ring, playback, cleanup, progress line), `ModelHelpersTest` (7: ramps
  parity, globe, formats, province hints, real presets carry every field — vacuous until P1-08 —,
  MemAvailable, guidance),
  `StagingMoverTest` (7: rename, copy+verify, corrupted manifest keeps staging, incomplete refused,
  partial target replaced, cleanup by age/held), `TimelineAcrossJniTest` (2: JNI pixels = disk
  pixels for every frame, a launched session feeds the timelapse, create errors and early cancel
  fail the completion), `ScreenSourcesTest` (2). GameTests (Mac, `./gametest.sh`): All 25 required
  tests passed — `acceptBuildsGenerator` (codec keeps `staging_id`, C 32,768 → TW 2,048 chunks,
  Create blocked/unblocked/blocked again), `stagingMovedOnStart` (copy route, restart moves nothing,
  bound terrain within 0.031 blocks of the analytic terrain), `editorRunsAcceptsAndBinds` (share code
  → timeline → real estimate → run of a fast placeholder at C 1,024 → 9 frames, 8 events → review map
  from world data, spawn marker, 1 province → Accept → rename into the world → bound, max error
  0.031), `shareCodeThroughModel` (the core refuses a preset with a `planet` section: "$.planet:
  unknown key"). cargo fmt, clippy -D warnings, test --workspace, deny: clean.
- **Client check (best effort, laptop, real GL).** `tools/client/screenshots.sh <jar> <linux .so>`
  (jar built on the Mac with `./gradlew jar`, the Linux library with `tools/natives/build-natives.sh
  --only linux-x86_64-gnu`, both pulled; HeadlessMC 2.10.0 under Xvfb + Mesa, Deep Time + TW only,
  `ScreenshotDriver` walks the screens): Create World with "Deep Time Planet" and Create disabled
  (`NOT_SIMULATED`) → setup page (Run disabled: "Choose how the continents start", estimate 2.0 MiB /
  ~10 s / 2 stages) → timeline set, Run → timelapse live (map, then globe) → done (9 keyframes, 13
  events), scrubbed to 5/9 → review (final map from world data, spawn −8.3° N 204.4° E, 6 province
  cells) → Accept → Create enabled → the world is created: `Moved staged Deep Time world data …
  (rename)`, `Bound … staged as <id>`, player in the world at (1114, 66, 377) — the review's spawn
  marker. With `GUIDANCE=1` (`-Ddeeptime.natives.forceFail=unsupported`): the guidance panel, and
  Create blocked (`NATIVES`). Twelve screenshots in `review/out/p1-11-client/` (gitignored), no
  client crash. Three things it took: HeadlessMC's `--jvm` passes only its first token (the -D
  properties go through `JAVA_TOOL_OPTIONS`); offline it stubs LWJGL (black screenshots) unless
  `hmc.check.xvfb=true` lets it see Xvfb; the game's STB PNG writer fails there, so the driver
  writes PNGs with ImageIO. It fixed two screen bugs: the scrubber label lagged while following
  live, and Linux "free memory" was `MemFree` (0.7 GiB of 15.4) — now `MemAvailable`. Layout and
  feel remain unreviewed by a person (plan §9.8).
- **Merged master f14d258 (P1-03 plates, P1-18 pole cap) into this branch.** Conflicts: STATUS.md
  (both build-log entries kept) and `en_us.json` (P1-18 added one too; keys unioned). Adapted:
  P1-18's six-field `WorldDataRef` (Accept keeps the world preset's pole-cap lines), and
  `ScreenSourcesTest` treats P1-18's `client/` package as client-only too. P1-03's planet pipeline
  (`params.rs`) uses exactly the catalogue's `planet.*` and `scale.m_per_block_*` paths; the
  catalogue's ranges now match its validation (tilt 0–180°, ocean fraction 0.05–0.95), and the new
  GameTest `realPresetFieldsLightUp` imports a P1-03 planet preset: those seven fields become
  editable, an edit lands at the path the core reads, and the core's estimate accepts it (174 MiB,
  1 stage). After the merge: JUnit tests=101 failures=0; ./gametest.sh All 31 required tests
  passed; cargo fmt, clippy -D warnings, test --locked (193 passed), deny clean.
- **Next.** When P1-08's real presets land: rerun `./gametest.sh` and the client check with them,
  make the `EditorFields` paths match the real schema (`ModelHelpersTest.realPresetsCarryEveryField`
  fails for a real built-in that lacks one), add P1-09's biome names and staged climate mapping at Accept.

### 2026-09-28 — P1-18 pole cap and client info (agent worktree agent-a4399e9f8080242f4)
- **The cap** (`wrap/PoleCap`, `wrap/PoleCapCurve`; replaces P0-06's `PoleCapPrototype` and its
  `prototypepushback` test). Runs after every tick of a Deep Time level (`LevelTickEvent.Post`).
  Lines per level from the generator's `world_data`: new optional codec fields `pole_cap_deg`
  (default 75) and `pole_stop_deg` (default 80), validated cap < stop ≤ 90 and written out in
  both world presets (and `tools/dev/gen_worldgen.py`); `WorldDataRef.poleCap()`. The Rust
  preset's `scale.pole_cap_deg` (plan §5.1) reaches the generator when P1-11 writes world presets
  from a preset; the placeholder presets name neither.
- **The curves** (the seam page's were illustrations; now the real ones, documented in
  `PoleCapCurve`, pinned by `PoleCapCurveTest`): s = depth into the 75°–80° band. Pushback
  0.10·s^1.5 blocks/tick² toward the equator (76° 0.0089, 77.5° 0.0354, 79° 0.0716, 80° 0.10;
  ground friction makes a per-tick push p a drift of 2.2p, the same factor as the walking input
  0.1, so at 80° walking is cancelled exactly, sprinting creeps, and anything airborne (factor 11)
  or gliding is thrown back hard). Storm s^0.7 (0.20 at 75.5°, 0.62 at 77.5°). Sable wind
  1.0·s^1.5 blocks/s added per tick (20 blocks/s² at the stop; a 40 blocks/s jet entering at 75°
  turns at 78.6° at 16k).
- **Player policy** (decided here): survival/adventure get wind, warning and stop; **creative**
  gets the storm, the warning and the hard stop but no wind (the vanilla border stops creative
  players too; builders at 76° should not be blown about); **spectators** only see the storm (no
  wind, no stop, no warning, as they pass the vanilla border); fake players are ignored.
  **Vehicles**: a player's root vehicle, under the strongest policy of its riders; empty vehicles and
  mob-only stacks are not capped (nothing gained).
- **Who moves whom.** A player's own movement (and a vehicle it steers: boat, horse) is its
  client's; a server motion packet would *replace* the client's velocity every tick (no momentum,
  broken jumps), so the server sends the wind in `PoleStormPayload.push` and Deep Time's client adds
  it before the player moves (`client/PolarStorm`). Vehicles the server simulates (minecarts, any
  ridden-but-not-steered root) are pushed server-side. The hard stop is always the server's: a
  player past the line is teleported onto it, 0.05 blocks inside (absolute teleport, which zeroes
  the client's velocity; y lifted out of terrain if the line cuts rising ground), a vehicle is moved
  with its riders (plus `ClientboundMoveVehiclePacket` when a client steers it), riders of anything
  else (Create contraptions, modded seats) are dismounted and stopped alone. Action bar: gold storm
  line on entry and every 40 ticks, red stop line at once (`assets/deeptime/lang/en_us.json`).
- **Sable** (`compat/sable/SablePoleCap`, loaded only with Sable; with `SableTestOps` the only
  classes naming Sable types): every sub-level of the level from
  `SubLevelContainer.getContainer(level).getAllSubLevels()`; wind at the pose through
  `RigidBodyHandle.addLinearAndAngularVelocity`; stop when the hull's poleward edge (global bounds)
  is past the line: poleward velocity cancelled plus the full wind, never a teleport (a teleport
  would leave the deck's players behind). Sable declared `optional` `[2.0.5,)` in mods.toml.
- **Networking** (`net/`): `DeepTimeNetwork` registers `deeptime:world_info` (`WorldInfoPayload`:
  planet flag, dimension, world-data format version, C, m/block h and v (manifest, new optional
  fields), cap and stop, planet name = the manifest source's preset/kind; sent on login, dimension
  change, respawn) and `deeptime:pole_storm` (`PoleStormPayload`: storm, push; sent on a change of
  0.02 storm or 0.002 push, and on every change to or from zero; clamped on arrival), both
  play-to-client, version "1", **required**. `ClientHandlers` only stores into `ClientPlanetState`
  (no client classes, safe on a dedicated server). NeoForge has no public view of its
  registrations: `DeepTimeNetwork.registrations()` reads `NetworkRegistry.PAYLOAD_REGISTRATIONS` by
  reflection. New probes: `/deeptime probe pole [z]`, `/deeptime probe network`.
- **Client** (`client/DeepTimeClient`, `client/PolarStorm`; loaded only when
  `FMLEnvironment.dist == CLIENT`): the F3 line (`net/PlanetDebugLine`, left column:
  `Deep Time: 76.904° N 90.000° E · wrap-test (C 16384, 300 m/block) · polar storm 62%`), the wind,
  fog (far plane from the render distance down to 20 blocks at full storm, near plane to 0,
  colour toward pale grey; camera in air only) and up to 16 snowflakes a tick blown equatorward;
  the drawn storm eases 0.02/tick. **Compiled only**: no client was booted. mic-climate's
  HeadlessMC route is not cheap here: its `-lwjgl` mode never got past "Waiting for player to
  load", its Xvfb mode is Linux-only (the Mac has no X server; the laptop must not run heavy
  jobs), and reaching the storm would also need a Deep Time world with the player at 75°+.
- **Tests.** Players are `gametest/MockPlayers`: real `ServerPlayer`s placed through
  `PlayerList.placeNewPlayer` on an `EmbeddedChannel` connection NeoForge treats as a full NeoForge
  client (`NetworkRegistry.configureMockConnection`), so tests read the packets the server sends
  (storm payloads, action bar = system chat with overlay) and see its teleports; nothing moves
  them. Planet namespace (`PoleCapGameTests`; each test forces its own polar chunk strips at
  y = max − 20.5, so the harness only needs a loaded spot): `entityPushedBack`, `hardStopAt80`,
  `sableHullPushedBack`. GameTest namespace (`NetworkGameTests`): `worldInfoRoundTrip`,
  `poleStormRoundTrip`, `worldInfoChannelRequired` (registration + NeoForge's own negotiator),
  `planetDebugLine`, `worldInfoFollowsThePlayer` (own batch). The planet server now also enables
  the `deeptime` namespace so `worldinfochannelrequired` runs there too. Headless connection test:
  `gametest-planet.sh --scenario refuse_client` (new `tools/planet/mcclient.py`, stdlib, protocol
  767 with compression) logs in over TCP three times with the channel list from
  `/deeptime probe network` (read from the server log: RCON returns only 4 kB of the 20 kB line).
  `--scenario` is repeatable now. `tools/review/phase0.py` runs the new pole tests instead of the
  prototype; the seam page shows the real curves, texts and "Verified".
- **Measured** (Mac; numbers from the planet run of the final commit):
  `tools/remote/mac.sh --server -- '(cd neoforge && ./gradlew build test) ; ./gametest.sh ;
  tools/smoke/install.sh standalone ; ./gametest-planet.sh --run columnscontinuous,entitycrossesseam,structurestraddlesseam,treestraddlesseam,shaftlineacrossseam@8185,250,0 --run jetcrossesseam@8185,250,64 --run entitypushedback,hardstopat80,sablehullpushedback,worldinfochannelrequired@0,300,64 --scenario sable_straddle --scenario refuse_client'`
  → JUnit `tests=65 failures=0 errors=0` (PoleCapCurveTest 8 new); GameTest server `All 26 required
  tests passed`; planet server `PASS`: the five seam tests, jetcrossesseam 20/20, sable_straddle OK,
  and the four P1-18 tests. entityPushedBack (C = 16,384, 40 ticks): players N and S identical —
  74.5° storm 0 / push 0 / no bar; 76° 0.3241 / 0.0089; 77.5° 0.6156 / 0.0354; 79° 0.8554 / 0.0716,
  bar each; creative 77.5° storm 0.616 push 0 + no-wind bar; spectator storm 0.616 push 0, no bar;
  minecarts with riders dz 0 / −3.91 / −12.95 / −14.58 at 74.5 / 76 / 77.5 / 79° S (the 0.4
  blocks/tick off-rail limit flattens the top) and +12.85 at 77.5° N, riders sent push 0.
  hardStopAt80: survival 80.5° N and 82° S and creative 80.5° S put at |z| 3640.84 (79.9989°, line
  3640.89); spectator at 81° N untouched; carts (one placed at 80.5° N, one running poleward at 0.4
  from 79.995° S) never observed past 79.9989°, riders still aboard. sableHullPushedBack (40 ticks):
  dz 0 at 74.5° S, −5.90 at 76.5° S, −20.81 at 78.5° S, +12.70 at 77.5° N; the 60 blocks/s hull
  from 79.3° N: poleward edge max 80.051° (allowed 80.088°, one tick of travel + a block), ends at
  79.60° moving back at 26.2 blocks/s. worldInfoChannelRequired: 199 channels (188 play); without
  Deep Time's two → refused naming exactly them. refuse_client over TCP: NeoForge client without
  Deep Time refused (`neoforge:modded_network_setup_failed` names deeptime:pole_storm and
  deeptime:world_info, then `multiplayer.disconnect.incompatible`), with every channel accepted
  (`neoforge:network`), vanilla refused (NeoForge's "you are not running NeoForge"). Logs:
  gitignored `review/out/p1-18/{junit,gametest,planet}.txt` and `run-planet/planet.log` in this worktree.
- **Next.** Merge; P1-19 lists the four planet tests and `refuse_client` in the Phase 1 exit. A
  real client boot (the storm visuals, the F3 line, the client-side wind) stays unverified (plan
  §9.8).

### 2026-09-28 — P1-03 plates and crust (agent worktree agent-a634f0216bff6a4a4, branch worktree-agent-a634f0216bff6a4a4)
- **What.** `crates/deeptime-core/src/plates/` (mod, state, kinematics, geometry, init, subduction,
  collision, rifting, ridges, hotspots, pools, events, normalise, noise, l1, stats, tests) — the
  plates stage of §8.1 implemented from Cortial, Peytavie, Galin & Guérin 2019 (the paper only;
  the community ports were not opened), rock pools ported from tectonics.js `RockColumn.js`/
  `Crust.js` (CC BY 4.0, commit ce371fa; NOTICE.md and the header of `pools.rs`).
  `params.rs` (`PlanetPreset`: planet, full scale, keyframes, resolution and a full `tectonics`
  section = Cortial's Appendix A + the design constants below) plugged into P1-13's preset layer
  as `PresetBody::Planet` (a preset with a `planet` section; the header keeps name, description,
  scale and timeline), with `Preset::check_runnable` (a planet preset without a timeline is
  refused by create/resume/estimate: no default story). The plate pipeline's presets are
  `presets/plates/plates_{quick,full}_{16k,32k}.json`, resolved by name like the built-ins; the
  `earthlike_*` built-ins stay P1-13's placeholders (the game bakes their synthetic terrain until
  P1-08) and take the plates sections when the stages up to P1-08 land. `WorldState.plates`
  holds the L0 crust (checkpoint schema 2 → 3); `Coupling::relink` (P1-02) lets moving crust be
  sampled through the coupling. CLI: `stage` and `bench` take `--start`/`--length-myr` like
  `run`, `stage --max-steps N` writes a mid-stage checkpoint, `run`/`stage`/`bench` print the
  plates summary line. `tools/dev/plates_sweep.py` runs tectonics overrides on the Mac for tuning.
  New ramps `plates`, `crust_age`, `regime` in `ramps.json`; `EVENTS.md` documents every plate
  event's params, magnitudes and causes. No new dependencies.
- **Model, per 2-Myr step.** Rigid rotation of each plate's groups about its Euler pole; overlap
  detection (8 nearest samples; a sample inside another plate's lattice triangle, located in that
  group's frame); contacts with a polarity (oceanic under continental, older under younger, the
  smaller plate forced under in continent–continent) and hidden subducted crust; Cortial's uplift
  ũ = u₀·f(d)·g(v)·h(z̃) within r_s = 1,800 km of the overriding plate's front (multi-source
  Dijkstra inside rigid groups), fold directions, Andean orogeny; arc magma, oxidation proxy,
  convergence/obliquity/slab age at the nearest trench and a heat-flow proxy per sample (the
  deposit accumulators); continental collision at 300 km of continental interpenetration with
  terrane transfer and Δz = Δc·A·(1−(d/r)²)², r = r_c√(v/v₀·A/A₀); hotspots and LIPs;
  continental erosion, oceanic dampening, trench accretion; rock pools (erosion one hop downhill,
  lithification 2.2 MPa, metamorphism 300 MPa, `Crust.overlap` accretion at resampling); Poisson
  rifting into 2–4 plates; global resampling every 10–60 steps (1,500 km of drift) onto a freshly
  rotated Fibonacci lattice with the hull rebuilt, Cortial's α-blend ridge crust in the gaps.
- **Design choices and deviations (all in module docs; tunable in `tectonics`).**
  1. Samples live in rigid *groups* (lattice position + the group's accumulated rotation), so rift
     fragments and transferred terranes need no remeshing between resamplings; a lattice triangle
     belongs to a plate only while its three corners share a group. Cortial reuses one mesh; here
     each resampling uses a new random rotation of the Fibonacci set and rebuilds the hull.
  2. The front is the overriding plate's own lattice boundary (fixed in its frame; no trench
     roll-back). f(d) is a piecewise cubic peaking at `subduction_peak_km` = 300 km; h(z̃) = z̃² as
     in the paper, which leaves abyssal overriding crust sinking, so an **arc band** (Gaussian at
     200 ± 60 km) adds `arc_uplift_mm_yr` = 0.1 to submarine overriding crust and arc crust that
     emerges becomes continental (island arcs, the continental crust factory).
  3. **Isostatic ceiling**: uplift of crust above sea level is scaled by (1 − z/z_c); without it
     repeated collisions saturated whole continents at z_c = 10 km (first run).
  4. **Collision**: d is measured from the contact (forced-subducted continental crust), not the
     whole terrane; Δc·A is capped at 4 km (Δc·A of a continent is tens of km); the receiving
     plate takes the area-weighted mean angular velocity. Events by terrane size: < 0.5 % of A₀
     silent, < 10 % terrane_accretion only, ≥ 10 % collision + terrane_accretion + orogeny_phase.
  5. **Polarity reversal** (continental crust on the subducting side of an oceanic overriding
     plate reverses the contact; Cortial's "an oceanic plate always plunges beneath a continental
     plate" applied continuously) and **locked sutures** (continental crust with a Himalayan
     orogeny < 30 Myr old is never forced under again). Without them a continent at a junction of
     three trenches was handed round the three plates every 2–8 Myr (309 "collisions" in the
     first 500-Myr run).
  6. **Subduction initiation** at ocean floor older than 150 Myr on plates that subduct nowhere
     (Poisson, rate 0.1 per A₀ of old crust per step): the old part splits off and converges on the
     rest. **Slab pull on speed**: a plate with ≥ 3,000 km of front relaxes (20 Myr) to 80 mm/yr at
     its centroid, others to the middle of `plate_speed_mm_yr`; plus Cortial's axis pull and a small
     Euler-pole random walk. Without these the oceans never recycled (p90 age 327 Myr).
  7. Rifting: f(x_P) = 0.08 + 0.92·x_P²; fragments diverge from the other fragments' centroid at
     20–50 mm/yr on top of the parent's motion; a LIP ×4 boosts the plate under it for 20 Myr.
  8. Resampling: a new point within 0.75 spacings of an old visible sample but in no whole
     triangle (a plate's boundary strip) copies that sample; only farther points are gaps that get
     new crust (at first every boundary strip became ridge crust and continents frayed).
  9. Events: subduction_onset only once a contact lasts 3 steps, converging ≥ 10 mm/yr over
     ≥ max(3, n/10,000) samples (dated to its start); ridge_birth once per plate pair when
     ≥ max(3, n/2,000) samples of new crust lie between them; Andean orogeny_phase only after
     ≥ 10 Myr. All places are points (region masks and paths are not stored yet).
  10. Initial planet: noise-warped weighted Voronoi plates; every continent rides whole on its
      majority plate; oceanic ages from the boundaries that diverge; ocean fraction exact to one
      sample; supercontinent/scattered/random as §5.1. A final resampling at the last step leaves
      no gaps or hidden slabs in the world P1-04 receives.
  11. L1: every cell is relinked from a k-d-tree query at each sampling (not only cells whose
      triangle left its plate), so sampling is history-free and a resumed run is bit-identical.
      L1 is sampled at the keyframe times and the end (`l1_every_steps` = 0); P1-04 sets 1.
  12. Keyframe layers `elevation` (block y), `plates` (colour index id·7 mod 20), `crust_age`
      (−1 on continents), `regime`; outputs `plates/{elevation_m, uplift_mm_yr, crust_type,
      orogeny_type, orogeny_age_myr, fold_trend_deg, plate_id}` on L1.
- **Measured** (`tools/remote/mac.sh`, M6 Mac mini, 12 threads, shared with other agents' builds,
  load average 9–14): `cargo test --workspace` → core 57 passed + 1 ignored (plates:: 11 incl.
  `rifting_conserves_area_exactly` (integer sample counts: relative error 0), `runs_keep_their_
  invariants` (partition, ages ≥ 0, finite fields, hull V − E + F = 2 + mutual neighbours + CCW
  after every resampling, all three starts, 80 steps), `resumed_run_is_identical` (byte-equal
  worlds), `a_preset_without_a_timeline_is_refused`), cli 11, format 27, inspect 19, jni 7,
  viewer 4; `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`,
  `cargo deny check licenses bans` clean. `cargo test --release -p deeptime-core
  plates::tests::statistics -- --ignored` (20k samples, 500 Myr) → supercontinent p90 167 Myr,
  98 % < 250 Myr, 15 plates (8 ≥ 5 %); scattered 169 Myr, 97 %, 21 plates (7 ≥ 5 %); random
  245 Myr, 90.5 %, 8 plates (4 ≥ 5 %).
  `deeptime stage plates --preset plates_quick_16k --start scattered_continents --length-myr 500` (100k)
  → `plates_final=13 ocean_age_p90_myr=202.3 ocean_age_p50_myr=69.7 ocean_younger_250=0.925
  ocean_frac=0.740 largest_plate_frac=0.179 plates_over_5pct=8 plates_under_1pct=0 … steps=250
  seconds=8.315`; 610 events (orogeny_phase 149, subduction_onset 142, ridge_birth 134,
  terrane_accretion 96, collision 26, hotspot_track 21, plate_death 15, plate_birth 14, LIP 7,
  rift_onset 6), 11 keyframes, peak 208 MiB. `plates_full_16k` (250k) → `plates_final=16
  ocean_age_p90_myr=122.0 ocean_younger_250=0.999 ocean_frac=0.692 … seconds=24.840`, 1,218 events.
  `deeptime bench --preset plates_quick_16k --start scattered_continents --length-myr 500
  --threads 1,4,12` → 23.0 / 10.5 / 8.3 s (load average 5; 29.8 / 15.7 / 9.3 s at load 10–14),
  peak 208 MiB; `plates_full_16k --threads 1,12` → 72.2 / 30.7 s, peak 538 MiB; identical
  results on every thread count. **Against Cortial's Table 2 estimate (§8.1): 100k 76–106 s →
  8.3 s (23 s single-threaded); 250k 330–437 s → 25–31 s (72 s single-threaded).** Checkpoint chain: `stage plates … --max-steps 100 --out mid.dtcp` (33.6 MB) then
  `stage plates --in mid.dtcp --out end.dtcp --viewer review/out/p1-plates-resumed` → the same
  final summary as the straight run; running it again from `end.dtcp` is refused ("already ran").
  Sweep (`tools/dev/plates_sweep.py`, 100k, 3 starts × 2 seeds): p90 141–228 Myr, 92.5–99.4 %
  younger than 250 Myr, ocean fraction 0.73–0.76, 10–22 plates.
  `deeptime run --preset plates_quick_16k --start scattered_continents --length-myr 500 --out …`
  → manifest, events/, keyframes/ (11 frames × 4 layers), sites/, bodies/; `stage plates --preset
  plates_quick_16k` without `--start` → "$.timeline.start: choose a start configuration …".
  All re-run on the tree after merging master a3a0e30 (P1-12/13/14): identical summaries;
  `cargo test --workspace` → cli 9, core 116 + 1 ignored (plates::/params::/preset:: 20),
  format 31, inspect 19, jni 7, viewer 4; clippy -D warnings, fmt and `cargo deny check licenses
  bans` clean; `cd neoforge && ./gradlew build test` → tests=57 failures=0 errors=0.
- **What the keyframes show** (review/out/p1-plates, pulled back and looked at): at −498 Myr six
  low green continents in a deep ocean with trench lines along plate boundaries in the oceans;
  by −348 Myr the continents have drifted and one has swung over the north pole, their margins
  raised to 1–2 km (Andean belts) and green island arcs trail across the eastern ocean; by
  −148 Myr a continent is splitting along a narrow new ocean (a rift opening like the Red Sea),
  arcs and accreted terranes fringe the others; at 0 Myr the continents are rearranged and
  scattered south and east with raised margins, trenches along active margins, and the crust-age
  map shows spreading ridges with symmetric red→blue age bands and a few old (blue) basins —
  plausible drifting, colliding and rifting. The plates map shows 13 plates with clean, slightly
  irregular boundaries (plates whose ids differ by 20 share a colour).
- **Not done / later.** Region masks and paths for events (points only). The review page
  "Deep Time — P1 plates" is not published (the main session publishes review/out/p1-plates).
  Continental area is resolution-dependent (ocean fraction 0.74 at 100k, 0.69 at 250k, 0.82–0.85
  at 20k: arc emergence and forced-subduction consumption scale with sample spacing) and the
  event count grows with resolution (610 → 1,218 events from 100k to 250k); both for P1-17's
  tuning. No Java change: JUnit passes on the merged tree (`{"planet":{}}` still fails at
  `$.planet`, as NativeLinkTest expects); GameTests not rerun (no server-side change).
- **P1-14's field interface.** The deposit engine's `FieldHistory` (`deposits/fields.rs`) expects
  per-frame L1 fields from P1-03 (`deeptime:arc_magma_flux`, `magma_oxidation`,
  `convergence_rate_km_myr`, `obliquity_deg`, `slab_age_myr`, `crust_thickness_km`,
  `fault_density`, `orogenic_uplift_m_myr`, `rift_subsidence_m_myr`, `heat_flow_mw_m2`,
  `dist_litho_step_km`, `inherited_bif`). The L0 accumulators behind most of them exist per sample
  (arc magma by type, oxidation, convergence, obliquity, slab age, heat flow, thickness, uplift);
  publishing them as frames is not done here (P1-14 landed while this task ran).
- **Next.** P1-04 (surface processes on L1) reads `plates/uplift_mm_yr` and the other `plates/*`
  fields; its pipeline should set `tectonics.l1_every_steps` = 1. Then the frames P1-14 reads.

### 2026-09-28 — P1-15 deposits in the world, on the synthetic deposit world (agent worktree agent-a76188b70a388c5da, sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Scope.** Everything of P1-15 that does not need P1-03…P1-08, driven by P1-14's synthetic run.
  The row stays `doing` until the real presets feed it (rerun the acceptance below unchanged).
- **A world with terrain and deposits (Rust).** A `deposits_synthetic` preset may carry
  `"terrain": {"strata": "placeholder" | "every-class"}`: `finalize_write` then bakes, next to the
  sites, bodies and events, a terrain whose elevation is the synthetic history's present elevation
  (`deposits::synthetic::HistoryTerrain`, `synth::build_with`, kind `deposit-history`), so every body
  sits in the ground it was placed against. For that the deposit stage now samples the present
  surface **bilinearly on L1 at the site's point** (`Finish::surface_at`, `bilinear_cells`) instead
  of the nearest cell; P1-14's tests and its 16k census (32 sites) are unchanged. Presets
  `crates/deeptime-core/tests/presets/deposits_world_{4k,16k}.json` (GameTests, smoke). Truncated ore
  bodies' clips lie within 3 blocks of the baked ground (`world_preset_bakes_terrain_under_the_bodies`).
- **Spawn and essentials (Rust + Java).** `Essential` gains `max_blocks_from_spawn`. New
  `deposits::spawn`: with such essentials the bake ranks spawn candidates (dry, flat, 16-block
  lattice; fewest essentials out of reach, then Σ min(d/max, 4), then distance from 0,0; top 10,
  64 blocks apart) and writes `ranked_by`, `unmet`, `score`, `essentials` into
  `spawn/candidates.json` (FORMAT.md). `SpawnHandler` takes the dry candidate with the fewest
  `unmet`. Java registries `deeptime:essentials`, `deeptime:scatter` and `deeptime:deposit_type`
  (lenient codec: products and the ore-block policy only) in `ore/OreData`; Deep Time ships vanilla
  essentials (iron, coal 1 per 4,096-block region, exposed, spawn ≤ 2,048; copper 1 per 8,192,
  spawn ≤ 3,072; gold 1 per 8,192) and scatter rules (coal, iron, copper, gold, zinc; values
  judged, marked initial). **Content bundle:** `FirstStartGenerator` passes
  `{"essentials": [...]}` from the registry to the native run; `Preset::with_content` replaces the
  preset's essentials with it (the recorded preset.json carries the list used).
- **Java world data.** `worlddata/{Sectioned, SiteData, BodyData, Body, EventData, Deposits}`: readers
  of `DTSR`/`DTSI`/`DTBD`/`DTEV` (CRCs, versions, bounds checked as in Rust), `Body.locate` = Rust's
  `Body::locate` operation for operation, `WorldData.deposits()` lazily. **Parity:**
  `DepositDataParityTest` over `deeptime run deposits_world_4k.json` + new `deeptime dump-deposits`
  (Gradle `testWorldDeposits4k`): every site field, 532 index lookups, 49 bodies × 401 samples
  (a quarter across the seam) — 19,649 samples, 1,388 inside, worst |Δd|,|Δv| 3.3e-16, 0
  inside/outside disagreements — and every forming event's cause chain.
- **API** `api/{DeepTimeSites, Site, SiteKind, Product, Extraction, GeoEvent}` (plan §5.6) via
  `worldgen/LevelSites`: `of(ServerLevel)` (empty outside Deep Time levels), `all`, `at` (body SDF),
  `inChunk`, `within`, `byId`, `formingEvent`, `causeChain`, `event`.
- **Strata pass:** bodies now paint their `rock_class` (a zone's winning, last zone last) over the
  crust or horizon rock inside their SDF, one member per body (`StrataPass.bodyKey`);
  `StrataPass.predictedRock` for tests. `RockPalette` gains `host(state)` (ROCK = crust or any
  consolidated class's block, LOOSE = the `unconsolidated` family, NONE), `classesOf`, `isLoose`.
- **Deposit feature** (`worldgen/DepositFeature`, `deeptime:deposits`) and **background scatter**
  (`worldgen/BackgroundScatterFeature`), both at `underground_ores` through one
  `neoforge:add_features` modifier (`biome_modifier/deposits.json`, deposits first). Per block one
  draw, a SplitMix hash of the canonical position and the body id, against the running sum of the
  densities of the zones with a material that apply there (zone order; FORMAT.md "Bodies"); host =
  ROCK, or LOOSE only in a body whose own class is loose (placers in gravel). Scatter: ⌊n⌋ + chance
  clusters per chunk, start depth from `y_window` as a fraction of the column from the world data's
  surface to min_y (the plan's "fraction of crust thickness"), a `size`-step random walk inside the
  chunk, ROCK hosts only, `host_bias` per class. Both write only their own chunk; both no-ops
  outside Deep Time generators.
- **OreBlockResolver** (`ore/`): EV variant for the host, else `#c:ores/<m>` ∩
  `#c:ores_in_ground/{deepslate below 0, stone}`, else the first `#c:ores/<m>` block; cached per
  (material, host block, below 0), rebuilt on tag reload. **EV** (`compat/ev/EvBridge` +
  `EvBridgeImpl`, compileOnly `maven.modrinth:excavated_variants:krUiU6UD` — the slug has an
  underscore): EV 4.3.1 registers its `ore`/`stone` registries in `BuiltInRegistries.REGISTRY`, and
  after registration `Ore.getStones()` maps each stone to the ore's block in it (originals and
  generated variants), so no id arithmetic. EV already ships stone configs for vanilla's and
  Create's stones; Deep Time ships only the four vanilla stand-ins EV lacks (polished tuff, packed
  mud, smooth sandstone, cobblestone) through DefaultResources (`defaultresources.meta.json` +
  `defaultresources/globaldata/deeptime/excavated_variants/stone/`; verified loaded under -PwithEv).
  **Finding: EV's own `ore_replacer` must not run in Deep Time levels** — added to every biome's
  `TOP_LAYER_MODIFICATION` in `AFTER_EVERYTHING`, it re-picks ore stones from neighbours with
  chunk-edge bookkeeping, and a regenerated deposit differed in 200 of 1,406 ore blocks. Compat mixin
  `mixin/compat/excavated_variants/OreReplacerMixin` (@Pseudo, applied only with EV) skips it in
  Deep Time levels; other levels and EV's config are untouched. Consequence: unknown ores in Deep
  Time levels stay their stone/deepslate blocks in granite/diorite/andesite/tuff hosts (follow-up:
  a deterministic EV conversion for unknown ores against `predictedRock`).
- **Ore claiming** (`ore/ClaimOresBiomeModifier`, `ClaimRules`; `biome_modifier/claim_ores.json`,
  `#minecraft:is_overworld`): REMOVE phase, every decoration step, only when
  `ServerLifecycleHooks.getCurrentServer()`'s overworld stem is a `GeologicChunkGenerator`. Claimed
  materials = deposit-type products + scatter materials (+ `materials`): c:coal, copper, crude_oil,
  gold, iron, lead, molybdenum, silver, zinc. Only **ore-shaped** feature types are judged
  (`minecraft:ore`, `scattered_ore`, `replace_single_block`, `create:layered_ore`,
  `lithostitched:ore`, + `ore_feature_types`); selectors are followed; the blocks placed are read by
  encoding the configured feature with its codec and collecting `{"Name": …}` states outside
  rule-test keys (works for Create's layered ore without Create on the classpath). ORE = places a
  `#c:ores/<claimed>` block; ROCK (with `claim_rock_features`, default on, **Q4 pending**) = every
  placed block is ROCK and nothing else is in the feature (+ `rock_features` list, which names
  `create:striated_ores_overworld`); loose pockets (dirt, gravel, clay) stay; `keep` list. A first
  version judged every feature and removed Terralith's floating islands, boulders, pillars, ponds
  and lava removers — hence the ore-shaped rule. `/deeptime probe claims` prints the dry run and
  what was removed.
- **Probes** (`server/SiteCommands`): `probe site <x> <y> <z>` (site, bodies, zones, d, v, the draw's
  material, the block), `sites near <x> <z> <radius>`, `why <siteId>` (site, forming event, 4-link
  chain; readable lines too), `probe claims`. `tools/smoke/run.py --rcon <cmd>` records probe lines
  in the report and `rcon.jsonl`; its generation check accepts a generation shorter than the
  progress gap, and its mixin check skips compat mixins the plugin skipped.
- **GameTests** (new level `deeptime:test_deposits` + `test_deposits_twin` over
  `run-gametest/world/deeptime-deposits`, C = 4,096, fixed biome `deeptime:test_ores` whose only
  features are the two above; gametest.sh writes it with `deeptime run`): `DepositGameTests.
  {bodyHasOre, vanillaOreBlock, evVariantWhenInstalled, depositsDeterministic, siteApiAndProbes,
  backgroundScatterPlaces}`, `ClaimGameTests.{claimOffWhenOverworldNotGeologic,
  claimRulesSortFeatures}` (the GameTest overworld is not Deep Time's, so claiming is off there and
  the rules are checked as a dry run), `SpawnGameTests.essentialsWithinReach`, and on the planet
  server `PlanetClaimGameTests.claimOnInPlanetWorld`. The site is picked from the registry (first
  deposit with products whose ore body is ≤ 96 blocks across and due ≥ 200 ore blocks, porphyry
  first): site 2, porphyry Cu-Mo-Au, stockwork-shell body 3, AABB (−1330, 117, 175)..(−1298, 156, 208).
- **Measured (Mac).** `cargo test --workspace` ok (core 102, format 31, cli 9, inspect 19 + 7, jni 7,
  viewer 4), clippy `-D warnings`, fmt, `deny check licenses bans` clean; `gradlew build test` →
  tests=62 failures=0. `mac.sh --server -- ./gametest.sh` → **All 30 required tests passed**:
  body 3 due 1,704, placed 1,222 = exact 1,222 (`minecraft:copper_ore`), missed 0, not host 482
  (air 365, dirt 84, grass 33: the clipped top and the soil), 184 other ores in the box all explained
  (178 another body's, 6 scatter), unexplained 0, 12 chunks; regeneration 44,880 blocks,
  1,406 ore, 0 differ; scatter 16 chunks, 539 planned, 404 placed, 0 wrong; spawn (−1184, 76, −462),
  iron 52 / coal 52 / copper 44 blocks (reach 1,024/1,536/1,024); why: deposit_formation ←
  subduction_onset, orogeny_phase. Claim dry run: 12 ORE (vanilla, incl. nether gold) and 7 ROCK of
  235 placed features; plains untouched (0 removed). `./gametest.sh -PwithEv` (TW + EV 4.3.1 + DAG
  6.1.2) → **All 30 passed**: the same 1,222 blocks as EV variants (1,115 copper-in-red-sandstone,
  107 copper-in-sandstone), 171 EV ores / 111 stones / 110 variants, Deep Time's four stones load.
  `./gametest.sh -PwithMic` (pack 303fd89 via `install.sh mic`, the rock tags as `deeptime_extra`)
  → **All 30 passed**: body 3 adds 110 `iceandfire:silver_ore`; claim dry run over 932 placed
  features: 16 ORE (vanilla, `create:zinc_ore`, `createnuclear:lead_ore`, `iceandfire:silver_ore`,
  `terralith:cave/fungal/ore_coal_extra`), 24 ROCK (vanilla blobs, `create:striated_ores_overworld`,
  `born_in_chaos_v1:black_argillite`, 16 Terralith rock/cliff features incl. `ds_reducer`,
  `noise_reducer`, `granitifier`, `calcitifier`, `desert/sandstone`), kept: uranium, platinum,
  nickel, fluorite, wolframite, sapphire, Cobblemon, `createnuclear:striated_ores_overworld`
  (rock + uranium), Terralith's floating islands, boulders and pillars. `./gametest-planet.sh --run
  claimoninplanetworld@0,100,0` → PASS (standalone set: 19 removed, plains 48 → 33 features) and with
  `-- -PwithMic` → PASS (38 removed; uranium, diamond, lapis, redstone, dirt kept).
- **Smoke and ore render.** `install.sh standalone --preset $PWD/crates/deeptime-core/tests/presets/
  deposits_world_16k.json --length-myr 500`, then `mac.sh --server -- 'python3 tools/smoke/run.py
  --profile standalone --fresh --chunky 112 --chunky-center -5570,-1030 --rcon "deeptime sites near
  -5570 -1030 600" --rcon "deeptime why 2" --rcon "deeptime probe site -5563 130 -1002" --rcon
  "deeptime probe claims"'` → **PASS**: first start ran the deposit engine through JNI with the
  registry's essentials (content bundle logged) in 2.2 s — 42 sites (32 with the preset's own
  essentials in the CLI run; the natural sites are identical), spawn (−16, 69, −2142) with coal and
  iron 2 blocks and copper 669 blocks away; claim removed 19 features; all four probes answered.
  `cargo run -p deeptime-inspect -- ore --world smoke/standalone/server/planet
  --area=-5682,-1142,-5458,-918 --scale 3` → `review/out/p1-15/ore-standalone/ore.png` (the ore
  render now outlines site footprints and counts ore columns inside/outside them; it also knows
  EV's variant names). What it shows: the placer's gold streak inside its box, the supergene
  blanket's copper cluster inside the porphyry's box, and outside the boxes only small scatter
  clusters (copper 625 columns outside vs 329 inside the footprints, iron 873/197, coal 680/165,
  zinc 134/42, gold 82/139) and the unclaimed vanilla ores (diamond, lapis, redstone ~8,100 blocks,
  kept by design). Porphyry site 10's box is empty: the every-class strata put soil, till and
  laterite (loose) at y 102–141 there, and loose rock hosts ore only in loose bodies.
- **Deviations and findings.** (1) Driven by the synthetic run, not `earthlike_quick_16k`. (2) The
  GameTest server cannot turn claiming on (its overworld is not Deep Time's), so claiming is a dry
  run there plus the planet-server test (the plan already put `claimOnInPlanetWorld` there).
  (3) Deposit tests use a fixed test biome so vanilla features cannot interfere. (4) EV's ore
  replacer is disabled in Deep Time levels (above); plan §7.6 updated. (5) P1-14 engine quirks seen
  in the renders, for its follow-up: the guarantee pass stacks guaranteed deposits of different
  materials on one cell (coal and BIF both at (−16, −2144), 2 blocks from the chosen spawn); several
  porphyries have no products left (leached into their blankets) and are still bodies; a porphyry
  ore body has no rock class of its own, so in loose horizons it places nothing (P1-07 could give
  intrusive types their intrusion class). (6) Not done: the BlueMap cut through a porphyry and the
  side-by-side EV renders for Ben (Q10); a spacesheep page; size-multiplier tuning on renders (the
  bodies look plausible at 20–60 blocks; left at P1-14's values); unknown-ore EV conversion; MIC's
  own essentials and scatter lists (pack branch).
- **Next step.** P1-16 (COE/CDG) can use `DeepTimeSites` directly. After P1-08, rerun: `./gametest.sh`,
  `-PwithEv`, `-PwithMic`, `./gametest-planet.sh --run claimoninplanetworld@0,100,0 [-- -PwithMic]`
  and the smoke/ore render on the real preset; answer Q4 and Q10.

### 2026-09-28 — P1-12 strata pass with rock tags and `rock_states` (agent worktree agent-a9583db34fe66882c)
- **Datapack registries** (`DeepTimeRegistries`, NeoForge `DataPackRegistryEvent.NewRegistry`, no
  network codec: clients never need them): `deeptime:rock_class` (`rock/RockClass`, codec per §5.4.1
  plus `sources` and `notes`; `vanilla_fallback` optional so third-party classes need none) and
  `deeptime:rock_states` (`rock/RockStates`, the adopted format: `class`, `use` member|fallback,
  `entries` of `block`, `state` (strings), `required` (default true)). `RockClassRegistry` /
  `RockStatesRegistry` hold the keys and list entries sorted by id. `deposit_type`, `sim_field` and
  `essentials` are not registered yet (P1-14, Phase 3, P1-15).
- **The 59 classes** ship in `neoforge/src/main/resources/data/deeptime/deeptime/rock_class/`, generated
  by `tools/dev/gen_rock_classes.py` (one table, validated: 59 classes, known names in paths, no
  filler except coal's stone, vanilla-only fallbacks, no terracotta). Values are initial and marked
  "tuned in P1-07": 1.0 = crystalline basement (granite, gneiss); erodibility 0.3 (chert, quartzite)
  … 4.0 (halite) for bedrock, 4–10 for unconsolidated (Sklar & Dietrich 2001 strength scaling,
  compressed by a square root; Stock & Montgomery 1999); diffusivity 0.5–5.0 (Roering et al. 1999 D
  used as a linear multiplier, Heimsath et al. 1997); density 700 (pumice) – 3400 (BIF, eclogite)
  kg/m³. `vanilla_fallback` = the README's vanilla member or last fallback entry. Judgment calls in
  the table: loose sediments carry their lithified equivalent's `metamorphic_path` (the schema has
  no lithification field; P1-07 needs its own rule), `facies_rule` names beyond the plan's example
  are placeholders for P1-07, `karst`/`ore_host` tags added. Deep Time still ships **no fallback
  tags** (Q15; `RockDataTest.noFallbackTagsShipped`).
- **`RockPalette`** (plan §7.5): the rules live in plain Java (`rock/PaletteRules`, JUnit-tested):
  members = `#deeptime:rock/<c>` in tag order, then `rock_states` member entries (absent
  `required: false` blocks skipped); else the first of `#deeptime:rock_fallback/<c>` then
  `rock_states` fallback entries; else `vanilla_fallback`; else unclassified crust. Air, stone and
  deepslate are rejected everywhere except coal → stone (reported as problems). Several members: one
  per horizon, `members[mix(key) mod n]` (SplitMix64 finaliser), primary = tag order's first. The
  palette is an immutable snapshot rebuilt on `TagsUpdatedEvent` with cause `SERVER_DATA_LOAD`
  (server start after tags bind, `/reload`); client tag syncs are ignored. `RockPalette.View`
  also builds the "vanilla only" palette (only `minecraft:` members, no fallback tags, no
  rock_states) the tests compare against. **Deviation:** a first version also rejected blocks that do
  not block motion (worldgen heightmaps); vanilla snow and Project Atmosphere's dust are
  `forceSolidOff`, so it rejected the adopted `rock_states` example; dropped (only air is rejected).
- **`StrataPass` v1:** per column the horizon tops are bilinear between the four strata cells (the
  nearest cell's top where a corner lacks the horizon), classes from the nearest; integer tops T_h,
  horizon h fills T_{h+1} < y ≤ T_h (v0's boundary rule differed from `classAt` at integer tops;
  both now share `StrataPass.column`). Runs are computed per column, then written section by section
  straight into the section palettes, skipping sections without stone or deepslate. Horizon key =
  `(h << 8) | classIndex` until world data carries unit ids (P1-08). **Bodies are not done:** the
  `bodies/` format arrives with P1-08/P1-14; the pass has no body hook yet.
- **`/deeptime probe palette`**: one `[deeptime-probe]` JSON line: `count`, `resolved`, `sources`
  (per-source counts), `classes` (class → block states, primary first), `source`, `state`,
  `unresolved`, `problems`. `deeptime-inspect rock --palette` now reads it (block states cut to
  their id; a raw log line works). `tools/smoke/run.py` writes it to `smoke/<profile>/palette.json`.
- **`-Ddeeptime.extraDatapack=<folder>`** (gap 29, plan §7.12 fixed): `GameTestSupport` adds the
  folder through `addRepositorySource` + `PathPackResources.PathResourcesSupplier` as the required
  top pack `deeptime_extra`, only with `-Ddeeptime.gametest=true`. Gradle sets it from
  `-PextraDatapack=<folder>`; `-PwithMic` defaults it to `<repo>/../../Docs/datapacks/deeptime-mic-rock-tags`
  (warns and goes without if absent). `tools/remote/mac.sh --pack` now also syncs that datapack to
  `/Volumes/Graphide/ban/mic/Docs/datapacks/`. The MIC datapack and `pack/` were not modified.
- **Synth:** `deeptime synth --strata every-class` (default `placeholder`): 8 horizons 10 blocks
  apart, horizon h of L1 cell column i has class (6·i + h) mod 59, so any ten neighbouring cell
  columns hold every class ≥ 20 blocks down; `gametest.sh` writes `deeptime-test` with it.
  `install.sh` gains `--strata` and `--datapack <folder>` (copied into the world's `datapacks/`).
- **Tests.** JUnit `PaletteRulesTest` (9) and `RockDataTest` (3); cargo `synth` +3
  (`every_class_matches_the_rock_class_data` keeps Rust's list equal to the JSON files).
  GameTests `StrataGameTests.{vanillaOnlyResolvesEveryClass, coalFallsBackToStone,
  firstInstalledFallback, rockStatesMember, createStonesKeepTheirClasses, everyClassInTheWorld,
  probePaletteListsEveryClass}`; `WorldDataGameTests.strataClassesResolve` now picks the member by
  horizon key. The gametest pack adds three test classes (`deeptime_test:test_fallback`,
  `test_state_fallback`, `test_states`), a fallback tag (absent optional block, stone, polished
  andesite, andesite) and the README's `rock_states` example (`projectatmosphere:dust[layers=8]` as
  loess, `required: false`).
- **Measured (Mac).** `mac.sh -- 'cargo test --workspace'` (cli 13, core 41, format 27, inspect 19+7,
  jni 7, viewer 4), `clippy --workspace --all-targets -D warnings`, `fmt --check`, `deny check licenses
  bans` clean; `gradlew build test` → tests=46 failures=0. `mac.sh --pack --server -- './gametest.sh'`
  → **All 21 required tests passed** (vanillaOnlyResolvesEveryClass sources member=16
  vanilla_fallback=43; coal=minecraft:stone; everyClassInTheWorld 59 of 59 classes, 240 samples, 0
  wrong; probe palette 62 classes, 62 resolved: member 17, fallback 2, vanilla_fallback 43).
  `mac.sh --pack --server -- './gametest.sh -PwithMic'` (160 mods, MIC pack 303fd89 via
  `install.sh mic`, the rock tags as `deeptime_extra`) → **All 21 required tests passed**: coal =
  born_in_chaos_v1:black_argillite; limestone/dolostone/ironstone/BIF/serpentinite/scoria =
  create:limestone/asurine/ochrum/crimsite/veridium/scoria; shale = black argillite, peridotite =
  create:veridium (first installed fallbacks); loess = projectatmosphere:dust[layers=8];
  everyClassInTheWorld 240 samples 0 wrong; probe palette 62 of 62 resolved: member 26, fallback 36,
  vanilla_fallback 0.
- **Smoke and rock render (Mac, production MIC server, pack 303fd89).** `install.sh mic --synth
  wrap-test --strata every-class --datapack ../../Docs/datapacks/deeptime-mic-rock-tags` then
  `mac.sh --server -- 'python3 tools/smoke/run.py --profile mic --fresh --chunky 96'` → PASS; the
  probe palette (no gametest pack here) resolves **59 of 59: member 24, fallback 35** — exactly the
  README's 24 classes with a real MIC block and 35 stand-in classes. `cargo run -p deeptime-inspect --
  rock --world smoke/mic/server/planet --palette smoke/mic/palette.json --colors <colors table>` →
  `review/out/p1-12/mic-rock/rock-0.png` (55,808 columns, 16-block bands of horizon-0 classes with
  Create's veridium/asurine/ochrum/crimsite among them). A block → class map is ambiguous where
  stand-ins share a block (calcite is marble, chalk, halite, …; the render names the alphabetically
  first class).
- **After merging master c7af155 (P0-06, P0-14, P0-15)** into the branch (conflicts: the probe
  builder in `DeepTimeCommands`, this log): cargo test/clippy/fmt/deny clean, JUnit 46/0,
  `./gametest.sh` → All 21 required tests passed, `./gametest.sh -PwithMic` → All 21 required tests
  passed (same palette numbers as above). **After merging master 3b05d00 (P1-13)**, which moved the
  synthetic worlds into `deeptime_core::synth`: the strata option moved with them (`Strata`,
  `EVERY_CLASS`, `build(kind, strata, c, codec)`; the session's placeholder presets bake
  `Strata::Placeholder`, so P1-13's worlds are unchanged). cargo test (core 59, cli 7, …), clippy,
  fmt, deny clean; JUnit 57/0; `./gametest.sh` → All 21 required tests passed; `./gametest.sh
  -PwithMic` → All 21 required tests passed (and one stalled run, below).
- **Known issue: `-PwithMic` `lakeHolds` hangs, then runs out of heap (flaky).** Of seven
  `-PwithMic` runs of this branch, four passed (All 21) and three stalled in `lakeHolds` (every other
  test done; it forceloads 240 test_lake chunks synchronously), then died of `OutOfMemoryError: Java
  heap space` (6 GB) 20 s to 5 min later; the JVM then ignores SIGTERM and needs `kill -9`, holding
  the server lock. No Deep Time frame in any stack; C2ME's workers report the OOM. One control run
  of master 3b05d00 (14 tests) passed. A first guess (TaCZ/Mega Showdown's unpacked packs in
  `run-gametest/`) was wrong: a run with both folders deleted also failed. The stall (a synchronous
  chunk load under C2ME that never returns, with memory growing while it waits) is the thing to chase;
  a thread dump (`jstack`) during a stall would show it. Until then, rerun a `-PwithMic` gate that
  stalls in `lakeHolds`.
  **2026-09-29 (P1-09):** now 4 of 4 `-PwithMic` runs on the tree merged with master (8 GB heap): every
  other test finished and passed, then `lakeHolds` stalled and the JVM died of heap (it exited on its
  own within ~10 min each time). `-PwithMic` had not run on master since P1-08 added the real-planet and
  deposit test levels; no master control run was made.
  **Resolved 2026-09-29** (build log "MIC GameTest heap stall"): not a C2ME hang and not `lakeHolds`'
  own chunks. The suite's scattered column samples held ~52,000+ chunks at once (each far sample pulls
  289 proto-chunks for structure starts, and nothing unloads inside a tick); `lakeHolds` was the one test
  still waiting when the heap ran out, and passes alone. The samples now run a few per tick; `-PwithMic`
  passed 5 of 5.
- **Not done / next.** Bodies in the strata pass (needs P1-08/P1-14's `bodies/`); the review
  artefacts (BlueMap cliffs with Create stones) wait for Ben's review round; `deposit_type`,
  `sim_field`, `essentials` registries come with their tasks. The MIC pack still has no copy of the
  rock tags (Q16 answer: `pack/kubejs/data/deeptime/` on a pack branch, not done here).

### 2026-09-28 — P1-14, first pass: the deposit engine on synthetic fields (sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Scope.** Everything that does not need P1-03…P1-07: the data, the schema, the engine behind a
  named-field interface, a synthetic field history, the CLI path and the viewer. The row stays
  `doing` until the real history feeds it.
- **Data** (`crates/deeptime-core/data/usgs/*.json`, public-domain USGS, digitised by two Opus
  sub-agents with the report, table or figure, page and method of every number; placeholders say
  so). Porphyry Cu: OFR 2008-1155 Table 1 (PDF p. 9), n 422, 33 / 240 / 1,500 Mt, Cu 0.24 / 0.44 /
  **0.75** % (the research note's 0.80 was a figure tick), Mo, Au, Ag with reporting fractions,
  r(tonnage, Au) −0.28 and three more (text, PDF p. 8), 2 km rule (p. 4), density 4 / 15 / 35 per
  10⁵ km² (Singer, Berger & Menzie 2005, USGS record 70197200). MVT and SEDEX: OFR 2009-1252
  (Singer, Berger & Moring 2009) has **no MVT or SEDEX model**; its CAam (carbonate-hosted
  amagmatic, n 132, 93 of them MVT) and SHam (shale-hosted amagmatic, n 25, all SEDEX) models are
  used (Table 2 pp. 9–10), MVT-only SIR 2010-5070-A Table 1 and Bulletin 1693 31a/32a-b in `extra`.
  BIF: Bulletin 1693 model 34a (covers Algoma too), 11 / 170 / 2,400 Mt, Fe 30 / 53 / 66 %.
  Placer Au: Bulletin 1693 model 39a, 0.022 / 1.1 / 50 Mt, Au 0.084 / 0.2 / 0.48 g/t, 2.0 t/m³,
  1.6 km rule. Supergene: SIR 2010-5070-B facts (≤3× Andes, 0.4–1.7 % Cu, ≤200 m thick). Coal:
  Circular 891 densities and thickness limits (no USGS grade–tonnage model for coal exists; the
  4.2 / 13 / 25 Mt block is a placeholder). Petroleum: Waples 1980 TTI 15 / 75 / 160 (journal
  fact), field sizes from the DDS-60 Niger Delta assessment unit (2.4 / 12 / 89 MMBO, p90 and p10
  read off the histogram). **Not found anywhere (placeholders):** deposit densities per area for
  MVT, SEDEX, BIF, coal and oil; ore densities for porphyry, MVT, SEDEX, BIF; separations for BIF,
  coal, oil; a USGS table of reflectance by coal rank.
- **Deposit types** (datapack JSON, `neoforge/src/main/resources/data/deeptime/deeptime/deposit_type/`,
  compiled into the core): `porphyry_cu_mo_au` (children `supergene_cu_blanket`, `placer_au`),
  `bif_fe`, `coal_seam`, `mvt_zn_pb`, `sedex_zn_pb`, `petroleum_trap`. Schema
  `crates/deeptime-format/schema/deposit_type.schema.json` (a test compares its properties with the
  parser, which refuses unknown keys; another checks every non-placeholder number against its USGS
  table). Create stones as preferred hosts: serpentinite (veridium) 1.5 for copper, dolostone
  (asurine) 1.5 for MVT and SEDEX, banded_iron_formation (crimsite) 2.0 and the BIF sheet's own
  class, the porphyry gossan is ironstone (ochrum) and carries gold. `minecraft`: COE vein ids
  `createoreexcavation:ore_vein_type/{copper,gold,iron,coal,zinc}` (read from the COE 1.6.8 jar),
  `cdg_reservoir` for oil, `ore_block_policy` `ev_then_vanilla` (`none` for oil).
- **Engine** (`crates/deeptime-core/src/deposits/`, stage `deposits`): `fields` (the interface),
  `types`, `usgs`, `rules` (membership curves, product/min/geometric mean, host and clock scores,
  time integration F = 1 − Π(1 − F_t)^(Δt/τ), limiting component), `tracts`, `count` (Poisson,
  Voronoi provinces), `cluster` (Thomas positions, separation max(USGS rule, r_i + r_j)), `sizing`
  (split lognormal + Gaussian copula), `geometry`, `ledger`, `children` (supergene, placers),
  `petroleum` (Lopatin TTI, kitchens, charge in order), `playability`, `sites` (cause lookup),
  `synthetic`, `random` (xoshiro256++, PTRS Poisson). Pass 1 = one frame per step; the last step
  places, replays the frames at the chosen cells (pass 2: ledger, supergene windows) and writes
  sites, bodies and events. New `bodies/bodies.dtb` (`DTBD`, FORMAT.md "Bodies": 11 shape names on
  two metrics, zones by normalised distance and vertical position, erosion clip), written by
  `finalize_write`; `WorldState` gains `bodies` and `notes` (the census goes to `manifest.json`
  notes), checkpoint schema 2. Preset `{"deposits_synthetic": {…}}` (`session::Preset`).
- **CLI and viewer.** `deeptime stage deposits --preset <file> --viewer <dir>` (and `run`) print the
  census per type next to ρ·A·F; the viewer draws sites coloured by type on the keyframes (layers
  `deposits/favourability/<type>`, `deposits/sites`, `input/*`), a census table, and per site the
  component scores, weakest component, host rock, fate, the "Why here?" chain and a plan/section of
  its bodies (`report.js`; `index.html?site=<id>` opens a site).
- **Measured (Mac, `tools/remote/mac.sh`).** `cargo test --workspace` ok (core 82 of which 41
  `deposits::`, format 31, cli 11, jni 7, viewer 4, inspect 26); `cargo fmt --all -- -l` clean,
  `cargo clippy --workspace --all-targets -- -D warnings` clean, `cargo deny check licenses bans` ok.
  `cargo run --release -p deeptime-cli -- stage deposits --preset
  crates/deeptime-core/tests/presets/deposits_synthetic_16k.json --viewer review/out/p1-deposits`
  (C 16,384, L1 1024 × 512, 50 frames of 10 Myr, strong provinces, count_mult 1): 52 steps, 32
  sites, 38 bodies, 110 events, 19.6 s wall and 0.83 GB peak footprint on the shared Mac.

  | type | ρ·A·F (§8.6) | drawn | sites | notes |
  |---|---|---|---|---|
  | porphyry_cu_mo_au | 9.41 | 9 | 6 | 1 buried, 4 truncated, 4 eroded; 1 guaranteed |
  | supergene_cu_blanket | — | — | 4 | over truncated porphyries in the dry arc |
  | placer_au | — | — | 2 | below eroded porphyries |
  | bif_fe | 2.50 | 4 | 5 | inherited, dated by the 2.2 Ga shelf-sea event; 1 guaranteed |
  | coal_seam | 9.69 | 8 | 10 | ranks from TTI; 2 guaranteed (exposed) |
  | mvt_zn_pb | 3.74 | 2 | 2 | |
  | sedex_zn_pb | 1.11 | 2 | 2 | |
  | petroleum_trap | 1.08 | 1 | 1 | charged 255 Myr ago by the rift kitchen |

  The pulled PNGs (`review/out/p1-deposits/keyframes/`) show the porphyry favourability on A's
  Andean arc, coal and MVT in B's foreland, SEDEX and oil in C's rift, BIF on C's craton; headless
  Chromium screenshots of `?site=` pages show the chains (placer ← exhumation ← erosion pulse,
  formation ← flare-up ← subduction onset; reservoir ← generation, subsidence, inversion).
- **Deviations from the plan.** (1) The acceptance run uses the synthetic preset file, not
  `earthlike_quick_16k` (which P1-03 creates). (2) The lognormal is split at the median so all
  three USGS percentiles are reproduced; its mean σ is the plan's (1.49 for porphyry). (3) Province
  endowments (lognormal, σ 0 / 0.35 / 0.7 / 1.2) redistribute a type's ρ·A·F between tracts and
  within them; they do not change the planet's expected count. (4) Iron is BIF only (no Fe skarn).
  (5) The essentials list comes from the preset until P1-15's `deeptime:essentials` registry;
  `guaranteed_occurrence` is emitted by the deposit stage (EVENTS.md updated). (6) Custom deposit
  types from datapacks are parsed by the same code but not yet passed in (the content bundle is not
  wired to the stage). (7) Causes are matched by point places (region masks arrive with P1-03).
  (8) The last frame's favourability is drawn one year before its end so the closing keyframe
  (sites) is a separate frame; a real pipeline that reaches the deposit stage at t = 0 needs a
  different trigger.
- **Field inputs the engine expects from P1-03…P1-07** (per L1 cell, per frame; frames oldest
  first, each standing for Δt years, **replayable** — pass 2 rereads them; `FieldHistory` in
  `deposits/fields.rs`, a `MemoryHistory` is there to record into):

  | field | units | provider |
  |---|---|---|
  | `deeptime:arc_magma_flux`, `deeptime:magma_oxidation` | 0–1 | P1-03 |
  | `deeptime:convergence_rate_km_myr`, `deeptime:obliquity_deg`, `deeptime:slab_age_myr` | km/Myr, °, Myr | P1-03 |
  | `deeptime:crust_thickness_km`, `deeptime:fault_density` | km, 0–1 | P1-03 |
  | `deeptime:orogenic_uplift_m_myr`, `deeptime:rift_subsidence_m_myr`, `deeptime:heat_flow_mw_m2` | m/Myr, m/Myr, mW/m² | P1-03 |
  | `deeptime:dist_litho_step_km` | km | P1-03 |
  | `deeptime:inherited_bif` | 0–1 | P1-03 (craton ages) + P1-07 |
  | `deeptime:elevation_m`, `deeptime:erosion_rate_m_myr`, `deeptime:deposition_rate_m_myr` | m, m/Myr | P1-04 |
  | `deeptime:topographic_gradient_m_km`, `deeptime:shelf` | m/km, 0–1 | P1-04 |
  | `deeptime:carbonate_platform`, `deeptime:evaporite`, `deeptime:euxinic_basin`, `deeptime:coal_facies` | 0–1 facies | P1-04 (strata log) |
  | `deeptime:source_rock_toc`, `deeptime:reservoir_quality`, `deeptime:seal` | wt%, 0–1 | P1-04 |
  | `deeptime:burial_temp_c` | °C of the organic layer | P1-04 + P1-07 |
  | `deeptime:trap_structure` | 0–1 closure | P1-07 |
  | `deeptime:water_table_depth_m` | m | P1-04 + P1-05 |
  | `deeptime:precip_mm`, `deeptime:t_mean_c` | mm/yr, °C | P1-05 |
  | `deeptime:host/<rock class>` | 0–1 fraction in the ore window | P1-07 (optional; absent = no lithology opinion) |
  | clocks `deeptime:clock/{oxygenated_oceans,land_plants,bif_acme}` | 0–1 per frame | the preset's `planet.atmosphere` (P1-03/P1-05) |
  | present-day drainage receivers (L1 cell → downstream cell) | — | P1-04 |

  Plus point-placed tectonic events for "Why here?" (`subduction_onset`, `collision`,
  `orogeny_phase`, `terrane_accretion`, `rift_onset`, `basin_subsidence`, `marine_transgression`,
  `erosion_pulse`) with t_start/t_end spanning the activity.
- **Not done.** Real-field integration (record frames in P1-03…P1-07, then a preset pipeline that
  ends with this stage); datapack content passed to the stage; Fe skarn; spawn ranking (§7.8);
  the Java side of bodies and its parity test (P1-15); size multipliers tuned on in-world renders
  (P1-15); merging placers that land on one drainage cell (the second is dropped by separation);
  a spacesheep review page. Tuning note: time integration saturates F near 1 inside long-lived
  systems (mean F 0.87–0.98), so counts follow ρ·A and favourability mostly shapes tracts.
- **Next step.** P1-03…P1-07 publish their fields as frames (`FieldHistory`); then point the
  real preset's pipeline at `DepositStage` and rerun the acceptance on `earthlike_quick_16k`.

### 2026-09-28 — P1-13 first-start generation, the mechanism (agent worktree agent-a31ba4add1e8bb6d2)
- **What.** A dedicated server whose overworld is Deep Time's (no test terrain, no staging) and whose
  world has no `deeptime/manifest.json` now generates on its first start (plan §7.13, updated):
  `server/FirstStartGenerator` (called by `WorldDataBinder`'s one HIGHEST `ServerAboutToStartEvent`
  listener before it binds — P0-10's binder therefore binds what generation wrote, and the order
  never depends on the bus ordering two HIGHEST listeners), `server/ServerConfig`
  (`config/deeptime/server.json` with `timeline` — review 2 gap 21 — and a valid-JSON template whose
  explanations are `_` keys), `server/ServerStop` + `mixin/DedicatedServerFirstStartMixin`
  (`server` side) + `mixin/MinecraftServerFirstStartMixin`, `preset/{ShareCode, Presets,
  PresetException}`, `SimSession.estimate`. Rust: `deeptime_core::{preset, share_code, synth}`,
  CLI `run --preset <name|file|DT1…> [--start] [--length-myr]`, `stage`/`bench` take the same
  preset argument, `deeptime preset list|show|encode|decode`. Smoke: `install.sh --start
  --length-myr --memory-cap-gib`, `run.py` generation checks and `--scenario`.
- **Placeholders, not presets.** The real stages (P1-03…P1-08) do not exist, so the four built-in
  presets (`crates/deeptime-core/presets/*.json`, one source: `include_str!` in the core, copied into
  the jar as `deeptime/presets/<name>.json` by `processResources`) are `"placeholder": true`: the
  dummy pipeline (P1-01's `stages`/`world`, 25 or 50 steps per stage at 200 ms) plus a new dummy key
  `"synth": "wrap-test"` that bakes P0-09's synthetic layers at `finalize_write` (the synth module
  moved from `deeptime-cli` to `deeptime-core::synth`; `deeptime synth` output is unchanged). The
  server logs a WARN for a placeholder. Nothing in the Java path knows about the dummy: when P1-08
  lands, replacing the four JSON files is the whole change.
- **Preset header (core).** `Preset` is now `{header, body}` (`preset.rs`; `session::Preset`
  re-exported): header keys `schema`, `name`, `description`, `placeholder`,
  `scale.circumference_blocks` (must equal the dummy world's C) and `timeline` (`start` ∈
  supercontinent | scattered_continents | random, `length_myr` 1…4600). On a dummy world the
  timeline sets the clock to −length…0 with eight keyframe intervals. `manifest.source` is
  `run:<name>`; `notes.run` gains `timeline` and `placeholder`.
- **Share codes.** `DT1.` + unpadded base64url(raw DEFLATE level 9 (canonical JSON)). Canonical =
  serde_json's compact sorted form; Java's `Presets.canonical` reproduces it byte for byte, including
  serde_json 1.0.151's float layout — it formats with **zmij**, not ryu: positive exponents carry a
  `+` (`1.5e+300`, `1e+16`), decimals for 1e-5 ≤ |x| < 1e16. Pinned by the same expected string in
  `share_code::canonical_numbers_follow_serde` and `PresetsTest.canonicalJsonIsSerdes`.
  Cross-test: `crates/deeptime-core/tests/share-codes.json` (3 cases) holds the code Rust writes and
  the code Java writes (`ShareCodeTest` writes `neoforge/build/share-codes-java.json`;
  `tools/dev/share_code_fixture.py` fills the fixture); both suites decode both, and Rust also checks
  its own code is stable. For two cases the two encoders' codes differ (zlib vs miniz_oxide), for one
  they are equal. Decoding caps the inflated text at 1 MiB.
- **The clean stop.** Vanilla has no clean "stop before loadLevel": `initServer` returning false or
  an exception both end in a crash report plus an NPE in `stopServer` (no overworld). So `ServerStop`
  halts the server and the mixins make `initServer` return `true` right after the event (before
  "Preparing level", RCON, the watchdog, `ServerStartingEvent`), skip `ServerStartedEvent`,
  `ServerStoppingEvent` and `saveAllChunks`; `stopServer` still closes the listener and
  `session.lock`, `ServerStoppedEvent` still fires, and a non-daemon `deeptime-stop` thread exits
  the JVM with status 1 after the server thread ends. **Finding:** `net.minecraft.server.Main`
  writes `level.dat` (line 276, `saveDataTag`) before the server thread starts, so it exists before
  any mod event and already records the level-type; the mismatch message therefore says to delete
  the world folder, not only change `level-type`. The smoke checks it is not rewritten at the stop.
- **Measured (Mac, `tools/remote/mac.sh`).** `cargo test --workspace` all ok (core 56 incl. preset 4,
  share_code 5, synth 2, session `placeholder_preset_bakes_terrain_next_to_the_run`); `cargo fmt
  --check`, `clippy --workspace --all-targets -D warnings`, `deny check licenses bans` clean
  (flate2 1.1.10 joins deeptime-core; already in the lock via deeptime-inspect). `gradlew build
  test` → tests=45 failures=0. `mac.sh --server -- ./gametest.sh` → All 14 required tests passed.
  `mac.sh --server -- 'tools/smoke/install.sh standalone --preset earthlike_quick_16k && python3
  tools/smoke/run.py --profile standalone --fresh --restart'` → PASS: generation 11.9 s,
  `[deeptime] dummy_drift 84% (−580 Ma) · total 42% · 5s elapsed · ~7s left · native 0.0/4.0 GiB`,
  `… world data written to …/planet/deeptime in 11s (13 events, 3 sites, 9 keyframes, 8 layers,
  source run:earthlike_quick_16k)`, native `baked the synthetic terrain wrap-test (C = 16384): 8
  layers, 21.0 MiB in 833 ms`, `Done (15.111s)`; restart `[deeptime] world data present at …:
  not regenerating`, `Done (0.676s)`, manifest mtime unchanged. `run.py --scenario
  first_start_no_config | first_start_circumference_mismatch | first_start_native_failure` → each
  `OK` (exit 1, no `Done (`, no `Preparing level`, no region/entities/data/deeptime, `level.dat`
  older than the stop line, no crash report or shutdown exception; the template parses; the
  mismatch line names `C = 32768` and `C = 16384`; the native case logs the UNSUPPORTED_PLATFORM
  guidance and `deeptime run --preset earthlike_quick_16k --start supercontinent --length-myr
  1000.0 --out …/planet/deeptime`). `deeptime run --preset earthlike_quick_16k --start
  supercontinent --length-myr 1000 --out …` → the same layout (8 layers, 17 files), 11.6 s.
- **Known limits.** The estimate of a placeholder is 0 bytes (the dummy estimates nothing and the
  bake runs at finalize; the CLI measured a 403 MiB peak for the 16k bake). Built-in presets are
  located by name only in `Presets.BUILTIN` (a test checks it equals the core's folder).
  `tools/smoke/__pycache__/run.cpython-313.pyc` is tracked in git (not by this task); left as is.
- **Next.** P1-08's bake replaces the placeholder JSON files; then rerun the acceptance above
  unchanged. P1-11 (staging) can hook into `WorldDataBinder.onAboutToStart` the same way.

### 2026-09-28 — P0-06 finished (sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Verdict: Toroidal World wraps a Deep Time planet in the MIC pack, Sable ships included.** No
  failure that configuration cannot fix, so no Q6 and no fallback. Commands (all on the Mac):
  `tools/remote/mac.sh --server -- './gametest-planet.sh --run columnscontinuous,entitycrossesseam,structurestraddlesseam,treestraddlesseam,shaftlineacrossseam@8185,250,0 --run jetcrossesseam@8185,250,64 --run prototypepushback@0,300,3527~16,160 --scenario sable_straddle'`
  → `PASS`; `tools/remote/mac.sh --server --pack -- 'tools/smoke/install.sh mic --size 32768 --test-terrain wrap-test && python3 tools/smoke/run.py --profile mic --fresh --chunky 256 --chunky-center seam'`
  → `PASS`, and the same with `--size 16384` → `PASS` (pack 303fd89 snapshot, 168 jars).
- **Findings table** (wrapped_planet_worlds.md §6.3, server side; C = 16,384 unless stated):

  | Item | Result | Evidence |
  |---|---|---|
  | TW stamps the generator; X loops, Z not; no climate compression | pass | probe tw `stamped=true loopsX=true loopsZ=false climateCompression=off`, `widthChunks` 1024 (16k) and 2048 (32k), on the planet server and in both MIC smokes |
  | Terrain continuous at the seam | pass | columnsContinuous: 16 rows, 0 bad folds, 0 jumps (terrain blocks; z=1: 111,111 → 111,110) |
  | Entity crosses the seam | pass | entityCrossesSeam: snowball at x = −8187.22 after crossing, in span |
  | Structure straddling the seam | pass | structureStraddlesSeam: `plains_small_house_1` over x 8188..8194 placed chunk by chunk through TW's `StructureStartMixin`; east 194 blocks right, west 147 right, 0 wrong |
  | Tree straddling the seam | pass | treeStraddlesSeam: oak trunk at x = 8191, 6 logs, 34 east + 20 west leaves, 0 decaying after the west leaves re-read their distance across the seam |
  | Create shaft line across the seam (informational) | works | shaftLineAcrossSeam: creative motor at C/2 − 4, 16 RPM on all 7 shafts, 4/4 west of the seam |
  | Sable jet across the seam, ×20 | 20/20 | jetCrossesSeam: 10-block hull at 40 blocks/s crosses at tick 6, x = −8187.76 after (folded, in span), 10/10 blocks, vx 38.6 kept; 20 `[gametest-attempt] … ok`, one PASS |
  | Sable hull straddling the seam survives save + restart | OK | `sable_straddle: OK (sub-level 82e61644-…, 19 blocks, x 8191.97 before, 8191.97 after)` |
  | Chunky across the seam, nothing stored outside the span | pass | 1089 chunks centred on x = C/2 (24 s at 32k, 29 s at 16k); `regions outside span: 0` (22 files at 32k, 19 at 16k); both edge columns hold chunks (1485 east, 1540 west) |
  | MIC boots with TW and the planet | pass | boot 24 s (32k) / 19 s (16k) on the Mac; every Deep Time mixin; 233–234 of 278 TW mixins by `Done (`; 7 TW compat configs; no C2ME/DH errors; village located; peak RSS 5.4 / 5.2 GB |
  | Pole pushback (PROTOTYPE) | pass | prototypePushback: dz 0 / −10.7 / −26.5 / −42.2 blocks in 40 ticks at 74.5° / 76° / 77.5° / 79°; minecart at 79° −14.4 |

- **Deferred (review 2, gap 28)**, all server-side testable but not in this task: a Create **train**
  across the seam; a **bearing or elevator contraption** straddling it; **belts** across it; the
  **pole-transit prototype** (teleport + 180° heading; decisions file: "later"); the **Sable-ship pole
  pushback** (P1-18's `sableHullPushedBack`); a jet flying into *unforced* chunks (destination chunk
  tickets); passengers, propellers and ropes (Aeronautics). Client-only items stay unverifiable (§9.8).
- **Harness facts found here** (the plan's §9.4/§9.5 now say "as built"):
  1. 1.21.1's `/test run` starts with `TestCommand.stopTests()`, cancelling every running test; the
     earlier "two /test run at one spot cancel each other" was this. The harness runs one test at a time.
  2. `/test run` ignores y: the test goes on the WORLD_SURFACE heightmap at (x, z + 3), which reads
     y = −352 while that chunk is not loaded (the first runs placed tests inside the bedrock floor).
  3. When a test batch ends, `GameTestRunner` un-forces **every** forced chunk of the level. The
     harness forceloads again and waits (`execute if loaded` on every corner and test spot) before
     each test; tests refuse a spot at the world floor with a clear message (`surfaceSpot`).
  4. TW's `/forceload` accepts canonical coordinates only ("8217 is outside this world"): boxes
     across the seam are split in two.
  5. Vanilla creates an empty `.mca` whenever it opens a region file for a read; after a Chunky square
     across the seam there are eight 0-byte files at rx = C/1024 and −C/1024 − 1 (0 chunks). run.py's
     span check now counts region/entities/poi files that store a chunk and records the empty ones.
  6. `attempts = 20, requiredSuccesses = 20` under `/test run` takes vanilla's flaky path: one PASS
     after 20 passing attempts (or one FAIL at the first failure), not one report per attempt. The test
     body reports each attempt to `DeepTimeTestReporter.attempt`; the summary carries
     `attempts/attemptsOk/requiredSuccesses/required`, and the harness requires attemptsOk ≥ requiredSuccesses.
  7. OCEAN_FLOOR counts trees and boulders, so one run's random seed put a 3-block "jump" at the seam:
     columnsContinuous now compares terrain blocks (stone, dirt, sand, gravel, clay).
- **Implementation details.** Sable is `compileOnly "maven.modrinth:sable:U678xqle"` plus its nested
  `sable-companion-common-1.21.1-1.6.0.jar` (extracted by `extractSableCompanion`: `BoundingBox3i`,
  `Pose3d` live there); signatures checked with javap on the 2.0.5 jar. `compat/sable/SableTestOps` is
  the only class naming Sable types (vanilla/JDK types in its signatures); removal goes through
  `ServerSubLevelContainer.removeSubLevel(plotX − originX, plotZ − originZ, REMOVED)` as Sable's own
  command does; block counts are the non-air blocks of the plot box (TW treats Sable plots as a foreign
  frame and never folds them). New commands: `/deeptime probe sable` (only with Sable loaded) and
  `/deeptime gametest sable straddle <z>`; `probe column` gained `chunkLoaded`/`worldSurface`.
  A straddling hull is assembled from raw contiguous positions (x = C/2 − 3 … C/2 + 2); its pose
  stays unfolded while its centre is west of C/2. `wrap/PoleCapPrototype`: `EntityTickEvent.Post`,
  players, vehicles and tagged probes, 0.08 blocks/tick at 80°, linear from 75°, dev runtimes only.
  run.py: `--chunky-center x,z|seam`, square shape, `report-C<size>.md` copy; run_planet.py:
  `--scenario sable_straddle`, spec `name,…@x,y,z[~dx,dz]`, optional tests informational.
- **For the P0-06 review page** (not published here; another agent owns `tools/review/publish.sh`):
  this worktree's gitignored `smoke/p0-06/` holds `report-C32768.md`, `report-C16384.md`,
  `server-C32768.log`, `server-C16384.log`, `planet.log` (the passing planet run), `planet-harness.out`
  and `smoke-harness.out`; publish them with the findings table above.
- **Next.** Publish the P0-06 page; the deferred items above go to a follow-up (trains and contraptions
  fit P1-16's compat work, the Sable pole pushback is P1-18).

### 2026-09-28 — P1-01 (sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Formats** (`crates/deeptime-format`, spec in `FORMAT.md` and the new `EVENTS.md`). Every new
  binary file is one "sectioned file" (`container.rs`): the §5.7.2 16-byte header, per-type fixed
  fields, a section table (u64 offset, u32 length, u32 CRC32) and 8-byte-aligned sections.
  `DTEV` event segments `events/<writer>-<n>.dtev` (string table, 64-byte records sorted by
  (t_start, id), causes, 16-byte typed params, a 64-record time index) + `events/index.json`
  (writers, segments with first/last seq, t range, bytes, CRC); `EventWriter` continues a writer's
  numbering from the index and `clear()`s only its own segments on a re-run; `EventLog` merges all
  writers and indexes by time, kind, cause (reverse links, `cause_chain`) and L1 cell. `DTSR` site
  records (104 bytes) + `DTSI` 512-block grid (CSR, x wraps). `DTKF` keyframes (RGBA8 per layer,
  LZ4 block) + `keyframes/index.json`. `ramps.rs`: the one colour code for ramps.json (core
  keyframes and viewer). Format version stays 0.0; no existing file's bytes changed.
- **Core.** `stage.rs` (Stage: name, estimate, total/steps_done, `step(world, ctx)` with the
  deadline in `ctx.until`, keyframe_layers, `save_state`/`load_state` — the plan's `checkpoint(w)`
  is split so the session builds checkpoints), `world.rs` (`WorldState`: clock, named f32 fields,
  events as writer 1, sites), `dummy.rs` (the dummy preset grows optional `stages` and `world`: a
  drifting continent, a causal event chain, deposit sites; without them it is exactly P0-07's),
  `keyframes.rs` (render with ramps, ring of 16, every frame on disk at once, dropped frames read
  back), `checkpoint.rs` (`DTCP`: JSON summary + postcard 1.1.3 body; schema constant). The session
  runs a list of stages, publishes events for `eventsSince` (cursor = count in emission order),
  takes keyframes on an interval grid counted from the run's start and at each stage end, resumes
  from checkpoints (mid-stage too; a resumed run ends in the identical world — unit test), and
  `finalize_write` writes preset.json, events, sites, keyframes/index.json and the standard
  `Manifest` last (source `run:dummy`, every file listed with size and CRC).
- **CLI.** `run --preset --out [--viewer] [--checkpoints]`, `stage <name> (--in|--preset) [--out]
  [--viewer]` (refuses a stage the checkpoint already ran), `bench --preset --threads --out
  [--viewer]` (per-stage ms incl. keyframes, finalize, peak bytes from a counting allocator).
- **Viewer.** Keyframe strip, scrubber with play, event ticker (events up to the frame, new ones
  bold), site list, details with params, causes/effects and the "Why here?" chain; markers on the
  frame; `events.json`/`sites.json`; the layer map hides itself when a bundle has no layers.
  Bench page: per-thread-count bars on one scale + table. Checked in headless Chromium.
- **Measured (Mac, `tools/remote/mac.sh`).** `cargo test -p deeptime-format` 27 passed;
  `cargo run -p deeptime-cli -- run --preset crates/deeptime-core/tests/presets/dummy.json --out
  $TMPDIR/p1-01/dummy --viewer review/out/p1-dummy` → 2 stages, 13 events, 9 keyframes, 3 sites,
  peak 1.5 MiB, 0.19 s; `cargo test --workspace` (core 41 incl. P1-02's 23, format 27, cli 10, jni 7, viewer 4),
  `fmt --check`, `clippy -D warnings`, `deny check licenses bans` clean; `gradlew build test`
  tests=34 failures=0 on master 2f81ae5 + this branch (NativeLinkTest gains
  `keyframesAndEventsCrossTheBoundary`);
  `mac.sh --server -- ./gametest.sh` → All 14 required tests passed (NativeGameTests
  `dummySessionRunsAndCancels`: steps=5 manifest ok, cancelled run wrote no manifest). Bench of the
  dummy (debug build): stages 36.7 ms at 1 thread → 7.7 ms at 8.
- **Contract changes to note.** The session's `manifest.json` is now the standard `Manifest`
  (P0-07 wrote an ad-hoc one; Java only ever checked `format_version`). `estimate` lists every
  stage (`{"peak_bytes":0,"stages":[{"bytes":0,"name":"dummy","seconds":0.0}]}`; NativeLinkTest
  updated). `keyframeInfo`/`copyKeyframe` are wired: `UnsupportedOperationException` only when the
  preset makes no keyframes (the Phase 0 dummy), `IllegalArgumentException` for an index not yet
  produced or a short/heap buffer. New `CoreError::{InvalidArgument, Format}` (JNI maps them to
  `IllegalArgumentException` and `DeepTimeIoException`).
- **Not done / later.** Region and path places are not cell-indexed (masks and paths are stored
  from P1-03); geology writes its events as one segment at `finalizeWrite` (no streaming yet);
  `run --data <jar|dir>` is `--content <file>` until P1-07; the kinds table in `EVENTS.md` reserves
  every v1 kind with "params set by P1-xx".
- **Longitude.** Events, sites, keyframes and the dummy world use plan §5.2's longitude
  360°·(x + C/2)/C (the seam is 0°), as P1-02's coupling and master's viewer do.

### 2026-09-28 — P0-11 `deeptime-inspect` (sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Created.** `crates/deeptime-inspect` (lib + binary): region files (4 KiB sectors; gzip, zlib,
  raw and lz4-java `LZ4Block` chunks; external `.mcc`), an in-crate NBT reader, 1.21 sections
  (palette + packed longs), heightmaps and structure pieces, and a small zip reader for jars.
  Subcommands `colors`, `topdown`, `rock`, `ore`, `section`, `seam`, `regions`, `compare`, plus
  `column` (one column's blocks) and `fixture` (copies chunks into a new region file). New deps,
  all MIT/Apache: clap, flate2, lz4_flex, png, memmap2. Fixture: chunks 4..=9 × 0..=3 of the
  standalone smoke world, 389 KB (`tests/fixtures/world/region/r.0.0.mca`).
- **Gap 18 (compare made machine-checkable; plan §9.7 and P0-11 updated).** Measured = the
  highest block that is not air, fluid or a feature (vegetation, logs, wood, leaves, roots, huge
  mushrooms, coral, sea pickles, snow layers, sheet ice, …): `OCEAN_FLOOR` minus logs, leaves and
  features. Expected = `⌈H⌉ − 1` (bilinear `elevation`, the GameTests' value). Full chunks only;
  excluded and counted: `structure` (12 blocks around pieces reaching the surface) and `carved`
  (2 columns around a pit ≥ 2 blocks below close₃₂(open₄(measured)); reads only the measured
  surface). A first rule (air/fluid under neighbours' surfaces) missed vertical ravines and smooth
  dry cave-entrance hollows on beaches (17 blocks deep, no overhang), hence the morphological
  test. `section` prints `water_top_y=<y> water_block_y=<y−1> columns=<n>`, `water_top_y` being
  the top face of the highest water block as P0-13's `lakeHolds` defines it. On P0-13's GameTest
  world (main checkout's Mac copy) `section --world run-gametest/world/dimensions/deeptime/test_lake
  --axis z --at 0 --from -88 --to 212` → `water_top_y=120 water_block_y=119 columns=209`.
- **Measured** (Mac; `install.sh standalone --synth wrap-test --size 16384 && run.py --profile
  standalone --fresh --chunky 128 --chunky-seam` → PASS): compare p99 1, p50 0, max 5,
  histogram −3:9 −2:145 −1:2478 0:105190 +1:1106 +2:124 +3:85 +4:35 +5:12; excluded carved
  11,792 (9.2 %), structure 7,024 (5.5 %). Renders in `review/out/p0-11/` (Mac and laptop,
  gitignored); the colour table from the client-extra jar + the 6 standalone jars resolves 1,836
  blocks.
- **Findings.** (1) With aquifers on, the ocean floor carries aquifer-barrier stone (then sand)
  2–6 blocks above H, and geodes poke through it: 0.2 % of the world's columns, 4 % in the
  fixture's patch. (2) The game creates 0-byte region files beyond the seam (r.16.*, r.−17.*)
  when it merely looks chunks up; TW folds every write. `run.py` now records them
  (`outside_empty`) instead of failing the span check; `regions` does the same. (3) `run.py
  --chunky-seam` pregenerates x ∈ [C/2 − 144, C/2) and [−C/2, −C/2 + 144), z ∈ [−64, 80): Chunky's
  square is `(int) centre >> 4 ± ceil(radius / 16)` chunks.
- **Limits.** The pit test would also drop real hollows narrower than 64 blocks (rivers, P2);
  `rock` maps blocks through the `#deeptime:rock/*` tags until `/deeptime probe palette` exists;
  format 0.0 has no site bodies to overlay. Next: P0-13 uses `section`, P0-14/15 the renders.

### 2026-09-28 — P0-13 (sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Created.** `worldgen/WaterPass` (after `StrataPass` in `buildSurface`; fills air below the water
  surface down to the ground with water sources through `ProtoChunk.setBlockState`, so the worldgen
  heightmaps stay current and no fluid tick or post-processing mark is made), `worldgen/WaterLevels`
  (plain-Java level lookup, JUnit-tested on the lake fixture), the gametest level `deeptime:test_lake`
  (folder `deeptime-lake`, C 1024, `planet_nocaves`, fixed biome `deeptime:test_bare`: no carvers, no
  features, temperature 0.8), `WaterGameTests.lakeHolds`; `gametest.sh` also writes
  `synth --kind lake-test --size 1024`.
- **Measured** (Mac, `tools/remote/mac.sh --server -- './gametest.sh'` → All 14 required tests passed):
  `[gametest] lakeHolds before=65304 after=65304 expected=65304 outside=0 outsideAtGeneration=0
  columns=7299 shoreColumns=2 reach=4 woken=65304 ticks=400 generatedFluidTicks=0
  pendingFluidTicksAfter=0 analytic=66062 floorCornerMaxDiff=0 cornerColumns=442 chunks=240` and
  `[gametest] lakeHolds water_top_y=120 at=(0, 0) top_water_block_y=119 floor_y=109` (gap 18's
  printed value; the plan's P0-13 acceptance now names it and fills in the section's z and x range).
  lakeHolds takes ~20 s (240 forceloaded chunks generated synchronously, then 400 ticks). JUnit
  tests=33 failures=0.
- **The test is stricter than the plan's:** after the first count it schedules a fluid tick on every
  water source it counted (as a changed neighbour would), so "after" proves the basin is watertight,
  not only that nothing ticked. `expected` is every open block (air or water) between each footprint
  column's ground and its surface; `outside` is any water in a box 16 blocks past the footprint (48 until P1-12),
  y 84…141, that is not in those blocks. `floorCornerMaxDiff=0` ties the basin to the raster at
  noise-cell corners; `analytic` is the same count from the raster's H (informational: bilinear
  interpolation over 4-block noise cells moves the shore by a block or two).
- **Shore rule (contradicts nothing in the plan; a new bake requirement).** The first run failed with
  `outside=2`: two shore columns (27,33)/(33,27) at d = 2.64 had ground 118 under the surface 120
  (raster H 119.89), but their nearest `water_level` cell (28,34) sits at d = 4.05, outside the synth's
  wet ring (d ≤ 4); the woken lake turned those pockets into new sources. Generated ground is the
  density interpolated bilinearly over 4-block noise cells, so on a steep shore it lies below the
  raster. `WaterLevels.levelAt` therefore gives a dry-celled column the level of the nearest wet cell
  within one noise cell (`reach` = the settings' cell width, 4); only air below the level is ever
  filled, so this adds no water on ground above the level. **Phase 2's bake must keep each basin's
  rim above its level for at least one noise cell (4 blocks) beyond the last wet cell.**
- **Ice (plan §7.9) is not placed by WaterPass:** vanilla's `freeze_top_layer` feature, present in
  every overworld biome, freezes the top water block where the biome is cold, and the lake exists
  before features run. A `SpreadingSnowyDirtBlock` (grass, mycelium) left under the water by the
  surface rules becomes dirt, as vanilla's underwater rule would make it (its random tick would
  otherwise do so later and wake the water).
- **Not run:** `./gametest.sh -PwithMic` (a pack biome modifier that targets every biome could give
  `test_bare` features); the `deeptime-inspect section` PNG waits for P0-11.
- **Next.** P0-11's `section --world run-gametest/world/dimensions/deeptime/test_lake --axis z --at 0
  --from -88 --to 212` for the review picture.

### 2026-09-28 — P0-08 (sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Created.** `tools/natives/build-natives.sh` (`nix run .#natives`; the app re-enters the dev
  shell so build scripts get a host linker on macOS too): cargo-zigbuild for Linux (gnu at `.2.17`)
  and macOS, cargo-xwin for Windows, then per-file checks (ELF/PE/Mach-O machine, musl NEEDED and
  GLIBC, gnu floor ≤ 2.17, macOS minos ≤ 11.7.1 and `/usr/lib`-only dylibs, no `target-cpu` in
  RUSTFLAGS) and `natives.json` (schema 1, same fields and glibc scan as `hostNatives`). Gradle
  `-Prelease` packs `target/natives-dist` after `checkReleaseNatives` (exactly the eight keys,
  size + sha256 per file, version/format equal to Cargo.toml, no stray files; clear message if
  missing). `.github/workflows/natives.yml`: build job (natives + release jar, both uploaded) and
  eight `NativeSmoke` load jobs against the release jar, each also asserting `platform=<its key>`.
- **Measured on the Mac** (`tools/remote/mac.sh -- 'nix run .#natives'`, then readelf/file):
  `file` → 2 PE32+ (x86-64, ARM64), 2 Mach-O (x86_64, arm64), 4 ELF (2 x86-64, 2 aarch64); musl
  NEEDED `libc.so` only (no `libgcc_s`: zig links the unwinder); gnu x86_64 and aarch64 highest
  `GLIBC_2.17`; `./gradlew build -Prelease` → 8 libraries in the jar, tests=30 failures=0, and
  NativeSmoke on that jar on the Mac → `status=OK … platform=macos-aarch64`.
- **CI** ([run 36492319610](https://github.com/MinecraftIndustrialComplex/deep-time/actions/runs/36492319610),
  first attempt): every job success; each load job printed `status=OK version=0.0.1 format=0.0
  selftest=1997318690096807936` with its own platform key (windows-x86_64, windows-aarch64 on
  `windows-11-arm`, macos-aarch64, macos-x86_64, linux-{x86_64,aarch64}-gnu, and
  linux-{x86_64,aarch64}-musl in Alpine). Cold build job 25 min (natives 18, Gradle 5); caches
  are saved now. `workflow_dispatch` returns 404 until natives.yml is on master, so the bring-up
  used a temporary push trigger on branch `p0-08-natives` (reverted).
- **Contradicts the plan (now §6.4 "Measured in P0-08").** macOS natives use cargo-zigbuild on the
  Mac as well: a plain `cargo build` in the Nix shell linked `/nix/store/…/libiconv.2.dylib`. zig
  0.13 sets minos 11.7.1 on both macOS libraries. The dylibs get install name
  `@rpath/libdeeptime_jni.dylib` (`.cargo/config.toml`). Windows DLLs import `VCRUNTIME140.dll`
  (the JVM loads it itself; no static CRT). cargo-xwin is 0.23.1 in nixpkgs and CI. cargo-xwin's
  README asks only for clang and "LLVM tools"; the shell's clang-cl, lld-link and llvm-lib suffice.
- **Next.** Merge; after natives.yml is on master, `gh workflow run natives.yml` works as written.
  Actions still warns that checkout/cache/setup-java/artifact @v4 run on Node 20 (forced to 24).


### 2026-09-28 — P1-02 L0 geometry (agent worktree agent-ab52c5f47be5db0ae)
- **What.** `crates/deeptime-core/src/sphere/{mod,fibonacci,hull,kdtree,coupling}.rs` and
  `crates/deeptime-core/examples/sphere_bench.rs`. New deps in deeptime-core only: glam 0.33.11 and
  kiddo 6.3.0 (default features off, `multi-threaded` on, which drops `fixed`), both MIT OR
  Apache-2.0; `cargo deny check licenses bans` → ok. Nothing ported, so NOTICE.md is unchanged.
- **Hull algorithm.** Randomized incremental 3D convex hull with conflict buckets (Clarkson & Shor
  1989; de Berg et al. 2008 ch. 11; the bucketing of Guibas, Knuth & Sharir 1992), inserted in a
  BRIO order (Amenta, Choi & Rote 2003), expected O(n log n). Exact predicates: coordinates snapped
  to multiples of 2⁻³⁸ (≤ 3.2e-12 movement), orientation by a float evaluation with Shewchuk's
  stage-A bound and an i128 fallback. Duplicates are skipped and listed (`skipped()`); cocircular
  sets (cube, octahedron) triangulate; point sets that do not surround the centre fail with
  `NotCovering`. Point location is a stochastic visibility walk; barycentrics are gnomonic (ray from
  the centre to the face plane).
- **Measured** with `tools/remote/mac.sh -- 'cargo build --release -p deeptime-core --example
  sphere_bench && ./target/release/examples/sphere_bench [--threads 1 250000]'` (M6 Mac mini, 12
  threads, min of 3): 250k samples → Fibonacci 0.2 ms, **hull 255 ms** (280 ms on 1 thread; the
  hull is sequential), k-d tree 7.8 ms, L1 coupling 12 ms at 1024×512 and 41 ms at 2048×1024
  (98 / 351 ms on 1 thread); 100k → hull 98 ms, tree 2.9 ms, coupling 10 / 36 ms. V − E + F = 2,
  nothing skipped, 0 bad cells on all four presets' L1 grids. BRIO saves ~6% over a plain shuffle
  (254 vs 270 ms, A/B in one session). Next to Cortial's Table 2 (1.24 s per regular step at 250k),
  re-triangulating at every resampling is cheap.
- **Conventions fixed in `coupling.rs`.** λ = 360°·(x + C/2)/C ∈ [0°, 360°) (§5.2; the seam
  x = −C/2 is λ = 0), φ = −360°·z/C (north is −z), v = (cos φ cos λ, cos φ sin λ, sin φ) with +Z
  the north pole and +X the seam meridian; raster cells are `GridDesc` point samples (row 0 = north
  pole). **Differs from existing code:** the viewer's inspector (`viewer.js` `inspect`) prints
  longitude 360°·x/C (seam at ±180°), and `AnalyticTestTerrain` uses θ = 2πx/C and φ = πz/(C/2)
  (+z north). Both are a constant 180° longitude shift (and, for the test terrain, a latitude sign
  that its |φ|/cos φ formulas never see); align the viewer's label when it next changes.
- **Coupling layout.** `CellLink { tri, nearest, w: [f32; 3] }`, 20 B per cell (42 MB at
  2048×1024). `Coupling::refresh(mesh, tree, stale)` re-links only the cells a predicate marks;
  P1-03 supplies "the cell's triangle left its plate". `interpolate` (barycentric) and
  `categorical` (nearest sample) sample L0 fields onto the grid.
- **Tests** (`cargo test -p deeptime-core sphere::`, 23 tests, 3.1 s debug): Euler and exact
  local-Delaunay/neighbour checks on Fibonacci 12…100k and 10k random points, a brute-force
  convexity check, degenerate sets, every L1 cell of the 16k-quick ladder (100k samples,
  1024×512), k-d nearest against brute force on 10,000 random points (100k Fibonacci and 20k random
  samples), and property tests for block ↔ lat/lon ↔ unit vector (round trips, x wraps at ±C/2,
  the columns either side of the seam one block apart).
- **Next.** P1-03 (plates) builds on `SphereMesh`, `SampleTree` and `Coupling`.

### 2026-09-28 (later) — builds moved to the Mac; P0-06 shaping; P0-07 and P0-09 merged
- **Builds run on Ben's M6 Mac mini** (his instruction, after parallel builds overloaded the laptop):
  `tools/remote/mac.sh` rsyncs the checkout to `/Volumes/Graphide/ban/mic/mods/<name>` on the external
  SSD and runs the command inside a GC-rooted dev-shell profile there (never a fresh flake eval per call:
  that pattern filled the Mac's SSD twice). Host native on the Mac: `macos-aarch64`.
- **Toroidal World shaping (P0-06).** TW's `ShapeModule` path runs only from its Create World button
  and never on a dedicated server (TW 0.19.3 source, `client/shape/mixin/CreateWorldScreenMixin`), so
  Deep Time shapes its own worlds from `WorldDimensions.bake` (`WorldDimensionsShapeMixin` →
  `wrap/PlanetShape`) with TW's public `ShapeDimensions.withSpans`: X loops over C/16 chunks, Nether
  1:8, End unlooped. Verified on a dedicated planet server: stamped, loopsX, not loopsZ, 1024 chunks.
  TW's climate compression matches vanilla climate functions only, so it never applies to Deep Time's.
- **GameTest naming.** With `@PrefixGameTestTemplate(false)` a test is named by its lower-case method
  name without the class prefix, so `/test runall <class>` finds nothing; the planet harness runs
  tests by name (`--run name1,name2@x,y,z`).
- **MIC in dev runs (`-PwithMic`).** Sinytra Connector cannot start in a ModDevGradle dev run ("Could
  not determine clean minecraft artifact path"), so dev runs leave out Connector, Fabric-only jars and
  whatever hard-requires them (antique-atlas, surveyor); Distant Horizons 3.3.0 casts the server to
  `DedicatedServer` and crashes the GameTest server, so it is in `tools/smoke/profiles/mic.gametest-excluded`.
  Production-launched smoke servers keep the whole pack. `linkRunMods` links one of two byte-identical
  jars (as FML does) and fails only on differing duplicates.
- **Pack findings for Ben** (the pack is not modified; decisions log 2026-09-28: Deep Time changes to
  MIC go on a pack branch):
  1. `index.toml` at 303fd89 has a stale hash for `kubejs/server_scripts/destroy_metallurgy_integration.js`,
     so packwiz-installer rejects the pack ("Hash invalid!").
  2. Ritchie's Projectile Library is pinned twice (`mods/rpl.pw.toml` and
     `mods/ritchies-projectile-library.pw.toml`, same 2.1.2 jar under two names).
  3. `index.toml` lists three raw jars that `.gitignore` keeps out of git (`mods/destroy-1.21.1-0.4.1.jar`,
     `mods/Dice-neoforge-1.0.0.jar`, `mods/playingcards-2.0.1-neoforge.jar`): they exist only in the
     laptop's working tree, so a fresh clone of the pack cannot install. `install.sh mic` copies them
     from the working tree into its snapshot.
- **The Mac has no real git** (`/usr/bin/git` is Apple's stub): `mac.sh` keeps a GC-rooted nixpkgs git
  on the SSD, and the dev shell carries git.
- **P0-10.** `WorldDataTerrain` reads the world data through `ActiveWorldData`, filled by
  `WorldDataBinder` at `ServerAboutToStartEvent` (HIGHEST) for every Deep Time stem without a test
  terrain. `StrataPass` v0 paints horizon stacks over stone/deepslate after the pack's surface rules;
  `RockPalette` v0 takes the first member of `#deeptime:rock/<class>` (Deep Time ships the 20 vanilla
  entries of the MIC rock tags). `SpawnHandler` takes the first spawn candidate that is dry land.
- **Seam tests need TW's rules:** TW folds a non-player entity at the end of its own server tick
  (`ServerLevelMixin.tickNonPassenger` TAIL), so a test entity must move under its own motion; and a
  position past the edge must be folded with `ToroidalShape.fold` before use. A synchronous load of a
  seam chunk from a test can wait indefinitely in TW's seam-generation queue, so seam tests read only
  forceloaded chunks (`getChunkNow`). Two `/test run` at one position cancel each other: the harness
  spaces tests 8 blocks apart.
- **P0-14 / BlueMap on spacesheep.** spacesheep's Free plan caps a space at 200 files and a file at
  10 MB; BlueMap tiles of a 992-block world are several MB each before gzip, and a map's
  textures.json with all ~160 MIC jars is 10.4 MB gzipped. The page works by (1) gzip storage plus
  BlueMap's own `clientDecompression` switch (the web app inflates `.prbm.gz` with DecompressionStream,
  so no Content-Encoding header is needed), (2) a bounded render box (`BLUEMAP_BOX`), (3) only the jars
  whose blocks occur in the box (`BLUEMAP_ONLY_PRESENT=1`, via the new `deeptime-inspect blocks`), and
  (4) dropping the source map, PHP backend and non-English translations. `deeptime-inspect blocks` on
  the MIC world also shows what P1-15 must claim: Create's stone blobs (crimsite, ochrum, veridium,
  limestone) and ores from Create Nuclear, Destroy, Create Propulsion, Ice and Fire and Cobblemon
  scatter through Deep Time terrain today.
- **P0-15, Phase 0 exit (2026-09-28).** `tools/review/phase0.py` runs every check from the laptop
  through `mac.sh` (each server step takes the lock on its own) and publishes the page. First run
  10/11: two planet tests were disturbed by terrain, not the seam or the cap. `columnscontinuous` now
  counts only rows whose two seam columns match the periodic terrain function within 2 blocks (the
  planet's real router carves caves at the seam) and needs ≥ 10 clean rows; `prototypepushback`
  flies its probes near the build top so rising ground cannot stop them. Re-run: 11/11.
- **TW semantics for tests:** TW moves an entity that *steps* over the border (its movement path);
  `setPos` is a teleport and stays unfolded.
- **P0-07 (agent, merged):** see the P0-07 entry below. **P0-09 (agent, merged):** on-disk layout in
  `crates/deeptime-format/FORMAT.md`; climate layers are named after the router fields
  (`climate/{temperature,vegetation,continents,erosion,ridges}`); lake-test geometry in its manifest
  `notes.lake`; no `bodies/` yet.

### 2026-09-28 — P0-01 to P0-05 (session f981c0d3-5144-49ec-8686-2946d755872c)
- **Repo.** Created `MinecraftIndustrialComplex/deep-time` (private) and moved the docs in; `mic/Docs`
  now holds pointers. Dev shell builds (cargo-zigbuild with zig 0.13 is rebuilt on XiaServer the
  first time). nixpkgs now carries cargo-xwin 0.23.1, not 0.22.0 as the plan says.
- **Toroidal World is a bundle.** Its Modrinth file is a `lowcodefml` jar carrying the NeoForge and
  Fabric builds as jar-in-jar; the NeoForge build (`toroidal_world`, API `com.toroidalworld.api.v1`)
  nests MixinSquared. Gradle therefore compiles against the extracted inner jar
  (`extractToroidalWorld`) and every run loads the bundle from its `mods/` folder (`linkRunMods*`),
  exactly as players install it. `localRuntime` carries nothing.
- **Smoke.** `standalone` boots in about 30 s, peak RSS 1.97 GB; `mic` (168 server jars) boots in
  about 290 s, peak RSS 4.26 GB. TW compat configs that applied in MIC: aeronautics, c2me, create,
  distanthorizons, lithium, sable, simpleclouds. TW mixins count as recorded, not required: 235 of
  278 applied by `Done (` in MIC; the rest target classes an idle server never loads.
- **Pack finding.** The MIC pack's committed `index.toml` (303fd89) has a stale hash for
  `kubejs/server_scripts/destroy_metallurgy_integration.js`, so packwiz-installer rejects the pack
  from HEAD as well as from the working tree. `install.sh mic` installs from a snapshot re-indexed
  with `packwiz refresh` and records every hash it had to fix (`smoke/mic/install.json`).
- **Router design (plan §7.1 deviation, implementation detail).** The router is Deep Time's own
  noise settings (`deeptime:planet`, `deeptime:planet_nocaves`), generated from vanilla's by
  `tools/dev/gen_worldgen.py`, rather than derived in Java from the pack's `minecraft:overworld`
  (Tectonic replaces that router in MIC). Per-level binding (review 2, gap 4): the generator overrides
  `generatorSettings()` with a lazily bound copy whose `deeptime:*` functions read that level's
  terrain source; `ChunkMap` builds the level's `RandomState` from it. The surface rule is
  `deeptime:delegate` → the pack's `minecraft:overworld` rule, resolved when applied. Slides:
  −352→−328 and 624→640 (gap 8). `planet_nocaves` also has aquifers off: with them on, a local
  aquifer put barrier stone 3 blocks above an ocean floor and moved the measured surface.
- **GameTests.** The GameTest-server mixin passes only `deeptime:*` datapack stems (gap 9);
  `heightMatchesReference` max difference 0 on all 128 columns; `periodicAcrossSeam` 0 mismatches.
- **Blocker fix (gap 1).** `DeepTimeTestReporter` logs `[gametest-result]` lines on the planet
  server, and `/deeptime gametest summary` prints `[gametest-summary]` JSON that
  `gametest-planet.sh` polls.

### 2026-09-28 — P0-07 (sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c)
- **Built and measured on the Mac mini** (`tools/remote/mac.sh`), so the host native there is
  `macos-aarch64` / `libdeeptime_jni.dylib` (1,055,136 bytes). `cargo test --workspace`, clippy
  `-D warnings`, `fmt --check`, `deny check licenses bans` all ok; `gradlew build test` →
  `tests=18 failures=0 errors=0` (NativeLoaderTest 9, NativeLinkTest 4, PlatformKeyTest 4, Sanity 1);
  `java -cp <jar> tools/natives/NativeSmoke.java <fresh dir>` → `status=OK version=0.0.1 format=0.0
  selftest=1997318690096807936`, and each forcing property → `status=FAILED class=<its class>`,
  exit 0 every time. The Linux paths (PT_INTERP, gnu/musl order, glibc floor scan) are unit-tested
  with synthetic ELF files and a Groovy check; CI's ubuntu Gradle job is their first real run.
- **The five failure classes and their guidance** (`FailureClass.guidance`, plan §6.6):
  - `UNSUPPORTED_PLATFORM`: "Deep Time has no native library that runs on this platform (<key>).
    Existing Deep Time worlds still play. To create a new one, run `deeptime run` (the Deep Time CLI)
    on a supported machine and copy its deeptime/ folder into the world folder."
  - `LIBC_MISMATCH`: "The native library needs a C library this system does not provide (<error>).
    Install the musl build, or update glibc (the floor is 2.17)." + the CLI route.
  - `ANTIVIRUS_SUSPECTED`: "The extracted native library vanished or changed between writing and
    loading, which is what antivirus quarantine looks like. Add an exclusion for <dir>, or start the
    game with -Ddeeptime.natives.path=<a copy of the library>."
  - `IO_ERROR`: "Could not create, lock, write or move files in <dir> (<exception>). Check that the
    folder is writable and the disk has space."
  - `VERSION_MISMATCH`: "The native library does not match this Deep Time (<what differs>).
    Reinstall the mod; a stray -Ddeeptime.natives.path library is the usual cause."
- **Implementation details.** `natives.json` schema 1 is now in plan §6.5 (gap 14); the format
  version's one source is `Cargo.toml` `[workspace.metadata.deeptime]`, read by `deeptime-format`'s
  new `build.rs` and by Gradle with the same line scan. `NativeLoader` is JDK-only (gap 13): a
  built-in JSON reader (`MiniJson`) and a `NativeLogSink` callback that `DeepTime` points at SLF4J.
  `forkEvery = 1` is on `test` (gap 17), so CI's single results folder holds everything; the fourth
  extraction case is four concurrent extractions into one directory (exactly one writes).
  jni-rs 0.22.4 matched the plan's sketch (`EnvUnowned`, `with_env`, `resolve::<P>()`); the policy's
  `Captures` must be `()` for `resolve`. Extra entry-point rules: `drainLog(0)` drains the
  process-wide ring; `destroy` and `drainLog` still work on a poisoned session. The dummy preset
  takes two optional test keys, `reserve_bytes` (memory cap) and `panic_at_step` (panic guard).
  `SimSession.start` creates the native session on the caller's thread so a bad preset throws at
  once; the worker thread advances, finalizes and destroys. `serde_json` joins the workspace.
- **Next.** Ben runs `./gametest.sh` for `NativeGameTests` (log line
  `[gametest] native version=… format=… target=… selftest=…`); then P0-08.

## Known issues and open questions
- **Mountain heights (2026-10-01).** The 13–20 km "mountains" were bookkeeping (fixed, stage 1); the height limit is
  now crust spreading over its strength (stage 2, `surface/spread.rs`), with the plate model's cap a 30 km guard that
  never binds; the stored elevation is squeezed under `max_y` only for extreme settings. Open: sea floors below
  `min_y` in under 0.1 % of the sea, summits at 6–7 km, the full preset's land still high (pending question above;
  build log 2026-10-01 stage 2).
- **One preset, one planet per platform (2026-10-01).** The simulation is not bit-identical across platforms (the
  platform's libm); it is across runs and thread counts on one machine. Ben does not want it otherwise. Tests on a
  simulated planet assert properties and floors measured over several planets (`DEEPTIME_TEST_PLANET_SET="seed=N"
  ./gametest.sh`); an exact number belongs only to tests on baked data.
- **P1-06 caveats.** (1) A checkpoint carries the preset it was made with, defaults included: to pick up
  a changed default of a stage that has not run, resume with `--set "amplify={}"` (or the key). (2) Tests
  that derive a small planet from a `pipeline_*` preset must remove the `young` and `amplify` sections
  (their L2/L3 are far finer than a test's L1); every existing helper does. (3) The surface stage keeps its
  L1 scratch after its last step, so the `young`/`amplify` estimates (world + own) exceed 4 GiB on the
  full presets although the gauge reads ~1 GiB there; P1-17 could drop it. (4) The amplifier's working
  set is 46 B a cell (67 B while a tile runs), not the plan's 20.
- **Long runs (parked by Ben, 2026-09-29: "DW about it for now"; memory part solved 2026-09-29, history spill).** The
  recorded history no longer has to fit in memory: over the cap its oldest frames go to a spill file in the run's folder
  and the run gets slower instead of failing (build log 2026-09-29, "the recorded history spills to the SSD"). The 1000
  Myr `pipeline_full_32k` run that failed at 4 GiB now finishes under 4 GiB (40 min on a loaded Mac, 2.3 GiB of disk);
  `pipeline_quick_16k` 2000 Myr finishes under 4 GiB (24 min, 3.8 GiB gauge peak). A 4,600 Myr 32k full run needs its history on disk (~11 GiB, 12.7 B per L1 cell per frame) plus a base
  of ~4.2 GiB (the amplifier; ~3 GiB before it), so it fits an 8 GiB cap now; its time (~3 h) is untested. Still open:
  the physics is tested only over 500 Myr and starts from existing continents (no early-Earth phase); the young and
  amplify stages allocate ~0.3 GiB at 32k without reserving it.
- The workbench offered the placeholder `earthlike_*` presets (dummy pipeline) next to the real `pipeline_*`
  ones; Ben ran one and got a single blob. The UI overhaul hides them behind a dev toggle and marks a
  placeholder run (build log 2026-09-29, UI overhaul). Since P1-08 the `earthlike_*` built-ins are the real
  pipeline presets (no placeholder is built in); the workbench lists both families, identical.
- The plan is at revision 2 plus a partial revision 3 (the decisions-log rule in §1.2). The 29 gaps of
  docs/plan-reviews/2026-09-28-cold-read-review-2.md are fixed in the plan by the task that hits each
  one; the build log names each fix as it lands.
