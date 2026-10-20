// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.model;

import com.minecraftindustrialcomplex.deeptime.worlddata.Projection;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * What the map marks and the readout says about a point, from Deep Time's own
 * {@link WorldProjection} (no projection formulas here): latitude and longitude as the F3 line shows
 * them ({@code net.PlanetDebugLine}), the distance to the seam and the edges, and the poles.
 *
 * <p>The longitude convention follows the F3 line: on a Petroff–Guyou world the simulation's
 * longitude folded to [−180°, 180°); on a Lambert world degrees east of x = 0
 * ({@link Projection#longitudeEastDeg}), which is the simulation's longitude less 180°.
 */
public final class PlanetGeometry {

    private PlanetGeometry() {}

    /** Whether the world is the seamless Petroff–Guyou world. */
    public static boolean petroffGuyou(WorldProjection p) {
        return p.kind() == WorldProjection.Kind.PETROFF_GUYOU;
    }

    /** The F3 longitude of a simulation longitude, degrees in [−180, 180). */
    public static double displayLon(WorldProjection p, double simLonDeg) {
        double l = petroffGuyou(p) ? simLonDeg : simLonDeg - 180.0;
        double w = ((l + 180.0) % 360.0 + 360.0) % 360.0 - 180.0;
        return w >= 180.0 ? w - 360.0 : w;
    }

    /** The simulation longitude, [0, 360), of an F3 longitude. */
    public static double simLon(WorldProjection p, double displayLonDeg) {
        double l = petroffGuyou(p) ? displayLonDeg : displayLonDeg + 180.0;
        double w = (l % 360.0 + 360.0) % 360.0;
        return w >= 360.0 ? 0.0 : w;
    }

    /** How far, in degrees, the guarded forward lets the inverse of its result miss the asked point. */
    static final double FORWARD_TOLERANCE_DEG = 1e-5;

    /**
     * Deep Time's forward projection {@link WorldProjection#blockOf}, guarded by its inverse
     * {@link WorldProjection#latLon}: a result whose inverse misses the asked point by more than
     * {@link #FORWARD_TOLERANCE_DEG} is logged as an error (once per session, since the map asks every
     * frame) and still returned, so a broken forward shows in the log without crashing the client
     * from a render path; {@link #strictBlockOf} throws instead. Until 2026-10-20 Deep Time's Petroff forward went wrong on exact
     * inputs on the square's diagonals (the world's equator and the meridians λc and λc + 180°, up to
     * 75 blocks off on the play planet) and at the cone points ((45° N, 236°) came back 616 blocks
     * along the edge), and this method retried from nudged inputs. The forward now places those
     * points by coordinates ({@code deeptime_format::petroff}, ledger SCL-083); its worst miss on
     * exact diagonal inputs is about 4e-7°, the inverse's own rounding. Returns {x, z}, canonical.
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
        if (petroffGuyou(p))
            return new double[]{ll[0], displayLon(p, ll[1])};
        // The F3 line's own formula on Lambert (PoleCapCurve.longitudeDeg is longitudeEastDeg).
        return new double[]{ll[0], Projection.longitudeEastDeg(p.period(), x)};
    }

    /** {@code 12.345° N 67.890° E}: the F3 line's text for a latitude and an F3 longitude. */
    public static String formatLatLon(double lat, double lon) {
        return String.format(Locale.ROOT, "%.3f° %s %.3f° %s", Math.abs(lat), lat >= 0 ? "N" : "S",
                Math.abs(lon), lon >= 0 ? "E" : "W");
    }

    /** Blocks from x to the wrap seam (x = ±period/2), the nearer way round. */
    public static double seamDistance(WorldProjection p, double x) {
        return p.period() / 2.0 - Math.abs(p.wrapX(x));
    }

    /** Blocks from z to the nearer of the world's z limits (the edges of a Petroff–Guyou world, the poles of a Lambert one). */
    public static double edgeDistance(WorldProjection p, double x, double z) {
        double cz = p.canonical(x, z)[1];
        return Math.min(cz - p.zMin(), p.zMax() - cz);
    }

    /**
     * A pole and where it lies in block space.
     *
     * @param name   {@code north} or {@code south}
     * @param latDeg +90 or −90
     * @param images its block points in the domain (two on a Petroff–Guyou world, the same point
     *               seen from both sides of its edge; one on Lambert, where the pole is the whole line
     *               z = {@code images[0][1]})
     * @param line   true when the pole is a line of constant z (Lambert)
     */
    public record Pole(String name, double latDeg, List<double[]> images, boolean line) {}

    /** The world's two poles. */
    public static List<Pole> poles(WorldProjection p) {
        List<Pole> out = new ArrayList<>(2);
        for (int s = 1; s >= -1; s -= 2) {
            String name = s > 0 ? "north" : "south";
            double lat = 90.0 * s;
            double[] c = blockOf(p, lat, 0.0);
            List<double[]> images = new ArrayList<>(2);
            if (petroffGuyou(p)) {
                // On an edge: the half-turn about the edge's midpoint (x, z) -> (−x, z) glues it to its mirror.
                images.add(new double[]{p.wrapX(c[0]), c[1]});
                double mx = p.wrapX(-c[0]);
                if (Math.abs(mx - p.wrapX(c[0])) > 1e-6)
                    images.add(new double[]{mx, c[1]});
                images.sort((a, b) -> Double.compare(a[0], b[0]));
                out.add(new Pole(name, lat, List.copyOf(images), false));
            } else {
                out.add(new Pole(name, lat, List.of(new double[]{0.0, s > 0 ? p.zMin() : p.zMax()}), true));
            }
        }
        return out;
    }

    /**
     * The cone points of a Petroff–Guyou world, where the gluing of its edges turns the plane
     * through 180° (the edges' midpoints and corners: (0, ±b/2), (±b, ±b/2)); none on Lambert.
     */
    public static List<double[]> conePoints(WorldProjection p) {
        if (!petroffGuyou(p))
            return List.of();
        double half = p.period() / 2.0;
        return List.of(new double[]{0.0, p.zMin()}, new double[]{-half, p.zMin()}, new double[]{half, p.zMin()},
                new double[]{0.0, p.zMax()}, new double[]{-half, p.zMax()}, new double[]{half, p.zMax()});
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
        String best = null;
        double bestD = Double.POSITIVE_INFINITY;
        for (Pole pole : poles(p)) {
            for (double[] im : pole.images()) {
                double d = pole.line() ? Math.abs(c[1] - im[1]) : Math.hypot(p.wrapX(c[0] - im[0]), c[1] - im[1]);
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
