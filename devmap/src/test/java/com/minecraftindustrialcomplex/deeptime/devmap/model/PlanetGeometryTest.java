// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.model;

import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeMap;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeTurn;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeWorld;
import com.minecraftindustrialcomplex.deeptime.worlddata.Manifest;
import com.minecraftindustrialcomplex.deeptime.worlddata.PetroffGuyou;
import com.minecraftindustrialcomplex.deeptime.worlddata.Projection;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
import org.junit.jupiter.api.Test;

import java.nio.file.Path;
import java.util.List;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The dev map's geometry on Deep Time's own projections: the play planet's Petroff-Guyou world
 * (b 6656, lambda_c 146°, the manifest of /data/ban/play/be17a7cf) and a Lambert world of C 16384.
 * Round trips block to latitude/longitude to block, the wrap seam, the poles and cone points
 * against the play manifest's values, the readout's distances and the F3 longitude conventions.
 * On the cube (cube plan G14), hand-made cubes of side 1024 and the synthetic cube world's manifest
 * ({@code cube1024}): the frame is the net's window, the void is off the planet, the seam is the
 * slide, the edges are the border's segments, the poles lie in N and S and the cone points are the
 * eight corners at their twelve images.
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
            assertEquals(0.0, PlanetGeometry.seamDistance(p, half, 0), 1e-9);
            assertEquals(0.0, PlanetGeometry.seamDistance(p, -half, 0), 1e-9);
            assertEquals(26.0, PlanetGeometry.seamDistance(p, half - 26, 0), 1e-9);
            assertEquals(26.0, PlanetGeometry.seamDistance(p, -half + 26, 0), 1e-9);
            assertEquals(half, PlanetGeometry.seamDistance(p, 0, 0), 1e-9);
            assertEquals(PlanetGeometry.seamDistance(p, 1234, 0), PlanetGeometry.seamDistance(p, 1234 + 3 * p.period(), 0), 1e-6);
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
    void theGuardCatchesAForwardThatMisses() {
        // PG with a forward 0.01° off in latitude: strictBlockOf refuses it; blockOf (the render
        // paths') logs it once and still returns the forward's result rather than crash the client.
        WorldProjection off = (WorldProjection) java.lang.reflect.Proxy.newProxyInstance(
                WorldProjection.class.getClassLoader(), new Class<?>[]{WorldProjection.class},
                (proxy, m, args) -> m.getName().equals("blockOf")
                        ? PG.blockOf((double) args[0] + 0.01, (double) args[1])
                        : m.invoke(PG, args));
        assertThrows(IllegalStateException.class, () -> PlanetGeometry.strictBlockOf(off, 10.0, 20.0));
        PlanetGeometry.FORWARD_MISS_LOGGED.set(false);
        assertEquals(off.blockOf(10.0, 20.0)[1], PlanetGeometry.blockOf(off, 10.0, 20.0)[1]);
        assertTrue(PlanetGeometry.FORWARD_MISS_LOGGED.get(), "the miss was logged");
        assertEquals(PG.blockOf(10.0, 20.0)[0], PlanetGeometry.strictBlockOf(PG, 10.0, 20.0)[0]);
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

    // ---- the cube (cube plan G14) ----

    /** A cube's side in the hand-made cubes (CubeNetViewsTest's). */
    static final int S = 1024;

    /** A cube of side 1024 turned 30° about the polar axis: its poles are inside N and S. */
    static final WorldProjection.Cube CUBE = new WorldProjection.Cube(new CubeWorld(CubeLayout.of(S), CubeMap.world(), CubeTurn.polar(30.0), 1.0));

    /** The synthetic cube world's projection, from its manifest ({@code cube1024}, {@code ./gradlew testWorlds}). */
    static WorldProjection.Cube fixtureCube() {
        String dir = System.getProperty("deeptime.testWorlds");
        assertTrue(dir != null, "deeptime.testWorlds is not set (run through Gradle)");
        WorldProjection p = Manifest.read(Path.of(dir).resolve("cube1024")).projection();
        assertTrue(p instanceof WorldProjection.Cube, "cube1024 is " + p.kind());
        return (WorldProjection.Cube) p;
    }

    /** The cubes the tests walk: the turned one, the unturned one and the fixture's. */
    static List<WorldProjection.Cube> cubes() {
        return List.of(CUBE, new WorldProjection.Cube(new CubeWorld(CubeLayout.of(S), CubeMap.world(), CubeTurn.IDENTITY, 1.0)),
                fixtureCube());
    }

    @Test
    void aCubesFrameIsItsNetWindow() {
        for (WorldProjection.Cube c : cubes()) {
            double h = c.side() / 2.0;
            MapFrame f = MapFrame.of(c, MapFrame.DEFAULT_WIDTH);
            assertEquals(-3 * h, f.x0());
            assertEquals(-3 * h, f.z0());
            assertEquals(-3 * h, PlanetGeometry.west(c));
            assertEquals(2048, f.w());
            assertEquals(1536, f.h(), "4s by 3s");
            assertEquals(4.0 * c.side() / 2048, f.blocksPerPx(), 1e-12);
            // The frame's columns wrap with the period from the window's west side.
            assertEquals(0, f.column(-3 * h + 0.1));
            assertEquals(2047, f.column(5 * h - 0.1));
            assertEquals(0, f.column(5 * h + 0.1));
        }
        // The polar worlds keep the seam as the west side.
        assertEquals(-6656.0, PlanetGeometry.west(PG));
        assertEquals(-8192.0, PlanetGeometry.west(LAMBERT));
    }

    @Test
    void aCubesVoidIsOffThePlanetAndXWrapsCentred() {
        double h = S / 2.0, period = 4.0 * S;
        for (double[] q : new double[][]{{0, 0}, {4 * h, 0}, {-2 * h, 0.5 * h}, {2 * h, -0.9 * h}, {0, -2 * h}, {0.5 * h, 2.9 * h},
                {4 * h + period, 0}, {-2 * h - 3 * period, 0}})
            assertTrue(PlanetGeometry.onPlanet(CUBE, q[0], q[1]), q[0] + ", " + q[1]);
        for (double[] q : new double[][]{{2 * h, -2 * h}, {-2 * h, -2 * h}, {4 * h, -2 * h}, {2 * h, 2 * h}, {-2 * h, 2 * h},
                {4 * h, 2 * h}, {2 * h + period, -2 * h}})
            assertFalse(PlanetGeometry.onPlanet(CUBE, q[0], q[1]), q[0] + ", " + q[1]);
        assertTrue(PlanetGeometry.onPlanet(PG, 0, 1e6));
        assertEquals("F0", PlanetGeometry.faceName(CUBE, 0, 0));
        assertEquals("N", PlanetGeometry.faceName(CUBE, 0, -2 * h));
        assertEquals("F2", PlanetGeometry.faceName(CUBE, 4 * h, 0));
        assertEquals("void", PlanetGeometry.faceName(CUBE, 2 * h, -2 * h));
        assertEquals(null, PlanetGeometry.faceName(PG, 0, 0));
        // The centred wrap is Toroidal World's frame [-2s, 2s), not the window.
        for (double x = -3 * period; x < 3 * period; x += 377.3) {
            double w = PlanetGeometry.wrapCentred(CUBE, x);
            assertTrue(w >= -period / 2 && w < period / 2, x + " -> " + w);
            double laps = (x - w) / period;
            assertEquals(Math.rint(laps), laps, 1e-9);
            assertEquals(PG.wrapX(x), PlanetGeometry.wrapCentred(PG, x));
        }
        // Period 8h: the frame runs [-4h, 4h), so 4.5h (on F2 in the window) is -3.5h, and 4h is -4h.
        assertEquals(-3.5 * h, PlanetGeometry.wrapCentred(CUBE, 4.5 * h), 1e-9);
        assertEquals(-4 * h, PlanetGeometry.wrapCentred(CUBE, 4 * h), 1e-9);
        assertEquals(-4 * h, PlanetGeometry.wrapCentred(CUBE, -4 * h), 1e-9);
        assertEquals(3.5 * h, PlanetGeometry.wrapCentred(CUBE, 3.5 * h), 1e-9);
    }

    @Test
    void aCubesLatitudeAndLongitudeRoundTripWithTheF3Longitude() {
        for (WorldProjection.Cube c : cubes()) {
            double h = c.side() / 2.0;
            double worst = 0;
            int n = 0;
            for (double x = -3 * h + 0.37; x < 5 * h; x += c.side() / 37.3)
                for (double z = -3 * h + 0.41; z < 3 * h; z += c.side() / 41.7) {
                    if (!PlanetGeometry.onPlanet(c, x, z))
                        continue;
                    double[] raw = c.latLon(x, z);
                    double[] ll = PlanetGeometry.displayLatLon(c, x, z);
                    // The F3 line's cube branch: the simulation's longitude folded to [-180, 180).
                    assertTrue(ll[1] >= -180 && ll[1] < 180, "lon " + ll[1]);
                    assertEquals(raw[1] >= 180 ? raw[1] - 360 : raw[1], ll[1], 1e-9);
                    double dLon = PlanetGeometry.simLon(c, ll[1]) - raw[1];
                    assertEquals(0.0, dLon - 360 * Math.round(dLon / 360), 1e-9);
                    double[] back = PlanetGeometry.strictBlockOf(c, raw[0], raw[1]);
                    worst = Math.max(worst, Math.hypot(PlanetGeometry.wrapCentred(c, back[0] - x), back[1] - z));
                    n++;
                }
            assertTrue(n > 700, "samples " + n);
            assertTrue(worst < 0.01, "worst round trip " + worst + " blocks");
        }
    }

    @Test
    void aCubesSeamIsTheSlideAndItsEdgesAreTheBorderSegments() {
        for (WorldProjection.Cube c : cubes()) {
            double h = c.side() / 2.0, period = c.period();
            assertEquals(0.0, PlanetGeometry.seamDistance(c, -3 * h, 0), 1e-9);
            assertEquals(0.0, PlanetGeometry.seamDistance(c, 5 * h, 0.3 * h), 1e-9);
            assertEquals(26.0, PlanetGeometry.seamDistance(c, -3 * h + 26, 10), 1e-9);
            assertEquals(26.0, PlanetGeometry.seamDistance(c, 5 * h - 26, -10), 1e-9);
            assertEquals(10.0, PlanetGeometry.seamDistance(c, -3 * h, h + 10), 1e-9, "past the slide's end");
            assertEquals(3 * h, PlanetGeometry.seamDistance(c, 0, 0), 1e-9);
            assertEquals(PlanetGeometry.seamDistance(c, 1234, 5), PlanetGeometry.seamDistance(c, 1234 - 3 * period, 5), 1e-6);
            // The border's twelve segments: N's east side, F1's top meeting F2's, F0's centre (its
            // nearest segments end at its corners), N's centre.
            assertEquals(40.0, PlanetGeometry.edgeDistance(c, h - 40, -2 * h), 1e-9);
            assertEquals(40.0, PlanetGeometry.edgeDistance(c, 3 * h, -h + 40), 1e-9);
            assertEquals(40.0, PlanetGeometry.edgeDistance(c, -2 * h, h - 40), 1e-9);
            assertEquals(h * Math.sqrt(2), PlanetGeometry.edgeDistance(c, 0, 0), 1e-9);
            assertEquals(h, PlanetGeometry.edgeDistance(c, 0, -2 * h), 1e-9);
            assertEquals(40.0, PlanetGeometry.edgeDistance(c, 4.5 * h + period, h - 40), 1e-9);
        }
        // The polar worlds' seam ignores z.
        assertEquals(PlanetGeometry.seamDistance(PG, 1234, 0), PlanetGeometry.seamDistance(PG, 1234, 3000), 0.0);
    }

    @Test
    void aCubesBorderOutlineSlideAndJoinedSides() {
        double h = S / 2.0;
        List<PlanetGeometry.Polyline> at0 = PlanetGeometry.borderOutline(CUBE, 0.0);
        assertEquals(2, at0.size());
        // At d = 0 every vertex lies on the planet's boundary (a border segment or a joined corner):
        // inside A(0), and on no point is the outline inside the planet by more than rounding.
        var border = com.minecraftindustrialcomplex.deeptime.wrap.CubeBorder.of(S);
        for (PlanetGeometry.Polyline l : at0)
            for (int i = 1; i < l.xs().length; i++)
                for (double t = 0; t <= 1.0; t += 0.125) {
                    double x = l.xs()[i - 1] + t * (l.xs()[i] - l.xs()[i - 1]), z = l.zs()[i - 1] + t * (l.zs()[i] - l.zs()[i - 1]);
                    assertTrue(border.inside(x, z, 0.0), x + ", " + z);
                    assertEquals(0.0, border.distanceToSegment(x, z), 1e-9, x + ", " + z);
                }
        // d = 40: every vertex is in A(40) and, but for F0's reachable corners (where the outline runs
        // through the notch), a block further out along the diagonal is not.
        List<PlanetGeometry.Polyline> at40 = PlanetGeometry.borderOutline(CUBE, 40.0);
        for (PlanetGeometry.Polyline l : at40)
            for (int i = 0; i < l.xs().length; i++) {
                double x = l.xs()[i], z = l.zs()[i];
                assertTrue(border.inside(x, z, 40.0), x + ", " + z);
                boolean f0Corner = Math.abs(Math.abs(x) - h) < 1e-9 && Math.abs(Math.abs(z) - h) < 1e-9;
                if (!f0Corner)
                    assertFalse(border.inside(x + Math.signum(x), z + Math.signum(z), 40.0), "vertex " + x + ", " + z + " is not on the outline");
            }
        PlanetGeometry.Polyline slide = PlanetGeometry.slide(CUBE);
        assertArrayEquals(new double[]{-3 * h, -3 * h}, slide.xs());
        assertArrayEquals(new double[]{-h, h}, slide.zs());
        assertEquals(5, PlanetGeometry.joinedSides(CUBE).size());
        for (PlanetGeometry.Polyline l : PlanetGeometry.joinedSides(CUBE)) {
            // Both sides of a joined side are on the planet.
            double mx = 0.5 * (l.xs()[0] + l.xs()[1]), mz = 0.5 * (l.zs()[0] + l.zs()[1]);
            boolean vertical = l.xs()[0] == l.xs()[1];
            assertTrue(PlanetGeometry.onPlanet(CUBE, mx + (vertical ? 3 : 0), mz + (vertical ? 0 : 3)));
            assertTrue(PlanetGeometry.onPlanet(CUBE, mx - (vertical ? 3 : 0), mz - (vertical ? 0 : 3)));
        }
        assertTrue(PlanetGeometry.borderOutline(PG, 0.0).isEmpty());
        assertEquals(null, PlanetGeometry.slide(PG));
        assertEquals(6, PlanetGeometry.pieces(CUBE).size());
        assertEquals(1, PlanetGeometry.pieces(PG).size());
    }

    @Test
    void aCubesPolesAreInsideNAndS() {
        for (WorldProjection.Cube c : cubes()) {
            List<PlanetGeometry.Pole> poles = PlanetGeometry.poles(c);
            assertEquals(2, poles.size());
            for (PlanetGeometry.Pole pole : poles) {
                assertFalse(pole.line());
                assertEquals(1, pole.images().size());
                double[] im = pole.images().get(0);
                assertTrue(PlanetGeometry.onPlanet(c, im[0], im[1]), pole.name());
                assertEquals(pole.latDeg(), c.latLon(im[0], im[1])[0], 1e-6, pole.name());
                // A turn about the polar axis (or none) keeps the poles at N's and S's centres.
                if (c.world().turn() == CubeTurn.IDENTITY || c == CUBE) {
                    assertEquals(pole.latDeg() > 0 ? "N" : "S", PlanetGeometry.faceName(c, im[0], im[1]), pole.name());
                    assertEquals(0.0, im[0], 1e-6);
                    assertEquals(pole.latDeg() > 0 ? -c.side() : c.side(), im[1], 1e-6);
                }
            }
            PlanetGeometry.Pole north = poles.get(0);
            assertEquals("north", north.name());
            double[] n = north.images().get(0);
            PlanetGeometry.NearestPole np = PlanetGeometry.nearestPole(c, n[0] + 10, n[1]);
            assertEquals("north", np.byBlocks());
            assertEquals(10.0, np.blocks(), 1e-6);
            assertEquals("north", np.byArc());
        }
        // Past the net's corners there is no planet point.
        double h = S / 2.0;
        PlanetGeometry.NearestPole none = PlanetGeometry.nearestPole(CUBE, 2 * h, -3.5 * h);
        assertEquals("none", none.byBlocks());
        assertTrue(Double.isNaN(none.blocks()));
    }

    @Test
    void aCubesConePointsAreItsEightCornersAtTwelveImages() {
        for (WorldProjection.Cube c : cubes()) {
            double h = c.side() / 2.0;
            List<PlanetGeometry.Corner> corners = PlanetGeometry.corners(c);
            assertEquals(12, corners.size());
            assertEquals(12, PlanetGeometry.conePoints(c).size());
            java.util.Map<String, List<PlanetGeometry.Corner>> byName = new java.util.TreeMap<>();
            for (PlanetGeometry.Corner k : corners) {
                assertTrue(k.x() >= -3 * h && k.x() < 5 * h, k.name() + " at x " + k.x());
                byName.computeIfAbsent(k.name(), q -> new java.util.ArrayList<>()).add(k);
            }
            assertEquals(List.of("n0", "n1", "n2", "n3", "s0", "s1", "s2", "s3"), List.copyOf(byName.keySet()));
            for (var e : byName.entrySet()) {
                List<PlanetGeometry.Corner> ims = e.getValue();
                boolean f0 = Set.of("n0", "n3", "s0", "s3").contains(e.getKey());
                assertEquals(f0 ? 1 : 2, ims.size(), e.getKey());
                for (PlanetGeometry.Corner k : ims) {
                    assertEquals(f0, k.reachable(), e.getKey());
                    if (f0)
                        assertEquals(h, Math.abs(k.x()), 0.0);
                    // Every image is the same point of the planet.
                    double[] a = c.unit(ims.get(0).x(), ims.get(0).z()), b = c.unit(k.x(), k.z());
                    double dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
                    assertEquals(1.0, dot, 1e-9, e.getKey() + " at " + k.x() + ", " + k.z());
                }
            }
        }
    }
}
