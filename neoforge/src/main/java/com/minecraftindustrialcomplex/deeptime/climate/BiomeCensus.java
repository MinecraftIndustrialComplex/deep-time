// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.worlddata.IceCover;
import com.minecraftindustrialcomplex.deeptime.worlddata.RasterSampler;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.Holder;
import net.minecraft.core.QuartPos;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.tags.TagKey;
import net.minecraft.world.level.biome.Biome;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Base64;
import java.util.Comparator;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import java.util.function.IntPredicate;

/**
 * Surface biomes against the Köppen map (P1-09): a grid of columns over a Deep Time level, each
 * with its Köppen class (the baked {@code climate/koppen}, nearest cell) and the biome the level's
 * biome source places at the terrain surface — what {@code /locate biome} and chunk generation
 * use, without loading a chunk. Per {@link Check}, the share of a Köppen family's columns whose
 * biome carries one of its NeoForge {@code c:} tags (Terralith and Biomes O' Plenty tag theirs the
 * same way). Used by {@code BiomeGameTests.climateConsistent} and {@code /deeptime probe biomes},
 * which can also write the whole grid for the review page. Its {@code volcanic} section is
 * {@link VolcanicCensus}'s: the volcanic biomes against the real volcanoes.
 */
public final class BiomeCensus {

    private BiomeCensus() {}

    static TagKey<Biome> c(String path) {
        return TagKey.create(Registries.BIOME, ResourceLocation.fromNamespaceAndPath("c", path));
    }

    /** Vanilla's cold | temperate temperature edge (the band coordinate 2): colder is the cold band or frozen. */
    static final double COLD_BAND_TOP = -0.15;

    /**
     * A Köppen family and the tags its biomes should carry. {@code notColderThan} leaves out the columns
     * whose baked temperature parameter is below it (null: none are left out): the cold deserts, which are
     * drawn as cold land on purpose (Ben, 2026-10-02) and have their own check, {@link Forbid}
     * {@code BWk:cold_not_sand}, instead of being counted against the desert tags.
     */
    public record Check(String name, String classes, IntPredicate koppen, List<TagKey<Biome>> tags, Double notColderThan) {
        public Check(String name, String classes, IntPredicate koppen, List<TagKey<Biome>> tags) {
            this(name, classes, koppen, tags, null);
        }
    }

    /** Köppen class indices (`koppen::CODES`): 0–3 A, 4–7 B, 8–16 C, 17–28 D, 29–30 E, 31 sea. */
    public static final List<Check> CHECKS = List.of(
            new Check("A", "Af Am Aw As", k -> k <= 3, List.of(c("is_hot"), c("is_jungle"), c("is_savanna"), c("is_wet"))),
            new Check("Af", "Af", k -> k == 0, List.of(c("is_jungle"), c("is_wet"))),
            new Check("BW", "BWh BWk (not the cold deserts)", k -> k == 4 || k == 5, List.of(c("is_desert"), c("is_badlands"), c("is_dry"), c("is_sandy")), COLD_BAND_TOP),
            new Check("BS", "BSh BSk", k -> k == 6 || k == 7, List.of(c("is_dry"), c("is_savanna"), c("is_plains"), c("is_sparse_vegetation"))),
            new Check("C", "C*", k -> k >= 8 && k <= 16, List.of(c("is_temperate"))),
            new Check("D", "D*", k -> k >= 17 && k <= 28, List.of(c("is_cold"), c("is_temperate"), c("is_taiga"), c("is_tree/coniferous"))),
            new Check("Dsub", "Dsc Dsd Dwc Dwd Dfc Dfd", k -> k == 19 || k == 20 || k == 23 || k == 24 || k == 27 || k == 28,
                    List.of(c("is_taiga"), c("is_tree/coniferous"), c("is_snowy"), c("is_cold"))),
            new Check("E", "ET EF", k -> k == 29 || k == 30, List.of(c("is_snowy"), c("is_icy"), c("is_cold"))),
            new Check("sea", "sea (below sea level)", k -> k == 31, List.of(c("is_ocean"), c("is_deep_ocean"), c("is_aquatic"))),
            // The narrow ones Ben asked to see: deserts in BW, rainforest in Af, taiga in Dfc, snow and ice in ET/EF.
            new Check("BW:desert", "BWh BWk (not the cold deserts)", k -> k == 4 || k == 5, List.of(c("is_desert"), c("is_badlands")), COLD_BAND_TOP),
            new Check("Af:jungle", "Af", k -> k == 0, List.of(c("is_jungle"))),
            new Check("Dfc:taiga", "Dfc", k -> k == 27, List.of(c("is_taiga"), c("is_tree/coniferous"))),
            new Check("E:snow", "ET EF", k -> k == 29 || k == 30, List.of(c("is_snowy"), c("is_icy"))));

