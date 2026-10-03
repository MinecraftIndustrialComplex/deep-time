// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.dh;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The planet-to-Distant-Horizons conversion and the rule that keeps the player's own value
 * (docs/science/world-scale-and-bake.md, "Distant Horizons world curvature"). The conversion is
 * checked against the geometry, not against a table of its own outputs: the sphere Distant Horizons
 * draws for the ratio must be the planet's within the rounding of an integer ratio.
 */
class DhCurvatureTest {

    @Test
    void theRadiusIsTheCircumferenceOverTwoPi() {
        assertEquals(2607.6, DhCurvature.radiusBlocks(16_384), 0.05);
        assertEquals(5215.2, DhCurvature.radiusBlocks(32_768), 0.05);
        assertEquals(1.0, DhCurvature.radiusBlocks(1_000) * 2.0 * Math.PI / 1_000, 1e-12);
    }

    @Test
    void theRatioDrawsTheSpherePlanetWithinOneRatioStep() {
        for (long c : new long[]{9_000, 16_384, 20_000, 32_768, 65_536, 131_072, 500_000}) {
            int ratio = DhCurvature.ratioFor(c);
            assertTrue(ratio >= DhCurvature.MIN_RATIO && ratio <= DhCurvature.MAX_RATIO, c + ": ratio " + ratio);
            double drawn = DhCurvature.radiusOfRatio(ratio), planet = DhCurvature.radiusBlocks(c);
            // An integer ratio is a relative step of at most 1/(2 ratio) in the radius.
            assertEquals(planet, drawn, planet * 0.5 / ratio + 1e-9, c + ": the drawn sphere is the planet's");
        }
    }

    @Test
    void theTwoWorldSizes() {
        // Distant Horizons draws a sphere of 6,371,000 / ratio blocks: 6,371,000 / 2,607.6 = 2,443.2.
        assertEquals(2443, DhCurvature.ratioFor(16_384));
        assertEquals(1222, DhCurvature.ratioFor(32_768));
        // Twice the circumference is half the ratio (up to the rounding).
        assertEquals(DhCurvature.ratioFor(16_384) / 2.0, DhCurvature.ratioFor(32_768), 0.5);
    }

    @Test
    void theDropIsTheShadersCosineAndAboutADSquaredOverTwoR() {
        double r16 = DhCurvature.radiusOfRatio(DhCurvature.ratioFor(16_384)), r32 = DhCurvature.radiusOfRatio(DhCurvature.ratioFor(32_768));
        for (double r : new double[]{r16, r32}) {
            assertEquals(0.0, DhCurvature.dropBlocks(r, 0), 1e-12);
            for (double d : new double[]{256, 512, 1024}) {
                double drop = DhCurvature.dropBlocks(r, d);
                assertEquals(d * d / (2 * r), drop, d * d / (2 * r) * 0.01, "d^2 / 2R to within 1 % at d = " + d);
                assertTrue(drop < d * d / (2 * r), "the cosine falls a little less than the parabola");
            }
            // Four times the distance falls sixteen times as far (the parabola), to within the cosine's correction.
            assertEquals(16.0, DhCurvature.dropBlocks(r, 1024) / DhCurvature.dropBlocks(r, 256), 0.1);
        }
        // The planet twice as big falls half as far at the same distance.
        assertEquals(2.0, DhCurvature.dropBlocks(r16, 512) / DhCurvature.dropBlocks(r32, 512), 0.02);
    }

    @Test
    void sizesDistantHorizonsCannotDrawAreClampedOrLeftFlat() {
        assertEquals(0, DhCurvature.ratioFor(0));
        assertEquals(0, DhCurvature.ratioFor(-5));
        // Smaller than the strongest curve Distant Horizons offers (1,274 blocks): the strongest.
        assertEquals(DhCurvature.MAX_RATIO, DhCurvature.ratioFor(4_096));
        assertEquals(DhCurvature.MAX_RATIO, DhCurvature.ratioFor(8_000));
        // Flatter than the weakest (127,420 blocks): flat, never a value in 1..49, which Distant
        // Horizons would rewrite to 50 in the player's config file.
        assertEquals(0, DhCurvature.ratioFor(1_000_000));
        for (long c = 1; c < 5_000_000; c += 997) {
            int r = DhCurvature.ratioFor(c);
            assertTrue(r == 0 || (r >= 50 && r <= 5000), c + " gave " + r);
        }
        // The edges: the weakest curve is a circumference of 800,6xx blocks.
        assertEquals(DhCurvature.MIN_RATIO, DhCurvature.ratioFor(800_000));
        assertEquals(0, DhCurvature.ratioFor(810_000));
    }

    // ---- the rule --------------------------------------------------------------------------

    /** Distant Horizons' two layers for one setting. */
    static final class FakeSlot implements DhCurvature.Slot {
        int player;
        Integer api;
        boolean overridable = true;
        int sets, clears;
        /** What the user would see and the shader would use. */
        int effective() {
            return api != null ? api : player;
        }

        @Override
        public int playerValue() {
            return player;
        }

        @Override
        public int defaultValue() {
            return 0;
        }

        @Override
        public Integer apiValue() {
            return api;
        }

        @Override
        public boolean overridable() {
            return overridable;
        }

        @Override
        public boolean setApi(int ratio) {
            sets++;
            api = ratio;
            return true;
        }

