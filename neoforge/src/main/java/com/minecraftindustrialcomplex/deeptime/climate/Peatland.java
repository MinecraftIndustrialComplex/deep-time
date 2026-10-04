// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.core.Holder;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.tags.TagKey;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.Climate;

import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;

/**
 * Where the pack has a peatland biome for cold wetland (Ben, 2026-10-03, "Taiga instead, peatland
 * if present"; ledger BIO-105). The bake flags boreal wetland (`climate/wetland` = 2) and leaves its
 * ground in the ordinary land erosion, so vanilla draws the class's own biome, taiga. A pack that has a
 * bog or muskeg shows it where vanilla's swamp row is: the game raises the erosion of the flagged
 * ground to the swamp band (E6) in exactly the regions of the pack that answer there with a
 * peatland biome, found by asking the pack's own biome source at the cold wetland's climate, never by
 * a biome name: a peatland biome is one the NeoForge tags call both a swamp ({@code c:is_swamp}) and
 * cold ({@code c:is_cold/overworld} or {@code c:is_snowy}) ({@link #isPeatland}). Biomes O' Plenty's
 * bog and muskeg are; vanilla's and Terralith's swamps are not.
 *
 * <p>The verdict is per <em>region</em> (a TerraBlender region answers differently from another:
 * Biomes O' Plenty's primary region puts its wetland, which is not cold, at the swamp row and its
 * secondary region puts the bog), decided once per world and stored in {@code climate_mapping.json}
 * so a pack update does not move biomes in unexplored chunks.
 *
 * @param erosion the pack's erosion for flagged ground in a peatland region, inside its swamp band (E6)
 * @param regions the region ids (those of the world's province table) that answer with a peatland
 *                biome; "" when the world has no table (the pack's one biome source)
 * @param biomes  the peatland biomes found, for the log
 */
public record Peatland(double erosion, List<String> regions, List<String> biomes) {

    /** The key in {@code climate_mapping.json}. */
    public static final String KEY = "peatland";

    /** The region id of a world without a province table. */
    public static final String NO_TABLE = "";

    /** The value of {@code climate/wetland} for cold wetland (the Rust {@code WETLAND_COLD}). */
    public static final int COLD_WETLAND = 2;

    /** The layer the bake writes. */
    public static final String LAYER = "climate/wetland";

    /** Share of the cold wetland's climate points at which a region must answer with a peatland biome. */
    static final double SHARE = 0.5;

    public Peatland {
        regions = List.copyOf(regions);
        biomes = List.copyOf(biomes);
    }

    public boolean appliesTo(String region) {
        return regions.contains(region == null ? NO_TABLE : region);
    }

    /** A swamp and cold: the tags say so, whatever the biome's name or mod. */
    public static boolean isPeatland(Holder<Biome> b) {
        return b.is(tag("is_swamp")) && (b.is(tag("is_cold/overworld")) || b.is(tag("is_snowy")));
    }

    private static TagKey<Biome> tag(String path) {
        return TagKey.create(Registries.BIOME, ResourceLocation.fromNamespaceAndPath("c", path));
    }

    /** What the pack's biome source answers in one region: the peatland biome's id there, or null. */
    @FunctionalInterface
    public interface Probe {
        String peatlandAt(String region, Climate.TargetPoint point);
    }

    /**
     * Asks {@code probe} in each region at the climate of cold wetland: the baked temperature
     * coordinates the bake calls cold wetland (band 1, vanilla's cold band, up to {@code coldBelow}),
     * the wet humidity coordinates, inland continentalness, both signs of weirdness, and the
     * erosion a peatland would be given. All through the world's mapping, so they are the pack's values.
     *
     * @param regionNames the regions to judge ({@link #NO_TABLE} for the pack's one biome source)
     * @param coldBelow   the preset's {@code biomes.wetland_cold_below}
     * @return the verdict (a record with no regions when the pack has no peatland there), never null
     */
    public static Peatland decide(ClimateMapping m, OverworldBands ref, List<String> regionNames, double coldBelow, Probe probe) {
        double erosionRef = 0.5 * (ref.erosion()[6] + 1.0);
        double erosion = m.apply(ClimateMapping.EROSION, erosionRef);
        List<Double> temps = new ArrayList<>(), hums = new ArrayList<>(), conts = new ArrayList<>(), weirds = new ArrayList<>();
        for (double c : new double[] {1.1, 1.3, 1.5, 1.7, 1.9})
            if (c < coldBelow)
                temps.add(m.apply(ClimateMapping.TEMPERATURE, value(ref.temperature(), c)));
        if (temps.isEmpty())
            return new Peatland(erosion, List.of(), List.of());
        for (double c : new double[] {2.5, 3.0, 3.5, 4.0, 4.5})
            hums.add(m.apply(ClimateMapping.VEGETATION, value(ref.vegetation(), c)));
        for (double c : new double[] {4.5, 5.5, 6.5})
            conts.add(m.apply(ClimateMapping.CONTINENTS, value(ref.continents(), c)));
        for (double w : new double[] {-0.7, -0.4, -0.15, 0.15, 0.4, 0.7})
            weirds.add(m.apply(ClimateMapping.RIDGES, w));
        List<String> yes = new ArrayList<>();
        Set<String> found = new LinkedHashSet<>();
        for (String region : regionNames) {
            int hit = 0, all = 0;
            Set<String> here = new LinkedHashSet<>();
            for (double t : temps)
                for (double h : hums)
                    for (double c : conts)
                        for (double w : weirds) {
                            all++;
                            String b = probe.peatlandAt(region, Climate.target((float) t, (float) h, (float) c, (float) erosion, 0f, (float) w));
                            if (b != null) {
                                hit++;
                                here.add(b);
                            }
                        }
            if (hit >= SHARE * all) {
                yes.add(region);
                found.addAll(here);
            }
        }
        return new Peatland(erosion, yes, new ArrayList<>(found));
    }

    /** The value at band coordinate {@code c} over band {@code edges} (the Rust {@code band_value}). */
    static double value(double[] edges, double c) {
        int k = Math.min(edges.length - 2, (int) Math.floor(c));
        return edges[k] + (c - k) * (edges[k + 1] - edges[k]);
    }

    public JsonObject toJson() {
        JsonObject o = new JsonObject();
        o.addProperty("erosion", erosion);
        JsonArray r = new JsonArray();
        regions.forEach(r::add);
        o.add("regions", r);
        JsonArray b = new JsonArray();
        biomes.forEach(b::add);
        o.add("biomes", b);
        return o;
    }

    public static Peatland fromJson(JsonObject o) {
        List<String> r = new ArrayList<>(), b = new ArrayList<>();
        o.getAsJsonArray("regions").forEach(e -> r.add(e.getAsString()));
        if (o.has("biomes"))
            o.getAsJsonArray("biomes").forEach(e -> b.add(e.getAsString()));
        return new Peatland(o.get("erosion").getAsDouble(), r, b);
    }
}
