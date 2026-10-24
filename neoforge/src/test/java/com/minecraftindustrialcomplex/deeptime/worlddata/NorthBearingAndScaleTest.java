// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout.Face;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout.NetEdge;
import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.SplittableRandom;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The geometry behind climate API 5 (cube plan G9): {@link WorldProjection#northBearingDeg} and
 * {@link WorldProjection#scale}. The bearing's convention; the simulation's Lambert frame (north is
 * −z, the scale 1 on the equal-area map); and the cube world
 * under three turns (none, the polar turn, and a free one that puts the poles off the face centres),
 * where north is checked against the face frames at the centres, against latitude stepped along and
 * across it, and around each pole, the scale against great-circle distances, and both across every
 * net edge into the copy beyond it.
 */
class NorthBearingAndScaleTest {

    private static final int S = 1024;

    @Test
    void theBearingIsClockwiseFromMinusZ() {
        assertEquals(0.0, WorldProjection.bearingDeg(0.0, -1.0));
        assertEquals(90.0, WorldProjection.bearingDeg(1.0, 0.0));
        assertEquals(180.0, WorldProjection.bearingDeg(0.0, 1.0));
        assertEquals(270.0, WorldProjection.bearingDeg(-1.0, 0.0));
        assertEquals(45.0, WorldProjection.bearingDeg(1.0, -1.0), 1e-12);
        assertEquals(315.0, WorldProjection.bearingDeg(-1.0, -1.0), 1e-12);
        double b = WorldProjection.bearingDeg(-1e-12, -1.0);
        assertTrue(b >= 0.0 && b < 360.0, "a hair west of -z is in [0, 360): " + b);
    }

    @Test
    void lambertNorthIsMinusZAndItsScaleIsOne() {
        WorldProjection p = new WorldProjection.Lambert(4096);
        for (double[] q : new double[][]{{0, 0}, {-2000, 300}, {1500, -600}, {10, 640}}) {
            assertEquals(0.0, p.northBearingDeg(q[0], q[1]));
            assertEquals(1.0, p.scale(q[0], q[1]));
        }
    }

    private static List<WorldProjection.Cube> cubes() {
        return List.of(
                new WorldProjection.Cube(new CubeWorld(CubeLayout.of(S), CubeMap.world(), CubeTurn.IDENTITY, 1.0)),
                new WorldProjection.Cube(new CubeWorld(CubeLayout.of(S), CubeMap.world(), CubeTurn.polar(30.0), 1.0)),
                new WorldProjection.Cube(new CubeWorld(CubeLayout.of(S), CubeMap.world(), CubeTurn.fromRows(rotation(1, 2, 3, 40.0)), 1.0)));
    }

    /** The rotation by {@code deg} about the axis (a, b, c) (Rodrigues), as rows. */
    private static double[][] rotation(double a, double b, double c, double deg) {
        double n = Math.sqrt(a * a + b * b + c * c);
        double x = a / n, y = b / n, z = c / n, t = Math.toRadians(deg), co = Math.cos(t), si = Math.sin(t), k = 1 - co;
        return new double[][]{
                {co + x * x * k, x * y * k - z * si, x * z * k + y * si},
                {y * x * k + z * si, co + y * y * k, y * z * k - x * si},
                {z * x * k - y * si, z * y * k + x * si, co + z * z * k}};
    }

    private static double dot(double[] a, double[] b) {
        return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    }

    private static double angle(double[] a, double[] b) {
        return Math.acos(Math.max(-1.0, Math.min(1.0, dot(a, b))));
    }

    private static double turnDiff(double a, double b) {
        double d = Math.abs(a - b) % 360.0;
        return Math.min(d, 360.0 - d);
    }

    /**
     * At a face's centre the conformal map's derivative is a positive multiple of the face frame, so
     * north in the net is (n·e_u, n·e_v) with n the planet's north there brought into the cube frame.
     */
    @Test
    void cubeNorthAtTheFaceCentresIsTheFaceFrames() {
        for (WorldProjection.Cube p : cubes()) {
            CubeWorld w = p.world();
            for (Face f : Face.all()) {
                double[][] fr = f.frame();
                double[] c = w.turn().toPlanet(fr[0]);
                double[] at = p.cube().pointOfLocal(f, 0.0, 0.0);
                assertTrue(angle(p.unit(at[0], at[1]), c) < 1e-7, "face " + f + " centre");
                if (Math.abs(p.latitudeDeg(at[0], at[1])) > 89.0)
                    continue;
                double[] n = w.turn().toCube(WorldProjection.localFrame(c)[1]);
                double want = WorldProjection.bearingDeg(dot(n, fr[1]), dot(n, fr[2]));
                assertEquals(0.0, turnDiff(p.northBearingDeg(at[0], at[1]), want), 1e-4, "face " + f + " north");
            }
        }
    }

    /**
     * On every face, away from the corners and poles: a step along the bearing raises latitude by the
     * step times the scale over R, a step across it leaves latitude alone, and the scale is the
     * great-circle distance per block in x and in z (the conformal map's one scale).
     */
    @Test
    void cubeNorthAndScaleAgreeWithLatitudeAndDistance() {
        SplittableRandom rnd = new SplittableRandom(7);
        for (WorldProjection.Cube p : cubes()) {
            double r = p.radius();
            int checked = 0;
            for (Face f : Face.all()) {
                for (int i = 0; i < 40; i++) {
                    double[] at = p.cube().pointOfLocal(f, rnd.nextDouble(-0.8, 0.8), rnd.nextDouble(-0.8, 0.8));
                    double x = at[0], z = at[1];
                    double sc = p.scale(x, z);
                    double gx = angle(p.unit(x - 1, z), p.unit(x + 1, z)) * r / 2.0;
                    double gz = angle(p.unit(x, z - 1), p.unit(x, z + 1)) * r / 2.0;
                    assertEquals(1.0, gx / sc, 5e-3, "scale in x at " + f + " (" + x + ", " + z + ")");
                    assertEquals(1.0, gz / sc, 5e-3, "scale in z at " + f + " (" + x + ", " + z + ")");
                    if (Math.abs(p.latitudeDeg(x, z)) > 80.0)
                        continue;
                    double b = Math.toRadians(p.northBearingDeg(x, z)), h = 2.0;
                    double dx = Math.sin(b), dz = -Math.cos(b);
                    double along = p.latitudeDeg(x + h * dx, z + h * dz) - p.latitudeDeg(x - h * dx, z - h * dz);
                    double across = p.latitudeDeg(x - h * dz, z + h * dx) - p.latitudeDeg(x + h * dz, z - h * dx);
                    double want = Math.toDegrees(2.0 * h * sc / r);
                    assertEquals(1.0, along / want, 1e-2, "latitude along north at " + f + " (" + x + ", " + z + ")");
                    assertTrue(Math.abs(across) < 1e-2 * along, "latitude across north at " + f + ": " + across + " vs " + along);
                    checked++;
                }
            }
            assertTrue(checked > 150, "points checked " + checked);
        }
    }

    /** Round each pole north points at the pole from every side (the free turn's poles are off the face centres). */
    @Test
    void cubeNorthPointsAtThePoles() {
        for (WorldProjection.Cube p : cubes()) {
            CubeLayout.LocalPoint np = p.world().northPole(), sp = p.world().southPole();
            for (CubeLayout.LocalPoint pole : new CubeLayout.LocalPoint[]{np, sp}) {
                double[] at = p.cube().pointOfLocal(pole.face(), pole.u(), pole.v());
                double lat = p.latitudeDeg(at[0], at[1]);
                assertEquals(pole == np ? 90.0 : -90.0, lat, 1e-5, "the pole's latitude");
                for (int k = 0; k < 8; k++) {
                    double t = Math.PI * k / 4.0, d = 40.0;
                    double x = at[0] + d * Math.cos(t), z = at[1] + d * Math.sin(t);
                    // Towards the north pole, away from the south pole.
                    double sign = pole == np ? -1.0 : 1.0;
                    double want = WorldProjection.bearingDeg(sign * Math.cos(t), sign * Math.sin(t));
                    assertEquals(0.0, turnDiff(p.northBearingDeg(x, z), want), 1.0,
                            (pole == np ? "north" : "south") + " pole on " + pole.face() + " from " + Math.toDegrees(t) + "°");
                }
            }
        }
    }

    /**
     * Across every net edge (the slide and the twelve turning edges) a point just inside and the point
     * just past it, in the copy of the face beyond, are about a block apart on the sphere, and north and
     * the scale run on.
     */
    @Test
    void cubeNorthAndScaleRunOnAcrossEveryEdge() {
        for (WorldProjection.Cube p : cubes()) {
            double r = p.radius();
            List<NetEdge> edges = p.cube().edges();
            assertEquals(14, edges.size());
            for (NetEdge e : edges) {
                for (int q = 1; q <= 3; q++) {
                    double along = e.spanFrom() + q * (e.spanTo() - e.spanFrom()) / 4.0 + 0.5;
                    double n = e.coord();
                    double in = e.beyond() == CubeLayout.Side.MAX ? n - 0.5 : n + 0.5;
                    double out = e.beyond() == CubeLayout.Side.MAX ? n + 0.5 : n - 0.5;
                    boolean ax = e.axis() == CubeLayout.Axis.X;
                    double xi = ax ? in : along, zi = ax ? along : in, xo = ax ? out : along, zo = ax ? along : out;
                    double blocks = angle(p.unit(xi, zi), p.unit(xo, zo)) * r;
                    double sc = p.scale(xi, zi);
                    assertEquals(1.0, blocks / sc, 2e-2, e.name() + " at " + along + ": " + blocks + " blocks across, scale " + sc);
                    assertEquals(1.0, p.scale(xo, zo) / sc, 1e-2, e.name() + " at " + along + ": the scale");
                    if (Math.abs(p.latitudeDeg(xi, zi)) < 85.0)
                        assertEquals(0.0, turnDiff(p.northBearingDeg(xi, zi), p.northBearingDeg(xo, zo)), 1.0,
                                e.name() + " at " + along + ": north");
                }
            }
        }
    }

    /**
     * {@link WorldProjection#greatCircleBlocks} (cube plan G11's planet distance): along Lambert's
     * equator the arc is the x step; on the cube the poles are πR apart, every point's distances to
     * the two poles add up to πR, and a block step is the local scale long.
     */
    @Test
    void greatCircleDistancesOnThePlanet() {
        WorldProjection l = new WorldProjection.Lambert(4096);
        assertEquals(700.0, l.greatCircleBlocks(-200, 0, 500, 0), 1e-9);
        SplittableRandom rnd = new SplittableRandom(5);
        for (WorldProjection.Cube p : cubes()) {
            double r = p.radius();
            CubeLayout.LocalPoint np = p.world().northPole(), sp = p.world().southPole();
            double[] n = p.cube().pointOfLocal(np.face(), np.u(), np.v()), s = p.cube().pointOfLocal(sp.face(), sp.u(), sp.v());
            assertEquals(Math.PI * r, p.greatCircleBlocks(n[0], n[1], s[0], s[1]), 1e-6 * r, "pole to pole");
            for (Face f : Face.all())
                for (int i = 0; i < 5; i++) {
                    double[] at = p.cube().pointOfLocal(f, rnd.nextDouble(-0.95, 0.95), rnd.nextDouble(-0.95, 0.95));
                    double sum = p.greatCircleBlocks(at[0], at[1], n[0], n[1]) + p.greatCircleBlocks(at[0], at[1], s[0], s[1]);
                    assertEquals(Math.PI * r, sum, 1e-6 * r, "to both poles from " + f);
                    assertEquals(1.0, p.greatCircleBlocks(at[0], at[1], at[0] + 1.0, at[1]) / p.scale(at[0] + 0.5, at[1]), 5e-3,
                            "a block step on " + f);
                }
        }
        WorldProjection.Cube c = cubes().get(0);
        long h = S / 2;
        assertTrue(c.cube().fold(h + 40, -3 * h - 40) instanceof CubeLayout.Ocean);
        assertTrue(Double.isNaN(c.greatCircleBlocks(0, 0, h + 40.5, -3 * h - 40.5)), "past the corners");
    }

    @Test
    void pastTheCubesCornersThereIsNoNorthAndNoScale() {
        WorldProjection.Cube p = cubes().get(2);
        int found = 0;
        // The ocean lies beyond the window's long edges, past the bands (CubeLayout.Ocean).
        for (long x = -3L * S / 2; x < 5L * S / 2 && found < 20; x += 61)
            for (long z = -5L * S / 2; z <= 5L * S / 2 && found < 20; z += 61)
                if (p.cube().fold(x, z) instanceof CubeLayout.Ocean) {
                    assertTrue(Double.isNaN(p.northBearingDeg(x + 0.5, z + 0.5)) && Double.isNaN(p.scale(x + 0.5, z + 0.5))
                            && Double.isNaN(p.latitudeDeg(x + 0.5, z + 0.5)), "past the corners at " + x + ", " + z);
                    found++;
                }
        assertEquals(20, found);
    }
}
