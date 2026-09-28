# Deep Time — status

Phase: 0 · Next: P0-06 rest, P0-11, P0-13, P1-01, P1-02 and the seam page (agents), then P0-12 (me), P0-14, P0-15 · Updated: 2026-09-28 (session f981c0d3-5144-49ec-8686-2946d755872c)
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
| P0-06 | Toroidal World spike (server-side items) | doing | 2026-09-28 | — | planet server (Mac): probe tw stamped/loopsX/!loopsZ/1024 chunks; columnscontinuous PASS (16 rows, 0 bad folds, 0 jumps); entitycrossesseam PASS (x=-8191.79 after crossing). Left: Sable jetCrossesSeam 20/20, sable_straddle, structure/shaft tests, pole prototype, smoke at 16k/32k with region span check, review page |
| P0-07 | JNI hello world, native loader and the five failure classes | done | 2026-09-28 | c813177 | Mac: JUnit tests=30 failures=0; NativeSmoke OK + 5 forced classes; ./gametest.sh NativeGameTests pass: native version=0.0.1 format=0.0 target=aarch64-apple-darwin selftest=1997318690096807936. Linux load path first runs in CI |
| P0-08 | Cross-builds and the OS load matrix | done | 2026-09-28 | b20fbb8 | Mac: `nix run .#natives` → 8 libraries, gnu GLIBC_2.17, musl NEEDED libc.so only; `gradlew build -Prelease` → 8 in the jar, tests=30 failures=0; [natives.yml run 36492319610](https://github.com/MinecraftIndustrialComplex/deep-time/actions/runs/36492319610): all 9 jobs success, 8/8 load jobs `status=OK version=0.0.1` (Windows arm64 included) |
| P0-09 | World-data format v0, synthetic planets, Java reader | done | 2026-09-28 | 75d90d3 | Mac: cargo test ok (format 15 tests); synth wrap-test 16k → format_version 0.0, viewer bundle; TilePackParityTest + Lz4RoundTripTest pass; wrap-test equals AnalyticTestTerrain within 1/32 block |
| P0-10 | The generator on world data | done | 2026-09-28 | — | Mac ./gametest.sh → All 10 required tests passed: heightMatchesRaster cornerMax 0 maxAbsDiff 1; strataClassesResolve checked=32 wrong=0; spawnOnLand y=456 > 63. StagingMover deferred to P1-11 (no staging before the editor exists) |
| P0-11 | `deeptime-inspect` | done | 2026-09-28 | 86140f2 | Mac: cargo test -p deeptime-inspect ok (19 unit + 7 fixture); `cargo run -p deeptime-inspect -- compare --world smoke/standalone/server/planet --world-data …/deeptime` → p99_abs_diff_blocks 1 (p50 0, max 5, 99.6 % within 1 block; 109,184 of 128,000 columns compared); seam 144 rows, 0 mismatches; clippy, fmt, deny ok |
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
  test. `section` prints `water_top_y=<y> surface_y=<y+1> columns=<n>`: sea level 63 gives
  `water_top_y=62`, so P0-13 must say whether its lake at `water_level` 120 means 119 or 120.
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
