// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The strata pass's column rule over whole chunks of the real test planet ({@code planet4k}): the
 * time {@link Columns#stack} takes for the 256 columns of a chunk, as {@code StrataPass.apply}
 * calls it (one reused stack per thread, columns of a chunk in order). Prints
 * {@code [bench] column rule …}; the test only checks the rule answers every column. The number
 * is what STATUS.md's strata entries quote as the strata pass's ms/chunk (before the writes).
 */
class ColumnRuleBenchTest {

    @Test
    void columnRuleMsPerChunk() {
        WorldData wd = WorldData.open(Fixtures.world("planet4k"));
        RasterSampler strata = wd.layer(WorldData.HORIZONS);
        int c = wd.circumference();
        Columns.Stack st = new Columns.Stack(strata.stackDepth());
        long horizons = 0;
        double best = Double.MAX_VALUE;
        int chunks = 0;
        for (int pass = 0; pass < 3; pass++) {
            long t0 = System.nanoTime();
            chunks = 0;
            for (int cz = -c / 4 / 16 + 4; cz < c / 4 / 16 - 4; cz += 7) {
                for (int cx = -c / 2 / 16; cx < c / 2 / 16; cx += 5) {
                    for (int lx = 0; lx < 16; lx++)
                        for (int lz = 0; lz < 16; lz++) {
                            Columns.stack(wd, strata, cx * 16 + lx, cz * 16 + lz, st);
                            horizons += st.count;
                            assertTrue(st.count > 0, "no horizons at chunk " + cx + ", " + cz);
                        }
                    chunks++;
                }
            }
            best = Math.min(best, (System.nanoTime() - t0) / 1e6 / chunks);
        }
        System.out.printf("[bench] column rule ms/chunk=%.4f chunks=%d mean_horizons=%.2f%n", best, chunks,
                horizons / (3.0 * chunks * 256));
    }
}
