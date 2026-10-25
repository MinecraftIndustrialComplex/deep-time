// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
import net.minecraft.gametest.framework.GameTestAssertException;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.MinecraftServer;

/**
 * Runs a GameTest's scattered column samples a few per server tick, under one budget shared by the
 * whole GameTest server, instead of all of them in the test's start tick.
 *
 * <p>Why: a chunk requested at NOISE or later needs STRUCTURE_STARTS in a radius of 8 around it
 * ({@code ChunkPyramid}), so one sample far from the others loads up to 17 × 17 = 289 proto-chunks.
 * A chunk's load ticket lasts one tick, and nothing unloads inside a tick, so a test that samples
 * 128 columns across a planet in its start tick holds all of them at once: the standalone census
 * ({@link ChunkCensus}) saw {@code heightMatchesReference} alone add 41,408 chunks and the suite
 * peak at 51,793, most of it on the C = 16,384 and C = 4,096 test planets. The 168-mod MIC server
 * (C2ME, the pack's structure sets) carries more per chunk and ran its 8 GB heap out every run,
 * then stalled in {@code lakeHolds} (the one test still waiting when the heap died; it passes
 * alone). Spreading the samples lets each tick's chunks unload before the next ones load, so the
 * resident set is bounded by the budget, not by the suite. Every sample is still taken, from the
 * same chunk statuses; only when changes.
 */
final class ChunkSampling {

    /**
     * Sample steps the whole GameTest server may run per tick. A step touches one column's chunk
     * (or two neighbouring ones), so at most about this many × 289 proto-chunks are loaded per tick.
     */
    static final int STEPS_PER_TICK = 8;

    private static long budgetTick = Long.MIN_VALUE;
    private static int budgetUsed;

    private ChunkSampling() {}

    /** One sample. Returns false when it requested no chunk (a skipped column), which costs no budget. */
    @FunctionalInterface
    interface Step {
        boolean sample(int i);
    }

    /**
     * Runs {@code step} for i = 0 … n − 1 within the shared per-tick budget, then {@code finish}
     * (which asserts and calls {@code helper.succeed()}). An assertion failure in a step fails the
     * test after the remaining steps are dropped.
     */
    static void forEach(GameTestHelper helper, int n, Step step, Runnable finish) {
        MinecraftServer server = helper.getLevel().getServer();
        int[] next = {0};
        RuntimeException[] failure = {null};
        helper.startSequence()
                .thenWaitUntil(() -> {
                    while (failure[0] == null && next[0] < n && take(server)) {
                        try {
                            if (!step.sample(next[0]))
                                budgetUsed--;
                        } catch (RuntimeException e) {
                            failure[0] = e;
                        }
                        next[0]++;
                    }
                    if (failure[0] == null && next[0] < n)
                        throw new GameTestAssertException("sampled " + next[0] + " of " + n);   // not done: again next tick
                })
                .thenExecute(() -> {
                    try {
                        if (failure[0] != null)
                            throw failure[0];
                        finish.run();
                    } catch (GameTestAssertException e) {
                        throw e;
                    } catch (RuntimeException e) {
                        DeepTime.LOGGER.error("[gametest] sampled test threw", e);
                        throw new GameTestAssertException(e.toString());
                    }
                });
    }

    /**
     * Whether block (x, z) lies on the world's faces (x wraps): always on a world that is not a cube.
     * Off a cube's faces a chunk is a turned copy of its partner up to its carvers ({@code CubeFill}):
     * at the noise step it is still empty, it has no features of its own, and the void squares'
     * diagonals join two partners' terrain, so a test of generation reads the faces only.
     */
    static boolean onFaces(WorldData wd, long x, long z) {
        return !(wd.projection() instanceof WorldProjection.Cube cube) || cube.cube().faceOfBlock(x, z) != null;
    }

    private static boolean take(MinecraftServer server) {
        long tick = server.getTickCount();
        if (tick != budgetTick) {
            budgetTick = tick;
            budgetUsed = 0;
        }
        if (budgetUsed >= STEPS_PER_TICK)
            return false;
        budgetUsed++;
        return true;
    }

    /**
     * The height comparison three tests share: corner columns (x ≡ z ≡ 0 mod 4) must match exactly,
     * the rest within a tolerance; keeps the worst column for the message.
     */
    static final class HeightTally {
        int checked, corners, cornerMax, maxDiff;
        String worst = "";

        void add(int x, int z, boolean corner, int expected, int actual, double h) {
            int diff = Math.abs(actual - expected);
            checked++;
            if (corner) {
                corners++;
                cornerMax = Math.max(cornerMax, diff);
            }
            if (diff > maxDiff) {
                maxDiff = diff;
                worst = "(" + x + ", " + z + ") expected " + expected + " got " + actual + " H=" + h;
            }
        }
    }
}
