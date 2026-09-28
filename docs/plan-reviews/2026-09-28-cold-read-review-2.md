# Deep Time implementation plan: cold-read review 2

Reviewed 2026-09-28 by a fresh Opus agent. I had only the plan
(`/home/greencheetah/Projects/mic/Docs/deeptime-implementation-plan.md`, revision 2, 3,278 lines), the
decisions file (updated 2026-09-28, 106 lines), the first cold-read review, and the files these list. I
followed §1.3's reading order and §11.1's checklist. I walked P0-01 to P0-07 in detail as if about to
implement them, and P0-08 to P0-15 briefly.

This was a dry run. I created no repository, wrote no code and ran no builds. Read-only lookups were
split between me and two Sonnet sub-agents. They covered the crates.io, npm, Modrinth and GitHub APIs,
single-attribute `nix eval`s, and the NeoForge 21.1.249 sources jar (files extracted into the session
scratchpad). Line numbers refer to the plan unless another file is named.

## What checked out

| Claim | Evidence |
|---|---|
| Every absolute path exists, apart from `mods/deep-time`, which the plan says to create | `test -e` on all 19 distinct paths, including the nine research notes and every mic-climate house-style file in §1.3 item 4 |
| Every internal `§` reference resolves | Script: every `§n[.n[.n]]` matches a heading. The one miss, `§0`, is `wrapped_planet_worlds.md` §0, which exists. The research-note anchors also exist: wrapped_planet §0–§6.3, aeronautics §3–§8 (six pitfalls in §8), MIA §1–§5, surface §1–§7 |
| Task dependencies agree with the work order | Every "Depends on" in P0-01 to P0-15 points to an earlier ID. P0-08 is correctly marked as parallelisable |
| **GameTest dimension mixin** | `GameTestServer.create` lines 94–103 bake the flat preset against `new MappedRegistry<>(LEVEL_STEM).freeze()`. `WorldDimensions.bake` takes `stemRegistry` entries first and the preset's second. `WorldLoader.DataLoadContext` has `datapackDimensions`. Line 103 is the only `bake` call in `GameTestServer`, and it sits in a static lambda whose only `DataLoadContext` argument is the lambda's own, so `@Local(argsOnly = true)` is unambiguous. `GameTestServer.initServer` posts `ServerAboutToStartEvent` before `loadLevel()` (lines 154–155), so the binder runs there too |
| MixinExtras | `neoforge-21.1.249.pom` (Gradle cache) declares `io.github.llamalad7:mixinextras-neoforge:0.5.3`, scope compile |
| NOISE-status heightmaps | `ChunkStatus` lines 20–31: EMPTY through SURFACE carry `WORLDGEN_HEIGHTMAPS` (OCEAN_FLOOR_WG, WORLD_SURFACE_WG), and CARVERS onward carry `FINAL_HEIGHTMAPS`. `NoiseBasedChunkGenerator.doFill` updates both WG maps (lines 355–410). `OCEAN_FLOOR_WG` uses `MATERIAL_MOTION_BLOCKING`, so water is skipped |
| `WorldCarver.carveBlock` has no neighbouring-water check | `WorldCarver.java` lines 146–190 check only `canReplaceBlock` (the carver's replaceable tag) and ask the aquifer |
| `AddPackFindersEvent.addPackFinders(ResourceLocation, PackType, Component, PackSource, boolean, Pack.Position)` | Signature matches exactly (line 71). The path goes through `getOwningFile().getFile().findResource(path)`. `GameTestServer.create` enables `packRepository.getAvailableIds()` (line 82) |
| GameTest details | Pass lines at `GameTestServer` lines 185 and 190. `@GameTest` has `attempts` and `requiredSuccesses`. `PrefixGameTestTemplate` exists. "Missing test structure" is thrown at `StructureUtils` line 108. `/test runall <testClassName>` and `runmultiple <testName> <amount>` exist. `/test` is registered when `GameTestHooks.isGametestEnabled()` (`Commands` line 217). mic-climate uses the `empty_5x5x5` template |
| Server start order | `DedicatedServer.initServer`: `handleServerAboutToStart` (191) → `loadLevel` (193) → RCON (209) → `handleServerStarting` (230). `ServerLifecycleHooks.handleServerAboutToStart` runs `runModifiers` before posting the event (lines 92–98). `LevelEvent.CreateSpawnPosition` is cancellable |
| spacesheep | npm `latest` is 1.17.0 (published 2026-09-28). Every `deploy` flag the plan lists exists. Visibility defaults to private. `.spacesheep.json` holds `space` and `url` (and `page_title`). The title-mismatch guard exists (and existed in 1.16.0). `list` prints visibility. `read` and `versions` exist |
| Rust toolchain | Stable is `1.98.1 (48a229cea 2026-09-01)`. rust-overlay has `manifests/stable/1.98.1.nix`. The glibc floor is still 2.17 |
| jni-rs 0.22.4 | Released 2026-03-16, MIT OR Apache-2.0, MSRV 1.85. `EnvUnowned::with_env` wraps `catch_unwind` and returns an `EnvOutcome` whose `resolve::<P: ErrorPolicy>()` exists. `Env::get_direct_buffer_address` exists. The `#[unsafe(no_mangle)] extern "system"` + `EnvUnowned<'l>` pattern is the documented one |
| zig override for cargo-zigbuild | `cargo-zigbuild.postInstall` is `wrapProgram … --prefix PATH : …-zig-0.16.0/bin`. `override.__functionArgs` has `zig`. Current `github:NixOS/nixpkgs/nixos-unstable` (rev `e158d9e`) still has `zig_0_13` 0.13.0, `cargo-zigbuild` 0.23.4 and `jdk21` 21.0.12.1+1 |
| Other pins | Every crate pin exists and is the current max_stable. kiddo 6.3.0's MSRV of 1.89 is the highest. cargo-deny 0.18.4's pedantic-GNU changelog entry exists. It prints `licenses ok` / `bans ok`. junit-bom 5.14.4 is the last 5.x, and JUnit 6 needs Java 17. ModDevGradle 2.0.74 exists, and its README documents `unitTest { enable(); testedMod = … }` and `ModModel.sourceSet`. nixpkgs versions for jdk21, jdk25, nodejs_22, cargo-xwin 0.22.0 and llvm 21.1.8 all match |
| Modrinth ids | TW `8YTSYi9y` (0.19.3+mc1.21.1, fabric+neoforge, MIT, no dependencies), Sable `U678xqle`, Aeronautics `44pLdPGg`, Chunky `LuFhm4eU`, TerraBlender `6e8GCrLb`, BOP `BtZKRp69` (requires GlitchCore and TerraBlender), GlitchCore `S2TfWrZR` and EV `krUiU6UD` (requires DAG) all match. `maven.modrinth:toroidal-world:8YTSYi9y` resolves: the POM returns 200 and the jar 307s to the CDN |
| TW source | `3c1f3a30` is the head of `mc/1.21`, and there is no `maven` branch. `docs/modding.md`, `docs/dedicated-server.md` and `api/v1/shape/{ShapeModule,ShapeDimensions,LoopSpans}.java` exist. `ShapeModule.of(id, default, Apply, Read)`, `ShapeDimensions.withSpans(dims, ow, nether, end, options)` and `LoopSpans.ofWidth(Axis, int)` match §7.10's snippet. `CreationShapeTest.aNoiseSubclassOverworldKeepsItsClassAndTakesTheShape` exists |
| Pack pins | `pack/mods/*.pw.toml` pin Tectonic 3.0.28, Lithostitched 1.8.0+beta6, Terralith 2.6.2, C2ME .122, DH 3.3.0, COE 1.6.8, CDG 1.3.15, Sable 2.0.5, Aeronautics 1.3.2, Create 6.0.10, Chunky 1.4.23 and GlitchCore 2.1.0.2, as §7 and §13 say. The pack has no TerraBlender and no TW |
| GitHub | The org is on the free plan with `total_private_repos: 0`. `deep-time` does not exist yet. `macos-15-intel` and `windows-11-arm` labels exist. BlueMap v5.28 has `bluemap-5.28-cli.jar`, and its `file.conf` documents `compression: gzip|zstd|deflate|none`. `gh` uses SSH for git, so pushing `.github/workflows/*` does not need the token's missing `workflow` scope |
| Rock-tag datapack | `rock_fallback/coal.json` is black argillite (`required: false`), then `minecraft:stone`. There are 77 `rock/` tags (59 classes, 17 groups and `unclassified_crust`) and 43 fallback tags |

## Gaps

Severity: **blocker** means a task cannot pass as written. **Major** means it would likely go wrong or
force an unrecorded design decision. **Minor** means friction, drift or a missing detail.

### Blocker

**1. [blocker] The planet-server harness cannot read GameTest results: in 1.21.1 a server with no players logs no passes and no summary.**
§9.4 item 2, lines 2066–2079 ("parses the results from the log"). This hits the acceptance of P0-06
(2445–2447), P1-15 (2881), P1-16 (2894) and P1-18 (2921), and the Phase 0 exit (2627).
- *Evidence.* `/test runall` is asynchronous. It answers only "Running N tests..." (`TestCommand`
  line 603), and its end-of-run summary ("All required tests passed :)", "N required tests failed :(",
  lines 708–710) goes through `say(ServerLevel, …)`. That function sends to
  `serverLevel.getPlayers(...)` only (line 467–468). Per-test passes ("… passed!", "passed N times of M
  attempts") use the same player-only `say` in `ReportGameListener` (lines 115 and 190–191). The log
  hook `GlobalTestReporter.onTestSuccess` delegates to `LogTestReporter`, whose `onTestSuccess` is
  empty (lines 20–22). Only failures reach the log (`LOGGER.error("{} failed at {}! {}")`). A headless
  RCON server has no players. So `gametest-planet.sh` cannot tell "3 required passed", "20/20" or even
  "finished". The GameTest-server path is fine: `GameTestServer.tick` logs its own summary.
- *Fix.* In §9.4 and in P0-05's Creates (the `planetServer` run):
  1. Add a dev-only reporter. When `neoforge.enableGameTest` is set, call
     `GlobalTestReporter.replaceWith(new DeepTimeTestReporter())`. It logs
     `[gametest-result] <test> PASS|FAIL required=<b> <runtime>ms` for every test, and delegates
     failures to `LogTestReporter`.
  2. Add a completion line. Either add a `/deeptime gametest run <class>` command that builds the
     batch through `TestFinder` and adds a `MultipleTestTracker` listener, or count results against
     the expected total. In both cases log
     `[gametest-summary] class=<c> total=<n> passed=<p> failed=<f>`.
  3. Have `gametest-planet.sh` wait for that line, with a timeout, and fail on a timeout.
  4. State that `attempts = 20, requiredSuccesses = 20` is judged from the summary. The per-attempt
     output is also player-only (`ReportGameListener.handleRetry`, line 47).

### Major

**2. [major] The two GameTest runtimes are not separated: from P0-06 on, `./gametest.sh` runs the planet-only tests and goes red.**
§9.4 lines 2045–2065 and 2080–2085. P0-06 step 3 (2428–2434), P0-07's acceptance (2481) and the P0-15
exit (2626) are affected.
- *Evidence.*
  - `GameTestHooks.registerGametests` registers every `@GameTestHolder` method in the enabled
    namespaces (lines 41–62), using the template namespace (`getTemplateNamespace`).
  - `Main.java` line 195 passes `GameTestRegistry.getAllTestFunctions()` to `GameTestServer.create`
    unfiltered. `manualOnly` is filtered only by the `/test` command's `TestFinder` (lines 132 and
    140), not by the GameTest server.
  - So `SeamGameTests`, `SableSeamGameTests` and `PoleCapGameTests` run in the flat overworld, with no
    seam and no Sable. P0-07's "All n required tests passed" then fails.
  - It gets worse. `addGameTestMethods` does `Class.forName(…, true, …)` and `getDeclaredMethods()`.
    If any method of a holder class has a Sable type in its signature, loading that class throws
    `NoClassDefFoundError` on the Sable-less GameTest server and aborts the boot.
- *Fix.* Put the planet-only classes in their own namespace:
  - annotate them `@GameTestHolder("deeptime_planet")`;
  - ship the template a second time at `data/deeptime_planet/structure/empty_5x5x5.nbt`;
  - set `neoforge.enabledGameTestNamespaces=deeptime` on `gameTestServer` and `deeptime_planet` on
    `planetServer`.

  The alternative is conditional registration through `RegisterGameTestsEvent` keyed on a system
  property. Keep every Sable-typed signature in a helper class (for example
  `compat/sable/SableTestOps`) that only test bodies call. Put this in §9.4, and make it the first
  item of P0-06's step 3.

**3. [major] The planet server's mod set is incomplete, and `-PwithMic` would load duplicate mod ids.**
Affected: §4.3 table (lines 463–478) and profiles (504–511); §9.4 (2066–2070); §9.5 (2100–2108);
P0-05 (2384), where the run is created; `./gametest.sh -PwithMic` in P0-05 (2408); and
`./gametest-planet.sh -PwithMic` in P0-15 (2627), P1-15 and P1-16.
- *Evidence.*
  - Modrinth: Aeronautics `44pLdPGg` requires Sable (`T9PomCSv`) and **Create** (`LNytGWDc`, slug
    `create`). No Create pin or runtime entry exists anywhere in §4.3, yet the planet server is meant
    to carry Aeronautics, and `shaftLineAcrossSeam` needs Create's kinetics.
  - In ModDevGradle `localRuntime` feeds every run's classpath (mic-climate `build.gradle` line 87:
    `runtimeClasspath.extendsFrom localRuntime`). So Sable, Aeronautics and Create would also sit on the
    GameTest server, although the table says "planet server".
  - MIC pins `sable` 2.0.5, `create-aeronautics` 1.3.2 and `create` 6.0.10 (`pack/mods/*.pw.toml`),
    while `linkMicMods` skips only Deep Time and TW (line 509). With `-PwithMic`, that gives two jars
    per mod id, which the plan itself calls "a hard FML failure" (line 510).
  - The standalone set resolves Aeronautics' dependencies "from the Modrinth API" (line 469), so its
    Create version is whatever is newest, not the pack's.
