// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;

/**
 * The terrain source of a real Deep Time level: the world data's rasters (plan §7.1). Pure Java
 * over memory-mapped tile packs, so chunk generation never needs the natives. The data is looked
 * up in {@link ActiveWorldData} on first use and then kept.
 */
public final class WorldDataTerrain implements TerrainSource {

    private final WorldDataRef ref;
    private volatile WorldData data;

    public WorldDataTerrain(WorldDataRef ref) {
        this.ref = ref;
    }

    public WorldData data() {
        WorldData d = data;
        if (d == null) {
            d = ActiveWorldData.get(ref.folder());
            if (d.circumference() != ref.circumferenceBlocks())
                throw new IllegalStateException("world data '" + ref.folder() + "' has C = " + d.circumference()
                        + " but its generator says circumference_blocks = " + ref.circumferenceBlocks());
            data = d;
        }
        return d;
    }

    @Override
    public int circumference() {
        return ref.circumferenceBlocks();
    }

    @Override
    public double height(int x, int z) {
        return data().height(wrapX(x), z);
    }

    @Override
    public double raster(String layer, int x, int z) {
        return data().raster(layer, wrapX(x), z);
    }
}
