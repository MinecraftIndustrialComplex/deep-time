// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.worldgen.LavaRules;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Where a Deep Time world has lava underground ({@link LavaRules}; Ben's preview of 2026-10-01:
 * every cave and every deep stretch of sea below y −54 was lava, because vanilla measures its lava
 * heights from y 0 and the world's floor was y −352; now −512). This covers what is plain Java: the two rules
 * measured from the floor, the sea-bed rule, the reader, and that a baked world carries the
 * settings. The generated blocks are the GameTest {@code lavaStaysAtTheFloor}'s.
 */
class LavaRulesTest {

    private static final int MIN_Y = -512, SEA = 63;

    private static JsonObject notes(String json) {
        return JsonParser.parseString(json).getAsJsonObject();
    }

    @Test
    void theDefaultsAreVanillasHeightsAboveTheFloor() {
        // Vanilla: floor -64, lava below -54, lava lakes at -10 and lower.
        assertEquals(-54, LavaRules.DEFAULT.floodY(-64, SEA));
        assertEquals(-10, -64 + LavaRules.DEFAULT.lakeBlocks());
        // Deep Time: the same distances from a floor at -512. The bug was lava below -54 here.
        assertEquals(-502, LavaRules.DEFAULT.floodY(MIN_Y, SEA));
        assertTrue(LavaRules.DEFAULT.floodY(MIN_Y, SEA) < -54 - 400, "the lava floor follows the world's floor");
    }

    @Test
    void lavaNeverStandsAboveTheSea() {
        assertEquals(40, new LavaRules(900, 54).floodY(MIN_Y, 40));
        assertEquals(MIN_Y, new LavaRules(0, 54).floodY(MIN_Y, SEA), "0 blocks: no lava floor");
    }

    @Test
    void aDeepSeaBedKeepsRockOverTheLava() {
        LavaRules r = LavaRules.DEFAULT;
        // Ground well above the lava floor: the floor's level.
        assertEquals(-502, r.columnFloodY(MIN_Y, SEA, -427.3));
        assertEquals(-502, r.columnFloodY(MIN_Y, SEA, 120.0));
        // A trench whose bed is within reach of it: the lava's surface drops below the bed.
        assertEquals(-505 - LavaRules.SEA_COVER, r.columnFloodY(MIN_Y, SEA, -504.5));
        // A bed at the world's floor: no lava in the column at all (nothing is below min_y).
        assertTrue(r.columnFloodY(MIN_Y, SEA, MIN_Y + 2.0) <= MIN_Y);
        // With the bug's level (floor + 458 = y -54) a 150-block-deep sea still stays water.
        LavaRules high = new LavaRules(458, 54);
        assertEquals(-54, high.floodY(MIN_Y, SEA));
        assertEquals(-100 - LavaRules.SEA_COVER, high.columnFloodY(MIN_Y, SEA, -100.0));
    }

    @Test
    void lavaLakesOnlyInTheCellsNearTheFloor() {
        LavaRules r = LavaRules.DEFAULT;
        // Cells are 40 blocks: cell -13 is y -520..-481 (centre -500), cell -12 is -480..-441 (-460).
        assertTrue(r.lakeCell(MIN_Y, -13));
        assertTrue(r.lakeCell(MIN_Y, -12));
        assertFalse(r.lakeCell(MIN_Y, -11), "centre -420 is 92 blocks above the floor");
        assertFalse(r.lakeCell(MIN_Y, -1), "vanilla's own lava-lake cell, y -40..-1");
        assertFalse(r.lakeCell(MIN_Y, 0));
        assertFalse(r.lakeCell(MIN_Y, 1));
        // The same rule in a vanilla-height world allows exactly vanilla's cells.
        assertTrue(r.lakeCell(-64, -1));
        assertTrue(r.lakeCell(-64, -2));
        assertFalse(r.lakeCell(-64, 0));
        for (int cell = -13; cell <= 16; cell++)
            assertFalse(new LavaRules(10, 0).lakeCell(MIN_Y, cell), "0 blocks: no lava lakes");
        assertEquals(-460 + LavaRules.LAKE_SPREAD, r.highestLakeY(MIN_Y));
        for (int cell = -13; cell <= 16; cell++)
            if (r.lakeCell(MIN_Y, cell))
                assertTrue(cell * 40 + 20 + LavaRules.LAKE_SPREAD <= r.highestLakeY(MIN_Y));
    }

    @Test
    void anAbsentSettingMeansTheDefaults() {
        assertEquals(LavaRules.DEFAULT, LavaRules.of(null));
        assertEquals(LavaRules.DEFAULT, LavaRules.of(notes("{}")));
        assertEquals(LavaRules.DEFAULT, LavaRules.of(notes("{\"bake\": {\"water\": {}}}")));
        assertEquals(LavaRules.DEFAULT, LavaRules.of(notes("{\"bake\": {\"underground\": {\"params\": []}}}")));
        assertEquals(LavaRules.DEFAULT,
                LavaRules.of(notes("{\"bake\": {\"underground\": {\"params\": {\"lava_floor_blocks\": \"deep\"}}}}")));
    }

    @Test
    void theSettingsAreReadFromTheBakeNotes() {
        LavaRules r = LavaRules.of(notes(
                "{\"bake\": {\"underground\": {\"params\": {\"lava_floor_blocks\": 24, \"lava_lake_blocks\": 0}}}}"));
        assertEquals(new LavaRules(24, 0), r);
        assertEquals(-488, r.floodY(MIN_Y, SEA));
        // One key alone leaves the other at its default; out-of-range values are clamped.
        assertEquals(new LavaRules(10, 120),
                LavaRules.of(notes("{\"bake\": {\"underground\": {\"params\": {\"lava_lake_blocks\": 120}}}}")));
        assertEquals(new LavaRules(0, LavaRules.MAX_BLOCKS),
                LavaRules.of(notes("{\"bake\": {\"underground\": {\"params\": {\"lava_floor_blocks\": -5, \"lava_lake_blocks\": 5000}}}}")));
    }

    @Test
    void aBakedWorldRecordsTheSettings() {
        WorldData wd = WorldData.open(ColumnParityTest.world());
        JsonObject underground = wd.manifest().notes().getAsJsonObject("bake").getAsJsonObject("underground");
        assertNotNull(underground, "the bake records the underground section");
        JsonObject params = underground.getAsJsonObject("params");
        assertEquals(10, params.get("lava_floor_blocks").getAsInt());
        assertEquals(54, params.get("lava_lake_blocks").getAsInt());
        assertEquals(LavaRules.DEFAULT, LavaRules.of(wd.manifest().notes()));
    }
}
