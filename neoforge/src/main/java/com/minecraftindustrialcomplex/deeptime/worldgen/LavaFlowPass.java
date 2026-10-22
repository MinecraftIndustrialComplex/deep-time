// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import net.minecraft.core.BlockPos;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;

/**
 * Lava flows on the ground of the world's active volcanoes ({@link LavaFlows}; review fix 8, 2026-10-22):
 * in every column a flow shows in, the ground's top block becomes the flow's ({@link LavaFlowPalette}), so
 * the terrain's height does not change. The pack's own lava stripes in its volcanic biomes (Terralith's
 * {@code volcano_surface} contours) are left out of Deep Time levels ({@link DelegatingRuleSource}), so the
 * dark lines on a cone are these flows.
 *
 * <p>Runs in {@link GeologicChunkGenerator#buildSurface} after {@link PlayaPass} and before the magma, so a
 * crater's lava, a geothermal site's ground and the ice cover lie over a flow. Columns whose surface is
 * fluid, and air, fluid and bedrock tops, are left as they are. Far terrain paints the same columns
 * ({@code farterrain.FarTerrain}).
 */
public final class LavaFlowPass {

    private LavaFlowPass() {}

    /** Paints the chunk's flows; returns the number of blocks replaced. */
    public static int apply(ChunkAccess chunk, LavaFlows flows) {
        int x0 = chunk.getPos().getMinBlockX(), z0 = chunk.getPos().getMinBlockZ();
        if (flows == null || flows.size() == 0 || !flows.touches(x0, z0, x0 + 15, z0 + 15))
            return 0;
        LavaFlowPalette palette = LavaFlowPalette.get();
        int minY = chunk.getMinBuildHeight();
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        int placed = 0;
        for (int lx = 0; lx < 16; lx++) {
            for (int lz = 0; lz < 16; lz++) {
                int x = x0 + lx, z = z0 + lz;
                int code = flows.code(x, z);
                if (code <= 0)
                    continue;
                int top = IcePass.surfaceY(chunk, x, z, minY);
                if (top < minY)
                    continue;   // a fluid or empty column
                BlockState here = chunk.getBlockState(pos.set(x, top, z));
                if (here.isAir() || !here.getFluidState().isEmpty() || here.is(Blocks.BEDROCK))
                    continue;
                BlockState block = palette.block(code, LavaFlowPalette.key(flows.wrap(x), z));
                if (block != null && here != block) {
                    chunk.setBlockState(pos, block, false);
                    placed++;
                }
            }
        }
        return placed;
    }
}
