// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.worlddata.ClimateLayers;
import com.minecraftindustrialcomplex.deeptime.worlddata.IceCover;
import com.minecraftindustrialcomplex.deeptime.worlddata.RasterSampler;
import com.minecraftindustrialcomplex.deeptime.worlddata.Water;
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
     * Biomes their mod leaves without a climate or vegetation {@code c:} tag, read by {@link #CHECKS} as carrying
     * the tags named here (the game's tags are not changed). Expanded Ecosphere 3.4.7 tags {@code wythers:ancient_taiga}
     * only {@code minecraft:is_overworld} and {@code wythers:is_wythers_surface} (its jar's data/{c,minecraft,wythers}/tags,
     * read 2026-10-18), while its other taigas ({@code old_growth_taiga_crags}, the boreal forests) carry {@code c:is_taiga}
     * and {@code c:is_tree/coniferous}, and the biome-mods survey puts it in the Dc/Dd row
     * (docs/research/notes/biome-mods-survey.md). On master 9091c163's test planet with -PwithEe its columns were every
     * miss of the D family (Dfb 5, Dfc 5: D 0.870 under its floor 0.88), of Dsub and of Dfc:taiga.
     * <p>
     * A biome its mod tags against its own placement is read the same way. Expanded Ecosphere 3.4.7 places
     * {@code wythers:subtropical_forest} (27 entries) and {@code wythers:subtropical_forest_edge} (27) at temperature 0.15
     * to 0.2 of its overworld parameter list (resources/ee_default/data/minecraft/dimension/overworld.json, read
     * 2026-10-18), inside vanilla's temperate band (-0.15 to 0.2, BIO-021), in the cell where it also places
     * {@code minecraft:forest}; it tags both {@code c:is_hot/overworld} and neither {@code c:is_temperate}. A broken
     * mapping that sent C ground to the warm or hot band would not reach them there. On master 3efc5ff8's test planet
     * with -PwithEe the subtropical forest was 49 of the C family's 61 misses (C 103 of 164, 0.628 under its floor 0.63).
     * The columns matched only through this table are counted in each check's {@code via_tag_gaps}.
     */
    static final Map<String, List<TagKey<Biome>>> TAG_GAPS = Map.of(
            "wythers:ancient_taiga", List.of(c("is_taiga"), c("is_tree/coniferous")),
            "wythers:subtropical_forest", List.of(c("is_temperate")),
            "wythers:subtropical_forest_edge", List.of(c("is_temperate")));

    /**
     * A Köppen family and biomes it must not land on (the share of its columns that do is bounded);
     * {@code glaciated} limits the family to its columns under the ice cover ({@code true}), or to
     * those without ({@code false}); null is either. {@code colderThan} limits the family to the columns
     * whose baked temperature parameter is below it (null: any), for a class that is only wrong in part of
     * its range (a cold desert, not a hot one). {@code steep} limits it to the columns whose baked erosion is
     * below vanilla's third erosion band ({@code true}: bands E0 and E1, under -0.375) or in it and above
     * ({@code false}: the flats); null is any.
     */
    public record Forbid(String name, String classes, IntPredicate koppen, Boolean glaciated, Double colderThan,
                         Boolean steep, java.util.Set<String> biomes) {
        public Forbid(String name, String classes, IntPredicate koppen, Boolean glaciated, java.util.Set<String> biomes) {
            this(name, classes, koppen, glaciated, null, null, biomes);
        }

        public Forbid(String name, String classes, IntPredicate koppen, Boolean glaciated, Double colderThan,
                      java.util.Set<String> biomes) {
            this(name, classes, koppen, glaciated, colderThan, null, biomes);
        }
    }

    /** The {@link #FORBIDDEN} entries that bound the ice cap's ice spikes, on its steep ground and on its flats. */
    public static final String EF_ICE_SPIKES_STEEP = "EF:few_ice_spikes_on_steep", EF_ICE_SPIKES_FLATS = "EF:few_ice_spikes_on_flats";

    /**
     * EF is ice cap where the glacier model puts ice and barren frozen ground elsewhere (Ben, 2026-09-29,
     * "Ice only where there's ice"): never Terralith's red, terracotta snowy badlands or its scarlet
     * (Nether-stone) mountains, and never a warm badlands; and ice spikes on no more than a share of the ice cap.
     */
    public static final List<Forbid> FORBIDDEN = List.of(
            new Forbid("EF:no_red", "EF", k -> k == 30, null, java.util.Set.of("terralith:snowy_badlands", "terralith:scarlet_mountains",
                    "minecraft:badlands", "minecraft:eroded_badlands", "minecraft:wooded_badlands")),
            // Packed-ice canyons only where the ice model says ice (Ben, 2026-09-30, "Fix both").
            new Forbid("EF:no_chasm_on_barren", "EF without ice", k -> k == 30, false, java.util.Set.of("terralith:glacial_chasm")),
            // Cherry groves are positive-weirdness variants: ice caps are steered off them.
            new Forbid("EF:no_cherry", "EF", k -> k == 30, null, java.util.Set.of("terralith:snowy_cherry_grove", "minecraft:cherry_grove")),
            // Ice spikes on a share of the ice cap only (Ben, 2026-09-30, "Bring ice spikes back"; the ice spikes report,
            // 2026-10-20, BIO-161: near-inland, Terralith drew them on the ice cap's flats on both signs of weirdness, 65 to
            // 71 % of them, and vanilla's builder on a fifth of its steep ground). Bounded shares, not bans: the ceilings are
            // BiomeGameTests.forbiddenCeiling's (ledger: BIO-162).
            new Forbid(EF_ICE_SPIKES_STEEP, "EF under ice, erosion E0 and E1", k -> k == 30, true, null, true,
                    java.util.Set.of("minecraft:ice_spikes")),
            new Forbid(EF_ICE_SPIKES_FLATS, "EF under ice, erosion E2 and up", k -> k == 30, true, null, false,
                    java.util.Set.of("minecraft:ice_spikes")),
            // Cold deserts (Ben, 2026-10-02): a BWk with a mean under 10 degC is in the frozen or cold band
            // (the placement's cold_desert and polar_ice ramps) and is not drawn as sand desert or badlands.
            new Forbid("BWk:cold_not_sand", "BWk colder than the cold band's top", k -> k == 5, null, COLD_BAND_TOP,
                    java.util.Set.of("minecraft:desert", "minecraft:badlands", "minecraft:eroded_badlands", "minecraft:wooded_badlands",
                            "terralith:ancient_sands", "terralith:desert_canyon", "terralith:desert_spires", "terralith:lush_desert",
                            "terralith:red_oasis", "terralith:desert_oasis", "terralith:white_mesa", "terralith:warped_mesa",
                            "terralith:savanna_badlands", "terralith:hot_shrubland")));

    /**
     * A pack placement table's check (decisions log 2026-10-04, "Deep Time-side fixes"): the share of a Köppen family's
     * land columns (wetlands set aside, as for {@link #CHECKS}) whose biome is one of {@code biomes}, at least
     * {@code bound} (or at most, with {@code atMost}). It counts only where the level's biome source can place
     * {@code requires} (the pack is installed); elsewhere it is reported inactive. Columns in a volcanic biome (the
     * volcanic layer's, not the table's) are left out, and so are those whose pack-side continentalness is below
     * {@code minContinents} or whose erosion is below {@code minErosion} (null: no limit): mountains (vanilla's slopes and
     * peaks take any class's high relief) and the coast strip are not the table's to place. The coast strip's columns (on a
     * beach, a stony shore or a sea biome, as the families count them apart) are left out of every check since
     * 2026-10-17: on P1-26's test planet 68 of Cs's 373 columns were coast and none could match. The columns under the
     * ice cover are left out since 2026-10-18 (the bake holds glaciated land in the frozen band whatever its class,
     * BIO-035, and the ice pass covers it): on the CLM-203 test planet 45 of Cs's 137 columns, 2 on the table's biomes.
     * The class border's columns are left out since 2026-10-21 ({@link #classBorder}): the game reads the biome inputs
     * bilinear, so between an Am cell and a warm sea cell (temperature band above 4, {@code thermal_sea}) the blend
     * crosses vanilla's T4 line, where every middle biome is desert, and Terralith puts a desert oasis there. On master
     * f43d1e1a's MIC test planet 19 of Am's 52 columns lay on a border and 4 of them on its jungles (13 of the 15 misses
     * beside the sea); the 33 inside Am cells were on its jungles in 27. Cs's 64 misses were all on a border.
     */
    public record PackCheck(String name, String classes, IntPredicate koppen, java.util.Set<String> biomes, double bound,
                            boolean atMost, String requires, Double minErosion, Double minContinents) {}

    /** Below this erosion (vanilla's E1 | E2 edge) a column is mountain relief: slopes and peaks whatever its class. */
    static final double MOUNTAIN_EROSION = -0.375;
    /** Vanilla's near | mid inland edge: Terralith's steppe begins there. */
    static final double MID_INLAND = 0.03;

    /** Terralith's table's checks: each fixed class reaches its intended biomes (bounds: ledger BIO-154, game choices). */
    public static final List<PackCheck> PACK_CHECKS = List.of(
            // Am (monsoon forest) in the jungles, not brushland.
            new PackCheck("terralith:Am_jungle", "Am", k -> k == 1, java.util.Set.of("minecraft:jungle", "minecraft:sparse_jungle",
                    "minecraft:bamboo_jungle", "terralith:tropical_jungle", "terralith:rocky_jungle", "terralith:jungle_mountains",
                    "terralith:amethyst_rainforest"), 0.6, false, "terralith:tropical_jungle", MOUNTAIN_EROSION, null),
            // Cs (Mediterranean) on Terralith's dry shrubland and steppe, not plains.
            new PackCheck("terralith:Cs_scrub", "Csa Csb Csc", k -> k >= 8 && k <= 10, java.util.Set.of("terralith:shrubland",
                    "terralith:steppe", "terralith:hot_shrubland", "terralith:cold_shrubland"), 0.5, false, "terralith:shrubland",
                    MOUNTAIN_EROSION, null),
            // BSk (cold steppe) on the steppe and the dry shrublands, and, mid and far inland, the steppe itself reached.
            new PackCheck("terralith:BSk_steppe", "BSk", k -> k == 7, java.util.Set.of("terralith:steppe", "terralith:shrubland",
                    "terralith:cold_shrubland"), 0.5, false, "terralith:steppe", MOUNTAIN_EROSION, null),
            new PackCheck("terralith:BSk_reaches_steppe", "BSk mid and far inland", k -> k == 7, java.util.Set.of("terralith:steppe"),
                    0.1, false, "terralith:steppe", MOUNTAIN_EROSION, MID_INLAND),
            // ET (tundra) less snowy: snowy plains on at most a tenth of it.
            new PackCheck("terralith:ET_few_snowy_plains", "ET", k -> k == 29, java.util.Set.of("minecraft:snowy_plains"), 0.10, true,
                    "terralith:gravel_desert", null, null));

    /**
     * The cloud forest rule's ground in the census (Terralith's table, class rule {@code cloud_forest}): land columns of
     * Cwb, Cwc, Cfb or Cfc below {@value #CLOUD_LAT_DEG} degrees of latitude, more than {@value #CLOUD_HEIGHT_M} m above
     * the sea and humid (P/PET above {@value #CLOUD_ARIDITY}, read from the level's monthly climate as the bake reads
     * it; wetlands included, since the rule puts these columns in the swamp band's erosion). Columns a volcanic role
     * decided are left out, as from every pack check: a volcano zone's biome comes before the table (BIO-154). Until
     * 2026-10-17 the census left P/PET unchecked; that held while it read the L1 class, whose tropical highlands are
     * few, and stopped holding when it moved to the L2 class (plate task 7's merge), which makes every high column of a
     * tropical L1 cell a C class, dry ones too.
     * <p>
     * The column must also lie in a cell where the bake applied the rule (since 2026-10-17, P1-26's planet): the bake
     * decides the rule per L2 cell on the young landscape's height and that height's P/PET, which the level does not
     * carry, so the census's block heights only estimate it. The bake's own mark is the rule's erosion and
     * continentalness floors in the cell's baked inputs ({@link #cloudFloors}; the bake clamps every rule cell to them,
     * continentalness inland of the beach strip). On P1-26's test planet only 5 of 34 block-level candidates lay in such
     * a cell; 4 of those 5 were on the cloud forest, against 4 of the other 29.
     */
    public static final double CLOUD_LAT_DEG = 25.0, CLOUD_HEIGHT_M = 1000.0; // ledger: BIO-144, BIO-145 (game-choice)
    public static final double CLOUD_ARIDITY = 1.0; // ledger: BIO-145 (game-choice: the rule's own cut)
    public static final String CLOUD_FOREST = "terralith:cloud_forest";
    /** The class rule whose ground {@link #CLOUD_FOREST} is (Terralith's table). */
    public static final String CLOUD_RULE = "cloud_forest";

    static boolean cloudClass(int k) {
        return k == 12 || k == 13 || k == 15 || k == 16;
    }

    /**
     * The cloud forest rule's erosion and continentalness floors from the table the bake recorded
     * ({@code notes.bake.climate.placement.table.class_rules}), null when that table has no such rule (or its world
     * predates the record): then no cell carries the rule and the census's ground is empty.
     */
    public static double[] cloudFloors(WorldData wd) {
        try {
            JsonObject p = wd.manifest().notes().getAsJsonObject("bake").getAsJsonObject("climate")
                    .getAsJsonObject("placement").getAsJsonObject("table");
            if (p == null || !p.has("class_rules"))
                return null;
            for (var e : p.getAsJsonArray("class_rules")) {
                JsonObject r = e.getAsJsonObject();
                if (CLOUD_RULE.equals(r.get("name").getAsString()))
                    return new double[] {r.getAsJsonObject("erosion").get("min").getAsDouble(),
                            r.getAsJsonObject("continentalness").get("min").getAsDouble()};
            }
        } catch (RuntimeException e) {
            return null;
        }
        return null;
    }

    /**
     * Thornthwaite's potential evapotranspiration from monthly mean temperatures, mm/yr, uncorrected for day length:
     * the arithmetic of the bake's {@code climate::snapshot::thornthwaite}, which the table's {@code aridity} input
     * divides the annual precipitation by.
     */
    static double thornthwaitePet(double[] t) {
        double heat = 0;
        for (double x : t)
            if (x > 0)
                heat += Math.pow(x / 5.0, 1.514); // ledger: MOI-065
        if (heat <= 0)
            return 0;
        double a = 6.75e-7 * heat * heat * heat - 7.71e-5 * heat * heat + 1.792e-2 * heat + 0.49239; // ledger: MOI-065
        double pet = 0;
        for (double x : t)
            if (x > 0)
                pet += x > 26.5 ? -415.85 + 32.24 * x - 0.43 * x * x : 16.0 * Math.pow(10.0 * x / heat, a); // ledger: MOI-065
        return pet;
    }

    /** P/PET as the bake computes it (capped at 10, 10 where PET is 0; ledger: MOI-066). */
    static double aridity(ClimateLayers.Sample s) {
        double pet = thornthwaitePet(s.tMonth());
        return pet > 0 ? Math.min(10.0, s.pAnnual() / pet) : 10.0;
    }

    public static final String[] KOPPEN = {"Af", "Am", "Aw", "As", "BWh", "BWk", "BSh", "BSk", "Csa", "Csb", "Csc", "Cwa", "Cwb",
            "Cwc", "Cfa", "Cfb", "Cfc", "Dsa", "Dsb", "Dsc", "Dsd", "Dwa", "Dwb", "Dwc", "Dwd", "Dfa", "Dfb", "Dfc", "Dfd", "ET", "EF",
            "sea"};

    /**
     * Counts per check: matching columns, columns. {@code packs}: per active {@link PackCheck}; {@code cloud}: cloud forest
     * candidates, of them on {@value #CLOUD_FOREST}, cloud forest columns that are not candidates, the columns of the
     * rule's class, latitude and height left out as volcanic and as dry, and those left out because their cell does not
     * carry the bake's rule, with how many of those are on {@value #CLOUD_FOREST}; the biome rules' census (null when no
     * rule file is active).
     */
    public record Result(Map<String, int[]> checks, Map<String, int[]> forbidden, Map<String, int[]> packs, int[] cloud,
                         JsonObject json, RuleCensus rules) {

        /** The share of an active {@link PackCheck}'s columns on its biomes (NaN when inactive or empty). */
        public double packShare(String name) {
            int[] c = packs.get(name);
            return c == null || c[1] == 0 ? Double.NaN : c[0] / (double) c[1];
        }

        public int packTotal(String name) {
            int[] c = packs.get(name);
            return c == null ? 0 : c[1];
        }

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

        public int forbiddenHits(String name) {
            int[] c = forbidden.get(name);
            return c == null ? 0 : c[0];
        }
    }

    /**
     * Samples every {@code step} blocks inside the pole edge (|z| ≤ R − {@code poleMargin}, R = C/2π the pole); on a
     * cube world the counts read the faces only ({@link WorldDataTerrain#onFaces}), the map the whole window.
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
        // The cover as the ice pass lays it (IceCover.of: the cover rule, BIO-159), so a column counts as glaciated
        // exactly where the game puts ice on it.
        IceCover ice = wd.iceCover().orElse(null);
        RasterSampler wetlandLayer = wd.has(Peatland.LAYER) ? wd.layer(Peatland.LAYER) : null;
        var sampler = level.getChunkSource().randomState().sampler();
        int c = t.circumference(), sea = g.getSeaLevel();
        int zp = (int) t.zHalf();   // the net's outer edges on the cube (3s/2), the poles on a Lambert-frame world
        int w = c / step, h = 2 * zp / step;
        short[] biomeIdx = new short[w * h];
        byte[] koppen = new byte[w * h];
        short[] height = new short[w * h];
        Map<String, Integer> palette = new LinkedHashMap<>();
        Map<Holder<Biome>, Integer> byHolder = new HashMap<>();
        List<Holder<Biome>> holders = new ArrayList<>();
        // Per check: matching columns, columns, matching only through TAG_GAPS; and the biomes of the columns that miss.
        Map<String, int[]> checks = new LinkedHashMap<>();
        Map<String, Map<String, Integer>> misses = new HashMap<>();
        for (Check ch : CHECKS) {
            checks.put(ch.name(), new int[3]);
            misses.put(ch.name(), new TreeMap<>());
        }
        Map<String, int[]> forbidden = new LinkedHashMap<>();
        for (Forbid f : FORBIDDEN)
            forbidden.put(f.name(), new int[2]);
        // The pack placement tables' checks whose pack the level's biome source has.
        java.util.Set<String> placeable = new java.util.HashSet<>();
        for (Holder<Biome> b : g.getBiomeSource().possibleBiomes())
            placeable.add(nameOf(b));
        Map<String, int[]> packs = new LinkedHashMap<>();
        // The biomes of each active pack check's counted columns that miss its biomes (an atMost check's: those on them).
        Map<String, Map<String, Integer>> packMisses = new HashMap<>(), packBorderMisses = new HashMap<>();
        for (PackCheck pc : PACK_CHECKS)
            if (placeable.contains(pc.requires())) {
                packs.put(pc.name(), new int[6]);
                packMisses.put(pc.name(), new TreeMap<>());
                packBorderMisses.put(pc.name(), new TreeMap<>());
            }
        boolean cloudActive = placeable.contains(CLOUD_FOREST);
        // Candidates, of them on the cloud forest, cloud forest columns off the candidates, and the columns of the
        // rule's class, latitude and height left out: on a volcanic role's biome, dry (P/PET at most the rule's cut), or in
        // a cell without the bake's rule (and of those, how many are on the cloud forest anyway).
        int[] cloud = new int[7];
        double[] cloudFloor = cloudActive ? cloudFloors(wd) : null;
        RasterSampler bakedErosion = wd.layer(ClimateMapping.EROSION), bakedContinents = wd.layer(ClimateMapping.CONTINENTS);
        Map<String, Integer> cloudOff = new TreeMap<>();
        ClimateLayers monthly = cloudActive ? ClimateLayers.of(wd) : null;
        double mV = wd.manifest().mPerBlockV();
        Map<Integer, Map<Integer, Integer>> classBiomes = new HashMap<>();
        int skippedUntagged = 0, landColumns = 0, wetlands = 0, wetlandSwamp = 0, coastColumns = 0, ruledColumns = 0;
        // Cold wetland (BIO-133, BIO-135): the bake's flagged boreal wetland, counted over every land column
        // (a mod's untagged biomes too: its bog is the point): on the pack's peatland biome, on taiga, on
        // another swamp (which it must not be), or elsewhere.
        int coldWet = 0, coldPeat = 0, coldTaiga = 0, coldSwamp = 0;
        Map<String, Integer> coldBiomes = new TreeMap<>();
        // The bake's wetlands (BIO-066): wet, flat, low, thawed land put in vanilla's swamp band E6 on
        // purpose (the relief's own erosion stays below it), so swamp there whatever the Köppen family.
        // The pack's peatland regions put cold wetland there too (Peatland), and it is set aside likewise.
        double e6 = OverworldBands.defaults().erosion()[6];
        // Vanilla's steep ground, erosion bands E0 and E1, ends at its third band's lower edge (-0.375; BIO-023).
        double e2 = OverworldBands.defaults().erosion()[2];
        // The biome rules (decisions log 2026-10-04): what decided each land column, with the level's bound rules.
        VolcanicBiomes.Level rules = sampler.spawnTarget() instanceof com.minecraftindustrialcomplex.deeptime.worldgen.RegionSpawnTarget m
                ? m.volcanic() : null;
        RasterSampler wetFlag = wd.has(BiomeRules.WETLAND_LAYER) ? wd.layer(BiomeRules.WETLAND_LAYER) : null;
        RuleCensus ruleCensus = null;
        // The land call (BIO-170): the sea rule of the column evaluator (Water.seaCovers, HYD-106) on the world's
        // elevation layer (the ground the bake's land call reads: the projected height, no detail term) against the
        // biome. Counted where the rule is the same 8 blocks away on all four sides, so the coast
        // between two biome nodes, which the game reads bilinear, is left out: columns it puts under the sea, of
        // them on a biome that is not of the water; columns it leaves dry, of them on an ocean biome.
        RasterSampler elevation = wd.layer(WorldData.ELEVATION);
        int seaColumns = 0, seaOnLandBiome = 0, dryColumns = 0, dryOnOceanBiome = 0, seaCoastStrip = 0;
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
                // Its own threshold, not Water.seaCovers (HYD-109); the rule's count is `sea_call`.
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
                // The map holds the whole window; the counts hold a cube world's faces only (a void column shows its
                // partner's, so each planet column counts once).
                if (!t.onFaces(x, z))
                    continue;
                int call = seaCall(wd, elevation, sea, x, z);
                if (call >= 0) {
                    boolean interior = seaCall(wd, elevation, sea, x + 8, z) == call && seaCall(wd, elevation, sea, x - 8, z) == call
                            && seaCall(wd, elevation, sea, x, z + 8) == call && seaCall(wd, elevation, sea, x, z - 8) == call;
                    if (!interior)
                        seaCoastStrip++;
                    else if (call == 1) {
                        seaColumns++;
                        if (!(b.is(c("is_ocean")) || b.is(c("is_deep_ocean")) || b.is(c("is_aquatic")) || b.is(c("is_river"))
                                || b.is(c("is_beach")) || b.is(c("is_stony_shores"))))
                            seaOnLandBiome++;
                    } else {
                        dryColumns++;
                        if (b.is(c("is_ocean")) || b.is(c("is_deep_ocean")))
                            dryOnOceanBiome++;
                    }
                }
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
                // The coast band: the bake puts land at the shore into vanilla's coast continentalness
                // by its distance to the sea, so its biome is a beach, a stony shore or a sea biome
                // whatever the climate. Set apart from the families like the wetlands (2026-10-05: on
                // the L2 Köppen classes, a cold upland's few columns are mostly coast, and its family's
                // share measured the coastline, not the mapping).
                boolean coast = land && !untagged && !wetland
                        && (b.is(c("is_beach")) || b.is(c("is_stony_shores")) || b.is(c("is_ocean")));
                if (coast)
                    coastColumns++;
                // A column a biome rule decided (a volcano zone's biome, a role), not the climate
                // search: the families check the climate mapping, so it is counted apart too.
                boolean ruled = land && !untagged && !wetland && !coast && decision != null;
                if (ruled)
                    ruledColumns++;
                if (land && !untagged && rules != null) {
                    if (ruleCensus == null)
                        ruleCensus = new RuleCensus(rules.rules());
                    // Cold wetland as the biome rules see it (ledger: BIO-189): the flag where the query reads its column's
                    // facts, the centre of the column's quart (VolcanicBiomes.find), and only where the ordinary rules may
                    // decide there: at the surface, off a river's channel and on land. At the waterline the census's own
                    // column can be dry while its quart's centre is under the sea, where no rule runs (2 of 29 cold-wetland
                    // columns of the 2026-10-24 test planet, both on the pack's dunes).
                    int qx = VolcanicBiomes.queryBlock(QuartPos.fromBlock(x)), qz = VolcanicBiomes.queryBlock(QuartPos.fromBlock(z));
                    boolean flagged = wetFlag != null ? wetFlag.nearest(qx, qz) >= 1.5
                            : wetland && BiomeRules.COLD_WETLAND_CLASSES.contains(KOPPEN[k]);
                    boolean ruleColdWet = flagged
                            && VolcanicBiomes.rulesAt(sampler.sample(QuartPos.fromBlock(x), QuartPos.fromBlock((int) Math.floor(y)), QuartPos.fromBlock(z)))
                            && VolcanicBiomes.onLand(rules.terrain(), rules.seaLevel(), qx, qz, rules.terrain().height(qx, qz));
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
                if (land && cloudActive) {
                    boolean candidate = cloudClass(k) && Math.abs(wd.projection().latitudeDeg(x, z)) < CLOUD_LAT_DEG
                            && (y - sea) * mV > CLOUD_HEIGHT_M;
                    if (candidate && b.is(VolcanicBiomes.KEEP_OUT)) {
                        candidate = false;
                        cloud[3]++;
                    } else if (candidate && monthly != null
                            && !(aridity(monthly.sample(x, (int) Math.floor(y), z)) > CLOUD_ARIDITY)) {
                        candidate = false;
                        cloud[4]++;
                    }
                    boolean onCloud = CLOUD_FOREST.equals(nameOf(b));
                    // The bake's mark of the rule in this cell (nearest: the cell's own value, 1e-3 for the i16 store).
                    if (candidate && (cloudFloor == null || bakedErosion.nearest(x, z) < cloudFloor[0] - 1e-3
                            || bakedContinents.nearest(x, z) < cloudFloor[1] - 1e-3)) {
                        candidate = false;
                        cloud[5]++;
                        if (onCloud)
                            cloud[6]++;
                    }
                    if (candidate) {
                        cloud[0]++;
                        if (onCloud)
                            cloud[1]++;
                        else
                            cloudOff.merge(nameOf(b), 1, Integer::sum);
                    } else if (onCloud) {
                        cloud[2]++;
                    }
                }
                if (land && !untagged && !wetland)
                    for (PackCheck pc : PACK_CHECKS) {
                        int[] n = packs.get(pc.name());
                        if (n == null || !pc.koppen().test(k) || b.is(VolcanicBiomes.KEEP_OUT))
                            continue;
                        if (pc.minErosion() != null && t.raster(ClimateMapping.EROSION, x, z) < pc.minErosion())
                            continue;
                        if (pc.minContinents() != null && t.raster(ClimateMapping.CONTINENTS, x, z) < pc.minContinents())
                            continue;
                        // The coast strip (since 2026-10-17, as the families since 2026-10-05): its biome is the
                        // coast band's whatever the class, so it cannot be on a table's biomes (BIO-154).
                        if (coast) {
                            n[2]++;
                            continue;
                        }
                        // Ground under the ice cover (since 2026-10-18): the bake holds glaciated land in the frozen band
                        // whatever its class (BIO-035) and the game lays snow and ice over it (BIO-084), so the table's
                        // class rule does not place it.
                        if (ice != null && ice.blocks(x, z) > 0) {
                            n[3]++;
                            continue;
                        }
                        boolean on = pc.biomes().contains(nameOf(b));
                        // The class border (since 2026-10-21): where the four cells of the class layer that the game's
                        // bilinear read blends are not all the column's class, its biome inputs mix two classes' values,
                        // or the land's and the sea's, so its biome says nothing of the table's rule for its class
                        // (BIO-154). Counted apart, with the biomes its misses are on. Read where the biome source samples
                        // the inputs, the quart cell's corner.
                        String border = classBorder(kp, QuartPos.toBlock(QuartPos.fromBlock(x)), QuartPos.toBlock(QuartPos.fromBlock(z)), k);
                        if (border != null) {
                            n[4]++;
                            if (on)
                                n[5]++;
                            if (on == pc.atMost())
                                packBorderMisses.get(pc.name()).merge(nameOf(b) + " @" + border, 1, Integer::sum);
                            continue;
                        }
                        n[1]++;
                        if (on)
                            n[0]++;
                        if (on == pc.atMost())
                            packMisses.get(pc.name()).merge(nameOf(b), 1, Integer::sum);
                    }
                if (!untagged && (k == 31 ? y < sea - 4 : land)) {
                    for (Check ch : CHECKS) {
                        if (wetland || coast || ruled)
                            break;   // counted apart (above), not against the family's tags
                        if (!ch.koppen().test(k))
                            continue;
                        if (ch.notColderThan() != null && t.raster("climate/temperature", x, z) < ch.notColderThan())
                            continue;
                        int[] n = checks.get(ch.name());
                        n[1]++;
                        boolean tagged = false, gap = false;
                        List<TagKey<Biome>> gapTags = TAG_GAPS.getOrDefault(nameOf(b), List.of());
                        for (TagKey<Biome> tag : ch.tags()) {
                            tagged |= b.is(tag);
                            gap |= gapTags.contains(tag);
                        }
                        if (tagged || gap)
                            n[0]++;
                        else
                            misses.get(ch.name()).merge(nameOf(b), 1, Integer::sum);
                        if (gap && !tagged)
                            n[2]++;
                    }
                    for (Forbid f : FORBIDDEN) {
                        if (!f.koppen().test(k))
                            continue;
                        if (f.glaciated() != null && f.glaciated() != (ice != null && ice.blocks(x, z) > 0))
                            continue;
                        if (f.colderThan() != null && !(t.raster("climate/temperature", x, z) < f.colderThan()))
                            continue;
                        if (f.steep() != null && f.steep() != (wd.raster(ClimateMapping.EROSION, x, z) < e2))
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
            e.addProperty("via_tag_gaps", n[2]);
            // The five biomes the most missing columns are on ("biome columns").
            JsonArray miss = new JsonArray();
            misses.get(ch.name()).entrySet().stream()
                    .sorted(Map.Entry.<String, Integer>comparingByValue().reversed().thenComparing(Map.Entry.comparingByKey()))
                    .limit(5).forEach(en -> miss.add(en.getKey() + " " + en.getValue()));
            e.add("misses", miss);
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
        JsonObject pj = new JsonObject();
        for (PackCheck pc : PACK_CHECKS) {
            int[] n = packs.get(pc.name());
            JsonObject e = new JsonObject();
            e.addProperty("classes", pc.classes());
            e.addProperty("active", n != null);
            if (n != null) {
                e.addProperty("columns", n[1]);
                e.addProperty("matching", n[0]);
                e.addProperty("share", n[1] == 0 ? null : Math.round(n[0] * 1000.0 / n[1]) / 1000.0);
                e.addProperty("left_out_coast", n[2]);
                e.addProperty("left_out_glaciated", n[3]);
                e.addProperty("left_out_border", n[4]);
                e.addProperty("left_out_border_matching", n[5]);
                // The six biomes the most columns that fail the check are on ("biome columns"), and those of the
                // class border's columns with the other classes of their cells ("biome @classes columns", classBorder).
                e.add("misses", topMisses(packMisses.get(pc.name())));
                e.add("border_misses", topMisses(packBorderMisses.get(pc.name())));
            }
            e.addProperty(pc.atMost() ? "at_most" : "at_least", pc.bound());
            JsonArray names = new JsonArray();
            pc.biomes().stream().sorted().forEach(names::add);
            e.add("biomes", names);
            pj.add(pc.name(), e);
        }
        o.add("pack_checks", pj);
        JsonObject cj2 = new JsonObject();
        cj2.addProperty("active", cloudActive);
        cj2.addProperty("candidates", cloud[0]);
        cj2.addProperty("on_cloud_forest", cloud[1]);
        cj2.addProperty("share", cloud[0] == 0 ? null : Math.round(cloud[1] * 1000.0 / cloud[0]) / 1000.0);
        cj2.addProperty("cloud_forest_elsewhere", cloud[2]);
        cj2.addProperty("aridity_checked", monthly != null);
        cj2.addProperty("left_out_volcanic", cloud[3]);
        cj2.addProperty("left_out_dry", cloud[4]);
        JsonObject off = new JsonObject();
        cloudOff.entrySet().stream().sorted(Map.Entry.<String, Integer>comparingByValue().reversed()).limit(6)
                .forEach(e -> off.addProperty(e.getKey(), e.getValue()));
        cj2.add("candidates_off_cloud_forest", off);
        cj2.addProperty("rule_floors_recorded", cloudFloor != null);
        cj2.addProperty("left_out_outside_rule_cells", cloud[5]);
        cj2.addProperty("outside_rule_cells_on_cloud_forest", cloud[6]);
        o.add("cloud_forest", cj2);
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
        o.addProperty("coast_set_aside", coastColumns);
        o.addProperty("rule_decided_set_aside", ruledColumns);
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
        JsonObject sc = new JsonObject();
        sc.addProperty("sea_columns", seaColumns);
        sc.addProperty("sea_on_land_biome", seaOnLandBiome);
        sc.addProperty("dry_columns", dryColumns);
        sc.addProperty("dry_on_ocean_biome", dryOnOceanBiome);
        sc.addProperty("coast_left_out", seaCoastStrip);
        o.add("sea_call", sc);
        // The volcanic biomes against the real volcanoes, now and as the plain climate search placed them
        // (decisions log 2026-10-03).
        o.add("volcanic", VolcanicCensus.run(level, step, poleMargin).json());
        if (ruleCensus != null && !rules.rules().isEmpty())
            o.add("rules", ruleCensus.json());
        // The rock-named biomes against their ground (the geology gate, decisions log 2026-10-04).
        o.add("geology", GeologyCensus.run(level, step, poleMargin).json());
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
        return new Result(checks, forbidden, packs, cloud, o, ruleCensus != null && !rules.rules().isEmpty() ? ruleCensus : null);
    }

    /**
     * The world's sea rule at block (x, z) on the elevation layer (1 the sea stands there, 0 dry ground, -1 where the
     * layer has no ground): {@link Water#seaCovers} on the top block of the projected height with the
     * {@code closed_basin} flag, the call the biome bake settles its land call to (BIO-170).
     */
    static int seaCall(WorldData wd, RasterSampler elevation, int sea, int x, int z) {
        double height = elevation.nearest(x, z);
        if (Double.isNaN(height))
            return -1;
        return Water.seaCovers(Water.topBlock(height), sea, () -> wd.closedBasin(x, z)) ? 1 : 0;
    }

    static String nameOf(Holder<Biome> b) {
        return b.unwrapKey().map(key -> key.location().toString()).orElse("?");
    }

    /** The six entries with the most columns, "name columns", most first. */
    private static JsonArray topMisses(Map<String, Integer> misses) {
        JsonArray out = new JsonArray();
        misses.entrySet().stream()
                .sorted(Map.Entry.<String, Integer>comparingByValue().reversed().thenComparing(Map.Entry.comparingByKey()))
                .limit(6).forEach(en -> out.add(en.getKey() + " " + en.getValue()));
        return out;
    }

    /**
     * Null when the four cells of the class layer that a bilinear read at block (x, z) blends all hold class {@code k} (the
     * column's biome inputs come from its class's cells alone: the climate layers lie on the same L2 grid); otherwise the
     * other classes among them ("sea" for a sea cell), sorted, joined by "+": the column lies on a class border, where the
     * inputs the game reads bilinear ({@code WorldData.raster}) mix two classes' values.
     */
    static String classBorder(RasterSampler kp, int x, int z, int k) {
        double[] cells = kp.bilinearCells(x, z);
        java.util.TreeSet<String> other = new java.util.TreeSet<>();
        for (int d = 0; d < 4; d++) {
            double cv = cells[d];
            int ck = Double.isNaN(cv) ? 31 : (int) Math.round(cv);
            if (ck != k)
                other.add(ck >= 0 && ck < KOPPEN.length ? KOPPEN[ck] : "?" + ck);
        }
        return other.isEmpty() ? null : String.join("+", other);
    }

    private static String b64(short[] v) {
        ByteBuffer bb = ByteBuffer.allocate(v.length * 2).order(ByteOrder.LITTLE_ENDIAN);
        for (short s : v)
            bb.putShort(s);
        return Base64.getEncoder().encodeToString(bb.array());
    }
}
