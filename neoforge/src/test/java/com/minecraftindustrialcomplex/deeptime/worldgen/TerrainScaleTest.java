// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * {@link TerrainScale}: gentle ground keeps k, and on a steep face the router's interpolation
 * between noise-cell corners draws the bilinear H instead of the P1-08 staircase ("striped snow").
 */
class TerrainScaleTest {

    /** The router's band: the terrain value within 1.5625 of zero, the cave field beyond it. */
    static final double BAND = 1.5625;
    /** A typical solid cave-field value (what a corner deep in rock carries instead). */
    static final double CAVE = 0.5;

    record Plane(double h0, double slope) implements TerrainSource {
        @Override
        public int circumference() {
            return 16_384;
        }

        @Override
        public double height(int x, int z) {
            return h0 - slope * x;
        }

        @Override
        public double raster(String layer, int x, int z) {
            return 0;
        }
    }

    /** The router's corner value: terrain in the band, the cave field below it. */
    static double corner(double h, double y, double scale) {
        double d = (h - y) / scale;
        return d < BAND ? d : CAVE;
    }

    /**
     * The top solid block of column x between corners 0 and 4 as vanilla draws it: the highest y
     * where the corner values, interpolated linearly in x, are positive.
     */
    static int drawnTop(TerrainSource src, int x, double k, boolean adaptive) {
        int x0 = Math.floorDiv(x, 4) * 4, x1 = x0 + 4;
        double h0 = src.height(x0, 0), h1 = src.height(x1, 0);
        double s0 = adaptive ? TerrainScale.at(src, x0, 0, h0, k) : k;
        double s1 = adaptive ? TerrainScale.at(src, x1, 0, h1, k) : k;
        double t = (x - x0) / 4.0;
        for (int y = 639; y > -352; y--) {
            double d = (1 - t) * corner(h0, y, s0) + t * corner(h1, y, s1);
            if (d > 0)
                return y;
        }
        return -352;
    }

    @Test
    void theScaleIsKTimesAPowerOfTwoCoveringTheDrop() {
        Plane p = new Plane(400, 6.0);
        double s = TerrainScale.at(p, 0, 0, p.height(0, 0), 8.0);
        assertEquals(32.0, s);   // 0.8 x 24 = 19.2: 8, 16, 32
        // Exact at a whole-number height: the interpolated density is zero at y = H, not 1e-16.
        double h = 110.0, d104 = (h - 104) / s, d112 = (h - 112) / s;
        assertEquals(0.0, d104 + (d112 - d104) * (6.0 / 8.0));
    }

    @Test
    void gentleGroundKeepsK() {
        Plane p = new Plane(100, 0.5);
        assertEquals(8.0, TerrainScale.at(p, 0, 0, p.height(0, 0), 8.0));
        assertEquals(8.0, TerrainScale.at(p, 400, 0, p.height(400, 0), 8.0));
    }

    @Test
    void aSteepFaceIsDrawnAsTheBilinearHeightNotAStaircase() {
        Plane p = new Plane(400, 6.0);   // 24 blocks between corners, like P1-08's east flank
        int worstFixed = 0, worstAdaptive = 0;
        for (int x = 0; x < 64; x++) {
            int want = (int) Math.ceil(p.height(x, 0)) - 1;
            worstFixed = Math.max(worstFixed, Math.abs(drawnTop(p, x, 8.0, false) - want));
            worstAdaptive = Math.max(worstAdaptive, Math.abs(drawnTop(p, x, 8.0, true) - want));
        }
        // The fixed scale pulls the surface up-slope by most of a cell's drop (the staircase) …
        assertTrue(worstFixed >= 8, "fixed k: worst " + worstFixed);
        // … the grown one keeps it where H is.
        assertTrue(worstAdaptive <= 1, "adaptive: worst " + worstAdaptive);
        // Corners stay exact either way.
        for (int x = 0; x < 64; x += 4)
            assertEquals((int) Math.ceil(p.height(x, 0)) - 1, drawnTop(p, x, 8.0, true));
    }
}
