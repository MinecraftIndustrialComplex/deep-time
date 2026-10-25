// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;

/** The rule that buries a structure placed at an absolute height (SCL-200); no Minecraft types needed. */
class StructureBurialTest {

    private static final int MIN_Y = -512, COVER = StructureBurial.COVER;

    private static int plan(int bottom, int top, int ground) {
        return StructureBurial.plan(bottom, top, ground, MIN_Y);
    }

    @Test
    void benTrialChamberOverATrenchIsMovedDownUnderTheFloor() {
        // The reported start: box y -40..17, over a sea floor near y -149. The top ends 8 blocks under it.
        int dy = plan(-40, 17, -149);
        assertEquals(-149 - COVER - 17, dy);
        assertEquals(-149 - COVER, 17 + dy, "the top sits `cover` blocks under the ground");
        assertEquals(-40 + dy, -214, "the whole structure moves, bottom included");
    }

    @Test
    void aStructureThatIsAlreadyBuriedStaysWhereItIs() {
        assertEquals(0, plan(-200, -100, 50), "deep inside high ground");
        assertEquals(0, plan(-40, 17, 17 + COVER), "the top exactly `cover` under the ground");
        assertEquals(-1, plan(-40, 17, 17 + COVER - 1), "one block short: one block down");
        assertEquals(0, plan(-40, 17, 400), "a trial chamber under a mountain is not touched");
    }

    @Test
    void aStartThatWouldReachTheWorldFloorIsDropped() {
        // Ground at -490: the top must end at -498, which puts a 57-block structure's bottom at -555 < min_y.
        assertEquals(StructureBurial.DROP, plan(-40, 17, -490));
        // The bottom may reach min_y + 8 exactly: top 17, bottom -40 (span 57); ground g puts the bottom at
        // g - COVER - 57.
        int ground = MIN_Y + StructureBurial.FLOOR_MARGIN + COVER + 57;
        assertEquals(ground - COVER - 17, plan(-40, 17, ground), "bottom lands exactly on min_y + 8: kept");
        assertEquals(StructureBurial.DROP, plan(-40, 17, ground - 1), "one block lower: dropped");
    }

    @Test
    void theRuleDoesNotOverflowOnAnEmptyGroundSample() {
        assertEquals(0, plan(-40, 17, Integer.MAX_VALUE), "no ground sample: nothing to bury against");
        assertEquals(StructureBurial.DROP, plan(-40, 17, Integer.MIN_VALUE + 100));
    }

    /**
     * On a cube world the ground past a turning edge is skipped: the void's chunks place no piece, so a trench the
     * void shows beside N's west edge (the partner face's ground) neither buries a start by the seam nor drops it.
     * Before the filter the trench set the least ground.
     */
    @Test
    void theGroundPastATurningEdgeIsSkipped() {
        CubeLayout cube = CubeLayout.of(1024);
        int west = -512;
        StructureBurial.Height trench = (x, z) -> cube.faceOfBlock(x, z) == null ? -300.0 : -20.0;
        assertEquals(-300, StructureBurial.leastGround(trench, west - 40, west + 70, -1100, -1000), "the plain rule reads the void");
        assertEquals(-20, StructureBurial.leastGround(StructureBurial.onFaces(trench, cube), west - 40, west + 70, -1100, -1000));
        assertEquals(Integer.MAX_VALUE, StructureBurial.leastGround(StructureBurial.onFaces(trench, cube), west - 90, west - 1, -1100, -1000),
                "a box wholly off the faces has nothing to bury against");
        assertTrue(StructureBurial.onFaces(trench, null) == trench, "no cube, no filter");
    }

    @Test
    void theLeastGroundIsFoundAtAnEdgeAndACorner() {
        // A dip of one column at a corner of a 111 x 70 box: the grid has to include the corners.
        StructureBurial.Height dip = (x, z) -> x == 656 + 110 && z == 1628 ? -300.0 : -50.0;
        assertEquals(-300, StructureBurial.leastGround(dip, 656, 656 + 110, 1628, 1628 + 70));
        StructureBurial.Height edge = (x, z) -> x == 656 && z == 1628 + 70 ? -80.5 : 10.0;
        assertEquals(-81, StructureBurial.leastGround(edge, 656, 656 + 110, 1628, 1628 + 70), "floor of the least height");
    }

    @Test
    void theGridNeverLeavesAGapWiderThanItsStepAndKeepsBothEnds() {
        for (int span : new int[]{0, 1, 15, 16, 17, 110, 255}) {
            for (int step : new int[]{1, 7, 16}) {
                int[] axis = StructureBurial.axis(-33, -33 + span, step);
                assertEquals(-33, axis[0]);
                assertEquals(-33 + span, axis[axis.length - 1]);
                for (int i = 1; i < axis.length; i++) {
                    assertTrue(axis[i] > axis[i - 1], "strictly increasing");
                    assertTrue(axis[i] - axis[i - 1] <= step, "span " + span + " step " + step + " gap at " + i);
                }
            }
        }
        assertEquals(1, StructureBurial.axis(5, 5, 16).length, "a one-block box is one sample");
    }

