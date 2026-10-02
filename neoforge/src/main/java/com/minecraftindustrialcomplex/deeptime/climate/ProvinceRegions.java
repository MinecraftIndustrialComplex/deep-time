// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Comparator;
import java.util.List;

/**
 * Which TerraBlender region each of a world's geologic provinces belongs to (plan §7.3, P1-10).
 * The bake cuts the planet into provinces ({@code meta/province}: contiguous areas of one USGS-style
 * class); the game gives every province one region, so a Biomes O' Plenty region covers whole
 * provinces rather than TerraBlender's own tessellation.
 *
 * <p><b>The assignment is stratified, not independent.</b> Each province gets a pseudo-random rank
 * from a hash of its id and the world's salt; the rank-ordered provinces are then dealt out to the
 * regions in proportion to their weights (province at rank r of n takes the region whose cumulative
 * weight share contains (r + ½) / n). Independent hashing would hand out the regions in the weights'
 * proportion only on average: with the dozens of provinces of a small planet a light region could
 * miss every one. The result is deterministic in (province count, regions, weights, salt), and is
 * stored by region <em>name</em> in the world's {@code climate_mapping.json}, so installing or
 * removing a biome mod later does not move the regions of provinces that were already given one.
 * Names that are no longer registered fall back to the default region at run time.
 *
 * @param names      the region ids (those of the catalogue with a positive weight, in its order)
 * @param byProvince for each province id, the index into {@code names}
 */
public record ProvinceRegions(List<String> names, int[] byProvince) {

    /** A registered region and its TerraBlender weight. */
    public record Entry(String name, int weight) {}

    /** The JSON key in {@code climate_mapping.json}. */
    public static final String KEY = "regions";

    public ProvinceRegions {
        names = List.copyOf(names);
        for (int i : byProvince)
            if (i < 0 || i >= names.size())
                throw new IllegalArgumentException("region index " + i + " outside " + names.size() + " names");
    }

    public int provinces() {
        return byProvince.length;
    }

    /** The region id of a province, or null for an id the table does not cover. */
    public String regionOf(int province) {
        return province < 0 || province >= byProvince.length ? null : names.get(byProvince[province]);
    }

    /** The share of provinces per region id, for the log and the probe. */
    public java.util.Map<String, Integer> counts() {
        java.util.Map<String, Integer> m = new java.util.TreeMap<>();
        for (int i : byProvince)
            m.merge(names.get(i), 1, Integer::sum);
        return m;
    }

    private static long mix(long z) {
        z = (z ^ (z >>> 30)) * 0xBF58476D1CE4E5B9L;
        z = (z ^ (z >>> 27)) * 0x94D049BB133111EBL;
        return z ^ (z >>> 31);
    }

    /**
     * Deals {@code provinces} provinces out to {@code regions} (weights ≤ 0 take none).
     *
     * @throws IllegalArgumentException when no region has a positive weight
     */
    public static ProvinceRegions assign(int provinces, List<Entry> regions, long salt) {
        List<Entry> live = new ArrayList<>();
        long total = 0;
        for (Entry e : regions)
            if (e.weight() > 0) {
                live.add(e);
                total += e.weight();
            }
        if (live.isEmpty())
            throw new IllegalArgumentException("no region has a positive weight");
        Integer[] order = new Integer[provinces];
        long[] h = new long[provinces];
        for (int p = 0; p < provinces; p++) {
            order[p] = p;
            h[p] = mix(salt ^ mix(p + 0x9E3779B97F4A7C15L));
        }
        Arrays.sort(order, Comparator.<Integer>comparingLong(p -> h[p]).thenComparingInt(p -> p));
        int[] by = new int[provinces];
        for (int rank = 0; rank < provinces; rank++) {
            double u = (rank + 0.5) / provinces * total;
            int k = 0;
            double cum = live.get(0).weight();
            while (u >= cum && k < live.size() - 1)
                cum += live.get(++k).weight();
            by[order[rank]] = k;
        }
        return new ProvinceRegions(live.stream().map(Entry::name).toList(), by);
    }

    public JsonObject toJson() {
        JsonObject o = new JsonObject();
        JsonArray n = new JsonArray();
        names.forEach(n::add);
        o.add("names", n);
        JsonArray b = new JsonArray();
        for (int i : byProvince)
            b.add(i);
        o.add("by_province", b);
        return o;
    }

    public static ProvinceRegions fromJson(JsonObject o) {
        List<String> names = new ArrayList<>();
        o.getAsJsonArray("names").forEach(e -> names.add(e.getAsString()));
        JsonArray b = o.getAsJsonArray("by_province");
        int[] by = new int[b.size()];
        for (int i = 0; i < by.length; i++)
            by[i] = b.get(i).getAsInt();
        return new ProvinceRegions(names, by);
    }
}
