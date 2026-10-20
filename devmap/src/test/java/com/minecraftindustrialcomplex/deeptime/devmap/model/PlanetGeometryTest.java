// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.model;

import com.minecraftindustrialcomplex.deeptime.worlddata.PetroffGuyou;
import com.minecraftindustrialcomplex.deeptime.worlddata.Projection;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The dev map's geometry on Deep Time's own projections: the play planet's Petroff-Guyou world
 * (b 6656, lambda_c 146°, the manifest of /data/ban/play/be17a7cf) and a Lambert world of C 16384.
 * Round trips block to latitude/longitude to block, the wrap seam, the poles and cone points
 * against the play manifest's values, the readout's distances and the F3 longitude conventions.
 */
class PlanetGeometryTest {

    static final PetroffGuyou PG = new PetroffGuyou(6656, 146.0);
    static final WorldProjection LAMBERT = new WorldProjection.Lambert(16384);

    /** Same point of the planet: equal, or (on a Petroff-Guyou edge) the edge's mirror image. */
    static double distance(WorldProjection p, double[] a, double[] b) {
        double d = Math.hypot(p.wrapX(a[0] - b[0]), a[1] - b[1]);
        if (PlanetGeometry.petroffGuyou(p) && (Math.abs(Math.abs(a[1]) - p.zMax()) < 1e-6))
            d = Math.min(d, Math.hypot(p.wrapX(a[0] + b[0]), a[1] - b[1]));
        return d;
    }

    @Test
    void playPlanetDimensions() {
        assertEquals(13312.0, PG.period());
        assertEquals(-3328.0, PG.zMin());
        assertEquals(3328.0, PG.zMax());
        assertEquals(6656 / Math.sqrt(2 * Math.PI), PG.radius(), 1e-6, "2b² = 4πR²");
        MapFrame f = MapFrame.of(PG, MapFrame.DEFAULT_WIDTH);
        assertEquals(2048, f.w());
        assertEquals(1024, f.h());
        assertEquals(6.5, f.blocksPerPx(), 1e-12);
        assertEquals(-6656.0, f.x0());
        assertEquals(-3328.0, f.z0());
    }

    @Test
    void blockToLatLonToBlockPetroffGuyou() {
        double worst = 0;
        int n = 0;
        for (double x = -6656 + 0.37; x < 6656; x += 61.3)
            for (double z = -3328 + 0.41; z < 3328; z += 47.9) {
                double[] ll = PG.latLon(x, z);
                double[] back = PG.blockOf(ll[0], ll[1]);
                worst = Math.max(worst, distance(PG, PG.canonical(x, z), back));
                n++;
            }
        assertTrue(n > 20000);
        assertTrue(worst < 1e-4, "worst round trip " + worst + " blocks");
    }

    @Test
    void blockToLatLonToBlockOutsideTheDomainFolds() {
        // Points one lap east and beyond the north edge come back as their canonical image.
        for (double[] q : new double[][]{{100 + 13312, 200}, {-5000 - 2 * 13312, -1000}, {1234, -3328 - 50}, {-4321, 3328 + 75}}) {
            double[] ll = PG.latLon(q[0], q[1]);
            double[] back = PG.blockOf(ll[0], ll[1]);
            assertTrue(distance(PG, PG.canonical(q[0], q[1]), back) < 1e-4, q[0] + ", " + q[1]);
        }
    }

    @Test
    void latLonToBlockToLatLon() {
        for (WorldProjection p : List.of(PG, LAMBERT)) {
            double worst = 0;
            for (double lat = -89.5; lat <= 89.5; lat += 1.7)
                for (double lon = 0.05; lon < 360; lon += 2.9) {
                    double[] b = p.blockOf(lat, lon);
                    double[] ll = p.latLon(b[0], b[1]);
                    double dLon = ll[1] - lon;
                    dLon -= 360 * Math.round(dLon / 360);
                    worst = Math.max(worst, Math.max(Math.abs(ll[0] - lat), Math.abs(dLon) * Math.cos(Math.toRadians(lat))));
                }
            assertTrue(worst < 1e-5, p.kind() + ": worst " + worst + "°");
        }
    }