        @Override
        public boolean clearApi() {
            clears++;
            api = null;
            return true;
        }
    }

    @Test
    void aDeepTimeLevelGetsThePlanetAndAVanillaLevelIsFlat() {
        FakeSlot dh = new FakeSlot();
        DhCurvature.Controller c = new DhCurvature.Controller();
        assertEquals(DhCurvature.Outcome.UNCHANGED, c.reconcile(dh, 0, true), "a vanilla world is never curved");
        assertEquals(0, dh.effective());
        assertEquals(DhCurvature.Outcome.SET, c.reconcile(dh, 16_384, true));
        assertEquals(2443, dh.effective());
        assertEquals(0, dh.player, "the player's own value is never written");
        // Joining a 32k world next: the new ratio replaces ours.
        assertEquals(DhCurvature.Outcome.SET, c.reconcile(dh, 32_768, true));
        assertEquals(1222, dh.effective());
        // Leaving to a level with no planet hands the setting back, at once and for good.
        assertEquals(DhCurvature.Outcome.CLEARED, c.reconcile(dh, 0, true));
        assertEquals(0, dh.effective());
        assertNull(dh.api);
        assertEquals(DhCurvature.Outcome.UNCHANGED, c.reconcile(dh, 0, true));
        assertEquals(2, dh.sets);
        assertEquals(1, dh.clears);
    }

    @Test
    void theSameLevelIsNotSetTwice() {
        FakeSlot dh = new FakeSlot();
        DhCurvature.Controller c = new DhCurvature.Controller();
        for (int i = 0; i < 50; i++)
            c.reconcile(dh, 16_384, true);
        assertEquals(1, dh.sets);
    }

    @Test
    void aValueThePlayerSetIsLeftAlone() {
        FakeSlot dh = new FakeSlot();
        dh.player = 1200;
        DhCurvature.Controller c = new DhCurvature.Controller();
        assertEquals(DhCurvature.Outcome.PLAYER_VALUE, c.reconcile(dh, 16_384, true));
        assertEquals(1200, dh.effective());
        assertEquals(0, dh.sets);
        assertEquals(0, dh.clears);
        c.release(dh);
        assertEquals(1200, dh.effective());
        assertEquals(0, dh.clears, "nothing of ours to take off");
    }

    @Test
    void aValueTheyTypeLaterReleasesOurs() {
        FakeSlot dh = new FakeSlot();
        DhCurvature.Controller c = new DhCurvature.Controller();
        c.reconcile(dh, 16_384, true);
        assertEquals(2443, dh.effective());
        dh.player = 800; // typed into Distant Horizons' screen, as a config-file edit would
        assertEquals(DhCurvature.Outcome.CLEARED, c.reconcile(dh, 16_384, true));
        assertEquals(800, dh.effective(), "their value shows");
        assertEquals(DhCurvature.Outcome.PLAYER_VALUE, c.reconcile(dh, 16_384, true));
        assertEquals(800, dh.effective());
    }

    @Test
    void theSwitchTurnsItOffAndHandsBack() {
        FakeSlot dh = new FakeSlot();
        DhCurvature.Controller c = new DhCurvature.Controller();
        c.reconcile(dh, 16_384, true);
        assertEquals(DhCurvature.Outcome.CLEARED, c.reconcile(dh, 16_384, false));
        assertEquals(0, dh.effective());
        assertEquals(DhCurvature.Outcome.UNCHANGED, c.reconcile(dh, 16_384, false));
        assertEquals(1, dh.sets);
    }

    @Test
    void anotherModsOverrideIsNeitherReplacedNorRemoved() {
        FakeSlot dh = new FakeSlot();
        dh.api = 300; // some other mod
        DhCurvature.Controller c = new DhCurvature.Controller();
        assertEquals(DhCurvature.Outcome.FOREIGN_OVERRIDE, c.reconcile(dh, 16_384, true));
        assertEquals(300, dh.effective());
        c.release(dh);
        assertEquals(300, dh.effective());
        assertEquals(0, dh.sets + dh.clears);

        // Ours first, then another mod takes the layer over: ours is not taken off again.
        FakeSlot dh2 = new FakeSlot();
        DhCurvature.Controller c2 = new DhCurvature.Controller();
        c2.reconcile(dh2, 16_384, true);
        dh2.api = 777;
        c2.reconcile(dh2, 0, true);
        assertEquals(777, dh2.effective());
        assertEquals(0, dh2.clears);
        assertNull(c2.ours());
    }

    @Test
    void aSettingDistantHorizonsDoesNotLetUsOverrideIsLeftAlone() {
        FakeSlot dh = new FakeSlot();
        dh.overridable = false;
        DhCurvature.Controller c = new DhCurvature.Controller();
        assertEquals(DhCurvature.Outcome.NOT_OVERRIDABLE, c.reconcile(dh, 16_384, true));
        assertEquals(0, dh.sets);
        assertFalse(dh.api != null);
    }

    @Test
    void aPlanetTooBigToCurveSetsNothing() {
        FakeSlot dh = new FakeSlot();
        DhCurvature.Controller c = new DhCurvature.Controller();
        assertEquals(DhCurvature.Outcome.UNCHANGED, c.reconcile(dh, 2_000_000, true));
        assertEquals(0, dh.sets);
    }
}
