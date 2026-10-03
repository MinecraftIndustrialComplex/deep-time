// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.RegionSource;
import com.minecraftindustrialcomplex.deeptime.worldgen.RegionSpawnTarget;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.Holder;
import net.minecraft.core.QuartPos;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.biome.Biome;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Base64;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.TreeMap;

/**
 * TerraBlender's regions over a Deep Time level (P1-10): a grid of columns, each with the province
 * the baked {@code meta/province} puts there, the region that province was given, and the biome
 * the level's biome source places at the terrain surface. {@code /deeptime probe regions} writes
 * the grid for {@code tools/review/region_map.py}, which draws the provinces coloured by region.
 */
public final class RegionCensus {

    private RegionCensus() {}

    /** The level's province → region source, if it has one. */
    public static Optional<RegionSource> sourceOf(ServerLevel level) {
        var sampler = level.getChunkSource().randomState().sampler();
        return sampler.spawnTarget() instanceof RegionSpawnTarget t ? t.source() : Optional.empty();
    }

    /**
     * Samples every {@code step} blocks inside the pole edge (|z| ≤ C/4, {@link com.minecraftindustrialcomplex.deeptime.worlddata.Projection#poleZ}).
     *
     * @param mapFile where to write the whole grid (JSON, base64 arrays), or null
     */
    public static JsonObject run(ServerLevel level, int step, Path mapFile) throws IOException {
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g) || !(g.terrain() instanceof WorldDataTerrain t))
            throw new IllegalStateException(level.dimension().location() + " is not a Deep Time level on world data");
        Optional<RegionSource> src = sourceOf(level);
        var sampler = level.getChunkSource().randomState().sampler();
        int c = t.circumference(), sea = g.getSeaLevel();
        int zp = (int) com.minecraftindustrialcomplex.deeptime.worlddata.Projection.poleZ(c);   // the poles (the planet's projection)
        int w = c / step, h = 2 * zp / step;
        short[] province = new short[w * h];
        short[] region = new short[w * h];
        short[] biomeIdx = new short[w * h];
        Map<String, Integer> palette = new LinkedHashMap<>();
        Map<Holder<Biome>, Integer> byHolder = new HashMap<>();
        Map<String, Integer> landByNamespace = new TreeMap<>();
        // Per province: its biomes' namespaces (land columns), to see which provinces got mod biomes.
        Map<Integer, Map<String, Integer>> byProvince = new TreeMap<>();
        for (int j = 0; j < h; j++) {
            int z = -zp + step / 2 + j * step;
            for (int i = 0; i < w; i++) {
                int x = -c / 2 + step / 2 + i * step;
                int cell = j * w + i;
                double y = t.height(x, z);
                Holder<Biome> b = g.getBiomeSource().getNoiseBiome(QuartPos.fromBlock(x), QuartPos.fromBlock((int) Math.floor(y)),
                        QuartPos.fromBlock(z), sampler);
                Integer idx = byHolder.get(b);
                if (idx == null) {
                    idx = palette.size();
                    byHolder.put(b, idx);
                    palette.put(b.unwrapKey().map(key -> key.location().toString()).orElse("?" + idx), idx);
                }
                biomeIdx[cell] = (short) (int) idx;
                int p = src.map(s -> s.provinceAt(x, z)).orElse(-1);
                province[cell] = (short) p;
                region[cell] = (short) (int) src.map(s -> s.regionAt(x, z)).orElse(-1);
                if (y > sea) {
                    String ns = b.unwrapKey().map(key -> key.location().getNamespace()).orElse("?");
                    landByNamespace.merge(ns, 1, Integer::sum);
                    byProvince.computeIfAbsent(p, k -> new TreeMap<>()).merge(ns, 1, Integer::sum);
                }
            }
        }
        JsonObject o = new JsonObject();
        o.addProperty("level", level.dimension().location().toString());
        o.addProperty("step", step);
        o.addProperty("w", w);
        o.addProperty("h", h);
        o.addProperty("has_regions", src.isPresent());
        o.addProperty("provinces", src.map(RegionSource::provinceCount).orElse(0));
        JsonObject ns = new JsonObject();
        landByNamespace.forEach(ns::addProperty);
        o.add("land_biomes_by_namespace", ns);
        JsonObject prov = new JsonObject();
        byProvince.forEach((p, m) -> {
            JsonObject e = new JsonObject();
            e.addProperty("region", src.map(s -> s.regionOfProvince(p)).orElse(-1));
            JsonObject mm = new JsonObject();
            m.forEach(mm::addProperty);
            e.add("land_biomes_by_namespace", mm);
            prov.add(Integer.toString(p), e);
        });
        o.add("by_province", prov);
        if (mapFile != null) {
            JsonObject map = new JsonObject();
            map.addProperty("schema", 1);
            map.addProperty("level", level.dimension().location().toString());
            map.addProperty("circumference", c);
            map.addProperty("step", step);
            map.addProperty("w", w);
            map.addProperty("h", h);
            map.addProperty("sea_level", sea);
            JsonArray pal = new JsonArray();
            palette.keySet().forEach(pal::add);
            map.add("palette", pal);
            JsonArray names = new JsonArray();
            src.ifPresent(s -> s.regionNames().forEach(names::add));
            map.add("region_names", names);
            map.addProperty("province_i16le", b64(province));
            map.addProperty("region_i16le", b64(region));
            map.addProperty("biome_u16le", b64(biomeIdx));
            map.add("census", o.deepCopy());
            Files.createDirectories(mapFile.getParent());
            Files.writeString(mapFile, map.toString(), StandardCharsets.UTF_8);
            o.addProperty("file", mapFile.toString());
        }
        return o;
    }

    private static String b64(short[] v) {
        ByteBuffer bb = ByteBuffer.allocate(v.length * 2).order(ByteOrder.LITTLE_ENDIAN);
        for (short s : v)
            bb.putShort(s);
        return Base64.getEncoder().encodeToString(bb.array());
    }

    /** The provinces of {@code run}'s census whose land holds a biome of {@code namespace}. */
    public static List<Integer> provincesWith(JsonObject census, String namespace) {
        List<Integer> out = new ArrayList<>();
        census.getAsJsonObject("by_province").entrySet().forEach(e -> {
            if (e.getValue().getAsJsonObject().getAsJsonObject("land_biomes_by_namespace").has(namespace))
                out.add(Integer.parseInt(e.getKey()));
        });
        return out;
    }
}
