# Deep Time implementation plan: cold-read review

Reviewed 2026-09-28 by a fresh Opus agent that had only the plan
(`/home/greencheetah/Projects/mic/Docs/deeptime-implementation-plan.md`, 2,855 lines) and the files it
lists. I followed §1.3's reading order and §11.1's checklist, then walked P0-01 to P0-04 as if I were
implementing them. This was a dry run: I created no repository, wrote no code, ran no builds and
downloaded no jars. Line numbers refer to the plan unless another file is named.

## What checked out

Everything below was checked read-only and matches the plan.

| Claim | Evidence |
|---|---|
| Every absolute path in the plan exists, apart from `mods/deep-time`, which the plan says to create | `test -e` on every path, including the nine research notes and the mic-climate house-style files |
| Every `§` cross-reference resolves to a heading | A script matched all internal `§n.n` references against the headings. External anchors resolve as well: aeronautics §3–§6 and §8, wrapped_planet §0–§6.3, MIA §1–§5, surface §1–§7, tectonics Q3, ore Q1–Q7, mod-compat KQ1–3 and the follow-up, and the report's "Nothing scientific embeds" |
| nixpkgs versions: `jdk21` 21.0.12.1+1, `jdk25` 25.0.4.1+1, `nodejs_22` 22.23.2, `cargo-zigbuild` 0.23.4, `cargo-xwin` 0.22.0, `zig` 0.16.0, `zig_0_13` 0.13.0 | `nix eval --raw nixpkgs#<pkg>.version`, run one attribute at a time. Also present: `cargo-deny` 0.19.6, `mcrcon` 0.7.2 |
| Rust 1.98.1 is the current stable, rust-overlay has it, and the glibc floor is still 2.17 | `channel-rust-stable.toml` gives `1.98.1 (48a229cea 2026-09-01)`; `oxalica/rust-overlay` has `manifests/stable/1.98.1.nix`; the platform-support page lists glibc 2.17+ for both gnu targets |
| jni-rs 0.22.4: released 2026-03-16, MIT OR Apache-2.0, MSRV 1.85.0. `EnvUnowned::with_env` → `EnvOutcome::resolve::<P>()`, `jni::errors::ErrorPolicy`, panics caught with `catch_unwind`, `Env::get_direct_buffer_address` | crates.io API; docs.rs 0.22.4 |
| Crate pins exist and are the current max_stable: rayon 1.12.0, lz4_flex 0.14.0, memmap2 0.9.11, serde 1.0.229, serde_json 1.0.151, postcard 1.1.3, kiddo 6.3.0 (MSRV 1.89), glam 0.33.11, rand 0.10.3, rand_pcg 0.10.2, png 0.18.1 | crates.io API |
| lz4-java 1.8.0 is on the 1.21.1 classpath, and `RegionFileVersion` imports `net.jpountz.lz4` | `~/.gradle/caches/minecraft/versions/1.21.1/metadata.json` lists `org.lz4:lz4-java:1.8.0`; lines 17–18 and 61–66 of `RegionFileVersion.java` in the NeoForge 21.1.249 sources jar |
| GameTest pass lines are `GAME TESTS COMPLETE` and `All {} required tests passed :)` | `GameTestServer.java` lines 185 and 190 |
| ModDevGradle 2.0.74, NeoForge 21.1.249, Parchment 2024.11.17, Java 21 match mic-climate; the jar name follows `${mod_id}-${minecraft_version}-${version}` | mic-climate `build.gradle` and `gradle.properties` |
| Toroidal World 0.19.3+mc1.21.1 is on Modrinth (slug `toroidal-world`, MIT) as version id `8YTSYi9y`, published 2026-09-27. At commit `3c1f3a30`, which is the head of `mc/1.21`: no `maven` branch, modId `toroidal_world`, requires neoforge `[21.1.248,)`, package `com.toroidalworld.api.v1`, and `ShapeModule`, `ShapeDimensions` and `LoopSpans` all exist | Modrinth API; `gh api` against `anyttng/toroidal_world` |
| TerraBlender 4.1.0.8, BOP 21.1.0.14, GlitchCore 2.1.0.2, Excavated Variants 4.3.1 and Chunky 1.4.23 exist for NeoForge 1.21.1 | Modrinth API |
| BlueMap v5.28: `bluemap-5.28-cli.jar` exists, targets Java 25, supports Minecraft 1.13.2–26.3 | GitHub release notes |
| ReTerraForged `ba35ca7` exists and is MIT; the `macos-15-intel` label exists; `macos-13` was retired on 2025-12-04; arm64 standard runners have served private repositories since 2026-01-29 | GitHub API and changelog |
| `gh` is authenticated as BenMac31 (scopes `repo`, `read:org`). Ben is an admin of `MinecraftIndustrialComplex`, and `deep-time` does not exist there yet | `gh auth status`; `gh api user/memberships/orgs/...`; `gh repo view` |
| The derived-geometry, memory, disk, cost-model and deposit-count arithmetic in §5 and §8 | Recomputed by hand; every figure agrees to the stated rounding |
| Rock-tag datapack: 59 classes, 17 groups, 43 fallback tags, 18 vanilla member blocks, every tag `replace: false` | File counts under `Docs/datapacks/deeptime-mic-rock-tags/data/deeptime/tags/block/`; README lines 5–6 |

## Gaps

Severity: **blocker** means the task cannot proceed as written; **major** means it would likely go
wrong; **minor** means friction, drift or a missing detail.

