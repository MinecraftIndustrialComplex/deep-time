// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * {@link DetailNoise} is Rust's {@code deeptime_format::detail::Noise} bit for bit: the values Rust's
 * {@code detail::tests::known_values} and {@code cube_known_values} pin (a change of either is a change
 * of the world-data contract), the periodicity in x and the range, and on a cube world the fold.
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

    /** The 4k test cube's net (s = 1024), as Rust's {@code detail::tests::cube_fixture}. */
    private static DetailNoise cubeNoise() {
        return new DetailNoise(0xDEAD_BEEF_0123_4567L, 4096, 1729, 3, 32.0, CubeLayout.shared(1024));
    }

    /** A cube world's noise (cube plan G3): the values Rust's {@code detail::tests::cube_known_values} pins. */
    @Test
    void cubeKnownValuesMatchRust() {
        DetailNoise n = cubeNoise();
        assertEquals(LATTICE3, n.lattice3(7, -3, 11, 5));
        double[][] at = {{100.0, -50.0}, {-900.0, 300.0}, {10.0, -900.0}, {-200.0, 1000.0}, {2000.0, 100.0}, {-600.0, -700.0}};
        for (int k = 0; k < at.length; k++)
            assertEquals(CUBE_VALUES[k], n.at(at[k][0], at[k][1]), "(" + at[k][0] + ", " + at[k][1] + ")");
        assertEquals(0.0, n.at(1000.0, -1600.0), "deep ocean past the corners");
    }

    private static final double LATTICE3 = -0.016565307453047184;
    private static final double[] CUBE_VALUES = {
            -0.16032818137761118, 0.2689732692952809, 0.15544098362422432,
            0.23472733008415383, 0.03172617598775018, 0.017139648300822326};

    /** A block off the cube's faces reads its fold partner's noise exactly, a lap east the same block's. */
    @Test
    void theCubeNoiseIsItsFoldPartners() {
        DetailNoise n = cubeNoise();
        CubeLayout cube = n.cube();
        int outside = 0;
        for (long x = -1536; x < 2560; x += 23) {
            for (long z = -1700; z < 1700; z += 23) {
                double here = n.at(x, z);
                assertTrue(here >= -1 && here <= 1, "in range: " + here);
                if (cube.fold(x, z) instanceof CubeLayout.Outside o) {
                    outside++;
                    assertEquals(n.at(o.x(), o.z()), here, "(" + x + ", " + z + ") shows (" + o.x() + ", " + o.z() + ")");
                }
                assertEquals(here, n.at(x + 4096, z), "a lap east");
            }
        }
        assertTrue(outside > 5_000, "blocks off the faces: " + outside);
    }

    @Test
    void anUnsignedSeedParses() {
        // The manifest writes the seed as a decimal u64; seeds above 2^63 wrap into Java's long.
        assertEquals(-6L, Long.parseUnsignedLong("18446744073709551610"));
    }
}