    @Test
    void aStartOf255BlocksIsSampledOnSeventeenByseventeenColumns() {
        List<int[]> seen = new ArrayList<>();
        StructureBurial.leastGround((x, z) -> {
            seen.add(new int[]{x, z});
            return 0.0;
        }, 0, 255, 0, 255);
        assertEquals(17 * 17, seen.size(), "a 255 x 255 box: 16 blocks apart, edges included");
    }

    @Test
    void nanSamplesAreSkippedAndAnAllNanBoxHasNoGround() {
        assertEquals(-4, StructureBurial.leastGround((x, z) -> x == 0 ? Double.NaN : -4.0, 0, 32, 0, 32));
        assertEquals(Integer.MAX_VALUE, StructureBurial.leastGround((x, z) -> Double.NaN, 0, 32, 0, 32));
    }

    private static boolean covers(String json) {
        return StructureBurial.covers(JsonParser.parseString(json));
    }

    @Test
    void theRuleLooksAtStructuresThatPlaceByAnAbsoluteHeightBelowGround() {
        // vanilla trial chambers
        assertTrue(covers("""
                {"type":"minecraft:jigsaw","step":"underground_structures","terrain_adaptation":"encapsulate",
                 "start_height":{"type":"minecraft:uniform","min_inclusive":{"absolute":-40},"max_inclusive":{"absolute":-20}}}"""));
        // vanilla ancient city
        assertTrue(covers("""
                {"type":"minecraft:jigsaw","step":"underground_decoration","start_height":{"absolute":-27}}"""));
        // a modded jigsaw kind in an underground step
        assertTrue(covers("""
                {"type":"cataclysm:cataclysm_jigsaw","step":"underground_structures","start_height":{"absolute":-30}}"""));
        // the ordinary mineshaft, and the stronghold whatever its step
        assertTrue(covers("""
                {"type":"minecraft:mineshaft","step":"underground_structures","mineshaft_type":"normal"}"""));
        assertTrue(covers("""
                {"type":"minecraft:stronghold","step":"surface_structures"}"""));
        assertTrue(covers("""
                {"type":"minecraft:stronghold","step":"strongholds"}"""));
    }

    @Test
    void theRuleLeavesStructuresThatFollowTheGroundOrAreNotItsOwn() {
        // buried treasure: the box sits at y 90 and the structure follows the heightmap
        assertFalse(covers("""
                {"type":"minecraft:buried_treasure","step":"underground_structures"}"""));
        // trail ruins: an absolute start height but projected to the surface
        assertFalse(covers("""
                {"type":"minecraft:jigsaw","step":"underground_structures","start_height":{"absolute":-15},
                 "project_start_to_heightmap":"WORLD_SURFACE_WG"}"""));
        // the mesa mineshaft is set between sea level and the ground
        assertFalse(covers("""
                {"type":"minecraft:mineshaft","step":"underground_structures","mineshaft_type":"mesa"}"""));
        // a village: above ground, in the surface step
        assertFalse(covers("""
                {"type":"minecraft:jigsaw","step":"surface_structures","start_height":{"absolute":0},
                 "project_start_to_heightmap":"WORLD_SURFACE_WG"}"""));
        // Terralith's underground sites are registered in the surface step (a known gap, SCL-200)
        assertFalse(covers("""
                {"type":"minecraft:jigsaw","step":"surface_structures","start_height":{"type":"minecraft:uniform",
                 "min_inclusive":{"absolute":-40},"max_inclusive":{"absolute":40}}}"""));
        // a type this rule does not know, in an underground step, with no start height of its own
        assertFalse(covers("""
                {"type":"iceandfire:fire_dragon_cave","step":"underground_structures"}"""));
        assertFalse(covers("[]"));
        assertFalse(covers("\"minecraft:jigsaw\""));
    }

    @Test
    void thePropertySwitchesTheRuleOffAndIsReadEachTime() {
        String old = System.getProperty(StructureBurial.PROPERTY);
        try {
            System.clearProperty(StructureBurial.PROPERTY);
            assertTrue(StructureBurial.enabled(), "on by default");
            System.setProperty(StructureBurial.PROPERTY, "false");
            assertFalse(StructureBurial.enabled());
            System.setProperty(StructureBurial.PROPERTY, "FALSE");
            assertFalse(StructureBurial.enabled());
            System.setProperty(StructureBurial.PROPERTY, "true");
            assertTrue(StructureBurial.enabled());
        } finally {
            if (old == null)
                System.clearProperty(StructureBurial.PROPERTY);
            else
                System.setProperty(StructureBurial.PROPERTY, old);
        }
    }
}
