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