- *Fix.*
  - Keep only TW in `localRuntime`.
  - Give `planetServer` its third-party jars by symlink from Deep Time's own
    `smoke/standalone/server/mods/`, through a `linkStandaloneMods` task analogous to `linkMicMods`.
    ModDevGradle's per-run `planetServerAdditionalRuntimeClasspath` is the other route.
  - Make both link tasks skip every jar whose mod id is already on the run's classpath, reading
    `META-INF/neoforge.mods.toml`.
  - Pin Create in `profiles/standalone.txt` to the pack's version id, from
    `pack/mods/create.pw.toml`.
  - Add a Create row to §4.3's table.

**4. [major] How a density function finds its level's world data is undefined; the design as written cannot serve two Deep Time levels.**
§7.1: line 1246 (`WorldDataRef`), lines 1279–1287 (terrain source) and lines 1303–1337 (router, types,
lazy binding). Also §9.4 lines 2045–2050, P0-05, P0-10 and P0-13.
- *Evidence.*
  - Every Deep Time density-function type carries a `world` field: `raster {world, layer, fallback}`,
    `terrain {world, mode, k}`, `depth {world}`, `beyond_pole {world}` and `region_index{world}`.
  - The router is shared data (`data/deeptime/worldgen/density_function/planet/*.json`), and
    `ActiveWorldData` is keyed by folder.
  - From P0-10 on, the GameTest server holds `test_planet` (analytic terrain, C = 16,384) and
    `test_planet_data` (folder `deeptime-test`, C = 1,024). From P0-13 it also holds `test_lake`
    (`deeptime-lake`, C = 1,024). All three name the same `planet_nocaves` router.
  - A JSON `world` value names one folder. The analytic terrain's period also needs C, which differs
    between these levels.
  - So the implementer must invent the binding at P0-05, and the P0-10 GameTests break on whatever
    choice ignores it.
