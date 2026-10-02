// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.terrablender;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.climate.ProvinceRegions;
import net.neoforged.fml.ModList;

import java.util.List;
import java.util.Optional;

/**
 * TerraBlender's overworld regions as Deep Time sees them (LGPL-3.0; plan §7.3, P1-10): their ids,
 * weights and indices. Only {@link TerraBlenderRegions} names TerraBlender's classes; it is loaded
 * reflectively and only when TerraBlender is installed, so Deep Time runs without it. Signatures
 * here use no TerraBlender types.
 */
public interface RegionCatalog {

    /** TerraBlender's mod id. */
    String MOD_ID = "terrablender";

    /** The registered overworld regions, in TerraBlender's index order. */
    List<ProvinceRegions.Entry> overworldRegions();

    /** TerraBlender's index of an overworld region id, or -1 when no such region is registered. */
    int indexOf(String regionId);

    /** The catalogue when TerraBlender is installed and answers; empty otherwise (logged). */
    static Optional<RegionCatalog> load() {
        if (ModList.get() == null || !ModList.get().isLoaded(MOD_ID))
            return Optional.empty();
        try {
            Class<?> impl = Class.forName("com.minecraftindustrialcomplex.deeptime.compat.terrablender.TerraBlenderRegions");
            return Optional.of((RegionCatalog) impl.getDeclaredConstructor().newInstance());
        } catch (Throwable t) {
            DeepTime.LOGGER.error("TerraBlender is installed but its API did not answer; biome regions are TerraBlender's own", t);
            return Optional.empty();
        }
    }
}
