// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.climate.ProvinceRegions;
import com.minecraftindustrialcomplex.deeptime.compat.terrablender.RegionCatalog;
import com.minecraftindustrialcomplex.deeptime.worlddata.RasterSampler;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * The TerraBlender region of every column of a Deep Time level (plan §7.3, P1-10): the baked
 * province at the column ({@code meta/province}, nearest cell, wrapping in x) looked up in the
 * world's province → region table ({@code climate_mapping.json}) and turned into TerraBlender's
 * region index. Pure and thread-safe; biome lookups call it for every quart position of every
 * chunk, so it is one raster read and one array read.
 *
 * <p>TerraBlender's own region is a layered noise of the position alone; this replaces it for
 * levels on Deep Time world data ({@code mixin.compat.terrablender.RegionBiomeSourceMixin}), so a
 * region covers whole geologic provinces.
 */
public final class RegionSource {

    /** TerraBlender's index of its default overworld region (vanilla's), where a province has none. */
    public static final int DEFAULT_REGION = 0;

    private final RasterSampler provinces;
    private final int[] regionOfProvince;
    private final TerrainSource terrain;
    private final List<String> names;

    RegionSource(RasterSampler provinces, int[] regionOfProvince, List<String> names, TerrainSource terrain) {
        this.provinces = provinces;
        this.regionOfProvince = regionOfProvince;
        this.names = names;
        this.terrain = terrain;
    }

    /** The province id at block (x, z), or −1 outside the layer's values. */
    public int provinceAt(int blockX, int blockZ) {
        double v = provinces.nearest(terrain.wrapX(blockX), blockZ);
        return Double.isNaN(v) ? -1 : (int) v;
    }

    /** TerraBlender's region index at block (x, z). */
    public int regionAt(int blockX, int blockZ) {
        int p = provinceAt(blockX, blockZ);
        return p < 0 || p >= regionOfProvince.length ? DEFAULT_REGION : regionOfProvince[p];
    }

    /** TerraBlender's region index of a province. */
    public int regionOfProvince(int province) {
        return province < 0 || province >= regionOfProvince.length ? DEFAULT_REGION : regionOfProvince[province];
    }

    public int provinceCount() {
        return regionOfProvince.length;
    }

    /** The region ids this source resolved, by TerraBlender index where known (for the probe). */
    public List<String> regionNames() {
        return names;
    }

    /**
     * The source for a level on world data whose mapping has a province → region table, resolved
     * against TerraBlender's registered regions; empty when there is none (TerraBlender absent,
     * no provinces baked, or the world's table not built yet).
     */
    public static Optional<RegionSource> of(WorldDataTerrain terrain, Optional<RegionCatalog> catalog) {
        if (catalog.isEmpty())
            return Optional.empty();
        ProvinceRegions table = terrain.mapping().regions();
        WorldData wd = terrain.data();
        if (table == null || !wd.has("meta/province"))
            return Optional.empty();
        int[] byProvince = new int[table.provinces()];
        List<String> unknown = new ArrayList<>();
        List<String> names = new ArrayList<>();
        for (int p = 0; p < byProvince.length; p++) {
            String id = table.regionOf(p);
            int idx = catalog.get().indexOf(id);
            if (idx < 0) {
                idx = DEFAULT_REGION;
                if (!unknown.contains(id))
                    unknown.add(id);
            }
            byProvince[p] = idx;
        }
        for (String n : table.names())
            names.add(n);
        if (!unknown.isEmpty())
            DeepTime.LOGGER.warn("Deep Time: this world's provinces name TerraBlender regions that are not installed now ({}); "
                    + "their provinces use the default overworld region", unknown);
        return Optional.of(new RegionSource(wd.layer("meta/province"), byProvince, List.copyOf(names), terrain));
    }
}
