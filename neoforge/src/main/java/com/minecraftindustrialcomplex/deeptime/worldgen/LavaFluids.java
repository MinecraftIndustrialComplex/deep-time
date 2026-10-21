// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.levelgen.Aquifer;
import net.minecraft.world.level.levelgen.NoiseGeneratorSettings;

/**
 * A Deep Time level's global fluid picker: what fills open space the aquifer does not decide
 * (plan §7.1). It replaces vanilla's, which fills everything below y −54 with lava whatever the
 * world's floor is ({@link LavaRules} has the measurement).
 *
 * <ul>
 *   <li>At and above the lava floor ({@link LavaRules#floodY}, y −342 by default): the sea, as
 *       vanilla. Sea water therefore reaches any sea bed above the floor.</li>
 *   <li>Except in a closed basin below sea level ({@link TerrainSource#closedBasin}, the baked
 *       {@code closed_basin} layer; HYD-100): no sea there. The status is water at the lava floor's
 *       level, so the aquifer's own local water tables under the basin stay water (its randomized
 *       levels take the global status's fluid), but nothing floods to sea level: neither the
 *       basin's open air (the aquifer reads the picker at the column's surface) nor its caves
 *       (vanilla's "flooded" aquifer cells take the global status's level).</li>
 *   <li>Below it: lava, up to the floor's level, except in a column whose ground is itself that
 *       low (a trench): there the lava's surface is {@link LavaRules#SEA_COVER} blocks below the
 *       ground and everything above it is the sea, so water never stands on lava.</li>
 * </ul>
 *
 * <p>Pure and thread-safe. The terrain height is read only below the lava floor, and the closed
 * basin layer only below sea level, so the picker costs nothing in the open air above the sea,
 * where vanilla asks it for every block.
 *
 * <p>The picker answers for the column it is asked about, but the aquifer also asks it at
 * neighbouring columns up to three chunks away and can take a nearby sea's status into a closed
 * basin near the coast; {@link WaterBodies} drains that water (FORMAT.md, "Water columns").
 */
public final class LavaFluids {

    private LavaFluids() {}

    public static Aquifer.FluidPicker picker(NoiseGeneratorSettings settings, TerrainSource terrain) {
        LavaRules rules = terrain.lava();
        int minY = settings.noiseSettings().minY();
        int sea = settings.seaLevel();
        int flood = rules.floodY(minY, sea);
        Aquifer.FluidStatus lava = new Aquifer.FluidStatus(flood, Blocks.LAVA.defaultBlockState());
        Aquifer.FluidStatus water = new Aquifer.FluidStatus(sea, settings.defaultFluid());
        Aquifer.FluidStatus closed = new Aquifer.FluidStatus(flood, settings.defaultFluid());
        return (x, y, z) -> {
            if (y >= flood)
                return y < sea && terrain.closedBasin(x, z) ? closed : water;
            int column = rules.columnFloodY(minY, sea, terrain.height(x, z));
            if (column >= flood)
                return lava;
            // A sea bed within reach of the lava floor: lava only well below the bed.
            return y < column ? new Aquifer.FluidStatus(column, Blocks.LAVA.defaultBlockState()) : water;
        };
    }
}
