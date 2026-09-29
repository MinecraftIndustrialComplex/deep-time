// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.Reader;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;

/**
 * Band edges of the five biome inputs (plan §7.2): {@code OverworldBiomeBuilder}'s (1.21.1), or a
 * pack's own in its copy of {@code data/deeptime/deeptime/climate_mapping/default.json}. The
 * bake (Rust, {@code bake/biomes.rs}) places values in band coordinates against the reference
 * edges and records them in {@code notes.bake.climate.reference}; {@link ClimateMappingBuilder}
 * maps them onto a pack's edges.
 *
 * @param temperature 5 bands: frozen, cold, temperate, warm, hot
 * @param vegetation  5 humidity bands
 * @param continents  mushroom, deep ocean, ocean, coast, near-, mid-, far-inland
 * @param erosion     E0 … E6
 * @param weirdness   slice edges of |weirdness| on the rising branch: valley, low, mid, high, peak
 */
public record OverworldBands(double[] temperature, double[] vegetation, double[] continents, double[] erosion,
                             double[] weirdness) {

    /** The resource the jar ships (copied from crates/deeptime-core/data/climate_mapping/default.json). */
    public static final String RESOURCE = "data/deeptime/deeptime/climate_mapping/default.json";

    public OverworldBands {
        check("temperature", temperature, 6);
        check("vegetation", vegetation, 6);
        check("continents", continents, 8);
        check("erosion", erosion, 8);
        check("weirdness", weirdness, 6);
    }

    private static void check(String name, double[] edges, int n) {
        if (edges == null || edges.length != n)
            throw new IllegalArgumentException("bands." + name + ": expected " + n + " edges, got "
                    + (edges == null ? "none" : edges.length));
        for (int i = 1; i < n; i++)
            if (!(edges[i] > edges[i - 1]))
                throw new IllegalArgumentException("bands." + name + " must ascend: " + Arrays.toString(edges));
    }

    public static double[] array(JsonArray a) {
        double[] out = new double[a.size()];
        for (int i = 0; i < out.length; i++)
            out[i] = a.get(i).getAsDouble();
        return out;
    }

    static JsonArray json(double[] v) {
        JsonArray a = new JsonArray();
        for (double d : v)
            a.add(d);
        return a;
    }

    /** From a {@code bands} object. */
    public static OverworldBands fromJson(JsonObject o) {
        return new OverworldBands(array(o.getAsJsonArray("temperature")), array(o.getAsJsonArray("vegetation")),
                array(o.getAsJsonArray("continents")), array(o.getAsJsonArray("erosion")),
                array(o.getAsJsonArray("weirdness")));
    }

    public JsonObject toJson() {
        JsonObject o = new JsonObject();
        o.add("temperature", json(temperature));
        o.add("vegetation", json(vegetation));
        o.add("continents", json(continents));
        o.add("erosion", json(erosion));
        o.add("weirdness", json(weirdness));
        return o;
    }

    /** The jar's reference file, whole (bands and vanilla's land distributions). */
    public static JsonObject defaultDocument() {
        try (InputStream in = OverworldBands.class.getClassLoader().getResourceAsStream(RESOURCE)) {
            if (in == null)
                throw new IllegalStateException("the jar has no " + RESOURCE);
            return parse(in);
        } catch (IOException e) {
            throw new IllegalStateException("cannot read " + RESOURCE + ": " + e.getMessage(), e);
        }
    }

    static JsonObject parse(InputStream in) throws IOException {
        try (Reader r = new InputStreamReader(in, StandardCharsets.UTF_8)) {
            return JsonParser.parseReader(r).getAsJsonObject();
        }
    }

    /** The jar's reference bands ({@code OverworldBiomeBuilder}'s; a GameTest checks them against the class). */
    public static OverworldBands defaults() {
        return fromJson(defaultDocument().getAsJsonObject("bands"));
    }

    /** The band coordinate of {@code v} over {@code edges} (clamped to [0, n]). */
    public static double coord(double[] edges, double v) {
        int n = edges.length - 1;
        if (v <= edges[0])
            return 0.0;
        for (int k = 0; k < n; k++)
            if (v <= edges[k + 1])
                return k + (v - edges[k]) / (edges[k + 1] - edges[k]);
        return n;
    }

    /** The band index of {@code v} (0 … n − 1). */
    public static int band(double[] edges, double v) {
        return Math.min(edges.length - 2, (int) Math.floor(coord(edges, v)));
    }

    @Override
    public boolean equals(Object o) {
        return o instanceof OverworldBands b && Arrays.equals(temperature, b.temperature)
                && Arrays.equals(vegetation, b.vegetation) && Arrays.equals(continents, b.continents)
                && Arrays.equals(erosion, b.erosion) && Arrays.equals(weirdness, b.weirdness);
    }

    @Override
    public int hashCode() {
        return Arrays.hashCode(temperature) * 31 + Arrays.hashCode(erosion);
    }

    @Override
    public String toString() {
        return "OverworldBands" + toJson();
    }
}
