// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.worlddata.RasterSampler;

/**
 * Where {@link WaterPass} puts water, in plain Java (no Minecraft types, so JUnit can test it):
 * the water surface over a column from the {@code water_level} layer, and the block convention.
 */
public final class WaterLevels {

    private WaterLevels() {}

    /**
     * The water surface over column (x, z), NaN where there is none: the nearest cell's value, else
     * that of the nearest wet cell within {@code reach} blocks (the shore rule, see {@link WaterPass}).
     */
    public static double levelAt(RasterSampler levels, int x, int z, int reach) {
        return com.minecraftindustrialcomplex.deeptime.worlddata.Water.levelAt(levels, x, z, reach);
    }

    /** Whether any cell whose block position lies in the box [x0, x1] × [z0, z1] (or next to it) is wet. */
    public static boolean anyWet(RasterSampler levels, int x0, int z0, int x1, int z1) {
        return levels.anyInBox(x0, z0, x1, z1, v -> !Double.isNaN(v));
    }

    /**
     * The highest water block under a water surface at {@code level}: {@code ceil(level) - 1}, as for
     * vanilla's sea level (63 → water up to y 62).
     */
    public static int topWaterBlock(double level) {
        return (int) Math.ceil(level) - 1;
    }
}
