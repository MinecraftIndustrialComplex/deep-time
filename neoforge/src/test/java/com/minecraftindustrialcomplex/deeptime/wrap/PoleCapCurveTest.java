// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrap;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/** The pole cap's geometry and curves (plan §5.1's table, §7.11; P1-18's chosen curves). */
class PoleCapCurveTest {

    private static final PoleCapCurve C16 = new PoleCapCurve(16384, 75, 80);
    private static final PoleCapCurve C32 = new PoleCapCurve(32768, 75, 80);

    @Test
    void linesMatchThePlanTable() {
        // Plan §5.1: z of ±75° is ±3,413 (16k) / ±6,827 (32k); z of ±80° is ±3,641 / ±7,282.
        assertEquals(3413, Math.round(C16.capZ()));
        assertEquals(3641, Math.round(C16.stopZ()));
        assertEquals(6827, Math.round(C32.capZ()));
        assertEquals(7282, Math.round(C32.stopZ()));
        assertEquals(90.0, PoleCapCurve.latitudeDeg(16384, -4096), 1e-12);
        assertEquals(-90.0, PoleCapCurve.latitudeDeg(16384, 4096), 1e-12);
    }

    @Test
    void latitudeRoundTripsAndNorthIsMinusZ() {
        for (double lat = -90; lat <= 90; lat += 0.37) {
            double z = PoleCapCurve.zOfLatitude(16384, lat);
            assertEquals(lat, PoleCapCurve.latitudeDeg(16384, z), 1e-9);
        }
        assertTrue(PoleCapCurve.latitudeDeg(16384, -100) > 0, "north is -z");
    }

    @Test
    void longitudeFoldsIntoHalfOpenRange() {
        assertEquals(0.0, PoleCapCurve.longitudeDeg(16384, 0), 1e-12);
        assertEquals(-180.0, PoleCapCurve.longitudeDeg(16384, -8192), 1e-12);
        assertEquals(-180.0, PoleCapCurve.longitudeDeg(16384, 8192), 1e-12);   // x = C/2 is the seam = -C/2
        assertEquals(90.0, PoleCapCurve.longitudeDeg(16384, 4096), 1e-12);
        assertEquals(PoleCapCurve.longitudeDeg(16384, 100), PoleCapCurve.longitudeDeg(16384, 100 + 16384), 1e-9);
    }

    @Test
    void nothingBelowTheCapInBothHemispheres() {
        for (double lat : new double[]{0, 30, 60, 74.0, 74.5, 74.999}) {
            for (double sign : new double[]{1, -1}) {
                double z = PoleCapCurve.zOfLatitude(16384, sign * lat);
                assertEquals(0.0, C16.push(z), "push at " + sign * lat);
                assertEquals(0.0, C16.storm(z), "storm at " + sign * lat);
                assertEquals(0.0, C16.wind(z), "wind at " + sign * lat);
                assertEquals(PoleCapCurve.Zone.OPEN, C16.zone(z));
            }
        }
    }

    @Test
    void curvesGrowWithLatitudeAndAreSymmetric() {
        double lastPush = 0, lastStorm = 0, lastWind = 0;
        for (int i = 1; i <= 100; i++) {
            double lat = 75 + i * 0.05;
            double zN = PoleCapCurve.zOfLatitude(16384, lat), zS = -zN;
            assertTrue(C16.push(zN) > lastPush && C16.storm(zN) > lastStorm && C16.wind(zN) > lastWind, "not increasing at " + lat);
            assertEquals(C16.push(zN), C16.push(zS), 1e-15);
            assertEquals(C16.storm(zN), C16.storm(zS), 1e-15);
            lastPush = C16.push(zN);
            lastStorm = C16.storm(zN);
            lastWind = C16.wind(zN);
        }
        // The stop line: full strength.
        double zStop = PoleCapCurve.zOfLatitude(16384, 80);
        assertEquals(PoleCapCurve.PUSH_MAX, C16.push(zStop), 1e-12);
        assertEquals(1.0, C16.storm(zStop), 1e-12);
        assertEquals(PoleCapCurve.WIND_MAX, C16.wind(zStop), 1e-12);
        // Documented values (PoleCapCurve's class comment).
        assertEquals(0.0089, C16.push(PoleCapCurve.zOfLatitude(16384, 76)), 1e-4);
        assertEquals(0.0354, C16.push(PoleCapCurve.zOfLatitude(16384, 77.5)), 1e-4);
        assertEquals(0.0716, C16.push(PoleCapCurve.zOfLatitude(16384, 79)), 1e-4);
        assertEquals(0.20, C16.storm(PoleCapCurve.zOfLatitude(16384, 75.5)), 0.005);
        assertEquals(0.62, C16.storm(PoleCapCurve.zOfLatitude(16384, 77.5)), 0.005);
    }

    @Test
    void equatorwardPointsToTheEquator() {
        assertEquals(1.0, PoleCapCurve.equatorward(-3500));    // north: toward +z
        assertEquals(-1.0, PoleCapCurve.equatorward(3500));    // south: toward -z
    }

    @Test
    void stopClampsBothHemispheres() {
        double stop = C16.stopZ();
        assertEquals(PoleCapCurve.Zone.STOP, C16.zone(stop + 0.01));
        assertEquals(PoleCapCurve.Zone.STOP, C16.zone(-stop - 0.01));
        assertEquals(PoleCapCurve.Zone.STORM, C16.zone(stop - 0.01));
        assertEquals(stop - 0.05, C16.clampZ(stop + 40, 0.05), 1e-12);
        assertEquals(-stop + 0.05, C16.clampZ(-stop - 40, 0.05), 1e-12);
        assertEquals(1000.0, C16.clampZ(1000, 0.05));
        // A clamped position is inside the stop.
        assertEquals(PoleCapCurve.Zone.STORM, C16.zone(C16.clampZ(stop + 3, 0.05)));
    }

    @Test
    void refusesNonsenseLines() {
        assertThrows(IllegalArgumentException.class, () -> new PoleCapCurve(16384, 80, 75));
        assertThrows(IllegalArgumentException.class, () -> new PoleCapCurve(16384, 75, 91));
        assertThrows(IllegalArgumentException.class, () -> new PoleCapCurve(0, 75, 80));
    }
}
