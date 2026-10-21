// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.farterrain;

import com.minecraftindustrialcomplex.deeptime.climate.ColdBiomeSearch;
import com.minecraftindustrialcomplex.deeptime.rock.PaletteRules;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.worlddata.Columns;
import com.minecraftindustrialcomplex.deeptime.worlddata.FarColumn;
import com.minecraftindustrialcomplex.deeptime.worlddata.IceCover;
import com.minecraftindustrialcomplex.deeptime.worlddata.RasterSampler;
import com.minecraftindustrialcomplex.deeptime.worlddata.Water;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.DelegatingRuleSource;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.StrataPass;
import com.minecraftindustrialcomplex.deeptime.worldgen.WaterBodies;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.core.QuartPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.BiomeSource;
import net.minecraft.world.level.biome.Climate;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;

import java.util.List;
import java.util.Optional;

/**
 * Far terrain of one Deep Time level (plan §10.5 P3-04): the columns of an area as layers of
 * blocks, straight from the world data, without generating a chunk. {@code compat/dh} hands them
 * to Distant Horizons as level-of-detail data; nothing here knows Distant Horizons.
 *
 * <p>A column is what {@link FarColumn} decides (the terrain height the generator draws, the
 * lake, sea or river over it, banks, the ice cover, the strata bed by bed where a cliff shows
 * them) with three things only the game knows:
 * <ul>
 *   <li><b>the biome</b>, from the level's own biome source and climate sampler at the column's
 *       top, at the quart cell a real chunk stores it in;</li>
 *   <li><b>the soil</b>, from the surface table ({@link FarSurface}) for that biome, and whether
 *       the biome freezes there ({@code Biome.coldEnoughToSnow}: ice on water, a snow layer on
 *       ground, as vanilla's {@code freeze_top_layer} lays them); the top block of dry ground, of
 *       the sea bed and of a lake's bed from the level's own surface rule wherever the part of it
 *       that reads no chunk decides it ({@link PackSurfaceRule}: Expanded Ecosphere's coral and sand
 *       at y 58 to 62), and a sea bed in vanilla's shallow band drawn as a lake's bed (BAK-050);</li>
 *   <li><b>the blocks</b>: rock classes through the {@link RockPalette} with the strata pass's
 *       member choice ({@link StrataPass#horizonKey}), so a bed is the same block far and near.</li>
 * </ul>
 *
 * <p><b>Wrapping.</b> Every sample folds x into the canonical lap first, as Toroidal World folds
 * the chunks it generates, so an area on any lap, or across the seam, is the planet's terrain.
 * Past the pole edge (Lambert) the rasters clamp, past a Petroff–Guyou world's turning edges they fold
 * (GridDesc.cell), exactly as the real generator reads them.
 *
 * <p><b>What is not here.</b> Trees and other features, caves, and the parts of the pack's surface
 * rules that read a chunk (its noise, its heightmap). Thread-safe: one scratch per thread, everything
 * read is immutable.
 */
public final class FarTerrain {

    /** Receives the columns of an area, in row order (z outer, x inner). */
    public interface Sink {
        /**
         * One column: {@code count} layers bottom-up, layer i filling blocks
         * {@code bottom[i] <= y < top[i]}, from the level's minimum y to its top without gaps;
         * {@code material} is each layer's {@link FarColumn} material. The arrays are reused for
         * the next column.
         */
        void column(int relX, int relZ, int count, int[] bottom, int[] top, BlockState[] block, int[] material, int[] skyLight,
                    Holder<Biome> biome);
    }

    /** The widest strata detail kept below a column's soil: the drop a cliff shows, capped. */
    public static final int MAX_ROCK_DEPTH = 256;
    /** Strata kept below the soil of every column, however flat the ground: a neighbour's soil may be thinner. */
    public static final int ROCK_MARGIN = 8;

