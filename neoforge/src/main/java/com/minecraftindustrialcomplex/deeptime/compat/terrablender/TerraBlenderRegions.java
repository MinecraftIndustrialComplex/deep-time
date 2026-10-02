// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.terrablender;

import com.minecraftindustrialcomplex.deeptime.climate.ProvinceRegions;
import net.minecraft.resources.ResourceLocation;
import terrablender.api.Region;
import terrablender.api.RegionType;
import terrablender.api.Regions;

import java.util.ArrayList;
import java.util.List;

/**
 * {@link RegionCatalog} over TerraBlender 4.1's {@code Regions} registry. The one class (with
 * {@link RegionTrees} and the mixin) that names TerraBlender's types; loaded only when it is
 * installed.
 */
public final class TerraBlenderRegions implements RegionCatalog {

    @Override
    public List<ProvinceRegions.Entry> overworldRegions() {
        List<ProvinceRegions.Entry> out = new ArrayList<>();
        for (Region r : Regions.get(RegionType.OVERWORLD))
            out.add(new ProvinceRegions.Entry(r.getName().toString(), r.getWeight()));
        return out;
    }

    @Override
    public int indexOf(String regionId) {
        ResourceLocation id = ResourceLocation.tryParse(regionId);
        if (id == null)
            return -1;
        try {
            return Regions.getIndex(RegionType.OVERWORLD, id);
        } catch (RuntimeException unregistered) {
            return -1;
        }
    }
}
