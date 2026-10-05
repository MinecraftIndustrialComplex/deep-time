// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Pack placement tables (decisions log 2026-10-04, "Deep Time-side fixes"): a table applies only when every biome it
 * requires is registered, in order of priority then id, one per id; the jar ships the core's files. The core merges and
 * checks them (Rust {@code bake::placement} tests); Java chooses and carries them.
 */
class PlacementPacksTest {

    private static PlacementPacks.Pack pack(String id, int priority, String... requires) {
        JsonObject o = new JsonObject();
        o.addProperty("schema", 1);
        o.addProperty("id", id);
        o.addProperty("priority", priority);
        var req = new com.google.gson.JsonArray();
        for (String r : requires)
            req.add(r);
        o.add("requires_biomes", req);
        o.add("patch", new JsonObject());
        return PlacementPacks.parse(o, id + ".json");
    }

    @Test
    void aTableAppliesOnlyWhenEveryBiomeItNeedsIsRegistered() {
        var terralith = pack("terralith", 0, "terralith:steppe", "terralith:cloud_forest");
        var bop = pack("biomesoplenty", 0, "biomesoplenty:bog");
        Set<String> mic = Set.of("minecraft:plains", "terralith:steppe", "terralith:cloud_forest");
        assertEquals(List.of("terralith"), PlacementPacks.ids(PlacementPacks.select(List.of(terralith, bop), mic::contains)));
        // Vanilla: none. One biome missing: none.
        assertEquals(List.of(), PlacementPacks.select(List.of(terralith, bop), Set.of("minecraft:plains")::contains));
        assertEquals(List.of(), PlacementPacks.select(List.of(terralith), Set.of("terralith:steppe")::contains));
    }

    @Test
    void tablesGoInOrderOfPriorityThenIdAndALaterFileReplacesAnId() {
        var a = pack("b_pack", 0, "x:a");
        var b = pack("a_pack", 0, "x:a");
        var c = pack("c_pack", -1, "x:a");
        var replaced = pack("b_pack", 5, "x:a");
        assertEquals(List.of("c_pack", "a_pack", "b_pack"), PlacementPacks.ids(PlacementPacks.select(List.of(a, b, c), id -> true)));
        assertEquals(List.of("c_pack", "a_pack", "b_pack"),
                PlacementPacks.ids(PlacementPacks.select(List.of(a, b, c, replaced), id -> true)));
        assertEquals(5, PlacementPacks.select(List.of(a, replaced), id -> true).get(0).priority());
    }

    @Test
    void theContentBundleCarriesTheChosenTablesWhole() {
        var t = pack("terralith", 0, "terralith:steppe");
        String essentials = "{\"essentials\":[]}";
        JsonObject both = JsonParser.parseString(PlacementPacks.addTo(essentials, List.of(t))).getAsJsonObject();
        assertTrue(both.has("essentials"));
        assertEquals(t.json(), both.getAsJsonArray(PlacementPacks.KEY).get(0));
        assertEquals(essentials, PlacementPacks.addTo(essentials, List.of()));
        // What a bake recorded.
        JsonObject note = JsonParser.parseString("{\"placement\":{\"patches\":[\"terralith\"]}}").getAsJsonObject();
        assertEquals(List.of("terralith"), PlacementPacks.recorded(note));
        assertEquals(List.of(), PlacementPacks.recorded(new JsonObject()));
    }

    @Test
    void aBadTableIsRefusedNamingItsFile() {
        var e = assertThrows(IllegalArgumentException.class, () -> PlacementPacks.parse("{\"schema\":1,\"id\":\"x\",\"patch\":{}}", "x.json"));
        assertTrue(e.getMessage().contains("x.json") && e.getMessage().contains("requires_biomes"), e.getMessage());
        assertThrows(IllegalArgumentException.class, () -> PlacementPacks.parse("{\"schema\":2}", "y.json"));
        assertThrows(IllegalArgumentException.class, () -> PlacementPacks.parse("[1]", "z.json"));
        assertThrows(IllegalArgumentException.class,
                () -> PlacementPacks.parse("{\"schema\":1,\"id\":\"x\",\"patch\":{},\"requires_biomes\":[\"steppe\"]}", "w.json"));
    }

    @Test
    void theJarShipsTheCoresPackTables() throws IOException {
        Path dir = Path.of(System.getProperty("deeptime.repoRoot"), "crates", "deeptime-core", "data", "climate_mapping", "packs");
        int n = 0;
        try (var files = Files.list(dir)) {
            for (Path p : files.filter(f -> f.toString().endsWith(".json")).toList()) {
                String res = "data/deeptime/" + PlacementPacks.DIR + "/" + p.getFileName();
                try (InputStream in = PlacementPacksTest.class.getClassLoader().getResourceAsStream(res)) {
                    assertTrue(in != null, "the jar ships " + res);
                    String jar = new String(in.readAllBytes(), StandardCharsets.UTF_8);
                    assertEquals(JsonParser.parseString(Files.readString(p)), JsonParser.parseString(jar), "one file, two readers");
                    var parsed = PlacementPacks.parse(jar, res);
                    assertEquals(p.getFileName().toString(), parsed.id() + ".json");
                }
                n++;
            }
        }
        assertTrue(n >= 1);
    }
}