- *Fix.* Write the binding into §7.1 and P0-05's Creates. `DerivedSettings.derive(pack, ref)` walks the
  router with `DensityFunction.mapAll(visitor)` and replaces every `deeptime:*` function with an
  instance bound to that generator's `WorldDataRef`: folder, circumference and test-terrain kind. The
  JSON codecs drop `world`, or keep it only as an optional override. Also name the router files
  (for example `planet/final_density.json`, `planet_nocaves/final_density.json`), and give the codec
  type of `test_terrain` (a kind string such as `"wrap-test"`).

**5. [major] Decisions end up in two or three places, and the authority order lets a newer answer lose to an older one.**
§1.2 lines 49–56; §2.4 lines 226–228; §12.1 lines 3143–3144; §12.3 lines 3163–3175; the draft
CLAUDE.md, lines 3102–3104; §14 lines 3257–3265; the decisions file, lines 3 and 18–20.
- *What is wrong.*
  - **Split storage.** After the move, a new answer from Ben is written by the agent into STATUS.md's
    "Questions asked and answers" table (§12.3). The decisions file, "Ben's document: agents never
    edit it" (§1.2), is updated only when "Ben (or the session coordinating for him) folds answers
    into it". The plan keeps "no stale text" (§12.3), so §14's row must be rewritten too. One answer
    therefore lives in up to three places, with no step that ever forces the fold. The
    "Recorded in decisions file?" column simply accumulates "no".
  - **Incoherent authority.** §1.2's order is decisions file > plan > report > notes. STATUS is not in
    it. Take an answer that changes a recorded decision, for example Q6 ("abandon Toroidal World").
    It sits in STATUS, contradicts the decisions file's "Wrapping: Toroidal World", and by §1.2 loses.
  - **Two versions of the rule.** §12.3 allows "the session coordinating for him" to edit the
    decisions file. The draft CLAUDE.md says "Ben updates docs/deeptime-design-decisions.md himself".
    Neither says which session is "coordinating", so an implementing agent cannot know whether it
    may fold an answer.
  - **The rule is already broken in practice.** The decisions file's header says "Decisions Ben made
    in the pre-implementation Q&A, 2026-09-27". Yet it carries 2026-09-28 rows ("Docs location",
    "Licence acceptances", "Planner / implementer"; lines 18–20) that a planning session wrote for
    Ben (file mtime 28 Sep 01:12).
