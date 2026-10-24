// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrap;

import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.ConcurrentHashMap;

/**
 * The geometry of a cube world's temporary border (cube plan W3, docs/design/cube-game.md section 5):
 * pure, so JUnit checks it without a level. {@link EdgeBorder} applies it to players and vehicles.
 *
 * <p>The net (FORMAT.md "The cube world's layout"), h = s/2: the band F3 F0 F1 F2 at |z| ≤ h, all x
 * (Toroidal World's slide joins its ends), and the polar faces N above F0 and S below it, |x| ≤ h,
 * h ≤ |z| ≤ 3h. The border stops at the 12 turning edges (the layout's {@code segments}, owned by the
 * wrap engine), {@code inset} blocks inside each. The allowed region at depth d is
 *
 * <pre>
 *   A(d) = { |z| ≤ h − d }  ∪  { |x| ≤ h − d, |z| ≤ 3h − d }  ∪  F0 = { |x| ≤ h, |z| ≤ h }
 * </pre>
 *
 * with x in Toroidal World's frame [−2s, 2s) (F0 at its centre). F0's square is in it whole: its four
 * edges are direct seams, open, so the corner where F0, F1 and N meet is a point a player may stand
 * on, and the void quadrant beyond it is closed by the two segments that end there. A point not in
 * A(d) is moved to the nearest point of A(d) (the nearest of the three rectangles' clamps), never
 * pushed by two segments in turn.
 */
public final class CubeBorder {

    private static final ConcurrentHashMap<Long, CubeBorder> SHARED = new ConcurrentHashMap<>();

    private final CubeLayout cube;
    private final double h;
    private final double period;
    /** The turning edges' lines: {axis is x ? 1 : 0, coord, spanFrom, spanTo} in the window frame. */
    private final double[][] segments;

    private CubeBorder(CubeLayout cube) {
        this.cube = cube;
        this.h = cube.side() / 2.0;
        this.period = (double) cube.period();
        List<double[]> segs = new ArrayList<>();
        for (CubeLayout.NetEdge e : cube.edges())
            if ("wrap_engine".equals(e.owner()))
                segs.add(new double[]{e.axis() == CubeLayout.Axis.X ? 1 : 0, e.coord(), e.spanFrom(), e.spanTo()});
        this.segments = segs.toArray(new double[0][]);
    }

    /** The border of the net of face side {@code s}, one per side for the process's life. */
    public static CubeBorder of(long s) {
        CubeBorder b = SHARED.get(s);
        return b != null ? b : SHARED.computeIfAbsent(s, k -> new CubeBorder(CubeLayout.shared(k)));
    }

    public CubeLayout cube() {
        return cube;
    }

    /** How many segments the border has (12). */
    public int segmentCount() {
        return segments.length;
    }

    /** x in Toroidal World's frame [−2s, 2s) (period 4s). */
    public double frameX(double x) {
        return x - period * Math.floor((x + period / 2) / period);
    }

    /** Whether (x, z) lies in A(d): on the planet and at least d inside every segment. */
    public boolean inside(double x, double z, double d) {
        double ax = Math.abs(frameX(x)), az = Math.abs(z);
        return az <= h - d || (ax <= h - d && az <= 3 * h - d) || (ax <= h && az <= h);
    }

    /**
     * The nearest point of A(d) to (x, z): {x, z}, x in the same period as the input (the shift into
     * Toroidal World's frame is undone). Unchanged when (x, z) is in A(d).
     */
    public double[] nearest(double x, double z, double d) {
        if (inside(x, z, d))
            return new double[]{x, z};
        double fx = frameX(x), shift = x - fx;
        double[][] c = {
                {fx, clamp(z, h - d)},
                {clamp(fx, h - d), clamp(z, 3 * h - d)},
                {clamp(fx, h), clamp(z, h)}};
        double[] best = c[0];
        double bestD = Double.POSITIVE_INFINITY;
        for (double[] p : c) {
            double dd = (p[0] - fx) * (p[0] - fx) + (p[1] - z) * (p[1] - z);
            if (dd < bestD) {
                bestD = dd;
                best = p;
            }
        }
        return new double[]{best[0] + shift, best[1]};
    }

