// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The game's per-chunk water path ({@link Water#region}, what {@code WaterBodies} evaluates: one
 * gather of the river segments and the noise-cell corners per chunk) gives exactly the columns the
 * per-column evaluator gives ({@link Water#column}, which {@code ColumnParityTest} holds to Rust),
 * on the real test planet, around river vertices and across the seam; and what it costs a chunk.
 */
class WaterRegionTest {

    @Test
    void theChunkPathIsThePerColumnEvaluator() {
        WorldData wd = WorldData.open(ColumnParityTest.world());
        Rivers r = wd.rivers();
        assertNotNull(r, "the real test planet has a river network");
        assertTrue(r.chainCount() > 0);
        Water.Spec spec = Water.Spec.of(wd, 4);
        int c = wd.circumference();
        int chunks = 0, water = 0, banks = 0;
        long nanos = 0;
        int step = Math.max(1, r.vertexCount() / 24);
        // A warm-up pass so the timing below is the JIT-compiled path, as in a running server.
        for (int i = 0; i < r.vertexCount(); i += step) {
            long x0 = Math.floorDiv(Math.round(r.vertexX(i)), 16) * 16L, z0 = Math.floorDiv(Math.round(r.vertexZ(i)), 16) * 16L;
            for (int k = 0; k < 20; k++)
                Water.region(wd, spec, x0 - 1 + 16L * k, z0 - 1, 18, 18).column(1, 1);
        }
        for (int i = 0; i <= r.vertexCount(); i += step) {
            // Chunks around river vertices, plus one straddling the seam.
            long x0, z0;
            if (i >= r.vertexCount()) {
                x0 = c / 2 - 8;
                z0 = 0;
            } else {
                x0 = Math.floorDiv(Math.round(r.vertexX(i)), 16) * 16L;
                z0 = Math.floorDiv(Math.round(r.vertexZ(i)), 16) * 16L;
            }
            long t = System.nanoTime();
            Water.Region reg = Water.region(wd, spec, x0 - 1, z0 - 1, 18, 18);
            Water.Column[] cols = new Water.Column[256];
            for (int dz = 0; dz < 16; dz++)
                for (int dx = 0; dx < 16; dx++)
                    cols[dz * 16 + dx] = reg.column(dx + 1, dz + 1);
            nanos += System.nanoTime() - t;
            chunks++;
            for (int dz = 0; dz < 16; dz++)
                for (int dx = 0; dx < 16; dx++) {
                    Water.Column a = cols[dz * 16 + dx];
                    Water.Column b = Water.column(wd, spec, x0 + dx, z0 + dz);
                    assertEquals(b, a, "column (" + (x0 + dx) + ", " + (z0 + dz) + ")");
                    if (a.kind().isWater())
                        water++;
                    if (a.kind() == Water.Kind.BANK)
                        banks++;
                }
        }
        System.out.printf("[junit] WaterRegionTest chunks=%d waterColumns=%d bankColumns=%d regionMicrosPerChunk=%.1f%n",
                chunks, water, banks, nanos / 1000.0 / chunks);
        assertTrue(water > 0 && banks > 0, "the chunks reach water and banks");
    }
}