    @Test
    void blockToLatLonToBlockLambert() {
        double worst = 0;
        double zLim = Projection.zOfLatitude(16384, 89.0);
        for (double x = -8192 + 0.3; x < 8192; x += 77.7)
            for (double z = -Math.abs(zLim); z <= Math.abs(zLim); z += 33.3) {
                double[] ll = LAMBERT.latLon(x, z);
                double[] back = LAMBERT.blockOf(ll[0], ll[1]);
                worst = Math.max(worst, distance(LAMBERT, LAMBERT.canonical(x, z), back));
            }
        assertTrue(worst < 1e-4, "worst " + worst);
    }

    @Test
    void seamAndWrap() {
        for (WorldProjection p : List.of(PG, LAMBERT)) {
            double half = p.period() / 2;
            assertEquals(0.0, PlanetGeometry.seamDistance(p, half), 1e-9);
            assertEquals(0.0, PlanetGeometry.seamDistance(p, -half), 1e-9);
            assertEquals(26.0, PlanetGeometry.seamDistance(p, half - 26), 1e-9);
            assertEquals(26.0, PlanetGeometry.seamDistance(p, -half + 26), 1e-9);
            assertEquals(half, PlanetGeometry.seamDistance(p, 0), 1e-9);
            assertEquals(PlanetGeometry.seamDistance(p, 1234), PlanetGeometry.seamDistance(p, 1234 + 3 * p.period()), 1e-6);
            // The seam is seamless: the two sides of it are the same place on the planet, a block apart.
            for (double z = p.zMin() + 100; z < p.zMax() - 100; z += 211) {
                double[] a = p.latLon(half - 0.5, z), b = p.latLon(-half + 0.5, z);
                double arc = arcDeg(a, b);
                double blockDeg = Math.toDegrees(1.0 / p.radius());
                assertTrue(arc < 3 * blockDeg, p.kind() + " z " + z + ": " + arc + "° across the seam");
            }
        }
        MapFrame f = MapFrame.of(PG, 2048);
        assertEquals(0, f.column(-6656));
        assertEquals(2047, f.column(6655.9));
        assertEquals(0, f.column(6656));
        assertEquals(f.column(100), f.column(100 + 13312));
        assertEquals(f.column(100), f.column(100 - 2 * 13312));
        assertEquals(0, f.row(-1e6));
        assertEquals(1023, f.row(1e6));
    }

    static double arcDeg(double[] a, double[] b) {
        double la = Math.toRadians(a[0]), lb = Math.toRadians(b[0]), dl = Math.toRadians(b[1] - a[1]);
        double c = Math.sin(la) * Math.sin(lb) + Math.cos(la) * Math.cos(lb) * Math.cos(dl);
        return Math.toDegrees(Math.acos(Math.max(-1, Math.min(1, c))));
    }

    @Test
    void polesOfThePlayPlanet() {
        List<PlanetGeometry.Pole> poles = PlanetGeometry.poles(PG);
        assertEquals(2, poles.size());
        PlanetGeometry.Pole n = poles.get(0), s = poles.get(1);
        assertEquals("north", n.name());
        assertFalse(n.line());
        // The play manifest: north pole at (±3328, -3328), south at (±3328, 3328).
        assertEquals(2, n.images().size());
        assertEquals(-3328.0, n.images().get(0)[0], 1e-3);
        assertEquals(3328.0, n.images().get(1)[0], 1e-3);
        assertEquals(-3328.0, n.images().get(0)[1], 1e-3);
        assertEquals(2, s.images().size());
        assertEquals(-3328.0, s.images().get(0)[0], 1e-3);
        assertEquals(3328.0, s.images().get(1)[0], 1e-3);
        assertEquals(3328.0, s.images().get(0)[1], 1e-3);
        // Both images are the pole.
        for (double[] im : n.images())
            assertEquals(90.0, PG.latLon(im[0], im[1] + 1e-9)[0], 1e-4);
        for (double[] im : s.images())
            assertEquals(-90.0, PG.latLon(im[0], im[1] - 1e-9)[0], 1e-4);
        // Lambert: the pole is the line z = zMin (north) / zMax (south).
        List<PlanetGeometry.Pole> lp = PlanetGeometry.poles(LAMBERT);
        assertTrue(lp.get(0).line());
        assertEquals(LAMBERT.zMin(), lp.get(0).images().get(0)[1]);
        assertEquals(90.0, LAMBERT.latLon(0, LAMBERT.zMin())[0], 1e-9);
        assertEquals(-90.0, LAMBERT.latLon(0, LAMBERT.zMax())[0], 1e-9);
    }