### Blocker

**1. [blocker, P0-07 onward] The GameTest server never loads the `test_planet` dimension.**
§9.4, lines 1820–1827; P0-07, lines 2100–2109. The same problem affects every GameTest-server test that
needs Deep Time terrain: P0-09 (2165), P0-10 (2180), P1-09, P1-11, P1-12 and P1-15.
- *What is wrong.* The plan says "Datapack level stems are baked into `WorldDimensions` alongside the
  flat overworld." In 1.21.1 they are not. `GameTestServer.create` (sources jar, lines 97–103) builds the
  flat preset and calls `.bake(registry)` on a **new, empty, frozen**
  `MappedRegistry<LevelStem>`. `WorldDimensions.bake` (lines 164–179) adds only the stems of the
  registry it is given. The dedicated server does it differently: `Main.java` line 208 passes
  `datapackDimensions().registryOrThrow(Registries.LEVEL_STEM)`. So `data/deeptime/dimension/test_planet.json`
  loads as data but never becomes a level, and all five of P0-07's `GeneratorGameTests` have no
  dimension to run in. Two smaller problems sit next to this one:
  - The GameTest server reuses its storage between runs. `Main.java` lines 185–199 delete nothing, so
    chunks generated by an earlier generator build would be tested again.
  - `spawnOnLand` cannot exercise `LevelEvent.CreateSpawnPosition` in this server, because that event
    fires only for the flat overworld.
- *Fix.* Pick one of two approaches and write it into §9.4 and P0-07:
  - (a) Recommended. Add a gametest-only mixin, applied only when `-Ddeeptime.gametest=true`, that
    redirects the `WorldDimensions.bake` call inside `GameTestServer.create`'s load lambda to pass
    `ctx.datapackDimensions().registryOrThrow(Registries.LEVEL_STEM)`.
  - (b) Run the generator, strata and water tests on `gametest-planet.sh`. P0-07 would then depend on
    P0-13's planet harness.

  In both cases, `gametest.sh` should delete the old test-dimension region folder before it runs
  `deeptime synth`. `spawnOnLand` should call `SpawnHandler`'s candidate selection directly against
  `test_planet`. Make the mixin (or the harness move) the first step of P0-07, and prove it with a
  one-line test that the dimension exists.

### Major

**2. [major, §11.1 step 4, lines 2621–2632] The bootstrap runs Nix before the flake is tracked by git.**
- *What is wrong.* The checklist runs `git init -b master`, writes the files, then runs `nix flake lock`
  and `nix develop -c rustc --version`, and only after that runs `git add -A`. Inside a git work tree,
  Nix (Lix 2.94.2 on this machine) sees only files that git tracks. An untracked `flake.nix` or
  `rust-toolchain.toml` does not exist as far as Nix is concerned, so the first command of the first
  session fails.
- *Fix.* Run `git add -A` (staging is enough) immediately after writing the files and before
  `nix flake lock`; commit afterwards. Add a note to §2.1: "flakes see only git-tracked files; stage new
  files before `nix develop`."

**3. [major, §6.4 lines 1015–1017, §4.4 lines 426–429, P0-01 acceptance line 1980] The zig 0.13 pin never
reaches cargo-zigbuild.**
- *What is wrong.* The nixpkgs `cargo-zigbuild` package wraps its binary with
  `--prefix PATH : /nix/store/…-zig-0.16.0/bin`; `nix eval --raw nixpkgs#cargo-zigbuild.postInstall`
  shows this. Putting `zig_0_13` in the dev shell therefore changes nothing for cargo-zigbuild, which
  always runs 0.16.0. Meanwhile the P0-01 check `nix develop -c zig version → 0.13.0` passes, so the
  plan's stated reason for the pin is silently defeated. Separately, the flake's nixpkgs input is never
  specified. mic-climate uses `nixos-unstable`, and whether a future lock still carries `zig_0_13` is
  unknown.
- *Fix.* In `flake.nix`, use `(pkgs.cargo-zigbuild.override { zig = pkgs.zig_0_13; })`; the override
  argument `zig` exists (`nix eval --json nixpkgs#cargo-zigbuild.override.__functionArgs`). Replace the
  acceptance check with one that inspects the wrapper, for example
  `nix develop -c sh -c 'grep -o "zig-0\.[0-9.]*" "$(command -v cargo-zigbuild)"'` → `zig-0.13.0`. State
  the nixpkgs input (`github:NixOS/nixpkgs/nixos-unstable`, as in mic-climate) in P0-01. The other
  option is to drop the pin and test zig 0.16, which is what nixpkgs itself ships with cargo-zigbuild
  0.23.4.

**4. [major, §2.3 line 163, §4.5 lines 450–452, P0-02 line 1997] The licence identifier is undecided, and
cargo-deny would reject Deep Time's own crates.**
- *What is wrong.*
  - "LGPL-3.0" is a deprecated SPDX identifier. `LGPL-3.0-only` and `LGPL-3.0-or-later` are different
    licences. Every file header, `Cargo.toml` `license`, `neoforge.mods.toml` `license` and the README
    must pick one. That is a licence decision, so under §2.4 it must go to Ben, yet it is not in §11.1's
    question list.
  - cargo-deny checks workspace members too. The §4.5 allow-list has no LGPL entry, so as written
    `nix develop -c cargo deny check licenses bans` fails on the six `deeptime-*` crates themselves.
    The only workarounds are `publish = false` on each crate plus `[licenses.private] ignore = true`,
    or an exception for them.
