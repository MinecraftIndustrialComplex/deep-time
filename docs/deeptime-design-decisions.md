# Deep Time — design decisions

Decisions Ben made in the pre-implementation Q&A, 2026-09-27. The research behind them is in
`Geologic history worldgen mod.md` and in `research_notes/Geologic history worldgen mod/`, which
includes `aeronautics_rust_bridge.md` and `wrapped_planet_worlds.md`. Everything below is
customizable unless stated otherwise; the listed value is the default.

## Product and scope
| Topic | Decision |
|---|---|
| Name | **Deep Time**, mod id `deeptime` (the Modrinth slug was free on 2026-09-27) |
| Audience | MIC first, public later; stays pack-agnostic |
| Platform | NeoForge 1.21.1 only; the Minecraft-free core keeps later ports cheap |
| Licence | **LGPL-3.0-or-later** (SPDX `LGPL-3.0-or-later`). No GPL code in the project. GPL tools (fastscapelib, goSPL, pyGPlates…) appear only as test oracles in unshipped tests, and GPL code is never ported line by line |
| Space planets | The planet model is parameterized from day one (radius, gravity, day length, tilt, star flux, ocean fraction, tectonic regime); v1 tunes only Earth-like presets |
| Civ / political sim | A later module that shares the world data and the event log, built after the geology MVP |
| Repo | `MinecraftIndustrialComplex/deep-time`, private until the MVP, checked out at `mods/deep-time`; the layout is a Cargo workspace (core, CLI, JNI) plus a Gradle NeoForge module and a Nix flake. Java package `com.minecraftindustrialcomplex.deeptime` |
| Docs location | When P0-01 creates the repo, the plan, this decisions file, the research report and the notes move into its `docs/` folder, versioned with the code; `mic/Docs` keeps a pointer |
| Licence acceptances | Ben accepted (2026-09-28): the Minecraft EULA for headless test servers, BlueMap's `accept-download`, and Microsoft's CRT/SDK terms via cargo-xwin |
| Planner / implementer | Opus writes the implementation plan and implements it (no Fable) |

## Engine and runtime
| Topic | Decision |
|---|---|
| Engine | **A Rust core called through JNI**, following Sable's pattern: a synchronous JNI call on a background thread, with Rayon inside. Java owns the screens, the generator and the Minecraft hooks |
| Native targets | Windows/macOS/Linux × x64/arm64, plus musl builds for x64 and arm64 (8 in total); conservative CPU baseline. A failed native load shows guidance instead of crashing |
| Standalone | Every stage also runs standalone through the CLI and writes to an HTML viewer (a development requirement) |
| Determinism | **Stored data only**: the world folder holds the authoritative data, and a seed need not reproduce a world bit-for-bit |
| History time budget | About 5 min for the quick preset and 15 min for the full one on a 6–8-core PC |
| Memory | Estimated from the preset before running; 4 GiB default cap, freely raisable (a 64 GB machine can run a huge simulation); big planets stream tiles to disk |
| History storage | One shared event log (what, where, when, cause) feeds the timelapse, "Why here?" and a future legends view |
| Timeline | Chosen by the player at setup; there is no fixed default story |
| Save compatibility | World data carries a format version. No promise before 1.0 (an update may refuse to extend old worlds); migrations are guaranteed from 1.0 |
| Sides | **Required on both client and server** |