    private final ServerLevel level;
    private final WorldDataTerrain terrain;
    private final BiomeSource biomes;
    private final Climate.Sampler sampler;
    private final FarSurface surface;
    /** The part of the level's surface rule run here (none when none can run without a chunk: the table then gives every top block). */
    private final PackSurfaceRule packRule;
    private final int levelMinY, levelMaxY;
    private final int cell;

    private FarTerrain(ServerLevel level, GeologicChunkGenerator generator, WorldDataTerrain terrain, FarSurface surface) {
        this.level = level;
        this.terrain = terrain;
        BiomeSource source = generator.getBiomeSource();
        Climate.Sampler climate = level.getChunkSource().randomState().sampler();
        this.biomes = source;
        this.sampler = climate;
        this.surface = surface;
        this.levelMinY = level.getMinBuildHeight();
        this.levelMaxY = level.getMaxBuildHeight() - 1;
        this.cell = generator.waterReach();
        // The rule SurfaceSystem paints this level's chunks with (NoiseBasedChunkGenerator.buildSurface
        // reads the registry settings), seeing the biomes the chunks store: x folded into the lap.
        this.packRule = PackSurfaceRule.of(level, generator, generator.sourceSettings().value().noiseSettings(), DelegatingRuleSource.levelRule(generator.sourceSettings().value().surfaceRule()),
                (qx, qy, qz) -> noiseBiome(QuartPos.fromBlock(terrain.wrapX(QuartPos.toBlock(qx))), qy, qz));
    }

    /** TEMPORARY experiment switch: false runs the biome searches warm, as before. */
    public static volatile boolean coldSearch = true;

    /**
     * The biome source at quart cell (qx, qy, qz), x already folded, searched cold
     * ({@link ColdBiomeSearch}): the answer at a tie between equally near climate entries is the
     * same on every thread, whatever the thread built before.
     */
    private Holder<Biome> noiseBiome(int qx, int qy, int qz) {
        if (!coldSearch)
            return biomes.getNoiseBiome(qx, qy, qz, sampler);
        ColdBiomeSearch.begin();
        try {
            return biomes.getNoiseBiome(qx, qy, qz, sampler);
        } finally {
            ColdBiomeSearch.end();
        }
    }

    /** Far terrain for {@code level}, when it is a Deep Time level generating from world data. */
    public static Optional<FarTerrain> of(ServerLevel level) {
        return of(level, null);
    }