    /**
     * A Köppen family and biomes it must not land on (the share of its columns that do is bounded);
     * {@code glaciated} limits the family to its columns under the ice cover ({@code true}), or to
     * those without ({@code false}); null is either. {@code colderThan} limits the family to the columns
     * whose baked temperature parameter is below it (null: any), for a class that is only wrong in part of
     * its range (a cold desert, not a hot one).
     */
    public record Forbid(String name, String classes, IntPredicate koppen, Boolean glaciated, Double colderThan,
                         java.util.Set<String> biomes) {
        public Forbid(String name, String classes, IntPredicate koppen, Boolean glaciated, java.util.Set<String> biomes) {
            this(name, classes, koppen, glaciated, null, biomes);
        }
    }

    /**
     * EF is ice cap where the glacier model puts ice and barren frozen ground elsewhere (Ben, 2026-09-29,
     * "Ice only where there's ice"): never Terralith's red, terracotta snowy badlands or its scarlet
     * (Nether-stone) mountains, and never a warm badlands.
     */
    public static final List<Forbid> FORBIDDEN = List.of(
            new Forbid("EF:no_red", "EF", k -> k == 30, null, java.util.Set.of("terralith:snowy_badlands", "terralith:scarlet_mountains",
                    "minecraft:badlands", "minecraft:eroded_badlands", "minecraft:wooded_badlands")),
            // Packed-ice canyons only where the ice model says ice (Ben, 2026-09-30, "Fix both").
            new Forbid("EF:no_chasm_on_barren", "EF without ice", k -> k == 30, false, java.util.Set.of("terralith:glacial_chasm")),
            // Cherry groves are positive-weirdness variants: ice caps are steered off them.
            new Forbid("EF:no_cherry", "EF", k -> k == 30, null, java.util.Set.of("terralith:snowy_cherry_grove", "minecraft:cherry_grove")),
            // Cold deserts (Ben, 2026-10-02): a BWk with a mean under 10 degC is in the frozen or cold band
            // (the placement's cold_desert and polar_ice ramps) and is not drawn as sand desert or badlands.
            new Forbid("BWk:cold_not_sand", "BWk colder than the cold band's top", k -> k == 5, null, COLD_BAND_TOP,
                    java.util.Set.of("minecraft:desert", "minecraft:badlands", "minecraft:eroded_badlands", "minecraft:wooded_badlands",
                            "terralith:ancient_sands", "terralith:desert_canyon", "terralith:desert_spires", "terralith:lush_desert",
                            "terralith:red_oasis", "terralith:desert_oasis", "terralith:white_mesa", "terralith:warped_mesa",
                            "terralith:savanna_badlands", "terralith:hot_shrubland")));

    public static final String[] KOPPEN = {"Af", "Am", "Aw", "As", "BWh", "BWk", "BSh", "BSk", "Csa", "Csb", "Csc", "Cwa", "Cwb",
            "Cwc", "Cfa", "Cfb", "Cfc", "Dsa", "Dsb", "Dsc", "Dsd", "Dwa", "Dwb", "Dwc", "Dwd", "Dfa", "Dfb", "Dfc", "Dfd", "ET", "EF",
            "sea"};

