// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import org.junit.jupiter.api.Test;

import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The strata pass's column rule over whole chunks of the real test planet ({@code planet4k}): the
 * time the rule takes for the 256 columns of a chunk, as {@code StrataPass.apply} calls it (one
 * reused stack per thread, columns of a chunk in order), without the world's folds and faults
 * ({@link Columns#stack}) and through them ({@link Columns#deformedStack}, P2-01). Prints
 * {@code [bench] column rule …}; the test only checks the rule answers every column. The numbers
 * are what STATUS.md's strata entries quote as the strata pass's ms/chunk (before the writes).
 *
 * <p>{@code DEEPTIME_BENCH_WORLD=<world-data folder>} measures another baked planet instead.
 */
class ColumnRuleBenchTest {

    private interface Rule {
        void stack(int x, int z, Columns.Stack out);
    }

    /** Best of three passes: {mean ms/chunk, the slowest chunk's ms, chunks, mean horizons}. */
    private static double[] measure(int c, Columns.Stack st, Rule rule) {
        double best = Double.MAX_VALUE, worst = 0;
        long horizons = 0;
        int chunks = 0;
        for (int pass = 0; pass < 3; pass++) {
            long t0 = System.nanoTime();
            chunks = 0;
            horizons = 0;
            double slowest = 0;
            for (int cz = -c / 4 / 16 + 4; cz < c / 4 / 16 - 4; cz += 7) {
                for (int cx = -c / 2 / 16; cx < c / 2 / 16; cx += 5) {
                    long t1 = System.nanoTime();
                    for (int lx = 0; lx < 16; lx++)
                        for (int lz = 0; lz < 16; lz++) {
                            rule.stack(cx * 16 + lx, cz * 16 + lz, st);
                            horizons += st.count;
                            assertTrue(st.count > 0, "no horizons at chunk " + cx + ", " + cz);
                        }
                    slowest = Math.max(slowest, (System.nanoTime() - t1) / 1e6);
                    chunks++;
                }
            }
            double mean = (System.nanoTime() - t0) / 1e6 / chunks;
            if (mean < best) {
                best = mean;
                worst = slowest;
            }
        }
        return new double[] {best, worst, chunks, horizons / (chunks * 256.0)};
    }

    @Test
    void columnRuleMsPerChunk() {
        String other = System.getenv("DEEPTIME_BENCH_WORLD");
        WorldData wd = WorldData.open(other == null || other.isEmpty() ? Fixtures.world("planet4k") : Path.of(other));
        RasterSampler strata = wd.layer(WorldData.HORIZONS);
        int c = wd.circumference();
        Columns.Stack st = new Columns.Stack(strata.stackDepth());
        double[] plain = measure(c, st, (x, z, out) -> Columns.stack(wd, strata, x, z, out));
        System.out.printf("[bench] column rule ms/chunk=%.4f chunks=%d mean_horizons=%.2f%n", plain[0],
                (int) plain[2], plain[3]);
        Structure structure = wd.structure();
        if (structure != null) {
            Structure.Scratch scratch = new Structure.Scratch();
            double[] bent = measure(c, st, (x, z, out) -> Columns.deformedStack(wd, strata, x, z, out, scratch));
            System.out.printf(
                    "[bench] column rule through folds and faults ms/chunk=%.4f slowest_chunk_ms=%.3f events=%d "
                            + "mean_horizons=%.2f (plain %.4f, slowest %.3f) circumference=%d%n",
                    bent[0], bent[1], structure.size(), bent[3], plain[0], plain[1], c);
        }
    }
}