- *Fix.* Add a question to the first session: "LGPL-3.0-only or LGPL-3.0-or-later?", with a
  recommendation. Specify `publish = false` in `[workspace.package]` and
  `[licenses] private = { ignore = true }` in `deny.toml` (the allow-list still governs every
  dependency). Also say that each crate's `license` field carries the chosen identifier.

**5. [major, §2.2 line 150, §4.3 lines 415–420, §7.1 lines 1128–1130, §7.12 line 1454, §10.2 lines 2255–2256,
§13 line 2818, draft CLAUDE.md line 2701] "The MIC pack's server-side jars" are a stale copy that
belongs to another project.**
- *What is wrong.* `mods/mic-climate/smoke/server/mods/` was installed on 2026-09-05. The pack moved on
  2026-09-17 (commit `36a2e51`, "Update all mods to latest compatible…"). Comparing the pack's
  non-client `.pw.toml` filenames with the smoke folder shows **about 40 jars differ**, including:
  - Tectonic: 3.0.26 in the copy, 3.0.28 in the pack;
  - C2ME: .120 → .122;
  - Lithostitched: 1.8.0+beta4 → beta6;
  - Distant Horizons: 3.2.0-b → 3.3.0;
  - petrolpark: 1.5.6 → 1.5.11;
  - Power Grid: 0.6.1 → 0.6.2.

  The plan's `javap`-verified facts, such as Tectonic's key check in §7.1 and its preset-filter mixin in
  §7.12, and §13's version column all come from the old set. P0-07's and P0-09's `-PwithMic` runs would
  test a MIC that no longer ships. They would also symlink into a runtime that mic-climate sessions can
  reinstall at any moment.
- *Fix.* Have Deep Time build its own MIC server set from `pack/pack.toml` with `packwiz-installer -s
  server` into its gitignored `smoke/mic/server/mods/`. That means moving P0-13's install step ahead of
  P0-07's `-PwithMic` check, or making that check depend on it, and pointing `-PwithMic`, §2.2 and the
  CLAUDE.md "Reference jars" line there. Re-run the Tectonic `javap` checks on 3.0.28 and update §13 to
  the pack's versions. Add a rule: "the MIC jar set is whatever `pack/` pins today; re-install before
  any `-PwithMic` gate."

**6. [major, §10.2 line 1962; P0-12 lines 2205 and 2208; P0-09 line 2169; P0-10 line 2181] The Phase 0 order
contradicts the task dependencies.**
- *What is wrong.* The order puts `P0-12 → P0-13`, but P0-12 "Depends on … P0-13 (for fixtures)" and
  its acceptance reads `smoke/standalone/server/planet`, which only P0-13 creates. An agent that follows
  the order starts P0-12 with no smoke world. There are also missing edges:
  - P0-09's acceptance needs the `mic` smoke report (P0-13) but lists only P0-07.
  - P0-10's acceptance runs `deeptime-inspect section` (P0-12) but lists only P0-07.
- *Fix.* Change the order to `… P0-07 → P0-13 → P0-12 → P0-08 → P0-09 → P0-10 → P0-11 → P0-14`. Add
  P0-13 to P0-09's "Depends on" and P0-12 to P0-10's.

**7. [major, P0-07 lines 2106–2108; §7.1 lines 1146–1147 and 1154–1158] `maxAbsDiff=0` is not a well-defined
or reachable criterion.**
- *What is wrong.*
  - `deeptime:terrain` sits inside vanilla's `final_density` graph, which is interpolated over
    4×8×4 cells (`size_horizontal 1`, `size_vertical 2`). Linearity in y makes the vertical
    interpolation exact, but bicubic H is not linear in x or z. Between cell corners the generated
    surface therefore departs from H(x, z) wherever the synthetic range (up to y 600) has curvature.
  - The router keeps vanilla's caves by reference, including cave entrances, so some columns lose
    their top block.
  - "Surface y" is never defined: floor of H, round of H, or the highest y with density > 0.
  - The "or 1 where the detail term is on" clause cannot apply in Phase 0, because `detail/amplitude`
    is not baked until P1-06.
- *Fix.* Define the reference as the highest y whose interpolated density is > 0. Either compare at
  cell-corner columns (x, z ≡ 0 mod 4) with tolerance 0 and at all other columns with tolerance 1, or
  compare against a Java reimplementation of the interpolation. Exclude columns whose top block was
  carved (air or cave-air below the expected surface), or give `test_planet` a router with caves off.
  Log `columns`, `excludedCaves` and `maxAbsDiff`.

**8. [major, P0-04 lines 2020–2021, 2031–2033, 2040–2042; Phase 0 exit lines 2242–2243] The goal says
"every failure class", but the tests cover two.**
- *What is wrong.* §6.6 (lines 1064–1068) lists five failure classes: unsupported platform,
  glibc/musl mismatch, file removed after writing, I/O error, and version mismatch. P0-04 names enum
  values only for `UNSUPPORTED_PLATFORM` and `ANTIVIRUS_SUSPECTED`, and `forceFail` accepts only
  `unsupported`. An agent can pass the acceptance while leaving three classes unimplemented and
  untested, or has to guess the names and the forcing mechanism.
