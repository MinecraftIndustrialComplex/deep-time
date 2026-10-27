// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import it.unimi.dsi.fastutil.objects.ObjectLists;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.core.QuartPos;
import net.minecraft.core.Registry;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.Registries;
import net.minecraft.server.level.WorldGenRegion;
import net.minecraft.util.Mth;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.StructureManager;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.BiomeGenerationSettings;
import net.minecraft.world.level.biome.BiomeManager;
import net.minecraft.world.level.biome.BiomeSource;
import net.minecraft.world.level.biome.Biomes;
import net.minecraft.world.level.biome.Climate;
import net.minecraft.world.level.block.Rotation;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.CarvingMask;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.LevelChunkSection;
import net.minecraft.world.level.chunk.ProtoChunk;
import net.minecraft.world.level.chunk.UpgradeData;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.minecraft.world.level.levelgen.Aquifer;
import net.minecraft.world.level.levelgen.Beardifier;
import net.minecraft.world.level.levelgen.GenerationStep;
import net.minecraft.world.level.levelgen.Heightmap;
import net.minecraft.world.level.levelgen.LegacyRandomSource;
import net.minecraft.world.level.levelgen.NoiseChunk;
import net.minecraft.world.level.levelgen.NoiseGeneratorSettings;
import net.minecraft.world.level.levelgen.NoiseSettings;
import net.minecraft.world.level.levelgen.RandomState;
import net.minecraft.world.level.levelgen.RandomSupport;
import net.minecraft.world.level.levelgen.WorldGenerationContext;
import net.minecraft.world.level.levelgen.WorldgenRandom;
import net.minecraft.world.level.levelgen.blending.Blender;
import net.minecraft.world.level.levelgen.carver.CarvingContext;
import net.minecraft.world.level.levelgen.carver.ConfiguredWorldCarver;
import net.minecraft.world.level.levelgen.structure.pools.JigsawJunction;

import java.util.ArrayList;
import java.util.EnumSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * What a cube world generates off its six faces (cube plan G5, section 1.4; docs/design/cube-game.md
 * section 3): every chunk wholly off the faces is an exact copy of its partner chunk's own generation
 * up to its carvers, turned by the motion of the net edge whose band holds it, so the view across a
 * turning seam is the joined side's terrain and the wrap engine's band can take it as its content.
 *
 * <p>A chunk is on a face or wholly off the faces (every face edge is a multiple of s/2, and s of
 * 256). The motions are turns about chunk corners, so a chunk maps onto one whole partner chunk; the
 * chunks a void square's diagonal cuts have two partners, one per side of the diagonal. For each
 * (partner chunk, motion) of a chunk's columns, a scratch {@link ProtoChunk} at the partner (at
 * Toroidal World's own x, where the partner is generated) runs the generator's own steps: the biomes,
 * the noise fill (with no structure's terrain adaptation), the surface rules with a biome manager that
 * answers as the partner's region does, Deep Time's passes, the air carvers as
 * {@code NoiseBasedChunkGenerator.applyCarvers} runs them and the water's wake. The columns are then
 * copied block for block, each state turned by {@link BlockState#rotate}, and the fluid updates the
 * fill and the carvers asked for carried over, as is the partner's air carving mask (what the carvers
 * cut and the water Deep Time kept from them), so the copy tells a carver's cut apart as a chunk on a
 * face does. Columns past the net's corners are deep ocean. Every
 * biome search of a cube world's chunks starts cold ({@code ColdBiomeSearch}, the generator's
 * {@code createBiomes}, {@code buildSurface} and {@code applyCarvers}), so at a climate tie a copy
 * holds its partner's biome.
 *
 * <p>Not copied: the terrain a structure adapts round its pieces (the partner's structure starts are
 * not known here) and features (trees, ores placed as features, springs). A wholly outside chunk
 * gets no structure starts, no carvers of its own, no features and no mobs (ledger SCL-210); both
 * missing parts belong to the wrap engine's band (cube plan E3).
 */
public final class CubeFill {

    /** The floor of the deep ocean past the net's corners, blocks below sea level. */
    public static final int OCEAN_DEPTH = 64; // ledger: SCL-211 (game-choice)

    private CubeFill() {}

