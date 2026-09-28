# Create Aeronautics / Sable: how the Rust physics bridge actually works

Researched 2026-09-27. Method: direct jar/bytecode inspection (unzip, `javap -p -c`,
`nm -D`, `readelf`, `strings`) of the exact jars in this pack's smoke-test copy, plus
the public `ryanhcode/sable` and `ryanhcode/rapier` GitHub repos (source, Cargo.toml,
CI workflows) and the `ryanhcode/sable` GitHub issue tracker. Every claim below is
tagged **[verified: <source>]** or **[inferred: <reasoning>]**.

Jars inspected (from `/home/greencheetah/Projects/mic/mods/mic-climate/smoke/server/mods/`,
matching `/home/greencheetah/Projects/mic/pack/mods/*.pw.toml`):
`create-aeronautics-bundled-1.21.1-1.3.2.jar`, `sable-neoforge-1.21.1-2.0.5.jar`,
`cc_sable-neoforge-1.3.4.jar`, `createpropulsion-1.1.5.jar`,
`ragdoll_reactions-1.21.1-0.7.0.jar`, `sable_player_ragdoll-1.21.1-0.7.5.jar`,
`kubejsable-1.1.0.jar`. (`drivebywire-0.3.0.jar` was not present in the local smoke
pack and was not fetched — see "Not covered" at the end; it is a thin gameplay layer
like the other companion mods and near-certain not to carry its own natives.)

---

## 1. Which jars contain native code

Only **one** artifact in the whole dependency chain contains native binaries, and it
is nested **two jarjar levels deep** from the mod the pack lists directly:

```
create-aeronautics-bundled-1.21.1-1.3.2.jar        (outer mod, MIT+ARR, 33 MB)
 └─ META-INF/jarjar/ (NeoForge nested-jar mechanism, per META-INF/jarjar/metadata.json)
     ├─ dev.eriksonn.aeronautics...jar
     ├─ dev.ryanhcode.offroad.offroad...jar
     └─ dev.simulated_team.simulated...jar   ← no natives, no further nesting; this is
                                                the Create-style content mod that only
                                                *consumes* Sable's Java API

sable-neoforge-1.21.1-2.0.5.jar   (separate Modrinth dependency, NOT nested inside
                                    the aeronautics bundle above — pulled in by its
                                    own pw.toml, PolyForm Shield 1.0.0, 13 MB)
 └─ META-INF/jarjar/ (its own nested jars)
     ├─ dev.ryanhcode.sable-companion...jar        (35 KB, no natives)
     ├─ foundry.veil.veil-neoforge...jar           (3.3 MB, no natives — rendering lib)
     └─ dev.ryanhcode.sable-sable_rapier-1.21.1-2.0.5.jar   (7.6 MB)
         └─ natives/sable_rapier/sable_rapier_binaries.zip.l4z   (9,072,872 bytes)
             = LZ4-framed ZIP, embedded directly as a jar resource (not downloaded
               at runtime). Decompresses to a 15,431,304-byte ZIP containing 6 files:
```

| File | Size | Format |
|---|---|---|
| `sable_rapier_x86_64_windows.dll` | 1,658,368 B | PE32+ DLL, x86-64 |
| `sable_rapier_aarch64_windows.dll` | 1,340,416 B | PE32+ DLL, ARM64 |
| `sable_rapier_x86_64_macos.dylib` | 4,693,160 B | Mach-O 64-bit, x86_64 |
| `sable_rapier_aarch64_macos.dylib` | 4,257,344 B | Mach-O 64-bit, arm64 |
| `sable_rapier_x86_64_linux.so` | 1,887,608 B | ELF64 LSB, x86-64, **stripped** |
| `sable_rapier_aarch64_linux.so` | 1,594,408 B | ELF64 LSB, ARM aarch64, **stripped** |

**[verified: `unzip -l`/`file` on the extracted jars, this session]**

Coverage is exactly 3 OSes × 2 arches (Win/macOS/Linux, x64/arm64). **No musl/Alpine
variant, no 32-bit, no RISC-V/LoongArch.** The Linux `.so` files are glibc-linked
(`readelf -d`: NEEDED = `libc.so.6, libm.so.6, libpthread.so.0, libdl.so.2,
ld-linux-x86-64.so.2`; `readelf -V` shows symbol versions up to `GLIBC_2.17`)
**[verified: readelf, this session]** — this is later confirmed as a real user-facing
gap by GitHub issues #13 and #884 (section 5).

