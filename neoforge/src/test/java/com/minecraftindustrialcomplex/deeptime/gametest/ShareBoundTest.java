// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/** {@link ShareBound}: the census floors compare with the share's one-sided upper bound. */
class ShareBoundTest {

    @Test
    void theWilsonBoundMatchesTheFormulaAndStaysAboveTheShare() {
        // 82 of 132 (0.621) at z 2.326: (p + z²/2n + z·sqrt(p(1-p)/n + z²/4n²)) / (1 + z²/n).
        assertEquals(0.7128, ShareBound.wilsonUpper(82, 132, ShareBound.Z_99), 1e-4);
        for (int n : new int[] {1, 40, 132, 1000})
            for (int h = 0; h <= n; h += Math.max(1, n / 7)) {
                double u = ShareBound.wilsonUpper(h, n, ShareBound.Z_99);
                assertTrue(u >= h / (double) n && u <= 1.0, h + "/" + n + " -> " + u);
            }
        assertEquals(1.0, ShareBound.wilsonUpper(0, 0, ShareBound.Z_99));
        // More columns, a tighter bound.
        assertTrue(ShareBound.wilsonUpper(400, 1000, ShareBound.Z_99) < ShareBound.wilsonUpper(40, 100, ShareBound.Z_99));
    }

    @Test
    void samplingNoiseAtTheFloorPassesAndABrokenMappingFails() {
        // The MIC gate of 2026-10-19: C 0.621 on 132 columns against 0.63 is within the sample's
        // noise (one standard error is 0.042).
        assertFalse(ShareBound.below(82, 132, 0.63, ShareBound.Z_99));
        // A broken mapping, shares of 0.1 to 0.3, fails C's floor on the fewest columns asserted
        // (40) and on 132.
        assertTrue(ShareBound.below(12, 40, 0.63, ShareBound.Z_99));
        assertTrue(ShareBound.below(40, 132, 0.63, ShareBound.Z_99));
        assertTrue(ShareBound.below(4, 40, 0.34, ShareBound.Z_99));
        // A real shortfall on many columns still fails: half of 132 under 0.63.
        assertTrue(ShareBound.below(66, 132, 0.63, ShareBound.Z_99));
    }

    @Test
    void theLowerBoundMirrorsTheUpper() {
        // The two bounds of h of n are symmetric about one half: lower(h, n) = 1 - upper(n - h, n).
        for (int n : new int[] {1, 40, 132, 1000})
            for (int h = 0; h <= n; h += Math.max(1, n / 7)) {
                double l = ShareBound.wilsonLower(h, n, ShareBound.Z_99);
                assertEquals(1.0 - ShareBound.wilsonUpper(n - h, n, ShareBound.Z_99), l, 1e-12, h + "/" + n);
                assertTrue(l >= 0.0 && l <= h / (double) n, h + "/" + n + " -> " + l);
            }
        assertEquals(0.0, ShareBound.wilsonLower(0, 0, ShareBound.Z_99));
    }

    @Test
    void theIceCapsIceSpikesCeilingsPassTheFixAndFailTheBypass() {
        // Shares measured on 2026-10-20 (docs/science/biomes.md, "Ice spikes and the rare variants"; ledger
        // BIO-162), checked from the fewest columns asserted (40) up to a large planet's thousands.
        int[] sizes = {40, 132, 1000, 20000};
        // The ice cap's steep ground (erosion E0 and E1): after the fix at most 0.018, before it 0.175 to 0.212
        // (vanilla's near-inland low slice).
        double steep = BiomeGameTests.STEEP_ICE_SPIKES_CEILING;
        for (int n : sizes) {
            assertFalse(above(0.018, n, steep), "steep 0.018 of " + n);
            for (double p : new double[] {0.175, 0.212})
                assertTrue(above(p, n, steep), "steep " + p + " of " + n);
        }
        // Its flats against the pack's own share: MIC after the fix at most 0.200, before it 0.648 to 0.706
        // (Terralith's near-inland slot); vanilla at most 0.343 either way (its bypass was on the steep ground).
        double terralith = BiomeGameTests.OWN_FLATS_ICE_SPIKES_TERRALITH, vanilla = BiomeGameTests.OWN_FLATS_ICE_SPIKES_VANILLA;
        for (int n : sizes) {
            assertFalse(above(0.200, n, terralith), "MIC flats 0.200 of " + n);
            for (double p : new double[] {0.648, 0.706})
                assertTrue(above(p, n, terralith), "MIC flats " + p + " of " + n);
            assertFalse(above(0.343, n, vanilla), "vanilla flats 0.343 of " + n);
        }
        // A share at its ceiling is the sample's noise, not a failure.
        assertFalse(above(0.345, 20000, terralith));
        assertFalse(above(0.05, 40, steep));
    }

    private static boolean above(double share, int n, double ceiling) {
        return ShareBound.above((int) Math.round(share * n), n, ceiling, ShareBound.Z_99);
    }
}