    /** Whether chunk {@code pos} lies wholly off the faces of {@code cube}. */
    public static boolean outside(CubeLayout cube, ChunkPos pos) {
        return cube.faceOfBlock(pos.getMinBlockX(), pos.getMinBlockZ()) == null;
    }

    /** x of a block in Toroidal World's frame [−2s, 2s), where its chunk is generated. */
    public static long twX(CubeLayout cube, long x) {
        long p = cube.period();
        return Math.floorMod(x + p / 2, p) - p / 2;
    }

    /** The rotation that turns a partner's block state into the outside chunk's frame, for a fold of k quarter turns. */
    public static Rotation rotation(int k) {
        // The fold's motion takes the outside block to its partner; a state read there turns back.
        return switch (Math.floorMod(k, 4)) {
            case 1 -> Rotation.COUNTERCLOCKWISE_90;
            case 2 -> Rotation.CLOCKWISE_180;
            case 3 -> Rotation.CLOCKWISE_90;
            default -> Rotation.NONE;
        };
    }

    /** The deep ocean biome among the biome source's, or null. */
    public static Holder<Biome> deepOcean(BiomeSource source) {
        for (Holder<Biome> h : source.possibleBiomes())
            if (h.is(Biomes.DEEP_OCEAN))
                return h;
        return null;
    }

    /**
     * The biome a cube world's chunks hold at quart (qx, qy, qz): the biome source's on a face, the
     * partner quart's off the faces (quarts map onto quarts), deep ocean past the net's corners.
     */
    public static Holder<Biome> foldedBiome(CubeLayout cube, BiomeSource source, Climate.Sampler sampler, Holder<Biome> ocean,
                                            int qx, int qy, int qz) {
        CubeLayout.Fold f = cube.fold(4L * qx + 1, 4L * qz + 1);
        if (f instanceof CubeLayout.Outside o)
            return source.getNoiseBiome((int) Math.floorDiv(twX(cube, o.x()), 4), qy, (int) Math.floorDiv(o.z(), 4), sampler);
        if (f instanceof CubeLayout.Ocean && ocean != null)
            return ocean;
        return source.getNoiseBiome(qx, qy, qz, sampler);
    }

    /** The BIOMES step of a wholly outside chunk ({@link #foldedBiome}). */
    static void biomes(GeologicChunkGenerator g, CubeLayout cube, RandomState random, ChunkAccess chunk) {
        BiomeSource source = g.getBiomeSource();
        Holder<Biome> ocean = deepOcean(source);
        chunk.fillBiomesFromNoise((qx, qy, qz, sampler) -> foldedBiome(cube, source, sampler, ocean, qx, qy, qz),
                random.sampler());
    }

    private record Group(ChunkPos partner, CubeLayout.Iso iso) {}

    /** The SURFACE step of a wholly outside chunk: the partners' copies and the deep ocean (see the class comment). */
    static void surface(GeologicChunkGenerator g, CubeLayout cube, WorldGenRegion region, StructureManager structures,
                        RandomState random, ChunkAccess chunk) {
        ChunkPos pos = chunk.getPos();
        Map<Group, List<Integer>> groups = new LinkedHashMap<>();
        List<Integer> ocean = new ArrayList<>();
        for (int lz = 0; lz < 16; lz++)
            for (int lx = 0; lx < 16; lx++) {
                CubeLayout.Fold f = cube.fold(pos.getMinBlockX() + lx, pos.getMinBlockZ() + lz);
                if (f instanceof CubeLayout.Outside o) {
                    ChunkPos p = new ChunkPos((int) Math.floorDiv(twX(cube, o.x()), 16), (int) Math.floorDiv(o.z(), 16));
                    groups.computeIfAbsent(new Group(p, o.iso()), k -> new ArrayList<>()).add(lx | lz << 4);
                } else {
                    ocean.add(lx | lz << 4);
                }
            }
        for (var e : groups.entrySet())
            copy(cube, partner(g, cube, region, structures, random, e.getKey().partner()), e.getKey().iso(), chunk,
                    e.getValue());
        if (!ocean.isEmpty())
            ocean(g, chunk, ocean);
        Heightmap.primeHeightmaps(chunk, EnumSet.of(Heightmap.Types.OCEAN_FLOOR_WG, Heightmap.Types.WORLD_SURFACE_WG));
    }