- *Durability verdict.* No. Once the docs move, future decisions would be split between STATUS and
  the decisions file, and the decisions file would go stale.
- *Fix (one place).* Make `docs/deeptime-design-decisions.md` the only record of Ben's decisions:
  1. Add an append-only section, `## Decided during implementation`, with one row per answered
     question: date, asking task, question id, the question, and Ben's answer verbatim.
  2. Agents may append rows there, in the same commit as the STATUS update, but never change Ben's
     sections above it or earlier rows. A superseding answer is a new row that names the row it
     replaces.
  3. Drop STATUS's "Questions asked and answers" table. Keep only "Pending questions" (asked, not
     yet answered), with a link.
  4. Mark §14 rows "answered → decisions file, <date>".
  5. Rewrite §1.2 ("agents append; they never edit Ben's rows"), §2.4, §12.1, §12.3 and the
     CLAUDE.md rule to match.

  If Ben prefers that agents never touch the file, the fallback is weaker. Rank STATUS's answers table
  above the decisions file in §1.2 for answers newer than the file. Then make every phase-exit script
  fail while any row is unfolded.

**6. [major] Open questions are not reachable from the tasks that need them, and Q5 conflicts with the decisions file.**
§14 line 3264 ("ask when the task in the last column starts"); §1.3 lines 74–78; P0-05 (2369–2411);
P0-06 (2413–2452); P1-14 and P1-15.
- *Evidence.*
  - §1.3's always-read list is the decisions file, §1, §2, the phase's §10 section and the sections
    its task cites. §14 is not on it.
  - Q5 (build range, needed by P0-05), Q11 (pin TW in MIC, P0-06), Q13 ("Why here?" in v1, P1-14) and
    Q10 (EV in MIC, P1-15) are named neither by their task nor by any section it cites. P0-06 cites Q6
    only in its failure branch (step 6).
  - A P0-05 agent therefore builds the fixed −352..639 dimension type without asking. But the
    decisions file says "Everything below is customizable unless stated otherwise" (lines 5–6) and
    gives the build range "as the baseline" (line 46).
