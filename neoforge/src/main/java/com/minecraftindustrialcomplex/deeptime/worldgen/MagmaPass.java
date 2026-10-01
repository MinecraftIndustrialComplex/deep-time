// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import net.minecraft.core.BlockPos;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.levelgen.Heightmap;

import java.util.List;
import java.util.concurrent.atomic.LongAdder;

/**
 * Places the magma of the world's active volcanoes in a chunk ({@link Magma} decides what every
 * block is; Ben, decisions log 2026-10-01, "Volcanoes is a go ahead"). Runs in the surface step,
 * after the strata and the water pass and before the ice, carvers and features.
 *
 * <ul>
 *   <li>Chamber and conduit blocks become lava, whatever they were, but never within
 *       {@link Magma#COVER} blocks of the column's ground (the sea bed under the sea), so a
 *       valley, a river's cut or a trench over a chamber never opens it.</li>
 *   <li>Shell blocks that are open (air, water) become basalt: magma is always walled in.</li>
 *   <li>The plug's solid blocks become magma blocks.</li>
 *   <li>A crater lake's blocks become lava and the ground above its surface air.</li>
 * </ul>
 *
 * <p>Everything is decided from the block's position and the level's {@link Magma}; only the
 * chunk's own blocks are read and written, so chunks generate in any order on any thread. No fluid
 * tick is scheduled: the lava rests until a player opens it.
 */
public final class MagmaPass {

    private MagmaPass() {}

    private static final BlockState LAVA = Blocks.LAVA.defaultBlockState();
    private static final BlockState BASALT = Blocks.SMOOTH_BASALT.defaultBlockState();
    private static final BlockState MAGMA = Blocks.MAGMA_BLOCK.defaultBlockState();
    private static final BlockState AIR = Blocks.AIR.defaultBlockState();

    private static final LongAdder LAVA_PLACED = new LongAdder(), CHUNKS = new LongAdder();

    /** Lava blocks placed and chunks touched since the JVM started (for logs and tests). */
    public static long[] totals() {
        return new long[]{LAVA_PLACED.sum(), CHUNKS.sum()};
    }

    static boolean open(BlockState s) {
        return s.isAir() || !s.getFluidState().isEmpty();
    }

    /** Applies the pass; returns the number of lava blocks placed. */
    public static int apply(ChunkAccess chunk, Magma magma) {
        int x0 = chunk.getPos().getMinBlockX(), z0 = chunk.getPos().getMinBlockZ();
        List<Magma.Body> bodies = magma.near(x0, z0, x0 + 15, z0 + 15);
        if (bodies.isEmpty())
            return 0;
        int minY = chunk.getMinBuildHeight() + 1, maxY = chunk.getMaxBuildHeight() - 1;
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        int placed = 0;
        for (Magma.Body b : bodies) {
            int lo = Math.max(minY, b.minY()), hi = Math.min(maxY, b.maxY());
            int dx0 = magma.dx(x0, b);
            for (int lz = 0; lz < 16; lz++) {
                int dz = z0 + lz - b.z();
                if (Math.abs(dz) > b.reach())
                    continue;
                for (int lx = 0; lx < 16; lx++) {
                    int dx = dx0 + lx;
                    if (Math.abs(dx) > b.reach())
                        continue;
                    int x = x0 + lx, z = z0 + lz;
                    // The column's ground: the highest block that is neither air nor fluid.
                    int ground = chunk.getHeight(Heightmap.Types.OCEAN_FLOOR_WG, lx, lz);
                    for (int y = lo; y <= hi; y++) {
                        Magma.Part part = b.at(dx, dz, y);
                        if (part == Magma.Part.NONE)
                            continue;
                        BlockState here = chunk.getBlockState(pos.set(x, y, z));
                        switch (part) {
                            case CHAMBER -> {
                                if (y <= ground - Magma.COVER) {
                                    if (!here.is(Blocks.LAVA)) {
                                        chunk.setBlockState(pos, LAVA, false);
                                        placed++;
                                    }
                                } else if (open(here)) {
                                    // Where the ground dips into the chamber's place: rock, not magma.
                                    chunk.setBlockState(pos, BASALT, false);
                                }
                            }
                            case CONDUIT -> {
                                if (y <= ground - Magma.BURIED) {
                                    if (!here.is(Blocks.LAVA)) {
                                        chunk.setBlockState(pos, LAVA, false);
                                        placed++;
                                    }
                                } else if (y <= ground) {
                                    chunk.setBlockState(pos, MAGMA, false);   // a cave here too: the lava below stays capped
                                }
                            }
                            case PLUG -> {
                                if (y <= ground)
                                    chunk.setBlockState(pos, MAGMA, false);
                            }
                            case LAKE -> {
                                if (!here.is(Blocks.LAVA)) {
                                    chunk.setBlockState(pos, LAVA, false);
                                    placed++;
                                }
                            }
                            case LAKE_AIR -> {
                                if (!here.isAir())
                                    chunk.setBlockState(pos, AIR, false);
                            }
                            case CHAMBER_SHELL, CONDUIT_SHELL, LAKE_SHELL -> {
                                if (open(here) && !here.is(Blocks.LAVA))
                                    chunk.setBlockState(pos, BASALT, false);
                            }
                            default -> { }
                        }
                    }
                }
            }
        }
        LAVA_PLACED.add(placed);
        CHUNKS.increment();
        return placed;
    }
}
