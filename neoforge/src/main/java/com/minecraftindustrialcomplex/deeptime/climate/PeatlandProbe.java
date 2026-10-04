// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.compat.terrablender.RegionCatalog;
import com.minecraftindustrialcomplex.deeptime.compat.terrablender.RegionTrees;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import net.minecraft.core.Holder;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.BiomeSource;
import net.minecraft.world.level.biome.MultiNoiseBiomeSource;

import java.util.List;
import java.util.Optional;

/**
 * Asks a level's biome source where the pack has a peatland biome ({@link Peatland}, BIO-105): the
 * registry-bound half of the decision, which reads the pack's parameter list (and TerraBlender's
 * region trees, when it is installed, one answer per region of the world's province table).
 */
public final class PeatlandProbe {

    private PeatlandProbe() {}

    /**
     * The world's peatland verdict, or null when it cannot be given yet (TerraBlender has not built its
     * region trees); a biome source that is not a multi-noise one has none, which is a verdict.
     */
    public static Peatland decide(BiomeSource source, ClimateMapping m, WorldData wd) {
        JsonObject climate = ClimateMappingBuilder.bakeClimate(wd);
        if (climate == null)
            return null;
        OverworldBands ref = OverworldBands.fromJson(climate.getAsJsonObject("reference").getAsJsonObject("bands"));
        double coldBelow = climate.has("wetland_cold_below") ? climate.get("wetland_cold_below").getAsDouble() : 2.0;
        if (!(source instanceof MultiNoiseBiomeSource ms))
            return Peatland.decide(m, ref, List.of(), coldBelow, (r, t) -> null);
        var params = ms.parameters();
        Optional<RegionCatalog> catalog = RegionCatalog.load();
        Peatland p;
        if (catalog.isPresent() && m.regions() != null) {
            if (!RegionTrees.ready(params))
                return null;
            p = Peatland.decide(m, ref, m.regions().names(), coldBelow, (region, t) -> {
                int index = catalog.get().indexOf(region);
                return index < 0 ? null : peat(RegionTrees.find(params, index, t));
            });
        } else {
            p = Peatland.decide(m, ref, List.of(Peatland.NO_TABLE), coldBelow, (region, t) -> peat(params.findValue(t)));
        }
        DeepTime.LOGGER.info("Peatland of {}: {}", wd.folder(), p.regions().isEmpty()
                ? "the pack has no peatland biome where cold wetland is: it is taiga"
                : "cold wetland is " + p.biomes() + " in " + p.regions() + " (erosion " + p.erosion() + "), taiga elsewhere");
        return p;
    }

    private static String peat(Holder<Biome> b) {
        return b != null && Peatland.isPeatland(b) ? b.unwrapKey().map(k -> k.location().toString()).orElse("?") : null;
    }
}