- *Fix.* Name all five enum values in §6.6. Add a
  `-Ddeeptime.natives.forceFail=<unsupported|libc_mismatch|io|version_mismatch>` value for each class
  (the antivirus class keeps `testDeleteAfterWrite`), with one JUnit case and one `NativeSmoke` line
  per class. Make the Phase 0 exit row check all five.

**9. [major, §1.2 lines 32–36, §12.3 lines 2749–2755, draft CLAUDE.md lines 2651–2656] The authoritative
documents are not under version control.**
- *What is wrong.* Neither `/home/greencheetah/Projects/mic` nor `Docs/` is a git repository
  (`git rev-parse` gives "not a git repository"). §12.3 says to "edit the plan's section in the same
  commit series" and to keep "no stale text", and the decisions file is appended to by every session.
  Several sessions share this machine, which §2.4 itself warns about. The plan and the decisions file
  therefore have no history, no review trail and no protection against two sessions editing them at
  once. A later agent cannot tell which revision of the plan a STATUS entry was written against.
- *Fix.* Ask Ben in the first session: either move both files into the repo at P0-01 (`docs/plan.md`
  and `docs/decisions.md`, leaving a pointer at the old paths), or put `Docs/` under git. Then update
  the paths in §1.2, §1.3, §11.2 and §12.3. If they stay outside git, rewrite §12.3's "commit series"
  sentence and add "record the plan's `sha256sum` in each STATUS build-log entry".

### Minor

**10. [minor, §7.5 lines 1269–1272; P1-12 lines 2428–2430] The coal-fallback text is stale.**
- *What is wrong.* The plan says the MIC datapack's `rock_fallback/coal.json` "still lists
  `minecraft:black_concrete` second" and that P1-12 must change it. The file already lists
  `{"id":"born_in_chaos_v1:black_argillite","required":false}` and then `"minecraft:stone"`. The README
  agrees (lines 223 and 251–253), and so does the decisions file (line 67). No plan text contradicts
  the decision itself.
- *Fix.* Delete the "still lists" sentence. Remove the datapack and README edits from P1-12's Creates.
  Keep `coalFallsBackToStone`.

**11. [minor, §6.4 lines 1002–1003; P0-01 line 1974] "The eight targets of §6.4" include invalid rustup
target names.**
- *What is wrong.* §6.4's "Rust target" column lists `x86_64-unknown-linux-gnu.2.17` and
  `aarch64-unknown-linux-gnu.2.17`. Those are cargo-zigbuild target specifications, not rustup targets.
  Copied into `rust-toolchain.toml`, they make rust-overlay fail.
- *Fix.* List the eight rustup targets explicitly in P0-01 and keep `.2.17` only in the zigbuild
  command.

**12. [minor, P0-02 lines 1987–1992; P0-03 lines 2003–2011] Files and steps a fresh agent would have to
invent.**
- *What is missing.*
  - The Gradle wrapper files (`gradle/wrapper/*`, `gradlew.bat`; mic-climate uses Gradle 8.11.1).
  - A GameTest template structure. mic-climate uses `data/mic_climate/structure/empty_5x5x5.nbt` with
    `@PrefixGameTestTemplate(false)`, and P0-03's `BootGameTests` cannot run without one.
  - `mod_version = 0.0.1`, which is implied only by the jar name, and `mod_name`, `mod_authors`,
    `mod_license` and `mod_description`.
  - An owner for adding the Gradle step to `ci.yml`. P0-02 creates CI before `neoforge/` exists, and
    P0-03 never says to add the step.
  - The Rust edition (2024 fits MSRV 1.85 and the `#[unsafe(no_mangle)]` style) and the
    `[workspace.package]` version.
- *Fix.* Add each of these to the relevant task's Creates list.

**13. [minor, P0-04 lines 2030–2037; P0-06 lines 2078–2080 and 2086] JUnit is not set up anywhere, and
"PASSED" never prints by default.**
- *What is wrong.* mic-climate has no JUnit, so there is no house style to copy. Gradle prints no
  per-test `PASSED` lines unless `testLogging { events 'passed', 'failed' }` is set. It is also unclear
  whether ModDevGradle puts lz4-java 1.8.0 on the *test* runtime classpath. And `NativeLoaderTest`
  loads the library repeatedly in one JVM, but `System.load` of a second copy of the same library in
  the same class loader does not rebind natives.
- *Fix.* Specify:
  - JUnit Jupiter (pin a version) with `useJUnitPlatform()`;
  - `testLogging`;
  - `testImplementation 'org.lz4:lz4-java:1.8.0'`;
  - `forkEvery = 1` for `NativeLoaderTest`, or a loader that separates extract-and-verify from
    `System.load`.

  Better, make the acceptance a check on `build/test-results/test/*.xml` (for example, zero `failures`
  and the expected number of test cases) instead of grepping console text.

**14. [minor, §4.3 lines 408–410; §6.5 step 5, lines 1043–1047] `hostNatives` is under-specified.**
- *What is wrong.* The plan does not say:
  - how Gradle tells a "dev" build from a release build;
  - that `hostNatives` must also write `natives.json` with the host library's sha256, which the
    loader's verification step requires;
  - that a later `nix run .#natives` leaves eight cross-built libraries in the same
    `target/natives-dist/`, which every dev jar would then pick up.
- *Fix.* Stage host natives in `target/natives-host/`, with their own `natives.json`. Make
  `processResources` take `natives-dist/` only when `-Prelease` is set.

