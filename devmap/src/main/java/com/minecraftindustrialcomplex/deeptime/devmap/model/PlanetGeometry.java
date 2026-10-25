// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.model;

import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
import com.minecraftindustrialcomplex.deeptime.wrap.CubeBorder;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Locale;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * What the map marks and the readout says about a point, from Deep Time's own
 * {@link WorldProjection} (no projection formulas here): latitude and longitude as the F3 line shows
 * them ({@code net.PlanetDebugLine}), the distance to the seam and the edges, and the poles.
 *
 * <p>The longitude convention follows the F3 line: the simulation's longitude folded to
 * [−180°, 180°), on the cube world and on a Lambert-frame test level alike (the playable Lambert and
 * Petroff–Guyou worlds went with cube plan §7.4, 2026-10-24).
 *
 * <p>On a cube world (docs/plans/cube-world-plan.md G14) the block space is the net's window, 4s
 * by 3s ({@code CubeLayout.window}), of which the six faces are the planet and the rest the void.
 * The seam is the slide (F2's east side against F3's west, x = −3s/2 ≡ 5s/2, Toroidal World's), the
 * edges are the border's twelve segments (the turning net edges), and the cone points are the eight
 * cube corners, at their twelve images in the window.
 */
public final class PlanetGeometry {

    private PlanetGeometry() {}

    /** The cube projection of a cube world, or null. */
    public static WorldProjection.Cube cube(WorldProjection p) {
        return p instanceof WorldProjection.Cube c ? c : null;
    }

    /**
     * Block x of the west side of the world's block-space picture: the seam (−period/2) on the
     * polar worlds, the west side of a cube's net window (−3s/2, F3's west side).
     */
    public static double west(WorldProjection p) {
        WorldProjection.Cube c = cube(p);
        return c != null ? c.cube().window()[0] : -p.period() / 2.0;
    }

    /**
     * x wrapped into [−period/2, period/2), Toroidal World's frame (its F3 x). A polar world's
     * {@link WorldProjection#wrapX} is this; a cube's wraps into its net window [−3s/2, 5s/2), so x
     * differences and the readout's x go through this.
     */
    public static double wrapCentred(WorldProjection p, double x) {
        if (cube(p) == null)
            return p.wrapX(x);
        double period = p.period();
        double t = x - period * Math.floor(x / period + 0.5);
        return t >= period / 2.0 ? t - period : t;
    }

    /**
     * Whether the point (x, z) is on the planet: on one of a cube's six faces (x wraps; the rest of
     * the window is the void, whose blocks are copies folded from the faces); every point of a polar
     * world.
     */
    public static boolean onPlanet(WorldProjection p, double x, double z) {
        WorldProjection.Cube c = cube(p);
        return c == null || c.cube().localOfPoint(x, z) != null;
    }

    /** The F3 longitude of a simulation longitude, degrees in [−180, 180). */
    public static double displayLon(WorldProjection p, double simLonDeg) {
        double w = ((simLonDeg + 180.0) % 360.0 + 360.0) % 360.0 - 180.0;
        return w >= 180.0 ? w - 360.0 : w;
    }

    /** The simulation longitude, [0, 360), of an F3 longitude. */
    public static double simLon(WorldProjection p, double displayLonDeg) {
        double w = (displayLonDeg % 360.0 + 360.0) % 360.0;
        return w >= 360.0 ? 0.0 : w;
    }

    /** How far, in degrees, the guarded forward lets the inverse of its result miss the asked point. */
    static final double FORWARD_TOLERANCE_DEG = 1e-5;

    /**
     * Deep Time's forward projection {@link WorldProjection#blockOf}, guarded by its inverse
     * {@link WorldProjection#latLon}: a result whose inverse misses the asked point by more than
     * {@link #FORWARD_TOLERANCE_DEG} is logged as an error (once per session, since the map asks every
     * frame) and still returned, so a broken forward shows in the log without crashing the client
     * from a render path; {@link #strictBlockOf} throws instead. (It was written for the
     * Petroff–Guyou forward, which until 2026-10-20 went wrong on exact inputs on its square's
     * diagonals; that world went with cube plan §7.4.) Returns {x, z}, canonical.
     */
    public static double[] blockOf(WorldProjection p, double latDeg, double lonDeg) {
        double[] b = p.blockOf(latDeg, lonDeg);
        String miss = forwardMiss(p, b, latDeg, lonDeg);
        if (miss != null && !FORWARD_MISS_LOGGED.getAndSet(true))
            LOGGER.error("{} (logged once; later misses are not)", miss);
        return b;
    }

    /** {@link #blockOf}, but a forward that misses throws {@link IllegalStateException}. For tests and tools. */
    public static double[] strictBlockOf(WorldProjection p, double latDeg, double lonDeg) {
        double[] b = p.blockOf(latDeg, lonDeg);
        String miss = forwardMiss(p, b, latDeg, lonDeg);
        if (miss != null)
            throw new IllegalStateException(miss);
        return b;
    }

    private static final Logger LOGGER = LoggerFactory.getLogger("deeptime_devmap");
    static final AtomicBoolean FORWARD_MISS_LOGGED = new AtomicBoolean();

    /** The message for a forward result whose inverse misses the asked point, or null when it is within tolerance. */
    private static String forwardMiss(WorldProjection p, double[] b, double latDeg, double lonDeg) {
        double err = forwardError(p, b, latDeg, lonDeg);
        if (err <= FORWARD_TOLERANCE_DEG)
            return null;
        return String.format(Locale.ROOT,
                "Deep Time's forward put (%s°, %s°) at (%s, %s), whose inverse is %s° away", latDeg, lonDeg, b[0], b[1], err);
    }

    /** Degrees between the asked point and the inverse of a block point (latitude, and longitude scaled by cos latitude). */
    static double forwardError(WorldProjection p, double[] b, double latDeg, double lonDeg) {
        double[] ll = p.latLon(b[0], b[1]);
        double dLon = ll[1] - lonDeg;
        dLon -= 360.0 * Math.round(dLon / 360.0);
        return Math.max(Math.abs(ll[0] - latDeg), Math.abs(dLon) * Math.cos(Math.toRadians(latDeg)));
    }

    /** {latitude, F3 longitude} of block (x, z), degrees. */
    public static double[] displayLatLon(WorldProjection p, double x, double z) {
        double[] ll = p.latLon(x, z);
        return new double[]{ll[0], displayLon(p, ll[1])};
    }

    /** {@code 12.345° N 67.890° E}: the F3 line's text for a latitude and an F3 longitude. */
    public static String formatLatLon(double lat, double lon) {
        return String.format(Locale.ROOT, "%.3f° %s %.3f° %s", Math.abs(lat), lat >= 0 ? "N" : "S",
                Math.abs(lon), lon >= 0 ? "E" : "W");
    }

    /**
     * Blocks from (x, z) to the seam, the nearer way round: on a polar world the wrap seam's line
     * x = ±period/2; on a cube the slide's segment, x = −3s/2 (≡ 5s/2) for |z| ≤ s/2.
     */
    public static double seamDistance(WorldProjection p, double x, double z) {
        WorldProjection.Cube c = cube(p);
        if (c == null)
            return p.period() / 2.0 - Math.abs(p.wrapX(x));
        double h = c.side() / 2.0;
        return Math.hypot(wrapCentred(p, x + 3.0 * h), Math.max(0.0, Math.abs(z) - h));
    }

    /**
     * Blocks from (x, z) to the nearer of the world's edges: the z limits of a Lambert-frame level
     * (its poles), the nearest of a cube's twelve
     * border segments ({@link CubeBorder#distanceToSegment}).
     */
    public static double edgeDistance(WorldProjection p, double x, double z) {
        WorldProjection.Cube c = cube(p);
        if (c != null)
            return CubeBorder.of(c.side()).distanceToSegment(x, z);
        double cz = p.canonical(x, z)[1];
        return Math.min(cz - p.zMin(), p.zMax() - cz);
    }

    /** A line in block space: its points' x and z. */
    public record Polyline(double[] xs, double[] zs) {}

    /**
     * The planet's pieces in block space, {x0, z0, x1, z1} each: the whole strip of one period from
     * the seam on a polar world, a cube's six faces in its net window. The map draws these when the
     * server sent no picture.
     */
    public static List<double[]> pieces(WorldProjection p) {
        WorldProjection.Cube c = cube(p);
        if (c == null)
            return List.of(new double[]{-p.period() / 2.0, p.zMin(), p.period() / 2.0, p.zMax()});
        double h = c.side() / 2.0;
        List<double[]> out = new ArrayList<>(6);
        for (CubeLayout.Face f : CubeLayout.Face.all()) {
            double cx = c.cube().faceCentreX(f), cz = c.cube().faceCentreZ(f);
            out.add(new double[]{cx - h, cz - h, cx + h, cz + h});
        }
        return out;
    }

    /** The face of a cube under (x, z) by name ({@code F0} .. {@code S}), {@code void} off the faces; null on a polar world. */
    public static String faceName(WorldProjection p, double x, double z) {
        WorldProjection.Cube c = cube(p);
        if (c == null)
            return null;
        CubeLayout.LocalPoint lp = c.cube().localOfPoint(x, z);
        return lp == null ? "void" : lp.face().name();
    }

    /**
     * The planet's outline on a cube, {@code d} blocks inside the border's segments: the boundary of
     * the region a player may stand in under a border stopping {@code d} inside them
     * ({@link CubeBorder#inside}: the equatorial band, the polar column and F0 whole), two polylines,
     * the north side and the south, across one period of Toroidal World's frame [−2s, 2s] (the map
     * draws them once a period). At d = 0 they run along the twelve segments. Empty on a polar world.
     */
    public static List<Polyline> borderOutline(WorldProjection p, double d) {
        WorldProjection.Cube c = cube(p);
        if (c == null)
            return List.of();
        double h = c.side() / 2.0;
        double a = h - d, b = 3.0 * h - d;
        double[] xs = {-4.0 * h, -h, -h, -a, -a, a, a, h, h, 4.0 * h};
        double[] zn = {-a, -a, -h, -h, -b, -b, -h, -h, -a, -a};
        double[] zs = new double[zn.length];
        for (int i = 0; i < zn.length; i++)
            zs[i] = -zn[i];
        return List.of(new Polyline(xs, zn), new Polyline(xs.clone(), zs));
    }

    /** A cube's slide, F2's east side against F3's west: the segment x = −3s/2, |z| ≤ s/2. Null on a polar world. */
    public static Polyline slide(WorldProjection p) {
        WorldProjection.Cube c = cube(p);
        if (c == null)
            return null;
        double h = c.side() / 2.0;
        return new Polyline(new double[]{-3.0 * h, -3.0 * h}, new double[]{-h, h});
    }

    /**
     * The five sides a cube's net keeps side by side (F3 | F0, F0 | F1, F1 | F2, N over F0, F0 over
     * S), where the faces meet with nothing to join. Empty on a polar world.
     */
    public static List<Polyline> joinedSides(WorldProjection p) {
        WorldProjection.Cube c = cube(p);
        if (c == null)
            return List.of();
        double h = c.side() / 2.0;
        List<Polyline> out = new ArrayList<>(5);
        for (double x : new double[]{-h, h, 3.0 * h})
            out.add(new Polyline(new double[]{x, x}, new double[]{-h, h}));
        for (double z : new double[]{-h, h})
            out.add(new Polyline(new double[]{-h, h}, new double[]{z, z}));
        return out;
    }

    /**
     * A pole and where it lies in block space.
     *
     * @param name   {@code north} or {@code south}
     * @param latDeg +90 or −90
     * @param images its block points in the domain (one on Lambert, where the pole is the whole
     *               line z = {@code images[0][1]}; one inside face N or S on a cube)
     * @param line   true when the pole is a line of constant z (Lambert)
     */
    public record Pole(String name, double latDeg, List<double[]> images, boolean line) {}

    /** The world's two poles. */
    public static List<Pole> poles(WorldProjection p) {
        List<Pole> out = new ArrayList<>(2);
        for (int s = 1; s >= -1; s -= 2) {
            String name = s > 0 ? "north" : "south";
            double lat = 90.0 * s;
            if (cube(p) != null) {
                double[] c = blockOf(p, lat, 0.0);
                out.add(new Pole(name, lat, List.of(new double[]{p.wrapX(c[0]), c[1]}), false));
            } else {
                out.add(new Pole(name, lat, List.of(new double[]{0.0, s > 0 ? p.zMin() : p.zMax()}), true));
            }
        }
        return out;
    }

    /**
     * A cube corner at one of its images in the net window.
     *
     * @param name      {@code n0} .. {@code n3}, {@code s0} .. {@code s3} ({@code CubeLayout.corners})
     * @param x         block x, in the window [−3s/2, 5s/2)
     * @param z         block z
     * @param reachable whether a player can stand on it under the border: the four corners of F0
     */
    public record Corner(String name, double x, double z, boolean reachable) {}

    /**
     * The eight corners of a cube at their twelve images in the window (the closed window's third
     * image of the F2/F3 corners, at x = 5s/2, is the one at x = −3s/2 a period on); none on a polar
     * world.
     */
    public static List<Corner> corners(WorldProjection p) {
        WorldProjection.Cube c = cube(p);
        if (c == null)
            return List.of();
        double x1 = c.cube().window()[1];
        List<Corner> out = new ArrayList<>(12);
        for (CubeLayout.Corner k : c.cube().corners()) {
            boolean reach = Arrays.asList(k.faces()).contains(CubeLayout.Face.F0);
            for (long[] im : k.images())
                if ((double) im[0] < x1)
                    out.add(new Corner(k.name(), (double) im[0], (double) im[1], reach));
        }
        return out;
    }

    /**
     * The cone points: a cube's corners at their twelve images ({@link #corners}), where three
     * faces meet in 270°; none on Lambert.
     */
    public static List<double[]> conePoints(WorldProjection p) {
        List<double[]> out = new ArrayList<>(12);
        for (Corner k : corners(p))
            out.add(new double[]{k.x(), k.z()});
        return out;
    }

    /**
     * The nearest pole of a point.
     *
     * @param byBlocks   the pole whose block-space image is nearest (x wraps)
     * @param blocks     the straight block distance to that image
     * @param byArc      the pole of the point's hemisphere (nearest along the sphere)
     * @param arcDeg     90° less |latitude|
     * @param arcBlocks  that arc on the equal-area sphere ({@link WorldProjection#radius}), blocks
     */
    public record NearestPole(String byBlocks, double blocks, String byArc, double arcDeg, double arcBlocks) {}

    public static NearestPole nearestPole(WorldProjection p, double x, double z) {
        double[] c = p.canonical(x, z);
        // Past a cube's corners there is no planet point.
        if (Double.isNaN(c[0]))
            return new NearestPole("none", Double.NaN, "none", Double.NaN, Double.NaN);
        String best = null;
        double bestD = Double.POSITIVE_INFINITY;
        for (Pole pole : poles(p)) {
            for (double[] im : pole.images()) {
                double d = pole.line() ? Math.abs(c[1] - im[1]) : Math.hypot(wrapCentred(p, c[0] - im[0]), c[1] - im[1]);
                if (d < bestD) {
                    bestD = d;
                    best = pole.name();
                }
            }
        }
        double lat = p.latLon(x, z)[0];
        double arc = 90.0 - Math.abs(lat);
        return new NearestPole(best, bestD, lat >= 0 ? "north" : "south", arc, Math.toRadians(arc) * p.radius());
    }
}
