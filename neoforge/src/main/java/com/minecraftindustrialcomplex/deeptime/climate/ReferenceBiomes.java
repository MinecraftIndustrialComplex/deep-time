// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.core.Holder;
import net.minecraft.core.QuartPos;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.Registries;
import net.minecraft.tags.BiomeTags;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.Climate;
import net.minecraft.world.level.biome.MultiNoiseBiomeSourceParameterList;
import net.minecraft.world.level.biome.MultiNoiseBiomeSourceParameterLists;
import net.minecraft.world.level.levelgen.NoiseGeneratorSettings;
import net.minecraft.world.level.levelgen.RandomState;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Comparator;
import java.util.Map;
import java.util.TreeMap;
import java.util.stream.IntStream;

/**
 * The surface biome shares of a normal world of the installed pack (ice spikes report, 2026-10-20): the registry's
 * {@code minecraft:overworld} noise settings (the pack's router: vanilla's, or Tectonic's under MIC) sampled with the
 * registry's {@code minecraft:overworld} multi-noise parameter list (vanilla's, or Terralith's) on a jittered grid over
 * several seeds, at depth 0 (the surface slot every surface biome holds). It is the reference the planet's own census
 * ({@link BiomeCensus}) is read against: what a player of this pack sees in a world without Deep Time. No chunk is
 * loaded and no level is touched. Land is every column whose biome is neither an ocean nor a river
 * ({@code #minecraft:is_ocean}, {@code #minecraft:is_river}). Besides each biome's count it keeps the land's temperature
 * and humidity band cross table (vanilla's band edges, {@link OverworldBands#defaults()}), the frozen and dry corner's
 * biomes (temperature band 0 and humidity band 0, where vanilla draws ice spikes) and the weirdness sign there, and it
 * writes the parameter list itself so the offline emulation ({@code tools/review/ice_spikes.py}) can run the same
 * climate search on a planet's baked inputs.
 */
public final class ReferenceBiomes {

    private ReferenceBiomes() {}

    /** Per-thread tallies for one grid row block. */
    private static final class Tally {
        final Map<String, long[]> land = new TreeMap<>();
        final Map<String, long[]> corner = new TreeMap<>();
        final long[][] cross = new long[5][5];
        final long[] erosion = new long[7];
        long samples, landN, cornerN, cornerPositive, positive;

        void add(Tally o) {
            o.land.forEach((k, v) -> land.computeIfAbsent(k, q -> new long[1])[0] += v[0]);
            o.corner.forEach((k, v) -> corner.computeIfAbsent(k, q -> new long[1])[0] += v[0]);
            for (int i = 0; i < 5; i++)
                for (int j = 0; j < 5; j++)
                    cross[i][j] += o.cross[i][j];
            for (int i = 0; i < 7; i++)
                erosion[i] += o.erosion[i];
            samples += o.samples;
            landN += o.landN;
            cornerN += o.cornerN;
            cornerPositive += o.cornerPositive;
            positive += o.positive;
        }
    }

    static int band(double[] edges, double v) {
        int k = 0;
        for (int i = 1; i < edges.length - 1; i++)
            if (v >= edges[i])
                k = i;
        return k;
    }

    private static long mix(long z) {
        z = (z ^ (z >>> 30)) * 0xBF58476D1CE4E5B9L;
        z = (z ^ (z >>> 27)) * 0x94D049BB133111EBL;
        return z ^ (z >>> 31);
    }