**15. [minor, §4.5 lines 443–444 versus §6.1 line 906] `NativeSmoke` "calls `version()` and `selfTest()`",
but `DeepTimeNative` is package-private.**
- *Fix.* Have `NativeSmoke` print the version and checksum from the `NativeStatus` that
  `NativeLoader.load(gameDir)` returns.

**16. [minor, §6.1 lines 907–908 versus P0-04 lines 2039 and 2042; §6.5 lines 1048–1049] Version strings
disagree, and the P0 native stubs are undefined.**
- *What is wrong.*
  - §6.1 shows `"0.1.0+<git sha>;format=0.1;…"`, while P0-04 expects `0.0.1+<sha> format=0.0`.
  - §6.5 makes a version mismatch fatal without saying what is compared, or where the version comes
    from (Cargo or Gradle).
  - `selfTest` is described as "a tiny erosion step", but no erosion code exists until P1-04.
  - Nothing says what `create`, `finalizeWrite`, `copyKeyframe`, `eventsSince` and `sampleColumn` do in
    P0-04, or what preset JSON `DummyStage` accepts.
- *Fix.*
  - Use one version source: the Cargo workspace version, which a test checks against Gradle's
    `mod_version`.
  - Compare `semver` and `format` only, never the sha.
  - Define the P0 `selfTest` as, for example, a fixed-input Rayon reduction with a known checksum.
  - Add a `{"dummy":{"steps":N,"sleep_ms":M}}` preset to P0-04, and say that the other entry points
    return empty results or throw `UnsupportedOperationException` until P1-01.

**17. [minor, §2.1 line 128; P0-08 line 2148; P0-10 line 2181; P0-12 line 2208; P1 acceptances] Bare
`cargo` runs outside the flake.**
- *What is wrong.* On this machine `cargo` is a rustup 1.29.0 shim
  (`~/.nix-profile/bin/cargo → rustup`) with no toolchains installed. A bare `cargo run` would therefore
  download 1.98.1 and all eight targets into `~/.rustup`, outside the flake. `jq`, `gh`, `python3`,
  `readelf`, `git` and `gcc` are in the user profile, so "nothing is installed globally" is not quite
  true, though harmless for them.
- *Fix.* Prefix every `cargo` acceptance command with `nix develop -c`.

**18. [minor, §1.3 lines 49–77 versus §11.1 line 2615; §1.3 line 53] There are two different reading
orders.**
- *What is wrong.* §11.1 step 1 lists only plan sections; §1.3 also lists nine research notes and the
  house-style files. §1.3 also says the decisions file is "104 lines"; `wc -l` gives 103.
- *Fix.* Make §11.1 step 1 read "follow §1.3 items 1–2 and 10; read items 3–9 when a task cites them".

**19. [minor, decisions file lines 24, 28, 38 and 40] Deviations from the decisions file that were never
recorded as decisions.**
- *What is wrong.*
  - The plan builds 8 natives where the decisions file says "plus a musl build" (plan line 1005
    acknowledges the difference but records no decision).
  - The memory cap is "4 GB" in the decisions file and 4 GiB in the plan (line 854).
  - The plan fixes C = 16,384 / 32,768, while `wrapped_planet_worlds.md` §1.2, §6.1 and §6.3 use
    1,000 / 2,000 chunks. P0-08 cites that note as an input without saying its figures are superseded.
  - The decisions file says "A Phase 0 spike comes first", but P0-08 is tenth in the order, and TW is a
    hard dependency from P0-03 on.
- *Fix.*
  - Ask Ben to confirm the eighth target and the C values, and append them under "Added during
    implementation".
  - Note in P0-08 that the research note's chunk counts are superseded.
  - Pull P0-08 step 1, the Sonnet source lookup of how `ShapeModule.Apply` runs on a server's first
    start, forward into the first session so the "spike first" intent is honoured cheaply.

**20. [minor, P0-13 line 2223; §9.5 lines 1854–1860] The standalone smoke cannot work as written.**
- *What is wrong.* P0-13's acceptance runs `install.sh standalone` without `--synth`. Phase 0 has no
  first-start generator, so that server has no world data. The `standalone` profile (NeoForge, Deep
  Time, TW and Sable) also lacks Chunky, which the scenario uses; Chunky 1.4.23 exists only in the MIC
  set.
- *Fix.* Add `--synth wrap-test --size 16384` to the command and add Chunky (Modrinth `LuFhm4eU`) to
  the `standalone` and `terrablender` profiles.

**21. [minor] Lists that disagree inside the plan.**
- *What is wrong.*
  - §4.5 (lines 443–446) lists 7 load targets; P0-05 lists 8 (arm64 musl).
  - §9.7's subcommands (lines 1906–1916) lack `regions --span`, which P0-08 and P0-12 use.
  - §9.4's GameTest class list (lines 1838–1840) lacks `BootGameTests`, `EditorModelGameTests`,
    `SpawnGameTests` and `NetworkGameTests`.
  - §5.5's geology event kinds (lines 748–752) lack `deeptime:guaranteed_occurrence` (line 1344).
- *Fix.* Reconcile each list with the tasks that use it.

**22. [minor, P0-03 line 2009; §4.3 line 415] Pins the plan could give but does not.**
- *What is missing.* TW 0.19.3+mc1.21.1 is Modrinth version id `8YTSYi9y`, a single fabric+neoforge
  jar. TW has shipped 0.17.2, 0.18.1 and 0.19.3 within nine days, so the id matters. GlitchCore for
  `-PwithTerraBlender` is unpinned; mic-climate pins `S2TfWrZR` (2.1.0.2).
