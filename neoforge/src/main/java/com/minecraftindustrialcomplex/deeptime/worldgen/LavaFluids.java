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
 *   <li>Below it: lava, up to the floor's level, except in a column whose ground is itself that
 *       low (a trench): there the lava's surface is {@link LavaRules#SEA_COVER} blocks below the
 *       ground and everything above it is the sea, so water never stands on lava.</li>
 * </ul>
 *
 * <p>Pure and thread-safe. The terrain height is read only below the lava floor, so the picker
 * costs nothing in the open air and sea, where vanilla asks it for every block.
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
        return (x, y, z) -> {
            if (y >= flood)
                return water;
            int column = rules.columnFloodY(minY, sea, terrain.height(x, z));
            if (column >= flood)
                return lava;
            // A sea bed within reach of the lava floor: lava only well below the bed.
            return y < column ? new Aquifer.FluidStatus(column, Blocks.LAVA.defaultBlockState()) : water;
        };
    }
}
