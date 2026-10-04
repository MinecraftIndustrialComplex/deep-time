// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.compat.terrablender.RegionCatalog;
import com.minecraftindustrialcomplex.deeptime.compat.terrablender.RegionTrees;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import net.minecraft.core.Holder;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.tags.TagKey;
import net.minecraft.world.level.biome.BiomeSource;
import net.minecraft.world.level.biome.Climate;
import net.minecraft.world.level.biome.MultiNoiseBiomeSource;

import java.util.List;
import java.util.Optional;

/**
 * Asks a level's biome source where the pack has a peatland biome ({@link Peatland}, BIO-117): the
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
        if (!(source instanceof MultiNoiseBiomeSource ms))
            return Peatland.decide(m, ref, List.of(), (r, t, h, c, e, w) -> null);
        var params = ms.parameters();
        Optional<RegionCatalog> catalog = RegionCatalog.load();
        Peatland p;
        if (catalog.isPresent() && m.regions() != null) {
            if (!RegionTrees.ready(params))
                return null;
            p = Peatland.decide(m, ref, m.regions().names(), (region, t, h, c, e, w) -> {
                int index = catalog.get().indexOf(region);
                return index < 0 ? null : peat(RegionTrees.find(params, index, target(t, h, c, e, w)));
            });
        } else {
            p = Peatland.decide(m, ref, List.of(Peatland.NO_TABLE), (region, t, h, c, e, w) -> peat(params.findValue(target(t, h, c, e, w))));
        }
        DeepTime.LOGGER.info("Peatland of {}: {}", wd.folder(), p.regions().isEmpty()
                ? "the pack has no peatland biome where cold wetland is: it is taiga"
                : "cold wetland is " + p.biomes() + " in " + p.regions() + " (erosion " + p.erosion() + "), taiga elsewhere");
        return p;
    }

    private static Climate.TargetPoint target(double t, double h, double c, double e, double w) {
        return Climate.target((float) t, (float) h, (float) c, (float) e, 0f, (float) w);
    }

    /** The shared tag of known bogs, fens and muskegs ({@code data/deeptime/tags/worldgen/biome/peatland.json}); every entry optional. */
    public static final TagKey<Biome> PEATLAND = TagKey.create(Registries.BIOME,
            ResourceLocation.fromNamespaceAndPath("deeptime", "peatland"));

    /**
     * In {@code #deeptime:peatland} (the shared list a pack's support adds its fen or bog to), or, for a pack
     * nobody listed, a swamp and cold by the NeoForge tags, whatever the biome's name or mod.
     */
    public static boolean isPeatland(Holder<Biome> b) {
        return b.is(PEATLAND) || (b.is(tag("is_swamp")) && (b.is(tag("is_cold/overworld")) || b.is(tag("is_snowy"))));
    }

    private static TagKey<Biome> tag(String path) {
        return TagKey.create(Registries.BIOME, ResourceLocation.fromNamespaceAndPath("c", path));
    }

    private static String peat(Holder<Biome> b) {
        return b != null && isPeatland(b) ? b.unwrapKey().map(k -> k.location().toString()).orElse("?") : null;
    }
}