The other five mods (`cc_sable`, `createpropulsion`, `ragdoll_reactions`,
`sable_player_ragdoll`, `kubejsable`) were re-checked with a corrected `unzip -l`
and contain **no** `.so/.dll/.dylib/.jnilib` and **no** `META-INF/jarjar/` nesting at
all **[verified: this session]** — they are pure-Java gameplay/scripting layers on
top of Sable's public API and never touch natives directly.

---

## 2. Is it Rust

Yes, unambiguously. From `strings` on the (stripped) Linux `.so`
**[verified: this session]**:

- Rustc debug paths: `/rustc/de6d33c033441c5880b863f94d7a3ec8cad141bd/library/{core,alloc,std}/...`
- Local workspace source paths baked into panic messages:
  `rapier/src/{algo,boxes,buoyancy,collider,config,contraptions,dispatcher,event_handler,groups,hooks,joints,lib,rope,scene,voxel_collider}.rs`
  and `marten/src/{level,octree,lib}.rs` — these paths match the real GitHub source
  tree exactly (section 6).
- Cargo-registry paths naming exact crate versions: `cesu8-1.1.0`, `crossbeam-deque-0.8.6`,
  `crossbeam-epoch-0.9.18`, `dashmap-7.0.0-rc2`, `fern-0.6.2`, `humantime-2.3.0`,
  `glam-0.30.10`, `glamx-0.1.3`, `hashbrown-0.15.5`/`0.16.1`, `jni-0.21.1`,
  `nalgebra-0.34.1`, `rayon-1.11.0`, `rayon-core-1.13.0`.
- `parry3d` panic strings (e.g. `parry3d::query::epa::epa3`, "Hit unexpected state in EPA…") —
  `parry3d` is the collision-detection crate that Rapier is built on.
- Rayon runtime strings: `RAYON_NUM_THREADS`, `RAYON_RS_NUM_CPUS`, "The global thread
  pool has not been initialized.", `ThreadPoolBuildError`, `GlobalPoolAlreadyInitialized`.
- No `_ZN…` mangled C++/Rust-legacy-mangling symbols (consistent with a stripped
  binary — internal names aren't directly enumerable, only inferable from leaked
  string literals as above).

This was independently confirmed from the public source (section 6): `Cargo.toml`
depends on `jni`, `rapier3d` (git dependency on `ryanhcode/rapier`), `rayon`,
`dashmap`, `nalgebra` (transitively via rapier3d), matching the strings 1:1
**[verified: raw.githubusercontent.com Cargo.toml fetch, this session]**.

---

## 3. How Java calls it — JNI, synchronous, no side-channel thread

**Mechanism: classic JNI**, not JNA/Panama-FFM/a separate process/shared memory.
`dev/ryanhcode/sable/physics/impl/rapier/Rapier3D.class` declares 53
`private static native` methods; `nm -D` on the Linux `.so` shows **exactly 53**
dynamic symbols, all named `Java_dev_ryanhcode_sable_physics_impl_rapier_Rapier3D_<method>`
— a 1:1 match with zero extra exported symbols **[verified: `javap -p -c` +
`nm -D`, this session]**. Representative methods: `initialize`, `tick`, `step`,
`createSubLevel`, `createBox`, `getPose`, `applyForce`, `applyForceAndTorque`,
`createKinematicContraption`, `createRope`, `queryRope`, `configSolverIterations`,
`dispose`.

**"Side-channel" answer: there isn't one — it's a synchronous blocking call, once
per tick, on whatever thread calls it.** Decompiling
`RapierPhysicsPipeline.class` (`javap -p -c`) shows the tick lifecycle
`prePhysicsTicks() → physicsTick(double) → postPhysicsTicks() → tick()` and a
field list (`level`, `accelerator`, `colliderBakery`, `activeSubLevels`
(Int2ObjectMap), `activeContraptions` (Object2ObjectMap), `recentCollisions`
(Long2LongOpenHashMap), `queuedWakeUps`, `poseCache` (double[]), `scene`
(RapierPhysicsScene)). A grep of the decompiled bytecode for
`Thread|Executor|Future|Runnable` returns **zero matches**
**[verified: this session]**. There is no Java-side background thread, queue, or
ring buffer — the mod calls into native code directly from the Minecraft tick loop
and blocks until it returns. **Any concurrency is internal to that one blocking
call**: the Rust side uses `rayon` (a work-stealing data-parallel thread pool) to
parallelize its own physics step across worker threads it manages itself, invisible
to Java.

