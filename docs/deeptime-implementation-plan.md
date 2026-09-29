# Deep Time — implementation plan

Written 2026-09-28 by Opus for Ben. **Revision 2** (same day) folds in the 30 gaps of the first
cold-read review (`docs/plan-reviews/2026-09-28-cold-read-review.md`) and Ben's answers to the first
open questions. Implementation started on 2026-09-28 (P0-01); from then on this plan is edited in the
same commit series as the code that changes it (§12.3), and `STATUS.md` records progress. The gaps of
the second cold-read review (`docs/plan-reviews/2026-09-28-cold-read-review-2.md`) are fixed here by
the task that hits each one.

**Contents.** 1 Start here · 2 Environment and conventions · 3 Architecture overview · 4 Repository
layout · 5 Core data model · 6 The JNI boundary · 7 The Minecraft adapter · 8 Engine stages ·
9 Review and test pipeline · 10 Phase plan (Phase 0 and 1 work items, Phases 2–3, later) ·
11 First session · 12 Progress and handoff · 13 Reuse vs build ledger · 14 Open questions

---

## 1. Start here

### 1.1 What Deep Time is

Deep Time (mod id `deeptime`) is a NeoForge 1.21.1 mod that simulates a planet's geologic history
and then lets you play that planet. A Rust simulation engine moves tectonic plates on a sphere,
erodes and deposits the land, runs a parametric climate, forms rock strata and ore deposits, and
records everything that happened in one shared event log. The player watches the history as a
timelapse on the Create World screen. The result is written into the world folder as tiled data,
and a Java adapter turns that data into a Minecraft overworld: terrain, climate-driven biomes
(Terralith, TerraBlender mods), strata built from the pack's own stone blocks, and ore deposits
that exist where the geology put them. The world wraps east–west like a real planet (through the
Toroidal World mod) and has soft polar caps. Deep Time ships no blocks of its own. It is built for
the MIC modpack first, is licensed LGPL-3.0-or-later, and stays usable by any pack.

### 1.2 Where the documents live, and which one wins

**Location.** Until P0-01 lands, every document is in `/home/greencheetah/Projects/mic/Docs/`. P0-01
moves them into the repository's `docs/` folder, where they are versioned with the code, and leaves
pointers behind (decisions file, "Docs location"). From then on the repository copy is the only one:

| Document | Until P0-01 | From P0-01 on (repo = `/home/greencheetah/Projects/mic/mods/deep-time`) |
|---|---|---|
| Decisions | `/home/greencheetah/Projects/mic/Docs/deeptime-design-decisions.md` | `docs/deeptime-design-decisions.md` |
| This plan | `/home/greencheetah/Projects/mic/Docs/deeptime-implementation-plan.md` | `docs/deeptime-implementation-plan.md` |
| Research report | `/home/greencheetah/Projects/mic/Docs/Geologic history worldgen mod.md` | `docs/research/geologic-history-worldgen-report.md` |
| Research notes | `/home/greencheetah/Projects/mic/Docs/research_notes/Geologic history worldgen mod/*.md` | `docs/research/notes/*.md` (same file names) |
| Plan reviews | `/home/greencheetah/Projects/mic/Docs/deeptime-plan/*.md` (every file there) | `docs/plan-reviews/2026-09-28-<same file name>` |

The rock-tag datapack stays where it is
(`/home/greencheetah/Projects/mic/Docs/datapacks/deeptime-mic-rock-tags/`): it belongs to the MIC
pack, not to Deep Time, and the pack will ship it as `pack/kubejs/data/deeptime/` (decisions file,
log 2026-09-28). This plan names research notes by file name (`wrapped_planet_worlds.md §6.3`), which
resolves in either location. The decisions file's own header names the report, the notes and the
datapack by paths relative to `mic/Docs`; read them through this table. Agents leave that header as
it is (the edit rule below allows no other change).

**Authority.** When two documents disagree, the higher one wins:

1. **The decisions file** is the single authority for Ben's decisions. Its tables hold the decisions
   of the pre-implementation Q&A; its last section, **"Decided during implementation (append-only
   log)"**, holds every answer Ben gives after that. A newer log entry overrides an older one, and
   overrides any table row it contradicts. Agents change the file in exactly two ways, and only when
   Ben has answered a question: they **append a log entry** (the date, the question, and his answer
   quoted verbatim), and they **update any table row above that the answer changes**, adding
   "(see log YYYY-MM-DD)". They never rewrite or delete an earlier log entry and never edit anything
   else in the file (§12.3).
2. **This plan.** It follows the decisions file; when an answer changes the plan, the plan is edited
   in the same commit series (§12.3), and §14 points to the log entry.
3. **The research report.**
4. **The research notes.**

`STATUS.md` is not on this list: it records progress and the questions still **pending**, never a
decision.

The research report predates the decisions. These report positions were overturned and must not be
followed: a pure-Java engine (now Rust over JNI), 8 m per block (now about 300 m horizontal and
20 m vertical), the MIT licence (now LGPL-3.0-or-later), Terralith pass-through overlays for floating
islands (now simulated sky islands), bit-for-bit seed determinism (now "stored data only"), a bordered
square map (now a wrapped planet), the `geohist` namespace (now `deeptime`), a window cut out of a
larger planet (now the whole planet is the world), a History Atlas item (the timelapse lives only on
the Create World screen), TerraBlender's own region layout by default (regions follow geography from
v1), a CLI-first server flow (a server now generates seamlessly on first start; the CLI remains an
option), a −64..639 build range (now −352..639), and the report's phase numbering (the decisions
file's Phases 0–3 apply). `wrapped_planet_worlds.md` sizes the world in round chunk counts (1,000 and
2,000 chunks); the decisions fix C at exactly 16,384 and 32,768 blocks (1,024 and 2,048 chunks).

### 1.3 Reading order

This is the only reading list; §11.1 and the repository's `CLAUDE.md` point here. Paths are given as
they are until P0-01; afterwards use the §1.2 table.

**Always, before any task:**

1. The decisions file, all of it, **including its log** at the end (the newest entries win).
2. This plan: §1, §2, then the phase you are working on (§10), your task's work item, and the
   sections it cites.
3. **Your task's questions:** its **Ask first** field and every row of §14 whose "Needed by" names
   your task. Ask each one (§12.3) before building the part it decides.
4. `STATUS.md` in the repository (once it exists): the current task, the build log, pending questions
   and known issues.
5. House style: `/home/greencheetah/Projects/mic/mods/mic-climate/` — `build.gradle`,
   `gradle.properties`, `gametest.sh`, `gametest-client.sh`, `smoke/install.sh`, `smoke/run.py`,
   `tools/make_test_structure.py`, `src/main/resources/mic_climate.mixins.json`,
   `src/main/templates/META-INF/neoforge.mods.toml`. Read only; other sessions use that checkout. Its
   `smoke/server/mods/` folder is a stale copy of the pack (§2.2) and is never a reference.

**When a task cites them:**

6. `wrapped_planet_worlds.md` — Toroidal World, Sable teleport, poles, the spike checklist (its §6.3).
7. `aeronautics_rust_bridge.md` — how Sable ships Rust over JNI, and the six pitfalls (its §8).
8. `minecraft_integration_architecture.md` — the verified 1.21.1 generator facts (noise router,
   pipeline, presets, spawn, server start).
9. `mod_compatibility_biomes_ores_fantasy.md` — TerraBlender gates, Terralith/Lithostitched internals,
   ore tags, the ReTerraForged redirect.
10. The research report (with §1.2's overrides in mind).
11. `tectonics_engines.md` (plates, Noddy), `surface_processes_climate.md` (erosion, climate,
    hydrology), `ore_genesis_modeling.md` (deposits, sizing, schema), `runtime_packaging_options.md`
    (natives, determinism, servers), `player_demand_prior_art.md` (market context only).
12. `/home/greencheetah/Projects/mic/Docs/datapacks/deeptime-mic-rock-tags/README.md` — the 59 rock
    classes, the tag vocabulary and the `rock_states` format.

Items 6–9 and 11 are research notes, in `/home/greencheetah/Projects/mic/Docs/research_notes/Geologic
history worldgen mod/` until P0-01 and in `docs/research/notes/` after it.

### 1.4 Glossary

| Term | Meaning in this plan |
|---|---|
| **Core** | The Rust crate `deeptime-core`: the simulation, with no Minecraft and no JNI types (§4.2). |
| **Adapter** | The Java NeoForge mod in `neoforge/`: screens, generator, hooks, compat (§7). |
| **World data** | The folder `<world>/deeptime/` holding everything the simulation produced (§5.7). It is authoritative: a world is defined by its world data, not by a seed. |
| **Format version** | `format_version` in `deeptime/manifest.json`. Before 1.0 a mismatch means "refuse to extend this world"; from 1.0 migrations are guaranteed. |
| **Preset** | A complete set of simulation parameters (planet, timeline, scales, multipliers). Shareable as a JSON file or a share code (§5.1). |
| **Staging** | `<gamedir>/deeptime/staging/<session-uuid>/`, where the Create World screen writes world data before the world folder exists (§7.12). |
| **Event log** | The shared, append-only record of what happened, where, when and why (§5.5). Feeds the timelapse, "Why here?" and a later legends view; the civ module appends to it. |
| **Site / site registry** | A site is one fixed-by-generation resource location (deposit, reservoir, placer) with extent, grade, type and forming event. The registry is the list plus a spatial index and a Java API (§5.6). |
| **Rock class** | One of the 59 lithologies in the rock-tag vocabulary, e.g. `granodiorite`, `black_shale`. The simulation decides classes; tags decide blocks (§5.4). |
| **Rock group** | A union tag of classes, e.g. `deeptime:rock/carbonate`. |
| **Stand-in / fallback** | A look-alike block used when a class has no member block: the first *installed* entry of `deeptime:rock_fallback/<class>` (never a random one). |
| **Unclassified crust** | `minecraft:stone` and `minecraft:deepslate`, used where the simulation has no lithology opinion. Not a class. |
| **`rock_states` entry** | A JSON file `data/<ns>/deeptime/rock_states/<name>.json` that puts a specific block *state* into a class (member or fallback). |
| **Deposit type** | A data-driven definition (datapack JSON) of a mineral system: genesis rules, sizing, geometry, blocks (§5.4.3). |
| **Claimed material** | A material Deep Time places itself (copper, iron, coal, zinc, lead, gold, …). Its scatter features from all mods are removed in Deep Time worlds. |
| **Background scatter** | Deep Time's own low-density ore feature that guarantees small finds everywhere, gated by host rock. |
| **Essentials list** | A pack-defined list of materials with regional minimums and spawn requirements (§7.8). |
| **TW** | Toroidal World, the MIT wrap mod Deep Time depends on. |
| **TW cylinder** | A TW world shape where X loops and Z does not. Deep Time registers its own shape through TW's API. |
| **Seam** | The longitude where x = −C/2 meets x = C/2 − 1 (C = circumference in blocks). With TW it is invisible; without TW it would be a teleport line. |
| **Canonical span** | The one copy of the world TW stores: x ∈ [−C/2, C/2). |
| **Soft cap** | The polar barrier at about ±75–80° latitude: pushback, storms and ship turbulence rather than a wall. |
| **Pole transit** | The later feature that teleports across a pole to 180° of longitude away with heading reversed. |
| **Block space** | The Minecraft grid. The planet is mapped to it equirectangularly: x ↔ longitude, z ↔ latitude (§5.2). |
| **L0 / L1 / L2 / L3** | The grid ladder: L0 the Fibonacci sphere for plates; L1 the history raster; L2 the young-landscape raster; L3 the final raster (§5.2). |
| **VE** | Vertical exaggeration: horizontal metres per block ÷ vertical metres per block = 300 / 20 = 15. |
| **Keyframe** | A downsampled snapshot of selected fields at one simulated time, used by the timelapse and the viewer. |
| **Viewer bundle** | A static HTML folder produced by the CLI for any stage: images, JSON and an `index.html` (§9.3). |
| **Review page** | A viewer bundle, render or report published as a private spacesheep page for Ben (§2.6). |
| **Smoke test** | A real dedicated server booted headlessly and driven over RCON, with a written report (§9.5). |
| **GameTest** | NeoForge's in-game test framework. Deep Time runs it in two headless runtimes, each with its own test namespace: the GameTest server (`./gametest.sh`, namespace `deeptime`) and a dev planet server (`./gametest-planet.sh`, namespace `deeptime_planet`) (§9.4). |
| **Decisions log** | The append-only section "Decided during implementation" at the end of the decisions file, where every answer Ben gives is recorded (§1.2, §12.3). |
| **`test_planet`** | A datapack-defined dimension that exists only in GameTest runs, so terrain can be tested beside the GameTest server's flat overworld (§9.4). |
| **Terrain source** | What `deeptime:terrain` samples: an analytic periodic test terrain (Phase 0, before world data exists) or the world data's rasters (§7.1). |
| **Server set** | A folder of server-side mod jars that Deep Time's own harness installs: `standalone`, `mic` (the current MIC pack) or `terrablender` (§9.5). |
| **Oracle test** | An unshipped comparison against a GPL tool (fastscapelib, goSPL), run outside the build (§9.2). |
| **Natives** | The compiled `deeptime_jni` shared libraries for each OS/architecture, bundled in the jar. |
| **COE / CDG** | Create Ore Excavation / Create: Diesel Generators, MIC's drill and oil mods. |
| **EV** | Excavated Variants, the optional ore-in-any-stone block generator. |
| **TB / BOP** | TerraBlender / Biomes O' Plenty. |
| **RTF** | ReTerraForged (Campione01/ReterraForged), the MIT 1.21.1 prior art for the generator bridge. |
| **Sub-level** | Sable's name for a physics body built from blocks (an Aeronautics ship). |

---

## 2. Environment and conventions

### 2.1 Machine and tooling

- The machine is **NixOS**. Use `nix shell nixpkgs#<pkg> -c <cmd>` for one-off tools (jdk25 for
  BlueMap, nodejs_22 for spacesheep, cfr for decompiling) and the repository flake's dev shell for
  everything the build needs: **run every build, test and `cargo` command as `nix develop -c …`**.
  The user profile does carry a few general tools (`git`, `gh`, `jq`, `python3`, `gcc`, `readelf`),
  which are fine to use directly, but its `cargo` is a rustup 1.29.0 shim with no toolchain installed
  (`readlink -f ~/.nix-profile/bin/cargo` → `…-rustup-1.29.0/bin/rustup`, and `~/.rustup/toolchains`
  does not exist): a bare `cargo` would download toolchains into `~/.rustup`, outside the flake.
  Versions checked on 2026-09-28 with single-package evals: `jdk21` 21.0.12.1+1, `jdk25`
  25.0.4.1+1, `nodejs_22` 22.23.2.
- **Flakes see only files git tracks.** Inside a git work tree, a new `flake.nix`,
  `rust-toolchain.toml` or any file the flake reads does not exist for Nix until it is staged. Run
  `git add` on new files before `nix flake lock` or `nix develop`.
- **Evaluation runs locally.** Never `nix eval` a whole configuration set
  (`.#nixosConfigurations`, `.#homeConfigurations`); one such eval reached 13 GB and froze the
  machine on 2026-09-23. Evaluate single attributes (`nix eval --raw nixpkgs#<pkg>.version`).
- **packwiz** runs as `nix develop --command packwiz …` from inside
  `/home/greencheetah/Projects/mic/pack/`. It is not on PATH.
- This laptop is an **i7-1185G7, 4 cores / 8 threads, 15 GB RAM** (about 6 GB free while other
  work runs). It is below the 6–8-core PC the time budgets target, so a run that meets a budget here
  meets it on the target machine; a run that misses it here is reported with its stage breakdown,
  not declared a failure (§9.1). **Never run the simulation and a server at the same time** on this
  machine: run the CLI first, then the server.

### 2.2 Reference files

| What | Path |
|---|---|
| NeoForge 21.1.249 + Minecraft 1.21.1 sources (Mojang names; Parchment parameter names where available — many methods keep SRG names such as `p_321472_`) | **From P0-03 on, Deep Time's own:** `neoforge/build/moddev/artifacts/neoforge-21.1.249-sources.jar`, produced by `cd neoforge && nix develop -c ./gradlew createMinecraftArtifacts` (ModDevGradle 2.0.74's task; any build also runs it), and extracted for grepping into the gitignored `libs/neoforge-src/` by `tools/dev/sources.sh` (P0-03). **Before P0-03 only:** read mic-climate's copy, `/home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar`, which a `clean` over there can delete at any time. |
| Vanilla 1.21.1 data and assets | The same folder: `neoforge-21.1.249-client-extra-aka-minecraft-resources.jar` |
| The MIC pack definition (packwiz) | `/home/greencheetah/Projects/mic/pack/` (`pack.toml` pins NeoForge 21.1.249; its version is 2.1.2 at `HEAD` `303fd89` and 2.1.3 in the working tree, which another session is editing) |
| The MIC pack's server-side jars | **Whatever `pack/` pins today.** From P0-04 on: Deep Time's own `smoke/mic/server/mods/` (gitignored), installed from `pack/pack.toml` by `tools/smoke/install.sh mic`. Before the repo exists: download a jar from the `[download] url` in its `pack/mods/<name>.pw.toml` into the session scratchpad and check the hash. |
| Rock-tag datapack (MIC's, not Deep Time's; Q16) | `/home/greencheetah/Projects/mic/Docs/datapacks/deeptime-mic-rock-tags/` |

- `mods/mic-climate/smoke/server/mods/` is **not** a reference. It was installed on 2026-09-05; the
  pack moved on 2026-09-17, and about 40 jars differ (the cold-read review compared the pack's
  `.pw.toml` file names with that folder). It also belongs to another project and can be reinstalled
  by another session at any moment.
