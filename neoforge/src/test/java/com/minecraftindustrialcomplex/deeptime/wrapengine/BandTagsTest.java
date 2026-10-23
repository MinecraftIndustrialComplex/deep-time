// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

import org.junit.jupiter.api.Test;

import java.util.Random;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The band's tag table (docs/design/wrap-engine.md §12.8): what a motion does to a position, an offset
 * and a direction index. The GameTests check the same turns against Minecraft's own {@code Rotation}.
 */
class BandTagsTest {

    private static final int DOWN = 0, UP = 1, NORTH = 2, SOUTH = 3, WEST = 4, EAST = 5;

    /** A motion's quarter turn takes north to east (x east, z south: (dx, dz) to (−dz, dx)). */
    @Test
    void directionsTurnClockwiseSeenFromAbove() {
        CellIsometry k1 = new CellIsometry(1, 7, -3);
        assertEquals(EAST, BandTags.turnDirection(NORTH, k1));
        assertEquals(SOUTH, BandTags.turnDirection(EAST, k1));
        assertEquals(WEST, BandTags.turnDirection(SOUTH, k1));
        assertEquals(NORTH, BandTags.turnDirection(WEST, k1));
        assertEquals(SOUTH, BandTags.turnDirection(NORTH, new CellIsometry(2, 0, 0)));
        assertEquals(WEST, BandTags.turnDirection(NORTH, new CellIsometry(3, 0, 0)));
        for (int k = 0; k < 4; k++) {
            CellIsometry g = new CellIsometry(k, 5, 9);
            assertEquals(UP, BandTags.turnDirection(UP, g), "up stays up");
            assertEquals(DOWN, BandTags.turnDirection(DOWN, g), "down stays down");
            assertEquals(17, BandTags.turnDirection(17, g), "an unknown value is left alone");
            for (int d = NORTH; d <= EAST; d++)
                assertEquals(d, BandTags.turnDirection(BandTags.turnDirection(d, g), g.inverse()), "a turn and its inverse cancel");
        }
    }

    /** Positions move with the motion and offsets turn with it: an offset between two positions stays that offset, turned. */
    @Test
    void positionsMoveAndOffsetsTurnTogether() {
        Random r = new Random(11);
        for (int i = 0; i < 200; i++) {
            CellIsometry g = new CellIsometry(r.nextInt(4), r.nextInt(4001) - 2000, r.nextInt(4001) - 2000);
            int[] p = {r.nextInt(2001) - 1000, r.nextInt(400) - 64, r.nextInt(2001) - 1000};
            int[] d = {r.nextInt(33) - 16, r.nextInt(9) - 4, r.nextInt(33) - 16};
            int[] gp = BandTags.movePos(p, g);
            assertArrayEquals(new int[]{g.applyX(p[0], p[2]), p[1], g.applyZ(p[0], p[2])}, gp, "a position moves as the motion moves a block");
            assertArrayEquals(p, BandTags.movePos(gp, g.inverse()), "and back");
            int[] q = {p[0] + d[0], p[1] + d[1], p[2] + d[2]};
            int[] gq = BandTags.movePos(q, g);
            assertArrayEquals(new int[]{gq[0] - gp[0], gq[1] - gp[1], gq[2] - gp[2]}, BandTags.turnOffset(d, g), "an offset turns with its two ends");
            // A direction is an offset of one block.
            for (int dir = NORTH; dir <= EAST; dir++) {
                int[] unit = dir == NORTH ? new int[]{0, 0, -1} : dir == SOUTH ? new int[]{0, 0, 1} : dir == WEST ? new int[]{-1, 0, 0} : new int[]{1, 0, 0};
                int[] t = BandTags.turnOffset(unit, g);
                int want = BandTags.turnDirection(dir, g);
                int[] wantUnit = want == NORTH ? new int[]{0, 0, -1} : want == SOUTH ? new int[]{0, 0, 1} : want == WEST ? new int[]{-1, 0, 0} : new int[]{1, 0, 0};
                assertArrayEquals(wantUnit, t, "a direction turns as its unit offset does");
            }
        }
    }

    @Test
    void vanillaNeedsTwoRows() {
        assertEquals(2, BandTags.ROWS.size());
        assertEquals(BandTags.Kind.BLOCK_POS, BandTags.rowsFor("minecraft:end_gateway").get(0).kind());
        assertEquals("exit_portal", BandTags.rowsFor("minecraft:end_gateway").get(0).key());
        assertEquals(BandTags.Kind.DIRECTION, BandTags.rowsFor("minecraft:brushable_block").get(0).kind());
        assertEquals("hit_direction", BandTags.rowsFor("minecraft:brushable_block").get(0).key());
        assertTrue(BandTags.rowsFor("minecraft:chest").isEmpty());
        assertTrue(BandTags.rowsFor("minecraft:sign").isEmpty());
    }
}