State is held as an **opaque native handle**: `RapierPhysicsScene` is just
`private final long handle;` (a boxed Rust pointer/index) **[verified: `javap -p`]**.
`Rapier3D.getSceneHandle(ServerLevel)` resolves per-level state via
`SubLevelPhysicsSystem.require(level).getPipeline()`.

**Provider selection (the "which implementation runs" question, resolved this
session):** `dev.ryanhcode.sable.api.physics.PhysicsPipelineProvider` is a Java
interface with a **`java.util.ServiceLoader`-based static factory**, decompiled in
full:

```
static {};
  ServiceLoader.load(PhysicsPipelineProvider.class)
    .stream()
    .max(Comparator.comparingInt(provider -> {
        LoadPriority ann = provider.type().getAnnotation(LoadPriority.class);
        return ann != null ? ann.value() : 1000;   // no annotation defaults to 1000
    }))
    .map(ServiceLoader.Provider::get)
    .orElseThrow(() -> new RuntimeException("Failed to find any physics pipeline providers"));
```

Two providers are registered via `META-INF/services/dev.ryanhcode.sable.api.physics.PhysicsPipelineProvider`:
`dev.ryanhcode.sable.physics.impl.none.StaticPhysicsPipelineProvider` (in the base
`sable` jar) and `dev.ryanhcode.sable.physics.impl.rapier.RapierPhysicsPipelineProvider`
(in the jarjar-nested `sable_rapier` jar). `StaticPhysicsPipelineProvider` carries an
explicit `@LoadPriority(900)` annotation; `RapierPhysicsPipelineProvider` carries
**no** annotation, so it defaults to priority **1000** and always wins the
`Stream.max()` comparison whenever it's present on the classpath
**[verified: `javap -v` RuntimeVisibleAnnotations on both provider classes, this
session]**. Since `sable_rapier` is *always* jarjar-bundled inside every normal
`sable-neoforge-*.jar` download, Rapier always wins in practice — the "none"
provider is a build-time/deployment-time fallback (only relevant if a modpack
author manually strips the `sable_rapier` nested jar), **not** a runtime
graceful-degradation response to a native-load failure.

---

## 4. How natives are loaded — extraction, selection, and failure behavior

Fully reverse-engineered from `javap -p -c` on `Rapier3D.class`'s static
initializer and its `resolveNativeDir()` / `getNativeName()` / `loadLibrary()`
methods **[verified: this session]**:

1. **Platform/arch naming** is built from a bootstrap-method string template
   embedded in the class file: `sable_rapier_\u0001_windows.dll`,
   `sable_rapier_\u0001_macos.dylib`, `sable_rapier_\u0001_linux.so`, where the
   placeholder is filled with `x86_64` or `aarch64` based on `os.name`/`os.arch`
   system properties.
