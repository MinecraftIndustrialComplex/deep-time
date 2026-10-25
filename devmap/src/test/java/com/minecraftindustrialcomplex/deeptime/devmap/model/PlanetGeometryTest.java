// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.model;

import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeMap;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeTurn;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeWorld;
import com.minecraftindustrialcomplex.deeptime.worlddata.Manifest;
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
 * The dev map's geometry on Deep Time's own projections: a Lambert-frame test level of C 16384
 * (the analytic test terrains; the Petroff-Guyou play planet's cases went with cube plan §7.4,
 * 2026-10-24). Round trips block to latitude/longitude to block, the wrap seam, the poles, the
 * readout's distances and the F3 longitude convention. On the cube (cube plan G14), hand-made cubes of side 1024 and the synthetic cube world's manifest
 * ({@code cube1024}): the frame is the net's window, the void is off the planet, the seam is the
 * slide, the edges are the border's segments, the poles lie in N and S and the cone points are the
 * eight corners at their twelve images.
 */
class PlanetGeometryTest {

    static final WorldProjection LAMBERT = new WorldProjection.Lambert(16384);

    /** Same point of the planet. */
    static double distance(WorldProjection p, double[] a, double[] b) {
        return Math.hypot(p.wrapX(a[0] - b[0]), a[1] - b[1]);
    }

    @Test
    void latLonToBlockToLatLon() {
        double worst = 0;
        for (double lat = -89.5; lat <= 89.5; lat += 1.7)
            for (double lon = 0.05; lon < 360; lon += 2.9) {
                double[] b = LAMBERT.blockOf(lat, lon);
                double[] ll = LAMBERT.latLon(b[0], b[1]);
                double dLon = ll[1] - lon;
                dLon -= 360 * Math.round(dLon / 360);
                worst = Math.max(worst, Math.max(Math.abs(ll[0] - lat), Math.abs(dLon) * Math.cos(Math.toRadians(lat))));
            }
        assertTrue(worst < 1e-5, "worst " + worst + "°");
    }

    @Test
    void blockToLatLonToBlockLambert() {
        double worst = 0;
        double zLim = LAMBERT.radius() * Math.sin(Math.toRadians(89.0));
        for (double x = -8192 + 0.3; x < 8192; x += 77.7)
            for (double z = -zLim; z <= zLim; z += 33.3) {
                double[] ll = LAMBERT.latLon(x, z);
                double[] back = LAMBERT.blockOf(ll[0], ll[1]);
                worst = Math.max(worst, distance(LAMBERT, LAMBERT.canonical(x, z), back));
            }
        assertTrue(worst < 1e-4, "worst " + worst);
    }

