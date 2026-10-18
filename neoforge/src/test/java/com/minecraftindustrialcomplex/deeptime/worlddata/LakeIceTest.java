// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Frozen lakes and salt flats (Ben, decisions log 2026-10-01): the rule's threshold and the salt's
 * rock class are read from the bake's note, and a world without them has neither.
 */
class LakeIceTest {

    private static JsonObject notes(String json) {
        return JsonParser.parseString(json).getAsJsonObject();
    }

    @Test
    void theThresholdIsReadFromTheBakesNote() {
        assertEquals(0.0, LakeIce.threshold(notes("{\"bake\": {\"water\": {\"params\": {\"lake_freeze_mean_c\": 0.0}}}}")));
        assertEquals(-4.5, LakeIce.threshold(notes("{\"bake\": {\"water\": {\"params\": {\"lake_freeze_mean_c\": -4.5}}}}")));
        assertNull(LakeIce.threshold(new JsonObject()));
        assertNull(LakeIce.threshold(notes("{\"bake\": {\"water\": {\"enabled\": false}}}")));
        assertNull(LakeIce.threshold(notes("{\"bake\": {\"water\": {\"params\": {\"lake_freeze_mean_c\": \"cold\"}}}}")));
    }

    @Test
    void aLakeIsFrozenBelowTheThresholdAndNeverWithoutATemperature() {
        assertTrue(LakeIce.frozen(-0.01, 0.0));
        assertFalse(LakeIce.frozen(0.0, 0.0));
        assertFalse(LakeIce.frozen(12.0, 0.0));
        assertTrue(LakeIce.frozen(-6.0, -4.5));
        assertFalse(LakeIce.frozen(-3.0, -4.5));
        assertFalse(LakeIce.frozen(Double.NaN, 0.0));
    }

    /** The synthetic lake world has no climate and no rule: its lake is left to the biome. */
    @Test
    void aWorldWithoutTheRuleHasNoLakeIce() {
        WorldData wd = WorldData.open(Fixtures.world("lake1024"));
        assertNull(LakeIce.of(wd));
        assertFalse(LakeIce.keepsOpen(wd, 4, 0, 119, 0));
        assertFalse(LakeIce.ices(wd, 4, 0, 119, 0));
    }

    @Test
    void theSaltsRockClassIsReadFromTheBakesNote() {
        assertEquals("gypsum", Playa.rockClass(notes("{\"bake\": {\"water\": {\"playa\": {\"rock_class\": \"gypsum\", \"blocks\": 2}}}}")));
        assertEquals(Playa.DEFAULT_CLASS, Playa.rockClass(new JsonObject()));
        assertEquals("halite", Playa.DEFAULT_CLASS);
    }
}