    @Test
    void conePointsOfThePlayPlanet() {
        // The play manifest's cone points: north-midpoint (0, -3328) at 45° N 236°, north-corner
        // (±6656, -3328) at 45° N 56°, south-midpoint (0, 3328) at 45° S 236°, south-corner
        // (±6656, 3328) at 45° S 56° (simulation longitudes).
        double[][] want = {{0, -3328, 45, 236}, {-6656, -3328, 45, 56}, {6656, -3328, 45, 56},
                {0, 3328, -45, 236}, {-6656, 3328, -45, 56}, {6656, 3328, -45, 56}};
        List<double[]> cones = PlanetGeometry.conePoints(PG);
        assertEquals(6, cones.size());
        for (double[] w : want) {
            assertTrue(cones.stream().anyMatch(c -> c[0] == w[0] && c[1] == w[1]), w[0] + ", " + w[1]);
            // Just inside the domain (the cone point itself is on the edge).
            double x = Math.max(-6655.999, Math.min(6655.999, w[0]));
            double z = w[1] < 0 ? w[1] + 1e-6 : w[1] - 1e-6;
            double[] ll = PG.latLon(x, z);
            assertEquals(w[2], ll[0], 1e-3, "lat at " + w[0] + ", " + w[1]);
            double dLon = ll[1] - w[3];
            dLon -= 360 * Math.round(dLon / 360);
            assertEquals(0.0, dLon, 1e-3, "lon at " + w[0] + ", " + w[1]);
            // The forward at the cone point itself (it landed 616 blocks along the edge before
            // 2026-10-20, ledger SCL-083).
            double[] b = PG.blockOf(w[2], w[3]);
            assertEquals(w[1], b[1], 1e-3, "z of " + w[0] + ", " + w[1]);
            assertEquals(0.0, Math.min(Math.abs(PG.wrapX(b[0] - w[0])), Math.abs(PG.wrapX(b[0] + w[0]))), 1e-3,
                    "x of " + w[0] + ", " + w[1] + ": " + b[0]);
        }
        assertTrue(PlanetGeometry.conePoints(LAMBERT).isEmpty());
    }

    @Test
    void forwardOnTheExactEquatorAndDiagonals() {
        // Exact inputs on the Petroff square's diagonals and edges: the world's equator (latitude
        // exactly 0) and the meridians through the hemisphere centres (λc and λc + 180°) and the
        // cone points (λc ± 90°). Deep Time's PetroffGuyou.blockOf missed some of these on this
        // planet by up to 10.5° before 2026-10-20 (ledger SCL-083); now none may miss.
        int raw = 0, n = 0;
        double worst = 0;
        for (double t = 0; t < 360.0; t += 0.25) {
            for (double[] ll : new double[][]{{0.0, t}, {t / 2 - 90, 146.0}, {t / 2 - 90, 326.0},
                    {t / 2 - 90, 56.0}, {t / 2 - 90, 236.0}}) {
                double e = PlanetGeometry.forwardError(PG, PG.blockOf(ll[0], ll[1]), ll[0], ll[1]);
                if (e > PlanetGeometry.FORWARD_TOLERANCE_DEG)
                    raw++;
                worst = Math.max(worst, e);
                n++;
            }
        }
        System.out.println("[devmap test] Deep Time's PetroffGuyou.blockOf on " + n
                + " exact diagonal inputs: " + raw + " miss by more than 1e-5°, worst " + worst + "°");
        assertEquals(0, raw, "exact diagonal inputs the forward misses");
        assertTrue(worst <= PlanetGeometry.FORWARD_TOLERANCE_DEG, "forward worst " + worst + "°");
        // On the equator the point is on z = 0 (the equator's line).
        for (double t = 0; t < 360.0; t += 0.25)
            assertEquals(0.0, PlanetGeometry.blockOf(PG, 0.0, t)[1], 1e-6, "equator at " + t);
        // Lambert's forward is closed form.
        for (double t = 0; t < 360.0; t += 7.25)
            assertTrue(PlanetGeometry.forwardError(LAMBERT, LAMBERT.blockOf(0.0, t), 0.0, t) < 1e-9);
    }

