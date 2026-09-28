// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.minecraftindustrialcomplex.deeptime.worldgen.WaterLevels;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * P0-13: {@link WaterLevels#levelAt} on the lake-test fixture. A column takes its nearest
 * {@code water_level} cell's value, else the nearest wet cell's within the reach (one noise cell).
 */
class WaterLevelsTest {

    private static final int REACH = 4;

    @Test
    void ownCellWins() {
        RasterSampler levels = WorldData.open(Fixtures.world("lake1024")).layer(WorldData.WATER_LEVEL);
        assertEquals(120.0, WaterLevels.levelAt(levels, 0, 0, REACH));
        assertEquals(120.0, WaterLevels.levelAt(levels, 100, 0, REACH));   // the channel
        assertEquals(120.0, WaterLevels.levelAt(levels, 1024, 0, REACH));  // x wraps
    }

    @Test
    void shoreColumnsTakeTheLevelWithinReach() {
        RasterSampler levels = WorldData.open(Fixtures.world("lake1024")).layer(WorldData.WATER_LEVEL);
        // A shore pocket of the first GameTest run: d = 2.64 from the footprint (ground 118 below
        // the water surface 120), but its nearest cell (28, 34) is at d = 4.05, outside the rim.
        assertTrue(Double.isNaN(WaterLevels.levelAt(levels, 27, 33, 0)));
        assertEquals(120.0, WaterLevels.levelAt(levels, 27, 33, REACH));
        // The last wet cell straight north is (0, 44) (d = 4): (0, 48) is within reach, (0, 50) not.
        assertEquals(120.0, WaterLevels.levelAt(levels, 0, 48, REACH));
        assertTrue(Double.isNaN(WaterLevels.levelAt(levels, 0, 50, REACH)));
        assertTrue(Double.isNaN(WaterLevels.levelAt(levels, 0, 200, REACH)));   // the plain
    }

    @Test
    void topWaterBlockFollowsSeaLevelConvention() {
        assertEquals(119, WaterLevels.topWaterBlock(120.0));
        assertEquals(62, WaterLevels.topWaterBlock(63.0));
        assertEquals(120, WaterLevels.topWaterBlock(120.5));
    }
}
