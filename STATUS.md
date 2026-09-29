# Deep Time — status

Phase: 0 · Next: P0-11, P0-13, P1-01, P1-02 and the seam page (agents), then P0-12 (me), P0-14, P0-15 · Updated: 2026-09-28 (session f981c0d3-5144-49ec-8686-2946d755872c)
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
| P0-15 | Phase 0 exit | todo | | | |
| P1-01 | Session, checkpoints, keyframes, event log and site formats | done | 2026-09-28 | — | Mac: cargo test -p deeptime-format 27 ok (segments + sites round-trip, two writers merge with no id collision); run --preset dummy.json → manifest.json, events/1-0.dtev, keyframes/index.json (13 events, 9 keyframes, 3 sites); test --workspace, fmt, clippy -D warnings, deny licenses bans clean; gradlew build test tests=34 failures=0 and ./gametest.sh all 14 pass (after merging master 2f81ae5) |
| P1-02 | Fibonacci sphere, triangulation and raster coupling | done | 2026-09-28 | 0ea8a1b | Mac: cargo test -p deeptime-core sphere:: → 23 passed (V−E+F=2; every 16k-quick L1 cell: weights sum 1 ±1e-6, ≥ −1e-9, containing triangle; k-d nearest = brute force on 10,000 points); sphere_bench 250k: hull 255 ms (12 threads; 280 ms on 1), k-d tree 8 ms, L1 2048×1024 coupling 41 ms, 0 bad cells on all four L1 grids |
| P1-03 | Plates and crust | todo | | | |
| P1-04 | Deep-time surface processes on L1 | todo | | | |
| P1-05 | Climate | todo | | | |
| P1-06 | Young landscape, amplification and the detail layer | todo | | | |
| P1-07 | Rock classes, column strata and metamorphism | todo | | | |
| P1-08 | Bake, world data v1 and parity | todo | | | |
| P1-09 | Climate mapping and biomes | todo | | | |
| P1-10 | TerraBlender regions follow provinces | todo | | | |
| P1-11 | Customize editor, timelapse and staging | todo | | | |
| P1-12 | Strata pass with rock tags and `rock_states` | todo | | | |
| P1-13 | Dedicated server: generate on first start | todo | | | |
| P1-14 | Deposit engine: the four families | todo | | | |
| P1-15 | Deposits in the world | todo | | | |
| P1-16 | COE and CDG hooks | todo | | | |
| P1-17 | Benchmark gate and preset tuning | todo | | | |
| P1-18 | Pole cap and client info | todo | | | |
| P1-19 | Phase 1 exit | todo | | | |

## Review pages
| Name | URL | Version label | Date |
|---|---|---|---|
| seams (how the seams work, interactive) | https://spacesheep.dev/@benmac/deeptime-seams | seams v2 2026-09-28 | 2026-09-28 |
| p0-bluemap (3D render on MIC blocks) | https://spacesheep.dev/@benmac/deeptime-p0-bluemap | P0-14 mountain render 2026-09-28 | 2026-09-28 |

## Pending questions
Questions asked of Ben and not yet answered. Answers go into the decisions file's log
(docs/deeptime-design-decisions.md, last section), never here.

| Date | Task | Question | Asked how |
|---|---|---|---|

## Build log

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
  column's ground and its surface; `outside` is any water in a box 48 blocks past the footprint,
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
- The plan is at revision 2 plus a partial revision 3 (the decisions-log rule in §1.2). The 29 gaps of
  docs/plan-reviews/2026-09-28-cold-read-review-2.md are fixed in the plan by the task that hits each
  one; the build log names each fix as it lands.
