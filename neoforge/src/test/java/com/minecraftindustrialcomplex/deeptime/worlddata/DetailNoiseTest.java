// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * {@link DetailNoise} is Rust's {@code deeptime_format::detail::Noise} bit for bit: the values Rust's
 * {@code detail::tests::known_values} pins (a change of either is a change of the world-data
 * contract), the periodicity in x and the range.
 */
class DetailNoiseTest {

    @Test
    void knownValuesMatchRust() {
        DetailNoise n = new DetailNoise(0xDEAD_BEEF_0123_4567L, 16_384, 1729, 3, 32.0);
        assertEquals(-0.9207152765116788, n.lattice(7, -3, 11));
        assertEquals(-0.06020044448424485, n.fbm(-8191.0, 4095.0, 32.0, 3, 1729));
        assertEquals(0.48318066841072793, n.fbm(1234.0, -567.0, 32.0, 3, 1729));
        assertEquals(n.fbm(1234.0, -567.0, 32.0, 3, 1729), n.at(1234.0, -567.0));
    }

    @Test
    void periodicInXAndBounded() {
        for (int c : new int[]{16_384, 32_768, 4096, 1000}) {
            DetailNoise n = new DetailNoise(7, c, 1, 3, 32.0);
            for (int k = 0; k < 300; k++) {
                double x = -c / 2.0 + (k * 13.37) % c;
                double z = -c / 4.0 + (k * 7.91) % (c / 2.0);
                double a = n.at(x, z);
                assertTrue(a >= -1 && a <= 1, "in range: " + a);
                assertTrue(Math.abs(a - n.at(x + c, z)) < 1e-9, "one lap east");
            }
        }
    }

    @Test
    void anUnsignedSeedParses() {
        // The manifest writes the seed as a decimal u64; seeds above 2^63 wrap into Java's long.
        assertEquals(-6L, Long.parseUnsignedLong("18446744073709551610"));
    }
}