- *Fix.*
  - Add an **Ask first** field to every work item that has a question: P0-05 → Q5, P0-06 → Q11,
    P1-14 → Q13, P1-15 → Q4 and Q10, P1-12 → Q15 and Q16.
  - Add "the §14 rows whose 'Needed by' is your task" to §1.3's always-read list and to CLAUDE.md's
    start routine.

### Minor

**7. [minor, P0-05 lines 2387–2400; cold-read Resolution #7] The height reference can be subtracted twice.**
`ChunkAccess.getHeight(type, x, z)` already returns `heightmap.getFirstAvailable(x, z) − 1`
(`ChunkAccess.java` line 195). The plan says the heightmap "stores that y + 1" and then calls
`getHeight`. The Resolution says "the surface is defined as the `OCEAN_FLOOR_WG` value − 1". An agent who
subtracts again is off by one at every corner column, where the tolerance is 0.
*Fix.* State that `getHeight(OCEAN_FLOOR_WG, x, z)` is compared directly with `ceil(H) − 1`.

**8. [minor, §7.1 lines 1281–1283 and 1312; §5.1 line 645] The top slide and the test terrain's peak collide, and the slide anchors are unspecified.**
Vanilla's `final_density` fades out 64–80 blocks below the top (`y_clamped_gradient` 240→256 for a
320 top; `noise_settings/overworld.json`). The same offsets under a 640 top give 560→576. The analytic
test terrain's "range peaking near y 600" would then be flattened, and a corner column there fails
`heightMatchesReference` at tolerance 0. Real terrain reaches y 563 at +10 km, right at that band. The
128 test columns are not specified either.
*Fix.*
- Give the slide constants in §7.1 (for example a top slide 624→640 and a bottom slide −352→−328).
- Cap `AnalyticTestTerrain` below the top slide.
- Say how the 128 test columns are chosen (for example 64 corner columns and 64 others spread over
  lowland, coast and range).

**9. [minor, §9.4 lines 2027–2037] The mixin passes the entire datapack stem registry, not just Deep Time's test stems.**
`bake` prefers `stemRegistry` entries over the preset's, so any pack datapack that defines
`minecraft:overworld` replaces the GameTest server's flat overworld, and any modded dimension becomes an
extra level. Terralith 2.6.2 ships `data/minecraft/dimension/overworld.json`. It is guarded by
`neoforge:not mod_loaded lithostitched`, so MIC is safe today, but the pack changes often.
*Fix.* Pass a frozen registry that holds only the `deeptime:*` entries of `datapackDimensions`.

**10. [minor, §4.1 line 436; §4.3 lines 497–499; §9.4 lines 2056–2060; P0-13 line 2598] Run directories are ambiguous.**
Every script path assumes `run-gametest/` and `run-planet/` at the repository root. mic-climate's run
config (`build.gradle` line 75, `gameDirectory = file('run-gametest')`), copied into `neoforge/`, puts
them in `neoforge/run-gametest/`, so the clean-up, the `synth --out` and inspect paths all miss.
*Fix.* Say `gameDirectory = file('../run-gametest')` and `file('../run-planet')`.

