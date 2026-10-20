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
    void theIceCapsIceSpikesCeilingPassesTheFixAndFailsTheBypass() {
        // Shares of the ice cap on ice spikes measured on 2026-10-20 (docs/science/biomes.md, "Ice spikes
        // and the rare variants"; ledger BIO-162): after the fix at most 0.071, before it 0.153 to 0.254.
        double ceiling = 0.10; // BiomeGameTests.FORBIDDEN_CEILINGS for BiomeCensus.EF_ICE_SPIKES
        for (int n : new int[] {40, 132, 1000, 20000})
            assertFalse(ShareBound.above((int) Math.round(0.071 * n), n, ceiling, ShareBound.Z_99), "0.071 of " + n);
        for (int n : new int[] {1000, 20000})
            for (double p : new double[] {0.153, 0.183, 0.254})
                assertTrue(ShareBound.above((int) Math.round(p * n), n, ceiling, ShareBound.Z_99), p + " of " + n);
        // On the fewest columns asserted (40) only the larger excess is told from the sample's noise.
        assertFalse(ShareBound.above(7, 40, ceiling, ShareBound.Z_99));
        assertTrue(ShareBound.above(10, 40, ceiling, ShareBound.Z_99));
    }
}
