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
        assertTrue(ee.keepOut().contains("wythers:mushroom_island") && ee.keepOut().contains("minecraft:mushroom_fields"), "the islands are kept out");
        Map<String, Rule> byId = ee.rules().stream().collect(Collectors.toMap(Rule::id, Function.identity()));
        assertEquals(ee.rules().size(), byId.size(), "rule ids are unique");
        for (Rule r : ee.rules())
            for (String b : r.biomes())
                assertTrue(b.startsWith("wythers:") || b.startsWith("minecraft:") || b.startsWith("#"), r.id() + " names " + b);
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
        // Hot desert is the pack's deserts; a cold desert (BWk under 10 degC) is not.
        assertEquals("hot_desert", first(ee, new F("BWh", 25, 33, 15, 80, 300, Wetland.NONE, false, false, false, "sandstone", "x"), null));
        assertEquals(null, first(ee, new F("BWk", 5, 18, -10, 150, 300, Wetland.NONE, false, false, false, "sandstone", "x"), null));
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

    /** The snow line (BIO-180): vanilla's snowy slots stand only on cold ground at the column's own height. */
    @Test
    void theSnowLineKeepsSnowToColdGround() throws IOException {
        BiomeRuleSet.File snow = shipped("snow_line");
        assertTrue(snow.requires().isEmpty(), "active with any pack");
        String slopes = "minecraft:snowy_slopes", field = "#deeptime:snow_line/snowfield";
        // A mild mountain (Csb at its own height) loses its snowy slopes and peaks; tundra and ice keep them.
        assertEquals("above_the_tree_line", first(snow, new F("Csb", 9, 17, 3, 700, 1500, Wetland.NONE, false, false, false, "",
                slopes, field), null));
        assertEquals("above_the_tree_line", first(snow, new F("BSk", 6, 19, -6, 350, 1200, Wetland.NONE, false, false, false, "",
                "minecraft:frozen_peaks", field), null));
        assertEquals(null, first(snow, new F("ET", -6, 7, -20, 400, 2500, Wetland.NONE, false, false, false, "", slopes, field), null));
        assertEquals(null, first(snow, new F("Csb", 9, 17, 3, 700, 1500, Wetland.NONE, true, false, false, "", slopes, field), null));
        // The warmest month at exactly 10 degC is above the polar line (Koppen's E is under 10).
        assertEquals("above_the_tree_line", first(snow, new F("Dfc", -2, 10, -14, 500, 900, Wetland.NONE, false, false, false, "",
                slopes, field), null));
        // A grove (snowy conifers) stands wherever the winter freezes, the boreal forest included; not in a mild winter.
        assertEquals(null, first(snow, new F("Dfc", -2, 15, -14, 500, 900, Wetland.NONE, false, false, false, "", "minecraft:grove"), null));
        assertEquals("freezing_winter", first(snow, new F("Cfb", 8, 15, 2, 1200, 900, Wetland.NONE, false, false, false, "",
                "minecraft:grove"), null));
        // Anything else is the pack's own.
        assertEquals(null, first(snow, land("Csb"), null));
        for (Rule r : snow.rules())
            assertEquals(List.of("#deeptime:snow_line/snowless"), r.biomes(), r.id());
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
        assertTrue(parse.apply("{\"rules\": [], \"keepout\": []}").getMessage().contains("unknown key 'keepout'"));
        assertTrue(parse.apply("{\"rules\": [{\"biomes\": \"a:b\", \"colour\": 1}]}").getMessage().contains("unknown key 'colour'"));
        assertTrue(parse.apply("{\"rules\": [{\"koppen\": \"ET\"}]}").getMessage().contains("no 'biomes'"));
        assertTrue(parse.apply("{\"rules\": [{\"biomes\": \"a:b\", \"role\": \"geyser\"}]}").getMessage().contains("unknown role"));
        assertTrue(parse.apply("{\"rules\": [{\"biomes\": \"a:b\", \"wetland\": \"damp\"}]}").getMessage().contains("wetland"));
        assertTrue(parse.apply("{\"rules\": [{\"biomes\": \"a:b\", \"p_mm\": {\"min\": 5, \"max\": 1}}]}").getMessage().contains("min > max"));
        assertTrue(parse.apply("{\"rules\": [{\"biomes\": \"a:b\", \"p_mm\": {}}]}").getMessage().contains("neither"));
    }

    /** A column for the geology gate: its ground role bits and the biome its own slot holds. */
    private record G(String koppen, int groundBits, String slot) implements Facts {
        public double tMean() { return 10; }
        public double tWarm() { return 18; }
        public double tCold() { return 2; }
        public double pMm() { return 900; }
        public double elevationM() { return 200; }
        public Wetland wetland() { return Wetland.NONE; }
        public boolean ice() { return false; }
        public boolean playa() { return false; }
        public boolean volcanic() { return false; }
        public String topRock() { return ""; }
        public boolean plainIn(List<String> ids) { return false; }

        @Override
        public int ground() {
            return groundBits;
        }

        @Override
        public boolean slotIn(List<String> ids) {
            return ids.contains(slot);
        }
    }

    @Test
    void theGeologyGateKeepsRockNamedBiomesToTheirGround() throws IOException {
        BiomeRuleSet.File gate = shipped("rock_named");
        assertEquals(List.of("#deeptime:geology"), gate.requires());
        assertEquals(com.minecraftindustrialcomplex.deeptime.worldgen.GeologyGround.Role.values().length, gate.rules().size(),
                "one rule per ground role");
        for (Rule r : gate.rules()) {
            var role = com.minecraftindustrialcomplex.deeptime.worldgen.GeologyGround.Role.byId(r.id());
            assertTrue(role != null, r.id() + " is a ground role");
            assertEquals(role.bit(), r.ground(), r.id() + " asks for its own ground");
            assertEquals(List.of("#deeptime:geology/" + r.id()), r.biomes(), r.id() + " takes its role's tag");
            assertTrue(r.exclusive() && r.inSlot(), r.id() + " is a gate: exclusive, in its own slot");
        }
        int shield = com.minecraftindustrialcomplex.deeptime.worldgen.GeologyGround.Role.SHIELD.bit();
        int granite = com.minecraftindustrialcomplex.deeptime.worldgen.GeologyGround.Role.GRANITE.bit();
        // Shield's slot on a shield province: the rule gives it back. Off the province, or another biome's slot: no rule.
        assertEquals("shield", first(gate, new G("Dfc", shield, "#deeptime:geology/shield"), null));
        assertEquals(null, first(gate, new G("Dfc", granite, "#deeptime:geology/shield"), null));
        assertEquals(null, first(gate, new G("Dfc", shield, "minecraft:taiga"), null), "the gate never adds a biome");
        assertEquals("granite", first(gate, new G("Dfb", shield | granite, "#deeptime:geology/granite"), null));
        // An unknown ground is refused, and the keys mean nothing to a rule that does not name them.
        IllegalArgumentException e = assertThrows(IllegalArgumentException.class, () -> BiomeRuleSet.parse("t:x",
                JsonParser.parseString("{\"rules\": [{\"biomes\": \"a:b\", \"ground\": \"marble\"}]}").getAsJsonObject()));
        assertTrue(e.getMessage().contains("unknown ground 'marble'"), e.getMessage());
        Rule plain = BiomeRuleSet.parse("t:x", JsonParser.parseString("{\"rules\": [{\"biomes\": \"a:b\"}]}").getAsJsonObject()).rules().get(0);
        assertTrue(plain.ground() == 0 && !plain.inSlot() && plain.matches(land("Cfb")));
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
