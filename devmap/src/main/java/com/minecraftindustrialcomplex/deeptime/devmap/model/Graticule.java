// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.model;

import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;

import java.util.ArrayList;
import java.util.List;

/**
 * The latitude/longitude graticule in block space, swept through Deep Time's own
 * {@link WorldProjection#blockOf}: each meridian and parallel is sampled every {@code sampleDeg} and
 * cut into polylines wherever the projection jumps (a Petroff–Guyou line crossing a turning edge
 * comes back in at the mirrored point). Across the x wrap a line is not cut: its points are
 * unwrapped (x may run past ±period/2), and the map draws every polyline once per period.
 *
 * <p>On a cube a line is cut exactly where it crosses a turning seam: where two consecutive samples
 * lie on faces the net does not keep side by side ({@link #netJoined}; N or S against F1, F2 or
 * F3). Within a face, across the five sides the net joins and across the slide (F2 against F3,
 * unwrapped like the x wrap) it runs on.
 */
public final class Graticule {

    /** What a line is. */
    public enum Kind { MERIDIAN, PARALLEL }

    /**
     * One unbroken piece of a graticule line, block coordinates.
     *
     * @param kind  meridian or parallel
     * @param value its F3 longitude or its latitude, degrees
     * @param xs    block x of each point (unwrapped: consecutive points differ by less than period/2)
     * @param zs    block z of each point
     */
    public record Line(Kind kind, double value, double[] xs, double[] zs) {
        public int size() {
            return xs.length;
        }
    }

    private Graticule() {}

    /**
     * Every meridian and parallel at multiples of {@code stepDeg} (the parallels short of the poles),
     * sampled every {@code sampleDeg}.
     */
    public static List<Line> lines(WorldProjection p, double stepDeg, double sampleDeg) {
        List<Line> out = new ArrayList<>();
        int nm = (int) Math.round(360.0 / stepDeg);
        for (int k = 0; k < nm; k++) {
            double d = -180.0 + k * stepDeg;
            double sim = PlanetGeometry.simLon(p, d);
            // Short of the poles, where every meridian meets.
            out.addAll(sweep(p, Kind.MERIDIAN, d, -89.9, 89.9, sampleDeg, t -> new double[]{t, sim}));
        }
        for (double lat = -90.0 + stepDeg; lat < 90.0 - 1e-9; lat += stepDeg) {
            final double la = lat;
            out.addAll(sweep(p, Kind.PARALLEL, lat, 0.0, 360.0, sampleDeg, t -> new double[]{la, t}));
        }
        return out;
    }

    /** One line of constant latitude or longitude, cut where the projection jumps. */
    public static List<Line> sweep(WorldProjection p, Kind kind, double value, double from, double to, double sampleDeg,
                                   java.util.function.DoubleFunction<double[]> latLonAt) {
        int n = Math.max(2, (int) Math.ceil((to - from) / sampleDeg) + 1);
        List<Line> out = new ArrayList<>();
        List<double[]> cur = new ArrayList<>();
        double[] prevLl = null;
        double[] prev = null;
        for (int i = 0; i < n; i++) {
            double t = from + (to - from) * i / (n - 1);
            double[] ll = latLonAt.apply(t);
            double[] q = PlanetGeometry.blockOf(p, ll[0], ll[1]);
            if (prev == null) {
                cur.add(q);
            } else {
                double[] cand = {prev[0] + PlanetGeometry.wrapCentred(p, q[0] - prev[0]), q[1]};
                if (cuts(p, prev, cand, prevLl, ll)) {
                    flush(out, kind, value, cur);
                    cur = new ArrayList<>();
                    cand = q;
                }
                cur.add(cand);
            }
            prev = cur.get(cur.size() - 1);
            prevLl = ll;
        }
        flush(out, kind, value, cur);
        return out;
    }

    /**
     * Whether the line is cut between two consecutive samples: on a cube where they lie on faces the
     * net does not join (a sample on no face, past rounding, falls back to {@link #jumps}); else
     * where the projection jumps.
     */
    static boolean cuts(WorldProjection p, double[] a, double[] b, double[] llA, double[] llB) {
        WorldProjection.Cube c = PlanetGeometry.cube(p);
        if (c == null)
            return jumps(p, a, b, llA, llB);
        CubeLayout.LocalPoint fa = c.cube().localOfPoint(a[0], a[1]), fb = c.cube().localOfPoint(b[0], b[1]);
        if (fa == null || fb == null)
            return jumps(p, a, b, llA, llB);
        return !netJoined(fa.face(), fb.face());
    }

    /**
     * Whether two faces of a cube meet side by side in the net, or across the slide, or are the same
     * face: F3 | F0 | F1 | F2 | F3 round the equator, N and S on F0. The other six pairs that share a
     * cube edge (N or S against F1, F2, F3) meet only through a turning seam.
     */
    static boolean netJoined(CubeLayout.Face a, CubeLayout.Face b) {
        if (a == b)
            return true;
        boolean polarA = a == CubeLayout.Face.N || a == CubeLayout.Face.S;
        boolean polarB = b == CubeLayout.Face.N || b == CubeLayout.Face.S;
        if (polarA || polarB)
            return (polarA ? b : a) == CubeLayout.Face.F0 && !(polarA && polarB);
        // Two equatorial faces: neighbours round the ring F3 F0 F1 F2 (all four pairs but F0 F2, F1 F3).
        return ring(a) % 2 != ring(b) % 2;
    }

    private static int ring(CubeLayout.Face f) {
        return switch (f) {
            case F3 -> 0;
            case F0 -> 1;
            case F1 -> 2;
            default -> 3;
        };
    }

    /**
     * Whether the projection jumps between two consecutive samples: the point halfway along the line
     * (in latitude and longitude) is not near the middle of the chord. On a continuous stretch the
     * midpoint sits on the chord to well under a quarter of its length; across a jump it lands near
     * one end.
     */
    static boolean jumps(WorldProjection p, double[] a, double[] b, double[] llA, double[] llB) {
        double dLon = llB[1] - llA[1];
        dLon -= 360.0 * Math.round(dLon / 360.0);
        double[] m = PlanetGeometry.blockOf(p, 0.5 * (llA[0] + llB[0]), llA[1] + 0.5 * dLon);
        double mx = 0.5 * (a[0] + b[0]), mz = 0.5 * (a[1] + b[1]);
        double off = Math.hypot(PlanetGeometry.wrapCentred(p, m[0] - mx), m[1] - mz);
        double chord = Math.hypot(b[0] - a[0], b[1] - a[1]);
        return off > 0.25 * chord + 2.0;
    }

    private static void flush(List<Line> out, Kind kind, double value, List<double[]> pts) {
        if (pts.size() < 2)
            return;
        double[] xs = new double[pts.size()], zs = new double[pts.size()];
        for (int i = 0; i < xs.length; i++) {
            xs[i] = pts.get(i)[0];
            zs[i] = pts.get(i)[1];
        }
        out.add(new Line(kind, value, xs, zs));
    }
}
