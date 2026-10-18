// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.worlddata.Body;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.LevelChunkSection;

import java.util.List;

/**
 * The outcrop of an exposed gossan (decisions log 2026-10-17: "the soil/surface rule leaves an exposed
 * gossan as the surface block"; ledger DEP-211).
 *
 * <p>A porphyry that reaches the surface weathers to a leached iron cap, the gossan (SIR 2010-5070-B
 * pp. 65 to 67, 120; Cook and Porter 2005), and a cap that crops out is bare iron-stained rock, not
 * grass. The pack's surface rules run first and paint soil over the default block;
 * {@link StrataPass#paintBodies} then replaces only stone and deepslate, so a gossan was dressed in
 * grass and dirt and no ironstone reached the ground. Here, for every column of a gossan body where
 * the ground is no more than {@link #EXPOSURE_SLACK} blocks above the body's top, every solid block
 * from the ground down to the body's bottom becomes the gossan's rock, the soil included.
 *
 * <p>The slack is the mismatch between the surface the body was placed on (the simulation's
 * heightmap) and the terrain the generator built (the same heightmap plus the amplifier's relief,
 * a block or three). A gossan buried by younger cover (a palaeo-gossan, placed at the surface that
 * cut it) sits further under the ground and is left alone, as is a column under water.
 */
final class GossanSkin {

    /** The {@link Body#role()} of an iron cap. */
    static final String ROLE = "gossan";

    /**
     * How far (blocks) the ground may stand above a gossan's top and still be its outcrop.
     * game-choice (ledger DEP-211): the exit run measured the simulation-to-terrain gap at 1 to 3
     * blocks over its 24 porphyries; one more block of margin.
     */
    static final int EXPOSURE_SLACK = 4;

    private GossanSkin() {}

    /** The blocks of one column to repaint: {@code from} (the ground) down to {@code to} (the body's bottom). */
    record Span(int from, int to) {}

    /**
     * The span to repaint in a column whose ground block is {@code ground}, or null when the gossan
     * is buried (the ground stands more than {@link #EXPOSURE_SLACK} above its top) or the ground is
     * below its whole bottom (nothing of the body is left in the column).
     */
    static Span span(int ground, int bodyTop, int bodyBottom) {
        if (ground < bodyBottom || ground - bodyTop > EXPOSURE_SLACK)
            return null;
        return new Span(ground, bodyBottom);
    }

    /** The highest and lowest y in [yLo, yHi] inside the body at column (x, z), or null when the column misses it. */
    static int[] extent(Body b, int c, int x, int z, int yLo, int yHi) {
        int top = Integer.MIN_VALUE, bottom = 0;
        for (int y = yHi; y >= yLo; y--) {
            if (b.locate(c, x, y, z) == null)
                continue;
            if (top == Integer.MIN_VALUE)
                top = y;
            bottom = y;
        }
        return top == Integer.MIN_VALUE ? null : new int[]{top, bottom};
    }

    /**
     * Repaints the exposed gossans of the chunk (run after {@link StrataPass}'s horizons and body
     * rock, whose key decides the member). Returns the number of columns skinned.
     */
    static int apply(ChunkAccess chunk, int c, List<Body> bodies, RockPalette palette) {
        int minY = chunk.getMinBuildHeight(), maxY = chunk.getMaxBuildHeight() - 1;
        int x0 = chunk.getPos().getMinBlockX(), z0 = chunk.getPos().getMinBlockZ();
        int total = 0;
        for (Body b : bodies) {
            if (!ROLE.equals(b.role()) || b.rockClass() == null)
                continue;
            BlockState rock = palette.block(b.rockClass(), StrataPass.bodyKey(b.id()));
            if (rock == null)
                continue;
            int yLo = Math.max(minY, b.min()[1]), yHi = Math.min(maxY, b.max()[1]);
            int skinned = 0, buried = 0, wet = 0, minGap = Integer.MAX_VALUE, maxGap = Integer.MIN_VALUE;
            for (int lx = 0; lx < 16; lx++) {
                for (int lz = 0; lz < 16; lz++) {
                    int x = x0 + lx, z = z0 + lz;
                    if (!b.aabbContains(c, x, yLo, z) && !b.aabbContains(c, x, yHi, z))
                        continue;
                    int[] e = extent(b, c, x, z, yLo, yHi);
                    if (e == null)
                        continue;
                    int ground = IcePass.surfaceY(chunk, x, z, minY);
                    if (ground < minY) {
                        wet++;   // a fluid or empty column: the sea or a lake covers it
                        continue;
                    }
                    int gap = ground - e[0];
                    minGap = Math.min(minGap, gap);
                    maxGap = Math.max(maxGap, gap);
                    Span s = span(ground, e[0], e[1]);
                    if (s == null) {
                        buried++;
                        continue;
                    }
                    skinned++;
                    for (int y = s.from(); y >= s.to(); y--) {
                        LevelChunkSection section = chunk.getSection(chunk.getSectionIndex(y));
                        BlockState here = section.getBlockState(lx, y & 15, lz);
                        if (here != rock && !here.isAir() && here.getFluidState().isEmpty() && !here.is(Blocks.BEDROCK))
                            section.setBlockState(lx, y & 15, lz, rock, false);
                    }
                }
            }
            total += skinned;
            if (skinned + buried + wet > 0)
                DeepTime.LOGGER.info("gossan skin: body {} in chunk ({}, {}): {} column(s) skinned, {} buried (ground more than {} above its top), "
                                + "{} under water; ground minus gossan top {}..{}", b.id(), chunk.getPos().x, chunk.getPos().z, skinned,
                        buried, EXPOSURE_SLACK, wet, minGap == Integer.MAX_VALUE ? "-" : minGap, maxGap == Integer.MIN_VALUE ? "-" : maxGap);
        }
        return total;
    }
}
