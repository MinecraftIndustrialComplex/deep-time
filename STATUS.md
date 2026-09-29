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
| P0-11 | `deeptime-inspect` | todo | | | |
| P0-12 | Terralith, TerraBlender and Biomes O' Plenty checks | done | 2026-09-28 | 00db177 | Mac: ./gametest.sh default/-PwithMic/-PwithTerraBlender → All 13 passed; terralith=3008 of 4096 land samples (MIC), biomesoplenty=1325 of 4096 (TB), surface painted 24/24; smoke mic --synth wrap-test (production server) PASS: probe tw stamped/loopsX/!loopsZ/1024, locate village found, no C2ME/DH errors, chunky done, regions outside span 0 |
| P0-13 | Water above sea level holds | todo | | | |
| P0-14 | BlueMap renders on MIC blocks | todo | | | |
| P0-15 | Phase 0 exit | todo | | | |
| P1-01 | Session, checkpoints, keyframes, event log and site formats | todo | | | |
| P1-02 | Fibonacci sphere, triangulation and raster coupling | todo | | | |
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
