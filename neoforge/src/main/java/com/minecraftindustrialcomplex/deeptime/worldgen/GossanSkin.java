// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.worlddata.Body;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.LevelChunkSection;

import java.util.List;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;

/**
 * The outcrop of an exposed gossan (decisions log 2026-10-17: "the soil/surface rule leaves an
 * exposed gossan as the surface block"; ledger DEP-211).
 *
 * <p>A porphyry that reaches the surface weathers to a leached iron cap, the gossan (SIR 2010-5070-B
 * pp. 65 to 67, 120; Cook and Porter 2005), and a cap that crops out is bare iron-stained rock, not
 * grass. The pack's surface rules run first and paint soil over the default block;
 * {@link StrataPass#paintBodies} then replaces only stone and deepslate, so a gossan was dressed in
 * grass and dirt and no ironstone reached the ground. Here the cap is painted onto the ground
 * itself: in every column of an exposed gossan's footprint, the top {@link #thickness} blocks of
 * solid ground (the soil included) become the gossan's rock.
 *
 * <p>The cap follows the ground, not the body's flat plane. The body is a flat blanket at the surface
 * the simulation had at its centre, while the terrain the generator builds is that surface plus the
 * detail relief (the exit run's gossan stood 13 blocks above the ground in one column and 13 below it
 * in the next): a cap cut to the flat plane would be buried on every rise and hang over every dip.
 *
 * <p>Whether a gossan is exposed is decided once per body, at its centre: the terrain the generator
 * builds there stands no more than {@link #EXPOSURE_SLACK} blocks above the body's top. A gossan that
 * a younger deposit buried again (the placement puts it under the surface that cut it) lies far under
 * the ground (the exit run measured 14 to 37 blocks) and is left alone, as is a column under water.
 */
final class GossanSkin {

    /** The {@link Body#role()} of an iron cap. */
    static final String ROLE = "gossan";

    /**
     * How far (blocks) the ground at a gossan's centre may stand above the gossan's top and the gossan
     * still be an outcrop. game-choice (ledger DEP-211): the best 24 porphyries of the exit run had
     * gaps of 1, 2 and 3 blocks between the gossan's top and the generated ground where the body is
     * exposed, and 14, 24 and 37 where a younger cover buries it; six is above the first and well
     * under the second.
     */
    static final double EXPOSURE_SLACK = 6.0;

    /** Bodies whose exposure was logged (once each: a body spans several chunks and every chunk decides alike). */
    private static final Set<Integer> LOGGED = ConcurrentHashMap.newKeySet();

    private GossanSkin() {}

    /** The blocks of one column to repaint: {@code from} (the ground) down to {@code to}. */
    record Span(int from, int to) {}

    /**
     * Whether the gossan crops out: the generated ground at its centre is no more than
     * {@link #EXPOSURE_SLACK} blocks above its top. Ground below the top counts (the body then
     * stands out of a lowered surface), a body without a clip does not (it was never at the surface).
     */
    static boolean exposed(double groundAtCentre, int clipTopY) {
        return clipTopY != Body.NO_CLIP && groundAtCentre - clipTopY <= EXPOSURE_SLACK;
    }

    /** How many blocks of ground the cap takes: the body's own thickness, two at least (DEP-210). */
    static int thickness(Body b) {
        return Math.max(2, (int) Math.round(2.0 * b.axes()[1]));
    }

    /** The span the cap takes in a column whose ground block is {@code ground}. */
    static Span cap(int ground, int thickness) {
        return new Span(ground, ground - thickness + 1);
    }

    /** Whether column (x, z) is under the body: its mid-plane holds the column. */
    static boolean inFootprint(Body b, int c, int x, int z) {
        int y = (int) Math.round(b.center()[1]);
        y = Math.max(b.min()[1], Math.min(b.max()[1], y));
        return b.locate(c, x, y, z) != null;
    }

    /** Whether some body of {@code placers} holds the block (x, y, z). */
    static boolean covered(List<Body> placers, int c, int x, int y, int z) {
        for (Body p : placers)
            if (p.locate(c, x, y, z) != null)
                return true;
        return false;
    }

    /**
     * Repaints the exposed gossans of the chunk (run after {@link StrataPass}'s horizons and body
     * rock, whose key decides the member). A placer's gravel keeps its place: a cell inside a body
     * of loose rock that carries ore is not repainted, since {@link DepositFeature} puts a placer's
     * gold only into loose blocks and the cap would leave it none (the exit run's placer did not
     * lie under a cap; this keeps one that does from losing its host). Returns the number of
     * columns capped.
     */
    static int apply(ChunkAccess chunk, WorldData wd, List<Body> bodies, RockPalette palette) {
        int c = wd.circumference();
        int minY = chunk.getMinBuildHeight(), maxY = chunk.getMaxBuildHeight() - 1;
        int x0 = chunk.getPos().getMinBlockX(), z0 = chunk.getPos().getMinBlockZ();
        int total = 0;
        List<Body> placers = bodies.stream()
                .filter(p -> !ROLE.equals(p.role()) && p.rockClass() != null && palette.isLoose(p.rockClass()) && DepositFeature.placesOre(p))
                .toList();
        for (Body b : bodies) {
            if (!ROLE.equals(b.role()) || b.rockClass() == null)
                continue;
            double ground = wd.height(b.center()[0], b.center()[2]);
            boolean open = exposed(ground, b.clipTopY());
            if (LOGGED.add(b.id()))
                DeepTime.LOGGER.info("gossan {} (site {}): generated ground at its centre {} vs its top {}: gap {} blocks, {}", b.id(), b.site(),
                        String.format("%.1f", ground), b.clipTopY(), String.format("%.1f", ground - b.clipTopY()),
                        open ? "exposed, capped with " + thickness(b) + " blocks of " + b.rockClass()
                                : "buried (more than " + EXPOSURE_SLACK + " blocks of cover), left alone");
            if (!open)
                continue;
            BlockState rock = palette.block(b.rockClass(), StrataPass.bodyKey(b.id()));
            if (rock == null)
                continue;
            int thick = thickness(b), capped = 0, wet = 0;
            for (int lx = 0; lx < 16; lx++) {
                for (int lz = 0; lz < 16; lz++) {
                    int x = x0 + lx, z = z0 + lz;
                    if (!b.aabbContains(c, x, (int) Math.round(b.center()[1]), z) || !inFootprint(b, c, x, z))
                        continue;
                    int top = IcePass.surfaceY(chunk, x, z, minY);
                    if (top < minY) {
                        wet++;   // a fluid or empty column: the sea or a lake covers it
                        continue;
                    }
                    Span s = cap(top, thick);
                    capped++;
                    for (int y = Math.min(s.from(), maxY); y >= Math.max(s.to(), minY); y--) {
                        if (!placers.isEmpty() && covered(placers, c, x, y, z))
                            continue;
                        LevelChunkSection section = chunk.getSection(chunk.getSectionIndex(y));
                        BlockState here = section.getBlockState(lx, y & 15, lz);
                        if (here != rock && !here.isAir() && here.getFluidState().isEmpty() && !here.is(Blocks.BEDROCK))
                            section.setBlockState(lx, y & 15, lz, rock, false);
                    }
                }
            }
            total += capped;
            if (capped + wet > 0)
                DeepTime.LOGGER.debug("gossan {} in chunk ({}, {}): {} column(s) capped, {} under water", b.id(), chunk.getPos().x, chunk.getPos().z, capped, wet);
        }
        return total;
    }
}