    /** As {@link #of(ServerLevel)} with a given surface table (null reads the server's datapacks). */
    public static Optional<FarTerrain> of(ServerLevel level, FarSurface surface) {
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g) || !(g.terrain() instanceof WorldDataTerrain t))
            return Optional.empty();
        FarSurface table = surface != null ? surface : FarSurface.read(level.getServer().getResourceManager());
        return Optional.of(new FarTerrain(level, g, t, table));
    }

    public ServerLevel level() {
        return level;
    }

    public FarSurface surface() {
        return surface;
    }

    public WorldData data() {
        return terrain.data();
    }

    private static final class Scratch {
        FarColumn.Shape[] shapes = new FarColumn.Shape[0];
        final FarColumn.Layers layers = new FarColumn.Layers();
        final FarColumn.Soil soil = new FarColumn.Soil();
        Columns.Stack stack = new Columns.Stack(8);
        BlockState[] blocks = new BlockState[16];
        int[] light = new int[16];
        Holder<Biome>[] biomeCache;
        int[] biomeCacheY = new int[0];
        final BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();

        @SuppressWarnings("unchecked")
        void size(int m, int biomeCells) {
            if (shapes.length < m * m) {
                shapes = new FarColumn.Shape[m * m];
                for (int i = 0; i < shapes.length; i++)
                    shapes[i] = new FarColumn.Shape();
            }
            if (biomeCacheY.length < biomeCells) {
                biomeCache = new Holder[biomeCells];
                biomeCacheY = new int[biomeCells];
            }
            java.util.Arrays.fill(biomeCacheY, 0, biomeCells, Integer.MIN_VALUE);
        }
    }

    private static final ThreadLocal<Scratch> SCRATCH = ThreadLocal.withInitial(Scratch::new);

    /** The blocks of the ice cover, resolved once per area. */
    private record Cover(IceCover.Spec spec, BlockState snow, BlockState ice, BlockState deepIce) {
        static Cover of(WorldData wd) {
            IceCover.Spec s = wd.iceCover().map(IceCover::spec).orElse(null);
            if (s == null)
                return new Cover(null, Blocks.SNOW_BLOCK.defaultBlockState(), Blocks.PACKED_ICE.defaultBlockState(),
                        Blocks.BLUE_ICE.defaultBlockState());
            return new Cover(s, block(s.snow(), Blocks.SNOW_BLOCK), block(s.ice(), Blocks.PACKED_ICE), block(s.deepIce(), Blocks.BLUE_ICE));
        }

        private static BlockState block(String id, Block fallback) {
            ResourceLocation rl = ResourceLocation.tryParse(id);
            return (rl == null ? Optional.<Block>empty() : BuiltInRegistries.BLOCK.getOptional(rl)).orElse(fallback).defaultBlockState();
        }
    }

    private static final BlockState AIR = Blocks.AIR.defaultBlockState(), WATER = Blocks.WATER.defaultBlockState(),
            ICE = Blocks.ICE.defaultBlockState(), SNOW = Blocks.SNOW.defaultBlockState(), STONE = Blocks.STONE.defaultBlockState(),
            GRAVEL = Blocks.GRAVEL.defaultBlockState(), SAND = Blocks.SAND.defaultBlockState();

    /** The biome a real chunk stores for block (x, y, z): the biome source at its quart cell, x folded. */
    public Holder<Biome> biomeAt(int x, int y, int z) {
        return noiseBiome(QuartPos.fromBlock(terrain.wrapX(x)), QuartPos.fromBlock(y), QuartPos.fromBlock(z));
    }

    /**
     * Sends {@code sink} the n × n columns, each {@code step} blocks wide (a power of two), of the
     * area whose minimum corner is block (x0, z0), on any lap. A column wider than a block is
     * sampled at its centre.
     */
    public void build(int x0, int z0, int step, int n, Sink sink) {
        if (step < 1 || Integer.bitCount(step) != 1 || n < 1)
            throw new IllegalArgumentException("far terrain: step " + step + " (a power of two), n " + n);
        WorldData wd = terrain.data();
        Water.Spec spec = Water.Spec.of(wd, cell);   // the real chunk's own constants (WaterBodies)
        int m = n + 2;
        // Columns of one quart cell share a biome lookup while they share its y.
        boolean quartCache = step < 4 && Math.floorMod(x0, 4) == 0 && Math.floorMod(z0, 4) == 0;
        int perQuart = 4 / Math.min(step, 4), quarts = quartCache ? (n + perQuart - 1) / perQuart : 0;
        Scratch s = SCRATCH.get();
        s.size(m, quarts * quarts);
        FarColumn.grid(wd, spec, x0, z0, step, n, s.shapes);

        RasterSampler strata = wd.has(WorldData.HORIZONS) ? wd.layer(WorldData.HORIZONS) : null;
        PaletteRules.Result<BlockState>[] rocks = strata == null ? null : RockPalette.get().forClasses(wd.classes());
        Cover cover = Cover.of(wd);
        int half = step / 2;
        FarColumn.Layers layers = s.layers;
        for (int j = 0; j < n; j++) {
            for (int i = 0; i < n; i++) {
                FarColumn.Shape shape = s.shapes[(j + 1) * m + (i + 1)];
                int x = x0 + i * step + half, z = z0 + j * step + half;
                int wx = terrain.wrapX(x);
                int topY = Math.max(spec.minY(), shape.water() ? Math.min(shape.waterTop, spec.maxY()) : shape.ground);

                Holder<Biome> biome;
                if (quartCache) {
                    int q = (j / perQuart) * quarts + (i / perQuart), qy = QuartPos.fromBlock(topY);
                    if (s.biomeCacheY[q] != qy) {
                        s.biomeCache[q] = biomeAt(wx, topY, z);
                        s.biomeCacheY[q] = qy;
                    }
                    biome = s.biomeCache[q];
                } else {
                    biome = biomeAt(wx, topY, z);
                }
                FarSurface.Choice soil = surface.of(biome);
                // The top block where the part of the level's surface rule run here decides it: on
                // dry ground, the sea bed and a lake's bed (the water pass fills a lake after the
                // rule, so the rule saw it dry, and leaves its bed as the rule painted it, grass as
                // dirt); a river's bed is the water pass's own sand or gravel.
                BlockState packTop = null;   // ledger: BAK-048
                // Where the table stands in, a sea bed in vanilla's shallow band is painted as under
                // a dry floor's top: the lake's bed, grass as dirt (BAK-050); the table's sea bed is deeper.
                BlockState seaBed = soil.seaBed();
                if (shape.ground >= levelMinY && shape.ground <= levelMaxY
                        && (!shape.water() || shape.kind == Water.Kind.SEA || shape.kind == Water.Kind.LAKE)) {
                    int waterHeight = shape.water() && shape.kind == Water.Kind.SEA ? shape.waterTop + 1 : Integer.MIN_VALUE;
                    packTop = packRule.top(wx, shape.ground, z, waterHeight);
                    if (packTop == null && waterHeight != Integer.MIN_VALUE && packRule.shallowFloor(wx, shape.ground, z, waterHeight))
                        seaBed = soil.lakeBed();
                }
                boolean bare = soil.bare();
                int depth = soil.depth();
                if (packTop != null && StrataPass.isCrust(packTop)) {
                    bare = true;   // the rule leaves the stone: the strata pass paints it
                    packTop = null;
                } else if (packTop != null && bare) {
                    bare = false;  // a bare biome's top block painted: one block over the rock
                    depth = 1;
                }
                s.soil.depth = depth;
                s.soil.bare = bare;
                s.soil.frozen = topY < spec.maxY() && biome.value().coldEnoughToSnow(s.pos.set(wx, topY + 1, z));

                int exposure = FarColumn.exposure(s.shapes, n, i, j);
                int rockDepth = Math.min(MAX_ROCK_DEPTH, exposure + ROCK_MARGIN);
                // The strata column rule is most of a narrow column's cost, so it is spent where
                // rock shows: at the surface (a bare biome) or on a side that drops below the soil.
                // A column whose rock is buried takes its nearest strata cell's stack.
                boolean rockShows = bare || exposure >= depth;
                if (strata != null)
                    FarColumn.rock(wd, strata, wx, z, step <= FarColumn.FULL_RULE_MAX_STEP && rockShows, s.stack);
                FarColumn.layers(levelMinY, levelMaxY, shape, s.soil, strata == null ? null : s.stack, rockDepth, cover.spec(), layers);

                if (s.blocks.length < layers.count) {
                    s.blocks = new BlockState[layers.bottom.length];
                    s.light = new int[layers.bottom.length];
                }
                // Light: what stands in the open is lit; what the ground buries is dark. A column's
                // exposed side counts as open (the renderer lights a side from the air beside it).
                int topSolid = layers.topSolid();
                int openFrom = shape.ground - exposure;
                int waterDepth = shape.water() ? Math.min(shape.waterTop, spec.maxY()) - shape.ground : 0;
                for (int k = 0; k < layers.count; k++) {
                    s.blocks[k] = block(layers, k, soil, packTop, seaBed, cover, rocks);
                    int light;
                    if (k > topSolid)
                        light = 15;
                    else if (k == topSolid)
                        light = Math.max(0, 15 - waterDepth);
                    else
                        light = waterDepth == 0 && layers.top[k] > openFrom ? 15 : 0;
                    s.light[k] = light;
                }
                sink.column(i, j, layers.count, layers.bottom, layers.top, s.blocks, layers.material, s.light, biome);
            }
        }
    }

    /** Layer k's block; {@code packTop}, when not null, is the top block the level's surface rule gives (dry ground, sea bed, lake bed), {@code seaBed} the sea bed's otherwise (BAK-050). */
    private static BlockState block(FarColumn.Layers layers, int k, FarSurface.Choice soil, BlockState packTop, BlockState seaBed, Cover cover,
                                    PaletteRules.Result<BlockState>[] rocks) {
        switch (layers.material[k]) {
            case FarColumn.AIR:
                return AIR;
            case FarColumn.ROCK: {
                int cls = layers.cls[k];
                PaletteRules.Result<BlockState> r = rocks != null && cls >= 0 && cls < rocks.length ? rocks[cls] : null;
                BlockState rock = r == null ? null : r.pick(StrataPass.horizonKey(layers.unit[k], cls));
                return rock == null ? STONE : rock;   // the unclassified crust
            }
            case FarColumn.SOIL_TOP:
                return packTop != null ? packTop : soil.top();
            case FarColumn.SOIL_UNDER:
                return soil.under();
            case FarColumn.SEA_BED:
                return packTop != null ? packTop : seaBed;
            case FarColumn.LAKE_BED:
                return packTop != null ? WaterBodies.underWater(packTop) : soil.lakeBed();
            case FarColumn.RIVER_GRAVEL:
                return GRAVEL;
            case FarColumn.RIVER_SAND:
                return SAND;
            case FarColumn.WATER:
                return WATER;
            case FarColumn.WATER_ICE:
                return ICE;
            case FarColumn.COVER_SNOW:
                return cover.snow();
            case FarColumn.COVER_ICE:
                return cover.ice();
            case FarColumn.COVER_DEEP_ICE:
                return cover.deepIce();
            case FarColumn.SNOW_LAYER:
                return SNOW;
            default:
                throw new IllegalStateException("far terrain: material " + layers.material[k]);
        }
    }

    /**
     * One column of far terrain, for probes and tests.
     *
     * @param ground      the y of the top solid block (below the level's minimum y for none)
     * @param groundBlock that block
     * @param top         the y of the highest block that is not air (the water's top, a snow layer)
     * @param topBlock    that block
     * @param water       whether water (or its ice) stands over the ground
     * @param waterTop    the y of the top water or ice block, when {@code water}
     * @param layers      every layer as "bottom..top block", bottom-up
     */
    public record Column(int x, int z, int ground, BlockState groundBlock, int top, BlockState topBlock, boolean water,
                         int waterTop, Holder<Biome> biome, List<String> layers) {}

    /** The one-block column at (x, z) as far terrain at full detail shows it. */
    public Column column(int x, int z) {
        Column[] out = new Column[1];
        build(x, z, 1, 1, (relX, relZ, count, bottom, top, block, material, light, biome) -> {
            int ground = levelMinY - 1, last = levelMinY - 1, waterTop = Integer.MIN_VALUE;
            BlockState groundBlock = AIR, topBlock = AIR;
            List<String> desc = new java.util.ArrayList<>();
            for (int k = 0; k < count; k++) {
                desc.add(bottom[k] + ".." + (top[k] - 1) + " " + BuiltInRegistries.BLOCK.getKey(block[k].getBlock()));
                int mat = material[k];
                if (mat == FarColumn.AIR)
                    continue;
                last = top[k] - 1;
                topBlock = block[k];
                if (mat == FarColumn.WATER || mat == FarColumn.WATER_ICE) {
                    waterTop = top[k] - 1;
                } else if (mat != FarColumn.SNOW_LAYER) {
                    ground = top[k] - 1;
                    groundBlock = block[k];
                }
            }
            out[0] = new Column(x, z, ground, groundBlock, last, topBlock, waterTop != Integer.MIN_VALUE, waterTop, biome, desc);
        });
        return out[0];
    }
}