- *Fix.* Write both ids into §4.3.

**23. [minor, §2.6 lines 206–212] `npx -y spacesheep` now resolves to 1.17.0.**
- *What is wrong.* The deploy flags were read from 1.13.0's `--help`. 1.13.0 still exists, but the
  latest version is 1.17.0 (npm registry).
- *Fix.* Use `npx -y spacesheep@1.13.0`, or re-check `--help` on the latest version and record it.

**24. [minor, §9.6 lines 1887–1891; P0-11 line 2197] BlueMap's compression setting exists.**
- *What is wrong.* The plan hedges on whether the setting exists. BlueMap 5.28's
  `storages/file.conf` has `compression: gzip | zstd | none` (default gzip).
- *Fix.* Set `compression: none` in `storages/file.conf.tmpl` and keep the service worker only as a
  fallback.

**25. [minor, §4.4 lines 426–429] cargo-xwin's linker needs are unverified.**
- *What is wrong.* nixpkgs `cargo-xwin` 0.22.0 is not wrapped with clang or lld
  (`propagatedBuildInputs = []`), and the dev-shell list has no LLVM tools. Whether cargo-xwin can link
  `*-pc-windows-msvc` cdylibs without `lld-link` and `clang-cl` on PATH is unchecked.
- *Fix.* Add a P0-05 step: confirm from cargo-xwin 0.22.0's README which tools it needs, and add
  `llvmPackages.lld` / `clang` to the shell if needed.

**26. [minor, §11.1 step 3, lines 2618–2620; Q12 line 2852] The first-session questions do not fit the
plan's own constraints.**
- *What is wrong.* `AskUserQuestion` takes at most four questions per call. Q2, Q12, Q1 and Q9 already
  use all four, so Q12's three licence acceptances must be a single multi-select question. Items 4 and
  9 above add two more questions, which needs a second call. Q12 also covers the EULA only for "smoke
  servers", but `gametest-planet.sh`'s dev `server` run (P0-08, P0-13) needs `eula=true` as well.
- *Fix.* Reword Q12 to cover every dedicated-server run, and plan for two question calls.

**27. [minor, §11.2 lines 2644–2702; §12.1 lines 2712–2736] The draft CLAUDE.md and STATUS.md leave out
rules a future agent would need.**
- *What is wrong.* The two drafts point to the plan, so an agent would get oriented. But CLAUDE.md
  omits:
  - the start-of-session routine: read STATUS.md, check `git status`, `git worktree list` and `ps`, and
    set the task to `doing` with date and session id. This lives only in §12.1.
  - the memory rule: never run the simulation and a smoke server at once, with about 6 GB free
    (§2.1).
  - "mic-climate paths are read-only".
  - "run every command through `nix develop -c`".
  - the decision-append format (`## Added during implementation`, table rows).
  - the fact that the plan and decisions file live outside git (item 9).

  Its "MIC server jars" reference points at the stale copy (item 5). STATUS.md has no field that says
  which plan revision it tracks.
- *Fix.* Add those rules as short bullets, repoint the jar path, and add
  `Plan revision: <sha256 or commit>` to the STATUS header.

**28. [minor, §4.5 lines 438–449] CI budget on the free plan.**
- *What is wrong.* `MinecraftIndustrialComplex` is on GitHub's **free** plan and has 0 private
  repositories. A private repository gets 2,000 Actions minutes a month. `ci.yml` runs on every push,
  including a full ModDevGradle setup for `./gradlew build test`. `rust-toolchain.toml` also makes
  every CI job install the standard library for all eight targets. The "land and push" workflow in
  §2.4 pushes often.
- *Fix.* Cache `~/.gradle` and `~/.cargo`, set `concurrency: cancel-in-progress`, and install the
  extra targets only in `natives.yml`, for example with a separate toolchain file or
  `rustup target add` inside that job.

**29. [minor, P0-06 line 2078] The synth arguments for `TilePackParityTest` are missing.**
- *What is wrong.* "The Gradle test task first runs `cargo run -p deeptime-cli -- synth`" gives no
  `--kind`, `--size` or `--out`. A 16k world at L3 is 64 MiB per layer, which is large for a unit-test
  fixture.
- *Fix.* Name a small fixture, for example `--kind wrap-test --size 1024`, and state which layers the
  test compares.

**30. [minor, §2.2 line 148] The sources jar is only partly Parchment-mapped.**
- *What is wrong.* It is labelled "Parchment-mapped", but many methods keep SRG parameter names, such
  as `p_321472_` in `RegionFileVersion`.
- *Fix.* Change the label to "(Mojang names; Parchment parameter names where available)" so nobody
  expects readable parameter names everywhere.

## Counts

- Blockers: 1 (item 1).
- Major: 8 (items 2–9).
- Minor: 21 (items 10–30).

## Verdict

**Yes. A fresh agent could start P0-01 today from these documents alone.** The paths, pins, crates,
toolchain, GitHub access and house-style inputs all exist and match. The agent would stumble on item 2
(the flake is not staged) within minutes, fix it, and must add item 4's licence question to the first
`AskUserQuestion`. With the fixes in items 3, 11–16 and 17 folded in, P0-01 to P0-04 are implementable
as written: every acceptance command runs headless on this machine, and the GameTest pass lines, the
jni-rs API and the Toroidal World coordinates are confirmed.