    /** The distance from (x, z) to the nearest border segment, blocks (x through every period image). */
    public double distanceToSegment(double x, double z) {
        double fx = frameX(x);
        double best = Double.POSITIVE_INFINITY;
        for (double[] s : segments) {
            for (int img = -1; img <= 1; img++) {
                double px = fx + img * period;
                double along = s[0] == 1 ? z : px, across = s[0] == 1 ? px : z;
                double t = Math.max(0.0, Math.max(s[2] - along, along - s[3]));
                double n = across - s[1];
                best = Math.min(best, Math.hypot(t, n));
            }
        }
        return best;
    }

    /**
     * Whether block (x, z) is fit to spawn on (cube plan W5): on the planet, at least {@code edgeMargin}
     * from every border segment and {@code cornerMargin} from every image of a cube corner in the net
     * (x through every period image). {@code SpawnHandler.cubeSpawnOk} passes its margins.
     */
    public boolean spawnOk(int x, int z, double edgeMargin, double cornerMargin) {
        double px = x + 0.5, pz = z + 0.5;
        if (!inside(px, pz, edgeMargin) || distanceToSegment(px, pz) < edgeMargin)
            return false;
        double fx = frameX(px);
        for (var corner : cube.corners())
            for (long[] img : corner.images()) {
                double dx = fx - frameX(img[0]);
                dx -= period * Math.rint(dx / period);
                if (Math.hypot(dx, pz - img[1]) < cornerMargin)
                    return false;
            }
        return true;
    }

    /**
     * The border zone of (x, z) for a stop line {@code inset} inside the segments and a warning band
     * {@code warn} blocks wide inside it: 0 open, 1 in the warning band, 2 past the stop line (or off
     * the planet).
     */
    public int zone(double x, double z, double inset, double warn) {
        if (!inside(x, z, inset))
            return 2;
        return distanceToSegment(x, z) < inset + warn ? 1 : 0;
    }

    /** How far (x, z) lies outside the planet (A(0)), blocks; 0 on it. */
    public double depthOutside(double x, double z) {
        double[] p = nearest(x, z, 0.0);
        return Math.hypot(p[0] - x, p[1] - z);
    }

    /**
     * The way out of A(d) for a footprint (the Sable border, cube plan W3): {@code xz} holds points
     * {x0, z0, x1, z1, ...} (a hull's bounds' four corners). Returns {nx, nz, depth}: n the unit
     * vector out of the planet, the sum over the points outside A(d) of each one's offset from its
     * nearest point of A(d), normalised (the deepest point's when the sum vanishes), and depth the
     * largest such offset, blocks. Null when every point is in A(d).
     */
    public double[] outward(double[] xz, double d) {
        double sx = 0, sz = 0, depth = 0, dx = 0, dz = 0;
        for (int i = 0; i + 1 < xz.length; i += 2) {
            double x = xz[i], z = xz[i + 1];
            if (inside(x, z, d))
                continue;
            double[] q = nearest(x, z, d);
            double ox = x - q[0], oz = z - q[1], len = Math.hypot(ox, oz);
            sx += ox;
            sz += oz;
            if (len > depth) {
                depth = len;
                dx = ox;
                dz = oz;
            }
        }
        if (!(depth > 0))
            return null;
        double len = Math.hypot(sx, sz);
        if (len < 1e-9 * depth) {
            sx = dx;
            sz = dz;
            len = depth;
        }
        return new double[]{sx / len, sz / len, depth};
    }

    /**
     * The velocity change that holds a hull at the border: {dvx, dvz} taking away the part of (vx, vz)
     * along {@code out} (from {@link #outward}) when it points out, and adding {@code push} blocks/s
     * back in. Velocity only, never a teleport, as at the pole cap's hard stop.
     */
    public static double[] stopVelocity(double vx, double vz, double[] out, double push) {
        double vn = vx * out[0] + vz * out[1];
        double dvx = -push * out[0], dvz = -push * out[1];
        if (vn > 0) {
            dvx -= vn * out[0];
            dvz -= vn * out[1];
        }
        return new double[]{dvx, dvz};
    }