    /** The partner chunk {@code p} as the generator makes it up to its carvers, in a scratch chunk no level holds. */
    static ProtoChunk partner(GeologicChunkGenerator g, CubeLayout cube, WorldGenRegion region, StructureManager structures,
                              RandomState random, ChunkPos p) {
        var level = region.getLevel();
        Registry<Biome> biomes = region.registryAccess().registryOrThrow(Registries.BIOME);
        ProtoChunk c = new ProtoChunk(p, UpgradeData.EMPTY, level, biomes, null);
        NoiseGeneratorSettings settings = g.sourceSettings().value();
        // The Beardifier a chunk with no structure piece near it has: no terrain adaptation.
        Beardifier none = new Beardifier(ObjectLists.<Beardifier.Rigid>emptyList().iterator(),
                ObjectLists.<JigsawJunction>emptyList().iterator());
        NoiseChunk noise = NoiseChunk.forChunk(c, random, none, settings, g.globalFluidPicker.get(), Blender.empty());
        c.getOrCreateNoiseChunk(x -> noise);
        g.doCreateBiomes(Blender.empty(), random, structures, c);
        // Each step's status as the chunk pipeline would set it: ProtoChunk.getNoiseBiome refuses
        // before BIOMES, and the surface rules read the chunk's own quarts.
        c.setPersistedStatus(ChunkStatus.BIOMES);
        NoiseSettings ns = settings.noiseSettings().clampToHeightAccessor(c.getHeightAccessorForGeneration());
        int cellCountY = Mth.floorDiv(ns.height(), ns.getCellHeight());
        if (cellCountY > 0)
            g.doFill(Blender.empty(), structures, random, c, Mth.floorDiv(ns.minY(), ns.getCellHeight()), cellCountY);
        c.setPersistedStatus(ChunkStatus.NOISE);
        // The partner's region answers its own quarts from the chunk, its neighbours' as they hold them.
        BiomeSource source = g.getBiomeSource();
        Holder<Biome> ocean = deepOcean(source);
        Climate.Sampler sampler = random.sampler();
        BiomeManager manager = new BiomeManager((qx, qy, qz) -> (qx >> 2) == p.x && (qz >> 2) == p.z
                ? c.getNoiseBiome(qx, qy, qz) : foldedBiome(cube, source, sampler, ocean, qx, qy, qz),
                BiomeManager.obfuscateSeed(region.getSeed()));
        g.buildSurface(c, new WorldGenerationContext(g, level), random, structures, manager, biomes, Blender.empty());
        g.deepTimeSurface(c);
        c.setPersistedStatus(ChunkStatus.SURFACE);
        carve(g, region.registryAccess(), region.getSeed(), random, c, noise);
        g.afterCarvers(c);
        c.setPersistedStatus(ChunkStatus.CARVERS);
        return c;
    }

    /** The air carvers of {@code chunk}, as {@code NoiseBasedChunkGenerator.applyCarvers} runs them (its neighbours' biomes from the biome source). */
    static void carve(GeologicChunkGenerator g, RegistryAccess access, long seed, RandomState random, ProtoChunk chunk,
                      NoiseChunk noise) {
        BiomeSource source = g.getBiomeSource();
        Climate.Sampler sampler = random.sampler();
        BiomeManager biomes = new BiomeManager((x, y, z) -> source.getNoiseBiome(x, y, z, sampler), BiomeManager.obfuscateSeed(seed));
        WorldgenRandom rng = new WorldgenRandom(new LegacyRandomSource(RandomSupport.generateUniqueSeed()));
        ChunkPos pos = chunk.getPos();
        Aquifer aquifer = noise.aquifer();
        CarvingContext ctx = new CarvingContext(g, access, chunk.getHeightAccessorForGeneration(), noise, random,
                g.sourceSettings().value().surfaceRule());
        GenerationStep.Carving step = GenerationStep.Carving.AIR;
        CarvingMask mask = chunk.getOrCreateCarvingMask(step);
        for (int dx = -8; dx <= 8; dx++)
            for (int dz = -8; dz <= 8; dz++) {
                ChunkPos from = new ChunkPos(pos.x + dx, pos.z + dz);
                BiomeGenerationSettings settings = g.getBiomeGenerationSettings(source.getNoiseBiome(
                        QuartPos.fromBlock(from.getMinBlockX()), 0, QuartPos.fromBlock(from.getMinBlockZ()), sampler));
                int l = 0;
                for (Holder<ConfiguredWorldCarver<?>> h : settings.getCarvers(step)) {
                    ConfiguredWorldCarver<?> carver = h.value();
                    rng.setLargeFeatureSeed(seed + (long) l, from.x, from.z);
                    if (carver.isStartChunk(rng))
                        carver.carve(ctx, chunk, biomes::getBiome, rng, aquifer, from, mask);
                    l++;
                }
            }
    }