- **Re-install before any `-PwithMic` gate** (`tools/smoke/install.sh mic` is idempotent and syncs to
  the pack's current pins). `install.sh` reads the pack's **working tree**, which other sessions edit,
  so it records `git -C /home/greencheetah/Projects/mic/pack describe --always --dirty` (for example
  `303fd89-dirty`) and, when that ends in `-dirty`, also
  `git -C /home/greencheetah/Projects/mic/pack status --porcelain | sha256sum`; both go into the smoke
  report and the gate's STATUS entry.
- Extract sources into a scratch directory before grepping (`unzip -o -q <jar> '<path>'`); never
  unpack into a repo. The mic-climate paths belong to another project: read them, never write them.

### 2.3 Constraints that shape every task

- **Ben cannot launch the game.** Every check in this plan is something an agent can run (a test, a
  GameTest, a headless server, a renderer) and every result has a form Ben can review in a browser
  (a viewer bundle, a BlueMap map, a report page). Anything that can only be judged in a live client
  is listed as unverifiable (§9.8) rather than silently assumed.
- **Licence: LGPL-3.0-or-later** (SPDX `LGPL-3.0-or-later`: every `Cargo.toml` `license` field,
  `neoforge.mods.toml` `license`, source-file header `SPDX-License-Identifier` and the README use it).
  No GPL code enters the repository, and GPL code is never ported line by line.
  GPL tools (fastscapelib, goSPL, pyGPlates, …) may appear only as oracles in unshipped tests under
  `tools/validation/`, run as separate processes. Porting from MIT, BSD, Apache-2.0 and CC BY 4.0
  sources is allowed with attribution in `NOTICE.md`. Research-only licences (the Inria
  "landscapes" codes) and unlicensed repositories (the community Cortial ports) are read-only:
  implement from the paper. `cargo deny check licenses` enforces the Rust side (§4.5).
- **Numbers.** Every number in this plan is cited, derived with its arithmetic shown, or a design
  choice (tile sizes, test sample counts, default intervals, byte budgets) that the implementing task
  may change and record. Measured values replace estimates as they arrive (§12.3).

### 2.4 Workflow rules

- **Other sessions share these checkouts.** Before anything destructive — `git reset`,
  `git checkout --`, deleting a branch or worktree, killing a process — check whether another session
  is mid-flight (`git status`, `git worktree list`, `ps`), and say what you found.
- **Never `git stash`.** The stash stack is repo-global and concurrent pops swap stashes between
  sessions.
- **Landing code is not a question.** Committing, merging a finished branch into `master` with
  `git merge --no-ff`, and pushing `master` are done without asking. A behind or dirty main checkout
  is caught up with `git merge --ff-only --autostash origin/master`, then committed and pushed. Stop
  only for a conflict between competing substantive edits, a history rewrite, or Ben saying stop.
- **Ask about anything that will outlive the session** (new design decisions, dependency choices,
  anything user-visible) through `AskUserQuestion` with a recommended option, unless Ben has said to
  work autonomously. Record the answer in the decisions file's log (§1.2, §12.3).
- **The repository stays private until the MVP** (the end of Phase 1): `MinecraftIndustrialComplex/deep-time`.
- **`AskUserQuestion` takes at most four questions per call**; batch larger sets into several calls.

### 2.5 Sub-agent model policy

Pass `model` explicitly on every Agent call:

- **Sonnet** for lookups: jar greps, `javap`, API and version checks, licence checks, summarising a
  package.
- **Opus** for real work: writing code, involved research, planning a chunk of work.
- **No Fable.** The decisions file ("Planner / implementer") has Opus plan and implement Deep Time,
  so Deep Time uses no Fable, for planning or for code.
- Long grep or `javap` investigations go to a Sonnet agent rather than into the main context.

### 2.6 Publishing review pages (spacesheep)

Every reviewable artifact is published as a **private** spacesheep page, through
`tools/review/publish.sh <name> <folder>` (from P0-06 on), which runs:

```sh
cd <folder containing index.html>
nix shell nixpkgs#nodejs_22 -c npx -y spacesheep@1.17.0 deploy . \
  [--space <uuid, when tools/review/spaces.json already has this page>] \
  --title "Deep Time — <what>" --slug deeptime-<what> \
  --description "<one sentence>" -m "<version label, e.g. P0 exit 2026-10-03>" --json
```

- **Pinned to 1.17.0**, the npm `latest` on 2026-09-28 (registry `dist-tags`). Its README documents
  `deploy [dir|file]` with `--space`, `--new`, `--title`, `--slug`, `--emoji`, `--description`,
  `--visibility`, `--org`, `-m` and `--json`; 1.13.0 (whose `--help` this plan first used) had the same
  options without `--new`. Re-check the README before bumping the pin.
- **Space ids are recorded in the repo.** A first deploy creates a space and writes `.spacesheep.json`
  (`{"space": "<uuid>", "url": "https://spacesheep.dev/@benmac/<slug>"}`) into the deployed folder,
  and later deploys from that folder publish a **new version of the same space**. Review folders live
  under the gitignored `review/out/`, so `publish.sh` also copies each page's space id into
  `tools/review/spaces.json` (committed) and passes `--space` on every later deploy. That keeps Ben's
  links stable on a fresh clone, and it avoids 1.17.0's refusal to update a space whose page `<title>`
  changed unless `--space` or `--new` is given.
- `--visibility` applies to new spaces only and **defaults to private**. Never run `share`, never
  change visibility, never pass `--org` or `--new`.
- Never run `spacesheep login`, `connect`, `logout`, `update`, `memory …`, `sessions …` or `talk …`.
  If a deploy fails on authentication, stop and report the exact error; Ben signs in himself.
- Verify with `spacesheep list` (the space exists and is private) and `spacesheep read <space>`.
- spacesheep serves a modified copy inside an iframe under a header bar. A page that needs direct
  loading (BlueMap's web app) may have an app-origin form, `https://<space-id>.spacesheep.app/v/<sha>/`
  (the version sha from `spacesheep versions <space>`). That form is **unverified**: spacesheep
  1.17.0's CLI and README name only `https://spacesheep.dev`, although `*.spacesheep.app` does resolve.
  P0-14 checks it and records the result here; until then link the normal page.

### 2.7 Nothing secret or licence-restricted is committed

- mic-climate gitignores `smoke/`, `libs/`, `run/` and `run-gametest/`; Deep Time does the same and
  adds `run-planet/`, `review/out/`, `.spacesheep.json`, `.direnv/` and `.graphide/` (the graphide
  tooling writes databases and history snapshots into checkouts here; mic-climate tracks none of its
  `.graphide/` files). RCON passwords (`smoke/rcon.pw`, `run-planet/rcon.pw`), every server runtime and
  every downloaded pack jar live under those folders.
- Third-party jars used only locally go in `libs/`. Project Atmosphere's licence forbids
  redistribution; its jar never enters the repo or a published page.
- BlueMap and smoke-server outputs that contain whole pack jars stay under gitignored folders; only
  rendered tiles and reports are published.
- **Licence acceptances.** Ben accepted three on 2026-09-28 (decisions file, "Licence acceptances"):
  the Minecraft EULA for headless test servers (every server this plan runs: the GameTest server, the
  dev planet server and the smoke servers, so scripts may write `eula=true`), BlueMap's
  `accept-download: true`, and Microsoft's CRT/SDK terms that using cargo-xwin implies. Any other
  licence acceptance is Ben's to make: ask first.

---

## 3. Architecture overview

### 3.1 Processes, threads and data flow

```text
 SINGLEPLAYER CLIENT PROCESS                          DEDICATED SERVER PROCESS (first start)
 ┌────────────────────────────────────────────┐      ┌─────────────────────────────────────────┐
 │ Render thread                              │      │ Server thread, inside ServerAboutToStart │
 │  CreateWorldScreen                         │      │  FirstStartGenerator:                    │
 │   └ PlanetEditorScreen (PresetEditor)      │      │   world/deeptime/manifest.json present?  │
 │       setup page → Run → timelapse page    │      │   yes → nothing to do                    │
 │       polls progress(handle) ≤ 20×/s       │      │   no  → read config/deeptime/server.json │
 │       copyKeyframe → NativeImage →         │      │         run a SimSession to completion   │
 │       DynamicTexture.upload()              │      │         (blocking; console progress      │
 │                ▲                           │      │         every 5 s; RCON not up yet)      │
 │ "deeptime-sim" Java worker thread          │      └──────────────────┬──────────────────────┘
 │  SimSession: loop advance(handle, 250 ms)  │                         │
 └────────────────┬───────────────────────────┘                         │
                  │ JNI: synchronous calls, one caller per session       │
 ┌────────────────▼───────────────────────────────────────────────────────▼─────────────────┐
 │ deeptime_jni (cdylib)  →  deeptime-core                                                  │
 │   handle table: u64 id → Arc<Session>   (never a raw pointer across JNI)                 │
 │   Session: stage machine · cancel flag (AtomicBool) · progress atomics · keyframe ring   │
 │   private rayon::ThreadPool ("deeptime-rayon-N"), sized per caller                       │
 └────────────────┬─────────────────────────────────────────────────────────────────────────┘
                  │ finalizeWrite(): tile packs, strata, bodies, sites, events, keyframes
 ┌────────────────▼───────────────────────────┐
 │ <gamedir>/deeptime/staging/<uuid>/ (SP)    │── moved at ServerAboutToStart, before
 │ or <world>/deeptime/ directly (server)     │   loadLevel() ──▶ <world>/deeptime/
 └────────────────┬───────────────────────────┘
                  │ pure-Java reader: FileChannel.map + LZ4 block decode + tile cache (no JNI)
 ┌────────────────▼─────────────────────────────────────────────────────────────────────────┐
 │ Worldgen threads (Util.backgroundExecutor, C2ME workers), any number, concurrently       │
 │   GeologicChunkGenerator (NoiseBasedChunkGenerator subclass)                             │
 │   density functions raster / terrain / depth / beyond_pole → noise router               │
 │   buildSurface: super (pack surface rules) → StrataPass → WaterPass                      │
 │   features at UNDERGROUND_ORES: DepositFeature, BackgroundScatterFeature                 │
 │ Server thread: TW shape, spawn/border, pole cap, COE/CDG hooks, site API, commands       │
 └──────────────────────────────────────────────────────────────────────────────────────────┘

 STANDALONE (no Minecraft):  deeptime-cli ──▶ deeptime-core ──▶ world data + viewer bundle (HTML)
```

Three rules keep this design robust:

1. **JNI is used only while a world is being created.** Chunk generation reads the finished world
   data with plain Java. An existing world therefore keeps generating chunks on a machine whose
   natives do not load, and a server that received a world folder from the CLI never needs natives.
2. **One Java thread drives one session.** Rayon parallelism is internal to each `advance` call,
   exactly as Sable runs Rapier (`aeronautics_rust_bridge.md` §3). Nothing on the Java side is
   concurrent with the native step except `progress` and `cancel`, which only touch atomics.
3. **The world data is the contract.** It is versioned (§5.7), written by Rust, read by Java, and
   verified by a parity test that compares Java's column evaluation with Rust's (§9.2).

### 3.2 Who owns what

| Concern | Owner | Notes |
|---|---|---|
| All simulation stages, keyframes, event log, site registry creation | Rust core | §8 |
| Presets: validation, memory and time estimate, share codes | Rust core | Java calls `estimate` before `create` |
| World-data writer | Rust `deeptime-format` | Also has a reader, for the CLI, viewer and tests |
| World-data reader for chunk generation | Java `worlddata` package | Parity-tested against Rust |
| Baking chunk-time structures (horizon stacks, bodies, water levels) | Rust core | So Java only samples; no geology logic in Java |
| Climate → vanilla parameter mapping | Java | Needs the pack's registries; computed once per world and stored (§7.2) |
| Block choice for rock classes and ores | Java | Tags, `rock_states`, Excavated Variants (§7.5, §7.6) |
| Screens, timelapse drawing, editor | Java (client) | Draws keyframes supplied by Rust |
| Generator, density functions, surface/strata/water passes, features | Java | §7.1 |
| World preset, dimension type, noise settings, tags, biome modifiers | Java resources | `neoforge/src/main/resources/data/` |
| Toroidal World shape, pole cap, spawn, networking, COE/CDG mixins | Java | §7.10–§7.16 |
| Data-driven content (rock classes, deposit types, fantasy fields, essentials) | Datapacks, read by Java, passed to Rust as JSON | §5.4 |

### 3.3 How the CLI and the HTML viewer reuse the core

- `deeptime-cli` links `deeptime-core` directly (no JNI) and drives the same `Session` state machine
  the JNI wrapper drives. `deeptime run --preset <file> --data <dir|jar>… --out <world>/deeptime`
  produces byte-for-byte the same file layout the in-game path produces, so a server owner can run
  it on any machine and copy the folder.
- Every stage also runs alone: `deeptime stage <plates|erosion|climate|hydrology|structure|deposits|fantasy>
  --in <checkpoint> --out <checkpoint> --viewer <dir>`. Stages read and write checkpoints (the
  in-memory stage state serialised with `postcard`), so one stage can be iterated without rerunning
  the ones before it.
- `--viewer <dir>` calls `deeptime-viewer`, which writes a static bundle: layer images as PNG,
  keyframe strips, the event log as JSON, and a fixed `index.html` + `viewer.js` (vanilla JS on a
  canvas: layer toggles, a time scrubber, an event ticker, a click-to-inspect readout and a globe or
  map projection toggle). The bundle is exactly what gets published to spacesheep.
- The in-game timelapse and the viewer draw the same keyframes with the same colour ramps. The ramps
  live in one JSON file (`crates/deeptime-viewer/assets/ramps.json`) that the Gradle build copies
  into the mod jar, so what Ben reviews in the browser is what a player sees on the Create World
  screen, apart from widget layout (§9.8).
- `deeptime bench --preset <p>` runs a preset with per-stage timings and peak memory, writing
  `bench.json` and a viewer page with a timing chart. It is the Phase 1 budget gate (§10.3).

---

## 4. Repository layout

### 4.1 Tree

The private repository `MinecraftIndustrialComplex/deep-time`, checked out at
`/home/greencheetah/Projects/mic/mods/deep-time`.

```text
deep-time/
├── CLAUDE.md                  agent instructions (draft in §11.2); AGENTS.md is a one-line pointer to it
├── AGENTS.md
├── STATUS.md                  task status table and build log (§12)
├── README.md                  what it is, how to build, how to run the CLI
├── COPYING  COPYING.LESSER    GPL-3.0 and LGPL-3.0 texts (LGPL-3.0-or-later needs both)
├── NOTICE.md                  third-party attributions (tectonics.js CC BY 4.0, Noddy MIT, …) §13
├── docs/                      the decisions file, this plan, the research report and notes, plan reviews (§1.2)
├── flake.nix  flake.lock      dev shell and apps (§4.4)
├── rust-toolchain.toml        pinned stable channel and components; no cross targets (§4.5)
├── Cargo.toml  Cargo.lock     workspace
├── deny.toml                  cargo-deny: licence allow-list, bans, advisories
├── .cargo/config.toml         per-target rustflags (musl cdylib, macOS codegen-units)
├── crates/
│   ├── deeptime-format/       on-disk schema and codecs; no simulation
│   ├── deeptime-core/         the simulation; depends on format
│   ├── deeptime-jni/          cdylib "deeptime_jni"; the only crate that depends on `jni`
│   ├── deeptime-cli/          binary `deeptime`
│   ├── deeptime-viewer/       viewer-bundle writer + web assets (index.html, viewer.js, ramps.json)
│   └── deeptime-inspect/      binary `deeptime-inspect`: renders Minecraft region files (dev tool)
├── neoforge/                  Gradle NeoForge module (§4.3)
│   ├── build.gradle  gradle.properties  settings.gradle  gradlew  gradlew.bat
│   ├── gradle/wrapper/        gradle-wrapper.jar, gradle-wrapper.properties (Gradle 8.11.1)
│   ├── src/main/java/com/minecraftindustrialcomplex/deeptime/…   (§7)
│   ├── src/main/resources/                                       mixins, lang, data/ (incl. structure/empty_5x5x5.nbt)
│   ├── src/main/templates/META-INF/neoforge.mods.toml
│   ├── src/test/java/…                                           JUnit (reader, LZ4, loader, share codes)
│   └── src/gametest/resources/deeptime_gametest/                 gametest-only pack: pack.mcmeta, data/deeptime/… (§7.12, §9.4)
├── tools/
│   ├── smoke/                 install.sh, run.py, profiles/{standalone,mic,terrablender}.txt, *.excluded
│   ├── bluemap/               config templates (core.conf, maps/*.conf, storages/file.conf)
│   ├── review/                publish.sh, spaces.json (page → space id), one exit script per phase
│   ├── natives/               build-natives.sh, NativeSmoke.java
│   ├── ci/                    junit-summary.py
│   ├── make_test_structure.py writes the 5×5×5 empty GameTest template
│   └── validation/            UNSHIPPED oracle scripts (Python, GPL tools via nix shell)
├── .github/workflows/
│   ├── ci.yml                 Rust job and Gradle job (§4.5)
│   └── natives.yml            8 cross-builds + load test on real OS runners
├── gametest.sh                GameTest server run (§9.4), after mic-climate's script
├── gametest-planet.sh         dev planet server with a real Deep Time + TW overworld (§9.4)
├── smoke/  run-gametest/  run-planet/  libs/  review/out/  target/   (gitignored runtimes and outputs)
└── .gitignore
```

### 4.2 Why these crate boundaries

| Crate | Depends on | Boundary rule | Why separate |
|---|---|---|---|
| `deeptime-format` | serde, serde_json, lz4_flex, memmap2, crc32fast | No simulation, no MC, no JNI | It is the contract with Java and with future modules (civ, other planets). Small, stable, versioned on its own; Java's reader is written against it. |
| `deeptime-core` | format, rayon, glam, rand + rand_pcg, kiddo, postcard | No MC, no JNI, no file layout beyond calling format | Reusable by the civ module and by future planets as a plain library. Testable with `cargo test` alone. |
| `deeptime-jni` | core, jni | Only marshalling, the handle table and panic guards | Keeps `jni` types out of the core; the whole unsafe surface is one small crate. |
| `deeptime-cli` | core, viewer, clap | No MC | The server tool and the per-stage development driver. |
| `deeptime-viewer` | format, png | No simulation | Renders any world data or checkpoint to HTML; used by the CLI, by tests and by the review scripts. |
| `deeptime-inspect` | fastnbt (or a small in-crate NBT reader), flate2, lz4_flex, png | May know Minecraft's region format; never linked into the mod | Reads region files that the game wrote and renders top-down, rock, ore and cross-section images for review (§9.7). |

The future civ module becomes `deeptime-civ`, depending on `deeptime-format` (to append event-log
segments and read sites) and on `deeptime-core` only if it needs the planet model.

### 4.3 The Gradle NeoForge module

- **Pins.** ModDevGradle `net.neoforged.moddev` 2.0.74, NeoForge 21.1.249, Parchment 2024.11.17,
  Java 21, Gradle 8.11.1 (wrapper) — the same as mic-climate. `neoforge/gradle.properties` copies
  mic-climate's comment style and explains each pin.
- **Mod properties.** `mod_id = deeptime`, `mod_name = Deep Time`, `mod_version = 0.0.1` (must equal
  the Cargo workspace version; `checkVersions`, §6.5), `mod_group_id = com.minecraftindustrialcomplex`,
  `mod_authors = Benjamin McIntyre`, `mod_license = LGPL-3.0-or-later`, `mod_description`. The jar is
  `deeptime-1.21.1-<version>.jar` (`${mod_id}-${minecraft_version}-${version}`, as in mic-climate).
- **Modrinth version ids** (checked with the Modrinth API on 2026-09-28; all NeoForge 1.21.1). Every
  required dependency of a pinned version is itself pinned here, so nothing floats to "latest":

  | Mod | Version | Modrinth version id | Used by |
  |---|---|---|---|
  | Toroidal World | 0.19.3+mc1.21.1 (one fabric+neoforge jar) | `8YTSYi9y` | `compileOnly` + `localRuntime` (the **only** mod in `localRuntime`), every run and every server set |
  | Sable | 2.0.5+mc1.21.1 | `U678xqle` (the pack's pin, `pack/mods/sable.pw.toml`) | `compileOnly` (API calls in `compat/sable`); `standalone` set |
  | Create Aeronautics | 1.3.2+mc1.21.1 | `44pLdPGg` (the pack's pin; requires Sable and Create) | `standalone` set |
  | Create | 6.0.10+mc1.21.1 | `UjX6dr61` (the pack's pin, `pack/mods/create.pw.toml`) | `standalone` set (Aeronautics requires it; `shaftLineAcrossSeam` uses its kinetics) |
  | Chunky | 1.4.23 | `LuFhm4eU` | `standalone` set |
  | TerraBlender | 4.1.0.8 | `6e8GCrLb` | `compileOnly` (`compat/terrablender`); `terrablender` set |
  | Biomes O' Plenty | 21.1.0.14 | `BtZKRp69` (requires GlitchCore and TerraBlender) | `terrablender` set |
  | GlitchCore | 2.1.0.2 | `S2TfWrZR` (the pin mic-climate uses) | `terrablender` set |
  | Excavated Variants | 4.3.1 | `krUiU6UD` (requires Dynamic Asset Generator) | `compileOnly` (`compat/ev`); `ev` set |
  | Dynamic Asset Generator | 6.1.2 | `U0ft2n2p` (its only NeoForge 1.21.1 version) | `ev` set |

  TW shipped 0.17.2, 0.18.1 and 0.19.3 for NeoForge 1.21.1 between 2026-09-18 and 2026-09-27 (Modrinth
  version list), so the id, not a version range, is the pin. Sable, Aeronautics and Create follow the
  pack: when `pack/mods/{sable,create-aeronautics,create}.pw.toml` change, the ids here and in
  `tools/smoke/profiles/standalone.txt` change with them, and `install.sh` warns when they differ.
  COE and CDG, which only the `mic` set carries, are compiled against with `compileOnly` from the
  Modrinth maven at the pack's pins (P1-16).
- **Natives in the jar.** `processResources` copies `../crates/deeptime-viewer/assets/ramps.json` into
  `assets/deeptime/ramps.json` and the natives into `natives/deeptime/`:
  - **Dev builds** (the default): task `hostNatives` runs plain `cargo build --release -p deeptime-jni`
    (the dev shell's cargo locally, rustup's in CI; §4.5), stages the host library in
    `../target/natives-host/<platform>/`, and writes `../target/natives-host/natives.json` (the §6.5
    schema). Only this folder is packed, so a leftover `natives-dist/` never leaks into a dev jar.
  - **Release builds** (`-Prelease`): `natives-dist/` from `nix run .#natives` or from the
    `natives.yml` artifacts, all eight libraries, and its `natives.json`.
  - The format version has one source, the workspace `Cargo.toml`
    (`[workspace.metadata.deeptime] format_version`, §6.5): `deeptime-format`'s `build.rs` turns it
    into `FORMAT_VERSION`, and Gradle reads the same line with the same line scan rather than keeping
    a copy (P0-07; cold-read review 2, gap 14).
- **JUnit.** Plain JUnit, without `neoForge.unitTest`, because these tests touch no Minecraft class:
  `testImplementation platform('org.junit:junit-bom:5.14.4')` (the last 5.x release; 6.x, which
  needs Java 17 or later, would also run here, so moving to it later is a free choice),
  `testImplementation 'org.junit.jupiter:junit-jupiter'`,
  `testRuntimeOnly 'org.junit.platform:junit-platform-launcher'`,
  `testImplementation 'org.lz4:lz4-java:1.8.0'` (the game's own version, stated explicitly so the
  test classpath does not depend on how ModDevGradle exposes Minecraft's libraries);
  `test { useJUnitPlatform(); forkEvery = 1; testLogging { events 'passed', 'failed' } }`. Gradle's
  `forkEvery` counts test **classes** per forked JVM and applies to the whole `test` task, so with 1
  every class, including `NativeLinkTest`, links the native library in a fresh JVM; the suite is
  small enough for that. Before running, `test` depends on `hostNatives` and on `testWorlds`, which runs
  plain `cargo run -p deeptime-cli -- synth …` for the fixtures (P0-09). If a later test needs the game
  loaded, ModDevGradle's `neoForge.unitTest { enable(); testedMod = mods.deeptime }` is the documented
  route.
- **Runs.** Every game directory sits at the repository root, next to the scripts that clean it, so
  the paths are written with `../`:
  - `client` and `server`: `gameDirectory = file('../run')`.
  - `gameTestServer`: `gameDirectory = file('../run-gametest')`, `-Xmx6G`,
    `systemProperty 'neoforge.enabledGameTestNamespaces', 'deeptime'`,
    `systemProperty 'deeptime.gametest', 'true'`.
  - `planetServer`: a dev `server()` run, `gameDirectory = file('../run-planet')`, `-Xmx6G`,
    `programArgument '--nogui'`, `systemProperty 'neoforge.enableGameTest', 'true'`,
    `systemProperty 'neoforge.enabledGameTestNamespaces', 'deeptime_planet'` (§9.4).
- **The `gametest` source set** holds only resources: the GameTest templates at its root
  (`data/deeptime/structure/empty_5x5x5.nbt` and the same file at
  `data/deeptime_planet/structure/empty_5x5x5.nbt`, one per test namespace, §9.4) and the test pack of
  §7.12 under `deeptime_gametest/`. It is added to the mod model
  (`neoForge { mods { deeptime { sourceSet(sourceSets.main); sourceSet(sourceSets.gametest) } } }`;
  `ModModel.sourceSet` in ModDevGradle 2.0.74), so every dev run finds it through the mod file,
  while the `jar` task packs only `main`, so no test data ships.
- **Runtime mod sets, by symlink.** `localRuntime` carries only Toroidal World. Every other runtime mod
  comes from one of Deep Time's own **server sets** (§9.5), which `tools/smoke/install.sh <set>`
  downloads by pinned version id into `../smoke/<set>/server/mods/`. Before each run, the task
  `linkRunMods` empties the run directory's `mods/` of symlinks, then links the jars of the run's set
  (FML loads mods from the game directory in dev runs). It reads every jar's
  `META-INF/neoforge.mods.toml` and **skips any jar whose mod id is already on the run's classpath**
  (`deeptime`, `toroidal_world`), and it **fails** if two linked jars declare the same mod id: as
  mic-climate found, two jars with one mod id are a hard FML failure. The sets per run and profile:

  | Run | Default | `-PwithMic` | `-PwithTerraBlender` | `-PwithEv` |
  |---|---|---|---|---|
  | `gameTestServer` (`./gametest.sh`) | none (Deep Time + TW) | `mic` | `terrablender` | `ev` |
  | `planetServer` (`./gametest-planet.sh`) | `standalone` | `mic` | — | — |

  A profile whose set is not installed fails with the `install.sh` command to run. The compile
  classpath never carries full third-party mod jars at runtime (`compileOnly` only).
- The **Gradle job in `ci.yml`** (§4.5) runs `./gradlew build test` with host natives and no server set.

### 4.4 The Nix flake

`flake.nix` (flake-parts, like mic-climate's) with inputs `nixpkgs = github:NixOS/nixpkgs/nixos-unstable`,
`flake-parts` and `oxalica/rust-overlay`, provides:

- `devShells.default`:
  - Rust: `rust-bin.stable."1.98.1".default.override { extensions = [ "rust-src" ]; targets = [ …the
    eight targets of §6.4… ]; }` from rust-overlay, so the shell carries all eight standard libraries
    while `rust-toolchain.toml` (which rustup reads in CI) lists none. A flake check asserts that the
    channel in `rust-toolchain.toml` equals the shell's.
  - `(pkgs.cargo-zigbuild.override { zig = pkgs.zig_0_13; })`. nixpkgs' cargo-zigbuild wraps its binary
    with `--prefix PATH : <zig>/bin` (its `postInstall`, currently zig 0.16.0), so putting `zig_0_13` in
    the shell alone would change nothing for it; the override replaces the wrapped zig. Also
    `pkgs.zig_0_13` itself for direct use.
  - `cargo-xwin` plus the LLVM tools it expects on PATH but nixpkgs does not wrap in
    (`propagatedBuildInputs = []`): `llvmPackages.clang-unwrapped` (`clang-cl`), `llvmPackages.lld`
    (`lld-link`) and `llvmPackages.llvm` (`llvm-lib` and friends). cargo-xwin's README asks for clang
    and a full LLVM install; it downloads only the Windows SDK and CRT itself.
  - `cargo-deny`, `jdk21`, `gradle`, `python3`, `mcrcon`, `unzip`, `jq`, `binutils` (`readelf`), and
    the GL/X libraries mic-climate's shell sets on `LD_LIBRARY_PATH` for client runs.
- `apps.natives`: builds all eight natives into `target/natives-dist/<platform>/` and writes
  `natives.json` (§6.5).
- `apps.cli`: `nix run .#cli -- run --preset …`.
- `apps.bluemap`: runs the pinned BlueMap CLI jar on `jdk25` (§9.6).
- `apps.publish`: `tools/review/publish.sh`.

### 4.5 GitHub Actions

**Budget.** `MinecraftIndustrialComplex` is on GitHub's **free** plan with no private repositories so
far (`gh api orgs/MinecraftIndustrialComplex` on 2026-09-28: `plan: free`, `total_private_repos: 0`).
A private repository there gets 2,000 Actions minutes a month, and runner time is billed per minute at
$0.006 (Linux, 2 cores), $0.010 (Windows) and $0.062 (macOS) (GitHub's Actions billing page), so one
macOS minute costs about ten Linux minutes. The workflows are shaped around that:

- `ci.yml` on pushes to `master` and pull requests, with `concurrency: { group: ci-${{ github.ref }},
  cancel-in-progress: true }`:
  - a Rust job: rustup reads `rust-toolchain.toml` (host target only; no cross targets are
    installed), `~/.cargo` and `target/` cached, then `cargo fmt --check`, `cargo clippy -D warnings`,
    `cargo test --workspace`, `cargo deny check`;
  - a Gradle job (added by P0-03): `actions/setup-java` (21), the host Rust toolchain (`hostNatives`
    runs cargo), `~/.gradle` and `~/.cargo` cached, `./gradlew build test` in `neoforge/`, then
    `tools/ci/junit-summary.py`.
- `natives.yml` on tags and manual dispatch only:
  - a build job on `ubuntu-latest` adds the eight targets with `rustup target add`, installs zig 0.13.0
    (`mlugg/setup-zig`), clang and lld, caches `XWIN_CACHE_DIR`, cross-compiles with cargo-zigbuild and
    cargo-xwin (Sable's single-runner approach, `aeronautics_rust_bridge.md` §6), runs the `readelf`
    assertions and uploads `natives-dist`; it also packs the release jar (`./gradlew jar -Prelease`)
    and uploads it, so every load job tests the jar players get (P0-08);
  - load jobs with `fail-fast: false` run `NativeSmoke` (§10.2, P0-07) against that jar on
    `windows-latest`, `windows-11-arm`,
    `macos-14` (arm64), `macos-15-intel` (x64; `macos-13` was retired on 2025-12-04), `ubuntu-24.04`,
    `ubuntu-24.04-arm`, and `docker run eclipse-temurin:21-jdk-alpine` for musl on x64 and on
    `ubuntu-24.04-arm` (JavaScript actions do not run inside Alpine containers on arm64 runners, so the
    container only runs the test). Each job also checks that the loader picked its own platform key.
    This is the only way to test the
    macOS extraction path that breaks Sable (§6.5); this laptop cannot.
  - `workflow_dispatch` finds a workflow only once it is on `master`; before that, a run needs a
    push trigger on the branch (P0-08's bring-up used one and reverted it).
- **Licences.** `cargo deny check licenses` allows only MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause,
  ISC, Zlib, Unicode-3.0, CC0-1.0, MPL-2.0 and **LGPL-3.0-or-later** (cargo-deny checks workspace
  members too, and Deep Time's own crates declare exactly `LGPL-3.0-or-later`; since cargo-deny 0.18.4
  GNU identifiers match pedantically, so the suffix must be exact). It fails on anything GPL or
  unlicensed. A crate outside the list needs a decision (§12.3).

---

## 5. Core data model

All physics runs in SI units (metres, years, kelvin or °C) and maps to blocks only when baking
(§8.8). Planet-scale time is stored in **years relative to world creation**: geologic history is
negative (−5.0e8 is 500 Myr ago), and anything the civ module adds later is positive.

### 5.1 Planet parameters, scales and presets

```rust
// crates/deeptime-core/src/params.rs
pub struct Preset {
    pub schema: u32,                 // preset schema version (independent of the world-data format)
    pub name: String,
    pub planet: PlanetParams,
    pub scale: ScaleParams,
    pub timeline: TimelineParams,
    pub resolution: ResolutionParams,     // grid ladder, §5.2
    pub tectonics: TectonicParams,        // Cortial constants at Earth scale; the Earth-normalized mapping, §8.1
    pub surface: SurfaceParams,           // erosion, diffusion, amplification, drama slider
    pub climate: ClimateParams,
    pub deposits: DepositParams,          // global count/size/richness multipliers, province strength
    pub fantasy: FantasyParams,           // enabled fantasy fields, sky islands, exotic landforms
    pub playability: PlayabilityParams,   // essentials list id, minimums, spawn rules
    pub constraints: Vec<Constraint>,     // paint-and-pin ("later", after Phase 3; empty until then)
    pub limits: RunLimits,                // memory cap, thread count
    pub seed: u64,
}

pub struct PlanetParams {
    pub gravity_m_s2: f64,           // Earth-like presets: 9.81
    pub day_length_h: f64,           // 24
    pub axial_tilt_deg: f64,         // 23.4
    pub star_flux_rel: f64,          // 1.0 = Earth's insolation
    pub ocean_fraction: f64,         // target ocean area fraction for crust generation
    pub regime: TectonicRegime,      // MobileLid (v1); StagnantLid, Episodic later
    pub atmosphere: AtmosphereClock, // oxygenated? land plants from? (secular switches, §8.6)
}

pub struct ScaleParams {
    pub circumference_blocks: u32,   // 16_384 (singleplayer) or 32_768 (server)
    pub m_per_block_h: f64,          // 300.0  (≈1:300)
    pub m_per_block_v: f64,          // 20.0
    pub min_y: i32, pub max_y: i32,  // -352, 639 (fixed by the dimension type in v1)
    pub sea_level_y: i32,            // 63
    pub pole_cap_deg: f64,           // 75.0 (soft cap starts); 80.0 = hard pushback
    pub feature_scale: FeatureScale, // per-feature multipliers: deposits, rivers, canyons, strata
}
// Derived: radius_m = circumference_blocks * m_per_block_h / (2π).
```

**Derived geometry** (the numbers every later section uses):

| Quantity | Singleplayer (C = 16,384) | Server (C = 32,768) | Derivation |
|---|---|---|---|
| Chunks around | 1,024 | 2,048 | C / 16 |
| Circumference | 4,915 km | 9,830 km | C × 300 m |
| Radius | 782 km | 1,565 km | C × 300 m / 2π (the decisions file's "about 1,565 km at 32k" fixes C = 32,768, not 32,000, which would give 1,528 km) |
| Pole-to-pole blocks | 8,192 | 16,384 | C / 2; z ∈ [−C/4, C/4) |
| Latitude per block of z | 0.02197° | 0.01099° | 180° / (C/2) |
| z of the ±75° cap | ±3,413 | ±6,827 | (75/90) × C/4 |
| z of ±80° | ±3,641 | ±7,282 | (80/90) × C/4 |
| Surface area | 7.69 M km² | 30.76 M km² | 4πR² |
| Area between ±75° | 7.43 M km² | 29.71 M km² | × sin 75° = 0.966 |
| Nether width for TW (portal scale 8) | 128 chunks | 256 chunks | C/16/8; TW requires an exact integer and ≥ 16 chunks |

**Vertical mapping.** 20 m per block and y ∈ [−352, 639] (992 blocks, 62 sections; −352 and 992
are multiples of 16 and −352 + 992 = 640 ≤ 2,032, as `DimensionType` requires). With sea level at
y = 63, the world holds 576 blocks = 11.52 km above sea level and 415 blocks = 8.3 km below.
Cortial's elevation envelope maps as: highest ridge −1 km → y 13, abyssal plain −6 km → y −237,
trench −10 km → y −437 (clamped to the floor band), highest continent +10 km → y 563. VE = 300/20
= 15, so a slope's in-game gradient is 15× its physical gradient; the drama slider (§8.2) and the
per-feature multipliers exist to manage that.

**Presets and share codes.** A preset is JSON (schema in
`crates/deeptime-format/schema/preset.schema.json`). A share code is `DT1.` + base64url(deflate(
canonical JSON)) — deflate so Java can decode it with `java.util.zip.Inflater` and no natives (the
server config accepts share codes). Built-in presets ship as JSON in the core and in the mod jar:
`earthlike_quick_16k`, `earthlike_full_16k`, `earthlike_quick_32k`, `earthlike_full_32k`. The
timeline has no default story: the editor makes the player choose a start configuration
(supercontinent / scattered continents / random) and a length before Run is enabled; the server
config must name them explicitly.

*As built in P1-13:* "deflate" is raw DEFLATE (RFC 1951, level 9; Java `new Inflater(true)`),
base64url is unpadded (decoders accept padding and skip whitespace), and canonical JSON is
serde_json's compact form with sorted keys (Java's `Presets.canonical` writes the same bytes,
serde_json 1.0.151's zmij float layout included); `crates/deeptime-core/tests/share-codes.json`
holds codes written by each side and both test suites decode both. A decoded code may inflate to
at most 1 MiB. The preset's header keys are `schema`, `name`, `description`, `placeholder`,
`scale.circumference_blocks` and `timeline` (`{"start": "supercontinent" | "scattered_continents" |
"random", "length_myr": 1…4600}`). Until the real stages exist the four built-in presets are
placeholders (`"placeholder": true`: the dummy pipeline plus the `wrap-test` synthetic terrain
baked at their C); they name no timeline. Their one source is `crates/deeptime-core/presets/`
(compiled into the core, copied into the jar as `deeptime/presets/<name>.json`).

### 5.2 The grids

**Choice.** Two geometries, joined by resampling:

1. **L0 — a Fibonacci sphere** for plates and crust (Cortial et al. 2019 sample their crust exactly
   this way). It samples the sphere uniformly with no pole singularity and no seam, which plate
   motion needs, and Cortial's per-step timings (§8.1) are measured on it. Neighbourhoods come from a
   spherical Delaunay triangulation (the convex hull of the unit vectors) rebuilt at each resampling,
   and a `kiddo` k-d tree on unit vectors for point queries.
2. **L1–L3 — equirectangular rasters in block space**, W × H cells with W = C / r and H = W / 2, where
   r is blocks per cell. Column x maps to longitude 360°·(x + C/2)/C and row z to latitude. They wrap
   in x (index modulo W) and clamp in z.

**Why lat/lon rasters for everything below the plates.** The playable world *is* an X-periodic
equirectangular grid (the TW cylinder, §7.10), so a raster whose cells are whole blocks can be
sampled by chunk generation with one multiply and a modulo, wraps in longitude for free, and produces
the 16k and 32k maps directly (W = C / r). HEALPix, cube-spheres and icosahedral grids would all need
a per-block reprojection with seams and interpolation across faces for every chunk.

**Which metric the surface processes use.** Erosion, deposition, diffusion, hydrology and
amplification run on the raster **in block metric**: every cell is r × 300 m wide in both x and z,
whatever its latitude. Landforms are therefore isotropic in the game world, which is what a player
walking through them sees. Planet-scale inputs (uplift from the plates, insolation from latitude)
are sampled at each cell's true latitude and longitude, so continents and orogens are stretched
east–west by sec φ on the map (2.0 at 60°, 3.9 at 75°; `wrapped_planet_worlds.md` §5.3), exactly
as on any equirectangular map, while valleys and rivers keep their shape. Near the soft caps the
stretch is largest, which is one more reason the caps exist.

**The ladder.** Rasters are named by role; r is set per preset.

| Grid | Role | 16k quick | 16k full | 32k quick | 32k full |
|---|---|---|---|---|---|
| L0 | plates, crust, orogeny (§8.1) | 100k samples | 250k samples | 100k | 250k |
| L1 | history: deep-time erosion, strata log, climate snapshots, deposit fields | 1024×512 (r 16) | 2048×1024 (r 8) | 1024×512 (r 32) | 2048×1024 (r 16) |
| L2 | young landscape: final 2–10 Myr of erosion, hydrology | 2048×1024 (r 8) | 4096×2048 (r 4) | 2048×1024 (r 16) | 4096×2048 (r 8) |
| L3 | amplified final terrain written for chunk generation | 8192×4096 (r 2) | 8192×4096 (r 2) | 8192×4096 (r 4) | 16384×8192 (r 2) |

L0 sample spacing is √(area / N): 8.8 km (29 blocks) at 16k/100k, 5.5 km (18 blocks) at 16k/250k,
17.5 km (58 blocks) at 32k/100k, 11.1 km (37 blocks) at 32k/250k. L1 cells are 4.8 km (r 16) and
9.6 km (r 32) across; the research's surface-process budget was set on 256²–512² history grids and a
1024² young landscape (`surface_processes_climate.md` §7), and L1/L2 here are the same order of
cells (0.5–2.1 M and 2.1–8.4 M). The table is the starting point; the Phase 1 benchmark (P1-17)
replaces it with measured values.

**Coupling L0 → rasters.** For each raster cell, the plate model is sampled through a
cell → (triangle, barycentric weights) lookup rebuilt after each resampling; per-step motion moves
samples rigidly, so between resamplings the lookup is refreshed with k-d-tree queries only for cells
whose triangle left its plate. Scalar fields interpolate barycentrically; categorical ones (plate id,
crust type, orogeny type) take the nearest sample.

### 5.3 Tiled raster layers

```rust
// crates/deeptime-format/src/raster.rs
pub struct LayerDesc {
    pub name: String,            // "elevation", "climate/temperature", …
    pub dtype: DType,            // U8 | I8 | U16 | I16 | U32 | F32
    pub scale: f64, pub offset: f64,   // physical = raw * scale + offset (quantised layers)
    pub units: String,           // "block/16", "vanilla_param*10000", "degC", …
    pub grid: GridDesc,          // W, H, r (blocks per cell), wraps_x: true, origin_block_x/z
    pub tile: u32,               // tile edge in cells: 256
    pub codec: Codec,            // Raw | Lz4Block
    pub nodata: Option<f64>,
}
pub trait RasterRead { fn get(&self, x: i64, z: i64) -> f64; fn bilinear(&self, bx: f64, bz: f64) -> f64; }
```

Tiles are 256 × 256 cells. In memory the core holds whole rasters (row-major, `Vec<T>`); the L3
amplifier alone works tile by tile with a halo when the memory cap requires it (§5.7.5). On disk
every layer is a tile pack (§5.7.3).

**Chunk-time layers (the Java adapter's inputs).** Phase noted where a layer first appears.

| Layer | Grid | Type | Meaning | Phase |
|---|---|---|---|---|
| `elevation` | L3 | i16, 1/16 block | terrain surface height (y) | 0 |
| `water_level` | L3 | i16, 1/16 block; nodata = none | lake/river water surface above sea level | 0 (test), 2 |
| `climate/{temperature,vegetation,continentalness,erosion,weirdness}` | L2 (half L1's cell size) | i16, vanilla parameter ×10,000 | the five 2D biome inputs after the §7.2 mapping | 1 |
| `climate/{t_mean_c,p_mm,koppen}` | L1 | i16 / u16 / u8 | physical climate, for maps and rules | 1 |
| `surface/regolith_blocks` | L2 | u8 | soil/regolith thickness | 1 |
| `surface/class_top` | L2 | u8 (rock class index) | surface lithology where no horizon stack applies | 1 |
| `strata/*` | L1 (Phase 1), L2 if Phase 2 needs it | horizon stack (§5.7.4) | 3D rock | 1 (simple), 2 (Noddy) |
| `fantasy/*`, `sky/*` | L1/L2 | per field | fantasy fields, sky-island density bounds | 3 |
| `meta/{plate_id,crust_age_myr,province}` | L1 | u16 | for the viewer, "Why here?" and legends; `province` also drives TerraBlender regions (§7.3) | 1 |

### 5.4 Rock classes, deposit types and fantasy fields (data-driven)

Content is data. The adapter registers four **NeoForge datapack registries**, so every pack and
mod can add or override entries with ordinary data files, and the Create World screen can read them
from `WorldCreationContext.worldgenLoadContext()` before the world exists:

| Registry | Path in a datapack | Contents |
|---|---|---|
| `deeptime:rock_class` | `data/<ns>/deeptime/rock_class/<name>.json` | physical properties per class |
| `deeptime:deposit_type` | `data/<ns>/deeptime/deposit_type/<name>.json` | mineral-system definitions |
| `deeptime:sim_field` | `data/<ns>/deeptime/sim_field/<name>.json` | fantasy and custom fields |
| `deeptime:rock_states` | `data/<ns>/deeptime/rock_states/<name>.json` | the adopted block-state format (§7.5) |

Plus a fifth for playability: `deeptime:essentials` (`data/<ns>/deeptime/essentials/<name>.json`).
Block tags (`deeptime:rock/<class>`, `deeptime:rock_fallback/<class>`) stay ordinary block tags;
they matter only at chunk generation, never to the simulation. Before a session starts, the adapter
serialises the registries' contents through their codecs into one JSON document and passes it to
`create` (§6.1). The CLI builds the same document from mod jars and datapack folders
(`--data <jar|dir>`), so CLI and in-game runs see the same content.

#### 5.4.1 Rock class

```json
{
  "class": "black_shale",
  "family": "clastic_sedimentary",
  "groups": ["deeptime:rock/clastic", "deeptime:rock/sedimentary"],
  "density_kg_m3": 2400,
  "erodibility": 1.6,
  "diffusivity": 1.2,
  "metamorphic_path": ["slate", "phyllite", "schist", "gneiss"],
  "weathers_to": "clay",
  "facies_rule": "euxinic_basin",
  "tags": ["source_rock", "reduced"],
  "vanilla_fallback": "minecraft:smooth_basalt"
}
```

Deep Time ships definitions for all 59 classes of the vocabulary in
`/home/greencheetah/Projects/mic/Docs/datapacks/deeptime-mic-rock-tags/README.md` in its own jar
(`neoforge/src/main/resources/data/deeptime/deeptime/rock_class/`). The numeric properties are
relative multipliers the erosion and structure stages consume; their values are tuning parameters
set in P1-07 against the viewer, and each file records where its value came from.
`vanilla_fallback` is the class's vanilla stand-in from the rock-tag README (the last entry of its
fallback list, or its vanilla member), so a pack without any rock tags still resolves every class
(§7.5).

#### 5.4.2 Fantasy fields

A fantasy field is a named scalar published per L1 cell and time step, exactly like the real fields
the deposit rules read (`geo:` fields in `ore_genesis_modeling.md` Q7). Real fields are built into
the core; fantasy fields are defined in data and computed by the core from a small set of kinds:

```json
{
  "id": "mymod:mana_flux",
  "kind": "coupled_noise",
  "coupled_to": "deeptime:plume_flux",
  "coupling": 0.6,
  "scale_km": 200,
  "decay_myr": 50,
  "range": [0, 1]
}
```

Kinds (v1, Phase 3): `coupled_noise`, `event_source` (spawned by listed event kinds, decaying with
time and distance), `diffused` (sources plus a diffusion step), `constant`. Deep Time ships its own
fantasy fields (`deeptime:aether_flux` for sky islands, `deeptime:ley_density`), and its fantasy
deposits and landforms read them.

#### 5.4.3 Deposit type

The schema is `ore_genesis_modeling.md` Q7's nine field groups, with `deeptime:` field names. The
abridged porphyry example there is the template; the v1 changes are:

- `hosts` names rock classes and groups from §5.4.1, with an optional `host_bias` map (class → weight)
  — this is how the Create stones become preferred hosts (crimsite/BIF → iron, ochrum/ironstone →
  gold, veridium/serpentinite → copper, asurine/dolostone → zinc).
- `sizing.tonnage_mt`, `grades`, `density_per_1e5km2`, `min_separation_m` keep the USGS values.
- `minecraft` gains `coe_vein` (a COE vein recipe id or `null`), `cdg_reservoir` (boolean), and
  `ore_block_policy` (`ev_then_vanilla`, the decisions file's rule).
- `balance` keeps `count_mult`, `size_mult_h`, `size_mult_v`, `richness_mult`, and adds `province`
  (regional-specialisation strength, default `strong`).
- `requires_fields` lists fantasy fields with a fallback, as in Q7's mythril example.

Phase 1 ships the four families the decisions file names first:
`deeptime:porphyry_cu_mo_au` (with `deeptime:supergene_cu_blanket` and `deeptime:placer_au`
children), `deeptime:bif_fe` and `deeptime:coal_seam`, `deeptime:mvt_zn_pb` and
`deeptime:sedex_zn_pb`, and `deeptime:petroleum_trap`.

### 5.5 The shared event log

One append-only log records every notable happening, for the timelapse ticker, "Why here?" and a
later legends view. It is designed so a second writer (the civ module) can append without touching
what geology wrote.

```rust
// crates/deeptime-format/src/events.rs
pub struct Event {
    pub id: EventId,             // u64 = (writer_id << 48) | sequence — no coordination between writers
    pub t_start: f64,            // years relative to world creation (negative = geologic past)
    pub t_end: f64,              // = t_start for instantaneous events
    pub kind: KindId,            // interned namespaced id: "deeptime:rift_onset", "civ:city_founded"
    pub source: SourceId,        // interned: "deeptime:plates", "deeptime:erosion", "civ:politics"
    pub place: Place,
    pub magnitude: f32,          // kind-defined scalar for ranking and filtering
    pub causes: Vec<EventId>,    // causal links, possibly across writers
    pub params: Params,          // kind-defined typed map (string → i64 | f64 | str | id)
}
pub enum Place {
    Point { lat: f32, lon: f32 },
    Plate { plate: u32 },
    Region { mask: MaskId },     // a stored L1 bitmask (e.g. an orogen's footprint)
    Site { site: SiteId },
    Path { path: PathId },       // stored polyline (a trench, a river)
    Global,
}
```

- **Storage.** `events/<writer>-<seq>.dtev` segments, each immutable once written: a header, a
  string table (kinds, sources, param keys), fixed-width event records, a params blob and a
  per-segment time index. `events/index.json` lists segments and writers. The core writes geology
  as writer 1; the civ module registers as another writer and adds its own segments.
- **Indexes.** By time (sorted within segments, merged on load), by L1 cell (for "what happened
  here"), by kind, and by cause (reverse links, built on load).
- **Kinds used by geology** (v1): plate birth/death, rift onset, ridge birth, subduction onset,
  collision, terrane accretion, orogeny phase, hotspot track, large igneous province, basin
  subsidence, marine transgression/regression, glaciation, erosion pulse, deposit formation,
  deposit exhumation, placer formation, supergene enrichment, petroleum generation and trap charge,
  and `deeptime:guaranteed_occurrence` (a deposit added by the playability pass, §7.8).
  Each kind's params are documented in `crates/deeptime-format/EVENTS.md`.
- **Why here?** Every site stores its forming event; the forming event's `causes` lead back through
  the mineral-system components (the collision that thickened the crust, the arc that supplied the
  magma, the uplift that exposed it), and its params record which component limited barren
  neighbours (`ore_genesis_modeling.md` Q3).

### 5.6 The site registry

```rust
// crates/deeptime-format/src/sites.rs
pub struct Site {
    pub id: SiteId,                    // u32, stable for the life of the world
    pub kind: SiteKind,                // Deposit | Reservoir | Placer | Supergene | Fantasy
    pub deposit_type: TypeId,          // e.g. deeptime:porphyry_cu_mo_au
    pub center: [i32; 3],              // block space, canonical span
    pub lat_lon: [f32; 2],
    pub aabb: [[i32; 3]; 2],           // block-space bounds of every body of this site
    pub body: BodyId,                  // geometry in bodies/ (§5.7.4)
    pub products: Vec<Product>,        // material, grade, tonnage_t, role
    pub forming_event: EventId,
    pub parent: Option<SiteId>,        // placer → its eroded source, blanket → its porphyry
    pub extraction: Extraction,        // finite?, amount_base, coe_vein, cdg_charge_mb
}
```

Stored as `sites/sites.dtsr` (records) plus `sites/index.dtsi` (a 512-block grid → site ids).
**Java API** (`com.minecraftindustrialcomplex.deeptime.api`), for the future unlimited-mining
system, prospecting and compat:

```java
public interface DeepTimeSites {
    static DeepTimeSites of(ServerLevel level);           // empty implementation outside Deep Time worlds
    boolean isDeepTimeLevel();
    Optional<Site> at(BlockPos pos);                      // the site whose body contains pos
    List<Site> inChunk(ChunkPos pos);                     // sites whose AABB touches the chunk
    List<Site> within(AABB box);
    Optional<Site> byId(int id);
    Optional<GeoEvent> formingEvent(Site site);
    List<GeoEvent> causeChain(GeoEvent event, int maxDepth);
}
public record Site(int id, SiteKind kind, ResourceLocation depositType, BlockPos center,
                   BoundingBox bounds, List<Product> products, long formingEvent,
                   OptionalInt parent, Extraction extraction) {}
```

### 5.7 The on-disk world-data format

#### 5.7.1 Layout

```text
<world>/deeptime/
  manifest.json          format_version, writer versions, preset hash, planet/scale params,
                         grid descriptors, layer list, file sizes and CRC32s, creation time
  preset.json            the exact preset used (shareable)
  content.json           the rock classes, deposit types and fields the run saw (for provenance)
  climate_mapping.json   written by Java at first start (§7.2)
  rasters/<layer>.dtr    tile packs (§5.7.3)
  strata/horizons.dtr    horizon stacks; strata/classes.json (class index table)
  bodies/bodies.dtb      intrusions, deposit bodies, sky islands (parametric shapes + small voxel blobs)
  sites/sites.dtsr  sites/index.dtsi
  events/*.dtev  events/index.json
  keyframes/kf-<n>.dtk   timelapse frames (downsampled, LZ4); keyframes/index.json
  checkpoints/           optional stage checkpoints (dev and CLI only; never needed to play)
```

#### 5.7.2 Versioning

`manifest.json` carries `"format_version": "0.<minor>"`. Before 1.0, the Java reader accepts exactly
its own `0.<minor>`; any other version refuses to extend the world with a message naming both
versions ("world data 0.3, this Deep Time reads 0.4; before 1.0 worlds are not migrated"), logs it,
and leaves already-generated chunks playable. From 1.0 on, every format bump ships a migration in
`deeptime-format` (run by the CLI and by the adapter at first start) and a golden migration test.
Each file has a 16-byte header: magic (`DTR1`, `DTEV`, `DTSR`, …), header length, format minor, CRC32
of the header.

#### 5.7.3 Tile packs and memory-mapped reads

A `.dtr` file is: header → JSON `LayerDesc` → tile index (per tile: u64 offset, u32 compressed
length, u32 CRC32) → tiles. Each tile is either raw little-endian cells or one **LZ4 raw block**
(`lz4_flex::block`, independent per tile). Java maps the whole file read-only with
`FileChannel.map(READ_ONLY, …)` and decodes a tile on first touch with lz4-java's pure-Java
`LZ4Factory.safeInstance().safeDecompressor()`. lz4-java is on the 1.21.1 game classpath (vanilla's
`RegionFileVersion.VERSION_LZ4` imports `net.jpountz.lz4`); the raw block format, not
`LZ4BlockInputStream`'s own framing, is what both sides share. Decoded tiles live in a bounded
concurrent cache (Guava `CacheBuilder`, on the game classpath; weight = tile bytes; default 256 MiB,
configurable). A P0 test round-trips tiles between `lz4_flex` and lz4-java (P0-09).

#### 5.7.4 Strata and bodies

- **Horizon stacks.** Per cell of a strata grid: up to K horizons, each an i16 elevation (1/16 block)
  of the horizon's top and a u8 class index, ordered top-down; K is a preset value (32 in the
  built-in presets). Evaluating a column is a walk down the stack — the "lattice + deformed horizon
  surfaces" design the research recommends so chunk generation costs a per-column lookup rather
  than per-block kinematics (report §"Nothing scientific embeds").
- **Bodies.** Intrusions, deposit bodies and sky islands as parametric primitives (ellipsoid,
  capsule, tilted sheet/dyke, pipe, stockwork shell with zones, lens, blanket) with transforms, AABBs
  and zoning, plus optional small voxel blobs (u8 class per voxel, LZ4) where a primitive is not
  enough. Java evaluates primitives with the same formulas as Rust; the parity test covers them.

#### 5.7.5 Memory estimate and the cap

The simulation's native memory is estimated from the preset before `create`, shown in the editor
and the server log, and enforced while running. The default cap is **4 GiB**; the editor and the
server config raise it freely (a 64 GB machine can run a huge planet). The estimate uses these
per-cell budgets (design budgets, checked against measurement in P1-17):

| State | Bytes per cell | Contents |
|---|---|---|
| LEM working set (L1, L2) | 40 | elevation, sediment, uplift, erodibility, runoff (f32); receiver, stack index, basin id (u32); drainage area, scratch (f32) |
| Strata log (L1 only) | 12 × K | per layer: age f32, thickness f32, class u16, provenance u16 |
| Climate (L1) | 100 | 12 monthly T and P (f32) + derived |
| Deposit fields (L1) | 160 | about 20 mineral-system fields (`ore_genesis_modeling.md` Q1) + per-type favourability |
| Amplifier (L3) | 20 | elevation, sediment, area (f32), receiver (u32), scratch (f32) |

Peak estimates (plates at L0 are under 50 MB at 250k samples and are ignored):

| Preset | L1 cells | L1 state (K = 32 → 40 + 384 + 100 + 160 = 684 B/cell) | L3 cells | L3 working set (20 B/cell) | Peak ≈ L1 + L3 |
|---|---|---|---|---|---|
| 16k quick | 524,288 | 0.33 GiB | 33.6 M | 0.63 GiB | 0.96 GiB |
| 16k full | 2,097,152 | 1.34 GiB | 33.6 M | 0.63 GiB | 1.96 GiB |
| 32k quick | 524,288 | 0.33 GiB | 33.6 M | 0.63 GiB | 0.96 GiB |
| 32k full | 2,097,152 | 1.34 GiB | 134.2 M | 2.50 GiB | 3.84 GiB — within 0.16 GiB of the 4 GiB cap |

The 32k full preset therefore streams L3 by default: 4,096² tiles with a 64-cell halo
(4,224² × 20 B = 0.33 GiB), so it peaks near 1.34 + 0.33 ≈ 1.7 GiB and spills finished tiles to the
staging folder. L2 (at most 4096 × 2048 × 40 B = 0.31 GiB) is freed before L3 starts. Any preset
whose estimate exceeds its cap streams L3 automatically before the editor or server asks the user to
raise the cap. The cap is enforced by a counting global
allocator in `deeptime-jni` and `deeptime-cli` (`#[global_allocator]` wrapping `System` with atomic
counters): a stage that would exceed it returns `CoreError::MemoryCap` before allocating, and the
counter's peak is reported in `bench.json` and the progress block. The native heap is separate from
the JVM heap, so a player needs RAM for both; the editor shows the machine's free memory beside the
estimate.

**Disk** (raw, before LZ4). 16k quick: `elevation` and `water_level` at L3 are 33.6 M × 2 B =
64 MiB each; the five biome-parameter layers at L2 (2048 × 1024 × 2 B) are 4 MiB each; horizon stacks
(K = 32, 3 B per horizon) at L1 (1024 × 512 × 96 B) are 48 MiB. 32k full: `elevation` and
`water_level` at L3 (16384 × 8192 × 2 B) are 256 MiB each; biome layers at L2 (4096 × 2048 × 2 B)
16 MiB each; horizon stacks at L1 (2048 × 1024 × 96 B) 192 MiB. LZ4 shrinks smooth fields; the ratio
is measured in P1-17, not assumed.

---

## 6. The JNI boundary

### 6.1 API sketch

Read-back for chunk generation deliberately does not cross JNI: it is the pure-Java world-data
reader (§3.1 rule 1, §5.7.3). The one native read-back, `sampleColumn`, exists so tests can compare
the Java and Rust column evaluators. Java side, one class of static natives plus a safe wrapper:

```java
package com.minecraftindustrialcomplex.deeptime.nativebridge;

final class DeepTimeNative {                     // package-private; only SimSession and NativeLoader call it
    static native String version();              // "<semver>+<git sha>;format=<f>;target=<rust triple>", e.g. "0.0.1+ab12cd3;format=0.0;target=x86_64-unknown-linux-gnu"
    static native long   selfTest();             // fixed-input integer Rayon reduction; returns a checksum Java recomputes
    static native String estimate(String presetJson, String contentJson);   // JSON: peak bytes, per-stage bytes and seconds
    static native long   create(String presetJson, String contentJson, String outDir,
                                int threads, long memCapBytes);             // returns a session id (never a pointer)
    static native int    advance(long session, int budgetMillis);           // RUNNING | STAGE_DONE | DONE | CANCELLED
    static native void   cancel(long session);                              // any thread; sets a flag
    static native void   progress(long session, long[] out);                // fills PROGRESS_FIELDS longs (below)
    static native int    keyframeCount(long session);
    static native String keyframeInfo(long session, int index);             // JSON: t, stage, w, h, layer names
    static native void   copyKeyframe(long session, int index, int layer, ByteBuffer dstRgba); // direct buffer, w*h*4
    static native String eventsSince(long session, long cursor, int max);   // JSON array for the ticker
    static native String drainLog(long session);                            // native log lines → SLF4J
    static native String finalizeWrite(long session);                       // after DONE: flushes files, returns manifest JSON
    static native void   destroy(long session);
    // Parity oracle, used only by tests (§9.2):
    static native String sampleColumn(String worldDataDir, int x, int z);   // JSON column as Rust evaluates it
}

public final class SimSession implements AutoCloseable {   // one per run
    public static SimSession start(Preset preset, ContentBundle content, Path outDir, SimOptions opts);
    public Progress progress();            // cheap; safe from the render thread
    public List<Keyframe> newKeyframes();  // copies frames produced since the last call
    public List<GeoEvent> newEvents();
    public void cancel();
    public CompletableFuture<Manifest> completion();
    @Override public void close();         // cancel if running, then destroy
}
```

Progress fields (`long[]`): state, stage index, stage count, stage permille, total permille, simulated
time (years × −1, as a long), elapsed ms, ETA ms, native bytes in use, native peak bytes, keyframes
ready, events total.

**What each entry point does in Phase 0** (P0-07), before real stages exist: `create` accepts only
the dummy preset `{"dummy":{"steps":N,"sleep_ms":M}}` (anything else throws
`PresetInvalidException`); `advance` runs `DummyStage`, which counts to N and sleeps M ms per step;
`estimate` returns `{"peak_bytes":0,"stages":[]}`; `keyframeCount` returns 0 and `keyframeInfo` /
`copyKeyframe` throw `UnsupportedOperationException`; `eventsSince` returns `[]`; `finalizeWrite`
writes a manifest with `format_version` `0.0` and no layers; `sampleColumn` throws
`UnsupportedOperationException` until P1-08. `selfTest` sums `(i × 2654435761) mod 2^64` for
i = 0 … 2^20 − 1 with a Rayon parallel reduction and returns the sum. Integer arithmetic makes the
result identical on every platform and in any reduction order, so `NativeLoader` computes the same sum
in a plain Java loop and compares; no checksum has to be produced by running a cross-built library.
It stays this simple after Phase 0: its job is to prove the library links, runs Rayon and returns.

Rust side (`crates/deeptime-jni/src/lib.rs`), with jni-rs 0.22.4 (released 2026-03-16, MIT or
Apache-2.0; MSRV 1.85):

```rust
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_minecraftindustrialcomplex_deeptime_nativebridge_DeepTimeNative_advance<'l>(
    mut env: EnvUnowned<'l>, _class: JClass<'l>, session: jlong, budget_ms: jint,
) -> jint {
    env.with_env(|env| -> Result<jint, BridgeError> {       // jni-rs catches unwinding panics inside with_env
        let s = SESSIONS.get(session as u64)?;               // handle table: RwLock<HashMap<u64, Arc<Session>>>
        let status = s.advance(Duration::from_millis(budget_ms as u64))?;
        Ok(status as jint)
    })
    .resolve::<ThrowDeepTimeException>()                     // custom ErrorPolicy: maps BridgeError → Java exception
}
```

- **Handles are ids.** `SESSIONS` maps a u64 to `Arc<Session>`. A stale or forged id raises
  `IllegalStateException` instead of dereferencing freed memory.
- **One stepping caller.** `Session::advance` takes the session's step mutex; `progress` and
  `cancel` touch only atomics and never block on it.
- **Keyframes** are produced by the core as already-coloured RGBA8 images (using `ramps.json`) at a
  fixed size per layer (1024 × 512 for the globe/map view), kept in a bounded ring in memory and
  written to `keyframes/` as they are produced, so the ring can drop old frames without losing them.
  `copyKeyframe` writes into a Java-allocated direct `ByteBuffer`
  (`get_direct_buffer_address`), which the render thread then copies into a `NativeImage`.

### 6.2 Threading

| Thread | Language | Does | Never does |
|---|---|---|---|
| Render thread (client) | Java | `progress`, `newKeyframes`, `newEvents`, draws, `cancel` | `advance`, file I/O |
| `deeptime-sim` worker (daemon) | Java | `create`, loops `advance(250)`, `finalizeWrite`, `destroy` | touch GUI objects |
| Server thread at first start | Java | the same loop, blocking, logging progress every 5 s | return before `finalizeWrite` |
| `deeptime-rayon-<n>` | Rust | data-parallel kernels inside `advance` | call back into the JVM |
| Worldgen threads | Java | read world data | call JNI |

The Rayon pool is a **private `rayon::ThreadPool` per session** (`ThreadPoolBuilder::num_threads`),
not the global pool, so each run can choose its thread count and the pool is dropped with the
session. Default threads: available processors − 1 on a client (the render thread stays responsive),
all processors at server first start and in the CLI. Rust threads never attach to the JVM.

### 6.3 Error handling

- Every native entry point runs inside `with_env`, which catches unwinding panics (jni-rs 0.22).
  The release profile keeps `panic = "unwind"`; `panic = "abort"` would make the guard inert and turn
  every bug into a JVM crash.
- `CoreError` variants map to Java exceptions in
  `com.minecraftindustrialcomplex.deeptime.nativebridge`: `PresetInvalidException` (with the field
  path), `MemoryCapException` (needed vs cap), `DeepTimeIoException`, `CancelledException`, and
  `NativePanicException` (panic message + location). After a panic the session is poisoned and every
  later call on it throws `SessionPoisonedException`; other sessions and the JVM carry on.
- Native log records go to a bounded ring that `drainLog` empties into the SLF4J logger
  `deeptime.native`, so native diagnostics land in `latest.log` in order with Java's.
- Java never trusts partial output: world data is written into `outDir` and becomes visible only
  when `finalizeWrite` writes `manifest.json` last (write to `manifest.json.tmp`, fsync, rename).

### 6.4 Build and targets

| Platform key | rustup target (the name in rust-overlay and `rustup target add`) | Build command target | Built with | Notes |
|---|---|---|---|---|
| `linux-x86_64-gnu` | `x86_64-unknown-linux-gnu` | `x86_64-unknown-linux-gnu.2.17` | cargo-zigbuild | the `.2.17` suffix is a cargo-zigbuild feature (glibc floor), not a rustup target; 2.17 is the floor Sable's library needs, and Sable issue #592 shows why a newer floor breaks old hosts |
| `linux-aarch64-gnu` | `aarch64-unknown-linux-gnu` | `aarch64-unknown-linux-gnu.2.17` | cargo-zigbuild | |
| `linux-x86_64-musl` | `x86_64-unknown-linux-musl` | same | cargo-zigbuild | `-C target-feature=-crt-static` in `.cargo/config.toml` (rustc drops `cdylib` for musl otherwise, rust-lang/cargo#10143); the `.so` needs `libgcc_s` on Alpine (rust-lang/rust#82521) |
| `linux-aarch64-musl` | `aarch64-unknown-linux-musl` | same | cargo-zigbuild | same flags (musl for both architectures: decisions file, "Native targets") |
| `windows-x86_64` | `x86_64-pc-windows-msvc` | same | cargo-xwin | |
| `windows-aarch64` | `aarch64-pc-windows-msvc` | same | cargo-xwin | |
| `macos-x86_64` | `x86_64-apple-darwin` | same | cargo-zigbuild | Sable needed `codegen-units=2` here to avoid a rustc hang; copy the setting |
| `macos-aarch64` | `aarch64-apple-darwin` | same | cargo-zigbuild | |

- **Conservative CPU baseline.** No `-C target-cpu` and no `+avx*` features anywhere. The default
  targets are already conservative: generic x86-64 on Linux, x86-64 plus SSE3/CMPXCHG16B on Windows,
  Penryn on Intel macOS, Armv8-A on Linux arm64, Apple M1 on macOS arm64 (rustc target specs). This
  avoids Power Grid's SIGILL on a QEMU virtual CPU (PowerGrid #457), which no Java code could catch.
- **Toolchain.** Stable Rust 1.98.1 (the latest stable on 2026-09-28), no nightly features. zig 0.13.0
  for cargo-zigbuild 0.23.4, wired in through the package override of §4.4 (nixpkgs' own pairing is
  zig 0.16.0, which cargo-zigbuild's documentation does not yet cover; if 0.13 ever blocks a build,
  switching to the nixpkgs default is a one-line change to test). cargo-xwin 0.22.0 downloads the
  Microsoft CRT and Windows SDK at build time; using it constitutes consent to Microsoft's licence for
  them (`https://go.microsoft.com/fwlink/?LinkId=2086102`), which Ben accepted on 2026-09-28 (§2.7).
  `NOTICE.md` records it.
- Every musl artifact is checked with `readelf -d` in CI: no `NEEDED libc.so.6`, no `ld-linux`, no
  `GLIBC_*` versions. `tools/natives/build-natives.sh` runs these checks wherever it builds (P0-08).
- **Measured in P0-08.** The macOS natives are built with cargo-zigbuild on every host, the Mac
  included: a plain `cargo build` inside the Nix dev shell on macOS links the Nix store's
  `libiconv.2.dylib`, which no player's Mac has. zig 0.13 raises the minimum macOS of both macOS
  libraries to 11.7.1 (its own floor; rustc's defaults are 10.12 and 11.0). They link only
  `/usr/lib/lib{System.B,iconv.2,charset.1}.dylib`, and `.cargo/config.toml` gives them the install
  name `@rpath/libdeeptime_jni.dylib` instead of the build path. The zig-built musl libraries need
  only `libc.so` (musl's loader itself; no `libgcc_s`). The Windows DLLs import `VCRUNTIME140.dll`
  and the UCRT; every JVM loads `VCRUNTIME140.dll` itself, so no static CRT flag is needed.
  nixpkgs carries cargo-xwin 0.23.1; CI pins the same with cargo-zigbuild 0.23.4.

### 6.5 Native loading and extraction

The jar carries `natives/deeptime/<platform key>/<file>` and `natives/deeptime/natives.json`
(per file: sha256, size, Rust target, glibc floor; plus the Cargo workspace version and the format
version). Its schema (schema 1; written by Gradle `hostNatives` and by `build-natives.sh`, read by
`NativeLoader`; P0-07):

```json
{
  "schema": 1,
  "version": "0.0.1",
  "format_version": "0.0",
  "libraries": [
    {
      "platform": "linux-x86_64-gnu",
      "file": "libdeeptime_jni.so",
      "rust_target": "x86_64-unknown-linux-gnu",
      "glibc_floor": "2.17",
      "sha256": "<64 lowercase hex digits>",
      "size": 1234567
    }
  ]
}
```

- `schema` is 1; the loader refuses any other value (reported as `VERSION_MISMATCH`: reinstall).
- `version` is the Cargo `[workspace.package] version`; `format_version` is
  `[workspace.metadata.deeptime] format_version`, the format version's one source (the writers read
  it from `Cargo.toml`; `deeptime-format`'s `build.rs` makes it `FORMAT_VERSION`, so the Rust constant
  and the JSON cannot drift).
- `libraries` has one entry per platform key of §6.4 (one in a dev jar, eight in a release jar);
  the file sits at `natives/deeptime/<platform>/<file>`.
- `glibc_floor` is the highest `GLIBC_x.y` symbol version the library needs (the writer scans the
  file), `null` for non-gnu targets. `size` is in bytes. An optional `git_sha` is informational.

`NativeLoader` is **JDK-only**: no Minecraft, NeoForge, Gson, SLF4J or Guava types, because
`NativeSmoke` runs it on a bare JVM with only the mod jar on the classpath. It parses
`natives.json` with a small built-in JSON reader and logs through a `NativeLogSink` callback, which
the mod points at SLF4J (default: `System.Logger`). The mod constructor calls
`NativeLoader.load(FMLPaths.GAMEDIR.get())` once on both sides; it **never throws** and returns a
`NativeStatus`. Extraction and verification are separate from `System.load`
(`NativeLoader.extract(gameDir) → ExtractResult` and `NativeLoader.link(ExtractResult) → NativeStatus`),
so unit tests can exercise extraction many times in one JVM while a library is linked at most once
per JVM (P0-07).

1. **Platform key.** `os.name`/`os.arch`, plus libc on Linux: read the ELF `PT_INTERP` of
   `/proc/self/exe` (the JVM's own dynamic loader — `ld-musl-*` means musl). If that fails, try the
   gnu build, then the musl build.
2. **Override.** `-Ddeeptime.natives.path=<file>` loads a user-supplied library (for antivirus
   exclusions or unusual systems) and skips extraction.
3. **Target directory.** `<gamedir>/deeptime/natives/<version>-<first 12 hex of sha256>/`, where
   `<version>` is the Cargo workspace version. The version-and-hash directory means a file that
   another running instance has loaded is never overwritten (Windows locks loaded DLLs, and Sable's
   overwrite-every-launch approach fights that).
4. **Create parents first.** `Files.createDirectories(dir)` before any file operation. Sable's
   recurring macOS failure (`NoSuchFileException: .sable/natives/…`, issues #1291, #1369, #1423) is a
   missing parent directory, not Gatekeeper; files the JVM writes itself are not quarantined, and
   library validation only applies to hardened-runtime launchers, which the common launchers are not.
5. **Lock, verify, reuse.** Take `FileChannel.lock()` on `dir/.lock` (two game instances can share a
   game directory). If the file exists with the right sha256, reuse it. Otherwise write
   `<file>.tmp`, fsync, verify the sha256, `Files.move(…, ATOMIC_MOVE)` (fall back to
   `REPLACE_EXISTING`), then re-verify: a file that vanishes or changes between write and load is
   the antivirus-quarantine signature (Sable #691).
6. **Link** with `System.load(absolutePath)`, then call `version()` and `selfTest()`. The version
   check compares **semver and format only** (never the git sha): the library's
   `<semver>;format=<f>` must equal the pair recorded in `natives.json`, which the build wrote from
   the same Cargo workspace, and `selfTest()` must return the sum Java computes itself (§6.1).
7. **Prune** other versions' directories best-effort (ignore locked files).

**One version source.** The Cargo `[workspace.package] version` is the version. `gradle.properties`
`mod_version` must equal it; the Gradle task `checkVersions` (a dependency of `build`) fails the build
when they differ.

### 6.6 When natives fail to load

`NativeStatus.failure(FailureClass, platformKey, attemptedPath, cause, guidance)` is logged once as an
ERROR with the full cause and a guidance block. Nothing crashes. Errors outside the table below are
folded into the nearest class (P0-07): an `UnsatisfiedLinkError` that names no libc, with the file
intact, is `UNSUPPORTED_PLATFORM` (the library cannot run here); a missing `natives.json` is
`UNSUPPORTED_PLATFORM`, an unreadable one `VERSION_MISMATCH`; any unexpected exception in the loader
is `IO_ERROR` with its text. What still works and what does not:

| Situation | Behaviour |
|---|---|
| Create World, Deep Time preset | The preset is listed. Its editor opens on a **guidance panel** instead of the setup page: the detected platform, the failure class and what to do. "Create" is disabled for Deep Time. Other presets are unaffected. |
| Dedicated server first start, no world data | Logs the guidance block and stops the server before `loadLevel()` with a clear message, so no half-made world is saved. The guidance names the CLI route: run `deeptime run` on any supported machine and copy `deeptime/` into the world folder. |
| Existing world (world data present) | Plays normally. Chunk generation, sites and every in-game feature read world data with pure Java. |
| Client joining a server | Unaffected: clients never run the simulation. |

The five failure classes, how each is detected, its guidance, and how tests force it:

| `FailureClass` | Detected when | Guidance | Forced in tests by |
|---|---|---|---|
| `UNSUPPORTED_PLATFORM` | no library in `natives.json` for the platform key | the CLI route above | `-Ddeeptime.natives.forceFail=unsupported` |
| `LIBC_MISMATCH` | `UnsatisfiedLinkError` whose message names `GLIBC_`, `ld-linux` or `ld-musl`, after both libc variants were tried | install the musl build, or update glibc (the floor is 2.17) | `-Ddeeptime.natives.forceFail=libc_mismatch` (throws a synthetic `UnsatisfiedLinkError` whose message names `GLIBC_2.99`) |
| `ANTIVIRUS_SUSPECTED` | the extracted file vanishes or changes between write and load | add an exclusion for the printed directory, or use `-Ddeeptime.natives.path` | `-Ddeeptime.natives.testDeleteAfterWrite=true` |
| `IO_ERROR` | any `IOException` while creating, locking, writing or moving | the directory and the exception text | `-Ddeeptime.natives.forceFail=io` (the target directory is made read-only) |
| `VERSION_MISMATCH` | the linked library's semver/format differs from `natives.json`, or its self-test sum differs from Java's | reinstall the mod; a stray `-Ddeeptime.natives.path` library is the usual cause | `-Ddeeptime.natives.forceFail=version_mismatch` (checks against a deliberately wrong expected version) |

---

## 7. The Minecraft adapter

Java package root: `com.minecraftindustrialcomplex.deeptime` (decided; decisions file, "Repo").
Sub-packages: `worldgen` (generator, density functions, passes, features), `worlddata` (reader),
`climate`, `rock`, `ore`, `wrap` (TW shape, pole cap, fallback seam), `screen` (client), `server`
(first start, staging, commands), `net`, `api` (sites), `compat/{coe,cdg,terrablender,sable,ev,dh}`,
`nativebridge`, `mixin`, `gametest`.

Every API fact below was checked against the NeoForge 21.1.249 sources jar or against the jars the
**current** MIC pack pins (downloaded from `pack/mods/*.pw.toml` on 2026-09-28 and hash-checked:
Tectonic 3.0.28, Lithostitched 1.8.0+beta6, Terralith 2.6.2, C2ME 0.4.0-alpha.0.122, Distant
Horizons 3.3.0, Create Ore Excavation 1.6.8, Create: Diesel Generators 1.3.15, Sable 2.0.5, Create
Aeronautics 1.3.2, Create 6.0.10, Chunky 1.4.23), unless it cites a research note. Deep Time
registers no blocks or items, so a world keeps only vanilla and pack blocks; after removing the mod,
the generator, dimension type and preset references in `level.dat` are the player's to clean up
(decisions file, "Removal").

### 7.1 `GeologicChunkGenerator`

```java
public final class GeologicChunkGenerator extends NoiseBasedChunkGenerator {   // NBCG is not final
    public static final MapCodec<GeologicChunkGenerator> CODEC = RecordCodecBuilder.mapCodec(i -> i.group(
        BiomeSource.CODEC.fieldOf("biome_source").forGetter(g -> g.biomeSource),
        NoiseGeneratorSettings.CODEC.fieldOf("settings").forGetter(g -> g.sourceSettings), // the pack's minecraft:overworld
        WorldDataRef.CODEC.fieldOf("world_data").forGetter(g -> g.dataRef)             // {folder, circumference_blocks, staging_id?, test_terrain?, router?}
    ).apply(i, GeologicChunkGenerator::new));

    private final Holder<NoiseGeneratorSettings> sourceSettings;  // serialised by reference
    private final Holder<NoiseGeneratorSettings> derived;         // Holder.direct(...) with our router
    private final WorldDataRef dataRef;

    GeologicChunkGenerator(BiomeSource src, Holder<NoiseGeneratorSettings> pack, WorldDataRef ref) {
        this(src, pack, ref, DerivedSettings.derive(pack, ref));
    }
    private GeologicChunkGenerator(BiomeSource src, Holder<NoiseGeneratorSettings> pack,
                                   WorldDataRef ref, Holder<NoiseGeneratorSettings> derived) {
        super(src, derived);               // super sees the direct holder: no registry key
        this.sourceSettings = pack; this.derived = derived; this.dataRef = ref;
    }
    @Override public Holder<NoiseGeneratorSettings> generatorSettings() { return derived; } // ChunkMap builds RandomState from this
    @Override protected MapCodec<? extends ChunkGenerator> codec() { return CODEC; }

    @Override public void buildSurface(WorldGenRegion region, StructureManager sm, RandomState rs, ChunkAccess chunk) {
        super.buildSurface(region, sm, rs, chunk);              // the pack's surface rules repaint default_block
        WorldData wd = ActiveWorldData.get(dataRef.folder());
        StrataPass.apply(chunk, wd, RockPalette.current());     // §7.5
        WaterPass.apply(chunk, wd);                             // §7.9
    }
}
```

- **Codec and registration.** Registered as `deeptime:geologic` in `Registries.CHUNK_GENERATOR`
  (a `DeferredRegister<MapCodec<? extends ChunkGenerator>>`). `ChunkMap` builds the `RandomState`
  from `generatorSettings().value()` when the generator is `instanceof NoiseBasedChunkGenerator`
  (`ChunkMap` constructor), so the derived router is what runs. `world_data.circumference_blocks`
  puts C in the generator itself, so Toroidal World's shape (§7.10) and the terrain source know it
  before any world data exists; the manifest's C must match it, or binding fails with a message.
- **Terrain source.** `deeptime:terrain` and `deeptime:depth` sample a `TerrainSource`, chosen when
  the level is bound: `AnalyticTestTerrain` — a closed-form height H(x, z) periodic in x with period
  C (a few sine continents, one straddling the seam, a range peaking near y 600, smooth enough that
  |H″| stays well under 0.01 block per block²) — when the codec has `test_terrain` or the server
  runs with `-Ddeeptime.testTerrain=<kind>` (logged as a warning); otherwise `WorldDataTerrain`,
  which reads the world data's `elevation` layer. Phase 0 builds the analytic source first, so the
  Toroidal World spike (P0-06) runs before any Rust exists. The optional `router` field names the
  density-function set (`deeptime:planet` by default; `deeptime:planet_nocaves` for the GameTest
  dimensions, §9.4).
- **`DerivedSettings.derive`** builds a new `NoiseGeneratorSettings` record: `noiseSettings`
  (min_y −352, height 992, size_horizontal 1, size_vertical 2), the pack's `defaultBlock` and
  `defaultFluid`, **our `NoiseRouter`**, a **delegating surface rule** (below), the pack's
  `spawnTarget`, `seaLevel` 63, `aquifersEnabled` true, **`oreVeinsEnabled` false** (vanilla's large
  veins off, as decided), `useLegacyRandomSource` from the pack. It is wrapped in `Holder.direct`, so
  it has no key: Tectonic's `NoiseBasedChunkGeneratorMixin` lava fix only fires for a settings key
  whose path is `overworld` (decompiled from Tectonic 3.0.28), and a keyless holder never
  matches.
- **Delegating surface rule.** `DelegatingRuleSource implements SurfaceRules.RuleSource` resolves
  `sourceSettings.value().surfaceRule()` when applied, not when derived, so surface-rule edits that
  TerraBlender or Lithostitched make to the pack's overworld settings after this generator was built
  still apply. TerraBlender tags a `RuleCategory` onto the settings object its per-stem
  initialisation reaches (`generatorSettings().value()`, our derived record), and its
  `MixinNoiseGeneratorSettings` wraps `surfaceRule()` of that object, with our delegating rule as the
  fallback. P0-12 checks both families on real jars.
- **The noise router** is data: `data/deeptime/worldgen/density_function/planet/*.json`, so packs can
  retune it. `DerivedSettings` reads the registered holders:

| Router field | Value |
|---|---|
| `barrier`, `fluid_level_floodedness`, `fluid_level_spread`, `lava` | the vanilla `minecraft:overworld/…` aquifer functions, by reference |
| `temperature`, `vegetation`, `continents`, `erosion`, `ridges` | `flat_cache(deeptime:raster{layer: "climate/<name>"})` |
| `depth` | `deeptime:depth` = (H(x,z) − y) / 128, zero at the terrain surface, as vanilla's surface biome points expect (P1-09's biome tests would expose a wrong offset). Vanilla's `overworld/depth` falls 3.0 over 384 blocks (1.5 at y = −64 to −1.5 at y = 320), i.e. 1/128 per block; keeping that slope keeps biome `depth` bands meaningful |
| `initial_density_without_jaggedness` | `deeptime:terrain{mode: "initial"}` |
| `final_density` | vanilla's overworld `final_density` graph with `overworld/sloped_cheese` replaced by `deeptime:terrain{mode: "final"}` and the top/bottom slides re-anchored to −352..639; cave functions (`minecraft:overworld/caves/*`) kept by reference, so the pack's caves (Terralith's, in MIC) still carve |
| `vein_toggle`, `vein_ridged`, `vein_gap` | `constant 0` |

- **Density function types** (registered in `Registries.DENSITY_FUNCTION_TYPE`,
  `worldgen/density_function_type`), all pure, allocation-free and thread-safe, because vanilla
  calls them from `Util.backgroundExecutor()` and C2ME's compiler runs unknown types as delegates
  (C2ME 0.4.0-alpha.0.122 nests the `c2me-neoforge-opts-dfc` compiler module, with integration
  packages for Lithostitched and Tectonic density functions):
  - `deeptime:raster {world, layer, fallback}` — bilinear sample of a 2D layer at (x, z), wrapping
    in x. A per-thread last-tile cache avoids the tile-cache lookup on consecutive samples.
  - `deeptime:terrain {world, mode, k}` — (H(x,z) − y)/k, where H comes from the level's terrain
    source (the closed form for the test terrain; bicubic L3 `elevation` for world data) plus, from
    P1-06 on, the sub-cell detail term of §8.2 (drama slider). Linear in y, so vertical interpolation in
    8-block cells is exact (`minecraft_integration_architecture.md` §2).
  - `deeptime:depth {world}` — as above.
  - `deeptime:beyond_pole {world}` — a mask for |z| beyond ±C/4 (past the pole edge), where terrain
    becomes an ice-sheet ocean; nobody reaches it through the soft cap, but chunks there still get
    generated at view distance.
  - Phase 3: `deeptime:sky_islands` and `deeptime:sky_aware` (§7.4).
- **Lazy binding.** Density functions get neither seed nor level
  (`minecraft_integration_architecture.md` §1). `ActiveWorldData` is a static map
  `folder → WorldData`, filled at `ServerAboutToStartEvent` (after staging is moved in, §7.12) and
  cleared at `ServerStoppedEvent`. A function evaluated before binding throws an
  `IllegalStateException` naming the folder, instead of returning plausible garbage. This is the
  ReTerraForged idea (`Supplier<WorldLookup>`, `CellSampler`) without its `RandomState` mixin,
  because Deep Time's data depends on the world folder, not the seed.
- Inherited `getBaseHeight`, `getBaseColumn`, `createBiomes`, carvers, structures and
  `applyBiomeDecoration` all keep working because they read the router. That is what keeps
  `/locate`, structures, World Preview-style tools and Distant Horizons' vanilla-chunk mode working
  (the TFC counter-example in the research).

### 7.2 Climate parameters for biomes

The core outputs physical climate per L1 cell (monthly temperature and precipitation, Köppen class,
aridity, distance to coast, local relief, a ridge/valley index). Java turns them into the five 2D
vanilla parameters once per world and bakes the result into the `climate/*` layers' companion
file `climate_mapping.json`:

| Vanilla parameter | From | Mapping |
|---|---|---|
| temperature | mean annual temperature, warmest/coldest month, Köppen group | piecewise-linear anchors placing Köppen E in the frozen band, D in cold, C in temperate, A/B-hot in warm/hot |
| vegetation (humidity) | annual precipitation and aridity index | anchors placing Köppen BW at the dry end and Af at the wet end |
| continentalness | signed distance to coast (sea negative) and water depth | pinned so the simulated coastline falls on vanilla's coast band and deep water on the deep-ocean band |
| erosion | local relief on L2 (high relief → low erosion value) | quantile map onto the pack's own erosion distribution |
| weirdness (ridges) | ridge/valley index (curvature) on L2 | quantile map onto the pack's distribution |

The band boundaries come from `OverworldBiomeBuilder` in the 1.21.1 sources (P1-09 reads them;
this plan does not restate numbers it has not verified), and the quantile maps come from sampling
the pack's own overworld router, built from the frozen registries, at 65,536 points — Genesis's
fixed-height trick (`minecraft_integration_architecture.md` §3). Anchors ship as
`data/deeptime/deeptime/climate_mapping/default.json` and can be overridden per pack. The mapping is
computed once (on the Create World screen's review page, or at a server's first start) and stored in
the world, so a later pack update does not shift biomes in unexplored chunks.

### 7.3 TerraBlender regions follow geography

TerraBlender applies to a level stem when its `DimensionType` is in the tag
`terrablender:overworld_regions`, the generator is `instanceof NoiseBasedChunkGenerator` and the
biome source is `instanceof MultiNoiseBiomeSource`; it keys on nothing else
(`mod_compatibility_biomes_ores_fantasy.md` follow-up). Deep Time satisfies all three: its dimension
type is added to the tag with one `"replace": false` file, and the biome source is the shared
`multi_noise` with `preset: minecraft:overworld`.

To make regions follow the simulated geography, port ReTerraForged's four MIT mixins (commit
`ba35ca7`) into `compat/terrablender/`, applied by the mixin plugin only when `terrablender` is
loaded:

1. `TargetPointMixin` adds a `uniqueness` field to `Climate.TargetPoint`.
2. `ClimateSamplerMixin` computes it at the end of `Climate.Sampler.sample`.
3. `NoiseChunkMixin` wraps the `terrablender:uniqueness` density function per chunk in
   `cachedClimateSampler`.
4. `ParameterListMixin` redirects TerraBlender's own `getUniqueness(x,y,z)` call inside
   `findValuePositional` to Deep Time's value, falling back to TerraBlender's noise underground.

Deep Time registers `terrablender:uniqueness` as `deeptime:region_index{world}`. The core writes a
**province id** per L1 cell (`meta/province`: cratons, orogens, arcs, basins and island groups as
contiguous provinces); Java maps province id → region index by a deterministic weighted hash over
TerraBlender's registered regions and their weights, stored in `climate_mapping.json`. One Biomes
O' Plenty region therefore covers whole provinces rather than TerraBlender's own tessellation.
MIC has no TerraBlender, so this is tested in the `-PwithTerraBlender` profile (P1-10).
Lithostitched's own `region` registry (used by Regions Unexplored, not in MIC) is out of scope.

### 7.4 Steering mod sky biomes onto simulated sky islands (Phase 3)

Terralith places `terralith:skylands_{winter,autumn,spring,summer}` where continentalness ∈ [−0.745,
−0.455], erosion ∈ [−0.2225, 0.05], weirdness ∈ [0.19, 1] and depth ∈ [−0.005, 0], split by
temperature and humidity (parsed from Terralith 2.6.2's `lithostitched:biomes` parameter list). Its
island *terrain* comes from a density spline that Deep Time's router replaces, so the biomes would
float over open ocean. Deep Time instead simulates its own islands (§8.7) and steers biomes onto
them:

- `deeptime:sky_islands` adds the islands' density to `final_density` as
  `max(terrain, islands)`, from `bodies/` (sky-island bodies with AABBs).
- `deeptime:sky_aware{inner, band_param}` wraps each of the five climate functions *without*
  `flat_cache`: inside a sky-island body's vertical range it returns the target band value; elsewhere
  it returns the ground value. `deeptime:depth` measures depth from the island top inside islands.
  Biome sampling is 3D (quart positions), so the sea floor under an island keeps its ocean biome.
- Target bands are data: `data/deeptime/deeptime/sky_biome_band/terralith.json` carries the values
  above and loads only when Terralith is present (a NeoForge `neoforge:mod_loaded` condition, the
  idiom Tectonic uses). Without a sky-biome mod, islands take the biome their climate implies.

### 7.5 The strata pass

`StrataPass.apply(chunk, wd, palette)` runs after the pack's surface rules (which only repaint
`default_block`, `SurfaceSystem.buildSurface`) and before carvers and features:

1. For each of the 256 columns, read the horizon stack for (x, z) — horizon tops interpolated
   bilinearly between the four surrounding strata cells, class from the nearest — and the bodies
   index for the chunk's AABB.
2. Walk down from the surface. Replace only `minecraft:stone` and `minecraft:deepslate` (the
   unclassified crust the router and surface rules leave behind) with the block for the class at that
   y; bodies (intrusions, alteration halos, gossans) override horizons inside their SDF.
3. Write whole runs into the section palettes, not block by block.

**Block resolution (`RockPalette`)**, rebuilt at server start and on `TagsUpdatedEvent`:

- Class *c* uses the blocks of `#deeptime:rock/<c>` plus `rock_states` entries with `use: member`
  (skipping `required: false` entries whose block is absent).
- If that is empty, it uses the **first installed** entry of `#deeptime:rock_fallback/<c>`, then
  `rock_states` entries with `use: fallback`. Tags keep list order, so the pack controls the look.
- If that is empty too, it uses the class's `vanilla_fallback` (§5.4.1). Deep Time itself ships
  only vanilla *member* tags (`deeptime:rock/<class>` for the 18 vanilla blocks the README assigns)
  and **no fallback tags**, so a pack's fallback lists (MIC's included, which use `replace: false`)
  are never pre-empted by entries from Deep Time's jar, and a pure-vanilla pack still resolves every
  class (Q15 in §14).
- `coal` is the one class allowed to fall back to `minecraft:stone` (the decisions file: Born in
  Chaos black argillite, then stone). The MIC datapack's `rock_fallback/coal.json` already lists
  `born_in_chaos_v1:black_argillite` (`required: false`), then `minecraft:stone` (checked 2026-09-28).
- A class that still resolves to nothing stays unclassified crust.
- With several members, one member is chosen per horizon or body (hash of its id), so a layer is one
  block type, not noise; tag order sets the primary member.
- `rock_states` entries set a full block state (e.g. Project Atmosphere `dust` at `layers=8` as
  `loess`); unlisted properties take the block's default.

A side effect the research noted still holds: unknown modded ores whose rule tests target
`#minecraft:stone_ore_replaceables` (stone, granite, diorite, andesite) or
`#minecraft:deepslate_ore_replaceables` (deepslate, tuff) can only appear in those hosts after the
strata pass, so "unknown ores keep scattering, gated by host rock" needs no extra code.

### 7.6 The deposit feature

`DepositFeature extends Feature<NoneFeatureConfiguration>` (`deeptime:deposits`), placed at
`UNDERGROUND_ORES` (ordinal 6 of `GenerationStep.Decoration`) through
`data/deeptime/neoforge/biome_modifier/deposits.json` (`neoforge:add_features`, biomes
`#minecraft:is_overworld`). It does nothing unless the chunk's generator is a
`GeologicChunkGenerator`. For each site whose AABB touches the chunk, it evaluates the body's zones
and grade into a per-block ore probability (a position hash, so regeneration is identical), and
replaces host rock with the ore block for the zone's material.

**Ore block choice (`OreBlockResolver`)**, following the decisions file (Excavated Variants when
installed, otherwise normal stone or deepslate ores):

1. **Excavated Variants present** (4.3.1 has a NeoForge 1.21.1 build; LGPL-3.0): EV's
   `RegistryKeys.ORE` entries whose base blocks are in `#c:ores/<material>`, then
   `Ore.getStones()` (`ResourceKey<Stone> → ResourceKey<Block>`) with the host block's EV stone key
   (a block → stone reverse map built from `RegistryKeys.STONE` at server start) gives the
   ore-in-that-rock block.
2. Otherwise, or if EV has no variant for that host: a block in both `#c:ores/<material>` and
   `#c:ores_in_ground/deepslate` below y = 0, else `#c:ores_in_ground/stone`.
3. Otherwise any block in `#c:ores/<material>`.

EV only generates variants for stones its configuration knows at startup (block registration
happens before tags load), so Deep Time ships EV stone configs for the vanilla and Create members of
the rock vocabulary under `globaldata/deeptime/excavated_variants/`; packs add their own for
bring-your-own stones. P1-15 verifies EV's config schema and whether EV's own
`attempt_worldgen_replacement` must be off in Deep Time worlds to avoid double replacement.
(P1-15, verified on EV 4.3.1 by javap: EV registers `excavated_variants:ore`/`:stone` in
`BuiltInRegistries.REGISTRY`; after registration `Ore.getStones()` maps each stone to the ore's
block in it, originals and generated variants alike, so no id arithmetic is needed. EV already ships
stone configs for vanilla's stone, deepslate, granite, diorite, andesite, tuff, calcite, sandstone,
red sandstone, basalt, smooth basalt and dripstone and for Create's seven stones, so Deep Time ships
only the vanilla stand-ins EV lacks (polished tuff, packed mud, smooth sandstone, cobblestone) through
DefaultResources' `defaultresources.meta.json` + `defaultresources/globaldata/deeptime/excavated_variants/stone/`.
EV's own replacement (`excavated_variants:ore_replacer`, added to every biome's
`TOP_LAYER_MODIFICATION` in the `AFTER_EVERYTHING` phase, gated by `attempt_worldgen_replacement` in
`config/excavated_variants.json`, default true) must be off in Deep Time levels: it re-picks an ore's
stone from its neighbours with chunk-edge bookkeeping, so a regenerated deposit differed in 200 of
1,406 ore blocks. Deep Time turns it off per level with a compat mixin
(`mixin/compat/excavated_variants/OreReplacerMixin`), so other levels of the pack keep it; the
config stays untouched.)

### 7.7 Ore claiming

- **Scatter removal.** A custom biome modifier type `deeptime:claim_ores` (serializer in
  `NeoForgeRegistries.BIOME_MODIFIER_SERIALIZERS`), shipped as
  `data/deeptime/neoforge/biome_modifier/claim_ores.json` for `#minecraft:is_overworld`. In its
  REMOVE phase it removes every placed feature whose configured feature (followed through
  `random_selector`, `simple_random_selector` and `weighted`) places a block in `#c:ores/<m>` for a
  **claimed** material `m`. NeoForge has no per-world biome-modifier scoping, but
  `ServerLifecycleHooks.handleServerAboutToStart` sets the current server and then calls
  `runModifiers(server)` before posting `ServerAboutToStartEvent`, so the modifier reads
  `ServerLifecycleHooks.getCurrentServer()`'s overworld level stem and does nothing unless its
  generator is a `GeologicChunkGenerator`. That scopes it to Deep Time worlds without a datapack
  toggle.
- **Claimed materials** are the products of loaded deposit types plus the background-scatter list
  (§7.8): in v1 copper, iron, coal, gold, zinc, lead and whatever MIC's datapack adds.
- **Unknown ores keep scattering**, gated by host rock (§7.5).
- **Vanilla large ore veins are off** (`oreVeinsEnabled = false`, §7.1).
- **Rock-blob features.** Vanilla's granite/diorite/andesite/tuff blobs, Terralith ReStoned's
  restored blobs and Create's `striated_ores_overworld` place rock-class blocks at random, which
  contradicts simulated lithology. The same modifier removes them when `claim_rock_features` is on
  (default on); this is Q4 in §14.
- (P1-15 as built: only ore-shaped features are judged — `minecraft:ore`, `scattered_ore`,
  `replace_single_block`, `create:layered_ore`, `lithostitched:ore`, plus the modifier's
  `ore_feature_types`; selectors are followed — and the blocks they place are read by encoding the
  configured feature with its codec, so Create's layered ore needs no Create API. A rock blob is an
  ore-shaped feature placing only bedrock-class blocks; loose pockets (dirt, gravel, clay) and
  terrain features that merely contain rock or ore (Terralith's floating islands, boulders,
  pillars) stay. The GameTest server's overworld is not Deep Time's, so its tests check the rules as
  a dry run and the planet server checks the removal.)

### 7.8 Background scatter, playability minimums, spawn

- **Background scatter** (`deeptime:background_scatter`, also at `UNDERGROUND_ORES`): small clusters
  of common materials anywhere, host-rock-gated, from data files
  `data/<ns>/deeptime/scatter/<material>.json` (`clusters_per_chunk`, `size`, `y_window` as a
  fraction of crust thickness, `host_bias` by rock class). It exists so nobody "searched for hours
  and found nothing" (IE #2229, COE #85). (P1-15 as built: `y_window` is [f0, f1] of the column from
  the world data's surface (0) to the bottom of the world (1) — the playable crust — and the rules
  are the `deeptime:scatter` registry.)
- **Playability minimums.** The `deeptime:essentials` registry lists materials with
  `min_sites_per_region`, `region_blocks`, `max_blocks_from_spawn` and `exposed`. The core's deposit
  stage adds, after natural placement, the smallest plausible deposit at the most favourable cell of
  any region short of its minimum, and records it as event kind `deeptime:guaranteed_occurrence`,
  so "Why here?" stays honest. Deep Time ships a vanilla essentials list; MIC's datapack ships its
  own (pack-specific fudging is expected).
- **Spawn.** The core ranks spawn candidates (land, temperate Köppen C or mild D, low slope, fresh
  water within reach, every essential within its `max_blocks_from_spawn`) and stores the top ten.
  The adapter handles `LevelEvent.CreateSpawnPosition` (cancellable; fired once for a new level): it
  cancels vanilla's search, sets the spawn to the best candidate that is not under water after chunk
  generation, and leaves the world border as §7.10 decides.

### 7.9 Water above sea level

Vanilla places fluid only through the aquifer (sea level and local cave levels), so lakes above sea
level need their own pass. `WaterPass.apply` runs after the strata pass: where `water_level` is above
the terrain surface, it fills the air between them with water source blocks (ice at the top where
the climate layer says frozen), without scheduling fluid ticks, so a basin stays still until a
neighbour changes. The bake guarantees `water_level` never exceeds the lowest rim of its basin
(Fill–Spill–Merge levels, Phase 2); Phase 0 proves the mechanism with a hand-made basin and channel
(P0-13). The generated ground is the density interpolated over 4-block noise cells, so on a steep
shore it lies a block or two below the elevation raster: a column whose own `water_level` cell is dry
takes the level of the nearest wet cell within one noise cell (P0-13's first run left two shore
pockets that the woken lake filled), and the bake must therefore keep each rim above its level for at
least one noise cell beyond the last wet cell. (P0-13: the ice comes from vanilla's
`freeze_top_layer` feature, which every overworld biome runs after this pass.) Carvers run after this pass and can open caves under a lake; that drains into caves the way
vanilla lakes do and is accepted.

### 7.10 Toroidal World: the cylinder shape

- **Dependency.** Toroidal World 0.19.3 (MIT, NeoForge 1.21.1). It has no published Maven artifact
  (its `maven` branch does not exist), so the build uses the Modrinth maven
  (`maven.modrinth:toroidal-world:<version id>`) for both `compileOnly` and `localRuntime`; the API
  classes (`com.toroidalworld.api.v1.*`) are inside the mod jar. `neoforge.mods.toml` declares it
  `required`, side `BOTH` (every player needs TW).
- **Shape.** Registered in the mod constructor:

```java
ShapeModule.of(ResourceLocation.fromNamespaceAndPath("deeptime", "planet"),
        PlanetShape.DEFAULT,                                   // C from the generator codec (world_data.circumference_blocks)
        (dims, s) -> ShapeDimensions.withSpans(dims,
                LoopSpans.ofWidth(Direction.Axis.X, s.chunks()),        // overworld: X loops, Z does not
                LoopSpans.ofWidth(Direction.Axis.X, s.chunks() / 8),    // nether: exact 1:8
                s.endSpans(),                                          // End: decided in the spike
                s.options()),                                          // climate compression OFF
        PlanetShape::read)
    .register();
```

- **Stamping.** TW rebuilds only generators whose class is exactly `NoiseBasedChunkGenerator` or
  `FlatLevelSource`; a subclass is *stamped* (same instance, shape attached), and TW's
  `NoiseBasedChunkGeneratorMixin` wraps `doFill`, `doCreateBiomes` and `iterateNoiseColumn` on the
  base class, so a subclass tiles at the seam (TW's own JUnit test
  `aNoiseSubclassOverworldKeepsItsClassAndTakesTheShape`). Structures straddling the seam are moved by
  TW's `StructureStartMixin`, independent of the generator.
- **Periodicity is Deep Time's job.** Every raster wraps in x, and `deeptime:terrain`'s detail noise
  is periodic with period C, so column −C/2 continues column C/2 − 1.
- **Open spike items (P0-06):** how and when a `ShapeModule`'s `Apply` runs for a world created from
  `deeptime:planet` (singleplayer and `level-type=deeptime:planet` on a server), and whether `Read`
  can take C from our generator's codec on both paths; TW's `climate_compression`
  option (default `auto`) must be off for Deep Time's stems; TW's terrain ceiling and crumb sweep must
  not touch Deep Time's settings; what TW expects of the vanilla world border for a cylinder (the
  border is square, so Z is limited by the soft cap instead, §7.11); the End's spans.
- **Fallback if the spike fails:** `wrap/fallback/SeamTeleporter` — Deep Time's own teleport at the
  seam (every entity each tick, vehicles with passengers, Sable bodies through
  `PhysicsPipeline.teleport(body, Vector3dc, Quaterniondc)`, which keeps velocity), with real ghost
  chunks beyond the seam from the periodic generator (`wrapped_planet_worlds.md` §6.1 item 4).

### 7.11 The pole soft cap

`wrap/PoleCap` (server tick): for players, vehicles and Sable sub-levels poleward of `pole_cap_deg`
(75°), apply a pushback toward the equator that grows with latitude, an action-bar warning, and a
storm state sent to the client (fog and particles, client-side only). Sable ships get wind through
`RigidBodyHandle.addLinearAndAngularVelocity` (public API in Sable 2.0.5). At 80° the pushback
becomes a hard stop. The level's sub-levels come from Sable's public
`SubLevelContainer.getContainer(ServerLevel)` → `ServerSubLevelContainer.getAllSubLevels()`
(`List<ServerSubLevel>`; `javap` on Sable 2.0.5). Terrain exists all the way to the poles;
beyond the pole edge (|z| ≥ C/4) `deeptime:beyond_pole` makes an ice-sheet ocean.

*As built in P1-18:* the lines are the generator's `world_data.pole_cap_deg` / `pole_stop_deg`
(defaults 75 / 80; `wrap/PoleCapCurve` holds the geometry and curves). With s the depth into the
band: pushback 0.10·s^1.5 blocks/tick² toward the equator, storm s^0.7, Sable wind 1.0·s^1.5
blocks/s per tick. A player's own movement belongs to its client, so the server sends the wind in
`PoleStormPayload` and Deep Time's client adds it (to the player or the vehicle it steers); the
server pushes the vehicles it simulates itself (a player's root vehicle, e.g. a minecart) and
enforces the hard stop (absolute teleport onto the line; vehicles moved with their riders; riders
of contraptions dismounted). Creative: storm, warning and stop, no wind; spectator: storm only.
Sable: wind at the pose, stop by the hull's poleward edge (poleward velocity cancelled, never a
teleport). Details in STATUS.md's P1-18 entry.

### 7.12 World preset, dimension type and the Customize editor

Resources in `neoforge/src/main/resources/data/`:

| File | Content |
|---|---|
| `deeptime/worldgen/world_preset/planet.json` | overworld: type `deeptime:planet_overworld`, generator `{type: "deeptime:geologic", biome_source: {type: "minecraft:multi_noise", preset: "minecraft:overworld"}, settings: "minecraft:overworld", world_data: {folder: "deeptime", circumference_blocks: 16384}}`; nether and End: vanilla |
| `deeptime/worldgen/world_preset/planet_32k.json` | the same with `circumference_blocks: 32768`; not in the `#minecraft:normal` tag (the editor sets the size in singleplayer), but selectable on a server with `level-type=deeptime\:planet_32k` |
| `minecraft/tags/worldgen/world_preset/normal.json` | `{"replace": false, "values": ["deeptime:planet"]}` (the Create World list is `#minecraft:normal`) |
| `deeptime/dimension_type/planet_overworld.json` | vanilla overworld values with `min_y` −352, `height` 992, `logical_height` 992 |
| `terrablender/tags/dimension_type/overworld_regions.json` | adds `deeptime:planet_overworld` (`required: false`) |
| `deeptime/worldgen/density_function/planet/*.json` | the router (§7.1) |

The level stem stays `minecraft:overworld`, which Lithostitched's `levels`-scoped modifiers and
biome injectors key on. Test-only resources (the test dimensions and the `deeptime:test_bare` biome
of §9.4) live in a separate built-in pack, `neoforge/src/gametest/resources/deeptime_gametest/`
(its own `pack.mcmeta` and `data/deeptime/…`; the `gametest` source set of §4.3, never in the jar).
Only when `-Ddeeptime.gametest=true`, an `AddPackFindersEvent` handler registers it with
`event.addPackFinders(ResourceLocation.fromNamespaceAndPath("deeptime", "deeptime_gametest"),
PackType.SERVER_DATA, Component.literal("Deep Time GameTest"), PackSource.BUILT_IN, true,
Pack.Position.TOP)`; NeoForge 21.1's helper resolves the path through the mod file's
`findResource`, and `alwaysActive = true` makes the pack required. The GameTest server enables every
available pack anyway (`GameTestServer.create` builds its `DataPackConfig` from
`packRepository.getAvailableIds()`). `-Ddeeptime.extraDatapack=<folder>` (P1-12) adds one more
required pack at the top, from a folder outside the repository, and is honoured only together with
`-Ddeeptime.gametest=true`. `addPackFinders` resolves paths inside the mod file only, so this pack
goes through `event.addRepositorySource` with `Pack.readMetaAndCreate(new PackLocationInfo(
"deeptime_extra", …), new PathPackResources.PathResourcesSupplier(folder), PackType.SERVER_DATA,
new PackSelectionConfig(true, Pack.Position.TOP, false))` (review 2, gap 29; fixed in P1-12). Gradle
sets the property from `-PextraDatapack=<folder>`; `-PwithMic` defaults it to the MIC rock-tag
datapack next to the pack (`…/mic/Docs/datapacks/deeptime-mic-rock-tags`, which
`tools/remote/mac.sh --pack` syncs to the Mac).

**The editor.** `RegisterPresetEditorsEvent.register(ResourceKey<WorldPreset> deeptime:planet,
PresetEditor)` on the client mod bus; `PresetEditor.createEditScreen(CreateWorldScreen,
WorldCreationContext)` returns `PlanetEditorScreen`:

1. **Setup page.** Size (16k / 32k), scale, timeline (start configuration and length; no default
   story), planet parameters, fantasy toggles, multipliers, the essentials list, a preset dropdown,
   share-code import/export, and the live memory and time estimate from `estimate` (§6.1). If
   natives failed, this page is the guidance panel (§6.6).
2. **Run page (the timelapse).** Map/globe toggle, time scrubber, event ticker, layer toggles
   (elevation, plates, crust age, rock, climate, ore provinces), progress, Cancel. Keyframes are
   drawn from `copyKeyframe` into a `NativeImage` (ABGR pixel order) → `DynamicTexture.upload()` →
   `GuiGraphics.blit`.
3. **Review page.** The final map with the pack's biome names (the pack's `MultiNoiseBiomeSource`
   plus the §7.2 mapping, computed here and staged), the spawn marker and ore-province hints (not
   exact deposits), then Accept.

Accept calls `lastScreen.getUiState().updateDimensions((registries, dims) ->
dims.replaceOverworldGenerator(registries, GeologicChunkGenerator.forStaging(stagingId, …)))` and
sets the TW shape's settings. The world folder does not exist yet (`WorldCreationUiState` picks it
when Create is pressed), so data waits in `<gamedir>/deeptime/staging/<stagingId>/`. No NeoForge
event fires between Create and the integrated server's `initServer`; the first hook is
`ServerAboutToStartEvent`, which runs before `loadLevel()`. There `server/StagingMover` moves the
staging folder into `<world>/deeptime/` (an atomic rename on the same filesystem, else copy, verify
CRCs, delete) and binds `ActiveWorldData`.

*As built in P1-11 (before the real stages):* the screens are client-only classes in `screen/`,
reached only through `screen/ClientSetup`, a `@Mod(dist = CLIENT)` entry point (registers the
editor for `deeptime:planet` and `planet_32k`); every rule lives in the plain-Java
`screen/model/` (`EditorModel`, `EditorFields`, `TimelapseModel`, `ReviewModel`, `RunHandle` /
`SessionRunHandle`, `Ramps`, `GlobeProjection`, `ProvinceHints`, `GuidanceModel`, `Formats`), which
JUnit and the GameTests drive. **Parameters**: `EditorFields` names each field's preset JSON path
(`planet.gravity_m_s2`, `scale.m_per_block_h`, `deposits.count_multiplier`,
`scale.feature_scale.rivers`, `surface.drama`, `playability.essentials`, `fantasy.*`, …); a field
is editable only when the selected preset carries its path, so the placeholder presets (which the
core rejects any unknown key for) show them unavailable and the real presets light them up with no
screen change (if P1-03…P1-17 name a path differently, only that catalogue line changes; fantasy
toggles stay disabled until Phase 3). **Size** selects the built-in of the chosen family
(`earthlike_quick` / `earthlike_full` × 16k / 32k); an imported code's size is fixed. **Run** is
disabled until a start configuration and a length are chosen, the estimate is in and under the cap
(default 4 GiB, any higher value accepted; the machine's free and total memory are shown beside it).
`SimSession.launch` creates the session on the `deeptime-sim` worker (plan §6.2), which also
collects keyframe infos and events after every step and before it destroys the session; keyframe
pixels come through `copyKeyframe` while the session lives and from `keyframes/kf-<n>.dtk`
(pure-Java `worlddata/KeyframeFiles`) afterwards, decoded on a loader thread into a 64 MiB ring. The
globe is an orthographic projection drawn on the CPU into a 256² texture. The review page's map is
the staged world data's `elevation` raster coloured with `ramps.json` (copied into the jar as
`deeptime/ramps.json`); the spawn marker is the first `spawn/candidates.json` entry above sea
level; ore-province hints are the dominant deposit type of 30° cells, sites blurred over their
neighbours (`worlddata/SiteRecords` reads `sites/sites.dtsr`). The pack's biome names and the
staged §7.2 mapping wait for P1-09. **Accept** is `PlanetGenerators.withStaging` (the current
overworld's biome source and settings, `WorldDataRef` with the staging id and the preset's C;
`PlanetShape` then shapes TW from that C at bake). `WorldDataBinder`'s HIGHEST listener runs
`StagingMover` first, then `FirstStartGenerator.prepare`, then binds; a failed move stops a
dedicated server (`ServerStop`) and throws on an integrated one. The copy route verifies every
file's CRC32 against the source and every manifest entry, with plain reads (a memory-mapped tile
pack would block the rename on Windows). **Create is disabled** for a Deep Time world until staged
data exists (`PlanetGenerators.createBlock`: no run accepted, natives missing, or the staging
folder gone); Enter is held back too (`screen/CreateWorldGuard`). Abandoned staging folders
(untouched for 24 h and held by no open editor) are deleted at client setup and at every server
start. A running simulation keeps going when the editor is closed; reopening Customize shows it.

Tectonic 3.0.28 ships a client mixin, `WorldCreationUiStateMixin.tectonic$removeUnusablePresets`
(injected at the return of `WorldCreationUiState.getNonEmptyList`). Decompiled, it only removes
presets in the tag `#tectonic:incompatible`, whose shipped entries are `minecraft:large_biomes` and
`minecraft:amplified`. It does not hide `deeptime:planet`, so no compat mixin is needed; Deep Time must
simply never be added to that tag.

### 7.13 Dedicated server: generate on first start

`server/FirstStartGenerator` listens to `ServerAboutToStartEvent` at priority HIGHEST (TerraBlender
initialises at LOWEST). In `DedicatedServer.initServer` the order is
`handleServerAboutToStart` → `loadLevel()` → RCON → `handleServerStarting`, so this runs before any
level exists and while RCON is down:

1. If the overworld generator is not Geologic, do nothing.
2. If `<world>/deeptime/manifest.json` exists (a previous start, or a CLI run copied in), bind and
   continue.
3. Otherwise read `config/deeptime/server.json`
   (`{"preset": "earthlike_full_32k" | "<file>" | "DT1.<share code>", "timeline": {"start":
   "supercontinent" | "scattered_continents" | "random", "length_myr": 1000}, "memory_cap_gib": 4,
   "threads": 0}`; review 2 gap 21: `timeline` is required unless the preset itself names one, and
   wins when both do);
   if the preset's circumference differs from the generator's `circumference_blocks` (set by the
   chosen `level-type`), stop with a message naming both. Then
   run a `SimSession` to completion on this thread, and log a progress line every 5 s
   (`[deeptime] plates 37% (−312 Ma) · 1m12s elapsed · ~2m40s left · native 1.3/4.0 GiB`).
4. If the config is missing, write a template, log what to do, and stop the server. The template
   is valid JSON (gap 21): its explanations are `_`-prefixed keys, which the reader ignores, and its
   `preset` and `timeline` are empty, so as written it starts nothing.

*As built in P1-13:* `WorldDataBinder`'s one HIGHEST listener calls `FirstStartGenerator.prepare`
before it binds (so the order never depends on two listeners of one priority). The session writes
into `<world>/deeptime.generating/`, which is renamed to `<world>/deeptime/` once its manifest
exists; a Ctrl-C during the run cancels it. The progress line adds the run's total
(`dummy_drift 84% (−580 Ma) · total 42% · 5s elapsed · ~7s left · native 0.0/4.0 GiB`). Natives
that did not load stop the server with their guidance block and the equivalent
`deeptime run --preset … --start … --length-myr … --out <world>/deeptime` command. Every stop
(`server/ServerStop`) leaves `initServer` right after the event (`DedicatedServerFirstStartMixin`),
skips `ServerStartedEvent`/`ServerStoppingEvent` and `saveAllChunks`
(`MinecraftServerFirstStartMixin`), writes no crash report, and exits with status 1. The world
folder then holds only what vanilla wrote before any mod event: `level.dat` (written by
`net.minecraft.server.Main` before the server thread starts, so it already records the
level-type: after a circumference mismatch the folder must be deleted, not only `level-type`
changed), `session.lock` and `serverconfig/`.

`server.properties` needs `level-type=deeptime\:planet` (16,384) or `level-type=deeptime\:planet_32k`
(32,768) before the first start (`level-type` is resolved as a world-preset key; `generator-settings`
is only read for `minecraft:flat`, so Deep Time's parameters cannot travel there). This step lands in
P1-13; until then (Phase 0) the binder (`server/WorldDataBinder`, §7.1) only binds world data that is
already in the world folder, or the analytic test terrain.

### 7.14 COE and CDG compat mixins

In `compat/coe` and `compat/cdg`, applied by `DeepTimeMixinPlugin` only when the target mod is
loaded, with `defaultRequire: 1` as in mic-climate (a signature drift then fails the boot loudly
instead of doing nothing). Signatures from `javap` and cfr on the current pack's jars (COE 1.6.8,
CDG 1.3.15):

- **Create Ore Excavation 1.6.8.** `public static RecipeHolder<VeinRecipe>
  com.tom.createores.OreVeinGenerator.pick(LevelChunk)` (null = no vein).
  - `OreVeinGeneratorMixin` HEAD (cancellable): in a Deep Time level, if a site covers the chunk and
    its type names a `coe_vein`, return that `RecipeHolder` (from the level's `RecipeManager`).
  - `OreVeinGeneratorMixin` RETURN: if COE picked a vein for a **claimed** material and no site
    covers the chunk, return null, so COE's noise grid cannot hand out claimed metals.
  - `OreDataMixin` on `OreData.populate(LevelChunk)` RETURN: `setRandomMul(q)`, where q is the site's
    tonnage quantile within its type's lognormal. COE computes the amount as
    `round(((max − min) × randomMul + min) × Config.finiteAmountBase)` (`getResourcesRemaining`,
    decompiled; `populate` otherwise sets `randomMul` from a chunk-seeded random float),
    so the vein recipe's own multiplier range scales the yield by the deposit's size. The vein state
    lives in COE's attachment `createoreexcavation:ore_vein`.
- **Create: Diesel Generators 1.3.15.** `public static int
  com.jesz.createdieselgenerators.world.OilChunksSavedData.getBaseOilAmount(ServerLevel, ChunkPos)`.
  `OilChunksSavedDataMixin` HEAD (cancellable): in a Deep Time level return the charge of the
  petroleum-trap site under the chunk, 0 if none. CDG's own first step is a KubeJS override
  (`CDGKubeJSPlugin.calculateOilChunks`, −1 = no override), which Deep Time's HEAD injection runs
  before. MIC's KubeJS scripts do not use that hook (no `calculateOilChunks` anywhere under
  `pack/kubejs/` on 2026-09-28; the CDG hits are recipe scripts), so nothing of MIC's is bypassed.

*As built in P1-16:* the mixins live in `mixin/compat/createoreexcavation/` and
`mixin/compat/createdieselgenerators/` (the plugin gates a compat mixin by its package, which is the
mod id); the decisions are plain Java in `compat/coe/CoeBridge` and `compat/cdg/CdgBridge` (vanilla
types only), COE's and CDG's own calls for probes and tests in `CoeAccess`/`CdgAccess` (their
`*Impl`, loaded reflectively when the mod is present). Signatures re-checked with javap on the MIC
set's jars (sha512 = the pack's pins). **"A site covers the chunk"** = the site's `ore` bodies have
blocks in the chunk's column (`compat/SiteChunks`: body volume per canonical chunk, computed once per
site; the site whose ore fills the chunk most wins). **Vein ids** are read at run time from the
`deeptime:deposit_type` registry (a pack remaps them by overriding the type's file in a datapack),
falling back to the site record's; a site without products has no vein. **Claimed veins** = every
loaded type's `coe_vein` plus those the site records name; the RETURN drops one unless the chunk is
that vein's site. **q** = the site's present tonnage (after erosion and leaching) as a quantile of its
type's split lognormal (`sizing.tonnage_mt`), equal to the core's drawn `tonnage_quantile` for intact
deposits. A third COE mixin, `RandomSpreadGeneratorMixin` (RETURN of the private
`pick(ServerLevel, ChunkPos, RecipeHolder)`), applies the same claimed-vein rule to COE's vein
search (the vein finder's "far" line and `/coe locate`), which walks the placement grid and checks a
chunk's stored vein only when the chunk is loaded. **CDG amount** per chunk = `round(cdg_charge_mb ×
cdg_scale × the chunk's share of the ore body's volume)`, infinite (`Integer.MAX_VALUE`) above CDG's
infinite threshold as in CDG's own generator; `cdg_scale` is a new `minecraft` key of the deposit
type (default 1; `petroleum_trap` 0.02, judged: a median trap gives chunks of a few million mB, the
cores of large traps reach CDG's infinite threshold). `compat/CompatHooks` loads the four target
classes while the mod is constructed, so a drifted signature fails the boot rather than the first
drill. Probes: `/deeptime probe coe|cdg <chunkX> <chunkZ>` (Deep Time's decision, the mod's stored
state, `agrees`); `sites near` lists each site's `ore_chunk`.

### 7.15 Networking: required on both sides

`RegisterPayloadHandlersEvent` → `event.registrar("1")` → `playToClient(WorldInfoPayload.TYPE,
WorldInfoPayload.STREAM_CODEC, ClientHandlers::onWorldInfo)`. Payloads are required unless
`.optional()` is called, and NeoForge's `NetworkComponentNegotiator` refuses a connection with a
missing required channel, so a client without Deep Time cannot join. `WorldInfoPayload` (sent at
login) carries the format version, circumference, scales, cap latitude and planet name, which the
client uses for the pole storm, an F3 latitude/longitude line and, later, maps. `neoforge.mods.toml`
declares `minecraft`, `neoforge` and `toroidal_world` as required with `side = "BOTH"`; COE, CDG,
TerraBlender, Excavated Variants, Sable and Distant Horizons as optional.

*As built in P1-18:* `net/DeepTimeNetwork` registers two required play-to-client channels,
version "1": `deeptime:world_info` (`WorldInfoPayload`: planet flag, dimension, format version,
C, m per block h/v, cap and stop latitudes, planet name; sent on login, dimension change and
respawn) and `deeptime:pole_storm` (`PoleStormPayload`: storm 0…1 and the client's wind). NeoForge
has no public view of its registrations, so `DeepTimeNetwork.registrations()` reads
`NetworkRegistry.PAYLOAD_REGISTRATIONS` by reflection (`/deeptime probe network`);
`NetworkGameTests.worldInfoChannelRequired` feeds it to `NetworkComponentNegotiator`, and the
planet harness's `--scenario refuse_client` logs in over TCP as a NeoForge client without Deep
Time (refused, naming exactly the two channels), with every channel (accepted) and as a vanilla
client (refused). The F3 line: `Deep Time: 76.904° N 90.000° E · <planet> (C 16384, 300 m/block)`.

### 7.16 Commands for tests and debugging

`/deeptime probe column <x> <z>` (surface y, water level, climate parameters, biome, Köppen,
province, the rock run list), `probe site <x> <y> <z>`, `probe coe <chunkX> <chunkZ>`,
`probe cdg <chunkX> <chunkZ>`, `probe natives`, `probe tw` (generator class, stamped, loop axes,
width, climate compression), `probe sable` (sub-level ids and block counts; only with Sable),
`probe palette` (every rock class → resolved block state, for `deeptime-inspect rock`),
`sites near <x> <z> <radius>`, `why <siteId>`, `info` (the manifest). Each prints one JSON line
prefixed `[deeptime-probe]`, which the smoke scenario and `gametest-planet.sh` parse — the same
pattern as mic-climate's `/mic_climate probe`. Permission level 2.

---

## 8. Engine stages

Each stage is a module of `deeptime-core` with a `Stage` implementation:

```rust
pub trait Stage: Send {
    fn name(&self) -> &'static str;
    fn estimate(&self, p: &Preset) -> StageEstimate;               // bytes and a cost model
    fn step(&mut self, w: &mut WorldState, until: Instant, pool: &ThreadPool) -> StepResult; // resumable
    fn checkpoint(&self, w: &WorldState) -> Checkpoint;            // for `deeptime stage …`
    fn keyframe_layers(&self) -> &[LayerId];
}
```

`Session` runs the stages in order: plates and deep-time surface processes interleaved per plate
step, then climate snapshots, young landscape, hydrology, structure, deposits, fantasy, bake. Every
stage emits events to the log and keyframes at a preset interval.

**Licence stance for every stage.** Algorithms come from papers or from MIT/BSD/Apache/CC BY code,
ported with attribution recorded in `NOTICE.md` and in the module's doc comment. GPL code
(fastscapelib, goSPL, RichDEM, OpenLEM, pyGPlates, pynoddy) is never read line by line for porting;
where a GPL code documents an algorithm (goSPL's technical guide), Deep Time implements from the
documentation's equations. GPL tools are oracles in `tools/validation/` only. Research-only code
(Inria's analytical-terrains, FastFlow, glacial-erosion) and unlicensed repositories (the Cortial
community ports) are not opened for code.

### 8.1 Plates and crust — `core::plates` (Phase 1)

- **Source.** Cortial, Peytavie, Galin & Guérin 2019, "Procedural Tectonic Planets" (paper; no
  licensed code exists). Rock-mass pools per sample from tectonics.js `RockColumn`/`Crust`
  (CC BY 4.0, port with attribution). Finite rotations and velocity v = ω × r are standard kinematics
  (GPlates concepts, no code).
- **State per L0 sample** (Cortial's Table 1 plus additions): crust type, thickness, elevation,
  oceanic age, ridge direction, orogeny type and age, fold direction, plate id, rock pools
  (sediment, sedimentary, metamorphic, felsic/mafic × plutonic/volcanic), and per-sample accumulators
  the deposit stage reads (arc magma flux by type, magma oxidation proxy, convergence rate and
  obliquity at the nearest trench, slab age, heat-flow proxy).
- **Processes** (Cortial et al. 2019, as summarised in `tectonics_engines.md` Q3): subduction (the
  older oceanic plate sinks; uplift
  u = u₀·f(distance to front)·g(relative speed)·h(elevation)), continental collision with terrane
  transfer when interpenetration exceeds 300 km, Poisson-triggered rifting into 2–4 plates, oceanic
  crust generation between diverging plates with global resampling every 10–60 steps, continental
  erosion, oceanic damping and trench sediment accretion every step. Hotspots are added as
  mantle-fixed points (Cortial's own suggested extension). Constants from Cortial's Appendix A:
  2 Myr step, subduction distance 1,800 km, collision distance 4,200 km, 100 mm/yr speed cap,
  damping 0.04 mm/yr, erosion 0.03 mm/yr, accretion 0.3 mm/yr, subduction uplift 0.6 mm/yr, and the
  elevation envelope of §5.1.
- **Scaling to a small planet: Earth-normalized** (decisions file, "Tectonics scaling"). The plate
  model runs as if on Earth — R = 6,370 km, Cortial's constants unchanged, Earth-like plate count,
  speeds and ages — on the unit sphere, and its results map onto the smaller planet by latitude and
  longitude, so every tectonic length shrinks by R/R⊕ (782/6,370 = 0.123 at 16k, 1,565/6,370 = 0.246
  at 32k). Elevations and uplift rates keep their physical values (they are vertical, and the vertical
  scale is 20 m per block everywhere). Surface physics — erosion, rivers, slopes, climate — runs in
  real metres at map scale (§8.2). Without the normalisation, Earth's large-plate size split near
  10^7.6 km² (≈ 40 M km², Mallard et al. 2016) would exceed the whole 32k planet (30.8 M km²).
- **Parallelism.** Per-sample updates and the elevation pass are data-parallel over samples; plate
  bookkeeping (rifting, collisions) is serial per event.
- **Cost model.** Cortial Table 2 (C++, i7-6700K, 2016): regular step 0.28 s at 100k samples and
  1.24 s at 250k; oceanic crust generation 1.22 s / 3.58 s; rifting 0.21 s / 1.50 s; generation and
  rifting every 20–120 Myr. For a 500 Myr timeline (250 steps): 100k → 70 s + 4–25 events × 1.43 s
  = 76–106 s; 250k → 310 s + 4–25 × 5.08 s = 330–437 s. Deep Time's timelines are chosen by the
  player, so the estimate is computed per preset, and P1-17 replaces these figures with measurements.
  **Measured (P1-03, M6 Mac mini, release, 500 Myr, shared with other builds):** 100k → 8.3 s on
  12 threads (10.5 s on 4, 23 s on 1); 250k → 25–31 s on 12 threads, 72 s on 1 (STATUS.md, P1-03).
- **Outputs.** Per-step uplift/subsidence rate, crust type, orogeny type/age, fold trend and regime
  sampled to L1 (§5.2); plate/rift/collision/arc/hotspot events; keyframes of elevation, plates and
  crust age.
- **Checks.** Unit and property tests (plate area conserved across rifting; crust ages non-negative;
  no sample in two plates); statistical targets from the research (oceanic crust mostly younger than
  about 250 Myr, a few large plates plus small ones); a viewer page per run.

### 8.2 Erosion, deposition, diffusion and amplification — `core::surface` (Phase 1)

- **Sources.** Braun & Willett 2013 (O(n) implicit stream-power law); Yuan et al. 2019
  (erosion–deposition term G); Barnes et al. 2014 priority-flood for depressions (paper; RichDEM is
  GPL-3.0 and `Barnes2013-Depressions` has no licence file, so implement from the paper), with
  Cordonnier et al. 2019's basin-graph resolver as the optimisation if priority-flood dominates;
  implicit/ADI hillslope diffusion; Landlab (MIT) `ExponentialWeatherer`, `LithoLayers`,
  `ThresholdEroder`, `SimpleSubmarineDiffuser`, `CarbonateProducer` (Bosscher–Schlager
  `G = Gm·tanh(I₀e^(−kz)/Ik)`), ported with attribution.
- **Deep time on L1**, coupled to the plate step: uplift from §8.1, erodibility from the current
  surface lithology, runoff from the latest climate snapshot. The strata log records every deposit
  as (age, thickness, class, provenance) with the research's facies rules (a proposal in
  `surface_processes_climate.md` §4, not a published scheme): coarse fluvial → sandstone or
  conglomerate, offshore or lacustrine fines → shale or mudstone, photic-zone carbonate → limestone,
  arid closed basin → evaporite, humid subsiding swamp → coal, duricrust flag → laterite. Adjacent
  layers of the same class merge; K caps the log (§5.7.5).
  *As built in P1-04* (`crates/deeptime-core/src/surface/`; STATUS.md has the details): the surface
  stage is coupled to the plates stage (`Stage::coupled_to`; the session steps plates, then surface,
  per plate step; `tectonics.l1_every_steps` = 1). Its elevation is the plate model's L1 elevation
  plus an offset advected with the plates (semi-Lagrangian, R(−Ω·Δt)), and its net change flows back
  onto the L0 samples at the next plate step, so uplift is exactly the plate model's. D8 routing
  (the O(n) implicit solve needs one receiver), Priority-Flood+ε with lakes, Braun–Willett + Yuan
  (n = 2, m = 1 by default), a critical slope, mouth-fill delivery to the sea, Landlab's marine
  diffusivity with 2D implicit diffusion (conjugate gradients), carbonates, evaporites, weathering
  with laterites, local isostasy smoothed over 50 km, and the deposit engine's frames every 10 Myr.
  Runoff comes from `climate/{t_mean_c,precip_mm}` when P1-05 writes them, else a zonal stand-in.
- **Young landscape on L2** for the last 2–10 Myr at finer resolution, starting from L1 upsampled.
- **Amplification to L3.** Schott et al. 2024, "Terrain Amplification using Multi-scale Erosion"
  (ACM TOG 43(4)); its code, `H-Schott/MultiScaleErosion` (MIT, commit 964a3d5), is GLSL compute
  shaders needing OpenGL 4.3. Deep Time ports the passes to CPU Rust (MIT permits this with
  attribution): alternate stream-power, thermal and deposition passes with ×2 upsampling from L2 to
  L3. The **drama slider** (0–2, default 1) scales the thermal critical slope and the amplitude of the
  chunk-time detail term, which `deeptime:terrain` adds below L3 resolution: a periodic noise whose
  amplitude is the local L3 relief × drama, damped along channels (a baked `detail/amplitude` layer)
  so valley floors and rivers stay smooth.
- **Parallelism.** Routing and priority-flood by tiles (Barnes 2016's tile/spillover approach, from
  the paper); the implicit stream-power solve in parallel across independent drainage basins after
  routing; diffusion by red-black or ADI sweeps.
- **Cost model.** About 1 µs per cell per step single-threaded for implicit stream power with
  depression handling (derived in `surface_processes_climate.md` §1 from Tzathas 2024 and FastFlow),
  and an assumed, unmeasured 3–4× on 8 threads. Deep time coupled at 2 Myr over 500 Myr (250 steps):
  L1 0.52 M cells → 131 s single-threaded (33–44 s parallel); L1 2.1 M → 524 s (131–175 s). Young
  landscape, 100 steps: L2 2.1 M → 210 s (52–70 s); L2 8.4 M → 839 s (210–280 s). Amplification has
  no published timing (research gap); P1-17 measures it.
- **As built in P1-06** (`surface/{young.rs, amplify.rs, amplify/{window, resample, kernels, tiles}.rs,
  thermal.rs, detail.rs}`; STATUS.md has the numbers). Two more stages follow the deposits in a
  planet preset with the sections `young` and `amplify` (both in the four `pipeline_*` presets;
  every key optional): `young` upsamples the final L1 surface to L2 (clamped Catmull–Rom), fills
  the depressions the interpolation made (under 5 m; deeper ones are L1's lakes) and runs 4 LEM
  steps of the deep-time physics (`spl`, `pflood`, `routing`, `diffusion`) for 2 Myr with uplift
  equal to the last step's erosion rate (`balance`; `plates` and `none` exist), without advancing
  the clock; `amplify` doubles L2 to L3 in `doublings(l2_r, l3_r)` levels (2 in every built-in
  preset but 16k full). Each level: Catmull–Rom terrain clamped between its two nearest samples,
  the level below's exact drainage area as a prior (log-interpolated, less the coarse cell's own
  area), lakes inherited and grown a cell; relief-seeding noise; a Priority-Flood+ε repair; 40
  iterations (×3 on the levels before the last) of multiple-flow drainage area (flow exponent 2,
  Schott's `flow_p` is 1.3) and stream-power incision (`incision_m` metres at 100 km² and slope 0.1,
  area term capped at 300 km², at most 30 % of the drop to the receiver per iteration and 3 × the
  initial drop to the lowest neighbour in all); 24 iterations of thermal slumping with the critical
  slope 0.6 × (0.25 + 0.75 · drama) and ±25 % noise; 12 of deposition of the eroded material along
  the same weights (capped at half the gap to the lowest higher neighbour, so it cannot make a
  pit); the flood again. **Every pass is a Jacobi iteration on a window**, so the last level runs in
  tiles of 2,048 cells with a halo of `radius + 2 · repair_margin + 3` = 196 cells and gives the
  same bits as the whole raster (tests, and 512 of 512 tiles of both L3 layers of the 500 Myr
  planet: one thread, twelve, streamed); it spills `rasters/elevation.dtr` (i16, 1/16 block) and
  `rasters/detail/amplitude.dtr` (u8, 1/16 block) tile by tile (`TilePackWriter`) into the run
  folder, `finalize_write` lists them in the manifest, and a thinned copy (`amplify/{elevation,
  detail, drainage}` on the `L3p` grid) feeds the keyframes and the workbench. The stream is
  `auto`: the levels stream when the gauge says their buffers would not fit under the cap. **Drama**
  (`surface.drama`, the editor's slider, 0..2) scales the critical slope and the detail amplitude
  (`detail.rs`, FORMAT.md defines the C-periodic noise the game adds). Deviations from the plan:
  the working set is 46–50 B a cell (67 B while a tile runs) against the plan's 20, so 16k quick
  needs 2.1 GiB whole; the halo is derived from the iteration counts (196 cells) not fixed at 64;
  the flood is the one non-local pass (its reach, `repair_margin_cells` 32, is in the halo);
  drama maps to 0.25 + 0.75 · drama times the base slope, not drama times it, so 0 is smooth and
  not flat; the young stage adds erosion time without advancing the clock; the fine drainage area
  refines the coarse level's exact area instead of starting from zero.
- **Oracle.** `tools/validation/fastscape_compare.py` runs fastscapelib (GPL-3.0, installed with
  `nix shell` + pip in a throwaway venv, never vendored) on the same uplift and parameters on a
  256² grid and compares hypsometry, χ/steepness statistics and drainage-area distributions with the
  CLI's output; tolerances are set when the test is first calibrated and recorded in the test.

### 8.3 Climate — `core::climate` (Phase 1)

Parametric, per snapshot (every 10 Myr in deep time by default, plus the present), on L1:

1. **Zonal energy balance** with ice-albedo feedback: climlab's diffusive EBM equations (MIT; port
   the equations), forced by insolation from latitude, `axial_tilt_deg` and `star_flux_rel`, run
   seasonally to monthly temperatures.
2. **Circulation belts**: three-cell wind belts and the ITCZ; wet belts near 0° and 60°, dry near 30°
   and the poles (the Worldbuilding Pasta rules of thumb in `surface_processes_climate.md` §5).
3. **Continentality and lapse rate** on the DEM.
4. **Orographic precipitation**: Smith & Barstad 2004 linear theory, porting the MIT
   `fastscape-lem/orographic-precipitation` module (96 lines, FFT-based; `rustfft`, MIT/Apache),
   run per latitude band along that band's prevailing wind. LFPM (Hergarten & Robl 2022) is the
   alternative if FFT on a wrapped raster misbehaves; its implementation (OpenLEM) is GPL, so from the
   paper only.
5. **Köppen classification** from 12 monthly temperatures and precipitations (the standard rules).
6. **Glaciers**: an equilibrium-line rule ELA(x) = z(x) + (T(x) − T_ELA)/Γ, as goSPL's documentation
   states it (implemented from the docs).
7. **Sea level**: a curve from the preset.

Cost: LFPM-class precipitation costs about 2.4 stream-power steps per snapshot, so a snapshot costs
about one or two erosion steps (`surface_processes_climate.md` §5). Outputs feed erosion (runoff),
deposits (paleolatitude, aridity), biomes (§7.2) and keyframes.

*As built in P1-05* (`crates/deeptime-core/src/climate/`; STATUS.md has the details): the climate is
a stage coupled to the plates like the surface, placed before it (`plates → climate → surface →
structure`, one step each per plate step), so the surface's runoff, weathering and facies, the
structure's metamorphism and the deposit frames read `climate/{t_mean_c, precip_mm}` of the ground
they are about to change. Full snapshots every `snapshot_every_myr` (10) plus the first and last
step; between them each cell's departure from its row's zonal reference moves with the plates
(semi-Lagrangian, as the surface's offset) and takes the current lapse rate. (1) The EBM is climlab's
equation with North, Cahalan & Coakley 1981's A, B, D and climlab's albedos, extended with a land
and a sea surface per band (the sea a surface layer over a 50-m slab with latent sea ice and ocean
heat diffusion), 72 implicit steps a year, a circular orbit; energy balanced to < 0.05 W/m². (2)
Belts: three cells shifted per column and month by a thermal-equator ITCZ (monsoons) and a
tilt-scaled shift of the highs; gyres from each ocean cell's position across its basin; drying
under the highs' eastern flanks, summer onshore air on subtropical east coasts. (3) Moisture,
maritime influence and rain shadows carried along each row with the wind (Earth-normalized decay
lengths), smoothed over 300 Earth km. (4) Smith & Barstad via the MIT module, FFT per wind regime
over the rows it reaches (x wraps; rows zero-padded). (5) Köppen from 12 months (Peel et al.
2007). (6) ELA from summer temperature and precipitation (Ohmura et al. 1992); `glaciation`
events. (7) Sea level stays the surface section's curve (P1-04). Monthly fields stay internal;
the published fields are the annual mean, warmest and coldest month, precipitation, aridity,
Köppen, ELA and glacier. `deeptime climate-dem` runs one snapshot on a given relief (Earth's
1° relief: 19 of 24 reference cities in the right Köppen group).

### 8.4 Hydrology — `core::hydro` (Phase 2; Phase 0 only proves water holds)

On L2: flow routing and priority-flood; river extraction as vectors with discharge above a drainage
threshold; lake levels with Fill–Spill–Merge (Barnes, Callaghan & Wickert 2021; MIT code
`r-barnes/Barnes2020-FillSpillMerge` plus `r-barnes/Barnes2019-DepressionHierarchy`, both MIT —
port with attribution) using climate runoff; floodplains with HAND (Landlab
`HeightAboveDrainageCalculator`, MIT); channel geometry from discharge by hydraulic geometry
(coefficients to be sourced in P2 — the research found none); waterfalls where rivers cross
erodibility knickpoints. Baked outputs: `water_level` (L3), `rivers/segments.dtb` (polylines with
width, depth, discharge; widths × `feature_scale.rivers`), and channel incision burned into
`elevation`. Cost: 1–4 M cells gives about 1–6 s per solve by linear scaling of Cordonnier 2019's
16,384² figure (`surface_processes_climate.md` §6).

### 8.5 3D structure, Noddy-style — `core::structure` (Phase 1 simple, Phase 2 full)

- **Phase 1 (column strata).** Horizon stacks come straight from the strata log, with metamorphism
  by regime history (collision zones step shale → slate → phyllite → schist → gneiss along each
  class's `metamorphic_path`) and plutons as simple bodies under arcs. No folds or faults yet.
  *As built in P1-07* (`crates/deeptime-core/src/structure/`; STATUS.md has the details): a stage
  `structure`, the third member of the coupled group (plates, surface, structure, one step each per
  plate step). **Metamorphism** steps every strata-log layer along its class's `metamorphic_path` as
  far as the temperature at its mid-depth allows — a linear geotherm from the plate model's heat flow,
  with depths scaled by the crust's thickening (e/35 km, at most ×2) under an active collision — in
  the surface's log itself, so it is irreversible and metamorphic rocks erode as what they became;
  the formation temperatures are rounded textbook facies boundaries (slate 250 °C … gneiss 650 °C;
  no high-pressure series, which waits for Q8/P2-02). **Plutons** are a Poisson process under arcs
  (and collision zones) at 2–6 km depth, sized by T = 0.6·L^0.6 (Cruden 1998); they ride their
  plate, are unroofed by the surface's erosion and end as ellipsoid bodies (role `pluton`) with a
  hornfels or skarn aureole zone. **Columns** are regolith, the strata log (the young, shallow part
  of a clastic deposit shown loose through the new `loose_equivalent`; buried loose material and
  palaeosols lithified through `lithifies_to`), the plate model's rock pools less the log (the
  surface's basement rule per pool) and peridotite. The **bake** turns a column into at most K
  horizons at least a block thick. The deposit engine's frames get `trap_structure` (fold-thrust
  belts, salt domes, rift fault blocks, growth faults, drape) and `host/<class>` from the columns.
- **Phase 2 (kinematic history).** Source: Noddy's inverse-kinematics evaluation (Jessell et al.
  2022; Wellmann et al. 2016); code: Loop3D `noddyverse` and `noddy_qt` (MIT) — port `unEvents.c`
  (`unfold`, `unfault`, `undyke`, `unplug`, `unbury`, `reverseEvents`), `taste.c` and `matrix.c`
  (2,024 lines of C in total per the report's ledger). Plate events become local events per L1
  region, as the research proposes: collision → folds, thrusts, tilt, metamorphism; rift → normal
  faults, tilted blocks, unconformity, syn-rift sediment; arc → plugs, dykes, volcanic strata;
  hotspot → plug and dyke swarm, lava strata; basin → STRAT/UNC packages; erosion surfaces →
  unconformities. The evaluator runs at bake time on a lattice (L2 columns × a vertical sample every
  few blocks), never per block at chunk time: Noddy's upper bound of about 1.9 µs per voxel would be
  0.19 s per 16×16×384 column (report), several times a chunk's budget. The lattice result is fitted
  back into horizon stacks (folds and thrust repeats become extra horizons) and bodies. Metamorphic
  grade comes from each point's restored burial and heat history.
- **Check.** The P2 exit review compares cliffs in BlueMap renders and cross-sections (§9.7) with
  the event log's folds and faults.

### 8.6 Deposits — `core::deposits` (Phase 1: four families)

The mineral-systems approach (Skirrow et al. 2019; McCuaig et al. 2010; Hronsky & Groves 2008) as a
rule engine over simulated fields, then the USGS three-part method (Singer 1993; Root, Menzie & Scott
1992) for counts and sizes, then a preservation ledger (Kesler & Wilkinson 2006, 2008). No code
exists to reuse; this is Deep Time's own engine.

1. **Favourability.** For each deposit type, each component (source, driver, pathway, trap) is a
   product of piecewise-linear membership curves over named fields (§5.4.3), combined by product so a
   missing process zeroes the score, integrated over the type's formation window and gated by
   secular clocks (land plants for coal and sandstone uranium; oxygenated oceans after 2.02 Ga for
   MVT and SEDEX; the 2.65–1.85 Ga BIF acme). The limiting component is recorded per cell.
2. **Tracts and counts.** Connected regions with favourability above a threshold are permissive
   tracts. N ~ Poisson(ρ · A_tract · mean F · count_mult), with ρ from USGS densities (porphyry P50 15
   per 100,000 km², Singer et al. 2005). At Deep Time's planet sizes real densities give playable
   counts without inflation: a 32k planet's 29.7 M km² between the caps with 1% permissive arc area
   gives 29.7 M × 0.01 × 15 / 100,000 ≈ 45 porphyries; 16k gives about 11. `count_mult` defaults to
   1 and is a tuning lever.
3. **Positions.** A Thomas cluster process with the USGS same-deposit separation (2 km for porphyry
   Cu), weighted by favourability; `province` strength (default `strong`) concentrates types into
   metallogenic provinces, which is what makes players build bases in different regions.
4. **Sizes and grades.** Lognormal tonnage with μ = ln P50 and σ = (ln P10 − ln P90)/2.563 (porphyry:
   33 / 240 / 1,500 Mt → σ ≈ 1.49, USGS OFR 2008-1155); grades by Gaussian copula with published
   correlations (porphyry Au vs tonnage r = −0.28). Grade–tonnage tables ship as JSON from
   public-domain USGS data. MVT, SEDEX, coal and placer percentiles were not extracted by the
   research; P1-14 digitises them from the cited USGS reports.
5. **Geometry.** Zoned bodies (porphyry: potassic core, phyllic, propylitic halo; lithocap above;
   gossan when exposed), sized from tonnage and enlarged by `size_mult_h` / `size_mult_v`: a median
   porphyry (240 Mt at 2.6 t/m³ ≈ 9.2 × 10⁷ m³, a 452 m cube) is 1.5 blocks wide at 300 m per block
   and 22.6 blocks tall at 20 m per block, so the horizontal multiplier does most of the work. The
   defaults are set in P1-14 by review of the renders.
6. **Preservation ledger.** Formation depth and time, then burial and erosion from §8.2 at the
   deposit's column (exhumation rates of 158 m/Myr for porphyry, 167 for epithermal, 63 for orogenic
   gold, Kesler & Wilkinson 2006). Fully eroded → spawn placer children downstream along the drainage
   routing of §8.2 (receivers on L1/L2; river vectors arrive in Phase 2): gold and cassiterite travel
   a few km, diamond up to 1,000 km. Partly eroded → truncate. Top within a few hundred metres, with
   the water table lowered, an eligible climate and 0.5–9 Myr of stability → supergene blanket at
   2–3× grade (Sillitoe 2005).
7. **The four Phase 1 families.**
   - Porphyry Cu-Mo-Au with supergene blanket and placer-Au children; porphyry favourability peaks
     near 100 km/Myr convergence, 15° obliquity and 25–70 Myr slab age (Butterworth et al. 2016).
   - Iron: banded iron formation (inherited cratonic basement where the timeline starts after the
     acme, Superior-type if it spans it) and Fe skarn; coal seams from coal facies in the strata log,
     ranked by burial.
   - Zn–Pb: MVT (orogenic uplift beside a carbonate platform with evaporite brines) and SEDEX
     (euxinic rift or sag basin within 200 km of a thick–thin lithosphere step, Hoggard et al. 2020).
   - Petroleum systems: Lopatin TTI = Σ Δtₙ·2ⁿ per source-rock layer (oil from TTI ≈ 15, peak ≈ 75,
     Waples 1980), traps from structure (anticlines, salt domes) and seals, charge in order → reservoir
     sites for CDG.
8. **Playability pass** (§7.8), then the site registry and the events.

### 8.7 Fantasy processes — `core::fantasy` (Phase 3)

- **Fantasy fields** (§5.4.2) are computed on L1 alongside real fields each step, so fantasy
  deposits use the same rule engine with `requires_fields`.
- **Sky islands** are simulated, not overlaid: where `deeptime:aether_flux` exceeds its threshold
  during plume or large-igneous-province events, crust blocks detach, rise to an altitude band below
  the 639 ceiling, drift with their source plate and erode as small bodies. They keep their source
  lithology, so islands have real strata and can carry deposits. Output: sky-island bodies (top
  surface, underside, AABB) for §7.4.
- **Exotic landforms** (arches, spires, crystal fields) as implicit 3D primitives placed by fields,
  following Paris et al. 2019 "Terrain Amplification with Implicit 3D Features" (MIT code,
  `aparis69/Implicit-Volumetric-Terrains`).
- Phase 3's own planning pass fixes the remaining parameters; this plan only commits to the
  mechanism.

### 8.8 Bake — `core::bake`

Converts physical results to block space: an elevation of h metres relative to sea level becomes
y = 63 + h/20 (clamped to a floor band above −352 and to 639 at the top); horizons get a minimum
thickness of one block after scaling; then feature multipliers, bodies in block coordinates, water
levels, climate layers, detail amplitude, province ids and spawn candidates; then writes tile packs,
strata, bodies, sites, events and the manifest last (§6.3). `sampleColumn` (§6.1) and the CLI's
`deeptime inspect-column` call the same column evaluator Java mirrors.

---

## 9. Review and test pipeline

Ben cannot launch the game, so every claim in this project has to be something an agent can run and
something Ben can look at in a browser.

### 9.1 Principles

- Every task's acceptance criteria are commands with expected output (§10). A task is not done
  until they pass and STATUS.md records the result (§12).
- Every phase exit publishes one private spacesheep page that links everything a reviewer needs.
- Performance targets (about 5 min quick, 15 min full on a 6–8-core PC) are checked on this
  4-core laptop. Meeting the target here is a pass. Missing it here is reported with the per-stage
  breakdown and the measured scaling across thread counts (1, 2, 4, 8), and the decision is Ben's;
  it is not extrapolated to the target machine.
- Anything that needs a live client is listed in §9.8, not assumed.

### 9.2 Rust tests, parity tests and oracles

- **Unit and property tests** in every crate (`cargo test --workspace`), with `proptest` for
  invariants: plate area conserved across rifting, mass conserved by diffusion, no cycles in
  receivers, lake levels never above their spill rim, tile packs round-trip, share codes round-trip.
- **Golden outputs.** Small fixed presets (a 64k-sample planet, 256×128 rasters) produce checked-in
  summary statistics (hypsometry bins, plate count, crust-age histogram, deposit counts), compared
  with tolerances. Because determinism is "stored data only", goldens compare statistics, not bytes.
- **Java↔Rust parity.** The CLI writes `parity/columns.json` (500 random columns: surface y, water
  level, the class run list, bodies hit) from a synthetic world; the JUnit test `ColumnParityTest`
  evaluates the same columns through the Java reader and asserts equality. The same file drives a
  GameTest that compares placed blocks with the expected classes. `sampleColumn` (§6.1) regenerates
  it.
- **Oracle tests (unshipped).** `tools/validation/` holds Python scripts that install GPL tools into
  a throwaway environment (`nix shell nixpkgs#python312 -c python -m venv …`, then pip) and compare
  numbers with CLI output: fastscapelib for erosion (§8.2), goSPL only for its documented behaviours
  if a comparison is worth its HPC cost. They are run by hand or by the phase exit script, never by
  CI on shipped code, and their output is numbers and plots, not code.

### 9.3 The CLI and the HTML viewer

- `deeptime run --preset <p> --data <jar|dir>… --out <dir> --viewer <dir>` — a whole planet.
- `deeptime stage <name> --in <checkpoint> --out <checkpoint> --viewer <dir>` — one stage.
- `deeptime synth --kind <wrap-test|lake-test|strata-test> --size <C> --out <dir>` — synthetic
  worlds for Phase 0 and GameTests.
- `deeptime bench --preset <p> --threads 1,2,4,8 --out bench.json --viewer <dir>` — timings and
  peak memory.
- `deeptime inspect-column --world-data <dir> <x> <z>` — the column evaluator.
- The viewer bundle (`index.html`, `viewer.js`, `ramps.json`, `layers/*.png`, `keyframes/*.png`,
  `events.json`, `sites.json`) shows: layer toggles; the timelapse with scrubber and event ticker;
  click-to-inspect (lat/lon, block x/z, every layer value, Köppen, province, the column's rock
  stack); sites with their "Why here?" cause chain; and a globe view. No build step, no npm: plain
  JS in the page.

### 9.4 NeoForge GameTests

Two runtimes. NeoForge enables `/test` in a normal server only outside production
(`GameTestHooks.isGametestEnabled()` = not production and (in IDE, or the GameTest server, or
`-Dneoforge.enableGameTest=true`)), and the GameTest server has two properties that shape how Deep Time
uses it (both read in the 1.21.1 sources):

- **It builds only the flat overworld.** `GameTestServer.create` bakes the flat preset against a new,
  empty, frozen `MappedRegistry<LevelStem>` (`.bake(registry)`), and `WorldDimensions.bake` keeps only
  the stems of the registry it is given plus the preset's own. A dedicated server instead passes
  `datapackDimensions().registryOrThrow(Registries.LEVEL_STEM)` (`Main`). So a datapack dimension
  loads as data but never becomes a level in a GameTest run.
- **It reuses its world folder** between runs, so chunks generated by an older build would be tested
  again.

**The fix (chosen over moving every terrain test to the planet server, because the GameTest server is
the fast, production-like suite):** a gametest-only mixin, `gametest/mixin/GameTestServerMixin`, in its
own config `deeptime.gametest.mixins.json`, which the mixin plugin applies only when
`-Ddeeptime.gametest=true`:

```java
@Mixin(GameTestServer.class)
abstract class GameTestServerMixin {
    // The only WorldDimensions.bake call in GameTestServer is inside create()'s WorldLoader lambda.
    @WrapOperation(method = "*", at = @At(value = "INVOKE",
        target = "Lnet/minecraft/world/level/levelgen/WorldDimensions;bake(Lnet/minecraft/core/Registry;)Lnet/minecraft/world/level/levelgen/WorldDimensions$Complete;"))
    private static WorldDimensions.Complete deeptime$bakeDatapackStems(WorldDimensions dims, Registry<LevelStem> empty,
            Operation<WorldDimensions.Complete> original, @Local(argsOnly = true) WorldLoader.DataLoadContext ctx) {
        return original.call(dims, ctx.datapackDimensions().registryOrThrow(Registries.LEVEL_STEM));
    }
}
```

`WrapOperation`, `Operation` and `@Local(argsOnly = true)` are MixinExtras, which NeoForge 21.1.249
depends on (`mixinextras-neoforge` 0.5.3 in its POM); `WorldLoader.DataLoadContext` has a
`datapackDimensions()` component. The first GameTest of P0-05,
`BootGameTests.testPlanetDimensionExists`, proves the mixin works.

1. **`./gametest.sh`** — ModDevGradle `runGameTestServer` (as in mic-climate), namespace `deeptime`,
   with `-Ddeeptime.gametest=true`. The gametest pack (§7.12) defines level stems in
   `data/deeptime/dimension/`: `test_planet` (the analytic test terrain, C = 16,384), from P0-10 on
   `test_planet_data` (world-data folder `deeptime-test`, C = 1,024) and from P0-13 on `test_lake`
   (world-data folder `deeptime-lake`, C = 1,024). All three use the router variant without the
   noise-cave functions, `deeptime:planet_nocaves`, so the density fill leaves every surface intact;
   the height tests read chunks at NOISE status, before carvers and features (P0-05). `test_lake`
   also uses a fixed biome source with the gametest biome `deeptime:test_bare` (no carvers, no
   features): 1.21.1's `WorldCarver.carveBlock` has no check for neighbouring water, so a vanilla
   cave carver could otherwise open the test basin. Tests run in the flat overworld, as GameTests
   always do, and read terrain by loading chunks of those levels through
   `helper.getLevel().getServer().getLevel(<key>)`. Before starting, the script deletes the test
   levels' region folders (`run-gametest/world/dimensions/deeptime/`) and the old world data
   (`run-gametest/world/deeptime-test/`, `…/deeptime-lake/`), then (from P0-09 on) runs
   `nix develop -c cargo run -p deeptime-cli -- synth --kind wrap-test --size 1024 --out run-gametest/world/deeptime-test`
   and the same with `--kind lake-test --out run-gametest/world/deeptime-lake` (from P0-13 on).
   `spawnOnLand` calls `SpawnHandler`'s candidate selection directly against the test level, because
   `LevelEvent.CreateSpawnPosition` fires only for the GameTest server's flat overworld. Profiles:
   default, `-PwithMic`, `-PwithTerraBlender`, `-PwithEv`. Pass lines (in `GameTestServer`):
   `GAME TESTS COMPLETE` and `All {n} required tests passed :)`; as in mic-climate's script, a run
   that never reached the tests is a failure.
2. **`./gametest-planet.sh`** — a ModDevGradle `planetServer` run (a dev `server()` run, so not
   production) in `run-planet/` with `-Dneoforge.enableGameTest=true
   -Dneoforge.enabledGameTestNamespaces=deeptime`, `level-type=deeptime\:planet` (or `planet_32k`),
   RCON on, Toroidal World, Sable and Create Aeronautics on the runtime classpath, and, for
   `-PwithMic`, the current MIC server set in `run-planet/mods/` (§9.5). Until P0-10 it passes
   `-Ddeeptime.testTerrain=wrap-test`. The script deletes `run-planet/world/`, starts the server, waits
   for `Done (`, then over RCON runs `execute positioned <x> <y> <z> run test runall <testClassName>`
   (or `test runmultiple <testName> <amount>` for repeated trials; both are in 1.21.1's
   `TestCommand`) with the position next to the seam or the cap, parses the results from the log,
   saves, stops, and (for the reload tests) starts again. This is where the seam, Sable and pole tests
   live, because they need a real TW-shaped Deep Time overworld. Flags: `--boot-only` (start, wait for
   `Done (`, run `/deeptime probe column 0 0`, stop), `--size 16384|32768`, `--scenario <name>` (one
   scripted scenario, such as P0-06's `sable_straddle`), `-PwithMic`.
   *As built (P0-06):* tests run by name (`--run name,…@x,y,z[~dx,dz]`), **one at a time**, because
   1.21.1's `/test run` begins with `TestCommand.stopTests()` and cancels any test still running;
   `/test run` ignores y and puts the test on the WORLD_SURFACE heightmap at (x, z + 3), which reads
   the world's floor until that chunk is loaded, so the harness forceloads first and waits for every
   test spot (`execute if loaded`); TW's `/forceload` accepts canonical coordinates only, so a box
   across the seam is split into two. Results come from `DeepTimeTestReporter` (§9.4 gap 1).

GameTest classes by task: `BootGameTests` (P0-03, P0-05), `GeneratorGameTests` (P0-05, P0-10),
`SeamGameTests`, `SableSeamGameTests`, `PoleCapGameTests` (P0-06, P1-18), `NativeGameTests` (P0-07),
`BiomeGameTests` (P0-12, P1-09), `TerraBlenderGameTests` (P0-12, P1-10), `WaterGameTests` (P0-13),
`EditorModelGameTests` (P1-11), `StrataGameTests` (P1-12), `DepositGameTests`,
`ClaimGameTests`, `SpawnGameTests` (P1-15), `CompatGameTests` (P1-16), `NetworkGameTests` (P1-18).
Each logs the numbers it compared on a `[gametest]` line, pass or fail (mic-climate's convention).
Every test uses the 5 × 5 × 5 empty template `data/deeptime/structure/empty_5x5x5.nbt`
(generated by `tools/make_test_structure.py`, adapted from mic-climate's) with
`@PrefixGameTestTemplate(false)`: NeoForge 21.1's GameTest path throws "Missing test structure" on an
empty template name (mic-climate `GameTests.java`).

### 9.5 Headless real-pack smoke test over RCON

`tools/smoke/install.sh <profile> [--size 16384|32768] [--synth <kind>] [--test-terrain <kind>] [--preset <p>]`
and `tools/smoke/run.py --profile <profile> [--fresh] [--level-type <preset>]` (`--fresh` deletes
the world first; `--level-type` overrides the preset for boot-only runs), adapted from mic-climate's
`smoke/install.sh` and `smoke/run.py` (standard library only; tools from `smoke/tools.env`, resolved
once from nixpkgs). Every byte they produce lands under Deep Time's own gitignored
`smoke/<profile>/`; nothing under `pack/` or mic-climate is written or read as a runtime.

- **Server sets (profiles).** Each profile has a pin file, `tools/smoke/profiles/<profile>.txt`:
  - `standalone`: NeoForge (the pack's version) + Deep Time + Toroidal World + Sable + Create
    Aeronautics + Chunky, each downloaded by Modrinth version id (the ids of §4.3; Sable and
    Aeronautics at the versions the pack pins) with its sha512 checked.
  - `mic`: **the current MIC pack**, server side: packwiz-installer-bootstrap `-g -s server
    /home/greencheetah/Projects/mic/pack/pack.toml` into `smoke/mic/server/` (the same method as
    mic-climate's script; packwiz-installer syncs, so a re-run picks up whatever `pack/` pins today),
    then any pack copy of Deep Time is removed and the fresh Deep Time jar and Toroidal World are added.
  - `terrablender`: `standalone` + TerraBlender + Biomes O' Plenty + GlitchCore.
  `smoke/mic/server/mods/` is also the jar set `-PwithMic` uses (§4.3), so a `-PwithMic` gate always
  re-runs `install.sh mic` first and records `git -C /home/greencheetah/Projects/mic/pack rev-parse HEAD`
  (the pack is a git repository) in its STATUS entry.
- **Server config.** `eula.txt` (`eula=true`; Ben accepted the EULA for test servers, §2.7),
  `server.properties` (RCON on with a random password in `smoke/rcon.pw`,
  `level-type=deeptime\:planet` or `deeptime\:planet_32k` from `--size`, `level-name=planet`,
  `max-tick-time=-1`), `user_jvm_args.txt` (`-Xmx6G`, plus `-Ddeeptime.testTerrain=<kind>` when
  `--test-terrain` is given), `config/deeptime/server.json`, and jars listed in
  `tools/smoke/profiles/<profile>.excluded` moved aside, each exclusion recorded.
- **World data before first start.** `--synth <kind>` runs
  `nix develop -c cargo run -p deeptime-cli -- synth --kind <kind> --size <C> --out smoke/<profile>/server/planet/deeptime`
  (from P0-09 on); the binder picks it up at `ServerAboutToStartEvent` (§7.13). `--test-terrain` is the
  Phase 0 alternative before any world data exists (§7.1).
- **Scenario.** Start with `-Dmixin.debug.verbose=true`; fail fast on mixin or crash signatures
  (mic-climate's FATAL regex); time first-start generation from the `[deeptime]` progress lines (from
  P1-13 on); wait for `Done (`; confirm every Deep Time mixin applied by name; run the probes (§7.16)
  and compare them with the world data; `locate structure #minecraft:village`; pregenerate with
  Chunky (`chunky center <x> <z>`, `chunky radius 256`, `chunky start`, poll `chunky progress`);
  check that every `region/r.<rx>.<rz>.mca` file name has −C/1024 ≤ rx < C/1024 (a 512-block region
  file; nothing written outside the canonical span) — as built (P0-06), every region, entities and
  poi file *that stores a chunk*: vanilla creates an empty `.mca` whenever it opens a region file for
  a read, and on a TW world some reads use raw coordinates past the edge (0-byte files at
  rx = C/1024 and −C/1024 − 1), which are recorded, not failed; `save-all flush`; stop; start again and confirm
  the world data was not regenerated; write `smoke/<profile>/report.md` (pass/fail table, boot time,
  generation time, peak RSS, mixins, probes, exclusions, the pack commit).
- The resulting world folder feeds BlueMap (§9.6) and `deeptime-inspect` (§9.7).
- Memory: mic-climate's MIC smoke peaked at 3.43 GB RSS under `-Xmx6G` (its `smoke/report.md`,
  2026-09-06); with about 6 GB free on this laptop, nothing else heavy runs alongside a server.

### 9.6 BlueMap renders with the MIC jars

- **Tool.** BlueMap CLI 5.28 (MIT), `bluemap-5.28-cli.jar` from the GitHub releases (supports
  Minecraft 1.13.2–26.3; needs Java 25 → `nix shell nixpkgs#jdk25`; skip 5.21, which had a
  map-breaking bug). Pinned by sha256 in `tools/bluemap/VERSION`, downloaded into `libs/bluemap/`
  (gitignored).
- **Command.**
  `java -jar bluemap-5.28-cli.jar -c tools/bluemap/out/config -n smoke/mic/server/mods -v 1.21.1 -r -g`
  (`-n` points at a folder of mod jars whose static block models and textures BlueMap loads; `-r`
  renders, `-g` writes the web app; `-f` forces a full re-render and needs `-r`).
- **Config.** `core.conf` with `accept-download: true` — setting it declares acceptance of Mojang's
  EULA and ownership of Java Edition, which Ben confirmed on 2026-09-28 (§2.7);
  `scan-for-mod-resources: true`. `maps/planet.conf`: `world` = the smoke world, `dimension:
  "minecraft:overworld"`, `remove-caves-below-y` lowered for a −352 floor, render bounds around spawn
  and the seam.
- **Known limits.** Blocks drawn by block-entity renderers (Create's shafts, cogwheels, belts) render
  wrong or not at all; worldgen output is mostly static-model rock, so this matters little. Custom
  world height (−352..639) is not confirmed to work (BlueMap issue #517 fixed a crash with inline
  dimension types; current behaviour untested): P0-14 verifies it.
- **As built (P0-14, 2026-09-28).** Storage stays gzip and `render-bluemap.sh` sets the web app's
  `clientDecompression: true`, so BlueMap's loader fetches `.prbm.gz`/`textures.json.gz` and inflates
  them in the browser; no service worker and no Content-Encoding header. spacesheep's Free plan limits
  (200 files per space, 10 MB per file) bound a review render: `BLUEMAP_BOX` picks the area and
  `BLUEMAP_ONLY_PRESENT=1` gives BlueMap only the jars whose blocks occur there.
- **Static hosting (original analysis).** By default BlueMap's file storage writes tiles gzip-compressed (`*.prbm.gz`) and
  expects the web server to send `Content-Encoding: gzip`, and 204 for missing tiles; spacesheep is a
  plain static host. BlueMap 5.28's `storages/file.conf` has a `compression` key whose documented
  values are `gzip` (the default), `zstd`, `deflate` and `none`
  (`common/src/main/resources/de/bluecolored/bluemap/config/storages/file.conf` at tag v5.28), so
  `tools/bluemap/storages/file.conf.tmpl` sets `compression: none`. Only if uncompressed tiles still do
  not load (for example because missing tiles return 404), the page ships a small service worker that
  answers missing tiles with 204.
- **Sources** (checked 2026-09-28): releases and supported versions,
  https://github.com/BlueMap-Minecraft/BlueMap/releases/tag/v5.28; CLI flags,
  https://github.com/BlueMap-Minecraft/BlueMap/blob/master/implementations/cli/src/main/java/de/bluecolored/bluemap/cli/BlueMapCLI.java;
  modded blocks and their limits, https://bluemap.bluecolored.de/wiki/customization/Mods.html;
  `accept-download` and `scan-for-mod-resources`, https://bluemap.bluecolored.de/wiki/configs/Core.html;
  gzip tiles on external hosts, https://bluemap.bluecolored.de/wiki/webserver/ExternalWebserversFile.html;
  inline dimension types, https://github.com/BlueMap-Minecraft/BlueMap/issues/517; licence MIT.

### 9.7 Our own renders from region files

`deeptime-inspect` reads a world's `region/*.mca` (4 KiB sectors; chunk NBT compressed with zlib by
default, LZ4 when `region-file-compression=lz4`; 1.21 sections with `block_states` palettes) and
writes PNGs plus a JSON summary:

- `topdown` — the highest non-air block per column, coloured from a colour table that
  `deeptime-inspect colors --jars <mods dir> --vanilla <client-extra jar>` builds from the jars' own
  top textures (average colour per block).
- `rock` — the rock class of the first solid non-surface block per column (block → class from
  `/deeptime probe palette`'s dump; until that probe exists, from the `#deeptime:rock/<class>` tags
  in the world's jars and datapacks).
- `ore` — ore blocks by material, projected from above with depth shading.
- `section --axis x|z --at <n> --from <a> --to <b>` — a vertical cross-section, blocks by colour,
  with site bodies and horizon lines overlaid from the world data. `--axis z --at <n>` is the plane
  z = n, running along x. It prints `water_top_y=<y> water_block_y=<y−1> columns=<n>` for every
  water surface it crosses: `water_top_y` is the top face of the highest block holding water above
  the terrain surface, as P0-13's `lakeHolds` logs it (sea level 63 gives 63), `water_block_y` that
  block's y (cold-read review 2, gap 18).
- `seam` — the 128 columns on either side of x = C/2 as one strip, and the per-column height
  difference between the region files and the world data.
- `compare --world <dir> --world-data <dir>` — generated surface height vs `elevation`, as a
  histogram and a map (cold-read review 2, gap 18). **Measured** is the y of the highest block that
  is not air, fluid or a feature (vegetation, logs, wood, leaves, roots, huge mushrooms, coral,
  sea pickles, snow layers, sheet ice, …): `OCEAN_FLOOR` minus logs, leaves and features.
  **Expected** is `⌈H⌉ − 1`, H the bilinear `elevation` (the GameTests' expected top). Only `full`
  chunks count, and two kinds of column are left out and counted: `structure` (within 12 blocks of
  a structure piece that reaches the surface) and `carved` (within 2 columns of a pit: at least 2
  blocks below the morphological closing, half-width up to 32, of the measured surface after an
  opening of half-width 4 that removes narrow bumps; the test reads only the measured surface,
  never the world data). The JSON reports `p99_abs_diff_blocks` (nearest rank over |measured −
  expected|), the histogram and the exclusion counts.
- `regions --world <dir> --span <C>` — lists region files and counts any outside the canonical span
  (−C/1024 ≤ rx < C/1024), printing `outside=<n>`; the same check `run.py` does by file name, kept
  here for worlds that did not come from a smoke run. A 0-byte file holds no chunk (the game
  creates it when it merely looks a chunk up) and is counted apart as `outside_empty`.

### 9.8 Publishing and what stays unverifiable

`tools/review/publish.sh <name> <folder>` publishes a folder; the phase exit scripts first assemble
`review/out/<phase>/index.html` (a report table with
pass/fail, then links to the viewer bundle, the BlueMap app, the inspect renders, the smoke report,
the GameTest logs and the bench chart) and deploys it with the §2.6 command. Where the Chrome
extension is available, the agent opens the published page and checks that the viewer and the
BlueMap app actually load before handing Ben the link.

**Not verifiable under this plan** (client-only):

- The timelapse screen in-game: layout, feel and frame rate. Its *content* is verifiable, because the
  viewer draws the same keyframes with the same colour ramps.
- How Distant Horizons' far terrain looks (Phase 3).
- How the seam feels while flying: hull continuity on the client, the hitch, pop-in; and the pole
  storm's visuals.
- Multiplayer client behaviour at the seam (entity tracking between two clients, sound, Simple Voice
  Chat), which needs two real clients.
- Rendering of Create and Aeronautics contraptions at the seam.

Best effort only: mic-climate's `gametest-client.sh` (HeadlessMC under Xvfb) boots a real modded
client into a world. Pointing it at a Deep Time world can catch client-side crashes (the editor
screen, the payload handler), but its GameTest stage has not run yet (mic-climate status
2026-09-05), so it is never an exit gate.

---

## 10. Phase plan

The decisions file's phases: **0** Rust↔JNI, the Toroidal World wrap, the generator on a static
wrapped raster, Terralith/BOP checks, above-sea water, BlueMap on MIC blocks. **1** plates, erosion,
climate, biomes and TerraBlender regions, the four deposit families and the COE/CDG hooks. **2** 3D
rock (Noddy-style), rivers, lakes and caves. **3** fantasy fields and deposits, sky islands, exotic
landforms, DH far terrain. **Later** pole transit, paint-and-pin constraints, the civ module, space planets.

### 10.1 How to read a work item

Every Phase 0 and Phase 1 item has the same fields. **Repo** means
`/home/greencheetah/Projects/mic/mods/deep-time`; commands run from the repo root unless they start
with `cd`. Every `cargo`, `gradlew`, `java`, `zig` and `readelf` command runs through
`nix develop -c` (§2.1). "Passes" means exit code 0 and the quoted text in the output. `$SCRATCH` is
the session's scratchpad directory. JUnit results are checked from Gradle's XML reports:
`tools/ci/junit-summary.py neoforge/build/test-results/test` prints `tests=<n> failures=<f> errors=<e>
skipped=<s>` (a 20-line script created in P0-03), so no acceptance depends on console formatting. When
an item finishes, its row in STATUS.md changes to `done` with the date, commit and evidence (§12).

Phase 0's IDs were renumbered in revision 2 so that the order of work is the numeric order; the
Toroidal World spike moved from tenth to sixth. The old IDs appear only in the cold-read review.

### 10.2 Phase 0 — the bridge

**Order:** P0-01 → P0-02 → P0-03 → P0-04 → P0-05 → **P0-06 (the Toroidal World spike)** → P0-07 →
P0-08 (can run in parallel with everything after it) → P0-09 → P0-10 → P0-11 → P0-12 → P0-13 →
P0-14 → P0-15.

The decisions file says the Toroidal World spike comes first. Its real dependencies are the Gradle
skeleton with Toroidal World (P0-03), Deep Time's own MIC server set (P0-04) and a Deep Time generator
that TW can stamp (P0-05). P0-05 therefore uses an analytic periodic test terrain in Java (§7.1), so
the spike runs before any Rust, JNI or world-data code exists and a TW failure is found before the rest
of Phase 0 is built on it.

#### P0-01 Repository bootstrap and the docs move
- **Goal.** The private repository `MinecraftIndustrialComplex/deep-time` with its conventions, licence
  files, dev shell, and the Deep Time documents versioned in `docs/`.
- **Inputs.** §1.2, §2, §4.1, §4.4, §11, §12; `/home/greencheetah/Projects/mic/mods/mic-climate/flake.nix`
  and `.gitignore` (house style).
- **Creates.**
  - `CLAUDE.md` (the §11.2 draft), `AGENTS.md` (one line: "Read CLAUDE.md."), `STATUS.md` (§12.1
    template, every P0 and P1 item `todo`), `README.md`, `COPYING` (the GPL-3.0 text) and
    `COPYING.LESSER` (the LGPL-3.0 text; LGPL-3.0-or-later needs both), `NOTICE.md` (empty table).
  - `.gitignore`: `target/`, `neoforge/build/`, `neoforge/.gradle/`, `smoke/`, `run-gametest/`,
    `run-planet/`, `libs/`, `review/out/`, `.spacesheep.json`.
  - `flake.nix` (§4.4) with inputs `nixpkgs = github:NixOS/nixpkgs/nixos-unstable` (as mic-climate),
    `flake-parts` and `oxalica/rust-overlay`; `flake.lock`.
  - `rust-toolchain.toml`: `channel = "1.98.1"`, `components = ["rustfmt", "clippy", "rust-src"]`, and
    **no `targets`** (CI installs only what each job needs, §4.5; the dev shell adds the eight targets
    through rust-overlay).
  - `docs/`: the documents of §1.2's table, copied to their new paths.
- **The docs move, step by step.** (1) Check for peers editing the originals (§2.4: `ps`, and whether
  any of them changed in the last hour, `find … -mmin -60`); if one did, stop and ask. (2) Copy each
  file to its §1.2 path. (3) `sha256sum` every original and copy and compare. (4) Commit and push.
  (5) Replace `/home/greencheetah/Projects/mic/Docs/deeptime-design-decisions.md`,
  `…/deeptime-implementation-plan.md` and `…/Geologic history worldgen mod.md` with three-line pointer
  files ("Moved to `<repo path>` in `MinecraftIndustrialComplex/deep-time` at commit `<sha>` on
  `<date>`"), write the same pointer as `MOVED.md` in the old research-notes folder and in
  `/home/greencheetah/Projects/mic/Docs/deeptime-plan/`, and delete the old notes and review file.
  `Docs/pages/` and `Docs/datapacks/` stay.
- **Depends on.** Nothing (repository owner, licence and docs location are decided).
- **Acceptance.**
  - `gh repo view MinecraftIndustrialComplex/deep-time --json visibility -q .visibility` → `PRIVATE`
  - `git ls-files docs | wc -l` → `13` (decisions, plan, report, nine notes, one review)
  - `nix develop -c rustc --version` → starts with `rustc 1.98.1`
  - `nix develop -c java -version 2>&1 | head -1` → contains `"21.`
  - `nix develop -c sh -c 'grep -o "zig-0\.[0-9.]*" "$(readlink -f "$(command -v cargo-zigbuild)")" | sort -u'`
    → `zig-0.13.0` (cargo-zigbuild's own wrapper puts zig on its PATH, so this checks the pin that
    matters; §4.4)
  - `nix develop -c cargo xwin --version` exits 0
  - `head -1 /home/greencheetah/Projects/mic/Docs/deeptime-design-decisions.md` → starts with `Moved to`
  - `git status --porcelain` → empty after the push
- **Ben reviews.** The repository on GitHub: `CLAUDE.md`, `STATUS.md`, `docs/`.

#### P0-02 Cargo workspace and CI
- **Goal.** Six crates that build and test, the licence policy enforced, CI green within the free
  plan's minutes.
- **Inputs.** §4.1, §4.2, §4.5, §6.4.
- **Creates.**
  - `Cargo.toml`: workspace with `[workspace.package]` `version = "0.0.1"`, `edition = "2024"`,
    `rust-version = "1.89"` (the highest MSRV among the pinned crates: kiddo 6.3.0), `license =
    "LGPL-3.0-or-later"`, `publish = false`, `repository`; release profile `panic = "unwind"`,
    `lto = "thin"`. Every crate inherits these with `*.workspace = true`.
  - Crate stubs `crates/deeptime-{format,core,jni,cli,viewer,inspect}` with one test each.
  - `deny.toml`: the allow-list of §4.5, which includes `LGPL-3.0-or-later` so Deep Time's own crates
    pass; `[bans]` deny duplicate major versions of `jni` and `rayon`.
  - `rustfmt.toml`; `.cargo/config.toml` (musl `-C target-feature=-crt-static`; `x86_64-apple-darwin`
    `codegen-units=2`).
  - `.github/workflows/ci.yml` (§4.5): Rust jobs only; P0-03 adds the Gradle job.
- **Depends on.** P0-01.
- **Acceptance.**
  - `nix develop -c cargo test --workspace` → `test result: ok` for every crate
  - `nix develop -c cargo clippy --workspace --all-targets -- -D warnings` → exit 0
  - `nix develop -c cargo deny check licenses bans` → `licenses ok`, `bans ok`
  - `gh run list -R MinecraftIndustrialComplex/deep-time -w ci.yml -L 1 --json conclusion -q '.[0].conclusion'` → `success`
- **Ben reviews.** Nothing visual.

#### P0-03 Gradle NeoForge module skeleton
- **Goal.** A mod jar that boots in the GameTest server with Toroidal World present, with JUnit and the
  GameTest template set up.
- **Inputs.** §4.3, §6.5 (version source), §7.15, §9.4; mic-climate's `build.gradle`,
  `gradle.properties`, `gradle/wrapper/gradle-wrapper.properties`, `gametest.sh`,
  `tools/make_test_structure.py`, `mic_climate.mixins.json`, `MicClimateMixinPlugin.java`,
  `neoforge.mods.toml` template.
- **Creates.**
  - `neoforge/{build.gradle, gradle.properties, settings.gradle, gradlew, gradlew.bat,
    gradle/wrapper/gradle-wrapper.jar, gradle/wrapper/gradle-wrapper.properties}` — Gradle 8.11.1, the
    version mic-climate's wrapper pins.
  - `gradle.properties` (§4.3): `mod_id = deeptime`, `mod_name = Deep Time`, `mod_version = 0.0.1`,
    `mod_group_id = com.minecraftindustrialcomplex`, `mod_authors = Benjamin McIntyre` (as in
    mic-climate), `mod_license = LGPL-3.0-or-later`, `mod_description`, and the NeoForge, Parchment and
    Toroidal World pins.
  - `com/minecraftindustrialcomplex/deeptime/DeepTime.java` (`@Mod("deeptime")`),
    `mixin/DeepTimeMixinPlugin.java`, `deeptime.mixins.json` (`defaultRequire: 1`, empty lists),
    `deeptime.gametest.mixins.json` (empty until P0-05), the `neoforge.mods.toml` template (both mixin
    configs; `minecraft`, `neoforge` and `toroidal_world` required, side `BOTH`).
  - JUnit (§4.3): `useJUnitPlatform()`, `testLogging { events 'passed', 'failed' }`,
    `testImplementation 'org.lz4:lz4-java:1.8.0'`; `tools/ci/junit-summary.py`; one trivial test.
  - Gradle task `checkVersions` (a dependency of `build`; compares `mod_version` with the Cargo
    workspace version).
  - `tools/make_test_structure.py` (adapted from mic-climate's) and its output
    `neoforge/src/main/resources/data/deeptime/structure/empty_5x5x5.nbt`.
  - `gametest/BootGameTests.toroidalWorldLoaded`; `gametest.sh` (mic-climate's script with Deep Time
    names, `-Ddeeptime.gametest=true`, and the clean-up of §9.4).
  - The Gradle job in `.github/workflows/ci.yml` (§4.5).
- **Depends on.** P0-02 (for `checkVersions`).
- **Acceptance.**
  - `cd neoforge && nix develop -c ./gradlew build` → `BUILD SUCCESSFUL`; `build/libs/deeptime-1.21.1-0.0.1.jar` exists
  - `python3 tools/ci/junit-summary.py neoforge/build/test-results/test` → `failures=0 errors=0`, `tests=1`
  - `./gametest.sh` → `GAME TESTS COMPLETE` and `All 1 required tests passed :)`
  - Changing `mod_version` to `0.0.2` makes `cd neoforge && nix develop -c ./gradlew checkVersions` fail (then revert)
- **Ben reviews.** Nothing visual.

#### P0-04 Deep Time's server sets and smoke harness
- **Goal.** Deep Time's own copies of the current MIC pack (server side) and of the `standalone` and
  `terrablender` sets, a smoke harness over RCON, and `-PwithMic` wired to that MIC set. This is also
  the spike's first check (`wrapped_planet_worlds.md` §6.3 step 1): the current MIC pack boots with
  Toroidal World and Deep Time.
- **Inputs.** §2.2, §4.3, §9.4, §9.5; mic-climate's `smoke/install.sh` and `smoke/run.py`.
- **Creates.** `tools/smoke/{install.sh, run.py, profiles/{standalone,mic,terrablender}.txt,
  profiles/*.excluded}` (§9.5; `run.py --level-type <preset>` allows a boot-only run before Deep Time's
  preset exists); the Gradle task `linkMicMods` behind `-PwithMic` (§4.3).
- **Depends on.** P0-03.
- **Acceptance.**
  - `tools/smoke/install.sh mic && python3 tools/smoke/run.py --profile mic --level-type minecraft:normal`
    → `PASS`; `smoke/mic/report.md` names the pack commit, lists every mixin of Deep Time and Toroidal
    World as applied, and lists excluded jars (none expected)
  - the same for `standalone`
  - `./gametest.sh -PwithMic` → `BootGameTests` passes with the MIC set in `run-gametest/mods/`
- **Ben reviews.** The smoke reports (published with the spike, P0-06).

#### P0-05 Generator skeleton on the analytic test terrain
- **Goal.** `GeologicChunkGenerator` registered with its preset and dimension type, producing the
  analytic test terrain (§7.1) periodic in x, testable in both GameTest runtimes.
- **Inputs.** §7.1, §7.12, §7.16, §9.4; `minecraft_integration_architecture.md` §1–§2;
  `mod_compatibility_biomes_ores_fantasy.md` follow-up section.
- **Creates.**
  - **First**, the GameTest-server fix of §9.4: `gametest/mixin/GameTestServerMixin`, the gametest pack
    with `data/deeptime/dimension/test_planet.json`, and `BootGameTests.testPlanetDimensionExists`.
  - `worldgen/{GeologicChunkGenerator, DerivedSettings, DelegatingRuleSource, WorldDataRef,
    ActiveWorldData, TerrainSource, AnalyticTestTerrain}`; `worldgen/density/{TerrainDensity,
    DepthDensity, RasterDensity, BeyondPoleDensity}` (`RasterDensity` reads the analytic source's
    climate values: temperature from latitude, continentalness from H against sea level, the rest
    mid-range); `server/{WorldDataBinder, DeepTimeCommands}` (`probe column`, `probe tw` stub).
  - Resources of §7.12: presets `planet` and `planet_32k`, dimension type, preset tag, TerraBlender tag,
    the router and its `planet_nocaves` variant.
  - The `planetServer` Gradle run and `gametest-planet.sh` (§9.4).
  - `GeneratorGameTests.{heightMatchesReference, periodicAcrossSeam, oreVeinsDisabled}` in
    `test_planet`:
    - `heightMatchesReference`: the surface is the highest block that blocks motion and holds no fluid
      (the `OCEAN_FLOOR_WG` heightmap stores that y + 1); the expected value is `ceil(H) − 1`. With a
      density linear in y, vanilla's 4 × 8 × 4 interpolation reproduces H exactly at cell-corner
      columns and bilinearly between them, so corner columns (x ≡ z ≡ 0 mod 4) must match with
      tolerance 0 and other columns within 1 block (the test terrain keeps the bilinear error far
      below a block).
    - How the height tests read terrain: `level.getChunkSource().getChunk(cx, cz, ChunkStatus.NOISE,
      true)` on chunks no other test touches, then `getHeight(Heightmap.Types.OCEAN_FLOOR_WG, x, z)`.
      At NOISE status only the density fill has run, so biome carvers (vanilla's `cave`,
      `cave_extra_underground` and `canyon`, which run at CARVERS), surface rules and features (trees,
      surface lava lakes) cannot move the surface; and the `*_WG` heightmaps exist only up to SURFACE
      status (`ChunkStatus`: from CARVERS on a chunk carries only the final heightmaps). Noise caves
      are part of `final_density` and do act at NOISE, which is why `test_planet` uses
      `planet_nocaves`.
    - `periodicAcrossSeam`: for 16 z values, columns x = C/2 − 1 and x = −C/2 continue each other
      (height difference no larger than between any two neighbouring columns of the same z).
- **Depends on.** P0-03 (P0-04 for the `-PwithMic` run).
- **Acceptance.**
  - `./gametest.sh` → `testPlanetDimensionExists` and the three generator tests pass; the log has
    `[gametest] heightMatchesReference columns=128 cornerColumns=64 cornerMaxAbsDiff=0 maxAbsDiff=<0 or 1>`
    and `[gametest] periodicAcrossSeam pairs=16 mismatches=0`
  - `./gametest.sh -PwithMic` → the same pass with the current MIC set
  - `./gametest-planet.sh --boot-only` → reaches `Done (` with `level-type=deeptime:planet` and the
    test terrain; `/deeptime probe column 0 0` prints a `[deeptime-probe]` JSON line
- **Ben reviews.** Covered by P0-06's page.

#### P0-06 Toroidal World spike (server-side items)
- **Goal.** Prove, or disprove, that TW wraps a Deep Time world in the current MIC pack, including Sable
  ships, using every item of `wrapped_planet_worlds.md` §6.3 that runs without a client. That note
  sizes worlds at 1,000 and 2,000 chunks; the decided sizes are 1,024 and 2,048.
- **Inputs.** `wrapped_planet_worlds.md` §0, §1.2, §3.2, §4, §6.3; §7.10, §7.11; TW source at commit
  `3c1f3a30` of `anyttng/toroidal_world`, branch `mc/1.21` (`docs/modding.md`,
  `docs/dedicated-server.md`, `engine/gen/ShapedDimensions.java`, `api/v1/shape/*`,
  `api/v1/ToroidalShape.java`).
- **Steps and creates.**
  1. Sonnet lookup in TW's source: when `ShapeModule.Apply` runs for a world created from a non-TW
     preset in singleplayer and with `level-type` on a server; how `Read` and `settings(S)` choose and
     store the shape (`withStoredShape`); the `climate_compression` option; what `TerrainCeiling` and
     `GenerationProbe.sweepCrumbs` key on; how a cylinder uses the vanilla border; End span rules.
  2. `wrap/PlanetShape` and its registration (§7.10); `/deeptime probe tw` →
     `{"generator":"…GeologicChunkGenerator","stamped":true,"loopsX":true,"loopsZ":false,"widthChunks":N,"climateCompression":"off"}`.
  3. Planet-server GameTests (`./gametest-planet.sh`, standalone set):
     `SeamGameTests.{columnsContinuous, entityCrossesSeam, structureStraddlesSeam, shaftLineAcrossSeam}`;
     `SableSeamGameTests.jetCrossesSeam` with `attempts = 20, requiredSuccesses = 20` (a hull assembled
     with `SubLevelAssemblyHelper.assembleBlocks(ServerLevel, BlockPos, Iterable<BlockPos>,
     BoundingBox3ic)`, pushed with `RigidBodyHandle.addLinearAndAngularVelocity`, checked for existence,
     block count and folded x after crossing); `PoleCapGameTests.prototypePushback`.
     `shaftLineAcrossSeam` is informational: it logs whether a kinetic network works across the seam.
  4. Scenario `sable_straddle` in `gametest-planet.sh`: assemble a hull straddling the seam,
     `save-all flush`, stop, start, and check with `/deeptime probe sable` that the same sub-level id
     exists with the same block count.
  5. Smoke `mic` at C = 16,384 and 32,768 with the test terrain: boot, every mixin applied, the TW
     probe, then Chunky over a rectangle straddling the seam and `run.py`'s region-file span check.
  6. If step 1 finds no way to shape a world created from Deep Time's preset (in singleplayer or on a
     server's first start), or the Sable test fails, or the MIC pack will not boot with TW in a way
     configuration cannot fix: stop and ask Ben (Q6) before building the fallback of §7.10.
  7. `tools/review/publish.sh` and `tools/review/spaces.json` (§2.6), then publish the spike page.
- **Depends on.** P0-04, P0-05.
- **Acceptance.**
  - `./gametest-planet.sh` → `SeamGameTests` 3 required passed (plus the informational one logged),
    `SableSeamGameTests.jetCrossesSeam` 20/20, `sable_straddle: OK`
  - `tools/smoke/install.sh mic --size 32768 --test-terrain wrap-test && python3 tools/smoke/run.py --profile mic`
    → `PASS`; the report shows `tw.stamped=true tw.widthChunks=2048 tw.climateCompression=off` and
    `regions outside span: 0`; the same at `--size 16384` with `tw.widthChunks=1024`
- **Ben reviews.** "Deep Time — P0 Toroidal World spike": the findings table, both smoke reports and
  the GameTest logs.

#### P0-07 JNI hello world, native loader and the five failure classes
- **Goal.** Java calls Rust through the §6.1 API shape in its Phase 0 form; loading follows §6.5; each
  of §6.6's five failure classes yields its guidance and no crash.
- **Inputs.** §6 entirely; `aeronautics_rust_bridge.md` §3–§5 and §8.
- **Creates.**
  - Rust: `deeptime-core/src/session.rs` (`Session`, the `Stage` trait, `DummyStage`),
    `deeptime-jni/src/{lib.rs, handles.rs, errors.rs, alloc.rs, selftest.rs}` (every §6.1 entry point
    with its Phase 0 behaviour, `with_env` + a custom `ErrorPolicy`, the counting allocator).
  - Java: `nativebridge/{DeepTimeNative, NativeLoader, ExtractResult, NativeStatus, FailureClass,
    PlatformKey, ElfInterp, SimSession, *Exception}`.
  - Gradle task `hostNatives` (§4.3): builds the host library into `target/natives-host/` and writes its
    `natives.json` (sha256, version, format).
  - `tools/natives/NativeSmoke.java`: calls `NativeLoader.load(Path.of(args[0]))` and prints
    `status=OK version=<semver> format=<f> selftest=<checksum>` or
    `status=FAILED class=<FailureClass>` from the returned `NativeStatus` (it never touches the
    package-private `DeepTimeNative`).
  - JUnit `NativeLoaderTest` for extraction (temporary game directories: first extraction, reuse without
    rewriting, re-extraction of a corrupted file, four concurrent extractions into one directory of
    which exactly one writes, and one case per failure class through its forcing property);
    `NativeLinkTest` links once per JVM and checks version and self-test (`forkEvery = 1` on the
    `test` task, so every test class gets its own JVM).
  - GameTests `NativeGameTests.{loadsAndSelfTests, dummySessionRunsAndCancels}` (the dummy preset of
    §6.1).
- **Depends on.** P0-02, P0-03.
- **Acceptance.**
  - `cd neoforge && nix develop -c ./gradlew test` then
    `python3 tools/ci/junit-summary.py neoforge/build/test-results/test` → `failures=0 errors=0` and at
    least 9 tests (4 extraction cases and 5 failure classes)
  - `./gametest.sh` → both `NativeGameTests` pass; the log has
    `[gametest] native version=0.0.1 format=0.0 target=x86_64-unknown-linux-gnu selftest=<checksum>`
  - `nix develop -c java -cp neoforge/build/libs/deeptime-1.21.1-0.0.1.jar tools/natives/NativeSmoke.java $SCRATCH/gd`
    → `status=OK version=0.0.1 format=0.0 …`; and with each of
    `-Ddeeptime.natives.forceFail=unsupported|libc_mismatch|io|version_mismatch` and
    `-Ddeeptime.natives.testDeleteAfterWrite=true` → `status=FAILED class=<the matching FailureClass>`,
    exit code 0 every time
- **Ben reviews.** Nothing visual; STATUS lists the five classes and their guidance text.

#### P0-08 Cross-builds and the OS load matrix
- **Goal.** All eight natives built from Linux, checked, and loaded on real operating systems.
- **Inputs.** §4.4, §4.5, §6.4, §6.5.
- **Creates.** `tools/natives/build-natives.sh` (behind `nix run .#natives`; writes
  `target/natives-dist/<platform>/` and `target/natives-dist/natives.json`); the release path in
  Gradle (`-Prelease` packs `natives-dist/`, §4.3); `.github/workflows/natives.yml` (§4.5).
  Before the first Windows build, confirm from cargo-xwin 0.22.0's README which LLVM tools it needs on
  PATH and that the dev shell provides them (§4.4).
- **Depends on.** P0-07.
- **Acceptance.**
  - `nix run .#natives` → `jq '.libraries | length' target/natives-dist/natives.json` → `8`
  - `nix shell nixpkgs#file -c file target/natives-dist/*/*.{so,dll,dylib}` → two PE32+ (x86-64,
    Aarch64), two Mach-O (x86_64, arm64), four ELF (two x86-64, two aarch64)
  - `nix develop -c readelf -d target/natives-dist/linux-x86_64-musl/libdeeptime_jni.so | grep NEEDED`
    → no `libc.so.6`, no `ld-linux`
  - `nix develop -c sh -c "readelf -V target/natives-dist/linux-x86_64-gnu/libdeeptime_jni.so | grep -o 'GLIBC_[0-9.]*' | sort -V | tail -1"`
    → `GLIBC_2.17` or lower
  - `cd neoforge && nix develop -c ./gradlew build -Prelease` → the jar contains 8 libraries
    (`unzip -l … | grep -c 'natives/deeptime/.*\.\(so\|dll\|dylib\)'` → `8`)
  - `gh workflow run natives.yml -R MinecraftIndustrialComplex/deep-time`, then
    `gh run list -R … -w natives.yml -L 1 --json databaseId -q '.[0].databaseId'` and
    `gh run watch <id> -R … --exit-status` (review 2, gap 17) → all jobs `success`; each load log
    prints `status=OK version=0.0.1 …` and `platform=<its key>`
- **Ben reviews.** The Actions run page, linked from STATUS.

#### P0-09 World-data format v0, synthetic planets, Java reader
- **Goal.** The Rust writer and the Java reader agree on tile packs; synthetic worlds exist for the
  rest of Phase 0.
- **Inputs.** §5.3, §5.7; `minecraft_integration_architecture.md` §5.
- **Creates.**
  - Rust: `deeptime-format` (manifest, `LayerDesc`, tile-pack writer and reader for Raw and LZ4-block
    tiles, CRC32, format version); `deeptime synth` with `wrap-test` (the §7.1 analytic terrain written
    as layers, latitude climate layers, a three-horizon strata stack, one test site body, spawn
    candidates) and `lake-test` (a basin with floor y 110, rim y 125, `water_level` 120, and a channel);
    `deeptime-viewer` v0 (layers, click-to-inspect, seam marker).
  - Java: `worlddata/{WorldData, Manifest, FormatVersion, TilePack, TileCache, RasterSampler}`.
  - JUnit `TilePackParityTest`: the Gradle `test` task first runs
    `nix develop -c cargo run -p deeptime-cli -- synth --kind wrap-test --size 1024 --out build/test-worlds/wrap1024`
    (C = 1,024 keeps the fixture small: L3 at r 2 is 512 × 256 cells, 256 KiB per i16 layer), then
    compares `elevation`, `water_level` and `climate/temperature` at every cell of the first and last
    tile plus 1,000 random cells against the values the CLI dumps with `--dump-samples`.
  - JUnit `Lz4RoundTripTest`: lz4-java 1.8.0 `LZ4Factory.safeInstance().safeDecompressor()` on
    `lz4_flex::block` output.
- **Depends on.** P0-02, P0-03.
- **Acceptance.**
  - `nix develop -c cargo test -p deeptime-format` → `test result: ok`
  - `nix develop -c cargo run -p deeptime-cli -- synth --kind wrap-test --size 16384 --out $SCRATCH/synth16k --viewer review/out/p0-synth`
    → `jq -r .format_version $SCRATCH/synth16k/manifest.json` → `0.0`
  - `cd neoforge && nix develop -c ./gradlew test --tests '*TilePackParityTest' --tests '*Lz4RoundTripTest'`
    then `junit-summary.py` → `failures=0 errors=0`
- **Ben reviews.** "Deep Time — P0 synthetic planet" (the viewer).

#### P0-10 The generator on world data
- **Goal.** The generator reads world data: rasters, horizons and bodies, with the pack's surface rules
  on top and strata beneath.
- **Inputs.** §7.1, §7.5, §7.12, §7.13 (binding only), §8.8.
- **Creates.** `worldgen/WorldDataTerrain`, `RasterDensity` on real layers, `StrataPass` v0 (horizons,
  vanilla member blocks only), `server/{StagingMover, SpawnHandler}` (binding of world data found in
  the world folder or in staging), the gametest dimension `test_planet_data` (world data
  `deeptime-test`, §9.4), `GeneratorGameTests.{heightMatchesRaster, strataClassesResolve, spawnOnLand}`
  (`spawnOnLand` calls the spawn-candidate selection directly against `test_planet_data`).
- **Depends on.** P0-05, P0-09.
- **Acceptance.**
  - `./gametest.sh` → the three pass; `heightMatchesRaster` reads chunks at NOISE status as P0-05's
    test does and logs the same fields with the same tolerances (corner columns 0, others 1)
  - `./gametest.sh -PwithMic` → the same with the current MIC set
- **Ben reviews.** Covered by P0-11 and P0-14's renders.

#### P0-11 `deeptime-inspect`
- **Goal.** Our own renders and checks from region files.
- **Inputs.** §9.7; `minecraft_integration_architecture.md` §1 (region format, 1.21 chunk NBT).
- **Creates.** `crates/deeptime-inspect` with `colors`, `topdown`, `rock`, `ore`, `section`, `seam`,
  `regions`, `compare`; a small region fixture copied from the standalone smoke world for its tests.
- **Depends on.** P0-04, P0-09, P0-10 (a standalone smoke world with world data:
  `tools/smoke/install.sh standalone --synth wrap-test --size 16384 && python3 tools/smoke/run.py --profile standalone --fresh --chunky 128 --chunky-seam`;
  `--chunky-seam` pregenerates the columns either side of the seam for `seam`).
- **Acceptance.**
  - `nix develop -c cargo test -p deeptime-inspect` → ok
  - `nix develop -c cargo run -p deeptime-inspect -- compare --world smoke/standalone/server/planet --world-data smoke/standalone/server/planet/deeptime`
    → JSON with `"p99_abs_diff_blocks"` ≤ 1 (measured and expected as §9.7 defines them)
- **Ben reviews.** Top-down, section and seam PNGs on the Phase 0 page.

#### P0-12 Terralith, TerraBlender and Biomes O' Plenty checks
- **Goal.** Show the pack's biome machinery survives the generator: Terralith (the Lithostitched
  family) in the current MIC pack, TerraBlender + BOP in a clean set.
- **Inputs.** §7.1–§7.3, §7.12; `mod_compatibility_biomes_ores_fantasy.md` key questions 1–3 and the
  follow-up.
- **Creates.** `BiomeGameTests.{terralithBiomesOnLand, surfaceRulesApplied}`,
  `TerraBlenderGameTests.bopBiomesPresent`, the `-PwithTerraBlender` runtime (§4.3), and the smoke
  check `locate structure #minecraft:village`. (No Tectonic compat mixin: §7.12 shows its preset filter
  cannot hide `deeptime:planet`.)
- **Depends on.** P0-04, P0-10.
- **Acceptance.**
  - `./gametest.sh -PwithMic` → both `BiomeGameTests` pass; log
    `[gametest] terralithBiomesOnLand terralith=<n> of <m>` with n > 0
  - `./gametest.sh -PwithTerraBlender` → `bopBiomesPresent` passes (`biomesoplenty:` biomes > 0 in a
    64 × 64 surface grid)
  - `tools/smoke/install.sh mic --synth wrap-test && python3 tools/smoke/run.py --profile mic` → `PASS`,
    with `locate village: found` and no errors from C2ME or Distant Horizons in the boot log
- **Ben reviews.** The BlueMap render (P0-14) with Terralith surfaces on raster terrain.

#### P0-13 Water above sea level holds
- **Goal.** Prove §7.9's mechanism: water placed above sea level stays put.
- **Inputs.** §7.9; the `lake-test` synthetic world (P0-09).
- **Creates.** `worldgen/WaterPass`; the gametest dimension `test_lake` (world data from
  `synth --kind lake-test --size 1024`; fixed biome `deeptime:test_bare`, §9.4);
  `WaterGameTests.lakeHolds` (counts water sources in the basin after generation and after 400 ticks
  with the chunks forced, and water outside the footprint).
- **Depends on.** P0-10, P0-11.
- **Acceptance.**
  - `./gametest.sh` → `lakeHolds` passes; log `[gametest] lakeHolds before=<N> after=<N> expected=<N> outside=0`
    with before = after = expected, and `[gametest] lakeHolds water_top_y=120` (the top face of the
    highest water block at the lake centre, `notes.lake.centre_block`; cold-read review 2, gap 18)
  - `nix develop -c cargo run -p deeptime-inspect -- section --world run-gametest/world/dimensions/deeptime/test_lake --axis z --at 0 --from -88 --to 212`
    → a PNG with the water surface at y 120 and the printed line `water_top_y=120` (the lake centre
    is z 0 and the channel runs along it to x 160; −88…212 is `lakeHolds`' scanned box; P0-11)
- **Ben reviews.** The cross-section and the BlueMap view of the lake.

#### P0-14 BlueMap renders on MIC blocks
- **Goal.** A 3D web map of a generated world in the current MIC pack, published privately.
- **Inputs.** §9.6, §2.6, §2.7.
- **Creates.** `tools/bluemap/{VERSION, core.conf.tmpl, maps/planet.conf.tmpl, storages/file.conf.tmpl}`,
  the flake app `bluemap`, `tools/review/render-bluemap.sh`; `compression: none` in the storage
  template (§9.6), and `tools/review/bluemap-sw.js` only if uncompressed tiles still fail to load.
- **Depends on.** P0-04, P0-10 (a MIC smoke world with world data).
- **Acceptance.**
  - `nix run .#bluemap -- render smoke/mic/server/planet` → exit 0 and rendered tiles under
    `tools/bluemap/out/web/maps/planet/`
  - the published page loads (checked in Chrome where available) and a cave cut shows blocks below
    y 0 down to the floor (the custom-height check)
  - STATUS records whether the service-worker fallback was needed
- **Ben reviews.** The BlueMap page: MIC blocks, Create stones in the strata.

#### P0-15 Phase 0 exit
- **Goal.** Run everything, publish one page, record the result.
- **Creates.** `tools/review/phase0.sh`, `review/out/phase0/index.html`.
- **Depends on.** P0-01 … P0-14.
- **Acceptance.** The Phase 0 exit test below.

#### Phase 0 exit test

`tools/review/phase0.sh` re-installs the `mic` set, then runs in order: `cargo test`, `clippy`,
`cargo deny`; `./gradlew test` with `junit-summary.py`; `./gametest.sh` in the default, `-PwithMic`,
`-PwithTerraBlender` profiles; `./gametest-planet.sh` (standalone and MIC); the smoke tests
(`standalone`, `mic` at C = 16,384 and 32,768); the inspect renders; BlueMap; it reads the latest
`natives.yml` run; then it builds and publishes "Deep Time — Phase 0" and writes the link into
STATUS.md. Phase 0 passes when every row is green:

1. Natives: eight builds, loaded on every available OS runner; each of the five failure classes is
   forced and shows its guidance without a crash.
2. Generator: the test dimension exists; height, periodicity and veins-off on the analytic terrain;
   height, strata and spawn on world data; with and without the current MIC set.
3. Toroidal World: the generator is stamped; X loops at 1,024 and 2,048 chunks and Z does not; climate
   compression is off; seam columns are continuous; an entity and 20 of 20 Sable hulls cross the seam;
   a hull saved straddling the seam reloads; nothing is written outside the span.
4. Biomes: Terralith biomes and surface rules on raster terrain in the current MIC pack; BOP biomes
   with TerraBlender; villages locate; C2ME and Distant Horizons load without errors.
5. Water: the test lake holds.
6. BlueMap: a 3D render of MIC blocks, including below y 0.
7. The private review page exists and STATUS.md links it, with the pack commit it was tested against.

**Top risks.** How TW applies a shape to Deep Time's preset on a dedicated server's first start (the
open API question, now answered in P0-06 before anything else depends on it); TW's version pins
against the current pack (§13); TW's youth (pre-1.0, one maintainer, no world-compatibility promise);
Sable seam crossing, fixed only in TW 0.19.3 by changelog inference; the gametest-only mixin into a
lambda of `GameTestServer` (it exists only for tests, and `testPlanetDimensionExists` fails loudly if a
NeoForge update breaks it); macOS extraction, testable only in CI; BlueMap's custom-height support;
the surface-rule delegation working with both TerraBlender and Lithostitched; this laptop's free
memory when running the MIC smoke.

### 10.3 Phase 1 — the MVP (plates, erosion, climate, biomes, four deposit families, COE/CDG)

Order: P1-01 → P1-02 → P1-03 → P1-04 → P1-05 → P1-06 → P1-07 → P1-08 → (P1-09, P1-10, P1-11,
P1-12, P1-13 in any order) → P1-14 → P1-15 → P1-16 → P1-17 → P1-18 → P1-19. P1-17 can run early as
a smoke benchmark and again at the end.

#### P1-01 Session, checkpoints, keyframes, event log and site formats
- **Goal.** The stage runner of §8 with real persistence: checkpoints, keyframes, events, sites.
- **Inputs.** §3.3, §5.5, §5.6, §6.1, §8 (Stage trait).
- **Creates.** `deeptime-core/src/{session.rs (extending P0-07's), stage.rs, checkpoint.rs, keyframes.rs}`,
  `deeptime-format/src/{events.rs, sites.rs}` plus `EVENTS.md`, CLI subcommands `run`, `stage`,
  `bench` (skeletons), viewer: keyframe strip, scrubber, event ticker, site list.
- **Depends on.** P0-07, P0-09.
- **Acceptance.**
  - `nix develop -c cargo test -p deeptime-format` → event segments and site records round-trip;
    two writers' segments merge with no id collision
  - `nix develop -c cargo run -p deeptime-cli -- run --preset crates/deeptime-core/tests/presets/dummy.json --out $SCRATCH/dummy --viewer review/out/p1-dummy`
    → `manifest.json`, `events/1-0.dtev`, `keyframes/index.json` exist
- **Ben reviews.** Nothing yet (dummy content).

#### P1-02 Fibonacci sphere, triangulation and raster coupling
- **Goal.** L0 geometry and the L0 → L1 lookup of §5.2.
- **Inputs.** §5.2; Cortial et al. 2019 (Fibonacci sampling and triangulation).
- **Creates.** `deeptime-core/src/sphere/{fibonacci.rs, hull.rs (spherical Delaunay as the convex
  hull of unit vectors), kdtree.rs (kiddo), coupling.rs}`.
- **Depends on.** P1-01.
- **Acceptance.**
  - `nix develop -c cargo test -p deeptime-core sphere::` → triangulation satisfies V − E + F = 2;
    every L1 cell has barycentric weights summing to 1 within 1e-6; nearest-sample queries agree
    with brute force on 10,000 random points
  - build time for 250k samples recorded in STATUS (no target; it feeds P1-17)
- **Ben reviews.** Nothing visual.

#### P1-03 Plates and crust
- **Goal.** §8.1 end to end, with events and keyframes.
- **Inputs.** §8.1; Cortial et al. 2019 (PDF linked in `tectonics_engines.md` Q3);
  tectonics.js `RockColumn.js` and `Crust.js` (CC BY 4.0; add to `NOTICE.md`).
- **Creates.** `deeptime-core/src/plates/{kinematics.rs, subduction.rs, collision.rs, rifting.rs,
  ridges.rs, hotspots.rs, pools.rs, events.rs, normalise.rs (the Earth-normalized mapping of §8.1)}`;
  built-in presets' tectonic section.
- **Depends on.** P1-02.
- **Acceptance.**
  - `nix develop -c cargo test -p deeptime-core plates::` → area conserved across rifting (relative
    error < 1e-9); no sample in two plates; ages non-negative
  - `nix develop -c cargo run --release -p deeptime-cli -- stage plates --preset plates_quick_16k --start scattered_continents --length-myr 500 --viewer review/out/p1-plates`
    → prints `plates_final=<n> ocean_age_p90_myr=<a> … steps=250 seconds=<s>` and writes the viewer
    (`--start` is required: the presets leave the start configuration to the player, decisions
    file "Timeline"; `plates_quick_16k` is the plate pipeline's preset while `earthlike_quick_16k`
    stays P1-13's placeholder until P1-08)
- **Ben reviews.** "Deep Time — P1 plates": the timelapse of plates, crust age and elevation, with
  the rift/collision ticker.

#### P1-04 Deep-time surface processes on L1
- **Goal.** §8.2's deep-time half, coupled to the plate step, with the strata log.
- **Inputs.** §8.2; Braun & Willett 2013, Yuan et al. 2019, Barnes et al. 2014 (papers); Landlab
  `exponential_weathering.py`, `litholayers.py`, `threshold_eroder.py`,
  `simple_submarine_diffuser.py`, `carbonate_producer.py` (MIT; `NOTICE.md`).
- **Creates.** `deeptime-core/src/surface/{routing.rs, pflood.rs, spl.rs, deposition.rs,
  diffusion.rs, weathering.rs, marine.rs, carbonate.rs, strata_log.rs, facies.rs}`;
  `tools/validation/fastscape_compare.py`.
- **Depends on.** P1-03.
- **Acceptance.**
  - `nix develop -c cargo test -p deeptime-core surface::` → receivers acyclic; mass conserved by
    diffusion; steady state under uniform uplift matches the analytic stream-power profile within
    the tolerance recorded in the test
  - `nix shell nixpkgs#python312 -c python3 tools/validation/fastscape_compare.py --grid 256` →
    `hypsometry_ks=<d> chi_slope_ratio=<r>` within the tolerances set on first calibration (recorded
    in the script)
  - `nix develop -c cargo run --release -p deeptime-cli -- stage erosion --preset earthlike_quick_16k --viewer review/out/p1-erosion`
- **Ben reviews.** "Deep Time — P1 erosion": elevation, drainage area and facies through time.

#### P1-05 Climate
- **Goal.** §8.3's snapshot pipeline.
- **Inputs.** §8.3; climlab EBM equations (MIT); `fastscape-lem/orographic-precipitation` (MIT,
  96 lines); Köppen rules.
- **Creates.** `deeptime-core/src/climate/{insolation.rs, ebm.rs, belts.rs, lapse.rs,
  orographic.rs (rustfft), koppen.rs, glaciers.rs, sealevel.rs}`.
- **Depends on.** P1-04.
- **Acceptance.**
  - `nix develop -c cargo test -p deeptime-core climate::` → the EBM converges to energy balance; a
    synthetic ridge across the wind gives more precipitation windward than leeward; Köppen unit
    cases (hand-built monthly series for Af, BWh, Cfb, Dfc, ET) classify correctly
  - `nix develop -c cargo run --release -p deeptime-cli -- stage climate --preset earthlike_quick_16k --viewer review/out/p1-climate`
- **Ben reviews.** Köppen, temperature and precipitation maps and their change through time.
- *As run (P1-05):* the acceptance command is `stage climate --preset pipeline_quick_16k --start
  scattered_continents --length-myr 500` (the `earthlike_*` built-ins stay placeholders until P1-08,
  as for P1-04); the climate is coupled into the plates group rather than a separate stage after it.
  A synthetic ridge test, EBM balance and Köppen cases pass as specified.

#### P1-06 Young landscape, amplification and the detail layer
- **Goal.** L2 young landscape and L3 amplification (§8.2), plus `detail/amplitude` and the drama
  slider.
- **Inputs.** §8.2; `H-Schott/MultiScaleErosion` (MIT, commit 964a3d5; `NOTICE.md`).
- **Creates.** `deeptime-core/src/surface/{young.rs, amplify.rs, thermal.rs, detail.rs}`, tile
  streaming for L3 (§5.7.5).
- **Depends on.** P1-04, P1-05.
- **Acceptance.**
  - `nix develop -c cargo test -p deeptime-core surface::amplify` → amplification creates no new
    internal sinks outside lakes; streamed and in-memory L3 agree on a small grid
  - timings per pass recorded (feeds P1-17)
- **Ben reviews.** Before/after amplification views in the viewer.

- *As run (P1-06):* the acceptance ran on `pipeline_quick_16k` (`stage young`, `stage amplify`; the
  built-ins stay placeholders until P1-08). `cargo test -p deeptime-core surface::amplify` passes (25
  tests: no new sink outside the lakes on a wrapped test planet with a lake, streamed = whole-raster
  to the bit, tiled middle level = whole, 1 = 5 threads, the seam, the flood, drama, the pack
  round trip, the coupled pipeline on 1 and 3 threads and a resume); timings per pass are in
  STATUS.md and `notes.amplify` of every run (`amplify_final=…` line).

#### P1-07 Rock classes, column strata and metamorphism
- **Goal.** §5.4.1 data for all 59 classes; §8.5's Phase 1 column strata; horizon bake.
- **Inputs.** §5.4.1, §5.7.4, §8.5; the rock-tag README's class table.
- **Creates.** `neoforge/src/main/resources/data/deeptime/deeptime/rock_class/*.json` (59 files);
  `deeptime-core/src/structure/{columns.rs, metamorphism.rs, plutons.rs, bake_horizons.rs}`.
- **Depends on.** P1-04.
- **Acceptance.**
  - `nix develop -c cargo test -p deeptime-core structure::` → horizon stacks are ordered top-down,
    thickness ≥ one block after scaling, classes valid
  - the viewer's rock layer and a meridian cross-section of horizon stacks
- **Ben reviews.** The rock map and cross-section.

- **As built (P1-07, on the coupled pipeline presets).** Files `structure/{mod, params, columns,
  metamorphism, plutons, traps, hosts, bake_horizons, tests}.rs`; the stage is `structure` and runs in
  every planet preset with a `surface` section (optional `structure` section). The 59 JSONs were
  P1-12's; P1-07 reviewed them, added `lithifies_to` and `loose_equivalent`
  (`tools/dev/gen_rock_classes.py`, the Java codec) and recorded the facies-rule vocabulary the core
  forms. The acceptance ran on `pipeline_quick_16k` (the built-ins stay placeholders until P1-08):
  `cargo run --release -p deeptime-cli -- stage structure --preset pipeline_quick_16k --start
  scattered_continents --length-myr 500 --viewer review/out/p1-structure`; the viewer gets the rock
  layer (keyframes `structure/rock` and the final `strata/outcrop`), the baked stacks in its
  inspector, and meridian cross-sections (`deeptime_viewer::section`, the Java column rule of
  `StrataPass`, mirrored as `LoadedLayer::stack_at`). The bake is produced on demand from the final
  world; P1-08 writes it and the pluton bodies into the world data.

#### P1-08 Bake, world data v1 and parity
- **Goal.** Every Phase 1 chunk-time layer of §5.3 written; Java reads it; parity holds.
- **Inputs.** §5.3, §5.7, §8.8, §9.2.
- **Creates.** `deeptime-core/src/bake/*`, format `0.1`, Java reader v1 (horizons, bodies, sites),
  `ColumnParityTest`, `deeptime inspect-column`.
- **Depends on.** P1-06, P1-07.
- **Acceptance.**
  - `nix develop -c cargo run --release -p deeptime-cli -- run --preset earthlike_quick_16k --out $SCRATCH/q16 --viewer review/out/p1-q16`
    → `jq -r .format_version $SCRATCH/q16/manifest.json` → `0.1`
  - `cd neoforge && nix develop -c ./gradlew test --tests '*ColumnParityTest'`, then `junit-summary.py`
    → `failures=0 errors=0` (the test asserts that all 500 columns of `parity/columns.json` are equal)
- **Ben reviews.** The full-run viewer.

#### P1-09 Climate mapping and biomes
- **Goal.** §7.2: physical climate → the five vanilla parameters, stored per world.
- **Inputs.** §7.2; `OverworldBiomeBuilder` and `Tags.Biomes` in the sources jar.
- **Creates.** `climate/{ClimateMappingBuilder, ClimateMapping, OverworldBands}`,
  `data/deeptime/deeptime/climate_mapping/default.json`, `BiomeGameTests.climateConsistent`.
- **Depends on.** P1-08.
- **Acceptance.** `./gametest.sh -PwithMic` → `climateConsistent` passes: for each Köppen group, the
  share of sampled columns whose biome carries the matching NeoForge biome tag is logged, and each
  share meets the floor recorded in the test at its first calibrated run.
- **Ben reviews.** A viewer layer with the pack's biome names beside the Köppen map.

#### P1-10 TerraBlender regions follow provinces
- **Goal.** §7.3.
- **Inputs.** §7.3; ReTerraForged `mixin/terrablender/*`, `TBCompat`, `TBNoiseRouterData` at commit
  `ba35ca7` (MIT; `NOTICE.md`).
- **Creates.** `compat/terrablender/{TargetPointMixin, ClimateSamplerMixin, NoiseChunkMixin,
  ParameterListMixin, RegionIndexDensity}`, the province → region map in `climate_mapping.json`,
  `TerraBlenderGameTests.regionsFollowProvinces`.
- **Depends on.** P1-09.
- **Acceptance.** `./gametest.sh -PwithTerraBlender` → `regionsFollowProvinces` passes (surface
  points in one province share a region index; points below the fallback depth use TerraBlender's
  own index).
- **Ben reviews.** A viewer layer of provinces coloured by TerraBlender region.

#### P1-11 Customize editor, timelapse and staging
- **Goal.** §7.12's screens, backed by a client-free model that GameTests can drive.
- **Inputs.** §6.1, §6.2, §7.12; ReTerraForged `PresetEditorPage` (MIT) for the `DynamicTexture`
  preview pattern.
- **Creates.** Common: `screen/model/{PlanetEditorModel, RunModel, ShareCodes}`; client:
  `screen/{PlanetEditorScreen, SetupPage, RunPage, ReviewPage, GuidancePanel, KeyframeTexture,
  MapWidget, GlobeWidget, EventTicker}`, `ClientSetup` (`RegisterPresetEditorsEvent`);
  `EditorModelGameTests.{acceptBuildsGenerator, stagingMovedOnStart}`; `ShareCodeTest` (JUnit,
  decodes Rust-made codes).
- **Depends on.** P1-08, P1-09.
- **Acceptance.**
  - `./gametest.sh` → both `EditorModelGameTests` pass
  - `cd neoforge && nix develop -c ./gradlew test --tests '*ShareCodeTest'`, then `junit-summary.py` → `failures=0 errors=0`
  - best effort, not a gate: mic-climate-style `gametest-client.sh` opens the Create World screen with
    the Deep Time preset without a client crash; result recorded
- **As built (before P1-08, on the placeholder presets).** Class names differ from the list above:
  the common models are `screen/model/{EditorModel, EditorFields, TimelapseModel, ReviewModel,
  RunHandle, SessionRunHandle, …}` (share codes stay in P1-13's `preset/ShareCode`, whose
  `ShareCodeTest` already decodes the Rust-made codes); staging is `server/{Staging, StagingMover}`
  plus `worldgen/PlanetGenerators` (Accept's generator and the Create guard) instead of
  `GeologicChunkGenerator.forStaging`. GameTests: `EditorModelGameTests.{acceptBuildsGenerator,
  stagingMovedOnStart, editorRunsAcceptsAndBinds, shareCodeThroughModel}`; JUnit:
  `EditorModelTest`, `TimelapseModelTest`, `ModelHelpersTest`, `StagingMoverTest`,
  `TimelineAcrossJniTest`, `ScreenSourcesTest` (dist separation and language keys). The client
  check is `tools/client/screenshots.sh` (HeadlessMC under Xvfb, `-Ddeeptime.dev.screenshots=true`
  drives the editor and takes screenshots). Rerun with the real presets after P1-08, and add the
  review page's biome names with P1-09.
- **Ben reviews.** The same keyframes in the viewer (the screen itself is unverifiable, §9.8).

#### P1-12 Strata pass with rock tags and `rock_states`
- **Goal.** §7.5 in full.
- **Inputs.** §7.5; `/home/greencheetah/Projects/mic/Docs/datapacks/deeptime-mic-rock-tags/README.md`.
- **Creates.** `rock/{RockPalette, RockClassRegistry, RockStatesRegistry}` (datapack registries of
  §5.4), vanilla member tags under `neoforge/src/main/resources/data/deeptime/tags/block/rock/`
  (no fallback tags, §7.5), `StrataPass` v1 (bodies), `StrataGameTests.{firstInstalledFallback,
  rockStatesMember, coalFallsBackToStone, vanillaOnlyResolvesEveryClass}`. The MIC rock-tag datapack is
  not in `pack/` yet (Q16), so the `-PwithMic` run adds it from
  `/home/greencheetah/Projects/mic/Docs/datapacks/deeptime-mic-rock-tags/` through the gametest pack
  finder (`-Ddeeptime.extraDatapack=<path>`, honoured only with `-Ddeeptime.gametest=true`).
- **Depends on.** P1-08.
- **Acceptance.**
  - `./gametest.sh` and `./gametest.sh -PwithMic` → the four pass (without Born in Chaos, coal is
    `minecraft:stone`; with MIC it is black argillite; with no rock datapack at all, all 59 classes
    resolve to a block)
  - `nix develop -c cargo run -p deeptime-inspect -- rock --world smoke/mic/server/planet` renders the smoke world
- **Ben reviews.** Rock render and BlueMap cliffs showing Create stones in their classes.

#### P1-13 Dedicated server: generate on first start
- **Goal.** §7.13.
- **Inputs.** §7.13, §9.5.
- **Creates.** `server/{FirstStartGenerator, ServerConfig}`, template `config/deeptime/server.json`,
  smoke scenario `first_start`.
- **Depends on.** P1-08.
- **Acceptance.**
  `tools/smoke/install.sh standalone --preset earthlike_quick_16k && python3 tools/smoke/run.py --profile standalone --fresh`
  (`--preset` writes `config/deeptime/server.json`; `--fresh` deletes the world first) → log has
  `[deeptime]` progress lines at most 5 s apart, `Done (` follows, the report records the generation
  time; a second start logs `world data present` and does not regenerate; with the config removed
  the server writes the template and stops cleanly (`first_start_no_config: OK`).
- **As built (mechanism before P1-08).** The mechanism landed ahead of P1-08 on the placeholder
  built-in presets (§5.1). `install.sh --preset <p> [--start <s>] [--length-myr <n>]` writes the
  timeline too (the harness names supercontinent / 1000 when only `--preset` is given; the mod has
  no default). `run.py … --fresh --restart` runs the acceptance above; `run.py --scenario
  first_start_no_config | first_start_circumference_mismatch | first_start_native_failure`
  (`-Ddeeptime.natives.forceFail=unsupported`) each expect a clean stop (exit 1, no `Done (`, no
  region/data/world data, `level.dat` not rewritten, no crash report) and print `<scenario>: OK`.
  When P1-08's bake exists, the real presets replace the files in `crates/deeptime-core/presets/`
  and nothing in `FirstStartGenerator` changes; rerun the acceptance then.
- **Ben reviews.** The smoke report.

#### P1-14 Deposit engine: the four families
- **Goal.** §8.6 for porphyry Cu-Mo-Au (+ supergene, placer Au), iron and coal, Zn–Pb (MVT, SEDEX)
  and petroleum systems.
- **Inputs.** §5.4.3, §5.6, §8.6; `ore_genesis_modeling.md` Q1–Q3, Q5, Q7; USGS OFR 2008-1155
  (porphyry), OFR 2009-1252 (sediment-hosted Zn-Pb), Bulletin 1693 (descriptive models).
- **Creates.** `deeptime-core/src/deposits/{fields.rs, rules.rs, tracts.rs, count.rs, cluster.rs,
  sizing.rs (lognormal, copula), geometry.rs, ledger.rs, children.rs, petroleum.rs (TTI),
  playability.rs, sites.rs}`; deposit-type JSON under
  `neoforge/src/main/resources/data/deeptime/deeptime/deposit_type/`; grade–tonnage JSON under
  `crates/deeptime-core/data/usgs/` with a source line per table (percentiles not in the research are
  digitised here from the cited USGS reports and marked as digitised).
- **Depends on.** P1-05, P1-07.
- **Acceptance.**
  - `nix develop -c cargo test -p deeptime-core deposits::` → porphyry σ = 1.49 ± 0.01 from
    33/240/1,500 Mt; Poisson means match ρ·A·F within sampling error over 10,000 draws; minimum
    separation respected; the ledger truncates, destroys-and-spawns-placer and enriches in the
    documented cases
  - `nix develop -c cargo run --release -p deeptime-cli -- stage deposits --preset earthlike_quick_16k --viewer review/out/p1-deposits`
    prints counts per type next to the
    §8.6 formula's expectation
- **Ben reviews.** Ore provinces, site list and three "Why here?" chains in the viewer.

#### P1-15 Deposits in the world
- **Goal.** §7.6–§7.8: the deposit feature, ore blocks (with Excavated Variants when present),
  background scatter, ore claiming, essentials and spawn.
- **Inputs.** §7.6–§7.8; Excavated Variants 4.3.1 `api/data/Ore.java`, `api/RegistryKeys.java`
  (LGPL-3.0); its globaldata config format (read in this task).
- **Creates.** `worldgen/{DepositFeature, BackgroundScatterFeature}`, `ore/{OreBlockResolver,
  ClaimOresBiomeModifier, ClaimRules, Essentials}`, `compat/ev/EvBridge`, EV stone configs under
  `globaldata/deeptime/excavated_variants/`, the biome-modifier JSON, `-PwithEv` runtime,
  `DepositGameTests.{bodyHasOre, vanillaOreBlock, evVariantWhenInstalled}`,
  `ClaimGameTests.claimOffWhenOverworldNotGeologic` (GameTest server) and
  `ClaimGameTests.claimOnInPlanetWorld` (planet server), `SpawnGameTests.essentialsWithinReach`.
- **Depends on.** P1-12, P1-14.
- **Acceptance.** `./gametest.sh`, `./gametest.sh -PwithEv` and `./gametest-planet.sh -PwithMic` →
  all listed tests pass; `deeptime-inspect ore` render shows ore only inside site bodies plus scatter.
- **Ben reviews.** Ore render and a BlueMap cut through a porphyry.

#### P1-16 COE and CDG hooks
- **Goal.** §7.14.
- **Inputs.** §7.14; the COE and CDG jars in Deep Time's `smoke/mic/server/mods/` (§9.5; signatures in
  §7.14); re-run `grep -rn calculateOilChunks /home/greencheetah/Projects/mic/pack/kubejs` in case a
  MIC oil script appeared since 2026-09-28.
- **Creates.** `compat/coe/{OreVeinGeneratorMixin, OreDataMixin, CoeBridge}`,
  `compat/cdg/{OilChunksSavedDataMixin, CdgBridge}`, `CompatGameTests.{coePicksSiteVein,
  coeDropsClaimedNoise, coeAmountScalesWithTonnage, cdgOilFromTrap, cdgZeroElsewhere}`, probe lines.
- **Depends on.** P1-15.
- **Acceptance.** `./gametest-planet.sh -PwithMic` → the five pass; the MIC smoke report contains
  `probe coe <chunk>` = the site's vein id and `probe cdg <chunk>` > 0 over a trap.
- *As built (P1-16):* the mixins are `mixin/compat/createoreexcavation/{OreVeinGeneratorMixin,
  OreDataMixin, RandomSpreadGeneratorMixin}` and `mixin/compat/createdieselgenerators/
  OilChunksSavedDataMixin` (the plugin gates by package); plus `compat/{SiteChunks, CompatHooks}`,
  `compat/coe/{CoeAccess, CoeAccessImpl}`, `compat/cdg/{CdgAccess, CdgAccessImpl}`,
  `server/CompatCommands` and the deposit-type key `minecraft.cdg_scale` (§7.14 "As built"). The
  five tests run on the GameTest server's Deep Time level `deeptime:test_deposits` (P1-15's synthetic
  deposit world; the flat overworld is the control), so `./gametest.sh -PwithMic` is their gate;
  `./gametest.sh` runs Deep Time's side of each and skips the mods' calls.
- **Ben reviews.** The report's COE/CDG table.

#### P1-17 Benchmark gate and preset tuning
- **Goal.** Measure time and memory for the four presets and fix the preset table.
- **Inputs.** §5.2, §5.7.5, §8 cost models, §9.1.
- **Creates.** `bench.json` per preset, a bench viewer page, updated preset JSON and §5.2/§5.7.5
  tables (the plan is edited, and STATUS records the change).
- **Depends on.** P1-06, P1-14 (final run); may run earlier on partial pipelines.
- **Acceptance.**
  - `nix develop -c cargo run --release -p deeptime-cli -- bench --preset earthlike_quick_16k --threads 1,2,4,8 --out review/out/p1-bench/quick16k.json`
    → `total_seconds` at 8 threads ≤ 300, or the report lists the stage breakdown and scaling for
    Ben's decision (§9.1)
  - the same for `earthlike_full_16k` against 900 s; 32k presets measured and reported
  - `peak_native_bytes` ≤ the estimate shown to players; any overshoot fixes the estimator
  - LZ4 compression ratio per layer recorded
- **Ben reviews.** The bench page: time per stage, scaling with threads, memory vs estimate.

#### P1-18 Pole cap and client info
- **Goal.** §7.11 and §7.15.
- **Inputs.** §7.11, §7.15; the P0-06 pole prototype; Sable `RigidBodyHandle`.
- **Creates.** `wrap/PoleCap`, `compat/sable/SablePoleCap`, `net/{WorldInfoPayload, ClientHandlers}`,
  the F3 line, `PoleCapGameTests.{entityPushedBack, sableHullPushedBack, hardStopAt80}`,
  `NetworkGameTests.worldInfoChannelRequired` (reads NeoForge's payload registration and asserts
  the channel is not optional; the registry API is located in this task).
- **Depends on.** P0-06.
- **Acceptance.** `./gametest-planet.sh` → the four pass.
- **Ben reviews.** Nothing visual (storm visuals are client-only).

#### P1-19 Phase 1 exit
- **Creates.** `tools/review/phase1.sh`, "Deep Time — Phase 1 (MVP)".
- **Depends on.** Everything above.

#### Phase 1 exit test

`tools/review/phase1.sh` re-runs Phase 0's suite and adds:

1. Bench: the quick 16k preset within 5 minutes and the full 16k within 15 minutes at 8 threads on
   this laptop, or a per-stage report for Ben's decision; peak memory within the estimate.
2. MIC smoke from an empty world with `earthlike_quick_16k`: first-start generation, boot, every
   mixin (including COE and CDG) applied, probes consistent with the world data.
3. Geology in the world: a porphyry site within the essentials reach of spawn; `probe site` returns
   its body; `deeptime-inspect ore` and `section` show copper ore inside the body, a gossan where it
   is exposed, and its placer-gold child downstream; BlueMap shows the arc range and the deposit.
4. COE reports the copper vein over the porphyry and nothing claimed where there is no site; yield
   follows tonnage. CDG reports oil over a trap and none elsewhere.
5. `climateConsistent` (MIC) and `regionsFollowProvinces` (TerraBlender) pass.
6. The viewer shows the timelapse, event ticker, layers, ore provinces and three "Why here?" chains.
7. A scale comparison for the decisions file's playtest question: BlueMap renders of the same
   planet region at about 1:100, 1:300 and Earth-proportioned (2π × 6,370 km / 32,768 blocks ≈
   1,221 m per block at 32k), vertical scale unchanged, side by side on the review page.
8. "Deep Time — Phase 1 (MVP)" is published privately; the question of making the repository public
   goes to Ben (the decisions file keeps it private until the MVP).

**Top risks.** Whether Earth-normalized plates (§8.1) read well on a planet 4–8× smaller than Earth;
the time budget on a 4-core laptop; the climate → biome mapping producing odd Terralith layouts
(Tectonic needed a hand-merged 325 KB noise-settings file just to coexist with Terralith); deposit
counts and sizes that feel wrong in play; COE/CDG signature drift on pack updates; memory headroom
on this machine.

### 10.4 Phase 2 — 3D rock, rivers, lakes and caves (coarse)

- **P2-01 Noddy evaluator.** Port `unEvents.c`, `taste.c`, `matrix.c` from `noddyverse` (MIT);
  translate plate events into local events (§8.5); lattice evaluation at bake; fit to horizons and
  bodies.
- **P2-02 Metamorphic grade** from restored burial and heat; facies table (hand-built from textbook
  facies boundaries, or generated offline — see Q8).
- **P2-03 Hydrology** (§8.4): routing, Fill–Spill–Merge lakes, HAND floodplains, river vectors,
  channel carving, waterfalls, deltas; `water_level` for real.
- **P2-04 Geology-driven caves.** Karst in carbonates (port of `aparis69/Karst-Synthesis`, MIT) and
  cave density modulated by lithology and water table, with vanilla cave functions kept as the
  baseline so every region stays playable; a connectivity metric per region.
- **P2-05 Weathering blocks**: regolith, saprolite, laterite and bauxite caps, scree.
- **P2-07 Per-chunk overhead**: Chunky throughput in the same smoke with a vanilla preset vs Deep
  Time.

**Exit test.** For ten fold and fault events, `deeptime-inspect section` at their locations shows
horizon dips and offsets matching the events' parameters within recorded tolerances, and BlueMap
cliffs show them; every lake's surface equals its lowest rim ±1 block; every river path descends
monotonically to the sea or a lake; the cave connectivity metric meets its floor in every region;
Deep Time's chunk throughput is reported against vanilla's. **Risks:** no Cortial-to-Noddy prototype
exists anywhere, so visual credibility at block scale is unknown; hydrology cost at L2; cave balance.

### 10.5 Phase 3 — fantasy, sky islands, exotic landforms, Distant Horizons (coarse)

- **P3-01 Fantasy fields and deposits** (§5.4.2, §8.7).
- **P3-02 Sky islands** (§8.7) and **sky-biome steering** (§7.4) with the Terralith band file.
- **P3-03 Exotic landforms** (Paris et al. 2019 implicit features, MIT).
- **P3-04 Distant Horizons far terrain**: `IDhApiWorldGenerator` (package
  `com.seibel.distanthorizons.api.interfaces.override.worldGenerator`; `generateLod(…)` returning
  full data sources built from the rasters), registered with
  `IDhApiWorldGeneratorOverrideRegister.registerWorldGeneratorOverride`. The DH API is LGPL-3.0.
  MIC ships DH 3.3.0 while TW compiles against 3.3.1 and folds DH through internal mixins, so the
  exact DH version matters.

**Exit test.** Sections through sky islands show strata from their source crust; `probe column`
at island tops returns Terralith skylands biomes, and skylands biomes do not appear on open ocean
without an island above; fantasy deposits follow their rules; the DH generator's column-to-LOD
conversion is unit-tested against the rasters. How DH looks stays unverifiable (§9.8). **Risks:**
biome steering with 3D climate functions; DH API churn (API 4.0.0 renamed methods); TW's DH mixins.

### 10.6 Later

Paint-and-pin constraints in the editor (pin a continent, paint ocean, fix a range; decisions file,
log 2026-09-28: after the fantasy features). Pole transit (teleport to x + C/2 with z mirrored, heading and velocity turned 180°; Sable through
`PhysicsPipeline.teleport` plus a velocity rotation — rigid, so it works where a mirrored seam cannot,
`wrapped_planet_worlds.md` §5.1); the civ module (`deeptime-civ`, appending event-log segments as a
second writer and reading sites); space planets as dimensions (the generator already takes a world-data
folder per level stem); format migrations from 1.0; public release (Modrinth, signing per Q3); a port
plan for the 26.x density-function changes (`flat_cache`, `cache_2d`, `cache_all_in_cell` and
`shifted_noise` removed in 26.3 snapshots, per the report).

---

## 11. First session

### 11.1 Checklist, in order

1. **Read** as §1.3 says: items 1, 2 and 4 now (items 3 and 5–11 when a task cites them).
2. **Check for peers.** `ls /home/greencheetah/Projects/mic/mods/deep-time` should fail (no repo yet).
   If it exists, another session got there first: read its `STATUS.md` and continue from it instead.
3. **No questions are pending before P0-01.** Ben has decided the repository
   (`MinecraftIndustrialComplex/deep-time`, private), the licence (`LGPL-3.0-or-later`), the Java
   package (`com.minecraftindustrialcomplex.deeptime`), Earth-normalized tectonics, the docs move and
   the three licence acceptances (decisions file). Open questions that remain are in §14 with the task
   that needs each answer.
4. **Create the repository** (P0-01). Stage files before any Nix command (flakes see only tracked files):
   ```sh
   mkdir -p /home/greencheetah/Projects/mic/mods/deep-time
   cd /home/greencheetah/Projects/mic/mods/deep-time
   git init -b master
   # write CLAUDE.md (§11.2), AGENTS.md, STATUS.md (§12.1), README.md, COPYING, COPYING.LESSER,
   # NOTICE.md, .gitignore, flake.nix (§4.4), rust-toolchain.toml; copy the docs (P0-01's move steps)
   git add -A                             # before nix: an untracked flake.nix does not exist for Nix
   nix flake lock && git add flake.lock
   nix develop -c rustc --version         # rustc 1.98.1
   git commit -m "Bootstrap Deep Time repository"
   gh repo create MinecraftIndustrialComplex/deep-time --private --source . --push
   ```
   Then finish P0-01's docs move (pointers in `mic/Docs`) and its acceptance checks.
5. **P0-02** (Cargo workspace and CI), then **P0-03** (the Gradle module), then **P0-04** (server sets).
   The first session is done when P0-04's `mic` smoke report says `PASS`; if time runs short, stop at
   the end of any finished task.
6. **Update STATUS.md** (items done, a build-log entry, the next item, the plan revision), commit, and
   push `master`.
7. **Leave a memory pointer** in the mic project's auto-memory (a short entry naming the repo path,
   `docs/deeptime-implementation-plan.md` and `STATUS.md`), so later sessions started in
   `/home/greencheetah/Projects/mic` find the project.

### 11.2 The repository's `CLAUDE.md`

P0-01 wrote it; the repository copy is the only version. It carries the start-of-session reading
routine (§1.3), the peer check, the decisions rule of §12.3, the landing rules, the sub-agent policy
and the end-of-session routine (§12.2).

---

## 12. Progress and handoff

### 12.1 `STATUS.md`

The single place where progress lives. Template:

```markdown
# Deep Time — status

Phase: 0 · Next: P0-05 · Updated: 2026-10-02 (session <id>)
Plan revision: <git log -1 --format=%h -- docs/deeptime-implementation-plan.md>
MIC pack tested against: <git -C /home/greencheetah/Projects/mic/pack rev-parse --short HEAD>

## Tasks
| ID | Title | State | Date | Commit | Evidence |
|---|---|---|---|---|---|
| P0-01 | Repository bootstrap and the docs move | done | 2026-10-01 | abc1234 | gh repo view … → PRIVATE |
| P0-05 | Generator skeleton on the analytic test terrain | doing | 2026-10-02 | — | started by session <id> |
| … | | todo | | | |

## Review pages
| Name | URL | Version label | Date |

## Pending questions
| Date | Task | Question | Asked how |

## Build log
### 2026-10-02 — P0-05 (session <id>, plan revision <sha>)
What was done, what surprised us, numbers measured (with commands), what is next.

## Known issues and open questions
```

States: `todo`, `doing` (set with the date and session when starting, so peers see it), `blocked`
(with the reason and the question asked), `done` (only when every acceptance command passed; the
evidence column names the command output, log path or review page).

### 12.2 Handing off

Every session ends with a build-log entry that says what changed, what was measured, anything that
contradicts this plan, and the exact next step, and updates the header's plan revision and pack
commit. A new agent should need nothing but the repository: `CLAUDE.md`, `STATUS.md` and `docs/`.

### 12.3 Recording decisions

- Implementation details inside a task (class names, file splits, test tolerances once calibrated)
  are decided by the implementing agent and recorded in STATUS.md's build log.
- Anything that outlives the session and changes behaviour, scope, dependencies, licences, formats or
  what players see is asked of Ben through `AskUserQuestion` with a recommended option (at most four
  questions per call; unless he has said to work autonomously). While it is open, the question sits in
  STATUS.md's "Pending questions" table. When Ben answers, the agent (the one that asked) **appends a
  log entry** to the decisions file's last section — the date, the question and his answer quoted
  verbatim — updates any table row above it that the answer changes with "(see log YYYY-MM-DD)",
  points the §14 row at the log entry, and removes the question from STATUS, all in one commit. Agents
  never rewrite or delete earlier log entries and never edit anything else in the decisions file
  (decisions file, log 2026-09-28: "One file, append-only log").
- If a decision changes this plan, edit the plan in `docs/` in the same commit series and note it in
  the build log. The plan keeps no stale text: superseded content is replaced, not struck through, and
  git history keeps the old versions.
- Measured numbers replace estimates in the plan (§5.2, §5.7.5, §8 cost models) once P1-17 has run,
  with the date and the command that produced them.

---

## 13. Reuse vs build ledger

Licences were checked against each project's LICENSE file or registry metadata on 2026-09-27/28
(commit or version in the Source column). **Use** values: **embed** — a runtime dependency (a crate
compiled in, or a mod required or optional at runtime); **port** — translate its code with
attribution in `NOTICE.md`; **paper** — implement from the publication; **reference** — read for
design only, copy no code; **oracle** — unshipped comparison in `tools/validation/`; **tool** —
development-time only; **build** — Deep Time's own code.

LGPL-3.0 compatibility: MIT, BSD, Apache-2.0, ISC, Zlib and CC0 code can be combined into an
LGPL-3.0 work. CC BY 4.0 permits porting with attribution (the research notes' reading; attribution
goes in `NOTICE.md` and the file header). GPL code cannot be linked or ported. EUPL-1.2 is copyleft
and is avoided. PolyForm Shield (Sable) permits calling the API but not vendoring code or binaries.

| Component | Source | Licence | Use | Phase |
|---|---|---|---|---|
| Toroidal World 0.19.3 | anyttng/toroidal_world `3c1f3a30` (branch mc/1.21); Modrinth `8YTSYi9y` | MIT | embed (required mod; compiled against its jar) | 0 |
| ReTerraForged | Campione01/ReterraForged `ba35ca7` | MIT | port (TerraBlender mixins, density-function binding pattern, preset preview) | 0–1 |
| jni-rs 0.22.4 | crates.io | MIT OR Apache-2.0 | embed | 0 |
| rayon 1.12.0 | crates.io | MIT OR Apache-2.0 | embed | 0 |
| lz4_flex 0.14.0 | crates.io | MIT | embed (tile codec) | 0 |
| lz4-java 1.8.0 | on the 1.21.1 game classpath (`libraries/org/lz4/lz4-java/1.8.0`) | Apache-2.0 | used from the game's classpath; not shipped | 0 |
| memmap2 0.9.11, serde 1.0.229, serde_json 1.0.151, postcard 1.1.3, kiddo 6.3.0, glam 0.33.11, rand 0.10.3, rand_pcg 0.10.2, png 0.18.1 | crates.io | MIT and/or Apache-2.0 | embed | 0–1 |
| rustfft | crates.io | checked by `cargo deny` on adoption | embed (orographic precipitation) | 1 |
| cargo-zigbuild 0.23.4 (with zig 0.13.0 through a package override), cargo-xwin 0.22.0 with clang/lld/llvm 21.1.8 | nixpkgs | — | tool (cross builds; cargo-xwin fetches the Microsoft CRT/SDK under Microsoft's terms, accepted by Ben) | 0 |
| JUnit 5.14.4 (via `junit-bom`) | Maven Central | EPL-2.0 | test-only dependency; never shipped in the jar | 0 |
| Sable's native pipeline (LZ4-zipped natives, zigbuild + xwin on one runner) | ryanhcode/sable `6966d29` | PolyForm Shield 1.0.0 | reference (pattern only, no code) | 0 |
| Sable 2.0.5 API (`SubLevelAssemblyHelper`, `SubLevelContainer`, `PhysicsPipeline.teleport`, `RigidBodyHandle`) | the pack's pin (Modrinth `U678xqle`) | PolyForm Shield 1.0.0 | embed as optional API calls (pole cap, tests); never vendored | 0–1 |
| Cortial et al. 2019 "Procedural Tectonic Planets" | paper (CGF 38(2)) | — | paper; the three community ports are unlicensed and are not opened | 1 |
| tectonics.js `RockColumn`, `Crust` | davidson16807/tectonics.js `e43927c` | CC BY 4.0 | port (rock pools) | 1 |
| tectonics.cpp | davidson16807/tectonics.cpp | CC BY-NC-SA 4.0 | not used | — |
| PlaTec | Mindwerks/plate-tectonics `2a27c4f` | LGPL-3.0 | reference only (2D, unitless) | — |
| GPlates / pyGPlates / GPlately | GPlates | GPL-2.0-only / GPL-2.0 | oracle and a later export target (`.rot`/`.gpml` files are data) | later |
| Braun & Willett 2013; Yuan et al. 2019; Barnes et al. 2014, 2016; Cordonnier et al. 2019 | papers | — | paper | 1 |
| fastscapelib | fastscape-lem/fastscapelib `b85cc6b` | GPL-3.0 | oracle | 1 |
| goSPL | Geodels/gospl `6c5b9e4` | GPL-3.0 | oracle; its technical guide's documented equations (duricrust, ELA) implemented from the docs | 1–2 |
| RichDEM; Barnes2013-Depressions | r-barnes | GPL-3.0 (not re-verified); no licence file | not used | — |
| Landlab components (weathering, LithoLayers, threshold eroder, submarine diffuser, carbonate, HAND) | landlab/landlab `7c60bd6` | MIT | port | 1–2 |
| Schott et al. 2024 multi-scale erosion | H-Schott/MultiScaleErosion `964a3d5` | MIT | port (GLSL passes → CPU Rust) | 1 |
| Schott et al. 2023 stream power | H-Schott/StreamPowerErosion `8a0c017` | MIT | reference | 1 |
| Tzathas et al. 2024; FastFlow; glacial erosion (Inria "landscapes") | papers; code research-only | Inria research licence | paper only, optional later | later |
| climlab EBM | climlab/climlab `fcbde37` | MIT | port (equations) | 1 |
| Smith & Barstad orographic precipitation | fastscape-lem/orographic-precipitation `baf8a0c` | MIT | port | 1 |
| LFPM (Hergarten & Robl 2022) | paper; OpenLEM code | OpenLEM GPL-2.0-or-later | paper only (alternative) | 1 |
| ExoPlaSim; SpeedyWeather | alphaparrot/ExoPlaSim; SpeedyWeather.jl | GPL-2.0; EUPL-1.2 | oracle (optional climate calibration table) | later |
| Fill–Spill–Merge + depression hierarchy | r-barnes/Barnes2020-FillSpillMerge `1c499ea`; r-barnes/Barnes2019-DepressionHierarchy `72bb3f5` | MIT | port | 2 |
| Noddy inverse kinematics | Loop3D/noddyverse `6a21b23`; Loop3D/noddy_qt `52ca78d` | MIT | port | 2 |
| LoopStructural; map2loop | Loop3D `bd57793`; `078f8a6` | MIT | reference (fault and fold maths) | 2 |
| Karst-Synthesis | aparis69/Karst-Synthesis `386b243` | MIT | port | 2 |
| Paris et al. 2019 implicit 3D features | aparis69/Implicit-Volumetric-Terrains | MIT (per research notes) | port | 3 |
| MAGEMin; Perple_X | ComputationalThermodynamics/MAGEMin; perplex.ethz.ch | GPL-3.0; no formal licence | not used unless Q8 says otherwise | 2 |
| Mineral-systems rules (Skirrow 2019; McCuaig 2010; Hronsky & Groves 2008; Butterworth 2016; Hoggard 2020) | papers | — | build (the ore engine is Deep Time's own) | 1 |
| USGS grade–tonnage and density data (OFR 2008-1155, 2009-1034, 2009-1252, 2011-1058; SIR 2010-5070; B1693) | USGS | US public domain | data (JSON tables with source lines) | 1 |
| MapMark4 | USGS-R/MapMark4 | CC0 | reference (three-part method) | 1 |
| Lopatin TTI (Waples 1980) | paper | — | paper | 1 |
| Excavated Variants 4.3.1 (`Ore.getStones()`, `RegistryKeys`; requires Dynamic Asset Generator) | lukebemishprojects/ExcavatedVariants `d399501` (1.21.x); Modrinth `krUiU6UD` | LGPL-3.0 | embed (optional API) | 1 |
| TerraBlender 4.1.0.8 | Glitchfiend/TerraBlender | LGPL-3.0 | optional compat target; test runtime | 0–1 |
| Biomes O' Plenty 21.1.0.14, GlitchCore | Modrinth | — | test runtime only | 0–1 |
| Terralith 2.6.2, Lithostitched 1.8.0+beta6, Tectonic 3.0.28, C2ME 0.4.0-alpha.0.122, Distant Horizons 3.3.0 | the current pack's pins (2026-09-28) | — | compat targets; MIC runtime only | 0–1 |
| Create Ore Excavation 1.6.8; Create: Diesel Generators 1.3.15 | the current pack's pins | — | mixin targets (our mixins; their code not copied) | 1 |
| Distant Horizons API (`IDhApiWorldGenerator`) | distant-horizons-team (GitLab) | LGPL-3.0-only | embed (optional API) | 3 |
| Chunky 1.4.23 | pop4959/Chunky `ab45b8b` | GPL-3.0 | tool (a separate mod in smoke tests; never linked) | 0 |
| BlueMap 5.28 CLI | BlueMap-Minecraft/BlueMap `b80ecb7` | MIT | tool (review renders) | 0 |
| mic-climate test harnesses (`gametest.sh`, `smoke/`, `gametest-client.sh`) | Ben's own repo | MIT | port (adapt scripts) | 0 |
| HeadlessMC; mc-runtime-test | headlesshq | MIT (per the mic auto-memory note) | tool (best-effort client boot) | 1 |

**What Deep Time builds itself:** the session and stage runner, the plate driver (from the paper), the
coupling between sphere and rasters, the deposit engine and its data, the bake, the world-data format
and both its readers, the generator and density functions, the strata, water, deposit and scatter
passes, ore claiming, the editor and timelapse, first-start generation, the COE/CDG/TW/Sable/EV
integrations, the native loader, the viewer and `deeptime-inspect`.

---

## 14. Open questions

**Answered by Ben on 2026-09-28** (now in the decisions file): Q1 tectonics scaling (Earth-normalized,
§8.1), Q2 repository (`MinecraftIndustrialComplex/deep-time`, private), Q9 Java package
(`com.minecraftindustrialcomplex.deeptime`), Q12 the three licence acceptances (accepted, §2.7), the
licence identifier (`LGPL-3.0-or-later`) and the docs location (moved into the repo at P0-01, §1.2).
Answered on 2026-09-28 in the decisions file's log: Q5, Q13, Q14 and Q16 (their rows below point to it).
**Resolved by verification:** Q7 — MIC's KubeJS scripts do not use CDG's `calculateOilChunks` hook
(§7.14), so Deep Time's injection bypasses nothing of MIC's.

Still open. Each has a recommendation; ask when the task in the last column starts.

| # | Question | Recommendation | Needed by |
|---|---|---|---|
| Q3 | **Windows DLL signing** against antivirus false positives (Sable #691). | No signing while private. At public release apply to SignPath Foundation (free for OSI-licensed open source, which LGPL-3.0-or-later is; needs a released, maintained project, CI-only signing and 2FA). Azure Artifact Signing is the paid fallback (individual accounts US/Canada only). Until then, the guidance text and `-Ddeeptime.natives.path`. | release |
| Q4 | **Remove random rock-blob features in Deep Time worlds** (vanilla granite/diorite/andesite/tuff blobs, Terralith ReStoned's restored ones, Create's striated layers), since the simulation owns lithology? | **Answered** (decisions file, log 2026-09-28): remove, on by default, switchable per pack. | P1-15 |
| Q5 | **Build range per world.** The dimension type is data, fixed at −352..639; a per-world range would need a per-world datapack. | **Answered** (decisions file, log 2026-09-28): fixed at −352..639 until 1.0. | P0-05 |
| Q6 | **When to abandon Toroidal World** for Deep Time's own seam teleport. | Only if one of P0-06 step 6's triggers holds (no way to shape a world created from Deep Time's preset; Sable crossing fails on TW ≥ 0.19.3; the MIC pack will not boot with TW and configuration cannot fix it), and only after asking. | P0-06 |
| Q8 | **Metamorphic facies table:** hand-built from textbook facies boundaries, or generated offline with MAGEMin (GPL-3.0) or Perple_X (no formal licence), whose output-data status is unclear. | Hand-built, with each boundary cited. | P2-02 |
| Q10 | **Excavated Variants in MIC.** It is not in the pack (and needs Dynamic Asset Generator); without it, ore in non-stone hosts falls back to stone/deepslate ores. | **Decided** (decisions file, log 2026-09-28): add EV + DAG on MIC's deep-time pack branch and fill its gaps (argillite stones, lead and silver ores); evidence at https://spacesheep.dev/@benmac/deeptime-q10-ev. | P1-15 |
| Q11 | **Pinning Toroidal World in MIC.** TW is pre-1.0 with no world-compatibility promise between versions, and mixes into mods MIC updates (C2ME, DH, Sable, Create). | Pin TW's Modrinth version id in the pack, record the C2ME/DH/Sable versions it was built against, and upgrade TW only after a Deep Time smoke passes. | P0-06 |
| Q13 | **"Why here?" in v1.** The decisions file puts the timelapse only on the Create World screen, and prospecting is planned separately. | **Answered** (decisions file, log 2026-09-28): site API + `/deeptime why <site>` + the HTML viewer; an in-game screen waits for the prospecting plan. | P1-14 |
| Q14 | **Paint-and-pin constraints** have no phase in the decisions file. | **Answered** (decisions file, log 2026-09-28): after the fantasy features ("later", §10.6). | — |
| Q15 | **Rock defaults outside MIC.** The rock tags ship in MIC, but Deep Time must also work on pure vanilla. If Deep Time shipped its own fallback tags, their entries would come before a pack's appended entries and override its chosen stand-ins. | Deep Time ships only vanilla member tags plus a `vanilla_fallback` per class in its `rock_class` data (§7.5); packs keep full control of fallback order. | P1-12 |
| Q16 | **How MIC ships the rock-tag datapack.** The decisions file says it ships in the MIC pack, but `pack/` has no copy and no global-datapack loader (checked at pack commit `303fd89`). The pack does run KubeJS, which loads `kubejs/data/` into every world, and already uses it (`pack/kubejs/data/destroy/`, `…/powergrid/`, and others). | **Answered** (decisions file, log 2026-09-28): copy the datapack to `pack/kubejs/data/deeptime/`. | P1-12 |
