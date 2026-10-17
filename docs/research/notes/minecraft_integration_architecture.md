# Minecraft integration architecture: bridging a geologic-history simulation to a bounded NeoForge 1.21.1 world

Source conventions. Links starting `jar:file://…neoforge-21.1.249-sources.jar!/` point into the decompiled NeoForge 21.1.249 / Minecraft 1.21.1 sources on Ben's machine, and I read each one directly this session (line numbers are approximate to ±2). Links into `…client-extra-aka-minecraft-resources.jar!/` are vanilla 1.21.1 data, also read directly. I read GitHub sources for TFC, ReTerraForged, C2ME and WorldPainter myself, via blobless git clones or raw file fetches pinned to the commit or branch named. Facts that came from sub-agents' web reads and that I did not re-open myself are marked "(agent-read)". Where the agent flagged doubt, it is marked "(agent-read, low confidence)". Research date: 2026-09-27. Web search quota ran out partway through, so a few prior-art gaps remain. They are listed under Gaps.

## 1. Which integration strategy: (a) offline bake, (b) partial bake, (c) runtime ChunkGenerator, or (d) density-function injection? Recommendation

### Takeaway
Build on strategy (d) as a hybrid. The generator is a `GeologicChunkGenerator extends NoiseBasedChunkGenerator`; the class is non-final in 1.21.1, so subclassing works. Its noise router's climate and terrain functions are custom `DensityFunction` types that sample the simulation's rasters. Everything else is inherited from the installed pack: surface rules, aquifers, carvers, the `MultiNoiseBiomeSource`, and structures. Two passes go on top: a per-column strata/lithology pass after the surface step, and a geology-driven ore feature. Offline baking fails the core goals. With (a) the installed mods never decorate or add structures. With (b) the game really does resume proto-chunks in 1.21.1, but structures and consistent carver fluids are lost and the bake costs gigabytes. Strategy (c) forfeits the multi-noise/TerraBlender biome pipeline. It also forces re-implementation of carvers, aquifers and base-column queries; TerraFirmaCraft shows what that costs.

### Cited Findings