2. **Extraction target**: a `.sable/natives/` directory (confirmed by crash-log
   paths in real issues, e.g. `.sable/natives/sable_rapier_aarch64_macos.dylib`,
   `/home/****/.sable/natives/sable_rapier_x86_64_linux.so`) under the game
   directory (obtained via `SableLoaderPlatformImpl.getGameDirectory()` →
   NeoForge's `FMLPaths.GAMEDIR`).
3. The matching binary is located inside the bundled `sable_rapier_binaries.zip.l4z`
   (LZ4-framed ZIP, decompressed via `net.jpountz.lz4.LZ4FrameInputStream` +
   `java.util.zip.ZipInputStream`, both in-process — **no network fetch at
   runtime**), written out to `.sable/natives/`, then loaded with
   `System.load(<absolute path>)` (not `System.loadLibrary`, so no
   `java.library.path` involvement).
4. No explicit version/hash check of the extracted file was found in the
   decompiled bytecode; behavior on repeated launches appears to be
   overwrite-every-launch (re-extract unconditionally) rather than checksum-and-skip.
5. **Failure behavior is a hard crash, not a graceful fallback.** The
   `loadLibrary()` bytecode wraps the extraction/`System.load` sequence in a
   catch-`Throwable` block that builds a Minecraft `CrashReport` and throws
   `ReportedException` — this propagates out of the static initializer (which runs
   the very first time `Rapier3D` is touched, from `RapierPhysicsPipeline.init()`
   called by `SubLevelPhysicsSystem.initialize()`) and crashes the client or
   dedicated server. **There is no automatic fallback to the pure-Java
   `StaticPhysicsPipeline` on a load failure** — that fallback (section 3) is only
   reachable if the Rapier *provider itself* is absent from the classpath, not if
   it's present but its native fails to load.
6. **The pure-Java fallback (`StaticPhysicsPipeline`) is a genuine complete no-op**,
   confirmed by decompiling every method: `init/dispose/prePhysicsTicks/physicsTick/
   postPhysicsTicks/tick/add/remove/handleChunkSectionAddition/handleChunkSectionRemoval/
   handleBlockChange` are all a bare `return`; `readPose`/`teleport` just copy through
   the already-set logical (pre-physics) pose unchanged; `addRope`/`addBox` return
   `null` handles. In other words: with the static pipeline, "physics objects" are
   purely kinematic passthroughs that never move under simulated forces — there is
   no partial/degraded simulation, just off.

Real-world crashes matching this exact mechanism are documented in the issue
tracker (section 5), including one with symbol-accurate source line numbers from a
stack trace: `Rapier3D.loadLibrary(Rapier3D.java:90)` →
`Rapier3D.<clinit>(Rapier3D.java:45)` → `RapierPhysicsPipeline.init(RapierPhysicsPipeline.java:121)`
→ `SubLevelPhysicsSystem.initialize(SubLevelPhysicsSystem.java:171)`
**[verified: github.com/ryanhcode/sable issue #1423 body]** — this independently
corroborates the call chain reconstructed from bytecode above.

---

## 5. Known problems and fixes (from the `ryanhcode/sable` issue tracker, GitHub API, 2026-09-27)

The repo has 977 open issues total; the following are the ones directly matching
the native-loading mechanism above, found by searching titles/bodies for the
keyword classes the task asked about **[verified: `api.github.com/search/issues`
+ individual issue fetches, this session — issue numbers/titles/states/quoted text
below are as returned by the GitHub API on 2026-09-27]**:

- **musl/Alpine — confirmed real, still unresolved.**
  - [#13](https://github.com/ryanhcode/sable/issues/13) "Natives missing for
    musl-based distros" (**open**, labels `type: crash`, `related: natives`).
    Reporter's `ldd` on the extracted `.so` inside an `itzg/minecraft-server:java21-alpine`
    container: `Error loading shared library ld-linux-x86-64.so.2: No such file or
    directory`. This directly confirms the glibc-linkage finding from section 1 as
    a real deployment failure, not just a theoretical risk.
  - [#884](https://github.com/ryanhcode/sable/issues/884) "Add musl-linked Rapier
    natives for Alpine Linux" (**closed**, claims "Closes #13" but #13 remains open)
    — body explains "Linux artifacts were only built for `*-unknown-linux-gnu`
    (glibc)... musl's dynamic linker... [natives] are not loadable there," and
    separately flags an unrelated `RuntimeDistCleaner` classloading warning found
    while debugging, noting explicitly that shipping musl natives would not fix
    *that* second problem. **Net effect as of this research: musl/Alpine hosting
    is still not supported** — no musl variant exists in the 6-file native bundle
    (section 1).
- **glibc version skew — confirmed.**
  [#592](https://github.com/ryanhcode/sable/issues/592) "Server Host (Modrinth)
  crash, glibc 2.27 incompatibility" (**closed**). Crash log:
  `UnsatisfiedLinkError: .../sable_rapier_x86_64_linux.so: /lib64/libm.so.6: version …`
  — an older server host's glibc lacked a required symbol version. (The issue body
  excerpt available via the API didn't show the specific fix; treat "closed" as
  "closed on the tracker," not necessarily "root-caused and patched.")
- **Antivirus false positive — confirmed, no code-side fix.**
  [#691](https://github.com/ryanhcode/sable/issues/691) "Microsoft Defender
  flaggin as Trojan" (**closed**, 14 comments). Windows Defender quarantined
  `sable_rapier_x86_64_windows.dll` as malicious, crashing the game when the file
  was missing. Only workaround reported: manually add a Defender exclusion; no
  mention of code signing being pursued.
- **macOS ARM64 native-extraction bug — confirmed, currently open, recurring.**
  Three independent open issues report the *same* failure signature
  (`java.nio.file.NoSuchFileException: .sable/natives/sable_rapier_aarch64_macos.dylib`)
  across different Sable versions and macOS releases:
  [#1423](https://github.com/ryanhcode/sable/issues/1423) (macOS ARM64, includes
  the source-line stack trace quoted in section 4),
  [#1291](https://github.com/ryanhcode/sable/issues/1291) (Sable ≥2.0.0, macOS
  Tahoe 26.3, reporter found no `.dylib` files at all and no `.sable` folder
  created), and [#1369](https://github.com/ryanhcode/sable/issues/1369) (Apple
  M4, macOS 15.7.4, Sable 2.0.1/2.0.3, tried both Java 21 and Java 25). This reads
  as a genuine bug in the extraction/directory-creation step on macOS specifically
  (the parent directory not existing when `Files.createFile()` runs), not
  Gatekeeper/notarization — **explicit keyword searches for "Gatekeeper" and
  "notariz(ation)" in the issue tracker returned zero results**, so the task's
  hypothesis that Gatekeeper is the macOS pain point is not supported by the
  evidence found; the actual observed macOS failure mode is this extraction bug.
- **Windows dedicated server + newer JDK.**
  [#912](https://github.com/ryanhcode/sable/issues/912) "UnsatisfiedLinkError:
  Rapier3D native fails to link on dedicated server (Windows 11, Java 25)"
  (**open**). Notable because the reporter states it "works fine on the client
  side — server-only reproduction," which is independent evidence (see section 7)
  that the client and a dedicated server each load/initialize the native library
  separately rather than the client only ever consuming server-sent state.
- **Combined with Create Aeronautics specifically.**
  [#754](https://github.com/ryanhcode/sable/issues/754) "Crash: UnsatisfiedLinkError
  (Rapier3D.initialize) when loading world with Aeronautics" (**open**) — confirms
  this exact failure class is user-visible in the Aeronautics+Sable combination
  this research is modeled on, not just in Sable standalone.
- No issues were found for SIGILL specifically (search returned 0 results), nor
  for architecture requests beyond what's shipped (a closed feature request,
  [#650](https://github.com/ryanhcode/sable/issues/650), asked for LoongArch64/RISC-V64
  natives — closed, presumably declined/wontfix given no such variants exist in
  the bundle).

**Overall pattern**: essentially all reported native-loading failures fall into
exactly the categories predictable from the static analysis in sections 1 and 4 —
platform/libc coverage gaps (musl), ABI version skew (old glibc), antivirus
heuristics on an unsigned, unusually-dense (Rust-runtime-containing) DLL, and a
platform-specific bug in the extract-and-load bootstrap code (macOS). There is no
evidence of a SIGILL/illegal-instruction class of bug (which would typically mean
"CPU doesn't support the instruction set the binary was compiled for," e.g. an
AVX512 build running on an older CPU) — consistent with the Cargo.toml using
`simd-nightly` (portable, runtime-width-selected SIMD) rather than a fixed-width
hand-picked instruction set.

---

## 6. Source, build, and distribution

**Licensing — three different licenses across the dependency chain**
**[verified: `LICENSE.md` extracted directly from each jar, this session; cross-checked
against Modrinth API `license` field for the `sable` project, this session]**:

| Component | License | Notes |
|---|---|---|
| `create-aeronautics-bundled` code | MIT | Copyright "The Simulated Team / The Creators of Aeronautics" |
| `create-aeronautics-bundled` assets | All Rights Reserved | under each sub-mod's `assets/` paths |
| `sable` base jar | **PolyForm Shield License 1.0.0** | noncompete source-available license, confirmed both from the jar's bundled `LICENSE.md` and Modrinth's API: `"license":{"id":"LicenseRef-PolyForm-Shield-License-1.0.0","url":"https://github.com/ryanhcode/sable/blob/main/LICENSE.md"}` |
| `sable_rapier` (the JNI bridge jar) | **PolyForm Shield License 1.0.0** | identical text to the base jar |
| bundled Rapier fork (`LICENSE-RAPIER` inside the natives folder) | Apache License 2.0 | matches upstream dimforge/rapier's own license |

**PolyForm Shield 1.0.0 is source-available, not OSI open source: it lets you use,
modify, and redistribute the software, but forbids using it to compete with the
licensor.** This matters concretely for Deep Time: the *architecture/technique*
(JNI + bundled LZ4-zipped natives + Rayon) isn't copyrightable and copying that
pattern is unrestricted, but literally vendoring Sable/sable_rapier source or
binaries would carry this noncompete condition (Deep Time, a geology/worldgen mod,
is very unlikely to "compete" with a physics/vehicle-simulation library, but this
is a real license term to be aware of, not a hypothetical one). The Rapier engine
core itself (`ryanhcode/rapier`, a fork of `dimforge/rapier`) is genuinely
Apache-2.0 and freely reusable with attribution.

**Source is public.** Both repos exist and are public on GitHub
**[verified: WebFetch of both repo pages + raw file fetches, this session]**:

- `github.com/ryanhcode/sable` — 488 stars, 221 forks, 286 commits on `main`.
  Gradle multi-module project: `common`, `fabric`, `neoforge` (the Java/Minecraft
  side) plus `sable_rapier` (Gradle module wrapping the Rust build).
- `github.com/ryanhcode/rapier` — a genuine git fork of `dimforge/rapier` (488→ no,
  11 stars/3 forks/1,264 commits on `master`), Apache-2.0, workspace crates
  `rapier2d`, `rapier2d-f64`, `rapier3d`, `rapier3d-f64` (+ testbeds/examples),
  version `0.32.0`, `rust-version = "1.86"`, edition 2024. This is the *generic*
  physics-engine fork; Sable depends on it as a git dependency, it is not itself
  Sable-specific.

**The actual Sable↔Rust integration layer** lives inside the `sable` repo itself,
at `sable_rapier/src/main/rust/`, fetched and read directly
**[verified: raw.githubusercontent.com, this session]**:

```
sable_rapier/src/main/rust/
├── .cargo/config.toml       # target-specific rustflags (see below)
├── Cargo.toml               # workspace root: members = ["marten", "rapier"]
├── Cargo.lock
├── rust-toolchain.toml      # channel = "nightly-2026-01-29"  (PINNED NIGHTLY, not stable)
├── container/
│   ├── xwin/Dockerfile      # cargo-xwin: cross-compile to Windows MSVC targets from Linux
│   └── zigbuild/Dockerfile  # cargo-zigbuild: Zig-backed cross-linking (glibc-version
│                             # targeting / cross-compiling to macOS from Linux)
├── marten/                  # small crate: level.rs, octree.rs — spatial partitioning
│                             # utility, only depends on jni/log/fern/humantime/colored,
│                             # no rapier/parry/nalgebra deps at all
└── rapier/                  # the actual JNI-facing crate, package name "sable_rapier"
    ├── Cargo.toml
    ├── benches/collision_benchmark.rs
    └── src/{algo,boxes,buoyancy,collider,config,contraptions,dispatcher,
             event_handler,groups,hooks,joints,lib,rope,scene,voxel_collider}.rs
```

This confirms (and refines) the strings-based inference from section 2: there are
really **two** "rapier"-named things — the generic forked engine repo
(`ryanhcode/rapier`, pulled in as a Cargo git dependency) and this local `rapier/`
crate inside `sable`'s own repo, which is Sable's Minecraft-specific extension on
top of it (voxel colliders, "contraptions" = Create-style moving structures,
buoyancy, rope) and is also the crate that is compiled to a `cdylib` and exports
the 53 `Java_...` JNI symbols. The workspace's exact dependency line, fetched
directly:

```toml
rapier3d = { git = "https://github.com/ryanhcode/rapier",
             rev = "38e92f117590862481a53df6fc69a5d893e29186",
             features = ["simd-nightly", "parallel"] }
#rapier3d = { version = "0.26.1", features = [ "simd-stable" ] }   # commented-out alternative
rayon = "1.10.0"
dashmap = "7.0.0-rc2"
jni = "0.21.1"
```

pinned to an exact commit (not a branch/tag) — good reproducibility practice — and
`[lib] crate-type = ["cdylib", "lib"]` on the `rapier` (package name `sable_rapier`)
crate is what actually produces the `.so`/`.dll`/`.dylib` files.

**Build/CI, concretely**: a dedicated GitHub Actions workflow,
`.github/workflows/build-rust.yml` (fetched in full)
**[verified: raw.githubusercontent.com, this session]**, triggers on push to `main`
when Rust source changes (or manual `workflow_dispatch`), runs on a single plain
`ubuntu-latest` runner (**no OS/arch build matrix** — all 6 platform binaries are
cross-compiled from one Linux box via the Dockerized `xwin`/`zigbuild` toolchains
above), executes `./gradlew sable_rapier:buildImages sable_rapier:buildRustNatives`,
and then **the CI bot commits the rebuilt native binaries directly back into the
git repository** (`git add . && git commit -m "Automated build rust natives" && git push`)
rather than publishing them as separate release artifacts fetched at Gradle-build
time. The repo also has a **separate Jenkins pipeline** (`.jenkins/Jenkinsfile`,
`.jenkins/snapshot/Jenkinsfile`) alongside the GitHub Actions workflows
(`build-rust.yml`, `gametests.yml`, `publish-wiki.yml`, `release.yml`) — i.e., dual
CI, GitHub Actions for the Rust-native build/gametests/wiki, Jenkins presumably for
actual release packaging/publishing.

Two documented cross-compilation gotchas baked into `.cargo/config.toml`
**[verified: raw fetch, this session]**:
```toml
[target.x86_64-pc-windows-gnu]
rustflags = ["-C", "link-arg=-Wl,-Brepro"]   # reproducible-build linker flag

[target.x86_64-apple-darwin]
# Required to avoid a rustc hang when building for this target.
rustflags = ["-C", "codegen-units=2"]
```

Binary sizes: see the table in section 1 (1.3–4.7 MB per platform binary,
~15.4 MB uncompressed for all 6, ~9.1 MB as the LZ4-compressed bundle actually
shipped in the jar).

---

## 7. Determinism

**Directly answered by the Cargo.toml feature flags — not inconclusive.**
`rapier3d` is pulled in with `features = ["simd-nightly", "parallel"]` and
**`enhanced-determinism` is not enabled** (nor is it present in the commented-out
`simd-stable` alternative line) **[verified: raw Cargo.toml fetch, this session]**.
Rapier's own upstream design explicitly documents `enhanced-determinism` as the
feature required to get cross-platform bit-exact reproducibility, and states that
enabling SIMD width optimizations and parallel (multi-threaded, work-stealing)
solving both change floating-point summation order in ways that break bit-exact
determinism across different machines/thread counts. Sable/Aeronautics enables
exactly the two features that trade away that guarantee for speed. **Practical
conclusion: this physics simulation is not guaranteed deterministic across
platforms, architectures, or even repeated runs on the same machine with a
different thread count.**

Whether that actually matters in practice depends on whether the client also runs
its own physics step or only renders server-authoritative state:
- **[verified]** issue [#912](https://github.com/ryanhcode/sable/issues/912)
  states the failure is server-only and explicitly "works fine on the client
  side" — meaning the **client independently loads and initializes its own copy of
  the native Rapier pipeline**, separate from a dedicated server's copy (this
  wouldn't be possible/meaningful if the client never touched the native library
  at all).
- **[inferred, not confirmed]**: this session did not confirm whether the client
  actually *re-simulates* physics locally (client-side prediction, which the lack
  of determinism would make visibly diverge from the server and require
  reconciliation) versus loading the native library client-side only for
  lower-stakes work (e.g. voxel-collider baking for rendering, or sub-level
  interpolation) while trusting server-sent transforms for authoritative motion.
  Resolving this would require decompiling the client-specific classes (not done
  in this pass) — flagged as an open question rather than asserted either way.

No `enhanced-determinism`, `simd-nightly`-vs-`simd-stable` divergence risk, or
determinism-related strings were found in the compiled binary's `strings` output
either (searched for `determinism|simd-nightly|enhanced_determinism`, zero
matches) — consistent with the feature being compiled out entirely rather than
present-but-disabled at runtime.

---

## 8. Takeaway for Deep Time

**What to copy:**
1. **JNI, called synchronously once per tick, no side-channel thread on the Java
   side.** This is much simpler than it might sound going in — you don't need a
   queue, ring buffer, or background thread pumping messages between Java and
   Rust. A plain blocking `native` call per tick, with all state kept behind an
   opaque `long handle` (boxed Rust pointer) on the Java side, is exactly what a
   proven, shipping, widely-used (5.7M+ Modrinth downloads on Sable alone per its
   API record) Minecraft mod does for a much harder real-time physics problem than
   Deep Time's turn-based/offline-style geologic simulation needs. Given Deep
   Time's workload (1k²–4k² grids, hundreds of steps) is not a hard 20-tick/sec
   real-time budget the way rigid-body physics is, this pattern comfortably has
   headroom to spare.
2. **Use `rayon` inside the native call for intra-step data parallelism** rather
   than trying to parallelize across the JNI boundary — this is exactly the kind
   of embarrassingly-parallel grid workload (per-cell erosion/plate-stress updates)
   Rayon is built for, and it requires zero Java-side awareness.
3. **Bundle natives as an in-jar resource (LZ4-zipped, extracted to a per-mod
   directory under the game dir at first use), not a network download.** Avoids
   both a runtime network dependency and antivirus/firewall friction on the
   download path itself (though not on the extracted DLL itself — see pitfalls).
4. **Use `cargo-zigbuild` + `cargo-xwin` in Docker to cross-compile all 6
   platform/arch targets from a single Linux CI runner.** This is a well-proven,
   concretely-documented way to avoid needing real Windows/macOS build machines —
   copy this over near-verbatim rather than re-deriving a cross-compile setup.
5. **Pin the Rust toolchain and the physics-engine commit exactly**
   (`rust-toolchain.toml` + a `rev = "<full commit sha>"` git dependency) for
   reproducibility.
6. **Ship a genuine pure-Java no-op fallback**, selected via `ServiceLoader` +
   a priority annotation, for the case where native code isn't present at all
   (e.g. an unsupported platform) — even though Sable's own fallback is only
   reachable at the "provider not on classpath" level rather than as an automatic
   response to a load *failure* (see pitfall below), having *some* legitimate
   pure-Java code path is cheap insurance and precedent to follow.

**Pitfalls to avoid (each backed by a real, currently-open issue against exactly
this codebase):**
1. **Don't silently support only glibc Linux.** Ship musl/Alpine natives too, or
   at minimum detect musl at startup and fail with an actionable message instead
   of a bare `UnsatisfiedLinkError`. Sable still hasn't fully solved this
   (issue #13, open) years into shipping — worth avoiding from day one rather than
   patching later.
2. **Don't let a native-load failure be an unrecoverable hard crash.** Sable's
   `loadLibrary()` catches the failure only to convert it into a full Minecraft
   crash report — it does not fall back to a degraded pure-Java mode
   automatically. For Deep Time (an offline/background-style simulation rather
   than a moment-to-moment gameplay-critical system), a caught failure should
   downgrade to a slower pure-Java (or simply "feature disabled") path with a
   clear log message, not crash world generation entirely.
3. **Expect antivirus false positives on the shipped DLL** (confirmed, issue #691)
   — an unsigned, dense, Rust-runtime-containing native DLL is a plausible
   heuristic-AV trigger. Code-signing the Windows DLL (cost/process TBD) is worth
   evaluating up front rather than after the first user bug report.
4. **Test the extraction/directory-creation path specifically on macOS ARM64
   before shipping** — this is Sable's single most recurring open bug class
   (3 separate issues, spanning macOS Sequoia and Tahoe, Sable versions 2.0.0
   through 2.0.5) and it's a bug in the mod's own bootstrap code, not an OS
   security feature fighting back.
5. **Decide deliberately about `enhanced-determinism` rather than defaulting it
   off for speed.** Sable/Aeronautics explicitly chose `simd-nightly` +
   `parallel` over `enhanced-determinism`, i.e. chose speed over cross-run
   reproducibility. For Deep Time, if any part of the pipeline needs to be
   reproducible (e.g. a fixed world seed producing an identical terrain history
   for sharing/debugging, or a server-authoritative result that must match a
   client's local preview), that needs to be an explicit design decision made
   early — retrofitting determinism into an already-parallel/SIMD physics-style
   pipeline is a much bigger change than choosing the feature flag up front.
6. **A pinned nightly Rust toolchain is a real, ongoing maintenance cost** —
   Sable pins `nightly-2026-01-29` specifically because `simd-nightly` requires
   nightly-only facilities. If Deep Time doesn't need that specific SIMD feature,
   staying on stable Rust removes a whole category of toolchain-breakage risk
   that this codebase has explicitly signed up for.

---

## Not covered / would need more time

- `drivebywire-0.3.0.jar` was not downloaded/inspected (not present in the local
  smoke pack; judged extremely unlikely to carry its own natives given it's a
  thin vehicle-control-scheme mod like the other companion mods, all of which
  were confirmed native-free).
- Whether the client performs its own local physics *prediction* step (vs. only
  loading the native lib for secondary purposes while trusting server state) was
  not resolved — see the caveat in section 7.
- The exact resolution/fix (if any) for glibc-version issue #592 wasn't visible in
  the issue body excerpt fetched; only its "closed" status is confirmed.
- Did not decompile `SubLevelPhysicsSystem.class` beyond confirming it's the
  `require(level).getPipeline()` entry point referenced by `Rapier3D` and the
  crash stack trace in issue #1423 — its full internals (how/when a
  `PhysicsPipeline` gets attached to a level) weren't traced end-to-end.
