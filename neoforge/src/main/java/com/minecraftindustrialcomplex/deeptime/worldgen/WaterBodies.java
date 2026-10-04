// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.worlddata.LakeIce;
import com.minecraftindustrialcomplex.deeptime.worlddata.Rivers;
import com.minecraftindustrialcomplex.deeptime.worlddata.Water;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import net.minecraft.core.BlockPos;
import net.minecraft.tags.BlockTags;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.SpreadingSnowyDirtBlock;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.CarvingMask;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.ProtoChunk;
import net.minecraft.world.level.levelgen.GenerationStep;
import net.minecraft.world.level.levelgen.Heightmap;
import net.minecraft.world.level.material.Fluids;

/**
 * Lakes, rivers and their banks (plan §7.9 and §8.4, P1-22): the pass that makes every column of a
 * chunk what {@link Water} decides for it. It runs after {@link StrataPass}, before carvers and
 * features, and replaces P0-13's lake-only pass ({@link WaterPass} delegates here).
 *
 * <ul>
 *   <li><b>River</b> columns are cut (or filled) to the channel's bed, water from the bed to the
 *       channel's top, air above: gravel on the bed of a narrow channel, sand on a wide one,
 *       never over air.</li>
 *   <li><b>Lake</b> columns are cut (or filled) to the lake's bed like a river's channel: the
 *       generated ground that stands in the water's range or over its top is removed (the
 *       density's ground between noise-cell corners can stand a block or two above the rule's), the
 *       air is filled with water from the top down; a cave under the lake bed is plugged, and grass
 *       left under the water becomes dirt.
 *       Where the climate freezes the lake ({@link LakeIce}: the mean annual temperature at the
 *       column is below the preset's {@code hydro.lake_freeze_mean_c}) its top water block is
 *       ice, whatever the biome; {@link LakeFreeze} keeps the other lake columns open.</li>
 *   <li><b>Sea</b> columns the aquifer left dry (it decides from its own noise near the surface, so
 *       a coastal flat a block below sea level can stay land) are filled to the sea's top, so the
 *       coast is where the terrain puts it and rivers reaching it meet water. The pass therefore
 *       also runs in every chunk with elevation cells within 17 blocks of sea level.</li>
 *   <li><b>Bank</b> columns are raised to their {@code minGround} (the neighbouring water's top,
 *       +1 beside a river), lowered to their {@code maxGround} (a 1:1 slope from the channel), and
 *       air between the neighbours' beds and {@code minGround} is sealed, so no water finds a way
 *       out. A bank keeps its top block (grass on a raised or lowered bank).</li>
 *   <li><b>Steps.</b> Where a water column's neighbour holds water higher up (a river's one-block
 *       step, a waterfall, a river entering a lake), the block the higher water would spread into
 *       gets the flowing water vanilla would put there (falling water down a waterfall), so the
 *       generated water is already at rest.</li>
 * </ul>
 *
 * <p>Everything is decided from (x, z) and the world data alone; only the column's own blocks
 * are read. The pass schedules no fluid tick and marks no position for post-processing; only
 * {@link #wake}, after the carvers, marks the few blocks a cave left hanging.
 *
 * <p><b>Carvers</b> run after this pass and may cut under the water: a cave can open an entrance
 * in a river or lake bed (Ben, decisions log 2026-09-30, "Allow caves under water"). They never
 * replace the water itself (Ben, 2026-10-01, "Protect water blocks"). Vanilla's
 * {@code #minecraft:overworld_carver_replaceables} lists {@code minecraft:water}, so left alone the
 * carvers cut air pockets through lakes and rivers and now and then a hole in a lake's surface.
 * Vanilla's carvers skip every position their chunk's carving mask already holds
 * ({@code WorldCarver.carveEllipsoid} tests the mask before {@code carveBlock}), so this pass marks
 * each water block it places or finds in a river or lake column, and only those: the ground under
 * and beside the water stays carvable ({@link #shield}). Where a cave then opens under the lowest
 * water block of a column, that block would hang over air, because nothing ticks a block placed at
 * generation; {@link #wake} marks it for post-processing, so vanilla's own fluid tick lets the
 * water run into the cave when the chunk is promoted. {@code hydro.protect_water_from_carvers}
 * (on by default, {@link #protectsWater}) switches both off.
 */
public final class WaterBodies {

    private WaterBodies() {}

    /** What one chunk's pass did. */
    public record Stats(int water, int cut, int filled, int flowing, int shielded) {
        public static final Stats NONE = new Stats(0, 0, 0, 0, 0);
    }