## World shape and scale
| Topic | Decision |
|---|---|
| Where worlds live | The overworld of a new world, through a world type. The generator works in any dimension, so future planets can be dimensions |
| Wrapping | **Real planet wrap.** East–west wrapping uses **Toroidal World** (MIT) as a dependency: an X-only cylinder, with Deep Time registering its own shape through TW's API and rasters that wrap in longitude. A Phase 0 spike comes first; if TW fails, the fallback is our own teleport at the seam |
| Poles | A soft cap at about ±75–80° (pushback and storms) first. Later, a visible pole transit: teleport 180° of longitude away with the heading reversed, for players and Sable ships. A seamless pole is geometrically impossible on Minecraft's grid (it would be a mirror seam) |
| Circumference | 16,384 blocks for singleplayer, 32,768 for servers; pole to pole is half |
| Tectonics scaling | Earth-normalized: plate count, speeds and ages as on Earth, mapped onto the smaller planet; surface physics (erosion, rivers, slopes) stays in real metres at map scale |
| Horizontal scale | A middle ground of about 1:300 (radius about 1,565 km at 32k), to be playtested against about 1:100 and Earth-proportioned; customizable in both directions |
| Vertical scale | 20 m per block |
| Build range | y −352 to 639, **fixed for every Deep Time world until 1.0** (per-world ranges after 1.0); the vertical scale stays customizable within it |
| Local features | Enlarged to playable size (deposits, rivers, canyons, strata), with per-feature multipliers |

