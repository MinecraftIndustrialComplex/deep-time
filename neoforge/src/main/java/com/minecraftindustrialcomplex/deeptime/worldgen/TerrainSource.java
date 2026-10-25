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
     * Whether the level is the cube world (format 0.7, docs/design/cube-game.md): its turning edges are
     * glued by turns and the terrain past them is the folded inside. False on the Lambert-frame test
     * worlds and the analytic test terrains.
     */
    default boolean cube() {
        return false;
    }

    /**
     * The half-extent of the world in z, blocks: the net's 3s/2 (the polar faces' outer edges) on the
     * cube, the poles R = C/2π of a Lambert-frame test world. The z range a census or a probe walks.
     */
    default double zHalf() {
        if (cube())
            return 3.0 * circumference() / 8.0;
        return com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection.Lambert.radius(circumference());
    }

    /**
     * How near column (x, z) lies to the level's rivers, lakes and shallow sea, for the router's
     * cave-entrance band ({@code deeptime:water_shield}, {@link WaterCaveShield}): 1 at and beside
     * the water, 0 away from it. 0 in a level without Deep Time's water (the analytic terrains).
     */
    default double waterShield(int x, int z) {
        return 0.0;
    }

    /**
     * Whether no sea stands on column (x, z) though its ground is below sea level: a closed basin that the open sea
     * does not reach (the baked {@code closed_basin} layer, HYD-100). False in a level without it (the analytic
     * terrains): the sea stands on all ground below sea level.
     */
    default boolean closedBasin(int x, int z) {
        return false;
    }

    /** Folds x into the canonical span [-C/2, C/2). */
    default int wrapX(int x) {
        int c = circumference();
        return Math.floorMod(x + c / 2, c) - c / 2;
    }
}
