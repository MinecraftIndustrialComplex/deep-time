// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * {@code hydro.protect_water_from_carvers} (Ben, decisions log 2026-10-01, "Protect water blocks"):
 * the setting the bake records in {@code notes.bake.water.params} and the game reads back to decide
 * whether {@code WaterBodies} puts the water it places in the carving masks. The pass itself needs a
 * running game (the GameTest {@code carversSpareWater} runs real carvers over rivers and lakes);
 * this covers what is plain Java: the reader, and that a baked world carries the setting.
 */
class WaterProtectionTest {

    private static JsonObject notes(String json) {
        return JsonParser.parseString(json).getAsJsonObject();
    }

    @Test
    void anAbsentSettingProtectsTheWater() {
        assertTrue(Water.protectsFromCarvers(null));
        assertTrue(Water.protectsFromCarvers(notes("{}")));
        assertTrue(Water.protectsFromCarvers(notes("{\"bake\": {}}")));
        assertTrue(Water.protectsFromCarvers(notes("{\"bake\": {\"water\": {\"enabled\": false}}}")));
        assertTrue(Water.protectsFromCarvers(notes("{\"bake\": {\"water\": {\"params\": {}}}}")));
        assertTrue(Water.protectsFromCarvers(notes("{\"bake\": {\"water\": {\"params\": []}}}")));
    }

    @Test
    void theSettingIsReadFromTheBakeNotes() {
        assertTrue(Water.protectsFromCarvers(
                notes("{\"bake\": {\"water\": {\"params\": {\"protect_water_from_carvers\": true}}}}")));
        assertFalse(Water.protectsFromCarvers(
                notes("{\"bake\": {\"water\": {\"params\": {\"protect_water_from_carvers\": false}}}}")));
        // Not a boolean: the default, never a guess.
        assertTrue(Water.protectsFromCarvers(
                notes("{\"bake\": {\"water\": {\"params\": {\"protect_water_from_carvers\": \"no\"}}}}")));
    }

    @Test
    void aBakedWorldRecordsTheSettingAndProtectsItsWater() {
        WorldData wd = WorldData.open(ColumnParityTest.world());
        JsonObject params = wd.manifest().notes().getAsJsonObject("bake").getAsJsonObject("water").getAsJsonObject("params");
        assertNotNull(params, "the bake records the hydro parameters");
        assertTrue(params.has("protect_water_from_carvers"), "the preset's hydro section carries the setting");
        assertEquals(true, params.get("protect_water_from_carvers").getAsBoolean(), "on by default");
        assertTrue(Water.protectsFromCarvers(wd.manifest().notes()));
    }
}
