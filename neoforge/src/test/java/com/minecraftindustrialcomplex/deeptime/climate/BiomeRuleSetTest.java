// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.climate.BiomeRuleSet.Facts;
import com.minecraftindustrialcomplex.deeptime.climate.BiomeRuleSet.Rule;
import com.minecraftindustrialcomplex.deeptime.climate.BiomeRuleSet.Wetland;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Map;
import java.util.function.Function;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Deep Time's biome rules (decisions log 2026-10-04): the file format, the shipped files, and which rule a
 * column's facts select. The search over a rule's candidates is the game's ({@code BiomeRules}, GameTests).
 */
class BiomeRuleSetTest {

    private static BiomeRuleSet.File shipped(String name) throws IOException {
        String res = "data/deeptime/" + BiomeRuleSet.DIR + "/" + name + ".json";
        try (InputStream in = BiomeRuleSetTest.class.getClassLoader().getResourceAsStream(res)) {
            assertTrue(in != null, "the jar ships " + res);
            JsonObject o = JsonParser.parseReader(new InputStreamReader(in, StandardCharsets.UTF_8)).getAsJsonObject();
            return BiomeRuleSet.parse("deeptime:" + name, o);
        }
    }

    /** A column: every fact settable; {@code plain} lists the ids and tags the plain search's biome is in. */
    private record F(String koppen, double tMean, double tWarm, double tCold, double pMm, double elevationM, Wetland wetland,
                     boolean ice, boolean playa, boolean volcanic, String topRock, List<String> plain) implements Facts {
        F(String koppen, double tMean, double tWarm, double tCold, double pMm, double elevationM, Wetland wetland, boolean ice,
          boolean playa, boolean volcanic, String topRock, String... plain) {
            this(koppen, tMean, tWarm, tCold, pMm, elevationM, wetland, ice, playa, volcanic, topRock, List.of(plain));
        }

        @Override
        public boolean plainIn(List<String> ids) {
            for (String id : ids)
                if (plain.contains(id))
                    return true;
            return false;
        }
    }

    private static F land(String koppen) {
        return new F(koppen, 10, 18, 2, 900, 200, Wetland.NONE, false, false, false, "sandstone", "wythers:forest");
    }

    /**
     * The first rule of {@code file} for volcanic role {@code role} (null: the ordinary rules) whose conditions hold
     * (no candidate check: every shipped candidate is the pack's).
     */
    private static String first(BiomeRuleSet.File file, Facts f, String role) {
        for (Rule r : file.rules())
            if (java.util.Objects.equals(r.role(), role) && r.matches(f))
                return r.id();
        return null;
    }

    @Test
    void theShippedFilesParse() throws IOException {
        BiomeRuleSet.File ee = shipped("expanded_ecosphere");
        assertEquals(List.of("wythers:ice_cap", "wythers:fen"), ee.requires());
        Map<String, Rule> byId = ee.rules().stream().collect(Collectors.toMap(Rule::id, Function.identity()));
        assertEquals(ee.rules().size(), byId.size(), "rule ids are unique");
        for (Rule r : ee.rules())
            for (String b : r.biomes())
                assertTrue(b.startsWith("wythers:") || b.startsWith("#"), r.id() + " names " + b);
        BiomeRuleSet.File peat = shipped("peatland");
        assertEquals(List.of("#deeptime:peatland"), peat.requires());
        assertEquals("cold", peat.rules().get(0).wetland());
    }