    /** Counts per check: matching columns, columns; the biome rules' census (null when no rule file is active). */
    public record Result(Map<String, int[]> checks, Map<String, int[]> forbidden, JsonObject json, RuleCensus rules) {
        public double share(String check) {
            int[] c = checks.get(check);
            return c == null || c[1] == 0 ? Double.NaN : c[0] / (double) c[1];
        }

        public int total(String check) {
            int[] c = checks.get(check);
            return c == null ? 0 : c[1];
        }

        /** The share of a {@link #FORBIDDEN} family's columns that landed on a forbidden biome. */
        public double forbiddenShare(String name) {
            int[] c = forbidden.get(name);
            return c == null || c[1] == 0 ? Double.NaN : c[0] / (double) c[1];
        }

        public int forbiddenTotal(String name) {
            int[] c = forbidden.get(name);
            return c == null ? 0 : c[1];
        }
    }

    /**
     * Samples every {@code step} blocks inside the pole edge (|z| ≤ R − {@code poleMargin}, R = C/2π the pole).
     *
     * @param mapFile write the whole grid (palette, biome indices, Köppen classes, heights) here as
     *                JSON with base64 arrays, or null
     */
    public static Result run(ServerLevel level, int step, int poleMargin, Path mapFile) throws IOException {
        return run(level, step, poleMargin, mapFile, java.util.Set.of());
    }

