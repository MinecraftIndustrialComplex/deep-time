// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api.planet;

import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeMap;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeTurn;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeWorld;
import com.minecraftindustrialcomplex.deeptime.worlddata.PetroffGuyou;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.MethodSource;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Optional;
import java.util.SplittableRandom;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotSame;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The planet API's contract (package-info, docs/api/planet.md), one suite run on every projection:
 * Lambert, Petroff-Guyou (two sizes and centres) and the cube world (three turns). Nothing here names
 * a projection's formulas: each check is a property of a sphere seen through a map, so a new or
 * changed projection must pass it unchanged.
 *
 * <ul>
 *   <li>Round trips: position to latitude and longitude and back; canonical positions (exact for
 *       block centres, the same for every position of one point); what is on the planet.</li>
 *   <li>The local frame against the distance call over short steps, in eight directions; east and
 *       north against the latitude gradient; the frame's area against the scale.</li>
 *   <li>Every seam (the x wrap, Petroff-Guyou's turning edges, the cube's 14 net edges) and every
 *       singular point (the cube's corners, Petroff-Guyou's cone points): distances and bearings run
 *       on, destinations land, the frame runs on where the map is smooth.</li>
 *   <li>Long range: symmetric distances, destination inverting distance and bearing, the pole
 *       convention, off-planet answers.</li>
 * </ul>
 *
 * Measurements (worst errors, the frame's jumps at seams) go to standard output, where the build's
 * test report keeps them.
 */
class DeepTimePlanetContractTest {

    /** Relative tolerance of the short-step checks where the map is smooth. */
    private static final double TOL = 1e-4;
    /** A map whose Jacobian changes more than this (relative) within a block is not smooth there (a crease or an edge). */
    private static final double KINK = 1e-4;
    /** Short-step checks keep this many blocks from a cone point or corner. */
    private static final double SINGULAR_BLOCKS = 16.0;

    /**
     * One world under test: its projection, the planet over it, the latitude past which the polar
     * checks stop, the unit vectors of its cone points or corners, and their positions in the window
     * ({x, z, index into {@code singular}}).
     */
    record World(String name, WorldProjection p, ProjectionPlanet planet, double latMax, List<double[]> singular, List<double[]> singularAt) {
        @Override
        public String toString() {
            return name;
        }

        double r() {
            return planet.radiusMetres();
        }

        double m() {
            return planet.mPerBlockH();
        }
    }

    static List<World> worlds() {
        int s = 1024;
        return List.of(
                world("lambert 16384", new WorldProjection.Lambert(16384), 300.0, 75.0),
                world("petroff-guyou b 4096 lc 37.25", new PetroffGuyou(4096, 37.25), 35.0, 80.0),
                world("petroff-guyou b 1536 lc 0", new PetroffGuyou(1536, 0.0), 35.0, 80.0),
                world("cube 1024 identity", cube(s, CubeTurn.IDENTITY), 35.0, 80.0),
                world("cube 1024 polar 30", cube(s, CubeTurn.polar(30.0)), 35.0, 80.0),
                world("cube 1024 free turn", cube(s, CubeTurn.fromRows(rotation(1, 2, 3, 40.0))), 35.0, 80.0));
    }

    private static World world(String name, WorldProjection p, double m, double latMax) {
        List<double[]> units = new ArrayList<>(), at = new ArrayList<>();
        if (p instanceof PetroffGuyou pg) {
            double b = pg.hemisphere();
            for (double[] q : new double[][]{{0, -b / 2}, {-b, -b / 2}, {b, -b / 2}, {0, b / 2}, {-b, b / 2}, {b, b / 2}}) {
                at.add(new double[]{q[0], q[1], units.size()});
                units.add(pg.unit(q[0], q[1]));
            }
        } else if (p instanceof WorldProjection.Cube c) {
            for (CubeLayout.Corner k : c.cube().corners()) {
                for (long[] im : k.images())
                    at.add(new double[]{im[0], im[1], units.size()});
                units.add(c.world().cornerUnit(k.cube()));
            }
        }
        return new World(name, p, new ProjectionPlanet(p, m), latMax, units, at);
    }

    private static WorldProjection.Cube cube(int s, CubeTurn turn) {
        return new WorldProjection.Cube(new CubeWorld(CubeLayout.of(s), CubeMap.world(), turn, 1.0));
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

    // ---- helpers ---------------------------------------------------------------------------

    private static double dot(double[] a, double[] b) {
        return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    }

    private static double angle(double[] a, double[] b) {
        double cx = a[1] * b[2] - a[2] * b[1], cy = a[2] * b[0] - a[0] * b[2], cz = a[0] * b[1] - a[1] * b[0];
        return Math.atan2(Math.sqrt(cx * cx + cy * cy + cz * cz), dot(a, b));
    }

    private static double turnDiff(double a, double b) {
        double d = Math.abs(a - b) % 360.0;
        return Math.min(d, 360.0 - d);
    }

    /** a − b in degrees, in (−180, 180]. */
    private static double signedTurn(double a, double b) {
        double d = (a - b) % 360.0;
        if (d > 180.0)
            d -= 360.0;
        if (d <= -180.0)
            d += 360.0;
        return d;
    }

    /** A uniformly random planet point's canonical position. */
    private static double[] randomPosition(World w, SplittableRandom rnd) {
        double lat = Math.toDegrees(Math.asin(rnd.nextDouble(-1.0, 1.0))), lon = rnd.nextDouble(0.0, 360.0);
        BlockXZ b = w.planet().blockPoint(new GeoPoint(lat, lon));
        return new double[]{b.x(), b.z()};
    }

    /** Blocks of the equal-area sphere from (x, z) to the nearest cone point or corner. */
    private static double blocksToSingular(World w, double x, double z) {
        double[] u = w.p().unit(x, z);
        double best = Double.POSITIVE_INFINITY;
        for (double[] s : w.singular())
            best = Math.min(best, angle(u, s) * w.p().radius());
        return best;
    }

    /**
     * How far the map is from smooth within a block of (x, z): the largest relative change of a
     * Jacobian column (3D, per block) between central steps of 1 and 1/4 block. Of order 1e-6 where
     * the map is smooth, of the crease's angle where one passes within a block.
     */
    private static double kink(World w, double x, double z) {
        double worst = 0.0;
        for (int axis = 0; axis < 2; axis++) {
            double[] big = column(w.p(), x, z, axis, 1.0), small = column(w.p(), x, z, axis, 0.25);
            double dx = big[0] - small[0], dy = big[1] - small[1], dz = big[2] - small[2];
            worst = Math.max(worst, Math.sqrt(dx * dx + dy * dy + dz * dz) / Math.sqrt(dot(small, small)));
        }
        return worst;
    }

    private static double[] column(WorldProjection p, double x, double z, int axis, double h) {
        double dx = axis == 0 ? h : 0.0, dz = axis == 0 ? 0.0 : h;
        double[] a = p.unit(x + dx, z + dz), b = p.unit(x - dx, z - dz);
        return new double[]{(a[0] - b[0]) / (2 * h), (a[1] - b[1]) / (2 * h), (a[2] - b[2]) / (2 * h)};
    }

    /** Positions that show the same planet point as (x, z) by the map's own gluing, besides (x, z). */
    private static List<double[]> images(World w, double x, double z) {
        double per = w.p().period();
        List<double[]> out = new ArrayList<>(List.of(new double[]{x + per, z}, new double[]{x - per, z}, new double[]{x + 3 * per, z}));
        if (w.p() instanceof PetroffGuyou pg) {
            double b = pg.hemisphere();
            out.add(new double[]{-x, -b - z});   // through the north edge's half-turn
            out.add(new double[]{-x, b - z});    // through the south edge's
            out.add(new double[]{-x + per, -b - z});
        }
        return out;
    }

    private static void report(String what, World w, String fmt, Object... args) {
        System.out.println("[contract] " + what + " | " + w + " | " + String.format(Locale.ROOT, fmt, args));
    }

    // ---- 1. the sphere --------------------------------------------------------------------

    @ParameterizedTest
    @MethodSource("worlds")
    void theRadiusIsTheEqualAreaSpheres(World w) {
        assertEquals(w.p().radius() * w.m(), w.r());
        // The planet's area in blocks is its blocks' area: 4πR² over the metres per block squared,
        // whatever the map (they are all equal-area spheres of the world's blocks).
        assertTrue(w.r() > 0.0 && Double.isFinite(w.r()));
    }

    // ---- 2. round trips -------------------------------------------------------------------

    @ParameterizedTest
    @MethodSource("worlds")
    void positionsLatitudeLongitudeAndCanonicalRoundTrip(World w) {
        DeepTimePlanet pl = w.planet();
        WorldProjection p = w.p();
        SplittableRandom rnd = new SplittableRandom(11);
        double per = p.period(), zLo = p.zMin(), zHi = p.zMax(), zSpan = zHi - zLo;
        double x0 = p instanceof WorldProjection.Cube cube ? -1.5 * cube.side() : -per / 2.0;   // the window's west edge
        int on = 0, n = 6000;
        double worstGeo = 0.0, worstBack = 0.0;
        for (int i = 0; i < n; i++) {
            double x = rnd.nextDouble(x0 - per, x0 + 2 * per), z = rnd.nextDouble(zLo - 0.25 * zSpan, zHi + 0.25 * zSpan);
            boolean isOn = pl.isOnPlanet(x, z);
            Optional<GeoPoint> g = pl.geoPoint(x, z);
            Optional<BlockXZ> c = pl.canonical(x, z);
            assertEquals(isOn, g.isPresent(), "geoPoint present exactly on the planet at " + x + ", " + z);
            assertEquals(isOn, c.isPresent(), "canonical present exactly on the planet");
            assertEquals(isOn, !Double.isNaN(pl.latitudeDeg(x, z)), "latitude NaN exactly off the planet");
            assertEquals(isOn, !Double.isNaN(pl.longitudeDeg(x, z)), "longitude NaN exactly off the planet");
            assertEquals(isOn, !Double.isNaN(pl.distanceMetres(x, z, x, z)), "distance NaN exactly off the planet");
            if (!isOn) {
                assertTrue(pl.frame(x, z).isEmpty(), "no frame off the planet");
                continue;
            }
            on++;
            GeoPoint gp = g.get();
            assertEquals(gp.latitudeDeg(), pl.latitudeDeg(x, z), 0.0);
            assertEquals(gp.longitudeDeg(), pl.longitudeDeg(x, z), 0.0);
            assertTrue(gp.longitudeDeg() >= 0.0 && gp.longitudeDeg() < 360.0);
            assertEquals(0.0, pl.distanceMetres(x, z, x, z), 0.0);
            // The canonical position: in the window, the same point, a fixed point.
            BlockXZ cc = c.get();
            assertEquals(cc.x(), p.wrapX(cc.x()), "canonical x is in the window");
            assertTrue(cc.z() >= zLo && cc.z() <= zHi, "canonical z in range: " + cc);
            double d = pl.distanceMetres(x, z, cc.x(), cc.z());
            assertTrue(d < 1e-6 * w.m(), "the canonical position shows the same point: " + d + " m");
            BlockXZ c2 = pl.canonical(cc.x(), cc.z()).orElseThrow();
            assertTrue(Math.abs(c2.x() - cc.x()) < 1e-9 && Math.abs(c2.z() - cc.z()) < 1e-9, "canonical is idempotent: " + cc + " then " + c2);
            // Latitude and longitude back to a position: the canonical position of the same point.
            BlockXZ back = pl.blockPoint(gp);
            BlockXZ bc = pl.canonical(back.x(), back.z()).orElseThrow();
            assertTrue(Math.abs(bc.x() - back.x()) < 1e-9 && Math.abs(bc.z() - back.z()) < 1e-9, "blockPoint returns a canonical position: " + back + " vs " + bc);
            double db = pl.distanceMetres(back.x(), back.z(), x, z);
            worstBack = Math.max(worstBack, db / w.m());
            assertTrue(db < 1e-4 * w.m(), "blockPoint(geoPoint) is the same point: " + db + " m at " + x + ", " + z);
            double dg = pl.distanceMetres(pl.geoPoint(back.x(), back.z()).orElseThrow(), gp);
            worstGeo = Math.max(worstGeo, dg / w.m());
            assertTrue(dg < 1e-4 * w.m(), "geoPoint(blockPoint) is the same point: " + dg + " m");
        }
        assertTrue(on > n / 4, "positions on the planet " + on + " of " + n);
        report("round trips", w, "on %d of %d, worst blockPoint(geoPoint) %.3g blocks, geoPoint(blockPoint) %.3g blocks", on, n, worstBack, worstGeo);
    }

    @ParameterizedTest
    @MethodSource("worlds")
    void blockCentresHaveExactCanonicalPositions(World w) {
        DeepTimePlanet pl = w.planet();
        WorldProjection p = w.p();
        SplittableRandom rnd = new SplittableRandom(13);
        int checked = 0;
        double worst = 0.0;
        for (int i = 0; i < 4000; i++) {
            double[] q = randomPosition(w, rnd);
            double x = Math.floor(q[0]) + 0.5, z = Math.floor(q[1]) + 0.5;
            Optional<BlockXZ> oc = pl.canonical(x, z);
            if (oc.isEmpty())
                continue;   // a block centre past the cube's corners (q was on a face's edge)
            BlockXZ c = oc.get();
            assertEquals(0.5, c.x() - Math.floor(c.x()), "a block centre's canonical position is a block centre: " + c);
            assertEquals(0.5, c.z() - Math.floor(c.z()), "a block centre's canonical position is a block centre: " + c);
            assertEquals(c, pl.canonical(c.x(), c.z()).orElseThrow(), "exactly a fixed point");
            for (double[] im : images(w, x, z))
                assertEquals(c, pl.canonical(im[0], im[1]).orElseThrow(), "every position of the point has one canonical position, " + im[0] + ", " + im[1]);
            if (blocksToSingular(w, x, z) > 4.0) {
                BlockXZ back = pl.blockPoint(pl.geoPoint(x, z).orElseThrow());
                double e = Math.hypot(back.x() - c.x(), back.z() - c.z());
                worst = Math.max(worst, e);
                assertTrue(e < 1e-6, "blockPoint(geoPoint(centre)) is the canonical centre: " + back + " vs " + c);
            }
            checked++;
        }
        assertTrue(checked > 3500, "block centres checked " + checked);
        // Every block of the window, cube copies included, comes to a block centre.
        if (p instanceof WorldProjection.Cube cube) {
            long h = cube.side() / 2;
            int folded = 0;
            for (long bx = -3 * h; bx < 5 * h; bx += 37)
                for (long bz = -3 * h; bz < 3 * h; bz += 41) {
                    Optional<BlockXZ> c = pl.canonical(bx + 0.5, bz + 0.5);
                    if (c.isEmpty())
                        continue;
                    assertEquals(0.5, c.get().x() - Math.floor(c.get().x()), "the cube folds block centres to block centres");
                    assertEquals(0.5, c.get().z() - Math.floor(c.get().z()), "the cube folds block centres to block centres");
                    folded++;
                }
            assertTrue(folded > 1000, "cube window blocks folded " + folded);
        }
        report("block centres", w, "checked %d, worst blockPoint(geoPoint(centre)) %.3g blocks", checked, worst);
    }

    @ParameterizedTest
    @MethodSource("worlds")
    void everyGeoPointHasAPositionOnThePlanet(World w) {
        DeepTimePlanet pl = w.planet();
        SplittableRandom rnd = new SplittableRandom(17);
        List<GeoPoint> pts = new ArrayList<>(List.of(new GeoPoint(90, 0), new GeoPoint(-90, 0), new GeoPoint(0, 0), new GeoPoint(0, 180),
                new GeoPoint(89.999999, 33), new GeoPoint(-89.999999, 271), new GeoPoint(45, 359.9999999)));
        for (int i = 0; i < 2000; i++)
            pts.add(new GeoPoint(Math.toDegrees(Math.asin(rnd.nextDouble(-1.0, 1.0))), rnd.nextDouble(-720.0, 720.0)));
        double worst = 0.0;
        for (GeoPoint g : pts) {
            BlockXZ b = pl.blockPoint(g);
            assertTrue(pl.isOnPlanet(b.x(), b.z()), "blockPoint(" + g + ") = " + b + " is on the planet");
            double d = pl.distanceMetres(pl.geoPoint(b.x(), b.z()).orElseThrow(), g);
            worst = Math.max(worst, d / w.m());
            assertTrue(d < 1e-4 * w.m(), "blockPoint(" + g + ") shows it: " + d + " m off");
        }
        report("geo points", w, "worst geoPoint(blockPoint(g)) %.3g blocks off", worst);
    }

    // ---- 3. the local frame against distance and latitude ---------------------------------

    @ParameterizedTest
    @MethodSource("worlds")
    void theFrameAgreesWithDistanceAndLatitudeOverShortSteps(World w) {
        ProjectionPlanet pl = w.planet();
        WorldProjection p = w.p();
        boolean conformal = p instanceof WorldProjection.Cube || p instanceof WorldProjection.Lambert;
        double rM = w.r(), degPerM = Math.toDegrees(1.0 / rM), h = 0.5;
        SplittableRandom rnd = new SplittableRandom(19);
        int n = 3000, tested = 0, kinked = 0, nearSingular = 0, polar = 0;
        double worstStep = 0, worstBearing = 0, worstArea = 0, worstEast = 0, worstNorth = 0, worstLon = 0, worstGrad = 0, worstGradNorth = 0;
        double worstKinkedStep = 0;
        for (int i = 0; i < n; i++) {
            double[] q = randomPosition(w, rnd);
            double x = q[0] + rnd.nextDouble(-0.5, 0.5), z = q[1] + rnd.nextDouble(-0.5, 0.5);
            if (!pl.isOnPlanet(x, z))
                continue;
            double lat = pl.latitudeDeg(x, z);
            if (Math.abs(lat) > w.latMax()) {
                polar++;
                continue;
            }
            if (blocksToSingular(w, x, z) < SINGULAR_BLOCKS) {
                nearSingular++;
                continue;
            }
            LocalFrame f = pl.frame(x, z).orElseThrow(() -> new AssertionError("no frame on the planet at " + x + ", " + z));
            // The frame is the projection's own north and scale, bit for bit (the climate API's answers).
            assertEquals(p.northBearingDeg(x, z), f.northBearingDeg());
            assertEquals(w.m() * p.scale(x, z), f.metresPerBlock());
            assertEquals(0.0, turnDiff(WorldProjection.bearingDeg(f.northX(), f.northZ()), f.northBearingDeg()), 1e-9);
            assertEquals(1.0, Math.hypot(f.northX(), f.northZ()), 1e-12);
            assertEquals(1.0, Math.hypot(f.eastX(), f.eastZ()), 1e-12);
            if (kink(w, x, z) > KINK) {
                kinked++;
                // Where a crease passes within a block the frame is the half-block average; it still
                // tells short steps to a few percent.
                for (int k = 0; k < 8; k++) {
                    double dx = Math.cos(k * Math.PI / 4), dz = Math.sin(k * Math.PI / 4);
                    double dist = pl.distanceMetres(x - dx / 2, z - dz / 2, x + dx / 2, z + dz / 2);
                    worstKinkedStep = Math.max(worstKinkedStep, Math.abs(f.metresOfStep(dx, dz) / dist - 1.0));
                }
                continue;
            }
            tested++;
            // A block's area: the square of metresPerBlock, and east × north over their angle.
            double cross = Math.abs(f.eastX() * f.northZ() - f.eastZ() * f.northX());
            double area = f.metresPerBlockEast() * f.metresPerBlockNorth() / cross;
            worstArea = Math.max(worstArea, Math.abs(area / (f.metresPerBlock() * f.metresPerBlock()) - 1.0));
            // Short steps in eight directions: length against the distance call, bearing against the
            // initial bearing (the mean of the chords forward and back cancels their bend).
            for (int k = 0; k < 8; k++) {
                double dx = Math.cos(k * Math.PI / 4 + 0.1), dz = Math.sin(k * Math.PI / 4 + 0.1);
                double dist = pl.distanceMetres(x - dx / 2, z - dz / 2, x + dx / 2, z + dz / 2);
                worstStep = Math.max(worstStep, Math.abs(f.metresOfStep(dx, dz) / dist - 1.0));
                double fwd = pl.initialBearingDeg(x, z, x + h * dx, z + h * dz), bwd = pl.initialBearingDeg(x, z, x - h * dx, z - h * dz) - 180.0;
                double mean = fwd + signedTurn(bwd, fwd) / 2.0;
                worstBearing = Math.max(worstBearing, turnDiff(mean, f.bearingOfStep(dx, dz)));
                BlockXZ back = f.step(f.bearingOfStep(dx, dz), f.metresOfStep(dx, dz));
                assertEquals(dx, back.x(), 1e-9, "step inverts bearingOfStep and metresOfStep");
                assertEquals(dz, back.z(), 1e-9, "step inverts bearingOfStep and metresOfStep");
            }
            // East and north: one block along each is metresPerBlockEast and North on the ground.
            double de = pl.distanceMetres(x - f.eastX() / 2, z - f.eastZ() / 2, x + f.eastX() / 2, z + f.eastZ() / 2);
            double dn = pl.distanceMetres(x - f.northX() / 2, z - f.northZ() / 2, x + f.northX() / 2, z + f.northZ() / 2);
            worstStep = Math.max(worstStep, Math.max(Math.abs(de / f.metresPerBlockEast() - 1), Math.abs(dn / f.metresPerBlockNorth() - 1)));
            assertEquals(90.0, f.bearingOfStep(f.eastX(), f.eastZ()), 1e-6, "east's geographic bearing");
            assertEquals(0.0, turnDiff(0.0, f.bearingOfStep(f.northX(), f.northZ())), 1e-6, "north's geographic bearing");
            // North against latitude: a step east keeps it, a step north raises it by the step's
            // metres over R and keeps the longitude, the latitude gradient is across east and along
            // north's side (exactly along north where the map is conformal).
            double latE = pl.latitudeDeg(x + h * f.eastX(), z + h * f.eastZ()) - pl.latitudeDeg(x - h * f.eastX(), z - h * f.eastZ());
            worstEast = Math.max(worstEast, Math.abs(latE) / (2 * h * f.metresPerBlockEast() * degPerM));
            double latN = pl.latitudeDeg(x + h * f.northX(), z + h * f.northZ()) - pl.latitudeDeg(x - h * f.northX(), z - h * f.northZ());
            worstNorth = Math.max(worstNorth, Math.abs(latN / (2 * h * f.metresPerBlockNorth() * degPerM) - 1.0));
            double lonN = signedTurn(pl.longitudeDeg(x + h * f.northX(), z + h * f.northZ()), pl.longitudeDeg(x - h * f.northX(), z - h * f.northZ()));
            worstLon = Math.max(worstLon, Math.abs(lonN) * Math.cos(Math.toRadians(lat)) / (2 * h * f.metresPerBlockNorth() * degPerM));
            double gx = (pl.latitudeDeg(x + h, z) - pl.latitudeDeg(x - h, z)) / (2 * h);
            double gz = (pl.latitudeDeg(x, z + h) - pl.latitudeDeg(x, z - h)) / (2 * h);
            double g = Math.hypot(gx, gz);
            worstGrad = Math.max(worstGrad, Math.abs(gx * f.eastX() + gz * f.eastZ()) / g);
            assertTrue(gx * f.northX() + gz * f.northZ() > 0.0, "latitude rises along north at " + x + ", " + z);
            worstGradNorth = Math.max(worstGradNorth, turnDiff(WorldProjection.bearingDeg(gx, gz), f.northBearingDeg()));
        }
        report("frame vs distance", w, "tested %d, crease within a block %d (worst step there %.3g), near a cone point or corner %d, polar %d; "
                        + "worst: step length %.3g, step bearing %.3g deg, area %.3g, latitude along east %.3g, along north %.3g, longitude along north %.3g, "
                        + "gradient vs east %.3g, gradient bearing vs north %.4f deg",
                tested, kinked, worstKinkedStep, nearSingular, polar, worstStep, worstBearing, worstArea, worstEast, worstNorth, worstLon, worstGrad, worstGradNorth);
        assertTrue(tested > n / 2, "smooth points tested " + tested + " of " + n);
        assertTrue(worstStep < TOL, "short steps' length against the distance call: " + worstStep);
        assertTrue(worstBearing < 1e-3, "short steps' bearing against the initial bearing: " + worstBearing + " deg");
        assertTrue(worstArea < TOL, "a block's area: " + worstArea);
        assertTrue(worstEast < TOL, "a step east keeps the latitude: " + worstEast);
        assertTrue(worstNorth < TOL, "a step north raises it by its metres over R: " + worstNorth);
        assertTrue(worstLon < TOL, "a step north keeps the longitude: " + worstLon);
        assertTrue(worstGrad < TOL, "the latitude gradient is across east: " + worstGrad);
        if (conformal)
            assertTrue(worstGradNorth < 1e-2, "on a conformal map the gradient is along north: " + worstGradNorth + " deg");
        assertTrue(worstKinkedStep < 0.05, "short steps near a crease: " + worstKinkedStep);
    }

    // ---- 4. seams, edges and corners ------------------------------------------------------

    /** A pair of positions a block apart across a seam: {x in, z in, x out, z out}. */
    private record Crossing(String seam, double xi, double zi, double xo, double zo) {}

    private static List<Crossing> crossings(World w) {
        List<Crossing> out = new ArrayList<>();
        WorldProjection p = w.p();
        if (p instanceof WorldProjection.Lambert l) {
            double c = l.circumference(), r = l.radius();
            for (int i = -20; i <= 20; i++) {
                double z = 0.9 * r * i / 20.0 + 0.25;
                out.add(new Crossing("x wrap east", c / 2 - 0.5, z, c / 2 + 0.5, z));
                out.add(new Crossing("x wrap west", -c / 2 + 0.5, z, -c / 2 - 0.5, z));
            }
        } else if (p instanceof PetroffGuyou pg) {
            double b = pg.hemisphere(), hz = b / 2.0;
            for (int i = -40; i <= 40; i++) {
                double z = 0.98 * hz * i / 40.0 + 0.25;
                out.add(new Crossing("x wrap east", b - 0.5, z, b + 0.5, z));
                out.add(new Crossing("x wrap west", -b + 0.5, z, -b - 0.5, z));
                out.add(new Crossing("hemispheres", -0.5, z, 0.5, z));
            }
            for (int i = 0; i < 160; i++) {
                double x = -b + 2 * b * (i + 0.37) / 160.0;
                out.add(new Crossing("north edge", x, -hz + 0.5, x, -hz - 0.5));
                out.add(new Crossing("south edge", x, hz - 0.5, x, hz + 0.5));
            }
        } else if (p instanceof WorldProjection.Cube cube) {
            List<CubeLayout.NetEdge> edges = cube.cube().edges();
            assertEquals(14, edges.size(), "the cube net's edges");
            for (CubeLayout.NetEdge e : edges)
                for (int q = 1; q < 32; q++) {
                    double along = e.spanFrom() + q * (e.spanTo() - e.spanFrom()) / 32.0 + 0.25;
                    double n = e.coord();
                    double in = e.beyond() == CubeLayout.Side.MAX ? n - 0.5 : n + 0.5;
                    double ou = e.beyond() == CubeLayout.Side.MAX ? n + 0.5 : n - 0.5;
                    boolean ax = e.axis() == CubeLayout.Axis.X;
                    out.add(new Crossing(e.name(), ax ? in : along, ax ? along : in, ax ? ou : along, ax ? along : ou));
                }
            long h = cube.side() / 2;
            for (int i = -15; i <= 15; i++) {
                double z = h * i / 16.0 + 0.25;
                out.add(new Crossing("x wrap", 5 * h - 0.5, z, 5 * h + 0.5, z));
            }
        }
        return out;
    }

    @ParameterizedTest
    @MethodSource("worlds")
    void distancesAndBearingsRunOnAcrossEverySeam(World w) {
        ProjectionPlanet pl = w.planet();
        double rM = w.r();
        SplittableRandom rnd = new SplittableRandom(23);
        List<double[]> far = new ArrayList<>();
        for (int i = 0; i < 6; i++)
            far.add(randomPosition(w, rnd));
        java.util.Map<String, double[]> jumps = new java.util.TreeMap<>();
        List<Crossing> cs = crossings(w);
        int checked = 0;
        for (Crossing c : cs) {
            assertTrue(pl.isOnPlanet(c.xi(), c.zi()) && pl.isOnPlanet(c.xo(), c.zo()), "both sides of " + c + " are on the planet");
            if (blocksToSingular(w, c.xi(), c.zi()) < SINGULAR_BLOCKS)
                continue;
            checked++;
            double d = pl.distanceMetres(c.xi(), c.zi(), c.xo(), c.zo());
            double latIn = pl.latitudeDeg(c.xi(), c.zi());
            LocalFrame fi = pl.frame(c.xi(), c.zi()).orElseThrow(), fo = pl.frame(c.xo(), c.zo()).orElseThrow();
            double sx = c.xo() - c.xi(), sz = c.zo() - c.zi();
            double stepErr = Math.abs(d / fi.metresOfStep(sx, sz) - 1.0);
            BlockXZ dest = pl.destination(c.xi(), c.zi(), fi.bearingOfStep(sx, sz), fi.metresOfStep(sx, sz)).orElseThrow();
            double destErr = pl.distanceMetres(dest.x(), dest.z(), c.xo(), c.zo()) / d;
            // From far away the seam is invisible: distances change by at most the step, bearings by
            // the angle it subtends; at the seam, bearings to far points move as little.
            for (double[] fp : far) {
                double theta = pl.distanceMetres(fp[0], fp[1], c.xi(), c.zi()) / rM;
                if (theta < 0.2 || theta > Math.PI - 0.2)
                    continue;
                double di = pl.distanceMetres(fp[0], fp[1], c.xi(), c.zi()), dout = pl.distanceMetres(fp[0], fp[1], c.xo(), c.zo());
                assertTrue(Math.abs(di - dout) <= d * (1 + 1e-9) + 1e-9, "triangle inequality across " + c);
                double bt = turnDiff(pl.initialBearingDeg(fp[0], fp[1], c.xi(), c.zi()), pl.initialBearingDeg(fp[0], fp[1], c.xo(), c.zo()));
                assertTrue(bt <= 1.5 * Math.toDegrees(d / (rM * Math.sin(theta))) + 1e-9, "bearing to the seam from afar jumps " + bt + " deg at " + c);
                if (Math.abs(latIn) < w.latMax()) {
                    double bs = turnDiff(pl.initialBearingDeg(c.xi(), c.zi(), fp[0], fp[1]), pl.initialBearingDeg(c.xo(), c.zo(), fp[0], fp[1]));
                    double bound = 1.5 * Math.toDegrees(d / rM * (Math.abs(Math.tan(Math.toRadians(latIn))) + 1.0 / Math.sin(theta))) + 1e-9;
                    assertTrue(bs <= bound, "bearing from the seam jumps " + bs + " deg (bound " + bound + ") at " + c);
                }
                BlockXZ there = pl.destination(c.xo(), c.zo(), pl.initialBearingDeg(c.xo(), c.zo(), fp[0], fp[1]), dout).orElseThrow();
                assertTrue(pl.distanceMetres(there.x(), there.z(), fp[0], fp[1]) < 1e-3 * w.m(), "destination from past the seam lands at " + c);
            }
            double[] j = jumps.computeIfAbsent(c.seam(), k -> new double[6]);
            if (Math.abs(latIn) < w.latMax()) {   // east and north turn fast near a pole, which Petroff-Guyou's edges cross
                j[0] = Math.max(j[0], turnDiff(fi.northBearingDeg(), fo.northBearingDeg()));
                j[1] = Math.max(j[1], Math.abs(fo.metresPerBlockEast() / fi.metresPerBlockEast() - 1.0));
                j[2] = Math.max(j[2], Math.abs(fo.metresPerBlockNorth() / fi.metresPerBlockNorth() - 1.0));
            }
            j[3] = Math.max(j[3], stepErr);
            j[4] = Math.max(j[4], destErr);
            j[5]++;
        }
        assertTrue(checked > cs.size() / 2, "crossings checked " + checked + " of " + cs.size());
        boolean pg = w.p() instanceof PetroffGuyou;
        for (var e : jumps.entrySet()) {
            double[] j = e.getValue();
            report("seam", w, "%-12s n %3.0f: frame jump north %.4f deg, metres east %.3g, north %.3g; step vs distance %.3g; destination off %.3g of the step",
                    e.getKey(), j[5], j[0], j[1], j[2], j[3], j[4]);
            assertTrue(j[3] < 0.05, e.getKey() + ": a block across is a block on the ground: " + j[3]);
            assertTrue(j[4] < 0.05, e.getKey() + ": the frame's step across lands past the seam: " + j[4]);
            if (!pg) {
                assertTrue(j[0] < 1.0, e.getKey() + ": north runs on: " + j[0]);
                assertTrue(j[1] < 2e-2 && j[2] < 2e-2, e.getKey() + ": the scale runs on: " + j[1] + ", " + j[2]);
            }
        }
    }

    /** Round every cone point and corner image the planet runs on: neighbours on a ring stay neighbours. */
    @ParameterizedTest
    @MethodSource("worlds")
    void distancesRunOnRoundEveryConePointAndCorner(World w) {
        ProjectionPlanet pl = w.planet();
        int rings = 0;
        double worst = 0.0;
        for (double[] c : w.singularAt()) {
            double[] centre = w.singular().get((int) c[2]);
            double r0 = 3.0;
            int k = 48;
            double step = 2 * Math.sin(Math.PI / k);   // a ring step over its radius, in the net
            double[] prev = null;
            double prevOut = 0.0;
            int on = 0;
            for (int i = 0; i <= k; i++) {
                double t = 2 * Math.PI * (i + 0.5) / k;
                double[] q = {c[0] + r0 * Math.cos(t), c[1] + r0 * Math.sin(t)};
                if (!pl.isOnPlanet(q[0], q[1])) {
                    prev = null;
                    continue;
                }
                on++;
                // On the ground the ring is a ring too, of its own radius (the map's scale near a
                // cone point or corner is not the nominal one), and a cone's angle is not 360°.
                double out = w.p().radius() * angle(w.p().unit(q[0], q[1]), centre) * w.m();
                assertTrue(out > 0.0 && out < 3 * r0 * w.m(), "ring point " + i + " round " + c[0] + ", " + c[1] + " is " + out + " m out");
                if (prev != null) {
                    double d = pl.distanceMetres(prev[0], prev[1], q[0], q[1]);
                    double ref = step * Math.max(out, prevOut);
                    worst = Math.max(worst, d / ref);
                    assertTrue(d < 6 * ref, "neighbours on the ring round " + c[0] + ", " + c[1] + " are " + d + " m apart (ring radius " + out + " m)");
                }
                prev = q;
                prevOut = out;
            }
            if (on > 0)
                rings++;
        }
        assertEquals(w.singularAt().isEmpty(), rings == 0);
        report("rings", w, "%d rings, worst neighbour distance %.3f of the net's angle step on the ground ring", rings, worst);
    }

    // ---- 5. long range --------------------------------------------------------------------

    @ParameterizedTest
    @MethodSource("worlds")
    void distanceBearingAndDestinationAgreeAtLongRange(World w) {
        ProjectionPlanet pl = w.planet();
        double rM = w.r();
        SplittableRandom rnd = new SplittableRandom(29);
        double worstDest = 0.0, worstGeoDest = 0.0;
        for (int i = 0; i < 1500; i++) {
            double[] a = randomPosition(w, rnd), b = randomPosition(w, rnd);
            if (i % 3 == 1) {   // a non-canonical position of the same point
                List<double[]> im = images(w, a[0], a[1]);
                a = im.get(i % im.size());
            }
            double d = pl.distanceMetres(a[0], a[1], b[0], b[1]);
            assertEquals(d, pl.distanceMetres(b[0], b[1], a[0], a[1]), "distance is symmetric");
            assertTrue(d >= 0.0 && d <= Math.PI * rM * (1 + 1e-12), "distance in [0, πR]: " + d);
            GeoPoint ga = pl.geoPoint(a[0], a[1]).orElseThrow(), gb = pl.geoPoint(b[0], b[1]).orElseThrow();
            assertEquals(d, pl.distanceMetres(ga, gb), 1e-9 * rM, "block and geographic distances agree");
            double brg = pl.initialBearingDeg(a[0], a[1], b[0], b[1]);
            assertTrue(brg >= 0.0 && brg < 360.0, "bearing in [0, 360): " + brg);
            if (d > 1e-3 * rM && d < (Math.PI - 1e-3) * rM)
                assertEquals(0.0, turnDiff(brg, pl.initialBearingDeg(ga, gb)), 1e-6, "block and geographic bearings agree");
            BlockXZ there = pl.destination(a[0], a[1], brg, d).orElseThrow();
            double off = pl.distanceMetres(there.x(), there.z(), b[0], b[1]);
            worstDest = Math.max(worstDest, off / w.m());
            assertTrue(off < 1e-3 * w.m(), "destination(a, bearing, distance) is b: " + off + " m off");
            BlockXZ tc = pl.canonical(there.x(), there.z()).orElseThrow();
            assertTrue(Math.abs(tc.x() - there.x()) < 1e-9 && Math.abs(tc.z() - there.z()) < 1e-9, "destination returns a canonical position: " + there + " vs " + tc);
            GeoPoint gThere = pl.destination(ga, pl.initialBearingDeg(ga, gb), pl.distanceMetres(ga, gb));
            worstGeoDest = Math.max(worstGeoDest, pl.distanceMetres(gThere, gb) / w.m());
            assertTrue(pl.distanceMetres(gThere, gb) < 1e-6 * w.m(), "geographic destination is b");
            // Going a negative distance goes backwards; going nowhere stays.
            BlockXZ back = pl.destination(a[0], a[1], brg, -d / 3).orElseThrow(), turned = pl.destination(a[0], a[1], brg + 180.0, d / 3).orElseThrow();
            assertTrue(pl.distanceMetres(back.x(), back.z(), turned.x(), turned.z()) < 1e-3 * w.m(), "a negative distance goes backwards");
            BlockXZ stay = pl.destination(a[0], a[1], brg, 0.0).orElseThrow();
            assertTrue(pl.distanceMetres(stay.x(), stay.z(), a[0], a[1]) < 1e-4 * w.m(), "going nowhere stays");
            assertTrue(Double.isNaN(pl.initialBearingDeg(a[0], a[1], a[0], a[1])), "no bearing to the same point");
        }
        report("long range", w, "worst destination %.3g blocks, geographic %.3g blocks", worstDest, worstGeoDest);
    }

    @ParameterizedTest
    @MethodSource("worlds")
    void thePoleConvention(World w) {
        ProjectionPlanet pl = w.planet();
        double rM = w.r(), d = 0.1 * rM, dd = Math.toDegrees(0.1);
        GeoPoint n = new GeoPoint(90, 0), s = new GeoPoint(-90, 0);
        assertGeo(new GeoPoint(90 - dd, 0), pl.destination(n, 180, d), "from the north pole bearing 180 is longitude 0");
        assertGeo(new GeoPoint(90 - dd, 90), pl.destination(n, 90, d), "bearing 90 is longitude 90");
        assertGeo(new GeoPoint(90 - dd, 180), pl.destination(n, 0, d), "bearing 0 is longitude 180");
        assertGeo(new GeoPoint(-90 + dd, 0), pl.destination(s, 0, d), "from the south pole bearing 0 is longitude 0");
        assertGeo(new GeoPoint(-90 + dd, 90), pl.destination(s, 90, d), "bearing 90 is longitude 90");
        assertEquals(180.0, pl.initialBearingDeg(n, new GeoPoint(80, 0)), 1e-9);
        assertEquals(90.0, pl.initialBearingDeg(n, new GeoPoint(80, 90)), 1e-9);
        assertEquals(0.0, turnDiff(0.0, pl.initialBearingDeg(s, new GeoPoint(-80, 0))), 1e-9);
        assertTrue(Double.isNaN(pl.initialBearingDeg(n, s)), "no bearing to the antipode");
        assertEquals(Math.PI * rM, pl.distanceMetres(n, s), 1e-9 * rM);
        GeoPoint g = new GeoPoint(23.5, 101.25);
        assertTrue(Double.isNaN(pl.initialBearingDeg(g, new GeoPoint(-23.5, 281.25))), "no bearing to the antipode");
        // The poles' positions are on the planet and read as the poles.
        for (GeoPoint pole : new GeoPoint[]{n, s}) {
            BlockXZ b = pl.blockPoint(pole);
            assertEquals(pole.latitudeDeg(), pl.latitudeDeg(b.x(), b.z()), 1e-6, "the pole's position reads as the pole");
        }
        assertThrows(IllegalArgumentException.class, () -> pl.destination(g, Double.NaN, 1.0));
        assertThrows(IllegalArgumentException.class, () -> pl.destination(g, 0.0, Double.POSITIVE_INFINITY));
    }

    private static void assertGeo(GeoPoint want, GeoPoint got, String what) {
        assertEquals(want.latitudeDeg(), got.latitudeDeg(), 1e-9, what + ": " + got);
        assertEquals(0.0, turnDiff(want.longitudeDeg(), got.longitudeDeg()), 1e-9, what + ": " + got);
    }

    // ---- 6. off the planet ----------------------------------------------------------------

    @ParameterizedTest
    @MethodSource("worlds")
    void offThePlanetEverythingIsEmptyOrNaN(World w) {
        ProjectionPlanet pl = w.planet();
        List<double[]> off = new ArrayList<>(List.of(new double[]{Double.NaN, 0}, new double[]{0, Double.NaN},
                new double[]{Double.POSITIVE_INFINITY, 0}, new double[]{0, Double.NEGATIVE_INFINITY}));
        WorldProjection p = w.p();
        if (p instanceof WorldProjection.Lambert l) {
            double r = l.radius();
            for (double z : new double[]{r + 0.001, r + 3, 10 * r})
                for (double x : new double[]{0.5, l.circumference() / 3.0}) {
                    off.add(new double[]{x, z});
                    off.add(new double[]{x, -z});
                }
        } else if (p instanceof WorldProjection.Cube cube) {
            long s = cube.side();
            int found = 0;
            for (long x = -3L * s / 2; x < 5L * s / 2 && found < 40; x += 61)
                for (long z = -5L * s / 2; z <= 5L * s / 2 && found < 40; z += 61)
                    if (cube.cube().fold(x, z) instanceof CubeLayout.Ocean) {
                        off.add(new double[]{x + 0.5, z + 0.5});
                        found++;
                    }
            assertEquals(40, found, "ocean past the corners");
        } else {
            // Petroff-Guyou folds every finite position onto the planet.
            SplittableRandom rnd = new SplittableRandom(31);
            for (int i = 0; i < 2000; i++) {
                double x = rnd.nextDouble(-1e6, 1e6), z = rnd.nextDouble(-1e5, 1e5);
                assertTrue(pl.isOnPlanet(x, z), "every finite position is on a folded world: " + x + ", " + z);
            }
        }
        double[] on = randomPosition(w, new SplittableRandom(37));
        for (double[] q : off) {
            String at = q[0] + ", " + q[1];
            assertFalse(pl.isOnPlanet(q[0], q[1]), "off the planet at " + at);
            assertTrue(pl.geoPoint(q[0], q[1]).isEmpty() && pl.canonical(q[0], q[1]).isEmpty() && pl.frame(q[0], q[1]).isEmpty(), "empty at " + at);
            assertTrue(Double.isNaN(pl.latitudeDeg(q[0], q[1])) && Double.isNaN(pl.longitudeDeg(q[0], q[1])), "NaN at " + at);
            assertTrue(Double.isNaN(pl.distanceMetres(q[0], q[1], on[0], on[1])) && Double.isNaN(pl.distanceMetres(on[0], on[1], q[0], q[1])), "no distance at " + at);
            assertTrue(Double.isNaN(pl.initialBearingDeg(q[0], q[1], on[0], on[1])) && Double.isNaN(pl.initialBearingDeg(on[0], on[1], q[0], q[1])), "no bearing at " + at);
            assertTrue(pl.destination(q[0], q[1], 30.0, 1000.0).isEmpty(), "no destination from " + at);
        }
        assertTrue(pl.destination(on[0], on[1], Double.NaN, 1.0).isEmpty() && pl.destination(on[0], on[1], 0.0, Double.NaN).isEmpty(), "no destination for NaN");
    }

    // ---- 7. the value types and the lookup ------------------------------------------------

    @Test
    void geoPointsAreNormalised() {
        assertEquals(new GeoPoint(10, 350), new GeoPoint(10, -10));
        assertEquals(new GeoPoint(10, 0), new GeoPoint(10, 720));
        assertEquals(new GeoPoint(0, 0), new GeoPoint(-0.0, -0.0));
        assertEquals(new GeoPoint(90, 0), new GeoPoint(90, 123));
        assertEquals(new GeoPoint(-90, 0), new GeoPoint(-90, -5));
        assertTrue(new GeoPoint(5, -1e-20).longitudeDeg() < 360.0);
        assertThrows(IllegalArgumentException.class, () -> new GeoPoint(90.0000001, 0));
        assertThrows(IllegalArgumentException.class, () -> new GeoPoint(Double.NaN, 0));
        assertThrows(IllegalArgumentException.class, () -> new GeoPoint(0, Double.POSITIVE_INFINITY));
        BlockXZ b = new BlockXZ(-0.25, 3.75);
        assertEquals(-1, b.blockX());
        assertEquals(3, b.blockZ());
    }

    @Test
    void aLambertFrameAtSixtyDegrees() {
        WorldProjection.Lambert l = new WorldProjection.Lambert(16384);
        ProjectionPlanet pl = new ProjectionPlanet(l, 50.0);
        double z = -l.radius() * Math.sin(Math.toRadians(60.0));
        LocalFrame f = pl.frame(100.5, z).orElseThrow();
        assertEquals(60.0, pl.latitudeDeg(100.5, z), 1e-9);
        assertEquals(0.0, f.northBearingDeg());
        assertEquals(0.0, f.northX(), 1e-12);
        assertEquals(-1.0, f.northZ(), 1e-12);
        assertEquals(1.0, f.eastX(), 1e-12);
        assertEquals(0.0, f.eastZ(), 1e-12);
        assertEquals(25.0, f.metresPerBlockEast(), 1e-9, "a block of x is cos 60° of the nominal");
        assertEquals(100.0, f.metresPerBlockNorth(), 1e-9, "a block of z is 1/cos 60° of it");
        assertEquals(50.0, f.metresPerBlock(), 1e-12, "the area is the nominal one");
        assertEquals(Math.hypot(25.0, 100.0), f.metresOfStep(1, 1), 1e-9);
        assertEquals(180.0, f.bearingOfStep(0, 1), 1e-12);
        BlockXZ st = f.step(45.0, Math.sqrt(2) * 100.0);
        assertEquals(4.0, st.x(), 1e-9);
        assertEquals(-1.0, st.z(), 1e-9);
        assertTrue(Double.isNaN(f.bearingOfStep(0, 0)));
    }

    @Test
    void theLookupKeepsOnePlanetPerWorld() {
        WorldProjection a = new WorldProjection.Lambert(4096);
        DeepTimePlanet p1 = PlanetLookup.planet(a, 30.0).orElseThrow();
        assertSame(p1, PlanetLookup.planet(a, 30.0).orElseThrow(), "the same projection and scale give the same planet");
        assertSame(p1, PlanetLookup.planet(new WorldProjection.Lambert(4096), 30.0).orElseThrow(), "an equal projection too");
        DeepTimePlanet p2 = PlanetLookup.planet(a, 31.0).orElseThrow();
        assertNotSame(p1, p2);
        assertEquals(a.radius() * 31.0, p2.radiusMetres());
        for (int i = 0; i < 20; i++)
            assertEquals(a.radius() * (40.0 + i), PlanetLookup.planet(a, 40.0 + i).orElseThrow().radiusMetres());
        assertEquals(a.radius() * 30.0, PlanetLookup.planet(a, 30.0).orElseThrow().radiusMetres(), "a planet dropped from the cache comes back right");
        assertEquals(1, DeepTimePlanet.API_VERSION);
    }
}
