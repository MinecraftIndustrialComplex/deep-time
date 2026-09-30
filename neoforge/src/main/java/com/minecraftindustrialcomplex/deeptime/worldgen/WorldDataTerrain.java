// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.climate.ClimateMapping;
import com.minecraftindustrialcomplex.deeptime.worlddata.Rivers;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;

/**
 * The terrain source of a real Deep Time level: the world data's rasters (plan §7.1). Pure Java
 * over memory-mapped tile packs, so chunk generation never needs the natives. The data is looked
 * up in {@link ActiveWorldData} on first use and then kept, with the world's climate mapping
 * (P1-09, plan §7.2), which the five biome inputs pass through.
 */
public final class WorldDataTerrain implements TerrainSource {

    private final WorldDataRef ref;
    private volatile WorldData data;
    private volatile ClimateMapping mapping;

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

    /** The world's climate mapping (identity for worlds without one). */
    public ClimateMapping mapping() {
        ClimateMapping m = mapping;
        if (m == null) {
            data();
            m = ActiveWorldData.mapping(ref.folder());
            mapping = m;
        }
        return m;
    }

    @Override
    public int circumference() {
        return ref.circumferenceBlocks();
    }

    @Override
    public double height(int x, int z) {
        return data().height(wrapX(x), z);
    }

    /**
     * A biome input through the world's mapping. In a world the P1-09 bake wrote, the weirdness is
     * sampled as an odd field ({@code RasterSampler.bilinearOdd}): its sign flips at the variant
     * noise's zero line, and a plain bilinear ramp through 0 there would draw lines of river biome
     * (the valley slice) across the land.
     */
    @Override
    public double raster(String layer, int x, int z) {
        ClimateMapping m = mapping();
        if (ClimateMapping.RIDGES.equals(layer)) {
            // P1-22: a channel wide enough for a river biome puts the column in the valley slice
            // (weirdness 0, the middle of the river slice of every overworld-style parameter list).
            Rivers r = data().rivers();
            if (r != null && r.sample(wrapX(x), z).biome())
                return 0.0;
        }
        if (!m.isIdentity() && ClimateMapping.RIDGES.equals(layer))
            return m.apply(layer, data().layer(layer).bilinearOdd(wrapX(x), z));
        return m.apply(layer, data().raster(layer, wrapX(x), z));
    }
}
