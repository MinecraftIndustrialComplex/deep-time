// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * {@link Playa#within}, the read that keeps the volcanic roles {@code VolcanicBiomes.PLAYA_MARGIN} blocks off a playa
 * (BIO-165). Checked on a layer made to hold the cases, so the check does not depend on where a simulated planet puts
 * its salt flats (the test planet of master 9c9c2012 has none).
 */
class PlayaTest {

    /**
     * For every block around a lone playa cell, a 2 × 2 flat, a flat across the x seam and one on the z edge, the read
     * answers as the blocks do: true exactly when some block within the margin, in x and in z, has a playa cell as its
     * nearest cell.
     */
    @Test
    void theMarginReadAnswersAsTheBlocksDo(@TempDir Path tmp) throws IOException {
        int w = 24, h = 12, r = 8, m = 20;
        int[] cells = new int[w * h];
        cells[5 * w + 6] = 1;
        cells[6 * w + 14] = cells[6 * w + 15] = cells[7 * w + 14] = cells[7 * w + 15] = 3;
        cells[2 * w] = 2;
        cells[2 * w + w - 1] = 2;
        cells[(h - 1) * w + 10] = 1;
        RasterSampler layer = IceCoverTest.rawU8Layer(tmp.resolve("playa.dtr"), w, h, r, cells);
        var g = layer.desc().grid();
        int x0 = (int) g.originBlockX() - 2 * r, x1 = (int) g.originBlockX() + (w + 2) * r;
        int z0 = (int) g.originBlockZ() - 3 * r, z1 = (int) g.originBlockZ() + (h + 3) * r;
        // Playa blocks over the checked area and the margin around it, summed so a window's count is four reads.
        int bx0 = x0 - m, bz0 = z0 - m, bw = x1 - x0 + 2 * m, bh = z1 - z0 + 2 * m;
        int[][] sum = new int[bh + 1][bw + 1];
        for (int j = 0; j < bh; j++)
            for (int i = 0; i < bw; i++)
                sum[j + 1][i + 1] = sum[j][i + 1] + sum[j + 1][i] - sum[j][i]
                        + (layer.nearest(bx0 + i, bz0 + j) > 0 ? 1 : 0);
        int near = 0, far = 0;
        for (int z = z0; z < z1; z++)
            for (int x = x0; x < x1; x++) {
                int i = x - m - bx0, j = z - m - bz0, s = 2 * m + 1;
                boolean blocks = sum[j + s][i + s] - sum[j][i + s] - sum[j + s][i] + sum[j][i] > 0;
                boolean read = Playa.within(layer, x, z, m);
                assertEquals(blocks, read, "at (" + x + ", " + z + ")");
                if (read)
                    near++;
                else
                    far++;
            }
        assertTrue(near > 0 && far > 0, near + " near, " + far + " far");
        // The edges: 20 blocks off a lone cell's last block is near, 21 is not.
        int cx = (int) g.originBlockX() + 6 * r, cz = (int) g.originBlockZ() + 5 * r;
        int last = cx + r / 2 - 1;
        assertTrue(layer.nearest(last, cz) > 0 && layer.nearest(last + 1, cz) == 0);
        assertTrue(Playa.within(layer, last + m, cz, m));
        assertFalse(Playa.within(layer, last + m + 1, cz, m));
        assertFalse(Playa.within(null, cx, cz, m));
    }
}
