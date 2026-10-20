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

    /** Where the level has lava underground: vanilla's rules, measured from the floor, unless the world data says otherwise. */
    default LavaRules lava() {
        return LavaRules.DEFAULT;
    }

    /**
     * Whether the level is a seamless Petroff–Guyou world (format 0.5): its north and south edges are
     * glued by half-turns and the terrain past them is the folded inside (no pole caps, nothing
     * "beyond the pole"). False on a Lambert world and on the analytic test terrains.
     */
    default boolean seamless() {
        return false;
    }

    /**
     * The half-extent of the world in z, blocks: the poles R = C/2π of a Lambert world, the turning
     * edges b/2 of a seamless one. The z range a census or a probe walks.
     */
    default double zHalf() {
        return seamless() ? circumference() / 4.0 : com.minecraftindustrialcomplex.deeptime.worlddata.Projection.poleZ(circumference());
    }

    /**
     * How near column (x, z) lies to the level's rivers, lakes and shallow sea, for the router's
     * cave-entrance band ({@code deeptime:water_shield}, {@link WaterCaveShield}): 1 at and beside
     * the water, 0 away from it. 0 in a level without Deep Time's water (the analytic terrains).
     */
    default double waterShield(int x, int z) {
        return 0.0;
    }

    /** Folds x into the canonical span [-C/2, C/2). */
    default int wrapX(int x) {
        int c = circumference();
        return Math.floorMod(x + c / 2, c) - c / 2;
    }
}
