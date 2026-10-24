// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
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

    /**
     * The rule answers for a lake's own columns alone (ledger HYD-072). The shore rule gives a lake's level to the dry
     * columns within its reach too, and a dry one there (a bank, or a hot-spring pool's ground standing over the lake's
     * top) holds no lake: on the real test planet, at the top block of the level each column within reach of a lake
     * cell takes, the rule says yes (keeps it open or ices it) exactly where the water pass puts a lake
     * ({@link Water#core}). Until 2026-10-23 it said yes in the dry columns too, and on master 4ab29084's planet kept
     * open five blocks of a hot-spring pool's water one block under the pool's surface.
     */
    @Test
    void theRuleIsTheLakeColumnsAlone() {
        WorldData wd = WorldData.open(Fixtures.world("planet4k"));
        assertNotNull(LakeIce.of(wd), "the test planet records the lake ice rule");
        int reach = Columns.NOISE_CELL;
        Water.Spec spec = Water.Spec.of(wd, reach);
        RasterSampler levels = wd.layer(WorldData.WATER_LEVEL);
        int c = wd.circumference();
        long z0 = (long) Math.ceil(wd.projection().zMin()), z1 = (long) Math.floor(wd.projection().zMax());
        int lake = 0, dry = 0, wrong = 0;
        StringBuilder first = new StringBuilder();
        for (long z = z0; z <= z1; z++)
            for (long x = -c / 2; x < c / 2; x++) {
                double level = Water.levelAt(levels, x, z, reach);
                if (!Double.isFinite(level))
                    continue;
                int t = Water.topBlock(level);
                boolean holds = Water.core(wd, spec, x, z).kind() == Water.Kind.LAKE;
                boolean says = LakeIce.keepsOpen(wd, reach, (int) x, t, (int) z) || LakeIce.ices(wd, reach, (int) x, t, (int) z);
                if (holds)
                    lake++;
                else
                    dry++;
                if (says != holds && wrong++ < 5)
                    first.append(String.format(" (%d, %d, %d: %s, the rule says %s)", x, t, z, holds ? "lake" : "dry", says));
            }
        System.out.printf("[LakeIceTest] planet4k: %d lake columns and %d dry columns within reach of a lake%n", lake, dry);
        assertEquals(0, wrong, wrong + " columns where the rule and the water pass disagree:" + first);
        assertTrue(lake > 0, "the test planet has no lake column");
        assertTrue(dry > 0, "the test planet has no dry column within reach of a lake: nothing tells the rule apart");
    }

    @Test
    void theSaltsRockClassIsReadFromTheBakesNote() {
        assertEquals("gypsum", Playa.rockClass(notes("{\"bake\": {\"water\": {\"playa\": {\"rock_class\": \"gypsum\", \"blocks\": 2}}}}")));
        assertEquals(Playa.DEFAULT_CLASS, Playa.rockClass(new JsonObject()));
        assertEquals("halite", Playa.DEFAULT_CLASS);
    }
}
