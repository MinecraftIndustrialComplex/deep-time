// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.dh;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * When Deep Time's far-terrain generator may pass Toroidal World's Distant Horizons guard
 * ({@link TwDhGuardRule}; Ben, 2026-10-02 "Mixin around TW's guard"): only where its terrain meets
 * the level's chunks at the seam, i.e. the level's east-west loop is the world data's
 * circumference and nothing else loops.
 */
class TwDhGuardRuleTest {

    private static final String OW = "minecraft:overworld";

    @Test
    void passesOnAPlanetWhoseLoopIsTheWorldDataCircumference() {
        for (int c : new int[]{4096, 16384, 32768}) {
            TwDhGuardRule.Verdict v = TwDhGuardRule.verdict(OW, OW, true, true, c, false, c);
            assertTrue(v.pass() && v.guarded(), c + ": " + v.reason());
        }
    }

    @Test
    void leavesEveryOtherCaseToToroidalWorld() {
        // The loop differs from the world data's period: far terrain would not meet the chunks at the seam.
        assertRefused(TwDhGuardRule.verdict(OW, OW, true, true, 16384, false, 32768));
        assertRefused(TwDhGuardRule.verdict(OW, OW, true, true, 8192, false, 16384));
        // The level does not loop east-west, or also loops north-south (the world data clamps at the poles).
        assertRefused(TwDhGuardRule.verdict(OW, OW, true, false, 0, false, 16384));
        assertRefused(TwDhGuardRule.verdict(OW, OW, true, true, 16384, true, 16384));
        // Not a world-data level, or a generator made for another level.
        assertRefused(TwDhGuardRule.verdict(OW, OW, true, true, 16384, false, -1));
        assertRefused(TwDhGuardRule.verdict("minecraft:the_nether", OW, true, true, 16384, false, 16384));
    }

    @Test
    void anUnshapedLevelNeedsNoPass() {
        TwDhGuardRule.Verdict v = TwDhGuardRule.verdict(OW, OW, false, false, 0, false, 16384);
        assertFalse(v.pass(), v.reason());
        assertFalse(v.guarded(), "TW's guard does not apply without a shape: " + v.reason());
    }

    private static void assertRefused(TwDhGuardRule.Verdict v) {
        assertFalse(v.pass(), v.reason());
        assertTrue(v.guarded(), v.reason());
        assertFalse(v.reason().isBlank());
    }
}