    /**
     * As {@link #run(ServerLevel, int, int, Path)}, leaving out of the tag checks the columns whose biome
     * belongs to one of {@code untaggedNamespaces}: a mod whose biomes do not carry the {@code c:} climate
     * and vegetation tags the checks count (Biomes O' Plenty's regions follow the geologic provinces since
     * P1-10, so which provinces hold its biomes decides these shares, not Deep Time's mapping). The
     * columns are counted under {@code skipped_untagged} in the census.
     */
    public static Result run(ServerLevel level, int step, int poleMargin, Path mapFile, java.util.Set<String> untaggedNamespaces)
            throws IOException {
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g) || !(g.terrain() instanceof WorldDataTerrain t))
            throw new IllegalStateException(level.dimension().location() + " is not a Deep Time level on world data");
        WorldData wd = t.data();
        // The class the biome inputs were placed by (L2, lapsed to each cell's height) when the bake
        // wrote it; the climate stage's L1 class otherwise. Against the L1 class, the high and low
        // ground of one L1 cell (a plateau's edge) count as one class, and a column a lapse rate
        // makes C or D under an L1 cell that is ET counted as a misplaced ET (2026-10-05).
        RasterSampler kp = wd.layer(wd.has("climate/koppen_l2") ? "climate/koppen_l2" : "climate/koppen");
        RasterSampler iceLayer = wd.has(IceCover.LAYER) ? wd.layer(IceCover.LAYER) : null;
        RasterSampler wetlandLayer = wd.has(Peatland.LAYER) ? wd.layer(Peatland.LAYER) : null;
        var sampler = level.getChunkSource().randomState().sampler();
        int c = t.circumference(), sea = g.getSeaLevel();
        int zp = (int) com.minecraftindustrialcomplex.deeptime.worlddata.Projection.poleZ(c);   // the poles (the planet's projection)
        int w = c / step, h = 2 * zp / step;
        short[] biomeIdx = new short[w * h];
        byte[] koppen = new byte[w * h];
        short[] height = new short[w * h];
        Map<String, Integer> palette = new LinkedHashMap<>();
        Map<Holder<Biome>, Integer> byHolder = new HashMap<>();
        List<Holder<Biome>> holders = new ArrayList<>();
        Map<String, int[]> checks = new LinkedHashMap<>();
        for (Check ch : CHECKS)
            checks.put(ch.name(), new int[2]);
        Map<String, int[]> forbidden = new LinkedHashMap<>();
        for (Forbid f : FORBIDDEN)
            forbidden.put(f.name(), new int[2]);
        Map<Integer, Map<Integer, Integer>> classBiomes = new HashMap<>();
        int skippedUntagged = 0, landColumns = 0, wetlands = 0, wetlandSwamp = 0;
        // Cold wetland (BIO-133, BIO-135): the bake's flagged boreal wetland, counted over every land column
        // (a mod's untagged biomes too: its bog is the point): on the pack's peatland biome, on taiga, on
        // another swamp (which it must not be), or elsewhere.
        int coldWet = 0, coldPeat = 0, coldTaiga = 0, coldSwamp = 0;
        Map<String, Integer> coldBiomes = new TreeMap<>();
        // The bake's wetlands (BIO-066): wet, flat, low, thawed land put in vanilla's swamp band E6 on
        // purpose (the relief's own erosion stays below it), so swamp there whatever the Köppen family.
        // The pack's peatland regions put cold wetland there too (Peatland), and it is set aside likewise.
        double e6 = OverworldBands.defaults().erosion()[6];
        // The biome rules (decisions log 2026-10-04): what decided each land column, with the level's bound rules.
        VolcanicBiomes.Level rules = sampler.spawnTarget() instanceof com.minecraftindustrialcomplex.deeptime.worldgen.RegionSpawnTarget m
                ? m.volcanic() : null;
        RasterSampler wetFlag = wd.has(BiomeRules.WETLAND_LAYER) ? wd.layer(BiomeRules.WETLAND_LAYER) : null;
        RuleCensus ruleCensus = null;
        for (int j = 0; j < h; j++) {
            int z = -zp + step / 2 + j * step;
            for (int i = 0; i < w; i++) {
                int x = -c / 2 + step / 2 + i * step;
                int cell = j * w + i;
                double kv = kp.nearest(x, z);
                int k = Double.isNaN(kv) ? 31 : (int) Math.round(kv);
                if (k < 0 || k > 31)
                    k = 31;
                double y = t.height(x, z);
                boolean land = k != 31 && y > sea;
                Holder<Biome> b = g.getBiomeSource().getNoiseBiome(QuartPos.fromBlock(x), QuartPos.fromBlock((int) Math.floor(y)),
                        QuartPos.fromBlock(z), sampler);
                String decision = VolcanicBiomes.lastDecision();
                Integer idx = byHolder.get(b);
                if (idx == null) {
                    idx = holders.size();
                    holders.add(b);
                    byHolder.put(b, idx);
                    palette.put(b.unwrapKey().map(key -> key.location().toString()).orElse("?" + idx), idx);
                }
                biomeIdx[cell] = (short) (int) idx;
                koppen[cell] = (byte) k;
                height[cell] = (short) Math.max(Short.MIN_VALUE, Math.min(Short.MAX_VALUE, Math.round(y)));
                if (Math.abs(z) > zp - poleMargin)
                    continue;
                if (land && wetlandLayer != null && Math.round(wetlandLayer.nearest(x, z)) == Peatland.COLD_WETLAND) {
                    coldWet++;
                    coldBiomes.merge(nameOf(b), 1, Integer::sum);
                    if (PeatlandProbe.isPeatland(b))
                        coldPeat++;
                    else if (b.is(c("is_swamp")))
                        coldSwamp++;
                    else if (b.is(c("is_taiga")) || b.is(c("is_tree/coniferous")))
                        coldTaiga++;
                }
                boolean untagged = !untaggedNamespaces.isEmpty()
                        && untaggedNamespaces.contains(b.unwrapKey().map(key -> key.location().getNamespace()).orElse("?"));
                if (untagged && (k == 31 ? y < sea - 4 : land))
                    skippedUntagged++;
                boolean wetland = land && !untagged && wd.raster(ClimateMapping.EROSION, x, z) >= e6;
                if (land && !untagged && rules != null) {
                    if (ruleCensus == null)
                        ruleCensus = new RuleCensus(rules.rules());
                    boolean ruleColdWet = wetFlag != null ? wetFlag.nearest(x, z) >= 1.5
                            : wetland && BiomeRules.COLD_WETLAND_CLASSES.contains(KOPPEN[k]);
                    ruleCensus.add(b, decision, KOPPEN[k], ruleColdWet);
                }
                if (land && !untagged) {
                    landColumns++;
                    if (wetland) {
                        wetlands++;
                        if (b.is(c("is_swamp")))
                            wetlandSwamp++;
                    }
                }
                if (!untagged && (k == 31 ? y < sea - 4 : land)) {
                    for (Check ch : CHECKS) {
                        if (wetland)
                            break;   // counted apart (above), not against the family's tags
                        if (!ch.koppen().test(k))
                            continue;
                        if (ch.notColderThan() != null && t.raster("climate/temperature", x, z) < ch.notColderThan())
                            continue;
                        int[] n = checks.get(ch.name());
                        n[1]++;
                        for (TagKey<Biome> tag : ch.tags())
                            if (b.is(tag)) {
                                n[0]++;
                                break;
                            }
                    }
                    for (Forbid f : FORBIDDEN) {
                        if (!f.koppen().test(k))
                            continue;
                        if (f.glaciated() != null && f.glaciated() != (iceLayer != null && iceLayer.nearest(x, z) > 0))
                            continue;
                        if (f.colderThan() != null && !(t.raster("climate/temperature", x, z) < f.colderThan()))
                            continue;
                        int[] n = forbidden.get(f.name());
                        n[1]++;
                        if (f.biomes().contains(nameOf(b)))
                            n[0]++;
                    }
                    classBiomes.computeIfAbsent(k, kk -> new HashMap<>()).merge(idx, 1, Integer::sum);
                }
            }
        }
        JsonObject o = new JsonObject();
        o.addProperty("level", level.dimension().location().toString());
        o.addProperty("step", step);
        o.addProperty("w", w);
        o.addProperty("h", h);
        o.addProperty("mapping", t.mapping().isIdentity() ? "identity" : t.mapping().summary());
        JsonObject cj = new JsonObject();
        for (Check ch : CHECKS) {
            int[] n = checks.get(ch.name());
            JsonObject e = new JsonObject();
            e.addProperty("classes", ch.classes());
            e.addProperty("columns", n[1]);
            e.addProperty("matching", n[0]);
            e.addProperty("share", n[1] == 0 ? null : Math.round(n[0] * 1000.0 / n[1]) / 1000.0);
            JsonArray tags = new JsonArray();
            ch.tags().forEach(tg -> tags.add("#" + tg.location()));
            e.add("tags", tags);
            cj.add(ch.name(), e);
        }
        o.add("checks", cj);
        JsonObject fj = new JsonObject();
        for (Forbid f : FORBIDDEN) {
            int[] n = forbidden.get(f.name());
            JsonObject e = new JsonObject();
            e.addProperty("classes", f.classes());
            e.addProperty("columns", n[1]);
            e.addProperty("forbidden", n[0]);
            e.addProperty("share", n[1] == 0 ? null : Math.round(n[0] * 1000.0 / n[1]) / 1000.0);
            JsonArray names = new JsonArray();
            f.biomes().stream().sorted().forEach(names::add);
            e.add("biomes", names);
            fj.add(f.name(), e);
        }
        o.add("forbidden", fj);
        // The five commonest biomes of each Köppen class present.
        JsonObject top = new JsonObject();
        for (int k = 0; k < 32; k++) {
            Map<Integer, Integer> m = classBiomes.get(k);
            if (m == null)
                continue;
            int total = m.values().stream().mapToInt(Integer::intValue).sum();
            JsonObject e = new JsonObject();
            e.addProperty("columns", total);
            JsonArray list = new JsonArray();
            m.entrySet().stream().sorted(Map.Entry.<Integer, Integer>comparingByValue(Comparator.reverseOrder())).limit(5)
                    .forEach(en -> list.add(nameOf(holders.get(en.getKey())) + " " + Math.round(en.getValue() * 100.0 / total) + "%"));
            e.add("top", list);
            top.add(KOPPEN[k], e);
        }
        o.add("by_koppen", top);
        o.addProperty("biomes", palette.size());
        o.addProperty("skipped_untagged", skippedUntagged);
        JsonObject wj = new JsonObject();
        wj.addProperty("land_columns", landColumns);
        wj.addProperty("wetland_columns", wetlands);
        wj.addProperty("on_swamp", wetlandSwamp);
        o.add("wetlands_set_aside", wj);
        JsonObject cw = new JsonObject();
        cw.addProperty("layer", wetlandLayer != null);
        cw.addProperty("columns", coldWet);
        cw.addProperty("on_peatland", coldPeat);
        cw.addProperty("on_taiga", coldTaiga);
        cw.addProperty("on_swamp", coldSwamp);
        JsonObject cb = new JsonObject();
        coldBiomes.entrySet().stream().sorted(Map.Entry.<String, Integer>comparingByValue(Comparator.reverseOrder())).limit(8)
                .forEach(e -> cb.addProperty(e.getKey(), e.getValue()));
        cw.add("top_biomes", cb);
        o.add("cold_wetland", cw);
        // The volcanic biomes against the real volcanoes, now and as the plain climate search placed them
        // (decisions log 2026-10-03).
        o.add("volcanic", VolcanicCensus.run(level, step, poleMargin).json());
        if (ruleCensus != null && !rules.rules().isEmpty())
            o.add("rules", ruleCensus.json());
        if (mapFile != null) {
            JsonObject map = new JsonObject();
            map.addProperty("schema", 1);
            map.addProperty("level", level.dimension().location().toString());
            map.addProperty("circumference", c);
            map.addProperty("step", step);
            map.addProperty("x0", -c / 2 + step / 2);
            map.addProperty("z0", -zp + step / 2);
            map.addProperty("w", w);
            map.addProperty("h", h);
            map.addProperty("sea_level", sea);
            JsonArray pal = new JsonArray();
            for (Holder<Biome> b : holders) {
                JsonObject e = new JsonObject();
                e.addProperty("id", nameOf(b));
                JsonArray tags = new JsonArray();
                b.tags().filter(tg -> tg.location().getNamespace().equals("c")).forEach(tg -> tags.add(tg.location().getPath()));
                e.add("c_tags", tags);
                pal.add(e);
            }
            map.add("palette", pal);
            JsonArray codes = new JsonArray();
            for (String s : KOPPEN)
                codes.add(s);
            map.add("koppen_codes", codes);
            map.addProperty("biome_u16le", b64(biomeIdx));
            map.addProperty("koppen_u8", Base64.getEncoder().encodeToString(koppen));
            map.addProperty("height_i16le", b64(height));
            map.add("census", o.deepCopy());
            Files.createDirectories(mapFile.getParent());
            Files.writeString(mapFile, map.toString(), StandardCharsets.UTF_8);
            o.addProperty("file", mapFile.toString());
        }
        return new Result(checks, forbidden, o, ruleCensus != null && !rules.rules().isEmpty() ? ruleCensus : null);
    }

    static String nameOf(Holder<Biome> b) {
        return b.unwrapKey().map(key -> key.location().toString()).orElse("?");
    }

    private static String b64(short[] v) {
        ByteBuffer bb = ByteBuffer.allocate(v.length * 2).order(ByteOrder.LITTLE_ENDIAN);
        for (short s : v)
            bb.putShort(s);
        return Base64.getEncoder().encodeToString(bb.array());
    }
}