    private static final BlockState AIR = Blocks.AIR.defaultBlockState();
    private static final BlockState WATER = Blocks.WATER.defaultBlockState();
    private static final BlockState DIRT = Blocks.DIRT.defaultBlockState();
    private static final BlockState ICE = Blocks.ICE.defaultBlockState();
    private static final BlockState FLOWING = Fluids.FLOWING_WATER.getFlowing(7, false).createLegacyBlock();
    private static final BlockState FALLING = Fluids.FLOWING_WATER.getFlowing(8, true).createLegacyBlock();

    /** Air or water: what water may fill. */
    static boolean open(BlockState s) {
        return s.isAir() || s.is(Blocks.WATER);
    }

    /** A top block worth keeping on a bank that moves: soil, sand, gravel, snow. */
    static boolean skin(BlockState s) {
        return s.getBlock() instanceof SpreadingSnowyDirtBlock || s.is(BlockTags.DIRT) || s.is(BlockTags.SAND)
                || s.is(Blocks.GRAVEL) || s.is(Blocks.SNOW_BLOCK) || s.is(Blocks.CLAY);
    }

    /** Whether any cell of {@code layer} in (or next to) block box [x0, x1] × [z0, z1] is below {@code level}. */
    static boolean anyBelow(com.minecraftindustrialcomplex.deeptime.worlddata.RasterSampler layer, int x0, int z0, int x1, int z1, double level) {
        var g = layer.desc().grid();
        int r = g.r();
        long i0 = Math.floorDiv(x0 - g.originBlockX(), r), i1 = Math.floorDiv(x1 - g.originBlockX(), r) + 1;
        long j0 = Math.floorDiv(z0 - g.originBlockZ(), r), j1 = Math.floorDiv(z1 - g.originBlockZ(), r) + 1;
        for (long i = i0; i <= i1; i++)
            for (long j = j0; j <= j1; j++)
                if (layer.get(i, j) < level)
                    return true;
        return false;
    }

    /**
     * Whether the world asks carvers to spare the water placed here: {@code
     * notes.bake.water.params.protect_water_from_carvers} (preset key {@code
     * hydro.protect_water_from_carvers}); a world baked before the setting existed protects it too.
     */
    public static boolean protectsWater(WorldData wd) {
        return Water.protectsFromCarvers(wd.manifest().notes());
    }

    /**
     * The chunk's carving masks (the air step's and the liquid step's), or none when the chunk is
     * not a proto-chunk or the world does not protect its water.
     */
    private static CarvingMask[] carvingMasks(ChunkAccess chunk, WorldData wd) {
        if (chunk instanceof ProtoChunk proto && protectsWater(wd))
            return new CarvingMask[]{proto.getOrCreateCarvingMask(GenerationStep.Carving.AIR),
                    proto.getOrCreateCarvingMask(GenerationStep.Carving.LIQUID)};
        return new CarvingMask[0];
    }

    /**
     * Marks the water blocks of local column (lx, lz) from {@code lo} to {@code hi} (clipped to the
     * build range) in the carving masks, so no carver replaces them; returns how many.
     */
    static int shield(CarvingMask[] masks, int lx, int lz, int lo, int hi, int minY, int maxY) {
        int from = Math.max(lo, minY), to = Math.min(hi, maxY);
        if (masks.length == 0 || to < from)
            return 0;
        for (CarvingMask m : masks)
            for (int y = from; y <= to; y++)
                m.set(lx, y, lz);
        return to - from + 1;
    }

    /** Whether any river segment or wet {@code water_level} cell reaches the chunk (the sea is not counted). */
    private static boolean inlandWater(ChunkAccess chunk, WorldData wd, Water.Spec spec) {
        int x0 = chunk.getPos().getMinBlockX(), z0 = chunk.getPos().getMinBlockZ();
        Rivers rivers = wd.rivers();
        if (rivers != null && rivers.entriesIn(x0 - 1, z0 - 1, x0 + 16, z0 + 16).length > 0)
            return true;
        int reach = spec.reach();
        return wd.has(WorldData.WATER_LEVEL)
                && WaterLevels.anyWet(wd.layer(WorldData.WATER_LEVEL), x0 - 1 - reach, z0 - 1 - reach, x0 + 16 + reach, z0 + 16 + reach);
    }