**The plan is not yet a complete handoff for Phase 0 as a whole.** Item 1 means P0-07's GameTests, and
every later GameTest that needs Deep Time terrain in the GameTest server, cannot run as designed.
Item 5 means the "with MIC" gates test a pack that no longer ships. Item 6 sends the agent into P0-12
before its inputs exist. Fix items 1–9 before the implementing agent reaches P0-06.

## Resolution (plan revision 2, 2026-09-28)

All 30 gaps are addressed in the plan. None is rejected outright. Where the plan took a different
route from the suggested fix, the Resolution column says why. Phase 0 was renumbered so that the
numeric order is the work order. This review's P0 numbers are the **old** ones:

| Old | New | Task |
|---|---|---|
| P0-01, P0-02, P0-03 | P0-01, P0-02, P0-03 | bootstrap (now with the docs move), Cargo and CI, Gradle skeleton |
| P0-13 | P0-04 (smoke) and P0-05 (`gametest-planet.sh`) | server sets, smoke harness, planet server |
| P0-07 | P0-05 (analytic test terrain) and P0-10 (world data) | the generator |
| P0-08 | P0-06 | Toroidal World spike |
| P0-04 | P0-07 | JNI, loader, failure classes |
| P0-05 | P0-08 | cross-builds |
| P0-06 | P0-09 | format, synth, Java reader |
| P0-12 | P0-11 | `deeptime-inspect` |
| P0-09 | P0-12 | Terralith, TerraBlender, BOP |
| P0-10 | P0-13 | water |
| P0-11 | P0-14 | BlueMap |
| P0-14 | P0-15 | exit |