    @Test
    void columnsSelectTheirRules() throws IOException {
        BiomeRuleSet.File ee = shipped("expanded_ecosphere");
        // Glaciated land is ice cap, a glaciated coast glacial cliffs, a glaciated peak keeps the pack's peak.
        F ice = new F("EF", -25, -3, -40, 150, 400, Wetland.NONE, true, false, false, "granite", "minecraft:snowy_plains");
        assertEquals("ice_cap", first(ee, ice, null));
        assertEquals("glacial_cliffs", first(ee, new F("EF", -25, -3, -40, 150, 2, Wetland.NONE, true, false, false, "granite",
                "minecraft:snowy_beach", "#minecraft:is_beach"), null));
        assertEquals(null, first(ee, new F("EF", -25, -3, -40, 150, 2000, Wetland.NONE, true, false, false, "granite",
                "minecraft:frozen_peaks", "#c:is_mountain/peak"), null));
        // Barren EF is polar desert; tundra splits at a warmest month of 5 degC.
        assertEquals("polar_desert", first(ee, new F("EF", -22, -1, -40, 100, 300, Wetland.NONE, false, false, false, "", "x"), null));
        assertEquals("low_arctic_tundra", first(ee, new F("ET", -8, 7, -25, 300, 100, Wetland.NONE, false, false, false, "", "x"), null));
        assertEquals("high_arctic_tundra", first(ee, new F("ET", -15, 3, -30, 200, 100, Wetland.NONE, false, false, false, "", "x"), null));
        // A playa is salt lakes whatever the class; carbonate in the humid tropics is tsingy, granite is not.
        assertEquals("salt_lakes", first(ee, new F("BWh", 25, 33, 15, 80, 300, Wetland.NONE, false, true, false, "halite", "x"), null));
        assertEquals("tsingy", first(ee, new F("Aw", 26, 28, 23, 1100, 100, Wetland.NONE, false, false, false, "limestone", "x"), null));
        assertEquals("dry_tropical_forest", first(ee, new F("Aw", 26, 28, 23, 1100, 100, Wetland.NONE, false, false, false, "granite", "x"), null));
        assertEquals(null, first(ee, new F("Aw", 26, 28, 23, 800, 100, Wetland.NONE, false, false, false, "granite", "x"), null));
        // Temperate rainforest needs the rain and the cool mean; Mediterranean scrub the dryness.
        assertEquals("temperate_rainforest", first(ee, new F("Cfb", 9, 15, 3, 2000, 50, Wetland.NONE, false, false, false, "", "x"), null));
        assertEquals(null, first(ee, new F("Cfb", 9, 15, 3, 900, 50, Wetland.NONE, false, false, false, "", "x"), null));
        assertEquals("mediterranean_scrub", first(ee, new F("Csa", 17, 25, 9, 500, 50, Wetland.NONE, false, false, false, "", "x"), null));
        assertEquals("cold_steppe", first(ee, land("BSk"), null));
        // Hot desert highland; a beach in a volcano's zone is black.
        assertEquals("tibesti_mountains", first(ee, new F("BWh", 22, 30, 12, 20, 1800, Wetland.NONE, false, false, false, "basalt", "x"), null));
        assertEquals("black_beach_volcanic", first(ee, new F("Cfb", 12, 17, 6, 1200, 2, Wetland.NONE, false, false, true, "sand",
                "minecraft:beach", "#c:is_beach"), null));
        assertEquals("black_beach_basalt", first(ee, new F("Cfb", 12, 17, 6, 1200, 2, Wetland.NONE, false, false, false, "basalt",
                "minecraft:beach", "#c:is_beach"), null));
        // An ordinary temperate forest is the pack's own.
        assertEquals(null, first(ee, land("Cfa"), null));
        // Volcanic roles by climate.
        assertEquals("icy_volcano", first(ee, land("Dfc"), "cone"));
        assertEquals("tropical_volcano", first(ee, land("Af"), "cone"));
        assertEquals(null, first(ee, land("Cfb"), "cone"), "other cones take the role tag's first biome");
        assertEquals("thermal_taiga", first(ee, land("ET"), "hot_spring_field"));
        assertEquals("mud_pools", first(ee, land("BWh"), "hot_spring_field"));
    }

    @Test
    void peatlandTakesColdWetlandOnly() throws IOException {
        Rule peat = shipped("peatland").rules().get(0);
        F cold = new F("Dfc", -3, 15, -25, 500, 60, Wetland.COLD, false, false, false, "", "x");
        F warm = new F("Cfb", 9, 15, 3, 900, 20, Wetland.WARM, false, false, false, "", "x");
        assertTrue(peat.matches(cold));
        assertFalse(peat.matches(warm));
        assertFalse(peat.matches(land("Dfc")));
    }

    @Test
    void badFilesAreRefusedWithTheirReason() {
        Function<String, IllegalArgumentException> parse = json -> assertThrows(IllegalArgumentException.class,
                () -> BiomeRuleSet.parse("t:x", JsonParser.parseString(json).getAsJsonObject()));
        assertTrue(parse.apply("{}").getMessage().contains("rules"));
        assertTrue(parse.apply("{\"rules\": [{\"biomes\": \"a:b\", \"colour\": 1}]}").getMessage().contains("unknown key 'colour'"));
        assertTrue(parse.apply("{\"rules\": [{\"koppen\": \"ET\"}]}").getMessage().contains("no 'biomes'"));
        assertTrue(parse.apply("{\"rules\": [{\"biomes\": \"a:b\", \"role\": \"geyser\"}]}").getMessage().contains("unknown role"));
        assertTrue(parse.apply("{\"rules\": [{\"biomes\": \"a:b\", \"wetland\": \"damp\"}]}").getMessage().contains("wetland"));
        assertTrue(parse.apply("{\"rules\": [{\"biomes\": \"a:b\", \"p_mm\": {\"min\": 5, \"max\": 1}}]}").getMessage().contains("min > max"));
        assertTrue(parse.apply("{\"rules\": [{\"biomes\": \"a:b\", \"p_mm\": {}}]}").getMessage().contains("neither"));
    }

    @Test
    void rangesAndWetlandStatesMeanWhatTheySay() {
        BiomeRuleSet.Range r = new BiomeRuleSet.Range(5, Double.NaN);
        assertTrue(r.has(5) && r.has(1e9));
        assertFalse(r.has(4.99) || r.has(Double.NaN), "below the minimum, or no climate at all, never matches");
        assertTrue(Wetland.meets("any", Wetland.COLD) && Wetland.meets("any", Wetland.WARM) && !Wetland.meets("any", Wetland.NONE));
        assertTrue(Wetland.meets("none", Wetland.NONE) && !Wetland.meets("cold", Wetland.WARM));
    }
}