**(a) Offline full bake (Amulet, Querz NBT, fastnbt, custom writer)**
- The loader accepts any chunk NBT that has a `Status` string: `isChunkDataValid(tag) { return tag.contains("Status", 8); }` — [ChunkMap.java L595-597](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/server/level/ChunkMap.java)
- Chunks already saved at a status load through `ChunkPyramid.LOADING_PYRAMID`. In that pyramid, every step from `structure_references` through `features` and `spawn` is a pass-through. `structure_starts` only runs `loadStructureStarts`, which just calls `onStructureStartsAvailable`. Only `initialize_light`, `light` and `full` do real work. So `applyBiomeDecoration`, `spawnOriginalMobs` and structure placement never run for a baked `full` chunk — [ChunkPyramid.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/status/ChunkPyramid.java); [ChunkStatusTasks.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/status/ChunkStatusTasks.java)
- Lighting: `isLighted(chunk) = status ≥ LIGHT && chunk.isLightCorrect()`. `ChunkSerializer.read` sets `setLightCorrect(tag.getBoolean("isLightOn"))`. A writer that omits `isLightOn` (or writes 0) therefore gets its chunks relit by the game's light steps — [ChunkStatusTasks.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/status/ChunkStatusTasks.java); [ChunkSerializer.java L93, L216](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/storage/ChunkSerializer.java)
- Missing heightmaps are recomputed. The loader adds every absent key in `getChunkSaveHeightmaps()` to a set and calls `Heightmap.primeHeightmaps` on it. A section without `biomes` is filled with `Biomes.PLAINS` — [ChunkSerializer.java L217-229, ~L125](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/storage/ChunkSerializer.java)
- Palette sizes: block states use `PalettedContainer.Strategy SECTION_STATES = new Strategy(4)`, i.e. 16³ = 4096 entries. Biomes use `SECTION_BIOMES = new Strategy(2)`, i.e. 4³ = 64 entries per section. This settles a conflict in the agent's wiki read, which claimed 4096 biome entries — [PalettedContainer.java L356, L367](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/PalettedContainer.java)
- Structures exist only if the writer serializes them into the `structures` compound (`setAllStarts(unpackStructureStart(...))`, `setAllReferences(...)`) — [ChunkSerializer.java L230-232](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/storage/ChunkSerializer.java)
- `SharedConstants.WORLD_VERSION = 3955` for `VERSION_STRING = "1.21.1"`. The wiki agrees: 1.21 = 3953, 1.21.1 = 3955 — [SharedConstants.java L37, L47](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/SharedConstants.java); [Data version (agent-read)](https://minecraft.wiki/w/Data_version)
- Chunks with an older DataVersion go through DataFixerUpper on read (`ChunkMap.upgradeChunkTag`). Schema 3450 runs `RemapChunkStatusFix`, "Remove liquid_carvers and heightmap chunk statuses", mapping `minecraft:liquid_carvers → minecraft:carvers` and `minecraft:heightmaps → minecraft:spawn`. Schema 3451 runs `ChunkDeleteLightFix`, which removes `isLightOn` and every section's `BlockLight`/`SkyLight` — [ChunkMap.java L945-949](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/server/level/ChunkMap.java); [DataFixers.java L1254-1265](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/util/datafix/DataFixers.java); [ChunkDeleteLightFix.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/util/datafix/fixes/ChunkDeleteLightFix.java)
- Mod data: `ChunkSerializer.read` restores NeoForge data attachments only when the NBT already holds `AttachmentHolder.ATTACHMENTS_NBT_KEY`, then fires `ChunkDataEvent.Load`. A FULL chunk read from disk is returned as `new ImposterProtoChunk((LevelChunk)…, false)`, and its block entities wait in the `PostLoadProcessor` of `postLoadChunk` until `ChunkStatusTasks.full` runs `runPostLoad`: read below FULL (`getChunk(…, FEATURES, true)`) it has no block entity object and no pending tag (the placer GameTest's second MIC failure, 2026-10-17). NeoForge's `ChunkEvent.Load` passes `newChunk = !(protochunk instanceof ImposterProtoChunk)`, so mods never see baked full chunks as newly generated. Sources: [ChunkSerializer.java L214-215, L249, ~L275](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/storage/ChunkSerializer.java); [ChunkStatusTasks.full](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/status/ChunkStatusTasks.java)
- Offline-writer ecosystem health:
  - Amulet Map Editor claims support for "all versions since Java 1.12" (0.10.56 is the latest seen), but its issue #785 reads "Partially generated chunks are not converted" (agent-read) — [Amulet-Map-Editor](https://github.com/Amulet-Team/Amulet-Map-Editor); [issue #785](https://github.com/Amulet-Team/Amulet-Map-Editor/issues/785)
  - fastanvil's README says it "Supports down to 1.13 inclusive" and never mentions 1.21 (agent-read) — [fastnbt README](https://github.com/owengage/fastnbt/blob/master/README.md)
  - Hephaistos (Minestom's Anvil library) has been archived since 2024-09-20 (agent-read) — [Hephaistos](https://github.com/Minestom/Hephaistos)
  - Querz/NBT ships `net.querz.mca.{Chunk, MCAFile, MCAUtil}`; its 1.21 status is unconfirmed (agent-read) — [Querz/NBT](https://github.com/Querz/NBT)
  - Arnis (OSM → Java 1.17+) is reported to write via fastnbt (agent-read, medium confidence) — [louis-e/arnis](https://github.com/louis-e/arnis)
- Region format: 32×32 chunks per `.mca`, 4 KiB sectors, two 4 KiB header tables. Compression may be GZip, Zlib, none, LZ4 or custom (127) (agent-read) — [Region file format](https://minecraft.wiki/w/Region_file_format)

**(b) Partial bake (proto-chunks at an intermediate status)**
- 1.21.1 resumes saved proto-chunks. `ChunkGenerationTask.scheduleChunkInLayer` computes `flag = persistedStatus != null && status.isAfter(persistedStatus)`. It uses `GENERATION_PYRAMID` only for steps after the chunk's persisted status and `LOADING_PYRAMID` for the rest. `canLoadWithoutGeneration` also checks that neighbours meet the loading dependencies — [ChunkGenerationTask.java L81-103, L126-134](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/server/level/ChunkGenerationTask.java)
- `ChunkStep.apply` runs the task. If the chunk was before the target status, `completeChunkGeneration` then advances `ProtoChunk.setPersistedStatus(targetStatus)` — [ChunkStep.java L21-33](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/status/ChunkStep.java)
- The 1.21.1 statuses, in order, are `empty, structure_starts, structure_references, biomes, noise, surface, carvers, features, initialize_light, light, spawn, full`. `liquid_carvers` and `heightmaps` no longer exist; DFU remaps them as shown in (a) — [ChunkStatus.java L20-31](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/status/ChunkStatus.java)
- Fields read for a proto-chunk:
  - `sections` (`Y`, `block_states`, `biomes`, `BlockLight`, `SkyLight`)
  - `Status`, `Heightmaps` (optional, recomputed), `structures` (`starts` and `References`)
  - `PostProcessing`, `CarvingMasks` (keys are `GenerationStep.Carving` names), `below_zero_retrogen` (optional), `blending_data` (optional), `isLightOn`
  - `entities`, `block_entities`, and NeoForge attachments

  [ChunkSerializer.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/storage/ChunkSerializer.java)
- Structure starts are computed at `structure_starts`, before `biomes` and `noise`; the loading path never recomputes them (`loadStructureStarts` only notifies). A chunk baked at `noise` or later without a `structures` compound gets no structures at all — [ChunkPyramid.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/status/ChunkPyramid.java); [ChunkStatusTasks.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/status/ChunkStatusTasks.java)
- Neighbour requirements in `GENERATION_PYRAMID`:
  - `structure_references`, `biomes`: `STRUCTURE_STARTS` within 8 chunks
  - `noise`, `surface`: `STRUCTURE_STARTS` within 8 and `BIOMES` within 1
  - `carvers`: `STRUCTURE_STARTS` within 8
  - `features`: `STRUCTURE_STARTS` within 8 and `CARVERS` within 1
  - `light`: `INITIALIZE_LIGHT` within 1
  - `spawn`: `BIOMES` within 1

  At the edge of a baked area the game therefore generates un-baked neighbours with the dimension's own generator, up to `structure_starts` at radius 8 — [ChunkPyramid.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/status/ChunkPyramid.java)
- What each resumed status would do with baked data:
  - The surface step (`SurfaceSystem.buildSurface`) walks every column top to bottom. It calls the surface rule only where `blockstate == this.defaultBlock`, and leaves all other solid blocks alone. A `noise`-status bake therefore keeps custom rock blocks, while its `stone` positions are painted by the pack's rules — [SurfaceSystem.java L118-160](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/SurfaceSystem.java)
  - `NoiseBasedChunkGenerator.doFill` writes only non-air states (`if (blockstate != AIR && …) setBlockState`). A chunk baked at `biomes` or earlier would get vanilla noise terrain merged on top of the baked blocks — [NoiseBasedChunkGenerator.java L401-408](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/NoiseBasedChunkGenerator.java)
  - The carver step builds the dimension generator's own `NoiseChunk` and aquifer (`chunk.getOrCreateNoiseChunk(… createNoiseChunk …)`, `noisechunk.aquifer()`). Carved-cave water and lava levels come from the generator's router, not from the bake — [NoiseBasedChunkGenerator.java L288-314](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/NoiseBasedChunkGenerator.java)
- WorldPainter's 1.18+ writer is prior art for partial baking. `MC118AnvilChunk.setTerrainPopulated(false)` writes `status = STATUS_LIQUID_CARVERS` (`"liquid_carvers"`) and otherwise `"full"`. New chunks get DataVersion `DATA_VERSION_MC_1_18_0` (2860), and `lightOn` defaults to true — [MC118AnvilChunk.java L52, L240-250, L464-471](https://github.com/Captain-Chaos/WorldPainter/blob/master/WorldPainter/WPCore/src/main/java/org/pepsoft/minecraft/MC118AnvilChunk.java); [Constants.java L788, L1297-1298](https://github.com/Captain-Chaos/WorldPainter/blob/master/WorldPainter/WPCore/src/main/java/org/pepsoft/minecraft/Constants.java)
- WorldPainter's Populate wiki page says:
  - Populate works for "Minecraft 1.12 and older or Minecraft 1.18 and later".
  - Minecraft then adds ores, lakes and vegetation, and "for some versions of Minecraft, structures".
  - "it varies with the Minecraft version whether it will generate structures".
  - Combining it with WorldPainter's own resources doubles the resources.

  [WorldPainter wiki: Populate](https://www.worldpainter.net/trac/wiki/Populate)

**(c) Runtime custom `ChunkGenerator` + `BiomeSource`**
- A subclass must implement these abstract methods: `codec()`, `applyCarvers(...)`, `buildSurface(WorldGenRegion, StructureManager, RandomState, ChunkAccess)`, `spawnOriginalMobs`, `getGenDepth`, `fillFromNoise(Blender, RandomState, StructureManager, ChunkAccess)`, `getSeaLevel`, `getMinY`, `getBaseHeight(x, z, Heightmap.Types, LevelHeightAccessor, RandomState)`, `getBaseColumn(...)` and `addDebugScreenInfo`. The base class supplies `createStructures`, `createReferences`, `createBiomes`, `applyBiomeDecoration`, `createState` and `getSpawnHeight` — [ChunkGenerator.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/ChunkGenerator.java)
- `BiomeSource` has three abstract methods: `codec()`, `collectPossibleBiomes()` and `getNoiseBiome(int x, int y, int z, Climate.Sampler)`, where coordinates are in quarts — [BiomeSource.java L27-38, L165](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/biome/BiomeSource.java)
- For a generator that is not a `NoiseBasedChunkGenerator`, `ChunkMap` builds `RandomState.create(NoiseGeneratorSettings.dummy(), …)` — [ChunkMap.java L181-189](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/server/level/ChunkMap.java)
- Carvers are tied to the noise generator: the constructor is `CarvingContext(NoiseBasedChunkGenerator generator, RegistryAccess, LevelHeightAccessor, NoiseChunk, RandomState, SurfaceRules.RuleSource)` — [CarvingContext.java L24-32](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/carver/CarvingContext.java)
- TerraFirmaCraft (branch 1.21.x, commit 8aa7f96, 2026-09-27) shows the cost of this route:
  - The class is `public class TFCChunkGenerator extends ChunkGenerator implements ChunkGeneratorExtension`.
  - It keeps `private final NoiseBasedChunkGenerator stupidMojangChunkGenerator; // Mojang fix your god awful deprecated carver nonsense` purely to build a `CarvingContext`, passing `null` for the registry access and the NoiseChunk.
  - It runs its own `TFCAquifer` through a `FastConcurrentCache`.
  - It reuses vanilla `Beardifier.forStructuresInChunk(...)` for terrain adaptation around structures.
  - `getBaseColumn` returns `new NoiseColumn(0, new BlockState[0])`.

  [TFCChunkGenerator.java L109-147, L249-265, L418-437, L482-485](https://github.com/TerraFirmaCraft/TerraFirmaCraft/blob/8aa7f96cae0e7b10c63bf5b183f42b14f693c76f/src/main/java/net/dries007/tfc/world/TFCChunkGenerator.java)
- Structures query the generator's heights through `getFirstOccupiedHeight` and `getBaseColumn`. The callers are `Structure`, `JigsawPlacement`, `RuinedPortalStructure`, `NetherFossilStructure` and `PieceGeneratorSupplier` — [Structure.java L114, L149-152](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/structure/Structure.java)
- Tool compatibility suffers too:
  - World Preview's Y-intersection view shows white for TFC, which has a dummy `getBaseColumn`; a separate "World Preview TFC" fork exists to fix this (agent-read) — [World Preview TFC](https://modrinth.com/mod/world-preview-terrafirmacraft)
  - There is an open Distant Horizons × TFC NullPointerException issue (agent-read) — [TFC #2764](https://github.com/TerraFirmaCraft/TerraFirmaCraft/issues/2764)

**(d) Density-function injection (keep `NoiseBasedChunkGenerator` + multi-noise biome source)**
- `NoiseBasedChunkGenerator` is `public class` (not final). Its codec contains only `biome_source` and `settings`, the latter a `NoiseGeneratorSettings.CODEC` holder — [NoiseBasedChunkGenerator.java L50-57](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/NoiseBasedChunkGenerator.java)
- When the dimension's generator is `instanceof NoiseBasedChunkGenerator`, `ChunkMap` builds `RandomState.create(generator.generatorSettings().value(), registryaccess.lookupOrThrow(Registries.NOISE), seed)`. A subclass that overrides `generatorSettings()` can therefore supply a derived router — [ChunkMap.java L183-184](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/server/level/ChunkMap.java)
- Custom DF types register as `MapCodec<? extends DensityFunction>` in `Registries.DENSITY_FUNCTION_TYPE` (`"worldgen/density_function_type"`). Each type's `codec()` returns `KeyDispatchDataCodec<? extends DensityFunction>`. Named DF instances live in the datapack registry `Registries.DENSITY_FUNCTION` and are referenced via `DensityFunction.CODEC = RegistryFileCodec.create(...)` — [Registries.java L132](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/core/registries/Registries.java); [DensityFunction.java L12-32](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/DensityFunction.java)
- Custom DF types receive neither seed nor level. `RandomState`'s `NoiseWiringHelper` special-cases only `NoiseHolder` (seeds `NormalNoise`), `BlendedNoise` (`withNewRandom`) and `DensityFunctions.EndIslandDensityFunction` (`new EndIslandDensityFunction(levelSeed)`). Every other function only gets `mapAll(visitor)` — [RandomState.java L35-128](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/RandomState.java)
- ReTerraForged (1.21.1, NeoForge and Fabric; commit ba35ca7, 2026-07-27) is working prior art for (d):
  - The tree contains no custom `ChunkGenerator` or `BiomeSource` class.
  - It registers DF types `cell`, `clamp_to_nearest_unit`, `linear_spline` and `noise`.
  - `CellSampler(Supplier<WorldLookup> deferredLookup, Field field)` computes a `Cell` from its heightmap engine at `(blockX, blockZ)`.
  - `MixinRandomState` `@Redirect`s `NoiseRouter.mapAll` inside `RandomState.<init>` to capture `seed` and a `GeneratorContext` for its DFs.
  - `MixinNoiseChunk` attaches a per-chunk `Tile.Chunk` and a `CellSampler.Cache2d`.
  - It also mixes into `Climate.Sampler` and `SurfaceSystem`, and ships `compat/terrablender/TBNoiseRouterData`.

  [RTFDensityFunctions.java](https://github.com/Campione01/ReterraForged/blob/ba35ca764f4ace341c0692d80cfa12342503a94b/common/src/main/java/raccoonman/reterraforged/world/worldgen/densityfunction/RTFDensityFunctions.java); [CellSampler.java](https://github.com/Campione01/ReterraForged/blob/ba35ca764f4ace341c0692d80cfa12342503a94b/common/src/main/java/raccoonman/reterraforged/world/worldgen/densityfunction/CellSampler.java); [MixinRandomState.java](https://github.com/Campione01/ReterraForged/blob/ba35ca764f4ace341c0692d80cfa12342503a94b/common/src/main/java/raccoonman/reterraforged/mixin/MixinRandomState.java); [MixinNoiseChunk.java](https://github.com/Campione01/ReterraForged/blob/ba35ca764f4ace341c0692d80cfa12342503a94b/common/src/main/java/raccoonman/reterraforged/mixin/MixinNoiseChunk.java)
- Inside `NoiseChunk`, final density is wrapped as `cacheAllInCell(add(router.finalDensity(), BeardifierMarker.INSTANCE))`, so structure terrain adaptation is automatic. The aquifer is built from the router's `barrier`, `fluid_level_*`, `lava`, `erosion` and `depth` functions. Ore veins come from `vein_toggle`, `vein_ridged` and `vein_gap` when enabled — [NoiseChunk.java L141-158](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/NoiseChunk.java); [Aquifer.java L100-135](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/Aquifer.java)
- Biomes come from the router. `NoiseBasedChunkGenerator.createBiomes` fills them through `NoiseChunk.cachedClimateSampler(router, spawnTarget)`, which wraps temperature, vegetation, continents, erosion, depth and ridges in the chunk's caches. `MultiNoiseBiomeSource.getNoiseBiome(x,y,z,sampler)` then returns `parameters().findValue(sampler.sample(x,y,z))` — [NoiseChunk.java L162-172](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/NoiseChunk.java); [MultiNoiseBiomeSource.java L60-66](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/biome/MultiNoiseBiomeSource.java)
- C2ME's density-function compiler (1.21.1 branch) compiles known vanilla types (add/mul/min/max, clamp, constant, noises, shifts, splines, y_clamped_gradient, range_choice, weird_scaled_sampler, markers). Any other DensityFunction class goes through `default -> new DelegateNode(df)`, i.e. its own `compute`/`fillArray` is called. A sub-agent's claim that the compiler logs once per unknown type is wrong: the statistics line is commented out — [McToAst.java](https://github.com/RelativityMC/C2ME-fabric/blob/ver/1.21.1/c2me-opts-dfc/src/main/java/com/ishland/c2me/opts/dfc/common/ast/McToAst.java)
- Tectonic, a noise-settings-level terrain mod, documents "Increased Height": "Increases the max build and generation height to y640". It needs a separate "Terratonic" variant for Terralith compatibility (agent-read; the Terratonic mechanism is low confidence) — [Tectonic Config wiki](https://raw.githubusercontent.com/wiki/Apollounknowndev/tectonic/Config.md); [Terratonic](https://modrinth.com/datapack/terratonic)

### Inferences
- **(a) loses too much.** A full offline bake runs no `features`, so there is no modded biome decoration and no ore features added by NeoForge biome modifiers. It also gets no structures, no generation-time mob spawns, and no NeoForge first-generation signals (`newChunk=false`). It must also know every modded block and biome ID of the target pack offline. It only suits exporting a finished map. With a bounded world, "pregenerate everything with Chunky, then ship the world folder" gives the same result using the installed mods (see §4).
- **(b) works mechanically but is the wrong trade-off.**
  - At `noise` status, the game runs surface rules (on `stone` only), carvers, features, light and spawn.
  - At `surface`, it runs carvers onward.
  - At `carvers`, it runs only features, light and spawn. This is the most robust level and the one WorldPainter effectively produces: its `liquid_carvers` plus DataVersion 2860 is remapped to `carvers` by DFU, and its light is stripped by schema 3451.
  - Structures are lost unless the offline tool writes starts; it cannot compute modded structures.
  - Carver fluids follow the dimension generator's aquifer, not the simulation.
  - Chunks outside the baked area are generated by whatever generator the dimension has, which leaves a seam unless that generator is itself sim-aware — and at that point you have (c) or (d).
  - The cost is a multi-GB bake plus the game's own features and light work anyway (see §5).
- **(c) gives the most control but re-implements what (d) gets free.** Its advantage is block-level control of terrain and lithology in a single `fillFromNoise` pass. Its costs:
  - A dummy `NoiseBasedChunkGenerator` just to use carvers (TFC's hack).
  - Its own aquifer.
  - Correct `getBaseHeight`/`getBaseColumn`, or World Preview, structures and DH break (TFC's dummy `getBaseColumn`).
  - A `BiomeSource` that computes climate itself, because the `RandomState` sampler is a dummy.
  - It also bypasses the `MultiNoiseBiomeSource` sampler path that TerraBlender and Terralith build on. A custom source *could* call the public `MultiNoiseBiomeSource.getNoiseBiome(Climate.TargetPoint)` with sim-derived climate, but whether TerraBlender's region logic survives that path is for the biome researcher to confirm (see Gaps).
- **(d) is the only path that keeps the pack's surface rules, aquifers, carvers, noise caves, ore veins, beardifier, structure heights and multi-noise biome placement,** and it lets World Preview, DH (vanilla-chunk mode) and C2ME work unchanged: RTF is the live proof on 1.21.1. Its limitations:
  1. The noise stage can only emit the default block, air, fluid or vein ores, so lithology must be a separate pass.
  2. Custom DFs have no seed or level context. Bind them to world data through a lazy handle, as RTF does with `Supplier<WorldLookup>`, or through a `RandomState` mixin.
  3. The router must emit vanilla-range climate values, or biome mods place biomes oddly (this is Tectonic's Terralith problem).
  4. Replacing the router means router-editing terrain mods (Tectonic, Terralith's terrain density parts) will not apply. Their surface rules still can.
- **Recommended hybrid ("sim-driven router + strata pass + ore pass"):**
  1. **`GeologicChunkGenerator extends NoiseBasedChunkGenerator`.**
     - Codec: `{biome_source, settings: <holder, default minecraft:overworld>, sim: <sim id>, vertical: {...}}`.
     - In the constructor, derive a `NoiseGeneratorSettings` from the referenced one. Keep its `surface_rule` (so Terralith and the pack's rules survive), `aquifers_enabled`, `ore_veins_enabled`, `default_block` and `default_fluid`. Replace `noise_router` (and optionally `noise`, `sea_level`, `spawn_target`) with sim-backed functions.
     - Override `generatorSettings()` to return the derived holder, so `ChunkMap` builds the right `RandomState`.
     - Referencing the pack's settings by key, rather than copying them, keeps level.dat small and picks up pack updates. Copying freezes the world against pack changes; that is a product decision.
  2. **Custom DF types**, e.g.:
     - `geohist:raster` samples a named 2D layer bilinearly/bicubically; wrap it in `flat_cache` or `cache_2d`.
     - `geohist:surface_relative` returns `(H(x,z) − y)/k` for `depth`, `initial_density_without_jaggedness` and a base `final_density`.
     - `geohist:mask` and `geohist:inside_bounds` handle masking and bounds.
     - Keep the vanilla cave DFs by registry reference (`minecraft:overworld/caves/*`) inside `final_density`, so caves and aquifers stay vanilla-like.
     - Map sim climate and relief onto vanilla ranges: temperature and vegetation from sim climate; continents from land/sea and elevation; erosion from relief/erosion state; ridges from a valley/ridge index. That keeps biome mods' parameter points meaningful.
  3. **Strata pass.** Override `fillFromNoise(...)` to call `super.fillFromNoise(...).thenApply(this::applyStrata)`, or override `buildSurface` to call super and then the strata pass. For each column, evaluate the stratigraphic horizons (folds and faults deform them; intrusions are 3D bodies), then replace the remaining `default_block` runs directly in the section palettes.
     - Doing it after surface rules matters because rules only touch `default_block`.
     - A custom surface `RuleSource` appended to the pack's sequence is the data-driven alternative, since `MATERIAL_RULE` types are registrable (see §2). It evaluates per block rather than per column.
  4. **Ore pass.** Add one "deposits" feature at `UNDERGROUND_ORES` (via a biome modifier, or by overriding `applyBiomeDecoration`) that reads the sim's deposit list for the chunk. Suppressing or keeping other mods' ore features is for the ore researcher.
  5. **Bounds.** Outside the sim extent, the DFs return ocean (or a configured edge). Set the border and spawn once at creation (§4).
  6. **Optional DH LOD provider.** Register an `IDhApiWorldGenerator` that builds LODs straight from the rasters (§3).

### Gaps
- Whether TerraBlender's region and surface-rule injection applies to a `NoiseBasedChunkGenerator` subclass whose settings holder is a derived direct holder, rather than the `minecraft:overworld` key, was not checked. That belongs to the biome/TerraBlender researcher, but it gates (d)'s biome-mod promise and should be tested first. RTF's `TBNoiseRouterData` and TerraBlender `MixinClimateSampler`/`MixinNoiseChunk` variants suggest special handling is needed; I did not read their contents.
- I found no source on how RTF places river water above sea level. In vanilla (d), fluids come only from the aquifer (sea level, lava level and underground local levels), so sim rivers and lakes above sea level probably need a custom fluid pass. That is an inference, unverified.
- I found no per-chunk timing of `fillFromNoise` with custom DFs versus vanilla; the estimates in §5 are derived.

## 2. Verified NeoForge 21.1.249 / Minecraft 1.21.1 code facts that decide the design

### Takeaway
Every hook the hybrid needs exists in 1.21.1:
- A non-final noise generator
- Registrable density-function and surface-rule types
- Dimension heights up to 4064 blocks
- Presets with NeoForge-registered editors
- A cancellable NeoForge spawn-creation event

The constraints that bite are these. Custom DFs get no seed or level. Surface rules only rewrite `default_block`. Carvers need a `NoiseBasedChunkGenerator`. Chunk generation ignores the world border. Worldgen runs on background threads even without C2ME.

### Cited Findings
**Chunk pipeline (which step calls what)**
- The `GENERATION_PYRAMID` tasks are:

  | Step | Task |
  |---|---|
  | `structure_starts` | `generateStructureStarts` → `generator.createStructures(...)`, only if `worldGenOptions().generateStructures()` |
  | `structure_references` | `generator.createReferences` |
  | `biomes` | `generator.createBiomes(randomState, Blender, StructureManager, chunk)` |
  | `noise` | `generator.fillFromNoise(...)`, then `BelowZeroRetrogen` fix-ups |
  | `surface` | `generator.buildSurface(...)` |
  | `carvers` | `generator.applyCarvers(..., GenerationStep.Carving.AIR)` |
  | `features` | `Heightmap.primeHeightmaps(MOTION_BLOCKING, …)`, then `generator.applyBiomeDecoration(...)`, then `Blender.generateBorderTicks` |
  | `initialize_light`, `light` | light engine |
  | `spawn` | `generator.spawnOriginalMobs(...)` |
  | `full` | promote to `LevelChunk`, fire NeoForge `ChunkEvent.Load` |

  [ChunkStatusTasks.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/status/ChunkStatusTasks.java)
- `GenerationStep.Carving` still has `AIR("air")` and `LIQUID("liquid")`, but the status pipeline only invokes `AIR`. `GenerationStep.Decoration` order: `RAW_GENERATION, LAKES, LOCAL_MODIFICATIONS, UNDERGROUND_STRUCTURES, SURFACE_STRUCTURES, STRONGHOLDS, UNDERGROUND_ORES, UNDERGROUND_DECORATION, FLUID_SPRINGS, VEGETAL_DECORATION, TOP_LAYER_MODIFICATION` — [GenerationStep.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/GenerationStep.java)
- Worldgen runs off-thread even in vanilla:
  - `NoiseBasedChunkGenerator.fillFromNoise` runs `CompletableFuture.supplyAsync(Util.wrapThreadWithTaskName("wgen_fill_noise", …), Util.backgroundExecutor())`.
  - `createBiomes` does the same with `"init_biomes"`.

  [NoiseBasedChunkGenerator.java L77-81, L324-330](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/NoiseBasedChunkGenerator.java)
- Saving: `ChunkMap.save` skips proto-chunks whose full version already exists, and skips `EMPTY` chunks with no valid structure start — [ChunkMap.java L769-786](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/server/level/ChunkMap.java)

**Noise router, density functions, settings**
- `record NoiseRouter` has 15 fields: `barrierNoise, fluidLevelFloodednessNoise, fluidLevelSpreadNoise, lavaNoise, temperature, vegetation, continents, erosion, depth, ridges, initialDensityWithoutJaggedness, finalDensity, veinToggle, veinRidged, veinGap`. The JSON keys are `barrier, fluid_level_floodedness, fluid_level_spread, lava, temperature, vegetation, continents, erosion, depth, ridges, initial_density_without_jaggedness, final_density, vein_toggle, vein_ridged, vein_gap` — [NoiseRouter.java L8-24](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/NoiseRouter.java); [overworld.json](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-client-extra-aka-minecraft-resources.jar!/data/minecraft/worldgen/noise_settings/overworld.json)
- The `DensityFunction` interface:
  - `double compute(FunctionContext)`, `void fillArray(double[], ContextProvider)`, `DensityFunction mapAll(Visitor)`, `minValue()`, `maxValue()`, `KeyDispatchDataCodec<? extends DensityFunction> codec()`.
  - `SimpleFunction` supplies default `fillArray`/`mapAll`.
  - `Visitor.visitNoise(NoiseHolder)` exists for noise wiring.

  [DensityFunction.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/DensityFunction.java)
- Vanilla registers these DF types in `DensityFunctions.bootstrap` (`Registry.register(registry, name, codec.codec())`):
  - `blend_alpha, blend_offset, beardifier, old_blended_noise`
  - the markers `interpolated, flat_cache, cache_2d, cache_once, cache_all_in_cell`
  - `noise, end_islands, weird_scaled_sampler, shifted_noise, range_choice, shift_a, shift_b, shift, blend_density, clamp`
  - the mapped `abs, square, cube, half_negative, quarter_negative, squeeze`
  - `add, mul, min, max, spline, constant, y_clamped_gradient`

  [DensityFunctions.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/DensityFunctions.java)
- Registries for custom types: `DENSITY_FUNCTION_TYPE` (`worldgen/density_function_type`), `MATERIAL_CONDITION` and `MATERIAL_RULE` (`worldgen/material_rule`), `CHUNK_GENERATOR` (`worldgen/chunk_generator`), `BIOME_SOURCE` (`worldgen/biome_source`) and `WORLD_PRESET` (`worldgen/world_preset`). The first five are `BuiltInRegistries.registerSimple` registries of `MapCodec`s — [Registries.java L117-227](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/core/registries/Registries.java); [BuiltInRegistries.java L235-244](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/core/registries/BuiltInRegistries.java)
- `record NoiseGeneratorSettings(NoiseSettings noiseSettings, BlockState defaultBlock, BlockState defaultFluid, NoiseRouter noiseRouter, SurfaceRules.RuleSource surfaceRule, List<Climate.ParameterPoint> spawnTarget, int seaLevel, @Deprecated boolean disableMobGeneration, boolean aquifersEnabled, boolean oreVeinsEnabled, boolean useLegacyRandomSource)`. Its JSON keys are `noise, default_block, default_fluid, noise_router, surface_rule, spawn_target, sea_level, disable_mob_generation, aquifers_enabled, ore_veins_enabled, legacy_random_source` — [NoiseGeneratorSettings.java L19-44](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/NoiseGeneratorSettings.java)
- Vanilla overworld values: `noise {min_y -64, height 384, size_horizontal 1, size_vertical 2}`, `default_block stone`, `default_fluid water`, `sea_level 63`, `aquifers_enabled true`, `ore_veins_enabled true`. The surface rule is ~30 KB of JSON. `floating_islands` uses `min_y 0, height 256, size_horizontal 2, size_vertical 1, sea_level -64`, with aquifers and veins off — [overworld.json](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-client-extra-aka-minecraft-resources.jar!/data/minecraft/worldgen/noise_settings/overworld.json); [floating_islands.json](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-client-extra-aka-minecraft-resources.jar!/data/minecraft/worldgen/noise_settings/floating_islands.json)
- Vanilla climate inputs:
  - `overworld/continents`, `erosion` and `ridges` are each `flat_cache(shifted_noise(xz_scale 0.25, y_scale 0))`.
  - `overworld/depth = y_clamped_gradient(1.5 at y=-64 → -1.5 at y=320) + overworld/offset`.
  - `final_density = min(squeeze(0.64 × interpolated(blend_density(… sloped_cheese / caves …))), …)`.

  [density_function/overworld/*.json](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-client-extra-aka-minecraft-resources.jar!/data/minecraft/worldgen/density_function/overworld/depth.json)
- `NoiseSettings(minY, height, noiseSizeHorizontal, noiseSizeVertical)` has codec ranges `min_y ∈ [-2032, 2031]`, `height ∈ [0, 4064]` and sizes 1–4. `guardY` enforces "min_y + height cannot be higher than: 2032" and multiples of 16. `clampToHeightAccessor` intersects the noise range with the dimension's build range — [NoiseSettings.java L13-60](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/NoiseSettings.java)

**Caching and sampling resolution**
- `NoiseChunk.wrapNew` maps each marker to its implementation: `Interpolated → NoiseInterpolator`, `FlatCache → FlatCache(…, true)`, `Cache2D`, `CacheOnce` and `CacheAllInCell`. It also substitutes `BeardifierMarker` with the chunk's beardifier and unwraps `HolderHolder` — [NoiseChunk.java L354-384](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/NoiseChunk.java)
- `FlatCache` precomputes `values[noiseSizeXZ+1][noiseSizeXZ+1]` at quart (4-block) positions with `SinglePointContext(x, 0, z)`, i.e. at Y=0. It returns the stored quart value, with no interpolation, for positions inside the chunk, and otherwise falls through to `compute` — [NoiseChunk.java L601-632](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/NoiseChunk.java)
- `NoiseChunk` takes `cellWidth = noiseSettings.getCellWidth()` and `cellHeight = getCellHeight()`, with `cellCountY = height / cellHeight` — [NoiseChunk.java L96-120](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/NoiseChunk.java). Per the wiki (agent-read), cell size is `size_horizontal×4` by `size_vertical×4`, giving 4×8 blocks for the overworld. The wiki's marker semantics:
  - `flat_cache`: "value per 4×4 column… calculated only once per column, at Y=0".
  - `cache_2d`: "once per horizontal position".
  - `cache_once`: "once per block position".
  - `cache_all_in_cell`: engine-only.

  The same page says 26.3-snap10 removed `flat_cache`, `cache_2d`, `cache_all_in_cell` and `shifted_noise`, and renamed `cache_once` to `cache` — [Density function](https://minecraft.wiki/w/Density_function)
- `NoiseChunk.preliminarySurfaceLevel(x,z)` scans down from the top in `cellHeight` steps for `initialDensityNoJaggedness > 0.390625`. The aquifer samples it at 13 chunk offsets, and marks `OverworldBiomeBuilder.isDeepDarkRegion(erosion, depth)` regions as fluid-free — [NoiseChunk.java L194-212](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/NoiseChunk.java); [Aquifer.java L385-430](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/Aquifer.java)
- Biome sampling is at quart resolution. `ChunkAccess.fillBiomesFromNoise` iterates sections and calls `section.fillBiomesFromNoise(resolver, sampler, quartX, quartY, quartZ)`. `Climate.Sampler.sample(x,y,z)` converts quarts to blocks and evaluates the six functions at `SinglePointContext(blockX, blockY, blockZ)` — [ChunkAccess.java L452-463](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/ChunkAccess.java); [Climate.java L479-491](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/biome/Climate.java)
- `RandomState`'s own `sampler()`, used outside chunk generation for spawn search, `/locate` and structure biome checks, is built with a visitor that strips `Marker`s and unwraps holders, so there is no caching. It carries `settings.spawnTarget()`, and `findSpawnPosition()` returns `BlockPos.ZERO` when `spawnTarget` is empty — [RandomState.java L102-128](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/RandomState.java); [Climate.java L494-495](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/biome/Climate.java)
- `RandomState.create(NoiseGeneratorSettings, HolderGetter<NoiseParameters>, long seed)` builds:
  - `random = settings.getRandomSource().newInstance(seed).forkPositional()`
  - `aquiferRandom` and `oreRandom`, from hashes of "aquifer" and "ore"
  - `SurfaceSystem(this, defaultBlock, seaLevel, random)`
  - `router = settings.noiseRouter().mapAll(NoiseWiringHelper)`

  [RandomState.java L27-101](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/RandomState.java)

**Surface rules, ore veins, carvers, structures**
- Surface-rule conditions: `biome, noise_threshold, vertical_gradient, y_above, water, temperature, steep, not, hole, above_preliminary_surface, stone_depth`. Rules: `bandlands, block, sequence, condition`. Both registries accept mod types — [SurfaceRules.java L243-254, L563-567](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/SurfaceRules.java)
- `SurfaceSystem.buildSurface` updates the context per block and tries the rule only when `blockstate == this.defaultBlock`. This covers every depth, which is how vanilla's deepslate gradient is applied — [SurfaceSystem.java L118-160](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/SurfaceSystem.java)
- `OreVeinifier`: `VEININESS_THRESHOLD = 0.4F` and `CHANCE_OF_RAW_ORE_BLOCK = 0.02F`. `vein_toggle > 0` gives `COPPER` (copper ore, raw copper block, granite filler, Y 0..50); otherwise `IRON` (deepslate iron ore, raw iron block, tuff filler, Y -60..-8). The vein types are hard-coded — [OreVeinifier.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/OreVeinifier.java)
- `Beardifier.forStructuresInChunk(structureManager, chunkPos)` gathers structures whose `terrainAdaptation() != NONE` — [Beardifier.java L38-51](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/Beardifier.java)

**Presets, create-world UI, dimension height, border, spawn, server**
- `WorldPreset` is a `dimensions` map, and presets must include an overworld ("Missing overworld dimension"). The vanilla `normal` preset is a `minecraft:noise` generator with a `multi_noise` source (`preset: minecraft:overworld`) and `settings: minecraft:overworld`. The UI lists presets from the tag `#minecraft:normal` (normal, flat, large_biomes, amplified, single_biome_surface); `#minecraft:extended` adds debug. `WorldCreationUiState` falls back to all presets if the tag is empty — [WorldPreset.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/presets/WorldPreset.java); [normal.json](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-client-extra-aka-minecraft-resources.jar!/data/minecraft/worldgen/world_preset/normal.json); [tags/worldgen/world_preset/normal.json](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-client-extra-aka-minecraft-resources.jar!/data/minecraft/tags/worldgen/world_preset/normal.json); [WorldCreationUiState.java L231-247](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/client/gui/screens/worldselection/WorldCreationUiState.java)
- Preset editors:
  - `PresetEditor` is `Screen createEditScreen(CreateWorldScreen, WorldCreationContext)`. Vanilla's `EDITORS` map (flat, single-biome) is `@Deprecated` in favour of `net.neoforged.neoforge.client.PresetEditorManager#get`.
  - The NeoForge mod-bus event is `RegisterPresetEditorsEvent.register(ResourceKey<WorldPreset>, PresetEditor)`.
  - `WorldCreationUiState.getPresetEditor()` resolves through `PresetEditorManager::get`.
  - Editors apply changes with `createWorldScreen.getUiState().updateDimensions(DimensionsUpdater)`, where `DimensionsUpdater` is a `BiFunction<RegistryAccess.Frozen, WorldDimensions, WorldDimensions>`; e.g. `replaceOverworldGenerator(registryAccess, generator)`.

  [PresetEditor.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/client/gui/screens/worldselection/PresetEditor.java); [RegisterPresetEditorsEvent.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/neoforged/neoforge/client/event/RegisterPresetEditorsEvent.java); [WorldCreationUiState.java L186, L226-228](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/client/gui/screens/worldselection/WorldCreationUiState.java); [WorldDimensions.java L64](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/WorldDimensions.java)
- `WorldCreationContext(WorldOptions options, Registry<LevelStem> datapackDimensions, WorldDimensions selectedDimensions, LayeredRegistryAccess worldgenRegistries, ReloadableServerResources, WorldDataConfiguration)`. `worldgenLoadContext()` returns the frozen registries (biomes, noise settings, DFs, presets) the editor can use for previews — [WorldCreationContext.java L20-98](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/client/gui/screens/worldselection/WorldCreationContext.java)
- `DimensionType` constants and checks:
  - `BITS_FOR_Y = 12`, `Y_SIZE = (1<<12) - 32 = 4064`, `MAX_Y = 2031`, `MIN_Y = -2032`.
  - Codec: `min_y ∈ [MIN_Y, MAX_Y]`, `height ∈ [16, 4064]`, `logical_height ∈ [0, 4064]`.
  - Runtime checks: "min_y + height cannot be higher than: 2032", "logical_height cannot be higher than height", "height has to be multiple of 16", "min_y has to be a multiple of 16".
  - The vanilla overworld is `min_y -64, height 384, logical_height 384`.

  [DimensionType.java L45-105](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/dimension/DimensionType.java); [dimension_type/overworld.json](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-client-extra-aka-minecraft-resources.jar!/data/minecraft/dimension_type/overworld.json)
- `WorldBorder` API: `setCenter(x,z)`, `setSize(d)`, `lerpSizeBetween(...)`, `setAbsoluteMaxSize(int)`, `isWithinBounds(ChunkPos)`, `MAX_SIZE = 5.999997E7`. The dedicated-server property `max-world-size` clamps to 1..29999984 — [WorldBorder.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/border/WorldBorder.java); [DedicatedServerProperties.java L89](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/server/dedicated/DedicatedServerProperties.java)
- The chunk system does not consult the border. `ChunkMap`, `DistanceManager` and `ChunkGenerationTask` contain no border references. In `server/level` the border only gates natural spawning (`ServerChunkCache` L369) and interaction (`ServerLevel` L807). `LevelChunk.isTicking(pos)` returns false outside it — [ServerChunkCache.java L369](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/server/level/ServerChunkCache.java); [LevelChunk.java L373-376](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/LevelChunk.java)
- Spawn at creation: `MinecraftServer.setInitialSpawn` first calls `EventHooks.onCreateWorldSpawn(level, levelData)`, which is the cancellable NeoForge `LevelEvent.CreateSpawnPosition` carrying `ServerLevelData`. Otherwise it uses `randomState().sampler().findSpawnPosition()` plus `generator.getSpawnHeight`. It runs once, inside `createLevels` when `!serverleveldata.isInitialized()` — [MinecraftServer.java L367-378, L432-444](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/server/MinecraftServer.java); [LevelEvent.java L101-116](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/neoforged/neoforge/event/level/LevelEvent.java)
- Server startup order:
  - Both `IntegratedServer.initServer` and `DedicatedServer.initServer` call `ServerLifecycleHooks.handleServerAboutToStart(this)` immediately before `loadLevel()`.
  - On the dedicated server, "Done (…)!" is logged after `loadLevel()`, and `RconThread.create` runs after that.
  - `prepareLevels` loads spawn chunks per the `spawnChunkRadius` game rule.

  [IntegratedServer.java L70-77](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/client/server/IntegratedServer.java); [DedicatedServer.java L191-209](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/server/dedicated/DedicatedServer.java); [MinecraftServer.java L493-504](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/server/MinecraftServer.java)
- `server.properties` `level-type` goes through `ResourceLocation.tryParse(levelType)` into a `WORLD_PRESET` key. Any registered preset such as `geohist:geologic` works, with legacy aliases `default` and `largebiomes`. `generator-settings` is parsed only when the preset is `minecraft:flat` — [DedicatedServerProperties.java L109-115, L207-229](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/server/dedicated/DedicatedServerProperties.java)
- Per-world files: `MinecraftServer.getWorldPath(LevelResource)`, with constants such as `LevelResource.ROOT`, `DATAPACK_DIR` and `GENERATED_DIR`. NeoForge copies chunk attachments from `ProtoChunk` to `LevelChunk` on promotion (`AttachmentInternals.copyChunkAttachmentsOnPromotion`) — [MinecraftServer.java L1954](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/server/MinecraftServer.java); [LevelResource.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/storage/LevelResource.java); [LevelChunk.java L142](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/LevelChunk.java)
- Client drawing APIs (1.21.1):
  - `NativeImage(int w, int h, boolean useCalloc)`, `setPixelRGBA(int x, int y, int abgrColor)` (ABGR despite the name), `fillRect`, `upload(...)`
  - `DynamicTexture(NativeImage)`, `DynamicTexture.upload()`, `getPixels()`
  - `TextureManager.register(ResourceLocation, AbstractTexture)` and `register(String, DynamicTexture) → ResourceLocation`
  - `GuiGraphics.blit(ResourceLocation, x, y, u, v, w, h)`

  [NativeImage.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/com/mojang/blaze3d/platform/NativeImage.java); [DynamicTexture.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/client/renderer/texture/DynamicTexture.java); [GuiGraphics.java L789](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/client/gui/GuiGraphics.java)

### Inferences
- **Vertical scale.** The overworld can be made up to 4064 blocks tall, e.g. `min_y -1024, height 2048` or the maximum `-2032..2031`. Router `depth` and `initial_density` must be defined against the chosen range. Since `NoiseSettings` is clamped to the dimension, the noise range can cover only the geologically interesting band. Each extra 16 blocks adds a section per chunk (memory, disk, lighting, DH columns), so a modest height like Tectonic's y640 top is likely the practical sweet spot. The simulation's vertical mapping will then need exaggeration or compression: kilometres of relief over a few hundred blocks.
- **Binding DFs to world data.** The generator and its DFs are decoded from level.dat before `ServerAboutToStartEvent`; that ordering was not traced for existing worlds and is flagged in Gaps. They must therefore hold a *lazy* handle keyed by sim id, resolved on first `compute` to `<world>/geohist/` on the server, or to the staging cache in the create-world screen. The alternative is RTF's approach: a `RandomState` mixin that injects context during `mapAll`. Either way, DFs must be pure, thread-safe functions: vanilla already calls them from `Util.backgroundExecutor()`, C2ME from more threads, and the standalone `Climate.Sampler` (spawn search, `/locate`) with no caching.
- **Resolution is not a real constraint.** A heightfield density `(H−y)/k` is linear in y, so vertical interpolation over 8-block cells reproduces it exactly. Horizontal detail is bilinear at 4-block cells. If sub-4-block terrain detail matters, a raster DF can be left un-`interpolated` (evaluated per block inside `cache_all_in_cell`), because raster lookups are cheap (§5).
- **Border handling.** Because chunk loading ignores the border, the generator must produce sensible terrain beyond the sim extent: ocean, void or an ice wall. It should set the border and spawn itself, in `LevelEvent.CreateSpawnPosition` or at first start. Chunky's border-driven pregen then covers exactly the map (§3/§4).

### Gaps
- `NoiseSettings.getCellWidth()`/`getCellHeight()` were not opened. The 4×8 cell size rests on the wiki (agent-read) plus the overworld's `size_horizontal 1 / size_vertical 2`.
- When `WorldGenSettings` / `LevelStem` generators are decoded for *existing* worlds relative to `ServerAboutToStartEvent` was not traced; the lazy-binding advice covers both orders.
- Whether a border set in `CreateSpawnPosition` is persisted to level.dat (it should be, as `/worldborder` edits the same object) was not traced to the save code.

## 3. Prior art: what to copy and what to avoid

### Takeaway
Two families exist:
- **Full custom generators:** TFC, original TerraForged, Terra.
- **Router/DF injection:** ReTerraForged, Tectonic/Lithostitched.

Only the second keeps biome mods and ecosystem tools working, and RTF proves a heightmap-engine-backed DF plus a live preset preview works on 1.21.1. Copy two things from TFC: its tectonic-region pipeline and its rock/vein coupling. Avoid its custom-generator plumbing. Offline tools (WorldPainter, Amulet, Arnis, libraries) prove region writing and the populate trick, but they suffer format churn. DH's world-generator API is the natural fast-LOD path for a bounded world, and Chunky's border mode pairs with it for pregeneration.

### Cited Findings
**TerraFirmaCraft (1.21.x)**
- TFC uses its own generator and biome source: `TFCChunkGenerator extends ChunkGenerator` with `RegionBiomeSource`. It carries a dummy `NoiseBasedChunkGenerator` for carvers, its own `TFCAquifer`, vanilla `Beardifier`, and an empty `getBaseColumn` — [TFCChunkGenerator.java](https://github.com/TerraFirmaCraft/TerraFirmaCraft/blob/8aa7f96cae0e7b10c63bf5b183f42b14f693c76f/src/main/java/net/dries007/tfc/world/TFCChunkGenerator.java)
- Its world `Settings` record has `flat_bedrock, spawn_distance, spawn_center_x, spawn_center_z, temperature_scale, rainfall_scale, rock_layer_settings, continentalness, grass_density, finite_continents`, plus optional `temperature_constant` and `rainfall_constant` (default 0). `RegionGenerator.continentFactor` uses `finiteContinents` with `rainfallScale`/`temperatureScale` as the world extent, giving a bounded, fades-to-ocean mode — [Settings.java L21-48](https://github.com/TerraFirmaCraft/TerraFirmaCraft/blob/8aa7f96cae0e7b10c63bf5b183f42b14f693c76f/src/main/java/net/dries007/tfc/world/settings/Settings.java); [RegionGenerator.java L198-204](https://github.com/TerraFirmaCraft/TerraFirmaCraft/blob/8aa7f96cae0e7b10c63bf5b183f42b14f693c76f/src/main/java/net/dries007/tfc/world/region/RegionGenerator.java)
- The region pipeline (agent-read) is a set of `RegionTask`s over Voronoi cells:
  - `AnnotateBoundaryTypes` / `RegionGenerator.getDivergence` compute signed plate divergence from per-cell pseudo-velocities.
  - `AddMountainsAndBarrierIslands` puts mountains where divergence < 0.
  - `AddHotspots` makes aging hotspot chains.
  - `ChooseRocks` picks one of four rock supertypes, OCEAN/VOLCANIC/LAND/UPLIFT, which select the rock-layer tables.
  - The class javadoc calls it a "single-instance, threadsafe generator".

  [TFC world/region tree](https://github.com/TerraFirmaCraft/TerraFirmaCraft/tree/1.21.x/src/main/java/net/dries007/tfc/world/region)
- TFC's ore veins are keyed to host rock (agent-read): `VeinConfig.states: Map<Block, IWeighted<BlockState>>`, plus indicator, rarity, density and Y range — [VeinConfig.java](https://github.com/TerraFirmaCraft/TerraFirmaCraft/blob/1.21.x/src/main/java/net/dries007/tfc/world/feature/vein/VeinConfig.java)
- Its create-world settings screen is `net.dries007.tfc.client.screen.CreateTFCWorldScreen`; I confirmed the file exists. Per the agent it uses `OptionInstance` sliders, including km-formatted world size, and is wired via `CreateWorldScreenMixin`/`WorldPresetsMixin` — [CreateTFCWorldScreen.java](https://github.com/TerraFirmaCraft/TerraFirmaCraft/blob/1.21.x/src/main/java/net/dries007/tfc/client/screen/CreateTFCWorldScreen.java)
- TFC on NeoForge 1.21.1 (agent-read via the Modrinth API): the first build was 4.0.0-beta on 2025-09-13, and the latest 4.2.10 on 2026-09-07 — [Modrinth TFC versions](https://modrinth.com/mod/terrafirmacraft/versions)

**ReTerraForged / TerraForged**
- RTF targets 1.21.1 on NeoForge and Fabric via vanilla `NoiseBasedChunkGenerator` plus custom DFs (details in §1). Its preset config screen has pages: World, Climate, Terrain, River, Surface, Cave, Structure, Island, Filter, Miscellaneous. `PresetEditorPage` embeds a `Preview extends AbstractWidget` backed by `new DynamicTexture(new NativeImage(SIZE, SIZE, false))`, with a `RenderMode` cycle defaulting to `BIOME_TYPE` and `regenerate()` on edits — [PresetEditorPage.java](https://github.com/Campione01/ReterraForged/blob/ba35ca764f4ace341c0692d80cfa12342503a94b/common/src/main/java/raccoonman/reterraforged/client/gui/screen/presetconfig/PresetEditorPage.java)
- The original TerraForged repository is archived; its last push was 2023-05-11 and its README describes an unfinished 1.18 port (agent-read) — [TerraForged/TerraForged](https://github.com/TerraForged/TerraForged)
- The fork ecosystem is fragmented (FreeTerraForged and others). FreeTerraForged issue #214 reads "C2ME DFC compiler guard isn't working on current releases" (agent-read) — [FreeTerraForged #214](https://github.com/ETcodehome/FreeTerraForged/issues/214)

**Terra (PolyhedralDev)**
- Terra runs its own engine: `com.dfsek.terra.api.world.chunk.generation.ChunkGenerator`, wrapped per platform (`BukkitChunkGeneratorWrapper`, `MinestomChunkGeneratorWrapper`, `mod.generation.MinecraftChunkGeneratorWrapper`), with its own `NoiseChunkGenerator3D` (agent-read, via the repo tree) — [PolyhedralDev/Terra](https://github.com/PolyhedralDev/Terra)
- Platform support per its versions page (agent-read): Paper/Folia/Purpur/Fabric/Quilt through 1.21.8, Bukkit/Spigot to 1.20.1, Forge at 1.19.2. NeoForge is not listed — [Terra versions](https://terra.polydev.org/install/versions.html)

**Tectonic / Lithostitched**
- Tectonic's config includes Increased Height (to y640), Vertical Scale, Continents/Erosion/Ridge Scale and Underground Rivers, and a NeoForge config screen exists (agent-read) — [Tectonic Config](https://raw.githubusercontent.com/wiki/Apollounknowndev/tectonic/Config.md)
- Lithostitched adds DF types `fast_noise, axis, ceil, floor, sin, cos, sqrt, mix, shift, select` and the `wrapped_marker`/`original_marker` markers for its router-wrapping modifiers. A "Neoforge 21.1" file exists (agent-read) — [Lithostitched Technical Changelog](https://github.com/Apollounknowndev/lithostitched/wiki/Technical-Changelog); [CurseForge file](https://www.curseforge.com/minecraft/mc-mods/lithostitched/files/8944659)
- Tectonic issue #429 reports that Java-side DFs cause trouble with C2ME's experimental OpenCL path (agent-read) — [Tectonic #429](https://github.com/Apollounknowndev/tectonic/issues/429)

**Offline and Earth tools**
- WorldPainter: see the populate status mechanism in §1(b). Its scripting API is JavaScript through a `wp` object (`wpscript` CLI; heightmap import including 16-bit; layers; biomes by ID). Modded blocks are specified by `modid:block` (agent-read). The changelog mentions support for "Minecraft 26.1+" maps and a non-standard build-height fix for 1.21.11+ via a generated datapack; dates are anomalous, so this is low confidence — [Scripting API](https://www.worldpainter.net/trac/wiki/Scripting/API); [FAQ](https://www.worldpainter.net/trac/wiki/FAQ); [CHANGELOG](https://www.pepsoft.org/worldpainter/CHANGELOG)
- terraplusplus builds on CubicChunks and CubicWorldGen for 1.12.2 and generates on demand (`/tpll`) from elevation, OSM and tree-cover datasets (agent-read). The Fabric port BTE-Germany/TerraPlusPlus-Fabric exists; its version support was not confirmed — [terraplusplus wiki](https://github.com/BuildTheEarth/terraplusplus/wiki); [TerraPlusPlus-Fabric](https://github.com/BTE-Germany/TerraPlusPlus-Fabric)
- Terrarium (1.12.2, agent-read): its customization GUI shows properties on the left and a live preview on the right. Data tiles download on demand and are cached locally. Without Cubic Chunks, terrain is clipped to Y 0–256 — [Terrarium README](https://github.com/Gegy/Terrarium/blob/1.12-dev/README.md)

**Preview, LOD and pregeneration ecosystem**
- World Preview (caeruleusDraconis) covers Fabric/Forge/Quilt 1.20–1.21 and shows biomes, a heightmap, structures and a Y-intersection view (agent-read). The unofficial NeoForge 1.21.1 port is by Alwis2000; I confirmed its page: client-side, same features, "TFC does not have full compatibility" — [World Preview](https://modrinth.com/mod/world-preview); [World Preview NeoForged](https://modrinth.com/mod/world-preview-neoforged-(unofficial))
- "Genesis – World Previewer & Pregenerator" (1.21.1 Fabric/NeoForge, agent-read) adds a create-world biome map "sampled from the actual world generator". It samples with up to 16 background threads in a three-pass spiral, fixes the sample height at Y=320 to keep cave biomes off the surface map, and bundles a pregenerator HUD — [Genesis](https://modrinth.com/project/xXuFelru)
- Distant Horizons API (I read the javadoc directly):
  - `IDhApiWorldGenerator` has `generateChunks(chunkPosMinX, chunkPosMinZ, widthCount, targetDataDetail, EDhApiDistantGeneratorMode, ExecutorService, Consumer<Object[]>)` for `VANILLA_CHUNKS`.
  - `generateApiChunks(…, Consumer<DhApiChunk>)` serves `API_CHUNKS` (API 2.0.0).
  - `generateLod(chunkPosMinX, chunkPosMinZ, lodPosX, lodPosZ, detailLevel, IDhApiFullDataSource pooled, mode, ExecutorService, Consumer<IDhApiFullDataSource>)` serves `API_DATA_SOURCES` (API 4.0.0).
  - Also: `getReturnType()` (default `VANILLA_CHUNKS`), `get{Smallest,Largest}DataDetailLevel()`, `runApiValidation()` (API 4.0.0) and `preGeneratorTaskStart()`.

  [IDhApiWorldGenerator javadoc](https://distant-horizons-team.gitlab.io/distant-horizons/com/seibel/distanthorizons/api/interfaces/override/worldGenerator/IDhApiWorldGenerator.html)
- More DH facts (agent-read):
  - Registration is through `DhApiWorldGeneratorOverrideRegister`, and generation modes are `EDhApiDistantGeneratorMode {PRE_EXISTING_ONLY, SURFACE, FEATURES, FULL}`.
  - The DH version for 1.21.1 is 3.3.1 (2026-09-18).
  - API 4.0.0 renamed `runApiChunkValidation` to `runApiValidation`, so the API still churns.

  [DH javadoc index](https://distant-horizons-team.gitlab.io/distant-horizons/allclasses-index.html); [DH 3.3.1-1.21.1](https://modrinth.com/mod/distanthorizons/version/3.3.1-1.21.1); [DH API 4.0.0](https://modrinth.com/mod/distanthorizonsapi/version/4.0.0)
- DH × C2ME friction is open on NeoForge (C2ME-neoforge #17 "Reduce CPU usage when Distant Horizons is present"; C2ME-fabric #258), and community reports conflict (agent-read). One community answer says `/dh pregen` can pregenerate to the world border (agent-read, low confidence) — [C2ME-neoforge #17](https://github.com/RelativityMC/C2ME-neoforge/issues/17); [answeroverflow](https://www.answeroverflow.com/m/1363222992821289130)
- Chunky (agent-read):
  - `chunky worldborder` uses the world's existing border as the pregen area.
  - Shapes: square, circle, diamond, rectangle, ellipse, pentagon, triangle, star.
  - The ChunkyBorder addon enforces the border.
  - The FAQ warns that pregenerating beyond the border or `max-world-size` can cause "no mobs" bugs.

  [Chunky World borders](https://github.com/pop4959/Chunky/wiki/World-borders); [Chunky Shapes](https://github.com/pop4959/Chunky/wiki/Shapes)
- C2ME has an official NeoForge port. The C2ME-neoforge repo has `ver/1.21.1` and `backports/1.21.1` branches (I saw them via the GitHub API). Modrinth lists 1.21.1 builds up to `0.3.0+alpha.0.89+1.21.1` (2026-01-11) (agent-read) — [C2ME-neoforge](https://github.com/RelativityMC/C2ME-neoforge); [Modrinth c2me-neoforge](https://modrinth.com/mod/c2me-neoforge)
- A DynamicTrees issue records a real C2ME thread-safety worldgen crash (agent-read) — [DynamicTrees #1127](https://github.com/DynamicTreesTeam/DynamicTrees/issues/1127)

### Inferences
- **Copy from RTF:**
  - The architecture: a vanilla generator, an engine-backed "cell/raster" DF with lazy world lookup, a per-chunk tile cache (RTF hooks `NoiseChunk`; a `cache_2d`/`flat_cache` wrapper around a tiled raster may suffice without a mixin), and a TerraBlender router variant.
  - The preset config screen with a live `DynamicTexture` preview and render modes. Extend it to a *history* player (§4).
- **Copy from TFC:** the region-task pipeline idea (plate-boundary divergence → orogeny, hotspot age chains, rock supertypes → strata tables), host-rock-keyed vein configs, `finite_continents`, and km-denominated create-world sliders.
- **Avoid from TFC:** a custom `ChunkGenerator` with a dummy carver generator, a private aquifer, and a dummy `getBaseColumn`. The last one demonstrably breaks World Preview, and the DH NPE shows the class of risk.
- **Avoid a Terra-style standalone engine.** Its per-platform wrappers leave loaders behind: Forge is stuck at 1.19.2 and NeoForge is absent. Keep the sim core Minecraft-independent, but integrate natively through the router.
- **Offline writers carry format churn.** Hephaistos is archived, fastanvil stops at 1.13, and Amulet has a proto-chunk bug. They are export tools, not the primary bridge.
- **For DH on a bounded world:** implement `IDhApiWorldGenerator` with `API_DATA_SOURCES` or `API_CHUNKS` fed from the rasters (surface height, surface block from lithology/biome, water). The whole 16k map then gets LODs without generating chunks. Real chunks overwrite LODs as they load; I infer DH updates LODs from loaded chunks, but did not verify it. Otherwise keep `VANILLA_CHUNKS`, which works because the generator is a `NoiseBasedChunkGenerator`.
- **C2ME:** custom DFs run under the DFC as delegates, so they cost interpretation overhead but break nothing. They must be thread-safe and allocation-light.

### Gaps
- I found no maintained 1.21.1 mod that samples an external image or raster as a density function. The agent found only an unverified lead, "More Density Functions" by Klinbee.
- ReTerraForged's river and erosion algorithm, and how it places above-sea-level water, were not read.
- Distant Horizons LOD-generation throughput and LOD-database size are unknown. Whether DH distant generation respects the world border is contradictory in community sources.
- I did not confirm the Terrarium preview is 3D. I found no mod that renders a 3D globe or heightfield mesh in a create-world screen.

## 4. World-creation and history-visualization workflow (singleplayer and dedicated server)

### Takeaway
In singleplayer, use a registered world preset whose NeoForge `PresetEditor` opens a "Geologic history" screen. There the sim runs on a background thread and streams frames into a `DynamicTexture` map with a time scrubber, like DF worldgen or RTF's live preview. On accept, the screen swaps the overworld generator via `updateDimensions`, then world creation proceeds normally. A dedicated server selects the preset with `level-type`, and gets its sim either from a standalone CLI run beforehand or from a blocking run in `ServerAboutToStartEvent`. That second option reports progress on the console only: RCON and clients come up after world load.

### Cited Findings
- The preset editor hook is NeoForge's `RegisterPresetEditorsEvent.register(key, editor)`, and `createEditScreen(CreateWorldScreen, WorldCreationContext)` returns any `Screen`. It applies results with `getUiState().updateDimensions(...)`, the same way vanilla's flat and buffet editors do — [RegisterPresetEditorsEvent.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/neoforged/neoforge/client/event/RegisterPresetEditorsEvent.java); [PresetEditor.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/client/gui/screens/worldselection/PresetEditor.java)
- The editor gets `WorldCreationContext.worldgenLoadContext()`, the frozen worldgen registries, so it can build the pack's biome source and router for previews — [WorldCreationContext.java L81-83](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/client/gui/screens/worldselection/WorldCreationContext.java)
- Vanilla's create flow runs `CreateWorldScreen.onCreate` → `WorldOpenFlows.confirmWorldCreation` → `createNewWorld`, which queues a `GenericMessageScreen` ("createWorld.preparing") and creates the world directory. The integrated server then fires `ServerAboutToStart` before `loadLevel` — [CreateWorldScreen.java L100-119, L215-252](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/client/gui/screens/worldselection/CreateWorldScreen.java); [IntegratedServer.java L70-77](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/client/server/IntegratedServer.java)
- Vanilla's own progress view is `LevelLoadingScreen(StoringChunkProgressListener)`. Its `renderChunks` fills one cell per chunk, coloured by `COLORS.getInt(chunkstatus)` — [LevelLoadingScreen.java L20-120](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/client/gui/screens/LevelLoadingScreen.java)
- The drawing primitives: `NativeImage.setPixelRGBA(x, y, abgrColor)`, then `DynamicTexture.upload()`, `TextureManager.register(...)` and `GuiGraphics.blit(...)` — [NativeImage.java L223](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/com/mojang/blaze3d/platform/NativeImage.java); [DynamicTexture.java L44](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/client/renderer/texture/DynamicTexture.java)
- Precedents:
  - RTF's `PresetEditorPage.Preview` (DynamicTexture/NativeImage, render modes, regenerate on edit) — [PresetEditorPage.java](https://github.com/Campione01/ReterraForged/blob/ba35ca764f4ace341c0692d80cfa12342503a94b/common/src/main/java/raccoonman/reterraforged/client/gui/screen/presetconfig/PresetEditorPage.java)
  - Genesis's multithreaded spiral sampling and pregen HUD (agent-read) — [Genesis](https://modrinth.com/project/xXuFelru)
  - Terrarium's live preview beside its settings (agent-read) — [Terrarium README](https://github.com/Gegy/Terrarium/blob/1.12-dev/README.md)
  - TFC's `CreateTFCWorldScreen` — [CreateTFCWorldScreen.java](https://github.com/TerraFirmaCraft/TerraFirmaCraft/blob/1.21.x/src/main/java/net/dries007/tfc/client/screen/CreateTFCWorldScreen.java)
- Spawn and border can be set at creation through `LevelEvent.CreateSpawnPosition`, which is cancellable, carries `ServerLevelData`, and fires once for uninitialized worlds — [LevelEvent.java L101-116](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/neoforged/neoforge/event/level/LevelEvent.java); [MinecraftServer.java L432-444](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/server/MinecraftServer.java)
- On a dedicated server, `level-type=<ns:preset>` is honoured for any registered preset, while `generator-settings` applies only to flat. `ServerAboutToStart` runs before `loadLevel`, and the "Done" message and `RconThread.create` come after — [DedicatedServerProperties.java L207-229](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/server/dedicated/DedicatedServerProperties.java); [DedicatedServer.java L191-209](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/server/dedicated/DedicatedServer.java)
- Chunky's `chunky worldborder` pregenerates exactly the bordered area (agent-read) — [Chunky World borders](https://github.com/pop4959/Chunky/wiki/World-borders)

### Inferences
**Singleplayer flow.** All steps below are proposals.
1. Create World → World Type "Geologic History" (in `#minecraft:normal`) → "Customize", which the PresetEditor enables.
2. **Setup page:** map size (e.g. 4k/8k/16k), plate count, simulated time span, sea level, climate belt, and a seed derived from the world seed. Show a cost estimate.
3. **History screen:**
   - The sim core (pure Java, no Minecraft classes) runs on its own executor at coarse resolution, e.g. a 512–1024² tectonic grid.
   - Every N steps it publishes an immutable frame: elevation, plate ID, crust age, ice, climate, and later rock type and ore provinces.
   - The render thread copies the latest frame into a `NativeImage`, calls `DynamicTexture.upload()` and `blit`s it with hillshading and a legend. Pixel writes must use ABGR.
   - Keep a ring of keyframes for a scrubber and timelapse playback.
   - Layer toggles work like RTF's `RenderMode`.
   - A 3D globe or heightfield mesh is possible with custom GUI rendering, but has no precedent I could verify; a 2D equirectangular or orthographic map is the low-risk choice.
4. **Detail synthesis:** after the history run, a final pass downscales to game resolution (4–16-block rasters, horizon surfaces, deposits). It writes tiles to a staging dir such as `.minecraft/geohist/staging/<simId>/` and shows a final biome preview. That preview samples the *actual* router and `MultiNoiseBiomeSource` built from `worldgenLoadContext()`, at a fixed high Y as Genesis does, so the player sees installed-mod biomes.
5. **Accept:** `updateDimensions` → `replaceOverworldGenerator(new GeologicChunkGenerator(..., simId))`. level.dat stores the generator codec, i.e. the sim id and settings reference.
6. **First server start:** in `ServerAboutToStartEvent`, move or copy the staging dir into `<world>/geohist/` if missing.
7. **Creation:** `CreateSpawnPosition` picks a sim-chosen spawn (e.g. a temperate lowland near a river) and sets the border center and size. Offer "pregenerate now" (Chunky plus DH LOD pregen).

**Dedicated-server flow.** All steps below are proposals.
1. Set `level-type=geohist:geologic`, with sim parameters in a mod config file.
2. **Option A (recommended):** run the same sim core as a CLI (`java -jar geohist-sim.jar --seed … --size 16384 --out world/geohist`). It can emit PNG frames or an MP4 and a static HTML viewer for the history. Then start the server. The same folder can come from a singleplayer run: copy `geohist/`, or the whole world.
3. **Option B:** if `world/geohist/` is missing at `ServerAboutToStartEvent`, run the sim there. Startup blocks, progress goes to the log only, and RCON is unavailable until done.
4. Clients cannot watch a startup-time sim. Letting ops watch would require a post-start `/geohist simulate` into a staging folder, streaming frames over a custom payload, then a restart onto a new world. That is feasible but complex; I did not verify NeoForge payload APIs this session.
5. **Bounded-world distribution:** after full pregeneration inside the border, the world folder is complete. Players can receive it with the pack, and the generator is then only exercised beyond the border.

### Gaps
- I found no source showing a mod replacing `LevelLoadingScreen` with a custom animated map; the agent reported none either.
- Mod-config file placement and lifecycle on NeoForge 1.21.1 (common vs server configs, and when each loads) were not checked.
- NeoForge networking APIs for streaming frames to clients were not checked.

## 5. Performance and storage estimates

### Takeaway
Sim outputs for a 16k×16k-block world are tens to hundreds of MiB (4–16-block rasters plus horizon surfaces). A pregenerated world is several to tens of GiB. Raster lookups add microseconds per chunk against chunk-generation costs of tens of milliseconds, so runtime generation from rasters is cheap. Full pregeneration of 16k×16k (~1.05 M chunks) takes about 1.5–10 hours depending on hardware and C2ME.

### Cited Findings
- **Throughput measurements:**
  - Paper 1.20.4 + Chunky, 4 worker threads, vanilla generation (agent-read): Ryzen 9 7900X3D 183 chunks/s, i7-12700KF 162, Ryzen 7 5800X3D 141, Ryzen 5 5600 132, Ryzen 5 3600 98, i5-4460 57, FX-8350 50 — [Nemez CPU benchmark](https://nemez.net/posts/20240414-minecraft-cpu-performance-testing/page-8/)
  - ishland's C2ME benchmark (I read it directly): MC 1.21.3, C2ME 0.3.0+rc.1.0, dual Xeon Gold 5218R capped at 2 GHz, a 3072-block-radius pregen (148,225 chunks). Vanilla ran 30.31 / 33.04 / 33.96 chunks/s at 10 / 16 / 80 threads; C2ME ran 137.37 / 219.11 / 565.03 — [ishland gist](https://gist.github.com/ishland/9601a033afad4be5b17ffaed859577fd)
  - C2ME is not a universal win: issue #249 is titled "Vanilla world generation/chunkloading seems faster than C2ME" (agent-read) — [C2ME-fabric #249](https://github.com/RelativityMC/C2ME-fabric/issues/249)
- **Storage:** the wiki's region-format page (agent-read, not version-specific) gives typical compressed chunk sizes of about 5–30 KB. Chunky's FAQ declines to give size figures and notes size is non-linear in radius (agent-read) — [Region file format](https://minecraft.wiki/w/Region_file_format); [Chunky wiki](https://github.com/pop4959/Chunky/wiki)
- **Per-chunk evaluation structure:**
  - `FlatCache` evaluates a 2D function on a `(noiseSizeXZ+1)²` quart grid, which is 5×5 = 25 samples per chunk for a 16-block chunk.
  - `cellCountY = height / cellHeight` (384/8 = 48 for the overworld).
  - The aquifer samples `preliminarySurfaceLevel` at 13 offsets per query position, memoized per quart column in a `Long2IntMap`.

  [NoiseChunk.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/NoiseChunk.java); [Aquifer.java](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/levelgen/Aquifer.java)
- A biome section holds 64 quart cells (4×4×4) and a block section holds 4096 cells — [PalettedContainer.java L356, L367](jar:file:///home/greencheetah/Projects/mic/mods/mic-climate/build/moddev/artifacts/neoforge-21.1.249-sources.jar!/net/minecraft/world/level/chunk/PalettedContainer.java)

### Inferences
All figures below are my arithmetic, not measurements.

**World geometry (16,384 × 16,384 blocks):**
- (16384/16)² = 1,048,576 chunks, in (16384/512)² = 1,024 region files.
- At 5–30 KB per chunk, a full pregen is ≈ 5–30 GiB, before DH's LOD database.
- For comparison: 8k×8k is 262,144 chunks (≈ 1.3–7.5 GiB), and 4k×4k is 65,536 chunks (≈ 0.3–2 GiB).

**Raster memory per layer (16k world):**

| Resolution | Cells | float32 | int16 | uint8 |
|---|---|---|---|---|
| 1 block | 268,435,456 | 1 GiB | 512 MiB | 256 MiB |
| 2 blocks | 67,108,864 | 256 MiB | 128 MiB | 64 MiB |
| 4 blocks (quart; matches biome and flat_cache grid) | 16,777,216 | 64 MiB | 32 MiB | 16 MiB |
| 8 blocks | 4,194,304 | 16 MiB | 8 MiB | 4 MiB |
| 16 blocks | 1,048,576 | 4 MiB | 2 MiB | 1 MiB |

**A plausible layer set:**
- Elevation int16 at 2–4 blocks: 128 or 32 MiB.
- Six climate and relief layers at 4–8 blocks, int16: 48–192 MiB.
- Plate or terrane ID, rock age and sediment thickness at 8–16 blocks: a few MiB.
- Total ≈ 100–400 MiB raw. Delta-encoding plus zstd or deflate typically shrinks smooth fields several-fold; I did not measure this.
- Stored as fixed-size tiles (e.g. 512×512) and memory-mapped, it stays off-heap. That matters on a 190-mod client.

**Strata storage.** The best representation is horizon surfaces: each stratigraphic boundary is a 2D elevation raster, deformed by folds and faults, with thrust repeats as extra horizons.
- 32 horizons × int16 at 16-block resolution = 64 MiB; at 8-block resolution = 256 MiB.
- The alternative, per-column stacks at 16-block resolution (≈ 24 layers × 3 bytes × 1.05 M columns), is ≈ 72 MiB plus an index.
- Intrusions are better as a list of 3D bodies (parametric shapes or small voxel blobs) with bounding boxes than as rasters.
- Ore deposits are a sparse list of bodies (type, geometry, grade), kilobytes to a few MiB.

**Lookup cost per chunk:**
- 2D layers through `flat_cache`: 25 bilinear samples per layer per chunk. At ~10–50 ns each, 10 layers cost ≈ 2.5–12.5 µs per chunk.
- Final density at cell corners: 5×5×49 = 1,225 evaluations. A per-block un-interpolated raster term would be 16×16×384 = 98,304 evaluations, ≈ 1–5 ms, which is still affordable.
- Strata pass, done per column: 256 columns × up to ~32 horizon samples ≈ 8k lookups, then run-fills into palettes, ≈ 0.1–0.5 ms per chunk.
- Doing strata per block via a custom surface `RuleSource` would evaluate at every stone position (~50–80k per chunk), and might cost a few ms.
- Against a per-chunk budget of ~20–70 ms of thread time, derived from 183 chunks/s on 4 threads and 137 chunks/s on 10 threads, the added cost is ≲ a few percent.

**Pregeneration time for 1,048,576 chunks:**

| Rate | Time |
|---|---|
| 30 chunks/s (vanilla on a slow server) | ≈ 9.7 h |
| 50 chunks/s | ≈ 5.8 h |
| 100 chunks/s | ≈ 2.9 h |
| 183 chunks/s (fast desktop, 4 threads) | ≈ 1.6 h |
| 565 chunks/s (80-thread server with C2ME) | ≈ 31 min |

An 8k map is a quarter of these times, and a 4k map a sixteenth (e.g. ≈ 11 min at 100 chunks/s).

**Taller dimensions** scale chunk memory, disk, lighting and generation roughly with the number of non-empty sections. Going from 384 to 704 blocks (Tectonic's y640 top) is ≈ 1.8× the sections. The real cost depends on how much of the column is uniform stone, which is cheap as a single-value palette. That last point is an inference; I did not measure it.

**Offline bake comparison:** (a) or (b) would write the same 5–30 GiB up front, and the game would still spend its `features`/`light` work at first load. The raster-driven runtime approach ships ≈ 0.1–0.5 GiB and lets Chunky and C2ME do the rest in parallel.

### Gaps
- I found no measured per-chunk timing for vanilla stage breakdown (noise vs surface vs features vs light) on 1.21.1. The figures above derive per-chunk budgets from throughput only.
- I found no 1.21-specific KB-per-chunk figure, and no measured 10k-radius pregen size.
- Distant Horizons LOD generation speed and database size are unknown; there was no primary source.
- Compression ratios for sim rasters are asserted from general experience, not measured.
- For later versions (1.21.2+ / 26.x), the only change sourced this session is the density-function changes noted in §2 (26.3-snap10; `find_top_surface` and `invert` added in 1.21.9, agent-read from the wiki). Other API changes that would affect a port were not checked: carver-step removal, `NativeImage` pixel API, and GUI blit signature changes.
