// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrap;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/** The pole cap's geometry and curves (plan §5.1's table, §7.11; P1-18's chosen curves). */
class PoleCapCurveTest {

    // The defaults (Ben, 2026-10-03: the band from 70°, the stop at 80°).
    private static final PoleCapCurve C16 = new PoleCapCurve(16384, PoleCapCurve.DEFAULT_CAP_DEG, PoleCapCurve.DEFAULT_STOP_DEG);
    private static final PoleCapCurve C32 = new PoleCapCurve(32768, PoleCapCurve.DEFAULT_CAP_DEG, PoleCapCurve.DEFAULT_STOP_DEG);

    @Test
    void linesMatchThePlanTable() {
        // The Lambert projection (docs/design/equal-area-projection.md): z = −R·sin φ, R = C/2π, so
        // z of ±70° is ±2,450 (16k) / ±4,901 (32k) and z of ±80° is ±2,568 / ±5,136: a band of 118 /
        // 235 blocks; the poles are at ±2,607.6 / ±5,215.2. (Equirectangular, 75° and 80° were
        // ±3,413 / ±3,641 at 16k.)
        assertEquals(70.0, C16.capDeg());
        assertEquals(2450, Math.round(C16.capZ()));
        assertEquals(2568, Math.round(C16.stopZ()));
        assertEquals(4901, Math.round(C32.capZ()));
        assertEquals(5136, Math.round(C32.stopZ()));
        assertEquals(118, Math.round(C16.stopZ() - C16.capZ()));
        double pole = 16384 / (2 * Math.PI);
        assertEquals(90.0, PoleCapCurve.latitudeDeg(16384, -pole), 1e-12);
        assertEquals(-90.0, PoleCapCurve.latitudeDeg(16384, pole), 1e-12);
        assertEquals(90.0, PoleCapCurve.latitudeDeg(16384, -4096), 1e-12, "clamped past the pole");
        assertEquals(30.0, PoleCapCurve.latitudeDeg(16384, -pole / 2), 1e-9, "sin 30° = 1/2");
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
            double lat = 70 + i * 0.1;
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
        assertEquals(0.0089, C16.push(PoleCapCurve.zOfLatitude(16384, 72)), 1e-4);
        assertEquals(0.0354, C16.push(PoleCapCurve.zOfLatitude(16384, 75)), 1e-4);
        assertEquals(0.0716, C16.push(PoleCapCurve.zOfLatitude(16384, 78)), 1e-4);
        assertEquals(0.20, C16.storm(PoleCapCurve.zOfLatitude(16384, 71)), 0.005);
        assertEquals(0.62, C16.storm(PoleCapCurve.zOfLatitude(16384, 75)), 0.005);
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
