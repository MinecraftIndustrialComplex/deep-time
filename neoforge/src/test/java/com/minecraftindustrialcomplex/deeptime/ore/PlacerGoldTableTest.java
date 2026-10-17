// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.ore;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The loot table of a placer's panned block ({@code deeptime:archaeology/placer_gold}; Ben,
 * 2026-10-16, "Could we use suspicious gravel w/ a loot table set to a gold nugget?"; ledger
 * ORE-267, a game choice). JUnit has no Minecraft bootstrap, so this reads the shipped JSON: the
 * GameTest {@code placerGoldLootTable} rolls the loaded table in a real server. The table must be
 * what {@code BrushableBlockEntity} can use: type {@code minecraft:archaeology}, one pool, one roll,
 * one entry (the block drops only the first stack the table rolls), gold nuggets, a count of 1 to
 * 9 (9 nuggets make an ingot, so one block never pays more than an ingot).
 */
class PlacerGoldTableTest {

    private static JsonObject table() throws IOException {
        String res = "data/deeptime/loot_table/archaeology/placer_gold.json";
        try (InputStream in = PlacerGoldTableTest.class.getClassLoader().getResourceAsStream(res)) {
            assertTrue(in != null, "the jar ships " + res);
            return JsonParser.parseReader(new InputStreamReader(in, StandardCharsets.UTF_8)).getAsJsonObject();
        }
    }

    @Test
    void theTableIsAnArchaeologyTableOfOneRollOfGoldNuggets() throws IOException {
        JsonObject t = table();
        assertEquals("minecraft:archaeology", t.get("type").getAsString());
        assertEquals("deeptime:archaeology/placer_gold", t.get("random_sequence").getAsString());
        JsonArray pools = t.getAsJsonArray("pools");
        assertEquals(1, pools.size(), "one pool: the brushed block gives the first stack only");
        JsonObject pool = pools.get(0).getAsJsonObject();
        assertEquals(1.0, pool.get("rolls").getAsDouble());
        assertEquals(0.0, pool.get("bonus_rolls").getAsDouble());
        JsonArray entries = pool.getAsJsonArray("entries");
        assertEquals(1, entries.size());
        JsonObject entry = entries.get(0).getAsJsonObject();
        assertEquals("minecraft:item", entry.get("type").getAsString());
        assertEquals("minecraft:gold_nugget", entry.get("name").getAsString());
    }

    /** The count is a uniform draw between whole numbers, from at least 1 to at most 9 (an ingot's worth). */
    @Test
    void theCountIsAUniformDrawOfAFewNuggets() throws IOException {
        JsonObject entry = table().getAsJsonArray("pools").get(0).getAsJsonObject().getAsJsonArray("entries").get(0).getAsJsonObject();
        JsonArray functions = entry.getAsJsonArray("functions");
        assertEquals(1, functions.size());
        JsonObject f = functions.get(0).getAsJsonObject();
        assertEquals("minecraft:set_count", f.get("function").getAsString());
        assertTrue(!f.has("add") || !f.get("add").getAsBoolean(), "set the count, do not add to it");
        JsonObject count = f.getAsJsonObject("count");
        assertEquals("minecraft:uniform", count.get("type").getAsString());
        double min = count.get("min").getAsDouble(), max = count.get("max").getAsDouble();
        assertEquals(Math.floor(min), min, "min is a whole number");
        assertEquals(Math.floor(max), max, "max is a whole number");
        assertTrue(1 <= min && min <= max && max <= 9, "count " + min + " to " + max);
    }
}