**11. [minor, P0-01 lines 2261–2262] The `.gitignore` list is incomplete.**
It lacks:
- `run/`, the default `client`/`server` run directory, which mic-climate ignores;
- `.direnv/`;
- `.graphide/`. The graphide tooling writes `graph.db`, `noderank.db` and `history-snapshots/` into
  checkouts here (see mic-climate's `.graphide/`). §11.1's `git add -A` would commit them, and P0-01's
  "`git status --porcelain` → empty" would fail later.

**12. [minor, §4.4 lines 534–538; P0-01 lines 2263–2264] It is unclear which flake outputs exist when.**
P0-01 writes "`flake.nix` (§4.4)", but `apps.natives`, `apps.bluemap` and `apps.publish` wrap scripts that
P0-08, P0-14 and P0-06 create. `apps.cli` has no owning task, yet CLAUDE.md uses `nix run .#cli`. The
"flake check" on the toolchain channel is in no acceptance. mic-climate's flake also has a `systems`
input, and rust-overlay has to be applied through `_module.args.pkgs` in flake-parts. Neither is said.
*Fix.*
- P0-01 creates `devShells.default` and the channel check only.
- P0-06 adds `apps.publish`, P0-08 `apps.natives`, P0-09 `apps.cli` and P0-14 `apps.bluemap`.

**13. [minor, §6.5 lines 1162–1163; P0-07 lines 2466–2469 and 2483] `NativeLoader` must be JDK-only, but the plan does not say so.**
`NativeSmoke` runs as `java -cp deeptime-1.21.1-0.0.1.jar NativeSmoke.java`, with no Gson, SLF4J or Guava
on the classpath. "No Minecraft or NeoForge imports" still allows Gson for `natives.json` and SLF4J for
the ERROR log, and both would crash `NativeSmoke`.
*Fix.* "JDK-only: parse `natives.json` by hand or use `java.util.Properties`; log through
`System.Logger` or a callback the mod wires to SLF4J."

**14. [minor, §6.5 lines 1160–1162; §4.3 line 483; P0-08 line 2500] `natives.json` has no schema and the format version has no single source.**
Three writers must agree with one reader: Gradle `hostNatives`, `build-natives.sh` and the Java loader.
Yet the key names are never given. P0-08 queries `.libraries`, which appears nowhere else. Where Gradle
gets "format" (for example `0.0`) is also unsaid.
*Fix.*
- Give the JSON shape in §6.5.
- Keep the format version in one place, for example `[workspace.metadata.deeptime] format_version`,
  read by `build.rs`, `cargo metadata`, Gradle and the scripts.

**15. [minor, P0-09 lines 2524–2528; §4.5 lines 553–555] CI will go red from P0-09.**
The Gradle `test` task "first runs `nix develop -c cargo run … synth`". The CI Gradle job has no Nix, only
setup-java and rustup. `hostNatives` already calls plain `cargo`.
*Fix.* The Gradle task calls plain `cargo`. The dev shell provides it locally, and rustup provides it
in CI.

**16. [minor, §4.5 lines 548–549; draft CLAUDE.md line 3058; §2.4] CI minutes go to STATUS-only pushes.**
Every session pushes a "doing" line, and every task pushes a STATUS update. `ci.yml` runs Rust and a full
ModDevGradle job on every push to `master`, against the free plan's 2,000 minutes.
*Fix.* `paths-ignore: ['STATUS.md', 'docs/**', '**/*.md']` on the push trigger.

**17. [minor] Acceptance commands that need a tweak to run headless.**
- P0-07 line 2483 and §10.1 line 2232: `$SCRATCH` is never set, and each tool call is a fresh shell.
  Write `SCRATCH=<scratchpad>` into the command.
- P0-08 line 2509: `gh run watch` with no id prompts, and fails without a TTY. Use
  `gh run list -w natives.yml -L 1 --json databaseId -q '.[0].databaseId'`, then
  `gh run watch <id> --exit-status`.
- P0-02 line 2312: `gh run list … conclusion` right after a push is empty while the run is in
  progress. Watch the run first.
- P0-07 lines 2470–2480: the text names 3 extraction cases, while the acceptance counts "4 extraction
  cases".
- §4.3 lines 494–495: `forkEvery = 1` is per Test task, not per class. Say "a separate `nativeLinkTest`
  task with `forkEvery = 1`", or set it on `test`.

**18. [minor, P0-11 line 2566; P0-13 lines 2598–2599; P0-14 lines 2611–2613; P0-12 line 2585] Some later acceptances are not machine-checkable.**
- `compare` does not say which height it measures. At FULL status, trees and cave entrances move the
  top block. Specify, for example, `OCEAN_FLOOR` minus logs, leaves and features, and exclude carved
  columns.
- "A PNG with the water surface at y 120" should be a printed `water_top_y=120`.
- "Loads (checked in Chrome where available)" has no fallback when Chrome is unavailable.
- "No errors from C2ME or Distant Horizons" needs a regex.
- The lake's z position, and `<a>`/`<b>`, are placeholders with no source.

**19. [minor, §1.2 lines 36–42; P0-01 lines 2268–2277 and 2281] The docs move is brittle.**
The table maps one review file, `cold-read-review.md`. Step (5) deletes "the old notes and review file",
and the acceptance expects exactly 13 files. `Docs/deeptime-plan/` now also holds this file, so the count
is 14, or this review is left behind or deleted.
*Fix.*
- Map `deeptime-plan/*.md` → `docs/plan-reviews/2026-09-28-<name>.md`.
- Make the count 12 plus the number of review files.
- Write `MOVED.md` rather than deleting anything that was not copied.

**20. [minor, decisions file lines 3–5 and 68] The decisions file's own paths go stale after the move.**
Its header cites `Geologic history worldgen mod.md`, `research_notes/…` and `datapacks/…` relative to
`mic/Docs`. In the repository these become `docs/research/…`, or stay outside the repository. Agents may
not fix them (gap 5).
*Fix.* In P0-01, ask Ben to approve a one-time path update, or fold it into gap 5's append rule.

**21. [minor] The plan contradicts the decisions file in ways that are not recorded as decisions.**

| Decisions file | Plan |
|---|---|
| "Opus writes the implementation plan and implements it (no Fable)" (line 20) | §2.5 lines 239–240 allow Fable for security-level planning after asking |
| Golden-output checks against fastscapelib **and goSPL** (line 91) | §9.2 lines 1988–1989: goSPL "only … if a comparison is worth its HPC cost" |
| "Everything below is customizable": circumference (line 42) and build range (line 46) | Two C values only, 16k/32k (§7.12 lines 1589 and 1613); the range is fixed (§5.1 line 618; Q5) |
| "A Phase 0 spike comes first" (line 40) | P0-06, with a justification (§10.2 line 2246). Reasoned, but never confirmed by Ben (Resolution #19 covers only natives, cap and C) |

Plan-internal, in the same vein:
- §5.1 lines 653–656 say the timeline has no default and the server config must name it. But the
  `server.json` schema (§7.13 line 1651) has no timeline fields, and P1-13 (line 2840) runs
  `--preset earthlike_quick_16k` alone.
- §7.13 line 1656's "commented template" is not valid JSON.

*Fix.* Ask Ben to confirm each deviation (it is a batch of four questions) and record the answers in
the decisions file (gap 5). Add the timeline fields to `server.json`.

**22. [minor] Facts to correct.**
- Excavated Variants and DAG are `LGPL-3.0-or-later` on Modrinth, not "LGPL-3.0" (§7.6 line 1464, §13
  line 3236). DAG's only NeoForge 1.21.1 version is 6.1.2 (`U0ft2n2p`), so §4.3 line 506 could pin it
  now instead of deferring to P1-15.
- TW's 0.17.2 → 0.19.3 span is about 9.7 days, not "within nine days" (line 476).
- `windows-11-arm` exists and serves private repositories (standard arm64 runners, since 2026-01-29),
  so §4.5 lines 565–566 can name it instead of hedging.
- §2.6 lines 272–274: the "app-origin form `https://<space-id>.spacesheep.app/v/<sha>/`" appears nowhere
  in spacesheep 1.17.0 (the CLI knows only `https://spacesheep.dev`). `*.spacesheep.app` does resolve
  (wildcard DNS), so it may exist server-side, but it is unsourced. Mark it "unverified; P0-14
  checks".
- §2.2 line 184 says "MIC 2.1.3". That is the working tree; `HEAD` (`303fd89`) still says 2.1.2.

**23. [minor, §2.2 lines 192–193; §9.5 lines 2109–2111; §12.1 line 3131] The recorded pack commit can misstate what was tested.**
`install.sh mic` reads the pack's working tree, which other sessions edit (`pack.toml` is modified right
now). `git rev-parse HEAD` hides that.
*Fix.* Record `git -C pack describe --always --dirty`, plus `git -C pack status --porcelain | sha256sum`
when dirty.

**24. [minor, §9.5 lines 2112–2117; §9.4 item 2] Server ports are unassigned while peers run servers.**
mic-climate's smoke uses RCON 25575 (`run.py` line 58, `install.sh` line 155) and the default game port.
Deep Time's smoke and planet servers would collide with a peer's.
*Fix.* Fixed Deep Time ports (for example game 25665, RCON 25675, planet server +1), and a
port-in-use check that says what it found.

**25. [minor, §9.5 lines 2104–2107; Q11] `install.sh mic` removes only a pack copy of Deep Time.**
Q11 recommends pinning TW in the pack. Once that happens, the set holds two TW jars.
*Fix.* "Remove any pack copy of Deep Time **and Toroidal World**."

**26. [minor, P0-04 lines 2351–2367] P0-04 has no failure path.**
P0-04 is "the spike's first check", but only P0-06 step 6 says what to do if MIC will not boot with TW.
The first session is defined to end on P0-04's PASS (§11.1 step 5), so it has no stated exit if that
check fails.
*Fix.* "If MIC + TW fails to boot for a reason configuration cannot fix, record it, stop, and ask Q6."

**27. [minor, §11.1 lines 3040–3042; §12.1 lines 3129 and 3137] Two handoff details are unnamed.**
- The auto-memory directory for "the mic project" is never named. Here it is
  `~/.config/graphide/accounts/profiles/<profile>/projects/-home-greencheetah-Projects-mic/memory/`, with
  an index line format of `- [name](file.md): summary`.
- "session <id>" has no source.

*Fix.* Say where each comes from, or "the auto-memory directory your system prompt names" and "the
scratchpad path's session UUID".

**28. [minor, P0-06 lines 2413–2416 versus `wrapped_planet_worlds.md` §6.3] P0-06 skips some server-side spike items without saying so.**
P0-06 says it covers "every item … that runs without a client". It leaves out:
- trees straddling the seam (item 2);
- a train, and a bearing or elevator contraption across the seam (item 4). These are server-side
  testable.
- the pole-transit prototype (item 8).

*Fix.* Add them, or list them as deferred.

**29. [minor, §7.12 lines 1605–1607] `-Ddeeptime.extraDatapack=<folder>` cannot go through `addPackFinders`.**
`addPackFinders` resolves paths inside the mod file (`findResource`). A folder outside the repository
needs `event.addRepositorySource(...)` with `Pack.readMetaAndCreate(..., new
PathPackResources.PathResourcesSupplier(path), ...)`. This affects P1-12.

## The draft `CLAUDE.md` and `STATUS.md` (check 7)

A future agent reading only these would find the repository, the commands, the landing rules and the
current task. It would **not** be fully oriented:
- **No reading routine.** "Start of every session" (lines 3054–3058) says to read STATUS, not the
  decisions file, plan §1, plan §2, the phase section and the task's §14 questions (gap 6). The
  pointer "§1.3 is the reading list" is buried in the authority list.
- **No end-of-session routine.** §12.2's build-log entry, header update and push are missing.
- **The decision rule disagrees with §12.3** (gap 5).
- **Reference jars point at another project's build output.** The sources path is mic-climate's
  `build/moddev/artifacts/` (line 3114), which a `gradle clean` over there deletes. From P0-03 on,
  Deep Time has its own copy in `neoforge/build/moddev/artifacts/`.
- **Nothing on runtime separation or ports** (gaps 2 and 24).

*Fix.*
- Add "Read the decisions file, plan §1–§2, your phase in §10 and your task's §14 questions" to the
  start routine.
- Add an "End of every session" block that mirrors §12.2.
- Repoint the sources jar.
- Add one line each for the GameTest namespaces and the ports.

STATUS's template is sound. After gap 5 it keeps "Pending questions" instead of "Questions asked and
answers".

## Earlier gaps whose fix is not fully real

All 30 resolutions are in the plan text. I spot-checked the blocker and all eight majors against the
plan and the jars. None is missing or fake. These are only partly fixed:

| Old # | What the resolution says | What remains |
|---|---|---|
| 1 | Gametest-only `@WrapOperation` on `bake`, NOISE-status reads, `test_bare` | Correct for the GameTest server. It passes every datapack stem (gap 9). The planet runtime never got the matching treatment: reporting (gap 1) and separation (gap 2) |
| 5 | Deep Time owns its MIC set; `linkMicMods` skips Deep Time and TW | The duplicate-mod-id hazard returns for Sable, Aeronautics and Create (gap 3) |
| 7 | "Surface = `OCEAN_FLOOR_WG` value − 1" | `getHeight` already subtracts 1 (gap 7). The slide anchors are undefined (gap 8) |
| 9 / 26 | Docs move into the repository; "agents never edit the decisions file"; answers go to STATUS | The move is real. The recording rule created the split in gap 5 |
| 13 | `forkEvery = 1` for linking tests | Not expressible per class in one Test task (gap 17) |
| 21 | Windows arm64 "when a runner label exists" | `windows-11-arm` exists for private repositories; still hedged (gap 22) |
| 27 | CLAUDE.md gains the session routine and rules | Still no reading routine or end-of-session routine, and the decision rule disagrees with §12.3 (check 7) |

## Counts

- Blockers: 1 (gap 1).
- Major: 5 (gaps 2–6).
- Minor: 23 (gaps 7–29).

## Verdict

**The first session (P0-01 to P0-04) can run from these documents alone.** Every path, pin, tool
version, API signature and GitHub fact it depends on checks out. The bootstrap stages files before
Nix, and every acceptance command is headless and runnable on this NixOS machine. The friction is
minor: the `.gitignore` gaps (11), which flake outputs exist when (12), and the CI race (17).

**Phase 0 end to end: not yet.**
- **P0-05** forces two unrecorded design decisions: how the router binds to a level's world data
  (gap 4), and where the planet server gets Sable, Aeronautics and Create (gap 3).
- **P0-06**, the spike the whole plan hinges on, cannot pass as written. On a player-less server the
  results it must parse never reach the log (gap 1). Its tests would also run, and fail, inside
  `./gametest.sh` (gap 2), which turns P0-07's acceptance red.
- **Decisions.** The recording rule needs settling before the docs move, or the decisions file goes
  stale from the first new answer (gap 5). Q5 and Q11 would be skipped (gap 6).

Each of gaps 1–6 is a small, well-defined edit to §7.1, §9.4, §4.3, §1.2/§12.3 and the P0-05 and P0-06
work items. With those folded in, plus gaps 7–10 for P0-05's tests, I expect a fresh agent to carry
Phase 0 through P0-15 without guessing.