    @Test
    void seamAndWrap() {
        WorldProjection p = LAMBERT;
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
            assertTrue(arc < 3 * blockDeg, "z " + z + ": " + arc + "° across the seam");
        }
        // The polar seam ignores z.
        assertEquals(PlanetGeometry.seamDistance(p, 1234, 0), PlanetGeometry.seamDistance(p, 1234, 2000), 0.0);
        MapFrame f = MapFrame.of(p, 2048);
        assertEquals(0, f.column(-8192));
        assertEquals(2047, f.column(8191.9));
        assertEquals(0, f.column(8192));
        assertEquals(f.column(100), f.column(100 + 16384));
        assertEquals(f.column(100), f.column(100 - 2 * 16384));
        assertEquals(0, f.row(-1e6));
        assertEquals(f.h() - 1, f.row(1e6));
    }

    static double arcDeg(double[] a, double[] b) {
        double la = Math.toRadians(a[0]), lb = Math.toRadians(b[0]), dl = Math.toRadians(b[1] - a[1]);
        double c = Math.sin(la) * Math.sin(lb) + Math.cos(la) * Math.cos(lb) * Math.cos(dl);
        return Math.toDegrees(Math.acos(Math.max(-1, Math.min(1, c))));
    }

    @Test
    void lambertPolesAreLinesAndItHasNoConePoints() {
        // The pole is the line z = zMin (north) / zMax (south).
        List<PlanetGeometry.Pole> lp = PlanetGeometry.poles(LAMBERT);
        assertEquals(2, lp.size());
        assertTrue(lp.get(0).line());
        assertEquals("north", lp.get(0).name());
        assertEquals(LAMBERT.zMin(), lp.get(0).images().get(0)[1]);
        assertEquals(LAMBERT.zMax(), lp.get(1).images().get(0)[1]);
        assertEquals(90.0, LAMBERT.latLon(0, LAMBERT.zMin())[0], 1e-9);
        assertEquals(-90.0, LAMBERT.latLon(0, LAMBERT.zMax())[0], 1e-9);
        assertTrue(PlanetGeometry.conePoints(LAMBERT).isEmpty());
        // Lambert's forward is closed form.
        for (double t = 0; t < 360.0; t += 7.25)
            assertTrue(PlanetGeometry.forwardError(LAMBERT, LAMBERT.blockOf(0.0, t), 0.0, t) < 1e-9);
    }

    @Test
    void theGuardCatchesAForwardThatMisses() {
        // A forward 0.01° off in latitude: strictBlockOf refuses it; blockOf (the render paths')
        // logs it once and still returns the forward's result rather than crash the client.
        WorldProjection off = (WorldProjection) java.lang.reflect.Proxy.newProxyInstance(
                WorldProjection.class.getClassLoader(), new Class<?>[]{WorldProjection.class},
                (proxy, m, args) -> m.getName().equals("blockOf")
                        ? LAMBERT.blockOf((double) args[0] + 0.01, (double) args[1])
                        : m.invoke(LAMBERT, args));
        assertThrows(IllegalStateException.class, () -> PlanetGeometry.strictBlockOf(off, 10.0, 20.0));
        PlanetGeometry.FORWARD_MISS_LOGGED.set(false);
        assertEquals(off.blockOf(10.0, 20.0)[1], PlanetGeometry.blockOf(off, 10.0, 20.0)[1]);
        assertTrue(PlanetGeometry.FORWARD_MISS_LOGGED.get(), "the miss was logged");
        assertEquals(LAMBERT.blockOf(10.0, 20.0)[0], PlanetGeometry.strictBlockOf(LAMBERT, 10.0, 20.0)[0]);
    }

    @Test
    void nearestPole() {
        // Lambert: the distance to the pole line.
        PlanetGeometry.NearestPole np = PlanetGeometry.nearestPole(LAMBERT, 500, LAMBERT.zMax() - 40);
        assertEquals("south", np.byBlocks());
        assertEquals(40.0, np.blocks(), 1e-9);
        assertEquals("south", np.byArc());
        assertEquals(40.0, PlanetGeometry.edgeDistance(LAMBERT, 500, LAMBERT.zMax() - 40), 1e-9);
        // At the equator both poles are far, the arc is 90°.
        np = PlanetGeometry.nearestPole(LAMBERT, 1234, 0);
        assertEquals(90.0, np.arcDeg(), 1e-6);
        assertEquals(Math.PI / 2 * LAMBERT.radius(), np.arcBlocks(), 1e-6);
    }

    @Test
    void f3LongitudeConvention() {
        // The F3 line (PlanetDebugLine) shows the simulation's longitude folded to [-180, 180), on a
        // Lambert-frame test level as on the cube.
        for (double sim = 0; sim < 360; sim += 7.3) {
            double d = PlanetGeometry.displayLon(LAMBERT, sim);
            assertTrue(d >= -180 && d < 180);
            assertEquals(sim >= 180 ? sim - 360 : sim, d, 1e-9);
            assertEquals(sim, PlanetGeometry.simLon(LAMBERT, d), 1e-9);
        }
        for (double x = -8192; x < 8192; x += 97.1) {
            double sim = LAMBERT.latLon(x, 0)[1];
            assertEquals(sim >= 180 ? sim - 360 : sim, PlanetGeometry.displayLatLon(LAMBERT, x, 0)[1], 1e-9);
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
        // A Lambert-frame level keeps the seam as the west side.
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
        assertTrue(PlanetGeometry.onPlanet(LAMBERT, 0, 1e6));
        assertEquals("F0", PlanetGeometry.faceName(CUBE, 0, 0));
        assertEquals("N", PlanetGeometry.faceName(CUBE, 0, -2 * h));
        assertEquals("F2", PlanetGeometry.faceName(CUBE, 4 * h, 0));
        assertEquals("void", PlanetGeometry.faceName(CUBE, 2 * h, -2 * h));
        assertEquals(null, PlanetGeometry.faceName(LAMBERT, 0, 0));
        // The centred wrap is Toroidal World's frame [-2s, 2s), not the window.
        for (double x = -3 * period; x < 3 * period; x += 377.3) {
            double w = PlanetGeometry.wrapCentred(CUBE, x);
            assertTrue(w >= -period / 2 && w < period / 2, x + " -> " + w);
            double laps = (x - w) / period;
            assertEquals(Math.rint(laps), laps, 1e-9);
            assertEquals(LAMBERT.wrapX(x), PlanetGeometry.wrapCentred(LAMBERT, x));
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
        assertTrue(PlanetGeometry.borderOutline(LAMBERT, 0.0).isEmpty());
        assertEquals(null, PlanetGeometry.slide(LAMBERT));
        assertEquals(6, PlanetGeometry.pieces(CUBE).size());
        assertEquals(1, PlanetGeometry.pieces(LAMBERT).size());
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
