// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.worlddata.Body;
import com.minecraftindustrialcomplex.deeptime.worlddata.Columns;
import com.minecraftindustrialcomplex.deeptime.worlddata.Water;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.LevelChunkSection;

import java.util.List;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicLong;
import java.util.function.IntBinaryOperator;
import java.util.function.Predicate;

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
 * <p>A leached cap holds no copper (ledger DEP-212): the leaching carried it down to the blanket at
 * the water table. {@link DepositFeature} therefore places none of the deposit's ore in the cap's
 * blocks, and the tests read the same rule ({@link Caps}). The rule cannot read the skin's blocks
 * back: when the feature runs, the carvers and earlier features have changed the column the skin
 * scanned. It is a pure window of the column instead, from the ground the terrain model gives
 * ({@link #groundAt}), and the skin keeps inside it ({@link #anchor}). The gossan's own gold zone
 * (the data file's "carries gold") is not the deposit's copper and stays.
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
public final class GossanSkin {

    /** The {@link Body#role()} of an iron cap. */
    public static final String ROLE = "gossan";

    /**
     * How far (blocks) the ground at a gossan's centre may stand above the gossan's top and the gossan
     * still be an outcrop. game-choice (ledger DEP-211): the best 24 porphyries of the exit run had
     * gaps of 1, 2 and 3 blocks between the gossan's top and the generated ground where the body is
     * exposed, and 14, 24 and 37 where a younger cover buries it; six is above the first and well
     * under the second.
     */
    static final double EXPOSURE_SLACK = 6.0;

    /**
     * How far (blocks) the ground the skin finds may stand above or below the ground the terrain model
     * gives ({@link #groundAt}) before the cap is held to that distance, and so how far above and
     * below the cap the ore-free window reaches. game-choice (ledger DEP-213): the generated surface
     * is the noise cells' interpolated density, which the model follows to the block on gentle
     * ground and misses by a few blocks on steep ground and at a cave mouth; the measured spread is
     * in the build log (2026-10-18, gossan cap task).
     */
    public static final int GROUND_MARGIN = 3;

    /** Bodies whose exposure was logged (once each: a body spans several chunks and every chunk decides alike). */
    private static final Set<Integer> LOGGED = ConcurrentHashMap.newKeySet();

    private static final AtomicLong CAPPED = new AtomicLong(), CLAMPED = new AtomicLong();

    /** Columns capped since the JVM started, and how many of them the margin held (a diagnostic for the tests). */
    public record Stats(long capped, long clamped) {}

    public static Stats stats() {
        return new Stats(CAPPED.get(), CLAMPED.get());
    }

    private GossanSkin() {}

    /** The blocks of one column to repaint: {@code from} (the ground) down to {@code to}. */
    public record Span(int from, int to) {}

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

    /** Whether the gossan's footprint holds column (x, z): inside its box at the mid-plane and under the blanket. */
    static boolean holdsColumn(Body b, int c, int x, int z) {
        return b.aabbContains(c, x, (int) Math.round(b.center()[1]), z) && inFootprint(b, c, x, z);
    }

    /** Whether some body of {@code placers} holds the block (x, y, z). */
    static boolean covered(List<Body> placers, int c, int x, int y, int z) {
        for (Body p : placers)
            if (p.locate(c, x, y, z) != null)
                return true;
        return false;
    }

    /** The bodies of loose rock that carry ore (placers): the skin leaves their cells alone. */
    static List<Body> placersOf(List<Body> bodies, Predicate<String> isLoose) {
        return bodies.stream()
                .filter(p -> !ROLE.equals(p.role()) && p.rockClass() != null && isLoose.test(p.rockClass()) && DepositFeature.placesOre(p))
                .toList();
    }

    /** Whether gossan {@code b} crops out in the generated terrain (decided at its centre). */
    static boolean cropsOut(WorldData wd, Body b) {
        return exposed(wd.height(b.center()[0], b.center()[2]), b.clipTopY());
    }

    /**
     * The ground block of column (x, z) as the terrain model gives it: the top block of the noise
     * cell's interpolated surface ({@link Water#cellSurface}, the surface the water pass reads as the
     * column's ground). A pure function of the world data; the generated top differs from it by a few
     * blocks where the density's scale varies across a cell (steep ground) or a cave opens the surface.
     */
    static int groundAt(WorldData wd, int x, int z) {
        int c = wd.circumference();
        long wx = Math.floorMod((long) x + c / 2, (long) c) - c / 2;
        return Water.topBlock(Water.cellSurface(wd, Water.Spec.of(wd, Columns.NOISE_CELL), wx, z));
    }

    /**
     * The block the cap hangs from in a column: the top the skin found, held within
     * {@link #GROUND_MARGIN} blocks of the model's {@code ground}. Every block the cap repaints then
     * lies in {@link #window}, whatever the terrain did.
     */
    static int anchor(int top, int ground) {
        return Math.max(ground - GROUND_MARGIN, Math.min(ground + GROUND_MARGIN, top));
    }

    /** Where in a column of model ground {@code ground} a cap {@code thickness} thick can lie: the cap's span moved {@link #GROUND_MARGIN} either way. */
    static Span window(int ground, int thickness) {
        return new Span(ground + GROUND_MARGIN, ground - thickness + 1 - GROUND_MARGIN);
    }

    /**
     * The caps over one chunk's bodies: the question the deposit feature and the tests ask of a
     * block, "does a leached cap hold this block?" (ledger DEP-212). Built from the bodies the strata
     * pass painted ({@link StrataPass#paintBodies} passes the same list to {@link #apply}). Whether a
     * body is held back by a cap is the caller's to ask: a gossan's own gold zone and a placer's gold
     * are not (a cell of a placer's gravel in the window is the placer's, the skin leaves it alone).
     */
    public static final class Caps {
        private final int c;
        private final List<Body> gossans;
        private final IntBinaryOperator ground;

        Caps(int c, List<Body> gossans, IntBinaryOperator ground) {
            this.c = c;
            this.gossans = gossans;
            this.ground = ground;
        }

        /** The exposed gossans among {@code bodies}. */
        public static Caps of(WorldData wd, List<Body> bodies) {
            List<Body> open = bodies.stream()
                    .filter(b -> ROLE.equals(b.role()) && b.rockClass() != null && cropsOut(wd, b))
                    .toList();
            return new Caps(wd.circumference(), open, (x, z) -> groundAt(wd, x, z));
        }

        /** True when no exposed gossan touches the chunk: nothing to look up. */
        public boolean isEmpty() {
            return gossans.isEmpty();
        }

        /**
         * The window of column (x, z), or null where no exposed gossan lies over it. Where two caps
         * lie over it the window is their hull.
         */
        public Span window(int x, int z) {
            Span w = null;
            int g = Integer.MIN_VALUE;
            for (Body b : gossans) {
                if (!holdsColumn(b, c, x, z))
                    continue;
                if (g == Integer.MIN_VALUE)
                    g = ground.applyAsInt(x, z);
                Span s = GossanSkin.window(g, thickness(b));
                w = w == null ? s : new Span(Math.max(w.from(), s.from()), Math.min(w.to(), s.to()));
            }
            return w;
        }

        /**
         * Whether the block at height y of a column whose {@link #window} is {@code window} is a
         * leached cap's. The rule is for the ore of the deposit's own bodies; a gossan's gold zone and
         * a placer's gold are not held back by it ({@link DepositFeature}): the skin leaves a placer's
         * cells alone, and the placer's gold goes there.
         */
        public boolean holdsNoOre(Span window, int y) {
            return window != null && y <= window.from() && y >= window.to();
        }
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
        List<Body> placers = placersOf(bodies, palette::isLoose);
        for (Body b : bodies) {
            if (!ROLE.equals(b.role()) || b.rockClass() == null)
                continue;
            double ground = wd.height(b.center()[0], b.center()[2]);
            boolean open = cropsOut(wd, b);
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
            int thick = thickness(b), capped = 0, wet = 0, held = 0;
            for (int lx = 0; lx < 16; lx++) {
                for (int lz = 0; lz < 16; lz++) {
                    int x = x0 + lx, z = z0 + lz;
                    if (!holdsColumn(b, c, x, z))
                        continue;
                    int top = IcePass.surfaceY(chunk, x, z, minY);
                    if (top < minY) {
                        wet++;   // a fluid or empty column: the sea or a lake covers it
                        continue;
                    }
                    // The cap hangs from the ground the terrain model gives, within the margin: the
                    // blocks it repaints are then inside the window the ore feature leaves ore-free.
                    int hang = anchor(top, groundAt(wd, x, z));
                    if (hang != top)
                        held++;
                    Span s = cap(hang, thick);
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
            CAPPED.addAndGet(capped);
            CLAMPED.addAndGet(held);
            if (capped + wet > 0)
                DeepTime.LOGGER.debug("gossan {} in chunk ({}, {}): {} column(s) capped ({} held to the margin), {} under water", b.id(),
                        chunk.getPos().x, chunk.getPos().z, capped, held, wet);
        }
        return total;
    }
}
