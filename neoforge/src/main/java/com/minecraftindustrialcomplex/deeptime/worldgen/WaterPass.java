// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.worlddata.RasterSampler;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import net.minecraft.core.BlockPos;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.SpreadingSnowyDirtBlock;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;

/**
 * Water above sea level (plan §7.9). Vanilla places fluid only through the aquifer, so a lake whose
 * surface is above sea level needs its own pass. Runs after {@link StrataPass}, before carvers and
 * features: in every column whose {@code water_level} cell (nearest, nodata = no water) lies above
 * the ground, the air from just below the water surface down to the first non-air block becomes
 * water source blocks.
 *
 * <p>Conventions, as for vanilla's sea level: a water surface at {@code L} fills every block with
 * {@code y < L}, so the highest water block is {@code ceil(L) - 1} and its top face is at {@code L}
 * (sea level 63 → water up to y 62).
 *
 * <p>No fluid tick is scheduled and no position is marked for post-processing, so a basin stays
 * still until a neighbour changes; the bake keeps {@code water_level} below its basin's lowest rim
 * (Fill–Spill–Merge, Phase 2), so a woken basin has nowhere to flow either. Ice is not placed here:
 * vanilla's {@code freeze_top_layer} feature, in every overworld biome, freezes the top water block
 * where the biome is cold, and our water exists before features run. A grass-like block
 * ({@link SpreadingSnowyDirtBlock}) left under the water by the surface rules, which ran before the
 * water existed, becomes dirt, as vanilla's rules make it under water; otherwise its random tick
 * would do so later and wake the water above it.
 */
public final class WaterPass {

    private WaterPass() {}

    /** Fills the chunk's lakes; returns the number of water blocks placed. */
    public static int apply(ChunkAccess chunk, WorldData wd) {
        if (!wd.has(WorldData.WATER_LEVEL))
            return 0;
        RasterSampler levels = wd.layer(WorldData.WATER_LEVEL);
        int minY = chunk.getMinBuildHeight();
        int maxY = chunk.getMaxBuildHeight() - 1;
        int x0 = chunk.getPos().getMinBlockX(), z0 = chunk.getPos().getMinBlockZ();
        BlockState water = Blocks.WATER.defaultBlockState();
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        int placed = 0;
        for (int lx = 0; lx < 16; lx++) {
            for (int lz = 0; lz < 16; lz++) {
                int x = x0 + lx, z = z0 + lz;
                double level = levels.nearest(x, z);   // x wraps in the sampler
                if (Double.isNaN(level))
                    continue;
                int top = Math.min(topWaterBlock(level), maxY);
                int y = top;
                // ProtoChunk.setBlockState keeps the worldgen heightmaps current; it schedules no
                // fluid tick and marks nothing for post-processing.
                for (; y >= minY; y--) {
                    if (!chunk.getBlockState(pos.set(x, y, z)).isAir())
                        break;
                    chunk.setBlockState(pos, water, false);
                    placed++;
                }
                if (y < top && y >= minY && chunk.getBlockState(pos.set(x, y, z)).getBlock() instanceof SpreadingSnowyDirtBlock)
                    chunk.setBlockState(pos, Blocks.DIRT.defaultBlockState(), false);
            }
        }
        return placed;
    }

    /** The highest water block under a water surface at {@code level}: {@code ceil(level) - 1}. */
    public static int topWaterBlock(double level) {
        return (int) Math.ceil(level) - 1;
    }
}