    @Test
    void theGuardThrowsOnAForwardThatMisses() {
        // PG with a forward 0.01° off in latitude: PlanetGeometry.blockOf must refuse it.
        WorldProjection off = (WorldProjection) java.lang.reflect.Proxy.newProxyInstance(
                WorldProjection.class.getClassLoader(), new Class<?>[]{WorldProjection.class},
                (proxy, m, args) -> m.getName().equals("blockOf")
                        ? PG.blockOf((double) args[0] + 0.01, (double) args[1])
                        : m.invoke(PG, args));
        assertThrows(IllegalStateException.class, () -> PlanetGeometry.blockOf(off, 10.0, 20.0));
        assertEquals(PG.blockOf(10.0, 20.0)[0], PlanetGeometry.blockOf(PG, 10.0, 20.0)[0]);
    }

    @Test
    void nearestPole() {
        PlanetGeometry.NearestPole np = PlanetGeometry.nearestPole(PG, -3328, -3328 + 60);
        assertEquals("north", np.byBlocks());
        assertEquals(60.0, np.blocks(), 1e-3);
        assertEquals("north", np.byArc());
        assertTrue(np.arcDeg() > 0 && np.arcDeg() < 2, "arc " + np.arcDeg());
        // The other image of the same pole, across the seam: x 3328 + 10 is 10 blocks from (3328, -3328) in x.
        np = PlanetGeometry.nearestPole(PG, 3338, -3300);
        assertEquals("north", np.byBlocks());
        assertEquals(Math.hypot(10, 28), np.blocks(), 1e-3);
        np = PlanetGeometry.nearestPole(PG, 3328, 3300);
        assertEquals("south", np.byBlocks());
        assertEquals(28.0, np.blocks(), 1e-3);
        // At the equator's middle both poles are far, the arc is 90°.
        double[] eq = PG.blockOf(0.0, 146.0);
        np = PlanetGeometry.nearestPole(PG, eq[0], eq[1]);
        assertEquals(90.0, np.arcDeg(), 1e-6);
        assertEquals(Math.PI / 2 * PG.radius(), np.arcBlocks(), 1e-6);
        // Lambert: the distance to the pole line.
        np = PlanetGeometry.nearestPole(LAMBERT, 500, LAMBERT.zMax() - 40);
        assertEquals("south", np.byBlocks());
        assertEquals(40.0, np.blocks(), 1e-9);
        assertEquals(40.0, PlanetGeometry.edgeDistance(LAMBERT, 500, LAMBERT.zMax() - 40), 1e-9);
        assertEquals(60.0, PlanetGeometry.edgeDistance(PG, -3328, -3328 + 60), 1e-9);
    }

    @Test
    void f3LongitudeConventions() {
        // Petroff-Guyou: the F3 longitude is the simulation's folded to [-180, 180).
        for (double sim = 0; sim < 360; sim += 7.3) {
            double d = PlanetGeometry.displayLon(PG, sim);
            assertTrue(d >= -180 && d < 180);
            assertEquals(sim >= 180 ? sim - 360 : sim, d, 1e-9);
            assertEquals(sim, PlanetGeometry.simLon(PG, d), 1e-9);
        }
        // Lambert: the F3 line's Projection.longitudeEastDeg, which is the simulation's minus 180°.
        for (double x = -8192; x < 8192; x += 97.1) {
            double sim = LAMBERT.latLon(x, 0)[1];
            double east = Projection.longitudeEastDeg(16384, x);
            assertEquals(east, PlanetGeometry.displayLatLon(LAMBERT, x, 0)[1], 1e-9);
            assertEquals(east, PlanetGeometry.displayLon(LAMBERT, sim), 1e-6, "x " + x);
            double back = PlanetGeometry.simLon(LAMBERT, east);
            double dLon = back - sim;
            dLon -= 360 * Math.round(dLon / 360);
            assertEquals(0.0, dLon, 1e-6);
        }
        assertEquals("12.345° N 67.890° W", PlanetGeometry.formatLatLon(12.345, -67.89));
        assertEquals("0.500° S 0.000° E", PlanetGeometry.formatLatLon(-0.5, 0.0));
    }
}