    /**
     * Samples a {@code 2 radius} square every {@code step} blocks (jittered within each cell) for each of {@code seeds}
     * seeds (0, 1, ...), and writes the result and the parameter list next to {@code out}.
     */
    public static JsonObject run(RegistryAccess access, int radius, int step, int seeds, Path out) throws IOException {
        var settings = access.registryOrThrow(Registries.NOISE_SETTINGS).getHolder(NoiseGeneratorSettings.OVERWORLD)
                .orElseThrow(() -> new IllegalStateException("no minecraft:overworld noise settings"));
        MultiNoiseBiomeSourceParameterList preset = access.registryOrThrow(Registries.MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST)
                .getOrThrow(MultiNoiseBiomeSourceParameterLists.OVERWORLD);
        Climate.ParameterList<Holder<Biome>> list = preset.parameters();
        OverworldBands bands = OverworldBands.defaults();
        int n = 2 * radius / step;
        Tally all = new Tally();
        for (int seed = 0; seed < seeds; seed++) {
            RandomState rs = RandomState.create(settings.value(), access.lookupOrThrow(Registries.NOISE), seed);
            Climate.Sampler sampler = rs.sampler();
            final long s = seed;
            Tally t = IntStream.range(0, n).parallel().mapToObj(j -> {
                Tally row = new Tally();
                for (int i = 0; i < n; i++) {
                    long h = mix(((long) j << 32) ^ i ^ (s * 0x9E3779B97F4A7C15L));
                    int x = (int) Math.floor(-radius + (i + (h & 0xFFFF) / 65536.0) * step);
                    int z = (int) Math.floor(-radius + (j + ((h >>> 16) & 0xFFFF) / 65536.0) * step);
                    Climate.TargetPoint p = sampler.sample(QuartPos.fromBlock(x), 0, QuartPos.fromBlock(z));
                    // Depth 0: the surface slot (vanilla's surface biomes hold depth 0 and 1, the cave biomes 0.2 to 0.9).
                    Climate.TargetPoint surface = new Climate.TargetPoint(p.temperature(), p.humidity(), p.continentalness(),
                            p.erosion(), 0L, p.weirdness());
                    Holder<Biome> b = list.findValue(surface);
                    row.samples++;
                    if (b.is(BiomeTags.IS_OCEAN) || b.is(BiomeTags.IS_RIVER))
                        continue;
                    String id = BiomeCensus.nameOf(b);
                    row.landN++;
                    row.land.computeIfAbsent(id, q -> new long[1])[0]++;
                    int tb = band(bands.temperature(), Climate.unquantizeCoord(p.temperature()));
                    int hb = band(bands.vegetation(), Climate.unquantizeCoord(p.humidity()));
                    row.cross[tb][hb]++;
                    row.erosion[band(bands.erosion(), Climate.unquantizeCoord(p.erosion()))]++;
                    boolean positive = p.weirdness() >= 0;
                    if (positive)
                        row.positive++;
                    if (tb == 0 && hb == 0) {
                        row.cornerN++;
                        if (positive)
                            row.cornerPositive++;
                        row.corner.computeIfAbsent(id, q -> new long[1])[0]++;
                    }
                }
                return row;
            }).reduce(new Tally(), (a, b) -> {
                a.add(b);
                return a;
            });
            all.add(t);
        }
        JsonObject o = new JsonObject();
        o.addProperty("noise_settings", NoiseGeneratorSettings.OVERWORLD.location().toString());
        o.addProperty("parameter_list", MultiNoiseBiomeSourceParameterLists.OVERWORLD.location().toString());
        o.addProperty("radius", radius);
        o.addProperty("step", step);
        o.addProperty("seeds", seeds);
        o.addProperty("samples", all.samples);
        o.addProperty("land", all.landN);
        o.addProperty("land_rule", "biome not #minecraft:is_ocean and not #minecraft:is_river; depth 0");
        o.addProperty("land_weirdness_positive", all.positive);
        o.add("land_biomes", counts(all.land));
        JsonArray cross = new JsonArray();
        for (long[] r : all.cross) {
            JsonArray a = new JsonArray();
            for (long v : r)
                a.add(v);
            cross.add(a);
        }
        o.add("land_temperature_x_humidity", cross);
        JsonArray er = new JsonArray();
        for (long v : all.erosion)
            er.add(v);
        o.add("land_erosion_bands", er);
        JsonObject corner = new JsonObject();
        corner.addProperty("land", all.cornerN);
        corner.addProperty("weirdness_positive", all.cornerPositive);
        corner.add("biomes", counts(all.corner));
        o.add("frozen_dry_corner", corner);
        Files.createDirectories(out.getParent());
        Files.writeString(out, o.toString(), StandardCharsets.UTF_8);
        Path pl = out.resolveSibling(out.getFileName().toString().replace(".json", "") + "-parameters.json");
        Files.writeString(pl, parameters(list).toString(), StandardCharsets.UTF_8);
        o.addProperty("file", out.toString());
        o.addProperty("parameters_file", pl.toString());
        return o;
    }

    private static JsonObject counts(Map<String, long[]> m) {
        JsonObject o = new JsonObject();
        m.entrySet().stream().sorted(Comparator.comparingLong((Map.Entry<String, long[]> e) -> -e.getValue()[0]))
                .forEach(e -> o.addProperty(e.getKey(), e.getValue()[0]));
        return o;
    }

    /** The parameter list in the data-pack layout ({@code {"biomes": [{"biome", "parameters"}]}}), vanilla's units. */
    static JsonObject parameters(Climate.ParameterList<Holder<Biome>> list) {
        JsonArray entries = new JsonArray();
        for (var pair : list.values()) {
            Climate.ParameterPoint p = pair.getFirst();
            JsonObject params = new JsonObject();
            params.add("temperature", range(p.temperature()));
            params.add("humidity", range(p.humidity()));
            params.add("continentalness", range(p.continentalness()));
            params.add("erosion", range(p.erosion()));
            params.add("weirdness", range(p.weirdness()));
            params.add("depth", range(p.depth()));
            params.addProperty("offset", Climate.unquantizeCoord(p.offset()));
            JsonObject e = new JsonObject();
            e.addProperty("biome", BiomeCensus.nameOf(pair.getSecond()));
            e.add("parameters", params);
            entries.add(e);
        }
        JsonObject o = new JsonObject();
        o.add("biomes", entries);
        return o;
    }

    private static JsonArray range(Climate.Parameter p) {
        JsonArray a = new JsonArray();
        a.add(Climate.unquantizeCoord(p.min()));
        a.add(Climate.unquantizeCoord(p.max()));
        return a;
    }
}