## Terrain, climate, water, caves, fantasy
| Topic | Decision |
|---|---|
| Sub-grid detail | Erosion-consistent amplification (in the style of Schott et al. 2024) plus a drama slider |
| Biomes | The physical climate drives vanilla's climate parameters; the planet spans pole to equator |
| TerraBlender regions | Follow simulated geography from v1 (ReTerraForged's MIT redirect pattern) |
| Rivers and lakes | Phase 0 proves that water above sea level holds; full lakes and carved rivers come in Phase 2 |
| Caves | Driven by geology, balanced so the world stays fully playable |
| Fantasy terrain | **Simulated by the engine**, not passed through. All three sets are in scope: fantasy fields plus fantasy deposits, sky islands, and exotic landforms. Mod sky biomes (e.g. Terralith's Skylands) are steered onto Deep Time's islands through climate values at island height |
| Distant Horizons | Far terrain fed straight from the rasters through `IDhApiWorldGenerator`, in a later phase (Phase 3) |

## Blocks and removability
| Topic | Decision |
|---|---|
| Rock blocks | The backend supports every rock class. Blocks come from tags (vanilla or bring-your-own). **Deep Time ships no blocks of its own and stays 100% vanilla / BYO-block compatible** |
| Removal | Best effort, blocks only: a baked world contains no Deep Time blocks. Cleaning up `level.dat` and the dimension type after removal is left to the player |
| Create's ore stones | Keep their geological classes, and each is a preferred host for its Create metal (crimsite → iron, ochrum → gold, veridium → copper, asurine → zinc) |
| Ore blocks | Excavated Variants' ore-in-rock blocks when it is installed; otherwise normal stone or deepslate ores |
| Other mods' ores | Materials that Deep Time simulates lose their scatter. Unknown ores keep scattering, gated by host rock. Vanilla's large ore veins are off |
| MIC rock tags | `datapacks/deeptime-mic-rock-tags/`, which ships **in the MIC pack**, not in Deep Time. 59 classes. Stone and deepslate are `unclassified_crust` filler. No Nether/End blocks and no terracotta. Ice and Fire's ash counts as volcanic ash; charred stone is excluded. No Create Nuclear blocks: Create Nuclear is leaving MIC for CROWNS, and uranium will map to CROWNS blocks |
| Tag rules (backend) | Always use the first installed fallback entry. Rock mappings support block-state entries through the adopted `data/<ns>/deeptime/rock_states/<name>.json` format (class, member/fallback role, entries of block + state + required; e.g. Project Atmosphere `dust` at `layers=8` as loess) |
| Coal stand-in | Born in Chaos black argillite, then `minecraft:stone` (the one class allowed to fall back to filler stone) |

## Ores, prospecting, MIC hooks, playability
| Topic | Decision |
|---|---|
| First deposit types | Porphyry Cu + placer Au; iron and coal; Zn–Pb (MVT/SEDEX); oil (petroleum systems) |
| Prospecting | Planned separately and tied into MIC progression. Generation must reward exploration and push players to set up bases in different regions |
| Regional specialization | Configurable (assumed default: strong provinces) |
| Unlimited-mining sites | A future system limits unlimited mining to locations fixed by generation. The engine exports a **site registry + Java API** (location, extent, grade, type, forming event) |
| COE / CDG | Optional compat mixins inside Deep Time: `OreVeinGenerator.pick(LevelChunk)` and `OilChunksSavedData.getBaseOilAmount` |
| Playability overrides | Regional minimums for essentials; a spawn chosen for its essentials; a pack-defined list of essentials (datapack/config); background ore scatter. Pack-specific fudging is expected |

## UI and servers
| Topic | Decision |
|---|---|
| History display | A timelapse in the Create World screen only (globe/map, scrubber, event ticker, layer toggles) |
| Customization | A full in-game editor with shareable presets; custom content through datapacks. Paint-and-pin constraints are a nice-to-have scheduled **after** the fantasy features ("later"). No heightmap/GPlates import |
| Dedicated servers | Seamless: a server started with the Deep Time world type and a preset generates on first start, with progress on the console only (RCON is not up yet) |

## Testing and review (Ben cannot launch the game)
Everything is planned around what Claude can run and Ben can review in a browser:
- Rust unit and property tests, plus golden-output checks against fastscapelib and goSPL (unshipped).
- The standalone HTML viewer for every stage.
- NeoForge GameTests: generator, strata, deposits, Terralith and TerraBlender presence, and server-side Sable ship crossings of the seam.
- A real-pack headless server smoke test over RCON, both standalone and inside MIC.
- BlueMap CLI 3D web renders of generated worlds, with the MIC jars used for textures.
- Our own top-down, rock, ore and cross-section renders taken from region files.
- Review builds published as private spacesheep pages.

Not reviewable under this plan: client-only visuals (the timelapse screen in-game, how DH looks, how the seam feels while flying).

## Phases
- **Phase 0:** Rust↔JNI; the Toroidal World wrap; the generator on a static wrapped raster; Terralith/BOP checks; above-sea water; BlueMap on MIC blocks.
- **Phase 1:** plates, erosion, climate, biomes and TerraBlender regions, plus the four deposit families and the COE/CDG hooks.
- **Phase 2:** 3D rock (Noddy-style), rivers, lakes and caves.
- **Phase 3:** fantasy fields and deposits, sky islands, exotic landforms, DH far terrain.
- **Later:** pole transit, paint-and-pin constraints, the civ module, space planets.

## Decided during implementation (append-only log)
**This file is the single authority for Ben's decisions.** When Ben answers a question, the agent appends
an entry here with the date, the question and his answer quoted verbatim, then updates any table row
above that the answer changes, noting "(see log YYYY-MM-DD)". Agents never rewrite or delete earlier
log entries. `STATUS.md` holds only questions still pending; the plan's open-questions table points to
the log entry once a question is answered. A newer log entry overrides an older one.

| Date | Question | Ben's answer |
|---|---|---|
| 2026-09-28 | How are future decisions recorded? | "One file, append-only log" (this section) |
| 2026-09-28 | How does MIC ship the rock-tag datapack? (plan Q16) | Copy it to `pack/kubejs/data/deeptime/` (KubeJS loads it into every world) |
| 2026-09-28 | Build range per world? (plan Q5) | Fixed at −352..639 until 1.0 |
| 2026-09-28 | "Why here?" in v1? (plan Q13) | Site API + `/deeptime why <site>` + the HTML viewer; an in-game screen waits for the prospecting plan |
| 2026-09-28 | When do paint-and-pin constraints arrive? (plan Q14) | "I'd put them even after the fantasy features, it's a nice to have but not something I think I'll personally use" |
| 2026-09-28 | When does Deep Time go into the MIC pack? | "don't merge your mod into the actual MIC pack immediately, at best keep it on a branch for now." Pack changes for Deep Time (the mod, the rock-tag datapack of Q16) go on a pack branch, never straight onto the pack's master |
| 2026-09-28 | Remove random rock-blob features in Deep Time worlds? (plan Q4) | "Remove, on by default (Recommended)": Deep Time worlds drop vanilla, Terralith, Create and Born in Chaos rock-blob features; a pack can switch it off; other worlds untouched |
| 2026-09-28 | Excavated Variants in MIC? (plan Q10) | "Decide after renders (Recommended)": side-by-side BlueMap renders of the same deposits with and without EV in the MIC pack come first; the decision follows from the pictures |
| 2026-09-28 | How should Ben watch a simulation while it runs? | "Live page served by the Mac (Recommended)": one command starts a run on the Mac and serves a viewer at `http://mac-mini:<port>`, on the tailnet only and never published; map, globe, layers and events update as each step lands, with pause, rewind and speed; it can also open fullscreen on the Mac's own screen. No start form. |
| 2026-09-28 | Should Create Ore Excavation veins be finite in Deep Time worlds? (MIC ships COE's default `defaultInfinite = true`, so the tonnage quantile P1-16 stores changes no yields) | "Decide later": park it until the real presets exist (after P1-08) and there are numbers to judge vein sizes by. |
| 2026-09-28 | Should Deep Time get a browser workbench on the Mac for trying things out without Minecraft? | "Workbench + 3D terrain (Recommended)": the live page grows into a workbench on the tailnet: the Create World editor's controls in the browser (preset, continents, sliders, seed, share codes), live runs and side-by-side comparison of runs; after P1-08, click-to-inspect columns (rock layers, ore bodies, "why here") and a 3D block preview of an area built from the same Rust code the mod uses. Staged: controls and live runs first, inspection and 3D once P1-08 lands. This widens the earlier "no start form" answer. |
| 2026-09-28 | Q10: Excavated Variants in MIC for Deep Time worlds? (evidence: https://spacesheep.dev/@benmac/deeptime-q10-ev) | "Keep EV, fill its gaps (Recommended)": EV + Dynamic Asset Generator go on MIC's deep-time pack branch (never pack master, per the 2026-09-28 branch-only entry), with EV stone configs for Born in Chaos's argillites and ore configs for Create Nuclear lead and Ice and Fire silver. Note: the question said "keep", but EV is not in MIC today (plan Q10); in the comparison "keep" meant the world with EV, so this answer means adding it on that branch. |
| 2026-09-29 | With real climate (P1-05) the continents end 2.5–3 km high and about half the land is ice cap or tundra; deposits are scarce (5 on a 16k planet, no Zn–Pb or oil). When to recalibrate? | "Now, beside P1-06 (Recommended)": an agent recalibrates erosion and uplift with climate on, turns the ice-sheet feedback on once elevations look Earth-like, and retunes deposit counts, in parallel with P1-06; P1-17 later does the final numbers. |
| 2026-09-29 | Which jar set should the workbench's 3D area preview generate with by default? | "MIC (Recommended)": the full MIC pack (~90 s per box) rather than standalone Deep Time (~45 s). |
| 2026-09-29 | Workbench redesign choices: accent colour, TV-mode idle behaviour, placeholder presets | "Keep indigo (Recommended)" (#7d8cff); "Replay on loop (Recommended)" (TV mode replays the latest run after 25 s idle); "Keep the dev toggle" (placeholder presets hidden behind "Show placeholder presets (dev)"). |
| 2026-09-29 | Default memory cap for workbench runs on the Mac (core default 4 GiB; a pipeline_full_32k 1000 Myr run failed 15 min in at the cap while the page said it could run) | "8 GiB (Recommended)": the workbench's default cap is 8 GiB, still changeable per run in the editor; the workbench refuses runs whose estimate exceeds the cap before they start. The core's and the in-game editor's defaults are unchanged. |
| 2026-09-29 | Can a run that takes too much memory slow down instead of being cancelled? | "Spill history to the SSD (Recommended)": history frames stay in memory up to the cap, then the oldest go to the SSD and are read back when needed; memory stays roughly flat at any length, runs over the cap get slower instead of failing, stages still fail cleanly if the base grids alone don't fit, results bit-identical. (No hard-cap removal / swapping.) |
| 2026-09-29 | Deposit counts: keep calibration's per-type multipliers, or change the §8.6 formula? | "Change the formula (Recommended)": treat the USGS densities as deposits that survive to the present (what USGS counts) and let the engine form extra so the survivors match; no per-type magic multipliers. |
| 2026-09-29 | Meaning of the editor's "ocean fraction" on the pipeline presets | "Water's share (Recommended)": the share of the surface under the sea (Earth 0.71); continents get flooded shelves (`tectonics.continental_shelf_share`). The plate-only presets keep P1-03's crust meaning. |
| 2026-09-29 | MVT zinc–lead deposits appear on only 3 of 6 test planets (they need carbonate in basins, which is scarce and erodes) | "Rework its driver": redesign MVT as a regional driver (basin-wide fluid flow) so most planets get some. |
| 2026-09-29 | P1-06: how strong should terrain carving (amplification's incision cap) be by default? | "Again, shoud be configurable. For now go for Medium": default medium (`amplify.incision_cap` 3.0), exposed as a setting in the editor and the workbench. Standing rule from this answer: tuning knobs that shape the world should be user-configurable settings, with a sensible default, rather than fixed constants. |
| 2026-09-29 | Run P1-06's young-landscape and amplification steps in every workbench run (+20–60 s)? | "Yes, always (Recommended)". |
| 2026-09-29 | P1-08 made the `earthlike_*` presets real (identical to the calibrated `pipeline_*`). Which family should the workbench list? | "Show only Earth-like (Recommended)": the workbench lists only Earth-like quick/full × 16k/32k; `pipeline_*` stay as internal/test presets. |
| 2026-09-29 | Eight editor fields have no simulation setting behind them yet ("planned", hidden). Wire them up or keep them hidden? | "Wire them up (Recommended)": give them real core settings (defaults reproduce today's calibrated worlds). |
| 2026-09-29 | Porphyry comes out at median 5.5 per 16k planet with the survivor formula (USGS P50); raise it? | "Use the USGS mean density (Recommended)": use the mean of the USGS density distribution per type, only where a real USGS figure exists (placeholder densities are not inflated). |
| 2026-09-29 | What counts as a "surviving" deposit for the density target? | "Within ~1 km (Recommended)": only deposits within about 1 km of the surface count toward the USGS density target (as USGS counts); deeper deposits come on top. |
| 2026-09-29 | SEDEX (on 3 of 6 planets) and coal (~21 seams per planet) rest on placeholder densities | "Research real densities (Recommended)": find published densities / grade-tonnage studies for SEDEX and coal and set them from sources. |
| 2026-09-29 | The MIC GameTest run now runs out of heap in `lakeHolds` on every run | "Chase the stall (Recommended)": find and fix the cause; prove MIC GameTests pass reliably. |
| 2026-09-29 | Make the biome mapping's remaining constants settings (inland distances, wetland thresholds, band placement per climate zone)? | "All of them": inland distances and wetland thresholds as Advanced settings, and the band placement per Köppen class as a pack-overridable data file. |
| 2026-09-29 | Deposit counts under the 1 km rule (deep extras) and coal's placeholder density | "OK; define coal as seams (Recommended)": keep the counts; a coal deposit is one minable seam field (tens of km across), with its density sized from coal-basin area statistics, labelled an estimate. |
| 2026-09-29 | Coal as one seam field per deposit (~30×45 km, 14 per 100,000 km²): keep, let fields overlap, or smaller/more? | Ben asked "What is the most natural / realistic?"; answer given: in real coal basins the fields adjoin and seams are stacked. Follow-up answer: "Yes, adjoining + stacked (Recommended)": coal fields in a basin may adjoin and fill its minable area (no spacing rule), and each field holds a stack of several seams at different depths, count and spacing from the basin's sediment thickness. |
| 2026-09-29 | Mountains look like smooth domes up close (P1-08 render). Add block-scale mountain character driven by the simulation? | "Do it now, in Phase 1 (Recommended)": ridgelines and valleys following simulated drainage, cliffs/crags on hard rock and gentler slopes on soft rock, sharper peaks where uplift is young, a ruggedness setting, the striped-snow artifact fixed; proven with side-by-side 3D renders against vanilla and Terralith. Scoped to terrain shape, not Phase 2's 3D rock. |
