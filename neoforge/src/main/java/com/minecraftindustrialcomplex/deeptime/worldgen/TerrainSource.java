// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

/**
 * What Deep Time's density functions sample for one level (plan §7.1): terrain height and the
 * five 2D biome inputs. Implementations are pure, allocation-free and thread-safe: vanilla and
 * C2ME call density functions from many worldgen threads at once.
 */
public interface TerrainSource {

    /** C, the east-west period in blocks. Every x-dependent value repeats with it. */
    int circumference();

    /** Terrain surface height H(x, z) in blocks: the ground is solid below H. */
    double height(int x, int z);

    /**
     * A 2D biome input in vanilla parameter units ([-1, 1] for the five climate parameters).
     *
     * @param layer {@code climate/temperature}, {@code climate/vegetation},
     *              {@code climate/continents}, {@code climate/erosion} or {@code climate/ridges}
     */
    double raster(String layer, int x, int z);

    /** Folds x into the canonical span [-C/2, C/2). */
    default int wrapX(int x) {
        int c = circumference();
        return Math.floorMod(x + c / 2, c) - c / 2;
    }
}