    /** Copies the columns {@code cols} (lx | lz << 4) of {@code chunk} from {@code src} through the fold {@code iso}. */
    static void copy(CubeLayout cube, ProtoChunk src, CubeLayout.Iso iso, ChunkAccess chunk, List<Integer> cols) {
        ChunkPos pos = chunk.getPos();
        Rotation rot = rotation(iso.k());
        int sections = chunk.getSectionsCount();
        CarvingMask cut = src.getCarvingMask(GenerationStep.Carving.AIR);
        CarvingMask into = cut != null && chunk instanceof ProtoChunk p ? p.getOrCreateCarvingMask(GenerationStep.Carving.AIR) : null;
        boolean[] mine = new boolean[256];
        for (int col : cols) {
            mine[col] = true;
            int lx = col & 15, lz = col >> 4;
            long bx = pos.getMinBlockX() + lx, bz = pos.getMinBlockZ() + lz;
            int sx = (int) (iso.cellX(bx, bz) & 15), sz = (int) (iso.cellZ(bx, bz) & 15);
            if (into != null)
                for (int y = chunk.getMinBuildHeight(); y < chunk.getMaxBuildHeight(); y++)
                    if (cut.get(sx, y, sz))
                        into.set(lx, y, lz);
            for (int si = 0; si < sections; si++) {
                LevelChunkSection from = src.getSection(si);
                if (from.hasOnlyAir())
                    continue;
                LevelChunkSection to = chunk.getSection(si);
                for (int ly = 0; ly < 16; ly++) {
                    BlockState s = from.getBlockState(sx, ly, sz);
                    if (!s.isAir())
                        to.setBlockState(lx, ly, lz, rot == Rotation.NONE ? s : s.rotate(rot), false);
                }
            }
        }
        // The fluid updates the fill and the carvers asked for, at their places in this chunk: the
        // partner's block in the window (Toroidal World's x moved by whole periods), then back.
        var marks = src.getPostProcessing();
        CubeLayout.Iso back = iso.inverse();
        long ax = iso.cellX(pos.getMinBlockX(), pos.getMinBlockZ());
        long shift = ax - twX(cube, ax);
        for (int si = 0; si < marks.length; si++) {
            if (marks[si] == null)
                continue;
            int sy = src.getSectionYFromSectionIndex(si);
            for (short packed : marks[si]) {
                BlockPos b = ProtoChunk.unpackOffsetCoordinates(packed, sy, src.getPos());
                long wx = b.getX() + shift, wz = b.getZ();
                long ox = back.cellX(wx, wz), oz = back.cellZ(wx, wz);
                if (Math.floorDiv(ox, 16) == pos.x && Math.floorDiv(oz, 16) == pos.z
                        && mine[(int) (ox & 15) | (int) (oz & 15) << 4])
                    chunk.markPosForPostprocessing(new BlockPos((int) ox, b.getY(), (int) oz));
            }
        }
    }

    /** Deep ocean in the columns {@code cols}: the default block up to sea level − {@link #OCEAN_DEPTH}, the sea above. */
    static void ocean(GeologicChunkGenerator g, ChunkAccess chunk, List<Integer> cols) {
        NoiseGeneratorSettings settings = g.sourceSettings().value();
        int sea = settings.seaLevel(), floor = sea - OCEAN_DEPTH;
        BlockState rock = settings.defaultBlock(), water = settings.defaultFluid();
        int minY = chunk.getMinBuildHeight();
        for (int col : cols) {
            int lx = col & 15, lz = col >> 4;
            for (int y = minY; y < sea; y++) {
                LevelChunkSection s = chunk.getSection(chunk.getSectionIndex(y));
                s.setBlockState(lx, y & 15, lz, y < floor ? rock : water, false);
            }
        }
    }
}
