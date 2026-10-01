// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import org.junit.jupiter.api.Test;

import java.nio.file.Path;

/** Temporary: far sections on a 16k world with two tile-cache shapes. Not committed. */
class TmpBigWorldBenchTest {

    @Test
    void bench() {
        String dir = System.getenv("DEEPTIME_BENCH_WORLD");
        if (dir == null)
            return;
        for (int conc : new int[]{32, 4}) {
            TileCache cache = new TileCache(TileCache.DEFAULT_BYTES, conc);
            WorldData wd = WorldData.open(Path.of(dir), cache);
            for (String l : new String[]{WorldData.ELEVATION, WorldData.HORIZONS, WorldData.WATER_LEVEL, IceCover.LAYER}) {
                if (!wd.has(l))
                    continue;
                LayerDesc d = wd.layer(l).desc();
                System.out.printf("[tmp] layer %s grid %dx%d r=%d tile=%d cellBytes=%d%n", l, d.grid().w(), d.grid().h(), d.grid().r(),
                        d.tile(), d.cellBytes());
            }
            FarColumn.Shape[] grid = FarColumnTest.shapes(64);
            Columns.Stack stack = new Columns.Stack(8);
            FarColumn.Layers layers = new FarColumn.Layers();
            for (int step : new int[]{1, 8, 64}) {
                double best = Double.MAX_VALUE;
                for (int pass = 0; pass < 2; pass++) {
                    long t0 = System.nanoTime();
                    for (int k = 0; k < 8; k++)
                        FarColumnTest.section(wd, 4096 + (long) k * 64 * step, -1024, step, grid, stack, layers);
                    best = Math.min(best, (System.nanoTime() - t0) / 1e6 / 8);
                }
                System.out.printf("[tmp] concurrency %d step %d ms/section=%.2f cache tiles=%d weight=%d MiB%n", conc, step, best,
                        cache.size(), cache.weightBytes() >> 20);
            }
        }
    }
}
