// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;

/** The rule that keeps the carvers off the underwater face of a sea cliff (no Minecraft types needed). */
class SeaCliffGuardTest {

    private static final int SEA = 63, MIN_Y = -352, SKIN = 6;

    /** A cliff along x = 0: the ground is 100 high to the east and the sea floor -200 to the west. */
    private static final Magma.Heights CLIFF = (x, z) -> x >= 0 ? 100.0 : -200.0;

    private static int[] column(Magma.Heights h, int x0, int lx, int skin) {
        int[] around = SeaCliffGuard.lowestAround(h, x0, 0, skin);
        return SeaCliffGuard.guarded(Magma.top(h.at(x0 + lx, 0)), around[lx], SEA, MIN_Y, skin);
    }

    @Test
    void aWallOfRockIsKeptTowardsTheOpenWaterBesideACliff() {
        // Chunk at x0 = 0: the cliff's top is its first columns; the water is at x < 0 and the
        // ground east of the face is rock to y 99. Columns 0..5 lie within the skin of the open sea.
        for (int lx = 0; lx < SKIN; lx++) {
            int[] r = column(CLIFF, 0, lx, SKIN);
            assertNotNull(r, "column " + lx + " is within the skin of the cliff face");
            assertEquals(-200, r[0], "from just above the sea floor at the foot of the cliff");
            assertEquals(SEA - 1, r[1], "to the top of the water: the cliff above the sea is not guarded");
        }
        for (int lx = SKIN; lx < 16; lx++)
            assertNull(column(CLIFF, 0, lx, SKIN), "column " + lx + " is further than the skin from the face");
    }

    @Test
    void noGuardOnLevelGroundOrOnTheSeaFloorBelowTheCliff() {
        Magma.Heights flat = (x, z) -> -120.0;
        for (int lx = 0; lx < 16; lx++)
            assertNull(column(flat, 0, lx, SKIN));
        // The sea floor at the cliff's foot is its own lowest ground: nothing under it is guarded.
        for (int lx = 0; lx < 16; lx++)
            assertNull(column(CLIFF, -16, lx, SKIN));
    }

    @Test
    void aSeaFloorIsNotACliff() {
        // One block in ten and one in two (about 27 degrees): the drop within the skin is under the skin.
        for (double gradient : new double[]{0.1, 0.5}) {
            Magma.Heights slope = (x, z) -> -100.0 + gradient * x;
            for (int lx = 0; lx < 16; lx++)
                assertNull(column(slope, 100, lx, SKIN), "gradient " + gradient + " column " + lx);
        }
        // One block in one (45 degrees) is a cliff: the drop within the skin reaches the skin.
        Magma.Heights steep = (x, z) -> -100.0 + 1.0 * x;
        for (int lx = 0; lx < 16; lx++)
            assertNotNull(column(steep, 100, lx, SKIN), "45 degrees, column " + lx);
    }

    @Test
    void theGuardStopsAtTheSeaLevelAndTheFloor() {
        assertArrayEquals(new int[]{-199, SEA - 1}, SeaCliffGuard.guarded(100, -200, SEA, MIN_Y, SKIN));
        assertArrayEquals(new int[]{MIN_Y, 10}, SeaCliffGuard.guarded(10, -900, SEA, MIN_Y, SKIN));
        assertNull(SeaCliffGuard.guarded(-210, -200, SEA, MIN_Y, SKIN), "the ground is no higher than the neighbours'");
        assertNull(SeaCliffGuard.guarded(-195, -200, SEA, MIN_Y, SKIN), "a drop of 5 blocks is under the skin of 6");
        assertNotNull(SeaCliffGuard.guarded(-194, -200, SEA, MIN_Y, SKIN), "a drop of 6 blocks is a cliff");
    }

    @Test
    void theWindowIsASquareAroundEachColumn() {
        // A single low column at (5, 5): every column within 3 of it sees it, none beyond.
        Magma.Heights pit = (x, z) -> x == 5 && z == 5 ? -50.0 : 10.0;
        int[] low = SeaCliffGuard.lowestAround(pit, 0, 0, 3);
        for (int lz = 0; lz < 16; lz++)
            for (int lx = 0; lx < 16; lx++) {
                boolean near = Math.abs(lx - 5) <= 3 && Math.abs(lz - 5) <= 3;
                assertEquals(near ? Magma.top(-50.0) : Magma.top(10.0), low[lz * 16 + lx], lx + "," + lz);
            }
    }

    @Test
    void theSettingIsReadFromTheBakeNotesAndClamped() {
        assertEquals(6, SeaCliffGuard.Rules.of(JsonParser.parseString("{}").getAsJsonObject()).skin());
        assertEquals(0, SeaCliffGuard.Rules.of(JsonParser.parseString(
                "{\"bake\":{\"underground\":{\"params\":{\"sea_cliff_skin_blocks\":0}}}}").getAsJsonObject()).skin());
        assertEquals(32, SeaCliffGuard.Rules.of(JsonParser.parseString(
                "{\"bake\":{\"underground\":{\"params\":{\"sea_cliff_skin_blocks\":99}}}}").getAsJsonObject()).skin());
    }
}
