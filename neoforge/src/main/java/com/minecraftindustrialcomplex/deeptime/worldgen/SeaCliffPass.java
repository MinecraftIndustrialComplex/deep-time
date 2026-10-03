// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import net.minecraft.core.BlockPos;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.CarvingMask;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.ProtoChunk;
import net.minecraft.world.level.levelgen.GenerationStep;

import java.util.concurrent.atomic.LongAdder;

/**
 * Applies {@link SeaCliffGuard}'s rule to a chunk (Minecraft types; the rule itself is plain Java
 * so JUnit can test it): the guarded blocks go in the carving masks and the open ones among them
 * are filled with rock. Called from the surface step, before the strata pass.
 */
public final class SeaCliffPass {

    private SeaCliffPass() {}

    /**
     * Guards the chunk: puts the guarded blocks in its carving masks and fills the open ones among
     * them with rock; returns how many blocks it filled. Call it before the strata pass.
     */
    public static int apply(ChunkAccess chunk, Magma.Heights heights, int seaLevel, SeaCliffGuard.Rules rules) {
        if (rules.skin() <= 0)
            return 0;
        int x0 = chunk.getPos().getMinBlockX(), z0 = chunk.getPos().getMinBlockZ();
        int minY = chunk.getMinBuildHeight() + 1;   // never the bedrock layer
        int[][] zone = SeaCliffGuard.zone(heights, x0, z0, seaLevel, minY, rules.skin());
        CarvingMask[] masks = chunk instanceof ProtoChunk proto
                ? new CarvingMask[]{proto.getOrCreateCarvingMask(GenerationStep.Carving.AIR),
                        proto.getOrCreateCarvingMask(GenerationStep.Carving.LIQUID)}
                : new CarvingMask[0];
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        int filled = 0;
        for (int lz = 0; lz < 16; lz++)
            for (int lx = 0; lx < 16; lx++) {
                int own = Magma.top(heights.at(x0 + lx, z0 + lz));
                int[] r = zone[lz * 16 + lx];
                if (r == null)
                    continue;
                for (int y = r[0]; y <= r[1]; y++) {
                    for (CarvingMask m : masks)
                        m.set(lx, y, lz);
                    if (y >= own)
                        continue;   // the ground's top block is the surface: leave it as the density drew it
                    BlockState s = chunk.getBlockState(pos.set(x0 + lx, y, z0 + lz));
                    boolean open = s.isAir() || (!s.getFluidState().isEmpty() && !s.is(Blocks.LAVA));
                    if (open) {
                        chunk.setBlockState(pos, y >= 0 ? STONE : DEEPSLATE, false);
                        filled++;
                    }
                }
            }
        FILLED.add(filled);
        return filled;
    }

    private static final BlockState STONE = Blocks.STONE.defaultBlockState(), DEEPSLATE = Blocks.DEEPSLATE.defaultBlockState();
    private static final LongAdder FILLED = new LongAdder();

    /** Blocks filled since the JVM started (for logs and tests). */
    public static long filledTotal() {
        return FILLED.sum();
    }
}