    /**
     * After the air carvers: marks for post-processing the lowest water block of every column whose
     * block below it is now air, a cave the carvers opened into a river or lake bed. Nothing ticks a
     * block placed at generation, so that water would hang over the cave until a neighbour changed;
     * marked, it gets vanilla's fluid tick when the chunk is promoted, and runs down into the cave
     * as a falling stream (the lake's own sources never drain). Does nothing where the world does not
     * protect its water, in a chunk no river or lake reaches, or in a chunk that is not a proto-chunk.
     * Returns the number of columns woken.
     */
    public static int wake(ChunkAccess chunk, WorldData wd, int cell) {
        if (!(chunk instanceof ProtoChunk) || !protectsWater(wd) || !inlandWater(chunk, wd, Water.Spec.of(wd, cell)))
            return 0;
        int minY = chunk.getMinBuildHeight(), maxY = chunk.getMaxBuildHeight() - 1;
        int x0 = chunk.getPos().getMinBlockX(), z0 = chunk.getPos().getMinBlockZ();
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        int woken = 0;
        for (int lz = 0; lz < 16; lz++) {
            for (int lx = 0; lx < 16; lx++) {
                int x = x0 + lx, z = z0 + lz;
                int y = Math.min(maxY, chunk.getHeight(Heightmap.Types.WORLD_SURFACE_WG, lx, lz));   // the highest block that is not air
                if (y < minY || !chunk.getBlockState(pos.set(x, y, z)).is(Blocks.WATER))
                    continue;
                int lowest = y;
                for (y--; y >= minY && chunk.getBlockState(pos.set(x, y, z)).is(Blocks.WATER); y--)
                    lowest = y;
                if (y >= minY && chunk.getBlockState(pos.set(x, y, z)).isAir()) {
                    chunk.markPosForPostprocessing(pos.set(x, lowest, z));
                    woken++;
                }
            }
        }
        WOKEN.add(woken);
        return woken;
    }

    /** Chunks seen, chunks with work, and nanoseconds spent (logged every {@link #LOG_EVERY} chunks). */
    private static final java.util.concurrent.atomic.LongAdder CHUNKS = new java.util.concurrent.atomic.LongAdder(),
            WORKED = new java.util.concurrent.atomic.LongAdder(), NANOS = new java.util.concurrent.atomic.LongAdder(),
            WORKED_NANOS = new java.util.concurrent.atomic.LongAdder(), SHIELDED = new java.util.concurrent.atomic.LongAdder(),
            WOKEN = new java.util.concurrent.atomic.LongAdder();
    static final long LOG_EVERY = 512;

    /** Applies the pass to {@code chunk} with the generator's noise cell width. */
    public static Stats apply(ChunkAccess chunk, WorldData wd, int cell) {
        long t0 = System.nanoTime();
        Stats s = applyTimed(chunk, wd, cell);
        long dt = System.nanoTime() - t0;
        NANOS.add(dt);
        if (s != Stats.NONE) {
            WORKED.increment();
            WORKED_NANOS.add(dt);
            SHIELDED.add(s.shielded());
        }
        CHUNKS.increment();
        if (CHUNKS.sum() % LOG_EVERY == 0) {
            long n = CHUNKS.sum(), w = WORKED.sum();
            com.minecraftindustrialcomplex.deeptime.DeepTime.LOGGER.info(
                    "[deeptime] water pass: {} chunks, {} with water or coast, {} µs a chunk on average ({} µs where it worked); "
                            + "{} water blocks kept from the carvers, {} columns woken over carved beds",
                    n, w, String.format("%.1f", NANOS.sum() / 1000.0 / n), String.format("%.1f", w == 0 ? 0.0 : WORKED_NANOS.sum() / 1000.0 / w),
                    SHIELDED.sum(), WOKEN.sum());
        }
        return s;
    }