| # | Resolution | Where in the plan |
|---|---|---|
| 1 | Option (a): a gametest-only MixinExtras `@WrapOperation` on `WorldDimensions.bake` inside `GameTestServer`, passing `ctx.datapackDimensions().registryOrThrow(Registries.LEVEL_STEM)` (`@Local(argsOnly = true) WorldLoader.DataLoadContext`). MixinExtras 0.5.3 is in NeoForge 21.1.249's POM. It is the first step of P0-05, proven by `BootGameTests.testPlanetDimensionExists`. `gametest.sh` deletes the test levels' region folders and the old world data before `synth`. `spawnOnLand` calls the selection directly. Went further than the review: the height tests read chunks at NOISE status, because `*_WG` heightmaps exist only up to SURFACE status and carvers and features run later. `test_lake` uses a fixed carver-free biome, because 1.21.1's `WorldCarver.carveBlock` does not check for neighbouring water. The test pack is registered through `AddPackFindersEvent.addPackFinders` from a `gametest` source set that never ships. | §9.4, §7.12, §4.3, P0-05, P0-10, P0-13 |
| 2 | `git add -A` before any Nix command. §2.1 now states that flakes see only tracked files. | §2.1, §11.1 step 4 |
| 3 | Kept the pin: `(pkgs.cargo-zigbuild.override { zig = pkgs.zig_0_13; })`. The P0-01 check greps the resolved wrapper for `zig-0.13.0`. The nixpkgs input is `nixos-unstable`, as in mic-climate. | §4.4, §6.4, P0-01 |
| 4 | Ben chose `LGPL-3.0-or-later`, and it is in the cargo-deny allow-list (his instruction), so the review's `private.ignore` is not used. `publish = false` is still set, and every crate inherits `license` from `[workspace.package]`. | §2.3, §4.5, P0-02 |
| 5 | Deep Time owns its MIC server set: `tools/smoke/install.sh mic` runs packwiz-installer on `pack/pack.toml` into its own gitignored `smoke/mic/server/`, and `-PwithMic` symlinks there (`linkMicMods`). P0-04 comes before every `-PwithMic` gate, and each gate re-installs and records the pack commit. Every jar-derived fact was re-verified on the current pack's jars, downloaded per `pack/mods/*.pw.toml` into scratch. The Tectonic preset filter is a denylist (`#tectonic:incompatible`), so no compat mixin is needed. §13's versions were updated. mic-climate's `smoke/` is named only as stale. | §2.2, §4.3, §7 preamble, §7.1, §7.12, §9.5, §11.2, §13, P0-04 |
| 6 | Resolved by renumbering (table above), with every "Depends on" re-derived. The smoke harness is P0-04. Inspect (P0-11) comes before water (P0-13), which uses it. | §10.2 |
| 7 | The surface is defined as the `OCEAN_FLOOR_WG` value − 1, and the expected value is `ceil(H) − 1`. Corner columns have tolerance 0 and other columns tolerance 1, both logged. `planet_nocaves` is used, and chunks are read at NOISE status (see 1). The detail clause is gone, because the detail term starts at P1-06. | P0-05, P0-10, §7.1 |
| 8 | §6.6 names all five `FailureClass` values, each with its detection, guidance and forcing property (`forceFail=unsupported\|libc_mismatch\|io\|version_mismatch`, `testDeleteAfterWrite`). P0-07 requires at least 9 JUnit cases and one `NativeSmoke` line per class. Exit row 1 checks all five. | §6.6, P0-07, P0-15 |
| 9 | Ben's answer: the docs move into the repository's `docs/` at P0-01, with pointer files left behind (steps and sha256 check in P0-01). The decisions file is Ben's, and agents never edit it. STATUS has a `Plan revision` header line. | §1.2, §1.3, §11, §12, P0-01 |
| 10 | The stale sentence is deleted. §7.5 states what the datapack already lists. P1-12 has no datapack edits and keeps `coalFallsBackToStone`. | §7.5, P1-12 |
| 11 | §6.4 separates the rustup targets from the zigbuild `.2.17` specifications. `rust-toolchain.toml` has no targets, and the dev shell adds all eight through rust-overlay. | §6.4, §4.4, P0-01 |
| 12 | P0-03 adds the Gradle 8.11.1 wrapper, `empty_5x5x5.nbt` (from `tools/make_test_structure.py`, with `@PrefixGameTestTemplate(false)`), all mod properties, and the CI Gradle job. P0-02 adds edition 2024, `version = "0.0.1"`, and `rust-version = "1.89"` (the highest MSRV among the pins: kiddo 6.3.0). | P0-02, P0-03, §4.3 |
| 13 | JUnit BOM 5.14.4, Jupiter, the platform launcher, explicit lz4-java 1.8.0, `testLogging`, and `forkEvery = 1` for linking tests. The loader separates extract from link. Acceptance reads the XML reports through `tools/ci/junit-summary.py`. | §4.3, §6.5, §10.1, P0-03, P0-07 |
| 14 | `hostNatives` stages into `target/natives-host/` with its own `natives.json`. Only `-Prelease` packs `natives-dist/`. | §4.3, §6.5 |
| 15 | `NativeSmoke` prints from the `NativeStatus` that `NativeLoader.load` returns. | §4.5, P0-07 |
| 16 | There is one version source, the Cargo workspace, checked against `mod_version` by `checkVersions`. The loader compares semver and format only. §6.1 defines every entry point's Phase 0 behaviour and the dummy preset. Went further than the review: `selfTest` has no stored checksum, because a cross-built library cannot be run on the build host to record one. It is an integer Rayon reduction that Java recomputes. | §6.1, §6.5, P0-03, P0-07 |
| 17 | Every `cargo`, `gradlew`, `java`, `zig` and `readelf` command runs through `nix develop -c`. §2.1 documents the rustup shim and the profile's tools. | §2.1, §10.1, all acceptances |
| 18 | §1.3 is the only reading list, and §11.1 points to it. The line count is removed. | §1.3, §11.1 |
| 19 | Ben settled the deviations: 8 natives, a 4 GiB cap, and C of exactly 16,384 and 32,768 blocks. The note's 1,000/2,000 chunks are marked superseded. The review suggested pulling forward only the source lookup; instead the whole spike moved to P0-06, as early as its real dependencies (P0-03, P0-04, P0-05) allow, by giving P0-05 an analytic test terrain. | §1.2, §7.1, §10.2, P0-06 |
| 20 | The standalone world is created with `--synth wrap-test --size 16384`, and `--test-terrain` is the pre-P0-09 alternative. Chunky (`LuFhm4eU`) is in the `standalone` and `terrablender` sets. | §4.3, §9.5, P0-11 |
| 21 | The CI load jobs cover all eight targets: musl on x64 and arm64, and Windows arm64 when a runner label exists. §9.7 has `regions --span`. §9.4's class list is complete, with tasks. §5.5 has `deeptime:guaranteed_occurrence`. | §4.5, §5.5, §9.4, §9.7 |
| 22 | The §4.3 Modrinth table has TW `8YTSYi9y`, GlitchCore `S2TfWrZR` and the other runtime ids. | §4.3 |
| 23 | Pinned `spacesheep@1.17.0`, the current `latest`, with its README flags (it adds `--new`, which the plan forbids). | §2.6 |
| 24 | `compression: none`. The documented values at v5.28 are gzip, zstd, deflate and none (the review omitted deflate). The service worker is kept only as a fallback. | §9.6, P0-14 |
| 25 | The dev shell adds `llvmPackages.clang-unwrapped`, `lld` and `llvm`. P0-08 still confirms against cargo-xwin 0.22.0's README before the first Windows build. | §4.4, P0-08 |
| 26 | Ben accepted all three licence acceptances, including the EULA for every server the plan runs. No question is pending before P0-01. §2.4 notes the limit of four questions per call. | §2.4, §2.7, §11.1 |
| 27 | The draft CLAUDE.md now has the start-of-session routine, the memory rule, "mic-climate is read-only", `nix develop -c`, the docs paths in the repository, the repointed jar path, and the decision rule. Answers go in STATUS's "Questions asked and answers" table, because agents never edit the decisions file. The STATUS header has `Plan revision` and `MIC pack tested against`. | §11.2, §12 |
| 28 | The CI jobs cache `~/.gradle` and `~/.cargo` and use `concurrency: cancel-in-progress`. The Rust job builds for the host target only. `natives.yml` runs only on tags and manual dispatch, with `rustup target add`. | §4.5 |
| 29 | `synth --kind wrap-test --size 1024 --out build/test-worlds/wrap1024`. The test compares `elevation`, `water_level` and `climate/temperature` at every cell of the first and last tile plus 1,000 random cells. | P0-09 |
| 30 | The label now reads "Mojang names; Parchment where available". | §2.2 |

New question from this pass: **Q16** in §14. How does MIC ship the rock-tag datapack? `pack/` has no copy
and no global-datapack loader at pack commit `303fd89`. The recommendation is `pack/kubejs/data/deeptime/`.
