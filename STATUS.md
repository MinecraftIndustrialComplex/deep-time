# Deep Time — status

Phase: 1 (Phase 0 exit passed 2026-09-28) · Landed 2026-09-29: P1-05 climate, P1-06 young landscape + amplification, P1-07 rock/strata, P1-14 deposits on the recorded history, P1-17 partials (parallel SPL, calibration with real climate), P1-20 workbench stages 1–2 + dark UI · In flight: history spill to SSD, diffusion loop fusion, workbench memory-cap refusal · Next: P1-08 bake (critical path), deposit formula (survivors) + MVT regional driver · Sub-agents: Sonnet 5.5 for well-specified work, Opus for hard work (Ben, 2026-09-29) · Updated: 2026-09-29 (session f981c0d3-5144-49ec-8686-2946d755872c)
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
| P1-08 | Bake, world data v1 and parity | doing | 2026-09-29 | | Agent worktree agent-a09db47c6d05e1417 (sub-agent of session f981c0d3-5144-49ec-8686-2946d755872c), branch worktree-agent-a09db47c6d05e1417 |
| P1-09 | Climate mapping and biomes | todo | | | |
| P1-10 | TerraBlender regions follow provinces | todo | | | |
| P1-11 | Customize editor, timelapse and staging | doing | 2026-09-28 | d42375e | Screens, models and staging built on the placeholder presets (real stages pending). Mac, merged with master f14d258 (P1-03, P1-18): JUnit tests=101 failures=0 (new: EditorModelTest, TimelapseModelTest, ModelHelpersTest, StagingMoverTest, TimelineAcrossJniTest, ScreenSourcesTest); ./gametest.sh → All 31 required tests passed (EditorModelGameTests acceptBuildsGenerator, stagingMovedOnStart, editorRunsAcceptsAndBinds, shareCodeThroughModel, realPresetFieldsLightUp); cargo fmt/clippy -D warnings/test --locked (193 passed)/deny clean; gradlew build clean. Client (laptop, HeadlessMC + Xvfb): whole walk Create World → editor → run → review → Accept → world created from the moved staging, no crash; guidance panel with natives forced off; screenshots in review/out/p1-11-client. Rerun with the real presets after P1-08; biome names on the review page with P1-09 |
| P1-12 | Strata pass with rock tags and `rock_states` | done | 2026-09-28 | — | Mac: ./gametest.sh and -PwithMic (+ MIC rock tags via -Ddeeptime.extraDatapack) → All 21 required tests passed each; coal stone / black argillite; Create stones in their classes; loess = projectatmosphere:dust[layers=8]; everyClassInTheWorld 59/59 classes, 240 samples 0 wrong; MIC smoke probe palette 59/59 (member 24, fallback 35); JUnit 46/0; bodies deferred (P1-08/P1-14) |
| P1-13 | Dedicated server: generate on first start | doing | 2026-09-28 | 743b69d | Mechanism verified on placeholder presets (dummy pipeline + wrap-test bake): standalone and MIC first starts generate in ~12 s with progress lines ≤ 5 s apart, restart does not regenerate; no-config / circumference-mismatch / native-failure all stop cleanly before loadLevel. Rerun with the real presets after P1-08 |
| P1-14 | Deposit engine: the four families | done | 2026-09-29 | — | On the real recorded history (plates → surface → structure → deposits, `pipeline_{quick,full}_{16k,32k}` have a `deposits` section). Mac: cargo test --workspace ok (core 155 + 1 ignored incl. deposits::recorded_tests 6: 1 vs 3 threads give identical sites/bodies/events/census, a checkpoint inside the deposit stage resumes on another thread count to the same sites, favourability of a rigidly drifting crust = the fixed-crust map moved cell for cell, missing fields skip exactly the types that read them and change nothing else, every family on recorded frames; cli 21 incl. `stage deposits` on the real history; format 32, inspect 28, jni 7, viewer 5); clippy -D warnings, deny licenses bans clean; gradlew build test 118/0; fmt clean for every file this task touches (watch/editor.rs on master is not); `stage deposits --preset pipeline_quick_16k --start scattered_continents --length-myr 500 --viewer review/out/p1-deposits-real` → 154 s (plates 30.0, surface 111.6, structure 5.0, deposits 4.4), 5 sites at count_mult 1 (porphyry 1, coal 4; MVT, SEDEX, petroleum expected 0.00 / 0.00 / 0.11), 42 at count_mult 10; 17 MB bundle. Ben reviews: ore provinces, site list, three chains; see the build log for what is empty and why |
| P1-15 | Deposits in the world | doing | 2026-09-28 | — | On the synthetic deposit world (real fields wait for P1-03…P1-08). Mac: ./gametest.sh, -PwithEv, -PwithMic → All 30 required tests passed each (DepositGameTests body 1222/1222 ore exact, 0 missed, 0 unexplained, regeneration 0 of 44,880 differ; ClaimGameTests; SpawnGameTests); ./gametest-planet.sh claimoninplanetworld PASS standalone (19 removed) and -PwithMic (38 removed); JUnit 62/0 incl. DepositDataParityTest (19,649 body samples, worst Δ 3.3e-16); smoke standalone first-start deposit world PASS + `deeptime-inspect ore` render |
| P1-16 | COE and CDG hooks | done | 2026-09-28 | — | Mac: `./gametest.sh` and `-PwithMic` → All 45 required tests passed each; CompatGameTests with COE 1.6.8 + CDG 1.3.15: 7 of 7 site chunks = the site's vein in COE, 30 of 30 claimed noise veins dropped (COE picks them in the overworld), 34 unclaimed kept, 33 stored randomMul = the site's quantile (24 = Rust's drawn quantile, worst Δ 2.8e-8), 16 trap chunks = charge × cdg_scale × share, 111 chunks outside traps 0. MIC smoke (16k deposit world) PASS: `probe coe` at 9 site chunks = the site's vein (agrees), `probe cdg` over trap 27 > 0 (infinite / 553,979 mB), 0 elsewhere; `/coe locate` copper not found, redstone found. JUnit 106/0; cargo fmt/clippy/test/deny clean. Driven by the synthetic deposit world (see build log) |
| P1-17 | Benchmark gate and preset tuning | doing | 2026-09-29 | | Partial (surface parallelism, branch worktree-agent-a9bc0ab99e68b4a07): stream-power solve 34.0 → 6.5 s at 12 threads, bit-identical; flood left serial on purpose; diffusion is now the largest surface cost; see the build log entry "P1-17 partial". Partial (calibration with the real climate, branch worktree-agent-ae48007457b607716, Ben 2026-09-29): pipeline presets + surface/climate/structure defaults + four deposit balance multipliers; Mac: `stage deposits --preset pipeline_quick_16k --length-myr 500` scattered / supercontinent → land median 2,352 / 1,743 → 601 / 624 m (Earth 427), above 2 km 0.56 / 0.45 → 0.11 / 0.07 (Earth 0.10), land 0.233 / 0.215 → 0.270 / 0.282, Köppen E 0.53 / 0.64 → 0.18 / 0.23, glaciated 0.31 (feedback off) → 0.06 / 0.07 (on), sites porphyry 1 / 1 → 5 / 34 (median 8 over 3 seeds × 2 starts), SEDEX, MVT, oil on 4, 3, 5 of 6 planets, plutons 7,830 / 5,374 → 1,192 / 885 bodies, mass residual ≤ 1.9e-3 km³; Earth check 19/24 (unchanged), 9.7 % glaciated with the feedback; cargo test/clippy/fmt/deny clean, gradlew build test 118/0; bundles review/out/p1-calib-{scattered_continents,supercontinent} (20 / 15 MB, not published); see the build log entry "P1-17 partial: calibration" |
| P1-18 | Pole cap and client info | done | 2026-09-28 | — | Mac: JUnit 65/0; ./gametest.sh All 26 passed; planet PASS: entitypushedback, hardstopat80, sablehullpushedback, worldinfochannelrequired + seam set + sable_straddle; refuse_client OK (NeoForge client without Deep Time refused naming deeptime:world_info + pole_storm; with it accepted; vanilla refused). Client code compiled only (see build log) |
| P1-19 | Phase 1 exit | todo | | | |
| P1-20 | Browser workbench on the Mac (log 2026-09-28): stage 1 editor controls + live runs + compare; stage 2 click-to-inspect and 3D block preview | doing | 2026-09-28 | | Both stages in parallel (Ben, 2026-09-28): stage 1 grows out of the live viewer; stage 2 runs the real generator headless on the Mac and exports the area (`deeptime-inspect export-area`), so it does not wait for P1-08; a fast Rust column path joins after P1-08. **Stage 1 built** (branch worktree-agent-a3abc4a868f0c3b81): `tools/viewer/mac-watch.sh` → http://mac-mini:8642/ (tailnet only); Mac: cargo test --workspace, clippy -D warnings, fmt, deny clean, JUnit 107/0; two real 500-Myr pipeline_quick_16k runs watched live (the scattered run equals P1-04's batch numbers: 1,238 events, land_frac 0.215), compared side by side, screenshots looked at; stage 2 hooks are 501 stubs (see build log). **Stage 2 built** (branch worktree-agent-a63e3a8af506069de): `deeptime-inspect export-area` + `tools/workbench/preview.py` (real generator headless on the Mac) + `tools/review/pages/area3d/` viewer, wired as `deeptime watch --preview`; 256 × 256 × 992 boxes in 44 s (standalone) / 90 s (MIC), verify 0 wrong, 6/6 probe site agree; until P1-08 a preview is the synthetic deposit world at the box's lat/lon and says so (see build log). **UI overhaul** (branch worktree-agent-a95571be05fdd89d6, Ben: "should be dark themed"): dark map-first layout, run cards, sectioned editor with the estimate by Run, split/side-by-side compare, timeline dock, stage-2 job cards, `?tv` HUD, 390 px; placeholders hidden; Mac gates clean, JUnit 118/0; before/after in review/out/workbench-ui/ (see build log) |

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
| p1-deposits (deposit engine on synthetic fields, Why-here chains) | https://spacesheep.dev/@benmac/deeptime-p1-deposits | P1-14 first pass 2026-09-28 | 2026-09-28 |
| p1-deposits-real (P1-14: deposits on the real 500 Myr history; not published, review/out/p1-deposits-real on the laptop and the Mac, 17 MB) | — | P1-14 real history 2026-09-29 | 2026-09-29 |
| p1-amplify (P1-06: hill-shaded before/after views of L3 with a slider, D8 rivers of each terrain, the change, the tile corner and the seam, the drama slider; not published, review/out/p1-amplify on the laptop and the Mac, 16 MB) | — | P1-06 amplification 2026-09-29 | 2026-09-29 |
| p0-bluemap (3D render on MIC blocks) | https://spacesheep.dev/@benmac/deeptime-p0-bluemap | P0-14 mountain render 2026-09-28 | 2026-09-28 |
| q10-ev (Q10 evidence: the same deposits in MIC without and with Excavated Variants; cut screenshots, per-host ore table, costs, trade-offs) | https://spacesheep.dev/@benmac/deeptime-q10-ev | Q10 evidence 2026-09-28 | 2026-09-28 |
| q10-map-plain (BlueMap, MIC without EV: 3 sites, surface + cut maps; linked from q10-ev) | https://spacesheep.dev/@benmac/deeptime-q10-map-plain | Q10 map without EV 2026-09-28 | 2026-09-28 |
| q10-map-ev (BlueMap, MIC with EV + the EV bridge fix; linked from q10-ev) | https://spacesheep.dev/@benmac/deeptime-q10-map-ev | Q10 map with EV, EV bridge fix (smooth basalt) 2026-09-28 | 2026-09-28 |

## Pending questions
Questions asked of Ben and not yet answered. Answers go into the decisions file's log
(docs/deeptime-design-decisions.md, last section), never here.

| Date | Task | Question | Asked how |
|---|---|---|---|
| 2026-09-28 | P1-16 | **COE vein size:** finite, tonnage-sized veins (`defaultInfinite = false` on MIC's deep-time pack branch) or keep COE's infinite default? Ben (log 2026-09-28): decide later, after P1-08, with numbers from the real presets. | AskUserQuestion, answered "Decide later" |

## Build log
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
- **P1-06 caveats.** (1) A checkpoint carries the preset it was made with, defaults included: to pick up
  a changed default of a stage that has not run, resume with `--set "amplify={}"` (or the key). (2) Tests
  that derive a small planet from a `pipeline_*` preset must remove the `young` and `amplify` sections
  (their L2/L3 are far finer than a test's L1); every existing helper does. (3) The surface stage keeps its
  L1 scratch after its last step, so the `young`/`amplify` estimates (world + own) exceed 4 GiB on the
  full presets although the gauge reads ~1 GiB there; P1-17 could drop it. (4) The amplifier's working
  set is 46 B a cell (67 B while a tile runs), not the plan's 20.
- **Long runs (parked by Ben, 2026-09-29: "DW about it for now").** A real 4,600 Myr run (timeline max, plan
  §5.1) is estimated by the workbench at ~45 min / 3.8 GB (pipeline_quick_16k) and ~3 h / 14.6 GB
  (pipeline_full_32k, too big for the 16 GB Mac): the deposit engine's recorded history (a frame every
  10 Myr) is held in memory and grows with length. The physics is tested only over 500 Myr and starts from
  existing continents (no early-Earth phase). Options offered: history on disk (flat memory), an early-Earth
  phase, or a 1 Gyr cap. Revisit in P1-17 or when Ben raises it. Since the memory-cap fix (build log,
  2026-09-29) the workbench refuses a run whose estimate plus 512 MiB is over its cap (default 8 GiB) instead
  of failing partway, so a run like this now says why before it starts.
- The workbench offered the placeholder `earthlike_*` presets (dummy pipeline) next to the real `pipeline_*`
  ones; Ben ran one and got a single blob. The UI overhaul hides them behind a dev toggle and marks a
  placeholder run (build log 2026-09-29, UI overhaul).
- The plan is at revision 2 plus a partial revision 3 (the decisions-log rule in §1.2). The 29 gaps of
  docs/plan-reviews/2026-09-28-cold-read-review-2.md are fixed in the plan by the task that hits each
  one; the build log names each fix as it lands.