    /**
     * The shortest walk from (x0, z0) to (x1, z1) on the planet under the border, blocks (cube plan
     * G11): the shortest path inside A(0), the band and the polar column, which the border's segments
     * bound, x through every period image. A(0) is a cross whose four inner corners (±h, ±h) are its
     * only reflex corners, so the path is straight or bends only there: the shortest path in the
     * graph of the two points and those corners (in five period images), joined where the segment
     * between them lies in A(0). NaN when either point is off the planet.
     */
    public double walkDistance(double x0, double z0, double x1, double z1) {
        if (!inside(x0, z0, 0.0) || !inside(x1, z1, 0.0))
            return Double.NaN;
        double sx = frameX(x0), tx = frameX(x1);
        List<double[]> nodes = new ArrayList<>();
        nodes.add(new double[]{sx, z0});
        for (int k = -2; k <= 2; k++)
            for (int sxn = -1; sxn <= 1; sxn += 2)
                for (int szn = -1; szn <= 1; szn += 2)
                    nodes.add(new double[]{k * period + sxn * h, szn * h});
        int firstTarget = nodes.size();
        for (int k = -1; k <= 1; k++)
            nodes.add(new double[]{tx + k * period, z1});
        int n = nodes.size();
        double[] dist = new double[n];
        boolean[] done = new boolean[n];
        java.util.Arrays.fill(dist, Double.POSITIVE_INFINITY);
        dist[0] = 0.0;
        for (int round = 0; round < n; round++) {
            int u = -1;
            for (int i = 0; i < n; i++)
                if (!done[i] && (u < 0 || dist[i] < dist[u]))
                    u = i;
            if (u < 0 || dist[u] == Double.POSITIVE_INFINITY)
                break;
            done[u] = true;
            if (u >= firstTarget)
                continue;   // a target is an end, never a corner to bend round
            double[] a = nodes.get(u);
            for (int v = 1; v < n; v++) {
                if (done[v])
                    continue;
                double[] b = nodes.get(v);
                double len = Math.hypot(b[0] - a[0], b[1] - a[1]);
                if (dist[u] + len < dist[v] && segmentOnPlanet(a[0], a[1], b[0], b[1]))
                    dist[v] = dist[u] + len;
            }
        }
        double best = Double.POSITIVE_INFINITY;
        for (int i = firstTarget; i < n; i++)
            best = Math.min(best, dist[i]);
        return best;
    }

    /**
     * Whether the segment from (ax, az) to (bx, bz) (x in any period) lies in A(0): the parts of it in
     * the band and in each period image of the polar column are intervals of its parameter, and they
     * must cover [0, 1].
     */
    private boolean segmentOnPlanet(double ax, double az, double bx, double bz) {
        List<double[]> parts = new ArrayList<>();
        double[] band = clip(ax, az, bx, bz, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY, -h, h);
        if (band != null)
            parts.add(band);
        double lo = Math.min(ax, bx), hi = Math.max(ax, bx);
        for (long k = (long) Math.floor((lo - h) / period); k <= (long) Math.ceil((hi + h) / period); k++) {
            double[] col = clip(ax, az, bx, bz, k * period - h, k * period + h, -3 * h, 3 * h);
            if (col != null)
                parts.add(col);
        }
        parts.sort((p, q) -> Double.compare(p[0], q[0]));
        double reach = 0.0, eps = 1e-9;
        for (double[] p : parts) {
            if (p[0] > reach + eps)
                return false;
            reach = Math.max(reach, p[1]);
        }
        return reach >= 1.0 - eps;
    }

    /** The parameter interval {t0, t1} of the segment inside the box (Liang–Barsky, edges included, 1e-9 slack), or null. */
    private static double[] clip(double ax, double az, double bx, double bz, double x0, double x1, double z0, double z1) {
        double t0 = 0.0, t1 = 1.0, eps = 1e-9;
        double[] p = {-(bx - ax), bx - ax, -(bz - az), bz - az};
        double[] q = {ax - (x0 - eps), (x1 + eps) - ax, az - (z0 - eps), (z1 + eps) - az};
        for (int i = 0; i < 4; i++) {
            if (Double.isInfinite(q[i]))
                continue;
            if (p[i] == 0.0) {
                if (q[i] < 0.0)
                    return null;
                continue;
            }
            double r = q[i] / p[i];
            if (p[i] < 0.0)
                t0 = Math.max(t0, r);
            else
                t1 = Math.min(t1, r);
            if (t0 > t1)
                return null;
        }
        return new double[]{t0, t1};
    }

    /** The yaw of a heading turned by a net motion (yaw 0 faces +z, 90 faces −x), in [−180, 180). */
    public static float turnYaw(float yaw, CubeLayout.Iso iso) {
        double r = Math.toRadians(yaw);
        double[] v = iso.vector(-Math.sin(r), Math.cos(r));
        double y = Math.toDegrees(Math.atan2(-v[0], v[1]));
        y -= 360.0 * Math.floor((y + 180.0) / 360.0);
        return (float) y;
    }

    private static double clamp(double v, double limit) {
        return v > limit ? limit : (v < -limit ? -limit : v);
    }
}