    private static Stats applyTimed(ChunkAccess chunk, WorldData wd, int cell) {
        Water.Spec spec = Water.Spec.of(wd, cell);
        int x0 = chunk.getPos().getMinBlockX(), z0 = chunk.getPos().getMinBlockZ();
        boolean inland = inlandWater(chunk, wd, spec);
        // A column is sea when H = E + A·N < sea level − 1; A is at most 255/16 blocks and |N| ≤ 1,
        // so no sea column lies where every elevation cell around is 17 blocks above sea level.
        boolean nearSea = wd.has(WorldData.ELEVATION)
                && anyBelow(wd.layer(WorldData.ELEVATION), x0 - 2, z0 - 2, x0 + 17, z0 + 17, spec.seaLevel() + 17);
        if (!inland && !nearSea)
            return Stats.NONE;
        Water.Region reg = Water.region(wd, spec, x0 - 1, z0 - 1, 18, 18);
        int minY = chunk.getMinBuildHeight(), maxY = chunk.getMaxBuildHeight() - 1;
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        int water = 0, cut = 0, filled = 0, flowing = 0, shielded = 0;
        CarvingMask[] masks = carvingMasks(chunk, wd);
        LakeIce lakeIce = LakeIce.of(wd);
        Water.Column[] cols = new Water.Column[256];
        for (int lz = 0; lz < 16; lz++) {
            for (int lx = 0; lx < 16; lx++) {
                Water.Column col = reg.column(lx + 1, lz + 1);
                cols[lz * 16 + lx] = col;
                if (col.kind() == Water.Kind.NONE)
                    continue;
                int x = x0 + lx, z = z0 + lz;
                if (col.kind() == Water.Kind.SEA) {
                    // The aquifer's sea, where it left the column dry: a flat a little below sea level
                    // (the aquifer decides from its own noise near the surface, not from the sea's
                    // extent). Filled to the sea's top so the coast is where the terrain says, and
                    // rivers and lakes that reach it meet water.
                    int t = Math.min(col.top(), maxY);
                    if (t >= minY && chunk.getBlockState(pos.set(x, t, z)).isAir()) {
                        int y = t;
                        for (; y > col.bed() && y >= minY && chunk.getBlockState(pos.set(x, y, z)).isAir(); y--) {
                            chunk.setBlockState(pos, WATER, false);
                            water++;
                        }
                        if (y == col.bed() && y >= minY && chunk.getBlockState(pos.set(x, y, z)).isAir()) {
                            chunk.setBlockState(pos, DIRT, false);
                            filled++;
                        }
                        shielded += shield(masks, lx, lz, y + 1, t, minY, maxY);
                        if (y < t && y >= minY && chunk.getBlockState(pos.set(x, y, z)).getBlock() instanceof SpreadingSnowyDirtBlock)
                            chunk.setBlockState(pos, DIRT, false);
                    }
                    continue;
                }
                int start = Math.min(maxY, chunk.getHeight(Heightmap.Types.WORLD_SURFACE_WG, lx, lz) + 1);
                int g = start;
                while (g >= minY && open(chunk.getBlockState(pos.set(x, g, z))))
                    g--;
                switch (col.kind()) {
                    case RIVER -> {
                        int t = Math.min(col.top(), maxY), bed = col.bed();
                        for (int y = Math.max(g, t); y > t; y--)
                            if (!chunk.getBlockState(pos.set(x, y, z)).isAir()) {
                                chunk.setBlockState(pos, AIR, false);
                                cut++;
                            }
                        for (int y = t; y > bed && y >= minY; y--) {
                            BlockState s = chunk.getBlockState(pos.set(x, y, z));
                            if (!s.is(Blocks.WATER) || !s.getFluidState().isSource()) {
                                if (!open(s))
                                    cut++;
                                chunk.setBlockState(pos, WATER, false);
                                water++;
                            }
                        }
                        shielded += shield(masks, lx, lz, bed + 1, t, minY, maxY);
                        for (int y = g + 1; y <= bed && y <= maxY; y++) {
                            chunk.setBlockState(pos.set(x, y, z), DIRT, false);
                            filled++;
                        }
                        if (bed >= minY && bed <= maxY) {
                            // Support under the bed: plug a cave right under it.
                            int y = bed - 1;
                            for (; y >= Math.max(minY, bed - 8) && chunk.getBlockState(pos.set(x, y, z)).isAir(); y--) {
                                chunk.setBlockState(pos, DIRT, false);
                                filled++;
                            }
                            boolean supported = y < minY || !chunk.getBlockState(pos.set(x, y, z)).isAir();
                            Rivers.Hit h = col.river();
                            BlockState bedBlock = !supported ? DIRT
                                    : h != null && h.hw() >= 3.0 ? Blocks.SAND.defaultBlockState() : Blocks.GRAVEL.defaultBlockState();
                            chunk.setBlockState(pos.set(x, bed, z), bedBlock, false);
                        }
                    }
                    case LAKE -> {
                        int t = Math.min(col.top(), maxY), bed = col.bed();
                        // The generated ground between noise-cell corners can stand a block or two
                        // above the rule's `ground` (the density's scale differs from corner to
                        // corner, TerrainScale): in a lake a block deep it reached the water's top
                        // and the column came out dry, flush with the lake (22 % of one test
                        // planet's lake columns, 2026-10-03). A lake column is cut to its bed, as a
                        // river's channel is: the ground over the top goes, the ground in the
                        // water's range becomes water.
                        for (int y = Math.max(g, t); y > t; y--)
                            if (!chunk.getBlockState(pos.set(x, y, z)).isAir()) {
                                chunk.setBlockState(pos, AIR, false);
                                cut++;
                            }
                        int y = t;
                        for (; y >= minY && y > bed; y--) {
                            BlockState s = chunk.getBlockState(pos.set(x, y, z));
                            if (!open(s))
                                cut++;
                            if (!s.is(Blocks.WATER)) {
                                chunk.setBlockState(pos, WATER, false);
                                water++;
                            }
                        }
                        // The loop ended on the lake's bed (the water is above it).
                        shielded += shield(masks, lx, lz, y + 1, t, minY, maxY);
                        // A frozen lake: ice on its water, whatever the biome.
                        if (lakeIce != null && y < t && t >= minY && lakeIce.frozen(x, z)
                                && chunk.getBlockState(pos.set(x, t, z)).is(Blocks.WATER))
                            chunk.setBlockState(pos, ICE, false);
                        if (y == bed && bed >= minY) {
                            // The analytic bed: open ground under it is a cave; plug a few blocks.
                            for (int p = bed; p >= Math.max(minY, bed - 3) && open(chunk.getBlockState(pos.set(x, p, z))); p--) {
                                chunk.setBlockState(pos, DIRT, false);
                                filled++;
                            }
                        }
                        if (y < t && y >= minY && chunk.getBlockState(pos.set(x, y, z)).getBlock() instanceof SpreadingSnowyDirtBlock)
                            chunk.setBlockState(pos, DIRT, false);
                    }
                    case BANK -> {
                        BlockState top = g >= minY ? chunk.getBlockState(pos.set(x, g, z)) : DIRT;
                        Integer maxG = col.maxGround(), minG = col.minGround();
                        if (maxG != null && g > maxG && maxG >= minY) {
                            for (int y = g; y > maxG; y--) {
                                chunk.setBlockState(pos.set(x, y, z), AIR, false);
                                cut++;
                            }
                            if (skin(top))
                                chunk.setBlockState(pos.set(x, maxG, z), top, false);
                            g = maxG;
                        }
                        if (minG != null && g < minG && minG <= maxY) {
                            if (g >= minY && chunk.getBlockState(pos.set(x, g, z)).getBlock() instanceof SpreadingSnowyDirtBlock)
                                chunk.setBlockState(pos, DIRT, false);
                            for (int y = Math.max(g + 1, minY); y < minG; y++) {
                                chunk.setBlockState(pos.set(x, y, z), DIRT, false);
                                filled++;
                            }
                            chunk.setBlockState(pos.set(x, minG, z), skin(top) ? top : DIRT, false);
                            filled++;
                            g = minG;
                        }
                        if (minG != null && col.sealFrom() != null) {
                            for (int y = Math.max(minY, col.sealFrom()); y <= Math.min(minG, maxY); y++)
                                if (chunk.getBlockState(pos.set(x, y, z)).isAir()) {
                                    chunk.setBlockState(pos, DIRT, false);
                                    filled++;
                                }
                        }
                    }
                    default -> {}
                }
            }
        }
        // Steps: the flowing water a higher neighbour would spread into this column.
        for (int lz = 0; lz < 16; lz++) {
            for (int lx = 0; lx < 16; lx++) {
                Water.Column col = cols[lz * 16 + lx];
                if (!col.kind().isWater())
                    continue;
                int tb = col.top();
                int m = Integer.MIN_VALUE;
                for (int[] d : Water.NEIGHBOURS) {
                    Water.Core n = reg.at(lx + 1 + d[0], lz + 1 + d[1]);
                    if (n.kind().isWater())
                        m = Math.max(m, n.top());
                }
                if (m <= tb || m > maxY)
                    continue;
                int x = x0 + lx, z = z0 + lz;
                // Flowing, never a source: where two higher neighbours meet a one-block step vanilla
                // would turn this block into a source on its next update, and a source here would
                // spread over a frozen pool or a boulder a feature drops in the channel; flowing
                // water resting on water stays put (vanilla's "water hole").
                if (chunk.getBlockState(pos.set(x, m, z)).isAir()) {
                    chunk.setBlockState(pos, FLOWING, false);
                    flowing++;
                    shielded += shield(masks, lx, lz, m, m, minY, maxY);
                }
                for (int y = Math.max(tb + 1, minY); y < m; y++)
                    if (chunk.getBlockState(pos.set(x, y, z)).isAir()) {
                        chunk.setBlockState(pos, FALLING, false);
                        flowing++;
                        shielded += shield(masks, lx, lz, y, y, minY, maxY);
                    }
            }
        }
        return new Stats(water, cut, filled, flowing, shielded);
    }
}
